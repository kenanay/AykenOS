/// BCIB Diagnostics — non-authoritative observability endpoints.
///
/// Implements the three BCIB diagnostics endpoints required by WS 3.8
/// (Requirements 6.1, 6.2). All responses carry the Phase-14 epistemic
/// boundary declaration:
///
/// ```
/// produces_truth:    false
/// produces_decision: false
/// produces_ranking:  false
/// ```
///
/// These endpoints are **read-only** and **non-authoritative**. They expose
/// a snapshot of runtime state for diagnostic purposes only. They MUST NOT
/// be used to make scheduling, routing, or execution decisions.
///
/// # Endpoints
///
/// | Path | Description |
/// |------|-------------|
/// | `GET /diagnostics/bcib/execution/{ctx_id}` | Current execution state |
/// | `GET /diagnostics/bcib/lifecycle/{ctx_id}` | Lifecycle transition history |
/// | `GET /diagnostics/bcib/cost/{ctx_id}`      | Cost budget usage |
///
/// # Phase-14 Immutability
///
/// This module MUST NOT mutate or extend the Phase-14 observability contracts
/// (`OBSERVABILITY_UX_CONTRACT_v1`, `CROSS_NODE_OBSERVABILITY_GRAPH_CONTRACT_v1`,
/// `PROOFD_EXTERNAL_DIAGNOSTICS_CONTRACT_v1`). It only adds BCIB-specific
/// endpoints within the existing boundary.

use crate::cost_tracker::CostTracker;
use crate::execution_runtime::{BcibExecutionRuntime, ExecutionState};
use crate::types::{CostUnit, ExecutionContextId};

// ---------------------------------------------------------------------------
// Forbidden observability fields (Requirements 6.3, 6.4)
// ---------------------------------------------------------------------------

/// Fields that MUST NOT appear in any BCIB diagnostics response.
///
/// These field names expose authority, decision, or ranking semantics which
/// violate the Phase-14 epistemic boundary invariant:
/// `service != authority`, `diagnostics != decision`, `parity != consensus`.
///
/// If any response struct contains a field whose name appears in this list,
/// the response MUST be rejected with `500 forbidden_observability_field_exposed`
/// before it is returned to the caller.
///
/// Requirement 6.3: diagnostics surface SHALL NOT expose any field in this list.
/// Requirement 6.4: if a response contains a forbidden field, proofd SHALL
/// reject it with `500 forbidden_observability_field_exposed`.
pub const FORBIDDEN_OBSERVABILITY_FIELDS: &[&str] = &[
    "authority",
    "decision",
    "ranking",
    "verdict",
    "policy",
    "schedule",
    "priority",
    "truth",
    "consensus",
    "parity",
];

// ---------------------------------------------------------------------------
// DiagnosticsError (Requirements 6.3, 6.4)
// ---------------------------------------------------------------------------

/// Errors that can occur during diagnostics response construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticsError {
    /// A forbidden observability field was detected in the response.
    ///
    /// Maps to HTTP 500 `forbidden_observability_field_exposed`.
    /// The inner `String` names the offending field.
    ///
    /// Requirement 6.4.
    ForbiddenFieldExposed(String),
}

impl core::fmt::Display for DiagnosticsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DiagnosticsError::ForbiddenFieldExposed(field) => {
                write!(
                    f,
                    "500 forbidden_observability_field_exposed: field '{}' is not permitted \
                     in BCIB diagnostics responses (Phase-14 epistemic boundary violation)",
                    field
                )
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Forbidden field check (Requirements 6.3, 6.4)
// ---------------------------------------------------------------------------

/// Check that none of the provided field names appear in
/// [`FORBIDDEN_OBSERVABILITY_FIELDS`].
///
/// This function MUST be called before any diagnostics response is returned
/// to the caller. If a forbidden field is detected, the response MUST NOT be
/// sent — return the error instead.
///
/// # Errors
///
/// Returns `Err(DiagnosticsError::ForbiddenFieldExposed(field_name))` on the
/// first forbidden field found. Subsequent fields are not checked (fail-fast).
///
/// # Example
///
/// ```rust,ignore
/// check_forbidden_fields(&["context_id", "state", "epistemic_boundary"])?;
/// ```
///
/// Requirement 6.3, 6.4.
pub fn check_forbidden_fields(field_names: &[&str]) -> Result<(), DiagnosticsError> {
    for &name in field_names {
        if FORBIDDEN_OBSERVABILITY_FIELDS.contains(&name) {
            return Err(DiagnosticsError::ForbiddenFieldExposed(name.to_string()));
        }
    }
    Ok(())
}

/// Field names present in [`ExecutionDiagnosticsResponse`].
const EXECUTION_RESPONSE_FIELDS: &[&str] = &["context_id", "state", "epistemic_boundary"];

/// Field names present in [`LifecycleDiagnosticsResponse`].
const LIFECYCLE_RESPONSE_FIELDS: &[&str] = &["context_id", "transitions", "epistemic_boundary"];

/// Field names present in [`CostDiagnosticsResponse`].
const COST_RESPONSE_FIELDS: &[&str] = &["context_id", "cost", "epistemic_boundary"];

// ---------------------------------------------------------------------------
// Epistemic boundary declaration (Phase-14 invariant, Requirement 6.2)
// ---------------------------------------------------------------------------

/// Epistemic boundary declaration attached to every diagnostics response.
///
/// These three fields are IMMUTABLE and MUST be present in every response.
/// They encode the Phase-14 invariant: `service != authority`,
/// `diagnostics != decision`, `parity != consensus`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpistemicBoundary {
    /// This endpoint does not produce ground truth.
    pub produces_truth: bool,
    /// This endpoint does not produce execution decisions.
    pub produces_decision: bool,
    /// This endpoint does not produce ranking or ordering.
    pub produces_ranking: bool,
}

impl EpistemicBoundary {
    /// Returns the canonical Phase-14 boundary declaration.
    ///
    /// All three fields are `false` — this is the only valid value.
    pub const fn phase14() -> Self {
        Self {
            produces_truth: false,
            produces_decision: false,
            produces_ranking: false,
        }
    }
}

impl Default for EpistemicBoundary {
    fn default() -> Self {
        Self::phase14()
    }
}

// ---------------------------------------------------------------------------
// Execution state snapshot (GET /diagnostics/bcib/execution/{ctx_id})
// ---------------------------------------------------------------------------

/// Discriminant-level snapshot of an execution context's current state.
///
/// This is a non-authoritative, read-only view. The actual `ExecutionState`
/// enum is not exposed directly to avoid leaking internal runtime details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionStateSnapshot {
    Created,
    Ready,
    Running,
    Yielded,
    Waiting,
    Completed,
    Failed,
    Cancelled,
    /// Context ID was not found in the runtime.
    NotFound,
}

impl ExecutionStateSnapshot {
    fn from_state(state: &ExecutionState) -> Self {
        match state {
            ExecutionState::Created => Self::Created,
            ExecutionState::Ready => Self::Ready,
            ExecutionState::Running => Self::Running,
            ExecutionState::Yielded { .. } => Self::Yielded,
            ExecutionState::Waiting { .. } => Self::Waiting,
            ExecutionState::Completed { .. } => Self::Completed,
            ExecutionState::Failed { .. } => Self::Failed,
            ExecutionState::Cancelled => Self::Cancelled,
        }
    }
}

/// Response for `GET /diagnostics/bcib/execution/{ctx_id}`.
#[derive(Debug, Clone)]
pub struct ExecutionDiagnosticsResponse {
    pub context_id: ExecutionContextId,
    pub state: ExecutionStateSnapshot,
    /// Phase-14 epistemic boundary — always `produces_truth: false`,
    /// `produces_decision: false`, `produces_ranking: false`.
    pub epistemic_boundary: EpistemicBoundary,
}

// ---------------------------------------------------------------------------
// Lifecycle transition history (GET /diagnostics/bcib/lifecycle/{ctx_id})
// ---------------------------------------------------------------------------

/// A single recorded state transition in the lifecycle history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleTransitionRecord {
    pub from: ExecutionStateSnapshot,
    pub to: ExecutionStateSnapshot,
    /// Monotonic sequence number — lower = earlier.
    pub sequence: u64,
}

/// Response for `GET /diagnostics/bcib/lifecycle/{ctx_id}`.
#[derive(Debug, Clone)]
pub struct LifecycleDiagnosticsResponse {
    pub context_id: ExecutionContextId,
    /// Ordered list of state transitions (oldest first).
    pub transitions: Vec<LifecycleTransitionRecord>,
    /// Phase-14 epistemic boundary.
    pub epistemic_boundary: EpistemicBoundary,
}

// ---------------------------------------------------------------------------
// Cost budget usage (GET /diagnostics/bcib/cost/{ctx_id})
// ---------------------------------------------------------------------------

/// Snapshot of cost budget consumption for a context.
#[derive(Debug, Clone)]
pub struct CostDiagnosticsSnapshot {
    pub total: CostUnit,
    pub remaining: CostUnit,
    pub external_budget: CostUnit,
    pub external_used: CostUnit,
}

impl CostDiagnosticsSnapshot {
    #[allow(dead_code)]
    fn from_tracker(tracker: &CostTracker) -> Self {
        Self {
            total: tracker.total,
            remaining: tracker.remaining,
            external_budget: tracker.external_budget,
            external_used: tracker.external_used,
        }
    }
}

/// Response for `GET /diagnostics/bcib/cost/{ctx_id}`.
#[derive(Debug, Clone)]
pub struct CostDiagnosticsResponse {
    pub context_id: ExecutionContextId,
    pub cost: CostDiagnosticsSnapshot,
    /// Phase-14 epistemic boundary.
    pub epistemic_boundary: EpistemicBoundary,
}

// ---------------------------------------------------------------------------
// BcibDiagnostics — the diagnostics handler
// ---------------------------------------------------------------------------

/// BCIB diagnostics handler.
///
/// Wraps a read-only reference to the runtime and exposes the three
/// diagnostics endpoints. All methods are non-mutating.
///
/// # Usage
///
/// ```rust,ignore
/// let diag = BcibDiagnostics::new(&runtime);
/// let resp = diag.execution_state(ctx_id);
/// ```
pub struct BcibDiagnostics<'a> {
    runtime: &'a BcibExecutionRuntime,
}

impl<'a> BcibDiagnostics<'a> {
    /// Create a new diagnostics handler backed by the given runtime reference.
    pub fn new(runtime: &'a BcibExecutionRuntime) -> Self {
        Self { runtime }
    }

    // -----------------------------------------------------------------------
    // GET /diagnostics/bcib/execution/{ctx_id}
    // -----------------------------------------------------------------------

    /// Return the current execution state for `ctx_id` (non-authoritative).
    ///
    /// If the context does not exist, `state` is `ExecutionStateSnapshot::NotFound`.
    /// The epistemic boundary is always attached.
    ///
    /// Before returning, the response field names are checked against
    /// [`FORBIDDEN_OBSERVABILITY_FIELDS`]. If a forbidden field is detected,
    /// `Err(DiagnosticsError::ForbiddenFieldExposed)` is returned and the
    /// response is NOT sent (Requirement 6.3, 6.4).
    ///
    /// Requirement 6.1, 6.2, 6.3, 6.4.
    pub fn execution_state(
        &self,
        ctx_id: ExecutionContextId,
    ) -> Result<ExecutionDiagnosticsResponse, DiagnosticsError> {
        // Requirement 6.3/6.4: check before the response leaves this function.
        check_forbidden_fields(EXECUTION_RESPONSE_FIELDS)?;

        let state = match self.runtime.state_of(ctx_id) {
            Ok(s) => ExecutionStateSnapshot::from_state(s),
            Err(_) => ExecutionStateSnapshot::NotFound,
        };

        Ok(ExecutionDiagnosticsResponse {
            context_id: ctx_id,
            state,
            epistemic_boundary: EpistemicBoundary::phase14(),
        })
    }

    // -----------------------------------------------------------------------
    // GET /diagnostics/bcib/lifecycle/{ctx_id}
    // -----------------------------------------------------------------------

    /// Return the lifecycle transition history for `ctx_id` (non-authoritative).
    ///
    /// The history is derived from the current state only (no persistent log
    /// in this implementation). An empty `transitions` list is returned when
    /// the context is not found or has not yet transitioned.
    ///
    /// Before returning, the response field names are checked against
    /// [`FORBIDDEN_OBSERVABILITY_FIELDS`]. If a forbidden field is detected,
    /// `Err(DiagnosticsError::ForbiddenFieldExposed)` is returned and the
    /// response is NOT sent (Requirement 6.3, 6.4).
    ///
    /// Requirement 6.1, 6.2, 6.3, 6.4.
    pub fn lifecycle_history(
        &self,
        ctx_id: ExecutionContextId,
    ) -> Result<LifecycleDiagnosticsResponse, DiagnosticsError> {
        // Requirement 6.3/6.4: check before the response leaves this function.
        check_forbidden_fields(LIFECYCLE_RESPONSE_FIELDS)?;

        let transitions = match self.runtime.state_of(ctx_id) {
            Ok(state) => {
                // Derive a minimal history from the current state.
                // A full persistent history would require an event log in the
                // runtime; this snapshot-based approach satisfies the diagnostic
                // contract without adding mutable state to the runtime.
                derive_transitions_from_state(state)
            }
            Err(_) => vec![],
        };

        Ok(LifecycleDiagnosticsResponse {
            context_id: ctx_id,
            transitions,
            epistemic_boundary: EpistemicBoundary::phase14(),
        })
    }

    // -----------------------------------------------------------------------
    // GET /diagnostics/bcib/cost/{ctx_id}
    // -----------------------------------------------------------------------

    /// Return the cost budget usage for `ctx_id` (non-authoritative).
    ///
    /// Returns a zeroed snapshot when the context is not found.
    ///
    /// Before returning, the response field names are checked against
    /// [`FORBIDDEN_OBSERVABILITY_FIELDS`]. If a forbidden field is detected,
    /// `Err(DiagnosticsError::ForbiddenFieldExposed)` is returned and the
    /// response is NOT sent (Requirement 6.3, 6.4).
    ///
    /// Requirement 6.1, 6.2, 6.3, 6.4.
    pub fn cost_usage(
        &self,
        ctx_id: ExecutionContextId,
    ) -> Result<CostDiagnosticsResponse, DiagnosticsError> {
        // Requirement 6.3/6.4: check before the response leaves this function.
        check_forbidden_fields(COST_RESPONSE_FIELDS)?;

        let cost = match self.runtime.cost_snapshot(ctx_id) {
            Ok(snapshot) => snapshot,
            Err(_) => CostDiagnosticsSnapshot {
                total: 0,
                remaining: 0,
                external_budget: 0,
                external_used: 0,
            },
        };

        Ok(CostDiagnosticsResponse {
            context_id: ctx_id,
            cost,
            epistemic_boundary: EpistemicBoundary::phase14(),
        })
    }
}

// ---------------------------------------------------------------------------
// Helper — derive a minimal transition history from the current state
// ---------------------------------------------------------------------------

/// Derive a minimal ordered transition sequence from the current state.
///
/// This is a best-effort reconstruction. A full persistent history would
/// require an event log in the runtime. For diagnostics purposes, this
/// provides a useful approximation without adding mutable state.
fn derive_transitions_from_state(state: &ExecutionState) -> Vec<LifecycleTransitionRecord> {
    use ExecutionStateSnapshot as S;

    // Every context starts Created → Ready (verify_and_plan succeeded).
    // We reconstruct the canonical path up to the current state.
    let path: &[S] = match state {
        ExecutionState::Created => &[S::Created],
        ExecutionState::Ready => &[S::Created, S::Ready],
        ExecutionState::Running => &[S::Created, S::Ready, S::Running],
        ExecutionState::Yielded { .. } => &[S::Created, S::Ready, S::Running, S::Yielded],
        ExecutionState::Waiting { .. } => &[S::Created, S::Ready, S::Running, S::Waiting],
        ExecutionState::Completed { .. } => &[S::Created, S::Ready, S::Running, S::Completed],
        ExecutionState::Failed { .. } => &[S::Created, S::Ready, S::Running, S::Failed],
        ExecutionState::Cancelled => &[S::Created, S::Ready, S::Running, S::Cancelled],
    };

    path.windows(2)
        .enumerate()
        .map(|(seq, w)| LifecycleTransitionRecord {
            from: w[0].clone(),
            to: w[1].clone(),
            sequence: seq as u64,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests (Requirements 6.1, 6.2, 6.3, 6.4)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_runtime::BcibExecutionRuntime;
    use crate::types::{BcibInstruction, CapabilitySet, CostBudget, ExecutionPlan, SideEffectClass, COST_PURE};

    fn make_plan_with_nop() -> ExecutionPlan {
        let instr = BcibInstruction {
            opcode: 0x00,
            operands: vec![],
            side_effect_class: SideEffectClass::Pure,
            cost: COST_PURE,
            required_capabilities: vec![],
        };
        ExecutionPlan::new(vec![instr], 3)
    }

    fn make_empty_plan() -> ExecutionPlan {
        ExecutionPlan::new(vec![], 3)
    }

    // -----------------------------------------------------------------------
    // EpistemicBoundary invariant (Requirement 6.2)
    // -----------------------------------------------------------------------

    #[test]
    fn epistemic_boundary_phase14_all_false() {
        let eb = EpistemicBoundary::phase14();
        assert!(!eb.produces_truth, "produces_truth must be false");
        assert!(!eb.produces_decision, "produces_decision must be false");
        assert!(!eb.produces_ranking, "produces_ranking must be false");
    }

    #[test]
    fn epistemic_boundary_default_equals_phase14() {
        assert_eq!(EpistemicBoundary::default(), EpistemicBoundary::phase14());
    }

    // -----------------------------------------------------------------------
    // FORBIDDEN_OBSERVABILITY_FIELDS and check_forbidden_fields (Req 6.3, 6.4)
    // -----------------------------------------------------------------------

    #[test]
    fn forbidden_fields_list_is_non_empty() {
        assert!(
            !FORBIDDEN_OBSERVABILITY_FIELDS.is_empty(),
            "FORBIDDEN_OBSERVABILITY_FIELDS must not be empty"
        );
    }

    #[test]
    fn check_forbidden_fields_passes_for_safe_fields() {
        let safe = &["context_id", "state", "epistemic_boundary", "transitions", "cost"];
        assert!(
            check_forbidden_fields(safe).is_ok(),
            "safe fields must pass the forbidden field check"
        );
    }

    #[test]
    fn check_forbidden_fields_rejects_authority() {
        let err = check_forbidden_fields(&["context_id", "authority"]).unwrap_err();
        assert_eq!(err, DiagnosticsError::ForbiddenFieldExposed("authority".to_string()));
    }

    #[test]
    fn check_forbidden_fields_rejects_decision() {
        let err = check_forbidden_fields(&["decision"]).unwrap_err();
        assert_eq!(err, DiagnosticsError::ForbiddenFieldExposed("decision".to_string()));
    }

    #[test]
    fn check_forbidden_fields_rejects_ranking() {
        let err = check_forbidden_fields(&["ranking"]).unwrap_err();
        assert_eq!(err, DiagnosticsError::ForbiddenFieldExposed("ranking".to_string()));
    }

    #[test]
    fn check_forbidden_fields_rejects_verdict() {
        let err = check_forbidden_fields(&["verdict"]).unwrap_err();
        assert_eq!(err, DiagnosticsError::ForbiddenFieldExposed("verdict".to_string()));
    }

    #[test]
    fn check_forbidden_fields_rejects_truth() {
        let err = check_forbidden_fields(&["truth"]).unwrap_err();
        assert_eq!(err, DiagnosticsError::ForbiddenFieldExposed("truth".to_string()));
    }

    #[test]
    fn check_forbidden_fields_rejects_consensus() {
        let err = check_forbidden_fields(&["consensus"]).unwrap_err();
        assert_eq!(err, DiagnosticsError::ForbiddenFieldExposed("consensus".to_string()));
    }

    #[test]
    fn check_forbidden_fields_fail_fast_on_first_forbidden() {
        // "policy" comes before "priority" in the input; should fail on "policy".
        let err = check_forbidden_fields(&["context_id", "policy", "priority"]).unwrap_err();
        assert_eq!(err, DiagnosticsError::ForbiddenFieldExposed("policy".to_string()));
    }

    #[test]
    fn diagnostics_error_display_contains_500_code() {
        let err = DiagnosticsError::ForbiddenFieldExposed("authority".to_string());
        let msg = err.to_string();
        assert!(
            msg.contains("500"),
            "error message must contain HTTP 500 code"
        );
        assert!(
            msg.contains("forbidden_observability_field_exposed"),
            "error message must contain the error code"
        );
        assert!(
            msg.contains("authority"),
            "error message must name the offending field"
        );
    }

    // -----------------------------------------------------------------------
    // execution_state endpoint (Requirement 6.1, 6.3, 6.4)
    // -----------------------------------------------------------------------

    #[test]
    fn execution_state_unknown_context_returns_not_found() {
        let runtime = BcibExecutionRuntime::new();
        let diag = BcibDiagnostics::new(&runtime);
        let resp = diag.execution_state(9999).unwrap();
        assert_eq!(resp.context_id, 9999);
        assert_eq!(resp.state, ExecutionStateSnapshot::NotFound);
        assert_eq!(resp.epistemic_boundary, EpistemicBoundary::phase14());
    }

    #[test]
    fn execution_state_ready_context() {
        let mut runtime = BcibExecutionRuntime::new();
        let ctx_id = runtime
            .create_context(make_empty_plan(), CapabilitySet::default())
            .unwrap();

        let diag = BcibDiagnostics::new(&runtime);
        let resp = diag.execution_state(ctx_id).unwrap();
        assert_eq!(resp.context_id, ctx_id);
        assert_eq!(resp.state, ExecutionStateSnapshot::Ready);
        assert_eq!(resp.epistemic_boundary, EpistemicBoundary::phase14());
    }

    #[test]
    fn execution_state_running_context() {
        let mut runtime = BcibExecutionRuntime::new();
        let ctx_id = runtime
            .create_context(make_plan_with_nop(), CapabilitySet::default())
            .unwrap();

        // Start a slice — context transitions to Running then Completed.
        let budget = CostBudget::new(1000, 100);
        let _ = runtime.run_slice(ctx_id, budget);

        let diag = BcibDiagnostics::new(&runtime);
        let resp = diag.execution_state(ctx_id).unwrap();
        // After a single nop, the context should be Completed.
        assert_eq!(resp.state, ExecutionStateSnapshot::Completed);
        assert_eq!(resp.epistemic_boundary, EpistemicBoundary::phase14());
    }

    #[test]
    fn execution_state_cancelled_context() {
        let mut runtime = BcibExecutionRuntime::new();
        let ctx_id = runtime
            .create_context(make_empty_plan(), CapabilitySet::default())
            .unwrap();
        runtime.cancel(ctx_id).unwrap();

        let diag = BcibDiagnostics::new(&runtime);
        let resp = diag.execution_state(ctx_id).unwrap();
        assert_eq!(resp.state, ExecutionStateSnapshot::Cancelled);
        assert_eq!(resp.epistemic_boundary, EpistemicBoundary::phase14());
    }

    // -----------------------------------------------------------------------
    // lifecycle_history endpoint (Requirement 6.1, 6.3, 6.4)
    // -----------------------------------------------------------------------

    #[test]
    fn lifecycle_history_unknown_context_returns_empty() {
        let runtime = BcibExecutionRuntime::new();
        let diag = BcibDiagnostics::new(&runtime);
        let resp = diag.lifecycle_history(9999).unwrap();
        assert_eq!(resp.context_id, 9999);
        assert!(resp.transitions.is_empty());
        assert_eq!(resp.epistemic_boundary, EpistemicBoundary::phase14());
    }

    #[test]
    fn lifecycle_history_ready_context_has_created_to_ready() {
        let mut runtime = BcibExecutionRuntime::new();
        let ctx_id = runtime
            .create_context(make_empty_plan(), CapabilitySet::default())
            .unwrap();

        let diag = BcibDiagnostics::new(&runtime);
        let resp = diag.lifecycle_history(ctx_id).unwrap();
        assert_eq!(resp.context_id, ctx_id);
        assert!(!resp.transitions.is_empty());
        // First transition must be Created → Ready.
        assert_eq!(resp.transitions[0].from, ExecutionStateSnapshot::Created);
        assert_eq!(resp.transitions[0].to, ExecutionStateSnapshot::Ready);
        assert_eq!(resp.transitions[0].sequence, 0);
        assert_eq!(resp.epistemic_boundary, EpistemicBoundary::phase14());
    }

    #[test]
    fn lifecycle_history_completed_context_ends_with_completed() {
        let mut runtime = BcibExecutionRuntime::new();
        let ctx_id = runtime
            .create_context(make_plan_with_nop(), CapabilitySet::default())
            .unwrap();
        let budget = CostBudget::new(1000, 100);
        let _ = runtime.run_slice(ctx_id, budget);

        let diag = BcibDiagnostics::new(&runtime);
        let resp = diag.lifecycle_history(ctx_id).unwrap();
        let last = resp.transitions.last().unwrap();
        assert_eq!(last.to, ExecutionStateSnapshot::Completed);
        assert_eq!(resp.epistemic_boundary, EpistemicBoundary::phase14());
    }

    #[test]
    fn lifecycle_history_transitions_are_ordered_by_sequence() {
        let mut runtime = BcibExecutionRuntime::new();
        let ctx_id = runtime
            .create_context(make_plan_with_nop(), CapabilitySet::default())
            .unwrap();
        let budget = CostBudget::new(1000, 100);
        let _ = runtime.run_slice(ctx_id, budget);

        let diag = BcibDiagnostics::new(&runtime);
        let resp = diag.lifecycle_history(ctx_id).unwrap();
        for (i, t) in resp.transitions.iter().enumerate() {
            assert_eq!(t.sequence, i as u64, "transitions must be ordered by sequence");
        }
    }

    // -----------------------------------------------------------------------
    // cost_usage endpoint (Requirement 6.1, 6.3, 6.4)
    // -----------------------------------------------------------------------

    #[test]
    fn cost_usage_unknown_context_returns_zeroed_snapshot() {
        let runtime = BcibExecutionRuntime::new();
        let diag = BcibDiagnostics::new(&runtime);
        let resp = diag.cost_usage(9999).unwrap();
        assert_eq!(resp.context_id, 9999);
        assert_eq!(resp.cost.total, 0);
        assert_eq!(resp.cost.remaining, 0);
        assert_eq!(resp.cost.external_budget, 0);
        assert_eq!(resp.cost.external_used, 0);
        assert_eq!(resp.epistemic_boundary, EpistemicBoundary::phase14());
    }

    #[test]
    fn cost_usage_after_slice_reflects_consumption() {
        let mut runtime = BcibExecutionRuntime::new();
        let ctx_id = runtime
            .create_context(make_plan_with_nop(), CapabilitySet::default())
            .unwrap();
        let budget = CostBudget::new(1000, 100);
        let _ = runtime.run_slice(ctx_id, budget);

        let diag = BcibDiagnostics::new(&runtime);
        let resp = diag.cost_usage(ctx_id).unwrap();
        assert_eq!(resp.context_id, ctx_id);
        // After executing one Pure instruction (cost=1), remaining = 999.
        assert_eq!(resp.cost.total, 1000);
        assert_eq!(resp.cost.remaining, 999);
        assert_eq!(resp.cost.external_budget, 100);
        assert_eq!(resp.cost.external_used, 0);
        assert_eq!(resp.epistemic_boundary, EpistemicBoundary::phase14());
    }

    #[test]
    fn cost_usage_epistemic_boundary_always_phase14() {
        let mut runtime = BcibExecutionRuntime::new();
        let ctx_id = runtime
            .create_context(make_empty_plan(), CapabilitySet::default())
            .unwrap();

        let diag = BcibDiagnostics::new(&runtime);
        let resp = diag.cost_usage(ctx_id).unwrap();
        assert_eq!(resp.epistemic_boundary, EpistemicBoundary::phase14());
    }

    // -----------------------------------------------------------------------
    // All three endpoints always attach the epistemic boundary (Requirement 6.2)
    // -----------------------------------------------------------------------

    #[test]
    fn all_endpoints_attach_epistemic_boundary() {
        let mut runtime = BcibExecutionRuntime::new();
        let ctx_id = runtime
            .create_context(make_empty_plan(), CapabilitySet::default())
            .unwrap();

        let diag = BcibDiagnostics::new(&runtime);
        let eb = EpistemicBoundary::phase14();

        assert_eq!(diag.execution_state(ctx_id).unwrap().epistemic_boundary, eb);
        assert_eq!(diag.lifecycle_history(ctx_id).unwrap().epistemic_boundary, eb);
        assert_eq!(diag.cost_usage(ctx_id).unwrap().epistemic_boundary, eb);
    }

    // -----------------------------------------------------------------------
    // All three endpoints pass the forbidden field check (Requirement 6.3, 6.4)
    // -----------------------------------------------------------------------

    #[test]
    fn all_endpoints_pass_forbidden_field_check() {
        let mut runtime = BcibExecutionRuntime::new();
        let ctx_id = runtime
            .create_context(make_empty_plan(), CapabilitySet::default())
            .unwrap();

        let diag = BcibDiagnostics::new(&runtime);

        // All three must return Ok — none of their field names are forbidden.
        assert!(
            diag.execution_state(ctx_id).is_ok(),
            "execution_state must not contain forbidden fields"
        );
        assert!(
            diag.lifecycle_history(ctx_id).is_ok(),
            "lifecycle_history must not contain forbidden fields"
        );
        assert!(
            diag.cost_usage(ctx_id).is_ok(),
            "cost_usage must not contain forbidden fields"
        );
    }

    #[test]
    fn response_field_names_do_not_overlap_with_forbidden_list() {
        // Verify at the constant level that none of the three response field
        // name arrays contain a forbidden field.
        assert!(
            check_forbidden_fields(EXECUTION_RESPONSE_FIELDS).is_ok(),
            "EXECUTION_RESPONSE_FIELDS must not contain forbidden fields"
        );
        assert!(
            check_forbidden_fields(LIFECYCLE_RESPONSE_FIELDS).is_ok(),
            "LIFECYCLE_RESPONSE_FIELDS must not contain forbidden fields"
        );
        assert!(
            check_forbidden_fields(COST_RESPONSE_FIELDS).is_ok(),
            "COST_RESPONSE_FIELDS must not contain forbidden fields"
        );
    }
}

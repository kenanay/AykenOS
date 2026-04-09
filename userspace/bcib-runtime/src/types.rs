/// Core types shared across BCIB v3 execution engine layers.
///
/// This module is the single source of truth for shared data models.
/// All three layers (verifier_planner, execution_runtime, scheduler_bridge)
/// communicate exclusively through these types — no cross-layer implementation
/// dependencies are permitted.

use std::fmt;

// ---------------------------------------------------------------------------
// Error taxonomy — BCIB_ERR_* codes (Requirements: 16.1–16.6, design.md)
// ---------------------------------------------------------------------------

/// All error codes produced by the BCIB v3 execution engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BcibError {
    /// BCIB header/format is invalid (structural validation failed).
    InvalidGraph(&'static str),
    /// Infinite loop or invalid jump target detected.
    ControlFlowViolation(&'static str),
    /// Required capability token is missing or has been revoked.
    CapabilityDenied(&'static str),
    /// Index or resource limit exceeded.
    BoundsViolation(&'static str),
    /// BCIB version is not supported and no backward-compat path exists.
    UnsupportedVersion(&'static str),
    /// Illegal state machine transition attempted.
    IllegalStateTransition(&'static str),
    /// ABDF capability enforcement rejected the access.
    AbdfAccessDenied(&'static str),
    /// ABDF handle has been revoked.
    AbdfHandleRevoked(&'static str),
    /// Cross-context access attempted without a capability token.
    IsolationViolation(&'static str),
    /// Bounded pool is exhausted.
    ResourceExhausted(&'static str),
    /// Cached execution plan is stale after a version bump.
    CacheStale(&'static str),
    /// Scheduler bridge failed to produce yield/resume signal.
    SchedulerBridgeFail(&'static str),
    /// BCIB attempted to store data outside ABDF boundary.
    AbdfBoundaryViolation(&'static str),
}

impl fmt::Display for BcibError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BcibError::InvalidGraph(m) => write!(f, "BCIB_ERR_INVALID_GRAPH: {}", m),
            BcibError::ControlFlowViolation(m) => write!(f, "BCIB_ERR_CONTROL_FLOW_VIOLATION: {}", m),
            BcibError::CapabilityDenied(m) => write!(f, "BCIB_ERR_CAPABILITY_DENIED: {}", m),
            BcibError::BoundsViolation(m) => write!(f, "BCIB_ERR_BOUNDS_VIOLATION: {}", m),
            BcibError::UnsupportedVersion(m) => write!(f, "BCIB_ERR_UNSUPPORTED_VERSION: {}", m),
            BcibError::IllegalStateTransition(m) => write!(f, "BCIB_ERR_ILLEGAL_STATE_TRANSITION: {}", m),
            BcibError::AbdfAccessDenied(m) => write!(f, "BCIB_ERR_ABDF_ACCESS_DENIED: {}", m),
            BcibError::AbdfHandleRevoked(m) => write!(f, "BCIB_ERR_ABDF_HANDLE_REVOKED: {}", m),
            BcibError::IsolationViolation(m) => write!(f, "BCIB_ERR_ISOLATION_VIOLATION: {}", m),
            BcibError::ResourceExhausted(m) => write!(f, "BCIB_ERR_RESOURCE_EXHAUSTED: {}", m),
            BcibError::CacheStale(m) => write!(f, "BCIB_ERR_CACHE_STALE: {}", m),
            BcibError::SchedulerBridgeFail(m) => write!(f, "BCIB_ERR_SCHEDULER_BRIDGE_FAIL: {}", m),
            BcibError::AbdfBoundaryViolation(m) => write!(f, "ABDF_BOUNDARY_VIOLATION: {}", m),
        }
    }
}

impl std::error::Error for BcibError {}

// ---------------------------------------------------------------------------
// Opcode / side-effect model
// ---------------------------------------------------------------------------

/// Opcode identifier — u8 range, six classes (design.md §Data Models).
pub type OpcodeId = u8;

/// Side-effect classification for every instruction (Requirement 16.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideEffectClass {
    /// No side effects; cost: COST_PURE.
    Pure,
    /// Mutates ABDF-managed data; capability required; cost: COST_DATA_MUTATING.
    DataMutating,
    /// AI/UI call; capability required; separate cost accounting; cost: COST_EXTERNAL.
    External,
}

/// Cost unit — base unit for instruction cost accounting (Requirement 17.1).
pub type CostUnit = u32;

pub const COST_PURE: CostUnit = 1;
pub const COST_DATA_MUTATING: CostUnit = 10;
pub const COST_EXTERNAL: CostUnit = 100;

// ---------------------------------------------------------------------------
// Capability types
// ---------------------------------------------------------------------------

/// Opaque capability token identifier.
pub type CapabilityTokenId = u64;

/// Opaque execution context identifier.
pub type ExecutionContextId = u64;

/// Set of capability token IDs granted to an execution context.
#[derive(Debug, Clone, Default)]
pub struct CapabilitySet {
    pub token_ids: Vec<CapabilityTokenId>,
}

// ---------------------------------------------------------------------------
// Resource limits
// ---------------------------------------------------------------------------

/// Per-context resource limits enforced by the verifier and runtime
/// (Requirements 16.3, 3.5, 2.8).
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_instruction_count: usize,
    /// Per-slice instruction count guard — cheap-op spam prevention.
    ///
    /// Even if the cost budget is not exhausted, a single slice may not
    /// execute more than this many instructions. This prevents a flood of
    /// cheap (Pure) instructions from monopolising the scheduler and
    /// violating fairness constraints (Requirement 2.8).
    ///
    /// Exceeding this limit causes `Running → Yielded` (yield, not fail).
    pub max_instructions_per_slice: usize,
    pub max_memory_per_context: usize,
    pub max_concurrent_handles: usize,
    pub max_ai_quota: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_instruction_count: 4096,
            max_instructions_per_slice: 256,
            max_memory_per_context: 1024 * 1024, // 1 MiB
            max_concurrent_handles: 64,
            max_ai_quota: 8,
        }
    }
}

// ---------------------------------------------------------------------------
// Instruction model
// ---------------------------------------------------------------------------

/// A single BCIB v3 instruction with resolved side-effect class and cost.
#[derive(Debug, Clone)]
pub struct BcibInstruction {
    pub opcode: OpcodeId,
    pub operands: Vec<u32>,
    pub side_effect_class: SideEffectClass,
    pub cost: CostUnit,
    /// Pre-bound capability token IDs required by this instruction (set during planning).
    pub required_capabilities: Vec<CapabilityTokenId>,
}

// ---------------------------------------------------------------------------
// ExecutionPlan — immutable after creation (Requirement 4.1, 1.6)
// ---------------------------------------------------------------------------

/// The output of `BcibVerifierPlanner::verify_and_plan()`.
///
/// Once produced, this plan is immutable. All jump targets are resolved to
/// absolute indices. All capability checks are pre-bound. The runtime MUST NOT
/// mutate this plan after it has been transferred.
///
/// Fields are `pub(crate)` to enforce the immutability contract at the type
/// level: code outside this crate cannot mutate the plan, and the runtime
/// layer receives it as a value (single-owner transfer).
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub(crate) instructions: Vec<BcibInstruction>,
    pub(crate) version: u16,
}

impl ExecutionPlan {
    /// Construct a new plan. Only `BcibVerifierPlanner` should call this.
    pub(crate) fn new(instructions: Vec<BcibInstruction>, version: u16) -> Self {
        Self { instructions, version }
    }

    /// Read-only view of the planned instructions.
    pub fn instructions(&self) -> &[BcibInstruction] {
        &self.instructions
    }

    /// BCIB version this plan was produced from.
    pub fn version(&self) -> u16 {
        self.version
    }

    /// Deterministic canonical hash of this plan's content.
    ///
    /// Produces a stable u64 hash over the canonical binary encoding of the
    /// plan. The encoding is fully deterministic: same plan content always
    /// yields the same hash (`DETERMINISM.GLOBAL` rule).
    ///
    /// ## Canonical encoding (little-endian throughout)
    ///
    /// ```text
    /// version:          u16 LE
    /// instruction_count: u32 LE
    /// for each instruction:
    ///   opcode:                u8
    ///   side_effect_class:     u8  (0=Pure, 1=DataMutating, 2=External)
    ///   cost:                  u32 LE
    ///   operand_count:         u32 LE
    ///   operands:              [u32 LE; operand_count]
    ///   required_cap_count:    u32 LE
    ///   required_capabilities: [u64 LE; required_cap_count]
    /// ```
    ///
    /// The hash algorithm is FNV-1a (64-bit), which is allocation-free,
    /// constant-time per byte, and produces no timing side-channels.
    /// Used as `PlanHash` in `ProgramCacheKey` (Requirement 19.3, 4.5).
    pub fn canonical_hash(&self) -> u64 {
        // FNV-1a 64-bit constants.
        const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
        const FNV_PRIME: u64 = 1099511628211;

        let mut hash = FNV_OFFSET_BASIS;

        macro_rules! feed {
            ($bytes:expr) => {
                for &b in $bytes {
                    hash ^= b as u64;
                    hash = hash.wrapping_mul(FNV_PRIME);
                }
            };
        }

        // version (u16 LE)
        feed!(&self.version.to_le_bytes());

        // instruction_count (u32 LE)
        feed!(&(self.instructions.len() as u32).to_le_bytes());

        for instr in &self.instructions {
            // opcode (u8)
            feed!(&[instr.opcode]);

            // side_effect_class (u8): 0=Pure, 1=DataMutating, 2=External
            let sec_byte: u8 = match instr.side_effect_class {
                SideEffectClass::Pure => 0,
                SideEffectClass::DataMutating => 1,
                SideEffectClass::External => 2,
            };
            feed!(&[sec_byte]);

            // cost (u32 LE)
            feed!(&instr.cost.to_le_bytes());

            // operands
            feed!(&(instr.operands.len() as u32).to_le_bytes());
            for &op in &instr.operands {
                feed!(&op.to_le_bytes());
            }

            // required_capabilities (pre-bound token IDs)
            feed!(&(instr.required_capabilities.len() as u32).to_le_bytes());
            for &cap in &instr.required_capabilities {
                feed!(&cap.to_le_bytes());
            }
        }

        hash
    }
}

// ---------------------------------------------------------------------------
// CostTracker — re-exported from cost_tracker module (Task 32, Group 8)
// ---------------------------------------------------------------------------

// The full implementation lives in `cost_tracker.rs`. We re-export it here
// so existing code that imports `CostTracker` from `types` continues to work.
pub use crate::cost_tracker::CostTracker;

// ---------------------------------------------------------------------------
// Cost budget (used by execution_runtime and cost_tracker)
// ---------------------------------------------------------------------------

/// Budget for a single execution slice (Requirement 2.1, 17.2).
#[derive(Debug, Clone)]
pub struct CostBudget {
    pub total: CostUnit,
    pub remaining: CostUnit,
    /// Separate budget for External instructions (AI/UI).
    pub external_budget: CostUnit,
}

impl CostBudget {
    pub fn new(total: CostUnit, external_budget: CostUnit) -> Self {
        Self { total, remaining: total, external_budget }
    }
}

// ---------------------------------------------------------------------------
// Slice result
// ---------------------------------------------------------------------------

/// Result returned by `BcibExecutionRuntime::run_slice()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SliceResult {
    /// Execution completed successfully.
    Completed,
    /// Budget exhausted — context is now Yielded; resume when scheduled.
    Yielded,
    /// Waiting for an external event (AI/data); context is now Waiting.
    Waiting,
    /// Execution failed; teardown contract has been applied.
    Failed(BcibError),
}

// ---------------------------------------------------------------------------
// Tests — ExecutionPlan canonicalization and immutability (Task 11b)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_plan(version: u16, instrs: Vec<BcibInstruction>) -> ExecutionPlan {
        ExecutionPlan::new(instrs, version)
    }

    fn nop_instr() -> BcibInstruction {
        BcibInstruction {
            opcode: 0x00,
            operands: vec![],
            side_effect_class: SideEffectClass::Pure,
            cost: COST_PURE,
            required_capabilities: vec![],
        }
    }

    fn data_instr(cap: CapabilityTokenId) -> BcibInstruction {
        BcibInstruction {
            opcode: 0x10,
            operands: vec![1, 2],
            side_effect_class: SideEffectClass::DataMutating,
            cost: COST_DATA_MUTATING,
            required_capabilities: vec![cap],
        }
    }

    // -----------------------------------------------------------------------
    // Property 12 — Plan/Runtime Consistency: canonical_hash() is stable
    // (Requirements 4.1, 1.6, 4.5)
    // -----------------------------------------------------------------------

    /// Same plan content → same hash (DETERMINISM.GLOBAL).
    #[test]
    fn canonical_hash_same_content_same_hash() {
        let plan_a = make_plan(3, vec![nop_instr()]);
        let plan_b = make_plan(3, vec![nop_instr()]);
        assert_eq!(
            plan_a.canonical_hash(),
            plan_b.canonical_hash(),
            "identical plans must produce identical canonical hashes"
        );
    }

    /// Different version → different hash.
    #[test]
    fn canonical_hash_different_version_different_hash() {
        let plan_v3 = make_plan(3, vec![nop_instr()]);
        let plan_v2 = make_plan(2, vec![nop_instr()]);
        assert_ne!(
            plan_v3.canonical_hash(),
            plan_v2.canonical_hash(),
            "plans with different versions must produce different hashes"
        );
    }

    /// Different instructions → different hash.
    #[test]
    fn canonical_hash_different_instructions_different_hash() {
        let plan_a = make_plan(3, vec![nop_instr()]);
        let plan_b = make_plan(3, vec![data_instr(42)]);
        assert_ne!(
            plan_a.canonical_hash(),
            plan_b.canonical_hash(),
            "plans with different instructions must produce different hashes"
        );
    }

    /// Different operands → different hash.
    #[test]
    fn canonical_hash_different_operands_different_hash() {
        let mut instr_a = nop_instr();
        instr_a.operands = vec![1];
        let mut instr_b = nop_instr();
        instr_b.operands = vec![2];

        let plan_a = make_plan(3, vec![instr_a]);
        let plan_b = make_plan(3, vec![instr_b]);
        assert_ne!(
            plan_a.canonical_hash(),
            plan_b.canonical_hash(),
            "plans with different operands must produce different hashes"
        );
    }

    /// Different required_capabilities → different hash.
    #[test]
    fn canonical_hash_different_capabilities_different_hash() {
        let plan_a = make_plan(3, vec![data_instr(1)]);
        let plan_b = make_plan(3, vec![data_instr(2)]);
        assert_ne!(
            plan_a.canonical_hash(),
            plan_b.canonical_hash(),
            "plans with different pre-bound capability tokens must produce different hashes"
        );
    }

    /// Empty plan → stable hash (not zero, not panic).
    #[test]
    fn canonical_hash_empty_plan_stable() {
        let plan = make_plan(3, vec![]);
        let h1 = plan.canonical_hash();
        let h2 = plan.canonical_hash();
        assert_eq!(h1, h2, "empty plan hash must be stable across calls");
        // FNV-1a offset basis after feeding version+count bytes — must not be zero.
        assert_ne!(h1, 0, "canonical hash of empty plan must not be zero");
    }

    /// Hash is stable across multiple calls on the same plan (idempotent).
    #[test]
    fn canonical_hash_idempotent() {
        let plan = make_plan(3, vec![nop_instr(), data_instr(99)]);
        let h1 = plan.canonical_hash();
        let h2 = plan.canonical_hash();
        let h3 = plan.canonical_hash();
        assert_eq!(h1, h2);
        assert_eq!(h2, h3);
    }

    /// Instruction order matters: [nop, data] ≠ [data, nop].
    #[test]
    fn canonical_hash_order_sensitive() {
        let plan_a = make_plan(3, vec![nop_instr(), data_instr(1)]);
        let plan_b = make_plan(3, vec![data_instr(1), nop_instr()]);
        assert_ne!(
            plan_a.canonical_hash(),
            plan_b.canonical_hash(),
            "instruction order must affect the canonical hash"
        );
    }

    // -----------------------------------------------------------------------
    // Immutability contract — pub(crate) fields are not accessible outside
    // the crate. This is enforced at the type level by Rust's visibility rules.
    //
    // The compile-time test is: code outside this crate cannot write to
    // `plan.instructions` or `plan.version` directly. We verify the read-only
    // public API works correctly here.
    // -----------------------------------------------------------------------

    /// instructions() returns the correct slice (read-only accessor).
    #[test]
    fn execution_plan_instructions_accessor() {
        let plan = make_plan(3, vec![nop_instr()]);
        assert_eq!(plan.instructions().len(), 1);
        assert_eq!(plan.instructions()[0].opcode, 0x00);
    }

    /// version() returns the correct version (read-only accessor).
    #[test]
    fn execution_plan_version_accessor() {
        let plan = make_plan(3, vec![]);
        assert_eq!(plan.version(), 3);
    }

    /// canonical_hash() value is consistent with the plan's instruction content.
    /// Changing a single field (side_effect_class) changes the hash.
    #[test]
    fn canonical_hash_side_effect_class_affects_hash() {
        let mut instr_pure = nop_instr();
        instr_pure.side_effect_class = SideEffectClass::Pure;

        let mut instr_ext = nop_instr();
        instr_ext.side_effect_class = SideEffectClass::External;
        instr_ext.cost = COST_EXTERNAL;

        let plan_pure = make_plan(3, vec![instr_pure]);
        let plan_ext = make_plan(3, vec![instr_ext]);
        assert_ne!(
            plan_pure.canonical_hash(),
            plan_ext.canonical_hash(),
            "different side_effect_class must produce different hashes"
        );
    }
}

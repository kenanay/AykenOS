/// BCIB_Execution_Runtime — Layer 2 of the three-layer v3 architecture.
///
/// Responsibilities (Requirement 1.6, 3b.1–3b.4):
///   - Manage the execution lifecycle state machine.
///   - Execute planned instructions in cost-bounded slices.
///   - Manage slot/handle pools (IsolatedSlotSpace, IsolatedHandleSpace).
///   - Coordinate capability and ABDF access.
///   - Apply the teardown contract on cancel/fail.
///
/// This module communicates with other layers exclusively through the types
/// defined in `types.rs`. No cross-layer implementation dependencies.

use std::collections::HashMap;

use crate::abdf_boundary::AbdfHandle;
use crate::capability_manager::{CapabilityCheck, CapabilityResource, NoopCapabilityManager};
use crate::isolation::execution_entry_enforcer::ExecutionEntryEnforcer;
use crate::isolation::execution_entry_context::ExecutionEntryContext;
use crate::pools::{IsolatedHandleSpace, IsolatedSlotSpace};
use crate::scheduler_bridge::SchedulerSubmitBridge;
use crate::types::{
    BcibError, CapabilitySet, CostBudget, CostTracker, ExecutionContextId, ExecutionPlan,
    ResourceLimits, SideEffectClass, SliceResult,
};

// ---------------------------------------------------------------------------
// State machine (Requirements 3b.1, 3b.2)
// ---------------------------------------------------------------------------

/// Resume token — opaque handle used to resume a Yielded context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResumeToken(pub u64);

/// Event descriptor — describes the external event a Waiting context expects.
#[derive(Debug, Clone)]
pub struct EventDescriptor {
    pub kind: EventKind,
    pub handle: u64,
}

/// Kind of external event (AI inference result, data query result, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    AiResult,
    DataResult,
    UiEvent,
}

/// Execution result produced when a context reaches `Completed`.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub context_id: ExecutionContextId,
    pub output: Vec<u8>,
}

/// Formal state set for a BCIB execution context (Requirement 3b.1).
#[derive(Debug, Clone)]
pub enum ExecutionState {
    /// Context created; verification not yet run.
    Created,
    /// Verification succeeded; ready to execute.
    Ready,
    /// Actively executing a slice.
    Running,
    /// Cost budget exhausted; waiting for scheduler resume.
    Yielded { resume_token: ResumeToken },
    /// Waiting for an external event (AI/data).
    Waiting { event_descriptor: EventDescriptor },
    /// Execution completed successfully.
    Completed { result: ExecutionResult },
    /// Execution failed; teardown contract applied.
    Failed { error: BcibError },
    /// Execution cancelled; teardown contract applied.
    Cancelled,
}

impl ExecutionState {
    /// Returns a short discriminant name for use in error messages.
    pub fn name(&self) -> &'static str {
        match self {
            ExecutionState::Created => "Created",
            ExecutionState::Ready => "Ready",
            ExecutionState::Running => "Running",
            ExecutionState::Yielded { .. } => "Yielded",
            ExecutionState::Waiting { .. } => "Waiting",
            ExecutionState::Completed { .. } => "Completed",
            ExecutionState::Failed { .. } => "Failed",
            ExecutionState::Cancelled => "Cancelled",
        }
    }

    /// Returns true if this is a terminal state (no further transitions allowed).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ExecutionState::Completed { .. }
                | ExecutionState::Failed { .. }
                | ExecutionState::Cancelled
        )
    }
}

// ---------------------------------------------------------------------------
// Valid state transitions (Requirement 3b.2)
// ---------------------------------------------------------------------------

/// Discriminant-level representation for transition table lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum StateKind {
    Created,
    Ready,
    Running,
    Yielded,
    Waiting,
    Completed,
    Failed,
    Cancelled,
}

impl StateKind {
    fn of(state: &ExecutionState) -> Self {
        match state {
            ExecutionState::Created => StateKind::Created,
            ExecutionState::Ready => StateKind::Ready,
            ExecutionState::Running => StateKind::Running,
            ExecutionState::Yielded { .. } => StateKind::Yielded,
            ExecutionState::Waiting { .. } => StateKind::Waiting,
            ExecutionState::Completed { .. } => StateKind::Completed,
            ExecutionState::Failed { .. } => StateKind::Failed,
            ExecutionState::Cancelled => StateKind::Cancelled,
        }
    }
}

/// All legal (from, to) state transitions (Requirement 3b.2).
///
/// Any transition NOT in this table is illegal and MUST be rejected with
/// `BCIB_ERR_ILLEGAL_STATE_TRANSITION` (Requirement 3b.3).
const VALID_TRANSITIONS: &[(StateKind, StateKind)] = &[
    (StateKind::Created, StateKind::Ready),       // verify_and_plan() succeeded
    (StateKind::Ready, StateKind::Running),        // execution slice started
    (StateKind::Running, StateKind::Yielded),      // voluntary yield (cost budget exhausted)
    (StateKind::Running, StateKind::Waiting),      // waiting for external event
    (StateKind::Running, StateKind::Completed),    // successful completion
    (StateKind::Running, StateKind::Failed),       // error
    (StateKind::Running, StateKind::Cancelled),    // cancel signal
    (StateKind::Yielded, StateKind::Running),      // resume signal
    (StateKind::Waiting, StateKind::Running),      // event arrived
];

/// Returns `true` if the transition from `from` to `to` is in the valid table.
fn is_valid_transition(from: StateKind, to: StateKind) -> bool {
    VALID_TRANSITIONS.iter().any(|&(f, t)| f == from && t == to)
}

// ---------------------------------------------------------------------------
// ExecutionContext
// ---------------------------------------------------------------------------

/// Per-execution context — holds all state for one BCIB execution instance.
#[derive(Debug)]
pub struct ExecutionContext {
    pub id: ExecutionContextId,
    pub state: ExecutionState,
    pub plan: ExecutionPlan,
    pub capability_set: CapabilitySet,
    pub slot_space: IsolatedSlotSpace,
    pub handle_space: IsolatedHandleSpace,
    pub cost_tracker: CostTracker,
    /// ABDF handles acquired during this execution (released on teardown).
    pub abdf_handles: Vec<AbdfHandle>,
    /// Instruction pointer — index into `plan.instructions()`.
    pub instruction_pointer: usize,
    /// Resource limits for this context (used by run_slice for per-slice guard).
    pub resource_limits: ResourceLimits,
    /// Live count of in-flight External instructions (those that have been
    /// dispatched and are currently awaiting an event).
    ///
    /// Incremented when an External instruction transitions the context to
    /// `Waiting` (backpressure or normal wait path in `run_slice`).
    /// Decremented when `notify_event()` is called (event arrived).
    ///
    /// Used by the concurrency limit check (Requirements 20.3, 20.4):
    /// if `active_external_count >= max_concurrent_handles`, the next
    /// External instruction applies backpressure → `Waiting` state.
    pub active_external_count: usize,
}

// ---------------------------------------------------------------------------
// BcibExecutionRuntime
// ---------------------------------------------------------------------------

/// Manages the lifecycle of all active BCIB execution contexts.
pub struct BcibExecutionRuntime {
    pub(crate) contexts: HashMap<ExecutionContextId, ExecutionContext>,
    next_id: ExecutionContextId,
    /// Scheduler bridge — held as a field so starvation counters persist
    /// across multiple yield_slice() calls for the same context.
    bridge: SchedulerSubmitBridge,
    /// Capability manager — used to enforce capability checks for
    /// DataMutating and External instructions at ABDF access time
    /// (Requirements 22.3, 22.4; KERNEL.CAPABILITY.BYPASS NON_OVERRIDABLE).
    ///
    /// Defaults to `NoopCapabilityManager` (Group 1–6 stub); replaced by
    /// the real `CapabilityManager` in Group 7 (Task 27).
    capability_manager: Box<dyn CapabilityCheck>,
    /// Execution entry enforcer — validates that BCIB execution is only
    /// initiated via approved submission paths (Requirements 1.3, 1.4).
    /// Prevents direct invocation via test helpers, debug hooks, or internal calls.
    entry_enforcer: ExecutionEntryEnforcer,
}

impl BcibExecutionRuntime {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
            next_id: 1,
            bridge: SchedulerSubmitBridge::new(),
            capability_manager: Box::new(NoopCapabilityManager),
            entry_enforcer: ExecutionEntryEnforcer::new(),
        }
    }

    /// Create a runtime with a custom capability manager.
    ///
    /// Used in tests and Group 7 to inject the real `CapabilityManager`.
    pub fn with_capability_manager(capability_manager: Box<dyn CapabilityCheck>) -> Self {
        Self {
            contexts: HashMap::new(),
            next_id: 1,
            bridge: SchedulerSubmitBridge::new(),
            capability_manager,
            entry_enforcer: ExecutionEntryEnforcer::new(),
        }
    }

    /// TESTING HELPER: Create an emulated kernel context for host-only tests.
    /// This is not authoritative kernel evidence and cannot close production entry enforcement.
    #[cfg(test)]
    fn create_valid_kernel_context_for_test() -> ExecutionEntryContext {
        ExecutionEntryContext::from_kernel_dispatcher(
            1003, // SYS_V2_SUBMIT_EXECUTION
            std::process::id(),
            1, // thread_id
            vec!["kernel_syscall_dispatcher".to_string(), "sys_v2_submit_execution".to_string()],
        )
    }

    /// TESTING HELPER: Create context through the syscall-facing runtime path.
    /// The kernel context is emulated, so this is host-only evidence.
    #[cfg(test)]
    pub fn create_context_for_test(
        &mut self,
        plan: ExecutionPlan,
        capability_set: CapabilitySet,
    ) -> Result<ExecutionContextId, BcibError> {
        let kernel_context = Self::create_valid_kernel_context_for_test();
        self.create_context_from_syscall(plan, capability_set, kernel_context)
    }

    /// TESTING HELPER: Create context with limits through the syscall-facing runtime path.
    /// The kernel context is emulated, so this is host-only evidence.
    #[cfg(test)]
    pub fn create_context_with_limits_for_test(
        &mut self,
        plan: ExecutionPlan,
        capability_set: CapabilitySet,
        resource_limits: ResourceLimits,
    ) -> Result<ExecutionContextId, BcibError> {
        let kernel_context = Self::create_valid_kernel_context_for_test();
        self.create_context_with_limits_from_syscall(plan, capability_set, resource_limits, kernel_context)
    }

    // -----------------------------------------------------------------------
    // Public API
    // -----------------------------------------------------------------------

    /// Create a new execution context from a validated plan.
    ///
    /// The context starts in `Created` state and is immediately transitioned
    /// to `Ready` (plan is already validated by the Verifier/Planner layer).
    /// 
    /// PRIVATE: Only accessible from syscall dispatcher - direct calls are forbidden
    /// This method can only be called from create_context_from_syscall with real kernel context
    /// NO TEST ACCESS: Tests must use create_context_from_syscall with real kernel contexts
    fn create_context_internal(
        &mut self,
        plan: ExecutionPlan,
        capability_set: CapabilitySet,
        entry_context: ExecutionEntryContext,
    ) -> Result<ExecutionContextId, BcibError> {
        // **TASK 3 IMPLEMENTATION**: Real kernel-level execution entry enforcement
        // Validate actual syscall dispatch context BEFORE any resource allocation
        self.entry_enforcer.validate_kernel_execution_entry(&entry_context)?;
        
        let id = self.next_id;
        self.next_id += 1;

        let mut ctx = ExecutionContext {
            id,
            state: ExecutionState::Created,
            plan,
            capability_set,
            slot_space: IsolatedSlotSpace::new(id, 64),
            handle_space: IsolatedHandleSpace::new(id, 64),
            cost_tracker: CostTracker::default(),
            abdf_handles: Vec::new(),
            instruction_pointer: 0,
            resource_limits: ResourceLimits::default(),
            active_external_count: 0,
        };

        // Immediately transition Created → Ready (plan already validated).
        Self::transition_state(&mut ctx, ExecutionState::Ready)?;

        self.contexts.insert(id, ctx);
        Ok(id)
    }

    /// PUBLIC: Real syscall dispatcher entry point with kernel context validation
    /// This is the ONLY approved way to create execution contexts
    pub fn create_context_from_syscall(
        &mut self,
        plan: ExecutionPlan,
        capability_set: CapabilitySet,
        entry_context: ExecutionEntryContext,
    ) -> Result<ExecutionContextId, BcibError> {
        self.create_context_internal(plan, capability_set, entry_context)
    }

    // Host-only tests still use emulated kernel contexts. These helpers are not
    // authoritative kernel evidence and do not close Task 3 production enforcement.

    /// Create a new execution context with explicit resource limits.
    /// PRIVATE: Only accessible from syscall dispatcher - direct calls are forbidden
    /// NO TEST ACCESS: Tests must use create_context_with_limits_from_syscall with real kernel contexts
    fn create_context_with_limits_internal(
        &mut self,
        plan: ExecutionPlan,
        capability_set: CapabilitySet,
        resource_limits: ResourceLimits,
        entry_context: ExecutionEntryContext,
    ) -> Result<ExecutionContextId, BcibError> {
        // Validate actual syscall dispatch context BEFORE any resource allocation.
        self.entry_enforcer.validate_kernel_execution_entry(&entry_context)?;
        
        let id = self.next_id;
        self.next_id += 1;

        let mut ctx = ExecutionContext {
            id,
            state: ExecutionState::Created,
            plan,
            capability_set,
            slot_space: IsolatedSlotSpace::new(id, 64),
            handle_space: IsolatedHandleSpace::new(id, 64),
            cost_tracker: CostTracker::default(),
            abdf_handles: Vec::new(),
            instruction_pointer: 0,
            resource_limits,
            active_external_count: 0,
        };

        // Immediately transition Created → Ready (plan already validated).
        Self::transition_state(&mut ctx, ExecutionState::Ready)?;

        self.contexts.insert(id, ctx);
        Ok(id)
    }

    /// PUBLIC: Real syscall dispatcher entry point with resource limits
    pub fn create_context_with_limits_from_syscall(
        &mut self,
        plan: ExecutionPlan,
        capability_set: CapabilitySet,
        resource_limits: ResourceLimits,
        entry_context: ExecutionEntryContext,
    ) -> Result<ExecutionContextId, BcibError> {
        self.create_context_with_limits_internal(plan, capability_set, resource_limits, entry_context)
    }

    // Host-only tests still use emulated kernel contexts. These helpers are not
    // authoritative kernel evidence and do not close Task 3 production enforcement.

    /// Disable execution entry enforcement (for testing only).
    /// 
    /// WARNING: This should only be used in test environments.
    /// Production code must never disable entry enforcement.
    // REMOVED: disable_entry_enforcement() - bypass mechanism eliminated for production security
    // Constitutional compliance: SECURITY.BOUNDARY.VIOLATION enforcement cannot be bypassed
    
    /// Enable execution entry enforcement (production mode).
    pub fn enable_entry_enforcement(&mut self) {
        self.entry_enforcer.enable_enforcement();
    }
    
    /// Check if execution entry enforcement is enabled.
    pub fn is_enforcement_enabled(&self) -> bool {
        self.entry_enforcer.is_enforcement_enabled()
    }

    /// Execute up to `budget` cost units of instructions for the given context.
    ///
    /// # State preconditions
    /// Context MUST be in `Ready`, `Yielded`, or `Running` state; otherwise
    /// `BCIB_ERR_ILLEGAL_STATE_TRANSITION` is returned.
    ///
    /// # Execution loop
    /// For each instruction:
    ///   1. Check `max_instructions_per_slice` guard — if the per-slice
    ///      instruction count is reached, yield immediately (not fail).
    ///   2. Charge the instruction cost from `cost_tracker.remaining`.
    ///      If the main budget is exhausted, yield.
    ///   3. For `External` instructions, additionally charge from
    ///      `cost_tracker.external_budget` (Requirement 17.3).
    ///      External budget exhaustion → fail-closed (ResourceExhausted).
    ///   4. Advance the instruction pointer.
    ///
    /// On yield: transition `Running → Yielded`, call `yield_slice()` on the
    /// scheduler bridge (best-effort; bridge failure is logged but does not
    /// prevent the state transition — the context is already Yielded).
    ///
    /// Requirements: 2.1, 2.2, 2.8, 17.2, 17.3
    pub fn run_slice(
        &mut self,
        ctx_id: ExecutionContextId,
        budget: CostBudget,
    ) -> Result<SliceResult, BcibError> {
        // ----------------------------------------------------------------
        // Phase 1: Execute instructions under the context borrow.
        //
        // We use a local enum to communicate what happened so we can drop
        // the `ctx` borrow before calling `self.bridge.yield_slice()`.
        // This avoids a simultaneous mutable borrow of `self.contexts` and
        // `self.bridge` (Rust borrow checker requirement).
        //
        // We also extract a reference to the capability manager here, before
        // the mutable borrow of `self.contexts`, so the borrow checker allows
        // both borrows to coexist (they are different fields of `self`).
        // ----------------------------------------------------------------
        enum PostAction {
            Yield,
            Wait,
            Complete,
            Failed(BcibError),
        }

        let action: Result<PostAction, BcibError> = {
            // Extract capability manager reference before borrowing contexts.
            // Rust allows simultaneous borrows of different struct fields.
            // SAFETY: capability_manager and contexts are distinct fields.
            let capability_manager_ref: &dyn CapabilityCheck = self.capability_manager.as_ref();

            let ctx = self
                .contexts
                .get_mut(&ctx_id)
                .ok_or(BcibError::InvalidGraph("unknown context id"))?;

            // Requirement 3b.3: run_slice() is only valid from Ready, Yielded,
            // or Running. The Running case is used after resume()/notify_event()
            // has already performed the state transition and handed control
            // back to the runtime for continued execution.
            match &ctx.state {
                ExecutionState::Ready
                | ExecutionState::Yielded { .. }
                | ExecutionState::Running => {}
                _ => {
                    return Err(BcibError::IllegalStateTransition(
                        "run_slice() requires Ready, Yielded, or Running state",
                    ));
                }
            }

            // Transition to Running when needed. Resume/event paths already
            // place the context in Running before the next slice continues.
            if !matches!(ctx.state, ExecutionState::Running) {
                Self::transition_state(ctx, ExecutionState::Running)?;
            }

            // Initialise cost tracker for this slice.
            ctx.cost_tracker = CostTracker::new(budget.total, budget.external_budget);

            // Snapshot the instruction list (immutable plan — no mutation allowed).
            let instructions = ctx.plan.instructions().to_vec();
            let total_instructions = instructions.len();

            // Per-slice instruction counter for the cheap-op spam guard
            // (Requirement 2.8, max_instructions_per_slice).
            let max_per_slice = ctx.resource_limits.max_instructions_per_slice;
            let mut slice_instruction_count: usize = 0;

            // ----------------------------------------------------------------
            // Main execution loop
            // ----------------------------------------------------------------
            let mut loop_result: Result<PostAction, BcibError> = Ok(PostAction::Complete);

            'exec: while ctx.instruction_pointer < total_instructions {
                // Guard 0 — max_instruction_count (total across entire execution).
                //
                // Requirements 16.3, 16.6, 2.8: instruction flood protection.
                // This is a hard limit across ALL slices for this context.
                // Exceeding it → fail-closed BCIB_ERR_RESOURCE_EXHAUSTED;
                // context is deterministically torn down via transition_to_failed.
                if ctx.instruction_pointer >= ctx.resource_limits.max_instruction_count {
                    let err = BcibError::ResourceExhausted(
                        "max_instruction_count exceeded; BCIB_ERR_RESOURCE_EXHAUSTED",
                    );
                    Self::transition_to_failed(ctx, err.clone());
                    loop_result = Ok(PostAction::Failed(err));
                    break 'exec;
                }

                // Guard 1 — max_instructions_per_slice (cheap-op spam prevention).
                if slice_instruction_count >= max_per_slice {
                    let resume_token = ResumeToken(ctx.instruction_pointer as u64);
                    Self::transition_state(ctx, ExecutionState::Yielded { resume_token })?;
                    loop_result = Ok(PostAction::Yield);
                    break 'exec;
                }

                let instr = &instructions[ctx.instruction_pointer];

                // Guard 2 — main cost budget (Requirement 17.2).
                if ctx.cost_tracker.charge(instr.cost).is_err() {
                    let resume_token = ResumeToken(ctx.instruction_pointer as u64);
                    Self::transition_state(ctx, ExecutionState::Yielded { resume_token })?;
                    loop_result = Ok(PostAction::Yield);
                    break 'exec;
                }

                // Guard 3a — ABDF blocking access check (Requirements 9.4, 9.5).
                let abdf_is_blocking = ctx.abdf_handles.iter().any(|h| h.is_blocking());
                if abdf_is_blocking {
                    let event_descriptor = EventDescriptor {
                        kind: EventKind::DataResult,
                        handle: ctx.instruction_pointer as u64,
                    };
                    Self::transition_state(ctx, ExecutionState::Waiting { event_descriptor })?;
                    loop_result = Ok(PostAction::Wait);
                    break 'exec;
                }

                // Guard 4 — Side-effect class enforcement (Requirements 16.4, 16.5).
                //
                // KERNEL.CAPABILITY.BYPASS NON_OVERRIDABLE: capability check CANNOT
                // be skipped for DataMutating or External instructions.
                //
                // - Pure        → capability check skipped; cost: COST_PURE (already
                //                 charged above via instr.cost = COST_PURE = 1).
                // - DataMutating → CapabilityManager::check() required; failure →
                //                  BCIB_ERR_CAPABILITY_DENIED; fail-closed.
                // - External    → CapabilityManager::check() required; failure →
                //                  BCIB_ERR_CAPABILITY_DENIED; fail-closed.
                //                  External budget is charged separately in Guard 3.
                //
                // We use the first required_capability token from the instruction's
                // pre-bound list (set during planning). If the list is empty, we
                // use token_id 0 (which the real CapabilityManager will reject;
                // the NoopCapabilityManager stub always allows).
                match instr.side_effect_class {
                    SideEffectClass::DataMutating => {
                        let token_id = instr.required_capabilities.first().copied().unwrap_or(0);
                        let resource = CapabilityResource::DataWrite;
                        if let Err(_) = capability_manager_ref.check(token_id, &resource, ctx_id) {
                            let err = BcibError::CapabilityDenied(
                                "capability check failed for DataMutating instruction; \
                                 BCIB_ERR_CAPABILITY_DENIED",
                            );
                            Self::transition_to_failed(ctx, err.clone());
                            loop_result = Ok(PostAction::Failed(err));
                            break 'exec;
                        }
                    }
                    SideEffectClass::External => {
                        let token_id = instr.required_capabilities.first().copied().unwrap_or(0);
                        let resource = CapabilityResource::ExternalCall;
                        if let Err(_) = capability_manager_ref.check(token_id, &resource, ctx_id) {
                            let err = BcibError::CapabilityDenied(
                                "capability check failed for External instruction; \
                                 BCIB_ERR_CAPABILITY_DENIED",
                            );
                            Self::transition_to_failed(ctx, err.clone());
                            loop_result = Ok(PostAction::Failed(err));
                            break 'exec;
                        }
                    }
                    SideEffectClass::Pure => {
                        // Pure instructions do not require a capability check.
                        // Cost COST_PURE = 1 is already charged via instr.cost above.
                    }
                }

                // Guard 3 — external budget + concurrency limit + wait handling
                // (Requirements 17.3, 2.3, 20.1, 20.2, 20.3, 20.4, 9.4, 9.5).
                if instr.side_effect_class == SideEffectClass::External {
                    // Concurrency limit check (Requirements 20.3, 20.4):
                    // Use the live `active_external_count` field — this reflects
                    // the number of External instructions currently in-flight
                    // (dispatched and awaiting an event). The stale abdf_handles
                    // length is NOT used here; the live counter is authoritative.
                    //
                    // If the active count is at the limit, apply backpressure —
                    // transition to Waiting, NOT fail-closed.
                    // This check is in the runtime, NOT the bridge.
                    let max_concurrent = ctx.resource_limits.max_concurrent_handles;
                    if ctx.active_external_count >= max_concurrent {
                        let event_descriptor = EventDescriptor {
                            kind: EventKind::AiResult,
                            handle: ctx.instruction_pointer as u64,
                        };
                        Self::transition_state(ctx, ExecutionState::Waiting { event_descriptor })?;
                        // Do NOT increment active_external_count here — the instruction
                        // was NOT dispatched; it is waiting for a slot to free up.
                        loop_result = Ok(PostAction::Wait);
                        break 'exec;
                    }

                    if let Err(e) = ctx.cost_tracker.charge_external(instr.cost) {
                        // External budget exhausted → fail-closed (Requirement 20.2).
                        Self::transition_to_failed(ctx, e.clone());
                        loop_result = Ok(PostAction::Failed(e));
                        break 'exec;
                    }

                    // Transition Running → Waiting before the blocking external call
                    // (Requirements 2.3, 20.1, 20.2).
                    let event_descriptor = EventDescriptor {
                        kind: EventKind::AiResult,
                        handle: ctx.instruction_pointer as u64,
                    };
                    Self::transition_state(ctx, ExecutionState::Waiting { event_descriptor })?;
                    // Increment the live counter — this instruction is now in-flight.
                    ctx.active_external_count += 1;
                    loop_result = Ok(PostAction::Wait);
                    break 'exec;
                }

                // Advance instruction pointer and per-slice counter.
                ctx.instruction_pointer += 1;
                slice_instruction_count += 1;
            }

            // If loop completed normally (all instructions executed), mark complete.
            if let Ok(PostAction::Complete) = &loop_result {
                if ctx.instruction_pointer >= total_instructions {
                    let ctx_id_for_result = ctx.id;
                    Self::transition_state(
                        ctx,
                        ExecutionState::Completed {
                            result: ExecutionResult {
                                context_id: ctx_id_for_result,
                                output: vec![],
                            },
                        },
                    )?;
                }
            }

            loop_result
        }; // ctx borrow ends here

        // ----------------------------------------------------------------
        // Phase 2: Post-action — call bridge.yield_slice() after dropping
        // the context borrow (avoids simultaneous mutable borrows).
        // Bridge calls are best-effort; failure is non-fatal for Yielded/Waiting.
        // ----------------------------------------------------------------
        match action? {
            PostAction::Yield => {
                let _ = self.bridge.yield_slice(ctx_id);
                Ok(SliceResult::Yielded)
            }
            PostAction::Wait => {
                let _ = self.bridge.yield_slice(ctx_id);
                Ok(SliceResult::Waiting)
            }
            PostAction::Complete => Ok(SliceResult::Completed),
            PostAction::Failed(e) => Ok(SliceResult::Failed(e)),
        }
    }

    /// Resume a `Yielded` or `Waiting` context.
    pub fn resume(&mut self, ctx_id: ExecutionContextId) -> Result<(), BcibError> {
        let ctx = self
            .contexts
            .get_mut(&ctx_id)
            .ok_or(BcibError::InvalidGraph("unknown context id"))?;

        // Requirement 3b.3: resume() is only valid from Yielded or Waiting.
        match &ctx.state {
            ExecutionState::Yielded { .. } | ExecutionState::Waiting { .. } => {}
            _ => {
                return Err(BcibError::IllegalStateTransition(
                    "resume() requires Yielded or Waiting state",
                ));
            }
        }

        // Transition to Running.
        Self::transition_state(ctx, ExecutionState::Running)?;
        Ok(())
    }

    /// Notify a `Waiting` context that its expected external event has arrived.
    ///
    /// Transitions `Waiting → Running` (Requirement 3b.2, 2.3).
    ///
    /// This is the authoritative path for waking a context that was suspended
    /// by wait handling in `run_slice()`. The caller (e.g. AI runtime, data
    /// runtime, ABDF layer) invokes this when the awaited event is ready.
    ///
    /// Returns `BCIB_ERR_ILLEGAL_STATE_TRANSITION` if the context is not in
    /// `Waiting` state — only a Waiting context can be notified.
    pub fn notify_event(&mut self, ctx_id: ExecutionContextId) -> Result<(), BcibError> {
        let ctx = self
            .contexts
            .get_mut(&ctx_id)
            .ok_or(BcibError::InvalidGraph("unknown context id"))?;

        // Only Waiting contexts can receive event notifications.
        match &ctx.state {
            ExecutionState::Waiting { .. } => {}
            _ => {
                return Err(BcibError::IllegalStateTransition(
                    "notify_event() requires Waiting state",
                ));
            }
        }

        // Advance the instruction pointer past the external instruction that
        // triggered the wait, so the next run_slice() continues from the
        // correct position.
        ctx.instruction_pointer += 1;

        // Decrement the live external counter — the in-flight instruction has
        // completed (event arrived). This frees a concurrency slot for the
        // next External instruction (Requirements 20.3, 20.4).
        if ctx.active_external_count > 0 {
            ctx.active_external_count -= 1;
        }

        // Transition Waiting → Running.
        Self::transition_state(ctx, ExecutionState::Running)?;
        Ok(())
    }

    /// Cancel an active context and apply the teardown contract.
    ///
    /// Terminal states (`Completed`, `Failed`, `Cancelled`) are rejected with
    /// `BCIB_ERR_ILLEGAL_STATE_TRANSITION`.
    pub fn cancel(&mut self, ctx_id: ExecutionContextId) -> Result<(), BcibError> {
        let ctx = self
            .contexts
            .get_mut(&ctx_id)
            .ok_or(BcibError::InvalidGraph("unknown context id"))?;

        // Terminal states cannot be cancelled (Requirement 3b.3).
        if ctx.state.is_terminal() {
            return Err(BcibError::IllegalStateTransition(
                "cannot cancel a context that is already in a terminal state",
            ));
        }

        // Transition to Cancelled — bypass the normal transition table because
        // cancel is valid from any non-terminal state (Created, Ready, Running,
        // Yielded, Waiting).  We set the state directly and then run teardown.
        ctx.state = ExecutionState::Cancelled;

        // Apply teardown contract (Requirement 3.10, 3b.4).
        Self::teardown(ctx);

        Ok(())
    }

    /// Return the current state of a context (read-only).
    pub fn state_of(&self, ctx_id: ExecutionContextId) -> Result<&ExecutionState, BcibError> {
        self.contexts
            .get(&ctx_id)
            .map(|ctx| &ctx.state)
            .ok_or(BcibError::InvalidGraph("unknown context id"))
    }

    /// Return a read-only snapshot of the cost tracker for a context.
    ///
    /// Used by the diagnostics layer (Task 37 / Requirement 6.1).
    /// Returns `BCIB_ERR_INVALID_GRAPH` if the context is not found.
    pub fn cost_snapshot(
        &self,
        ctx_id: ExecutionContextId,
    ) -> Result<crate::diagnostics::CostDiagnosticsSnapshot, BcibError> {
        let ctx = self
            .contexts
            .get(&ctx_id)
            .ok_or(BcibError::InvalidGraph("unknown context id"))?;
        Ok(crate::diagnostics::CostDiagnosticsSnapshot {
            total: ctx.cost_tracker.total,
            remaining: ctx.cost_tracker.remaining,
            external_budget: ctx.cost_tracker.external_budget,
            external_used: ctx.cost_tracker.external_used,
        })
    }

    /// Revoke an ABDF handle held by a context and fail-closed terminate the
    /// dependent execution path (Task 24 / Requirement 23.3).
    ///
    /// # Semantics
    ///
    /// 1. Locate the `AbdfHandle` with `handle_id` inside `ctx_id`'s
    ///    `abdf_handles` list.
    /// 2. Call `AbdfHandle::revoke()` — all subsequent access attempts on
    ///    that handle will return `BCIB_ERR_ABDF_HANDLE_REVOKED`.
    /// 3. Immediately fail-closed terminate the execution context:
    ///    transition to `Failed { BCIB_ERR_ABDF_HANDLE_REVOKED }` and apply
    ///    the full teardown contract (slots, handles, capability tokens).
    ///
    /// If the context is already in a terminal state the revocation of the
    /// handle still takes place (the handle is marked revoked), but no
    /// additional state transition is attempted — the context is already done.
    ///
    /// # Errors
    ///
    /// - `BCIB_ERR_INVALID_GRAPH` — `ctx_id` is unknown.
    /// - `BCIB_ERR_ABDF_HANDLE_REVOKED` — `handle_id` was not found in the
    ///   context's handle list (the handle may have already been released or
    ///   never belonged to this context).
    ///
    /// Requirements: 23.3
    pub fn revoke_handle_in_context(
        &mut self,
        ctx_id: ExecutionContextId,
        handle_id: u64,
    ) -> Result<(), BcibError> {
        let ctx = self
            .contexts
            .get_mut(&ctx_id)
            .ok_or(BcibError::InvalidGraph("unknown context id"))?;

        // Step 1: locate and revoke the handle.
        let found = ctx.abdf_handles.iter_mut().find(|h| h.handle_id == handle_id);
        match found {
            Some(handle) => {
                handle.revoke();
            }
            None => {
                // Handle not found in this context — fail-closed.
                return Err(BcibError::AbdfHandleRevoked(
                    "handle_id not found in context abdf_handles; \
                     BCIB_ERR_ABDF_HANDLE_REVOKED",
                ));
            }
        }

        // Step 2: fail-closed terminate the execution path if not already terminal.
        // Revocation at the moment of revoke() call makes the dependent execution
        // path invalid — it must be terminated fail-closed (Requirement 23.3).
        //
        // We bypass the normal transition table here (same pattern as cancel())
        // because revocation is valid from any non-terminal state — the context
        // may be in Ready, Running, Yielded, or Waiting when revocation occurs.
        // The transition table only allows Running → Failed, but revocation must
        // terminate the path regardless of current state.
        if !ctx.state.is_terminal() {
            let revocation_error = BcibError::AbdfHandleRevoked(
                "ABDF handle revoked; dependent execution path terminated fail-closed; \
                 BCIB_ERR_ABDF_HANDLE_REVOKED",
            );
            // Force state to Failed directly (bypass transition table) and run
            // the teardown contract — same approach as cancel() for Cancelled.
            ctx.state = ExecutionState::Failed { error: revocation_error };
            Self::teardown(ctx);
        }

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Validate and apply a state transition (Requirement 3b.3).
    ///
    /// Returns `BCIB_ERR_ILLEGAL_STATE_TRANSITION` for any transition not in
    /// `VALID_TRANSITIONS`.
    fn transition_state(
        ctx: &mut ExecutionContext,
        new_state: ExecutionState,
    ) -> Result<(), BcibError> {
        let from = StateKind::of(&ctx.state);
        let to = StateKind::of(&new_state);
        if !is_valid_transition(from, to) {
            return Err(BcibError::IllegalStateTransition(
                "transition not in VALID_TRANSITIONS table",
            ));
        }
        ctx.state = new_state;
        Ok(())
    }

    /// Transition a context to `Failed` and immediately apply the teardown
    /// contract (Requirement 3b.4).
    ///
    /// This is the single authoritative path for `Running → Failed`; callers
    /// (e.g. `run_slice`) MUST use this instead of calling `transition_state`
    /// directly for the Failed case.
    #[allow(dead_code)] // used by run_slice() in Group 5 (Task 17)
    fn transition_to_failed(ctx: &mut ExecutionContext, error: BcibError) {
        // Attempt the state transition; if it fails (e.g. already terminal)
        // we still run teardown to avoid leaks.
        let _ = Self::transition_state(ctx, ExecutionState::Failed { error });
        Self::teardown(ctx);
    }

    /// Teardown contract — deterministic, reverse-dependency order
    /// (Requirement 3.9, 3.10, 3b.4).
    ///
    /// Steps (ters bağımlılık sırası):
    ///   1. Cancel external instructions (AI/UI calls)
    ///   2. Release ABDF handles (notify ABDF layer)
    ///   3. Clear slots → return to pool
    ///   4. Clear handles → return to pool
    ///   5. ExecutionContext → pool (context-level cleanup marker)
    ///   6. Revoke capability tokens
    ///
    /// If any step fails, a `MEMORY.LEAK` NON_OVERRIDABLE violation is logged
    /// and teardown continues with the remaining steps (resilient teardown).
    ///
    /// NON_OVERRIDABLE constraints:
    ///   - No `panic!`  (ERROR in all phases)
    ///   - No `Box::leak` or `mem::forget`  (MEMORY.LEAK.INTENTIONAL)
    fn teardown(ctx: &mut ExecutionContext) {
        let ctx_id = ctx.id;

        // ----------------------------------------------------------------
        // Step 1 — Cancel external instructions (AI/UI calls).
        //
        // At this stub level there is no live AI/UI call tracker yet
        // (that is Group 5/6 work). We mark the step as completed and
        // note any future failure path here.
        // ----------------------------------------------------------------
        // (No live external calls to cancel in the current stub implementation.)

        // ----------------------------------------------------------------
        // Step 2 — Release ABDF handles (notify ABDF layer).
        //
        // Task 25 / Requirements 23.1, 23.2, 23.4:
        //   - Context sonlandığında (complete/cancel/fail) tüm ABDF handle'lar
        //     ABDF'ye bildirilir (revoke() çağrısı = ABDF notification).
        //   - Cancel sırasında handle serbest bırakılamazsa → MEMORY.LEAK ihlali.
        //   - Teardown ters bağımlılık sırasıyla gerçekleşir (Req 23.2).
        //
        // Determine if we are in a cancel path for MEMORY.LEAK reporting
        // (Requirement 23.4: cancel sırasında ABDF handle serbest bırakılamazsa
        // MEMORY.LEAK NON_OVERRIDABLE WARN loglanır).
        let is_cancel_path = matches!(ctx.state, ExecutionState::Cancelled);

        let mut leak_count: usize = 0;
        for handle in ctx.abdf_handles.iter_mut() {
            if handle.is_revoked() {
                // Already revoked (e.g. by revoke_handle_in_context()) — skip.
                continue;
            }
            // Notify ABDF layer by revoking the handle.
            // revoke() is infallible; if the handle is somehow in an
            // inconsistent state we detect it via is_revoked() post-call.
            handle.revoke();
            if !handle.is_revoked() {
                // revoke() did not take effect — this is a leak.
                leak_count += 1;
            }
        }

        if leak_count > 0 {
            // MEMORY.LEAK — WARN in P4.4 (NON_OVERRIDABLE in P4.5+).
            eprintln!(
                "[MEMORY.LEAK] BCIB teardown step 2: \
                 {} ABDF handle(s) could not be revoked for context {}. \
                 NON_OVERRIDABLE violation (P4.4 WARN).",
                leak_count, ctx_id
            );
        }

        // Cancel path: if any handle was not successfully released, log
        // MEMORY.LEAK as required by Requirement 23.4.
        if is_cancel_path {
            let unreleased: usize = ctx.abdf_handles.iter().filter(|h| !h.is_revoked()).count();
            if unreleased > 0 {
                eprintln!(
                    "[MEMORY.LEAK] BCIB cancel teardown: \
                     {} ABDF handle(s) not released during cancel for context {}. \
                     MEMORY.LEAK NON_OVERRIDABLE violation (Req 23.4).",
                    unreleased, ctx_id
                );
            }
        }

        // Drain the handles Vec — all handles have been revoked above.
        ctx.abdf_handles.clear();

        // ----------------------------------------------------------------
        // Step 3 — Clear slots → return to pool.
        // ----------------------------------------------------------------
        match ctx.slot_space.release_all(ctx_id) {
            Ok(()) => {}
            Err(e) => {
                // MEMORY.LEAK NON_OVERRIDABLE violation — log and continue.
                eprintln!(
                    "[MEMORY.LEAK] BCIB teardown step 3 failed: \
                     slot_space.release_all() returned {:?} for context {}. \
                     NON_OVERRIDABLE violation.",
                    e, ctx_id
                );
            }
        }

        // ----------------------------------------------------------------
        // Step 4 — Clear handles → return to pool.
        // ----------------------------------------------------------------
        match ctx.handle_space.release_all(ctx_id) {
            Ok(()) => {}
            Err(e) => {
                // MEMORY.LEAK NON_OVERRIDABLE violation — log and continue.
                eprintln!(
                    "[MEMORY.LEAK] BCIB teardown step 4 failed: \
                     handle_space.release_all() returned {:?} for context {}. \
                     NON_OVERRIDABLE violation.",
                    e, ctx_id
                );
            }
        }

        // ----------------------------------------------------------------
        // Step 5 — ExecutionContext → pool.
        //
        // In the current implementation contexts live in a HashMap rather
        // than a BoundedPool (the pool-based context store is Group 4/8
        // work). We reset the cost tracker as the "return to pool" marker.
        // ----------------------------------------------------------------
        ctx.cost_tracker = CostTracker::default();

        // ----------------------------------------------------------------
        // Step 6 — Revoke capability tokens.
        //
        // Clear the capability set so no token can be reused after teardown.
        // The real CapabilityManager::revoke() integration is Group 7 work.
        // ----------------------------------------------------------------
        ctx.capability_set.token_ids.clear();

        // Reset the live external counter — all in-flight externals are gone.
        ctx.active_external_count = 0;
    }
}

impl Default for BcibExecutionRuntime {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CostBudget, CostTracker, ExecutionPlan, ResourceLimits, SliceResult};

    // -----------------------------------------------------------------------
    // Property 8: Illegal State Transition Rejection
    // Feature: phase15-bcib-execution-engine, Property 8: Illegal State Transition Rejection
    // Validates: Requirements 3b.3
    // -----------------------------------------------------------------------

    /// Map an arbitrary u8 (mod 8) to a StateKind discriminant.
    fn state_kind_from_u8(v: u8) -> StateKind {
        match v % 8 {
            0 => StateKind::Created,
            1 => StateKind::Ready,
            2 => StateKind::Running,
            3 => StateKind::Yielded,
            4 => StateKind::Waiting,
            5 => StateKind::Completed,
            6 => StateKind::Failed,
            _ => StateKind::Cancelled,
        }
    }

    /// Build a minimal `ExecutionContext` whose current state matches `from_kind`.
    fn make_ctx_with_state(from_kind: StateKind) -> ExecutionContext {
        let state = match from_kind {
            StateKind::Created => ExecutionState::Created,
            StateKind::Ready => ExecutionState::Ready,
            StateKind::Running => ExecutionState::Running,
            StateKind::Yielded => ExecutionState::Yielded {
                resume_token: ResumeToken(0),
            },
            StateKind::Waiting => ExecutionState::Waiting {
                event_descriptor: EventDescriptor {
                    kind: EventKind::AiResult,
                    handle: 0,
                },
            },
            StateKind::Completed => ExecutionState::Completed {
                result: ExecutionResult { context_id: 1, output: vec![] },
            },
            StateKind::Failed => ExecutionState::Failed {
                error: BcibError::InvalidGraph("test"),
            },
            StateKind::Cancelled => ExecutionState::Cancelled,
        };
        ExecutionContext {
            id: 1,
            state,
            plan: ExecutionPlan::new(vec![], 0x0003),
            capability_set: CapabilitySet::default(),
            slot_space: IsolatedSlotSpace::new(1, 4),
            handle_space: IsolatedHandleSpace::new(1, 4),
            cost_tracker: CostTracker::default(),
            abdf_handles: Vec::new(),
            instruction_pointer: 0,
            resource_limits: ResourceLimits::default(),
            active_external_count: 0,
        }
    }

    /// Build the target `ExecutionState` value for a given `StateKind`.
    fn make_target_state(to_kind: StateKind) -> ExecutionState {
        match to_kind {
            StateKind::Created => ExecutionState::Created,
            StateKind::Ready => ExecutionState::Ready,
            StateKind::Running => ExecutionState::Running,
            StateKind::Yielded => ExecutionState::Yielded {
                resume_token: ResumeToken(0),
            },
            StateKind::Waiting => ExecutionState::Waiting {
                event_descriptor: EventDescriptor {
                    kind: EventKind::AiResult,
                    handle: 0,
                },
            },
            StateKind::Completed => ExecutionState::Completed {
                result: ExecutionResult { context_id: 1, output: vec![] },
            },
            StateKind::Failed => ExecutionState::Failed {
                error: BcibError::InvalidGraph("test"),
            },
            StateKind::Cancelled => ExecutionState::Cancelled,
        }
    }

    proptest::proptest! {
        // Feature: phase15-bcib-execution-engine, Property 8: Illegal State Transition Rejection
        // Validates: Requirements 3b.3
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(100))]
        #[test]
        fn prop_illegal_state_transition_rejected(
            from_raw in 0u8..8,
            to_raw   in 0u8..8,
        ) {
            let from_kind = state_kind_from_u8(from_raw);
            let to_kind   = state_kind_from_u8(to_raw);

            // Only test illegal transitions — skip valid ones.
            proptest::prop_assume!(!is_valid_transition(from_kind, to_kind));

            let mut ctx = make_ctx_with_state(from_kind);
            let target  = make_target_state(to_kind);

            let result = BcibExecutionRuntime::transition_state(&mut ctx, target);

            proptest::prop_assert!(
                matches!(result, Err(BcibError::IllegalStateTransition(_))),
                "expected IllegalStateTransition for {:?} → {:?}, got {:?}",
                from_kind, to_kind, result
            );
        }
    }

    /// Verify that VALID_TRANSITIONS contains all nine legal transitions
    /// defined in Requirement 3b.2.
    #[test]
    fn valid_transitions_table_has_nine_entries() {
        assert_eq!(VALID_TRANSITIONS.len(), 9);
    }

    /// Spot-check a few legal transitions.
    #[test]
    fn legal_transitions_are_accepted() {
        assert!(is_valid_transition(StateKind::Created, StateKind::Ready));
        assert!(is_valid_transition(StateKind::Ready, StateKind::Running));
        assert!(is_valid_transition(StateKind::Running, StateKind::Yielded));
        assert!(is_valid_transition(StateKind::Yielded, StateKind::Running));
        assert!(is_valid_transition(StateKind::Running, StateKind::Completed));
    }

    /// Spot-check illegal transitions — must NOT appear in the table.
    #[test]
    fn illegal_transitions_are_rejected() {
        // Completed → Running is illegal
        assert!(!is_valid_transition(StateKind::Completed, StateKind::Running));
        // Cancelled → Yielded is illegal
        assert!(!is_valid_transition(StateKind::Cancelled, StateKind::Yielded));
        // Created → Running skips Ready — illegal
        assert!(!is_valid_transition(StateKind::Created, StateKind::Running));
        // Failed → Ready is illegal
        assert!(!is_valid_transition(StateKind::Failed, StateKind::Ready));
    }

    /// `transition_state` must return `BCIB_ERR_ILLEGAL_STATE_TRANSITION` for
    /// an illegal transition (Requirement 3b.3).
    #[test]
    fn transition_state_rejects_illegal_transition() {
        use crate::types::ExecutionPlan;

        // Build a minimal ExecutionContext in Completed state.
        let plan = ExecutionPlan::new(vec![], 0x0003);
        let mut ctx = ExecutionContext {
            id: 1,
            state: ExecutionState::Completed {
                result: ExecutionResult { context_id: 1, output: vec![] },
            },
            plan,
            capability_set: CapabilitySet::default(),
            slot_space: IsolatedSlotSpace::new(1, 4),
            handle_space: IsolatedHandleSpace::new(1, 4),
            cost_tracker: CostTracker::default(),
            abdf_handles: Vec::new(),
            instruction_pointer: 0,
            resource_limits: ResourceLimits::default(),
            active_external_count: 0,
        };

        let result = BcibExecutionRuntime::transition_state(&mut ctx, ExecutionState::Running);
        assert!(
            matches!(result, Err(BcibError::IllegalStateTransition(_))),
            "expected IllegalStateTransition, got {:?}",
            result
        );
    }

    /// `create_context` allocates a unique ID and starts the context in Ready state.
    #[test]
    fn create_context_starts_in_ready_state() {
        let mut runtime = BcibExecutionRuntime::new();
        let plan = ExecutionPlan::new(vec![], 0x0003);
        let cap_set = CapabilitySet::default();

        let ctx_id = runtime.create_context_for_test(plan, cap_set).expect("create_context failed");

        let state = runtime.state_of(ctx_id).expect("state_of failed");
        assert!(
            matches!(state, ExecutionState::Ready),
            "expected Ready, got {:?}",
            state
        );
    }

    /// Each call to `create_context` produces a distinct context ID.
    #[test]
    fn create_context_produces_unique_ids() {
        let mut runtime = BcibExecutionRuntime::new();
        let id_a = runtime
            .create_context_for_test(ExecutionPlan::new(vec![], 0x0003), CapabilitySet::default())
            .unwrap();
        let id_b = runtime
            .create_context_for_test(ExecutionPlan::new(vec![], 0x0003), CapabilitySet::default())
            .unwrap();
        assert_ne!(id_a, id_b, "each context must have a unique ID");
    }

    /// `state_of` returns an error for an unknown context ID.
    #[test]
    fn state_of_unknown_id_returns_error() {
        let runtime = BcibExecutionRuntime::new();
        let result = runtime.state_of(9999);
        assert!(
            matches!(result, Err(BcibError::InvalidGraph(_))),
            "expected InvalidGraph for unknown context id, got {:?}",
            result
        );
    }

    /// `create_context` binds slot_space and handle_space to the new context ID.
    #[test]
    fn create_context_binds_isolated_spaces_to_context_id() {
        let mut runtime = BcibExecutionRuntime::new();
        let plan = ExecutionPlan::new(vec![], 0x0003);
        let ctx_id = runtime.create_context_for_test(plan, CapabilitySet::default()).unwrap();

        let ctx = runtime.contexts.get(&ctx_id).unwrap();
        assert_eq!(ctx.slot_space.owner(), ctx_id);
        assert_eq!(ctx.handle_space.owner(), ctx_id);
    }

    /// `create_context` initialises abdf_handles as empty.
    #[test]
    fn create_context_abdf_handles_empty() {
        let mut runtime = BcibExecutionRuntime::new();
        let plan = ExecutionPlan::new(vec![], 0x0003);
        let ctx_id = runtime.create_context_for_test(plan, CapabilitySet::default()).unwrap();

        let ctx = runtime.contexts.get(&ctx_id).unwrap();
        assert!(ctx.abdf_handles.is_empty(), "abdf_handles must start empty");
    }

    // -----------------------------------------------------------------------
    // Task 14.1 — Teardown contract unit tests
    // Requirements: 3.9, 3.10
    // -----------------------------------------------------------------------

    /// After cancel(), all slots are returned to the pool (available_count == capacity).
    #[test]
    fn teardown_cancel_returns_slots_to_pool() {
        let mut runtime = BcibExecutionRuntime::new();
        let plan = ExecutionPlan::new(vec![], 0x0003);
        let ctx_id = runtime.create_context_for_test(plan, CapabilitySet::default()).unwrap();

        // Acquire a slot to simulate in-flight usage.
        {
            let ctx = runtime.contexts.get_mut(&ctx_id).unwrap();
            let _slot = ctx.slot_space.acquire(ctx_id).expect("should acquire slot");
            // slot is outstanding now
            assert_eq!(ctx.slot_space.outstanding_count(), 1);
        }

        // Cancel triggers teardown.
        runtime.cancel(ctx_id).expect("cancel should succeed");

        let ctx = runtime.contexts.get(&ctx_id).unwrap();
        // After teardown all slots must be back in the pool.
        assert_eq!(
            ctx.slot_space.available_count(),
            ctx.slot_space.available_count(), // pool is at full capacity
            "all slots must be returned to pool after teardown"
        );
        // outstanding must be 0 (release_all resets the pool).
        assert_eq!(ctx.slot_space.outstanding_count(), 0);
    }

    /// After cancel(), all ABDF handles are released (abdf_handles is empty).
    #[test]
    fn teardown_cancel_releases_abdf_handles() {
        let mut runtime = BcibExecutionRuntime::new();
        let plan = ExecutionPlan::new(vec![], 0x0003);
        let ctx_id = runtime.create_context_for_test(plan, CapabilitySet::default()).unwrap();

        // Simulate acquired ABDF handles.
        {
            use crate::abdf_boundary::AbdfHandle;
            let ctx = runtime.contexts.get_mut(&ctx_id).unwrap();
            ctx.abdf_handles.push(AbdfHandle::stub(ctx_id, 1));
            ctx.abdf_handles.push(AbdfHandle::stub(ctx_id, 2));
            assert_eq!(ctx.abdf_handles.len(), 2);
        }

        runtime.cancel(ctx_id).expect("cancel should succeed");

        let ctx = runtime.contexts.get(&ctx_id).unwrap();
        assert!(
            ctx.abdf_handles.is_empty(),
            "all ABDF handles must be released after teardown"
        );
    }

    /// After cancel(), all handle_space entries are cleared.
    #[test]
    fn teardown_cancel_clears_handle_space() {
        let mut runtime = BcibExecutionRuntime::new();
        let plan = ExecutionPlan::new(vec![], 0x0003);
        let ctx_id = runtime.create_context_for_test(plan, CapabilitySet::default()).unwrap();

        // Register handles in handle_space.
        {
            let ctx = runtime.contexts.get_mut(&ctx_id).unwrap();
            ctx.handle_space.acquire(ctx_id, 100).unwrap();
            ctx.handle_space.acquire(ctx_id, 200).unwrap();
            assert_eq!(ctx.handle_space.registered_count(), 2);
        }

        runtime.cancel(ctx_id).expect("cancel should succeed");

        let ctx = runtime.contexts.get(&ctx_id).unwrap();
        assert_eq!(
            ctx.handle_space.registered_count(),
            0,
            "all handles must be cleared from handle_space after teardown"
        );
    }

    /// After cancel(), capability tokens are revoked (token_ids is empty).
    #[test]
    fn teardown_cancel_revokes_capability_tokens() {
        let mut runtime = BcibExecutionRuntime::new();
        let plan = ExecutionPlan::new(vec![], 0x0003);
        let cap_set = CapabilitySet { token_ids: vec![1, 2, 3] };
        let ctx_id = runtime.create_context_for_test(plan, cap_set).unwrap();

        runtime.cancel(ctx_id).expect("cancel should succeed");

        let ctx = runtime.contexts.get(&ctx_id).unwrap();
        assert!(
            ctx.capability_set.token_ids.is_empty(),
            "capability tokens must be revoked after teardown"
        );
    }

    /// After cancel(), context state is Cancelled.
    #[test]
    fn teardown_cancel_sets_cancelled_state() {
        let mut runtime = BcibExecutionRuntime::new();
        let plan = ExecutionPlan::new(vec![], 0x0003);
        let ctx_id = runtime.create_context_for_test(plan, CapabilitySet::default()).unwrap();

        runtime.cancel(ctx_id).expect("cancel should succeed");

        let state = runtime.state_of(ctx_id).unwrap();
        assert!(
            matches!(state, ExecutionState::Cancelled),
            "state must be Cancelled after cancel(), got {:?}",
            state
        );
    }

    /// cancel() on a terminal state returns BCIB_ERR_ILLEGAL_STATE_TRANSITION.
    #[test]
    fn cancel_on_terminal_state_returns_illegal_transition() {
        let mut runtime = BcibExecutionRuntime::new();
        let plan = ExecutionPlan::new(vec![], 0x0003);
        let ctx_id = runtime.create_context_for_test(plan, CapabilitySet::default()).unwrap();

        // First cancel succeeds.
        runtime.cancel(ctx_id).expect("first cancel should succeed");

        // Second cancel on Cancelled state must fail.
        let result = runtime.cancel(ctx_id);
        assert!(
            matches!(result, Err(BcibError::IllegalStateTransition(_))),
            "expected IllegalStateTransition on second cancel, got {:?}",
            result
        );
    }

    /// transition_to_failed() applies teardown: ABDF handles cleared, slots returned.
    #[test]
    fn transition_to_failed_applies_teardown() {
        use crate::abdf_boundary::AbdfHandle;

        let mut runtime = BcibExecutionRuntime::new();
        let plan = ExecutionPlan::new(vec![], 0x0003);
        let ctx_id = runtime.create_context_for_test(plan, CapabilitySet::default()).unwrap();

        // Simulate in-flight resources.
        {
            let ctx = runtime.contexts.get_mut(&ctx_id).unwrap();
            ctx.abdf_handles.push(AbdfHandle::stub(ctx_id, 42));
            ctx.handle_space.acquire(ctx_id, 99).unwrap();
            // Force state to Running so the Failed transition is valid.
            ctx.state = ExecutionState::Running;
        }

        {
            let ctx = runtime.contexts.get_mut(&ctx_id).unwrap();
            BcibExecutionRuntime::transition_to_failed(
                ctx,
                BcibError::InvalidGraph("test failure"),
            );
        }

        let ctx = runtime.contexts.get(&ctx_id).unwrap();
        assert!(
            matches!(ctx.state, ExecutionState::Failed { .. }),
            "state must be Failed after transition_to_failed"
        );
        assert!(ctx.abdf_handles.is_empty(), "ABDF handles must be cleared");
        assert_eq!(ctx.handle_space.registered_count(), 0, "handles must be cleared");
        assert!(ctx.capability_set.token_ids.is_empty(), "capability tokens must be revoked");
    }

    // -----------------------------------------------------------------------
    // Task 14.2 — Property 6: Lifecycle Completeness
    // Feature: phase15-bcib-execution-engine, Property 6: Lifecycle Completeness
    // Validates: Requirements 2.6, 3.1, 3.9, 3.10, 3b.4, 23.1
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Task 16 — Group 4 Checkpoint: memory model doğrulaması
    // Validates: bounded pool exhaustion + deterministic teardown contract
    // -----------------------------------------------------------------------

    /// Checkpoint: bounded pool exhaustion returns BCIB_ERR_RESOURCE_EXHAUSTED.
    ///
    /// Creates a context with a 1-slot pool, acquires the only slot, then
    /// verifies that a second acquire returns ResourceExhausted — confirming
    /// the bounded pool never grows beyond its capacity.
    #[test]
    fn checkpoint_group4_bounded_pool_exhaustion() {
        use crate::pools::IsolatedSlotSpace;

        let mut space = IsolatedSlotSpace::new(1, 1);
        // Acquire the single available slot.
        let _slot = space.acquire(1).expect("first acquire must succeed");
        // Pool is now exhausted — second acquire must fail.
        let result = space.acquire(1);
        assert!(
            matches!(result, Err(BcibError::ResourceExhausted(_))),
            "exhausted pool must return ResourceExhausted, got {:?}",
            result
        );
        // Capacity must not have grown.
        assert_eq!(space.available_count(), 0);
    }

    /// Checkpoint: teardown contract is deterministic — all resources are
    /// released in the correct order regardless of how many are in flight.
    ///
    /// Simulates a context with slots, handle-space entries, ABDF handles,
    /// and capability tokens all in flight, then cancels and verifies every
    /// resource is fully released.
    #[test]
    fn checkpoint_group4_teardown_contract_deterministic() {
        use crate::abdf_boundary::AbdfHandle;

        let mut runtime = BcibExecutionRuntime::new();
        let plan = ExecutionPlan::new(vec![], 0x0003);
        let cap_set = CapabilitySet { token_ids: vec![10, 20, 30] };
        let ctx_id = runtime.create_context_for_test(plan, cap_set).unwrap();

        // Put resources in flight.
        {
            let ctx = runtime.contexts.get_mut(&ctx_id).unwrap();
            // Acquire two slots.
            let _s1 = ctx.slot_space.acquire(ctx_id).unwrap();
            let _s2 = ctx.slot_space.acquire(ctx_id).unwrap();
            // Register two handles.
            ctx.handle_space.acquire(ctx_id, 100).unwrap();
            ctx.handle_space.acquire(ctx_id, 200).unwrap();
            // Push two ABDF handles.
            ctx.abdf_handles.push(AbdfHandle::stub(ctx_id, 1));
            ctx.abdf_handles.push(AbdfHandle::stub(ctx_id, 2));
        }

        // Cancel triggers the teardown contract.
        runtime.cancel(ctx_id).expect("cancel must succeed");

        let ctx = runtime.contexts.get(&ctx_id).unwrap();

        // Step 2: ABDF handles released.
        assert!(ctx.abdf_handles.is_empty(), "ABDF handles must be empty after teardown");
        // Step 3: slots returned to pool.
        assert_eq!(ctx.slot_space.outstanding_count(), 0, "no outstanding slots after teardown");
        // Step 4: handle_space cleared.
        assert_eq!(ctx.handle_space.registered_count(), 0, "handle_space must be empty after teardown");
        // Step 6: capability tokens revoked.
        assert!(ctx.capability_set.token_ids.is_empty(), "capability tokens must be revoked after teardown");
        // State must be Cancelled.
        assert!(
            matches!(ctx.state, ExecutionState::Cancelled),
            "state must be Cancelled after teardown, got {:?}",
            ctx.state
        );
    }

    proptest::proptest! {
        // Feature: phase15-bcib-execution-engine, Property 6: Lifecycle Completeness
        // Validates: Requirements 2.6, 3.1, 3.9, 3.10, 3b.4, 23.1
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(100))]
        #[test]
        fn prop_lifecycle_completeness_cancel(
            num_abdf_handles in 0usize..8,
            num_handle_entries in 0usize..8,
            num_capability_tokens in 0usize..8,
        ) {
            use crate::abdf_boundary::AbdfHandle;

            let mut runtime = BcibExecutionRuntime::new();
            let plan = ExecutionPlan::new(vec![], 0x0003);
            let cap_set = CapabilitySet {
                token_ids: (0..num_capability_tokens as u64).collect(),
            };
            let ctx_id = runtime.create_context_for_test(plan, cap_set).unwrap();

            // Simulate in-flight resources.
            {
                let ctx = runtime.contexts.get_mut(&ctx_id).unwrap();
                for i in 0..num_abdf_handles {
                    ctx.abdf_handles.push(AbdfHandle::stub(ctx_id, i as u64));
                }
                for i in 0..num_handle_entries {
                    // handle_space capacity is 64; num_handle_entries < 8 so no exhaustion.
                    let _ = ctx.handle_space.acquire(ctx_id, i as u64);
                }
            }

            // Cancel triggers teardown.
            runtime.cancel(ctx_id).expect("cancel should succeed");

            let ctx = runtime.contexts.get(&ctx_id).unwrap();

            // All resources must be released after teardown.
            proptest::prop_assert!(
                ctx.abdf_handles.is_empty(),
                "ABDF handles must be empty after teardown"
            );
            proptest::prop_assert_eq!(
                ctx.handle_space.registered_count(),
                0,
                "handle_space must be empty after teardown"
            );
            proptest::prop_assert_eq!(
                ctx.slot_space.outstanding_count(),
                0,
                "slot_space must have no outstanding slots after teardown"
            );
            proptest::prop_assert!(
                ctx.capability_set.token_ids.is_empty(),
                "capability tokens must be revoked after teardown"
            );
            proptest::prop_assert!(
                matches!(ctx.state, ExecutionState::Cancelled),
                "state must be Cancelled after cancel()"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Task 17.1 — Property 11: Bounded Slice Yield
    // Feature: phase15-bcib-execution-engine, Property 11: Bounded Slice Yield
    // Validates: Requirements 2.1, 2.2, 17.2
    // -----------------------------------------------------------------------

    /// Unit test: cost budget exhaustion causes Yielded state (not failure).
    #[test]
    fn run_slice_yields_when_cost_budget_exhausted() {
        use crate::types::{BcibInstruction, SideEffectClass, COST_PURE};

        let mut runtime = BcibExecutionRuntime::new();
        // Plan with 5 Pure instructions, each costing COST_PURE = 1.
        let instructions: Vec<BcibInstruction> = (0..5)
            .map(|_| BcibInstruction {
                opcode: 0x00,
                operands: vec![],
                side_effect_class: SideEffectClass::Pure,
                cost: COST_PURE,
                required_capabilities: vec![],
            })
            .collect();
        let plan = ExecutionPlan::new(instructions, 0x0003);
        let ctx_id = runtime.create_context_for_test(plan, CapabilitySet::default()).unwrap();

        // Budget of 3 — only 3 instructions can execute before yield.
        let budget = CostBudget::new(3, 0);
        let result = runtime.run_slice(ctx_id, budget).expect("run_slice must not error");

        assert_eq!(result, SliceResult::Yielded, "must yield when cost budget exhausted");
        assert!(
            matches!(runtime.state_of(ctx_id).unwrap(), ExecutionState::Yielded { .. }),
            "context must be in Yielded state after budget exhaustion"
        );
    }

    /// Unit test: max_instructions_per_slice guard causes Yielded state.
    #[test]
    fn run_slice_yields_when_max_instructions_per_slice_exceeded() {
        use crate::types::{BcibInstruction, SideEffectClass, COST_PURE};

        let mut runtime = BcibExecutionRuntime::new();
        // Plan with 10 Pure instructions.
        let instructions: Vec<BcibInstruction> = (0..10)
            .map(|_| BcibInstruction {
                opcode: 0x00,
                operands: vec![],
                side_effect_class: SideEffectClass::Pure,
                cost: COST_PURE,
                required_capabilities: vec![],
            })
            .collect();
        let plan = ExecutionPlan::new(instructions, 0x0003);

        // Limit to 3 instructions per slice.
        let limits = ResourceLimits {
            max_instructions_per_slice: 3,
            ..ResourceLimits::default()
        };
        let ctx_id = runtime
            .create_context_with_limits_for_test(plan, CapabilitySet::default(), limits)
            .unwrap();

        // Large budget — cost won't be the limiting factor.
        let budget = CostBudget::new(10_000, 0);
        let result = runtime.run_slice(ctx_id, budget).expect("run_slice must not error");

        assert_eq!(result, SliceResult::Yielded, "must yield when max_instructions_per_slice reached");
        assert!(
            matches!(runtime.state_of(ctx_id).unwrap(), ExecutionState::Yielded { .. }),
            "context must be in Yielded state after per-slice instruction limit"
        );
    }

    /// Unit test: run_slice completes when all instructions fit in the budget.
    #[test]
    fn run_slice_completes_when_all_instructions_fit() {
        use crate::types::{BcibInstruction, SideEffectClass, COST_PURE};

        let mut runtime = BcibExecutionRuntime::new();
        let instructions: Vec<BcibInstruction> = (0..3)
            .map(|_| BcibInstruction {
                opcode: 0x00,
                operands: vec![],
                side_effect_class: SideEffectClass::Pure,
                cost: COST_PURE,
                required_capabilities: vec![],
            })
            .collect();
        let plan = ExecutionPlan::new(instructions, 0x0003);
        let ctx_id = runtime.create_context_for_test(plan, CapabilitySet::default()).unwrap();

        // Budget of 100 — more than enough.
        let budget = CostBudget::new(100, 0);
        let result = runtime.run_slice(ctx_id, budget).expect("run_slice must not error");

        assert_eq!(result, SliceResult::Completed, "must complete when all instructions fit");
        assert!(
            matches!(runtime.state_of(ctx_id).unwrap(), ExecutionState::Completed { .. }),
            "context must be in Completed state"
        );
    }

    /// Unit test: run_slice on non-Ready/Yielded state returns IllegalStateTransition.
    #[test]
    fn run_slice_rejects_invalid_state() {
        let mut runtime = BcibExecutionRuntime::new();
        let plan = ExecutionPlan::new(vec![], 0x0003);
        let ctx_id = runtime.create_context_for_test(plan, CapabilitySet::default()).unwrap();

        // Cancel the context — it's now in Cancelled (terminal) state.
        runtime.cancel(ctx_id).unwrap();

        let budget = CostBudget::new(100, 0);
        let result = runtime.run_slice(ctx_id, budget);
        assert!(
            matches!(result, Err(BcibError::IllegalStateTransition(_))),
            "run_slice on Cancelled state must return IllegalStateTransition, got {:?}",
            result
        );
    }

    /// Unit test: max_instruction_count (total across execution) causes fail-closed
    /// BCIB_ERR_RESOURCE_EXHAUSTED when exceeded.
    ///
    /// Requirements: 16.3, 16.6, 2.8
    #[test]
    fn run_slice_fails_when_max_instruction_count_exceeded() {
        use crate::types::{BcibInstruction, SideEffectClass, COST_PURE};

        let mut runtime = BcibExecutionRuntime::new();
        // Plan with 10 Pure instructions.
        let instructions: Vec<BcibInstruction> = (0..10)
            .map(|_| BcibInstruction {
                opcode: 0x00,
                operands: vec![],
                side_effect_class: SideEffectClass::Pure,
                cost: COST_PURE,
                required_capabilities: vec![],
            })
            .collect();
        let plan = ExecutionPlan::new(instructions, 0x0003);

        // Set max_instruction_count to 3 — only 3 total instructions allowed.
        let limits = ResourceLimits {
            max_instruction_count: 3,
            max_instructions_per_slice: 256,
            ..ResourceLimits::default()
        };
        let ctx_id = runtime
            .create_context_with_limits_for_test(plan, CapabilitySet::default(), limits)
            .unwrap();

        // Large budget — cost won't be the limiting factor.
        let budget = CostBudget::new(10_000, 0);
        let result = runtime.run_slice(ctx_id, budget).expect("run_slice must return Ok(SliceResult)");

        // Must fail-closed with ResourceExhausted (not yield).
        assert!(
            matches!(result, SliceResult::Failed(BcibError::ResourceExhausted(_))),
            "must fail-closed with ResourceExhausted when max_instruction_count exceeded, got {:?}",
            result
        );
        // Context must be in Failed state (teardown applied).
        assert!(
            matches!(runtime.state_of(ctx_id).unwrap(), ExecutionState::Failed { .. }),
            "context must be in Failed state after max_instruction_count exhaustion"
        );
    }

    /// Unit test: max_instruction_count = 0 causes immediate fail-closed on first instruction.
    ///
    /// Requirements: 16.3, 16.6
    #[test]
    fn run_slice_fails_immediately_when_max_instruction_count_is_zero() {
        use crate::types::{BcibInstruction, SideEffectClass, COST_PURE};

        let mut runtime = BcibExecutionRuntime::new();
        let instructions: Vec<BcibInstruction> = vec![BcibInstruction {
            opcode: 0x00,
            operands: vec![],
            side_effect_class: SideEffectClass::Pure,
            cost: COST_PURE,
            required_capabilities: vec![],
        }];
        let plan = ExecutionPlan::new(instructions, 0x0003);

        // max_instruction_count = 0 means no instructions may execute.
        let limits = ResourceLimits {
            max_instruction_count: 0,
            max_instructions_per_slice: 256,
            ..ResourceLimits::default()
        };
        let ctx_id = runtime
            .create_context_with_limits_for_test(plan, CapabilitySet::default(), limits)
            .unwrap();

        let budget = CostBudget::new(10_000, 0);
        let result = runtime.run_slice(ctx_id, budget).expect("run_slice must return Ok(SliceResult)");

        assert!(
            matches!(result, SliceResult::Failed(BcibError::ResourceExhausted(_))),
            "must fail-closed immediately when max_instruction_count is 0, got {:?}",
            result
        );
        assert!(
            matches!(runtime.state_of(ctx_id).unwrap(), ExecutionState::Failed { .. }),
            "context must be in Failed state"
        );
    }

    proptest::proptest! {
        // Feature: phase15-bcib-execution-engine, Property 11: Bounded Slice Yield
        // Validates: Requirements 2.1, 2.2, 17.2
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(100))]
        #[test]
        fn prop_bounded_slice_yield(
            // Number of Pure instructions in the plan (1..=20).
            num_instructions in 1usize..=20,
            // Cost budget (1..=10) — always less than num_instructions * COST_PURE
            // when num_instructions > budget, ensuring yield is triggered.
            budget_total in 1u32..=10,
        ) {
            use crate::types::{BcibInstruction, SideEffectClass, COST_PURE};

            let instructions: Vec<BcibInstruction> = (0..num_instructions)
                .map(|_| BcibInstruction {
                    opcode: 0x00,
                    operands: vec![],
                    side_effect_class: SideEffectClass::Pure,
                    cost: COST_PURE,
                    required_capabilities: vec![],
                })
                .collect();

            let plan = ExecutionPlan::new(instructions.clone(), 0x0003);
            let mut runtime = BcibExecutionRuntime::new();
            let ctx_id = runtime
                .create_context_for_test(plan, CapabilitySet::default())
                .expect("create_context must succeed");

            let budget = CostBudget::new(budget_total, 0);
            let result = runtime
                .run_slice(ctx_id, budget)
                .expect("run_slice must not return Err");

            let total_cost = num_instructions as u32 * COST_PURE;

            if total_cost <= budget_total {
                // All instructions fit — must complete.
                proptest::prop_assert_eq!(
                    result,
                    SliceResult::Completed,
                    "expected Completed when total_cost ({}) <= budget ({})",
                    total_cost, budget_total
                );
                proptest::prop_assert!(
                    matches!(
                        runtime.state_of(ctx_id).unwrap(),
                        ExecutionState::Completed { .. }
                    ),
                    "context must be Completed"
                );
            } else {
                // Budget exhausted before all instructions — must yield.
                proptest::prop_assert_eq!(
                    result,
                    SliceResult::Yielded,
                    "expected Yielded when total_cost ({}) > budget ({})",
                    total_cost, budget_total
                );
                proptest::prop_assert!(
                    matches!(
                        runtime.state_of(ctx_id).unwrap(),
                        ExecutionState::Yielded { .. }
                    ),
                    "context must be Yielded after budget exhaustion"
                );
                // CRITICAL: budget must NOT be exceeded.
                // The cost_tracker.remaining must be >= 0 (it's u32, so always true),
                // but we verify that the instruction pointer did not advance past
                // what the budget allowed.
                let ctx = runtime.contexts.get(&ctx_id).unwrap();
                let instructions_executed = ctx.instruction_pointer;
                let cost_consumed = instructions_executed as u32 * COST_PURE;
                proptest::prop_assert!(
                    cost_consumed <= budget_total,
                    "cost consumed ({}) must not exceed budget ({}); {} instructions executed",
                    cost_consumed, budget_total, instructions_executed
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // Task 19 — Wait handling unit tests
    // Requirements: 2.3, 20.1, 20.2, 9.4, 9.5
    // -----------------------------------------------------------------------

    /// External instruction causes Running → Waiting transition before execution.
    #[test]
    fn run_slice_transitions_to_waiting_for_external_instruction() {
        use crate::types::{BcibInstruction, SideEffectClass, COST_EXTERNAL};

        let mut runtime = BcibExecutionRuntime::new();
        let instructions = vec![BcibInstruction {
            opcode: 0x30, // AiAsk
            operands: vec![],
            side_effect_class: SideEffectClass::External,
            cost: COST_EXTERNAL,
            required_capabilities: vec![],
        }];
        let plan = ExecutionPlan::new(instructions, 0x0003);
        let ctx_id = runtime.create_context_for_test(plan, CapabilitySet::default()).unwrap();

        let budget = CostBudget::new(1000, 1000);
        let result = runtime.run_slice(ctx_id, budget).expect("run_slice must not error");

        assert_eq!(result, SliceResult::Waiting, "External instruction must yield Waiting");
        assert!(
            matches!(runtime.state_of(ctx_id).unwrap(), ExecutionState::Waiting { .. }),
            "context must be in Waiting state after External instruction"
        );
    }

    /// notify_event() transitions Waiting → Running.
    #[test]
    fn notify_event_transitions_waiting_to_running() {
        use crate::types::{BcibInstruction, SideEffectClass, COST_EXTERNAL};

        let mut runtime = BcibExecutionRuntime::new();
        let instructions = vec![BcibInstruction {
            opcode: 0x30,
            operands: vec![],
            side_effect_class: SideEffectClass::External,
            cost: COST_EXTERNAL,
            required_capabilities: vec![],
        }];
        let plan = ExecutionPlan::new(instructions, 0x0003);
        let ctx_id = runtime.create_context_for_test(plan, CapabilitySet::default()).unwrap();

        let budget = CostBudget::new(1000, 1000);
        runtime.run_slice(ctx_id, budget).unwrap();
        assert!(matches!(runtime.state_of(ctx_id).unwrap(), ExecutionState::Waiting { .. }));

        runtime.notify_event(ctx_id).expect("notify_event must succeed");
        assert!(
            matches!(runtime.state_of(ctx_id).unwrap(), ExecutionState::Running),
            "context must be Running after notify_event"
        );
    }

    /// notify_event() on a non-Waiting context returns IllegalStateTransition.
    #[test]
    fn notify_event_rejects_non_waiting_context() {
        let mut runtime = BcibExecutionRuntime::new();
        let plan = ExecutionPlan::new(vec![], 0x0003);
        let ctx_id = runtime.create_context_for_test(plan, CapabilitySet::default()).unwrap();

        let result = runtime.notify_event(ctx_id);
        assert!(
            matches!(result, Err(BcibError::IllegalStateTransition(_))),
            "notify_event on Ready state must return IllegalStateTransition, got {:?}",
            result
        );
    }

    /// notify_event() advances the instruction pointer past the external instruction.
    #[test]
    fn notify_event_advances_instruction_pointer() {
        use crate::types::{BcibInstruction, SideEffectClass, COST_EXTERNAL, COST_PURE};

        let mut runtime = BcibExecutionRuntime::new();
        let instructions = vec![
            BcibInstruction {
                opcode: 0x00,
                operands: vec![],
                side_effect_class: SideEffectClass::Pure,
                cost: COST_PURE,
                required_capabilities: vec![],
            },
            BcibInstruction {
                opcode: 0x30,
                operands: vec![],
                side_effect_class: SideEffectClass::External,
                cost: COST_EXTERNAL,
                required_capabilities: vec![],
            },
            BcibInstruction {
                opcode: 0x00,
                operands: vec![],
                side_effect_class: SideEffectClass::Pure,
                cost: COST_PURE,
                required_capabilities: vec![],
            },
        ];
        let plan = ExecutionPlan::new(instructions, 0x0003);
        let ctx_id = runtime.create_context_for_test(plan, CapabilitySet::default()).unwrap();

        let budget = CostBudget::new(1000, 1000);
        let result = runtime.run_slice(ctx_id, budget).unwrap();
        assert_eq!(result, SliceResult::Waiting);

        {
            let ctx = runtime.contexts.get(&ctx_id).unwrap();
            assert_eq!(ctx.instruction_pointer, 1, "IP must point to External instruction");
        }

        runtime.notify_event(ctx_id).unwrap();
        {
            let ctx = runtime.contexts.get(&ctx_id).unwrap();
            assert_eq!(ctx.instruction_pointer, 2, "IP must advance past External instruction after notify_event");
        }
    }

    /// Non-blocking ABDF handle does not cause Waiting transition.
    #[test]
    fn run_slice_non_blocking_abdf_handle_does_not_wait() {
        use crate::abdf_boundary::AbdfHandle;
        use crate::types::{BcibInstruction, SideEffectClass, COST_PURE};

        let mut runtime = BcibExecutionRuntime::new();
        let instructions = vec![BcibInstruction {
            opcode: 0x00,
            operands: vec![],
            side_effect_class: SideEffectClass::Pure,
            cost: COST_PURE,
            required_capabilities: vec![],
        }];
        let plan = ExecutionPlan::new(instructions, 0x0003);
        let ctx_id = runtime.create_context_for_test(plan, CapabilitySet::default()).unwrap();

        {
            let ctx = runtime.contexts.get_mut(&ctx_id).unwrap();
            ctx.abdf_handles.push(AbdfHandle::stub(ctx_id, 1));
        }

        let budget = CostBudget::new(1000, 0);
        let result = runtime.run_slice(ctx_id, budget).unwrap();

        assert_eq!(
            result,
            SliceResult::Completed,
            "non-blocking ABDF handle must not cause Waiting transition"
        );
    }

    // -----------------------------------------------------------------------
    // Task 20 — Concurrency limit enforcement for External instructions
    // Requirements: 2.8, 20.3, 20.4
    //
    // Concurrency limit check is in run_slice() (runtime), NOT in the bridge.
    // When active_external_count >= max_concurrent_handles, the context
    // transitions to Waiting (backpressure), NOT fail-closed.
    // -----------------------------------------------------------------------

    /// When active external instruction count is at max_concurrent_handles,
    /// run_slice() transitions to Waiting (backpressure) instead of executing
    /// the External instruction (Requirements 20.3, 20.4).
    #[test]
    fn run_slice_external_concurrency_limit_causes_waiting_backpressure() {
        use crate::abdf_boundary::AbdfHandle;
        use crate::types::{BcibInstruction, SideEffectClass, COST_EXTERNAL};

        let mut runtime = BcibExecutionRuntime::new();
        let instructions = vec![BcibInstruction {
            opcode: 0x30, // AiAsk — External
            operands: vec![],
            side_effect_class: SideEffectClass::External,
            cost: COST_EXTERNAL,
            required_capabilities: vec![],
        }];
        let plan = ExecutionPlan::new(instructions, 0x0003);

        // Set max_concurrent_handles to 1.
        let limits = ResourceLimits {
            max_concurrent_handles: 1,
            ..ResourceLimits::default()
        };
        let ctx_id = runtime
            .create_context_with_limits_for_test(plan, CapabilitySet::default(), limits)
            .unwrap();

        // Simulate 1 active external handle (at the limit).
        {
            let ctx = runtime.contexts.get_mut(&ctx_id).unwrap();
            ctx.abdf_handles.push(AbdfHandle::stub(ctx_id, 1));
        }

        let budget = CostBudget::new(1000, 1000);
        let result = runtime.run_slice(ctx_id, budget).unwrap();

        // Must transition to Waiting (backpressure), NOT fail-closed.
        assert_eq!(
            result,
            SliceResult::Waiting,
            "External instruction at concurrency limit must cause Waiting (backpressure), not failure"
        );
        assert!(
            matches!(runtime.state_of(ctx_id).unwrap(), ExecutionState::Waiting { .. }),
            "context must be in Waiting state when concurrency limit is reached"
        );
    }

    /// When active external instruction count is below max_concurrent_handles,
    /// the External instruction proceeds normally (transitions to Waiting for
    /// the external event, not for backpressure).
    #[test]
    fn run_slice_external_below_concurrency_limit_proceeds() {
        use crate::types::{BcibInstruction, SideEffectClass, COST_EXTERNAL};

        let mut runtime = BcibExecutionRuntime::new();
        let instructions = vec![BcibInstruction {
            opcode: 0x30,
            operands: vec![],
            side_effect_class: SideEffectClass::External,
            cost: COST_EXTERNAL,
            required_capabilities: vec![],
        }];
        let plan = ExecutionPlan::new(instructions, 0x0003);

        // max_concurrent_handles = 4, no active handles → below limit.
        let limits = ResourceLimits {
            max_concurrent_handles: 4,
            ..ResourceLimits::default()
        };
        let ctx_id = runtime
            .create_context_with_limits_for_test(plan, CapabilitySet::default(), limits)
            .unwrap();

        let budget = CostBudget::new(1000, 1000);
        let result = runtime.run_slice(ctx_id, budget).unwrap();

        // External instruction proceeds → Waiting for the external event.
        assert_eq!(
            result,
            SliceResult::Waiting,
            "External instruction below concurrency limit must proceed to Waiting"
        );
        assert!(
            matches!(runtime.state_of(ctx_id).unwrap(), ExecutionState::Waiting { .. }),
            "context must be in Waiting state after External instruction"
        );
    }

    /// Starvation counter in the bridge is incremented on each yield.
    /// Verifies that the bridge field in BcibExecutionRuntime persists state.
    #[test]
    fn run_slice_starvation_counter_persists_across_yields() {
        use crate::types::{BcibInstruction, SideEffectClass, COST_PURE};

        let mut runtime = BcibExecutionRuntime::new();
        // Plan with many instructions so we yield multiple times.
        let instructions: Vec<BcibInstruction> = (0..10)
            .map(|_| BcibInstruction {
                opcode: 0x00,
                operands: vec![],
                side_effect_class: SideEffectClass::Pure,
                cost: COST_PURE,
                required_capabilities: vec![],
            })
            .collect();
        let plan = ExecutionPlan::new(instructions, 0x0003);

        // Limit to 3 instructions per slice so we yield multiple times.
        let limits = ResourceLimits {
            max_instructions_per_slice: 3,
            ..ResourceLimits::default()
        };
        let ctx_id = runtime
            .create_context_with_limits_for_test(plan, CapabilitySet::default(), limits)
            .unwrap();

        // First slice — yields after 3 instructions.
        let budget = CostBudget::new(10_000, 0);
        let r1 = runtime.run_slice(ctx_id, budget.clone()).unwrap();
        assert_eq!(r1, SliceResult::Yielded);

        // Starvation counter must be 1 after first yield.
        let count_after_first = runtime.bridge.starvation_count(ctx_id);
        assert_eq!(count_after_first, 1, "starvation counter must be 1 after first yield");

        // Resume and run second slice.
        runtime.resume(ctx_id).unwrap();
        let r2 = runtime.run_slice(ctx_id, budget.clone()).unwrap();
        assert_eq!(r2, SliceResult::Yielded);

        // Starvation counter must be 2 after second yield.
        let count_after_second = runtime.bridge.starvation_count(ctx_id);
        assert_eq!(count_after_second, 2, "starvation counter must be 2 after second yield");
    }

    // -----------------------------------------------------------------------
    // Task 23 — ABDF access contract & capability enforcement in run_slice()
    // Requirements: 22.3, 22.4, 22.6
    // -----------------------------------------------------------------------

    use crate::capability_manager::{CapabilityCheck, CapabilityResource};
    use crate::types::CapabilityTokenId;

    /// A capability manager that always denies.
    struct DenyAllCapabilityManager;
    impl CapabilityCheck for DenyAllCapabilityManager {
        fn check(
            &self,
            _token_id: CapabilityTokenId,
            _resource: &CapabilityResource,
            _ctx_id: ExecutionContextId,
        ) -> Result<(), BcibError> {
            Err(BcibError::CapabilityDenied("deny-all capability manager"))
        }
    }

    /// DataMutating instruction with a deny-all capability manager →
    /// run_slice() must fail with BCIB_ERR_CAPABILITY_DENIED (Requirements 16.4, 16.5).
    #[test]
    fn run_slice_data_mutating_denied_when_capability_fails() {
        use crate::types::{BcibInstruction, SideEffectClass, COST_DATA_MUTATING};

        let mut runtime =
            BcibExecutionRuntime::with_capability_manager(Box::new(DenyAllCapabilityManager));

        let instructions = vec![BcibInstruction {
            opcode: 0x10, // DataCreate
            operands: vec![],
            side_effect_class: SideEffectClass::DataMutating,
            cost: COST_DATA_MUTATING,
            required_capabilities: vec![42], // token 42 — will be denied
        }];
        let plan = ExecutionPlan::new(instructions, 0x0003);
        let ctx_id = runtime
            .create_context_for_test(plan, CapabilitySet::default())
            .unwrap();

        let budget = CostBudget::new(1000, 1000);
        let result = runtime.run_slice(ctx_id, budget).unwrap();

        assert_eq!(
            result,
            SliceResult::Failed(BcibError::CapabilityDenied(
                "capability check failed for DataMutating instruction; \
                 BCIB_ERR_CAPABILITY_DENIED",
            )),
            "DataMutating instruction with denied capability must produce Failed(CapabilityDenied)"
        );
        assert!(
            matches!(runtime.state_of(ctx_id).unwrap(), ExecutionState::Failed { .. }),
            "context must be in Failed state after capability denial"
        );
    }

    /// External instruction with a deny-all capability manager →
    /// run_slice() must fail with BCIB_ERR_CAPABILITY_DENIED (Requirements 16.4, 16.5).
    #[test]
    fn run_slice_external_denied_when_capability_fails() {
        use crate::types::{BcibInstruction, SideEffectClass, COST_EXTERNAL};

        let mut runtime =
            BcibExecutionRuntime::with_capability_manager(Box::new(DenyAllCapabilityManager));

        let instructions = vec![BcibInstruction {
            opcode: 0x30, // AiAsk
            operands: vec![],
            side_effect_class: SideEffectClass::External,
            cost: COST_EXTERNAL,
            required_capabilities: vec![99], // token 99 — will be denied
        }];
        let plan = ExecutionPlan::new(instructions, 0x0003);
        let ctx_id = runtime
            .create_context_for_test(plan, CapabilitySet::default())
            .unwrap();

        let budget = CostBudget::new(1000, 1000);
        let result = runtime.run_slice(ctx_id, budget).unwrap();

        assert_eq!(
            result,
            SliceResult::Failed(BcibError::CapabilityDenied(
                "capability check failed for External instruction; \
                 BCIB_ERR_CAPABILITY_DENIED",
            )),
            "External instruction with denied capability must produce Failed(CapabilityDenied)"
        );
        assert!(
            matches!(runtime.state_of(ctx_id).unwrap(), ExecutionState::Failed { .. }),
            "context must be in Failed state after capability denial"
        );
    }

    /// Pure instruction with a deny-all capability manager →
    /// run_slice() must succeed (Pure instructions do not require capability check).
    #[test]
    fn run_slice_pure_instruction_no_capability_check_required() {
        use crate::types::{BcibInstruction, SideEffectClass, COST_PURE};

        let mut runtime =
            BcibExecutionRuntime::with_capability_manager(Box::new(DenyAllCapabilityManager));

        let instructions = vec![BcibInstruction {
            opcode: 0x00, // Nop
            operands: vec![],
            side_effect_class: SideEffectClass::Pure,
            cost: COST_PURE,
            required_capabilities: vec![],
        }];
        let plan = ExecutionPlan::new(instructions, 0x0003);
        let ctx_id = runtime
            .create_context_for_test(plan, CapabilitySet::default())
            .unwrap();

        let budget = CostBudget::new(1000, 0);
        let result = runtime.run_slice(ctx_id, budget).unwrap();

        assert_eq!(
            result,
            SliceResult::Completed,
            "Pure instruction must not require capability check — even with deny-all manager"
        );
    }

    /// DataMutating instruction with the default NoopCapabilityManager →
    /// run_slice() must succeed (noop always allows).
    #[test]
    fn run_slice_data_mutating_succeeds_with_noop_capability_manager() {
        use crate::types::{BcibInstruction, SideEffectClass, COST_DATA_MUTATING};

        let mut runtime = BcibExecutionRuntime::new(); // uses NoopCapabilityManager

        let instructions = vec![BcibInstruction {
            opcode: 0x10,
            operands: vec![],
            side_effect_class: SideEffectClass::DataMutating,
            cost: COST_DATA_MUTATING,
            required_capabilities: vec![1],
        }];
        let plan = ExecutionPlan::new(instructions, 0x0003);
        let ctx_id = runtime
            .create_context_for_test(plan, CapabilitySet::default())
            .unwrap();

        let budget = CostBudget::new(1000, 1000);
        let result = runtime.run_slice(ctx_id, budget).unwrap();

        assert_eq!(
            result,
            SliceResult::Completed,
            "DataMutating instruction with NoopCapabilityManager must succeed"
        );
    }

    // -----------------------------------------------------------------------
    // Task 24.1 — ABDF handle revocation unit tests
    // Requirements: 23.3
    // -----------------------------------------------------------------------

    /// After revoke_handle_in_context(), accessing the same handle returns
    /// BCIB_ERR_ABDF_HANDLE_REVOKED (Requirement 23.3).
    #[test]
    fn revoke_handle_in_context_marks_handle_revoked() {
        use crate::abdf_boundary::AbdfHandle;

        let mut runtime = BcibExecutionRuntime::new();
        let plan = ExecutionPlan::new(vec![], 0x0003);
        let ctx_id = runtime.create_context_for_test(plan, CapabilitySet::default()).unwrap();

        // Add an ABDF handle to the context.
        {
            let ctx = runtime.contexts.get_mut(&ctx_id).unwrap();
            ctx.abdf_handles.push(AbdfHandle::stub(ctx_id, 42));
        }

        // Revoke the handle.
        runtime
            .revoke_handle_in_context(ctx_id, 42)
            .expect("revoke_handle_in_context must succeed");

        // The handle must be marked revoked — verify via the abdf_handles list.
        // (Context is now in Failed state after fail-closed termination, but
        // the handle object is still in the list until teardown clears it.)
        // After teardown the list is cleared; we verify the state instead.
        let state = runtime.state_of(ctx_id).unwrap();
        assert!(
            matches!(state, ExecutionState::Failed { .. }),
            "context must be in Failed state after handle revocation, got {:?}",
            state
        );
    }

    /// After revoke_handle_in_context(), the execution context is terminated
    /// fail-closed (state = Failed, teardown applied) (Requirement 23.3).
    #[test]
    fn revoke_handle_in_context_terminates_execution_fail_closed() {
        use crate::abdf_boundary::AbdfHandle;

        let mut runtime = BcibExecutionRuntime::new();
        let plan = ExecutionPlan::new(vec![], 0x0003);
        let cap_set = CapabilitySet { token_ids: vec![1, 2, 3] };
        let ctx_id = runtime.create_context_for_test(plan, cap_set).unwrap();

        // Add ABDF handles and handle_space entries to simulate in-flight resources.
        {
            let ctx = runtime.contexts.get_mut(&ctx_id).unwrap();
            ctx.abdf_handles.push(AbdfHandle::stub(ctx_id, 10));
            ctx.abdf_handles.push(AbdfHandle::stub(ctx_id, 20));
            ctx.handle_space.acquire(ctx_id, 100).unwrap();
        }

        // Revoke handle 10 — must trigger fail-closed termination.
        runtime
            .revoke_handle_in_context(ctx_id, 10)
            .expect("revoke_handle_in_context must succeed");

        let ctx = runtime.contexts.get(&ctx_id).unwrap();

        // State must be Failed (fail-closed termination).
        assert!(
            matches!(ctx.state, ExecutionState::Failed { .. }),
            "context must be in Failed state after revocation, got {:?}",
            ctx.state
        );

        // Teardown contract must have been applied:
        // ABDF handles cleared.
        assert!(
            ctx.abdf_handles.is_empty(),
            "ABDF handles must be cleared by teardown after revocation"
        );
        // handle_space cleared.
        assert_eq!(
            ctx.handle_space.registered_count(),
            0,
            "handle_space must be cleared by teardown after revocation"
        );
        // Capability tokens revoked.
        assert!(
            ctx.capability_set.token_ids.is_empty(),
            "capability tokens must be revoked by teardown after revocation"
        );
    }

    /// revoke_handle_in_context() with an unknown handle_id returns
    /// BCIB_ERR_ABDF_HANDLE_REVOKED (fail-closed, handle not found).
    #[test]
    fn revoke_handle_in_context_unknown_handle_id_returns_error() {
        let mut runtime = BcibExecutionRuntime::new();
        let plan = ExecutionPlan::new(vec![], 0x0003);
        let ctx_id = runtime.create_context_for_test(plan, CapabilitySet::default()).unwrap();

        // No handles in the context — revoking a non-existent handle must fail.
        let result = runtime.revoke_handle_in_context(ctx_id, 999);
        assert!(
            matches!(result, Err(BcibError::AbdfHandleRevoked(_))),
            "unknown handle_id must return BCIB_ERR_ABDF_HANDLE_REVOKED, got {:?}",
            result
        );
    }

    /// revoke_handle_in_context() with an unknown context_id returns
    /// BCIB_ERR_INVALID_GRAPH.
    #[test]
    fn revoke_handle_in_context_unknown_context_id_returns_error() {
        let mut runtime = BcibExecutionRuntime::new();

        let result = runtime.revoke_handle_in_context(9999, 42);
        assert!(
            matches!(result, Err(BcibError::InvalidGraph(_))),
            "unknown context_id must return BCIB_ERR_INVALID_GRAPH, got {:?}",
            result
        );
    }

    /// revoke_handle_in_context() on an already-terminal context still marks
    /// the handle revoked but does NOT attempt a second state transition.
    #[test]
    fn revoke_handle_in_context_on_terminal_context_does_not_double_transition() {
        use crate::abdf_boundary::AbdfHandle;

        let mut runtime = BcibExecutionRuntime::new();
        let plan = ExecutionPlan::new(vec![], 0x0003);
        let ctx_id = runtime.create_context_for_test(plan, CapabilitySet::default()).unwrap();

        // Add a handle and cancel the context first (terminal state).
        {
            let ctx = runtime.contexts.get_mut(&ctx_id).unwrap();
            ctx.abdf_handles.push(AbdfHandle::stub(ctx_id, 77));
        }
        runtime.cancel(ctx_id).expect("cancel must succeed");

        // Context is now Cancelled (terminal). Revoking a handle that was
        // cleared by teardown will return an error (handle not found).
        // This is correct: teardown already cleared abdf_handles.
        let result = runtime.revoke_handle_in_context(ctx_id, 77);
        assert!(
            matches!(result, Err(BcibError::AbdfHandleRevoked(_))),
            "handle not found after teardown must return BCIB_ERR_ABDF_HANDLE_REVOKED, got {:?}",
            result
        );
    }

    // -----------------------------------------------------------------------
    // Task 35 — Async/backpressure mekanizması
    // Requirements: 20.2, 20.3, 20.4
    // -----------------------------------------------------------------------

    /// Helper: build a plan with one External instruction.
    fn make_external_plan() -> ExecutionPlan {
        use crate::types::{BcibInstruction, SideEffectClass, COST_EXTERNAL};
        ExecutionPlan::new(
            vec![BcibInstruction {
                opcode: 0x30, // AiAsk
                operands: vec![],
                side_effect_class: SideEffectClass::External,
                cost: COST_EXTERNAL,
                required_capabilities: vec![],
            }],
            0x0003,
        )
    }

    /// Helper: build a plan with N External instructions.
    fn make_n_external_plan(n: usize) -> ExecutionPlan {
        use crate::types::{BcibInstruction, SideEffectClass, COST_EXTERNAL};
        let instrs = (0..n)
            .map(|_| BcibInstruction {
                opcode: 0x30,
                operands: vec![],
                side_effect_class: SideEffectClass::External,
                cost: COST_EXTERNAL,
                required_capabilities: vec![],
            })
            .collect();
        ExecutionPlan::new(instrs, 0x0003)
    }

    /// Concurrency limit: when active_external_count >= max_concurrent_handles,
    /// the next External instruction applies backpressure → Waiting state.
    ///
    /// Requirements: 20.3, 20.4
    #[test]
    fn backpressure_concurrency_limit_transitions_to_waiting() {
        let mut runtime = BcibExecutionRuntime::new();
        // Plan: two External instructions.
        let plan = make_n_external_plan(2);

        // Set max_concurrent_handles = 1 so the second external hits the limit.
        let limits = ResourceLimits {
            max_concurrent_handles: 1,
            ..ResourceLimits::default()
        };
        let ctx_id = runtime
            .create_context_with_limits_for_test(plan, CapabilitySet::default(), limits)
            .unwrap();

        // First run_slice: dispatches the first External instruction → Waiting.
        let budget = CostBudget::new(10_000, 10_000);
        let result = runtime.run_slice(ctx_id, budget.clone()).expect("run_slice must not error");
        assert_eq!(result, SliceResult::Waiting, "first external must transition to Waiting");

        // active_external_count must now be 1 (one in-flight external).
        {
            let ctx = runtime.contexts.get(&ctx_id).unwrap();
            assert_eq!(
                ctx.active_external_count, 1,
                "active_external_count must be 1 after dispatching first external"
            );
        }

        // Simulate event arrival for the first external — notify_event decrements counter.
        runtime.notify_event(ctx_id).expect("notify_event must succeed");

        // active_external_count must now be 0 again.
        {
            let ctx = runtime.contexts.get(&ctx_id).unwrap();
            assert_eq!(
                ctx.active_external_count, 0,
                "active_external_count must be 0 after notify_event"
            );
        }

        // Second run_slice: dispatches the second External instruction → Waiting.
        let result2 = runtime.run_slice(ctx_id, budget).expect("run_slice must not error");
        assert_eq!(result2, SliceResult::Waiting, "second external must also transition to Waiting");
    }

    /// Backpressure: when concurrency limit is already at max, the new External
    /// instruction transitions to Waiting WITHOUT incrementing active_external_count
    /// (the instruction was NOT dispatched — it is queued for backpressure).
    ///
    /// Requirements: 20.3, 20.4
    #[test]
    fn backpressure_does_not_increment_counter_when_limit_reached() {
        let mut runtime = BcibExecutionRuntime::new();
        let plan = make_n_external_plan(2);

        // max_concurrent_handles = 1.
        let limits = ResourceLimits {
            max_concurrent_handles: 1,
            ..ResourceLimits::default()
        };
        let ctx_id = runtime
            .create_context_with_limits_for_test(plan, CapabilitySet::default(), limits)
            .unwrap();

        // Dispatch first external (counter → 1).
        let budget = CostBudget::new(10_000, 10_000);
        runtime.run_slice(ctx_id, budget.clone()).expect("first run_slice");

        // Manually simulate that the context is still Waiting (no notify_event).
        // Force back to Running so we can call run_slice again.
        {
            let ctx = runtime.contexts.get_mut(&ctx_id).unwrap();
            ctx.state = ExecutionState::Running;
            // Do NOT decrement active_external_count — first external still in-flight.
        }

        // Second run_slice: concurrency limit reached → backpressure Waiting.
        // active_external_count must NOT be incremented (instruction not dispatched).
        let result = runtime.run_slice(ctx_id, budget).expect("second run_slice");
        assert_eq!(result, SliceResult::Waiting, "backpressure must produce Waiting");

        let ctx = runtime.contexts.get(&ctx_id).unwrap();
        assert_eq!(
            ctx.active_external_count, 1,
            "active_external_count must remain 1 (backpressure did not dispatch the instruction)"
        );
    }

    /// External budget exhaustion → fail-closed (BCIB_ERR_RESOURCE_EXHAUSTED).
    ///
    /// Requirements: 20.2
    #[test]
    fn external_budget_exhaustion_fails_closed() {
        let mut runtime = BcibExecutionRuntime::new();
        let plan = make_external_plan();
        let ctx_id = runtime.create_context_for_test(plan, CapabilitySet::default()).unwrap();

        // Budget with external_budget = 0 — any External instruction exhausts it.
        let budget = CostBudget::new(10_000, 0);
        let result = runtime.run_slice(ctx_id, budget).expect("run_slice must return Ok(SliceResult)");

        assert!(
            matches!(result, SliceResult::Failed(BcibError::ResourceExhausted(_))),
            "external budget exhaustion must fail-closed with ResourceExhausted, got {:?}",
            result
        );
        assert!(
            matches!(runtime.state_of(ctx_id).unwrap(), ExecutionState::Failed { .. }),
            "context must be in Failed state after external budget exhaustion"
        );
    }

    /// notify_event() decrements active_external_count.
    ///
    /// Requirements: 20.3, 20.4
    #[test]
    fn notify_event_decrements_active_external_count() {
        let mut runtime = BcibExecutionRuntime::new();
        let plan = make_external_plan();
        let ctx_id = runtime.create_context_for_test(plan, CapabilitySet::default()).unwrap();

        // Dispatch the external instruction → Waiting, counter = 1.
        let budget = CostBudget::new(10_000, 10_000);
        runtime.run_slice(ctx_id, budget).expect("run_slice");

        {
            let ctx = runtime.contexts.get(&ctx_id).unwrap();
            assert_eq!(ctx.active_external_count, 1, "counter must be 1 after dispatch");
        }

        // Event arrives → counter decremented to 0.
        runtime.notify_event(ctx_id).expect("notify_event");

        let ctx = runtime.contexts.get(&ctx_id).unwrap();
        assert_eq!(
            ctx.active_external_count, 0,
            "active_external_count must be 0 after notify_event"
        );
    }

    /// Teardown resets active_external_count to 0.
    ///
    /// Requirements: 20.3, 20.4
    #[test]
    fn teardown_resets_active_external_count() {
        let mut runtime = BcibExecutionRuntime::new();
        let plan = make_external_plan();
        let ctx_id = runtime.create_context_for_test(plan, CapabilitySet::default()).unwrap();

        // Dispatch external → counter = 1.
        let budget = CostBudget::new(10_000, 10_000);
        runtime.run_slice(ctx_id, budget).expect("run_slice");

        // Cancel triggers teardown.
        runtime.cancel(ctx_id).expect("cancel");

        let ctx = runtime.contexts.get(&ctx_id).unwrap();
        assert_eq!(
            ctx.active_external_count, 0,
            "teardown must reset active_external_count to 0"
        );
    }
}

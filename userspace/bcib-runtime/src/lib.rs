/// BCIB Execution Engine v3 — public API surface.
///
/// The three-layer architecture (Requirement 1.6):
///   - `verifier_planner`  — BCIB_Verifier/Planner
///   - `execution_runtime` — BCIB_Execution_Runtime
///   - `scheduler_bridge`  — Scheduler_Submit_Bridge
///
/// Shared types live in `types`. The legacy `executor` module is preserved
/// for backward compatibility; existing call sites are not broken.

pub mod abdf_boundary;
pub mod binary_format;
pub mod capability_manager;
pub mod compat;
pub mod cost_tracker;
pub mod diagnostics;
pub mod executor;
pub mod execution_runtime;
pub mod opcode_registry;
pub mod pools;
pub mod program_cache;
pub mod scheduler_bridge;
pub mod types;
pub mod verifier_planner;

// ---------------------------------------------------------------------------
// v0.2 backward-compatible re-exports (existing call sites must not break)
// ---------------------------------------------------------------------------
pub use executor::{
    BcibExecutor, BcibGraph, CapabilityManager, CapabilityToken, ExecutionContext,
    ExecutionError,
};

// ---------------------------------------------------------------------------
// v3 public API re-exports
// ---------------------------------------------------------------------------
pub use types::{
    BcibError, BcibInstruction, CapabilitySet, CostBudget, CostTracker, CostUnit,
    ExecutionContextId, ExecutionPlan, ResourceLimits, SideEffectClass, SliceResult,
    COST_DATA_MUTATING, COST_EXTERNAL, COST_PURE,
};

pub use verifier_planner::BcibVerifierPlanner;

pub use capability_manager::{CapabilityCheck, CapabilityResource, NoopCapabilityManager};

pub use execution_runtime::{
    BcibExecutionRuntime, EventDescriptor, EventKind,
    ExecutionResult as RuntimeExecutionResult,
    ExecutionState, ResumeToken,
};

pub use scheduler_bridge::{ExecutionId, ExecutionResult as BridgeExecutionResult, SchedulerSubmitBridge};

pub use abdf_boundary::AbdfHandle;

pub use pools::{BoundedPool, ExecutionSlot, HandleEntry, IsolatedHandleSpace, IsolatedSlotSpace};

pub use binary_format::{
    parse_header, parse_section_table, BcibHeader, SectionEntry, SectionId,
    BCIB_MAGIC, BCIB_VERSION_V02, BCIB_VERSION_V3, HEADER_SIZE, SECTION_ENTRY_SIZE,
};

pub use opcode_registry::{lookup_opcode, is_reserved_v02, OpcodeClass, OpcodeDescriptor, RESERVED_V02};

pub use compat::{check_version_compatibility, validate_opcode_no_conflict, CompatResult};

pub use program_cache::{ProgramCache, ProgramCacheKey};

pub use diagnostics::{
    BcibDiagnostics, CostDiagnosticsResponse, CostDiagnosticsSnapshot,
    EpistemicBoundary, ExecutionDiagnosticsResponse, ExecutionStateSnapshot,
    LifecycleDiagnosticsResponse, LifecycleTransitionRecord,
};

use crate::error::RuntimeResult;
use crate::runtime::RuntimeState;
use crate::types::{BcibInstruction, PendingOp};

/// UI executor (stub for v0.1)
/// UI render MUST be blocking (BCIB Execution Semantics v0.1)
/// UI is read-only consumer
pub fn execute_ui(
    _inst: BcibInstruction,
    state: &RuntimeState,
) -> RuntimeResult<PendingOp> {
    Ok(PendingOp::RenderUi {
        context_id: state.current_ctx,
    })
}

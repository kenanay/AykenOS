use crate::error::RuntimeResult;
use crate::runtime::RuntimeState;
use crate::types::{BcibInstruction, PendingOp};

/// GPU executor (stub for v0.1)
/// GPU buffer ownership remains with runtime
/// GPU only borrows (BCIB Execution Semantics v0.1)
pub fn execute_gpu(
    _inst: BcibInstruction,
    _state: &RuntimeState,
) -> RuntimeResult<PendingOp> {
    // GPU operations are stubs for v0.1
    Ok(PendingOp::None)
}

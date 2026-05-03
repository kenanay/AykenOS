pub mod cpu;
pub mod gpu;
pub mod ui;

use crate::error::{RuntimeError, RuntimeResult};
use crate::runtime::RuntimeState;
use crate::types::{BcibInstruction, Opcode, PendingOp};

/// Execute instruction and produce pending operation
/// Execute phase CANNOT mutate persistent state
pub fn execute(inst: BcibInstruction, state: &RuntimeState) -> RuntimeResult<PendingOp> {
    match inst.opcode {
        Opcode::CtxSelect
        | Opcode::DataCreate
        | Opcode::DataInsert
        | Opcode::DataQuery => cpu::execute_cpu(inst, state),

        Opcode::GpuBufferCreate | Opcode::GpuBufferBind => gpu::execute_gpu(inst, state),

        Opcode::UiSceneCreate | Opcode::UiRender => ui::execute_ui(inst, state),

        Opcode::SysHwStatus => Ok(PendingOp::None),

        Opcode::End => Ok(PendingOp::None),

        Opcode::Unknown(_) => Err(RuntimeError::UnknownOpcode),

        _ => Ok(PendingOp::None),
    }
}

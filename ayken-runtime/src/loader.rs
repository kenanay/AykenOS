use crate::error::RuntimeResult;
use crate::types::{BcibInstruction, Opcode};

pub fn load_demo_program() -> RuntimeResult<Vec<BcibInstruction>> {
    Ok(vec![
        // ctx 0
        BcibInstruction {
            opcode: Opcode::DataCreate,
            flags: 0,
            arg_start: 0,
            arg_count: 0,
        },
        BcibInstruction {
            opcode: Opcode::DataInsert,
            flags: 0,
            arg_start: 0,
            arg_count: 0,
        },
        BcibInstruction {
            opcode: Opcode::DataQuery,
            flags: 0,
            arg_start: 0,
            arg_count: 0,
        },
        // switch ctx 1
        BcibInstruction {
            opcode: Opcode::CtxSelect,
            flags: 0,
            arg_start: 1,
            arg_count: 0,
        },
        BcibInstruction {
            opcode: Opcode::DataCreate,
            flags: 0,
            arg_start: 0,
            arg_count: 0,
        },
        BcibInstruction {
            opcode: Opcode::DataInsert,
            flags: 0,
            arg_start: 0,
            arg_count: 0,
        },
        BcibInstruction {
            opcode: Opcode::DataQuery,
            flags: 0,
            arg_start: 0,
            arg_count: 0,
        },
        // render reads runtime result buffers
        BcibInstruction {
            opcode: Opcode::UiRender,
            flags: 0,
            arg_start: 0,
            arg_count: 0,
        },
        BcibInstruction {
            opcode: Opcode::End,
            flags: 0,
            arg_start: 0,
            arg_count: 0,
        },
    ])
}

/// Rollback integrity test: insert without collection should fail
pub fn load_rollback_test_program() -> RuntimeResult<Vec<BcibInstruction>> {
    Ok(vec![
        BcibInstruction {
            opcode: Opcode::DataInsert, // collection doesn't exist, should fail
            flags: 0,
            arg_start: 0,
            arg_count: 0,
        },
    ])
}

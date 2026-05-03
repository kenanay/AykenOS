use std::collections::BTreeMap;

use crate::error::RuntimeResult;
use crate::runtime::RuntimeState;
use crate::types::{BcibInstruction, Opcode, PendingOp, Row};

/// CPU executor for data operations
/// Execute phase ONLY produces PendingOp, does NOT mutate state
pub fn execute_cpu(
    inst: BcibInstruction,
    state: &RuntimeState,
) -> RuntimeResult<PendingOp> {
    let ctx_id = state.current_ctx;

    match inst.opcode {
        Opcode::CtxSelect => {
            // v0.2 mock arg rule:
            // arg_start == target context id
            Ok(PendingOp::SelectContext {
                context_id: inst.arg_start as u64,
            })
        }

        Opcode::DataCreate => Ok(PendingOp::CreateCollection {
            context_id: ctx_id,
            name: "users".to_string(),
        }),

        Opcode::DataInsert => {
            let mut fields = BTreeMap::new();

            fields.insert("id".to_string(), format!("{}", ctx_id + 1));
            fields.insert("name".to_string(), format!("user_ctx_{}", ctx_id));

            Ok(PendingOp::InsertRow {
                context_id: ctx_id,
                collection: "users".to_string(),
                row: Row { fields },
            })
        }

        Opcode::DataQuery => Ok(PendingOp::QueryCollection {
            context_id: ctx_id,
            collection: "users".to_string(),
        }),

        _ => Ok(PendingOp::None),
    }
}

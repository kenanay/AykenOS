use crate::error::{RuntimeError, RuntimeResult};
use crate::runtime::RuntimeState;
use crate::types::{
    Collection, ContextStatus, DeltaOp, JournalEntry, PendingOp, QueryMetadata, ResultBuffer,
};

/// v0.3: Journal-based atomic commit
/// No snapshot clone - uses delta operations with rollback
pub fn commit(op: PendingOp, state: &mut RuntimeState) -> RuntimeResult<()> {
    // Skip journal for no-op operations
    if matches!(op, PendingOp::None | PendingOp::RenderUi { .. }) {
        return Ok(());
    }

    let delta = match build_delta(op, state) {
        Ok(d) => d,
        Err(e) => return Err(state.fail_closed(e)),
    };

    match apply_delta(&delta, state) {
        Ok(()) => {
            let entry = JournalEntry {
                journal_id: state.next_journal_id(),
                pc: state.pc,
                delta,
            };

            state.journal.push(entry);
            Ok(())
        }

        Err(e) => {
            rollback_delta(&delta, state);
            Err(state.fail_closed(e))
        }
    }
}

/// Public API for replay - applies delta without rollback
pub fn apply_delta_for_replay(delta: &DeltaOp, state: &mut RuntimeState) -> RuntimeResult<()> {
    apply_delta(delta, state)
}

/// Build delta operation from pending operation
fn build_delta(op: PendingOp, state: &mut RuntimeState) -> RuntimeResult<DeltaOp> {
    match op {
        PendingOp::None | PendingOp::RenderUi { .. } => Err(RuntimeError::CommitError),

        PendingOp::SelectContext { context_id } => Ok(DeltaOp::SelectContext {
            previous: state.current_ctx,
            next: context_id,
        }),

        PendingOp::CreateCollection { context_id, name } => {
            Ok(DeltaOp::CreateCollection { context_id, name })
        }

        PendingOp::InsertRow {
            context_id,
            collection,
            row,
        } => {
            let ctx = state
                .contexts
                .get(&context_id)
                .ok_or(RuntimeError::ContextError)?;

            let target = ctx
                .data_store
                .collections
                .get(&collection)
                .ok_or(RuntimeError::CommitError)?;

            Ok(DeltaOp::InsertRow {
                context_id,
                collection,
                row_index: target.rows.len(),
                row,
            })
        }

        PendingOp::QueryCollection {
            context_id,
            collection,
        } => {
            let buffer_id = state.allocate_result_buffer_id();

            let ctx = state
                .contexts
                .get(&context_id)
                .ok_or(RuntimeError::ContextError)?;

            let target = ctx
                .data_store
                .collections
                .get(&collection)
                .ok_or(RuntimeError::CommitError)?;

            let row_indices = (0..target.rows.len()).collect::<Vec<_>>();

            let result_buffer = ResultBuffer {
                buffer_id,
                metadata: QueryMetadata {
                    source_context: context_id,
                    source_collection: collection,
                    row_count: row_indices.len(),
                },
                row_indices,
                source_version: target.version,  // v0.5: capture collection version
            };

            Ok(DeltaOp::CreateResultBuffer {
                buffer: result_buffer,
            })
        }
    }
}

/// Apply delta operation to state
fn apply_delta(delta: &DeltaOp, state: &mut RuntimeState) -> RuntimeResult<()> {
    match delta {
        DeltaOp::SelectContext { next, .. } => {
            state.ensure_context(*next);
            state.current_ctx = *next;
            Ok(())
        }

        DeltaOp::CreateCollection { context_id, name } => {
            let ctx = state
                .contexts
                .get_mut(context_id)
                .ok_or(RuntimeError::ContextError)?;

            if ctx.status == ContextStatus::Failed {
                return Err(RuntimeError::ContextError);
            }

            if ctx.data_store.collections.contains_key(name) {
                return Err(RuntimeError::CommitError);
            }

            ctx.data_store.collections.insert(
                name.clone(),
                Collection {
                    name: name.clone(),
                    rows: Vec::new(),
                    version: 1,  // v0.5: initial version
                },
            );

            Ok(())
        }

        DeltaOp::InsertRow {
            context_id,
            collection,
            row_index,
            row,
        } => {
            let ctx = state
                .contexts
                .get_mut(context_id)
                .ok_or(RuntimeError::ContextError)?;

            let target = ctx
                .data_store
                .collections
                .get_mut(collection)
                .ok_or(RuntimeError::CommitError)?;

            if target.rows.len() != *row_index {
                return Err(RuntimeError::CommitError);
            }

            target.rows.push(row.clone());
            target.version += 1;  // v0.5: increment version on mutation

            Ok(())
        }

        DeltaOp::CreateResultBuffer { buffer } => {
            if state.result_buffers.contains_key(&buffer.buffer_id) {
                return Err(RuntimeError::CommitError);
            }

            state
                .result_buffers
                .insert(buffer.buffer_id, buffer.clone());

            Ok(())
        }
    }
}

/// Rollback delta operation (inverse operation)
fn rollback_delta(delta: &DeltaOp, state: &mut RuntimeState) {
    match delta {
        DeltaOp::SelectContext { previous, .. } => {
            state.current_ctx = *previous;
        }

        DeltaOp::CreateCollection { context_id, name } => {
            if let Some(ctx) = state.contexts.get_mut(context_id) {
                ctx.data_store.collections.remove(name);
            }
        }

        DeltaOp::InsertRow {
            context_id,
            collection,
            row_index,
            ..
        } => {
            if let Some(ctx) = state.contexts.get_mut(context_id) {
                if let Some(target) = ctx.data_store.collections.get_mut(collection) {
                    if target.rows.len() == row_index + 1 {
                        target.rows.pop();
                    }
                }
            }
        }

        DeltaOp::CreateResultBuffer { buffer } => {
            state.result_buffers.remove(&buffer.buffer_id);
        }
    }
}

use crate::commit::apply_delta_for_replay;
use crate::error::RuntimeResult;
use crate::runtime::RuntimeState;
use crate::types::JournalEntry;

/// Replay journal from empty state to reconstruct runtime state
/// This proves journal is the source of truth
pub fn replay_journal(journal: &[JournalEntry]) -> RuntimeResult<RuntimeState> {
    let mut state = RuntimeState::new();

    for entry in journal {
        apply_delta_for_replay(&entry.delta, &mut state)?;
        state.journal.push(entry.clone());
        state.pc = entry.pc + 1;
    }

    state.reconcile_replay_allocators()?;
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::replay_journal;
    use crate::commit::commit;
    use crate::runtime::RuntimeState;
    use crate::types::PendingOp;

    #[test]
    fn replay_advances_allocators_before_new_commit() {
        let mut original = RuntimeState::new();

        commit(
            PendingOp::CreateCollection {
                context_id: 0,
                name: "users".to_string(),
            },
            &mut original,
        )
        .unwrap();
        commit(
            PendingOp::QueryCollection {
                context_id: 0,
                collection: "users".to_string(),
            },
            &mut original,
        )
        .unwrap();

        let mut replayed = replay_journal(&original.journal).unwrap();
        commit(
            PendingOp::QueryCollection {
                context_id: 0,
                collection: "users".to_string(),
            },
            &mut replayed,
        )
        .unwrap();

        assert!(replayed.result_buffers.contains_key(&1));
        assert!(replayed.result_buffers.contains_key(&2));
        assert_eq!(replayed.journal.last().unwrap().journal_id, 3);
    }
}

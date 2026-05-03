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

    Ok(state)
}

use crate::runtime::{Hash32, RuntimeState};
use crate::types::JournalEntry;

/// v0.5: Checkpoint for journal compaction
/// Checkpoint = compacted state snapshot (without journal)
#[derive(Debug, Clone)]
pub struct Checkpoint {
    pub checkpoint_id: u64,
    pub pc: usize,
    pub state_hash: Hash32,
    pub state_snapshot: RuntimeState,  // Full state snapshot
}

impl RuntimeState {
    /// Create checkpoint and compact journal
    pub fn checkpoint(&mut self) -> Checkpoint {
        let checkpoint_id = self.next_checkpoint_id();
        let state_hash = self.canonical_state_hash_v2();

        // Clone current state (without journal for compaction)
        let mut state_snapshot = self.clone();
        state_snapshot.journal.clear();  // Don't include journal in checkpoint
        state_snapshot.trace.clear();  // Don't include trace in checkpoint

        let checkpoint = Checkpoint {
            checkpoint_id,
            pc: self.pc,
            state_hash,
            state_snapshot,
        };

        // Compact: clear journal after checkpoint
        self.journal.clear();

        checkpoint
    }

    fn next_checkpoint_id(&mut self) -> u64 {
        let id = self.next_checkpoint_id_counter;
        self.next_checkpoint_id_counter += 1;
        id
    }
}

/// Replay from checkpoint + tail journal
pub fn replay_from_checkpoint(
    checkpoint: &Checkpoint,
    tail_journal: &[JournalEntry],
) -> crate::error::RuntimeResult<RuntimeState> {
    // Start from checkpoint state
    let mut state = checkpoint.state_snapshot.clone();

    // Replay tail journal
    for entry in tail_journal {
        crate::commit::apply_delta_for_replay(&entry.delta, &mut state)?;
        state.journal.push(entry.clone());
        state.pc = entry.pc + 1;
    }

    Ok(state)
}


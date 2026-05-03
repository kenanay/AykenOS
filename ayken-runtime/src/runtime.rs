use std::collections::BTreeMap;

use crate::error::{RuntimeError, RuntimeResult};
use crate::types::{
    BcibInstruction, ContextId, ContextState, ContextStatus, DataStore, JournalEntry, JournalId,
    ResultBuffer, ResultBufferId,
};

// v0.4: BLAKE3 canonical hash
pub type Hash32 = [u8; 32];

pub fn blake3_hash(bytes: &[u8]) -> Hash32 {
    *blake3::hash(bytes).as_bytes()
}

pub fn hash_hex(hash: &Hash32) -> String {
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Ayken Runtime implementing fetch → decode → validate → execute → commit
/// Based on BCIB Execution Semantics v0.1
#[derive(Clone, Debug)]
pub struct RuntimeState {
    pub pc: usize,
    pub running: bool,
    pub current_ctx: ContextId,
    pub contexts: BTreeMap<ContextId, ContextState>,
    pub result_buffers: BTreeMap<ResultBufferId, ResultBuffer>,
    pub next_result_buffer_id: ResultBufferId,

    // v0.3: journal-based commit
    pub journal: Vec<JournalEntry>,
    pub next_journal_id: JournalId,

    // v0.5: checkpoint support
    pub next_checkpoint_id_counter: u64,

    pub trace: Vec<String>,
}

impl RuntimeState {
    pub fn new() -> Self {
        let mut contexts = BTreeMap::new();

        contexts.insert(0, Self::new_context(0));

        Self {
            pc: 0,
            running: true,
            current_ctx: 0,
            contexts,
            result_buffers: BTreeMap::new(),
            next_result_buffer_id: 1,
            journal: Vec::new(),
            next_journal_id: 1,
            next_checkpoint_id_counter: 1,
            trace: Vec::new(),
        }
    }

    pub fn new_context(id: ContextId) -> ContextState {
        ContextState {
            context_id: id,
            status: ContextStatus::Ready,
            data_store: DataStore::default(),
            last_hash: 0,
        }
    }

    pub fn ensure_context(&mut self, id: ContextId) {
        self.contexts
            .entry(id)
            .or_insert_with(|| Self::new_context(id));
    }

    pub fn current_context(&self) -> RuntimeResult<&ContextState> {
        self.contexts
            .get(&self.current_ctx)
            .ok_or(RuntimeError::ContextError)
    }

    pub fn current_context_mut(&mut self) -> RuntimeResult<&mut ContextState> {
        self.contexts
            .get_mut(&self.current_ctx)
            .ok_or(RuntimeError::ContextError)
    }

    pub fn allocate_result_buffer_id(&mut self) -> ResultBufferId {
        let id = self.next_result_buffer_id;
        self.next_result_buffer_id += 1;
        id
    }

    pub fn next_journal_id(&mut self) -> JournalId {
        let id = self.next_journal_id;
        self.next_journal_id += 1;
        id
    }

    pub fn fail_closed(&mut self, err: RuntimeError) -> RuntimeError {
        self.running = false;

        if let Some(ctx) = self.contexts.get_mut(&self.current_ctx) {
            ctx.status = ContextStatus::Failed;
        }

        err
    }

    /// Add trace entry with deterministic state hash
    pub fn add_trace(&mut self, inst: &BcibInstruction) {
        let state_hash = self.canonical_state_hash_v2();
        let result_hash = self.canonical_result_hash_v2();

        let collection_count = self
            .contexts
            .get(&self.current_ctx)
            .map(|ctx| ctx.data_store.collections.len())
            .unwrap_or(0);

        let row_count = self
            .contexts
            .get(&self.current_ctx)
            .map(|ctx| {
                ctx.data_store
                    .collections
                    .values()
                    .map(|c| c.rows.len())
                    .sum::<usize>()
            })
            .unwrap_or(0);

        self.trace.push(format!(
            "[{}] {:?} | ctx={} | collections={} | rows={} | results={} | journal={} | state={} | result={}",
            self.pc, inst.opcode, self.current_ctx, collection_count, row_count, 
            self.result_buffers.len(), self.journal.len(), 
            hash_hex(&state_hash), hash_hex(&result_hash)
        ));
    }

    /// v0.4: Canonical binary encoding + BLAKE3 hash
    pub fn canonical_state_hash_v2(&self) -> Hash32 {
        blake3_hash(&encode_state_canonical(self))
    }

    /// v0.4: Canonical result buffer hash with BLAKE3
    pub fn canonical_result_hash_v2(&self) -> Hash32 {
        blake3_hash(&encode_result_buffers_canonical(self))
    }

    /// Print deterministic trace
    pub fn print_trace(&self) {
        println!("\n=== Deterministic Trace ===");
        for entry in &self.trace {
            println!("{}", entry);
        }
        println!("===========================\n");
    }
}

/// v0.4: Canonical binary state encoding
pub fn encode_state_canonical(state: &RuntimeState) -> Vec<u8> {
    let mut out = Vec::new();

    out.extend_from_slice(b"AYKEN_STATE_V1");

    for (ctx_id, ctx) in &state.contexts {
        out.extend_from_slice(&ctx_id.to_le_bytes());

        out.push(match ctx.status {
            ContextStatus::Created => 0,
            ContextStatus::Ready => 1,
            ContextStatus::Running => 2,
            ContextStatus::Committed => 3,
            ContextStatus::Failed => 4,
            ContextStatus::Closed => 5,
        });

        for (collection_name, collection) in &ctx.data_store.collections {
            write_str(&mut out, collection_name);

            out.extend_from_slice(&(collection.rows.len() as u64).to_le_bytes());

            for row in &collection.rows {
                out.extend_from_slice(&(row.fields.len() as u64).to_le_bytes());

                for (k, v) in &row.fields {
                    write_str(&mut out, k);
                    write_str(&mut out, v);
                }
            }
        }
    }

    out
}

/// v0.4: Canonical binary result buffer encoding
pub fn encode_result_buffers_canonical(state: &RuntimeState) -> Vec<u8> {
    let mut out = Vec::new();

    out.extend_from_slice(b"AYKEN_RESULT_V1");

    for (id, buffer) in &state.result_buffers {
        out.extend_from_slice(&id.to_le_bytes());
        out.extend_from_slice(&buffer.metadata.source_context.to_le_bytes());
        write_str(&mut out, &buffer.metadata.source_collection);
        out.extend_from_slice(&(buffer.metadata.row_count as u64).to_le_bytes());

        out.extend_from_slice(&(buffer.row_indices.len() as u64).to_le_bytes());
        for idx in &buffer.row_indices {
            out.extend_from_slice(&(*idx as u64).to_le_bytes());
        }
    }

    out
}

fn write_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    out.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    out.extend_from_slice(bytes);
}

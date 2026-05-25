use std::collections::BTreeMap;

/// Opcode definitions from BCIB Opcode Set v0.1
/// Phase 1 minimum required opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    CtxSelect,
    DataCreate,
    DataInsert,
    DataQuery,
    AbdfValidate,
    AbdfRead,
    GpuBufferCreate,
    GpuBufferBind,
    UiSceneCreate,
    UiRender,
    SysHwStatus,
    End,
    Unknown(u16),
}

impl From<u16> for Opcode {
    fn from(value: u16) -> Self {
        match value {
            0x0001 => Opcode::CtxSelect,
            0x0100 => Opcode::DataCreate,
            0x0101 => Opcode::DataInsert,
            0x0102 => Opcode::DataQuery,
            0x0201 => Opcode::AbdfValidate,
            0x0203 => Opcode::AbdfRead,
            0x0300 => Opcode::GpuBufferCreate,
            0x0301 => Opcode::GpuBufferBind,
            0x0400 => Opcode::UiSceneCreate,
            0x0403 => Opcode::UiRender,
            0x0600 => Opcode::SysHwStatus,
            0xFFFF => Opcode::End,
            _ => Opcode::Unknown(value),
        }
    }
}

/// BCIB Instruction structure
#[derive(Debug, Clone, Copy)]
pub struct BcibInstruction {
    pub opcode: Opcode,
    pub flags: u16,
    pub arg_start: u32,
    pub arg_count: u32,
}

// ============================================================================
// Real State Types
// ============================================================================

pub type ContextId = u64;
pub type CollectionName = String;
pub type ResultBufferId = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextStatus {
    Created,
    Ready,
    Running,
    Committed,
    Failed,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub fields: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default)]
pub struct DataStore {
    pub collections: BTreeMap<CollectionName, Collection>,
}

#[derive(Debug, Clone)]
pub struct ContextState {
    pub context_id: ContextId,
    pub status: ContextStatus,
    pub data_store: DataStore,
    pub last_hash: u64,
}

// ============================================================================
// Query Result Buffer (v0.3: zero-copy view)
// ============================================================================

#[derive(Debug, Clone)]
pub struct QueryMetadata {
    pub source_context: ContextId,
    pub source_collection: CollectionName,
    pub row_count: usize,
}

#[derive(Debug, Clone)]
pub struct ResultBuffer {
    pub buffer_id: ResultBufferId,
    pub metadata: QueryMetadata,
    pub row_indices: Vec<usize>,  // zero-copy: indices instead of cloned rows
    pub source_version: u64,  // v0.5: collection version at query time
}

// ============================================================================
// Collection with versioning (v0.5)
// ============================================================================

#[derive(Debug, Clone)]
pub struct Collection {
    pub name: CollectionName,
    pub rows: Vec<Row>,
    pub version: u64,  // Incremented on mutation
}

// ============================================================================
// Journal-Based Commit (v0.3)
// ============================================================================

pub type JournalId = u64;

#[derive(Debug, Clone)]
pub enum DeltaOp {
    SelectContext {
        previous: ContextId,
        next: ContextId,
    },

    CreateCollection {
        context_id: ContextId,
        name: CollectionName,
    },

    InsertRow {
        context_id: ContextId,
        collection: CollectionName,
        row_index: usize,
        row: Row,
    },

    CreateResultBuffer {
        buffer: ResultBuffer,
    },
}

#[derive(Debug, Clone)]
pub struct JournalEntry {
    pub journal_id: JournalId,
    pub pc: usize,
    pub delta: DeltaOp,
}

// ============================================================================
// PendingOp - Execute produces operations, Commit applies them
// ============================================================================

#[derive(Debug, Clone)]
pub enum PendingOp {
    None,

    SelectContext {
        context_id: ContextId,
    },

    CreateCollection {
        context_id: ContextId,
        name: CollectionName,
    },

    InsertRow {
        context_id: ContextId,
        collection: CollectionName,
        row: Row,
    },

    QueryCollection {
        context_id: ContextId,
        collection: CollectionName,
    },

    RenderUi {
        context_id: ContextId,
    },
}

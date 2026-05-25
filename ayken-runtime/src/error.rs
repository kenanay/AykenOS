/// Runtime error types aligned with BCIB Execution Semantics v0.1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeError {
    InvalidHeader,
    InvalidPc,
    DecodeError,
    ValidationError,
    CapabilityError,
    ContextError,
    MemoryError,
    GpuFault,
    UiFault,
    CommitError,
    UnknownOpcode,
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

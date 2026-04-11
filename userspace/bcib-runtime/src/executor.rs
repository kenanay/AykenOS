#[cfg(all(target_arch = "x86_64", not(test), not(feature = "test-support")))]
use core::arch::asm;
use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use crate::{
    types::{BcibError, ResourceLimits},
    verifier_planner::BcibVerifierPlanner,
};
use bcib::{BcibBuffer, DecodeError};

/// Syscall base offset for v2 interface (1000-1009 range).
const SYS_V2_BASE: u64 = 1000;
/// Syscall numbers aligned with kernel/sys/syscall_v2.h
const SYS_V2_SUBMIT_EXECUTION: u64 = SYS_V2_BASE + 3;
const SYS_V2_WAIT_RESULT: u64 = SYS_V2_BASE + 4;

/// Capability resource kinds supported by the executor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityResource {
    Execution,
}

/// Simple capability permissions bitmask.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityPermission(pub u64);
impl CapabilityPermission {
    pub const EXECUTE: Self = CapabilityPermission(1 << 0);
}

/// Capability token bound to an execution submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityToken {
    pub id: u64,
    pub resource_type: CapabilityResource,
    pub permissions: CapabilityPermission,
    pub expires_at: Option<u64>,
}

/// Minimal capability manager placeholder for Phase 2.3 Ring3 runtime.
#[derive(Debug, Default)]
pub struct CapabilityManager {
    active_tokens: HashSet<u64>,
}

impl CapabilityManager {
    pub fn bind(&mut self, token: &CapabilityToken) -> Result<(), ExecutionError> {
        self.active_tokens.insert(token.id);
        Ok(())
    }

    pub fn revoke(&mut self, token_id: u64) {
        self.active_tokens.remove(&token_id);
    }
}

/// Execution context state tracked per submission.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub id: u64,
    pub active_container: Option<String>,
    pub string_pool: Vec<String>,
    pub logger_enabled: bool,
}

impl ExecutionContext {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            active_container: None,
            string_pool: Vec::new(),
            logger_enabled: true,
        }
    }
}

/// BCIB graph wrapper used for syscall submission.
#[derive(Debug, Clone, Copy)]
pub struct BcibGraph<'a> {
    data: &'a [u8],
}

impl<'a> BcibGraph<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Validate BCIB header/opcode set before submitting to Ring0.
    pub fn validate(&self) -> Result<(), ExecutionError> {
        match validate_v3_graph(self.data) {
            Ok(()) => Ok(()),
            Err(v3_error) => match BcibBuffer::decode(self.data) {
                Ok(_) => Ok(()),
                Err(v02_error) => {
                    if looks_like_v3_graph(self.data) {
                        Err(ExecutionError::RuntimeValidation(v3_error))
                    } else {
                        Err(ExecutionError::from(v02_error))
                    }
                }
            },
        }
    }
}

fn validate_v3_graph(data: &[u8]) -> Result<(), BcibError> {
    let planner = BcibVerifierPlanner::new();

    // BcibExecutor owns graph submittability only. Semantic capability
    // validation is enforced upstream by the submit-only router.
    planner.verify_submittable_graph(data, &ResourceLimits::default())
}

fn looks_like_v3_graph(data: &[u8]) -> bool {
    data.len() >= 6 && &data[0..4] == b"BCIB" && u16::from_le_bytes([data[4], data[5]]) == 3
}

/// Errors raised during BCIB execution submission.
#[derive(Debug)]
pub enum ExecutionError {
    InvalidGraph(&'static str),
    InvalidContext(&'static str),
    Decode(DecodeError),
    RuntimeValidation(BcibError),
    Syscall(i64),
    Capability(&'static str),
}

impl From<DecodeError> for ExecutionError {
    fn from(err: DecodeError) -> Self {
        ExecutionError::Decode(err)
    }
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::InvalidGraph(msg) => write!(f, "invalid BCIB graph: {}", msg),
            ExecutionError::InvalidContext(msg) => write!(f, "invalid execution context: {}", msg),
            ExecutionError::Decode(err) => write!(f, "BCIB decode failed: {}", err),
            ExecutionError::RuntimeValidation(err) => {
                write!(f, "BCIB runtime validation failed: {}", err)
            }
            ExecutionError::Syscall(code) => write!(f, "syscall returned error {}", code),
            ExecutionError::Capability(msg) => write!(f, "capability error: {}", msg),
        }
    }
}

impl std::error::Error for ExecutionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ExecutionError::Decode(err) => Some(err),
            ExecutionError::RuntimeValidation(err) => Some(err),
            _ => None,
        }
    }
}

/// Ring3 BCIB executor responsible for validating and submitting graphs.
pub struct BcibExecutor {
    pub execution_contexts: HashMap<u64, ExecutionContext>,
    pub capability_manager: CapabilityManager,
}

impl BcibExecutor {
    pub fn new() -> Self {
        Self {
            execution_contexts: HashMap::new(),
            capability_manager: CapabilityManager::default(),
        }
    }

    /// Submit BCIB graph to Ring0 via SYS_V2_SUBMIT_EXECUTION.
    ///
    /// Validates BCIB header/opcodes, requires an explicit target `context_id`,
    /// and forwards the buffer to Ring0 using the execution-centric syscall
    /// path. The returned `execution_id` remains authoritative and kernel-owned.
    pub fn submit_execution(
        &mut self,
        graph: &BcibGraph,
        context_id: u64,
    ) -> Result<u64, ExecutionError> {
        if graph.is_empty() {
            return Err(ExecutionError::InvalidGraph("BCIB graph is empty"));
        }
        if context_id == 0 {
            return Err(ExecutionError::InvalidContext(
                "context_id must be non-zero",
            ));
        }

        // Validate BCIB structure (magic, version, opcode set)
        graph.validate()?;

        self.execution_contexts
            .entry(context_id)
            .or_insert_with(|| ExecutionContext::new(context_id));

        // Submit to Ring0 with a real target context ID.
        let result = unsafe {
            syscall_v2(
                SYS_V2_SUBMIT_EXECUTION,
                graph.as_ptr() as u64,
                graph.len() as u64,
                context_id,
                0,
            )
        };

        if (result as i64) < 0 {
            return Err(ExecutionError::Syscall(result as i64));
        }
        if result == 0 {
            return Err(ExecutionError::Syscall(0));
        }

        let token = CapabilityToken {
            id: result,
            resource_type: CapabilityResource::Execution,
            permissions: CapabilityPermission::EXECUTE,
            expires_at: None,
        };

        self.capability_manager
            .bind(&token)
            .map_err(|_| ExecutionError::Capability("capability bind failed"))?;

        Ok(result)
    }

    /// Wait for execution result via SYS_V2_WAIT_RESULT (placeholder wiring).
    pub fn wait_result(&self, execution_id: u64, timeout_ms: u64) -> Result<u64, ExecutionError> {
        let result = unsafe { syscall_v2(SYS_V2_WAIT_RESULT, execution_id, timeout_ms, 0, 0) };
        if (result as i64) < 0 {
            return Err(ExecutionError::Syscall(result as i64));
        }
        Ok(result)
    }
}

/// Low-level syscall shim using INT 0x80 (aligns with existing Ring3 callers).
#[cfg(all(target_arch = "x86_64", not(test), not(feature = "test-support")))]
#[inline(always)]
unsafe fn syscall_v2(num: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
    let ret: u64;
    asm!(
        "int 0x80",
        in("rax") num,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("r10") arg4,
        lateout("rax") ret,
        options(nostack, preserves_flags)
    );
    ret
}

/// Fallback syscall implementation for non-x86_64 architectures
#[cfg(all(not(target_arch = "x86_64"), not(test), not(feature = "test-support")))]
#[inline(always)]
unsafe fn syscall_v2(_num: u64, _arg1: u64, _arg2: u64, _arg3: u64, _arg4: u64) -> u64 {
    // For ARM macOS and other architectures, return a mock success value
    // This is acceptable for Phase 4.2 since we're focusing on measurement infrastructure
    0
}

#[cfg(any(test, feature = "test-support"))]
pub mod test_support {
    use std::sync::{Mutex, OnceLock};

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct RecordedSyscall {
        pub num: u64,
        pub arg1: u64,
        pub arg2: u64,
        pub arg3: u64,
        pub arg4: u64,
    }

    #[derive(Debug, Default)]
    struct TestState {
        enabled: bool,
        return_value: u64,
        last_call: Option<RecordedSyscall>,
    }

    fn state() -> &'static Mutex<TestState> {
        static STATE: OnceLock<Mutex<TestState>> = OnceLock::new();
        STATE.get_or_init(|| Mutex::new(TestState::default()))
    }

    pub fn install(return_value: u64) {
        let mut guard = state().lock().expect("test syscall state");
        guard.enabled = true;
        guard.return_value = return_value;
        guard.last_call = None;
    }

    pub fn take_last_call() -> Option<RecordedSyscall> {
        let mut guard = state().lock().expect("test syscall state");
        guard.last_call.take()
    }

    pub fn uninstall() {
        let mut guard = state().lock().expect("test syscall state");
        *guard = TestState::default();
    }

    pub fn invoke(num: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
        let mut guard = state().lock().expect("test syscall state");
        assert!(
            guard.enabled,
            "test syscall hook must be installed before submit_execution"
        );
        guard.last_call = Some(RecordedSyscall {
            num,
            arg1,
            arg2,
            arg3,
            arg4,
        });
        guard.return_value
    }
}

#[cfg(any(test, feature = "test-support"))]
#[inline(always)]
unsafe fn syscall_v2(num: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
    test_support::invoke(num, arg1, arg2, arg3, arg4)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bcib::BcibInstruction;
    use std::sync::{Mutex, OnceLock};

    struct TestSupportReset;

    impl Drop for TestSupportReset {
        fn drop(&mut self) {
            test_support::uninstall();
        }
    }

    fn test_syscall_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn sample_graph_bytes() -> Vec<u8> {
        let mut buf = BcibBuffer::new();
        buf.add(BcibInstruction::nop());
        buf.add(BcibInstruction::end());
        buf.encode()
    }

    fn sample_v3_graph_bytes() -> Vec<u8> {
        let instruction_bytes = [
            0x12, 0x02, 1, 0, 0, 0, 0, 0, 0, 0, // DataQuery [1, 0]
            0x01, 0x00, // End
        ];
        let instruction_offset = 24u32;
        let instruction_length = instruction_bytes.len() as u16;

        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"BCIB");
        bytes.extend_from_slice(&3u16.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&[0u8; 4]);
        bytes.extend_from_slice(&[0u8; 2]);
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&instruction_offset.to_le_bytes());
        bytes.extend_from_slice(&instruction_length.to_le_bytes());
        bytes.extend_from_slice(&instruction_bytes);
        bytes
    }

    #[test]
    fn test_submit_execution_empty_graph() {
        let mut executor = BcibExecutor::new();
        let empty_graph = BcibGraph::new(&[]);

        let result = executor.submit_execution(&empty_graph, 42);
        assert!(result.is_err());

        if let Err(ExecutionError::InvalidGraph(msg)) = result {
            assert_eq!(msg, "BCIB graph is empty");
        } else {
            panic!("Expected InvalidGraph error for empty graph");
        }
    }

    #[test]
    fn test_submit_execution_rejects_zero_context_id() {
        let mut executor = BcibExecutor::new();
        let graph_bytes = sample_graph_bytes();
        let graph = BcibGraph::new(&graph_bytes);

        let result = executor.submit_execution(&graph, 0);
        assert!(matches!(
            result,
            Err(ExecutionError::InvalidContext(
                "context_id must be non-zero"
            ))
        ));
    }

    #[test]
    fn test_bcib_graph_creation() {
        let test_data = b"test_bcib_data";
        let graph = BcibGraph::new(test_data);

        assert_eq!(graph.len(), test_data.len());
        assert!(!graph.is_empty());
        assert_eq!(graph.as_ptr(), test_data.as_ptr());
    }

    #[test]
    fn test_capability_manager() {
        let mut manager = CapabilityManager::default();
        let token = CapabilityToken {
            id: 123,
            resource_type: CapabilityResource::Execution,
            permissions: CapabilityPermission::EXECUTE,
            expires_at: None,
        };

        assert!(manager.bind(&token).is_ok());
        assert!(manager.active_tokens.contains(&123));

        manager.revoke(123);
        assert!(!manager.active_tokens.contains(&123));
    }

    #[test]
    fn test_execution_context_creation() {
        let ctx = ExecutionContext::new(42);

        assert_eq!(ctx.id, 42);
        assert!(ctx.active_container.is_none());
        assert!(ctx.string_pool.is_empty());
        assert!(ctx.logger_enabled);
    }

    #[test]
    fn test_submit_execution_passes_context_id_and_uses_kernel_execution_id() {
        let _lock = test_syscall_lock().lock().expect("test syscall lock");
        let _reset = TestSupportReset;
        let mut executor = BcibExecutor::new();
        let graph_bytes = sample_graph_bytes();
        let graph = BcibGraph::new(&graph_bytes);

        test_support::install(77);
        let result = executor.submit_execution(&graph, 42);
        let call = test_support::take_last_call();

        assert_eq!(result.unwrap(), 77);
        assert_eq!(
            call,
            Some(test_support::RecordedSyscall {
                num: SYS_V2_SUBMIT_EXECUTION,
                arg1: graph.as_ptr() as u64,
                arg2: graph.len() as u64,
                arg3: 42,
                arg4: 0,
            })
        );
        assert!(executor.execution_contexts.contains_key(&42));
        assert!(executor.capability_manager.active_tokens.contains(&77));
        assert!(!executor.capability_manager.active_tokens.contains(&42));
    }

    #[test]
    fn test_submit_execution_accepts_v3_graph() {
        let _lock = test_syscall_lock().lock().expect("test syscall lock");
        let _reset = TestSupportReset;
        let mut executor = BcibExecutor::new();
        let graph_bytes = sample_v3_graph_bytes();
        let graph = BcibGraph::new(&graph_bytes);

        test_support::install(88);
        let result = executor.submit_execution(&graph, 42);
        let call = test_support::take_last_call();

        assert_eq!(result.unwrap(), 88);
        assert_eq!(
            call,
            Some(test_support::RecordedSyscall {
                num: SYS_V2_SUBMIT_EXECUTION,
                arg1: graph.as_ptr() as u64,
                arg2: graph.len() as u64,
                arg3: 42,
                arg4: 0,
            })
        );
    }
}

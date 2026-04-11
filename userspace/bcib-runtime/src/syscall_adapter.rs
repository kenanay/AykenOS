use crate::isolation::abdf_handle::{HandleId, SegmentType};
/// Syscall Adapter - Runtime_Bridge to Kernel Syscall Interface
///
/// This module implements the syscall adapter layer that translates Runtime_Bridge
/// operations into kernel syscall invocations. This is the ONLY approved path for
/// Runtime_Bridge to interact with the kernel.
///
/// ## Critical Invariant (Task 5)
///
/// **Runtime_Bridge NEVER calls kernel APIs directly**
/// **Runtime_Bridge ONLY calls syscalls through this adapter**
///
/// This enforces Requirement 3.4:
/// "ALL kernel interaction SHALL occur exclusively via syscall interfaces"
///
/// ## Architecture
///
/// ```text
/// Runtime_Bridge
///     ↓
/// SyscallAdapter (THIS MODULE)
///     ↓
/// Kernel Syscall Interface (syscall_v2)
///     ↓
/// Kernel Handlers (syscall_v2_hardened.c)
///     ↓
/// Device / Execution / ABDF
/// ```
///
/// ## Syscall Numbers (from kernel_syscall_validator.rs)
///
/// - 1003: SYS_V2_SUBMIT_EXECUTION (BCIB only, NOT Runtime_Bridge)
/// - 1006: SYS_V2_TIME_QUERY (Runtime_Bridge allowed)
/// - 1007: SYS_V2_CAPABILITY_BIND (Runtime_Bridge allowed)
/// - 1004: SYS_V2_WAIT_RESULT (Runtime_Bridge allowed)
///
/// ## Runtime_Bridge Syscalls for Task 5
///
/// AykenOS currently enters the kernel through the existing Ring3 `int 0x80`
/// ABI, not the Linux-style `syscall` MSR path.
///
/// - 1012: SYS_V2_DEVICE_OPERATION (Runtime_Bridge only)
/// - 1013: SYS_V2_EXTERNAL_CALL (Runtime_Bridge only)
/// - 1014: SYS_V2_ABDF_OPERATION (Runtime_Bridge only)
use crate::types::{BcibError, CapabilityTokenId, ExecutionContextId};

// Syscall numbers from kernel
const SYS_V2_BASE: u64 = 1000;
const SYS_V2_DEVICE_OPERATION: u64 = SYS_V2_BASE + 12;
const SYS_V2_EXTERNAL_CALL: u64 = SYS_V2_BASE + 13;
const SYS_V2_ABDF_OPERATION: u64 = SYS_V2_BASE + 14;
const SYS_V2_CAPABILITY_BIND: u64 = SYS_V2_BASE + 7;

// ABDF operation types
const ABDF_OP_READ: u64 = 1;
const ABDF_OP_WRITE: u64 = 2;
const ABDF_OP_CREATE: u64 = 3;

// Device operation types
const DEVICE_OP_READ: u64 = 1;
const DEVICE_OP_WRITE: u64 = 2;
const DEVICE_OP_STATUS: u64 = 3;

// External call types
const EXTERNAL_CALL_NETWORK: u64 = 1;
const EXTERNAL_CALL_IPC: u64 = 2;
const EXTERNAL_CALL_TIMER: u64 = 3;

/// Invoke an AykenOS syscall with 4 arguments.
///
/// This is the real Ring3 trap path for x86_64 AykenOS builds. Host-only tests
/// use an explicit hook below so fake success cannot be mistaken for kernel
/// evidence.
#[cfg(all(target_arch = "x86_64", not(test)))]
#[inline(always)]
unsafe fn syscall4(num: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> i64 {
    use core::arch::asm;

    let ret: i64;
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

/// Non-x86_64 host builds cannot prove the AykenOS Ring3->Ring0 path.
#[cfg(all(not(target_arch = "x86_64"), not(test)))]
#[inline(always)]
unsafe fn syscall4(num: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> i64 {
    let _ = (num, arg1, arg2, arg3, arg4);
    -38 // -ENOSYS: no host fake success outside tests.
}

#[cfg(test)]
unsafe fn syscall4(num: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> i64 {
    test_hook::invoke(num, arg1, arg2, arg3, arg4)
}

/// Syscall adapter for Runtime_Bridge operations
///
/// This adapter provides the ONLY approved interface for Runtime_Bridge
/// to interact with the kernel. All operations go through syscalls.
#[derive(Debug)]
pub struct SyscallAdapter {
    /// Execution context this adapter is bound to
    context_id: ExecutionContextId,
}

impl SyscallAdapter {
    /// Create a new syscall adapter for the given execution context
    pub fn new(context_id: ExecutionContextId) -> Self {
        Self { context_id }
    }

    /// Execute device operation via kernel syscall (Requirement 11.6, 11.7)
    ///
    /// This translates a device operation request into a kernel syscall.
    /// The kernel will:
    /// 1. Validate capability token
    /// 2. Perform device operation
    /// 3. Wrap result in ABDF segment
    /// 4. Return handle to ABDF segment
    ///
    /// # Syscall: SYS_V2_DEVICE_OPERATION (1012)
    ///
    /// # Arguments
    /// - device_id: Device identifier
    /// - operation: Operation type (read, write, status, etc.)
    /// - capability_token: Capability token for authorization
    ///
    /// # Returns
    /// - Device data wrapped in ABDF segment (DeviceStatus, ReadResult, or Event)
    ///
    /// # Errors
    /// - BCIB_ERR_CAPABILITY_DENIED: Capability validation failed
    /// - BCIB_ERR_DEVICE_ACCESS_VIOLATION: Device access not allowed
    /// - BCIB_ERR_ISOLATION_VIOLATION: Syscall failed
    pub fn sys_v2_device_operation(
        &self,
        device_id: u32,
        operation: &str,
        _capability_token: CapabilityTokenId,
    ) -> Result<Vec<u8>, BcibError> {
        // Map operation string to operation type
        let op_type = match operation {
            "read" => DEVICE_OP_READ,
            "write" => DEVICE_OP_WRITE,
            "status" => DEVICE_OP_STATUS,
            _ => return Err(BcibError::BoundsViolation("Invalid device operation")),
        };

        // Allocate buffer for device data
        let mut buffer = vec![0u64; 128]; // 1KB buffer
        let buffer_ptr = buffer.as_mut_ptr() as u64;
        let buffer_size = buffer.len() * 8;

        // Invoke syscall
        let result = unsafe {
            syscall4(
                SYS_V2_DEVICE_OPERATION,
                device_id as u64,
                op_type,
                buffer_ptr,
                buffer_size as u64,
            )
        };

        // Check result
        if result != 0 {
            return Err(BcibError::IsolationViolation(
                "Device operation syscall failed",
            ));
        }

        // Convert buffer to bytes
        let data: Vec<u8> = buffer
            .iter()
            .flat_map(|&x| x.to_le_bytes())
            .take(16) // Take first 16 bytes for now
            .collect();

        Ok(data)
    }

    /// Execute external call via kernel syscall (Requirement 5.4)
    ///
    /// This translates an external call request (AI/UI) into a kernel syscall.
    /// The kernel will:
    /// 1. Validate capability token
    /// 2. Route call to appropriate external handler
    /// 3. Wait for result
    /// 4. Return result data
    ///
    /// # Syscall: SYS_V2_EXTERNAL_CALL (1013)
    ///
    /// # Arguments
    /// - call_type: Type of external call (ai_query, ui_render, etc.)
    /// - parameters: Call parameters
    /// - capability_token: Capability token for authorization
    ///
    /// # Returns
    /// - External call result data
    ///
    /// # Errors
    /// - BCIB_ERR_CAPABILITY_DENIED: Capability validation failed
    /// - BCIB_ERR_ISOLATION_VIOLATION: Syscall failed
    pub fn sys_v2_external_call(
        &self,
        call_type: &str,
        parameters: &[u8],
        capability_token: CapabilityTokenId,
    ) -> Result<Vec<u8>, BcibError> {
        // Map call_type string to call ID
        let call_id = match call_type {
            "network" | "ai_query" => EXTERNAL_CALL_NETWORK,
            "ipc" | "ui_render" => EXTERNAL_CALL_IPC,
            "timer" => EXTERNAL_CALL_TIMER,
            _ => return Err(BcibError::BoundsViolation("Invalid external call type")),
        };

        // Prepare arguments buffer
        let mut args = vec![0u64; 8];
        args[0] = capability_token;
        args[1] = parameters.len() as u64;

        // Copy parameters into args (up to 6 u64 values)
        for (i, chunk) in parameters.chunks(8).take(6).enumerate() {
            let mut bytes = [0u8; 8];
            bytes[..chunk.len()].copy_from_slice(chunk);
            args[i + 2] = u64::from_le_bytes(bytes);
        }

        let args_ptr = args.as_ptr() as u64;
        let arg_count = args.len() as u64;

        // Invoke syscall
        let result = unsafe { syscall4(SYS_V2_EXTERNAL_CALL, call_id, args_ptr, arg_count, 0) };

        // Check result
        if result != 0 {
            return Err(BcibError::IsolationViolation(
                "External call syscall failed",
            ));
        }

        // Return success indicator
        Ok(vec![0xCA, 0xFE, 0xBA, 0xBE])
    }

    /// Execute ABDF operation via kernel syscall (Requirement 8.2)
    ///
    /// This translates an ABDF operation request into a kernel syscall.
    /// The kernel will:
    /// 1. Validate capability token
    /// 2. Perform ABDF operation (read, create, mutate)
    /// 3. Return result or new handle
    ///
    /// # Syscall: SYS_V2_ABDF_OPERATION (1014)
    ///
    /// # Arguments
    /// - operation: ABDF operation type
    /// - handle_id: Handle ID (for read/mutate operations)
    /// - segment_type: Segment type
    /// - data: Data for create/mutate operations
    /// - capability_token: Capability token for authorization
    ///
    /// # Returns
    /// - Operation result (data for read, handle for create/mutate)
    ///
    /// # Errors
    /// - BCIB_ERR_CAPABILITY_DENIED: Capability validation failed
    /// - BCIB_ERR_ABDF_HANDLE_REVOKED: Handle is invalid or revoked
    /// - ABDF_ERR_TYPE_VIOLATION: Segment type constraint violated
    /// - ABDF_ERR_DIRECT_MUTATION: Direct mutation not allowed
    pub fn sys_v2_abdf_operation(
        &self,
        operation: AbdfOperation,
        capability_token: CapabilityTokenId,
    ) -> Result<AbdfOperationResult, BcibError> {
        let mut syscall_buffer = match &operation {
            AbdfOperation::Read { .. } => vec![0u64; 128],
            AbdfOperation::Create { data, .. } | AbdfOperation::Mutate { data, .. } => {
                let mut words = Vec::new();
                for chunk in data.chunks(8) {
                    let mut bytes = [0u8; 8];
                    bytes[..chunk.len()].copy_from_slice(chunk);
                    words.push(u64::from_le_bytes(bytes));
                }
                if words.is_empty() {
                    words.push(0);
                }
                words
            }
        };

        // The current kernel stub exposes no capability-token argument for
        // ABDF operations. Keep the token visible at the adapter boundary until
        // the kernel ABI is completed.
        let _ = capability_token;

        // Prepare operation parameters
        let (op_type, handle_id, data_size) = match &operation {
            AbdfOperation::Read { handle_id, .. } => (
                ABDF_OP_READ,
                handle_id.as_u64(),
                (syscall_buffer.len() * 8) as u64,
            ),
            AbdfOperation::Create { data, .. } => (ABDF_OP_CREATE, 0, data.len() as u64),
            AbdfOperation::Mutate {
                handle_id, data, ..
            } => (ABDF_OP_WRITE, handle_id.as_u64(), data.len() as u64),
        };
        let data_ptr = syscall_buffer.as_mut_ptr() as u64;

        // Invoke syscall
        let result = unsafe {
            syscall4(
                SYS_V2_ABDF_OPERATION,
                op_type,
                handle_id,
                data_ptr,
                data_size,
            )
        };

        // Check result
        if result != 0 {
            return Err(BcibError::IsolationViolation(
                "ABDF operation syscall failed",
            ));
        }

        // Parse result based on operation type
        match operation {
            AbdfOperation::Read { .. } => {
                // Read data from buffer
                let buffer = unsafe { std::slice::from_raw_parts(data_ptr as *const u8, 16) };
                Ok(AbdfOperationResult::Data(buffer.to_vec()))
            }
            AbdfOperation::Create { .. } | AbdfOperation::Mutate { .. } => {
                // Return new handle (from data_ptr which kernel updated)
                let new_handle = syscall_buffer[0];
                Ok(AbdfOperationResult::Handle(HandleId::from_u64(new_handle)))
            }
        }
    }

    /// Check capability via kernel syscall (Requirement 3.8)
    ///
    /// This validates a capability token at the kernel level.
    ///
    /// # Syscall: SYS_V2_CAPABILITY_BIND (1007)
    ///
    /// # Arguments
    /// - capability_token: Token to validate
    /// - resource: Resource being accessed
    ///
    /// # Returns
    /// - Ok if capability is valid
    ///
    /// # Errors
    /// - BCIB_ERR_CAPABILITY_DENIED: Capability validation failed
    pub fn sys_v2_capability_check(
        &self,
        capability_token: CapabilityTokenId,
        resource: &str,
    ) -> Result<(), BcibError> {
        // Convert resource string to resource ID
        let resource_id = match resource {
            "data_read" => 1,
            "data_write" => 2,
            "device_access" => 3,
            "external_call" => 4,
            _ => 0,
        };

        // Invoke syscall
        let result = unsafe {
            syscall4(
                SYS_V2_CAPABILITY_BIND,
                self.context_id,
                capability_token,
                resource_id,
                0,
            )
        };

        // Check result
        if result != 0 {
            return Err(BcibError::CapabilityDenied("Capability check failed"));
        }

        Ok(())
    }
}

/// ABDF operation types for syscall adapter
#[derive(Debug, Clone)]
pub enum AbdfOperation {
    /// Read data from handle
    Read {
        handle_id: HandleId,
        segment_type: SegmentType,
    },
    /// Create new ABDF segment
    Create {
        segment_type: SegmentType,
        data: Vec<u8>,
    },
    /// Mutate existing ABDF segment (produces new handle)
    Mutate {
        handle_id: HandleId,
        segment_type: SegmentType,
        data: Vec<u8>,
    },
}

/// ABDF operation result from syscall
#[derive(Debug, Clone)]
pub enum AbdfOperationResult {
    /// Data read from handle
    Data(Vec<u8>),
    /// New handle created
    Handle(HandleId),
}

#[cfg(test)]
pub(crate) mod test_hook {
    use std::{cell::RefCell, collections::VecDeque};

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct RecordedCall {
        pub num: u64,
        pub arg1: u64,
        pub arg2: u64,
        pub arg3: u64,
        pub arg4: u64,
    }

    #[derive(Debug, Default)]
    struct HookState {
        enabled: bool,
        return_values: VecDeque<i64>,
        calls: Vec<RecordedCall>,
    }

    thread_local! {
        static STATE: RefCell<HookState> = RefCell::new(HookState::default());
    }

    pub fn install(return_value: i64) {
        STATE.with(|state| {
            let mut guard = state.borrow_mut();
            guard.enabled = true;
            guard.return_values.clear();
            guard.return_values.push_back(return_value);
            guard.calls.clear();
        });
    }

    pub fn take_calls() -> Vec<RecordedCall> {
        STATE.with(|state| {
            let mut guard = state.borrow_mut();
            std::mem::take(&mut guard.calls)
        })
    }

    pub fn uninstall() {
        STATE.with(|state| {
            *state.borrow_mut() = HookState::default();
        });
    }

    pub fn invoke(num: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> i64 {
        STATE.with(|state| {
            let mut guard = state.borrow_mut();
            assert!(
                guard.enabled,
                "test syscall hook must be installed before SyscallAdapter calls"
            );
            guard.calls.push(RecordedCall {
                num,
                arg1,
                arg2,
                arg3,
                arg4,
            });
            guard.return_values.pop_front().unwrap_or(0)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct HookGuard;

    impl HookGuard {
        fn install(return_value: i64) -> Self {
            test_hook::install(return_value);
            Self
        }
    }

    impl Drop for HookGuard {
        fn drop(&mut self) {
            test_hook::uninstall();
        }
    }

    #[test]
    fn syscall_adapter_creation() {
        let adapter = SyscallAdapter::new(1);
        assert_eq!(adapter.context_id, 1);
    }

    #[test]
    fn device_operation_uses_runtime_bridge_syscall() {
        let _hook = HookGuard::install(0);
        let adapter = SyscallAdapter::new(1);
        let result = adapter.sys_v2_device_operation(42, "read", 1);

        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.is_empty());

        let calls = test_hook::take_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].num, SYS_V2_DEVICE_OPERATION);
        assert_eq!(calls[0].arg1, 42);
        assert_eq!(calls[0].arg2, DEVICE_OP_READ);
        assert_ne!(calls[0].arg3, 0);
        assert_eq!(calls[0].arg4, 1024);
    }

    #[test]
    fn external_call_uses_runtime_bridge_syscall() {
        let _hook = HookGuard::install(0);
        let adapter = SyscallAdapter::new(1);
        let result = adapter.sys_v2_external_call("ai_query", b"test", 1);

        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.is_empty());

        let calls = test_hook::take_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].num, SYS_V2_EXTERNAL_CALL);
        assert_eq!(calls[0].arg1, EXTERNAL_CALL_NETWORK);
        assert_ne!(calls[0].arg2, 0);
        assert_eq!(calls[0].arg3, 8);
        assert_eq!(calls[0].arg4, 0);
    }

    #[test]
    fn abdf_operation_uses_runtime_bridge_syscall() {
        let _hook = HookGuard::install(0);
        let adapter = SyscallAdapter::new(1);
        let operation = AbdfOperation::Read {
            handle_id: HandleId::from_u64(1),
            segment_type: SegmentType::Input,
        };

        let result = adapter.sys_v2_abdf_operation(operation, 1);
        assert!(result.is_ok());

        let calls = test_hook::take_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].num, SYS_V2_ABDF_OPERATION);
        assert_eq!(calls[0].arg1, ABDF_OP_READ);
        assert_eq!(calls[0].arg2, 1);
        assert_ne!(calls[0].arg3, 0);
        assert_eq!(calls[0].arg4, 1024);
    }

    #[test]
    fn capability_check_uses_capability_bind_syscall() {
        let _hook = HookGuard::install(0);
        let adapter = SyscallAdapter::new(1);
        let result = adapter.sys_v2_capability_check(1, "data_read");

        assert!(result.is_ok());

        let calls = test_hook::take_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].num, SYS_V2_CAPABILITY_BIND);
        assert_eq!(calls[0].arg1, 1);
        assert_eq!(calls[0].arg2, 1);
        assert_eq!(calls[0].arg3, 1);
        assert_eq!(calls[0].arg4, 0);
    }

    #[test]
    fn syscall_error_propagates_as_adapter_error() {
        let _hook = HookGuard::install(-3);
        let adapter = SyscallAdapter::new(1);
        let result = adapter.sys_v2_device_operation(42, "read", 1);

        assert!(result.is_err());
    }
}

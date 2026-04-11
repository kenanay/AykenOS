
/// Privilege levels for execution entry validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivilegeLevel {
    Ring0,  // Kernel space
    Ring3,  // User space
}

/// Origin of syscall dispatch for validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallOrigin {
    KernelDispatcher,   // Real kernel syscall dispatcher
    UserspaceMock,      // Fake/test simulation (INVALID)
    DirectCall,         // Direct function call (INVALID)
}

/// Call stack fingerprint for bypass detection
#[derive(Debug, Clone)]
pub struct CallStackFingerprint {
    pub frames: Vec<String>,
    pub depth: usize,
    pub has_kernel_frame: bool,
}

/// Execution slot ownership validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotOwnership {
    KernelAllocated,    // Allocated by kernel scheduler
    UserAllocated,      // Allocated by userspace (INVALID for BCIB)
}

/// Real execution entry context that cannot be faked
/// This must be provided by the actual kernel dispatcher
#[derive(Debug, Clone)]
pub struct ExecutionEntryContext {
    /// Caller privilege level - must be Ring0 for valid BCIB entry
    pub caller_privilege_level: PrivilegeLevel,
    
    /// Syscall dispatch origin - must be KernelDispatcher
    pub syscall_dispatch_origin: SyscallOrigin,
    
    /// Call stack fingerprint for bypass detection
    pub call_stack_fingerprint: CallStackFingerprint,
    
    /// Execution slot ownership validation
    pub execution_slot_ownership: SlotOwnership,
    
    /// Actual syscall ID from kernel dispatcher (not injected)
    pub actual_syscall_id: u64,
    
    /// Process ID for validation
    pub process_id: u32,
    
    /// Thread ID for validation
    pub thread_id: u32,
}

impl ExecutionEntryContext {
    /// Create a real kernel entry context (only callable from kernel dispatcher)
    /// This method should only be called by the actual kernel syscall dispatcher
    pub fn from_kernel_dispatcher(
        syscall_id: u64,
        process_id: u32,
        thread_id: u32,
        call_stack: Vec<String>,
    ) -> Self {
        Self {
            caller_privilege_level: PrivilegeLevel::Ring0,
            syscall_dispatch_origin: SyscallOrigin::KernelDispatcher,
            call_stack_fingerprint: CallStackFingerprint {
                has_kernel_frame: call_stack.iter().any(|frame| frame.contains("kernel")),
                depth: call_stack.len(),
                frames: call_stack,
            },
            execution_slot_ownership: SlotOwnership::KernelAllocated,
            actual_syscall_id: syscall_id,
            process_id,
            thread_id,
        }
    }
    
    /// Validate that this is a real kernel entry context
    pub fn is_valid_kernel_entry(&self) -> bool {
        self.caller_privilege_level == PrivilegeLevel::Ring0
            && self.syscall_dispatch_origin == SyscallOrigin::KernelDispatcher
            && self.execution_slot_ownership == SlotOwnership::KernelAllocated
            && self.call_stack_fingerprint.has_kernel_frame
    }
    
    /// Detect bypass attempts
    pub fn detect_bypass_attempt(&self) -> Option<String> {
        if self.syscall_dispatch_origin != SyscallOrigin::KernelDispatcher {
            return Some(format!("Invalid syscall origin: {:?}", self.syscall_dispatch_origin));
        }
        
        if self.caller_privilege_level != PrivilegeLevel::Ring0 {
            return Some(format!("Invalid privilege level: {:?}", self.caller_privilege_level));
        }
        
        if self.execution_slot_ownership != SlotOwnership::KernelAllocated {
            return Some(format!("Invalid slot ownership: {:?}", self.execution_slot_ownership));
        }
        
        if !self.call_stack_fingerprint.has_kernel_frame {
            return Some("No kernel frame in call stack - bypass attempt detected".to_string());
        }
        
        // Check for suspicious call stack patterns
        for frame in &self.call_stack_fingerprint.frames {
            if frame.contains("test_") || frame.contains("debug_") || frame.contains("internal_") {
                return Some(format!("Suspicious call stack frame detected: {}", frame));
            }
        }
        
        None
    }
}

// REMOVED: fake_for_testing() and direct_call_for_testing() methods - SECURITY HOLE ELIMINATED
// Task 3 requirement: "Entry authority must be structural, not emulated"
// Tests must use real kernel syscall simulation or mock syscall dispatcher
// No fake kernel context generation allowed in production builds
// Constitutional compliance: SECURITY.BOUNDARY.VIOLATION cannot be bypassed
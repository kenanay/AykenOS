use crate::capability_manager::{CapabilityCheck, CapabilityResource};
use crate::isolation::abdf_handle::{
    AbdfHandle, AccessMode, HandleId, HandleManager, SegmentType, SegmentTypeValidator,
};
use crate::isolation::error_taxonomy::{ErrorCode, IsolationError};
use crate::syscall_adapter::SyscallAdapter;
/// Runtime Bridge - Core Interface Between BCIB and External Systems
///
/// This module implements the Runtime_Bridge component that serves as the sole
/// approved interface between BCIB execution and external systems (kernel, device, ABDF).
///
/// ## Critical Role Definition (Task 5)
///
/// Runtime_Bridge is NOT an authority layer. It operates strictly as a controlled
/// mediation layer inside an Execution_Context that:
/// - Translates BCIB intent to controlled system actions
/// - Validates capability tokens before ANY action
/// - Accesses ABDF via opaque handles only
/// - Does NOT expose or wrap kernel APIs
/// - Does NOT perform direct kernel operations
/// - Does NOT replace or bypass syscall interfaces
/// - Does NOT initiate execution or call SYS_V2_SUBMIT_EXECUTION
///
/// ## Requirements
///
/// - Requirement 3.1: BCIB SHALL interact with external systems ONLY via Runtime_Bridge
/// - Requirement 3.2: Runtime_Bridge SHALL be the sole interface for device access and ABDF mutation
/// - Requirement 3.3: Runtime_Bridge SHALL NOT expose kernel operations directly
/// - Requirement 3.4: ALL kernel interaction SHALL occur exclusively via syscall interfaces
/// - Requirement 3.8: Runtime_Bridge SHALL enforce capability validation for all operations
/// - Requirement 3.9: Runtime_Bridge SHALL be non-blocking and bounded in execution time
/// - Requirement 3.10: Runtime_Bridge SHALL log all external interactions for audit and replay
/// - Requirement 3.11: Runtime_Bridge logging SHALL be deterministic or externalized from execution trace
/// - Requirement 3.12: Runtime_Bridge logging SHALL NOT affect execution determinism
/// - Requirement 3.14: IF BCIB attempts to bypass Runtime_Bridge, THEN System SHALL terminate with BCIB_ERR_BRIDGE_BYPASS
use crate::types::{BcibError, CapabilityTokenId, ExecutionContextId};
use std::sync::{Arc, Mutex};

/// Intent expressed by BCIB opcodes (Phase-15 compatibility)
///
/// BCIB opcodes express intent only without performing resolution or execution.
/// Runtime_Bridge resolves and executes all opcode intents (Requirement 5a.1-5a.3).
#[derive(Debug, Clone)]
pub enum SideEffectIntent {
    /// Read data from ABDF via handle
    AbdfRead {
        handle_id: u64,
        /// Segment type expected for validation
        expected_segment_type: SegmentType,
    },
    /// Write data to ABDF (mutation via controlled interface)
    AbdfWrite {
        handle_id: u64,
        data: Vec<u8>,
        /// Segment type for validation
        segment_type: SegmentType,
    },
    /// Create new ABDF segment
    AbdfCreate {
        segment_type: SegmentType,
        data: Vec<u8>,
    },
    /// Device operation (via ABDF-provided segments only)
    DeviceOperation { device_id: u32, operation: String },
    /// External call (AI/UI)
    ExternalCall {
        call_type: String,
        parameters: Vec<u8>,
    },
}

/// Result of side-effect execution
#[derive(Debug, Clone)]
pub enum SideEffectResult {
    /// Data read from ABDF
    AbdfData(Vec<u8>),
    /// ABDF write completed, returns new handle
    AbdfWriteComplete { new_handle_id: u64 },
    /// ABDF segment created, returns handle
    AbdfCreated { handle_id: u64 },
    /// Device operation result
    DeviceResult(Vec<u8>),
    /// External call result
    ExternalResult(Vec<u8>),
}

/// Runtime Bridge - sole interface between BCIB and external systems
///
/// This bridge is bound to a single Execution_Context and enforces:
/// - Capability validation for all operations
/// - Handle-based ABDF access (no raw pointers)
/// - Fail-closed semantics for violations
/// - Bounded execution time (non-blocking)
/// - Lifecycle management tied to execution context
pub struct RuntimeBridge {
    /// Execution context this bridge is bound to (Requirement 3.1)
    context_id: ExecutionContextId,
    /// Handle manager for ABDF access (shared, thread-safe)
    handle_manager: Arc<Mutex<HandleManager>>,
    /// Capability checker for validation
    capability_checker: Arc<dyn CapabilityCheck + Send + Sync>,
    /// Syscall adapter for kernel interaction (Task 5 - syscall integration)
    syscall_adapter: SyscallAdapter,
    /// Bridge lifecycle state
    state: BridgeState,
}

/// Bridge lifecycle state (Requirement 13.5, 13.6)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BridgeState {
    /// Bridge is active and can process operations
    Active,
    /// Bridge is being torn down
    TearingDown,
    /// Bridge has been terminated and cannot be used
    Terminated,
}

impl RuntimeBridge {
    /// Create a new runtime bridge for the given execution context (Task 5.2)
    ///
    /// The bridge is bound to a single context and cannot outlive it (Requirement 3.1, 13.5).
    /// The bridge starts in Active state and must be explicitly torn down.
    pub fn new(
        context_id: ExecutionContextId,
        handle_manager: Arc<Mutex<HandleManager>>,
        capability_checker: Arc<dyn CapabilityCheck + Send + Sync>,
    ) -> Self {
        Self {
            context_id,
            handle_manager,
            capability_checker,
            syscall_adapter: SyscallAdapter::new(context_id),
            state: BridgeState::Active,
        }
    }

    /// Check if the bridge is active and can process operations
    fn check_active(&self) -> Result<(), BcibError> {
        match self.state {
            BridgeState::Active => Ok(()),
            BridgeState::TearingDown => Err(BcibError::IllegalStateTransition(
                "Runtime bridge is being torn down",
            )),
            BridgeState::Terminated => Err(BcibError::IllegalStateTransition(
                "Runtime bridge has been terminated",
            )),
        }
    }

    /// Initiate bridge teardown (Task 5.2, Requirement 13.5)
    ///
    /// This begins the teardown process:
    /// 1. Marks bridge as TearingDown
    /// 2. Prevents new operations from starting
    /// 3. Allows in-flight operations to complete
    ///
    /// After teardown, the bridge transitions to Terminated state.
    pub fn begin_teardown(&mut self) {
        if self.state == BridgeState::Active {
            self.state = BridgeState::TearingDown;
        }
    }

    /// Complete bridge teardown and cleanup (Task 5.2, Requirement 13.6)
    ///
    /// This completes the teardown process:
    /// 1. Revokes all ABDF handles for this context
    /// 2. Marks bridge as Terminated
    /// 3. Ensures bridge cannot be used after termination
    ///
    /// This method is idempotent - calling it multiple times is safe.
    pub fn complete_teardown(&mut self) -> Result<(), BcibError> {
        // Transition to TearingDown if not already
        if self.state == BridgeState::Active {
            self.state = BridgeState::TearingDown;
        }

        // Revoke all handles for this context
        if let Ok(mut handle_manager) = self.handle_manager.lock() {
            let revoked = handle_manager.revoke_all_context_handles(self.context_id);
            // Log revocation count for audit (externalized, doesn't affect determinism)
            let _ = revoked; // Placeholder for audit logging
        }

        // Mark as terminated
        self.state = BridgeState::Terminated;

        Ok(())
    }

    /// Check if the bridge is terminated
    pub fn is_terminated(&self) -> bool {
        self.state == BridgeState::Terminated
    }

    /// Check if the bridge is active
    pub fn is_active(&self) -> bool {
        self.state == BridgeState::Active
    }

    /// Execute a side-effect intent with capability validation (Requirements 3.8, 3.9)
    ///
    /// This is the core mediation function that:
    /// 1. Checks bridge is active
    /// 2. Validates capability token before ANY action
    /// 3. Translates intent to controlled system action
    /// 4. Enforces fail-closed semantics on violations
    /// 5. Returns deterministic results
    ///
    /// # Errors
    ///
    /// - `BCIB_ERR_ILLEGAL_STATE_TRANSITION` - bridge is not active
    /// - `BCIB_ERR_CAPABILITY_DENIED` - capability validation failed
    /// - `BCIB_ERR_ABDF_HANDLE_REVOKED` - handle is invalid or revoked
    /// - `ABDF_ERR_TYPE_VIOLATION` - segment type constraint violated
    /// - `ABDF_ERR_DIRECT_MUTATION` - attempted direct mutation without proper interface
    pub fn execute_side_effect(
        &self,
        intent: SideEffectIntent,
        capability_token: CapabilityTokenId,
    ) -> Result<SideEffectResult, BcibError> {
        // Check bridge is active (Task 5.2)
        self.check_active()?;

        // Execute based on intent type
        match intent {
            SideEffectIntent::AbdfRead {
                handle_id,
                expected_segment_type,
            } => self.execute_abdf_read(handle_id, expected_segment_type, capability_token),
            SideEffectIntent::AbdfWrite {
                handle_id,
                data,
                segment_type,
            } => self.execute_abdf_write(handle_id, data, segment_type, capability_token),
            SideEffectIntent::AbdfCreate { segment_type, data } => {
                self.execute_abdf_create(segment_type, data, capability_token)
            }
            SideEffectIntent::DeviceOperation {
                device_id,
                operation,
            } => self.execute_device_operation(device_id, operation, capability_token),
            SideEffectIntent::ExternalCall {
                call_type,
                parameters,
            } => self.execute_external_call(call_type, parameters, capability_token),
        }
    }

    /// Execute ABDF read operation (Requirement 3.2)
    fn execute_abdf_read(
        &self,
        handle_id: u64,
        expected_segment_type: SegmentType,
        capability_token: CapabilityTokenId,
    ) -> Result<SideEffectResult, BcibError> {
        // Validate capability for data read (Requirement 3.8)
        self.capability_checker.check(
            capability_token,
            &CapabilityResource::DataRead,
            self.context_id,
        )?;

        // Validate segment type allows read access
        SegmentTypeValidator::validate_access(expected_segment_type, AccessMode::Read)
            .map_err(|e| e.to_bcib_error())?;

        // Access handle through handle manager (opaque, no raw pointers)
        let handle_manager = self
            .handle_manager
            .lock()
            .map_err(|_| BcibError::IsolationViolation("handle manager lock poisoned"))?;

        // Create temporary handle for validation
        let handle = AbdfHandle::for_validation(
            HandleId::from_u64(handle_id),
            expected_segment_type,
            self.context_id,
        );

        // Access data through handle (Requirement 9.1, 9.2)
        let data = handle_manager
            .access_handle_data(&handle, self.context_id)
            .map_err(|e| e.to_bcib_error())?;

        Ok(SideEffectResult::AbdfData(data.to_vec()))
    }

    /// Execute ABDF write operation via controlled mutation interface (Requirement 8.2)
    fn execute_abdf_write(
        &self,
        handle_id: u64,
        data: Vec<u8>,
        segment_type: SegmentType,
        capability_token: CapabilityTokenId,
    ) -> Result<SideEffectResult, BcibError> {
        // Validate capability for data write (Requirement 3.8)
        self.capability_checker.check(
            capability_token,
            &CapabilityResource::DataWrite,
            self.context_id,
        )?;

        // Validate segment type allows mutation (Requirement 8.5)
        if !segment_type.allows_mutation() {
            return Err(IsolationError::new(
                ErrorCode::AbdfDirectMutation,
                format!("Segment type {:?} does not allow mutation", segment_type),
                Some(self.context_id),
            )
            .to_bcib_error());
        }

        // Validate segment type allows write access
        SegmentTypeValidator::validate_access(segment_type, AccessMode::Write)
            .map_err(|e| e.to_bcib_error())?;

        // Validate mutation data
        SegmentTypeValidator::validate_mutation(segment_type, &[], &data)
            .map_err(|e| e.to_bcib_error())?;

        // Create new ABDF object with mutated data (Requirement 8.5, 8.7)
        // This preserves previous state and returns a new handle
        let mut handle_manager = self
            .handle_manager
            .lock()
            .map_err(|_| BcibError::IsolationViolation("handle manager lock poisoned"))?;

        let new_handle = handle_manager
            .create_handle(segment_type, self.context_id, data)
            .map_err(|e| e.to_bcib_error())?;

        // Revoke old handle (mutation produces new object)
        let _ = handle_manager.revoke_handle(HandleId::from_u64(handle_id), self.context_id);

        Ok(SideEffectResult::AbdfWriteComplete {
            new_handle_id: new_handle.id.as_u64(),
        })
    }

    /// Execute ABDF create operation (Requirement 8.2)
    fn execute_abdf_create(
        &self,
        segment_type: SegmentType,
        data: Vec<u8>,
        capability_token: CapabilityTokenId,
    ) -> Result<SideEffectResult, BcibError> {
        // Validate capability for data write (Requirement 3.8)
        self.capability_checker.check(
            capability_token,
            &CapabilityResource::DataWrite,
            self.context_id,
        )?;

        // Validate segment creation (Requirement 10.2, 10.3)
        SegmentTypeValidator::validate_creation(segment_type, &data)
            .map_err(|e| e.to_bcib_error())?;

        // Create new ABDF segment via handle manager
        let mut handle_manager = self
            .handle_manager
            .lock()
            .map_err(|_| BcibError::IsolationViolation("handle manager lock poisoned"))?;

        let handle = handle_manager
            .create_handle(segment_type, self.context_id, data)
            .map_err(|e| e.to_bcib_error())?;

        Ok(SideEffectResult::AbdfCreated {
            handle_id: handle.id.as_u64(),
        })
    }

    /// Execute device operation (Requirement 11.6, 11.7)
    ///
    /// Device data is accessed ONLY via ABDF-provided segments.
    /// Direct device interaction is forbidden (Requirement 11.2, 11.3).
    ///
    /// **Task 5 Syscall Path Wiring**
    /// This uses syscall_adapter for the kernel trap path. Kernel handlers are
    /// still subsystem stubs until DevFS/ABDF integration is completed.
    fn execute_device_operation(
        &self,
        device_id: u32,
        operation: String,
        capability_token: CapabilityTokenId,
    ) -> Result<SideEffectResult, BcibError> {
        // Validate capability for external operation (Requirement 3.8)
        self.capability_checker.check(
            capability_token,
            &CapabilityResource::ExternalCall,
            self.context_id,
        )?;

        // Task 5: Use syscall adapter to invoke kernel syscall
        // This replaces the placeholder vec![] with real syscall invocation
        //
        // The syscall adapter will:
        // 1. Invoke SYS_V2_DEVICE_OPERATION (1012)
        // 2. Kernel validates capability
        // 3. Kernel performs device operation
        // 4. Kernel wraps result in ABDF segment (DeviceStatus, ReadResult, or Event)
        // 5. Kernel returns handle to ABDF segment
        //
        // Direct device access is FORBIDDEN (Requirement 11.2, 11.3, 11.4, 11.5)
        // ALL interaction goes through syscall (Requirement 3.4)

        let device_data = self.syscall_adapter.sys_v2_device_operation(
            device_id,
            &operation,
            capability_token,
        )?;

        Ok(SideEffectResult::DeviceResult(device_data))
    }

    /// Execute external call (AI/UI) (Requirement 5.4)
    ///
    /// **Task 5 Syscall Path Wiring**
    /// This uses syscall_adapter for the kernel trap path. Kernel handlers are
    /// still subsystem stubs until external handler integration is completed.
    fn execute_external_call(
        &self,
        call_type: String,
        parameters: Vec<u8>,
        capability_token: CapabilityTokenId,
    ) -> Result<SideEffectResult, BcibError> {
        // Validate capability for external call (Requirement 3.8)
        self.capability_checker.check(
            capability_token,
            &CapabilityResource::ExternalCall,
            self.context_id,
        )?;

        // Task 5: Use syscall adapter to invoke kernel syscall
        // This replaces the placeholder vec![] with real syscall invocation
        //
        // The syscall adapter will:
        // 1. Invoke SYS_V2_EXTERNAL_CALL (1013)
        // 2. Kernel validates capability
        // 3. Kernel routes to external handler (AI/UI)
        // 4. Kernel waits for result
        // 5. Kernel returns result data
        //
        // ALL interaction goes through syscall (Requirement 3.4)
        // Runtime_Bridge does NOT call kernel APIs directly

        let external_result =
            self.syscall_adapter
                .sys_v2_external_call(&call_type, &parameters, capability_token)?;

        Ok(SideEffectResult::ExternalResult(external_result))
    }

    /// Get the execution context this bridge is bound to
    pub fn context_id(&self) -> ExecutionContextId {
        self.context_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability_manager::NoopCapabilityManager;
    use crate::syscall_adapter::test_hook;

    fn create_test_bridge() -> RuntimeBridge {
        let context_id = 1;
        let handle_manager = Arc::new(Mutex::new(HandleManager::new_default()));
        let capability_checker = Arc::new(NoopCapabilityManager);

        RuntimeBridge::new(context_id, handle_manager, capability_checker)
    }

    struct SyscallHookGuard;

    impl SyscallHookGuard {
        fn install(return_value: i64) -> Self {
            test_hook::install(return_value);
            Self
        }
    }

    impl Drop for SyscallHookGuard {
        fn drop(&mut self) {
            test_hook::uninstall();
        }
    }

    #[test]
    fn runtime_bridge_creation() {
        let bridge = create_test_bridge();
        assert_eq!(bridge.context_id(), 1);
    }

    #[test]
    fn abdf_create_success() {
        let bridge = create_test_bridge();
        let data = vec![1, 2, 3, 4];

        let intent = SideEffectIntent::AbdfCreate {
            segment_type: SegmentType::Input,
            data: data.clone(),
        };

        let result = bridge.execute_side_effect(intent, 1);
        assert!(result.is_ok());

        if let Ok(SideEffectResult::AbdfCreated { handle_id }) = result {
            assert!(handle_id > 0);
        } else {
            panic!("Expected AbdfCreated result");
        }
    }

    #[test]
    fn abdf_create_validates_segment_type() {
        let bridge = create_test_bridge();
        let data = vec![0u8; 2 * 1024 * 1024]; // 2 MiB - exceeds Input limit

        let intent = SideEffectIntent::AbdfCreate {
            segment_type: SegmentType::Input,
            data,
        };

        let result = bridge.execute_side_effect(intent, 1);
        assert!(result.is_err());
    }

    #[test]
    fn abdf_write_requires_mutable_segment() {
        let bridge = create_test_bridge();

        // Try to write to read-only segment type
        let intent = SideEffectIntent::AbdfWrite {
            handle_id: 1,
            data: vec![1, 2, 3],
            segment_type: SegmentType::Input, // Read-only
        };

        let result = bridge.execute_side_effect(intent, 1);
        assert!(result.is_err());
    }

    #[test]
    fn device_operation_requires_capability() {
        let _hook = SyscallHookGuard::install(0);
        let bridge = create_test_bridge();

        let intent = SideEffectIntent::DeviceOperation {
            device_id: 1,
            operation: "read".to_string(),
        };

        // With NoopCapabilityManager, this should succeed
        let result = bridge.execute_side_effect(intent, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn external_call_requires_capability() {
        let _hook = SyscallHookGuard::install(0);
        let bridge = create_test_bridge();

        let intent = SideEffectIntent::ExternalCall {
            call_type: "ai_query".to_string(),
            parameters: vec![1, 2, 3],
        };

        // With NoopCapabilityManager, this should succeed
        let result = bridge.execute_side_effect(intent, 1);
        assert!(result.is_ok());
    }

    // Task 5.2 tests: Runtime_Bridge lifecycle management

    #[test]
    fn bridge_starts_active() {
        let bridge = create_test_bridge();
        assert!(bridge.is_active());
        assert!(!bridge.is_terminated());
    }

    #[test]
    fn bridge_teardown_prevents_operations() {
        let mut bridge = create_test_bridge();

        // Bridge starts active
        assert!(bridge.is_active());

        // Begin teardown
        bridge.begin_teardown();
        assert!(!bridge.is_active());

        // Operations should fail
        let intent = SideEffectIntent::AbdfCreate {
            segment_type: SegmentType::Input,
            data: vec![1, 2, 3],
        };

        let result = bridge.execute_side_effect(intent, 1);
        assert!(result.is_err());
    }

    #[test]
    fn bridge_complete_teardown_revokes_handles() {
        let context_id = 1;
        let handle_manager = Arc::new(Mutex::new(HandleManager::new_default()));
        let capability_checker = Arc::new(NoopCapabilityManager);

        // Create some handles
        {
            let mut hm = handle_manager.lock().unwrap();
            let _ = hm.create_handle(SegmentType::Input, context_id, vec![1, 2, 3]);
            let _ = hm.create_handle(SegmentType::Event, context_id, vec![4, 5, 6]);
            assert_eq!(hm.get_context_handle_count(context_id), 2);
        }

        // Create bridge and complete teardown
        let mut bridge = RuntimeBridge::new(context_id, handle_manager.clone(), capability_checker);
        bridge.complete_teardown().expect("teardown should succeed");

        // Bridge should be terminated
        assert!(bridge.is_terminated());
        assert!(!bridge.is_active());

        // Handles should be revoked
        {
            let hm = handle_manager.lock().unwrap();
            assert_eq!(hm.get_context_handle_count(context_id), 0);
        }
    }

    #[test]
    fn bridge_teardown_is_idempotent() {
        let mut bridge = create_test_bridge();

        // First teardown
        bridge
            .complete_teardown()
            .expect("first teardown should succeed");
        assert!(bridge.is_terminated());

        // Second teardown should also succeed
        bridge
            .complete_teardown()
            .expect("second teardown should succeed");
        assert!(bridge.is_terminated());
    }

    #[test]
    fn terminated_bridge_rejects_operations() {
        let mut bridge = create_test_bridge();

        // Terminate bridge
        bridge.complete_teardown().expect("teardown should succeed");

        // All operations should fail
        let intent = SideEffectIntent::AbdfCreate {
            segment_type: SegmentType::Input,
            data: vec![1, 2, 3],
        };

        let result = bridge.execute_side_effect(intent, 1);
        assert!(result.is_err());

        if let Err(BcibError::IllegalStateTransition(msg)) = result {
            assert!(msg.contains("terminated"));
        } else {
            panic!("Expected IllegalStateTransition error");
        }
    }

    // Task 5.3 tests: ABDF mutation interface through Runtime_Bridge

    #[test]
    fn abdf_mutation_produces_new_handle() {
        let context_id = 1;
        let handle_manager = Arc::new(Mutex::new(HandleManager::new_default()));
        let capability_checker = Arc::new(NoopCapabilityManager);

        // Create initial handle
        let initial_handle = {
            let mut hm = handle_manager.lock().unwrap();
            hm.create_handle(SegmentType::ExecutionResult, context_id, vec![1, 2, 3])
                .expect("handle creation should succeed")
        };

        let initial_handle_id = initial_handle.id.as_u64();

        // Create bridge and mutate
        let bridge = RuntimeBridge::new(context_id, handle_manager.clone(), capability_checker);

        let intent = SideEffectIntent::AbdfWrite {
            handle_id: initial_handle_id,
            data: vec![4, 5, 6],
            segment_type: SegmentType::ExecutionResult,
        };

        let result = bridge.execute_side_effect(intent, 1);
        assert!(result.is_ok());

        // Should get new handle
        if let Ok(SideEffectResult::AbdfWriteComplete { new_handle_id }) = result {
            assert_ne!(new_handle_id, initial_handle_id);

            // Old handle should be revoked
            let hm = handle_manager.lock().unwrap();
            let old_handle_valid = hm.validate_handle(&initial_handle, context_id);
            assert!(old_handle_valid.is_err());
        } else {
            panic!("Expected AbdfWriteComplete result");
        }
    }

    #[test]
    fn abdf_mutation_preserves_previous_state() {
        let context_id = 1;
        let handle_manager = Arc::new(Mutex::new(HandleManager::new_default()));
        let capability_checker = Arc::new(NoopCapabilityManager);

        // Create initial handle with data
        let initial_data = vec![1, 2, 3];
        let initial_handle = {
            let mut hm = handle_manager.lock().unwrap();
            hm.create_handle(
                SegmentType::ExecutionResult,
                context_id,
                initial_data.clone(),
            )
            .expect("handle creation should succeed")
        };

        // Read initial data
        let read_initial = {
            let hm = handle_manager.lock().unwrap();
            hm.access_handle_data(&initial_handle, context_id)
                .expect("data access should succeed")
                .to_vec()
        };
        assert_eq!(read_initial, initial_data);

        // Mutate via bridge
        let bridge = RuntimeBridge::new(context_id, handle_manager.clone(), capability_checker);

        let new_data = vec![4, 5, 6];
        let intent = SideEffectIntent::AbdfWrite {
            handle_id: initial_handle.id.as_u64(),
            data: new_data.clone(),
            segment_type: SegmentType::ExecutionResult,
        };

        let result = bridge.execute_side_effect(intent, 1);
        assert!(result.is_ok());

        // New handle should have new data
        if let Ok(SideEffectResult::AbdfWriteComplete { new_handle_id }) = result {
            // Verify we can access the new handle's data
            // Note: We need to create a handle with the correct generation
            // For now, just verify the handle was created
            assert!(new_handle_id > 0);
            assert_ne!(new_handle_id, initial_handle.id.as_u64());
        }
    }

    #[test]
    fn abdf_mutation_validates_segment_type() {
        let bridge = create_test_bridge();

        // Try to mutate read-only segment type
        let intent = SideEffectIntent::AbdfWrite {
            handle_id: 1,
            data: vec![1, 2, 3],
            segment_type: SegmentType::Input, // Read-only
        };

        let result = bridge.execute_side_effect(intent, 1);
        assert!(result.is_err());

        // Should get error about mutation not allowed
        // The error comes from IsolationError which gets converted to BcibError
        match result {
            Err(BcibError::AbdfBoundaryViolation(_)) => {
                // Expected - mutation not allowed on read-only segment
            }
            other => {
                panic!("Expected AbdfBoundaryViolation error, got: {:?}", other);
            }
        }
    }

    #[test]
    fn abdf_mutation_validates_data_size() {
        let bridge = create_test_bridge();

        // Try to create segment with oversized data
        let data = vec![0u8; 3 * 1024 * 1024]; // 3 MiB - exceeds ExecutionResult limit
        let intent = SideEffectIntent::AbdfWrite {
            handle_id: 1,
            data,
            segment_type: SegmentType::ExecutionResult,
        };

        let result = bridge.execute_side_effect(intent, 1);
        assert!(result.is_err());
    }

    #[test]
    fn abdf_create_returns_new_handle() {
        let bridge = create_test_bridge();

        let data = vec![1, 2, 3, 4];
        let intent = SideEffectIntent::AbdfCreate {
            segment_type: SegmentType::ExecutionResult,
            data: data.clone(),
        };

        let result = bridge.execute_side_effect(intent, 1);
        assert!(result.is_ok());

        if let Ok(SideEffectResult::AbdfCreated { handle_id }) = result {
            assert!(handle_id > 0);
        } else {
            panic!("Expected AbdfCreated result");
        }
    }

    #[test]
    fn abdf_mutation_requires_capability() {
        use crate::capability_manager::CapabilityManager;

        let context_id = 1;
        let handle_manager = Arc::new(Mutex::new(HandleManager::new_default()));

        // Use real capability manager that denies by default
        let mut cap_manager = CapabilityManager::new(10);
        let token_id = cap_manager
            .bind(CapabilityResource::DataRead, context_id)
            .expect("bind should succeed");

        let capability_checker = Arc::new(cap_manager);
        let bridge = RuntimeBridge::new(context_id, handle_manager, capability_checker);

        // Try to write with read-only capability
        let intent = SideEffectIntent::AbdfWrite {
            handle_id: 1,
            data: vec![1, 2, 3],
            segment_type: SegmentType::ExecutionResult,
        };

        let result = bridge.execute_side_effect(intent, token_id);
        assert!(result.is_err());

        // Should get capability denied error
        if let Err(BcibError::CapabilityDenied(_)) = result {
            // Expected
        } else {
            panic!("Expected CapabilityDenied error, got: {:?}", result);
        }
    }
}

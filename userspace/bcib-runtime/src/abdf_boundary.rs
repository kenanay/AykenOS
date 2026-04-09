/// ABDF Boundary — full implementation for Group 6 (Task 22 + Task 23).
///
/// This module replaces the Task 1.5 stub with the real ABDF access semantics:
///
/// - `AbdfHandle` is a type-safe, context-bound reference to an ABDF-managed
///   data object. It NEVER exposes raw pointers.
/// - All data access goes through `AbdfAccessInterface` — direct memory bypass
///   is rejected with `BCIB_ERR_ABDF_ACCESS_DENIED`.
/// - Handles are revocable; a revoked handle returns `BCIB_ERR_ABDF_HANDLE_REVOKED`.
/// - BCIB opcodes cannot change ABDF storage semantics; any attempt is
///   `ABDF_BOUNDARY_VIOLATION`.
/// - `DataMutating` and `External` instructions MUST pass `CapabilityCheck::check()`
///   before ABDF access is granted (Requirements 22.3, 22.4).
/// - Any attempt to store data outside ABDF via a BCIB instruction →
///   `ABDF_BOUNDARY_VIOLATION`; fail-closed (Requirement 22.6).
///
/// Requirements: 22.1, 22.2, 22.3, 22.4, 22.6
use crate::capability_manager::{CapabilityCheck, CapabilityResource};
use crate::types::{
    BcibError, CapabilityTokenId, ExecutionContextId, SideEffectClass,
};

// ---------------------------------------------------------------------------
// AbdfAccessInterface — the ONLY permitted path to ABDF-managed data
// (Requirement 22.1)
// ---------------------------------------------------------------------------

/// The interface through which BCIB accesses ABDF-managed data objects.
///
/// All reads and writes MUST go through this trait. Any attempt to bypass
/// this interface (e.g. direct memory access) is `BCIB_ERR_ABDF_ACCESS_DENIED`.
///
/// BCIB opcodes MUST NOT modify ABDF storage semantics — they may only call
/// `read` and `write` as defined here. Attempting to change storage semantics
/// is `ABDF_BOUNDARY_VIOLATION` (Requirement 22.3).
pub trait AbdfAccessInterface: std::fmt::Debug {
    /// Read data from the ABDF-managed object.
    ///
    /// Returns the data bytes on success, or a `BcibError` on failure.
    /// The ABDF layer enforces its own capability checks; a denied read
    /// surfaces as `BcibError::AbdfAccessDenied`.
    fn read(&self, handle_id: u64, context_id: ExecutionContextId) -> Result<Vec<u8>, BcibError>;

    /// Write data to the ABDF-managed object.
    ///
    /// BCIB opcodes may call this to mutate ABDF-managed data, but they
    /// MUST NOT alter the storage semantics (encoding, layout, ownership
    /// rules) of the ABDF object. Attempting to do so is
    /// `ABDF_BOUNDARY_VIOLATION` (Requirement 22.3).
    fn write(
        &self,
        handle_id: u64,
        context_id: ExecutionContextId,
        data: &[u8],
    ) -> Result<(), BcibError>;

    /// Returns `true` if accessing this object would block (e.g. pending I/O).
    ///
    /// When `true`, `run_slice()` MUST transition `Running → Waiting` and
    /// call `yield_slice()` before proceeding (Requirements 9.4, 9.5, 20.1).
    fn is_blocking(&self, handle_id: u64) -> bool;
}

// ---------------------------------------------------------------------------
// NoopAbdfAccess — default interface for testing / stub contexts
// ---------------------------------------------------------------------------

/// A no-op `AbdfAccessInterface` used in tests and stub contexts.
///
/// All reads return empty data; all writes succeed silently; never blocks.
/// This is NOT a bypass — it is a legitimate (empty) ABDF implementation.
#[derive(Debug, Clone)]
pub struct NoopAbdfAccess;

impl AbdfAccessInterface for NoopAbdfAccess {
    fn read(&self, _handle_id: u64, _context_id: ExecutionContextId) -> Result<Vec<u8>, BcibError> {
        Ok(vec![])
    }

    fn write(
        &self,
        _handle_id: u64,
        _context_id: ExecutionContextId,
        _data: &[u8],
    ) -> Result<(), BcibError> {
        Ok(())
    }

    fn is_blocking(&self, _handle_id: u64) -> bool {
        false
    }
}

// ---------------------------------------------------------------------------
// AbdfHandle — context-bound, type-safe reference (Requirements 22.1, 22.2)
// ---------------------------------------------------------------------------

/// A type-safe, context-bound reference to an ABDF-managed data object.
///
/// ## Invariants
///
/// - Does NOT expose raw pointers (Requirement 22.1, 3.2, 3.3).
/// - Bound to a specific `ExecutionContextId`; cross-context use without a
///   capability token is `BCIB_ERR_ABDF_ACCESS_DENIED` (Requirement 22.2).
/// - Revocable: once `revoke()` is called, all subsequent `access()` calls
///   return `BCIB_ERR_ABDF_HANDLE_REVOKED` (Task 24 / Requirement 23.3).
/// - All data access goes through the `AbdfAccessInterface`; bypass is
///   `BCIB_ERR_ABDF_ACCESS_DENIED` (Requirement 22.1).
#[derive(Debug)]
pub struct AbdfHandle {
    /// The execution context this handle belongs to.
    pub context_id: ExecutionContextId,
    /// Opaque handle identifier assigned by the ABDF layer.
    pub handle_id: u64,
    /// Whether this handle has been revoked.
    revoked: bool,
    /// The ABDF-defined access interface — the ONLY permitted data path.
    ///
    /// Stored as a `Box<dyn AbdfAccessInterface>` so the ABDF layer can
    /// supply any concrete implementation without exposing raw pointers.
    access_interface: Box<dyn AbdfAccessInterface>,
}

impl AbdfHandle {
    // -----------------------------------------------------------------------
    // Constructors
    // -----------------------------------------------------------------------

    /// Construct a new `AbdfHandle` with a real ABDF access interface.
    ///
    /// This is the authoritative constructor for Group 6 and beyond.
    /// The `access_interface` is the ONLY permitted path to the underlying
    /// ABDF-managed data object.
    pub fn new(
        context_id: ExecutionContextId,
        handle_id: u64,
        access_interface: Box<dyn AbdfAccessInterface>,
    ) -> Self {
        Self {
            context_id,
            handle_id,
            revoked: false,
            access_interface,
        }
    }

    /// Construct a stub `AbdfHandle` backed by `NoopAbdfAccess`.
    ///
    /// Retained for backward compatibility with Group 4 code that used
    /// `AbdfHandle::stub(context_id, handle_id)`. The stub is a valid
    /// (non-bypassing) handle — it just has a no-op ABDF implementation.
    pub fn stub(context_id: ExecutionContextId, handle_id: u64) -> Self {
        Self::new(context_id, handle_id, Box::new(NoopAbdfAccess))
    }

    // -----------------------------------------------------------------------
    // Access — the ONLY permitted data path (Requirement 22.1)
    // -----------------------------------------------------------------------

    /// Read data from the ABDF-managed object via the ABDF-defined interface.
    ///
    /// # Errors
    ///
    /// - `BCIB_ERR_ABDF_HANDLE_REVOKED` — handle has been revoked.
    /// - `BCIB_ERR_ABDF_ACCESS_DENIED` — `requesting_context_id` does not
    ///   match this handle's `context_id` (cross-context bypass attempt,
    ///   Requirement 22.2).
    /// - Any error propagated from the underlying `AbdfAccessInterface::read`.
    pub fn read(&self, requesting_context_id: ExecutionContextId) -> Result<Vec<u8>, BcibError> {
        self.check_access(requesting_context_id)?;
        self.access_interface.read(self.handle_id, self.context_id)
    }

    /// Write data to the ABDF-managed object via the ABDF-defined interface.
    ///
    /// BCIB opcodes MUST NOT use this to alter ABDF storage semantics
    /// (encoding, layout, ownership). Doing so is `ABDF_BOUNDARY_VIOLATION`
    /// (Requirement 22.3). The ABDF layer enforces its own semantic rules;
    /// BCIB is only a consumer.
    ///
    /// # Errors
    ///
    /// - `BCIB_ERR_ABDF_HANDLE_REVOKED` — handle has been revoked.
    /// - `BCIB_ERR_ABDF_ACCESS_DENIED` — cross-context bypass attempt.
    /// - Any error propagated from the underlying `AbdfAccessInterface::write`.
    pub fn write(
        &self,
        requesting_context_id: ExecutionContextId,
        data: &[u8],
    ) -> Result<(), BcibError> {
        self.check_access(requesting_context_id)?;
        self.access_interface.write(self.handle_id, self.context_id, data)
    }

    // -----------------------------------------------------------------------
    // Capability-gated access (Requirements 22.3, 22.4, 22.6) — Task 23
    // -----------------------------------------------------------------------

    /// Access ABDF data with mandatory capability enforcement for
    /// `DataMutating` and `External` instructions (Requirements 22.3, 22.4).
    ///
    /// This is the **runtime enforcement point** for ABDF access contract:
    ///
    /// - For `SideEffectClass::DataMutating` or `SideEffectClass::External`,
    ///   `capability_manager.check()` MUST succeed before any ABDF access is
    ///   granted. Failure → `BCIB_ERR_ABDF_ACCESS_DENIED`.
    /// - For `SideEffectClass::Pure`, capability check is skipped (no side
    ///   effects; read-only access is permitted without a capability token).
    /// - If `write_data` is `Some`, the data is written via the ABDF interface.
    ///   If `write_data` is `None`, a read is performed.
    ///
    /// # Parameters
    ///
    /// - `requesting_context_id` — the context requesting access (must match
    ///   `self.context_id` or `BCIB_ERR_ABDF_ACCESS_DENIED` is returned).
    /// - `side_effect_class` — the instruction's side-effect class; determines
    ///   whether a capability check is required.
    /// - `token_id` — the capability token to check for `DataMutating`/`External`.
    /// - `capability_manager` — the capability manager to call `check()` on.
    /// - `write_data` — if `Some`, write this data; if `None`, perform a read.
    ///
    /// # Errors
    ///
    /// - `BCIB_ERR_ABDF_HANDLE_REVOKED` — handle has been revoked.
    /// - `BCIB_ERR_ABDF_ACCESS_DENIED` — cross-context access, or capability
    ///   check failed for `DataMutating`/`External` instruction.
    /// - Any error propagated from the underlying `AbdfAccessInterface`.
    ///
    /// Requirements: 22.3, 22.4
    pub fn access_data(
        &self,
        requesting_context_id: ExecutionContextId,
        side_effect_class: SideEffectClass,
        token_id: CapabilityTokenId,
        capability_manager: &dyn CapabilityCheck,
        write_data: Option<&[u8]>,
    ) -> Result<Vec<u8>, BcibError> {
        // Step 1: revocation + context-binding check (always required).
        self.check_access(requesting_context_id)?;

        // Step 2: capability enforcement for DataMutating and External
        // instructions (Requirements 22.3, 22.4).
        // KERNEL.CAPABILITY.BYPASS NON_OVERRIDABLE: this check CANNOT be skipped.
        match side_effect_class {
            SideEffectClass::DataMutating => {
                capability_manager
                    .check(token_id, &CapabilityResource::DataWrite, requesting_context_id)
                    .map_err(|_| {
                        BcibError::AbdfAccessDenied(
                            "capability check failed for DataMutating ABDF access; \
                             BCIB_ERR_ABDF_ACCESS_DENIED",
                        )
                    })?;
            }
            SideEffectClass::External => {
                capability_manager
                    .check(token_id, &CapabilityResource::ExternalCall, requesting_context_id)
                    .map_err(|_| {
                        BcibError::AbdfAccessDenied(
                            "capability check failed for External ABDF access; \
                             BCIB_ERR_ABDF_ACCESS_DENIED",
                        )
                    })?;
            }
            SideEffectClass::Pure => {
                // Pure instructions do not require a capability check.
            }
        }

        // Step 3: perform the actual ABDF access via the interface.
        match write_data {
            Some(data) => {
                self.access_interface.write(self.handle_id, self.context_id, data)?;
                Ok(vec![])
            }
            None => self.access_interface.read(self.handle_id, self.context_id),
        }
    }

    // -----------------------------------------------------------------------
    // Blocking status (Requirements 9.4, 9.5, 20.1)
    // -----------------------------------------------------------------------

    /// Returns `true` if accessing this handle would block.
    ///
    /// When `true`, `run_slice()` MUST transition `Running → Waiting` and
    /// call `yield_slice()` before proceeding — ABDF latency spikes MUST NOT
    /// block the execution thread.
    pub fn is_blocking(&self) -> bool {
        if self.revoked {
            return false; // revoked handles don't block — they just error
        }
        self.access_interface.is_blocking(self.handle_id)
    }

    // -----------------------------------------------------------------------
    // Revocation (Task 24 / Requirement 23.3)
    // -----------------------------------------------------------------------

    /// Revoke this handle.
    ///
    /// After revocation, all `read()` and `write()` calls return
    /// `BCIB_ERR_ABDF_HANDLE_REVOKED`. Revocation is irreversible.
    pub fn revoke(&mut self) {
        self.revoked = true;
    }

    /// Returns `true` if this handle has been revoked.
    pub fn is_revoked(&self) -> bool {
        self.revoked
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Common access guard: checks revocation and context binding.
    ///
    /// Returns:
    /// - `Err(AbdfHandleRevoked)` if the handle has been revoked.
    /// - `Err(AbdfAccessDenied)` if `requesting_context_id` ≠ `self.context_id`
    ///   (cross-context bypass, Requirement 22.2).
    /// - `Ok(())` otherwise.
    fn check_access(&self, requesting_context_id: ExecutionContextId) -> Result<(), BcibError> {
        if self.revoked {
            return Err(BcibError::AbdfHandleRevoked(
                "ABDF handle has been revoked; access denied",
            ));
        }
        if requesting_context_id != self.context_id {
            return Err(BcibError::AbdfAccessDenied(
                "cross-context ABDF handle access without capability token; \
                 BCIB_ERR_ABDF_ACCESS_DENIED",
            ));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ABDF storage semantics guard (Requirement 22.3)
// ---------------------------------------------------------------------------

/// Assert that a BCIB opcode is NOT attempting to modify ABDF storage semantics.
///
/// BCIB opcodes may read and write data through `AbdfHandle`, but they MUST NOT
/// alter the storage semantics (encoding, layout, ownership rules) of the
/// underlying ABDF object. This function is the enforcement point.
///
/// Call this before any opcode that could be interpreted as a storage-semantics
/// mutation. If the opcode is flagged as a semantics-mutation attempt, this
/// returns `ABDF_BOUNDARY_VIOLATION` (fail-closed).
///
/// # Parameters
///
/// - `is_storage_semantics_mutation`: `true` if the opcode attempts to change
///   ABDF storage semantics (e.g. re-encoding, re-layout, ownership transfer
///   outside ABDF contract).
pub fn assert_no_storage_semantics_mutation(
    is_storage_semantics_mutation: bool,
) -> Result<(), BcibError> {
    if is_storage_semantics_mutation {
        Err(BcibError::AbdfBoundaryViolation(
            "BCIB opcode attempted to modify ABDF storage semantics; \
             ABDF_BOUNDARY_VIOLATION — fail-closed",
        ))
    } else {
        Ok(())
    }
}

/// Assert that a BCIB instruction is NOT attempting to store data outside ABDF.
///
/// Requirement 22.6: Any BCIB instruction that attempts to store data outside
/// the ABDF boundary (i.e. not through an `AbdfHandle` / `AbdfAccessInterface`)
/// MUST be rejected with `ABDF_BOUNDARY_VIOLATION`; fail-closed.
///
/// This is the runtime enforcement point for out-of-ABDF storage attempts.
/// Call this whenever a BCIB instruction tries to persist data to a location
/// that is not an ABDF-managed object.
///
/// # Parameters
///
/// - `is_out_of_abdf_storage`: `true` if the instruction attempts to store
///   data outside the ABDF boundary (e.g. raw memory write, direct file I/O,
///   or any storage path that bypasses `AbdfHandle`).
pub fn assert_no_out_of_abdf_storage(is_out_of_abdf_storage: bool) -> Result<(), BcibError> {
    if is_out_of_abdf_storage {
        Err(BcibError::AbdfBoundaryViolation(
            "BCIB instruction attempted to store data outside ABDF boundary; \
             ABDF_BOUNDARY_VIOLATION — fail-closed",
        ))
    } else {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Helper: a mock AbdfAccessInterface that records calls and can be
    // configured to fail or block.
    // -----------------------------------------------------------------------

    #[derive(Debug)]
    struct MockAbdfAccess {
        /// If `Some(err)`, `read()` returns that error.
        read_error: Option<BcibError>,
        /// If `Some(err)`, `write()` returns that error.
        write_error: Option<BcibError>,
        /// Whether `is_blocking()` returns true.
        blocking: bool,
    }

    impl MockAbdfAccess {
        fn ok() -> Self {
            Self { read_error: None, write_error: None, blocking: false }
        }

        fn blocking() -> Self {
            Self { read_error: None, write_error: None, blocking: true }
        }

        fn denied() -> Self {
            Self {
                read_error: Some(BcibError::AbdfAccessDenied("mock denied")),
                write_error: Some(BcibError::AbdfAccessDenied("mock denied")),
                blocking: false,
            }
        }
    }

    impl AbdfAccessInterface for MockAbdfAccess {
        fn read(
            &self,
            _handle_id: u64,
            _context_id: ExecutionContextId,
        ) -> Result<Vec<u8>, BcibError> {
            if let Some(ref e) = self.read_error {
                return Err(e.clone());
            }
            Ok(vec![0xAB, 0xCD])
        }

        fn write(
            &self,
            _handle_id: u64,
            _context_id: ExecutionContextId,
            _data: &[u8],
        ) -> Result<(), BcibError> {
            if let Some(ref e) = self.write_error {
                return Err(e.clone());
            }
            Ok(())
        }

        fn is_blocking(&self, _handle_id: u64) -> bool {
            self.blocking
        }
    }

    // -----------------------------------------------------------------------
    // Req 22.1 — access only via interface; no raw pointer
    // -----------------------------------------------------------------------

    #[test]
    fn read_via_interface_succeeds() {
        let handle = AbdfHandle::new(1, 42, Box::new(MockAbdfAccess::ok()));
        let data = handle.read(1).expect("read should succeed");
        assert_eq!(data, vec![0xAB, 0xCD]);
    }

    #[test]
    fn write_via_interface_succeeds() {
        let handle = AbdfHandle::new(1, 42, Box::new(MockAbdfAccess::ok()));
        handle.write(1, &[0x01, 0x02]).expect("write should succeed");
    }

    // -----------------------------------------------------------------------
    // Req 22.2 — context-bound; cross-context → BCIB_ERR_ABDF_ACCESS_DENIED
    // -----------------------------------------------------------------------

    #[test]
    fn cross_context_read_denied() {
        let handle = AbdfHandle::new(1, 42, Box::new(MockAbdfAccess::ok()));
        let err = handle.read(99).expect_err("cross-context read must be denied");
        assert_eq!(err, BcibError::AbdfAccessDenied(
            "cross-context ABDF handle access without capability token; \
             BCIB_ERR_ABDF_ACCESS_DENIED",
        ));
    }

    #[test]
    fn cross_context_write_denied() {
        let handle = AbdfHandle::new(1, 42, Box::new(MockAbdfAccess::ok()));
        let err = handle.write(99, &[]).expect_err("cross-context write must be denied");
        assert_eq!(err, BcibError::AbdfAccessDenied(
            "cross-context ABDF handle access without capability token; \
             BCIB_ERR_ABDF_ACCESS_DENIED",
        ));
    }

    // -----------------------------------------------------------------------
    // Revocation — revoked handle → BCIB_ERR_ABDF_HANDLE_REVOKED
    // -----------------------------------------------------------------------

    #[test]
    fn revoked_handle_read_denied() {
        let mut handle = AbdfHandle::new(1, 42, Box::new(MockAbdfAccess::ok()));
        handle.revoke();
        assert!(handle.is_revoked());
        let err = handle.read(1).expect_err("revoked handle read must fail");
        assert_eq!(err, BcibError::AbdfHandleRevoked(
            "ABDF handle has been revoked; access denied",
        ));
    }

    #[test]
    fn revoked_handle_write_denied() {
        let mut handle = AbdfHandle::new(1, 42, Box::new(MockAbdfAccess::ok()));
        handle.revoke();
        let err = handle.write(1, &[]).expect_err("revoked handle write must fail");
        assert_eq!(err, BcibError::AbdfHandleRevoked(
            "ABDF handle has been revoked; access denied",
        ));
    }

    #[test]
    fn revoked_handle_is_not_blocking() {
        let mut handle = AbdfHandle::new(1, 42, Box::new(MockAbdfAccess::blocking()));
        // Before revocation, blocking.
        assert!(handle.is_blocking());
        handle.revoke();
        // After revocation, not blocking (it will just error on access).
        assert!(!handle.is_blocking());
    }

    // -----------------------------------------------------------------------
    // Req 22.3 — BCIB opcodes cannot change ABDF storage semantics
    // -----------------------------------------------------------------------

    #[test]
    fn storage_semantics_mutation_is_boundary_violation() {
        let err = assert_no_storage_semantics_mutation(true)
            .expect_err("storage semantics mutation must be ABDF_BOUNDARY_VIOLATION");
        assert_eq!(
            err,
            BcibError::AbdfBoundaryViolation(
                "BCIB opcode attempted to modify ABDF storage semantics; \
                 ABDF_BOUNDARY_VIOLATION — fail-closed",
            )
        );
    }

    #[test]
    fn non_mutation_opcode_passes() {
        assert_no_storage_semantics_mutation(false)
            .expect("non-mutation opcode must not produce ABDF_BOUNDARY_VIOLATION");
    }

    // -----------------------------------------------------------------------
    // Req 22.6 — out-of-ABDF storage → ABDF_BOUNDARY_VIOLATION (Task 23)
    // -----------------------------------------------------------------------

    #[test]
    fn out_of_abdf_storage_is_boundary_violation() {
        let err = assert_no_out_of_abdf_storage(true)
            .expect_err("out-of-ABDF storage must be ABDF_BOUNDARY_VIOLATION");
        assert_eq!(
            err,
            BcibError::AbdfBoundaryViolation(
                "BCIB instruction attempted to store data outside ABDF boundary; \
                 ABDF_BOUNDARY_VIOLATION — fail-closed",
            )
        );
    }

    #[test]
    fn in_abdf_storage_passes() {
        assert_no_out_of_abdf_storage(false)
            .expect("in-ABDF storage must not produce ABDF_BOUNDARY_VIOLATION");
    }

    // -----------------------------------------------------------------------
    // Task 23 — access_data() capability enforcement (Requirements 22.3, 22.4)
    // -----------------------------------------------------------------------

    use crate::capability_manager::{CapabilityCheck, CapabilityResource};
    use crate::types::{CapabilityTokenId, SideEffectClass};

    /// A capability manager that always denies.
    struct DenyAllCapabilityManager;
    impl CapabilityCheck for DenyAllCapabilityManager {
        fn check(
            &self,
            _token_id: CapabilityTokenId,
            _resource: &CapabilityResource,
            _ctx_id: ExecutionContextId,
        ) -> Result<(), BcibError> {
            Err(BcibError::CapabilityDenied("deny-all capability manager"))
        }
    }

    /// A capability manager that always allows.
    struct AllowAllCapabilityManager;
    impl CapabilityCheck for AllowAllCapabilityManager {
        fn check(
            &self,
            _token_id: CapabilityTokenId,
            _resource: &CapabilityResource,
            _ctx_id: ExecutionContextId,
        ) -> Result<(), BcibError> {
            Ok(())
        }
    }

    #[test]
    fn access_data_pure_no_capability_check_required() {
        // Pure instructions do not require a capability check — even with a
        // deny-all manager, access_data() must succeed.
        let handle = AbdfHandle::new(1, 42, Box::new(MockAbdfAccess::ok()));
        let mgr = DenyAllCapabilityManager;
        let result = handle.access_data(1, SideEffectClass::Pure, 0, &mgr, None);
        assert!(result.is_ok(), "Pure instruction must not require capability check");
    }

    #[test]
    fn access_data_data_mutating_denied_when_capability_fails() {
        // DataMutating instruction with a deny-all manager → BCIB_ERR_ABDF_ACCESS_DENIED.
        let handle = AbdfHandle::new(1, 42, Box::new(MockAbdfAccess::ok()));
        let mgr = DenyAllCapabilityManager;
        let err = handle
            .access_data(1, SideEffectClass::DataMutating, 99, &mgr, Some(&[0x01]))
            .expect_err("DataMutating with denied capability must fail");
        assert_eq!(
            err,
            BcibError::AbdfAccessDenied(
                "capability check failed for DataMutating ABDF access; \
                 BCIB_ERR_ABDF_ACCESS_DENIED",
            )
        );
    }

    #[test]
    fn access_data_external_denied_when_capability_fails() {
        // External instruction with a deny-all manager → BCIB_ERR_ABDF_ACCESS_DENIED.
        let handle = AbdfHandle::new(1, 42, Box::new(MockAbdfAccess::ok()));
        let mgr = DenyAllCapabilityManager;
        let err = handle
            .access_data(1, SideEffectClass::External, 99, &mgr, None)
            .expect_err("External with denied capability must fail");
        assert_eq!(
            err,
            BcibError::AbdfAccessDenied(
                "capability check failed for External ABDF access; \
                 BCIB_ERR_ABDF_ACCESS_DENIED",
            )
        );
    }

    #[test]
    fn access_data_data_mutating_succeeds_when_capability_passes() {
        // DataMutating instruction with an allow-all manager → success.
        let handle = AbdfHandle::new(1, 42, Box::new(MockAbdfAccess::ok()));
        let mgr = AllowAllCapabilityManager;
        let result = handle.access_data(1, SideEffectClass::DataMutating, 1, &mgr, Some(&[0xAA]));
        assert!(result.is_ok(), "DataMutating with valid capability must succeed");
    }

    #[test]
    fn access_data_external_succeeds_when_capability_passes() {
        // External instruction with an allow-all manager → success (read).
        let handle = AbdfHandle::new(1, 42, Box::new(MockAbdfAccess::ok()));
        let mgr = AllowAllCapabilityManager;
        let result = handle.access_data(1, SideEffectClass::External, 1, &mgr, None);
        assert!(result.is_ok(), "External with valid capability must succeed");
    }

    #[test]
    fn access_data_revoked_handle_denied_before_capability_check() {
        // Revoked handle must be rejected before capability check is even attempted.
        let mut handle = AbdfHandle::new(1, 42, Box::new(MockAbdfAccess::ok()));
        handle.revoke();
        // Even with allow-all manager, revoked handle must fail.
        let mgr = AllowAllCapabilityManager;
        let err = handle
            .access_data(1, SideEffectClass::DataMutating, 1, &mgr, None)
            .expect_err("revoked handle must be denied");
        assert!(matches!(err, BcibError::AbdfHandleRevoked(_)));
    }

    #[test]
    fn access_data_cross_context_denied_before_capability_check() {
        // Cross-context access must be rejected before capability check.
        let handle = AbdfHandle::new(1, 42, Box::new(MockAbdfAccess::ok()));
        let mgr = AllowAllCapabilityManager;
        let err = handle
            .access_data(99, SideEffectClass::DataMutating, 1, &mgr, None)
            .expect_err("cross-context access must be denied");
        assert!(matches!(err, BcibError::AbdfAccessDenied(_)));
    }

    // -----------------------------------------------------------------------
    // Stub constructor backward compatibility
    // -----------------------------------------------------------------------

    #[test]
    fn stub_constructor_is_valid_handle() {
        let handle = AbdfHandle::stub(5, 100);
        assert_eq!(handle.context_id, 5);
        assert_eq!(handle.handle_id, 100);
        assert!(!handle.is_revoked());
        // Stub reads return empty data (NoopAbdfAccess).
        let data = handle.read(5).expect("stub read should succeed");
        assert!(data.is_empty());
    }

    #[test]
    fn stub_cross_context_still_denied() {
        let handle = AbdfHandle::stub(5, 100);
        let err = handle.read(6).expect_err("cross-context stub read must be denied");
        assert!(matches!(err, BcibError::AbdfAccessDenied(_)));
    }

    // -----------------------------------------------------------------------
    // Interface-level denial propagates correctly
    // -----------------------------------------------------------------------

    #[test]
    fn interface_denial_propagates() {
        let handle = AbdfHandle::new(1, 42, Box::new(MockAbdfAccess::denied()));
        let err = handle.read(1).expect_err("interface denial must propagate");
        assert!(matches!(err, BcibError::AbdfAccessDenied(_)));
    }

    // -----------------------------------------------------------------------
    // Property 10 — ABDF Boundary (proptest)
    // Feature: phase15-bcib-execution-engine, Property 10: ABDF Boundary
    // Validates: Requirements 22.2, 22.3, 22.4, 23.3
    // -----------------------------------------------------------------------

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(proptest::test_runner::Config::with_cases(200))]

            /// Property 10a: Any cross-context access attempt (bypass) →
            /// BCIB_ERR_ABDF_ACCESS_DENIED.
            ///
            /// Feature: phase15-bcib-execution-engine, Property 10: ABDF Boundary
            /// Validates: Requirements 22.2, 22.3, 22.4, 23.3
            #[test]
            fn prop_cross_context_access_denied(
                owner_ctx in 1u64..=1000u64,
                // requesting_ctx is always different from owner_ctx
                offset in 1u64..=1000u64,
                handle_id in 0u64..=u64::MAX,
            ) {
                let requesting_ctx = owner_ctx.wrapping_add(offset);
                // Ensure they differ (wrapping_add with offset ≥ 1 guarantees this
                // unless overflow wraps back to owner_ctx, which is astronomically
                // unlikely in the 1..=1000 range).
                prop_assume!(requesting_ctx != owner_ctx);

                let handle = AbdfHandle::new(owner_ctx, handle_id, Box::new(NoopAbdfAccess));
                let read_err = handle.read(requesting_ctx)
                    .expect_err("cross-context read must be denied");
                prop_assert!(
                    matches!(read_err, BcibError::AbdfAccessDenied(_)),
                    "expected AbdfAccessDenied, got {:?}", read_err
                );

                let handle2 = AbdfHandle::new(owner_ctx, handle_id, Box::new(NoopAbdfAccess));
                let write_err = handle2.write(requesting_ctx, &[])
                    .expect_err("cross-context write must be denied");
                prop_assert!(
                    matches!(write_err, BcibError::AbdfAccessDenied(_)),
                    "expected AbdfAccessDenied, got {:?}", write_err
                );
            }

            /// Property 10b: Revoked handle access → BCIB_ERR_ABDF_HANDLE_REVOKED.
            ///
            /// Feature: phase15-bcib-execution-engine, Property 10: ABDF Boundary
            /// Validates: Requirements 22.2, 22.3, 22.4, 23.3
            #[test]
            fn prop_revoked_handle_access_denied(
                ctx_id in 1u64..=1000u64,
                handle_id in 0u64..=u64::MAX,
                data in proptest::collection::vec(any::<u8>(), 0..=64),
            ) {
                let mut handle = AbdfHandle::new(ctx_id, handle_id, Box::new(NoopAbdfAccess));
                handle.revoke();

                let read_err = handle.read(ctx_id)
                    .expect_err("revoked handle read must fail");
                prop_assert!(
                    matches!(read_err, BcibError::AbdfHandleRevoked(_)),
                    "expected AbdfHandleRevoked, got {:?}", read_err
                );

                let mut handle2 = AbdfHandle::new(ctx_id, handle_id, Box::new(NoopAbdfAccess));
                handle2.revoke();
                let write_err = handle2.write(ctx_id, &data)
                    .expect_err("revoked handle write must fail");
                prop_assert!(
                    matches!(write_err, BcibError::AbdfHandleRevoked(_)),
                    "expected AbdfHandleRevoked, got {:?}", write_err
                );
            }

            /// Property 10c: Storage semantics mutation always → ABDF_BOUNDARY_VIOLATION.
            ///
            /// Feature: phase15-bcib-execution-engine, Property 10: ABDF Boundary
            /// Validates: Requirements 22.3
            #[test]
            fn prop_storage_semantics_mutation_always_violation(
                _dummy in any::<u8>(),
            ) {
                let err = assert_no_storage_semantics_mutation(true)
                    .expect_err("storage semantics mutation must always be a violation");
                prop_assert!(
                    matches!(err, BcibError::AbdfBoundaryViolation(_)),
                    "expected AbdfBoundaryViolation, got {:?}", err
                );
            }

            /// Property 10d: Same-context, non-revoked access always succeeds
            /// (with NoopAbdfAccess).
            ///
            /// Feature: phase15-bcib-execution-engine, Property 10: ABDF Boundary
            /// Validates: Requirements 22.1, 22.2
            #[test]
            fn prop_same_context_non_revoked_access_succeeds(
                ctx_id in 1u64..=1000u64,
                handle_id in 0u64..=u64::MAX,
                data in proptest::collection::vec(any::<u8>(), 0..=64),
            ) {
                let handle = AbdfHandle::new(ctx_id, handle_id, Box::new(NoopAbdfAccess));
                prop_assert!(
                    handle.read(ctx_id).is_ok(),
                    "same-context non-revoked read must succeed"
                );

                let handle2 = AbdfHandle::new(ctx_id, handle_id, Box::new(NoopAbdfAccess));
                prop_assert!(
                    handle2.write(ctx_id, &data).is_ok(),
                    "same-context non-revoked write must succeed"
                );
            }
        }
    }
}

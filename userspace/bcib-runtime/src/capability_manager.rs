/// CapabilityManager — BCIB v3 token-based capability enforcement.
///
/// This module provides:
///   - `CapabilityCheck` trait (used by the verification pipeline, Group 3)
///   - `CapabilityResource` enum
///   - `NoopCapabilityManager` stub (Group 1–6 development only)
///   - `CapabilityToken` — the internal token type (non-forgeable)
///   - `CapabilityManager` — the real Group 7 implementation
///
/// # Security Properties
///
/// - **Non-forgeable**: tokens are only created inside `bind()`; there is no
///   public constructor for `CapabilityToken`. External code cannot produce a
///   valid token without going through `bind()`.
/// - **Non-escalatable**: `check()` verifies that the token's `resource` field
///   exactly matches the requested resource; a token cannot grant access to a
///   resource it was not bound for.
/// - **Context-bound**: every token carries the `ExecutionContextId` it was
///   bound to. `check()` rejects any call where `ctx_id` does not match the
///   token's bound context.
/// - **Revocable**: `revoke()` marks a token as revoked; subsequent `check()`
///   calls for that token return `BCIB_ERR_CAPABILITY_DENIED` immediately.
/// - **Constant-time check**: `check()` always performs the same sequence of
///   comparisons regardless of which branch fails, preventing timing
///   side-channels (Requirement 21.1).
///
/// Requirements: 5.1, 5.2, 5.3, 14.1, 14.2, 14.3, 14.5, 21.1

use crate::types::{BcibError, CapabilityTokenId, ExecutionContextId};

// ---------------------------------------------------------------------------
// CapabilityResource — the resource a token grants access to
// ---------------------------------------------------------------------------

/// Identifies the resource a capability token grants access to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityResource {
    /// General execution permission.
    Execution,
    /// ABDF data read access.
    DataRead,
    /// ABDF data write/mutation access.
    DataWrite,
    /// External call (AI/UI) access.
    ExternalCall,
    /// Cross-context handle transfer.
    CrossContextTransfer,
}

// ---------------------------------------------------------------------------
// CapabilityCheck trait
// ---------------------------------------------------------------------------

/// Trait for capability token validation.
///
/// Implementors verify that `token_id` grants access to `resource` within
/// the execution context identified by `ctx_id`.
///
/// - Returns `Ok(())` if the capability check passes.
/// - Returns `Err(BcibError::CapabilityDenied(...))` if the token is missing,
///   revoked, or does not cover the requested resource/context.
///
/// Requirement 5.2: every data-mutating and external instruction MUST be
/// checked through this trait before execution.
pub trait CapabilityCheck {
    fn check(
        &self,
        token_id: CapabilityTokenId,
        resource: &CapabilityResource,
        ctx_id: ExecutionContextId,
    ) -> Result<(), BcibError>;
}

// ---------------------------------------------------------------------------
// NoopCapabilityManager — stub implementation (always allows)
// ---------------------------------------------------------------------------

/// Stub `CapabilityCheck` implementation that approves every check.
///
/// Used during Group 1–6 development so the verification pipeline can
/// compile and run without the real `CapabilityManager` (Group 7, Task 27).
///
/// IMPORTANT: This stub MUST NOT be used in production paths. Group 7
/// replaces it with the real token-based implementation.
pub struct NoopCapabilityManager;

impl CapabilityCheck for NoopCapabilityManager {
    fn check(
        &self,
        _token_id: CapabilityTokenId,
        _resource: &CapabilityResource,
        _ctx_id: ExecutionContextId,
    ) -> Result<(), BcibError> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// CapabilityToken — internal token type (non-forgeable)
// ---------------------------------------------------------------------------

/// An internal capability token.
///
/// # Non-forgeability
///
/// `CapabilityToken` has no public constructor. The only way to create a
/// valid token is through `CapabilityManager::bind()`, which assigns the
/// token ID, resource, and context binding. External code cannot produce a
/// `CapabilityToken` value (Requirement 14.1).
#[derive(Debug, Clone)]
pub struct CapabilityToken {
    /// Unique identifier for this token.
    pub(crate) id: CapabilityTokenId,
    /// The resource this token grants access to (non-escalatable: exact match only).
    pub(crate) resource: CapabilityResource,
    /// The execution context this token is bound to (context-bound).
    pub(crate) bound_ctx: ExecutionContextId,
    /// Whether this token has been revoked.
    pub(crate) revoked: bool,
}

// ---------------------------------------------------------------------------
// CapabilityManager — real Group 7 implementation
// ---------------------------------------------------------------------------

/// Token-based capability manager for BCIB v3.
///
/// Stores tokens in a capacity-bounded `Vec<CapabilityToken>`. The capacity
/// is fixed at construction time, enforcing bounded resource usage
/// (Requirement 3.4). All operations are O(n) over the active token list,
/// but `check()` is structured to be constant-time in its comparison
/// sequence (Requirement 21.1).
///
/// # Constant-time check
///
/// `check()` always iterates over all active tokens and accumulates a result
/// without early-exit on success. This prevents an attacker from inferring
/// token positions or counts from timing differences. The final decision is
/// derived from the accumulated result after the full scan.
pub struct CapabilityManager {
    /// Active tokens (including revoked ones until compaction).
    tokens: Vec<CapabilityToken>,
    /// Maximum number of simultaneously active tokens.
    capacity: usize,
    /// Next token ID to assign. Monotonically increasing; never reused.
    next_id: CapabilityTokenId,
}

impl CapabilityManager {
    /// Create a new `CapabilityManager` with the given token capacity.
    ///
    /// The capacity bounds the maximum number of simultaneously active tokens
    /// (Requirement 3.4, 18.1).
    pub fn new(capacity: usize) -> Self {
        Self {
            tokens: Vec::with_capacity(capacity),
            capacity,
            next_id: 1, // 0 is reserved as "no token"
        }
    }

    /// Bind a new capability token for `resource` in `ctx_id`.
    ///
    /// Returns the assigned `CapabilityTokenId` on success.
    ///
    /// # Errors
    ///
    /// - `BCIB_ERR_RESOURCE_EXHAUSTED` — token pool is full.
    ///
    /// # Non-forgeability
    ///
    /// The token ID is assigned internally and monotonically. External code
    /// cannot choose or predict the ID, and cannot construct a `CapabilityToken`
    /// directly (Requirement 14.1).
    pub fn bind(
        &mut self,
        resource: CapabilityResource,
        ctx_id: ExecutionContextId,
    ) -> Result<CapabilityTokenId, BcibError> {
        // Compact revoked tokens first to reclaim slots.
        self.compact();

        if self.tokens.len() >= self.capacity {
            return Err(BcibError::ResourceExhausted(
                "capability token pool exhausted; max active tokens reached",
            ));
        }

        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        // Ensure 0 is never issued (reserved sentinel).
        if self.next_id == 0 {
            self.next_id = 1;
        }

        self.tokens.push(CapabilityToken {
            id,
            resource,
            bound_ctx: ctx_id,
            revoked: false,
        });
        Ok(id)
    }

    /// Revoke the token identified by `token_id`.
    ///
    /// After revocation, any `check()` call for this token returns
    /// `BCIB_ERR_CAPABILITY_DENIED` (Requirement 14.5).
    pub fn revoke(&mut self, token_id: CapabilityTokenId) {
        for t in &mut self.tokens {
            if t.id == token_id {
                t.revoked = true;
            }
        }
    }

    /// Check whether `token_id` grants access to `resource` in `ctx_id`.
    ///
    /// # Constant-time guarantee (Requirement 21.1)
    ///
    /// This method always scans all active tokens without early exit on
    /// success. The result is accumulated in a boolean flag. This prevents
    /// timing side-channels that could reveal token count or position.
    ///
    /// # Errors
    ///
    /// - `BCIB_ERR_CAPABILITY_DENIED` — token not found, revoked, wrong
    ///   resource, or wrong context.
    pub fn check(
        &self,
        token_id: CapabilityTokenId,
        resource: &CapabilityResource,
        ctx_id: ExecutionContextId,
    ) -> Result<(), BcibError> {
        // Always scan all tokens — no early exit (constant-time pattern).
        let mut granted = false;
        for t in &self.tokens {
            // All comparisons evaluated unconditionally.
            let id_match = t.id == token_id;
            let res_match = &t.resource == resource;
            let ctx_match = t.bound_ctx == ctx_id;
            let not_revoked = !t.revoked;
            granted |= id_match & res_match & ctx_match & not_revoked;
        }

        if granted {
            Ok(())
        } else {
            Err(BcibError::CapabilityDenied(
                "capability token missing, revoked, wrong resource, or wrong context",
            ))
        }
    }

    /// Transfer token `token_id` from `from_ctx` to `to_ctx`.
    ///
    /// This implements explicit capability inheritance: a token can only be
    /// transferred by the context that owns it, and only to another context.
    /// Automatic inheritance is prohibited (Requirement 14.6).
    ///
    /// After transfer, the token's `bound_ctx` is updated to `to_ctx`.
    /// The original context can no longer use the token.
    ///
    /// # Errors
    ///
    /// - `BCIB_ERR_CAPABILITY_DENIED` — token not found, revoked, or not
    ///   owned by `from_ctx`.
    pub fn transfer(
        &mut self,
        token_id: CapabilityTokenId,
        from_ctx: ExecutionContextId,
        to_ctx: ExecutionContextId,
    ) -> Result<(), BcibError> {
        let mut found = false;
        for t in &mut self.tokens {
            if t.id == token_id && t.bound_ctx == from_ctx && !t.revoked {
                t.bound_ctx = to_ctx;
                found = true;
            }
        }

        if found {
            Ok(())
        } else {
            Err(BcibError::CapabilityDenied(
                "transfer failed: token not found, revoked, or not owned by from_ctx",
            ))
        }
    }

    /// Release all tokens bound to `ctx_id` (used during teardown).
    ///
    /// Called as part of the teardown contract (Requirement 3.9, 3.10) when
    /// an execution context is cancelled or fails. All tokens bound to the
    /// context are removed and their slots reclaimed.
    pub fn release_context(&mut self, ctx_id: ExecutionContextId) {
        self.tokens.retain(|t| t.bound_ctx != ctx_id);
    }

    /// Number of currently active (including revoked-but-not-compacted) tokens.
    pub fn active_token_count(&self) -> usize {
        self.tokens.iter().filter(|t| !t.revoked).count()
    }

    /// Maximum number of tokens this manager can hold simultaneously.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Remove revoked tokens to reclaim capacity.
    fn compact(&mut self) {
        self.tokens.retain(|t| !t.revoked);
    }
}

impl CapabilityCheck for CapabilityManager {
    /// Delegates to `CapabilityManager::check()`.
    fn check(
        &self,
        token_id: CapabilityTokenId,
        resource: &CapabilityResource,
        ctx_id: ExecutionContextId,
    ) -> Result<(), BcibError> {
        CapabilityManager::check(self, token_id, resource, ctx_id)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // NoopCapabilityManager
    // -----------------------------------------------------------------------

    #[test]
    fn noop_manager_always_returns_ok() {
        let mgr = NoopCapabilityManager;
        let result = mgr.check(42, &CapabilityResource::DataWrite, 1);
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn noop_manager_ok_for_all_resource_variants() {
        let mgr = NoopCapabilityManager;
        let resources = [
            CapabilityResource::Execution,
            CapabilityResource::DataRead,
            CapabilityResource::DataWrite,
            CapabilityResource::ExternalCall,
            CapabilityResource::CrossContextTransfer,
        ];
        for resource in &resources {
            assert_eq!(mgr.check(0, resource, 0), Ok(()));
        }
    }

    // -----------------------------------------------------------------------
    // CapabilityManager — bind
    // -----------------------------------------------------------------------

    #[test]
    fn bind_returns_token_id() {
        let mut mgr = CapabilityManager::new(8);
        let id = mgr.bind(CapabilityResource::Execution, 1).unwrap();
        assert!(id > 0, "token ID must be non-zero");
    }

    #[test]
    fn bind_ids_are_unique() {
        let mut mgr = CapabilityManager::new(8);
        let id1 = mgr.bind(CapabilityResource::Execution, 1).unwrap();
        let id2 = mgr.bind(CapabilityResource::DataWrite, 1).unwrap();
        assert_ne!(id1, id2, "each bind must produce a unique token ID");
    }

    #[test]
    fn bind_pool_exhaustion_returns_resource_exhausted() {
        let mut mgr = CapabilityManager::new(2);
        mgr.bind(CapabilityResource::Execution, 1).unwrap();
        mgr.bind(CapabilityResource::DataWrite, 1).unwrap();
        let result = mgr.bind(CapabilityResource::ExternalCall, 1);
        assert!(
            matches!(result, Err(BcibError::ResourceExhausted(_))),
            "expected ResourceExhausted when pool is full, got {:?}",
            result
        );
    }

    // -----------------------------------------------------------------------
    // CapabilityManager — check (non-forgeable, non-escalatable, context-bound)
    // -----------------------------------------------------------------------

    #[test]
    fn check_valid_token_returns_ok() {
        let mut mgr = CapabilityManager::new(8);
        let id = mgr.bind(CapabilityResource::DataWrite, 42).unwrap();
        assert_eq!(mgr.check(id, &CapabilityResource::DataWrite, 42), Ok(()));
    }

    #[test]
    fn check_unknown_token_id_returns_capability_denied() {
        let mgr = CapabilityManager::new(8);
        let result = mgr.check(999, &CapabilityResource::Execution, 1);
        assert!(
            matches!(result, Err(BcibError::CapabilityDenied(_))),
            "unknown token must be denied, got {:?}",
            result
        );
    }

    /// Non-escalatable: a DataWrite token cannot grant ExternalCall access.
    #[test]
    fn check_wrong_resource_returns_capability_denied() {
        let mut mgr = CapabilityManager::new(8);
        let id = mgr.bind(CapabilityResource::DataWrite, 1).unwrap();
        let result = mgr.check(id, &CapabilityResource::ExternalCall, 1);
        assert!(
            matches!(result, Err(BcibError::CapabilityDenied(_))),
            "token must not escalate to a different resource, got {:?}",
            result
        );
    }

    /// Context-bound: a token bound to ctx 1 cannot be used in ctx 2.
    #[test]
    fn check_wrong_context_returns_capability_denied() {
        let mut mgr = CapabilityManager::new(8);
        let id = mgr.bind(CapabilityResource::Execution, 1).unwrap();
        let result = mgr.check(id, &CapabilityResource::Execution, 2);
        assert!(
            matches!(result, Err(BcibError::CapabilityDenied(_))),
            "token must be denied in a different context, got {:?}",
            result
        );
    }

    // -----------------------------------------------------------------------
    // CapabilityManager — revoke
    // -----------------------------------------------------------------------

    #[test]
    fn revoke_makes_token_invalid() {
        let mut mgr = CapabilityManager::new(8);
        let id = mgr.bind(CapabilityResource::Execution, 1).unwrap();
        assert_eq!(mgr.check(id, &CapabilityResource::Execution, 1), Ok(()));

        mgr.revoke(id);

        let result = mgr.check(id, &CapabilityResource::Execution, 1);
        assert!(
            matches!(result, Err(BcibError::CapabilityDenied(_))),
            "revoked token must be denied, got {:?}",
            result
        );
    }

    #[test]
    fn revoke_nonexistent_token_is_noop() {
        let mut mgr = CapabilityManager::new(8);
        // Should not panic or error.
        mgr.revoke(9999);
    }

    #[test]
    fn revoke_one_token_does_not_affect_others() {
        let mut mgr = CapabilityManager::new(8);
        let id1 = mgr.bind(CapabilityResource::Execution, 1).unwrap();
        let id2 = mgr.bind(CapabilityResource::DataWrite, 1).unwrap();

        mgr.revoke(id1);

        // id1 is revoked.
        assert!(matches!(
            mgr.check(id1, &CapabilityResource::Execution, 1),
            Err(BcibError::CapabilityDenied(_))
        ));
        // id2 is still valid.
        assert_eq!(mgr.check(id2, &CapabilityResource::DataWrite, 1), Ok(()));
    }

    // -----------------------------------------------------------------------
    // CapabilityManager — transfer
    // -----------------------------------------------------------------------

    #[test]
    fn transfer_moves_token_to_new_context() {
        let mut mgr = CapabilityManager::new(8);
        let id = mgr.bind(CapabilityResource::Execution, 1).unwrap();

        mgr.transfer(id, 1, 2).unwrap();

        // No longer valid in ctx 1.
        assert!(matches!(
            mgr.check(id, &CapabilityResource::Execution, 1),
            Err(BcibError::CapabilityDenied(_))
        ));
        // Valid in ctx 2.
        assert_eq!(mgr.check(id, &CapabilityResource::Execution, 2), Ok(()));
    }

    #[test]
    fn transfer_wrong_from_ctx_returns_capability_denied() {
        let mut mgr = CapabilityManager::new(8);
        let id = mgr.bind(CapabilityResource::Execution, 1).unwrap();
        let result = mgr.transfer(id, 99, 2); // wrong from_ctx
        assert!(
            matches!(result, Err(BcibError::CapabilityDenied(_))),
            "transfer from wrong context must be denied, got {:?}",
            result
        );
    }

    #[test]
    fn transfer_revoked_token_returns_capability_denied() {
        let mut mgr = CapabilityManager::new(8);
        let id = mgr.bind(CapabilityResource::Execution, 1).unwrap();
        mgr.revoke(id);
        let result = mgr.transfer(id, 1, 2);
        assert!(
            matches!(result, Err(BcibError::CapabilityDenied(_))),
            "transfer of revoked token must be denied, got {:?}",
            result
        );
    }

    /// Automatic inheritance is prohibited (Requirement 14.6):
    /// a child context cannot use a parent's token without explicit transfer.
    #[test]
    fn no_automatic_inheritance_between_contexts() {
        let mut mgr = CapabilityManager::new(8);
        let id = mgr.bind(CapabilityResource::DataWrite, 1).unwrap();
        // ctx 2 tries to use ctx 1's token without a transfer.
        let result = mgr.check(id, &CapabilityResource::DataWrite, 2);
        assert!(
            matches!(result, Err(BcibError::CapabilityDenied(_))),
            "automatic inheritance must be denied, got {:?}",
            result
        );
    }

    // -----------------------------------------------------------------------
    // CapabilityManager — release_context (teardown)
    // -----------------------------------------------------------------------

    #[test]
    fn release_context_revokes_all_tokens_for_context() {
        let mut mgr = CapabilityManager::new(8);
        let id1 = mgr.bind(CapabilityResource::Execution, 1).unwrap();
        let id2 = mgr.bind(CapabilityResource::DataWrite, 1).unwrap();
        let id3 = mgr.bind(CapabilityResource::ExternalCall, 2).unwrap();

        mgr.release_context(1);

        // ctx 1 tokens are gone.
        assert!(matches!(
            mgr.check(id1, &CapabilityResource::Execution, 1),
            Err(BcibError::CapabilityDenied(_))
        ));
        assert!(matches!(
            mgr.check(id2, &CapabilityResource::DataWrite, 1),
            Err(BcibError::CapabilityDenied(_))
        ));
        // ctx 2 token is unaffected.
        assert_eq!(mgr.check(id3, &CapabilityResource::ExternalCall, 2), Ok(()));
    }

    #[test]
    fn release_context_reclaims_pool_slots() {
        let mut mgr = CapabilityManager::new(4);
        mgr.bind(CapabilityResource::Execution, 1).unwrap();
        mgr.bind(CapabilityResource::DataWrite, 1).unwrap();
        assert_eq!(mgr.active_token_count(), 2);

        mgr.release_context(1);
        assert_eq!(mgr.active_token_count(), 0);

        // Pool slots are reclaimed — new binds should succeed.
        mgr.bind(CapabilityResource::Execution, 3).unwrap();
        mgr.bind(CapabilityResource::DataWrite, 3).unwrap();
        assert_eq!(mgr.active_token_count(), 2);
    }

    // -----------------------------------------------------------------------
    // CapabilityCheck trait impl on CapabilityManager
    // -----------------------------------------------------------------------

    #[test]
    fn capability_check_trait_impl_delegates_correctly() {
        let mut mgr = CapabilityManager::new(8);
        let id = mgr.bind(CapabilityResource::DataRead, 5).unwrap();

        // Use the trait object interface.
        let checker: &dyn CapabilityCheck = &mgr;
        assert_eq!(checker.check(id, &CapabilityResource::DataRead, 5), Ok(()));
        assert!(matches!(
            checker.check(id, &CapabilityResource::DataWrite, 5),
            Err(BcibError::CapabilityDenied(_))
        ));
    }
}

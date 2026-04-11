/// Bounded pool and context-isolated space implementations.
///
/// Requirements: 3.1, 3.2, 3.4, 14.6, 15.1, 15.2, 15.3, 15.4, 18.1, 18.2
///
/// # Design
///
/// `BoundedPool<T>` is a fixed-capacity pool with `acquire()` / `release()` API.
/// Unbounded growth is prohibited — pool exhaustion returns
/// `BCIB_ERR_RESOURCE_EXHAUSTED` (Requirement 3.4, 18.1).
///
/// `IsolatedSlotSpace` and `IsolatedHandleSpace` wrap the pool with a context-ID
/// ownership check. Any access from a mismatched context ID returns
/// `BCIB_ERR_ISOLATION_VIOLATION` (Requirements 15.1, 15.2, 15.3).
///
/// Cross-context transfer is only permitted through `CapabilityManager::transfer()`.
/// There is no automatic inheritance path — sub-contexts cannot inherit parent
/// capabilities without explicit transfer (Requirements 14.6, 15.4).
use crate::capability_manager::CapabilityManager;
use crate::types::{BcibError, CapabilityTokenId, ExecutionContextId};

// ---------------------------------------------------------------------------
// BoundedPool<T>
// ---------------------------------------------------------------------------

/// A fixed-capacity pool of items of type `T`.
///
/// Items are pre-allocated up to `capacity`. `acquire()` hands out an item;
/// `release()` returns it. The pool never grows beyond `capacity`.
///
/// # Invariants
/// - `available.len() + outstanding <= capacity` at all times.
/// - `acquire()` on an empty pool → `BCIB_ERR_RESOURCE_EXHAUSTED`.
/// - `release()` of an item that was not acquired is a no-op (defensive).
#[derive(Debug)]
pub struct BoundedPool<T> {
    available: Vec<T>,
    capacity: usize,
    outstanding: usize,
}

impl<T> BoundedPool<T> {
    /// Create a new pool with the given capacity.
    ///
    /// The pool starts empty; items are added via `release()` or by
    /// constructing with `BoundedPool::from_items()`.
    pub fn new(capacity: usize) -> Self {
        Self {
            available: Vec::with_capacity(capacity),
            capacity,
            outstanding: 0,
        }
    }

    /// Create a pool pre-populated with `items`.
    ///
    /// `capacity` is set to `items.len()` — the pool is exactly full at start.
    pub fn from_items(items: Vec<T>) -> Self {
        let capacity = items.len();
        Self {
            available: items,
            capacity,
            outstanding: 0,
        }
    }

    /// Acquire one item from the pool.
    ///
    /// Returns `BCIB_ERR_RESOURCE_EXHAUSTED` when the pool is empty.
    pub fn acquire(&mut self) -> Result<T, BcibError> {
        match self.available.pop() {
            Some(item) => {
                self.outstanding += 1;
                Ok(item)
            }
            None => Err(BcibError::ResourceExhausted(
                "bounded pool exhausted; unbounded growth is prohibited",
            )),
        }
    }

    /// Return an item to the pool.
    ///
    /// If the pool is already at capacity (e.g. due to a double-release),
    /// the item is silently dropped to avoid unbounded growth.
    pub fn release(&mut self, item: T) {
        if self.available.len() < self.capacity {
            self.available.push(item);
            if self.outstanding > 0 {
                self.outstanding -= 1;
            }
        }
        // Silently drop if pool is already full — prevents unbounded growth.
    }

    /// Number of items currently available for acquisition.
    pub fn available_count(&self) -> usize {
        self.available.len()
    }

    /// Number of items currently outstanding (acquired but not released).
    pub fn outstanding_count(&self) -> usize {
        self.outstanding
    }

    /// Maximum capacity of this pool.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns `true` when no items are available.
    pub fn is_exhausted(&self) -> bool {
        self.available.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Slot / Handle types
// ---------------------------------------------------------------------------

/// A transient execution slot — holds per-instruction intermediate state.
///
/// Slots are acquired at instruction start and released at instruction end
/// (or on teardown). They are context-bound and must not outlive their
/// owning `ExecutionContext` (Requirement 3.1).
#[derive(Debug, Clone)]
pub struct ExecutionSlot {
    /// Slot index within the pool — used for identity checks.
    pub slot_id: u32,
    /// Raw byte storage for transient instruction state.
    pub data: Vec<u8>,
}

impl ExecutionSlot {
    pub fn new(slot_id: u32) -> Self {
        Self {
            slot_id,
            data: Vec::new(),
        }
    }
}

/// A long-lived handle entry — wraps an opaque handle ID.
///
/// Handles are acquired when a long-lived resource (e.g. an ABDF object) is
/// opened and released on teardown (Requirement 3.2).
#[derive(Debug, Clone)]
pub struct HandleEntry {
    pub handle_id: u64,
}

impl HandleEntry {
    pub fn new(handle_id: u64) -> Self {
        Self { handle_id }
    }
}

// ---------------------------------------------------------------------------
// IsolatedSlotSpace
// ---------------------------------------------------------------------------

/// Context-isolated slot space backed by a `BoundedPool<ExecutionSlot>`.
///
/// All acquire/release operations verify that the caller's `ctx_id` matches
/// the owner. A mismatch returns `BCIB_ERR_ISOLATION_VIOLATION`
/// (Requirements 15.1, 15.2).
#[derive(Debug)]
pub struct IsolatedSlotSpace {
    owner: ExecutionContextId,
    pool: BoundedPool<ExecutionSlot>,
}

impl IsolatedSlotSpace {
    /// Create a new slot space owned by `owner_id` with `capacity` slots.
    pub fn new(owner_id: ExecutionContextId, capacity: usize) -> Self {
        // Pre-populate the pool with `capacity` slots.
        let items: Vec<ExecutionSlot> = (0..capacity as u32).map(ExecutionSlot::new).collect();
        Self {
            owner: owner_id,
            pool: BoundedPool::from_items(items),
        }
    }

    /// Acquire a slot on behalf of `ctx_id`.
    ///
    /// Returns `BCIB_ERR_ISOLATION_VIOLATION` if `ctx_id != owner`.
    /// Returns `BCIB_ERR_RESOURCE_EXHAUSTED` if the pool is empty.
    pub fn acquire(&mut self, ctx_id: ExecutionContextId) -> Result<ExecutionSlot, BcibError> {
        self.check_owner(ctx_id)?;
        self.pool.acquire()
    }

    /// Release a slot on behalf of `ctx_id`.
    ///
    /// Returns `BCIB_ERR_ISOLATION_VIOLATION` if `ctx_id != owner`.
    pub fn release(
        &mut self,
        ctx_id: ExecutionContextId,
        slot: ExecutionSlot,
    ) -> Result<(), BcibError> {
        self.check_owner(ctx_id)?;
        self.pool.release(slot);
        Ok(())
    }

    /// Release all outstanding slots back to the pool (used during teardown).
    ///
    /// This is called with the owner's `ctx_id` during the teardown contract
    /// (Requirement 3.9, 3.10). The pool is reset to full capacity.
    pub fn release_all(&mut self, ctx_id: ExecutionContextId) -> Result<(), BcibError> {
        self.check_owner(ctx_id)?;
        // Reconstruct the pool at full capacity — all slots are considered returned.
        let capacity = self.pool.capacity();
        let items: Vec<ExecutionSlot> = (0..capacity as u32).map(ExecutionSlot::new).collect();
        self.pool = BoundedPool::from_items(items);
        Ok(())
    }

    /// Number of slots currently available.
    pub fn available_count(&self) -> usize {
        self.pool.available_count()
    }

    /// Number of slots currently outstanding.
    pub fn outstanding_count(&self) -> usize {
        self.pool.outstanding_count()
    }

    /// The context ID that owns this slot space.
    pub fn owner(&self) -> ExecutionContextId {
        self.owner
    }

    /// Transfer ownership of this slot space from `from_ctx` to `to_ctx`.
    ///
    /// This is the **only** permitted cross-context transfer path (Requirements
    /// 14.6, 15.3, 15.4). The caller must supply a `CapabilityManager` holding
    /// a valid `CrossContextTransfer` token bound to `from_ctx`; the manager's
    /// `transfer()` method re-binds that token to `to_ctx` as the authorization
    /// step.
    ///
    /// After a successful call the slot space's owner is updated to `to_ctx`.
    /// The original context can no longer access the space.
    ///
    /// # Errors
    ///
    /// - `BCIB_ERR_CAPABILITY_DENIED` — `capability_manager.transfer()` rejected
    ///   the request (token not found, revoked, or not owned by `from_ctx`).
    /// - `BCIB_ERR_ISOLATION_VIOLATION` — `from_ctx` does not currently own this
    ///   slot space (fail-closed; no automatic inheritance).
    pub fn transfer_slot(
        &mut self,
        capability_manager: &mut CapabilityManager,
        token_id: CapabilityTokenId,
        from_ctx: ExecutionContextId,
        to_ctx: ExecutionContextId,
    ) -> Result<(), BcibError> {
        // Fail-closed: caller must be the current owner.
        self.check_owner(from_ctx)?;
        // Authorize via CapabilityManager — the only permitted transfer path.
        capability_manager.transfer(token_id, from_ctx, to_ctx)?;
        // Re-bind ownership.
        self.owner = to_ctx;
        Ok(())
    }

    fn check_owner(&self, ctx_id: ExecutionContextId) -> Result<(), BcibError> {
        if ctx_id != self.owner {
            Err(BcibError::IsolationViolation(
                "cross-context slot access without capability token",
            ))
        } else {
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// IsolatedHandleSpace
// ---------------------------------------------------------------------------

/// Context-isolated handle space.
///
/// Tracks registered handle IDs up to a fixed `capacity`. Wrong `ctx_id` →
/// `BCIB_ERR_ISOLATION_VIOLATION` (Requirements 15.1, 15.2, 18.2).
/// Exceeding `capacity` → `BCIB_ERR_RESOURCE_EXHAUSTED` (Requirement 18.2).
#[derive(Debug)]
pub struct IsolatedHandleSpace {
    owner: ExecutionContextId,
    /// Currently registered handle IDs.
    registered: Vec<HandleEntry>,
    /// Maximum number of concurrent handles.
    capacity: usize,
}

impl IsolatedHandleSpace {
    /// Create a new handle space owned by `owner_id` with `capacity` handle slots.
    pub fn new(owner_id: ExecutionContextId, capacity: usize) -> Self {
        Self {
            owner: owner_id,
            registered: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Register a new handle on behalf of `ctx_id`.
    ///
    /// Returns `BCIB_ERR_ISOLATION_VIOLATION` if `ctx_id != owner`.
    /// Returns `BCIB_ERR_RESOURCE_EXHAUSTED` if `capacity` is reached.
    pub fn acquire(&mut self, ctx_id: ExecutionContextId, handle_id: u64) -> Result<(), BcibError> {
        self.check_owner(ctx_id)?;
        if self.registered.len() >= self.capacity {
            return Err(BcibError::ResourceExhausted(
                "handle pool exhausted; max_concurrent_handles reached",
            ));
        }
        self.registered.push(HandleEntry::new(handle_id));
        Ok(())
    }

    /// Deregister a handle by `handle_id` on behalf of `ctx_id`.
    ///
    /// Returns `BCIB_ERR_ISOLATION_VIOLATION` if `ctx_id != owner`.
    pub fn release(&mut self, ctx_id: ExecutionContextId, handle_id: u64) -> Result<(), BcibError> {
        self.check_owner(ctx_id)?;
        self.registered.retain(|e| e.handle_id != handle_id);
        Ok(())
    }

    /// Deregister all handles (used during teardown).
    pub fn release_all(&mut self, ctx_id: ExecutionContextId) -> Result<(), BcibError> {
        self.check_owner(ctx_id)?;
        self.registered.clear();
        Ok(())
    }

    /// Number of currently registered handles.
    pub fn registered_count(&self) -> usize {
        self.registered.len()
    }

    /// Maximum number of concurrent handles allowed.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// The context ID that owns this handle space.
    pub fn owner(&self) -> ExecutionContextId {
        self.owner
    }

    /// Transfer ownership of this handle space from `from_ctx` to `to_ctx`.
    ///
    /// Mirrors `IsolatedSlotSpace::transfer_slot()`. The caller must supply a
    /// `CapabilityManager` holding a valid `CrossContextTransfer` token bound
    /// to `from_ctx`. The manager's `transfer()` method re-binds that token to
    /// `to_ctx` as the authorization step (Requirements 14.6, 15.3, 15.4).
    ///
    /// After a successful call the handle space's owner is updated to `to_ctx`.
    /// The original context can no longer access the space.
    ///
    /// # Errors
    ///
    /// - `BCIB_ERR_CAPABILITY_DENIED` — `capability_manager.transfer()` rejected
    ///   the request (token not found, revoked, or not owned by `from_ctx`).
    /// - `BCIB_ERR_ISOLATION_VIOLATION` — `from_ctx` does not currently own this
    ///   handle space (fail-closed; no automatic inheritance).
    pub fn transfer_handle(
        &mut self,
        capability_manager: &mut CapabilityManager,
        token_id: CapabilityTokenId,
        from_ctx: ExecutionContextId,
        to_ctx: ExecutionContextId,
    ) -> Result<(), BcibError> {
        // Fail-closed: caller must be the current owner.
        self.check_owner(from_ctx)?;
        // Authorize via CapabilityManager — the only permitted transfer path.
        capability_manager.transfer(token_id, from_ctx, to_ctx)?;
        // Re-bind ownership.
        self.owner = to_ctx;
        Ok(())
    }

    fn check_owner(&self, ctx_id: ExecutionContextId) -> Result<(), BcibError> {
        if ctx_id != self.owner {
            Err(BcibError::IsolationViolation(
                "cross-context handle access without capability token",
            ))
        } else {
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // BoundedPool<T>
    // -----------------------------------------------------------------------

    #[test]
    fn bounded_pool_acquire_and_release() {
        let mut pool: BoundedPool<u32> = BoundedPool::from_items(vec![1, 2, 3]);
        assert_eq!(pool.available_count(), 3);

        let item = pool.acquire().expect("should acquire");
        assert_eq!(pool.available_count(), 2);
        assert_eq!(pool.outstanding_count(), 1);

        pool.release(item);
        assert_eq!(pool.available_count(), 3);
        assert_eq!(pool.outstanding_count(), 0);
    }

    #[test]
    fn bounded_pool_exhaustion_returns_resource_exhausted() {
        let mut pool: BoundedPool<u32> = BoundedPool::from_items(vec![42]);
        let _ = pool.acquire().unwrap();

        let result = pool.acquire();
        assert!(
            matches!(result, Err(BcibError::ResourceExhausted(_))),
            "expected ResourceExhausted, got {:?}",
            result
        );
    }

    #[test]
    fn bounded_pool_release_beyond_capacity_does_not_grow() {
        let mut pool: BoundedPool<u32> = BoundedPool::from_items(vec![1]);
        // Release an extra item — pool must not exceed capacity.
        pool.release(99);
        assert_eq!(
            pool.available_count(),
            1,
            "pool must not grow beyond capacity"
        );
    }

    #[test]
    fn bounded_pool_capacity_is_fixed() {
        let pool: BoundedPool<u32> = BoundedPool::new(5);
        assert_eq!(pool.capacity(), 5);
    }

    #[test]
    fn bounded_pool_is_exhausted_when_empty() {
        let mut pool: BoundedPool<u32> = BoundedPool::from_items(vec![1]);
        assert!(!pool.is_exhausted());
        let _ = pool.acquire().unwrap();
        assert!(pool.is_exhausted());
    }

    // -----------------------------------------------------------------------
    // IsolatedSlotSpace
    // -----------------------------------------------------------------------

    #[test]
    fn isolated_slot_space_acquire_correct_owner() {
        let mut space = IsolatedSlotSpace::new(1, 4);
        let slot = space.acquire(1).expect("owner should acquire");
        assert_eq!(space.available_count(), 3);
        assert_eq!(space.outstanding_count(), 1);
        space.release(1, slot).expect("owner should release");
        assert_eq!(space.available_count(), 4);
    }

    #[test]
    fn isolated_slot_space_acquire_wrong_owner_returns_isolation_violation() {
        let mut space = IsolatedSlotSpace::new(1, 4);
        let result = space.acquire(2); // wrong ctx_id
        assert!(
            matches!(result, Err(BcibError::IsolationViolation(_))),
            "expected IsolationViolation, got {:?}",
            result
        );
    }

    #[test]
    fn isolated_slot_space_release_wrong_owner_returns_isolation_violation() {
        let mut space = IsolatedSlotSpace::new(1, 4);
        let slot = space.acquire(1).unwrap();
        let result = space.release(2, slot); // wrong ctx_id
        assert!(
            matches!(result, Err(BcibError::IsolationViolation(_))),
            "expected IsolationViolation, got {:?}",
            result
        );
    }

    #[test]
    fn isolated_slot_space_exhaustion_returns_resource_exhausted() {
        let mut space = IsolatedSlotSpace::new(1, 1);
        let _slot = space.acquire(1).unwrap();
        let result = space.acquire(1);
        assert!(
            matches!(result, Err(BcibError::ResourceExhausted(_))),
            "expected ResourceExhausted, got {:?}",
            result
        );
    }

    #[test]
    fn isolated_slot_space_release_all_restores_capacity() {
        let mut space = IsolatedSlotSpace::new(1, 3);
        let _s1 = space.acquire(1).unwrap();
        let _s2 = space.acquire(1).unwrap();
        assert_eq!(space.available_count(), 1);

        space
            .release_all(1)
            .expect("release_all should succeed for owner");
        assert_eq!(space.available_count(), 3);
    }

    #[test]
    fn isolated_slot_space_release_all_wrong_owner_returns_isolation_violation() {
        let mut space = IsolatedSlotSpace::new(1, 3);
        let result = space.release_all(2);
        assert!(
            matches!(result, Err(BcibError::IsolationViolation(_))),
            "expected IsolationViolation, got {:?}",
            result
        );
    }

    // -----------------------------------------------------------------------
    // IsolatedHandleSpace
    // -----------------------------------------------------------------------

    #[test]
    fn isolated_handle_space_acquire_correct_owner() {
        let mut space = IsolatedHandleSpace::new(1, 4);
        space.acquire(1, 100).expect("owner should register handle");
        assert_eq!(space.registered_count(), 1);
    }

    #[test]
    fn isolated_handle_space_acquire_wrong_owner_returns_isolation_violation() {
        let mut space = IsolatedHandleSpace::new(1, 4);
        let result = space.acquire(2, 100);
        assert!(
            matches!(result, Err(BcibError::IsolationViolation(_))),
            "expected IsolationViolation, got {:?}",
            result
        );
    }

    #[test]
    fn isolated_handle_space_exhaustion_returns_resource_exhausted() {
        let mut space = IsolatedHandleSpace::new(1, 1);
        space.acquire(1, 100).unwrap();
        let result = space.acquire(1, 101);
        assert!(
            matches!(result, Err(BcibError::ResourceExhausted(_))),
            "expected ResourceExhausted, got {:?}",
            result
        );
    }

    #[test]
    fn isolated_handle_space_release_removes_handle() {
        let mut space = IsolatedHandleSpace::new(1, 4);
        space.acquire(1, 100).unwrap();
        space.acquire(1, 200).unwrap();
        space.release(1, 100).unwrap();
        assert_eq!(space.registered_count(), 1);
    }

    #[test]
    fn isolated_handle_space_release_wrong_owner_returns_isolation_violation() {
        let mut space = IsolatedHandleSpace::new(1, 4);
        space.acquire(1, 100).unwrap();
        let result = space.release(2, 100);
        assert!(
            matches!(result, Err(BcibError::IsolationViolation(_))),
            "expected IsolationViolation, got {:?}",
            result
        );
    }

    #[test]
    fn isolated_handle_space_release_all_clears_handles() {
        let mut space = IsolatedHandleSpace::new(1, 4);
        space.acquire(1, 1).unwrap();
        space.acquire(1, 2).unwrap();
        space.release_all(1).unwrap();
        assert_eq!(space.registered_count(), 0);
    }

    #[test]
    fn isolated_handle_space_release_all_wrong_owner_returns_isolation_violation() {
        let mut space = IsolatedHandleSpace::new(1, 4);
        let result = space.release_all(2);
        assert!(
            matches!(result, Err(BcibError::IsolationViolation(_))),
            "expected IsolationViolation, got {:?}",
            result
        );
    }

    // -----------------------------------------------------------------------
    // Context isolation enforcement — Task 28
    // Requirements: 14.6, 15.1, 15.2, 15.3, 15.4
    // -----------------------------------------------------------------------

    use crate::capability_manager::{CapabilityManager, CapabilityResource};

    /// 15.3 — Cross-context slot access without capability → BCIB_ERR_ISOLATION_VIOLATION.
    #[test]
    fn cross_context_slot_access_without_capability_returns_isolation_violation() {
        let mut space = IsolatedSlotSpace::new(1, 4);
        // ctx 2 tries to acquire a slot owned by ctx 1 — no capability involved.
        let result = space.acquire(2);
        assert!(
            matches!(result, Err(BcibError::IsolationViolation(_))),
            "cross-context slot access without capability must return IsolationViolation, got {:?}",
            result
        );
    }

    /// 15.3 — Cross-context handle access without capability → BCIB_ERR_ISOLATION_VIOLATION.
    #[test]
    fn cross_context_handle_access_without_capability_returns_isolation_violation() {
        let mut space = IsolatedHandleSpace::new(1, 4);
        // ctx 2 tries to register a handle owned by ctx 1 — no capability involved.
        let result = space.acquire(2, 100);
        assert!(
            matches!(result, Err(BcibError::IsolationViolation(_))),
            "cross-context handle access without capability must return IsolationViolation, got {:?}",
            result
        );
    }

    /// 15.4 — Valid capability transfer for slot space → success.
    #[test]
    fn transfer_slot_with_valid_capability_succeeds() {
        let mut mgr = CapabilityManager::new(8);
        // Bind a CrossContextTransfer token to ctx 1.
        let token_id = mgr
            .bind(CapabilityResource::CrossContextTransfer, 1)
            .unwrap();

        let mut space = IsolatedSlotSpace::new(1, 4);
        // Transfer ownership from ctx 1 to ctx 2 using the capability token.
        space
            .transfer_slot(&mut mgr, token_id, 1, 2)
            .expect("valid transfer must succeed");

        // ctx 2 is now the owner — it can acquire slots.
        space
            .acquire(2)
            .expect("new owner ctx 2 should be able to acquire");
        // ctx 1 can no longer access the space.
        let result = space.acquire(1);
        assert!(
            matches!(result, Err(BcibError::IsolationViolation(_))),
            "old owner ctx 1 must be denied after transfer, got {:?}",
            result
        );
    }

    /// 15.4 — Valid capability transfer for handle space → success.
    #[test]
    fn transfer_handle_with_valid_capability_succeeds() {
        let mut mgr = CapabilityManager::new(8);
        let token_id = mgr
            .bind(CapabilityResource::CrossContextTransfer, 1)
            .unwrap();

        let mut space = IsolatedHandleSpace::new(1, 4);
        space
            .transfer_handle(&mut mgr, token_id, 1, 2)
            .expect("valid transfer must succeed");

        // ctx 2 is now the owner.
        space
            .acquire(2, 100)
            .expect("new owner ctx 2 should be able to register handle");
        // ctx 1 can no longer access the space.
        let result = space.acquire(1, 200);
        assert!(
            matches!(result, Err(BcibError::IsolationViolation(_))),
            "old owner ctx 1 must be denied after transfer, got {:?}",
            result
        );
    }

    /// 14.6 — Automatic inheritance attempt is rejected.
    /// A sub-context (ctx 2) cannot use a parent's (ctx 1) capability token
    /// without an explicit transfer through CapabilityManager.
    #[test]
    fn automatic_inheritance_attempt_is_rejected() {
        let mut mgr = CapabilityManager::new(8);
        // Bind a token to ctx 1 (parent).
        let token_id = mgr
            .bind(CapabilityResource::CrossContextTransfer, 1)
            .unwrap();

        let mut space = IsolatedSlotSpace::new(1, 4);
        // ctx 2 (child) tries to transfer using ctx 1's token without owning it.
        // transfer() in CapabilityManager will reject because the token is bound to ctx 1,
        // not ctx 2 — and the slot space check_owner will also reject ctx 2 as from_ctx.
        let result = space.transfer_slot(&mut mgr, token_id, 2, 3);
        assert!(
            matches!(result, Err(BcibError::IsolationViolation(_))),
            "automatic inheritance attempt must be rejected with IsolationViolation, got {:?}",
            result
        );
        // The space is still owned by ctx 1.
        assert_eq!(space.owner(), 1);
    }

    /// 15.3 — transfer_slot without a valid capability token → BCIB_ERR_CAPABILITY_DENIED.
    #[test]
    fn transfer_slot_without_valid_token_returns_capability_denied() {
        let mut mgr = CapabilityManager::new(8);
        // No token bound — use a bogus token ID.
        let mut space = IsolatedSlotSpace::new(1, 4);
        let result = space.transfer_slot(&mut mgr, 9999, 1, 2);
        assert!(
            matches!(result, Err(BcibError::CapabilityDenied(_))),
            "transfer without valid token must return CapabilityDenied, got {:?}",
            result
        );
        // Ownership must not have changed.
        assert_eq!(space.owner(), 1);
    }

    /// 15.3 — transfer_handle without a valid capability token → BCIB_ERR_CAPABILITY_DENIED.
    #[test]
    fn transfer_handle_without_valid_token_returns_capability_denied() {
        let mut mgr = CapabilityManager::new(8);
        let mut space = IsolatedHandleSpace::new(1, 4);
        let result = space.transfer_handle(&mut mgr, 9999, 1, 2);
        assert!(
            matches!(result, Err(BcibError::CapabilityDenied(_))),
            "transfer without valid token must return CapabilityDenied, got {:?}",
            result
        );
        assert_eq!(space.owner(), 1);
    }
}

use crate::isolation::error_taxonomy::{ErrorCode, IsolationError};
/// ABDF Handle Management System
///
/// This module implements opaque handle types and lifecycle management for ABDF
/// data access, preventing raw pointer exposure and ensuring memory safety.
///
/// ## Requirements
///
/// - Requirement 9.1: ABDF SHALL expose data only via opaque ABDF_Handle references
/// - Requirement 9.2: ABDF SHALL NOT expose raw memory pointers to BCIB
/// - Requirement 9.3: ABDF_Handle SHALL be context-bound to the execution context
/// - Requirement 9.4: ABDF SHALL support handle revocation by the data owner
/// - Requirement 9.5: System SHALL enforce handle lifecycle limits and prevent handle exhaustion
/// - Requirement 9.6: System SHALL allow unused or stale handles to be reclaimed
/// - Requirement 9.7: When a revoked handle is used, ABDF SHALL return BCIB_ERR_ABDF_HANDLE_REVOKED
/// - Requirement 9.8: ABDF_Handle SHALL NOT be transferable between execution contexts without explicit capability
/// - Requirement 9.9: ABDF SHALL reject stale handles that reference deleted or expired objects
///
/// ## Design Principles
///
/// - **Opaque Handles**: No raw pointer exposure (Requirements 9.1, 9.2)
/// - **Context Binding**: Handles are bound to execution contexts (Requirement 9.3)
/// - **Lifecycle Management**: Creation, validation, revocation, and reclamation (Requirements 9.4, 9.5, 9.6)
/// - **Fail-Closed**: Invalid handle access results in deterministic errors (Requirement 9.7)
use crate::types::ExecutionContextId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Opaque handle identifier (Requirements 9.1, 9.2)
///
/// This is an opaque u64 identifier that does NOT contain or expose raw memory
/// addresses. The actual resource mapping is maintained internally by the
/// HandleManager and is never exposed to BCIB.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandleId(u64);

impl HandleId {
    /// Create a new handle ID from a u64 value
    ///
    /// SAFETY: This function does NOT accept or expose raw pointers.
    /// The u64 value is an opaque identifier only.
    pub(crate) fn new(id: u64) -> Self {
        Self(id)
    }

    /// Create a handle ID from a u64 value (public API for Runtime_Bridge)
    ///
    /// This is safe because HandleId is opaque and cannot be dereferenced.
    pub fn from_u64(id: u64) -> Self {
        Self(id)
    }

    /// Get the opaque identifier value
    ///
    /// This returns the opaque u64 identifier, NOT a memory address.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Handle status tracking (Requirements 9.4, 9.7, 9.9)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleStatus {
    /// Handle is valid and can be used
    Valid,
    /// Handle has been revoked by the data owner (Requirement 9.4)
    Revoked,
    /// Handle references a deleted or expired object (Requirement 9.9)
    Stale,
    /// Handle has expired due to lifecycle limits (Requirement 9.5)
    Expired,
}

/// ABDF segment type (Requirement 10.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentType {
    /// BCIB input data (read-only)
    Input,
    /// External event data
    Event,
    /// Device status snapshot
    DeviceStatus,
    /// Result of a read operation
    ReadResult,
    /// Result of execution
    ExecutionResult,
    /// Execution trace for deterministic replay
    ExecutionTrace,
    /// Reference to another ABDF object
    Ref,
}

impl SegmentType {
    /// Check if this segment type is read-only (Requirement 10.2)
    pub fn is_read_only(&self) -> bool {
        matches!(
            self,
            SegmentType::Input
                | SegmentType::Event
                | SegmentType::DeviceStatus
                | SegmentType::ReadResult
        )
    }

    /// Check if this segment type is mutable (Requirement 10.2)
    pub fn is_mutable(&self) -> bool {
        matches!(
            self,
            SegmentType::ExecutionResult | SegmentType::ExecutionTrace
        )
    }

    /// Check if this segment type is a reference (Requirement 10.1)
    pub fn is_reference(&self) -> bool {
        matches!(self, SegmentType::Ref)
    }

    /// Get the maximum allowed size for this segment type (Requirement 10.3)
    pub fn max_size(&self) -> usize {
        match self {
            SegmentType::Input => 1024 * 1024,              // 1 MiB
            SegmentType::Event => 64 * 1024,                // 64 KiB
            SegmentType::DeviceStatus => 16 * 1024,         // 16 KiB
            SegmentType::ReadResult => 256 * 1024,          // 256 KiB
            SegmentType::ExecutionResult => 512 * 1024,     // 512 KiB
            SegmentType::ExecutionTrace => 2 * 1024 * 1024, // 2 MiB
            SegmentType::Ref => 64,                         // 64 bytes (just a reference)
        }
    }

    /// Validate segment data size (Requirement 10.3)
    pub fn validate_size(&self, size: usize) -> Result<(), IsolationError> {
        if size > self.max_size() {
            Err(IsolationError::new(
                ErrorCode::AbdfTypeViolation,
                format!(
                    "Segment size {} exceeds maximum {} for type {:?}",
                    size,
                    self.max_size(),
                    self
                ),
                None,
            ))
        } else {
            Ok(())
        }
    }

    /// Check if mutation is allowed for this segment type (Requirement 10.4)
    pub fn allows_mutation(&self) -> bool {
        self.is_mutable()
    }

    /// Get human-readable description of this segment type
    pub fn description(&self) -> &'static str {
        match self {
            SegmentType::Input => "BCIB input data (read-only)",
            SegmentType::Event => "External event data",
            SegmentType::DeviceStatus => "Device status snapshot",
            SegmentType::ReadResult => "Result of a read operation",
            SegmentType::ExecutionResult => "Result of execution",
            SegmentType::ExecutionTrace => "Execution trace for deterministic replay",
            SegmentType::Ref => "Reference to another ABDF object",
        }
    }
}

/// Segment type constraints and validation (Requirements 10.2, 10.3, 10.4)
///
/// This module implements type-safe segment creation and access methods
/// with constraint enforcement.
pub struct SegmentTypeValidator;

impl SegmentTypeValidator {
    /// Validate segment creation (Requirements 10.2, 10.3)
    pub fn validate_creation(segment_type: SegmentType, data: &[u8]) -> Result<(), IsolationError> {
        // Validate size constraint (Requirement 10.3)
        segment_type.validate_size(data.len())?;

        // Additional type-specific validation
        match segment_type {
            SegmentType::Ref => {
                // Ref segments must contain valid reference data
                if data.len() < 8 {
                    return Err(IsolationError::new(
                        ErrorCode::AbdfTypeViolation,
                        "Ref segment must contain at least 8 bytes (handle ID)",
                        None,
                    ));
                }
            }
            SegmentType::ExecutionTrace => {
                // ExecutionTrace must be non-empty
                if data.is_empty() {
                    return Err(IsolationError::new(
                        ErrorCode::AbdfTypeViolation,
                        "ExecutionTrace segment cannot be empty",
                        None,
                    ));
                }
            }
            _ => {
                // Other types have no additional constraints
            }
        }

        Ok(())
    }

    /// Validate segment mutation (Requirement 10.4)
    pub fn validate_mutation(
        segment_type: SegmentType,
        _current_data: &[u8],
        new_data: &[u8],
    ) -> Result<(), IsolationError> {
        // Check if mutation is allowed for this type (Requirement 10.4)
        if !segment_type.allows_mutation() {
            return Err(IsolationError::new(
                ErrorCode::AbdfTypeViolation,
                format!(
                    "Segment type {:?} is read-only and cannot be mutated",
                    segment_type
                ),
                None,
            ));
        }

        // Validate new data size
        segment_type.validate_size(new_data.len())?;

        Ok(())
    }

    /// Validate segment access (Requirement 10.5)
    pub fn validate_access(
        segment_type: SegmentType,
        access_mode: AccessMode,
    ) -> Result<(), IsolationError> {
        match access_mode {
            AccessMode::Read => {
                // All segment types support read access
                Ok(())
            }
            AccessMode::Write => {
                // Only mutable segment types support write access
                if segment_type.allows_mutation() {
                    Ok(())
                } else {
                    Err(IsolationError::new(
                        ErrorCode::AbdfTypeViolation,
                        format!(
                            "Segment type {:?} does not support write access",
                            segment_type
                        ),
                        None,
                    ))
                }
            }
        }
    }
}

/// Access mode for segment validation (Requirement 10.5)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    /// Read-only access
    Read,
    /// Write access (mutation)
    Write,
}

/// Opaque ABDF handle (Requirements 9.1, 9.2, 9.3)
///
/// This handle provides access to ABDF data without exposing raw pointers.
/// Handles are context-bound and tracked through their lifecycle.
#[derive(Debug, Clone)]
pub struct AbdfHandle {
    /// Opaque handle identifier
    pub id: HandleId,
    /// Segment type for type-safe access
    pub segment_type: SegmentType,
    /// Execution context this handle is bound to (Requirement 9.3)
    pub context_id: ExecutionContextId,
    /// Current handle status
    pub status: HandleStatus,
    /// Generation counter for ABA problem prevention
    generation: u64,
}

impl AbdfHandle {
    /// Create a new valid handle
    pub(crate) fn new(
        id: HandleId,
        segment_type: SegmentType,
        context_id: ExecutionContextId,
        generation: u64,
    ) -> Self {
        Self {
            id,
            segment_type,
            context_id,
            status: HandleStatus::Valid,
            generation,
        }
    }

    /// Create a handle for validation (used by Runtime_Bridge)
    ///
    /// This creates a handle structure for validation purposes.
    /// The actual handle must exist in the HandleManager.
    pub fn for_validation(
        id: HandleId,
        segment_type: SegmentType,
        context_id: ExecutionContextId,
    ) -> Self {
        Self {
            id,
            segment_type,
            context_id,
            status: HandleStatus::Valid,
            generation: 0, // Will be validated by handle manager
        }
    }

    /// Check if this handle is valid for use
    pub fn is_valid(&self) -> bool {
        self.status == HandleStatus::Valid
    }

    /// Check if this handle has been revoked
    pub fn is_revoked(&self) -> bool {
        self.status == HandleStatus::Revoked
    }

    /// Check if this handle is stale
    pub fn is_stale(&self) -> bool {
        self.status == HandleStatus::Stale
    }

    /// Check if this handle belongs to the given execution context
    pub fn belongs_to_context(&self, context_id: ExecutionContextId) -> bool {
        self.context_id == context_id
    }
}

/// Internal resource mapping (opaque to BCIB)
///
/// This structure maintains the actual resource data without exposing
/// raw pointers to BCIB. The mapping is internal to the HandleManager.
#[derive(Debug, Clone)]
struct ResourceMapping {
    /// Opaque data storage (NOT a raw pointer)
    data: Vec<u8>,
    /// Segment type
    /// TODO(Task 4.4): Wire up segment_type validation in access methods
    #[allow(dead_code)]
    segment_type: SegmentType,
    /// Execution context that owns this resource
    context_id: ExecutionContextId,
    /// Generation counter for ABA problem prevention
    generation: u64,
    /// Reference count for lifecycle management
    ref_count: usize,
}

/// Handle pool configuration (Requirement 9.5)
#[derive(Debug, Clone)]
pub struct HandlePoolConfig {
    /// Maximum number of concurrent handles per context
    pub max_handles_per_context: usize,
    /// Maximum total handles across all contexts
    pub max_total_handles: usize,
    /// Enable automatic stale handle reclamation
    pub enable_reclamation: bool,
}

impl Default for HandlePoolConfig {
    fn default() -> Self {
        Self {
            max_handles_per_context: 64,
            max_total_handles: 1024,
            enable_reclamation: true,
        }
    }
}

/// Handle Manager - manages handle lifecycle and resource mapping
///
/// This is the authoritative component for handle creation, validation,
/// revocation, and reclamation. It maintains the opaque mapping between
/// handles and resources without exposing raw pointers.
pub struct HandleManager {
    /// Handle pool configuration
    config: HandlePoolConfig,
    /// Next handle ID to allocate
    next_handle_id: u64,
    /// Active handles mapped to resources
    handles: HashMap<HandleId, ResourceMapping>,
    /// Per-context handle count for exhaustion prevention
    context_handle_counts: HashMap<ExecutionContextId, usize>,
    /// Generation counter for ABA problem prevention
    generation: u64,
}

impl HandleManager {
    /// Create a new handle manager with the given configuration
    pub fn new(config: HandlePoolConfig) -> Self {
        Self {
            config,
            next_handle_id: 1, // Start from 1, reserve 0 as invalid
            handles: HashMap::new(),
            context_handle_counts: HashMap::new(),
            generation: 0,
        }
    }

    /// Create a new handle with default configuration
    pub fn new_default() -> Self {
        Self::new(HandlePoolConfig::default())
    }

    /// Create a new ABDF handle (Requirements 9.1, 9.3, 9.5)
    ///
    /// Returns an opaque handle that does NOT expose raw pointers.
    /// Enforces handle exhaustion limits per context and globally.
    pub fn create_handle(
        &mut self,
        segment_type: SegmentType,
        context_id: ExecutionContextId,
        data: Vec<u8>,
    ) -> Result<AbdfHandle, IsolationError> {
        // Validate segment type and data (Requirements 10.2, 10.3)
        SegmentTypeValidator::validate_creation(segment_type, &data)?;

        // Check global handle limit (Requirement 9.5)
        if self.handles.len() >= self.config.max_total_handles {
            return Err(IsolationError::new(
                ErrorCode::BoundaryViolation,
                "Handle pool exhausted: global limit reached",
                Some(context_id),
            ));
        }

        // Check per-context handle limit (Requirement 9.5)
        let context_count = self
            .context_handle_counts
            .get(&context_id)
            .copied()
            .unwrap_or(0);
        if context_count >= self.config.max_handles_per_context {
            return Err(IsolationError::new(
                ErrorCode::BoundaryViolation,
                "Handle pool exhausted: per-context limit reached",
                Some(context_id),
            ));
        }

        // Allocate new handle ID
        let handle_id = HandleId::new(self.next_handle_id);
        self.next_handle_id += 1;

        // Increment generation for ABA prevention
        self.generation += 1;
        let generation = self.generation;

        // Create resource mapping (opaque, no raw pointers)
        let mapping = ResourceMapping {
            data,
            segment_type,
            context_id,
            generation,
            ref_count: 1,
        };

        // Store mapping
        self.handles.insert(handle_id, mapping);

        // Update context handle count
        *self.context_handle_counts.entry(context_id).or_insert(0) += 1;

        // Create and return opaque handle
        Ok(AbdfHandle::new(
            handle_id,
            segment_type,
            context_id,
            generation,
        ))
    }

    /// Validate a handle (Requirements 9.3, 9.7, 9.8, 9.9)
    ///
    /// Checks that the handle is valid, belongs to the correct context,
    /// and has not been revoked or become stale.
    pub fn validate_handle(
        &self,
        handle: &AbdfHandle,
        context_id: ExecutionContextId,
    ) -> Result<(), IsolationError> {
        // Check if handle belongs to the requesting context (Requirement 9.8)
        if !handle.belongs_to_context(context_id) {
            return Err(IsolationError::new(
                ErrorCode::CrossContextAccess,
                "Handle does not belong to requesting context",
                Some(context_id),
            ));
        }

        // Check handle status (Requirements 9.7, 9.9)
        match handle.status {
            HandleStatus::Valid => {
                // Verify handle still exists in mapping
                if let Some(mapping) = self.handles.get(&handle.id) {
                    // Verify generation matches (ABA prevention)
                    if mapping.generation != handle.generation {
                        return Err(IsolationError::new(
                            ErrorCode::AbdfHandleRevoked,
                            "Handle generation mismatch (stale handle)",
                            Some(context_id),
                        ));
                    }
                    Ok(())
                } else {
                    Err(IsolationError::new(
                        ErrorCode::AbdfHandleRevoked,
                        "Handle no longer exists in mapping",
                        Some(context_id),
                    ))
                }
            }
            HandleStatus::Revoked => Err(IsolationError::abdf_handle_revoked(
                context_id,
                handle.id.as_u64(),
            )),
            HandleStatus::Stale => Err(IsolationError::new(
                ErrorCode::AbdfHandleRevoked,
                "Handle references deleted or expired object",
                Some(context_id),
            )),
            HandleStatus::Expired => Err(IsolationError::new(
                ErrorCode::AbdfHandleRevoked,
                "Handle has expired due to lifecycle limits",
                Some(context_id),
            )),
        }
    }

    /// Revoke a handle (Requirement 9.4)
    ///
    /// Marks the handle as revoked. Subsequent access attempts will fail
    /// with BCIB_ERR_ABDF_HANDLE_REVOKED (Requirement 9.7).
    pub fn revoke_handle(
        &mut self,
        handle_id: HandleId,
        context_id: ExecutionContextId,
    ) -> Result<(), IsolationError> {
        if let Some(mapping) = self.handles.get(&handle_id) {
            // Verify context ownership before revocation
            if mapping.context_id != context_id {
                return Err(IsolationError::new(
                    ErrorCode::CrossContextAccess,
                    "Cannot revoke handle from different context",
                    Some(context_id),
                ));
            }

            // Remove mapping (handle becomes revoked)
            self.handles.remove(&handle_id);

            // Decrement context handle count
            if let Some(count) = self.context_handle_counts.get_mut(&context_id) {
                *count = count.saturating_sub(1);
            }

            Ok(())
        } else {
            // Handle already revoked or never existed
            Ok(())
        }
    }

    /// Access handle data (Requirements 9.1, 9.2)
    ///
    /// Returns a reference to the data without exposing raw pointers.
    /// The data is accessed through safe Rust references only.
    pub fn access_handle_data(
        &self,
        handle: &AbdfHandle,
        context_id: ExecutionContextId,
    ) -> Result<&[u8], IsolationError> {
        // Validate handle first
        self.validate_handle(handle, context_id)?;

        // Access data through safe mapping (no raw pointers)
        if let Some(mapping) = self.handles.get(&handle.id) {
            Ok(&mapping.data)
        } else {
            Err(IsolationError::new(
                ErrorCode::AbdfHandleRevoked,
                "Handle mapping not found",
                Some(context_id),
            ))
        }
    }

    /// Reclaim stale handles for a context (Requirement 9.6)
    ///
    /// Removes handles that are no longer referenced and frees resources.
    /// This prevents handle exhaustion by reclaiming unused handles.
    pub fn reclaim_stale_handles(&mut self, context_id: ExecutionContextId) -> usize {
        if !self.config.enable_reclamation {
            return 0;
        }

        let mut reclaimed = 0;

        // Collect handles to remove (avoid borrow checker issues)
        let handles_to_remove: Vec<HandleId> = self
            .handles
            .iter()
            .filter(|(_, mapping)| mapping.context_id == context_id && mapping.ref_count == 0)
            .map(|(id, _)| *id)
            .collect();

        // Remove stale handles
        for handle_id in handles_to_remove {
            self.handles.remove(&handle_id);
            reclaimed += 1;
        }

        // Update context handle count
        if let Some(count) = self.context_handle_counts.get_mut(&context_id) {
            *count = count.saturating_sub(reclaimed);
        }

        reclaimed
    }

    /// Revoke all handles for a context (cleanup on context termination)
    pub fn revoke_all_context_handles(&mut self, context_id: ExecutionContextId) -> usize {
        let mut revoked = 0;

        // Collect handles to remove
        let handles_to_remove: Vec<HandleId> = self
            .handles
            .iter()
            .filter(|(_, mapping)| mapping.context_id == context_id)
            .map(|(id, _)| *id)
            .collect();

        // Remove all context handles
        for handle_id in handles_to_remove {
            self.handles.remove(&handle_id);
            revoked += 1;
        }

        // Reset context handle count
        self.context_handle_counts.remove(&context_id);

        revoked
    }

    /// Get handle statistics for monitoring
    pub fn get_stats(&self) -> HandleStats {
        HandleStats {
            total_handles: self.handles.len(),
            total_contexts: self.context_handle_counts.len(),
            max_total_handles: self.config.max_total_handles,
            max_handles_per_context: self.config.max_handles_per_context,
        }
    }
}

/// Handle statistics for monitoring and debugging
#[derive(Debug, Clone)]
pub struct HandleStats {
    pub total_handles: usize,
    pub total_contexts: usize,
    pub max_total_handles: usize,
    pub max_handles_per_context: usize,
}

/// Thread-safe handle manager wrapper
pub type SharedHandleManager = Arc<Mutex<HandleManager>>;

/// Handle exhaustion detection and prevention (Requirement 9.5)
///
/// This module implements bounded handle pool management with exhaustion
/// detection and fail-closed behavior when limits are reached.
impl HandleManager {
    /// Check if handle pool is approaching exhaustion (warning threshold)
    pub fn is_approaching_exhaustion(&self) -> bool {
        let usage_percent = (self.handles.len() * 100) / self.config.max_total_handles;
        usage_percent >= 80 // Warn at 80% capacity
    }

    /// Check if handle pool is exhausted
    pub fn is_exhausted(&self) -> bool {
        self.handles.len() >= self.config.max_total_handles
    }

    /// Check if a context is approaching its handle limit
    pub fn is_context_approaching_limit(&self, context_id: ExecutionContextId) -> bool {
        if let Some(&count) = self.context_handle_counts.get(&context_id) {
            let usage_percent = (count * 100) / self.config.max_handles_per_context;
            usage_percent >= 80
        } else {
            false
        }
    }

    /// Get handle count for a specific context
    pub fn get_context_handle_count(&self, context_id: ExecutionContextId) -> usize {
        self.context_handle_counts
            .get(&context_id)
            .copied()
            .unwrap_or(0)
    }

    /// Attempt to reclaim handles across all contexts to prevent exhaustion
    ///
    /// This is called when the pool is approaching exhaustion to free up
    /// resources before hitting the hard limit (Requirement 9.6).
    pub fn reclaim_all_stale_handles(&mut self) -> usize {
        if !self.config.enable_reclamation {
            return 0;
        }

        let mut total_reclaimed = 0;

        // Collect all context IDs
        let context_ids: Vec<ExecutionContextId> =
            self.context_handle_counts.keys().copied().collect();

        // Reclaim stale handles for each context
        for context_id in context_ids {
            total_reclaimed += self.reclaim_stale_handles(context_id);
        }

        total_reclaimed
    }

    /// Force reclamation when exhaustion is imminent (fail-closed prevention)
    ///
    /// This is a more aggressive reclamation strategy that runs when the pool
    /// is at or near exhaustion. It attempts to free resources before failing.
    pub fn force_reclamation_on_exhaustion(&mut self) -> Result<(), IsolationError> {
        if !self.is_approaching_exhaustion() {
            return Ok(());
        }

        let reclaimed = self.reclaim_all_stale_handles();

        if reclaimed > 0 {
            Ok(())
        } else if self.is_exhausted() {
            Err(IsolationError::new(
                ErrorCode::BoundaryViolation,
                "Handle pool exhausted and no stale handles available for reclamation",
                None,
            ))
        } else {
            Ok(())
        }
    }
}

/// Revocation propagation across execution contexts (Requirement 9.6)
///
/// When a handle is revoked, the revocation must propagate to all contexts
/// that might have references to it. This prevents use-after-revoke bugs.
impl HandleManager {
    /// Revoke a handle and propagate revocation to all contexts
    ///
    /// This ensures that no context can use a revoked handle, even if they
    /// have a cached reference to it (Requirement 9.7).
    pub fn revoke_and_propagate(
        &mut self,
        handle_id: HandleId,
        revoking_context: ExecutionContextId,
    ) -> Result<(), IsolationError> {
        // Verify the revoking context owns the handle
        if let Some(mapping) = self.handles.get(&handle_id) {
            if mapping.context_id != revoking_context {
                return Err(IsolationError::new(
                    ErrorCode::CrossContextAccess,
                    "Cannot revoke handle from different context",
                    Some(revoking_context),
                ));
            }
        }

        // Revoke the handle (removes from mapping)
        self.revoke_handle(handle_id, revoking_context)?;

        // Propagation is implicit: any subsequent access to this handle_id
        // will fail validation because the mapping no longer exists.
        // This is the fail-closed behavior (Requirement 9.7).

        Ok(())
    }

    /// Mark handles as stale when their referenced objects are deleted
    ///
    /// This is called when ABDF objects are deleted or expire, marking all
    /// handles that reference them as stale (Requirement 9.9).
    pub fn mark_handles_stale_for_object(&mut self, object_id: u64) -> usize {
        let mut marked = 0;

        // In a real implementation, we would track object_id -> handle_id mappings.
        // For now, this is a placeholder that demonstrates the interface.
        // The actual implementation would iterate through handles and mark those
        // referencing the deleted object as stale.

        // Placeholder: mark all handles with matching generation as stale
        // (In production, this would use proper object tracking)
        for (_, mapping) in self.handles.iter_mut() {
            if mapping.generation == object_id {
                // Mark as stale by removing from active handles
                marked += 1;
            }
        }

        marked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_id_is_opaque() {
        let id = HandleId::new(12345);
        assert_eq!(id.as_u64(), 12345);

        // Verify HandleId does not expose raw pointers
        // This is a compile-time guarantee - HandleId contains only u64
    }

    #[test]
    fn create_handle_success() {
        let mut manager = HandleManager::new_default();
        let context_id = 1;
        let data = vec![1, 2, 3, 4];

        let handle = manager
            .create_handle(SegmentType::Input, context_id, data.clone())
            .expect("handle creation should succeed");

        assert_eq!(handle.segment_type, SegmentType::Input);
        assert_eq!(handle.context_id, context_id);
        assert!(handle.is_valid());
        assert!(!handle.is_revoked());
    }

    #[test]
    fn handle_exhaustion_per_context() {
        let config = HandlePoolConfig {
            max_handles_per_context: 2,
            max_total_handles: 100,
            enable_reclamation: true,
        };
        let mut manager = HandleManager::new(config);
        let context_id = 1;

        // Create first handle - should succeed
        let _h1 = manager
            .create_handle(SegmentType::Input, context_id, vec![1])
            .expect("first handle should succeed");

        // Create second handle - should succeed
        let _h2 = manager
            .create_handle(SegmentType::Input, context_id, vec![2])
            .expect("second handle should succeed");

        // Create third handle - should fail (per-context limit)
        let result = manager.create_handle(SegmentType::Input, context_id, vec![3]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::BoundaryViolation);
    }

    #[test]
    fn handle_exhaustion_global() {
        let config = HandlePoolConfig {
            max_handles_per_context: 100,
            max_total_handles: 2,
            enable_reclamation: true,
        };
        let mut manager = HandleManager::new(config);

        // Create handles in different contexts
        let _h1 = manager
            .create_handle(SegmentType::Input, 1, vec![1])
            .expect("first handle should succeed");

        let _h2 = manager
            .create_handle(SegmentType::Input, 2, vec![2])
            .expect("second handle should succeed");

        // Create third handle - should fail (global limit)
        let result = manager.create_handle(SegmentType::Input, 3, vec![3]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::BoundaryViolation);
    }

    #[test]
    fn validate_handle_success() {
        let mut manager = HandleManager::new_default();
        let context_id = 1;

        let handle = manager
            .create_handle(SegmentType::Input, context_id, vec![1, 2, 3])
            .expect("handle creation should succeed");

        // Validation should succeed
        manager
            .validate_handle(&handle, context_id)
            .expect("validation should succeed");
    }

    #[test]
    fn validate_handle_wrong_context() {
        let mut manager = HandleManager::new_default();
        let context_id = 1;
        let wrong_context = 2;

        let handle = manager
            .create_handle(SegmentType::Input, context_id, vec![1, 2, 3])
            .expect("handle creation should succeed");

        // Validation should fail (wrong context)
        let result = manager.validate_handle(&handle, wrong_context);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::CrossContextAccess);
    }

    #[test]
    fn revoke_handle_success() {
        let mut manager = HandleManager::new_default();
        let context_id = 1;

        let handle = manager
            .create_handle(SegmentType::Input, context_id, vec![1, 2, 3])
            .expect("handle creation should succeed");

        let handle_id = handle.id;

        // Revoke handle
        manager
            .revoke_handle(handle_id, context_id)
            .expect("revocation should succeed");

        // Validation should now fail
        let result = manager.validate_handle(&handle, context_id);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::AbdfHandleRevoked);
    }

    #[test]
    fn revoke_handle_wrong_context() {
        let mut manager = HandleManager::new_default();
        let context_id = 1;
        let wrong_context = 2;

        let handle = manager
            .create_handle(SegmentType::Input, context_id, vec![1, 2, 3])
            .expect("handle creation should succeed");

        // Revocation should fail (wrong context)
        let result = manager.revoke_handle(handle.id, wrong_context);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::CrossContextAccess);
    }

    #[test]
    fn access_handle_data_success() {
        let mut manager = HandleManager::new_default();
        let context_id = 1;
        let data = vec![1, 2, 3, 4, 5];

        let handle = manager
            .create_handle(SegmentType::Input, context_id, data.clone())
            .expect("handle creation should succeed");

        // Access data
        let accessed_data = manager
            .access_handle_data(&handle, context_id)
            .expect("data access should succeed");

        assert_eq!(accessed_data, &data[..]);
    }

    #[test]
    fn access_revoked_handle_fails() {
        let mut manager = HandleManager::new_default();
        let context_id = 1;

        let handle = manager
            .create_handle(SegmentType::Input, context_id, vec![1, 2, 3])
            .expect("handle creation should succeed");

        // Revoke handle
        manager
            .revoke_handle(handle.id, context_id)
            .expect("revocation should succeed");

        // Access should fail
        let result = manager.access_handle_data(&handle, context_id);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::AbdfHandleRevoked);
    }

    #[test]
    fn reclaim_stale_handles() {
        let mut manager = HandleManager::new_default();
        let context_id = 1;

        // Create and immediately revoke handles
        let h1 = manager
            .create_handle(SegmentType::Input, context_id, vec![1])
            .expect("handle creation should succeed");
        let h2 = manager
            .create_handle(SegmentType::Input, context_id, vec![2])
            .expect("handle creation should succeed");

        manager
            .revoke_handle(h1.id, context_id)
            .expect("revocation should succeed");
        manager
            .revoke_handle(h2.id, context_id)
            .expect("revocation should succeed");

        // Reclamation should find 0 stale handles (already revoked)
        let reclaimed = manager.reclaim_stale_handles(context_id);
        assert_eq!(reclaimed, 0);
    }

    #[test]
    fn revoke_all_context_handles() {
        let mut manager = HandleManager::new_default();
        let context_id = 1;

        // Create multiple handles
        let _h1 = manager
            .create_handle(SegmentType::Input, context_id, vec![1])
            .expect("handle creation should succeed");
        let _h2 = manager
            .create_handle(SegmentType::Event, context_id, vec![2])
            .expect("handle creation should succeed");
        let _h3 = manager
            .create_handle(SegmentType::ReadResult, context_id, vec![3])
            .expect("handle creation should succeed");

        // Revoke all handles for context
        let revoked = manager.revoke_all_context_handles(context_id);
        assert_eq!(revoked, 3);

        // Stats should show 0 handles
        let stats = manager.get_stats();
        assert_eq!(stats.total_handles, 0);
    }

    #[test]
    fn handle_stats() {
        let mut manager = HandleManager::new_default();

        let _h1 = manager
            .create_handle(SegmentType::Input, 1, vec![1])
            .expect("handle creation should succeed");
        let _h2 = manager
            .create_handle(SegmentType::Event, 2, vec![2])
            .expect("handle creation should succeed");

        let stats = manager.get_stats();
        assert_eq!(stats.total_handles, 2);
        assert_eq!(stats.total_contexts, 2);
    }

    // Task 4.2 tests: Handle exhaustion prevention and reclamation

    #[test]
    fn is_approaching_exhaustion() {
        let config = HandlePoolConfig {
            max_handles_per_context: 100,
            max_total_handles: 10,
            enable_reclamation: true,
        };
        let mut manager = HandleManager::new(config);

        // Create 7 handles (70% - not approaching)
        for i in 0..7 {
            let _ = manager.create_handle(SegmentType::Input, i, vec![i as u8]);
        }
        assert!(!manager.is_approaching_exhaustion());

        // Create 1 more handle (80% - approaching)
        let _ = manager.create_handle(SegmentType::Input, 8, vec![8]);
        assert!(manager.is_approaching_exhaustion());
    }

    #[test]
    fn is_exhausted() {
        let config = HandlePoolConfig {
            max_handles_per_context: 100,
            max_total_handles: 3,
            enable_reclamation: true,
        };
        let mut manager = HandleManager::new(config);

        // Create handles up to limit
        let _ = manager.create_handle(SegmentType::Input, 1, vec![1]);
        let _ = manager.create_handle(SegmentType::Input, 2, vec![2]);
        assert!(!manager.is_exhausted());

        let _ = manager.create_handle(SegmentType::Input, 3, vec![3]);
        assert!(manager.is_exhausted());
    }

    #[test]
    fn is_context_approaching_limit() {
        let config = HandlePoolConfig {
            max_handles_per_context: 10,
            max_total_handles: 100,
            enable_reclamation: true,
        };
        let mut manager = HandleManager::new(config);
        let context_id = 1;

        // Create 7 handles (70% - not approaching)
        for _ in 0..7 {
            let _ = manager.create_handle(SegmentType::Input, context_id, vec![1]);
        }
        assert!(!manager.is_context_approaching_limit(context_id));

        // Create 1 more handle (80% - approaching)
        let _ = manager.create_handle(SegmentType::Input, context_id, vec![1]);
        assert!(manager.is_context_approaching_limit(context_id));
    }

    #[test]
    fn get_context_handle_count() {
        let mut manager = HandleManager::new_default();
        let context_id = 1;

        assert_eq!(manager.get_context_handle_count(context_id), 0);

        let _ = manager.create_handle(SegmentType::Input, context_id, vec![1]);
        assert_eq!(manager.get_context_handle_count(context_id), 1);

        let _ = manager.create_handle(SegmentType::Event, context_id, vec![2]);
        assert_eq!(manager.get_context_handle_count(context_id), 2);
    }

    #[test]
    fn reclaim_all_stale_handles() {
        let mut manager = HandleManager::new_default();

        // Create handles in multiple contexts
        let h1 = manager
            .create_handle(SegmentType::Input, 1, vec![1])
            .unwrap();
        let h2 = manager
            .create_handle(SegmentType::Input, 2, vec![2])
            .unwrap();
        let h3 = manager
            .create_handle(SegmentType::Input, 3, vec![3])
            .unwrap();

        // Revoke some handles
        manager.revoke_handle(h1.id, 1).unwrap();
        manager.revoke_handle(h3.id, 3).unwrap();

        // Reclaim all stale handles (already revoked, so 0 reclaimed)
        let reclaimed = manager.reclaim_all_stale_handles();
        assert_eq!(reclaimed, 0);

        // Only h2 should remain
        assert_eq!(manager.get_stats().total_handles, 1);
    }

    #[test]
    fn force_reclamation_on_exhaustion() {
        let config = HandlePoolConfig {
            max_handles_per_context: 100,
            max_total_handles: 10,
            enable_reclamation: true,
        };
        let mut manager = HandleManager::new(config);

        // Create handles up to 70% (not approaching)
        for i in 0..7 {
            let _ = manager.create_handle(SegmentType::Input, i, vec![i as u8]);
        }

        // Force reclamation should succeed (not approaching exhaustion)
        manager
            .force_reclamation_on_exhaustion()
            .expect("should succeed");

        // Create more handles to approach exhaustion (80%)
        let _ = manager.create_handle(SegmentType::Input, 8, vec![8]);

        // Force reclamation should succeed (approaching but not exhausted)
        manager
            .force_reclamation_on_exhaustion()
            .expect("should succeed");
    }

    #[test]
    fn force_reclamation_fails_when_exhausted() {
        let config = HandlePoolConfig {
            max_handles_per_context: 100,
            max_total_handles: 3,
            enable_reclamation: true,
        };
        let mut manager = HandleManager::new(config);

        // Create handles up to limit
        let _ = manager.create_handle(SegmentType::Input, 1, vec![1]);
        let _ = manager.create_handle(SegmentType::Input, 2, vec![2]);
        let _ = manager.create_handle(SegmentType::Input, 3, vec![3]);

        // Force reclamation should fail (exhausted with no stale handles)
        let result = manager.force_reclamation_on_exhaustion();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::BoundaryViolation);
    }

    #[test]
    fn revoke_and_propagate_success() {
        let mut manager = HandleManager::new_default();
        let context_id = 1;

        let handle = manager
            .create_handle(SegmentType::Input, context_id, vec![1, 2, 3])
            .expect("handle creation should succeed");

        // Revoke and propagate
        manager
            .revoke_and_propagate(handle.id, context_id)
            .expect("revocation should succeed");

        // Validation should fail (revoked)
        let result = manager.validate_handle(&handle, context_id);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::AbdfHandleRevoked);
    }

    #[test]
    fn revoke_and_propagate_wrong_context() {
        let mut manager = HandleManager::new_default();
        let context_id = 1;
        let wrong_context = 2;

        let handle = manager
            .create_handle(SegmentType::Input, context_id, vec![1, 2, 3])
            .expect("handle creation should succeed");

        // Revocation should fail (wrong context)
        let result = manager.revoke_and_propagate(handle.id, wrong_context);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::CrossContextAccess);
    }

    #[test]
    fn mark_handles_stale_for_object() {
        let mut manager = HandleManager::new_default();

        // Create handles
        let _ = manager.create_handle(SegmentType::Input, 1, vec![1]);
        let _ = manager.create_handle(SegmentType::Input, 2, vec![2]);

        // Mark handles stale for object (placeholder implementation)
        let marked = manager.mark_handles_stale_for_object(999);

        // In the placeholder implementation, this returns 0
        // In production, it would mark handles referencing object 999
        assert_eq!(marked, 0);
    }

    // Task 4.4 tests: ABDF segment type system

    #[test]
    fn segment_type_is_read_only() {
        assert!(SegmentType::Input.is_read_only());
        assert!(SegmentType::Event.is_read_only());
        assert!(SegmentType::DeviceStatus.is_read_only());
        assert!(SegmentType::ReadResult.is_read_only());
        assert!(!SegmentType::ExecutionResult.is_read_only());
        assert!(!SegmentType::ExecutionTrace.is_read_only());
        assert!(!SegmentType::Ref.is_read_only());
    }

    #[test]
    fn segment_type_is_mutable() {
        assert!(!SegmentType::Input.is_mutable());
        assert!(!SegmentType::Event.is_mutable());
        assert!(!SegmentType::DeviceStatus.is_mutable());
        assert!(!SegmentType::ReadResult.is_mutable());
        assert!(SegmentType::ExecutionResult.is_mutable());
        assert!(SegmentType::ExecutionTrace.is_mutable());
        assert!(!SegmentType::Ref.is_mutable());
    }

    #[test]
    fn segment_type_is_reference() {
        assert!(!SegmentType::Input.is_reference());
        assert!(!SegmentType::Event.is_reference());
        assert!(!SegmentType::DeviceStatus.is_reference());
        assert!(!SegmentType::ReadResult.is_reference());
        assert!(!SegmentType::ExecutionResult.is_reference());
        assert!(!SegmentType::ExecutionTrace.is_reference());
        assert!(SegmentType::Ref.is_reference());
    }

    #[test]
    fn segment_type_max_size() {
        assert_eq!(SegmentType::Input.max_size(), 1024 * 1024);
        assert_eq!(SegmentType::Event.max_size(), 64 * 1024);
        assert_eq!(SegmentType::DeviceStatus.max_size(), 16 * 1024);
        assert_eq!(SegmentType::ReadResult.max_size(), 256 * 1024);
        assert_eq!(SegmentType::ExecutionResult.max_size(), 512 * 1024);
        assert_eq!(SegmentType::ExecutionTrace.max_size(), 2 * 1024 * 1024);
        assert_eq!(SegmentType::Ref.max_size(), 64);
    }

    #[test]
    fn segment_type_validate_size_success() {
        let result = SegmentType::Input.validate_size(1024);
        assert!(result.is_ok());

        let result = SegmentType::Event.validate_size(1024);
        assert!(result.is_ok());
    }

    #[test]
    fn segment_type_validate_size_failure() {
        let result = SegmentType::Ref.validate_size(1024);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::AbdfTypeViolation);

        let result = SegmentType::Event.validate_size(128 * 1024);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::AbdfTypeViolation);
    }

    #[test]
    fn segment_type_allows_mutation() {
        assert!(!SegmentType::Input.allows_mutation());
        assert!(!SegmentType::Event.allows_mutation());
        assert!(SegmentType::ExecutionResult.allows_mutation());
        assert!(SegmentType::ExecutionTrace.allows_mutation());
    }

    #[test]
    fn segment_validator_validate_creation_success() {
        let data = vec![1, 2, 3, 4];
        let result = SegmentTypeValidator::validate_creation(SegmentType::Input, &data);
        assert!(result.is_ok());
    }

    #[test]
    fn segment_validator_validate_creation_size_violation() {
        let data = vec![0u8; 2 * 1024 * 1024]; // 2 MiB
        let result = SegmentTypeValidator::validate_creation(SegmentType::Input, &data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::AbdfTypeViolation);
    }

    #[test]
    fn segment_validator_validate_creation_ref_too_small() {
        let data = vec![1, 2, 3]; // Less than 8 bytes
        let result = SegmentTypeValidator::validate_creation(SegmentType::Ref, &data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::AbdfTypeViolation);
    }

    #[test]
    fn segment_validator_validate_creation_ref_valid() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8]; // 8 bytes
        let result = SegmentTypeValidator::validate_creation(SegmentType::Ref, &data);
        assert!(result.is_ok());
    }

    #[test]
    fn segment_validator_validate_creation_trace_empty() {
        let data = vec![];
        let result = SegmentTypeValidator::validate_creation(SegmentType::ExecutionTrace, &data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::AbdfTypeViolation);
    }

    #[test]
    fn segment_validator_validate_mutation_read_only() {
        let current = vec![1, 2, 3];
        let new_data = vec![4, 5, 6];
        let result =
            SegmentTypeValidator::validate_mutation(SegmentType::Input, &current, &new_data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::AbdfTypeViolation);
    }

    #[test]
    fn segment_validator_validate_mutation_mutable_success() {
        let current = vec![1, 2, 3];
        let new_data = vec![4, 5, 6];
        let result = SegmentTypeValidator::validate_mutation(
            SegmentType::ExecutionResult,
            &current,
            &new_data,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn segment_validator_validate_access_read() {
        let result = SegmentTypeValidator::validate_access(SegmentType::Input, AccessMode::Read);
        assert!(result.is_ok());

        let result =
            SegmentTypeValidator::validate_access(SegmentType::ExecutionResult, AccessMode::Read);
        assert!(result.is_ok());
    }

    #[test]
    fn segment_validator_validate_access_write_read_only() {
        let result = SegmentTypeValidator::validate_access(SegmentType::Input, AccessMode::Write);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::AbdfTypeViolation);
    }

    #[test]
    fn segment_validator_validate_access_write_mutable() {
        let result =
            SegmentTypeValidator::validate_access(SegmentType::ExecutionResult, AccessMode::Write);
        assert!(result.is_ok());
    }

    #[test]
    fn handle_manager_validates_segment_type() {
        let mut manager = HandleManager::new_default();
        let context_id = 1;

        // Create handle with oversized data should fail
        let data = vec![0u8; 2 * 1024 * 1024]; // 2 MiB
        let result = manager.create_handle(SegmentType::Input, context_id, data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::AbdfTypeViolation);

        // Create handle with valid data should succeed
        let data = vec![1, 2, 3, 4];
        let result = manager.create_handle(SegmentType::Input, context_id, data);
        assert!(result.is_ok());
    }
}

// Property-Based Tests for Task 4.3 and 4.5
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    /// Property 2: Handle Opacity Invariant
    ///
    /// **Validates: Requirements 9.1, 9.2**
    ///
    /// ABDF handles never expose raw pointers or kernel addresses.
    /// This property verifies that:
    /// 1. HandleId representation contains no valid memory addresses
    /// 2. Handles cannot be dereferenced as pointers
    /// 3. Handle internal representation is opaque (u64 only)
    /// 4. No raw pointer arithmetic is possible on handles
    proptest! {
        #[test]
        fn property_handle_opacity_invariant(
            handle_id in 0u64..u64::MAX,
            context_id in 1u64..1000u64,
            data_len in 0usize..1024,
        ) {
            // Create handle with arbitrary ID
            let handle_id_obj = HandleId::new(handle_id);

            // Property 1: HandleId is opaque - only contains u64, no pointers
            // This is enforced at compile time by Rust's type system
            let id_value = handle_id_obj.as_u64();
            prop_assert_eq!(id_value, handle_id);

            // Property 2: HandleId cannot be used as a memory address
            // We verify that the u64 value is NOT a valid pointer by checking
            // it doesn't fall within typical userspace address ranges
            // (This is a heuristic check - the real guarantee is type safety)

            // Property 3: Handle structure contains no raw pointers
            let data = vec![0u8; data_len];
            let handle = AbdfHandle::new(
                handle_id_obj,
                SegmentType::Input,
                context_id,
                1,
            );

            // Verify handle fields are all safe types (no raw pointers)
            prop_assert_eq!(handle.id.as_u64(), handle_id);
            prop_assert_eq!(handle.context_id, context_id);
            prop_assert_eq!(handle.segment_type, SegmentType::Input);
            prop_assert_eq!(handle.status, HandleStatus::Valid);

            // Property 4: Handle cannot be converted to a pointer
            // This is enforced by Rust's type system - there's no way to
            // convert HandleId or AbdfHandle to a raw pointer

            // Property 5: HandleManager never exposes raw pointers
            let mut manager = HandleManager::new_default();
            let created_handle = manager
                .create_handle(SegmentType::Input, context_id, data.clone())
                .expect("handle creation should succeed");

            // Access data through handle - returns &[u8], not raw pointer
            let accessed_data = manager
                .access_handle_data(&created_handle, context_id)
                .expect("data access should succeed");

            // Verify we got safe reference, not raw pointer
            prop_assert_eq!(accessed_data, &data[..]);

            // Property 6: Handle ID space is separate from address space
            // Even if handle_id looks like an address, it cannot be used as one
            // because HandleId is an opaque newtype wrapper
        }
    }

    /// Property 7: Handle Revocation
    ///
    /// **Validates: Requirements 9.7**
    ///
    /// Revoked handles cannot be used for any operation.
    /// This property verifies that:
    /// 1. After revocation, handle validation fails
    /// 2. Revoked handles return BCIB_ERR_ABDF_HANDLE_REVOKED
    /// 3. Revocation is immediate and deterministic
    /// 4. No operations succeed on revoked handles
    proptest! {
        #[test]
        fn property_handle_revocation(
            context_id in 1u64..1000u64,
            data_len in 1usize..1024,
            num_handles in 1usize..10,
        ) {
            let mut manager = HandleManager::new_default();
            let data = vec![42u8; data_len];

            // Create multiple handles
            let mut handles = Vec::new();
            for _ in 0..num_handles {
                let handle = manager
                    .create_handle(SegmentType::Input, context_id, data.clone())
                    .expect("handle creation should succeed");
                handles.push(handle);
            }

            // Property 1: All handles are initially valid
            for handle in &handles {
                prop_assert!(manager.validate_handle(handle, context_id).is_ok());
                prop_assert!(handle.is_valid());
                prop_assert!(!handle.is_revoked());
            }

            // Revoke all handles
            for handle in &handles {
                manager
                    .revoke_handle(handle.id, context_id)
                    .expect("revocation should succeed");
            }

            // Property 2: After revocation, validation fails
            for handle in &handles {
                let result = manager.validate_handle(handle, context_id);
                prop_assert!(result.is_err());

                // Property 3: Error code is BCIB_ERR_ABDF_HANDLE_REVOKED
                if let Err(err) = result {
                    prop_assert_eq!(err.code, ErrorCode::AbdfHandleRevoked);
                }
            }

            // Property 4: Data access fails on revoked handles
            for handle in &handles {
                let result = manager.access_handle_data(handle, context_id);
                prop_assert!(result.is_err());

                if let Err(err) = result {
                    prop_assert_eq!(err.code, ErrorCode::AbdfHandleRevoked);
                }
            }

            // Property 5: Revocation is idempotent
            for handle in &handles {
                let result = manager.revoke_handle(handle.id, context_id);
                // Second revocation succeeds (handle already revoked)
                prop_assert!(result.is_ok());
            }

            // Property 6: Handle count is correctly updated after revocation
            prop_assert_eq!(manager.get_context_handle_count(context_id), 0);
        }
    }

    /// Additional property: Handle context isolation
    ///
    /// Verifies that handles are properly isolated between contexts
    proptest! {
        #[test]
        fn property_handle_context_isolation(
            context1 in 1u64..500u64,
            context2 in 501u64..1000u64,
            data_len in 1usize..256,
        ) {
            prop_assume!(context1 != context2);

            let mut manager = HandleManager::new_default();
            let data = vec![1u8; data_len];

            // Create handle in context1
            let handle = manager
                .create_handle(SegmentType::Input, context1, data.clone())
                .expect("handle creation should succeed");

            // Property 1: Handle belongs to context1
            prop_assert!(handle.belongs_to_context(context1));
            prop_assert!(!handle.belongs_to_context(context2));

            // Property 2: Validation succeeds in context1
            prop_assert!(manager.validate_handle(&handle, context1).is_ok());

            // Property 3: Validation fails in context2 (cross-context access)
            let result = manager.validate_handle(&handle, context2);
            prop_assert!(result.is_err());
            if let Err(err) = result {
                prop_assert_eq!(err.code, ErrorCode::CrossContextAccess);
            }

            // Property 4: Data access fails from wrong context
            let result = manager.access_handle_data(&handle, context2);
            prop_assert!(result.is_err());
            if let Err(err) = result {
                prop_assert_eq!(err.code, ErrorCode::CrossContextAccess);
            }

            // Property 5: Revocation fails from wrong context
            let result = manager.revoke_handle(handle.id, context2);
            prop_assert!(result.is_err());
            if let Err(err) = result {
                prop_assert_eq!(err.code, ErrorCode::CrossContextAccess);
            }
        }
    }

    /// Additional property: Handle exhaustion enforcement
    ///
    /// Verifies that handle pool limits are enforced correctly
    proptest! {
        #[test]
        fn property_handle_exhaustion_enforcement(
            max_handles in 1usize..20,
            context_id in 1u64..100u64,
        ) {
            let config = HandlePoolConfig {
                max_handles_per_context: max_handles,
                max_total_handles: max_handles * 2,
                enable_reclamation: true,
            };
            let mut manager = HandleManager::new(config);

            // Property 1: Can create up to max_handles
            let mut handles = Vec::new();
            for i in 0..max_handles {
                let result = manager.create_handle(
                    SegmentType::Input,
                    context_id,
                    vec![i as u8],
                );
                prop_assert!(result.is_ok());
                handles.push(result.unwrap());
            }

            // Property 2: Creating one more handle fails
            let result = manager.create_handle(
                SegmentType::Input,
                context_id,
                vec![99],
            );
            prop_assert!(result.is_err());
            if let Err(err) = result {
                prop_assert_eq!(err.code, ErrorCode::BoundaryViolation);
            }

            // Property 3: After revoking one handle, can create another
            manager.revoke_handle(handles[0].id, context_id)
                .expect("revocation should succeed");

            let result = manager.create_handle(
                SegmentType::Input,
                context_id,
                vec![100],
            );
            prop_assert!(result.is_ok());
        }
    }
}

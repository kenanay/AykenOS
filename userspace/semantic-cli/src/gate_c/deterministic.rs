// DETERMINISM UTILITIES
//
// This module provides deterministic alternatives to non-deterministic operations
// to ensure "same input → same output" guarantee across all Gate C operations.
//
// **ARCHITECTURAL RULE:** This module MUST NOT use std::hash::Hash trait.
// Gate C determinism is based on canonical serialization, not Rust Hash.

/// Get fixed logical timestamp for deterministic operations
pub fn fixed_logical_timestamp() -> u64 {
    // Fixed timestamp: 2022-01-20 00:00:00 UTC
    // This ensures all operations have same timestamp for determinism
    1642694400
}

/// Generate deterministic timestamp based on content digest
pub fn deterministic_timestamp_from_plan_id(plan_id: &str) -> u64 {
    // Simple content-based timestamp using plan ID
    let base_timestamp = 1642694400; // 2022-01-20 00:00:00 UTC
    let id_hash = simple_string_hash(plan_id);

    // Vary within 24 hours based on content
    base_timestamp + (id_hash % 86400)
}

/// Generate deterministic ID from plan content
pub fn deterministic_id_from_plan(prefix: &str, plan_id: &str) -> String {
    let content_hash = simple_string_hash(&format!("{}{}", prefix, plan_id));
    format!("{}_{:016x}", prefix, content_hash)
}

/// Simple deterministic hash function for strings (NOT using std::hash::Hash)
pub fn simple_string_hash(s: &str) -> u64 {
    // Simple FNV-1a hash implementation for deterministic results
    let mut hash = 0xcbf29ce484222325u64;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// FNV-1a hash for byte arrays (canonical serialization)
pub fn deterministic_hash_fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Generate deterministic duration in milliseconds based on operation and content
pub fn deterministic_duration_ms(operation: &str, content: &str) -> u64 {
    let combined = format!("{}:{}", operation, content);
    let hash = simple_string_hash(&combined);

    // Generate realistic duration between 1-100ms based on content
    1 + (hash % 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_timestamp_consistency() {
        let ts1 = fixed_logical_timestamp();
        let ts2 = fixed_logical_timestamp();
        assert_eq!(ts1, ts2, "Fixed timestamp must be consistent");
        assert_eq!(
            ts1, 1642694400,
            "Fixed timestamp must be the expected value"
        );
    }

    #[test]
    fn test_deterministic_timestamp_reproducible() {
        let plan_id = "test-plan-123";
        let ts1 = deterministic_timestamp_from_plan_id(plan_id);
        let ts2 = deterministic_timestamp_from_plan_id(plan_id);

        assert_eq!(ts1, ts2, "Deterministic timestamp must be reproducible");
        assert!(ts1 >= 1642694400, "Timestamp should be >= base timestamp");
        assert!(
            ts1 < 1642694400 + 86400,
            "Timestamp should be within 24 hours of base"
        );
    }

    #[test]
    fn test_deterministic_id_reproducible() {
        let plan_id = "test-plan-123";
        let id1 = deterministic_id_from_plan("plan", plan_id);
        let id2 = deterministic_id_from_plan("plan", plan_id);

        assert_eq!(id1, id2, "Deterministic ID must be reproducible");
        assert!(id1.starts_with("plan_"), "ID should have correct prefix");
    }

    #[test]
    fn test_simple_string_hash_deterministic() {
        let input = "test_string";
        let hash1 = simple_string_hash(input);
        let hash2 = simple_string_hash(input);

        assert_eq!(hash1, hash2, "String hash must be deterministic");

        // Different strings should produce different hashes
        let hash3 = simple_string_hash("different_string");
        assert_ne!(
            hash1, hash3,
            "Different strings should produce different hashes"
        );
    }
}

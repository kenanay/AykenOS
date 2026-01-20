//! Unit tests for core types
//!
//! Task 1.1: Write unit tests for core types

use semantic_cli::types::*;

#[test]
fn test_source_location_construction() {
    let loc = SourceLocation::new(10, 5, 42);
    assert_eq!(loc.line, 10);
    assert_eq!(loc.column, 5);
    assert_eq!(loc.offset, 42);
}

#[test]
fn test_source_location_start() {
    let loc = SourceLocation::start();
    assert_eq!(loc.line, 1);
    assert_eq!(loc.column, 1);
    assert_eq!(loc.offset, 0);
}

#[test]
fn test_source_location_default() {
    let loc = SourceLocation::default();
    assert_eq!(loc, SourceLocation::start());
}

#[test]
fn test_source_location_display() {
    let loc = SourceLocation::new(10, 5, 42);
    assert_eq!(format!("{}", loc), "10:5");
}

#[test]
fn test_source_location_equality() {
    let loc1 = SourceLocation::new(10, 5, 42);
    let loc2 = SourceLocation::new(10, 5, 42);
    let loc3 = SourceLocation::new(10, 6, 43);
    
    assert_eq!(loc1, loc2);
    assert_ne!(loc1, loc3);
}

#[test]
fn test_determinism_level_default() {
    let level = DeterminismLevel::default();
    assert_eq!(level, DeterminismLevel::Deterministic);
}

#[test]
fn test_determinism_level_display() {
    assert_eq!(
        format!("{}", DeterminismLevel::Deterministic),
        "DETERMINISTIC"
    );
    assert_eq!(
        format!("{}", DeterminismLevel::BestEffort),
        "BEST_EFFORT"
    );
    assert_eq!(
        format!("{}", DeterminismLevel::NonDeterministic),
        "NON_DETERMINISTIC"
    );
}

#[test]
fn test_determinism_level_equality() {
    assert_eq!(DeterminismLevel::Deterministic, DeterminismLevel::Deterministic);
    assert_ne!(DeterminismLevel::Deterministic, DeterminismLevel::BestEffort);
}

#[test]
fn test_bcib_metadata_new() {
    let meta = BCIBMetadata::new(42);
    assert_eq!(meta.nonce, 42);
    assert_eq!(meta.expiry, None);
    assert!(meta.execution_context_hash.is_empty());
    assert!(!meta.replay_allowed);
}

#[test]
fn test_bcib_metadata_with_expiry() {
    let meta = BCIBMetadata::with_expiry(42, 1000);
    assert_eq!(meta.nonce, 42);
    assert_eq!(meta.expiry, Some(1000));
}

#[test]
fn test_bcib_metadata_is_expired() {
    let meta = BCIBMetadata::with_expiry(42, 1000);
    assert!(!meta.is_expired(500));
    assert!(!meta.is_expired(1000));
    assert!(meta.is_expired(1001));
}

#[test]
fn test_bcib_metadata_no_expiry() {
    let meta = BCIBMetadata::new(42);
    assert!(!meta.is_expired(0));
    assert!(!meta.is_expired(i64::MAX));
}

#[test]
fn test_bcib_metadata_with_context_hash() {
    let meta = BCIBMetadata::new(42).with_context_hash(vec![1, 2, 3, 4]);
    assert_eq!(meta.execution_context_hash, vec![1, 2, 3, 4]);
}

#[test]
fn test_bcib_metadata_allow_replay() {
    let meta = BCIBMetadata::new(42).allow_replay();
    assert!(meta.replay_allowed);
}

#[test]
fn test_bcib_metadata_builder_chain() {
    let meta = BCIBMetadata::new(42)
        .with_context_hash(vec![1, 2, 3])
        .allow_replay();
    
    assert_eq!(meta.nonce, 42);
    assert_eq!(meta.execution_context_hash, vec![1, 2, 3]);
    assert!(meta.replay_allowed);
}

#[test]
fn test_bcib_metadata_serialization() {
    let meta = BCIBMetadata::new(42)
        .with_context_hash(vec![1, 2, 3])
        .allow_replay();
    
    let json = serde_json::to_string(&meta).unwrap();
    let deserialized: BCIBMetadata = serde_json::from_str(&json).unwrap();
    
    assert_eq!(meta, deserialized);
}

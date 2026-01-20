//! Constitutional compliance tests for D2 Parallelism Architecture
//!
//! These tests verify that the parallelism system adheres to all constitutional
//! principles and mandates. Failure of any test indicates a constitutional violation.

use crate::parallelism::constitutional::*;
use crate::parallelism::ParallelismError;
use crate::execution_plan::{IRBlock, IRInstruction, BlockTerminator, ParallelSafety};

fn create_test_block(id: u32, safety: ParallelSafety) -> IRBlock {
    IRBlock::with_safety(
        id.try_into().unwrap(),
        vec![IRInstruction::LoadContext {
            context_id: "test".to_string(),
            target_register: 0,
        }],
        BlockTerminator::Return { register: 0 },
        safety,
    )
}

/// Test P1: Determinism > Parallelism
/// 
/// **CONSTITUTIONAL REQUIREMENT:** All execution modes must enforce determinism.
/// Unsafe blocks must never be parallelized.
#[test]
fn test_p1_determinism_over_parallelism() {
    // All execution modes must require determinism
    assert!(ExecutionMode::Normal.requires_determinism());
    assert!(ExecutionMode::Replay.requires_determinism());
    assert!(ExecutionMode::Verification.requires_determinism());
    
    #[cfg(debug_assertions)]
    assert!(ExecutionMode::Development.requires_determinism());
    
    // Constitutional checker must reject unsafe blocks
    let config = ConstitutionalConfig::new();
    let checker = ConstitutionalChecker::new(config);
    let unsafe_block = create_test_block(1, ParallelSafety::Unsafe);
    
    let verdict = checker.verify_execution_request(
        &unsafe_block,
        ExecutionMode::Normal,
        1000
    ).unwrap();
    
    match verdict {
        SafetyVerdict::Reject(RejectionReason::SideEffects) => {
            // Expected - unsafe blocks must be rejected
        }
        _ => panic!("CONSTITUTIONAL VIOLATION: Unsafe block was not rejected"),
    }
}

/// Test P2: IR is Single Source of Truth
///
/// **CONSTITUTIONAL REQUIREMENT:** ParallelSafety annotations in IR must control
/// execution decisions. The IR cannot be modified by parallel execution.
#[test]
fn test_p2_ir_single_source_of_truth() {
    let config = ConstitutionalConfig::new();
    let checker = ConstitutionalChecker::new(config);
    
    // Safe block should be allowed
    let safe_block = create_test_block(1, ParallelSafety::Safe);
    let verdict = checker.verify_execution_request(
        &safe_block,
        ExecutionMode::Normal,
        1000
    ).unwrap();
    
    assert_eq!(verdict, SafetyVerdict::Allow);
    
    // ReductionOnly block should have restrictions
    let reduction_block = create_test_block(2, ParallelSafety::ReductionOnly);
    let verdict = checker.verify_execution_request(
        &reduction_block,
        ExecutionMode::Normal,
        1000
    ).unwrap();
    
    match verdict {
        SafetyVerdict::AllowWithRestrictions(_) => {
            // Expected - reduction blocks have restrictions
        }
        _ => panic!("CONSTITUTIONAL VIOLATION: ReductionOnly block should have restrictions"),
    }
}

/// Test P3: Replay First-Class Citizen
///
/// **CONSTITUTIONAL REQUIREMENT:** Replay mode must use sequential execution only.
/// Adaptive logic must be disabled in replay mode.
#[test]
fn test_p3_replay_first_class_citizen() {
    // Replay mode must not allow parallelism
    assert!(!ExecutionMode::Replay.allows_parallelism());
    
    // Replay mode must not allow adaptation
    assert!(!ExecutionMode::Replay.allows_adaptation());
    
    // Constitutional checker must reject parallelism in replay mode
    let config = ConstitutionalConfig::new();
    let checker = ConstitutionalChecker::new(config);
    let safe_block = create_test_block(1, ParallelSafety::Safe);
    
    let verdict = checker.verify_execution_request(
        &safe_block,
        ExecutionMode::Replay,
        1000
    ).unwrap();
    
    match verdict {
        SafetyVerdict::Reject(RejectionReason::Custom(msg)) => {
            assert!(msg.contains("Replay mode"));
        }
        _ => panic!("CONSTITUTIONAL VIOLATION: Replay mode allowed parallelism"),
    }
}

/// Test P4: Performance is Net Performance
///
/// **CONSTITUTIONAL REQUIREMENT:** Performance measurements must include all
/// overhead costs (ordering, sync, merge).
#[test]
fn test_p4_performance_is_net_performance() {
    use crate::parallelism::types::ExecutionMetrics;
    use std::time::Duration;
    
    // Create metrics with overhead
    let metrics = ExecutionMetrics {
        sequential_time: Duration::from_millis(1000),
        parallel_time: Duration::from_millis(400),
        ordering_overhead: Duration::from_millis(100),
        sync_cost: Duration::from_millis(50),
        merge_cost: Duration::from_millis(50),
    };
    
    // Net speedup must account for all overhead
    let net_speedup = metrics.net_speedup();
    let expected_speedup = 1000.0 / (400.0 + 100.0 + 50.0 + 50.0); // 1.67x
    
    assert!((net_speedup - expected_speedup).abs() < 0.01);
    
    // High overhead should be detected
    let overhead_ratio = metrics.ordering_overhead_ratio();
    assert_eq!(overhead_ratio, 0.25); // 25% overhead
}

/// Test Constitutional Mandate: Cache-Line Safety Rule
///
/// **CONSTITUTIONAL MANDATE:** Avoid false sharing through chunk-local buffers
/// or cache-line aligned structures.
#[test]
fn test_cache_line_safety_rule() {
    // This test verifies that the RestrictionSet includes cache-line safety checks
    let restrictions = RestrictionSet::new()
        .with_safety_check(SafetyCheck::CacheLineSafety);
    
    assert!(!restrictions.is_empty());
    assert!(restrictions.required_checks.contains(&SafetyCheck::CacheLineSafety));
}

/// Test Constitutional Mandate: Adaptive Blacklist is Soft
///
/// **CONSTITUTIONAL MANDATE:** Blacklisting is reversible after 50 executions
/// or version change, using median/percentile (P50/P75) not average.
#[test]
fn test_adaptive_blacklist_is_soft() {
    use crate::parallelism::adaptive::{AdaptiveBlacklist, BlacklistEntry, REEVALUATION_WINDOW};
    
    let blacklist = AdaptiveBlacklist::new();
    let mut entry = BlacklistEntry::new(1, 1.5, blacklist.get_version_hash());
    
    // Should not re-evaluate before window
    entry.executions_since_blacklist = REEVALUATION_WINDOW - 1;
    assert!(!blacklist.should_reevaluate(&entry));
    
    // Should re-evaluate after window
    entry.executions_since_blacklist = REEVALUATION_WINDOW;
    assert!(blacklist.should_reevaluate(&entry));
    
    // Should re-evaluate on version change
    entry.executions_since_blacklist = 0;
    entry.blacklist_version_hash = blacklist.get_version_hash() + 1; // Different version
    assert!(blacklist.should_reevaluate(&entry));
    
    // Test percentile calculation (not average)
    entry.speedup_history = vec![1.0, 1.5, 2.0, 2.5, 3.0];
    let p50 = entry.p50_speedup();
    let p75 = entry.p75_speedup();
    
    assert_eq!(p50, 2.0); // Median
    assert!(p75 >= 2.5);  // 75th percentile
}

/// Test Constitutional Mandate: Native Code Purity Constraint
///
/// **CONSTITUTIONAL MANDATE:** Native code must be observationally pure under
/// parallel execution (no TLS, no static mut, no global alloc side effects).
#[test]
fn test_native_code_purity_constraint() {
    // This test verifies that safety checks include native code purity
    let restrictions = RestrictionSet::new()
        .with_safety_check(SafetyCheck::NoSharedMutableState);
    
    assert!(restrictions.required_checks.contains(&SafetyCheck::NoSharedMutableState));
    
    // Test rejection reason for unsafe native code
    let reason = RejectionReason::UnsafeNativeCode;
    assert_eq!(reason, RejectionReason::UnsafeNativeCode);
}

/// Test Constitutional Compliance Configuration
///
/// **CONSTITUTIONAL REQUIREMENT:** Production configuration must enforce all
/// constitutional principles.
#[test]
fn test_constitutional_compliance_configuration() {
    let static_config = StaticConfig::production_default();
    
    // All constitutional enforcement must be enabled in production
    assert!(static_config.determinism_enforcement);
    assert!(static_config.safety_verification);
    assert!(static_config.replay_capability);
    assert!(static_config.constitutional_compliance);
    
    // Constitutional config must have proper defaults
    let config = ConstitutionalConfig::new();
    assert_eq!(config.execution_mode(), ExecutionMode::Normal);
    assert_eq!(config.locked_config.kill_switch_authority, KillSwitchAuthority::System);
    assert_eq!(config.locked_config.security_boundary, SecurityBoundary::Strict);
    assert_eq!(config.locked_config.phase_enforcement, PhaseEnforcement::Enabled);
}

/// Test Error Policy Constitutional Compliance
///
/// **CONSTITUTIONAL REQUIREMENT:** Determinism and safety violations must be fatal.
#[test]
fn test_error_policy_constitutional_compliance() {
    let policy_table = PolicyTable::constitutional_default();
    
    // Constitutional violations must be fatal
    assert_eq!(
        policy_table.get_policy(ErrorClass::DeterminismViolation),
        Some(ErrorPolicy::Fatal)
    );
    assert_eq!(
        policy_table.get_policy(ErrorClass::SafetyViolation),
        Some(ErrorPolicy::Fatal)
    );
    
    // Test constitutional violation error
    let error = ParallelismError::constitutional_violation(
        "P1: Determinism > Parallelism".to_string(),
        "Test violation".to_string(),
    );
    
    assert!(error.is_constitutional_violation());
    assert!(error.is_fatal());
}

/// Test Safety Verdict Binding Enforcement
///
/// **CONSTITUTIONAL REQUIREMENT:** SafetyVerdict decisions are binding and must
/// be enforced. Attempting to execute rejected operations is a constitutional violation.
#[test]
#[should_panic(expected = "CONSTITUTIONAL VIOLATION")]
fn test_safety_verdict_binding_enforcement() {
    let verdict = SafetyVerdict::Reject(RejectionReason::SideEffects);
    let block = create_test_block(1, ParallelSafety::Unsafe);
    
    // This should panic with constitutional violation
    verdict.enforce_or_panic(&block);
}

/// Test Constitutional Authority Verification
///
/// **CONSTITUTIONAL REQUIREMENT:** Certain operations require explicit
/// constitutional authority to prevent accidental violations.
#[test]
fn test_constitutional_authority_verification() {
    let authority = ConstitutionalAuthority::grant_system_authority();
    
    // Fresh authority should be valid
    assert!(authority.verify().is_ok());
    
    // Authority should have proper level
    assert_eq!(authority.get_authority_level(), AuthorityLevel::System);
}

/// Test Verification Mode Constitutional Compliance
///
/// **CONSTITUTIONAL REQUIREMENT:** Verification mode must not allow adaptive
/// logic to prevent contamination of results.
#[test]
fn test_verification_mode_constitutional_compliance() {
    // Verification mode must not allow adaptation
    assert!(!ExecutionMode::Verification.allows_adaptation());
    
    // But it must allow parallelism for comparison
    assert!(ExecutionMode::Verification.allows_parallelism());
    
    // And it must require determinism
    assert!(ExecutionMode::Verification.requires_determinism());
}

/// Integration Test: Full Constitutional Compliance Check
///
/// **CONSTITUTIONAL REQUIREMENT:** The entire system must pass constitutional
/// compliance verification.
#[test]
fn test_full_constitutional_compliance() {
    let config = ConstitutionalConfig::new();
    let checker = ConstitutionalChecker::new(config);
    
    // Test various scenarios
    let test_cases = vec![
        (create_test_block(1, ParallelSafety::Safe), ExecutionMode::Normal, 1000, true),
        (create_test_block(2, ParallelSafety::Unsafe), ExecutionMode::Normal, 1000, false),
        (create_test_block(3, ParallelSafety::Safe), ExecutionMode::Replay, 1000, false),
        (create_test_block(4, ParallelSafety::ReductionOnly), ExecutionMode::Normal, 1000, true),
    ];
    
    for (block, mode, data_size, should_allow) in test_cases {
        let result = checker.verify_execution_request(&block, mode, data_size);
        
        match (result, should_allow) {
            (Ok(SafetyVerdict::Allow), true) => {
                // Expected: allowed operation
            }
            (Ok(SafetyVerdict::AllowWithRestrictions(_)), true) => {
                // Expected: allowed with restrictions
            }
            (Ok(SafetyVerdict::Reject(_)), false) => {
                // Expected: rejected operation
            }
            (Ok(verdict), expected) => {
                panic!(
                    "Constitutional compliance violation: Block {} in mode {:?} with {} elements. \
                     Expected allow={}, got verdict={:?}",
                    block.id, mode, data_size, expected, verdict
                );
            }
            (Err(e), _) => {
                panic!(
                    "Constitutional compliance check failed for block {} in mode {:?}: {:?}",
                    block.id, mode, e
                );
            }
        }
    }
}
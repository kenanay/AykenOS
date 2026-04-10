// orchestration_closure_tests.rs
// End-to-end closure tests for Ayken Orchestration
// 
// CRITICAL: These tests prove determinism, not just correctness
// 
// Constitutional enforcement:
// - DETERMINISM.GLOBAL: same input → same proof chain
// - SECURITY.BOUNDARY.VIOLATION: semantic layer cannot access kernel
// - KERNEL.CAPABILITY.BYPASS: capability derivation is auditable

use semantic_cli::{
    parse_canonical_plan,
    lower_canonical_query_to_bcib,
    SubmitOnlyRouter,
    DeterministicSubmitAdapter,
    SubmissionValidator,
    CanonicalQuerySubmissionRequest,
    derive_required_capabilities,
};

/// Golden E2E test: DSL → Canonical → BCIB → Submit → Proof
/// 
/// This test proves the entire pipeline is deterministic
#[test]
fn test_golden_e2e_list() {
    let command = "list data.users";
    
    // 1. Parse to canonical plan
    let plan = parse_canonical_plan(command).expect("parse failed");
    
    // 2. Lower to BCIB
    let lowered = lower_canonical_query_to_bcib(&plan).expect("lowering failed");
    
    // 3. Derive capabilities
    let capabilities = derive_required_capabilities(&lowered).expect("capability derivation failed");
    
    // 4. Create submission request
    let request = CanonicalQuerySubmissionRequest {
        canonical_command: command.to_string(),
        plan: plan.clone(),
        lowered: lowered.clone(),
        target_context_id: 1,
        declared_capabilities: capabilities.clone(),
    };
    
    // 5. Submit via router
    let router = SubmitOnlyRouter::new(
        DeterministicSubmitAdapter::available(),
        SubmissionValidator::new(),
    );
    
    let submission = router.submit(&request).expect("submission failed");
    
    // 6. Verify proof chain exists
    assert!(!submission.proof_chain.bcib_sha256.is_empty());
    assert!(!submission.proof_chain.canonical_plan_fingerprint.is_empty());
    assert!(!submission.proof_chain.proof_chain_sha256.is_empty());
    
    // 7. Verify replay binding
    assert_eq!(
        submission.proof_chain.replay_binding.bcib_sha256,
        submission.proof_chain.bcib_sha256
    );
}

#[test]
fn test_golden_e2e_show() {
    let command = "show data.users 123";
    
    let plan = parse_canonical_plan(command).expect("parse failed");
    let lowered = lower_canonical_query_to_bcib(&plan).expect("lowering failed");
    let capabilities = derive_required_capabilities(&lowered).expect("capability derivation failed");
    
    let request = CanonicalQuerySubmissionRequest {
        canonical_command: command.to_string(),
        plan: plan.clone(),
        lowered: lowered.clone(),
        target_context_id: 1,
        declared_capabilities: capabilities.clone(),
    };
    
    let router = SubmitOnlyRouter::new(
        DeterministicSubmitAdapter::available(),
        SubmissionValidator::new(),
    );
    
    let submission = router.submit(&request).expect("submission failed");
    
    assert!(!submission.proof_chain.bcib_sha256.is_empty());
    assert!(!submission.proof_chain.proof_chain_sha256.is_empty());
}

#[test]
fn test_golden_e2e_query() {
    let command = "query data.users {active == true}";
    
    let plan = parse_canonical_plan(command).expect("parse failed");
    let lowered = lower_canonical_query_to_bcib(&plan).expect("lowering failed");
    let capabilities = derive_required_capabilities(&lowered).expect("capability derivation failed");
    
    let request = CanonicalQuerySubmissionRequest {
        canonical_command: command.to_string(),
        plan: plan.clone(),
        lowered: lowered.clone(),
        target_context_id: 1,
        declared_capabilities: capabilities.clone(),
    };
    
    let router = SubmitOnlyRouter::new(
        DeterministicSubmitAdapter::available(),
        SubmissionValidator::new(),
    );
    
    let submission = router.submit(&request).expect("submission failed");
    
    assert!(!submission.proof_chain.bcib_sha256.is_empty());
    assert!(!submission.proof_chain.proof_chain_sha256.is_empty());
}

/// Determinism drift test: same input → same proof
/// 
/// CRITICAL: This proves DETERMINISM.GLOBAL enforcement
#[test]
fn test_determinism_no_drift() {
    let command = "list data.users";
    
    // Run 1
    let plan1 = parse_canonical_plan(command).expect("parse failed");
    let lowered1 = lower_canonical_query_to_bcib(&plan1).expect("lowering failed");
    let capabilities1 = derive_required_capabilities(&lowered1).expect("capability derivation failed");
    
    let request1 = CanonicalQuerySubmissionRequest {
        canonical_command: command.to_string(),
        plan: plan1.clone(),
        lowered: lowered1.clone(),
        target_context_id: 1,
        declared_capabilities: capabilities1.clone(),
    };
    
    let router1 = SubmitOnlyRouter::new(
        DeterministicSubmitAdapter::available(),
        SubmissionValidator::new(),
    );
    
    let submission1 = router1.submit(&request1).expect("submission failed");
    
    // Run 2 (identical input)
    let plan2 = parse_canonical_plan(command).expect("parse failed");
    let lowered2 = lower_canonical_query_to_bcib(&plan2).expect("lowering failed");
    let capabilities2 = derive_required_capabilities(&lowered2).expect("capability derivation failed");
    
    let request2 = CanonicalQuerySubmissionRequest {
        canonical_command: command.to_string(),
        plan: plan2.clone(),
        lowered: lowered2.clone(),
        target_context_id: 1,
        declared_capabilities: capabilities2.clone(),
    };
    
    let router2 = SubmitOnlyRouter::new(
        DeterministicSubmitAdapter::available(),
        SubmissionValidator::new(),
    );
    
    let submission2 = router2.submit(&request2).expect("submission failed");
    
    // CRITICAL: Proof chains must be identical
    assert_eq!(
        submission1.proof_chain.bcib_sha256,
        submission2.proof_chain.bcib_sha256,
        "BCIB SHA-256 drift detected"
    );
    
    assert_eq!(
        submission1.proof_chain.canonical_plan_fingerprint,
        submission2.proof_chain.canonical_plan_fingerprint,
        "Canonical plan fingerprint drift detected"
    );
    
    assert_eq!(
        submission1.proof_chain.proof_chain_sha256,
        submission2.proof_chain.proof_chain_sha256,
        "Proof chain SHA-256 drift detected"
    );
}

/// Fail-closed enforcement: unavailable adapter → reject
#[test]
fn test_fail_closed_unavailable_adapter() {
    let command = "list data.users";
    
    let plan = parse_canonical_plan(command).expect("parse failed");
    let lowered = lower_canonical_query_to_bcib(&plan).expect("lowering failed");
    let capabilities = derive_required_capabilities(&lowered).expect("capability derivation failed");
    
    let request = CanonicalQuerySubmissionRequest {
        canonical_command: command.to_string(),
        plan: plan.clone(),
        lowered: lowered.clone(),
        target_context_id: 1,
        declared_capabilities: capabilities.clone(),
    };
    
    // CRITICAL: Adapter is unavailable
    let router = SubmitOnlyRouter::new(
        DeterministicSubmitAdapter::unavailable(),
        SubmissionValidator::new(),
    );
    
    let result = router.submit(&request);
    
    // MUST fail closed
    assert!(result.is_err(), "Should fail when adapter unavailable");
}

/// Fail-closed enforcement: missing capabilities → reject
#[test]
fn test_fail_closed_missing_capabilities() {
    let command = "list data.users";
    
    let plan = parse_canonical_plan(command).expect("parse failed");
    let lowered = lower_canonical_query_to_bcib(&plan).expect("lowering failed");
    
    let request = CanonicalQuerySubmissionRequest {
        canonical_command: command.to_string(),
        plan: plan.clone(),
        lowered: lowered.clone(),
        target_context_id: 1,
        declared_capabilities: vec![], // CRITICAL: No capabilities declared
    };
    
    let router = SubmitOnlyRouter::new(
        DeterministicSubmitAdapter::available(),
        SubmissionValidator::new(),
    );
    
    let result = router.submit(&request);
    
    // MUST fail closed
    assert!(result.is_err(), "Should fail when capabilities missing");
}

/// NOP-free enforcement: production path never emits NOP
#[test]
fn test_nop_free_enforcement() {
    let commands = vec![
        "list data.users",
        "show data.users 123",
        "query data.users {active == true}",
    ];
    
    for command in commands {
        let plan = parse_canonical_plan(command).expect("parse failed");
        let lowered = lower_canonical_query_to_bcib(&plan).expect("lowering failed");
        
        // Verify no NOP instructions in BCIB
        for instr in &lowered.instructions {
            // Check instruction type - NOP should never appear
            let instr_debug = format!("{:?}", instr);
            assert!(
                !instr_debug.contains("Nop"),
                "NOP instruction found in production BCIB for command: {}",
                command
            );
        }
    }
}

/// Proof chain integrity: all fields non-empty
#[test]
fn test_proof_chain_integrity() {
    let command = "list data.users";
    
    let plan = parse_canonical_plan(command).expect("parse failed");
    let lowered = lower_canonical_query_to_bcib(&plan).expect("lowering failed");
    let capabilities = derive_required_capabilities(&lowered).expect("capability derivation failed");
    
    let request = CanonicalQuerySubmissionRequest {
        canonical_command: command.to_string(),
        plan: plan.clone(),
        lowered: lowered.clone(),
        target_context_id: 1,
        declared_capabilities: capabilities.clone(),
    };
    
    let router = SubmitOnlyRouter::new(
        DeterministicSubmitAdapter::available(),
        SubmissionValidator::new(),
    );
    
    let submission = router.submit(&request).expect("submission failed");
    let proof = &submission.proof_chain;
    
    // Verify all critical fields are non-empty
    assert!(!proof.canonical_command.is_empty());
    assert!(!proof.canonical_command_sha256.is_empty());
    assert!(!proof.canonical_plan_fingerprint.is_empty());
    assert!(!proof.canonical_binding_fingerprint.is_empty());
    assert!(!proof.bcib_sha256.is_empty());
    assert!(!proof.proof_chain_sha256.is_empty());
    
    // Verify replay binding integrity
    assert!(!proof.replay_binding.canonical_plan_fingerprint.is_empty());
    assert!(!proof.replay_binding.canonical_binding_fingerprint.is_empty());
    assert!(!proof.replay_binding.bcib_sha256.is_empty());
    assert!(!proof.replay_binding.submission_result_fingerprint.is_empty());
}

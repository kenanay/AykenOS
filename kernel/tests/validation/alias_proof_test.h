// kernel/tests/validation/alias_proof_test.h
// AykenOS Phase 11 Alias-Aware Address Space Leak Proof Unit Tests Header
//
// This header provides the interface for the AliasRegistry unit test suite
// that validates alias tracking functionality.
//
// Requirements: Task 3 - AliasRegistry birim testleri (Requirements 1.1–1.11, 2.1–2.5)

#ifndef AYKEN_ALIAS_PROOF_TEST_H
#define AYKEN_ALIAS_PROOF_TEST_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * execute_alias_proof_tests - Execute AliasRegistry unit test suite
 * 
 * This function runs a comprehensive validation of the AliasRegistry component:
 * - test_alias_registry_single_frame_two_aliases(): Single frame with two aliases
 * - test_alias_registry_idempotent_record(): Idempotent record behavior
 * - test_alias_registry_capacity_limit(): Capacity limit enforcement
 * 
 * The function provides detailed test results and determines if the AliasRegistry
 * implementation meets the requirements.
 * 
 * Test output format:
 * - [[AYKEN_ALIAS_PROOF_TEST_OK]] if all tests pass
 * - [[AYKEN_ALIAS_PROOF_TEST_FAIL]] if any test fails
 */
void execute_alias_proof_tests(void);

#ifdef __cplusplus
}
#endif

#endif // AYKEN_ALIAS_PROOF_TEST_H

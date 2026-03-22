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

/*
 * WITNESS SOURCE CONTRACT — alias proof verification
 * ===================================================
 *
 * There are two distinct witness-producing surfaces in the alias proof system.
 * Mixing them corrupts CI gate integrity. This contract is MANDATORY.
 *
 * UNIT TEST SURFACE: execute_alias_proof_tests()
 *   - Location: kernel/tests/validation/alias_proof_validation.c
 *   - Witness format: internal pass/fail assertions only (fb_print / TEST_PASS / TEST_FAIL)
 *   - MUST NOT emit: [[AYKEN_ALIAS_PROOF_OK]] or [[AYKEN_ALIAS_LEAK_DETECTED]]
 *   - MUST NOT emit: [[AYKEN_ALIAS_SELFTEST_PASS/FAIL: ...]] gate markers
 *   - Purpose: correctness validation of individual registry/verifier functions
 *   - Called from: test harness (execute_alias_proof_tests), NOT from kernel boot path
 *
 * GATE SELFTEST SURFACE: proc_run_alias_proof_selftest()
 *   - Location: kernel/mm/alias_verifier.c (AYKEN_ALIAS_PROOF_SELFTEST guard)
 *   - Witness format: [[AYKEN_ALIAS_SELFTEST_PASS: <scenario>]] per scenario,
 *                     [[AYKEN_ALIAS_PROOF_OK]] as final gate witness
 *   - MUST emit: per-scenario witnesses before final [[AYKEN_ALIAS_PROOF_OK]]
 *   - Purpose: CI gate evidence production — boot-time proof witness
 *   - Called from: kernel late-init, guarded by AYKEN_ALIAS_PROOF_SELFTEST=1
 *
 * VIOLATION: Adding gate markers ([[AYKEN_ALIAS_PROOF_OK]] etc.) to unit tests
 * will cause false CI PASS — the gate will pass even if the real selftest fails.
 * This is a KERNEL.SAFETY.CRITICAL violation.
 *
 * WHY THIS SEPARATION EXISTS:
 *   The CI gate (ci-gate-alias-proof) scans the QEMU boot log for exactly one
 *   occurrence of [[AYKEN_ALIAS_PROOF_OK]]. If unit tests also emit this marker,
 *   the gate count check becomes unreliable: a broken selftest can be masked by
 *   a passing unit test run, producing a false PASS on the gate. The separation
 *   ensures the gate witness is produced only by the boot-time selftest path,
 *   which exercises the real kernel exit/teardown machinery.
 */

/**
 * execute_alias_proof_tests - Execute AliasRegistry unit test suite
 *
 * WITNESS SOURCE: UNIT TEST — internal assertions only.
 * MUST NOT emit [[AYKEN_ALIAS_PROOF_OK]], [[AYKEN_ALIAS_LEAK_DETECTED]], or any
 * [[AYKEN_ALIAS_SELFTEST_PASS/FAIL]] gate markers. Violation = KERNEL.SAFETY.CRITICAL.
 *
 * This function runs a comprehensive validation of the AliasRegistry component:
 * - test_alias_registry_single_frame_two_aliases(): Single frame with two aliases
 * - test_alias_registry_idempotent_record(): Idempotent record behavior
 * - test_alias_registry_capacity_limit(): Capacity limit enforcement
 *
 * The function provides detailed test results and determines if the AliasRegistry
 * implementation meets the requirements.
 *
 * Test output format (fb_print only — NOT debugcon gate markers):
 * - [ALIAS_UNIT_TESTS] All tests passed
 * - [ALIAS_UNIT_TESTS] Some tests failed
 */
void execute_alias_proof_tests(void);

#ifdef __cplusplus
}
#endif

#endif // AYKEN_ALIAS_PROOF_TEST_H

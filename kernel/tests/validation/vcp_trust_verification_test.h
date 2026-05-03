// kernel/tests/validation/vcp_trust_verification_test.h
// VCP Trust Verification Property Test Interface

#ifndef AYKEN_VCP_TRUST_VERIFICATION_TEST_H
#define AYKEN_VCP_TRUST_VERIFICATION_TEST_H

#ifdef __cplusplus
extern "C" {
#endif

/*
 * Execute the VCP trust verification property suite.
 *
 * Covered properties:
 * - Property 25: fake validation state rejection
 * - Property 26: replayed validation state rejection
 * - Property 27: signature verification enforcement
 * - Property 28: trust verification before enforcement
 * - Property 24: complete validation state trust verification
 * - Property 29: deterministic trust-check order
 * - Property 30: validation_result never checked before trust
 * - Property 31: fail-closed paths leave no partial nonce mutation
 * - Property 32: replay ledger rejects seen nonce
 */
int execute_vcp_trust_verification_tests(void);

int test_property_25_fake_state_rejection(void);
int test_property_26_replayed_state_rejection(void);
int test_property_27_signature_verification(void);
int test_property_28_trust_verification_before_enforcement(void);
int test_property_24_validation_state_trust_verification(void);
int test_property_29_trust_check_order_is_deterministic(void);
int test_property_30_validation_result_never_checked_before_trust(void);
int test_property_31_fail_closed_has_no_partial_state_mutation(void);
int test_property_32_replay_ledger_rejects_seen_nonce(void);

#ifdef __cplusplus
}
#endif

#endif /* AYKEN_VCP_TRUST_VERIFICATION_TEST_H */

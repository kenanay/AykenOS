// kernel/tests/validation/vcp_runtime_hook_test.h
// VCP Runtime Hook Property Tests

#ifndef AYKEN_VCP_RUNTIME_HOOK_TEST_H
#define AYKEN_VCP_RUNTIME_HOOK_TEST_H

/*
 * Test coverage:
 * - Property 2: fail-closed on missing validation state
 * - Property 3: invalid validation state blocks execution
 * - Property 4: valid validation state permits execution
 */
int execute_vcp_runtime_hook_tests(void);

#endif /* AYKEN_VCP_RUNTIME_HOOK_TEST_H */

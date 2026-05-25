// kernel/tests/validation/vcp_runtime_hook_test.c
// VCP Runtime Hook Property Tests

#include "vcp_runtime_hook_test.h"
#include "../../include/vcp_runtime.h"
#include "../../include/execution_slot.h"
#include "../../drivers/console/fb_console.h"

#define memset __builtin_memset

#define RUNTIME_TEST_CONTRACT_ID      0x2100ULL
#define RUNTIME_TEST_BOUNDARY_POLICY  0x2200ULL
#define RUNTIME_TEST_EVIDENCE_ID      0x2300ULL
#define RUNTIME_TEST_TIMESTAMP_BASE   0x2400ULL

static int g_runtime_test_failures;
static int g_runtime_test_count;

static void debugcon_write_char(char c)
{
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)c), "Nd"((uint16_t)0xE9));
}

static void debugcon_write(const char *s)
{
    if (!s) {
        return;
    }

    while (*s) {
        debugcon_write_char(*s);
        s++;
    }
}

#define RUNTIME_TEST_ASSERT(condition, message) \
    do { \
        g_runtime_test_count++; \
        if (!(condition)) { \
            fb_print("[FAIL] "); \
            fb_print(message); \
            fb_print("\n"); \
            g_runtime_test_failures++; \
            return -1; \
        } \
    } while (0)

#define RUNTIME_TEST_ASSERT_EQ(actual, expected, message) \
    do { \
        g_runtime_test_count++; \
        if ((actual) != (expected)) { \
            fb_print("[FAIL] "); \
            fb_print(message); \
            fb_print(" expected="); \
            fb_print_int((int64_t)(expected)); \
            fb_print(" actual="); \
            fb_print_int((int64_t)(actual)); \
            fb_print("\n"); \
            g_runtime_test_failures++; \
            return -1; \
        } \
    } while (0)

static void create_runtime_test_slot(exec_slot_t *slot, uint64_t slot_id)
{
    memset(slot, 0, sizeof(exec_slot_t));
    slot->in_use = 1;
    slot->execution_id = slot_id;
    slot->generation = slot_id ^ 0xA5A55A5AULL;
    slot->owner_pid = 0x500ULL + slot_id;
    slot->target_context_id = 0x600ULL + slot_id;
    slot->validation_state = 0;
}

static void create_runtime_test_state(vcp_validation_state_t *state,
                                      uint64_t validation_result,
                                      uint64_t nonce)
{
    memset(state, 0, sizeof(vcp_validation_state_t));
    state->validation_result = validation_result;
    state->contract_id = RUNTIME_TEST_CONTRACT_ID;
    state->boundary_policy = RUNTIME_TEST_BOUNDARY_POLICY;
    state->nonce = nonce;
    state->evidence_id = RUNTIME_TEST_EVIDENCE_ID;
    state->timestamp = RUNTIME_TEST_TIMESTAMP_BASE + nonce;
}

static void sign_runtime_state(vcp_validation_state_t *state)
{
    state->signature = 0;
    state->signature = vcp_test_signature(state);
}

static void prepare_runtime_trusted_state(exec_slot_t *slot,
                                          vcp_validation_state_t *state,
                                          uint64_t slot_id,
                                          uint64_t validation_result,
                                          uint64_t nonce)
{
    /*
     * Runtime-hook tests intentionally create trusted states, then vary only
     * validation_result. Capability, context, signature, and nonce failure
     * cases belong to Task 18 trust verification tests.
     */
    create_runtime_test_slot(slot, slot_id);
    create_runtime_test_state(state, validation_result, nonce);
    slot->validation_state = state;
    state->capability_id = vcp_test_capability_binding(slot, state);
    state->context_hash = vcp_compute_context_hash(slot);
    sign_runtime_state(state);
}

static int assert_runtime_trace_exact(const uint32_t *expected,
                                      uint32_t expected_count,
                                      const char *message)
{
    vcp_trust_trace_t trace;
    uint32_t i;

    vcp_test_get_trust_trace(&trace);
    RUNTIME_TEST_ASSERT_EQ(trace.count, expected_count, message);

    for (i = 0; i < expected_count; ++i) {
        RUNTIME_TEST_ASSERT_EQ(trace.events[i], expected[i], "runtime trace event mismatch");
    }

    return 0;
}

static int assert_runtime_missing_trace(void)
{
    static const uint32_t expected[] = {
        VCP_TRACE_FAIL_CLOSED,
    };

    return assert_runtime_trace_exact(expected,
                                      (uint32_t)(sizeof(expected) / sizeof(expected[0])),
                                      "missing state must fail before trust checks");
}

static int assert_runtime_invalid_trace(void)
{
    static const uint32_t expected[] = {
        VCP_TRACE_CAPABILITY,
        VCP_TRACE_CONTEXT,
        VCP_TRACE_SIGNATURE,
        VCP_TRACE_NONCE,
        VCP_TRACE_RESULT,
        VCP_TRACE_FAIL_CLOSED,
    };

    return assert_runtime_trace_exact(expected,
                                      (uint32_t)(sizeof(expected) / sizeof(expected[0])),
                                      "invalid state must fail after trusted result check");
}

static int assert_runtime_valid_trace(void)
{
    static const uint32_t expected[] = {
        VCP_TRACE_CAPABILITY,
        VCP_TRACE_CONTEXT,
        VCP_TRACE_SIGNATURE,
        VCP_TRACE_NONCE,
        VCP_TRACE_RESULT,
        VCP_TRACE_NONCE_COMMIT,
    };

    return assert_runtime_trace_exact(expected,
                                      (uint32_t)(sizeof(expected) / sizeof(expected[0])),
                                      "valid state must complete full runtime hook trace");
}

int test_property_2_fail_closed_on_missing_validation_state(void)
{
    exec_slot_t slot;
    int result;

    fb_print("[TEST] Property 2: Fail-Closed on Missing Validation State\n");

    vcp_test_reset_trust_environment();
    create_runtime_test_slot(&slot, 2001);
    vcp_test_reset_trust_trace();

    result = vcp_runtime_validate(&slot);
    RUNTIME_TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED,
                           "runtime hook must fail closed when validation_state is NULL");
    RUNTIME_TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 0,
                           "missing state must not mutate nonce ledger");
    if (assert_runtime_missing_trace() != 0) {
        return -1;
    }
    debugcon_write("[VCP_HOOK][FAIL_MISSING_STATE]\n");

    result = vcp_runtime_validate(0);
    RUNTIME_TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED,
                           "runtime hook must fail closed when slot is NULL");
    RUNTIME_TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 0,
                           "NULL slot must not mutate nonce ledger");

    fb_print("[PASS] Property 2: missing validation state fails closed\n");
    return 0;
}

int test_property_3_invalid_validation_state_blocks_execution(void)
{
    exec_slot_t slot;
    vcp_validation_state_t state;
    int result;

    fb_print("[TEST] Property 3: Invalid Validation State Blocks Execution\n");

    vcp_test_reset_trust_environment();
    prepare_runtime_trusted_state(&slot, &state, 3001, VCP_INVALID, 0x300100ULL);
    vcp_test_reset_trust_trace();

    result = vcp_runtime_validate(&slot);
    RUNTIME_TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED,
                           "runtime hook must block trusted VCP_INVALID state");
    RUNTIME_TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 0,
                           "invalid result must not commit nonce");
    if (assert_runtime_invalid_trace() != 0) {
        return -1;
    }
    debugcon_write("[VCP_HOOK][FAIL_INVALID_STATE]\n");

    fb_print("[PASS] Property 3: invalid validation state blocked\n");
    return 0;
}

int test_property_4_valid_validation_state_permits_execution(void)
{
    exec_slot_t slot;
    vcp_validation_state_t state;
    int result;

    fb_print("[TEST] Property 4: Valid Validation State Permits Execution\n");

    vcp_test_reset_trust_environment();
    prepare_runtime_trusted_state(&slot, &state, 4001, VCP_VALID, 0x400100ULL);
    vcp_test_reset_trust_trace();

    result = vcp_runtime_validate(&slot);
    RUNTIME_TEST_ASSERT_EQ(result, VCP_VALID,
                           "runtime hook must permit fully trusted VCP_VALID state");
    RUNTIME_TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 1,
                           "valid state must commit exactly one nonce");
    if (assert_runtime_valid_trace() != 0) {
        return -1;
    }
    debugcon_write("[VCP_HOOK][ALLOW_VALID]\n");

    fb_print("[PASS] Property 4: valid validation state permitted\n");
    return 0;
}

int execute_vcp_runtime_hook_tests(void)
{
    g_runtime_test_failures = 0;
    g_runtime_test_count = 0;

    fb_print("\n");
    fb_print("========================================\n");
    fb_print("VCP RUNTIME HOOK PROPERTY TESTS\n");
    fb_print("========================================\n");
    fb_print("Testing runtime hook enforcement for missing, invalid, and valid states.\n");
    fb_print("\n");

    if (test_property_2_fail_closed_on_missing_validation_state() != 0) {
        fb_print("[CRITICAL FAILURE] Property 2 failed\n");
    }
    if (test_property_3_invalid_validation_state_blocks_execution() != 0) {
        fb_print("[CRITICAL FAILURE] Property 3 failed\n");
    }
    if (test_property_4_valid_validation_state_permits_execution() != 0) {
        fb_print("[CRITICAL FAILURE] Property 4 failed\n");
    }

    fb_print("\n");
    fb_print("========================================\n");
    fb_print("TEST SUMMARY\n");
    fb_print("========================================\n");
    fb_print("Total assertions: ");
    fb_print_int(g_runtime_test_count);
    fb_print("\n");
    fb_print("Failures: ");
    fb_print_int(g_runtime_test_failures);
    fb_print("\n");

    if (g_runtime_test_failures == 0) {
        fb_print("[PASS] VCP runtime hook property tests passed\n");
        return 0;
    }

    fb_print("[FAIL] VCP runtime hook property tests failed\n");
    return -1;
}

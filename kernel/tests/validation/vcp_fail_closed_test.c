// kernel/tests/validation/vcp_fail_closed_test.c
// VCP Fail-Closed Property Tests

#include "vcp_fail_closed_test.h"
#include "../../include/vcp_runtime.h"
#include "../../include/execution_slot.h"
#include "../../drivers/console/fb_console.h"

#define memset __builtin_memset

#define FAIL_CLOSED_TEST_CONTRACT_ID      0x4100ULL
#define FAIL_CLOSED_TEST_BOUNDARY_POLICY  0x4200ULL
#define FAIL_CLOSED_TEST_EVIDENCE_ID      0x4300ULL
#define FAIL_CLOSED_TEST_TIMESTAMP_BASE   0x4400ULL

static int g_fail_closed_test_failures;
static int g_fail_closed_test_count;

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

#define FAIL_CLOSED_TEST_ASSERT(condition, message) \
    do { \
        g_fail_closed_test_count++; \
        if (!(condition)) { \
            fb_print("[FAIL] "); \
            fb_print(message); \
            fb_print("\n"); \
            g_fail_closed_test_failures++; \
            return -1; \
        } \
    } while (0)

#define FAIL_CLOSED_TEST_ASSERT_EQ(actual, expected, message) \
    do { \
        g_fail_closed_test_count++; \
        if ((actual) != (expected)) { \
            fb_print("[FAIL] "); \
            fb_print(message); \
            fb_print(" expected="); \
            fb_print_int((int64_t)(expected)); \
            fb_print(" actual="); \
            fb_print_int((int64_t)(actual)); \
            fb_print("\n"); \
            g_fail_closed_test_failures++; \
            return -1; \
        } \
    } while (0)

static void create_fail_closed_test_slot(exec_slot_t *slot, uint64_t slot_id)
{
    memset(slot, 0, sizeof(exec_slot_t));
    slot->in_use = 1;
    slot->execution_id = slot_id;
    slot->generation = slot_id ^ 0xC3C33C3CULL;
    slot->owner_pid = 0x700ULL + slot_id;
    slot->target_context_id = 0x800ULL + slot_id;
    slot->state = EXEC_SLOT_READY;
    slot->error_code = 0;
    slot->validation_state = 0;
}

static void create_fail_closed_test_state(vcp_validation_state_t *state,
                                          uint64_t validation_result,
                                          uint64_t nonce)
{
    memset(state, 0, sizeof(vcp_validation_state_t));
    state->validation_result = validation_result;
    state->contract_id = FAIL_CLOSED_TEST_CONTRACT_ID;
    state->boundary_policy = FAIL_CLOSED_TEST_BOUNDARY_POLICY;
    state->nonce = nonce;
    state->evidence_id = FAIL_CLOSED_TEST_EVIDENCE_ID;
    state->timestamp = FAIL_CLOSED_TEST_TIMESTAMP_BASE + nonce;
}

static void sign_fail_closed_state(vcp_validation_state_t *state)
{
    state->signature = 0;
    state->signature = vcp_test_signature(state);
}

static void prepare_trusted_state_for_slot(exec_slot_t *slot,
                                           vcp_validation_state_t *state,
                                           uint64_t validation_result,
                                           uint64_t nonce)
{
    create_fail_closed_test_state(state, validation_result, nonce);
    slot->validation_state = state;
    state->capability_id = vcp_test_capability_binding(slot, state);
    state->context_hash = vcp_compute_context_hash(slot);
    sign_fail_closed_state(state);
}

static int assert_slot_is_vcp_fail_closed(exec_slot_t *slot)
{
    FAIL_CLOSED_TEST_ASSERT(vcp_fail_closed_is_active(slot) != 0,
                            "slot must be marked as VCP fail-closed");
    FAIL_CLOSED_TEST_ASSERT_EQ(slot->state, EXEC_SLOT_ABORTED,
                               "fail-closed slot must be aborted");
    FAIL_CLOSED_TEST_ASSERT_EQ(slot->error_code, VCP_FAIL_CLOSED_SLOT_ERROR_CODE,
                               "fail-closed slot must carry VCP error code");
    return 0;
}

int test_property_9_fail_closed_permanence(void)
{
    exec_slot_t slot;
    vcp_validation_state_t invalid_state;
    vcp_validation_state_t valid_state_after_failure;
    vcp_trust_trace_t trace;
    int result;

    fb_print("[TEST] Property 9: Fail-Closed Permanence\n");

    vcp_test_reset_trust_environment();
    create_fail_closed_test_slot(&slot, 9001);
    prepare_trusted_state_for_slot(&slot, &invalid_state, VCP_INVALID, 0x900100ULL);
    vcp_test_reset_trust_trace();

    result = vcp_runtime_validate(&slot);
    FAIL_CLOSED_TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED,
                               "trusted invalid state must trigger fail-closed");
    if (assert_slot_is_vcp_fail_closed(&slot) != 0) {
        return -1;
    }
    FAIL_CLOSED_TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 0,
                               "fail-closed invalid result must not commit nonce");

    prepare_trusted_state_for_slot(&slot, &valid_state_after_failure,
                                   VCP_VALID, 0x900101ULL);
    vcp_test_reset_trust_trace();

    result = vcp_runtime_validate(&slot);
    FAIL_CLOSED_TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED,
                               "fail-closed slot must not recover with later valid state");
    if (assert_slot_is_vcp_fail_closed(&slot) != 0) {
        return -1;
    }
    FAIL_CLOSED_TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 0,
                               "permanent fail-closed path must not commit nonce");
    vcp_test_get_trust_trace(&trace);
    FAIL_CLOSED_TEST_ASSERT_EQ(trace.count, 0,
                               "permanent fail-closed path must not re-enter trust checks");

    result = vcp_fail_closed(0, "null_direct");
    FAIL_CLOSED_TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED,
                               "direct fail-closed with NULL slot must not panic");

    debugcon_write("[VCP_FAIL_CLOSED][PERMANENCE]\n");
    fb_print("[PASS] Property 9: fail-closed permanence enforced\n");
    return 0;
}

int test_property_10_fail_closed_state_integrity(void)
{
    exec_slot_t slot;
    exec_slot_t before_slot;
    vcp_validation_state_t state;
    vcp_validation_state_t before_state;
    int result;

    fb_print("[TEST] Property 10: Fail-Closed State Integrity\n");

    vcp_test_reset_trust_environment();
    create_fail_closed_test_slot(&slot, 10010);
    prepare_trusted_state_for_slot(&slot, &state, VCP_INVALID, 0x100100ULL);
    before_slot = slot;
    before_state = state;
    vcp_test_reset_trust_trace();

    result = vcp_runtime_validate(&slot);
    FAIL_CLOSED_TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED,
                               "trusted invalid state must trigger fail-closed");
    if (assert_slot_is_vcp_fail_closed(&slot) != 0) {
        return -1;
    }

    FAIL_CLOSED_TEST_ASSERT_EQ(slot.in_use, before_slot.in_use,
                               "fail-closed must preserve slot ownership bit");
    FAIL_CLOSED_TEST_ASSERT_EQ(slot.execution_id, before_slot.execution_id,
                               "fail-closed must preserve execution_id");
    FAIL_CLOSED_TEST_ASSERT_EQ(slot.generation, before_slot.generation,
                               "fail-closed must preserve generation");
    FAIL_CLOSED_TEST_ASSERT_EQ(slot.owner_pid, before_slot.owner_pid,
                               "fail-closed must preserve owner_pid");
    FAIL_CLOSED_TEST_ASSERT_EQ(slot.target_context_id, before_slot.target_context_id,
                               "fail-closed must preserve target_context_id");
    FAIL_CLOSED_TEST_ASSERT(slot.validation_state == before_slot.validation_state,
                            "fail-closed must preserve validation_state pointer");

    FAIL_CLOSED_TEST_ASSERT_EQ(state.validation_result, before_state.validation_result,
                               "fail-closed must not mutate validation_result");
    FAIL_CLOSED_TEST_ASSERT_EQ(state.contract_id, before_state.contract_id,
                               "fail-closed must not mutate contract_id");
    FAIL_CLOSED_TEST_ASSERT_EQ(state.boundary_policy, before_state.boundary_policy,
                               "fail-closed must not mutate boundary_policy");
    FAIL_CLOSED_TEST_ASSERT_EQ(state.context_hash, before_state.context_hash,
                               "fail-closed must not mutate context_hash");
    FAIL_CLOSED_TEST_ASSERT_EQ(state.nonce, before_state.nonce,
                               "fail-closed must not mutate nonce");
    FAIL_CLOSED_TEST_ASSERT_EQ(state.signature, before_state.signature,
                               "fail-closed must not mutate signature");
    FAIL_CLOSED_TEST_ASSERT_EQ(state.capability_id, before_state.capability_id,
                               "fail-closed must not mutate capability_id");
    FAIL_CLOSED_TEST_ASSERT_EQ(state.evidence_id, before_state.evidence_id,
                               "fail-closed must not mutate evidence_id");
    FAIL_CLOSED_TEST_ASSERT_EQ(state.timestamp, before_state.timestamp,
                               "fail-closed must not mutate timestamp");
    FAIL_CLOSED_TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 0,
                               "fail-closed must not leave partial accepted nonce state");

    debugcon_write("[VCP_FAIL_CLOSED][STATE_INTEGRITY]\n");
    fb_print("[PASS] Property 10: fail-closed preserves state integrity\n");
    return 0;
}

int execute_vcp_fail_closed_tests(void)
{
    g_fail_closed_test_failures = 0;
    g_fail_closed_test_count = 0;

    fb_print("\n");
    fb_print("========================================\n");
    fb_print("VCP FAIL-CLOSED PROPERTY TESTS\n");
    fb_print("========================================\n");
    fb_print("Testing permanent blocking and state integrity for fail-closed paths.\n");
    fb_print("\n");

    if (test_property_9_fail_closed_permanence() != 0) {
        fb_print("[CRITICAL FAILURE] Property 9 failed\n");
    }
    if (test_property_10_fail_closed_state_integrity() != 0) {
        fb_print("[CRITICAL FAILURE] Property 10 failed\n");
    }

    fb_print("\n");
    fb_print("========================================\n");
    fb_print("TEST SUMMARY\n");
    fb_print("========================================\n");
    fb_print("Total assertions: ");
    fb_print_int(g_fail_closed_test_count);
    fb_print("\n");
    fb_print("Failures: ");
    fb_print_int(g_fail_closed_test_failures);
    fb_print("\n");

    if (g_fail_closed_test_failures == 0) {
        fb_print("[PASS] VCP fail-closed property tests passed\n");
        return 0;
    }

    fb_print("[FAIL] VCP fail-closed property tests failed\n");
    return -1;
}

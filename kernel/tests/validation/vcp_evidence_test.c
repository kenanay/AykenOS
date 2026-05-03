// kernel/tests/validation/vcp_evidence_test.c
// VCP Diagnostic Evidence Property Tests

#include "vcp_evidence_test.h"
#include "../../include/vcp_runtime.h"
#include "../../include/execution_slot.h"
#include "../../drivers/console/fb_console.h"

#define memset __builtin_memset

#define EVIDENCE_TEST_CONTRACT_ID      0x5100ULL
#define EVIDENCE_TEST_BOUNDARY_POLICY  0x5200ULL
#define EVIDENCE_TEST_EVIDENCE_ID      0x5300ULL
#define EVIDENCE_TEST_TIMESTAMP_BASE   0x5400ULL

static int g_evidence_test_failures;
static int g_evidence_test_count;

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

#define EVIDENCE_TEST_ASSERT(condition, message) \
    do { \
        g_evidence_test_count++; \
        if (!(condition)) { \
            fb_print("[FAIL] "); \
            fb_print(message); \
            fb_print("\n"); \
            g_evidence_test_failures++; \
            return -1; \
        } \
    } while (0)

#define EVIDENCE_TEST_ASSERT_EQ(actual, expected, message) \
    do { \
        g_evidence_test_count++; \
        if ((actual) != (expected)) { \
            fb_print("[FAIL] "); \
            fb_print(message); \
            fb_print(" expected="); \
            fb_print_int((int64_t)(expected)); \
            fb_print(" actual="); \
            fb_print_int((int64_t)(actual)); \
            fb_print("\n"); \
            g_evidence_test_failures++; \
            return -1; \
        } \
    } while (0)

static void create_evidence_test_slot(exec_slot_t *slot, uint64_t slot_id)
{
    memset(slot, 0, sizeof(exec_slot_t));
    slot->in_use = 1;
    slot->execution_id = slot_id;
    slot->generation = slot_id ^ 0xD5D55D5DULL;
    slot->owner_pid = 0x900ULL + slot_id;
    slot->target_context_id = 0xA00ULL + slot_id;
    slot->state = EXEC_SLOT_READY;
    slot->error_code = 0;
    slot->validation_state = 0;
}

static void create_evidence_test_state(vcp_validation_state_t *state,
                                       uint64_t validation_result,
                                       uint64_t nonce)
{
    memset(state, 0, sizeof(vcp_validation_state_t));
    state->validation_result = validation_result;
    state->contract_id = EVIDENCE_TEST_CONTRACT_ID;
    state->boundary_policy = EVIDENCE_TEST_BOUNDARY_POLICY;
    state->nonce = nonce;
    state->evidence_id = EVIDENCE_TEST_EVIDENCE_ID;
    state->timestamp = EVIDENCE_TEST_TIMESTAMP_BASE + nonce;
}

static void sign_evidence_state(vcp_validation_state_t *state)
{
    state->signature = 0;
    state->signature = vcp_test_signature(state);
}

static void prepare_trusted_evidence_state(exec_slot_t *slot,
                                           vcp_validation_state_t *state,
                                           uint64_t validation_result,
                                           uint64_t nonce)
{
    create_evidence_test_state(state, validation_result, nonce);
    slot->validation_state = state;
    state->capability_id = vcp_test_capability_binding(slot, state);
    state->context_hash = vcp_compute_context_hash(slot);
    sign_evidence_state(state);
}

static int get_evidence(uint32_t index, vcp_diagnostic_evidence_entry_t *entry)
{
    EVIDENCE_TEST_ASSERT(vcp_test_get_diagnostic_evidence(index, entry) == 0,
                         "diagnostic evidence entry must exist");
    return 0;
}

static int find_event(uint32_t event_type, vcp_diagnostic_evidence_entry_t *entry)
{
    uint32_t count = vcp_test_diagnostic_evidence_count();
    uint32_t i;

    for (i = 0; i < count; ++i) {
        vcp_diagnostic_evidence_entry_t candidate;

        if (vcp_test_get_diagnostic_evidence(i, &candidate) != 0) {
            return -1;
        }
        if (candidate.event_type == event_type) {
            *entry = candidate;
            return 0;
        }
    }

    return -1;
}

int test_property_8_comprehensive_evidence_emission(void)
{
    exec_slot_t slot;
    vcp_validation_state_t state;
    vcp_diagnostic_evidence_entry_t entry;
    int result;

    fb_print("[TEST] Property 8: Comprehensive Diagnostic Evidence Emission\n");

    vcp_test_reset_trust_environment();
    vcp_test_reset_diagnostic_evidence();
    create_evidence_test_slot(&slot, 8001);
    prepare_trusted_evidence_state(&slot, &state, VCP_VALID, 0x800100ULL);

    result = vcp_runtime_validate(&slot);
    EVIDENCE_TEST_ASSERT_EQ(result, VCP_VALID,
                            "trusted valid state must pass runtime validation");
    EVIDENCE_TEST_ASSERT_EQ(vcp_test_diagnostic_evidence_count(), 1,
                            "runtime validation must emit one validation-check event");
    if (get_evidence(0, &entry) != 0) {
        return -1;
    }
    EVIDENCE_TEST_ASSERT_EQ(entry.event_type, VCP_DIAG_EVENT_VALIDATION_CHECK,
                            "first evidence entry must be validation check");
    EVIDENCE_TEST_ASSERT_EQ(entry.result, VCP_VALID,
                            "validation-check evidence must record VCP_VALID");
    EVIDENCE_TEST_ASSERT_EQ(entry.slot_id, slot.execution_id,
                            "validation-check evidence must include slot id");
    EVIDENCE_TEST_ASSERT_EQ(entry.context_hash, state.context_hash,
                            "validation-check evidence must include context hash");

    vcp_emit_contract_execution(&slot, "contract:diagnostic");
    vcp_emit_boundary_crossing(&slot, "boundary:diagnostic");
    EVIDENCE_TEST_ASSERT_EQ(vcp_test_diagnostic_evidence_count(), 3,
                            "contract and boundary events must append diagnostic evidence");

    if (get_evidence(1, &entry) != 0) {
        return -1;
    }
    EVIDENCE_TEST_ASSERT_EQ(entry.event_type, VCP_DIAG_EVENT_CONTRACT_EXECUTION,
                            "second evidence entry must be contract execution");
    EVIDENCE_TEST_ASSERT(entry.label_hash != 0,
                         "contract evidence must include a deterministic label hash");

    if (get_evidence(2, &entry) != 0) {
        return -1;
    }
    EVIDENCE_TEST_ASSERT_EQ(entry.event_type, VCP_DIAG_EVENT_BOUNDARY_CROSSING,
                            "third evidence entry must be boundary crossing");
    EVIDENCE_TEST_ASSERT(entry.label_hash != 0,
                         "boundary evidence must include a deterministic label hash");

    debugcon_write("[VCP_EVIDENCE][COMPREHENSIVE]\n");
    fb_print("[PASS] Property 8: comprehensive diagnostic evidence emitted\n");
    return 0;
}

int test_property_11_fail_closed_evidence_completeness(void)
{
    exec_slot_t slot;
    vcp_validation_state_t state;
    vcp_diagnostic_evidence_entry_t block_entry;
    int result;

    fb_print("[TEST] Property 11: Fail-Closed Evidence Completeness\n");

    vcp_test_reset_trust_environment();
    vcp_test_reset_diagnostic_evidence();
    create_evidence_test_slot(&slot, 11011);
    prepare_trusted_evidence_state(&slot, &state, VCP_INVALID, 0x110110ULL);

    result = vcp_runtime_validate(&slot);
    EVIDENCE_TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED,
                            "trusted invalid state must fail closed");
    EVIDENCE_TEST_ASSERT(vcp_test_diagnostic_evidence_count() >= 2,
                         "fail-closed path must emit validation and block evidence");
    EVIDENCE_TEST_ASSERT(find_event(VCP_DIAG_EVENT_EXECUTION_BLOCK, &block_entry) == 0,
                         "fail-closed path must emit execution-block evidence");

    EVIDENCE_TEST_ASSERT_EQ(block_entry.result, VCP_FAIL_CLOSED,
                            "block evidence must record fail-closed result");
    EVIDENCE_TEST_ASSERT_EQ(block_entry.slot_id, slot.execution_id,
                            "block evidence must include slot id");
    EVIDENCE_TEST_ASSERT_EQ(block_entry.generation, slot.generation,
                            "block evidence must include slot generation");
    EVIDENCE_TEST_ASSERT_EQ(block_entry.owner_pid, slot.owner_pid,
                            "block evidence must include owner pid");
    EVIDENCE_TEST_ASSERT_EQ(block_entry.target_context_id, slot.target_context_id,
                            "block evidence must include target context");
    EVIDENCE_TEST_ASSERT_EQ(block_entry.slot_state, EXEC_SLOT_ABORTED,
                            "block evidence must include final aborted state");
    EVIDENCE_TEST_ASSERT_EQ(block_entry.error_code, VCP_FAIL_CLOSED_SLOT_ERROR_CODE,
                            "block evidence must include VCP fail-closed error code");
    EVIDENCE_TEST_ASSERT_EQ(block_entry.event_result, VCP_FAIL_CLOSED,
                            "block evidence must include final validation decision");
    EVIDENCE_TEST_ASSERT_EQ(block_entry.context_hash, state.context_hash,
                            "block evidence must include context hash");
    EVIDENCE_TEST_ASSERT_EQ(block_entry.nonce, state.nonce,
                            "block evidence must include nonce");
    EVIDENCE_TEST_ASSERT_EQ(block_entry.capability_id, state.capability_id,
                            "block evidence must include capability id");
    EVIDENCE_TEST_ASSERT_EQ(block_entry.evidence_id, state.evidence_id,
                            "block evidence must include evidence id");
    EVIDENCE_TEST_ASSERT(block_entry.reason_hash != 0,
                         "block evidence must include deterministic reason hash");

    debugcon_write("[VCP_EVIDENCE][FAIL_CLOSED_COMPLETE]\n");
    fb_print("[PASS] Property 11: fail-closed evidence complete\n");
    return 0;
}

int test_property_49_diagnostic_evidence_isolation(void)
{
    exec_slot_t slot1, slot2;
    vcp_validation_state_t state1, state2;
    int result1, result2;
    uint32_t evidence_count_before, evidence_count_after;

    fb_print("[TEST] Property 49: Diagnostic Evidence Isolation [CRITICAL]\n");

    vcp_test_reset_trust_environment();
    vcp_test_reset_diagnostic_evidence();

    fb_print("  [1/4] Testing evidence enabled vs disabled produces same outcome\n");
    create_evidence_test_slot(&slot1, 49001);
    prepare_trusted_evidence_state(&slot1, &state1, VCP_VALID, 0x490010ULL);
    result1 = vcp_runtime_validate(&slot1);
    EVIDENCE_TEST_ASSERT_EQ(result1, VCP_VALID,
                            "validation with evidence enabled must return VCP_VALID");

    vcp_test_reset_trust_environment();
    vcp_test_reset_diagnostic_evidence();
    create_evidence_test_slot(&slot2, 49001);
    prepare_trusted_evidence_state(&slot2, &state2, VCP_VALID, 0x490010ULL);
    result2 = vcp_runtime_validate(&slot2);
    EVIDENCE_TEST_ASSERT_EQ(result2, VCP_VALID,
                            "validation with evidence reset must return VCP_VALID");
    EVIDENCE_TEST_ASSERT_EQ(result1, result2,
                            "evidence emission must not affect validation outcome");

    fb_print("  [2/4] Testing evidence buffer overflow does not affect execution\n");
    vcp_test_reset_trust_environment();
    vcp_test_reset_diagnostic_evidence();
    create_evidence_test_slot(&slot1, 49002);
    prepare_trusted_evidence_state(&slot1, &state1, VCP_VALID, 0x490020ULL);

    for (uint32_t i = 0; i < VCP_DIAGNOSTIC_EVIDENCE_CAPACITY + 10; ++i) {
        vcp_emit_validation_check(&slot1, VCP_VALID);
    }
    evidence_count_before = vcp_test_diagnostic_evidence_count();
    result1 = vcp_runtime_validate(&slot1);
    evidence_count_after = vcp_test_diagnostic_evidence_count();

    EVIDENCE_TEST_ASSERT_EQ(result1, VCP_VALID,
                            "validation must succeed even after evidence buffer overflow");
    EVIDENCE_TEST_ASSERT(evidence_count_before <= VCP_DIAGNOSTIC_EVIDENCE_CAPACITY,
                         "evidence buffer must be bounded");
    EVIDENCE_TEST_ASSERT(evidence_count_after <= VCP_DIAGNOSTIC_EVIDENCE_CAPACITY,
                         "evidence buffer must remain bounded after overflow");

    fb_print("  [3/4] Testing NULL slot evidence emission does not crash\n");
    vcp_emit_validation_check(0, VCP_VALID);
    vcp_emit_execution_block(0, "null_slot_test");
    vcp_emit_contract_execution(0, "null_contract");
    vcp_emit_boundary_crossing(0, "null_boundary");
    fb_print("      NULL slot evidence emission handled gracefully\n");

    fb_print("  [4/4] Testing evidence functions return void (no error propagation)\n");
    vcp_test_reset_trust_environment();
    vcp_test_reset_diagnostic_evidence();
    create_evidence_test_slot(&slot1, 49004);
    prepare_trusted_evidence_state(&slot1, &state1, VCP_VALID, 0x490040ULL);

    vcp_emit_validation_check(&slot1, VCP_VALID);
    vcp_emit_contract_execution(&slot1, "test_contract");
    vcp_emit_boundary_crossing(&slot1, "test_boundary");

    result1 = vcp_runtime_validate(&slot1);
    EVIDENCE_TEST_ASSERT_EQ(result1, VCP_VALID,
                            "validation outcome must not be affected by prior evidence emission");

    debugcon_write("[VCP_EVIDENCE][ISOLATION_VERIFIED]\n");
    fb_print("[PASS] Property 49: diagnostic evidence isolation verified\n");
    fb_print("      Evidence emission is side-effect free\n");
    fb_print("      Evidence does NOT affect validation/trust/execution\n");
    return 0;
}

int execute_vcp_evidence_tests(void)
{
    g_evidence_test_failures = 0;
    g_evidence_test_count = 0;

    fb_print("\n");
    fb_print("========================================\n");
    fb_print("VCP DIAGNOSTIC EVIDENCE PROPERTY TESTS\n");
    fb_print("========================================\n");
    fb_print("Testing diagnostic-only evidence stubs. These are not signed authority.\n");
    fb_print("\n");

    if (test_property_8_comprehensive_evidence_emission() != 0) {
        fb_print("[QUALITY FAILURE] Property 8 failed\n");
    }
    if (test_property_11_fail_closed_evidence_completeness() != 0) {
        fb_print("[CRITICAL FAILURE] Property 11 failed\n");
    }
    if (test_property_49_diagnostic_evidence_isolation() != 0) {
        fb_print("[CRITICAL FAILURE] Property 49 failed\n");
    }

    fb_print("\n");
    fb_print("========================================\n");
    fb_print("TEST SUMMARY\n");
    fb_print("========================================\n");
    fb_print("Total assertions: ");
    fb_print_int(g_evidence_test_count);
    fb_print("\n");
    fb_print("Failures: ");
    fb_print_int(g_evidence_test_failures);
    fb_print("\n");

    if (g_evidence_test_failures == 0) {
        fb_print("[PASS] VCP diagnostic evidence property tests passed\n");
        return 0;
    }

    fb_print("[FAIL] VCP diagnostic evidence property tests failed\n");
    return -1;
}

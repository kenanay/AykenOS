// kernel/tests/validation/vcp_trust_verification_test.c
// VCP Trust Verification Property Tests

#include "vcp_trust_verification_test.h"
#include "../../include/vcp_runtime.h"
#include "../../include/execution_slot.h"
#include "../../drivers/console/fb_console.h"

#define memset __builtin_memset

#define TEST_CONTRACT_ID      0x1000ULL
#define TEST_BOUNDARY_POLICY  0x2000ULL
#define TEST_EVIDENCE_ID      0x3000ULL
#define TEST_TIMESTAMP_BASE   0x4000ULL

static int g_test_failures;
static int g_test_count;

#define TEST_ASSERT(condition, message) \
    do { \
        g_test_count++; \
        if (!(condition)) { \
            fb_print("[FAIL] "); \
            fb_print(message); \
            fb_print("\n"); \
            g_test_failures++; \
            return -1; \
        } \
    } while (0)

#define TEST_ASSERT_EQ(actual, expected, message) \
    do { \
        g_test_count++; \
        if ((actual) != (expected)) { \
            fb_print("[FAIL] "); \
            fb_print(message); \
            fb_print(" expected="); \
            fb_print_int((int64_t)(expected)); \
            fb_print(" actual="); \
            fb_print_int((int64_t)(actual)); \
            fb_print("\n"); \
            g_test_failures++; \
            return -1; \
        } \
    } while (0)

static void create_test_slot(exec_slot_t *slot, uint64_t slot_id)
{
    memset(slot, 0, sizeof(exec_slot_t));
    slot->in_use = 1;
    slot->execution_id = slot_id;
    slot->generation = slot_id ^ 0x55AA55AAULL;
    slot->owner_pid = 0x100ULL + slot_id;
    slot->target_context_id = 0x200ULL + slot_id;
    slot->validation_state = 0;
}

static void create_test_validation_state(vcp_validation_state_t *state,
                                         uint64_t validation_result,
                                         uint64_t nonce)
{
    memset(state, 0, sizeof(vcp_validation_state_t));
    state->validation_result = validation_result;
    state->contract_id = TEST_CONTRACT_ID;
    state->boundary_policy = TEST_BOUNDARY_POLICY;
    state->nonce = nonce;
    state->evidence_id = TEST_EVIDENCE_ID;
    state->timestamp = TEST_TIMESTAMP_BASE + nonce;
}

static void sign_state(vcp_validation_state_t *state)
{
    state->signature = 0;
    state->signature = vcp_test_signature(state);
}

static void prepare_trusted_state(exec_slot_t *slot,
                                  vcp_validation_state_t *state,
                                  uint64_t slot_id,
                                  uint64_t validation_result,
                                  uint64_t nonce)
{
    create_test_slot(slot, slot_id);
    create_test_validation_state(state, validation_result, nonce);
    slot->validation_state = state;
    state->capability_id = vcp_test_capability_binding(slot, state);
    state->context_hash = vcp_compute_context_hash(slot);
    sign_state(state);
}

static int trace_index(uint32_t event)
{
    vcp_trust_trace_t trace;
    uint32_t i;

    vcp_test_get_trust_trace(&trace);
    for (i = 0; i < trace.count; ++i) {
        if (trace.events[i] == event) {
            return (int)i;
        }
    }

    return -1;
}

static int trace_contains(uint32_t event)
{
    return trace_index(event) >= 0;
}

static int assert_trace_exact(const uint32_t *expected,
                              uint32_t expected_count,
                              const char *message)
{
    vcp_trust_trace_t trace;
    uint32_t i;

    vcp_test_get_trust_trace(&trace);
    TEST_ASSERT_EQ(trace.count, expected_count, message);

    for (i = 0; i < expected_count; ++i) {
        TEST_ASSERT_EQ(trace.events[i], expected[i], "trust trace event mismatch");
    }

    return 0;
}

static int assert_no_result_check(const char *message)
{
    TEST_ASSERT(!trace_contains(VCP_TRACE_RESULT), message);
    return 0;
}

static int assert_valid_trace(void)
{
    static const uint32_t expected[] = {
        VCP_TRACE_CAPABILITY,
        VCP_TRACE_CONTEXT,
        VCP_TRACE_SIGNATURE,
        VCP_TRACE_NONCE,
        VCP_TRACE_RESULT,
        VCP_TRACE_NONCE_COMMIT,
    };

    return assert_trace_exact(expected,
                              (uint32_t)(sizeof(expected) / sizeof(expected[0])),
                              "trusted state must run full trust trace");
}

static int assert_fail_trace_capability(void)
{
    static const uint32_t expected[] = {
        VCP_TRACE_CAPABILITY,
        VCP_TRACE_FAIL_CLOSED,
    };

    return assert_trace_exact(expected,
                              (uint32_t)(sizeof(expected) / sizeof(expected[0])),
                              "capability failure trace must be deterministic");
}

static int assert_fail_trace_context(void)
{
    static const uint32_t expected[] = {
        VCP_TRACE_CAPABILITY,
        VCP_TRACE_CONTEXT,
        VCP_TRACE_FAIL_CLOSED,
    };

    return assert_trace_exact(expected,
                              (uint32_t)(sizeof(expected) / sizeof(expected[0])),
                              "context failure trace must be deterministic");
}

static int assert_fail_trace_signature(void)
{
    static const uint32_t expected[] = {
        VCP_TRACE_CAPABILITY,
        VCP_TRACE_CONTEXT,
        VCP_TRACE_SIGNATURE,
        VCP_TRACE_FAIL_CLOSED,
    };

    return assert_trace_exact(expected,
                              (uint32_t)(sizeof(expected) / sizeof(expected[0])),
                              "signature failure trace must be deterministic");
}

static int assert_fail_trace_nonce(void)
{
    static const uint32_t expected[] = {
        VCP_TRACE_CAPABILITY,
        VCP_TRACE_CONTEXT,
        VCP_TRACE_SIGNATURE,
        VCP_TRACE_NONCE,
        VCP_TRACE_FAIL_CLOSED,
    };

    return assert_trace_exact(expected,
                              (uint32_t)(sizeof(expected) / sizeof(expected[0])),
                              "nonce failure trace must be deterministic");
}

static int assert_fail_trace_result(void)
{
    static const uint32_t expected[] = {
        VCP_TRACE_CAPABILITY,
        VCP_TRACE_CONTEXT,
        VCP_TRACE_SIGNATURE,
        VCP_TRACE_NONCE,
        VCP_TRACE_RESULT,
        VCP_TRACE_FAIL_CLOSED,
    };

    return assert_trace_exact(expected,
                              (uint32_t)(sizeof(expected) / sizeof(expected[0])),
                              "validation_result failure trace must be deterministic");
}

int test_property_25_fake_state_rejection(void)
{
    exec_slot_t slot;
    vcp_validation_state_t state;
    int result;

    fb_print("[TEST] Property 25: Fake Validation State Rejection\n");

    vcp_test_reset_trust_environment();
    create_test_slot(&slot, 2501);
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "NULL validation_state must fail closed");
    {
        static const uint32_t expected[] = { VCP_TRACE_FAIL_CLOSED };
        if (assert_trace_exact(expected, 1, "NULL state trace must fail closed") != 0) {
            return -1;
        }
    }

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot, &state, 2502, VCP_VALID, 0x2502ULL);
    state.capability_id = 0;
    sign_state(&state);
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "capability_id=0 must fail closed");
    if (assert_fail_trace_capability() != 0) {
        return -1;
    }

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot, &state, 2503, VCP_VALID, 0x2503ULL);
    state.capability_id ^= 0x5A5A5A5AULL;
    sign_state(&state);
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "forged capability binding must fail closed");
    if (assert_fail_trace_capability() != 0) {
        return -1;
    }

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot, &state, 2504, VCP_VALID, 0x2504ULL);
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_VALID, "valid capability binding must be accepted");
    if (assert_valid_trace() != 0) {
        return -1;
    }

    fb_print("[PASS] Property 25: fake state rejection validated\n");
    return 0;
}

int test_property_26_replayed_state_rejection(void)
{
    exec_slot_t slot;
    vcp_validation_state_t state;
    int result;

    fb_print("[TEST] Property 26: Replayed Validation State Rejection\n");

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot, &state, 2601, VCP_VALID, 0x2601ULL);
    state.context_hash ^= 0x123456789ABCDEF0ULL;
    sign_state(&state);
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "context hash mismatch must fail closed");
    TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 0, "context failure must not commit nonce");
    if (assert_fail_trace_context() != 0) {
        return -1;
    }

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot, &state, 2602, VCP_VALID, 0x2602ULL);
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_VALID, "first use of nonce must pass");
    TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 1, "accepted state must commit nonce");

    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "replayed nonce must fail closed");
    TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 1, "replay failure must not append nonce");
    if (assert_fail_trace_nonce() != 0) {
        return -1;
    }

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot, &state, 2603, VCP_VALID, 0x2603ULL);
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_VALID, "fresh context and nonce must be accepted");
    if (assert_valid_trace() != 0) {
        return -1;
    }

    fb_print("[PASS] Property 26: replayed state rejection validated\n");
    return 0;
}

int test_property_27_signature_verification(void)
{
    exec_slot_t slot;
    vcp_validation_state_t state;
    int result;

    fb_print("[TEST] Property 27: Signature Verification Enforcement\n");

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot, &state, 2701, VCP_VALID, 0x2701ULL);
    state.signature = 0;
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "missing signature must fail closed");
    TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 0, "signature failure must not commit nonce");
    if (assert_fail_trace_signature() != 0) {
        return -1;
    }

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot, &state, 2702, VCP_VALID, 0x2702ULL);
    state.signature ^= 0x00F00F00F00F00F0ULL;
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "modified signature must fail closed");
    TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 0, "invalid signature must not commit nonce");
    if (assert_fail_trace_signature() != 0) {
        return -1;
    }

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot, &state, 2703, VCP_VALID, 0x2703ULL);
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_VALID, "valid deterministic signature must pass");
    if (assert_valid_trace() != 0) {
        return -1;
    }

    fb_print("[PASS] Property 27: signature verification validated\n");
    return 0;
}

int test_property_28_trust_verification_before_enforcement(void)
{
    exec_slot_t slot;
    vcp_validation_state_t state;
    int result;
    int nonce_index;
    int result_index;

    fb_print("[TEST] Property 28: Trust Verification Before Enforcement\n");

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot, &state, 2801, VCP_VALID, 0x2801ULL);
    state.capability_id ^= 0x11111111ULL;
    sign_state(&state);
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "VCP_VALID with forged capability must fail");
    if (assert_no_result_check("result must not be checked after capability failure") != 0) {
        return -1;
    }

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot, &state, 2802, VCP_VALID, 0x2802ULL);
    state.context_hash ^= 0x22222222ULL;
    sign_state(&state);
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "VCP_VALID with context mismatch must fail");
    if (assert_no_result_check("result must not be checked after context failure") != 0) {
        return -1;
    }

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot, &state, 2803, VCP_VALID, 0x2803ULL);
    state.signature ^= 0x33333333ULL;
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "VCP_VALID with invalid signature must fail");
    if (assert_no_result_check("result must not be checked after signature failure") != 0) {
        return -1;
    }

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot, &state, 2804, VCP_VALID, 0x2804ULL);
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_VALID, "first nonce use must pass before replay test");
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "VCP_VALID with replayed nonce must fail");
    if (assert_no_result_check("result must not be checked after nonce failure") != 0) {
        return -1;
    }

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot, &state, 2805, VCP_INVALID, 0x2805ULL);
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "trusted VCP_INVALID result must fail closed");
    nonce_index = trace_index(VCP_TRACE_NONCE);
    result_index = trace_index(VCP_TRACE_RESULT);
    TEST_ASSERT(nonce_index >= 0, "nonce must be checked before result");
    TEST_ASSERT(result_index > nonce_index, "validation_result must be checked after trust");
    TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 0, "invalid result must not commit nonce");
    if (assert_fail_trace_result() != 0) {
        return -1;
    }

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot, &state, 2806, VCP_VALID, 0x2806ULL);
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_VALID, "fully trusted VCP_VALID result must pass");
    if (assert_valid_trace() != 0) {
        return -1;
    }

    fb_print("[PASS] Property 28: trust-before-result enforcement validated\n");
    return 0;
}

int test_property_24_validation_state_trust_verification(void)
{
    exec_slot_t slot;
    vcp_validation_state_t state;
    int result;

    fb_print("[TEST] Property 24: Validation State Trust Verification\n");

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot, &state, 2401, VCP_VALID, 0x2401ULL);
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_VALID, "complete trusted state must pass all checks");
    if (assert_valid_trace() != 0) {
        return -1;
    }
    TEST_ASSERT(trace_contains(VCP_TRACE_CAPABILITY), "capability check must be performed");
    TEST_ASSERT(trace_contains(VCP_TRACE_CONTEXT), "context check must be performed");
    TEST_ASSERT(trace_contains(VCP_TRACE_SIGNATURE), "signature check must be performed");
    TEST_ASSERT(trace_contains(VCP_TRACE_NONCE), "nonce check must be performed");
    TEST_ASSERT(trace_contains(VCP_TRACE_RESULT), "result check must be performed after trust");

    vcp_test_reset_trust_environment();
    create_test_slot(&slot, 2402);
    create_test_validation_state(&state, VCP_VALID, 0);
    state.context_hash = 0xBADBADULL;
    state.signature = 0;
    state.capability_id = 0;
    slot.validation_state = &state;
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "all-fake trust state must fail closed");
    if (assert_fail_trace_capability() != 0) {
        return -1;
    }
    TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 0, "fake state must not mutate nonce ledger");

    fb_print("[PASS] Property 24: complete trust verification validated\n");
    return 0;
}

int test_property_29_trust_check_order_is_deterministic(void)
{
    exec_slot_t slot;
    vcp_validation_state_t state;
    int result;

    fb_print("[TEST] Property 29: Trust Check Order Is Deterministic\n");

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot, &state, 2901, VCP_VALID, 0x2901ULL);
    state.capability_id ^= 0x29ULL;
    sign_state(&state);
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "first capability failure must fail closed");
    if (assert_fail_trace_capability() != 0) {
        return -1;
    }

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot, &state, 2901, VCP_VALID, 0x2901ULL);
    state.capability_id ^= 0x29ULL;
    sign_state(&state);
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "repeated capability failure must fail closed");
    if (assert_fail_trace_capability() != 0) {
        return -1;
    }

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot, &state, 2902, VCP_VALID, 0x2902ULL);
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_VALID, "valid trust state must pass deterministic order");
    if (assert_valid_trace() != 0) {
        return -1;
    }

    fb_print("[PASS] Property 29: deterministic trust-check order validated\n");
    return 0;
}

int test_property_30_validation_result_never_checked_before_trust(void)
{
    exec_slot_t slot;
    vcp_validation_state_t state;
    int result;
    int capability_index;
    int context_index;
    int signature_index;
    int nonce_index;
    int result_index;

    fb_print("[TEST] Property 30: Validation Result Never Checked Before Trust\n");

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot, &state, 3001, VCP_VALID, 0x3001ULL);
    state.capability_id ^= 0x3001ULL;
    sign_state(&state);
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "forged state with VCP_VALID must fail closed");
    if (assert_no_result_check("VCP_VALID flag must not bypass failed trust") != 0) {
        return -1;
    }

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot, &state, 3002, VCP_INVALID, 0x3002ULL);
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "trusted VCP_INVALID must fail closed after trust");

    capability_index = trace_index(VCP_TRACE_CAPABILITY);
    context_index = trace_index(VCP_TRACE_CONTEXT);
    signature_index = trace_index(VCP_TRACE_SIGNATURE);
    nonce_index = trace_index(VCP_TRACE_NONCE);
    result_index = trace_index(VCP_TRACE_RESULT);
    TEST_ASSERT(capability_index >= 0, "capability must be checked");
    TEST_ASSERT(context_index > capability_index, "context must follow capability");
    TEST_ASSERT(signature_index > context_index, "signature must follow context");
    TEST_ASSERT(nonce_index > signature_index, "nonce must follow signature");
    TEST_ASSERT(result_index > nonce_index, "result must follow nonce");

    fb_print("[PASS] Property 30: result-after-trust order validated\n");
    return 0;
}

int test_property_31_fail_closed_has_no_partial_state_mutation(void)
{
    exec_slot_t slot;
    vcp_validation_state_t state;
    int result;

    fb_print("[TEST] Property 31: Fail-Closed Has No Partial State Mutation\n");

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot, &state, 3101, VCP_VALID, 0x3101ULL);
    state.capability_id ^= 0x3101ULL;
    sign_state(&state);
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "capability failure must fail closed");
    TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 0, "capability failure must not append nonce");

    prepare_trusted_state(&slot, &state, 3102, VCP_VALID, 0x3102ULL);
    state.context_hash ^= 0x3102ULL;
    sign_state(&state);
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "context failure must fail closed");
    TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 0, "context failure must not append nonce");

    prepare_trusted_state(&slot, &state, 3103, VCP_VALID, 0x3103ULL);
    state.signature ^= 0x3103ULL;
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "signature failure must fail closed");
    TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 0, "signature failure must not append nonce");

    prepare_trusted_state(&slot, &state, 3104, VCP_INVALID, 0x3104ULL);
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "invalid result must fail closed");
    TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 0, "invalid result must not append nonce");

    prepare_trusted_state(&slot, &state, 3105, VCP_VALID, 0x3105ULL);
    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_VALID, "valid state must append exactly one nonce");
    TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 1, "valid state must append nonce");

    result = vcp_verify_validation_state(&slot);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "replay must fail closed");
    TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 1, "replay failure must not append duplicate nonce");

    fb_print("[PASS] Property 31: fail-closed mutation boundary validated\n");
    return 0;
}

int test_property_32_replay_ledger_rejects_seen_nonce(void)
{
    exec_slot_t slot_a;
    exec_slot_t slot_b;
    vcp_validation_state_t state_a;
    vcp_validation_state_t state_b;
    int result;

    fb_print("[TEST] Property 32: Replay Ledger Rejects Seen Nonce\n");

    vcp_test_reset_trust_environment();
    prepare_trusted_state(&slot_a, &state_a, 3201, VCP_VALID, 0x3200ULL);
    result = vcp_verify_validation_state(&slot_a);
    TEST_ASSERT_EQ(result, VCP_VALID, "first nonce owner must pass");
    TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 1, "first nonce must be recorded");

    prepare_trusted_state(&slot_b, &state_b, 3202, VCP_VALID, 0x3200ULL);
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot_b);
    TEST_ASSERT_EQ(result, VCP_FAIL_CLOSED, "same nonce in a different valid context must fail");
    TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 1, "seen nonce rejection must not grow ledger");
    if (assert_fail_trace_nonce() != 0) {
        return -1;
    }

    prepare_trusted_state(&slot_b, &state_b, 3203, VCP_VALID, 0x3203ULL);
    vcp_test_reset_trust_trace();
    result = vcp_verify_validation_state(&slot_b);
    TEST_ASSERT_EQ(result, VCP_VALID, "fresh nonce after replay rejection must pass");
    TEST_ASSERT_EQ(vcp_test_nonce_ledger_count(), 2, "fresh nonce must be appended");
    if (assert_valid_trace() != 0) {
        return -1;
    }

    fb_print("[PASS] Property 32: replay ledger rejection validated\n");
    return 0;
}

int execute_vcp_trust_verification_tests(void)
{
    g_test_failures = 0;
    g_test_count = 0;

    fb_print("\n");
    fb_print("========================================\n");
    fb_print("VCP TRUST VERIFICATION PROPERTY TESTS\n");
    fb_print("========================================\n");
    fb_print("Testing deterministic trust checks for capability, context, signature, and nonce.\n");
    fb_print("\n");

    if (test_property_25_fake_state_rejection() != 0) {
        fb_print("[CRITICAL FAILURE] Property 25 failed\n");
    }
    if (test_property_26_replayed_state_rejection() != 0) {
        fb_print("[CRITICAL FAILURE] Property 26 failed\n");
    }
    if (test_property_27_signature_verification() != 0) {
        fb_print("[CRITICAL FAILURE] Property 27 failed\n");
    }
    if (test_property_28_trust_verification_before_enforcement() != 0) {
        fb_print("[CRITICAL FAILURE] Property 28 failed\n");
    }
    if (test_property_24_validation_state_trust_verification() != 0) {
        fb_print("[CRITICAL FAILURE] Property 24 failed\n");
    }
    if (test_property_29_trust_check_order_is_deterministic() != 0) {
        fb_print("[CRITICAL FAILURE] Property 29 failed\n");
    }
    if (test_property_30_validation_result_never_checked_before_trust() != 0) {
        fb_print("[CRITICAL FAILURE] Property 30 failed\n");
    }
    if (test_property_31_fail_closed_has_no_partial_state_mutation() != 0) {
        fb_print("[CRITICAL FAILURE] Property 31 failed\n");
    }
    if (test_property_32_replay_ledger_rejects_seen_nonce() != 0) {
        fb_print("[CRITICAL FAILURE] Property 32 failed\n");
    }

    fb_print("\n");
    fb_print("========================================\n");
    fb_print("TEST SUMMARY\n");
    fb_print("========================================\n");
    fb_print("Total assertions: ");
    fb_print_int(g_test_count);
    fb_print("\n");
    fb_print("Failures: ");
    fb_print_int(g_test_failures);
    fb_print("\n");

    if (g_test_failures == 0) {
        fb_print("[PASS] VCP trust verification property tests passed\n");
        return 0;
    }

    fb_print("[FAIL] VCP trust verification property tests failed\n");
    return -1;
}

// kernel/sys/vcp_runtime.c
// VCP Runtime Validation Enforcement

#include "../include/vcp_runtime.h"
#include "../include/execution_slot.h"
#include "../include/sha256.h"

#define memset __builtin_memset

#define VCP_NONCE_LEDGER_CAPACITY 128u

static uint64_t g_vcp_nonce_ledger[VCP_NONCE_LEDGER_CAPACITY];
static uint32_t g_vcp_nonce_ledger_count;

#if AYKEN_VCP_TEST_HOOKS
static vcp_trust_trace_t g_vcp_trust_trace;

static void vcp_trace_event(uint32_t event)
{
    if (g_vcp_trust_trace.count < VCP_TRUST_TRACE_CAPACITY) {
        g_vcp_trust_trace.events[g_vcp_trust_trace.count++] = event;
    }
    g_vcp_trust_trace.nonce_ledger_count = g_vcp_nonce_ledger_count;
}
#else
#define vcp_trace_event(event) ((void)(event))
#endif

static void vcp_hash_u64(ayken_sha256_ctx_t *ctx, uint64_t value)
{
    ayken_sha256_update(ctx, &value, sizeof(value));
}

static uint64_t vcp_digest_u64(const uint8_t digest[AYKEN_SHA256_DIGEST_SIZE])
{
    uint64_t value;

    value = ((uint64_t)digest[0] << 0) |
            ((uint64_t)digest[1] << 8) |
            ((uint64_t)digest[2] << 16) |
            ((uint64_t)digest[3] << 24) |
            ((uint64_t)digest[4] << 32) |
            ((uint64_t)digest[5] << 40) |
            ((uint64_t)digest[6] << 48) |
            ((uint64_t)digest[7] << 56);

    if (value == 0) {
        value = 0xA7C5000000000001ULL;
    }

    return value;
}

static uint64_t vcp_compute_capability_binding_internal(struct exec_slot *slot,
                                                        vcp_validation_state_t *state)
{
    static const char domain[] = "AYKEN:VCP:CAPABILITY_BINDING:v1";
    ayken_sha256_ctx_t hash_ctx;
    uint8_t digest[AYKEN_SHA256_DIGEST_SIZE];

    if (!slot || !state) {
        return 0;
    }

    ayken_sha256_init(&hash_ctx);
    ayken_sha256_update(&hash_ctx, domain, sizeof(domain) - 1u);
    vcp_hash_u64(&hash_ctx, slot->execution_id);
    vcp_hash_u64(&hash_ctx, slot->generation);
    vcp_hash_u64(&hash_ctx, slot->owner_pid);
    vcp_hash_u64(&hash_ctx, slot->target_context_id);
    vcp_hash_u64(&hash_ctx, state->contract_id);
    vcp_hash_u64(&hash_ctx, state->boundary_policy);
    ayken_sha256_final(&hash_ctx, digest);

    return vcp_digest_u64(digest);
}

static uint64_t vcp_compute_validation_signature_internal(vcp_validation_state_t *state)
{
    static const char domain[] = "AYKEN:VCP:VALIDATION_SIGNATURE:v1";
    ayken_sha256_ctx_t hash_ctx;
    uint8_t digest[AYKEN_SHA256_DIGEST_SIZE];

    if (!state) {
        return 0;
    }

    ayken_sha256_init(&hash_ctx);
    ayken_sha256_update(&hash_ctx, domain, sizeof(domain) - 1u);
    vcp_hash_u64(&hash_ctx, state->validation_result);
    vcp_hash_u64(&hash_ctx, state->contract_id);
    vcp_hash_u64(&hash_ctx, state->boundary_policy);
    vcp_hash_u64(&hash_ctx, state->context_hash);
    vcp_hash_u64(&hash_ctx, state->nonce);
    vcp_hash_u64(&hash_ctx, state->capability_id);
    vcp_hash_u64(&hash_ctx, state->evidence_id);
    vcp_hash_u64(&hash_ctx, state->timestamp);
    ayken_sha256_final(&hash_ctx, digest);

    return vcp_digest_u64(digest);
}

static int vcp_nonce_seen(uint64_t nonce)
{
    uint32_t i;

    for (i = 0; i < g_vcp_nonce_ledger_count; ++i) {
        if (g_vcp_nonce_ledger[i] == nonce) {
            return 1;
        }
    }

    return 0;
}

static int vcp_commit_nonce(vcp_validation_state_t *state)
{
    if (!state || state->nonce == 0) {
        return -1;
    }

    if (vcp_nonce_seen(state->nonce)) {
        return -1;
    }

    if (g_vcp_nonce_ledger_count >= VCP_NONCE_LEDGER_CAPACITY) {
        return -1;
    }

    g_vcp_nonce_ledger[g_vcp_nonce_ledger_count++] = state->nonce;
    vcp_trace_event(VCP_TRACE_NONCE_COMMIT);
    return 0;
}

/*
 * vcp_compute_context_hash - Compute execution context hash
 *
 * Computes a deterministic hash of the execution context to prevent replay
 * across slots, generations, owners, target contexts, contracts, or boundary
 * policies.
 */
uint64_t vcp_compute_context_hash(struct exec_slot *slot)
{
    static const char domain[] = "AYKEN:VCP:CONTEXT:v1";
    ayken_sha256_ctx_t hash_ctx;
    uint8_t digest[AYKEN_SHA256_DIGEST_SIZE];
    uint64_t contract_id = 0;
    uint64_t boundary_policy = 0;

    if (!slot) {
        return 0;
    }

    if (slot->validation_state) {
        contract_id = slot->validation_state->contract_id;
        boundary_policy = slot->validation_state->boundary_policy;
    }

    ayken_sha256_init(&hash_ctx);
    ayken_sha256_update(&hash_ctx, domain, sizeof(domain) - 1u);
    vcp_hash_u64(&hash_ctx, slot->execution_id);
    vcp_hash_u64(&hash_ctx, slot->generation);
    vcp_hash_u64(&hash_ctx, slot->owner_pid);
    vcp_hash_u64(&hash_ctx, slot->target_context_id);
    vcp_hash_u64(&hash_ctx, contract_id);
    vcp_hash_u64(&hash_ctx, boundary_policy);
    ayken_sha256_final(&hash_ctx, digest);

    return vcp_digest_u64(digest);
}

/*
 * vcp_verify_capability - Verify capability binding
 *
 * The current verifier uses a deterministic capability binding token derived
 * from the slot identity and validation-state context fields. This closes the
 * property-test gap where any non-zero capability_id was accepted as trusted.
 */
int vcp_verify_capability(struct exec_slot *slot, vcp_validation_state_t *state)
{
    uint64_t expected_capability;

    if (!slot || !state || state->capability_id == 0) {
        return -1;
    }

    expected_capability = vcp_compute_capability_binding_internal(slot, state);
    if (state->capability_id != expected_capability) {
        return -1;
    }

    return 0;
}

/*
 * vcp_verify_signature - Verify VCP validation-state signature
 *
 * Production cryptographic trust-root verification is still a later integration
 * concern. This deterministic verifier is intentionally stricter than the old
 * stub: the signature must cover the validation state fields, not merely be
 * non-zero.
 */
int vcp_verify_signature(vcp_validation_state_t *state)
{
    uint64_t expected_signature;

    if (!state || state->signature == 0) {
        return -1;
    }

    expected_signature = vcp_compute_validation_signature_internal(state);
    if (state->signature != expected_signature) {
        return -1;
    }

    return 0;
}

/*
 * vcp_verify_nonce - Verify nonce uniqueness
 *
 * This check is side-effect free. A nonce is appended to the ledger only after
 * every trust check and validation_result check has passed, which prevents
 * fail-closed paths from leaving partial accepted state behind.
 */
int vcp_verify_nonce(vcp_validation_state_t *state)
{
    if (!state || state->nonce == 0) {
        return -1;
    }

    if (vcp_nonce_seen(state->nonce)) {
        return -1;
    }

    return 0;
}

/*
 * vcp_verify_validation_state - Verify validation state trust
 *
 * Verification order is contractual:
 *   1. validation_state present
 *   2. capability binding
 *   3. context hash
 *   4. signature
 *   5. nonce uniqueness
 *   6. validation result
 *   7. nonce commit for accepted states only
 */
int vcp_verify_validation_state(struct exec_slot *slot)
{
    vcp_validation_state_t *state;
    uint64_t computed_context_hash;

    if (!slot || !slot->validation_state) {
        vcp_trace_event(VCP_TRACE_FAIL_CLOSED);
        return VCP_FAIL_CLOSED;
    }

    state = slot->validation_state;

    vcp_trace_event(VCP_TRACE_CAPABILITY);
    if (vcp_verify_capability(slot, state) != 0) {
        vcp_trace_event(VCP_TRACE_FAIL_CLOSED);
        return VCP_FAIL_CLOSED;
    }

    vcp_trace_event(VCP_TRACE_CONTEXT);
    computed_context_hash = vcp_compute_context_hash(slot);
    if (state->context_hash != computed_context_hash) {
        vcp_trace_event(VCP_TRACE_FAIL_CLOSED);
        return VCP_FAIL_CLOSED;
    }

    vcp_trace_event(VCP_TRACE_SIGNATURE);
    if (vcp_verify_signature(state) != 0) {
        vcp_trace_event(VCP_TRACE_FAIL_CLOSED);
        return VCP_FAIL_CLOSED;
    }

    vcp_trace_event(VCP_TRACE_NONCE);
    if (vcp_verify_nonce(state) != 0) {
        vcp_trace_event(VCP_TRACE_FAIL_CLOSED);
        return VCP_FAIL_CLOSED;
    }

    vcp_trace_event(VCP_TRACE_RESULT);
    if (state->validation_result != VCP_VALID) {
        vcp_trace_event(VCP_TRACE_FAIL_CLOSED);
        return VCP_FAIL_CLOSED;
    }

    if (vcp_commit_nonce(state) != 0) {
        vcp_trace_event(VCP_TRACE_FAIL_CLOSED);
        return VCP_FAIL_CLOSED;
    }

    return VCP_VALID;
}

/*
 * vcp_runtime_validate - Runtime validation enforcement hook
 */
int vcp_runtime_validate(struct exec_slot *slot)
{
    int trust_result;

    if (!slot) {
        vcp_emit_validation_check(slot, VCP_FAIL_CLOSED);
        return vcp_fail_closed(slot, "null_slot");
    }

    if (vcp_fail_closed_is_active(slot)) {
        vcp_emit_validation_check(slot, VCP_FAIL_CLOSED);
        return vcp_fail_closed(slot, "already_fail_closed");
    }

    trust_result = vcp_verify_validation_state(slot);
    vcp_emit_validation_check(slot, trust_result);
    if (trust_result != VCP_VALID) {
        return vcp_fail_closed(slot, "validation_failed");
    }

    return VCP_VALID;
}

#if AYKEN_VCP_TEST_HOOKS
void vcp_test_reset_trust_environment(void)
{
    memset(g_vcp_nonce_ledger, 0, sizeof(g_vcp_nonce_ledger));
    g_vcp_nonce_ledger_count = 0;
    vcp_test_reset_trust_trace();
}

void vcp_test_reset_trust_trace(void)
{
    memset(&g_vcp_trust_trace, 0, sizeof(g_vcp_trust_trace));
    g_vcp_trust_trace.nonce_ledger_count = g_vcp_nonce_ledger_count;
}

void vcp_test_get_trust_trace(vcp_trust_trace_t *out)
{
    if (!out) {
        return;
    }

    *out = g_vcp_trust_trace;
    out->nonce_ledger_count = g_vcp_nonce_ledger_count;
}

uint32_t vcp_test_nonce_ledger_count(void)
{
    return g_vcp_nonce_ledger_count;
}

uint64_t vcp_test_capability_binding(struct exec_slot *slot,
                                     vcp_validation_state_t *state)
{
    return vcp_compute_capability_binding_internal(slot, state);
}

uint64_t vcp_test_signature(vcp_validation_state_t *state)
{
    return vcp_compute_validation_signature_internal(state);
}
#endif

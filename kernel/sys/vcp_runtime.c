// kernel/sys/vcp_runtime.c
// VCP Runtime Validation Enforcement

#include "../include/vcp_runtime.h"
#include "../include/execution_slot.h"
#include "../include/sha256.h"

#define memset __builtin_memset

/*
 * VCP Runtime Validation Implementation
 * 
 * This file implements the trust verification and runtime validation
 * enforcement for the AYKEN Validation Control Plane (VCP).
 * 
 * CRITICAL PRINCIPLE: This is a VERIFIED-INPUT system.
 * 
 * Trust verification order:
 *   1. NULL check (validation_state exists?)
 *   2. Capability binding (prevents forgery)
 *   3. Context hash (prevents replay)
 *   4. Signature (ensures authenticity)
 *   5. Nonce (prevents reuse)
 *   6. Validation result (only after trust verified)
 * 
 * Constitutional Compliance:
 *   - NO global state mutations (DETERMINISM.GLOBAL)
 *   - Deterministic execution (same input → same output)
 *   - NO capability bypass
 */

/*
 * vcp_compute_context_hash - Compute execution context hash
 * 
 * Computes deterministic hash of execution context to prevent replay attacks.
 * 
 * Hash inputs:
 *   - execution_slot_id (unique slot identifier)
 *   - contract_id (BCIB contract identifier from validation state)
 *   - boundary_policy (ABDF boundary policy from validation state)
 *   - metadata (additional context)
 * 
 * CRITICAL: This function MUST be deterministic.
 * Same execution context → same hash.
 * 
 * Constitutional compliance:
 *   - NO global state mutations
 *   - NO nondeterministic inputs (no wall clock, no random)
 *   - Deterministic hash algorithm (SHA256)
 * 
 * @slot: Execution slot
 * @return: Context hash (uint64_t)
 */
uint64_t vcp_compute_context_hash(struct exec_slot *slot)
{
    ayken_sha256_ctx_t hash_ctx;
    uint8_t digest[AYKEN_SHA256_DIGEST_SIZE];
    uint64_t context_hash;
    uint64_t slot_id;
    uint64_t contract_id = 0;
    uint64_t boundary_policy = 0;

    if (!slot) {
        return 0;
    }

    /* Extract context components */
    slot_id = slot->execution_id;
    
    /* If validation_state exists, include contract_id and boundary_policy */
    if (slot->validation_state) {
        contract_id = slot->validation_state->contract_id;
        boundary_policy = slot->validation_state->boundary_policy;
    }

    /* Compute deterministic hash */
    ayken_sha256_init(&hash_ctx);
    ayken_sha256_update(&hash_ctx, (const char *)&slot_id, sizeof(slot_id));
    ayken_sha256_update(&hash_ctx, (const char *)&contract_id, sizeof(contract_id));
    ayken_sha256_update(&hash_ctx, (const char *)&boundary_policy, sizeof(boundary_policy));
    ayken_sha256_final(&hash_ctx, digest);

    /* Extract first 8 bytes as context_hash */
    memset(&context_hash, 0, sizeof(context_hash));
    context_hash = ((uint64_t)digest[0] << 0) |
                   ((uint64_t)digest[1] << 8) |
                   ((uint64_t)digest[2] << 16) |
                   ((uint64_t)digest[3] << 24) |
                   ((uint64_t)digest[4] << 32) |
                   ((uint64_t)digest[5] << 40) |
                   ((uint64_t)digest[6] << 48) |
                   ((uint64_t)digest[7] << 56);

    return context_hash;
}

/*
 * vcp_verify_capability - Verify capability binding
 * 
 * Verifies that validation state is bound to a kernel-issued capability.
 * This prevents fake state injection attacks.
 * 
 * STUB IMPLEMENTATION: Returns success for now.
 * Full implementation will integrate with capability_manager.
 * 
 * @slot: Execution slot
 * @state: Validation state to verify
 * @return: 0 if valid, non-zero if invalid
 */
int vcp_verify_capability(struct exec_slot *slot, vcp_validation_state_t *state)
{
    if (!slot || !state) {
        return -1;
    }

    /*
     * TODO (Task 18.3): Implement capability binding verification
     * 
     * Full implementation will:
     *   1. Check state->capability_id is valid
     *   2. Verify capability is bound to this slot
     *   3. Verify capability has not been revoked
     *   4. Return failure if any check fails
     * 
     * For now: Accept all non-NULL states (stub)
     */
    if (state->capability_id == 0) {
        return -1;  /* No capability binding */
    }

    return 0;  /* Stub: capability valid */
}

/*
 * vcp_verify_signature - Verify VCP trust root signature
 * 
 * Verifies signature against VCP trust root.
 * 
 * STUB IMPLEMENTATION: Returns success for now.
 * Full implementation will use cryptographic verification.
 * 
 * Evidence Producer Key Model:
 *   - Kernel holds evidence producer key (NOT trust root private key)
 *   - Signature verified with producer key
 *   - CI verifies producer key authorized by trust root
 * 
 * @state: Validation state to verify
 * @return: 0 if valid, non-zero if invalid
 */
int vcp_verify_signature(vcp_validation_state_t *state)
{
    if (!state) {
        return -1;
    }

    /*
     * TODO (Task 18.4): Implement signature verification
     * 
     * Full implementation will:
     *   1. Get VCP trust root public key
     *   2. Verify signature covers all state fields
     *   3. Use cryptographic verification (e.g., Ed25519, ECDSA)
     *   4. Return failure if signature invalid
     * 
     * For now: Accept all non-zero signatures (stub)
     */
    if (state->signature == 0) {
        return -1;  /* No signature */
    }

    return 0;  /* Stub: signature valid */
}

/*
 * vcp_verify_nonce - Verify nonce uniqueness (replay protection)
 * 
 * Verifies that nonce has not been used before.
 * 
 * CRITICAL: Nonce registry MUST be append-only ledger (NOT hidden mutable global map)
 * This ensures compliance with DETERMINISM.GLOBAL constitutional rule.
 * 
 * STUB IMPLEMENTATION: Returns success for now.
 * Full implementation will use append-only nonce ledger.
 * 
 * @state: Validation state to verify
 * @return: 0 if unique, non-zero if replayed
 */
int vcp_verify_nonce(vcp_validation_state_t *state)
{
    if (!state) {
        return -1;
    }

    /*
     * TODO (Task 18.5): Implement nonce verification with append-only ledger
     * 
     * Full implementation will:
     *   1. Check nonce against append-only ledger
     *   2. If nonce exists → replay detected → return failure
     *   3. If nonce unique → append to ledger → return success
     *   4. Ledger MUST be deterministic (no hidden global state)
     * 
     * CRITICAL: Ledger = append-only structure, NOT mutable map
     * 
     * For now: Accept all non-zero nonces (stub)
     */
    if (state->nonce == 0) {
        return -1;  /* No nonce */
    }

    return 0;  /* Stub: nonce unique */
}

/*
 * vcp_verify_validation_state - Verify validation state trust
 * 
 * Performs complete trust verification before trusting validation_result.
 * 
 * Verification order (CRITICAL - DO NOT REORDER):
 *   1. NULL check (state exists?)
 *   2. Capability binding (prevents forgery)
 *   3. Context hash (prevents replay)
 *   4. Signature (ensures authenticity)
 *   5. Nonce (prevents reuse)
 *   6. Validation result (only after trust verified)
 * 
 * CRITICAL PRINCIPLE: validation_result is NEVER authoritative by itself.
 * Trust verification MUST pass first.
 * 
 * @slot: Execution slot containing validation state
 * @return: VCP_VALID if all checks pass, VCP_FAIL_CLOSED otherwise
 */
int vcp_verify_validation_state(struct exec_slot *slot)
{
    vcp_validation_state_t *state;
    uint64_t computed_context_hash;

    /* Step 1: NULL check (validation state exists?) */
    if (!slot || !slot->validation_state) {
        return VCP_FAIL_CLOSED;  /* Missing state → fail-closed */
    }

    state = slot->validation_state;

    /* Step 2: Verify capability binding (prevents forgery) */
    if (vcp_verify_capability(slot, state) != 0) {
        return VCP_FAIL_CLOSED;  /* Capability binding failed */
    }

    /* Step 3: Verify context hash (prevents replay) */
    computed_context_hash = vcp_compute_context_hash(slot);
    if (state->context_hash != computed_context_hash) {
        return VCP_FAIL_CLOSED;  /* Context hash mismatch → replay attack */
    }

    /* Step 4: Verify signature (ensures authenticity) */
    if (vcp_verify_signature(state) != 0) {
        return VCP_FAIL_CLOSED;  /* Signature verification failed */
    }

    /* Step 5: Verify nonce (prevents reuse) */
    if (vcp_verify_nonce(state) != 0) {
        return VCP_FAIL_CLOSED;  /* Nonce replayed */
    }

    /* Step 6: Check validation result (only after trust verified) */
    if (state->validation_result != VCP_VALID) {
        return VCP_FAIL_CLOSED;  /* CI rejected execution */
    }

    /* All checks passed → state is trusted */
    return VCP_VALID;
}

/*
 * vcp_runtime_validate - Runtime validation enforcement hook
 * 
 * Main enforcement point called by BCIB, ABDF, and CLI handlers.
 * 
 * Enforcement flow:
 *   1. Verify trust (capability + context + signature + nonce)
 *   2. Check validation result (only after trust verified)
 *   3. Emit evidence (Task 5 will implement)
 * 
 * CRITICAL: This function calls vcp_verify_validation_state() FIRST.
 * 
 * @slot: Execution slot to validate
 * @return: VCP_VALID if validation passed, VCP_FAIL_CLOSED otherwise
 */
int vcp_runtime_validate(struct exec_slot *slot)
{
    int trust_result;

    if (!slot) {
        return VCP_FAIL_CLOSED;
    }

    /* Step 1: Verify trust (capability + context + signature + nonce) */
    trust_result = vcp_verify_validation_state(slot);
    if (trust_result != VCP_VALID) {
        /*
         * Trust verification failed → fail-closed
         * 
         * Task 4 will implement vcp_fail_closed() to:
         *   - Block execution permanently
         *   - Emit evidence describing failure
         *   - Preserve system state integrity
         */
        return VCP_FAIL_CLOSED;
    }

    /* Step 2: Emit evidence (Task 5 will implement) */
    /*
     * TODO (Task 5): Emit validation check evidence
     * vcp_emit_validation_check(slot, VCP_VALID);
     */

    /* All checks passed → execution may proceed */
    return VCP_VALID;
}

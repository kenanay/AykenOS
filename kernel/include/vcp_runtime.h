#ifndef AYKEN_VCP_RUNTIME_H
#define AYKEN_VCP_RUNTIME_H

#include <stdint.h>
#include "execution_slot.h"

/*
 * VCP Runtime Validation API
 * 
 * This header defines the runtime validation enforcement interface for the
 * AYKEN Validation Control Plane (VCP). It provides trust verification,
 * validation state checking, and fail-closed enforcement mechanisms.
 * 
 * CRITICAL PRINCIPLE: This is a VERIFIED-INPUT system, NOT a trusted-input system.
 * 
 * Trust Model:
 *   - Validation state MUST be verified before trust
 *   - Verification checks: capability, context, signature, nonce
 *   - Verification failure → fail-closed (no bypass)
 *   - validation_result flag is NEVER authoritative by itself
 * 
 * Constitutional Compliance:
 *   - NO global state mutations (DETERMINISM.GLOBAL)
 *   - NO capability bypass (KERNEL.CAPABILITY.BYPASS)
 *   - NO Ring3→Ring0 direct access (SECURITY.BOUNDARY.VIOLATION)
 *   - Deterministic execution (same input → same output)
 */

/*
 * VCP Validation Result Codes
 * 
 * These codes indicate the result of validation state verification.
 * 
 * CRITICAL: These are VERIFICATION results, not trust indicators.
 * A validation_state with validation_result=VCP_VALID is NOT trusted
 * until ALL trust checks pass (capability, context, signature, nonce).
 */
typedef enum {
    VCP_VALID = 0,           /* Validation state verified and valid */
    VCP_INVALID = 1,         /* Validation state verified but invalid */
    VCP_MISSING = 2,         /* Validation state is NULL (fail-closed trigger) */
    VCP_FAIL_CLOSED = 3,     /* Validation failed, execution blocked */
} vcp_validation_result_t;

/*
 * VCP Trust Verification Result Codes
 * 
 * These codes indicate which trust verification check failed.
 * Used for evidence emission and debugging.
 */
typedef enum {
    VCP_TRUST_VERIFIED = 0,              /* All trust checks passed */
    VCP_TRUST_FAILED_CAPABILITY = 1,     /* Capability binding invalid */
    VCP_TRUST_FAILED_CONTEXT = 2,        /* Context hash mismatch (replay) */
    VCP_TRUST_FAILED_SIGNATURE = 3,      /* Signature verification failed */
    VCP_TRUST_FAILED_NONCE = 4,          /* Nonce replayed (reuse detected) */
} vcp_trust_result_t;

/*
 * ABI Verification (CRITICAL - DO NOT REMOVE)
 * 
 * This section verifies that the vcp_validation_state_t structure from
 * execution_slot.h matches the FINAL ABI contract defined in Task 1.
 * 
 * If these assertions fail, the ABI has drifted and Task 18 verification
 * functions will break.
 * 
 * CRITICAL: Task 1 defined the FINAL ABI. Task 18 VERIFIES that layout.
 * This is NOT a redefinition - it's a verification checkpoint.
 */
_Static_assert(sizeof(vcp_validation_state_t) == 72,
               "VCP ABI DRIFT: Task 1 defined 72 bytes, current size differs");
_Static_assert(__alignof__(vcp_validation_state_t) == 8,
               "VCP ABI DRIFT: Task 1 defined 8-byte alignment, current alignment differs");

/* Verify individual field sizes (prevent accidental type changes) */
_Static_assert(sizeof(((vcp_validation_state_t *)0)->validation_result) == 8,
               "VCP ABI DRIFT: validation_result must be 8 bytes");
_Static_assert(sizeof(((vcp_validation_state_t *)0)->contract_id) == 8,
               "VCP ABI DRIFT: contract_id must be 8 bytes");
_Static_assert(sizeof(((vcp_validation_state_t *)0)->boundary_policy) == 8,
               "VCP ABI DRIFT: boundary_policy must be 8 bytes");
_Static_assert(sizeof(((vcp_validation_state_t *)0)->context_hash) == 8,
               "VCP ABI DRIFT: context_hash must be 8 bytes");
_Static_assert(sizeof(((vcp_validation_state_t *)0)->nonce) == 8,
               "VCP ABI DRIFT: nonce must be 8 bytes");
_Static_assert(sizeof(((vcp_validation_state_t *)0)->signature) == 8,
               "VCP ABI DRIFT: signature must be 8 bytes");
_Static_assert(sizeof(((vcp_validation_state_t *)0)->capability_id) == 8,
               "VCP ABI DRIFT: capability_id must be 8 bytes");
_Static_assert(sizeof(((vcp_validation_state_t *)0)->evidence_id) == 8,
               "VCP ABI DRIFT: evidence_id must be 8 bytes");
_Static_assert(sizeof(((vcp_validation_state_t *)0)->timestamp) == 8,
               "VCP ABI DRIFT: timestamp must be 8 bytes");

/*
 * Trust Verification Functions
 * 
 * These functions implement the trust verification model:
 *   1. Capability binding verification (prevents forgery)
 *   2. Context hash verification (prevents replay)
 *   3. Signature verification (ensures authenticity)
 *   4. Nonce verification (prevents reuse)
 * 
 * CRITICAL ORDER: All checks must pass before trusting validation_result.
 */

/*
 * vcp_verify_validation_state - Verify validation state trust
 * 
 * @slot: Execution slot containing validation state
 * 
 * Performs complete trust verification:
 *   1. Check validation_state is not NULL
 *   2. Verify capability binding
 *   3. Verify context hash
 *   4. Verify signature
 *   5. Verify nonce uniqueness
 *   6. Check validation_result
 * 
 * Returns:
 *   VCP_VALID - All checks passed, state is trusted
 *   VCP_FAIL_CLOSED - Any check failed, execution must be blocked
 * 
 * CRITICAL: This is the PRIMARY trust verification function.
 * Task 2 (runtime hook) will call this function.
 */
int vcp_verify_validation_state(struct exec_slot *slot);

/*
 * vcp_verify_capability - Verify capability binding
 * 
 * @slot: Execution slot
 * @state: Validation state to verify
 * 
 * Verifies that validation state is bound to a kernel-issued capability.
 * This prevents fake state injection attacks.
 * 
 * Returns:
 *   0 - Capability binding valid
 *   non-zero - Capability binding invalid or missing
 */
int vcp_verify_capability(struct exec_slot *slot, vcp_validation_state_t *state);

/*
 * vcp_compute_context_hash - Compute execution context hash
 * 
 * @slot: Execution slot
 * 
 * Computes deterministic hash of execution context:
 *   - BCIB contract_id
 *   - ABDF boundary_policy
 *   - execution_slot_id
 *   - metadata
 * 
 * Returns: Context hash (uint64_t)
 * 
 * CRITICAL: Must be deterministic (same execution → same hash)
 */
uint64_t vcp_compute_context_hash(struct exec_slot *slot);

/*
 * vcp_verify_signature - Verify VCP trust root signature
 * 
 * @state: Validation state to verify
 * 
 * Verifies signature against VCP trust root.
 * Uses kernel evidence producer key model:
 *   - Kernel holds evidence producer key (NOT trust root private key)
 *   - Signature verified with producer key
 *   - CI verifies producer key authorized by trust root
 * 
 * Returns:
 *   0 - Signature valid
 *   non-zero - Signature invalid
 */
int vcp_verify_signature(vcp_validation_state_t *state);

/*
 * vcp_verify_nonce - Verify nonce uniqueness (replay protection)
 * 
 * @state: Validation state to verify
 * 
 * Verifies that nonce has not been used before.
 * 
 * CRITICAL: Nonce registry MUST be append-only ledger (NOT hidden mutable global map)
 * This ensures compliance with DETERMINISM.GLOBAL constitutional rule.
 * 
 * Implementation:
 *   - Nonce ledger is append-only structure
 *   - Deterministic, no global state mutation
 *   - Same execution → same nonce checks → same result
 * 
 * Returns:
 *   0 - Nonce is unique (not replayed)
 *   non-zero - Nonce has been used before (replay detected)
 */
int vcp_verify_nonce(vcp_validation_state_t *state);

/*
 * Runtime Validation Hook
 * 
 * This is the main enforcement point called by BCIB, ABDF, and CLI handlers.
 * 
 * CRITICAL: This function calls vcp_verify_validation_state() FIRST,
 * then checks validation_result ONLY if trust verification passes.
 */

/*
 * vcp_runtime_validate - Runtime validation enforcement hook
 * 
 * @slot: Execution slot to validate
 * 
 * Performs runtime validation enforcement:
 *   1. Verify trust (capability + context + signature + nonce)
 *   2. Check validation result (only after trust verified)
 *   3. Emit evidence
 * 
 * Returns:
 *   VCP_VALID - Validation passed, execution may proceed
 *   VCP_FAIL_CLOSED - Validation failed, execution must be blocked
 * 
 * CRITICAL: This function is called by:
 *   - BCIB executor (before contract execution)
 *   - ABDF boundary validator (before boundary crossing)
 *   - CLI handler (before CLI execution)
 */
int vcp_runtime_validate(struct exec_slot *slot);

#endif /* AYKEN_VCP_RUNTIME_H */

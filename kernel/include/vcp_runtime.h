#ifndef AYKEN_VCP_RUNTIME_H
#define AYKEN_VCP_RUNTIME_H

#include <stdint.h>
#include "execution_slot.h"

#ifndef AYKEN_VCP_TRUST_VERIFICATION_TEST
#define AYKEN_VCP_TRUST_VERIFICATION_TEST 0
#endif

#ifndef AYKEN_VCP_RUNTIME_HOOK_TEST
#define AYKEN_VCP_RUNTIME_HOOK_TEST 0
#endif

#ifndef AYKEN_VCP_FAIL_CLOSED_TEST
#define AYKEN_VCP_FAIL_CLOSED_TEST 0
#endif

#ifndef AYKEN_VCP_EVIDENCE_TEST
#define AYKEN_VCP_EVIDENCE_TEST 0
#endif

#define AYKEN_VCP_TEST_HOOKS \
    (AYKEN_VCP_TRUST_VERIFICATION_TEST || \
     AYKEN_VCP_RUNTIME_HOOK_TEST || \
     AYKEN_VCP_FAIL_CLOSED_TEST || \
     AYKEN_VCP_EVIDENCE_TEST)

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
 *   - Verification failure -> fail-closed (no bypass)
 *   - validation_result flag is NEVER authoritative by itself
 * 
 * Constitutional Compliance:
 *   - Nonce state is append-only and deterministic
 *   - NO capability bypass (KERNEL.CAPABILITY.BYPASS)
 *   - NO Ring3->Ring0 direct access (SECURITY.BOUNDARY.VIOLATION)
 *   - Deterministic execution (same input -> same output)
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

#define VCP_FAIL_CLOSED_SLOT_ERROR_CODE 0x565043FCu

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
 * CRITICAL: Must be deterministic (same execution -> same hash)
 */
uint64_t vcp_compute_context_hash(struct exec_slot *slot);

/*
 * vcp_verify_signature - Verify validation-state signature
 * 
 * @state: Validation state to verify
 * 
 * Verifies that signature covers the validation state fields accepted by the
 * deterministic runtime verifier. Production trust-root crypto can replace the
 * deterministic verifier without weakening the call contract.
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
 * CRITICAL: Nonce registry MUST be append-only ledger (NOT hidden mutable map).
 * 
 * Implementation:
 *   - Nonce ledger is append-only structure
 *   - vcp_verify_nonce() is a pure uniqueness check
 *   - Accepted states append their nonce only after validation_result passes
 *   - Same execution -> same nonce checks -> same result
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
 * CRITICAL: This function calls vcp_verify_validation_state(); that verifier
 * checks validation_result only after trust verification has passed.
 */

/*
 * vcp_runtime_validate - Runtime validation enforcement hook
 * 
 * @slot: Execution slot to validate
 * 
 * Performs runtime validation enforcement:
 *   1. Verify trust (capability + context + signature + nonce)
 *   2. Check validation result inside the verifier only after trust
 *   3. Return fail-closed on any failure
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

/*
 * vcp_fail_closed - Permanent fail-closed enforcement handler
 *
 * @slot: Execution slot to block, or NULL for non-slot failures
 * @reason: Deterministic diagnostic reason string
 *
 * Permanently blocks the slot by moving it to EXEC_SLOT_ABORTED and assigning
 * VCP_FAIL_CLOSED_SLOT_ERROR_CODE. Repeated calls are idempotent.
 *
 * Returns:
 *   VCP_FAIL_CLOSED - Execution must remain blocked
 */
int vcp_fail_closed(struct exec_slot *slot, const char *reason);

/*
 * vcp_fail_closed_is_active - Check permanent fail-closed state
 *
 * Returns non-zero only when this slot was blocked by the VCP fail-closed
 * handler. Generic EXEC_SLOT_ABORTED states are not treated as VCP failures.
 */
int vcp_fail_closed_is_active(const struct exec_slot *slot);

/*
 * Diagnostic evidence emission API surface.
 *
 * Task 5 provides diagnostic-only stub definitions. Authoritative, signed, and
 * durable-before-proceed evidence is introduced by Tasks 20-23.
 */
void vcp_emit_validation_check(struct exec_slot *slot, int result);
void vcp_emit_execution_block(struct exec_slot *slot, const char *reason);
void vcp_emit_contract_execution(struct exec_slot *slot, const char *contract_id);
void vcp_emit_boundary_crossing(struct exec_slot *slot, const char *boundary_id);

#define VCP_DIAGNOSTIC_EVIDENCE_CAPACITY 64u

typedef enum {
    VCP_DIAG_EVENT_NONE = 0,
    VCP_DIAG_EVENT_VALIDATION_CHECK = 1,
    VCP_DIAG_EVENT_EXECUTION_BLOCK = 2,
    VCP_DIAG_EVENT_CONTRACT_EXECUTION = 3,
    VCP_DIAG_EVENT_BOUNDARY_CROSSING = 4,
} vcp_diagnostic_evidence_type_t;

typedef struct vcp_diagnostic_evidence_entry {
    uint32_t index;
    uint32_t event_type;
    int32_t result;
    uint32_t reason_hash;
    uint32_t label_hash;
    uint32_t reserved0;
    uint64_t slot_id;
    uint64_t generation;
    uint64_t owner_pid;
    uint64_t target_context_id;
    uint64_t slot_state;
    uint64_t error_code;
    uint64_t event_result;
    uint64_t context_hash;
    uint64_t nonce;
    uint64_t capability_id;
    uint64_t evidence_id;
} vcp_diagnostic_evidence_entry_t;

#define VCP_TRUST_TRACE_CAPACITY 16u

typedef enum {
    VCP_TRACE_NONE = 0,
    VCP_TRACE_CAPABILITY = 1,
    VCP_TRACE_CONTEXT = 2,
    VCP_TRACE_SIGNATURE = 3,
    VCP_TRACE_NONCE = 4,
    VCP_TRACE_RESULT = 5,
    VCP_TRACE_NONCE_COMMIT = 6,
    VCP_TRACE_FAIL_CLOSED = 7,
} vcp_trust_trace_event_t;

#if AYKEN_VCP_TEST_HOOKS
typedef struct vcp_trust_trace {
    uint32_t count;
    uint32_t nonce_ledger_count;
    uint32_t events[VCP_TRUST_TRACE_CAPACITY];
} vcp_trust_trace_t;

/*
 * Test-mode deterministic issuer hooks.
 *
 * These helpers are intentionally available only in VCP validation test builds.
 * They let property tests create states that satisfy the verifier without
 * treating arbitrary non-zero capability/signature/nonce values as trusted.
 */
void vcp_test_reset_trust_environment(void);
void vcp_test_reset_trust_trace(void);
void vcp_test_get_trust_trace(vcp_trust_trace_t *out);
uint32_t vcp_test_nonce_ledger_count(void);
uint64_t vcp_test_capability_binding(struct exec_slot *slot,
                                     vcp_validation_state_t *state);
uint64_t vcp_test_signature(vcp_validation_state_t *state);
void vcp_test_reset_diagnostic_evidence(void);
uint32_t vcp_test_diagnostic_evidence_count(void);
int vcp_test_get_diagnostic_evidence(uint32_t logical_index,
                                     vcp_diagnostic_evidence_entry_t *out);
#endif

#endif /* AYKEN_VCP_RUNTIME_H */

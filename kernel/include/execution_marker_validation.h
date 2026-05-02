#ifndef EXECUTION_MARKER_VALIDATION_H
#define EXECUTION_MARKER_VALIDATION_H

/*
 * Execution Marker Validation
 * 
 * PURPOSE:
 * Validate execution marker sequence deterministically.
 * 
 * RULES:
 * - NO dynamic allocation
 * - NO I/O
 * - NO side effects
 * - Pure function behavior
 * 
 * Kernel and userspace must share same logic.
 * 
 * SCOPE:
 * Phase-17 execution pipeline marker order enforcement.
 * 
 * IMMUTABILITY:
 * Marker order is IMMUTABLE. Changes require spec update.
 */

#include <stdint.h>
#include <stddef.h>

/* Marker IDs (canonical order - IMMUTABLE) */
typedef enum {
    MARKER_EXEC_START = 0,
    MARKER_EXEC_OUTPUT_WRITTEN,
    MARKER_EXEC_COMPLETE_OK,
    MARKER_VERIFY_START,
    MARKER_VERIFY_PASS,
    MARKER_RESULT_OK,
    MARKER_WAIT_OK,
    MARKER_COUNT
} execution_marker_t;

/* Validation result codes */
typedef enum {
    MARKER_VALIDATION_OK = 0,
    MARKER_VALIDATION_INVALID_ORDER,
    MARKER_VALIDATION_MISSING,
    MARKER_VALIDATION_DUPLICATE,
    MARKER_VALIDATION_OUT_OF_BOUNDS
} marker_validation_result_t;

/* Marker error codes (stored in exec_slot_t::marker_error_code) */
typedef enum {
    MARKER_ERROR_NONE = 0,
    MARKER_ERROR_INVALID_ORDER = 1,
    MARKER_ERROR_DUPLICATE = 2,
    MARKER_ERROR_OVERFLOW = 3,
    MARKER_ERROR_OUT_OF_BOUNDS = 4
} marker_error_code_t;

/*
 * Validate marker sequence
 * 
 * markers: array of marker IDs
 * count: number of markers
 * 
 * returns: validation result
 * 
 * DETERMINISM:
 * Same input → same output (no side effects)
 * 
 * THREAD SAFETY:
 * Pure function, no shared state
 */
marker_validation_result_t 
execution_marker_validate(
    const execution_marker_t *markers,
    size_t count
);

/*
 * Validate single marker transition
 * 
 * current: current marker
 * next: next marker
 * 
 * returns: validation result
 * 
 * RULE:
 * next must be current + 1 (strict sequential order)
 */
marker_validation_result_t
execution_marker_validate_transition(
    execution_marker_t current,
    execution_marker_t next
);

/*
 * Get marker name (for debugging/logging)
 * 
 * marker: marker ID
 * 
 * returns: marker name string (static, never NULL)
 * 
 * NOTE:
 * No I/O is performed.
 * Returns static string; no allocation.
 */
const char *
execution_marker_name(execution_marker_t marker);

/*
 * execution_slot_validate_markers_locked - Validate captured markers (read-only)
 * 
 * @slot: Execution slot
 * 
 * Pure validation - NO state mutation, NO side effects.
 * Returns error code if validation fails.
 * 
 * RULE: VALIDATE AFTER WRITE
 * This is called AFTER all markers are captured (Step 4).
 * 
 * Pre-commit guard: Called before state transition to COMPLETED/RESULT_MAPPED.
 * 
 * Returns:
 *   0 = validation passed
 *   non-zero = marker_error_code_t value
 */
int execution_slot_validate_markers_locked(const void *slot);

#endif /* EXECUTION_MARKER_VALIDATION_H */

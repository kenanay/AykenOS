/*
 * Execution Marker Validation - Kernel Implementation
 * 
 * PURPOSE:
 * Deterministic validation of execution marker sequences.
 * 
 * RULES:
 * - NO dynamic allocation
 * - NO I/O (except via debugcon wrapper if needed)
 * - NO side effects
 * - Pure function behavior
 * 
 * DETERMINISM:
 * Same input → same output, always.
 * 
 * THREAD SAFETY:
 * Pure functions, no shared mutable state.
 */

#include <execution_marker_validation.h>

/* Marker name strings (static, immutable) */
static const char *MARKER_NAMES[MARKER_COUNT] = {
    [MARKER_EXEC_START] = "EXEC_START",
    [MARKER_EXEC_OUTPUT_WRITTEN] = "EXEC_OUTPUT_WRITTEN",
    [MARKER_EXEC_COMPLETE_OK] = "EXEC_COMPLETE_OK",
    [MARKER_VERIFY_START] = "VERIFY_START",
    [MARKER_VERIFY_PASS] = "VERIFY_PASS",
    [MARKER_RESULT_OK] = "RESULT_OK",
    [MARKER_WAIT_OK] = "WAIT_OK"
};

/*
 * Get marker name
 * 
 * Pure function: no I/O, no allocation, no side effects.
 */
const char *
execution_marker_name(execution_marker_t marker)
{
    if (marker >= MARKER_COUNT) {
        return "INVALID_MARKER";
    }
    return MARKER_NAMES[marker];
}

/*
 * Validate single marker transition
 * 
 * RULE: next must be current + 1 (strict sequential order)
 * 
 * Pure function: deterministic, no side effects.
 */
marker_validation_result_t
execution_marker_validate_transition(
    execution_marker_t current,
    execution_marker_t next
)
{
    /* Bounds check */
    if (current >= MARKER_COUNT || next >= MARKER_COUNT) {
        return MARKER_VALIDATION_OUT_OF_BOUNDS;
    }
    
    /* Sequential order check */
    if (next != current + 1) {
        return MARKER_VALIDATION_INVALID_ORDER;
    }
    
    return MARKER_VALIDATION_OK;
}

/*
 * Validate complete marker sequence
 * 
 * RULES:
 * - Markers must be in strict sequential order
 * - No gaps allowed
 * - No duplicates allowed
 * - Must start from MARKER_EXEC_START
 * 
 * Pure function: deterministic, no side effects.
 */
marker_validation_result_t
execution_marker_validate(
    const execution_marker_t *markers,
    size_t count
)
{
    /* Null check */
    if (markers == ((void *)0)) {
        return MARKER_VALIDATION_INVALID_ORDER;
    }
    
    /* Empty sequence check */
    if (count == 0) {
        return MARKER_VALIDATION_MISSING;
    }
    
    /* First marker must be EXEC_START */
    if (markers[0] != MARKER_EXEC_START) {
        return MARKER_VALIDATION_INVALID_ORDER;
    }
    
    /* Validate each transition */
    for (size_t i = 0; i < count - 1; i++) {
        marker_validation_result_t result = 
            execution_marker_validate_transition(markers[i], markers[i + 1]);
        
        if (result != MARKER_VALIDATION_OK) {
            return result;
        }
    }
    
    /* Check for duplicates (strict sequential means no duplicates) */
    for (size_t i = 0; i < count; i++) {
        for (size_t j = i + 1; j < count; j++) {
            if (markers[i] == markers[j]) {
                return MARKER_VALIDATION_DUPLICATE;
            }
        }
    }
    
    return MARKER_VALIDATION_OK;
}

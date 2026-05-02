/*
 * execution_marker_injection.c - Phase-17 Marker Validation Injection Harness
 *
 * ⚠️ CRITICAL: TEST-ONLY CODE
 * All injection functions are bounds-safe and deterministic.
 * They corrupt marker state in controlled ways to test validation.
 *
 * Authority: Kenan AY - Architectural Steward
 * Mandate: Test-only guard is NON-NEGOTIABLE
 */

#include "execution_marker_injection.h"

/*
 * ⚠️ TOP-LEVEL GUARD: All injection code behind test-only flag
 */
#if defined(AYKEN_PHASE17_MARKER_INJECTION_TEST) && (AYKEN_PHASE17_MARKER_INJECTION_TEST == 1)

/*
 * Test 1: Invalid Order Injection
 * Swap markers 1 and 2 to create invalid sequence
 * Expected: [0,1,2,3,4] → [0,2,1,3,4]
 * Validation: Layer 2 (sequence check) should fail
 */
#if defined(AYKEN_MARKER_INJECT_INVALID_ORDER) && (AYKEN_MARKER_INJECT_INVALID_ORDER == 1)
void inject_invalid_order(exec_slot_t *slot) {
    /* Bounds-safe: only swap if we have at least 3 markers */
    if (slot->marker_count >= 3) {
        uint8_t temp = slot->marker_sequence[1];
        slot->marker_sequence[1] = slot->marker_sequence[2];
        slot->marker_sequence[2] = temp;
    }
}
#endif

/*
 * Test 2: Duplicate Marker Injection
 * Duplicate marker 1 at position 2
 * Expected: [0,1,2,3,4] → [0,1,1,3,4]
 * Validation: Layer 2 (sequence check) should fail
 */
#if defined(AYKEN_MARKER_INJECT_DUPLICATE) && (AYKEN_MARKER_INJECT_DUPLICATE == 1)
void inject_duplicate(exec_slot_t *slot) {
    /* Bounds-safe: only duplicate if we have at least 3 markers */
    if (slot->marker_count >= 3) {
        slot->marker_sequence[2] = slot->marker_sequence[1];
    }
}
#endif

/*
 * Test 3: Missing Marker Injection
 * Remove marker 2 by shifting sequence and reducing count
 * Expected: [0,1,2,3,4] → [0,1,3,4] (count=4)
 * Validation: Layer 1 (count check) should fail
 */
#if defined(AYKEN_MARKER_INJECT_MISSING) && (AYKEN_MARKER_INJECT_MISSING == 1)
void inject_missing(exec_slot_t *slot) {
    /* Bounds-safe: only shift if we have exactly 5 markers */
    if (slot->marker_count == 5) {
        /* Shift markers 3,4 left to positions 2,3 */
        slot->marker_sequence[2] = slot->marker_sequence[3];
        slot->marker_sequence[3] = slot->marker_sequence[4];
        /* Clear position 4 (hygiene) */
        slot->marker_sequence[4] = 0;
        /* Reduce count */
        slot->marker_count = 4;
    }
}
#endif

/*
 * Test 4: Overflow Injection
 * Force overflow condition by setting count > 7
 * Expected: marker_count = 8, error_code = OVERFLOW
 * Validation: Pre-validation error code check should fail
 */
#if defined(AYKEN_MARKER_INJECT_OVERFLOW) && (AYKEN_MARKER_INJECT_OVERFLOW == 1)
void inject_overflow(exec_slot_t *slot) {
    /* Force overflow state */
    slot->marker_count = 8;
    slot->marker_error_code = MARKER_ERROR_OVERFLOW;
}
#endif

/*
 * Test 5: Stale Buffer Data Injection
 * Valid markers but garbage in unused buffer space
 * Expected: [0,1,2,3,4,0,0] → [0,1,2,3,4,0xAA,0xBB]
 * Validation: Layer 4 (hygiene check) should fail
 */
#if defined(AYKEN_MARKER_INJECT_STALE_DATA) && (AYKEN_MARKER_INJECT_STALE_DATA == 1)
void inject_stale_data(exec_slot_t *slot) {
    /* Bounds-safe: only inject if we have exactly 5 markers */
    if (slot->marker_count == 5) {
        /* Inject garbage at positions 5 and 6 (unused buffer space) */
        slot->marker_sequence[5] = 0xAA;
        slot->marker_sequence[6] = 0xBB;
    }
}
#endif

/*
 * Test 6: Corrupted Bitmap Injection
 * Valid sequence but bitmap has extra bits set
 * Expected: bitmap = 0x1F → 0x3F (bit 5 also set)
 * Validation: Layer 3 (bitmap check) should fail
 */
#if defined(AYKEN_MARKER_INJECT_CORRUPT_BITMAP) && (AYKEN_MARKER_INJECT_CORRUPT_BITMAP == 1)
void inject_corrupt_bitmap(exec_slot_t *slot) {
    /* Corrupt bitmap by setting extra bit */
    slot->marker_bitmap = 0x3F; /* bits 0-5 set instead of 0-4 */
}
#endif

/*
 * Test 7: Partial Write Injection
 * Simulate interrupted capture (race condition)
 * Expected: [0,1,2,3,4] → [0,1,2] (count=3)
 * Validation: Layer 1 (count check) should fail
 */
#if defined(AYKEN_MARKER_INJECT_PARTIAL_WRITE) && (AYKEN_MARKER_INJECT_PARTIAL_WRITE == 1)
void inject_partial_write(exec_slot_t *slot) {
    /* Simulate partial write by truncating count */
    if (slot->marker_count >= 3) {
        slot->marker_count = 3;
        /* Clear remaining positions (hygiene) */
        slot->marker_sequence[3] = 0;
        slot->marker_sequence[4] = 0;
    }
}
#endif

#endif /* AYKEN_PHASE17_MARKER_INJECTION_TEST */

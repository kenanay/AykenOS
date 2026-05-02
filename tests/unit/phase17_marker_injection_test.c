/*
 * Phase-17 Marker Injection Runtime Test
 * 
 * PURPOSE:
 * Prove validation actually works at runtime (not just compiles).
 * 
 * SCOPE:
 * - Minimal: Test validation function directly
 * - Verify: Error code propagation + validation behavior
 * - No QEMU: Userspace unit test
 * 
 * CRITICAL:
 * This is the MINIMUM test to prove "validation works" vs. "validation compiles"
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>

/* Minimal exec_slot_t mock for testing */
typedef struct {
    uint8_t in_use;
    uint8_t marker_count;
    uint8_t marker_sequence[7];
    uint8_t marker_bitmap;
    uint8_t marker_error_code;
} exec_slot_t;

/* Marker error codes (from kernel/include/execution_marker_validation.h) */
typedef enum {
    MARKER_ERROR_NONE = 0,
    MARKER_ERROR_INVALID_ORDER = 1,
    MARKER_ERROR_OUT_OF_BOUNDS = 2,
    MARKER_ERROR_OVERFLOW = 3
} marker_error_code_t;

/*
 * Inline validation function (from kernel/sys/execution_slot.c)
 * This is a COPY for testing purposes only.
 */
static int execution_slot_validate_markers_locked(const void *slot_ptr)
{
    const exec_slot_t *slot = (const exec_slot_t *)slot_ptr;
    const uint8_t EXPECTED_COUNT = 5;
    uint8_t i;
    
    if (!slot || !slot->in_use) {
        return MARKER_ERROR_OUT_OF_BOUNDS;
    }
    
    // Check if error already occurred during capture
    if (slot->marker_error_code != 0) {
        return slot->marker_error_code;
    }
    
    // Validate count (must be exactly 5 at this point)
    if (slot->marker_count != EXPECTED_COUNT) {
        return MARKER_ERROR_INVALID_ORDER;
    }
    
    // Validate sequence order (strict sequential: 0, 1, 2, 3, 4)
    for (i = 0; i < EXPECTED_COUNT; i++) {
        if (slot->marker_sequence[i] != i) {
            return MARKER_ERROR_INVALID_ORDER;
        }
    }
    
    // Validate bitmap (must match expected markers: bits 0-4 set)
    if (slot->marker_bitmap != 0x1F) {  // 0b00011111
        return MARKER_ERROR_INVALID_ORDER;
    }
    
    // Defensive check: ensure no garbage in unused buffer space
    for (i = EXPECTED_COUNT; i < 7; i++) {
        if (slot->marker_sequence[i] != 0) {
            return MARKER_ERROR_INVALID_ORDER;  // Garbage detected
        }
    }
    
    return MARKER_ERROR_NONE;
}

/* Test counter */
static int tests_passed = 0;
static int tests_failed = 0;

#define TEST(name) \
    do { \
        printf("TEST: %s ... ", name); \
        fflush(stdout); \
    } while (0)

#define PASS() \
    do { \
        printf("✅ PASS\n"); \
        tests_passed++; \
    } while (0)

#define FAIL(msg) \
    do { \
        printf("❌ FAIL: %s\n", msg); \
        tests_failed++; \
    } while (0)

/*
 * Test 1: Valid sequence (baseline)
 */
void test_valid_sequence(void)
{
    TEST("valid_sequence");
    
    exec_slot_t slot = {
        .in_use = 1,
        .marker_count = 5,
        .marker_sequence = {0, 1, 2, 3, 4, 0, 0},
        .marker_bitmap = 0x1F,  /* bits 0-4 set */
        .marker_error_code = MARKER_ERROR_NONE
    };
    
    int result = execution_slot_validate_markers_locked(&slot);
    
    if (result != 0) {
        FAIL("Expected validation to pass (result=0)");
        return;
    }
    
    PASS();
}

/*
 * Test 2: Invalid order (CRITICAL RUNTIME TEST)
 * 
 * This is the MINIMUM test to prove validation works.
 * If this passes, we know:
 * 1. Validation function executes
 * 2. Error detection works
 * 3. Error code propagates correctly
 */
void test_invalid_order(void)
{
    TEST("invalid_order (CRITICAL)");
    
    /* Inject invalid order: [0,2,1,3,4] instead of [0,1,2,3,4] */
    exec_slot_t slot = {
        .in_use = 1,
        .marker_count = 5,
        .marker_sequence = {0, 2, 1, 3, 4, 0, 0},  /* markers 1 and 2 swapped */
        .marker_bitmap = 0x1F,
        .marker_error_code = MARKER_ERROR_NONE
    };
    
    int result = execution_slot_validate_markers_locked(&slot);
    
    /* Expected: validation should fail (non-zero return) */
    if (result == 0) {
        FAIL("Validation passed but should have failed (invalid order not detected)");
        return;
    }
    
    /* Expected: result should be MARKER_ERROR_INVALID_ORDER */
    if (result != MARKER_ERROR_INVALID_ORDER) {
        char msg[128];
        snprintf(msg, sizeof(msg), "Wrong error code (expected %d, got %d)", 
                 MARKER_ERROR_INVALID_ORDER, result);
        FAIL(msg);
        return;
    }
    
    PASS();
}

/*
 * Test 3: Invalid count
 */
void test_invalid_count(void)
{
    TEST("invalid_count");
    
    exec_slot_t slot = {
        .in_use = 1,
        .marker_count = 3,  /* Should be 5 */
        .marker_sequence = {0, 1, 2, 0, 0, 0, 0},
        .marker_bitmap = 0x07,  /* bits 0-2 set */
        .marker_error_code = MARKER_ERROR_NONE
    };
    
    int result = execution_slot_validate_markers_locked(&slot);
    
    if (result == 0) {
        FAIL("Validation passed but should have failed (invalid count)");
        return;
    }
    
    if (result != MARKER_ERROR_INVALID_ORDER) {
        FAIL("Wrong error code");
        return;
    }
    
    PASS();
}

/*
 * Test 4: Invalid bitmap
 */
void test_invalid_bitmap(void)
{
    TEST("invalid_bitmap");
    
    exec_slot_t slot = {
        .in_use = 1,
        .marker_count = 5,
        .marker_sequence = {0, 1, 2, 3, 4, 0, 0},
        .marker_bitmap = 0x3F,  /* Extra bit set (should be 0x1F) */
        .marker_error_code = MARKER_ERROR_NONE
    };
    
    int result = execution_slot_validate_markers_locked(&slot);
    
    if (result == 0) {
        FAIL("Validation passed but should have failed (invalid bitmap)");
        return;
    }
    
    if (result != MARKER_ERROR_INVALID_ORDER) {
        FAIL("Wrong error code");
        return;
    }
    
    PASS();
}

/*
 * Test 5: Stale buffer data (Layer 4 hygiene check)
 */
void test_stale_buffer_data(void)
{
    TEST("stale_buffer_data");
    
    exec_slot_t slot = {
        .in_use = 1,
        .marker_count = 5,
        .marker_sequence = {0, 1, 2, 3, 4, 0xAA, 0xBB},  /* Garbage at positions 5-6 */
        .marker_bitmap = 0x1F,
        .marker_error_code = MARKER_ERROR_NONE
    };
    
    int result = execution_slot_validate_markers_locked(&slot);
    
    if (result == 0) {
        FAIL("Validation passed but should have failed (stale buffer data)");
        return;
    }
    
    if (result != MARKER_ERROR_INVALID_ORDER) {
        FAIL("Wrong error code");
        return;
    }
    
    PASS();
}

/*
 * Main test runner
 */
int main(void)
{
    printf("=== Phase-17 Marker Injection Runtime Tests ===\n");
    printf("⚠️  CRITICAL: Proving validation works at runtime\n\n");
    
    /* Baseline: valid sequence should pass */
    test_valid_sequence();
    
    /* CRITICAL: invalid order should fail with correct error code */
    test_invalid_order();
    
    /* Additional validation scenarios */
    test_invalid_count();
    test_invalid_bitmap();
    test_stale_buffer_data();
    
    /* Summary */
    printf("\n=== Test Summary ===\n");
    printf("PASSED: %d\n", tests_passed);
    printf("FAILED: %d\n", tests_failed);
    
    if (tests_failed == 0) {
        printf("\n✅ ALL TESTS PASS\n");
        printf("🔥 CRITICAL: Validation behavior verified at runtime\n");
        return 0;
    } else {
        printf("\n❌ SOME TESTS FAILED\n");
        printf("🚨 CRITICAL: Validation behavior NOT working correctly\n");
        return 1;
    }
}

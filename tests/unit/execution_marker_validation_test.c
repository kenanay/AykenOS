/*
 * Execution Marker Validation - Userspace Test
 * 
 * PURPOSE:
 * Test marker validation logic in isolation (no kernel dependencies).
 * 
 * SCOPE:
 * - Valid sequences
 * - Invalid sequences (gaps, duplicates, wrong order)
 * - Boundary conditions
 * - Marker name resolution
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>

/* Include kernel header (pure interface, no kernel dependencies) */
#include "../../kernel/include/execution_marker_validation.h"

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
        printf("PASS\n"); \
        tests_passed++; \
    } while (0)

#define FAIL(msg) \
    do { \
        printf("FAIL: %s\n", msg); \
        tests_failed++; \
    } while (0)

#define ASSERT_EQ(a, b, msg) \
    do { \
        if ((a) != (b)) { \
            FAIL(msg); \
            return; \
        } \
    } while (0)

/* Kernel implementation (linked from kernel/sys/execution_marker_validation.c) */
extern marker_validation_result_t execution_marker_validate(
    const execution_marker_t *markers, size_t count);
extern marker_validation_result_t execution_marker_validate_transition(
    execution_marker_t current, execution_marker_t next);
extern const char *execution_marker_name(execution_marker_t marker);

/*
 * Test: Valid full sequence
 */
void test_valid_full_sequence(void)
{
    TEST("valid_full_sequence");
    
    execution_marker_t markers[] = {
        MARKER_EXEC_START,
        MARKER_EXEC_OUTPUT_WRITTEN,
        MARKER_EXEC_COMPLETE_OK,
        MARKER_VERIFY_START,
        MARKER_VERIFY_PASS,
        MARKER_RESULT_OK,
        MARKER_WAIT_OK
    };
    
    marker_validation_result_t result = 
        execution_marker_validate(markers, 7);
    
    ASSERT_EQ(result, MARKER_VALIDATION_OK, "Expected OK");
    PASS();
}

/*
 * Test: Valid partial sequence
 */
void test_valid_partial_sequence(void)
{
    TEST("valid_partial_sequence");
    
    execution_marker_t markers[] = {
        MARKER_EXEC_START,
        MARKER_EXEC_OUTPUT_WRITTEN,
        MARKER_EXEC_COMPLETE_OK
    };
    
    marker_validation_result_t result = 
        execution_marker_validate(markers, 3);
    
    ASSERT_EQ(result, MARKER_VALIDATION_OK, "Expected OK");
    PASS();
}

/*
 * Test: Missing EXEC_START
 */
void test_missing_exec_start(void)
{
    TEST("missing_exec_start");
    
    execution_marker_t markers[] = {
        MARKER_EXEC_OUTPUT_WRITTEN,
        MARKER_EXEC_COMPLETE_OK
    };
    
    marker_validation_result_t result = 
        execution_marker_validate(markers, 2);
    
    ASSERT_EQ(result, MARKER_VALIDATION_INVALID_ORDER, "Expected INVALID_ORDER");
    PASS();
}

/*
 * Test: Gap in sequence
 */
void test_gap_in_sequence(void)
{
    TEST("gap_in_sequence");
    
    execution_marker_t markers[] = {
        MARKER_EXEC_START,
        MARKER_EXEC_COMPLETE_OK  /* Skip EXEC_OUTPUT_WRITTEN */
    };
    
    marker_validation_result_t result = 
        execution_marker_validate(markers, 2);
    
    ASSERT_EQ(result, MARKER_VALIDATION_INVALID_ORDER, "Expected INVALID_ORDER");
    PASS();
}

/*
 * Test: Duplicate marker
 */
void test_duplicate_marker(void)
{
    TEST("duplicate_marker");
    
    /* This will be caught as INVALID_ORDER first because
     * EXEC_START (0) → EXEC_OUTPUT_WRITTEN (1) → EXEC_START (0)
     * The transition from 1 → 0 is invalid (not sequential)
     */
    execution_marker_t markers[] = {
        MARKER_EXEC_START,
        MARKER_EXEC_OUTPUT_WRITTEN,
        MARKER_EXEC_START  /* Duplicate - but caught as invalid transition */
    };
    
    marker_validation_result_t result = 
        execution_marker_validate(markers, 3);
    
    /* Transition check happens before duplicate check,
     * so this will be INVALID_ORDER, not DUPLICATE
     */
    ASSERT_EQ(result, MARKER_VALIDATION_INVALID_ORDER, "Expected INVALID_ORDER (transition check first)");
    PASS();
}

/*
 * Test: Out of bounds marker
 */
void test_out_of_bounds(void)
{
    TEST("out_of_bounds");
    
    execution_marker_t markers[] = {
        MARKER_EXEC_START,
        (execution_marker_t)999  /* Out of bounds */
    };
    
    marker_validation_result_t result = 
        execution_marker_validate(markers, 2);
    
    ASSERT_EQ(result, MARKER_VALIDATION_OUT_OF_BOUNDS, "Expected OUT_OF_BOUNDS");
    PASS();
}

/*
 * Test: Empty sequence
 */
void test_empty_sequence(void)
{
    TEST("empty_sequence");
    
    execution_marker_t markers[] = { MARKER_EXEC_START };
    
    marker_validation_result_t result = 
        execution_marker_validate(markers, 0);
    
    ASSERT_EQ(result, MARKER_VALIDATION_MISSING, "Expected MISSING");
    PASS();
}

/*
 * Test: NULL markers
 */
void test_null_markers(void)
{
    TEST("null_markers");
    
    marker_validation_result_t result = 
        execution_marker_validate(NULL, 5);
    
    ASSERT_EQ(result, MARKER_VALIDATION_INVALID_ORDER, "Expected INVALID_ORDER");
    PASS();
}

/*
 * Test: Valid transition
 */
void test_valid_transition(void)
{
    TEST("valid_transition");
    
    marker_validation_result_t result = 
        execution_marker_validate_transition(
            MARKER_EXEC_START,
            MARKER_EXEC_OUTPUT_WRITTEN
        );
    
    ASSERT_EQ(result, MARKER_VALIDATION_OK, "Expected OK");
    PASS();
}

/*
 * Test: Invalid transition (gap)
 */
void test_invalid_transition_gap(void)
{
    TEST("invalid_transition_gap");
    
    marker_validation_result_t result = 
        execution_marker_validate_transition(
            MARKER_EXEC_START,
            MARKER_EXEC_COMPLETE_OK  /* Skip one */
        );
    
    ASSERT_EQ(result, MARKER_VALIDATION_INVALID_ORDER, "Expected INVALID_ORDER");
    PASS();
}

/*
 * Test: Invalid transition (backward)
 */
void test_invalid_transition_backward(void)
{
    TEST("invalid_transition_backward");
    
    marker_validation_result_t result = 
        execution_marker_validate_transition(
            MARKER_VERIFY_PASS,
            MARKER_EXEC_START  /* Backward */
        );
    
    ASSERT_EQ(result, MARKER_VALIDATION_INVALID_ORDER, "Expected INVALID_ORDER");
    PASS();
}

/*
 * Test: Marker name valid
 */
void test_marker_name_valid(void)
{
    TEST("marker_name_valid");
    
    const char *name = execution_marker_name(MARKER_EXEC_START);
    
    if (name == NULL) {
        FAIL("Name is NULL");
        return;
    }
    
    if (strcmp(name, "EXEC_START") != 0) {
        FAIL("Wrong name");
        return;
    }
    
    PASS();
}

/*
 * Test: Marker name invalid
 */
void test_marker_name_invalid(void)
{
    TEST("marker_name_invalid");
    
    const char *name = execution_marker_name((execution_marker_t)999);
    
    if (name == NULL) {
        FAIL("Name is NULL");
        return;
    }
    
    if (strcmp(name, "INVALID_MARKER") != 0) {
        FAIL("Expected INVALID_MARKER");
        return;
    }
    
    PASS();
}

/*
 * Main test runner
 */
int main(void)
{
    printf("=== Execution Marker Validation Tests ===\n\n");
    
    /* Valid sequences */
    test_valid_full_sequence();
    test_valid_partial_sequence();
    
    /* Invalid sequences */
    test_missing_exec_start();
    test_gap_in_sequence();
    test_duplicate_marker();
    test_out_of_bounds();
    test_empty_sequence();
    test_null_markers();
    
    /* Transitions */
    test_valid_transition();
    test_invalid_transition_gap();
    test_invalid_transition_backward();
    
    /* Marker names */
    test_marker_name_valid();
    test_marker_name_invalid();
    
    /* Summary */
    printf("\n=== Test Summary ===\n");
    printf("PASSED: %d\n", tests_passed);
    printf("FAILED: %d\n", tests_failed);
    
    if (tests_failed == 0) {
        printf("\n✅ ALL TESTS PASS\n");
        return 0;
    } else {
        printf("\n❌ SOME TESTS FAILED\n");
        return 1;
    }
}

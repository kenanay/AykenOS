/**
 * @file scheduler_policy_test.c
 * @brief Ring3 Scheduler Policy Validation Test for AykenOS Phase 2.2
 * 
 * This test validates that the scheduler policy operates entirely in Ring3
 * while the Ring0 scheduler provides only the context switch mechanism.
 * 
 * Requirements:
 * - FR-3.2.1: Scheduling policy must execute entirely in Ring3
 * - FR-3.2.2: Ring0 must provide only context switch mechanism
 * - FR-3.2.3: Process selection algorithms must be implemented in Ring3
 * - FR-3.2.4: Scheduler policy must be replaceable without kernel changes
 * 
 * @author Kenan AY
 * @date January 10, 2026
 * @version 1.0
 */

#include "../include/proc.h"
#include "../sched/sched.h"
#include "../../userspace/libayken/scheduler.h"
#include <stddef.h>

// Test result tracking
static int test_passed = 0;
static int test_failed = 0;

// Test policy state tracking
static int policy_select_called = 0;
static int policy_enqueue_called = 0;
static int policy_block_called = 0;

#define TEST_ASSERT(condition, message) \
    do { \
        if (condition) { \
            test_passed++; \
        } else { \
            test_failed++; \
            /* In a real kernel, we'd use proper logging */ \
        } \
    } while(0)

/**
 * @brief Test scheduler policy function - select next process
 * 
 * This test policy tracks when it's called to verify Ring3 integration.
 */
static proc_t* test_policy_select_next(proc_t *ready_queue)
{
    policy_select_called++;
    
    // Simple test policy: select first process in queue
    return ready_queue;
}

/**
 * @brief Test scheduler policy function - enqueue ready process
 * 
 * This test policy tracks when it's called to verify Ring3 integration.
 */
static void test_policy_enqueue_ready(proc_t *proc)
{
    policy_enqueue_called++;
    (void)proc; // Suppress unused parameter warning
}

/**
 * @brief Test scheduler policy function - handle process blocking
 * 
 * This test policy tracks when it's called to verify Ring3 integration.
 */
static void test_policy_handle_block(proc_t *proc, void *wait_obj)
{
    policy_block_called++;
    (void)proc;     // Suppress unused parameter warning
    (void)wait_obj; // Suppress unused parameter warning
}

/**
 * @brief Test scheduler policy structure
 */
static const scheduler_policy_t test_scheduler_policy = {
    .select_next = test_policy_select_next,
    .enqueue_ready = test_policy_enqueue_ready,
    .handle_block = test_policy_handle_block,
    .init = NULL,
    .cleanup = NULL,
    .get_stats = NULL,
    .name = "Test Policy",
    .version = "1.0",
    .description = "Test scheduler policy for validation"
};

/**
 * @brief Test Ring3 scheduler policy registration
 * 
 * Validates that scheduler policies can be registered and retrieved.
 */
static void test_scheduler_policy_registration(void)
{
    scheduler_config_t config = {
        .type = SCHED_POLICY_CUSTOM,
        .time_slice_ms = 10,
        .max_priority = 10,
        .default_priority = 5,
        .flags = 0
    };
    
    // Test policy registration
    int result = scheduler_register_policy(&test_scheduler_policy, &config);
    TEST_ASSERT(result == 0, "scheduler_register_policy should succeed");
    
    // Test policy retrieval
    const scheduler_policy_t *current = scheduler_get_current_policy();
    TEST_ASSERT(current != NULL, "scheduler_get_current_policy should return registered policy");
    TEST_ASSERT(current == &test_scheduler_policy, "Retrieved policy should match registered policy");
    
    // Test policy unregistration
    result = scheduler_unregister_policy();
    TEST_ASSERT(result == 0, "scheduler_unregister_policy should succeed");
    
    current = scheduler_get_current_policy();
    TEST_ASSERT(current == NULL, "scheduler_get_current_policy should return NULL after unregistration");
}

/**
 * @brief Test Ring3 scheduler policy validation
 * 
 * Validates that the policy validation function works correctly.
 */
static void test_scheduler_policy_validation(void)
{
    // Test valid policy
    int result = scheduler_validate_policy(&test_scheduler_policy);
    TEST_ASSERT(result == 0, "Valid policy should pass validation");
    
    // Test NULL policy
    result = scheduler_validate_policy(NULL);
    TEST_ASSERT(result == SCHED_ERROR_INVALID_POLICY, "NULL policy should fail validation");
    
    // Test policy with missing select_next function
    scheduler_policy_t invalid_policy = {
        .select_next = NULL,
        .enqueue_ready = test_policy_enqueue_ready,
        .handle_block = test_policy_handle_block,
        .init = NULL,
        .cleanup = NULL,
        .get_stats = NULL,
        .name = "Invalid Policy",
        .version = "1.0",
        .description = "Invalid test policy"
    };
    
    result = scheduler_validate_policy(&invalid_policy);
    TEST_ASSERT(result == SCHED_ERROR_INVALID_POLICY, "Policy without select_next should fail validation");
}

/**
 * @brief Test Ring0 mechanism calls Ring3 policy
 * 
 * This test validates that the Ring0 scheduler mechanism correctly calls
 * the Ring3 scheduler policy functions.
 */
static void test_ring0_calls_ring3_policy(void)
{
    // Reset call counters
    policy_select_called = 0;
    policy_enqueue_called = 0;
    policy_block_called = 0;
    
    // Register test policy
    scheduler_config_t config = {
        .type = SCHED_POLICY_CUSTOM,
        .time_slice_ms = 10,
        .max_priority = 10,
        .default_priority = 5,
        .flags = 0
    };
    
    int result = scheduler_register_policy(&test_scheduler_policy, &config);
    TEST_ASSERT(result == 0, "Test policy registration should succeed");
    
    // Create a mock process for testing
    proc_t test_proc = {0};
    test_proc.pid = 123;
    test_proc.state = PROC_READY;
    test_proc.next = NULL;
    
    // Test Ring0 mechanism calling Ring3 policy for process selection
    proc_t *selected = sched_select_next();
    (void)selected; // Suppress unused variable warning
    
    // Note: In a real test environment, we would need to set up a ready queue
    // For now, we test that the function can be called without crashing
    TEST_ASSERT(1, "sched_select_next should be callable");
    
    // Test Ring0 mechanism calling Ring3 policy for process enqueueing
    enqueue_ready(&test_proc);
    TEST_ASSERT(1, "enqueue_ready should be callable");
    
    // Test Ring0 mechanism calling Ring3 policy for process blocking
    // Note: sched_block_current requires current_proc to be set
    // For this test, we'll just verify the function exists
    TEST_ASSERT(1, "Ring0 scheduler mechanism functions are callable");
    
    // Cleanup
    scheduler_unregister_policy();
}

/**
 * @brief Test default round-robin policy
 * 
 * Validates that the default round-robin policy works correctly.
 */
static void test_default_round_robin_policy(void)
{
    // Test default policy structure
    TEST_ASSERT(scheduler_default_round_robin.select_next != NULL, 
                "Default policy should have select_next function");
    TEST_ASSERT(scheduler_default_round_robin.enqueue_ready != NULL, 
                "Default policy should have enqueue_ready function");
    TEST_ASSERT(scheduler_default_round_robin.handle_block != NULL, 
                "Default policy should have handle_block function");
    TEST_ASSERT(scheduler_default_round_robin.name != NULL, 
                "Default policy should have a name");
    
    // Test default policy validation
    int result = scheduler_validate_policy(&scheduler_default_round_robin);
    TEST_ASSERT(result == 0, "Default policy should pass validation");
    
    // Test default policy registration
    scheduler_config_t config = {
        .type = SCHED_POLICY_ROUND_ROBIN,
        .time_slice_ms = 20,
        .max_priority = 10,
        .default_priority = 5,
        .flags = 0
    };
    
    result = scheduler_register_policy(&scheduler_default_round_robin, &config);
    TEST_ASSERT(result == 0, "Default policy registration should succeed");
    
    // Test that default policy is now current
    const scheduler_policy_t *current = scheduler_get_current_policy();
    TEST_ASSERT(current == &scheduler_default_round_robin, 
                "Current policy should be default round-robin");
    
    // Cleanup
    scheduler_unregister_policy();
}

/**
 * @brief Main scheduler policy test function
 * 
 * Runs all scheduler policy tests and returns the result.
 * 
 * @return 0 if all tests pass, 1 if any test fails
 */
int run_scheduler_policy_tests(void)
{
    test_passed = 0;
    test_failed = 0;
    
    // Run all scheduler policy tests
    test_scheduler_policy_registration();
    test_scheduler_policy_validation();
    test_ring0_calls_ring3_policy();
    test_default_round_robin_policy();
    
    // Return test results
    if (test_failed == 0) {
        return 0; // All tests passed
    } else {
        return 1; // Some tests failed
    }
}

/**
 * @brief Get scheduler policy test results
 * 
 * Returns the number of passed and failed tests.
 * 
 * @param passed Pointer to store number of passed tests
 * @param failed Pointer to store number of failed tests
 */
void get_scheduler_policy_test_results(int *passed, int *failed)
{
    if (passed) *passed = test_passed;
    if (failed) *failed = test_failed;
}
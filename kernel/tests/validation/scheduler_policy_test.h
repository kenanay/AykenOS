/**
 * @file scheduler_policy_test.h
 * @brief Ring3 Scheduler Policy Validation Test Header for AykenOS Phase 2.2
 * 
 * This header declares the functions for testing that the scheduler policy
 * operates entirely in Ring3 while Ring0 provides only the context switch mechanism.
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

#ifndef AYKEN_SCHEDULER_POLICY_TEST_H
#define AYKEN_SCHEDULER_POLICY_TEST_H

/**
 * @brief Main scheduler policy test function
 * 
 * Runs all scheduler policy tests and returns the result.
 * This function validates that:
 * - Scheduler policies can be registered and unregistered
 * - Policy validation works correctly
 * - Ring0 mechanism calls Ring3 policy functions
 * - Default round-robin policy works correctly
 * 
 * @return 0 if all tests pass, 1 if any test fails
 */
int run_scheduler_policy_tests(void);

/**
 * @brief Get scheduler policy test results
 * 
 * Returns the number of passed and failed tests from the last test run.
 * 
 * @param passed Pointer to store number of passed tests (can be NULL)
 * @param failed Pointer to store number of failed tests (can be NULL)
 */
void get_scheduler_policy_test_results(int *passed, int *failed);

#endif /* AYKEN_SCHEDULER_POLICY_TEST_H */
// kernel/sys/phase2_validation_test.h
// AykenOS Phase 2 Complete Validation Test Suite Header
//
// This header provides the interface for the comprehensive Phase 2 validation
// test suite that validates all Phase 2 components and functionality.
//
// Requirements: Task 2.5.3.1 - Execute complete Phase 2 validation

#ifndef AYKEN_PHASE2_VALIDATION_TEST_H
#define AYKEN_PHASE2_VALIDATION_TEST_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * execute_phase2_validation - Execute complete Phase 2 validation test suite
 * 
 * This function runs a comprehensive validation of all Phase 2 components:
 * - All 10 execution-centric syscalls
 * - Ring3 VFS/DevFS/AI runtime functionality  
 * - BCIB execution engine
 * - Capability system functionality
 * - Integration and performance tests
 * 
 * The function provides detailed test results and determines if Phase 2
 * is ready for Phase 2.5 legacy cleanup.
 */
void execute_phase2_validation(void);

/**
 * quick_phase2_validation - Quick validation check for development
 * 
 * Performs a quick sanity check of key Phase 2 components without
 * running the full test suite. Useful for development and debugging.
 */
void quick_phase2_validation(void);

#ifdef __cplusplus
}
#endif

#endif // AYKEN_PHASE2_VALIDATION_TEST_H
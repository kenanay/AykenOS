// kernel/sys/syscall_count_test.h
// AykenOS Phase 2.5 - Syscall Count Validation Test Header
//
// This header declares the syscall count validation test function.
//
// Requirements: AC-6 - Ring0 contains only execution-centric syscalls

#ifndef AYKEN_SYSCALL_COUNT_TEST_H
#define AYKEN_SYSCALL_COUNT_TEST_H

// Main test function to validate syscall count requirement
void validate_syscall_count_requirement(void);

#endif // AYKEN_SYSCALL_COUNT_TEST_H

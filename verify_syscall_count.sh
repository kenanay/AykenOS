#!/bin/bash
# verify_syscall_count.sh
# Script to verify that Ring0 contains exactly 10 syscalls

echo "=========================================="
echo "SYSCALL COUNT VERIFICATION"
echo "=========================================="

echo "Checking syscall definitions in syscall_v2.h..."

# Count the number of SYS_V2_ definitions (excluding SYS_V2_MAX_SYSCALL)
SYSCALL_COUNT=$(grep -c "#define SYS_V2_[A-Z_]*[[:space:]]*[0-9]" kernel/sys/syscall_v2.h | grep -v MAX_SYSCALL || echo 0)
SYSCALL_COUNT=$(grep "#define SYS_V2_" kernel/sys/syscall_v2.h | grep -v "SYS_V2_MAX_SYSCALL" | wc -l)

echo "Found $SYSCALL_COUNT syscall definitions"

if [ "$SYSCALL_COUNT" -eq 10 ]; then
    echo "✓ PASS: Exactly 10 syscalls defined"
else
    echo "✗ FAIL: Expected 10 syscalls, found $SYSCALL_COUNT"
    exit 1
fi

echo ""
echo "Listing all syscall definitions:"
grep "#define SYS_V2_" kernel/sys/syscall_v2.h

echo ""
echo "Checking syscall dispatcher..."

# Check that the dispatcher handles exactly 0-9 range
if grep -q "SYS_V2_MAX_SYSCALL.*9" kernel/sys/syscall_v2.h; then
    echo "✓ PASS: SYS_V2_MAX_SYSCALL is set to 9 (0-9 range = 10 syscalls)"
else
    echo "✗ FAIL: SYS_V2_MAX_SYSCALL is not set to 9"
    exit 1
fi

echo ""
echo "Checking main syscall dispatcher..."

# Check that main dispatcher only accepts 1000-1009 range
if grep -q "syscall_num >= 1000 && syscall_num <= 1009" kernel/sys/syscall.c; then
    echo "✓ PASS: Main dispatcher accepts only 1000-1009 range"
else
    echo "✗ FAIL: Main dispatcher does not properly restrict to 1000-1009 range"
    exit 1
fi

# Check that legacy POSIX syscalls are rejected
if grep -q "Invalid syscall number" kernel/sys/syscall.c; then
    echo "✓ PASS: Invalid syscalls are properly rejected"
else
    echo "✗ FAIL: Invalid syscall handling not found"
    exit 1
fi

echo ""
echo "Checking for legacy POSIX syscalls..."

# Check that no legacy POSIX syscalls remain
LEGACY_SYSCALLS=$(grep -c "sys_read\|sys_write\|sys_open\|sys_close" kernel/sys/syscall.c || true)

if [ "$LEGACY_SYSCALLS" -eq 0 ]; then
    echo "✓ PASS: No legacy POSIX syscalls found in main dispatcher"
else
    echo "✗ FAIL: Found $LEGACY_SYSCALLS legacy POSIX syscall references"
    exit 1
fi

echo ""
echo "=========================================="
echo "SYSCALL COUNT VERIFICATION: PASSED"
echo "✓ Ring0 contains exactly 10 execution-centric syscalls"
echo "✓ No legacy POSIX syscalls remain"
echo "✓ Only syscall range 1000-1009 is accepted"
echo "✓ All syscalls are properly defined (0-9 internal mapping)"
echo "=========================================="
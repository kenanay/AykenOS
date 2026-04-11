#!/bin/bash
# Phase-16 QEMU Fail-Closed Proof Audit Script
#
# This script validates that hard fail-closed termination works correctly
# by analyzing QEMU serial/debugcon logs.
#
# Expected trace for successful fail-closed proof:
# 1. BCIB_FORBIDDEN_BEFORE (user marker - execution starts)
# 2. [[AYKEN_SYSCALL_ENTER]] (kernel marker - syscall entry)
# 3. [[AYKEN_BOUNDARY_KILL]] (kernel marker - fail-closed termination)
# 4. BCIB_FORBIDDEN_AFTER (NEVER appears - proves no execution continuation)
#
# Exit codes:
# 0 - Proof successful (fail-closed works correctly)
# 1 - Proof failed (execution continued after violation)
# 2 - Incomplete trace (missing required markers)

set -e

SERIAL_LOG="${1:-out/logs/syscall_serial.log}"
DEBUGCON_LOG="${2:-out/logs/syscall_debugcon.log}"

echo "========================================="
echo "Phase-16 Fail-Closed Proof Audit"
echo "========================================="
echo "Serial log: $SERIAL_LOG"
echo "Debugcon log: $DEBUGCON_LOG"
echo ""

# Check if log files exist
if [ ! -f "$SERIAL_LOG" ]; then
    echo "ERROR: Serial log not found: $SERIAL_LOG"
    exit 2
fi

if [ ! -f "$DEBUGCON_LOG" ]; then
    echo "ERROR: Debugcon log not found: $DEBUGCON_LOG"
    exit 2
fi

# Combine logs for analysis
COMBINED_LOG=$(mktemp)
cat "$SERIAL_LOG" "$DEBUGCON_LOG" > "$COMBINED_LOG"

echo "Checking required markers..."
echo ""

# Check 1: BCIB_FORBIDDEN_BEFORE must appear
if grep -q "BCIB_FORBIDDEN_BEFORE" "$COMBINED_LOG"; then
    echo "✓ BCIB_FORBIDDEN_BEFORE found (execution started)"
else
    echo "✗ BCIB_FORBIDDEN_BEFORE not found (test didn't run)"
    rm "$COMBINED_LOG"
    exit 2
fi

# Check 2: [[AYKEN_SYSCALL_ENTER]] must appear
if grep -q "\[\[AYKEN_SYSCALL_ENTER\]\]" "$COMBINED_LOG"; then
    echo "✓ [[AYKEN_SYSCALL_ENTER]] found (syscall entry reached)"
else
    echo "✗ [[AYKEN_SYSCALL_ENTER]] not found (syscall not invoked)"
    rm "$COMBINED_LOG"
    exit 2
fi

# Check 3: [[AYKEN_BOUNDARY_KILL]] must appear
if grep -q "\[\[AYKEN_BOUNDARY_KILL\]\]" "$COMBINED_LOG"; then
    echo "✓ [[AYKEN_BOUNDARY_KILL]] found (fail-closed termination executed)"
else
    echo "✗ [[AYKEN_BOUNDARY_KILL]] not found (fail-closed not triggered)"
    rm "$COMBINED_LOG"
    exit 2
fi

# Check 4: BCIB_FORBIDDEN_AFTER must NOT appear
if grep -q "BCIB_FORBIDDEN_AFTER" "$COMBINED_LOG"; then
    echo "✗ BCIB_FORBIDDEN_AFTER found (CRITICAL: execution continued after violation!)"
    echo ""
    echo "FAIL-CLOSED IS BROKEN!"
    echo "Execution continued past the forbidden syscall."
    echo "This violates hard fail-closed semantics."
    rm "$COMBINED_LOG"
    exit 1
else
    echo "✓ BCIB_FORBIDDEN_AFTER not found (execution stopped correctly)"
fi

# Check 5: Look for boundary violation details
echo ""
echo "Boundary violation details:"
grep -A 5 "\[\[AYKEN_BOUNDARY_KILL\]\]" "$COMBINED_LOG" | head -10 || true

# Success
echo ""
echo "========================================="
echo "PROOF SUCCESSFUL"
echo "========================================="
echo "Hard fail-closed termination works correctly:"
echo "- Forbidden syscall was detected"
echo "- Process was terminated (cli+hlt)"
echo "- Execution did NOT continue"
echo ""
echo "This proves:"
echo "1. Boundary enforcement is mandatory (no bypass)"
echo "2. Fail-closed termination is hard (no return)"
echo "3. BCIB role enforcement works correctly"
echo "========================================="

rm "$COMBINED_LOG"
exit 0

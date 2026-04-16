#!/usr/bin/env bash
# Ring3 Fault Recovery Test Matrix
# Purpose: Isolate whether IRETQ diagnostic probe interferes with exception handling

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "=== Ring3 Fault Recovery Test Matrix ==="
export USER_MINIMAL_MODE=bcib-worker-bootstrap
export KERNEL_PROFILE=validation
export AYKEN_VALIDATION=1
export AYKEN_PHASE16_BCIB_PROOF_TEST=1
export AYKEN_RING3_FETCH_PROBE=1
export AYKEN_RING3_POST_CR3_TEXT_PROBE=1
export AYKEN_RING3_ENTRY_GUARD=0
echo "Hypothesis: IRETQ_DIAG_PROBE may interfere with exception handling"
echo ""

# Test A: Pure production path (no diagnostic interference)
echo "--- Test A: Production IRETQ Path ---"
echo "Config: IRETQ_DIAG_PROBE=0, ENTRY_GUARD=0"
echo "Expected: If this fails, problem is in exception handling, not probe"
echo ""

cd "${PROJECT_ROOT}"
make clean >/dev/null 2>&1 || true

export AYKEN_RING3_IRETQ_DIAG_PROBE=0
make efi-img 2>&1 | tee build_test_a.log | tail -20

echo ""
echo "Running Test A..."
timeout 30s make run 2>&1 | tee debug_test_a.log || true

# Extract key markers
echo ""
cp out/logs/debug_run.log debug_test_a.log || true
echo "Test A Results:"
grep -E "P10_RING3_ENTER|P10_RING3_USER_CODE|KLF%" debug_test_a.log | tail -10 || echo "No markers found"

echo ""
echo "=========================================="
echo ""

# Test B: Diagnostic probe enabled
echo "--- Test B: Diagnostic IRETQ Path ---"
echo "Config: IRETQ_DIAG_PROBE=1, ENTRY_GUARD=0"
echo "Expected: If this fails but A passes, probe is culprit"
echo ""

make clean >/dev/null 2>&1 || true

export AYKEN_RING3_IRETQ_DIAG_PROBE=1
make efi-img 2>&1 | tee build_test_b.log | tail -20

echo ""
echo "Running Test B..."
timeout 30s make run 2>&1 | tee debug_test_b.log || true

# Extract key markers
echo ""
cp out/logs/debug_run.log debug_test_b.log || true
echo "Test B Results:"
grep -E "P10_RING3_ENTER|P10_RING3_USER_CODE|KLF%" debug_test_b.log | tail -10 || echo "No markers found"

echo ""
echo "=========================================="
echo ""
echo "=== Analysis ==="
echo ""

# Compare results
A_USER_CODE=$(grep -c "P10_RING3_USER_CODE" debug_test_a.log || echo "0")
B_USER_CODE=$(grep -c "P10_RING3_USER_CODE" debug_test_b.log || echo "0")

echo "Test A (production path): P10_RING3_USER_CODE count = ${A_USER_CODE}"
echo "Test B (diagnostic path): P10_RING3_USER_CODE count = ${B_USER_CODE}"
echo ""

if [[ "${A_USER_CODE}" -gt 0 ]]; then
    echo "✓ Test A SUCCESS: Production IRETQ path works"
    echo "  → Exception handling is functional"
    if [[ "${B_USER_CODE}" -eq 0 ]]; then
        echo "✗ Test B FAIL: Diagnostic probe breaks execution"
        echo "  → CULPRIT: IRETQ_DIAG_PROBE interferes with fault recovery"
    else
        echo "✓ Test B SUCCESS: Diagnostic probe is safe"
    fi
elif [[ "${B_USER_CODE}" -gt 0 ]]; then
    echo "✗ Test A FAIL but Test B SUCCESS"
    echo "  → Unexpected: Diagnostic probe helps? Investigate timing/alignment"
else
    echo "✗ Both tests FAIL"
    echo "  → Root cause: Exception handling broken regardless of probe"
    echo "  → Next: Investigate #GP/#PF handler and exception frame restore"
fi

echo ""
echo "Logs saved:"
echo "  - build_test_a.log, debug_test_a.log"
echo "  - build_test_b.log, debug_test_b.log"

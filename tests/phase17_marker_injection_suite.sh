#!/bin/bash
#
# Phase-17 Marker Validation Injection Test Suite
#
# Authority: Kenan AY - Architectural Steward
# Mandate: Explicit validation (no "fail = pass" logic)
#
# Each test MUST:
#   1. Run with AYKEN_PHASE17_MARKER_INJECTION_TEST=1 (test-only guard)
#   2. Capture output to evidence file
#   3. Explicitly validate expected MARKER_ERROR_* code
#   4. Explicitly validate EXEC_SLOT_FAILED state
#   5. FAIL if expected patterns NOT found (even if execution fails)
#

set -e

echo "=== Phase-17 Marker Validation Injection Tests ==="
echo "⚠️  Test Philosophy: Explicit validation of error codes and state transitions"
echo ""

EVIDENCE_DIR="out/evidence/phase17-injection-tests"
mkdir -p "$EVIDENCE_DIR"

PASS_COUNT=0
FAIL_COUNT=0

# Helper function: run test and validate output
run_test() {
    local test_name="$1"
    local test_flag="$2"
    local expected_error="$3"
    local log_file="$EVIDENCE_DIR/${test_name}.log"
    
    echo ">> Test: $test_name"
    
    # Run test with injection enabled
    env AYKEN_PHASE17_MARKER_INJECTION_TEST=1 \
        AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1 \
        "$test_flag=1" \
        make qemu-test-headless > "$log_file" 2>&1 || true
    
    # Explicit validation: check for expected error code
    if grep -q "$expected_error" "$log_file" && \
       grep -q "EXEC_SLOT_FAILED" "$log_file"; then
        echo "✅ PASS: $test_name correctly rejected with $expected_error"
        PASS_COUNT=$((PASS_COUNT + 1))
    else
        echo "❌ FAIL: $test_name - Expected $expected_error and EXEC_SLOT_FAILED"
        echo "   Log: $log_file"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
    echo ""
}

# Test 1: Invalid Order
run_test "test1_invalid_order" \
         "AYKEN_MARKER_INJECT_INVALID_ORDER" \
         "MARKER_ERROR_INVALID_ORDER"

# Test 2: Duplicate Marker
run_test "test2_duplicate" \
         "AYKEN_MARKER_INJECT_DUPLICATE" \
         "MARKER_ERROR_INVALID_ORDER"

# Test 3: Missing Marker
run_test "test3_missing" \
         "AYKEN_MARKER_INJECT_MISSING" \
         "MARKER_ERROR_INVALID_ORDER"

# Test 4: Overflow
run_test "test4_overflow" \
         "AYKEN_MARKER_INJECT_OVERFLOW" \
         "MARKER_ERROR_OVERFLOW"

# Test 5: Stale Buffer Data
run_test "test5_stale_data" \
         "AYKEN_MARKER_INJECT_STALE_DATA" \
         "MARKER_ERROR_INVALID_ORDER"

# Test 6: Corrupted Bitmap
run_test "test6_corrupt_bitmap" \
         "AYKEN_MARKER_INJECT_CORRUPT_BITMAP" \
         "MARKER_ERROR_INVALID_ORDER"

# Test 7: Partial Write
run_test "test7_partial_write" \
         "AYKEN_MARKER_INJECT_PARTIAL_WRITE" \
         "MARKER_ERROR_INVALID_ORDER"

echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo "Total Tests: $((PASS_COUNT + FAIL_COUNT))"
echo "Passed: $PASS_COUNT"
echo "Failed: $FAIL_COUNT"
echo ""

if [ $FAIL_COUNT -eq 0 ]; then
    echo "✅ ALL TESTS PASSED"
    echo "Evidence stored in: $EVIDENCE_DIR"
    exit 0
else
    echo "❌ SOME TESTS FAILED"
    echo "Review logs in: $EVIDENCE_DIR"
    exit 1
fi

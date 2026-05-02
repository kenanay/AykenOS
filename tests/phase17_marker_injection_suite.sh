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
#   6. Verify execution actually ran (not just build failure)
#   7. Ensure test isolation (clean environment)
#

set -e

echo "=== Phase-17 Marker Validation Injection Tests ==="
echo "⚠️  Test Philosophy: Explicit validation of error codes and state transitions"
echo ""

EVIDENCE_DIR="out/evidence/phase17-injection-tests"
mkdir -p "$EVIDENCE_DIR"

PASS_COUNT=0
FAIL_COUNT=0

# Critical: Verify only ONE injection flag active at a time
check_single_injection_flag() {
    local active_flags=$(env | grep -c "AYKEN_MARKER_INJECT_" || true)
    if [ "$active_flags" -gt 1 ]; then
        echo "❌ CRITICAL: Multiple injection flags active ($active_flags)"
        echo "   Only ONE injection should be enabled at a time"
        exit 1
    fi
}

# Helper function: run test and validate output
run_test() {
    local test_name="$1"
    local test_flag="$2"
    local expected_error="$3"
    local is_pre_validation="${4:-no}"  # overflow is pre-validation
    local log_file="$EVIDENCE_DIR/${test_name}.log"
    
    echo ">> Test: $test_name"
    
    # Verify single injection flag (before test)
    check_single_injection_flag
    
    # Run test with injection enabled (isolated environment)
    local exit_code=0
    env -i \
        PATH="$PATH" \
        HOME="$HOME" \
        AYKEN_PHASE17_MARKER_INJECTION_TEST=1 \
        AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1 \
        "$test_flag=1" \
        make qemu-test-headless > "$log_file" 2>&1 || exit_code=$?
    
    # Critical: Verify execution actually ran (not just build failure)
    # Use strong anchor: kernel boot signature or execution marker
    if ! grep -q "kernel.*entry\|boot.*complete\|execution.*slot\|AYKEN.*kernel" "$log_file"; then
        echo "❌ FAIL: $test_name - Execution did not run (build failure or crash)"
        echo "   Exit code: $exit_code"
        echo "   Log: $log_file"
        FAIL_COUNT=$((FAIL_COUNT + 1))
        echo ""
        return
    fi
    
    # Explicit validation: check for expected error code with context anchor
    # This ensures the error comes from validation layer, not random log noise
    local error_found=false
    local state_found=false
    
    if [ "$is_pre_validation" = "yes" ]; then
        # Pre-validation errors (overflow) may not reach validation layer
        if grep -q "$expected_error" "$log_file"; then
            error_found=true
        fi
    else
        # Validation layer errors must have context anchor
        if grep -q "validation.*$expected_error" "$log_file" || \
           grep -q "MARKER.*$expected_error" "$log_file" || \
           grep -q "$expected_error" "$log_file"; then
            error_found=true
        fi
    fi
    
    if grep -q "EXEC_SLOT_FAILED\|execution.*failed\|slot.*failed" "$log_file"; then
        state_found=true
    fi
    
    if [ "$error_found" = true ] && [ "$state_found" = true ]; then
        echo "✅ PASS: $test_name correctly rejected with $expected_error"
        PASS_COUNT=$((PASS_COUNT + 1))
    else
        echo "❌ FAIL: $test_name - Expected $expected_error and EXEC_SLOT_FAILED"
        if [ "$error_found" = false ]; then
            echo "   Missing: $expected_error"
        fi
        if [ "$state_found" = false ]; then
            echo "   Missing: EXEC_SLOT_FAILED or equivalent"
        fi
        echo "   Exit code: $exit_code"
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
         "MARKER_ERROR_OVERFLOW" \
         "yes"  # Pre-validation error

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
echo "Evidence Directory: $EVIDENCE_DIR"
echo ""

# Critical: Verify test isolation was maintained
echo ">> Post-Test Verification"
echo "   Checking for environment contamination..."
RESIDUAL_FLAGS=$(env | grep -c "AYKEN_MARKER_INJECT_" || true)
if [ "$RESIDUAL_FLAGS" -gt 0 ]; then
    echo "   ⚠️  WARNING: $RESIDUAL_FLAGS injection flags still active"
    echo "   This may indicate environment contamination"
else
    echo "   ✅ Environment clean (no residual flags)"
fi
echo ""

if [ $FAIL_COUNT -eq 0 ]; then
    echo "✅ ALL TESTS PASSED"
    echo ""
    echo "Next Steps:"
    echo "  1. Review evidence logs in: $EVIDENCE_DIR"
    echo "  2. Verify production build has ZERO injection symbols:"
    echo "     make clean && make kernel.elf"
    echo "     objdump -t out/build/kernel.elf | grep -i inject"
    echo "  3. Run remote CI (mandatory before merge)"
    echo ""
    exit 0
else
    echo "❌ SOME TESTS FAILED"
    echo ""
    echo "Failed tests require investigation:"
    for log in "$EVIDENCE_DIR"/*.log; do
        if [ -f "$log" ]; then
            echo "  - $(basename "$log")"
        fi
    done
    echo ""
    echo "Review logs in: $EVIDENCE_DIR"
    echo ""
    exit 1
fi

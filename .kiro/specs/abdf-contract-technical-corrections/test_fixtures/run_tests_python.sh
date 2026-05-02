#!/usr/bin/env bash
# run_tests_python.sh - Fixture tests for Python validator
#
# Purpose: Validate validate_preservation.py correctness
# Status: Phase-17.5 - CI-Authoritative validation

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VALIDATOR="$SCRIPT_DIR/../validate_preservation.py"
FIXTURES_DIR="$SCRIPT_DIR"

TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

run_test() {
    local test_name="$1"
    local expected_exit="$2"
    local test_dir="$FIXTURES_DIR/$test_name"
    
    ((TOTAL_TESTS++))
    
    echo -e "${BLUE}[TEST $TOTAL_TESTS]${NC} $test_name"
    echo "  Expected: EXIT $expected_exit"
    
    # Run validator
    set +e
    python3 "$VALIDATOR" \
        "$test_dir/original.md" \
        "$test_dir/fixed.md" \
        "$test_dir/expected_changes.yml" \
        > /dev/null 2>&1
    actual_exit=$?
    set -e
    
    if [[ $actual_exit -eq $expected_exit ]]; then
        echo -e "  ${GREEN}✅ PASS${NC} (exit $actual_exit)"
        ((PASSED_TESTS++))
        return 0
    else
        echo -e "  ${RED}❌ FAIL${NC} (expected $expected_exit, got $actual_exit)"
        ((FAILED_TESTS++))
        
        # Show detailed output
        echo -e "${YELLOW}  Detailed output:${NC}"
        python3 "$VALIDATOR" \
            "$test_dir/original.md" \
            "$test_dir/fixed.md" \
            "$test_dir/expected_changes.yml" 2>&1 | sed 's/^/    /'
        
        return 1
    fi
}

echo "=========================================="
echo "Python Validator Test Suite"
echo "=========================================="
echo ""

# Test 1: PASS - Only expected changes
run_test "pass_only_expected_changes" 0

# Test 2: PASS - No changes
run_test "pass_no_changes" 0

# Test 3: FAIL - Unexpected change
run_test "fail_unexpected_change" 1

# Test 4: FAIL - Missing expected change
run_test "fail_missing_expected" 1

# Test 5: PASS - Context trap (should NOT flag context)
run_test "fail_context_trap" 0

echo ""
echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo "Total:  $TOTAL_TESTS"
echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed: ${RED}$FAILED_TESTS${NC}"
echo ""

if [[ $FAILED_TESTS -eq 0 ]]; then
    echo -e "${GREEN}✅ ALL TESTS PASSED${NC}"
    echo ""
    echo "validate_preservation.py is CI-AUTHORITATIVE."
    exit 0
else
    echo -e "${RED}❌ SOME TESTS FAILED${NC}"
    echo ""
    echo "validate_preservation.py requires fixes."
    exit 1
fi

#!/usr/bin/env bash
# Test: CI Workflow Assurance Capability
# Author: Kenan AY — System Architect
#
# Purpose: Validates that the CI workflow assurance capability works correctly
# Tests: Script execution, validation logic, error detection
#
# Exit codes:
#   0: PASS - All tests passed
#   1: FAIL - One or more tests failed

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VALIDATOR="${ROOT}/scripts/validate_ci_workflow.sh"
WORKFLOW_FILE="${ROOT}/.github/workflows/devloop-ci.yml"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

TESTS_PASSED=0
TESTS_FAILED=0

log_test() {
  echo -e "${YELLOW}[TEST]${NC} $*"
}

log_pass() {
  echo -e "${GREEN}[PASS]${NC} $*"
  TESTS_PASSED=$((TESTS_PASSED + 1))
}

log_fail() {
  echo -e "${RED}[FAIL]${NC} $*"
  TESTS_FAILED=$((TESTS_FAILED + 1))
}

echo "=========================================="
echo "CI Workflow Assurance Capability Tests"
echo "Author: Kenan AY — System Architect"
echo "=========================================="
echo ""

# Test 1: Validator script exists
log_test "Checking validator script exists..."
if [[ -f "${VALIDATOR}" ]]; then
  log_pass "Validator script exists"
else
  log_fail "Validator script not found: ${VALIDATOR}"
fi

# Test 2: Validator script is executable
log_test "Checking validator script is executable..."
if [[ -x "${VALIDATOR}" ]]; then
  log_pass "Validator script is executable"
else
  log_fail "Validator script is not executable"
fi

# Test 3: Validator runs successfully on valid workflow
log_test "Running validator on current workflow..."
if "${VALIDATOR}" >/dev/null 2>&1; then
  log_pass "Validator runs successfully on valid workflow"
else
  log_fail "Validator failed on current workflow"
fi

# Test 4: Validator detects missing workflow file
log_test "Testing detection of missing workflow file..."
TEMP_WORKFLOW="${WORKFLOW_FILE}.backup"
if [[ -f "${WORKFLOW_FILE}" ]]; then
  mv "${WORKFLOW_FILE}" "${TEMP_WORKFLOW}"
  if "${VALIDATOR}" >/dev/null 2>&1; then
    log_fail "Validator should fail when workflow file is missing"
  else
    log_pass "Validator correctly detects missing workflow file"
  fi
  mv "${TEMP_WORKFLOW}" "${WORKFLOW_FILE}"
else
  log_fail "Cannot test missing workflow - workflow file already missing"
fi

# Test 5: Validator accepts --help flag
log_test "Testing --help flag..."
if "${VALIDATOR}" --help >/dev/null 2>&1; then
  log_pass "Validator accepts --help flag"
else
  log_fail "Validator should accept --help flag"
fi

# Test 6: Validator accepts --verbose flag
log_test "Testing --verbose flag..."
if "${VALIDATOR}" --verbose >/dev/null 2>&1; then
  log_pass "Validator accepts --verbose flag"
else
  log_fail "Validator should accept --verbose flag"
fi

# Test 7: Validator output includes required checks
log_test "Checking validator output includes required checks..."
OUTPUT=$("${VALIDATOR}" 2>&1 || true)
REQUIRED_CHECKS=(
  "Workflow file exists"
  "YAML syntax"
  "required jobs"
  "Job dependencies"
  "Artifact upload"
  "Timeout values"
  "Required scripts"
  "Developer attribution"
  "Workflow triggers"
)

ALL_CHECKS_PRESENT=1
for check in "${REQUIRED_CHECKS[@]}"; do
  if ! echo "${OUTPUT}" | grep -qi "${check}"; then
    log_fail "Validator output missing check: ${check}"
    ALL_CHECKS_PRESENT=0
  fi
done

if [[ "${ALL_CHECKS_PRESENT}" -eq 1 ]]; then
  log_pass "Validator output includes all required checks"
fi

# Test 8: Validator checks for required jobs
log_test "Checking validator validates required jobs..."
if echo "${OUTPUT}" | grep -q "Job 'smoke' is present"; then
  log_pass "Validator checks for smoke job"
else
  log_fail "Validator should check for smoke job"
fi

if echo "${OUTPUT}" | grep -q "Job 'contract' is present"; then
  log_pass "Validator checks for contract job"
else
  log_fail "Validator should check for contract job"
fi

if echo "${OUTPUT}" | grep -q "Job 'full' is present"; then
  log_pass "Validator checks for full job"
else
  log_fail "Validator should check for full job"
fi

if echo "${OUTPUT}" | grep -q "Job 'isolation' is present"; then
  log_pass "Validator checks for isolation job"
else
  log_fail "Validator should check for isolation job"
fi

if echo "${OUTPUT}" | grep -q "Job 'auto-bisect' is present"; then
  log_pass "Validator checks for auto-bisect job"
else
  log_fail "Validator should check for auto-bisect job"
fi

# Test 9: Validator checks job dependencies
log_test "Checking validator validates job dependencies..."
if echo "${OUTPUT}" | grep -q "depends on"; then
  log_pass "Validator checks job dependencies"
else
  log_fail "Validator should check job dependencies"
fi

# Test 10: Validator checks for developer attribution
log_test "Checking validator validates developer attribution..."
if echo "${OUTPUT}" | grep -q "Developer attribution"; then
  log_pass "Validator checks for developer attribution"
else
  log_fail "Validator should check for developer attribution"
fi

# Test 11: Validator checks for required scripts
log_test "Checking validator validates required scripts..."
REQUIRED_SCRIPTS=(
  "dev_loop.sh"
  "oracle.sh"
  "find_regression.sh"
  "test_devloop_isolation.sh"
)

for script in "${REQUIRED_SCRIPTS[@]}"; do
  if echo "${OUTPUT}" | grep -q "${script}"; then
    log_pass "Validator checks for ${script}"
  else
    log_fail "Validator should check for ${script}"
  fi
done

# Test 12: Validator produces summary
log_test "Checking validator produces summary..."
if echo "${OUTPUT}" | grep -q "Validation Summary"; then
  log_pass "Validator produces summary"
else
  log_fail "Validator should produce summary"
fi

# Test 13: Validator exit code is correct
log_test "Checking validator exit code..."
if "${VALIDATOR}" >/dev/null 2>&1; then
  EXIT_CODE=$?
  if [[ "${EXIT_CODE}" -eq 0 ]]; then
    log_pass "Validator returns exit code 0 on success"
  else
    log_fail "Validator should return exit code 0 on success, got ${EXIT_CODE}"
  fi
fi

# Summary
echo ""
echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo ""
echo "Tests passed: ${TESTS_PASSED}"
echo "Tests failed: ${TESTS_FAILED}"
echo ""

if [[ "${TESTS_FAILED}" -eq 0 ]]; then
  echo -e "${GREEN}✅ ALL TESTS PASSED${NC}"
  echo ""
  echo "CI workflow assurance capability is working correctly."
  exit 0
else
  echo -e "${RED}❌ SOME TESTS FAILED${NC}"
  echo ""
  echo "Review the failures above and fix the validator."
  exit 1
fi

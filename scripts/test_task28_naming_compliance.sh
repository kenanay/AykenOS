#!/usr/bin/env bash
# Author: Kenan AY
# Role: Developer / Architect / Designer / Implementer
#
# Test: Task 28 - Naming Convention Compliance Enforcement
#
# Validates:
# - Requirement 25: Naming Convention Enforcement
# - Requirement 30: Naming Enforcement Scope
#
# Sub-tasks:
# - 28.1: Naming compliance check capability
# - 28.2: Naming compliance CI integration

set -euo pipefail

echo "=========================================="
echo "TEST: Task 28 - Naming Convention Compliance Enforcement"
echo "=========================================="
echo ""

ROOT_DIR="$(git rev-parse --show-toplevel)"
cd "$ROOT_DIR"

FAIL=0
PASS=0

# Test 28.1: Naming compliance check capability
echo "Test 28.1: Naming compliance check capability"
echo "----------------------------------------------"

# Verify check script exists
if [ ! -f "scripts/check_naming_compliance.sh" ]; then
  echo "❌ FAIL: scripts/check_naming_compliance.sh not found"
  FAIL=$((FAIL + 1))
else
  echo "✅ PASS: Naming compliance check script exists"
  PASS=$((PASS + 1))
fi

# Verify script is executable
if [ -f "scripts/check_naming_compliance.sh" ] && [ -x "scripts/check_naming_compliance.sh" ]; then
  echo "✅ PASS: Script is executable"
  PASS=$((PASS + 1))
else
  echo "❌ FAIL: Script is not executable"
  FAIL=$((FAIL + 1))
fi

# Verify script contains required checks
if [ -f "scripts/check_naming_compliance.sh" ]; then
  if grep -q "aykenos" scripts/check_naming_compliance.sh; then
    echo "✅ PASS: Script checks for 'aykenos' violations"
    PASS=$((PASS + 1))
  else
    echo "❌ FAIL: Script does not check for 'aykenos'"
    FAIL=$((FAIL + 1))
  fi

  if grep -q "phase-" scripts/check_naming_compliance.sh; then
    echo "✅ PASS: Script checks for 'phase-*' violations"
    PASS=$((PASS + 1))
  else
    echo "❌ FAIL: Script does not check for 'phase-*'"
    FAIL=$((FAIL + 1))
  fi

  if grep -q "ayken" scripts/check_naming_compliance.sh; then
    echo "✅ PASS: Script verifies canonical 'ayken' usage"
    PASS=$((PASS + 1))
  else
    echo "❌ FAIL: Script does not verify canonical usage"
    FAIL=$((FAIL + 1))
  fi
fi

# Verify script has proper exit codes
if [ -f "scripts/check_naming_compliance.sh" ]; then
  if grep -q "exit 0" scripts/check_naming_compliance.sh && grep -q "exit 1" scripts/check_naming_compliance.sh; then
    echo "✅ PASS: Script has proper exit codes (0=pass, 1=fail)"
    PASS=$((PASS + 1))
  else
    echo "❌ FAIL: Script missing proper exit codes"
    FAIL=$((FAIL + 1))
  fi
fi

# Verify script includes developer signature
if [ -f "scripts/check_naming_compliance.sh" ]; then
  if grep -q "Kenan AY" scripts/check_naming_compliance.sh; then
    echo "✅ PASS: Script includes developer signature"
    PASS=$((PASS + 1))
  else
    echo "❌ FAIL: Script missing developer signature"
    FAIL=$((FAIL + 1))
  fi
fi

echo ""

# Test 28.2: Naming compliance CI integration
echo "Test 28.2: Naming compliance CI integration"
echo "--------------------------------------------"

# Verify CI workflow exists
if [ ! -f ".github/workflows/governance-naming-compliance.yml" ]; then
  echo "❌ FAIL: CI workflow not found"
  FAIL=$((FAIL + 1))
else
  echo "✅ PASS: CI workflow exists"
  PASS=$((PASS + 1))
fi

# Verify workflow triggers on push and PR
if [ -f ".github/workflows/governance-naming-compliance.yml" ]; then
  if grep -q "push:" .github/workflows/governance-naming-compliance.yml && \
     grep -q "pull_request:" .github/workflows/governance-naming-compliance.yml; then
    echo "✅ PASS: Workflow triggers on push and pull_request"
    PASS=$((PASS + 1))
  else
    echo "❌ FAIL: Workflow missing proper triggers"
    FAIL=$((FAIL + 1))
  fi
fi

# Verify workflow runs the check script
if [ -f ".github/workflows/governance-naming-compliance.yml" ]; then
  if grep -q "check_naming_compliance.sh" .github/workflows/governance-naming-compliance.yml; then
    echo "✅ PASS: Workflow executes naming compliance check"
    PASS=$((PASS + 1))
  else
    echo "❌ FAIL: Workflow does not execute check script"
    FAIL=$((FAIL + 1))
  fi
fi

# Verify workflow targets main/master branches
if [ -f ".github/workflows/governance-naming-compliance.yml" ]; then
  if grep -q "main" .github/workflows/governance-naming-compliance.yml || \
     grep -q "master" .github/workflows/governance-naming-compliance.yml; then
    echo "✅ PASS: Workflow targets main/master branches"
    PASS=$((PASS + 1))
  else
    echo "❌ FAIL: Workflow missing branch targets"
    FAIL=$((FAIL + 1))
  fi
fi

# Verify workflow uploads artifacts on failure
if [ -f ".github/workflows/governance-naming-compliance.yml" ]; then
  if grep -q "upload-artifact" .github/workflows/governance-naming-compliance.yml && \
     grep -q "if: failure()" .github/workflows/governance-naming-compliance.yml; then
    echo "✅ PASS: Workflow uploads artifacts on failure"
    PASS=$((PASS + 1))
  else
    echo "❌ FAIL: Workflow missing artifact upload on failure"
    FAIL=$((FAIL + 1))
  fi
fi

# Verify governance summary workflow exists
if [ -f ".github/workflows/governance-summary.yml" ]; then
  if grep -q "Naming Compliance" .github/workflows/governance-summary.yml || \
     grep -q "check_naming_compliance" .github/workflows/governance-summary.yml; then
    echo "✅ PASS: Governance summary includes naming compliance"
    PASS=$((PASS + 1))
  else
    echo "⚠ WARNING: Governance summary may not include naming compliance"
  fi
fi

echo ""

# Additional validation: Check for advanced CI naming convention script
echo "Additional Validation: Advanced CI Integration"
echo "----------------------------------------------"

if [ -f "scripts/ci/check_naming_convention.sh" ]; then
  echo "✅ PASS: Advanced CI naming convention script exists"
  PASS=$((PASS + 1))

  # Verify regex configuration files
  if [ -f "scripts/ci/naming-convention-deny.regex" ]; then
    echo "✅ PASS: Deny regex configuration exists"
    PASS=$((PASS + 1))
  else
    echo "⚠ WARNING: Deny regex configuration not found"
  fi

  if [ -f "scripts/ci/naming-convention-legacy-allow.regex" ]; then
    echo "✅ PASS: Legacy allow regex configuration exists"
    PASS=$((PASS + 1))
  else
    echo "⚠ WARNING: Legacy allow regex configuration not found"
  fi

  if [ -f "scripts/ci/naming-convention-scope.regex" ]; then
    echo "✅ PASS: Scope regex configuration exists"
    PASS=$((PASS + 1))
  else
    echo "⚠ WARNING: Scope regex configuration not found"
  fi
else
  echo "ℹ INFO: Advanced CI script not found (optional)"
fi

echo ""

# Functional test: Verify script can run without errors on clean state
echo "Functional Test: Script Execution"
echo "----------------------------------"

# Save current git state
git stash push -u -m "test_task28_temp_stash" >/dev/null 2>&1 || true

# Create a test file with violation
TEST_FILE="test_naming_violation_temp.txt"
echo "This file contains aykenos which should be detected" > "$TEST_FILE"
git add "$TEST_FILE" 2>/dev/null || true

# Run the check (expect failure due to violation)
CHECK_OUTPUT=$(./scripts/check_naming_compliance.sh 2>&1 || true)
if echo "$CHECK_OUTPUT" | grep -q "VIOLATION" && echo "$CHECK_OUTPUT" | grep -q "aykenos"; then
  echo "✅ PASS: Check correctly detects 'aykenos' violation"
  PASS=$((PASS + 1))
else
  echo "❌ FAIL: Check did not detect 'aykenos' violation"
  FAIL=$((FAIL + 1))
fi

# Cleanup test file
git reset HEAD "$TEST_FILE" 2>/dev/null || true
rm -f "$TEST_FILE"

# Restore original state
if git stash list | grep -q "test_task28_temp_stash"; then
  git stash pop >/dev/null 2>&1 || true
fi

echo ""

# Constitutional compliance verification
echo "Constitutional Compliance Verification"
echo "---------------------------------------"

# Verify requirement references
if [ -f "scripts/check_naming_compliance.sh" ]; then
  if grep -q "Requirement 25" scripts/check_naming_compliance.sh || \
     grep -q "R25" scripts/check_naming_compliance.sh; then
    echo "✅ PASS: Script references Requirement 25"
    PASS=$((PASS + 1))
  else
    echo "⚠ WARNING: Script may not reference requirements"
  fi
fi

# Verify constitutional reference
if [ -f "scripts/check_naming_compliance.sh" ]; then
  if grep -q "DEV_LOOP_CONSTITUTION" scripts/check_naming_compliance.sh || \
     grep -q "Constitutional" scripts/check_naming_compliance.sh; then
    echo "✅ PASS: Script references constitutional authority"
    PASS=$((PASS + 1))
  else
    echo "⚠ WARNING: Script may not reference constitution"
  fi
fi

echo ""

# Summary
echo "=========================================="
echo "TEST SUMMARY"
echo "=========================================="
echo "PASS: $PASS"
echo "FAIL: $FAIL"
echo ""

if [ "$FAIL" -eq 0 ]; then
  echo "✅ Task 28: Naming Convention Compliance Enforcement - COMPLETE"
  echo ""
  echo "Validated:"
  echo "  ✓ 28.1: Naming compliance check capability"
  echo "  ✓ 28.2: Naming compliance CI integration"
  echo ""
  echo "Requirements Satisfied:"
  echo "  ✓ R25: Naming Convention Enforcement"
  echo "  ✓ R30: Naming Enforcement Scope"
  echo ""
  echo "Constitutional Compliance:"
  echo "  ✓ Naming Law (Section 10)"
  echo "  ✓ Governance Enforcement"
  echo ""
  exit 0
else
  echo "❌ Task 28: INCOMPLETE - $FAIL failures detected"
  echo ""
  echo "Review failures above and ensure:"
  echo "  - Naming compliance check script is complete"
  echo "  - CI workflow is properly configured"
  echo "  - All governance checks are integrated"
  echo ""
  exit 1
fi

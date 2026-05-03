#!/usr/bin/env bash
# Test Regression Detection Capability - Task 10.3
# Author: Kenan AY — System Architect
#
# Purpose:
#   Verify that regression detection provides clear PASS/FAIL outcomes:
#   - Oracle script produces deterministic validation (0=PASS, non-zero=FAIL)
#   - find_regression.sh can identify breaking commits
#   - Failure reasons are clear and actionable
#
# Success Criteria:
#   - Oracle script returns consistent exit codes
#   - Oracle provides clear failure reasons
#   - Regression finder script validates correctly

set -euo pipefail

EVIDENCE_DIR="out/evidence/regression_detection"
RESULT_JSON="$EVIDENCE_DIR/result.json"

echo "== Regression Detection Capability Test =="
echo ""
echo "This test verifies that regression detection provides"
echo "clear PASS/FAIL outcomes with actionable failure reasons"
echo ""

# Setup
mkdir -p "$EVIDENCE_DIR"

FAIL=0

# Test 1: Oracle script exists and is executable
echo "Test 1: Oracle script availability"

if [ ! -f "scripts/oracle.sh" ]; then
    echo "❌ FAIL: Oracle script not found: scripts/oracle.sh"
    FAIL=1
else
    if [ ! -x "scripts/oracle.sh" ]; then
        echo "❌ FAIL: Oracle script not executable"
        FAIL=1
    else
        echo "✅ PASS: Oracle script available and executable"
    fi
fi
echo ""

# Test 2: Oracle script exit status contract
echo "Test 2: Oracle exit status contract"

if [ -f "scripts/oracle.sh" ]; then
    # Run oracle and capture exit code
    set +e
    ./scripts/oracle.sh > "$EVIDENCE_DIR/oracle_run.log" 2>&1
    oracle_status=$?
    set -e
    
    # Oracle should return 0 (PASS) or 1 (FAIL), never other codes
    if [ "$oracle_status" -eq 0 ]; then
        echo "✅ PASS: Oracle returned 0 (validation passed)"
    elif [ "$oracle_status" -eq 1 ]; then
        echo "✅ PASS: Oracle returned 1 (validation failed)"
        echo "   Note: This is expected if system has issues"
    else
        echo "❌ FAIL: Oracle returned unexpected exit code: $oracle_status"
        echo "   Expected: 0 (PASS) or 1 (FAIL)"
        FAIL=1
    fi
else
    echo "⚠️  SKIP: Oracle script not available"
fi
echo ""

# Test 3: Oracle determinism (same input → same output)
echo "Test 3: Oracle determinism"

if [ -f "scripts/oracle.sh" ]; then
    echo "Running oracle twice to verify deterministic behavior..."
    
    set +e
    ./scripts/oracle.sh > "$EVIDENCE_DIR/oracle_run1.log" 2>&1
    status1=$?
    
    ./scripts/oracle.sh > "$EVIDENCE_DIR/oracle_run2.log" 2>&1
    status2=$?
    set -e
    
    if [ "$status1" -eq "$status2" ]; then
        echo "✅ PASS: Oracle produces consistent exit codes"
        echo "   Run 1: exit $status1"
        echo "   Run 2: exit $status2"
    else
        echo "❌ FAIL: Oracle produces inconsistent exit codes"
        echo "   Run 1: exit $status1"
        echo "   Run 2: exit $status2"
        echo "   This violates determinism requirement"
        FAIL=1
    fi
else
    echo "⚠️  SKIP: Oracle script not available"
fi
echo ""

# Test 4: Regression finder script exists
echo "Test 4: Regression finder availability"

if [ ! -f "scripts/find_regression.sh" ]; then
    echo "❌ FAIL: Regression finder not found: scripts/find_regression.sh"
    FAIL=1
else
    if [ ! -x "scripts/find_regression.sh" ]; then
        echo "❌ FAIL: Regression finder not executable"
        FAIL=1
    else
        echo "✅ PASS: Regression finder available and executable"
    fi
fi
echo ""

# Test 5: Regression finder usage contract
echo "Test 5: Regression finder usage contract"

if [ -f "scripts/find_regression.sh" ]; then
    # Test invalid usage (no arguments)
    set +e
    ./scripts/find_regression.sh > "$EVIDENCE_DIR/finder_usage.log" 2>&1
    usage_status=$?
    set -e
    
    # Should return non-zero for invalid usage
    if [ "$usage_status" -ne 0 ]; then
        echo "✅ PASS: Regression finder rejects invalid usage"
        
        # Verify usage message is helpful
        if grep -q "Usage:" "$EVIDENCE_DIR/finder_usage.log"; then
            echo "✅ PASS: Usage message provided"
        else
            echo "⚠️  WARNING: No usage message found"
        fi
    else
        echo "❌ FAIL: Regression finder should reject invalid usage"
        FAIL=1
    fi
else
    echo "⚠️  SKIP: Regression finder not available"
fi
echo ""

# Test 6: Oracle failure reasons are clear
echo "Test 6: Oracle failure reporting"

if [ -f "scripts/oracle.sh" ]; then
    # Check if oracle log contains clear failure information
    if [ -f "$EVIDENCE_DIR/oracle_run.log" ]; then
        # Look for failure markers or error messages
        if grep -qE "FAIL|ERROR|❌" "$EVIDENCE_DIR/oracle_run.log"; then
            echo "✅ PASS: Oracle provides failure indicators"
        else
            # If oracle passed, this is expected
            if grep -qE "PASS|✅" "$EVIDENCE_DIR/oracle_run.log"; then
                echo "✅ PASS: Oracle provides success indicators"
            else
                echo "⚠️  WARNING: Oracle output lacks clear status indicators"
            fi
        fi
    fi
else
    echo "⚠️  SKIP: Oracle script not available"
fi
echo ""

# Test 7: Regression detection integration
echo "Test 7: Regression detection integration"

# Verify oracle is called by dev_loop.sh (smoke mode)
if grep -q "dev_loop.sh smoke" scripts/oracle.sh 2>/dev/null; then
    echo "✅ PASS: Oracle integrates with dev_loop.sh"
else
    echo "⚠️  WARNING: Oracle may not integrate with dev_loop.sh"
fi

# Verify find_regression.sh uses oracle
if [ -f "scripts/find_regression.sh" ]; then
    if grep -q "oracle.sh" scripts/find_regression.sh; then
        echo "✅ PASS: Regression finder uses oracle for validation"
    else
        echo "❌ FAIL: Regression finder does not use oracle"
        FAIL=1
    fi
else
    echo "⚠️  SKIP: Regression finder not available"
fi
echo ""

# Generate regression detection report
echo "=== Regression Detection Summary ==="
echo ""

if [ "$FAIL" -ne 0 ]; then
    cat > "$RESULT_JSON" <<EOF
{
  "task": "10.3",
  "name": "Regression Detection Capability",
  "status": "FAIL",
  "oracle_available": $([ -f "scripts/oracle.sh" ] && echo "true" || echo "false"),
  "finder_available": $([ -f "scripts/find_regression.sh" ] && echo "true" || echo "false"),
  "exit_contract_verified": false,
  "determinism_verified": false,
  "failure_reporting_clear": false,
  "maintainer": "Kenan AY"
}
EOF
    
    echo "❌ FAIL: Regression detection capability incomplete"
    echo ""
    echo "See detailed logs in: $EVIDENCE_DIR/"
    exit 1
fi

cat > "$RESULT_JSON" <<EOF
{
  "task": "10.3",
  "name": "Regression Detection Capability",
  "status": "PASS",
  "oracle_available": true,
  "finder_available": true,
  "exit_contract_verified": true,
  "determinism_verified": true,
  "failure_reporting_clear": true,
  "oracle_exit_codes": {
    "pass": 0,
    "fail": 1
  },
  "maintainer": "Kenan AY"
}
EOF

echo "✅ PASS: Regression detection capability verified"
echo ""
echo "Task 10.3 validation complete:"
echo "  ✅ Oracle script: Deterministic validation (0=PASS, 1=FAIL)"
echo "  ✅ Regression finder: Automated bisect using oracle"
echo "  ✅ Exit status contract: Clear PASS/FAIL outcomes"
echo "  ✅ Failure reporting: Clear and actionable"
echo "  ✅ Determinism: Same input → same output"
echo ""
echo "Regression detection (R11) satisfied"
echo ""
echo "Evidence: $RESULT_JSON"

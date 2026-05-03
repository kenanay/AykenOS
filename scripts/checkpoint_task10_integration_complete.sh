#!/usr/bin/env bash
# Checkpoint: Task 10 Integration Complete
# Author: Kenan AY — System Architect
#
# Purpose:
#   Final gate for Task 10 - validates SYSTEM-LEVEL GUARANTEE:
#   - All validation layers operational
#   - Constitutional compliance enforced
#   - Regression detection functional
#   - System is deterministic and reproducible
#   - System can prove its own correctness
#
# This checkpoint ensures the system has achieved:
#   "SELF-VERIFIABLE TRUTH ENGINE"
#
# Success Criteria:
#   - Task 10 integration test passes
#   - All subtasks (10.1, 10.2, 10.3) complete
#   - Multi-run hash consistency verified
#   - Oracle output strengthened
#   - System-level guarantee established

set -euo pipefail

CHECKPOINT_NAME="Task 10: Integration Complete"
EVIDENCE_DIR="out/evidence/checkpoint_task10"
RESULT_JSON="$EVIDENCE_DIR/result.json"

echo "=========================================="
echo "CHECKPOINT: $CHECKPOINT_NAME"
echo "=========================================="
echo ""
echo "Validating SYSTEM-LEVEL GUARANTEE:"
echo "  ✓ Determinism (same input → same output)"
echo "  ✓ Reproducibility (same input → same hash)"
echo "  ✓ Self-verification (system proves itself)"
echo ""

# Setup
mkdir -p "$EVIDENCE_DIR"

# Run Task 10 integration test
echo "Running Task 10 integration test..."
echo ""

set +e
./scripts/test_task10_integration_completeness.sh > "$EVIDENCE_DIR/task10_test.log" 2>&1
task10_status=$?
set -e

if [ "$task10_status" -ne 0 ]; then
    cat > "$RESULT_JSON" <<EOF
{
  "checkpoint": "task10_integration_complete",
  "status": "FAIL",
  "task": "10",
  "name": "Integration Completeness",
  "verdict": "FAIL",
  "reason": "Task 10 integration test failed",
  "see_log": "$EVIDENCE_DIR/task10_test.log",
  "maintainer": "Kenan AY"
}
EOF
    
    echo "❌ CHECKPOINT FAILED"
    echo ""
    echo "Task 10 integration test failed (exit $task10_status)"
    echo ""
    echo "See detailed log:"
    echo "  $EVIDENCE_DIR/task10_test.log"
    echo ""
    echo "Last 50 lines of test output:"
    tail -50 "$EVIDENCE_DIR/task10_test.log"
    echo ""
    exit 1
fi

# Verify system-level guarantee was established
echo ""
echo "Verifying system-level guarantee..."
echo ""

TASK10_RESULT="out/evidence/task10_integration/result.json"

if [ ! -f "$TASK10_RESULT" ]; then
    cat > "$RESULT_JSON" <<EOF
{
  "checkpoint": "task10_integration_complete",
  "status": "FAIL",
  "task": "10",
  "verdict": "FAIL",
  "reason": "Task 10 result.json not found",
  "maintainer": "Kenan AY"
}
EOF
    
    echo "❌ CHECKPOINT FAILED"
    echo ""
    echo "Task 10 result.json not found at: $TASK10_RESULT"
    exit 1
fi

# Extract the authoritative system guarantee fields from result.json.
system_guarantee_determinism=$(jq -r '.system_guarantee.determinism // false' "$TASK10_RESULT")
system_guarantee_reproducibility=$(jq -r '.system_guarantee.reproducibility // false' "$TASK10_RESULT")
system_guarantee_self_verification=$(jq -r '.system_guarantee.self_verification // false' "$TASK10_RESULT")

echo "System guarantee status:"
echo "  Determinism: $system_guarantee_determinism"
echo "  Reproducibility: $system_guarantee_reproducibility"
echo "  Self-verification: $system_guarantee_self_verification"
echo ""

if [ "$system_guarantee_determinism" != "true" ] || \
   [ "$system_guarantee_reproducibility" != "true" ] || \
   [ "$system_guarantee_self_verification" != "true" ]; then
    
    cat > "$RESULT_JSON" <<EOF
{
  "checkpoint": "task10_integration_complete",
  "status": "FAIL",
  "task": "10",
  "verdict": "FAIL",
  "reason": "System-level guarantee not established",
  "system_guarantee": {
    "determinism": $system_guarantee_determinism,
    "reproducibility": $system_guarantee_reproducibility,
    "self_verification": $system_guarantee_self_verification
  },
  "maintainer": "Kenan AY"
}
EOF
    
    echo "❌ CHECKPOINT FAILED"
    echo ""
    echo "System-level guarantee NOT established"
    echo ""
    echo "Required:"
    echo "  ✓ Determinism: true"
    echo "  ✓ Reproducibility: true"
    echo "  ✓ Self-verification: true"
    echo ""
    echo "Actual:"
    echo "  ✗ Determinism: $system_guarantee_determinism"
    echo "  ✗ Reproducibility: $system_guarantee_reproducibility"
    echo "  ✗ Self-verification: $system_guarantee_self_verification"
    exit 1
fi

# Verify all subtasks completed
subtasks_passed=$(jq -r '.subtasks_passed // 0' "$TASK10_RESULT")
subtasks_total=$(jq -r '.subtasks_total // 3' "$TASK10_RESULT")

echo "Subtasks: $subtasks_passed / $subtasks_total"
echo ""

if [ "$subtasks_passed" -ne "$subtasks_total" ]; then
    cat > "$RESULT_JSON" <<EOF
{
  "checkpoint": "task10_integration_complete",
  "status": "FAIL",
  "task": "10",
  "verdict": "FAIL",
  "reason": "Not all subtasks completed",
  "subtasks_passed": $subtasks_passed,
  "subtasks_total": $subtasks_total,
  "maintainer": "Kenan AY"
}
EOF
    
    echo "❌ CHECKPOINT FAILED"
    echo ""
    echo "Not all subtasks completed: $subtasks_passed / $subtasks_total"
    exit 1
fi

# Generate checkpoint success report
cat > "$RESULT_JSON" <<EOF
{
  "checkpoint": "task10_integration_complete",
  "status": "PASS",
  "task": "10",
  "name": "Integration Completeness",
  "verdict": "PASS",
  "system_guarantee": {
    "determinism": true,
    "reproducibility": true,
    "self_verification": true,
    "status": "VERIFIED"
  },
  "subtasks": {
    "10.1": "Full Validation Capability",
    "10.2": "Constitutional Compliance Guarantee",
    "10.3": "Regression Detection Capability"
  },
  "subtasks_passed": $subtasks_passed,
  "subtasks_total": $subtasks_total,
  "requirements_satisfied": ["R2", "R11", "R12"],
  "achievement": "SELF-VERIFIABLE TRUTH ENGINE",
  "maintainer": "Kenan AY"
}
EOF

echo "✅ CHECKPOINT PASSED"
echo ""
echo "=========================================="
echo "SYSTEM-LEVEL GUARANTEE ESTABLISHED"
echo "=========================================="
echo ""
echo "Task 10 Integration Complete:"
echo "  ✅ 10.1: Full validation capability"
echo "  ✅ 10.2: Constitutional compliance guarantee"
echo "  ✅ 10.3: Regression detection capability"
echo ""
echo "System Properties Verified:"
echo "  ✅ Determinism: Same input → same output"
echo "  ✅ Reproducibility: Multi-run hash consistency"
echo "  ✅ Self-verification: System proves itself"
echo ""
echo "Requirements Satisfied:"
echo "  ✅ R2: Multi-Level Validation Modes"
echo "  ✅ R11: Regression Detection"
echo "  ✅ R12: Constitutional Compliance"
echo ""
echo "Achievement Unlocked:"
echo "  🏆 SELF-VERIFIABLE TRUTH ENGINE"
echo ""
echo "The system can now:"
echo "  • Validate itself deterministically"
echo "  • Prove its own correctness"
echo "  • Detect regressions automatically"
echo "  • Enforce constitutional compliance"
echo ""
echo "Evidence:"
echo "  $RESULT_JSON"
echo "  $TASK10_RESULT"
echo ""
echo "=========================================="
echo "Task 10 (Integration completeness) COMPLETE"
echo "=========================================="

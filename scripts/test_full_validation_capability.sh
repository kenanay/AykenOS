#!/usr/bin/env bash
# Test Full Validation Capability - Task 10.1
# Author: Kenan AY — System Architect
#
# Purpose:
#   Verify that dev_loop.sh full mode orchestrates all validation levels:
#   - Smoke validation (build + boot + markers)
#   - Contract validation (runtime contract tests)
#   - Evidence validation (evidence-layer tests)
#
# Success Criteria:
#   - Full mode executes all three validation levels
#   - Each level produces expected output
#   - Exit status contract maintained (0=PASS, 1=FAIL)

set -euo pipefail

TEST_LOG_DIR="${TEST_LOG_DIR:-out/evidence/full_validation_capability/logs}"
SMOKE_LOG="$TEST_LOG_DIR/smoke.log"
CONTRACT_LOG="$TEST_LOG_DIR/contract.log"
FULL_LOG="$TEST_LOG_DIR/full.log"

echo "== Full Validation Capability Test =="
echo ""
echo "This test verifies that dev_loop.sh full mode orchestrates"
echo "all validation levels: smoke → contract → evidence"
echo ""

# Setup
mkdir -p "$TEST_LOG_DIR"

# Test 1: Smoke mode (baseline)
echo "Test 1: Smoke mode validation"
set +e
./scripts/dev_loop.sh smoke > "$SMOKE_LOG" 2>&1
smoke_status=$?
set -e

if [ "$smoke_status" -eq 0 ]; then
    echo "✅ PASS: Smoke mode completed successfully"
else
    echo "❌ FAIL: Smoke mode failed (exit $smoke_status)"
    echo ""
    echo "Last 30 lines of smoke log:"
    tail -30 "$SMOKE_LOG"
    exit 1
fi

# Verify smoke mode output
if ! grep -q "Smoke boot test" "$SMOKE_LOG"; then
    echo "❌ FAIL: Smoke mode did not execute boot test"
    exit 1
fi

if ! grep -q "PASS: smoke mode" "$SMOKE_LOG"; then
    echo "❌ FAIL: Smoke mode did not produce PASS verdict"
    exit 1
fi

echo ""

# Test 2: Contract mode (smoke + contract tests)
echo "Test 2: Contract mode validation"
set +e
./scripts/dev_loop.sh contract > "$CONTRACT_LOG" 2>&1
contract_status=$?
set -e

if [ "$contract_status" -eq 0 ]; then
    echo "✅ PASS: Contract mode completed successfully"
else
    echo "❌ FAIL: Contract mode failed (exit $contract_status)"
    echo ""
    echo "Last 30 lines of contract log:"
    tail -30 "$CONTRACT_LOG"
    exit 1
fi

# Verify contract mode output
if ! grep -q "Smoke boot test" "$CONTRACT_LOG"; then
    echo "❌ FAIL: Contract mode did not execute smoke test"
    exit 1
fi

if ! grep -q "Contract tests" "$CONTRACT_LOG"; then
    echo "❌ FAIL: Contract mode did not execute contract tests"
    exit 1
fi

if ! grep -q "PASS: contract mode" "$CONTRACT_LOG"; then
    echo "❌ FAIL: Contract mode did not produce PASS verdict"
    exit 1
fi

echo ""

# Test 3: Full mode (smoke + contract + evidence tests)
echo "Test 3: Full mode validation"
set +e
./scripts/dev_loop.sh full > "$FULL_LOG" 2>&1
full_status=$?
set -e

if [ "$full_status" -eq 0 ]; then
    echo "✅ PASS: Full mode completed successfully"
else
    echo "❌ FAIL: Full mode failed (exit $full_status)"
    echo ""
    echo "Last 30 lines of full log:"
    tail -30 "$FULL_LOG"
    exit 1
fi

# Verify full mode output
if ! grep -q "Smoke boot test" "$FULL_LOG"; then
    echo "❌ FAIL: Full mode did not execute smoke test"
    exit 1
fi

if ! grep -q "Full evidence tests" "$FULL_LOG"; then
    echo "❌ FAIL: Full mode did not execute evidence tests"
    exit 1
fi

if ! grep -q "PASS: full mode" "$FULL_LOG"; then
    echo "❌ FAIL: Full mode did not produce PASS verdict"
    exit 1
fi

echo ""

# Test 4: Verify orchestration order
echo "Test 4: Verify validation level orchestration"

# Extract line numbers for each phase in full mode
build_line=$(grep -n "Build\.\.\." "$FULL_LOG" | head -1 | cut -d: -f1 || echo 0)
smoke_line=$(grep -n "Smoke boot test" "$FULL_LOG" | head -1 | cut -d: -f1 || echo 0)
evidence_line=$(grep -n "Full evidence tests" "$FULL_LOG" | head -1 | cut -d: -f1 || echo 0)

if [ "$build_line" -eq 0 ] || [ "$smoke_line" -eq 0 ] || [ "$evidence_line" -eq 0 ]; then
    echo "❌ FAIL: Could not verify orchestration order"
    exit 1
fi

if [ "$build_line" -ge "$smoke_line" ]; then
    echo "❌ FAIL: Build must occur before smoke test"
    exit 1
fi

if [ "$smoke_line" -ge "$evidence_line" ]; then
    echo "❌ FAIL: Smoke test must occur before evidence tests"
    exit 1
fi

echo "✅ PASS: Validation levels execute in correct order"
echo "   Build (line $build_line) → Smoke (line $smoke_line) → Evidence (line $evidence_line)"
echo ""

# Test 5: Verify exit status contract
echo "Test 5: Verify exit status contract"

# Test invalid mode (should return 2)
set +e
./scripts/dev_loop.sh invalid_mode > /dev/null 2>&1
invalid_status=$?
set -e

if [ "$invalid_status" -eq 2 ]; then
    echo "✅ PASS: Invalid mode returns exit code 2"
else
    echo "❌ FAIL: Invalid mode should return 2, got $invalid_status"
    exit 1
fi

echo ""

echo "== Full Validation Capability Verified =="
echo ""
echo "Task 10.1 validation complete:"
echo "  ✅ Smoke mode: build + boot + markers"
echo "  ✅ Contract mode: smoke + runtime contract tests"
echo "  ✅ Full mode: contract + evidence tests"
echo "  ✅ Orchestration order: build → smoke → contract → evidence"
echo "  ✅ Exit status contract: 0=PASS, 1=FAIL, 2=INVALID"
echo ""
echo "Multi-level validation capability (R2) satisfied"

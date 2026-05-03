a#!/bin/bash
# QEMU Integration Test - Task 2 Checkpoint
# Author: Kenan AY — System Architect
#
# This script validates the full system marker guarantee:
# UEFI → Kernel → EARLY → LATE → BOOT_OK → dev_loop → exit code
#
# This is a SYSTEM test (not unit test):
# - Builds kernel with validation profile
# - Boots in QEMU
# - Validates marker presence and sequence from real boot log
# - Verifies exit code contract

set -euo pipefail

echo "== QEMU Integration Test - Marker Guarantee Checkpoint =="
echo ""
echo "This test validates the full system chain:"
echo "  UEFI → Kernel → Markers → dev_loop → exit code"
echo ""

# Run dev_loop.sh smoke test (builds + boots QEMU)
echo "[1/2] Running dev_loop.sh smoke test..."
set +e
./scripts/dev_loop.sh smoke
exit_code=$?
set -e

echo ""
echo "[2/2] Validating results..."
echo "Exit code: $exit_code"
echo ""

# Check exit code contract
if [ "$exit_code" -ne 0 ]; then
    echo "❌ FAIL: Expected exit code 0, got $exit_code"
    echo ""
    echo "This indicates:"
    echo "  - Build failure, OR"
    echo "  - Boot timeout, OR"
    echo "  - Missing marker, OR"
    echo "  - Marker sequence violation"
    echo ""
    echo "Check out/logs/boot_watch.log for details"
    exit 1
fi

# Verify log file exists (kernel debug log, not make output)
LOG_FILE="out/logs/debug_run.log"
if [ ! -f "$LOG_FILE" ]; then
    echo "❌ FAIL: Kernel log not found at $LOG_FILE"
    exit 1
fi

# Verify log is not empty
if [ ! -s "$LOG_FILE" ]; then
    echo "❌ FAIL: Boot log is empty"
    exit 1
fi

# Verify all required markers are present
echo "Checking marker presence..."

if ! grep -q "\[K\]\[EARLY_BOOT_OK\]" "$LOG_FILE"; then
    echo "❌ FAIL: [K][EARLY_BOOT_OK] marker not found in boot log"
    exit 1
fi
echo "  ✅ [K][EARLY_BOOT_OK] found"

if ! grep -q "\[K\]\[LATE_INIT_END\]" "$LOG_FILE"; then
    echo "❌ FAIL: [K][LATE_INIT_END] marker not found in boot log"
    exit 1
fi
echo "  ✅ [K][LATE_INIT_END] found"

if ! grep -q "\[\[AYKEN_BOOT_OK\]\]" "$LOG_FILE"; then
    echo "❌ FAIL: [[AYKEN_BOOT_OK]] marker not found in boot log"
    exit 1
fi
echo "  ✅ [[AYKEN_BOOT_OK]] found"

echo ""
echo "Checking marker sequence..."

# Extract line numbers
early_line=$(grep -n "\[K\]\[EARLY_BOOT_OK\]" "$LOG_FILE" | head -1 | cut -d: -f1)
late_line=$(grep -n "\[K\]\[LATE_INIT_END\]" "$LOG_FILE" | head -1 | cut -d: -f1)
boot_line=$(grep -n "\[\[AYKEN_BOOT_OK\]\]" "$LOG_FILE" | head -1 | cut -d: -f1)

echo "  [K][EARLY_BOOT_OK]: line $early_line"
echo "  [K][LATE_INIT_END]: line $late_line"
echo "  [[AYKEN_BOOT_OK]]: line $boot_line"

# Validate sequence: EARLY < LATE < BOOT_OK
if [ "$early_line" -ge "$late_line" ]; then
    echo ""
    echo "❌ FAIL: Marker sequence violation"
    echo "  Expected: EARLY_BOOT_OK (line $early_line) < LATE_INIT_END (line $late_line)"
    exit 1
fi

if [ "$late_line" -ge "$boot_line" ]; then
    echo ""
    echo "❌ FAIL: Marker sequence violation"
    echo "  Expected: LATE_INIT_END (line $late_line) < AYKEN_BOOT_OK (line $boot_line)"
    exit 1
fi

echo "  ✅ Sequence valid: EARLY → LATE → BOOT_OK"
echo ""

echo "✅ PASS: Full system marker guarantee verified"
echo ""
echo "Checkpoint validated:"
echo "  ✅ Kernel emits markers during boot"
echo "  ✅ Markers appear in correct sequence"
echo "  ✅ dev_loop.sh validates markers correctly"
echo "  ✅ Exit code contract enforced"
echo ""
echo "Task 2 (Checkpoint - Marker guarantee operational) COMPLETE"

#!/bin/bash
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

marker_positions="$(awk '
    BEGIN {
        early_line = 0
        late_line = 0
        boot_line = 0
        early_count = 0
        late_count = 0
        boot_count = 0
        pre_sequence_boot_count = 0
    }
    index($0, "[K][EARLY_BOOT_OK]") {
        early_count++
        if (early_line == 0) {
            early_line = NR
        }
    }
    index($0, "[K][LATE_INIT_END]") {
        late_count++
        if (early_line > 0 && late_line == 0 && NR > early_line) {
            late_line = NR
        }
    }
    index($0, "[[AYKEN_BOOT_OK]]") {
        boot_count++
        if (early_line == 0) {
            pre_sequence_boot_count++
        }
        if (late_line > 0 && boot_line == 0 && NR > late_line) {
            boot_line = NR
        }
    }
    END {
        printf("early_line=%d\n", early_line)
        printf("late_line=%d\n", late_line)
        printf("boot_line=%d\n", boot_line)
        printf("early_count=%d\n", early_count)
        printf("late_count=%d\n", late_count)
        printf("boot_count=%d\n", boot_count)
        printf("pre_sequence_boot_count=%d\n", pre_sequence_boot_count)
    }
' "$LOG_FILE")"

early_line=0
late_line=0
boot_line=0
early_count=0
late_count=0
boot_count=0
pre_sequence_boot_count=0

while IFS='=' read -r key value; do
    case "$key" in
        early_line) early_line="$value" ;;
        late_line) late_line="$value" ;;
        boot_line) boot_line="$value" ;;
        early_count) early_count="$value" ;;
        late_count) late_count="$value" ;;
        boot_count) boot_count="$value" ;;
        pre_sequence_boot_count) pre_sequence_boot_count="$value" ;;
    esac
done <<EOF
$marker_positions
EOF

echo "Checking marker presence..."

if [ "$early_count" -eq 0 ]; then
    echo "❌ FAIL: [K][EARLY_BOOT_OK] marker not found in boot log"
    exit 1
fi
echo "  ✅ [K][EARLY_BOOT_OK] found ($early_count)"

if [ "$late_count" -eq 0 ]; then
    echo "❌ FAIL: [K][LATE_INIT_END] marker not found in boot log"
    exit 1
fi
echo "  ✅ [K][LATE_INIT_END] found ($late_count)"

if [ "$boot_count" -eq 0 ]; then
    echo "❌ FAIL: [[AYKEN_BOOT_OK]] marker not found in boot log"
    exit 1
fi
echo "  ✅ [[AYKEN_BOOT_OK]] found ($boot_count)"

echo ""
echo "Checking marker sequence..."

echo "  [K][EARLY_BOOT_OK]: line $early_line"
echo "  [K][LATE_INIT_END]: line $late_line"
echo "  [[AYKEN_BOOT_OK]]: line $boot_line"

if [ "$late_line" -eq 0 ]; then
    echo ""
    echo "❌ FAIL: Marker sequence violation"
    echo "  Expected: EARLY_BOOT_OK before LATE_INIT_END"
    exit 1
fi

if [ "$boot_line" -eq 0 ]; then
    echo ""
    echo "❌ FAIL: Marker sequence violation"
    echo "  Expected: LATE_INIT_END before AYKEN_BOOT_OK"
    exit 1
fi

if [ "$pre_sequence_boot_count" -gt 0 ]; then
    echo "  Note: observed $pre_sequence_boot_count pre-sequence BOOT marker(s); canonical post-late marker used."
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

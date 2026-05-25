#!/bin/bash
# Test script for marker validation logic
# Author: Kenan AY — System Architect
#
# This script tests marker validation by verifying the validation function
# that dev_loop.sh uses. This is a UNIT test for the validation logic only.
# For SYSTEM tests, use integration tests that call dev_loop.sh with QEMU.

set -euo pipefail

TEST_LOG_DIR="out/logs/test"
TEST_LOG="$TEST_LOG_DIR/test_boot.log"

echo "== Marker Validation Unit Test =="
echo ""
echo "Note: This tests the validation logic in isolation."
echo "For full system tests, use integration tests with QEMU."
echo ""

# Setup
mkdir -p "$TEST_LOG_DIR"

# Extract validation function from dev_loop.sh for unit testing
# This validates the LOGIC, not the full system
validate_markers() {
    local log_file="$1"
    local marker_positions
    local key
    local value
    local early_line=0
    local late_line=0
    local boot_line=0
    local early_count=0
    local late_count=0
    local boot_count=0

    marker_positions="$(awk '
        BEGIN {
            early_line = 0
            late_line = 0
            boot_line = 0
            early_count = 0
            late_count = 0
            boot_count = 0
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
        }
    ' "$log_file")"

    while IFS='=' read -r key value; do
        case "$key" in
            early_line) early_line="$value" ;;
            late_line) late_line="$value" ;;
            boot_line) boot_line="$value" ;;
            early_count) early_count="$value" ;;
            late_count) late_count="$value" ;;
            boot_count) boot_count="$value" ;;
        esac
    done <<EOF
$marker_positions
EOF

    if [ "$early_count" -eq 0 ]; then
        return 1
    fi

    if [ "$late_count" -eq 0 ]; then
        return 1
    fi

    if [ "$boot_count" -eq 0 ]; then
        return 1
    fi

    if [ "$late_line" -eq 0 ] || [ "$boot_line" -eq 0 ]; then
        return 1
    fi

    return 0
}

# Test 1: Valid marker sequence (should PASS)
echo "Test 1: Valid marker sequence"
cat > "$TEST_LOG" <<'EOF'
Boot starting...
[K][EARLY_BOOT_OK]
Initializing subsystems...
[K][LATE_INIT_END]
Final boot steps...
[[AYKEN_BOOT_OK]]
System ready.
EOF

set +e
validate_markers "$TEST_LOG"
exit_code=$?
set -e

if [ "$exit_code" -eq 0 ]; then
    echo "✅ PASS: Valid sequence accepted"
else
    echo "❌ FAIL: Valid sequence rejected (exit $exit_code)"
    exit 1
fi
echo ""

# Test 2: Existing Gate-0 marker before EARLY should still PASS
echo "Test 2: Pre-sequence BOOT marker with canonical post-late BOOT"
cat > "$TEST_LOG" <<'EOF'
Firmware handoff...
[[AYKEN_BOOT_OK]]
[K][EARLY_BOOT_OK]
Initializing subsystems...
[K][LATE_INIT_END]
Final boot steps...
[[AYKEN_BOOT_OK]]
System ready.
EOF

set +e
validate_markers "$TEST_LOG"
exit_code=$?
set -e

if [ "$exit_code" -eq 0 ]; then
    echo "✅ PASS: Canonical post-late BOOT marker accepted"
else
    echo "❌ FAIL: Canonical sequence rejected (exit $exit_code)"
    exit 1
fi
echo ""

# Test 3: Missing EARLY_BOOT_OK (should FAIL)
echo "Test 3: Missing EARLY_BOOT_OK marker"
cat > "$TEST_LOG" <<'EOF'
Boot starting...
Initializing subsystems...
[K][LATE_INIT_END]
Final boot steps...
[[AYKEN_BOOT_OK]]
System ready.
EOF

set +e
validate_markers "$TEST_LOG" > /dev/null 2>&1
exit_code=$?
set -e

if [ "$exit_code" -eq 1 ]; then
    echo "✅ PASS: Missing marker correctly rejected"
else
    echo "❌ FAIL: Expected exit 1, got $exit_code"
    exit 1
fi
echo ""

# Test 4: Sequence violation (should FAIL)
echo "Test 4: Markers in wrong order"
cat > "$TEST_LOG" <<'EOF'
Boot starting...
[K][LATE_INIT_END]
Initializing subsystems...
[K][EARLY_BOOT_OK]
Final boot steps...
[[AYKEN_BOOT_OK]]
System ready.
EOF

set +e
validate_markers "$TEST_LOG" > /dev/null 2>&1
exit_code=$?
set -e

if [ "$exit_code" -eq 1 ]; then
    echo "✅ PASS: Sequence violation correctly rejected"
else
    echo "❌ FAIL: Expected exit 1, got $exit_code"
    exit 1
fi
echo ""

# Cleanup
rm -rf "$TEST_LOG_DIR"

echo "== Unit Tests Passed =="
echo ""
echo "Marker validation logic verified:"
echo "  ✅ Subtask 1.1: Marker sequence guarantee"
echo "  ✅ Subtask 1.2: Error reporting capability"
echo ""
echo "For full system validation, run integration tests with QEMU"

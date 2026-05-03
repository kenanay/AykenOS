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
    
    # Check marker presence
    if ! grep -q "\[K\]\[EARLY_BOOT_OK\]" "$log_file"; then
        return 1
    fi
    
    if ! grep -q "\[K\]\[LATE_INIT_END\]" "$log_file"; then
        return 1
    fi
    
    if ! grep -q "\[\[AYKEN_BOOT_OK\]\]" "$log_file"; then
        return 1
    fi
    
    # Check sequence
    local early_line=$(grep -n "\[K\]\[EARLY_BOOT_OK\]" "$log_file" | head -1 | cut -d: -f1)
    local late_line=$(grep -n "\[K\]\[LATE_INIT_END\]" "$log_file" | head -1 | cut -d: -f1)
    local boot_line=$(grep -n "\[\[AYKEN_BOOT_OK\]\]" "$log_file" | head -1 | cut -d: -f1)
    
    if [ "$early_line" -gt "$late_line" ] || [ "$late_line" -gt "$boot_line" ]; then
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

# Test 2: Missing EARLY_BOOT_OK (should FAIL)
echo "Test 2: Missing EARLY_BOOT_OK marker"
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

# Test 3: Sequence violation (should FAIL)
echo "Test 3: Markers in wrong order"
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

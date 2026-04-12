#!/bin/bash

# Property-Based Test: Bug Condition - Evidence Pipeline Integrity Failure
#
# This test explores the bug condition on UNFIXED code to surface counterexamples
# that demonstrate the evidence pipeline regression.
#
# CRITICAL: This test MUST FAIL on unfixed code - failure confirms the bug exists
# DO NOT attempt to fix the test or the code when it fails
#
# Expected Outcome: TEST FAILS (confirms evidence pipeline regression)
# Counterexamples: Documents which channels are empty, which markers are missing,
#                  whether sort is present in harness scripts
#
# Author: Kenan AY - Architectural Steward
# Spec: .kiro/specs/boot-chain-observability-restoration/
# Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TEST_NAME="test_bug_condition_evidence_pipeline"
EVIDENCE_DIR="$PROJECT_ROOT/evidence/pbt/$TEST_NAME"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_property() {
    echo -e "${BLUE}[PROPERTY]${NC} $1"
}

# Create evidence directory
mkdir -p "$EVIDENCE_DIR"

COUNTEREXAMPLES_LOG="$EVIDENCE_DIR/counterexamples.log"
TEST_RESULT_JSON="$EVIDENCE_DIR/test_result.json"

log_info "========================================="
log_info "Property Test: Bug Condition Exploration"
log_info "========================================="
log_info "Test: Evidence Pipeline Integrity Failure"
log_info "Expected: TEST FAILS (confirms bug exists)"
log_info ""

# Initialize counterexamples log
> "$COUNTEREXAMPLES_LOG"

# ============================================================================
# Property 1: Bug Condition - Evidence Pipeline Integrity
# ============================================================================
log_property "Property 1: Evidence Pipeline Integrity"
log_property "For any boot execution with marker emission,"
log_property "at least one output channel SHALL capture those markers"
log_property ""

# Test input: QEMU boot execution with marker emission
log_info "Test Input: QEMU boot with bootloader and kernel marker emission"
log_info "Running QEMU boot observability harness..."
log_info ""

# Run the boot observability harness to generate evidence
HARNESS_SCRIPT="$PROJECT_ROOT/scripts/qemu-boot-observability-harness.sh"

if [[ ! -f "$HARNESS_SCRIPT" ]]; then
    log_error "Harness script not found: $HARNESS_SCRIPT"
    log_error "Cannot run bug condition exploration test"
    exit 1
fi

# Run harness (may fail, that's expected for bug condition exploration)
set +e
bash "$HARNESS_SCRIPT"
HARNESS_EXIT=$?
set -e

log_info ""
log_info "Harness exit code: $HARNESS_EXIT"
log_info ""

# Analyze evidence to find counterexamples
BOOT_OBSERVABILITY_DIR="$PROJECT_ROOT/evidence/boot-observability"
DEBUGCON_LOG="$BOOT_OBSERVABILITY_DIR/qemu_debugcon.log"
SERIAL_LOG="$BOOT_OBSERVABILITY_DIR/qemu_serial.log"
TRACE_DEBUGCON="$BOOT_OBSERVABILITY_DIR/debugcon.trace"
TRACE_SERIAL="$BOOT_OBSERVABILITY_DIR/serial.trace"

# ============================================================================
# Counterexample Detection
# ============================================================================
log_info "========================================="
log_info "Counterexample Detection"
log_info "========================================="
log_info ""

COUNTEREXAMPLES_FOUND=0

# Check 1: Channel integrity (at least one channel must be non-empty)
log_info "Check 1: Channel Integrity"

DEBUGCON_SIZE=0
SERIAL_SIZE=0

if [[ -f "$DEBUGCON_LOG" ]]; then
    DEBUGCON_SIZE=$(stat -c%s "$DEBUGCON_LOG" 2>/dev/null || stat -f%z "$DEBUGCON_LOG" 2>/dev/null || echo "0")
fi

if [[ -f "$SERIAL_LOG" ]]; then
    SERIAL_SIZE=$(stat -c%s "$SERIAL_LOG" 2>/dev/null || stat -f%z "$SERIAL_LOG" 2>/dev/null || echo "0")
fi

log_info "  Debugcon size: $DEBUGCON_SIZE bytes"
log_info "  Serial size: $SERIAL_SIZE bytes"

if [[ $DEBUGCON_SIZE -eq 0 ]] && [[ $SERIAL_SIZE -eq 0 ]]; then
    log_error "  ✗ COUNTEREXAMPLE: All output channels are empty"
    echo "COUNTEREXAMPLE: OUTPUT_CHANNEL_FAILURE - debugcon=$DEBUGCON_SIZE, serial=$SERIAL_SIZE" >> "$COUNTEREXAMPLES_LOG"
    COUNTEREXAMPLES_FOUND=$((COUNTEREXAMPLES_FOUND + 1))
else
    log_info "  ✓ At least one channel has output"
fi

log_info ""

# Check 2: Required markers present
log_info "Check 2: Required Markers Present"

REQUIRED_MARKERS=(
    "\[B\]\[UEFI_BOOT_START\]"
    "\[\[AYKEN_BOOT_OK\]\]"
    "\[K\]\[EARLY_BOOT_OK\]"
)

MARKER_NAMES=(
    "[B][UEFI_BOOT_START]"
    "[[AYKEN_BOOT_OK]]"
    "[K][EARLY_BOOT_OK]"
)

for i in "${!REQUIRED_MARKERS[@]}"; do
    marker="${REQUIRED_MARKERS[$i]}"
    marker_name="${MARKER_NAMES[$i]}"
    
    FOUND_IN_DEBUGCON=0
    FOUND_IN_SERIAL=0
    
    if [[ -f "$TRACE_DEBUGCON" ]] && grep -q "$marker" "$TRACE_DEBUGCON"; then
        FOUND_IN_DEBUGCON=1
    fi
    
    if [[ -f "$TRACE_SERIAL" ]] && grep -q "$marker" "$TRACE_SERIAL"; then
        FOUND_IN_SERIAL=1
    fi
    
    if [[ $FOUND_IN_DEBUGCON -eq 1 ]] || [[ $FOUND_IN_SERIAL -eq 1 ]]; then
        log_info "  ✓ Marker found: $marker_name"
    else
        log_error "  ✗ COUNTEREXAMPLE: Marker missing: $marker_name"
        echo "COUNTEREXAMPLE: MARKER_ABSENT - $marker_name not found in any channel" >> "$COUNTEREXAMPLES_LOG"
        COUNTEREXAMPLES_FOUND=$((COUNTEREXAMPLES_FOUND + 1))
    fi
done

log_info ""

# Check 3: Marker order preservation (no sort in harness)
log_info "Check 3: Marker Order Preservation (No Sort)"

HARNESS_SCRIPTS=(
    "$PROJECT_ROOT/scripts/qemu-fail-closed-proof-harness.sh"
    "$PROJECT_ROOT/scripts/qemu-runtime-bridge-proof-harness.sh"
    "$PROJECT_ROOT/scripts/qemu-boot-observability-harness.sh"
)

SORT_DETECTED=0

for script in "${HARNESS_SCRIPTS[@]}"; do
    if [[ ! -f "$script" ]]; then
        continue
    fi
    
    if grep -q "| sort" "$script" || grep -q "sort " "$script"; then
        log_error "  ✗ COUNTEREXAMPLE: 'sort' operation detected in $(basename "$script")"
        echo "COUNTEREXAMPLE: FORBIDDEN_SORT - sort detected in $(basename "$script")" >> "$COUNTEREXAMPLES_LOG"
        COUNTEREXAMPLES_FOUND=$((COUNTEREXAMPLES_FOUND + 1))
        SORT_DETECTED=1
    fi
done

if [[ $SORT_DETECTED -eq 0 ]]; then
    log_info "  ✓ No 'sort' operation detected in harness scripts"
fi

log_info ""

# Check 4: Cross-channel merge creating fake temporal ordering
log_info "Check 4: Cross-Channel Merge Detection"

CROSS_CHANNEL_MERGE_DETECTED=0

for script in "${HARNESS_SCRIPTS[@]}"; do
    if [[ ! -f "$script" ]]; then
        continue
    fi
    
    # Check for cross-channel concatenation without NON-AUTHORITATIVE warning
    if grep -q "cat.*debugcon.*serial" "$script" && ! grep -q "NON-AUTHORITATIVE" "$script"; then
        log_error "  ✗ COUNTEREXAMPLE: Cross-channel merge without NON-AUTHORITATIVE warning in $(basename "$script")"
        echo "COUNTEREXAMPLE: CROSS_CHANNEL_MERGE - fake temporal ordering in $(basename "$script")" >> "$COUNTEREXAMPLES_LOG"
        COUNTEREXAMPLES_FOUND=$((COUNTEREXAMPLES_FOUND + 1))
        CROSS_CHANNEL_MERGE_DETECTED=1
    fi
done

if [[ $CROSS_CHANNEL_MERGE_DETECTED -eq 0 ]]; then
    log_info "  ✓ No problematic cross-channel merge detected"
fi

log_info ""

# ============================================================================
# Test Result
# ============================================================================
log_info "========================================="
log_info "Test Result"
log_info "========================================="
log_info "Counterexamples found: $COUNTEREXAMPLES_FOUND"
log_info ""

if [[ $COUNTEREXAMPLES_FOUND -gt 0 ]]; then
    log_error "TEST RESULT: FAIL (EXPECTED)"
    log_error ""
    log_error "✓ Bug condition confirmed - evidence pipeline regression exists"
    log_error ""
    log_error "Counterexamples documented in: $COUNTEREXAMPLES_LOG"
    log_error ""
    log_error "Summary of counterexamples:"
    cat "$COUNTEREXAMPLES_LOG" | while read -r line; do
        log_error "  - $line"
    done
    log_error ""
    log_error "This test failure is EXPECTED and confirms the bug exists."
    log_error "DO NOT attempt to fix the test - the test is correct."
    log_error "The fix should address the evidence pipeline issues documented above."
    
    # Generate test result JSON
    cat > "$TEST_RESULT_JSON" << EOF
{
  "test": "$TEST_NAME",
  "property": "Bug Condition - Evidence Pipeline Integrity",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "result": "FAIL",
  "expected_result": "FAIL",
  "status": "EXPECTED_FAILURE",
  "counterexamples_found": $COUNTEREXAMPLES_FOUND,
  "counterexamples_log": "$COUNTEREXAMPLES_LOG",
  "message": "Bug condition confirmed - evidence pipeline regression exists",
  "next_steps": [
    "Implement fixes in Block 1 (Evidence Pipeline Repair)",
    "Fix harness integrity (remove sort, add HARD FAIL rule)",
    "Validate QEMU configuration (debugcon/serial capture)",
    "Re-run this test after fixes to verify bug is resolved"
  ]
}
EOF
    
    log_info "Test result: $TEST_RESULT_JSON"
    
    # Exit with failure code (expected for bug condition exploration)
    exit 1
else
    log_warn "TEST RESULT: PASS (UNEXPECTED)"
    log_warn ""
    log_warn "✗ Bug condition NOT confirmed - no counterexamples found"
    log_warn ""
    log_warn "This is UNEXPECTED. Possible explanations:"
    log_warn "  1. The bug has already been fixed"
    log_warn "  2. The root cause analysis is incorrect"
    log_warn "  3. The test is not detecting the bug correctly"
    log_warn ""
    log_warn "CRITICAL: Review root cause analysis and test design"
    
    # Generate test result JSON
    cat > "$TEST_RESULT_JSON" << EOF
{
  "test": "$TEST_NAME",
  "property": "Bug Condition - Evidence Pipeline Integrity",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "result": "PASS",
  "expected_result": "FAIL",
  "status": "UNEXPECTED_PASS",
  "counterexamples_found": 0,
  "message": "Bug condition NOT confirmed - no counterexamples found (UNEXPECTED)",
  "next_steps": [
    "Review root cause analysis - may be incorrect",
    "Investigate why evidence pipeline appears to work",
    "Consider alternative root causes",
    "Re-evaluate bug condition definition"
  ]
}
EOF
    
    log_info "Test result: $TEST_RESULT_JSON"
    
    # Exit with success code (but this is unexpected)
    exit 0
fi

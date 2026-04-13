#!/bin/bash
# Runtime_Bridge Syscall Path Audit Script
# Phase-16 Task 5: Validates Runtime_Bridge-specific marker flow

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
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

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <qemu_trace_log>"
    exit 1
fi

TRACE_LOG="$1"

if [[ ! -f "$TRACE_LOG" ]]; then
    log_error "Trace log not found: $TRACE_LOG"
    exit 1
fi

log_info "Auditing Runtime_Bridge syscall path: $TRACE_LOG"

# Extract userspace payload output (which is interleaved with [[AYKEN_ markers)
PAYLOAD_OUTPUT=$(python3 -c "
import sys, re
try:
    with open('$TRACE_LOG') as f:
        text = f.read()
    # Match any characters between P10_SYSCALL_ENTER and the next [[AYKEN_ marker
    matches = re.findall(r'P10_SYSCALL_ENTER\n(.*?)\[\[AYKEN_', text)
    print(''.join(matches))
except Exception as e:
    print('')
")

# Create a temporary file for the payload output to make grepping easier
TMP_PAYLOAD=$(mktemp)
echo "$PAYLOAD_OUTPUT" > "$TMP_PAYLOAD"

# Check for required markers in payload output
MARKER_START=$(grep -c "\[U\]\[RUNTIME_BRIDGE_TEST_START\]" "$TMP_PAYLOAD" 2>/dev/null || echo "0")
MARKER_DEVICE_BEFORE=$(grep -c "\[U\]\[RUNTIME_BRIDGE_DEVICE_OP_BEFORE\]" "$TMP_PAYLOAD" 2>/dev/null || echo "0")
MARKER_DEVICE_AFTER=$(grep -c "\[U\]\[RUNTIME_BRIDGE_DEVICE_OP_AFTER\]" "$TMP_PAYLOAD" 2>/dev/null || echo "0")
MARKER_EXTERNAL_BEFORE=$(grep -c "\[U\]\[RUNTIME_BRIDGE_EXTERNAL_CALL_BEFORE\]" "$TMP_PAYLOAD" 2>/dev/null || echo "0")
MARKER_EXTERNAL_AFTER=$(grep -c "\[U\]\[RUNTIME_BRIDGE_EXTERNAL_CALL_AFTER\]" "$TMP_PAYLOAD" 2>/dev/null || echo "0")
MARKER_ABDF_BEFORE=$(grep -c "\[U\]\[RUNTIME_BRIDGE_ABDF_OP_BEFORE\]" "$TMP_PAYLOAD" 2>/dev/null || echo "0")
MARKER_ABDF_AFTER=$(grep -c "\[U\]\[RUNTIME_BRIDGE_ABDF_OP_AFTER\]" "$TMP_PAYLOAD" 2>/dev/null || echo "0")
MARKER_COMPLETE=$(grep -c "\[U\]\[RUNTIME_BRIDGE_TEST_COMPLETE\]" "$TMP_PAYLOAD" 2>/dev/null || echo "0")

rm -f "$TMP_PAYLOAD"

# Check for kernel syscall markers in the raw trace
SYSCALL_ENTER=$(grep -c "\[\[AYKEN_SYSCALL_ENTER\]\]" "$TRACE_LOG" 2>/dev/null || echo "0")
SYSCALL_EXIT=$(grep -c "\[\[AYKEN_SYSCALL_RETURN\]\]" "$TRACE_LOG" 2>/dev/null || echo "0")

# Sanitize counts (remove any whitespace/newlines)
MARKER_START=$(echo "$MARKER_START" | tr -d '[:space:]')
MARKER_DEVICE_BEFORE=$(echo "$MARKER_DEVICE_BEFORE" | tr -d '[:space:]')
MARKER_DEVICE_AFTER=$(echo "$MARKER_DEVICE_AFTER" | tr -d '[:space:]')
MARKER_EXTERNAL_BEFORE=$(echo "$MARKER_EXTERNAL_BEFORE" | tr -d '[:space:]')
MARKER_EXTERNAL_AFTER=$(echo "$MARKER_EXTERNAL_AFTER" | tr -d '[:space:]')
MARKER_ABDF_BEFORE=$(echo "$MARKER_ABDF_BEFORE" | tr -d '[:space:]')
MARKER_ABDF_AFTER=$(echo "$MARKER_ABDF_AFTER" | tr -d '[:space:]')
MARKER_COMPLETE=$(echo "$MARKER_COMPLETE" | tr -d '[:space:]')
SYSCALL_ENTER=$(echo "$SYSCALL_ENTER" | tr -d '[:space:]')
SYSCALL_EXIT=$(echo "$SYSCALL_EXIT" | tr -d '[:space:]')

log_info "Marker counts:"
log_info "  TEST_START: $MARKER_START"
log_info "  DEVICE_OP_BEFORE: $MARKER_DEVICE_BEFORE"
log_info "  DEVICE_OP_AFTER: $MARKER_DEVICE_AFTER"
log_info "  EXTERNAL_CALL_BEFORE: $MARKER_EXTERNAL_BEFORE"
log_info "  EXTERNAL_CALL_AFTER: $MARKER_EXTERNAL_AFTER"
log_info "  ABDF_OP_BEFORE: $MARKER_ABDF_BEFORE"
log_info "  ABDF_OP_AFTER: $MARKER_ABDF_AFTER"
log_info "  TEST_COMPLETE: $MARKER_COMPLETE"
log_info "  SYSCALL_ENTER: $SYSCALL_ENTER"
log_info "  SYSCALL_EXIT: $SYSCALL_EXIT"

# Validation
PASS=true

if [[ $MARKER_START -eq 0 ]]; then
    log_error "✗ Missing RUNTIME_BRIDGE_TEST_START marker"
    PASS=false
fi

if [[ $MARKER_DEVICE_BEFORE -eq 0 ]] || [[ $MARKER_DEVICE_AFTER -eq 0 ]]; then
    log_error "✗ Missing DEVICE_OP markers (BEFORE: $MARKER_DEVICE_BEFORE, AFTER: $MARKER_DEVICE_AFTER)"
    PASS=false
fi

if [[ $MARKER_EXTERNAL_BEFORE -eq 0 ]] || [[ $MARKER_EXTERNAL_AFTER -eq 0 ]]; then
    log_error "✗ Missing EXTERNAL_CALL markers (BEFORE: $MARKER_EXTERNAL_BEFORE, AFTER: $MARKER_EXTERNAL_AFTER)"
    PASS=false
fi

if [[ $MARKER_ABDF_BEFORE -eq 0 ]] || [[ $MARKER_ABDF_AFTER -eq 0 ]]; then
    log_error "✗ Missing ABDF_OP markers (BEFORE: $MARKER_ABDF_BEFORE, AFTER: $MARKER_ABDF_AFTER)"
    PASS=false
fi

if [[ $MARKER_COMPLETE -eq 0 ]]; then
    log_error "✗ Missing RUNTIME_BRIDGE_TEST_COMPLETE marker"
    PASS=false
fi

# Expect at least 3 syscall enter/exit pairs (1012, 1013, 1014)
if [[ $SYSCALL_ENTER -lt 3 ]]; then
    log_error "✗ Expected at least 3 SYSCALL_ENTER markers, found $SYSCALL_ENTER"
    PASS=false
fi

if [[ $SYSCALL_EXIT -lt 3 ]]; then
    log_error "✗ Expected at least 3 SYSCALL_EXIT markers, found $SYSCALL_EXIT"
    PASS=false
fi

# Final verdict
if [[ "$PASS" == "true" ]]; then
    log_info "✓ Runtime_Bridge syscall path validation: PASS"
    log_info "  All required markers present"
    log_info "  Test execution completed successfully"
    exit 0
else
    log_error "✗ Runtime_Bridge syscall path validation: FAIL"
    log_error "  Missing required markers (see errors above)"
    exit 1
fi
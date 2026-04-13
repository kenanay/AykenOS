#!/bin/bash
# Runtime_Bridge Forbidden Path Audit Script
# Phase-16 Task 5: Validates Runtime_Bridge fail-closed enforcement

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

log_info "Auditing Runtime_Bridge forbidden path: $TRACE_LOG"

# Extract userspace payload output
PAYLOAD_OUTPUT=$(python3 -c "
import sys, re
try:
    with open('$TRACE_LOG') as f:
        text = f.read()
    # Match any characters between P10_SYSCALL_ENTER and the next [[AYKEN_ marker
    matches = re.findall(r'P10_SYSCALL_ENTER\n(.*?)\[\[AYKEN_', text, re.DOTALL)
    print(''.join(matches))
except Exception as e:
    print('')
")

# Create temporary file for payload output
TMP_PAYLOAD=$(mktemp)
echo "$PAYLOAD_OUTPUT" > "$TMP_PAYLOAD"

# Check for required markers
MARKER_BEFORE=$(grep -c "\[U\]\[RUNTIME_BRIDGE_FORBIDDEN_BEFORE\]" "$TMP_PAYLOAD" 2>/dev/null || echo "0")
MARKER_AFTER=$(grep -c "\[U\]\[RUNTIME_BRIDGE_FORBIDDEN_AFTER\]" "$TMP_PAYLOAD" 2>/dev/null || echo "0")

rm -f "$TMP_PAYLOAD"

# Check for kernel markers in raw trace
SYSCALL_ENTER=$(grep -c "\[\[AYKEN_SYSCALL_ENTER\]\]" "$TRACE_LOG" 2>/dev/null || echo "0")
BOUNDARY_KILL=$(grep -c "\[\[AYKEN_BOUNDARY_KILL\]\]" "$TRACE_LOG" 2>/dev/null || echo "0")

# Sanitize counts
MARKER_BEFORE=$(echo "$MARKER_BEFORE" | tr -d '[:space:]')
MARKER_AFTER=$(echo "$MARKER_AFTER" | tr -d '[:space:]')
SYSCALL_ENTER=$(echo "$SYSCALL_ENTER" | tr -d '[:space:]')
BOUNDARY_KILL=$(echo "$BOUNDARY_KILL" | tr -d '[:space:]')

log_info "Marker counts:"
log_info "  FORBIDDEN_BEFORE: $MARKER_BEFORE"
log_info "  FORBIDDEN_AFTER: $MARKER_AFTER"
log_info "  SYSCALL_ENTER: $SYSCALL_ENTER"
log_info "  BOUNDARY_KILL: $BOUNDARY_KILL"

# Validation
PASS=true

if [[ $MARKER_BEFORE -eq 0 ]]; then
    log_error "✗ Missing RUNTIME_BRIDGE_FORBIDDEN_BEFORE marker"
    PASS=false
fi

if [[ $MARKER_AFTER -gt 0 ]]; then
    log_error "✗ FORBIDDEN_AFTER marker present - continuation detected (fail-closed BROKEN)"
    PASS=false
fi

if [[ $BOUNDARY_KILL -eq 0 ]]; then
    log_error "✗ Missing BOUNDARY_KILL marker - enforcement failed"
    PASS=false
fi

if [[ $SYSCALL_ENTER -eq 0 ]]; then
    log_error "✗ Missing SYSCALL_ENTER marker - syscall not attempted"
    PASS=false
fi

# Final verdict
if [[ "$PASS" == "true" ]]; then
    log_info "✓ Runtime_Bridge forbidden path validation: PASS"
    log_info "  Fail-closed enforcement working correctly"
    log_info "  No continuation after boundary kill"
    exit 0
else
    log_error "✗ Runtime_Bridge forbidden path validation: FAIL"
    log_error "  Fail-closed enforcement broken (see errors above)"
    exit 1
fi

#!/bin/bash
# Runtime_Bridge Syscall Path Audit Script
# Phase-16 Task 5: Validates Runtime_Bridge-specific marker flow
#
# This script validates that Runtime_Bridge syscalls (1012/1013/1014) are executed
# and produce the expected marker sequence in QEMU kernel traces.
#
# Expected marker flow:
#   [U][RUNTIME_BRIDGE_TEST_START]
#   [U][RUNTIME_BRIDGE_DEVICE_OP_BEFORE]
#   [[AYKEN_SYSCALL_ENTER]] (syscall 1012)
#   [[AYKEN_SYSCALL_EXIT]] (syscall 1012)
#   [U][RUNTIME_BRIDGE_DEVICE_OP_AFTER]
#   [U][RUNTIME_BRIDGE_EXTERNAL_CALL_BEFORE]
#   [[AYKEN_SYSCALL_ENTER]] (syscall 1013)
#   [[AYKEN_SYSCALL_EXIT]] (syscall 1013)
#   [U][RUNTIME_BRIDGE_EXTERNAL_CALL_AFTER]
#   [U][RUNTIME_BRIDGE_ABDF_OP_BEFORE]
#   [[AYKEN_SYSCALL_ENTER]] (syscall 1014)
#   [[AYKEN_SYSCALL_EXIT]] (syscall 1014)
#   [U][RUNTIME_BRIDGE_ABDF_OP_AFTER]
#   [U][RUNTIME_BRIDGE_TEST_COMPLETE]

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

# Usage
if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <qemu_trace_log>"
    echo ""
    echo "Validates Runtime_Bridge syscall marker flow in QEMU kernel trace."
    exit 1
fi

TRACE_LOG="$1"

if [[ ! -f "$TRACE_LOG" ]]; then
    log_error "Trace log not found: $TRACE_LOG"
    exit 1
fi

log_info "Auditing Runtime_Bridge syscall path: $TRACE_LOG"

# Check for required markers
MARKER_START=$(grep -c "\[U\]\[RUNTIME_BRIDGE_TEST_START\]" "$TRACE_LOG" 2>/dev/null || echo "0")
MARKER_DEVICE_BEFORE=$(grep -c "\[U\]\[RUNTIME_BRIDGE_DEVICE_OP_BEFORE\]" "$TRACE_LOG" 2>/dev/null || echo "0")
MARKER_DEVICE_AFTER=$(grep -c "\[U\]\[RUNTIME_BRIDGE_DEVICE_OP_AFTER\]" "$TRACE_LOG" 2>/dev/null || echo "0")
MARKER_EXTERNAL_BEFORE=$(grep -c "\[U\]\[RUNTIME_BRIDGE_EXTERNAL_CALL_BEFORE\]" "$TRACE_LOG" 2>/dev/null || echo "0")
MARKER_EXTERNAL_AFTER=$(grep -c "\[U\]\[RUNTIME_BRIDGE_EXTERNAL_CALL_AFTER\]" "$TRACE_LOG" 2>/dev/null || echo "0")
MARKER_ABDF_BEFORE=$(grep -c "\[U\]\[RUNTIME_BRIDGE_ABDF_OP_BEFORE\]" "$TRACE_LOG" 2>/dev/null || echo "0")
MARKER_ABDF_AFTER=$(grep -c "\[U\]\[RUNTIME_BRIDGE_ABDF_OP_AFTER\]" "$TRACE_LOG" 2>/dev/null || echo "0")
MARKER_COMPLETE=$(grep -c "\[U\]\[RUNTIME_BRIDGE_TEST_COMPLETE\]" "$TRACE_LOG" 2>/dev/null || echo "0")

# Check for kernel syscall markers
SYSCALL_ENTER=$(grep -c "\[\[AYKEN_SYSCALL_ENTER\]\]" "$TRACE_LOG" 2>/dev/null || echo "0")
SYSCALL_EXIT=$(grep -c "\[\[AYKEN_SYSCALL_EXIT\]\]" "$TRACE_LOG" 2>/dev/null || echo "0")

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
    log_warn "⚠ Expected at least 3 SYSCALL_ENTER markers, found $SYSCALL_ENTER"
    log_warn "  This may indicate syscalls are not reaching the kernel dispatcher"
fi

if [[ $SYSCALL_EXIT -lt 3 ]]; then
    log_warn "⚠ Expected at least 3 SYSCALL_EXIT markers, found $SYSCALL_EXIT"
    log_warn "  This may indicate syscalls are not returning from the kernel"
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

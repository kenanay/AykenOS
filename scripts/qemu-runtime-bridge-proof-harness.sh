#!/bin/bash
# QEMU Runtime_Bridge Proof Harness
# Phase-16 Task 5: Runtime_Bridge Syscall Path Evidence Generation
#
# This harness generates QEMU kernel trace evidence for Runtime_Bridge role enforcement:
# 1. Allowed path: 1012/1013/1014 syscalls succeed
# 2. Forbidden path: 1003 syscall triggers fail-closed termination
#
# Output: evidence/runtime-bridge-proof/qemu_kernel_trace_*.log

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
EVIDENCE_DIR="$PROJECT_ROOT/evidence/runtime-bridge-proof"
BUILD_DIR="$PROJECT_ROOT/build/runtime-bridge-tests"

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

# Create evidence directory
mkdir -p "$EVIDENCE_DIR"

# Check if test binaries exist
if [[ ! -f "$BUILD_DIR/runtime_bridge_allowed_test.elf" ]] || [[ ! -f "$BUILD_DIR/runtime_bridge_forbidden_test.elf" ]]; then
    log_warn "Runtime_Bridge test binaries not found, building..."
    "$SCRIPT_DIR/build-runtime-bridge-tests.sh"
fi

# Check if kernel exists
KERNEL_ELF="$PROJECT_ROOT/build/kernel.elf"
if [[ ! -f "$KERNEL_ELF" ]]; then
    log_error "Kernel not found: $KERNEL_ELF"
    log_error "Run 'make kernel' first"
    exit 1
fi

log_info "Starting Runtime_Bridge QEMU proof harness..."

# Test 1: Allowed Path (1012/1013/1014)
log_info "Test 1: Runtime_Bridge allowed syscalls (1012/1013/1014)..."

ALLOWED_DEBUGCON="$EVIDENCE_DIR/qemu_allowed_debugcon.log"
ALLOWED_SERIAL="$EVIDENCE_DIR/qemu_allowed_serial.log"
ALLOWED_TRACE="$EVIDENCE_DIR/qemu_kernel_trace_allowed.log"

# Launch QEMU with allowed test
timeout 5s qemu-system-x86_64 \
    -kernel "$KERNEL_ELF" \
    -initrd "$BUILD_DIR/runtime_bridge_allowed_test.elf" \
    -append "execution_role=PROC_EXECUTION_ROLE_RUNTIME_BRIDGE" \
    -nographic \
    -debugcon file:"$ALLOWED_DEBUGCON" \
    -serial file:"$ALLOWED_SERIAL" \
    -no-reboot \
    -no-shutdown \
    > /dev/null 2>&1 || true

# Merge outputs
cat "$ALLOWED_DEBUGCON" "$ALLOWED_SERIAL" 2>/dev/null | sort > "$ALLOWED_TRACE" || true

log_info "Allowed path trace: $ALLOWED_TRACE"

# Analyze allowed trace
MARKER_BEFORE=$(grep -c "RUNTIME_BRIDGE_ALLOWED_BEFORE" "$ALLOWED_TRACE" || echo "0")
MARKER_AFTER=$(grep -c "RUNTIME_BRIDGE_ALLOWED_AFTER" "$ALLOWED_TRACE" || echo "0")
SYSCALL_ENTER=$(grep -c "\[\[AYKEN_SYSCALL_ENTER\]\]" "$ALLOWED_TRACE" || echo "0")
SYSCALL_EXIT=$(grep -c "\[\[AYKEN_SYSCALL_EXIT\]\]" "$ALLOWED_TRACE" || echo "0")

log_info "Allowed path markers:"
log_info "  BEFORE: $MARKER_BEFORE"
log_info "  AFTER: $MARKER_AFTER"
log_info "  SYSCALL_ENTER: $SYSCALL_ENTER"
log_info "  SYSCALL_EXIT: $SYSCALL_EXIT"

if [[ $MARKER_BEFORE -gt 0 ]] && [[ $MARKER_AFTER -gt 0 ]]; then
    log_info "✓ Allowed path: execution continued (PASS)"
else
    log_warn "✗ Allowed path: execution did not complete (check trace)"
fi

# Test 2: Forbidden Path (1003)
log_info ""
log_info "Test 2: Runtime_Bridge forbidden syscall (1003)..."

FORBIDDEN_DEBUGCON="$EVIDENCE_DIR/qemu_forbidden_debugcon.log"
FORBIDDEN_SERIAL="$EVIDENCE_DIR/qemu_forbidden_serial.log"
FORBIDDEN_TRACE="$EVIDENCE_DIR/qemu_kernel_trace_forbidden.log"

# Launch QEMU with forbidden test
timeout 5s qemu-system-x86_64 \
    -kernel "$KERNEL_ELF" \
    -initrd "$BUILD_DIR/runtime_bridge_forbidden_test.elf" \
    -append "execution_role=PROC_EXECUTION_ROLE_RUNTIME_BRIDGE" \
    -nographic \
    -debugcon file:"$FORBIDDEN_DEBUGCON" \
    -serial file:"$FORBIDDEN_SERIAL" \
    -no-reboot \
    -no-shutdown \
    > /dev/null 2>&1 || true

# Merge outputs
cat "$FORBIDDEN_DEBUGCON" "$FORBIDDEN_SERIAL" 2>/dev/null | sort > "$FORBIDDEN_TRACE" || true

log_info "Forbidden path trace: $FORBIDDEN_TRACE"

# Analyze forbidden trace
MARKER_BEFORE=$(grep -c "RUNTIME_BRIDGE_FORBIDDEN_BEFORE" "$FORBIDDEN_TRACE" || echo "0")
MARKER_AFTER=$(grep -c "RUNTIME_BRIDGE_FORBIDDEN_AFTER" "$FORBIDDEN_TRACE" || echo "0")
MARKER_KILL=$(grep -c "\[\[AYKEN_BOUNDARY_KILL\]\]" "$FORBIDDEN_TRACE" || echo "0")

log_info "Forbidden path markers:"
log_info "  BEFORE: $MARKER_BEFORE"
log_info "  AFTER: $MARKER_AFTER (should be 0)"
log_info "  BOUNDARY_KILL: $MARKER_KILL"

if [[ $MARKER_BEFORE -gt 0 ]] && [[ $MARKER_KILL -gt 0 ]] && [[ $MARKER_AFTER -eq 0 ]]; then
    log_info "✓ Forbidden path: fail-closed termination (PASS)"
else
    log_warn "✗ Forbidden path: fail-closed may be broken (check trace)"
fi

log_info ""
log_info "Runtime_Bridge QEMU proof harness complete"
log_info "Evidence directory: $EVIDENCE_DIR"
log_info ""
log_info "Next step: Run ci-gate-fail-closed-proof on forbidden trace"
log_info "  ./scripts/ci-gate-fail-closed-proof.sh"

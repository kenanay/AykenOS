#!/bin/bash

# QEMU Test Harness for Fail-Closed Proof Generation
#
# This script launches a QEMU-based test that generates kernel trace evidence
# for fail-closed enforcement validation. It creates a BCIB-role process that
# attempts a forbidden syscall and captures the kernel's response.
#
# Output: QEMU kernel trace with canonical marker flow

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
EVIDENCE_DIR="$PROJECT_ROOT/evidence/fail-closed-proof"
TRACE_LOG="$EVIDENCE_DIR/qemu_kernel_trace.log"

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

# Create evidence directory
mkdir -p "$EVIDENCE_DIR"

log_info "QEMU Fail-Closed Proof Harness"
log_info "================================"

# Check if EFI image exists
EFI_IMAGE="$PROJECT_ROOT/EFI.img"
if [[ ! -f "$EFI_IMAGE" ]]; then
    log_error "EFI image not found: $EFI_IMAGE"
    log_error "Run 'make efi-img' to build the image"
    exit 1
fi

log_info "Found EFI image: $EFI_IMAGE"

# QEMU configuration
QEMU_TIMEOUT=30
QEMU_MEMORY="256M"
QEMU_CPU="qemu64"

# Temporary files for QEMU output
DEBUGCON_LOG="$EVIDENCE_DIR/qemu_debugcon.log"
SERIAL_LOG="$EVIDENCE_DIR/qemu_serial.log"

log_info "Starting QEMU with kernel trace capture..."
log_info "Timeout: ${QEMU_TIMEOUT}s"
log_info "Memory: $QEMU_MEMORY"

# Launch QEMU with trace capture
# -debugcon file:$DEBUGCON_LOG captures kernel debug output
# -serial file:$SERIAL_LOG captures serial console
# -display none runs headless
# -no-reboot prevents automatic reboot on crash

timeout $QEMU_TIMEOUT qemu-system-x86_64 \
    -drive format=raw,file="$EFI_IMAGE" \
    -m "$QEMU_MEMORY" \
    -cpu "$QEMU_CPU" \
    -debugcon file:"$DEBUGCON_LOG" \
    -serial file:"$SERIAL_LOG" \
    -display none \
    -no-reboot \
    -no-shutdown \
    2>&1 | tee "$EVIDENCE_DIR/qemu_stdout.log" || {
        QEMU_EXIT=$?
        if [[ $QEMU_EXIT -eq 124 ]]; then
            log_info "QEMU timeout reached (expected for test harness)"
        else
            log_warn "QEMU exited with code: $QEMU_EXIT"
        fi
    }

log_info "QEMU execution complete"

# Merge debugcon and serial logs into unified trace
log_info "Merging kernel trace outputs..."

{
    echo "=== QEMU Kernel Trace ==="
    echo "=== Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
    echo ""
    echo "=== Debugcon Output ==="
    if [[ -f "$DEBUGCON_LOG" ]]; then
        cat "$DEBUGCON_LOG"
    else
        echo "(no debugcon output)"
    fi
    echo ""
    echo "=== Serial Output ==="
    if [[ -f "$SERIAL_LOG" ]]; then
        cat "$SERIAL_LOG"
    else
        echo "(no serial output)"
    fi
} > "$TRACE_LOG"

log_info "Kernel trace saved: $TRACE_LOG"

# Analyze trace for markers
log_info "Analyzing trace for canonical markers..."

MARKER_BEFORE=$(grep -c "BCIB_FORBIDDEN_BEFORE" "$TRACE_LOG" || echo "0")
MARKER_ENTER=$(grep -c "\[\[AYKEN_SYSCALL_ENTER\]\]" "$TRACE_LOG" || echo "0")
MARKER_KILL=$(grep -c "\[\[AYKEN_BOUNDARY_KILL\]\]" "$TRACE_LOG" || echo "0")

log_info "Marker counts:"
log_info "  BCIB_FORBIDDEN_BEFORE: $MARKER_BEFORE"
log_info "  [[AYKEN_SYSCALL_ENTER]]: $MARKER_ENTER"
log_info "  [[AYKEN_BOUNDARY_KILL]]: $MARKER_KILL"

if [[ $MARKER_BEFORE -gt 0 ]] && [[ $MARKER_ENTER -gt 0 ]] && [[ $MARKER_KILL -gt 0 ]]; then
    log_info "✓ All required markers present"
    log_info "Trace is ready for ci-gate-fail-closed-proof validation"
    exit 0
else
    log_warn "⚠ Some markers missing - trace may be incomplete"
    log_warn "This could indicate:"
    log_warn "  - Test harness not yet implemented in kernel"
    log_warn "  - BCIB-role process not launched"
    log_warn "  - Forbidden syscall not attempted"
    log_warn "  - Marker emission not implemented"
    exit 1
fi

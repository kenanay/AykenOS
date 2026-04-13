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

resolve_ovmf_firmware() {
    # Known firmware locations across macOS/Linux
    local candidates=(
        "$PROJECT_ROOT/firmware/ovmf/OVMF_CODE.fd|$PROJECT_ROOT/firmware/ovmf/OVMF_VARS.fd"
        "/usr/share/OVMF/OVMF_CODE_4M.fd|/usr/share/OVMF/OVMF_VARS_4M.fd"
        "/usr/share/OVMF/OVMF_CODE.fd|/usr/share/OVMF/OVMF_VARS.fd"
        "/usr/share/edk2/ovmf/OVMF_CODE.fd|/usr/share/edk2/ovmf/OVMF_VARS.fd"
        "/usr/share/qemu/OVMF_CODE.fd|/usr/share/qemu/OVMF_VARS.fd"
        "/opt/homebrew/share/qemu/edk2-x86_64-code.fd|/opt/homebrew/share/qemu/edk2-x86_64-vars.fd"
    )

    local entry code vars
    for entry in "${candidates[@]}"; do
        code="${entry%%|*}"
        vars="${entry##*|}"
        if [[ -f "${code}" && -f "${vars}" ]]; then
            printf "%s\n%s\n" "${code}" "${vars}"
            return 0
        fi
    done

    return 1
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

# Resolve OVMF firmware
OVMF_PAIR="$(resolve_ovmf_firmware || true)"
if [[ -z "$OVMF_PAIR" ]]; then
    log_error "OVMF firmware not found"
    log_error "Install OVMF package (e.g., 'apt install ovmf' or 'brew install qemu')"
    exit 1
fi

OVMF_CODE="$(printf "%s\n" "$OVMF_PAIR" | sed -n '1p')"
OVMF_VARS="$(printf "%s\n" "$OVMF_PAIR" | sed -n '2p')"

log_info "Using OVMF CODE: $OVMF_CODE"
log_info "Using OVMF VARS: $OVMF_VARS"

# Create temporary directory for this run
RUN_TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/fail_closed_run.XXXXXX" 2>/dev/null || mktemp -d -t fail_closed_run 2>/dev/null)"
if [[ -z "$RUN_TMP_DIR" || ! -d "$RUN_TMP_DIR" ]]; then
    log_error "Failed to create temporary directory"
    exit 1
fi

# Prepare OVMF VARS copy (blank varstore for deterministic boot)
OVMF_VARS_COPY="$RUN_TMP_DIR/OVMF_VARS.fd"
OVMF_VARS_SIZE="$(wc -c < "$OVMF_VARS" 2>/dev/null | tr -d '[:space:]')"
dd if=/dev/zero of="$OVMF_VARS_COPY" bs=1 count="$OVMF_VARS_SIZE" >/dev/null 2>&1

# Prepare EFI image copy (avoid write-lock contention)
EFI_IMG_RUN="$RUN_TMP_DIR/EFI.img"
cp -f "$EFI_IMAGE" "$EFI_IMG_RUN"

# QEMU configuration
QEMU_TIMEOUT=30
QEMU_MEMORY="256M"

# Temporary files for QEMU output
DEBUGCON_LOG="$EVIDENCE_DIR/qemu_debugcon.log"
SERIAL_LOG="$EVIDENCE_DIR/qemu_serial.log"

log_info "Starting QEMU with kernel trace capture..."
log_info "Timeout: ${QEMU_TIMEOUT}s"
log_info "Memory: $QEMU_MEMORY"

# Launch QEMU with OVMF + EFI.img (correct boot path)
timeout $QEMU_TIMEOUT qemu-system-x86_64 \
    -machine q35 \
    -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
    -drive if=pflash,format=raw,file="$OVMF_VARS_COPY" \
    -drive format=raw,file="$EFI_IMG_RUN" \
    -serial file:"$SERIAL_LOG" \
    -chardev file,id=dbgcon,path="$DEBUGCON_LOG" \
    -device isa-debugcon,iobase=0xe9,chardev=dbgcon \
    -m "$QEMU_MEMORY" \
    -no-reboot \
    -no-shutdown \
    -display none \
    > /dev/null 2>&1 || {
        QEMU_EXIT=$?
        if [[ $QEMU_EXIT -eq 124 ]]; then
            log_info "QEMU timeout reached (expected for test harness)"
        else
            log_warn "QEMU exited with code: $QEMU_EXIT"
        fi
    }

log_info "QEMU execution complete"

# Cleanup temporary directory
rm -rf "$RUN_TMP_DIR"

# Channel integrity validation (HARD FAIL rule)
# NOTE: Task 1 scope = debugcon + serial only
# UEFI output validation will be added in Block 2/3 for bootloader execution diagnosis
log_info "Validating output channel integrity..."

DEBUGCON_SIZE=0
SERIAL_SIZE=0

if [[ -f "$DEBUGCON_LOG" ]]; then
    DEBUGCON_SIZE=$(stat -c%s "$DEBUGCON_LOG" 2>/dev/null || stat -f%z "$DEBUGCON_LOG" 2>/dev/null || echo "0")
fi

if [[ -f "$SERIAL_LOG" ]]; then
    SERIAL_SIZE=$(stat -c%s "$SERIAL_LOG" 2>/dev/null || stat -f%z "$SERIAL_LOG" 2>/dev/null || echo "0")
fi

log_info "Channel sizes: debugcon=$DEBUGCON_SIZE bytes, serial=$SERIAL_SIZE bytes"

# HARD FAIL: All channels zero (Task 1 scope: debugcon + serial)
# UEFI fallback diagnosis will be implemented in Block 2 (bootloader markers)
if [[ $DEBUGCON_SIZE -eq 0 ]] && [[ $SERIAL_SIZE -eq 0 ]]; then
    log_error "OUTPUT_CHANNEL_FAILURE: All output channels are empty (debugcon + serial)"
    log_error "Cannot proceed with validation - no observable evidence"
    log_error "Possible causes:"
    log_error "  - QEMU debugcon/serial misconfiguration"
    log_error "  - Bootloader/kernel not emitting markers"
    log_error "  - Output capture path broken"
    log_error ""
    log_error "Next diagnosis step: Check UEFI Print output (Block 2)"
    exit 1
fi

# Keep separate channel-local traces (NO cross-channel merge)
TRACE_DEBUGCON="$EVIDENCE_DIR/debugcon.trace"
TRACE_SERIAL="$EVIDENCE_DIR/serial.trace"

log_info "Preserving channel-local traces (no cross-channel merge)..."

if [[ -f "$DEBUGCON_LOG" ]] && [[ $DEBUGCON_SIZE -gt 0 ]]; then
    cp "$DEBUGCON_LOG" "$TRACE_DEBUGCON"
    log_info "Debugcon trace: $TRACE_DEBUGCON"
fi

if [[ -f "$SERIAL_LOG" ]] && [[ $SERIAL_SIZE -gt 0 ]]; then
    cp "$SERIAL_LOG" "$TRACE_SERIAL"
    log_info "Serial trace: $TRACE_SERIAL"
fi

# Analyze markers in channel-local traces (NO sort, NO reorder)
log_info "Analyzing channel-local traces for canonical markers..."

MARKER_BEFORE=0
MARKER_ENTER=0
MARKER_KILL=0

# Check debugcon channel
if [[ -f "$TRACE_DEBUGCON" ]]; then
    BEFORE_COUNT=$(grep -c "BCIB_FORBIDDEN_BEFORE" "$TRACE_DEBUGCON" 2>/dev/null || echo "0")
    ENTER_COUNT=$(grep -c "\[\[AYKEN_SYSCALL_ENTER\]\]" "$TRACE_DEBUGCON" 2>/dev/null || echo "0")
    KILL_COUNT=$(grep -c "\[\[AYKEN_BOUNDARY_KILL\]\]" "$TRACE_DEBUGCON" 2>/dev/null || echo "0")
    MARKER_BEFORE=$((MARKER_BEFORE + BEFORE_COUNT))
    MARKER_ENTER=$((MARKER_ENTER + ENTER_COUNT))
    MARKER_KILL=$((MARKER_KILL + KILL_COUNT))
fi

# Check serial channel
if [[ -f "$TRACE_SERIAL" ]]; then
    BEFORE_COUNT=$(grep -c "BCIB_FORBIDDEN_BEFORE" "$TRACE_SERIAL" 2>/dev/null || echo "0")
    ENTER_COUNT=$(grep -c "\[\[AYKEN_SYSCALL_ENTER\]\]" "$TRACE_SERIAL" 2>/dev/null || echo "0")
    KILL_COUNT=$(grep -c "\[\[AYKEN_BOUNDARY_KILL\]\]" "$TRACE_SERIAL" 2>/dev/null || echo "0")
    MARKER_BEFORE=$((MARKER_BEFORE + BEFORE_COUNT))
    MARKER_ENTER=$((MARKER_ENTER + ENTER_COUNT))
    MARKER_KILL=$((MARKER_KILL + KILL_COUNT))
fi

log_info "Marker counts (channel-local aggregation):"
log_info "  BCIB_FORBIDDEN_BEFORE: $MARKER_BEFORE"
log_info "  [[AYKEN_SYSCALL_ENTER]]: $MARKER_ENTER"
log_info "  [[AYKEN_BOUNDARY_KILL]]: $MARKER_KILL"

# Create unified trace for human-readable reference ONLY (NOT authoritative)
# CRITICAL: This file is NON-AUTHORITATIVE and MUST NOT be used for temporal ordering
# Authoritative evidence: debugcon.trace and serial.trace (channel-local only)
{
    echo "=== QEMU Kernel Trace (NON-AUTHORITATIVE SUMMARY) ==="
    echo "=== Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
    echo "=== WARNING: This file is for human reference only ==="
    echo "=== Authoritative evidence: debugcon.trace and serial.trace ==="
    echo "=== DO NOT use this file for temporal ordering or CI gates ==="
    echo ""
    echo "=== Debugcon Output (raw append order) ==="
    if [[ -f "$TRACE_DEBUGCON" ]]; then
        cat "$TRACE_DEBUGCON"
    else
        echo "(no debugcon output)"
    fi
    echo ""
    echo "=== Serial Output (raw append order) ==="
    if [[ -f "$TRACE_SERIAL" ]]; then
        cat "$TRACE_SERIAL"
    else
        echo "(no serial output)"
    fi
} > "$TRACE_LOG"

log_info "Non-authoritative summary: $TRACE_LOG"
log_info "Authoritative evidence: $TRACE_DEBUGCON, $TRACE_SERIAL"

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

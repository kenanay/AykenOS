#!/bin/bash

# QEMU Boot Observability Test Harness
# 
# This harness generates boot chain observability evidence for CI gate validation.
# It launches QEMU with the EFI image and captures debugcon/serial output to
# evidence/boot-observability/ for ci-gate-boot-observability validation.
#
# Author: Kenan AY - Architectural Steward
# Spec: .kiro/specs/boot-chain-observability-restoration/
# Status: Non-architectural bugfix (architectural freeze compliant)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
EVIDENCE_DIR="$PROJECT_ROOT/evidence/boot-observability"

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

log_info "QEMU Boot Observability Test Harness"
log_info "====================================="

# Create evidence directory
mkdir -p "$EVIDENCE_DIR"

# Check if EFI image exists
EFI_IMAGE="$PROJECT_ROOT/EFI.img"
if [[ ! -f "$EFI_IMAGE" ]]; then
    log_error "EFI image not found: $EFI_IMAGE"
    log_error "Run 'make efi-img' to build the image"
    exit 1
fi

# Check for OVMF firmware
OVMF_CODE="$PROJECT_ROOT/firmware/ovmf/OVMF_CODE.fd"
OVMF_VARS_TEMPLATE="$PROJECT_ROOT/firmware/ovmf/OVMF_VARS.fd"
OVMF_VARS_RUN="$PROJECT_ROOT/build/OVMF_VARS_RUN.fd"

if [[ ! -f "$OVMF_CODE" ]]; then
    log_error "OVMF firmware not found: $OVMF_CODE"
    log_error "OVMF is required for UEFI boot"
    exit 1
fi

if [[ ! -f "$OVMF_VARS_TEMPLATE" ]]; then
    log_error "OVMF NVRAM template not found: $OVMF_VARS_TEMPLATE"
    exit 1
fi

log_info "Found EFI image: $EFI_IMAGE"
log_info "Found OVMF firmware: $OVMF_CODE"

# Prepare clean NVRAM (avoid BootOrder corruption)
mkdir -p "$(dirname "$OVMF_VARS_RUN")"
cp -f "$OVMF_VARS_TEMPLATE" "$OVMF_VARS_RUN"

# QEMU configuration
QEMU_TIMEOUT="${QEMU_TIMEOUT:-30}"
QEMU_MEMORY="256M"
QEMU_CPU="qemu64"

# Output files (canonical names for CI gate)
DEBUGCON_LOG="$EVIDENCE_DIR/qemu_debugcon.log"
SERIAL_LOG="$EVIDENCE_DIR/qemu_serial.log"
STDOUT_LOG="$EVIDENCE_DIR/qemu_stdout.log"

# Clean previous evidence
rm -f "$DEBUGCON_LOG" "$SERIAL_LOG" "$STDOUT_LOG"

log_info "Starting QEMU with boot observability capture..."
log_info "Timeout: ${QEMU_TIMEOUT}s"
log_info "Memory: $QEMU_MEMORY"
log_info "Debugcon: $DEBUGCON_LOG (primary evidence channel)"
log_info "Serial: $SERIAL_LOG (secondary/diagnostic channel)"

# Launch QEMU with evidence capture
# CRITICAL: Use OVMF firmware for UEFI boot (same as 'make run')
# -machine q35: modern chipset
# -drive if=pflash (OVMF_CODE): UEFI firmware code
# -drive if=pflash (OVMF_VARS): UEFI NVRAM variables
# -drive format=raw (EFI.img): boot disk
# -boot order=c: boot from disk
# -debugcon file:$DEBUGCON_LOG: kernel debug output (primary channel)
# -global isa-debugcon.iobase=0xe9: debugcon port
# -serial file:$SERIAL_LOG: serial console (secondary channel)
# -nographic: no display (text mode)

timeout $QEMU_TIMEOUT qemu-system-x86_64 \
    -machine q35 \
    -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
    -drive if=pflash,format=raw,file="$OVMF_VARS_RUN" \
    -drive format=raw,file="$EFI_IMAGE" \
    -boot order=c \
    -debugcon file:"$DEBUGCON_LOG" \
    -global isa-debugcon.iobase=0xe9 \
    -serial file:"$SERIAL_LOG" \
    -nographic \
    2>&1 | tee "$STDOUT_LOG" || {
        QEMU_EXIT=$?
        if [[ $QEMU_EXIT -eq 124 ]]; then
            log_info "QEMU timeout reached (expected for test harness)"
        else
            log_warn "QEMU exited with code: $QEMU_EXIT"
        fi
    }

log_info "QEMU execution complete"

# Channel integrity check
log_info "Validating output channel integrity..."

DEBUGCON_SIZE=0
SERIAL_SIZE=0

if [[ -f "$DEBUGCON_LOG" ]]; then
    DEBUGCON_SIZE=$(stat -c%s "$DEBUGCON_LOG" 2>/dev/null || stat -f%z "$DEBUGCON_LOG" 2>/dev/null || true)
fi

if [[ -f "$SERIAL_LOG" ]]; then
    SERIAL_SIZE=$(stat -c%s "$SERIAL_LOG" 2>/dev/null || stat -f%z "$SERIAL_LOG" 2>/dev/null || true)
fi

log_info "Channel sizes: debugcon=$DEBUGCON_SIZE bytes, serial=$SERIAL_SIZE bytes"

# HARD FAIL: All channels zero
if [[ $DEBUGCON_SIZE -eq 0 ]] && [[ $SERIAL_SIZE -eq 0 ]]; then
    log_error "OUTPUT_CHANNEL_FAILURE: All output channels are empty"
    log_error "Cannot proceed - no observable evidence captured"
    log_error ""
    log_error "Possible causes:"
    log_error "  - QEMU debugcon/serial misconfiguration"
    log_error "  - Bootloader/kernel not emitting markers"
    log_error "  - Output capture path broken"
    exit 1
fi

# Create channel-local traces (NO cross-channel merge)
# CRITICAL: Preserve raw append-order per channel
TRACE_DEBUGCON="$EVIDENCE_DIR/debugcon.trace"
TRACE_SERIAL="$EVIDENCE_DIR/serial.trace"

log_info "Creating channel-local traces (raw append-order preserved)..."

if [[ -f "$DEBUGCON_LOG" ]] && [[ $DEBUGCON_SIZE -gt 0 ]]; then
    cp "$DEBUGCON_LOG" "$TRACE_DEBUGCON"
    log_info "✓ Debugcon trace: $TRACE_DEBUGCON ($DEBUGCON_SIZE bytes)"
fi

if [[ -f "$SERIAL_LOG" ]] && [[ $SERIAL_SIZE -gt 0 ]]; then
    cp "$SERIAL_LOG" "$TRACE_SERIAL"
    log_info "✓ Serial trace: $TRACE_SERIAL ($SERIAL_SIZE bytes)"
fi

# Quick marker check (non-authoritative, for harness feedback only)
log_info ""
log_info "Quick marker check (non-authoritative):"

MARKER_COUNT=0

if [[ -f "$TRACE_DEBUGCON" ]]; then
    BOOT_START=$(grep -c "\[B\]\[UEFI_BOOT_START\]" "$TRACE_DEBUGCON" || true)
    BOOT_OK=$(grep -c "\[\[AYKEN_BOOT_OK\]\]" "$TRACE_DEBUGCON" || true)
    EARLY_BOOT=$(grep -c "\[K\]\[EARLY_BOOT_OK\]" "$TRACE_DEBUGCON" || true)
    
    log_info "  Debugcon: [B][UEFI_BOOT_START]=$BOOT_START, [[AYKEN_BOOT_OK]]=$BOOT_OK, [K][EARLY_BOOT_OK]=$EARLY_BOOT"
    MARKER_COUNT=$((MARKER_COUNT + BOOT_START + BOOT_OK + EARLY_BOOT))
fi

if [[ -f "$TRACE_SERIAL" ]]; then
    BOOT_START=$(grep -c "\[B\]\[UEFI_BOOT_START\]" "$TRACE_SERIAL" || true)
    BOOT_OK=$(grep -c "\[\[AYKEN_BOOT_OK\]\]" "$TRACE_SERIAL" || true)
    EARLY_BOOT=$(grep -c "\[K\]\[EARLY_BOOT_OK\]" "$TRACE_SERIAL" || true)
    
    log_info "  Serial: [B][UEFI_BOOT_START]=$BOOT_START, [[AYKEN_BOOT_OK]]=$BOOT_OK, [K][EARLY_BOOT_OK]=$EARLY_BOOT"
    MARKER_COUNT=$((MARKER_COUNT + BOOT_START + BOOT_OK + EARLY_BOOT))
fi

log_info ""
log_info "========================================="
log_info "Boot Observability Evidence Generated"
log_info "========================================="
log_info "Evidence directory: $EVIDENCE_DIR"
log_info "Primary channel: debugcon ($DEBUGCON_SIZE bytes)"
log_info "Secondary channel: serial ($SERIAL_SIZE bytes)"
log_info "Markers detected: $MARKER_COUNT (non-authoritative count)"
log_info ""
log_info "Next step: Run CI gate validation"
log_info "  make ci-gate-boot-observability"
log_info ""

if [[ $MARKER_COUNT -gt 0 ]]; then
    log_info "✓ Evidence capture successful"
    exit 0
else
    log_warn "⚠ No markers detected - evidence may be incomplete"
    log_warn "CI gate may fail - check bootloader/kernel marker emission"
    exit 0  # Don't fail harness, let CI gate decide
fi

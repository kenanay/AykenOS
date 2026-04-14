#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

log_info() { echo "[INFO] $*"; }
log_error() { echo "[ERROR] $*" >&2; }

# Resolve OVMF firmware
resolve_ovmf_firmware() {
    local candidates=(
        "$PROJECT_ROOT/firmware/ovmf/OVMF_CODE.fd|$PROJECT_ROOT/firmware/ovmf/OVMF_VARS.fd"
        "/usr/share/OVMF/OVMF_CODE_4M.fd|/usr/share/OVMF/OVMF_VARS_4M.fd"
        "/usr/share/OVMF/OVMF_CODE.fd|/usr/share/OVMF/OVMF_VARS.fd"
        "/usr/share/edk2/ovmf/OVMF_CODE.fd|/usr/share/edk2/ovmf/OVMF_VARS.fd"
        "/usr/share/qemu/OVMF_CODE.fd|/usr/share/qemu/OVMF_VARS.fd"
        "/opt/homebrew/share/qemu/edk2-x86_64-code.fd|/opt/homebrew/share/qemu/edk2-x86_64-vars.fd"
    )
    
    for pair in "${candidates[@]}"; do
        local code="${pair%%|*}"
        local vars="${pair##*|}"
        if [[ -f "$code" && -f "$vars" ]]; then
            printf "%s\n%s\n" "$code" "$vars"
            return 0
        fi
    done
    
    log_error "OVMF firmware not found"
    return 1
}

OVMF_PAIR="$(resolve_ovmf_firmware)" || exit 1
OVMF_CODE="$(printf "%s\n" "$OVMF_PAIR" | sed -n '1p')"
OVMF_VARS="$(printf "%s\n" "$OVMF_PAIR" | sed -n '2p')"

log_info "Using OVMF CODE: $OVMF_CODE"
log_info "Using OVMF VARS: $OVMF_VARS"

# Create evidence directory
EVIDENCE_DIR="evidence/execution-marker-proof"
mkdir -p "$EVIDENCE_DIR"

# Copy firmware and EFI image
OVMF_VARS_COPY="$EVIDENCE_DIR/ovmf_vars.fd"
EFI_IMG_RUN="$EVIDENCE_DIR/EFI.img"
cp -f "$OVMF_VARS" "$OVMF_VARS_COPY"
cp -f "EFI.img" "$EFI_IMG_RUN"

TRACE_LOG="$EVIDENCE_DIR/qemu_kernel_trace.log"

log_info "Running QEMU with execution-marker-only payload..."
log_info "Trace output: $TRACE_LOG"

timeout 10s qemu-system-x86_64 \
    -machine q35 \
    -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
    -drive if=pflash,format=raw,file="$OVMF_VARS_COPY" \
    -drive format=raw,file="$EFI_IMG_RUN" \
    -debugcon file:"$TRACE_LOG" \
    -global isa-debugcon.iobase=0x402 \
    -nographic \
    -no-reboot \
    -m 512M \
    2>&1 | tail -20 || true

log_info "QEMU execution complete"
log_info "Checking trace for markers..."

if grep -q "^S$" "$TRACE_LOG" && grep -q "^T$" "$TRACE_LOG" && grep -q "^O$" "$TRACE_LOG" && grep -q "^K$" "$TRACE_LOG"; then
    log_info "✅ SUCCESS: All markers found (S, T, O, K)"
    exit 0
else
    log_error "❌ FAIL: Markers not found in trace"
    log_info "Trace excerpt:"
    tail -50 "$TRACE_LOG" | grep -E "(^[STOK]$|RING3|SYSCALL)" || echo "(no relevant markers)"
    exit 1
fi

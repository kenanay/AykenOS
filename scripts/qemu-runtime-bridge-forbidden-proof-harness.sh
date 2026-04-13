#!/bin/bash
# QEMU Runtime_Bridge Forbidden Proof Harness
# Phase-16 Task 5: Runtime_Bridge Syscall Path Evidence Generation (Forbidden Path)
#
# This harness generates QEMU kernel trace evidence for Runtime_Bridge role enforcement:
# - Forbidden path: 1003 syscall triggers fail-closed termination
#
# Output: evidence/runtime-bridge-proof/qemu_kernel_trace_forbidden.log

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
EVIDENCE_DIR="$PROJECT_ROOT/evidence/runtime-bridge-proof"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

resolve_ovmf_firmware() {
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

mkdir -p "$EVIDENCE_DIR"

log_info "Starting Runtime_Bridge Forbidden QEMU proof harness..."

# Build EFI image with runtime-bridge-forbidden payload
log_info "Building EFI image with runtime-bridge-forbidden payload..."
if ! USER_MINIMAL_MODE=runtime-bridge-forbidden KERNEL_PROFILE=validation AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=1 make -C "$PROJECT_ROOT" efi-img > "$EVIDENCE_DIR/build_forbidden.log" 2>&1; then
    log_error "Build failed. Check: $EVIDENCE_DIR/build_forbidden.log"
    exit 1
fi

EFI_IMG="$PROJECT_ROOT/out/build/EFI.img"
if [[ ! -f "$EFI_IMG" ]]; then
    log_error "EFI image not found: $EFI_IMG"
    exit 1
fi

OVMF_PAIR="$(resolve_ovmf_firmware || true)"
if [[ -z "$OVMF_PAIR" ]]; then
    log_error "OVMF firmware not found"
    exit 1
fi

OVMF_CODE="$(printf "%s\n" "$OVMF_PAIR" | sed -n '1p')"
OVMF_VARS="$(printf "%s\n" "$OVMF_PAIR" | sed -n '2p')"

RUN_TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/runtime_bridge_forbidden.XXXXXX" 2>/dev/null || mktemp -d -t runtime_bridge_forbidden 2>/dev/null)"
OVMF_VARS_COPY="$RUN_TMP_DIR/OVMF_VARS.fd"
OVMF_VARS_SIZE="$(wc -c < "$OVMF_VARS" 2>/dev/null | tr -d '[:space:]')"
dd if=/dev/zero of="$OVMF_VARS_COPY" bs=1 count="$OVMF_VARS_SIZE" >/dev/null 2>&1
EFI_IMG_RUN="$RUN_TMP_DIR/EFI.img"
cp -f "$EFI_IMG" "$EFI_IMG_RUN"

FORBIDDEN_DEBUGCON="$EVIDENCE_DIR/qemu_forbidden_debugcon.log"
FORBIDDEN_SERIAL="$EVIDENCE_DIR/qemu_forbidden_serial.log"
FORBIDDEN_TRACE="$EVIDENCE_DIR/qemu_kernel_trace_forbidden.log"

log_info "Test 2: Runtime_Bridge forbidden syscall (1003)..."

timeout 10s qemu-system-x86_64 \
    -machine q35 \
    -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
    -drive if=pflash,format=raw,file="$OVMF_VARS_COPY" \
    -drive format=raw,file="$EFI_IMG_RUN" \
    -serial file:"$FORBIDDEN_SERIAL" \
    -chardev file,id=dbgcon,path="$FORBIDDEN_DEBUGCON" \
    -device isa-debugcon,iobase=0xe9,chardev=dbgcon \
    -m 256M \
    -no-reboot \
    -no-shutdown \
    -display none \
    > /dev/null 2>&1 || true

DEBUGCON_SIZE=$(stat -c%s "$FORBIDDEN_DEBUGCON" 2>/dev/null || stat -f%z "$FORBIDDEN_DEBUGCON" 2>/dev/null || echo "0")
SERIAL_SIZE=$(stat -c%s "$FORBIDDEN_SERIAL" 2>/dev/null || stat -f%z "$FORBIDDEN_SERIAL" 2>/dev/null || echo "0")

if [[ $DEBUGCON_SIZE -eq 0 ]] && [[ $SERIAL_SIZE -eq 0 ]]; then
    log_error "OUTPUT_CHANNEL_FAILURE: All output channels are empty"
    rm -rf "$RUN_TMP_DIR"
    exit 1
fi

if [[ $DEBUGCON_SIZE -gt 0 ]]; then
    cp "$FORBIDDEN_DEBUGCON" "$FORBIDDEN_TRACE"
elif [[ $SERIAL_SIZE -gt 0 ]]; then
    cp "$FORBIDDEN_SERIAL" "$FORBIDDEN_TRACE"
fi

log_info "Forbidden path trace: $FORBIDDEN_TRACE"

# Extract userspace payload output
PAYLOAD_OUTPUT=$(python3 -c "
import sys, re
try:
    with open('$FORBIDDEN_TRACE') as f:
        text = f.read()
    matches = re.findall(r'P10_SYSCALL_ENTER\n(.*?)\[\[AYKEN_', text)
    print(''.join(matches))
except Exception as e:
    print('')
")

# Auditing the trace
PASS=true

if ! echo "$PAYLOAD_OUTPUT" | grep -q "\[U\]\[RUNTIME_BRIDGE_FORBIDDEN_BEFORE\]"; then
    log_error "✗ Missing RUNTIME_BRIDGE_FORBIDDEN_BEFORE marker in payload"
    PASS=false
fi

if ! grep -q "\[\[AYKEN_SYSCALL_ENTER\]\]" "$FORBIDDEN_TRACE"; then
    log_error "✗ Missing AYKEN_SYSCALL_ENTER marker in kernel trace"
    PASS=false
fi

if ! grep -q "\[\[AYKEN_BOUNDARY_KILL\]\]" "$FORBIDDEN_TRACE"; then
    log_error "✗ Missing AYKEN_BOUNDARY_KILL marker in kernel trace"
    PASS=false
fi

if echo "$PAYLOAD_OUTPUT" | grep -q "\[U\]\[RUNTIME_BRIDGE_FORBIDDEN_AFTER\]"; then
    log_error "✗ Found RUNTIME_BRIDGE_FORBIDDEN_AFTER marker! Fail-closed is broken."
    PASS=false
fi

if [[ "$PASS" == "true" ]]; then
    log_info "✓ Forbidden path: Runtime_Bridge fail-closed audit PASS"
    exit 0
else
    log_error "✗ Forbidden path: Runtime_Bridge fail-closed audit FAIL"
    exit 1
fi

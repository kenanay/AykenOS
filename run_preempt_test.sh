#!/bin/bash
set -euo pipefail

QEMU_TIMEOUT="${QEMU_TIMEOUT:-12}"
EFI_IMG="${EFI_IMG:-EFI.img}"
OVMF_CODE="${OVMF_CODE:-firmware/ovmf/OVMF_CODE.fd}"
OVMF_VARS_RUN="${OVMF_VARS_RUN:-ovmf_vars.fd}"
OVMF_VARS_CLEAN="${OVMF_VARS_CLEAN:-OVMF_VARS.clean.fd}"
DEBUG_LOG="${DEBUG_LOG:-PHASE_4_5_OUTPUT.log}"
SERIAL_LOG="${SERIAL_LOG:-PHASE_4_5_SERIAL.log}"

if [[ ! -f "$EFI_IMG" ]]; then
  make efi-img
fi

if [[ -f "$OVMF_VARS_CLEAN" ]]; then
  cp -f "$OVMF_VARS_CLEAN" "$OVMF_VARS_RUN"
elif [[ ! -f "$OVMF_VARS_RUN" ]]; then
  cp -f firmware/ovmf/OVMF_VARS.fd "$OVMF_VARS_RUN"
fi

: > "$DEBUG_LOG"
: > "$SERIAL_LOG"

if command -v timeout >/dev/null 2>&1; then
  TIMEOUT_CMD=(timeout "$QEMU_TIMEOUT")
elif command -v gtimeout >/dev/null 2>&1; then
  TIMEOUT_CMD=(gtimeout "$QEMU_TIMEOUT")
else
  TIMEOUT_CMD=()
fi

"${TIMEOUT_CMD[@]}" qemu-system-x86_64 \
  -machine q35 \
  -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
  -drive if=pflash,format=raw,file="$OVMF_VARS_RUN" \
  -drive format=raw,file="$EFI_IMG" \
  -boot order=c \
  -m 256M \
  -debugcon "file:$DEBUG_LOG" \
  -global isa-debugcon.iobase=0xe9 \
  -serial "file:$SERIAL_LOG" \
  -monitor none \
  -display none \
  -no-reboot \
  -no-shutdown || true

echo "=== Preempt debug tail ==="
tail -n 120 "$DEBUG_LOG" || true

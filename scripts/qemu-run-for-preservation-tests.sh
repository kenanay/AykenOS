#!/usr/bin/env bash
#
# AykenOS QEMU Runner for Preservation Tests
#
# Purpose: Run QEMU and capture debugcon output for preservation test analysis
# This script does NOT run the old analyzer - preservation tests analyze the log themselves

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUT_DIR="$REPO_ROOT/out/second-syscall-evidence"

# Create output directory
mkdir -p "$OUT_DIR"

# Prepare OVMF paths
OVMF_CODE="${OVMF_CODE:-$REPO_ROOT/firmware/ovmf/OVMF_CODE.fd}"
OVMF_VARS="$OUT_DIR/OVMF_VARS.fd"
cp -f "$REPO_ROOT/firmware/ovmf/OVMF_VARS.fd" "$OVMF_VARS"

# Run QEMU with timeout (test should complete quickly)
set +e
timeout 60s qemu-system-x86_64 \
    -machine q35 \
    -m 256M \
    -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
    -drive if=pflash,format=raw,file="$OVMF_VARS" \
    -drive format=raw,file="$REPO_ROOT/out/build/EFI.img" \
    -debugcon file:"$OUT_DIR/debugcon.log" \
    -global isa-debugcon.iobase=0xE9 \
    -serial file:"$OUT_DIR/serial.log" \
    -display none \
    -no-reboot \
    -no-shutdown > /dev/null 2>&1
QEMU_EXIT=$?
set -e

if [ ! -f "$OUT_DIR/debugcon.log" ]; then
    echo "ERROR: debugcon.log not found - QEMU may not have run successfully"
    exit 2
fi

echo "✓ QEMU execution complete - evidence captured"
exit 0

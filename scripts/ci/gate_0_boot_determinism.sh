#!/usr/bin/env bash
# ============================================================================
# Gate-0: Boot Determinism Validation
# ============================================================================
# Purpose:
#   Validate deterministic UEFI boot without Shell fallback
#
# Success Criteria:
#   - [[AYKEN_BOOT_OK]] marker appears in debugcon output
#   - Boot occurs within timeout window
#   - UEFI Shell fallback is not observed
#
# Copyright © 2026 Kenan AY
# ============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$PROJECT_ROOT"

# Configuration
QEMU_TIMEOUT=${QEMU_TIMEOUT:-10}
KERNEL_PROFILE=${KERNEL_PROFILE:-validation}
EVIDENCE_DIR=${EVIDENCE_DIR:-evidence/gate-0-boot}
OVMF_CODE=${OVMF_CODE:-firmware/ovmf/OVMF_CODE.fd}
OVMF_VARS_TEMPLATE=${OVMF_VARS_TEMPLATE:-OVMF_VARS.clean.fd}
OVMF_VARS_RUN=${OVMF_VARS_RUN:-ovmf_vars.fd}
DEBUGCON_LOG="$PROJECT_ROOT/$EVIDENCE_DIR/debugcon.log"
QEMU_LOG="$PROJECT_ROOT/$EVIDENCE_DIR/qemu.log"
REPORT_JSON="$PROJECT_ROOT/$EVIDENCE_DIR/report.json"

# Create evidence directory
mkdir -p "$EVIDENCE_DIR"
: > "$DEBUGCON_LOG"
: > "$QEMU_LOG"

echo "=== Gate-0: Boot Determinism Validation ==="
echo "Kernel profile: $KERNEL_PROFILE"
echo "QEMU timeout: ${QEMU_TIMEOUT}s"
echo "Evidence dir: $EVIDENCE_DIR"
echo ""

if [[ ! -f "$OVMF_CODE" ]]; then
    echo "ERROR: OVMF code not found: $OVMF_CODE"
    exit 1
fi

if [[ ! -f "$OVMF_VARS_TEMPLATE" ]]; then
    echo "ERROR: OVMF vars template not found: $OVMF_VARS_TEMPLATE"
    exit 1
fi

# Clean build
echo "[*] Building kernel and bootloader..."
make KERNEL_PROFILE="$KERNEL_PROFILE" clean >/dev/null 2>&1
make KERNEL_PROFILE="$KERNEL_PROFILE" kernel bootloader >/dev/null 2>&1

# Create EFI image (deterministic direct boot path)
echo "[*] Creating EFI image (direct BOOTX64.EFI path)..."
make KERNEL_PROFILE="$KERNEL_PROFILE" efi-img >/dev/null 2>&1

# Prepare clean NVRAM
echo "[*] Preparing clean NVRAM (blank varstore)..."
VARS_BYTES=$(wc -c < "$OVMF_VARS_TEMPLATE" 2>/dev/null || echo 0)
VARS_BYTES=${VARS_BYTES//[[:space:]]/}
if [[ -z "$VARS_BYTES" || "$VARS_BYTES" -le 0 ]]; then
    echo "ERROR: failed to detect vars template size: $OVMF_VARS_TEMPLATE"
    exit 1
fi
dd if=/dev/zero of="$OVMF_VARS_RUN" bs=1 count="$VARS_BYTES" >/dev/null 2>&1

# Run QEMU
echo "[*] Booting kernel (timeout: ${QEMU_TIMEOUT}s)..."
set +e
timeout "$QEMU_TIMEOUT" qemu-system-x86_64 \
    -machine q35 \
    -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
    -drive if=pflash,format=raw,file="$OVMF_VARS_RUN" \
    -drive format=raw,file=EFI.img \
    -debugcon "file:$DEBUGCON_LOG" \
    -global isa-debugcon.iobase=0xe9 \
    -nographic \
    >"$QEMU_LOG" 2>&1
QEMU_EXIT=$?
set -e
DEBUGCON_BYTES=$(wc -c < "$DEBUGCON_LOG" 2>/dev/null || echo 0)
DEBUGCON_BYTES=${DEBUGCON_BYTES//[[:space:]]/}
QEMU_LOG_BYTES=$(wc -c < "$QEMU_LOG" 2>/dev/null || echo 0)
QEMU_LOG_BYTES=${QEMU_LOG_BYTES//[[:space:]]/}

echo "[DEBUG] qemu_exit=$QEMU_EXIT debugcon_bytes=$DEBUGCON_BYTES qemu_log_bytes=$QEMU_LOG_BYTES"

# Validate boot marker
echo "[*] Validating boot marker..."
BOOT_MARKER_FOUND=0
SHELL_FALLBACK_FOUND=0

if grep -q "\[\[AYKEN_BOOT_OK\]\]" "$DEBUGCON_LOG"; then
    BOOT_MARKER_FOUND=1
fi

if grep -Eq "UEFI Interactive Shell|Boot0006 \"EFI Internal Shell\"" "$QEMU_LOG"; then
    SHELL_FALLBACK_FOUND=1
fi

if [[ "$QEMU_EXIT" -ne 0 && "$QEMU_EXIT" -ne 124 ]]; then
    VERDICT="FAIL"
    REASON="QEMU exited unexpectedly (exit_code=$QEMU_EXIT)"
elif [[ "$SHELL_FALLBACK_FOUND" -eq 1 ]]; then
    VERDICT="FAIL"
    REASON="UEFI Shell fallback detected in QEMU output"
elif [[ "$BOOT_MARKER_FOUND" -ne 1 ]]; then
    VERDICT="FAIL"
    REASON="Boot marker [[AYKEN_BOOT_OK]] not found in debugcon output"
else
    VERDICT="PASS"
    REASON="Direct BOOTX64.EFI path reached kernel and marker was detected"
fi

if [[ "$VERDICT" == "PASS" ]]; then
    echo "✅ Boot marker found: [[AYKEN_BOOT_OK]]"
    echo "✅ No Shell fallback detected"
else
    echo "❌ Gate-0 validation failed: $REASON"
    echo ""
    echo "QEMU output (first 25 lines):"
    head -25 "$QEMU_LOG" || true
    echo ""
    echo "Debugcon output (first 25 lines):"
    head -25 "$DEBUGCON_LOG" || true
fi

# Generate report
cat > "$REPORT_JSON" <<EOF
{
  "gate": "boot-determinism",
  "verdict": "$VERDICT",
  "reason": "$REASON",
  "kernel_profile": "$KERNEL_PROFILE",
  "qemu_timeout": $QEMU_TIMEOUT,
  "qemu_exit_code": $QEMU_EXIT,
  "shell_fallback_detected": $SHELL_FALLBACK_FOUND,
  "boot_marker_found": $BOOT_MARKER_FOUND,
  "debugcon_bytes": $DEBUGCON_BYTES,
  "qemu_log_bytes": $QEMU_LOG_BYTES,
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
}
EOF

echo ""
echo "Report: $REPORT_JSON"
cat "$REPORT_JSON"
echo ""

if [[ "$VERDICT" != "PASS" ]]; then
    exit 1
fi

echo "✅ Gate-0: Boot Determinism PASS"

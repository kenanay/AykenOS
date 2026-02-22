#!/usr/bin/env bash
# ============================================================================
# Gate-3: Ring3 Runtime Validation
# ============================================================================
# Purpose:
#   Validate that Ring3 code executes and can communicate with Ring0
#
# Success Criteria:
#   - [[AYKEN_BOOT_OK]] marker appears (Gate-0)
#   - [[AYKEN_TICK]] marker appears (Gate-1)
#   - [[AYKEN_CTX_SWITCH]] marker appears (Gate-2)
#   - [[AYKEN_RING3_OK]] marker appears (Gate-3)
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
EVIDENCE_DIR=${EVIDENCE_DIR:-evidence/gate-3-ring3-runtime}
OVMF_CODE=${OVMF_CODE:-firmware/ovmf/OVMF_CODE.fd}
OVMF_VARS_TEMPLATE=${OVMF_VARS_TEMPLATE:-OVMF_VARS.clean.fd}
OVMF_VARS_RUN=${OVMF_VARS_RUN:-ovmf_vars.fd}
BOOT_MARKER="[[AYKEN_BOOT_OK]]"
TICK_MARKER="[[AYKEN_TICK]]"
CTX_MARKER="[[AYKEN_CTX_SWITCH]]"
RING3_MARKER="[[AYKEN_RING3_OK]]"
DEBUGCON_LOG="$PROJECT_ROOT/$EVIDENCE_DIR/debugcon.log"
QEMU_LOG="$PROJECT_ROOT/$EVIDENCE_DIR/qemu.log"
REPORT_JSON="$PROJECT_ROOT/$EVIDENCE_DIR/report.json"

# Use absolute paths for QEMU (relative paths don't work reliably)
ABS_DEBUGCON_LOG="$(cd "$PROJECT_ROOT" && pwd)/$EVIDENCE_DIR/debugcon.log"
ABS_QEMU_LOG="$(cd "$PROJECT_ROOT" && pwd)/$EVIDENCE_DIR/qemu.log"

mkdir -p "$EVIDENCE_DIR"
: > "$DEBUGCON_LOG"
: > "$QEMU_LOG"

echo "=== Gate-3: Ring3 Runtime Validation ==="
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

echo "[*] Building kernel and bootloader..."
make KERNEL_PROFILE="$KERNEL_PROFILE" clean >/dev/null 2>&1
make KERNEL_PROFILE="$KERNEL_PROFILE" kernel bootloader >/dev/null 2>&1

echo "[*] Creating EFI image (direct BOOTX64.EFI path)..."
make KERNEL_PROFILE="$KERNEL_PROFILE" efi-img >/dev/null 2>&1

echo "[*] Preparing clean NVRAM (blank varstore)..."
VARS_BYTES=$(wc -c < "$OVMF_VARS_TEMPLATE" 2>/dev/null || echo 0)
VARS_BYTES=${VARS_BYTES//[[:space:]]/}
if [[ -z "$VARS_BYTES" || "$VARS_BYTES" -le 0 ]]; then
    echo "ERROR: failed to detect vars template size: $OVMF_VARS_TEMPLATE"
    exit 1
fi
dd if=/dev/zero of="$OVMF_VARS_RUN" bs=1 count="$VARS_BYTES" >/dev/null 2>&1

echo "[*] Booting kernel (timeout: ${QEMU_TIMEOUT}s)..."
set +e
timeout "$QEMU_TIMEOUT" qemu-system-x86_64 \
    -machine q35 \
    -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
    -drive if=pflash,format=raw,file="$OVMF_VARS_RUN" \
    -drive format=raw,file=EFI.img \
    -debugcon "file:$ABS_DEBUGCON_LOG" \
    -global isa-debugcon.iobase=0xe9 \
    -nographic \
    >"$ABS_QEMU_LOG" 2>&1
QEMU_EXIT=$?
set -e

DEBUGCON_BYTES=$(wc -c < "$DEBUGCON_LOG" 2>/dev/null || echo 0)
DEBUGCON_BYTES=${DEBUGCON_BYTES//[[:space:]]/}
QEMU_LOG_BYTES=$(wc -c < "$QEMU_LOG" 2>/dev/null || echo 0)
QEMU_LOG_BYTES=${QEMU_LOG_BYTES//[[:space:]]/}

echo "[DEBUG] qemu_exit=$QEMU_EXIT debugcon_bytes=$DEBUGCON_BYTES qemu_log_bytes=$QEMU_LOG_BYTES"

echo "[*] Validating markers..."
BOOT_MARKER_FOUND=0
TICK_MARKER_FOUND=0
CTX_MARKER_FOUND=0
RING3_MARKER_FOUND=0
SHELL_FALLBACK_FOUND=0

if grep -Fq "$BOOT_MARKER" "$DEBUGCON_LOG"; then
    BOOT_MARKER_FOUND=1
fi

if grep -Fq "$TICK_MARKER" "$DEBUGCON_LOG"; then
    TICK_MARKER_FOUND=1
fi

if grep -Fq "$CTX_MARKER" "$DEBUGCON_LOG"; then
    CTX_MARKER_FOUND=1
fi

if grep -Fq "$RING3_MARKER" "$DEBUGCON_LOG"; then
    RING3_MARKER_FOUND=1
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
    REASON="Boot marker $BOOT_MARKER not found in debugcon output"
elif [[ "$TICK_MARKER_FOUND" -ne 1 ]]; then
    VERDICT="FAIL"
    REASON="Tick marker $TICK_MARKER not found in debugcon output"
elif [[ "$CTX_MARKER_FOUND" -ne 1 ]]; then
    VERDICT="FAIL"
    REASON="Context switch marker $CTX_MARKER not found in debugcon output"
elif [[ "$RING3_MARKER_FOUND" -ne 1 ]]; then
    VERDICT="FAIL"
    REASON="Ring3 runtime marker $RING3_MARKER not found in debugcon output"
else
    VERDICT="PASS"
    REASON="Ring3 runtime proof complete (all markers present)"
fi

if [[ "$VERDICT" == "PASS" ]]; then
    echo "✅ Boot marker found: $BOOT_MARKER"
    echo "✅ Tick marker found: $TICK_MARKER"
    echo "✅ Context switch marker found: $CTX_MARKER"
    echo "✅ Ring3 runtime marker found: $RING3_MARKER"
    echo "✅ No Shell fallback detected"
else
    echo "❌ Gate-3 validation failed: $REASON"
    echo ""
    echo "QEMU output (last 50 lines):"
    tail -50 "$QEMU_LOG" || true
    echo ""
    echo "Debugcon output (last 50 lines):"
    tail -50 "$DEBUGCON_LOG" || true
fi

cat > "$REPORT_JSON" <<EOF
{
  "gate": "ring3-runtime",
  "verdict": "$VERDICT",
  "reason": "$REASON",
  "kernel_profile": "$KERNEL_PROFILE",
  "qemu_timeout": $QEMU_TIMEOUT,
  "qemu_exit_code": $QEMU_EXIT,
  "shell_fallback_detected": $SHELL_FALLBACK_FOUND,
  "boot_marker_found": $BOOT_MARKER_FOUND,
  "tick_marker_found": $TICK_MARKER_FOUND,
  "ctx_switch_marker_found": $CTX_MARKER_FOUND,
  "ring3_marker_found": $RING3_MARKER_FOUND,
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

echo "✅ Gate-3: Ring3 Runtime Validation PASS"

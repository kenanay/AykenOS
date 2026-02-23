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
QEMU_TIMEOUT=${QEMU_TIMEOUT:-15}
KERNEL_PROFILE=${KERNEL_PROFILE:-validation}
EVIDENCE_DIR=${EVIDENCE_DIR:-evidence/gate-3-ring3-runtime}
OVMF_CODE=${OVMF_CODE:-firmware/ovmf/OVMF_CODE.fd}
OVMF_VARS_TEMPLATE=${OVMF_VARS_TEMPLATE:-OVMF_VARS.clean.fd}
OVMF_VARS_RUN=${OVMF_VARS_RUN:-$EVIDENCE_DIR/ovmf_vars.fd}
BOOT_MARKER="[[AYKEN_BOOT_OK]]"
TICK_MARKER="[[AYKEN_TICK]]"
CTX_MARKER="[[AYKEN_CTX_SWITCH]]"
RING3_MARKER="[[AYKEN_RING3_OK]]"
DEBUGCON_LOG="$PROJECT_ROOT/$EVIDENCE_DIR/debugcon.log"
QEMU_LOG="$PROJECT_ROOT/$EVIDENCE_DIR/qemu.log"
REPORT_JSON="$PROJECT_ROOT/$EVIDENCE_DIR/report.json"

mkdir -p "$EVIDENCE_DIR"
: > "$DEBUGCON_LOG"
: > "$QEMU_LOG"

ABS_PROJECT_ROOT="$(cd "$PROJECT_ROOT" && pwd)"
ABS_DEBUGCON_LOG="$ABS_PROJECT_ROOT/$EVIDENCE_DIR/debugcon.log"
ABS_QEMU_LOG="$ABS_PROJECT_ROOT/$EVIDENCE_DIR/qemu.log"
ABS_OVMF_CODE="$ABS_PROJECT_ROOT/$OVMF_CODE"
ABS_EFI_IMG="$ABS_PROJECT_ROOT/EFI.img"

if [[ "$OVMF_VARS_RUN" = /* ]]; then
    ABS_OVMF_VARS="$OVMF_VARS_RUN"
else
    ABS_OVMF_VARS="$ABS_PROJECT_ROOT/$OVMF_VARS_RUN"
fi

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
if ! make KERNEL_PROFILE="$KERNEL_PROFILE" clean >/dev/null 2>&1; then
    echo "ERROR: make clean failed"
    exit 1
fi

if ! make KERNEL_PROFILE="$KERNEL_PROFILE" kernel bootloader >/dev/null 2>&1; then
    echo "ERROR: make kernel bootloader failed"
    exit 1
fi

echo "[*] Validating kernel build (marker sanity check)..."
KERNEL_STRINGS="$PROJECT_ROOT/$EVIDENCE_DIR/kernel_strings.txt"
if ! strings -a kernel.elf > "$KERNEL_STRINGS"; then
    echo "ERROR: failed to extract kernel strings"
    exit 1
fi
if ! grep -Fq "AYKEN_BOOT_OK" "$KERNEL_STRINGS"; then
    echo "ERROR: Boot marker not found in kernel.elf"
    echo "This indicates KERNEL_PROFILE=validation was not used or markers not compiled in."
    echo "Expected: [[AYKEN_BOOT_OK]] in kernel.elf strings output"
    exit 1
fi
echo "    ✓ Boot marker found in kernel.elf"

echo "[*] Creating EFI image (direct BOOTX64.EFI path)..."
if ! make KERNEL_PROFILE="$KERNEL_PROFILE" efi-img >/dev/null 2>&1; then
    echo "ERROR: make efi-img failed"
    exit 1
fi

prepare_blank_varstore() {
    local vars_bytes
    vars_bytes=$(wc -c < "$OVMF_VARS_TEMPLATE" 2>/dev/null || echo 0)
    vars_bytes=${vars_bytes//[[:space:]]/}
    if [[ -z "$vars_bytes" || "$vars_bytes" -le 0 ]]; then
        echo "ERROR: failed to detect vars template size: $OVMF_VARS_TEMPLATE"
        return 1
    fi
    dd if=/dev/zero of="$ABS_OVMF_VARS" bs=1 count="$vars_bytes" >/dev/null 2>&1
}

echo "[*] Preparing clean NVRAM (blank varstore)..."
if ! prepare_blank_varstore; then
    exit 1
fi

echo "[*] Booting kernel (timeout: ${QEMU_TIMEOUT}s)..."

run_qemu_debugcon_backend() {
    set +e
    timeout "$QEMU_TIMEOUT" qemu-system-x86_64 \
        -machine q35 \
        -drive if=pflash,format=raw,readonly=on,file="$ABS_OVMF_CODE" \
        -drive if=pflash,format=raw,file="$ABS_OVMF_VARS" \
        -drive format=raw,file="$ABS_EFI_IMG" \
        -debugcon "file:$ABS_DEBUGCON_LOG" \
        -global isa-debugcon.iobase=0xe9 \
        -nographic \
        -no-reboot \
        >"$ABS_QEMU_LOG" 2>&1
    QEMU_EXIT=$?
    set -e
}

run_qemu_chardev_backend() {
    set +e
    timeout "$QEMU_TIMEOUT" qemu-system-x86_64 \
        -machine q35 \
        -drive if=pflash,format=raw,readonly=on,file="$ABS_OVMF_CODE" \
        -drive if=pflash,format=raw,file="$ABS_OVMF_VARS" \
        -drive format=raw,file="$ABS_EFI_IMG" \
        -chardev file,id=debugcon0,path="$ABS_DEBUGCON_LOG" \
        -device isa-debugcon,iobase=0xe9,chardev=debugcon0 \
        -nographic \
        -no-reboot \
        >"$ABS_QEMU_LOG" 2>&1
    QEMU_EXIT=$?
    set -e
}

QEMU_BACKEND="debugcon"
run_qemu_debugcon_backend

DEBUGCON_BYTES=$(wc -c < "$DEBUGCON_LOG" 2>/dev/null || echo 0)
DEBUGCON_BYTES=${DEBUGCON_BYTES//[[:space:]]/}
QEMU_LOG_BYTES=$(wc -c < "$QEMU_LOG" 2>/dev/null || echo 0)
QEMU_LOG_BYTES=${QEMU_LOG_BYTES//[[:space:]]/}

if [[ "$DEBUGCON_BYTES" -eq 0 ]]; then
    # Retry with chardev backend for macOS/QEMU regressions.
    QEMU_BACKEND="chardev"
    : > "$DEBUGCON_LOG"
    : > "$QEMU_LOG"
    if ! prepare_blank_varstore; then
        exit 1
    fi
    run_qemu_chardev_backend
    DEBUGCON_BYTES=$(wc -c < "$DEBUGCON_LOG" 2>/dev/null || echo 0)
    DEBUGCON_BYTES=${DEBUGCON_BYTES//[[:space:]]/}
    QEMU_LOG_BYTES=$(wc -c < "$QEMU_LOG" 2>/dev/null || echo 0)
    QEMU_LOG_BYTES=${QEMU_LOG_BYTES//[[:space:]]/}
fi

echo "[DEBUG] qemu_backend=$QEMU_BACKEND qemu_exit=$QEMU_EXIT debugcon_bytes=$DEBUGCON_BYTES qemu_log_bytes=$QEMU_LOG_BYTES"

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
elif [[ "$DEBUGCON_BYTES" -eq 0 ]]; then
    VERDICT="FAIL"
    REASON="Debugcon output empty (0 bytes) - both debugcon backends failed"
elif [[ "$QEMU_LOG_BYTES" -eq 0 ]]; then
    VERDICT="FAIL"
    REASON="QEMU log empty (0 bytes) - QEMU stdout/stderr not captured"
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
  "qemu_backend": "$QEMU_BACKEND",
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

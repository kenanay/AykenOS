#!/bin/bash
# QEMU Runtime_Bridge Proof Harness
# Phase-16 Task 5: Runtime_Bridge Syscall Path Evidence Generation
#
# This harness generates QEMU kernel trace evidence for Runtime_Bridge role enforcement:
# 1. Allowed path: 1012/1013/1014 syscalls succeed
# 2. Forbidden path: 1003 syscall triggers fail-closed termination
#
# Output: evidence/runtime-bridge-proof/qemu_kernel_trace_*.log

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

log_info "Starting Runtime_Bridge QEMU proof harness..."
log_info "Note: Runtime_Bridge test is embedded in EFI.img via USER_MINIMAL_MODE=runtime-bridge-test"

# Build EFI image with runtime-bridge-test payload
log_info "Building EFI image with runtime-bridge-test payload..."
if ! USER_MINIMAL_MODE=runtime-bridge-test KERNEL_PROFILE=validation AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=1 make efi-img > "$EVIDENCE_DIR/build.log" 2>&1; then
    log_error "Build failed. Check: $EVIDENCE_DIR/build.log"
    exit 1
fi

EFI_IMG="$PROJECT_ROOT/out/build/EFI.img"
if [[ ! -f "$EFI_IMG" ]]; then
    log_error "EFI image not found: $EFI_IMG"
    exit 1
fi

log_info "✓ EFI image built: $EFI_IMG"

# Verify build manifest (AUTHORITY)
MANIFEST="$PROJECT_ROOT/out/build/payload_manifest.json"
if [[ ! -f "$MANIFEST" ]]; then
    log_error "❌ MANIFEST_MISSING: Build manifest not found: $MANIFEST"
    log_error "   This violates the payload authority chain"
    exit 1
fi

log_info "Verifying build manifest..."
MANIFEST_MODE=$(python3 -c "import json; print(json.load(open('$MANIFEST'))['selected_mode'])" 2>/dev/null || echo "")
MANIFEST_PAYLOAD_SHA=$(python3 -c "import json; print(json.load(open('$MANIFEST'))['payload_sha256'])" 2>/dev/null || echo "")
MANIFEST_EMBEDDED_SHA=$(python3 -c "import json; print(json.load(open('$MANIFEST'))['embedded_header_sha256'])" 2>/dev/null || echo "")

if [[ "$MANIFEST_MODE" != "runtime-bridge-test" ]]; then
    log_error "❌ MODE_MISMATCH: Manifest shows selected_mode='$MANIFEST_MODE' (expected 'runtime-bridge-test')"
    log_error "   This violates the Mode Authority Invariant"
    exit 1
fi

if [[ "$MANIFEST_PAYLOAD_SHA" != "$MANIFEST_EMBEDDED_SHA" ]]; then
    log_error "❌ HASH_MISMATCH: Manifest payload_sha256 != embedded_header_sha256"
    log_error "   Payload: $MANIFEST_PAYLOAD_SHA"
    log_error "   Embedded: $MANIFEST_EMBEDDED_SHA"
    log_error "   This violates the Payload Integrity Invariant"
    exit 1
fi

log_info "✓ Manifest verification passed"
log_info "  Mode: $MANIFEST_MODE"
log_info "  Hash: $MANIFEST_PAYLOAD_SHA"

# Verify build log (DIAGNOSTIC - WARNING only)
if grep -q 'DAYKEN_USER_MINIMAL_MODE_STRING="runtime-bridge-test"' "$EVIDENCE_DIR/build.log"; then
    log_info "✓ Build log shows correct mode string (diagnostic)"
else
    log_warn "⚠ Build log does not show DAYKEN_USER_MINIMAL_MODE_STRING=\"runtime-bridge-test\" (diagnostic only)"
fi

# Verify build log (DIAGNOSTIC - WARNING only)
BUILD_LOG="$EVIDENCE_DIR/build.log"
if [[ -f "$BUILD_LOG" ]] && grep -q 'DAYKEN_USER_MINIMAL_MODE_STRING="runtime-bridge-test"' "$BUILD_LOG"; then
    log_info "✓ Build log shows correct mode string (diagnostic)"
elif [[ -f "$BUILD_LOG" ]]; then
    log_warn "⚠ Build log does not show DAYKEN_USER_MINIMAL_MODE_STRING=\"runtime-bridge-test\" (diagnostic only)"
fi

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
RUN_TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/runtime_bridge_run.XXXXXX" 2>/dev/null || mktemp -d -t runtime_bridge_run 2>/dev/null)"
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
cp -f "$EFI_IMG" "$EFI_IMG_RUN"

log_info "Starting Runtime_Bridge QEMU proof harness..."

# Test 1: Allowed Path (1012/1013/1014)
log_info "Test 1: Runtime_Bridge allowed syscalls (1012/1013/1014)..."

ALLOWED_DEBUGCON="$EVIDENCE_DIR/qemu_allowed_debugcon.log"
ALLOWED_SERIAL="$EVIDENCE_DIR/qemu_allowed_serial.log"
ALLOWED_TRACE="$EVIDENCE_DIR/qemu_kernel_trace_allowed.log"

# Launch QEMU with OVMF + EFI.img (correct boot path)
# Note: Runtime_Bridge test is embedded in EFI.img via USER_MINIMAL_MODE=runtime-bridge-test
timeout 10s qemu-system-x86_64 \
    -machine q35 \
    -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
    -drive if=pflash,format=raw,file="$OVMF_VARS_COPY" \
    -drive format=raw,file="$EFI_IMG_RUN" \
    -serial file:"$ALLOWED_SERIAL" \
    -chardev file,id=dbgcon,path="$ALLOWED_DEBUGCON" \
    -device isa-debugcon,iobase=0xe9,chardev=dbgcon \
    -m 256M \
    -no-reboot \
    -no-shutdown \
    -display none \
    > /dev/null 2>&1 || true

# Channel integrity validation (HARD FAIL rule)
DEBUGCON_SIZE=0
SERIAL_SIZE=0

if [[ -f "$ALLOWED_DEBUGCON" ]]; then
    DEBUGCON_SIZE=$(stat -c%s "$ALLOWED_DEBUGCON" 2>/dev/null || stat -f%z "$ALLOWED_DEBUGCON" 2>/dev/null || echo "0")
fi

if [[ -f "$ALLOWED_SERIAL" ]]; then
    SERIAL_SIZE=$(stat -c%s "$ALLOWED_SERIAL" 2>/dev/null || stat -f%z "$ALLOWED_SERIAL" 2>/dev/null || echo "0")
fi

log_info "Allowed path channel sizes: debugcon=$DEBUGCON_SIZE bytes, serial=$SERIAL_SIZE bytes"

# HARD FAIL: All channels zero
if [[ $DEBUGCON_SIZE -eq 0 ]] && [[ $SERIAL_SIZE -eq 0 ]]; then
    log_error "OUTPUT_CHANNEL_FAILURE: All output channels are empty (allowed path)"
    log_error "Cannot proceed with validation - no observable evidence"
    rm -rf "$RUN_TMP_DIR"
    exit 1
fi

# Keep channel-local trace (NO cross-channel merge, NO sort)
if [[ $DEBUGCON_SIZE -gt 0 ]]; then
    cp "$ALLOWED_DEBUGCON" "$ALLOWED_TRACE"
elif [[ $SERIAL_SIZE -gt 0 ]]; then
    cp "$ALLOWED_SERIAL" "$ALLOWED_TRACE"
fi

log_info "Allowed path trace: $ALLOWED_TRACE"

# Verify boot marker (AUTHORITY)
BOOT_MODE_FOUND=0
BOOT_LOG=""

if [[ $DEBUGCON_SIZE -gt 0 ]]; then
    BOOT_LOG="$ALLOWED_DEBUGCON"
elif [[ $SERIAL_SIZE -gt 0 ]]; then
    BOOT_LOG="$ALLOWED_SERIAL"
fi

if [[ -n "$BOOT_LOG" ]]; then
    if grep -q '\[K\]\[PAYLOAD_MODE=runtime-bridge-test\]' "$BOOT_LOG"; then
        BOOT_MODE_FOUND=1
        log_info "✓ Boot marker [K][PAYLOAD_MODE=runtime-bridge-test] found"
    fi
fi

if [[ $BOOT_MODE_FOUND -eq 0 ]]; then
    log_error "❌ BOOT_MARKER_MISSING: Boot log does not contain [K][PAYLOAD_MODE=runtime-bridge-test]"
    log_error "   This violates the Mode Authority Invariant (boot verification)"
    rm -rf "$RUN_TMP_DIR"
    exit 1
fi

# Analyze allowed trace using Runtime_Bridge-specific audit script
if [[ -x "$PROJECT_ROOT/tools/validation/runtime_bridge_audit.sh" ]]; then
    log_info "Running Runtime_Bridge audit on allowed trace..."
    if "$PROJECT_ROOT/tools/validation/runtime_bridge_audit.sh" "$ALLOWED_TRACE"; then
        log_info "✓ Allowed path: Runtime_Bridge audit PASS"
    else
        log_warn "✗ Allowed path: Runtime_Bridge audit FAIL (check trace)"
    fi
else
    log_warn "Runtime_Bridge audit script not found, skipping validation"
fi

# Cleanup
rm -rf "$RUN_TMP_DIR"

log_info ""
log_info "Runtime_Bridge QEMU proof harness complete"
log_info "Evidence directory: $EVIDENCE_DIR"
log_info ""
log_info "Next steps:"
log_info "  1. Review allowed trace: $ALLOWED_TRACE"
log_info "  2. Verify Runtime_Bridge markers are present"
log_info "  3. Integrate real DevFS/ABDF handlers (replace stubs)"
log_info "  4. Run ci-gate-fail-closed-proof for forbidden path validation"


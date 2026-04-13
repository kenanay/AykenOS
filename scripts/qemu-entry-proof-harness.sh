#!/bin/bash
# QEMU Entry Proof Harness
# HAMLE 3: Proves userspace payload actually executes
#
# This is the MINIMAL test - only checks if payload starts
# No syscall logic, no complex validation - just entry proof

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
EVIDENCE_DIR="$PROJECT_ROOT/evidence/entry-proof"

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

log_info "========================================="
log_info "HAMLE 3: Userspace Entry Proof"
log_info "========================================="
log_info ""
log_info "This test proves ONLY that userspace payload executes"
log_info "Expected marker: [RB_PAYLOAD_V1_ENTRY]"
log_info ""

# Build EFI image with entry-proof payload
log_info "Building EFI image with entry-proof payload..."
if ! USER_MINIMAL_MODE=entry-proof KERNEL_PROFILE=validation AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=1 make efi-img > "$EVIDENCE_DIR/build.log" 2>&1; then
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

if [[ "$MANIFEST_MODE" != "entry-proof" ]]; then
    log_error "❌ MODE_MISMATCH: Manifest shows selected_mode='$MANIFEST_MODE' (expected 'entry-proof')"
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
if grep -q 'DAYKEN_USER_MINIMAL_MODE_STRING="entry-proof"' "$EVIDENCE_DIR/build.log"; then
    log_info "✓ Build log shows correct mode string (diagnostic)"
else
    log_warn "⚠ Build log does not show DAYKEN_USER_MINIMAL_MODE_STRING=\"entry-proof\" (diagnostic only)"
fi

# Resolve OVMF firmware
OVMF_PAIR="$(resolve_ovmf_firmware || true)"
if [[ -z "$OVMF_PAIR" ]]; then
    log_error "OVMF firmware not found"
    exit 1
fi

OVMF_CODE="$(printf "%s\n" "$OVMF_PAIR" | sed -n '1p')"
OVMF_VARS="$(printf "%s\n" "$OVMF_PAIR" | sed -n '2p')"

log_info "Using OVMF CODE: $OVMF_CODE"

# Create temporary directory
RUN_TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/entry_proof_run.XXXXXX" 2>/dev/null || mktemp -d -t entry_proof_run 2>/dev/null)"
if [[ -z "$RUN_TMP_DIR" || ! -d "$RUN_TMP_DIR" ]]; then
    log_error "Failed to create temporary directory"
    exit 1
fi

# Prepare OVMF VARS copy
OVMF_VARS_COPY="$RUN_TMP_DIR/OVMF_VARS.fd"
OVMF_VARS_SIZE="$(wc -c < "$OVMF_VARS" 2>/dev/null | tr -d '[:space:]')"
dd if=/dev/zero of="$OVMF_VARS_COPY" bs=1 count="$OVMF_VARS_SIZE" >/dev/null 2>&1

# Prepare EFI image copy
EFI_IMG_RUN="$RUN_TMP_DIR/EFI.img"
cp -f "$EFI_IMG" "$EFI_IMG_RUN"

DEBUGCON="$EVIDENCE_DIR/qemu_debugcon.log"
SERIAL="$EVIDENCE_DIR/qemu_serial.log"

log_info ""
log_info "Running QEMU (10 second timeout)..."

# Launch QEMU
timeout 10s qemu-system-x86_64 \
    -machine q35 \
    -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
    -drive if=pflash,format=raw,file="$OVMF_VARS_COPY" \
    -drive format=raw,file="$EFI_IMG_RUN" \
    -serial file:"$SERIAL" \
    -chardev file,id=dbgcon,path="$DEBUGCON" \
    -device isa-debugcon,iobase=0xe9,chardev=dbgcon \
    -m 256M \
    -no-reboot \
    -no-shutdown \
    -display none \
    > /dev/null 2>&1 || true

# Check output channels
DEBUGCON_SIZE=0
SERIAL_SIZE=0

if [[ -f "$DEBUGCON" ]]; then
    DEBUGCON_SIZE=$(stat -c%s "$DEBUGCON" 2>/dev/null || stat -f%z "$DEBUGCON" 2>/dev/null || echo "0")
fi

if [[ -f "$SERIAL" ]]; then
    SERIAL_SIZE=$(stat -c%s "$SERIAL" 2>/dev/null || stat -f%z "$SERIAL" 2>/dev/null || echo "0")
fi

log_info ""
log_info "Channel sizes: debugcon=$DEBUGCON_SIZE bytes, serial=$SERIAL_SIZE bytes"

# HARD FAIL: All channels zero
if [[ $DEBUGCON_SIZE -eq 0 ]] && [[ $SERIAL_SIZE -eq 0 ]]; then
    log_error "❌ OUTPUT_CHANNEL_FAILURE: All output channels are empty"
    log_error "   This means kernel didn't produce ANY output"
    log_error "   Possible causes:"
    log_error "   - Kernel panic before output initialization"
    log_error "   - QEMU configuration issue"
    log_error "   - Build artifact mismatch"
    rm -rf "$RUN_TMP_DIR"
    exit 1
fi

# Check for entry marker
# NOTE: Payload outputs characters one-by-one via syscalls, so we need to extract
# characters between syscall markers before checking for the entry marker
MARKER_FOUND=0

if [[ $DEBUGCON_SIZE -gt 0 ]]; then
    # Extract characters that appear after P10_SYSCALL_ENTER markers
    # Remove syscall return markers and reconstruct the payload output
    PAYLOAD_OUTPUT=$(grep -A1 "P10_SYSCALL_ENTER" "$DEBUGCON" 2>/dev/null | \
                     grep -v "P10_SYSCALL_ENTER" | \
                     grep -v "^--$" | \
                     grep -v "^\[\[" | \
                     sed 's/\[\[AYKEN_SYSCALL_RETURN\]\]//g' | \
                     tr -d '\n' | \
                     tr -d ' ' || echo "")
    
    if echo "$PAYLOAD_OUTPUT" | grep -q "RB_PAYLOAD_V1_ENTRY"; then
        MARKER_FOUND=1
        log_info "✓ Entry marker found in debugcon (extracted from syscall output)"
    fi
fi

if [[ $SERIAL_SIZE -gt 0 ]] && [[ $MARKER_FOUND -eq 0 ]]; then
    # Try direct grep first (in case output is not via syscalls)
    if grep -q "RB_PAYLOAD_V1_ENTRY" "$SERIAL"; then
        MARKER_FOUND=1
        log_info "✓ Entry marker found in serial"
    else
        # Try extracting from syscall markers
        PAYLOAD_OUTPUT=$(grep -A1 "P10_SYSCALL_ENTER" "$SERIAL" 2>/dev/null | \
                         grep -v "P10_SYSCALL_ENTER" | \
                         grep -v "^--$" | \
                         grep -v "^\[\[" | \
                         sed 's/\[\[AYKEN_SYSCALL_RETURN\]\]//g' | \
                         tr -d '\n' | \
                         tr -d ' ' || echo "")
        
        if echo "$PAYLOAD_OUTPUT" | grep -q "RB_PAYLOAD_V1_ENTRY"; then
            MARKER_FOUND=1
            log_info "✓ Entry marker found in serial (extracted from syscall output)"
        fi
    fi
fi

# Verify boot marker (AUTHORITY)
BOOT_MODE_FOUND=0
BOOT_LOG=""

if [[ $DEBUGCON_SIZE -gt 0 ]]; then
    BOOT_LOG="$DEBUGCON"
elif [[ $SERIAL_SIZE -gt 0 ]]; then
    BOOT_LOG="$SERIAL"
fi

if [[ -n "$BOOT_LOG" ]]; then
    if grep -q '\[K\]\[PAYLOAD_MODE=entry-proof\]' "$BOOT_LOG"; then
        BOOT_MODE_FOUND=1
        log_info "✓ Boot marker [K][PAYLOAD_MODE=entry-proof] found"
    fi
fi

if [[ $BOOT_MODE_FOUND -eq 0 ]]; then
    log_error "❌ BOOT_MARKER_MISSING: Boot log does not contain [K][PAYLOAD_MODE=entry-proof]"
    log_error "   This violates the Mode Authority Invariant (boot verification)"
    rm -rf "$RUN_TMP_DIR"
    exit 1
fi

# Cleanup
rm -rf "$RUN_TMP_DIR"

log_info ""
log_info "========================================="
if [[ $MARKER_FOUND -eq 1 ]]; then
    log_info "✅ SUCCESS: Userspace payload executed"
    log_info "========================================="
    log_info ""
    log_info "Evidence:"
    log_info "  - Debugcon: $DEBUGCON"
    log_info "  - Serial: $SERIAL"
    log_info "  - Build log: $EVIDENCE_DIR/build.log"
    log_info ""
    log_info "Next steps:"
    log_info "  1. Add kernel entry markers (user_selected, user_mapped, jump_to_entry)"
    log_info "  2. Test Runtime_Bridge syscall handlers"
    log_info "  3. Verify ABDF/DevFS integration"
    exit 0
else
    log_error "❌ FAILURE: Entry marker NOT found"
    log_info "========================================="
    log_info ""
    log_info "Kernel produced output but payload didn't execute"
    log_info "This means:"
    log_info "  - Kernel boots successfully"
    log_info "  - BUT userspace entry path is broken"
    log_info ""
    log_info "Debug files:"
    log_info "  - Debugcon: $DEBUGCON"
    log_info "  - Serial: $SERIAL"
    log_info "  - Build log: $EVIDENCE_DIR/build.log"
    log_info ""
    log_info "Check for:"
    log_info "  - User ELF selection logic"
    log_info "  - User ELF mapping"
    log_info "  - Jump to userspace entry point"
    exit 1
fi

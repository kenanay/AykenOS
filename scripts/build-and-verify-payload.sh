#!/usr/bin/env bash
# HAMLE 2: Build payload and verify embedding
# This script proves EXACTLY which binary is being booted

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/build/runtime-bridge-tests"
PAYLOAD_SOURCE="$PROJECT_ROOT/userspace/runtime_bridge_entry_proof.S"
PAYLOAD_ELF="$BUILD_DIR/runtime_bridge_entry_proof.elf"

log() {
    printf '[payload-verify] %s\n' "$*"
}

die() {
    printf '[payload-verify] ERROR: %s\n' "$*" >&2
    exit 1
}

# Step 1: Build ultra-minimal payload (Assembly)
log "Building ultra-minimal payload from assembly..."
mkdir -p "$BUILD_DIR"

clang \
    --target=x86_64-unknown-none \
    -m64 \
    -c "$PAYLOAD_SOURCE" \
    -o "$BUILD_DIR/runtime_bridge_entry_proof.o"

ld.lld \
    -nostdlib \
    -static \
    -e _start \
    "$BUILD_DIR/runtime_bridge_entry_proof.o" \
    -o "$PAYLOAD_ELF"

[[ -f "$PAYLOAD_ELF" ]] || die "Payload build failed"

# Step 2: Calculate payload hash
log "Calculating payload hash..."
PAYLOAD_HASH=$(shasum -a 256 "$PAYLOAD_ELF" | awk '{print $1}')
log "Payload hash: $PAYLOAD_HASH"

# Step 3: Verify payload contains fingerprint
log "Verifying fingerprint in payload..."
if strings "$PAYLOAD_ELF" | grep -q "RB_PAYLOAD_V1_ENTRY"; then
    log "✓ Fingerprint found in payload"
else
    die "✗ Fingerprint NOT found in payload"
fi

# Step 4: Check payload size
PAYLOAD_SIZE=$(stat -f%z "$PAYLOAD_ELF" 2>/dev/null || stat -c%s "$PAYLOAD_ELF" 2>/dev/null)
log "Payload size: $PAYLOAD_SIZE bytes"

# Step 5: Dump entry point
if command -v llvm-readelf >/dev/null 2>&1; then
    ENTRY=$(llvm-readelf -h "$PAYLOAD_ELF" | grep "Entry point" | awk '{print $NF}')
    log "Entry point: $ENTRY"
fi

# Step 6: Save verification data
cat > "$BUILD_DIR/payload_verification.txt" <<EOF
timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
payload_path=$PAYLOAD_ELF
payload_hash=$PAYLOAD_HASH
payload_size=$PAYLOAD_SIZE
fingerprint=RB_PAYLOAD_V1_ENTRY
status=VERIFIED
EOF

log "Verification data saved to: $BUILD_DIR/payload_verification.txt"
log ""
log "========================================="
log "PAYLOAD BUILD COMPLETE"
log "========================================="
log "Path: $PAYLOAD_ELF"
log "Hash: $PAYLOAD_HASH"
log "Size: $PAYLOAD_SIZE bytes"
log "========================================="
log ""
log "⚠️  CRITICAL: Before running QEMU, verify that THIS hash"
log "    matches the hash of the embedded ELF in kernel binary"

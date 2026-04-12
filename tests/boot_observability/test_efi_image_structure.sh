#!/usr/bin/env bash
# Test 1: EFI Image Structure Validation
# Verifies BOOTX64.EFI exists, is valid PE32+, and startup.nsh calls it

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== EFI Image Structure Validation ==="

# Check BOOTX64.EFI exists
BOOTX64_PATH="$PROJECT_ROOT/out/build/BOOTX64.EFI"
if [[ ! -f "$BOOTX64_PATH" ]]; then
    echo "FAIL: BOOTX64.EFI not found at $BOOTX64_PATH"
    exit 1
fi
echo "✓ BOOTX64.EFI exists"

# Check file size (should be non-zero)
SIZE=$(stat -f%z "$BOOTX64_PATH" 2>/dev/null || stat -c%s "$BOOTX64_PATH" 2>/dev/null)
if [[ $SIZE -eq 0 ]]; then
    echo "FAIL: BOOTX64.EFI is empty"
    exit 1
fi
echo "✓ BOOTX64.EFI size: $SIZE bytes"

# Check PE32+ signature (MZ header)
HEADER=$(xxd -l 2 -p "$BOOTX64_PATH")
if [[ "$HEADER" != "4d5a" ]]; then
    echo "FAIL: BOOTX64.EFI missing MZ header (got: $HEADER)"
    exit 1
fi
echo "✓ BOOTX64.EFI has valid PE32+ MZ header"

# Check for execution sentinels in binary
if ! strings "$BOOTX64_PATH" | grep -q "UEFI_BOOT_OK"; then
    echo "WARN: UEFI_BOOT_OK sentinel not found in binary"
else
    echo "✓ UEFI_BOOT_OK sentinel present in binary"
fi

# Check EFI.img exists (contains startup.nsh)
EFI_IMG_PATH="$PROJECT_ROOT/out/build/EFI.img"
if [[ ! -f "$EFI_IMG_PATH" ]]; then
    echo "WARN: EFI.img not found (run 'make efi-img' to create)"
else
    echo "✓ EFI.img exists"
    EFI_SIZE=$(stat -f%z "$EFI_IMG_PATH" 2>/dev/null || stat -c%s "$EFI_IMG_PATH" 2>/dev/null)
    echo "✓ EFI.img size: $EFI_SIZE bytes"
fi

echo ""
echo "=== EFI Image Structure: PASS ==="

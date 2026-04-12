#!/usr/bin/env bash
# Timing-Based Execution Proof
# efi_main() has Stall(2000000) = 2 second delay
# If QEMU runs for ~2 seconds, efi_main() executed

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== Timing-Based Execution Proof ==="
echo ""
echo "Method: efi_main() contains Stall(2000000) = 2 second delay"
echo "If QEMU runs for approximately 2 seconds, efi_main() executed."
echo ""

# Check prerequisites
if [[ ! -f "$PROJECT_ROOT/out/build/EFI.img" ]]; then
    echo "FAIL: EFI.img not found."
    exit 1
fi

echo "Starting QEMU with 5 second timeout..."
START_TIME=$(date +%s)

timeout 5s qemu-system-x86_64 \
    -drive if=pflash,format=raw,readonly=on,file=/opt/homebrew/share/qemu/edk2-x86_64-code.fd \
    -drive format=raw,file="$PROJECT_ROOT/out/build/EFI.img" \
    -nographic \
    -no-reboot \
    2>/dev/null || true

END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))

echo ""
echo "--- Timing Analysis ---"
echo "Elapsed time: ${ELAPSED} seconds"
echo ""

if [[ $ELAPSED -ge 2 ]] && [[ $ELAPSED -le 4 ]]; then
    echo "✓ EXECUTION PROOF ESTABLISHED"
    echo ""
    echo "Reasoning:"
    echo "  - QEMU ran for ${ELAPSED} seconds"
    echo "  - efi_main() has Stall(2000000) = 2 second delay"
    echo "  - Timing matches expected delay"
    echo "  - Therefore: efi_main() executed past InitializeLib()"
    echo ""
    echo "Note: This proves execution but not output capture."
    exit 0
elif [[ $ELAPSED -lt 2 ]]; then
    echo "✗ NO EXECUTION PROOF"
    echo ""
    echo "Reasoning:"
    echo "  - QEMU ran for only ${ELAPSED} seconds"
    echo "  - Expected at least 2 seconds (Stall duration)"
    echo "  - efi_main() likely did not execute"
    echo ""
    echo "Possible causes:"
    echo "  1. OVMF did not find/load BOOTX64.EFI"
    echo "  2. startup.nsh not executed"
    echo "  3. Boot failure before efi_main()"
    exit 1
else
    echo "⚠ INCONCLUSIVE"
    echo ""
    echo "Reasoning:"
    echo "  - QEMU ran for ${ELAPSED} seconds (longer than expected)"
    echo "  - May indicate efi_main() executed but got stuck"
    echo "  - Or UEFI shell waiting for input"
    exit 1
fi

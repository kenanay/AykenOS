#!/usr/bin/env bash
# Phase 4.4 QEMU Boot Audit Script (deterministic, fail-closed)
# Purpose: Produce audit-grade evidence for QEMU boot success.

set -euo pipefail

# Defaults
QEMU_TIMEOUT=30
MARKER="[K][BOOT_OK] Phase 4.4 minimal boot reached"
OUT_DIR=""

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info() { echo -e "${CYAN}ℹ $1${NC}"; }
success() { echo -e "${GREEN}✓ $1${NC}"; }
error() { echo -e "${RED}✗ $1${NC}"; }
warning() { echo -e "${YELLOW}⚠ $1${NC}"; }

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

detect_timeout_cmd() {
    if command_exists "timeout"; then
        echo "timeout"
    elif command_exists "gtimeout"; then
        echo "gtimeout"
    else
        echo ""
    fi
}

usage() {
    cat <<EOF
Phase 4.4 QEMU Boot Audit (deterministic, fail-closed)

Usage: $0 [OPTIONS]

Options:
  --timeout N       Set timeout in seconds (default: 30)
  --marker TEXT     Canonical boot marker (default: ${MARKER})
  --out-dir PATH    Output directory for logs (default: reports/phase_4_4_closure_YYYY-MM-DD)
  --help            Show this help
EOF
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --timeout)
            QEMU_TIMEOUT="$2"
            shift 2
            ;;
        --marker)
            MARKER="$2"
            shift 2
            ;;
        --out-dir)
            OUT_DIR="$2"
            shift 2
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            usage
            exit 2
            ;;
    esac
done

DATE_STAMP="$(date +%Y-%m-%d)"
if [[ -z "$OUT_DIR" ]]; then
    OUT_DIR="reports/phase_4_4_closure_${DATE_STAMP}"
fi
mkdir -p "$OUT_DIR"

LOG_OUT="${OUT_DIR}/qemu_boot.log"
LOG_ERR="${OUT_DIR}/qemu_boot.err"

info "Phase 4.4 QEMU Boot Audit"
info "Timeout: ${QEMU_TIMEOUT}s"
info "Marker: ${MARKER}"
info "Logs: ${LOG_OUT}, ${LOG_ERR}"

if ! command_exists "qemu-system-x86_64"; then
    error "QEMU not found (qemu-system-x86_64). Audit cannot proceed."
    exit 2
fi

if [[ ! -f "EFI.img" ]]; then
    info "Creating EFI image..."
    if [[ -x "./make_efi_img.sh" ]]; then
        ./make_efi_img.sh
    elif command_exists "make"; then
        make efi-img
    else
        error "Cannot create EFI image - no creation method available."
        exit 2
    fi
fi

timeout_cmd="$(detect_timeout_cmd)"
if [[ -z "$timeout_cmd" ]]; then
    warning "No timeout command found; enforcing manual time limit."
fi

boot_success=false
qemu_pid=""
start_time=$(date +%s)

if [[ -n "$timeout_cmd" ]]; then
    "$timeout_cmd" "$QEMU_TIMEOUT" qemu-system-x86_64 \
        -drive format=raw,file=EFI.img \
        -serial stdio \
        -display none \
        -no-reboot \
        -no-shutdown \
        > "$LOG_OUT" 2> "$LOG_ERR" &
else
    qemu-system-x86_64 \
        -drive format=raw,file=EFI.img \
        -serial stdio \
        -display none \
        -no-reboot \
        -no-shutdown \
        > "$LOG_OUT" 2> "$LOG_ERR" &
fi

qemu_pid=$!

while kill -0 "$qemu_pid" 2>/dev/null; do
    current_time=$(date +%s)
    if (( current_time - start_time > QEMU_TIMEOUT )); then
        break
    fi

    if [[ -f "$LOG_OUT" ]]; then
        if grep -q -F "$MARKER" "$LOG_OUT"; then
            boot_success=true
            success "Canonical boot marker detected."
            break
        fi
    fi

    sleep 0.5
done

if kill -0 "$qemu_pid" 2>/dev/null; then
    kill "$qemu_pid" 2>/dev/null || true
    wait "$qemu_pid" 2>/dev/null || true
fi

echo ""
echo "================== Phase 4.4 QEMU Boot Audit =================="
echo "Marker: ${MARKER}"
echo "Timeout: ${QEMU_TIMEOUT}s"
echo "Logs: ${LOG_OUT}, ${LOG_ERR}"
echo "Result: $([ "$boot_success" == "true" ] && echo "PASS" || echo "FAIL")"
echo "==============================================================="

if [[ "$boot_success" == "true" ]]; then
    exit 0
fi

error "QEMU boot did not reach canonical BOOT_OK marker."
exit 2

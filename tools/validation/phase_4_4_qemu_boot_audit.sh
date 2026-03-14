#!/usr/bin/env bash
# Phase 4.4 QEMU Boot Audit Script (deterministic, fail-closed)
# Purpose: Produce audit-grade evidence for QEMU boot success.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "${ROOT}/tools/lib/ayken_path_contract.sh"
cd "${ROOT}"
ayken_prepare_out_dirs

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
  --out-dir PATH    Output directory for logs (default: out/reports/phase_4_4_closure_YYYY-MM-DD)
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
    OUT_DIR="${AYKEN_LOCAL_REPORT_DIR}/phase_4_4_closure_${DATE_STAMP}"
fi
mkdir -p "$OUT_DIR"

EFI_IMAGE="${EFI_IMG:-${AYKEN_EFI_IMG}}"

LOG_OUT="${OUT_DIR}/qemu_boot.log"
LOG_ERR="${OUT_DIR}/qemu_boot.err"
LOG_SERIAL="${OUT_DIR}/qemu_serial.log"
LOG_DEBUGCON="${OUT_DIR}/qemu_debugcon.log"

EARLY_MARKER="[K][EARLY_BOOT_OK]"

info "Phase 4.4 QEMU Boot Audit"
info "Timeout: ${QEMU_TIMEOUT}s"
info "Marker: ${MARKER}"
info "Logs: ${LOG_OUT}, ${LOG_ERR}"

if ! command_exists "qemu-system-x86_64"; then
    error "QEMU not found (qemu-system-x86_64). Audit cannot proceed."
    exit 2
fi

if [[ ! -f "${EFI_IMAGE}" ]]; then
    info "Creating EFI image..."
    if command_exists "make"; then
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

# Optional UEFI firmware (OVMF) for deterministic boot
ovmf_code=""
ovmf_vars=""
ovmf_vars_copy=""
if [[ -f "OVMF_CODE.fd" && -f "OVMF_VARS.fd" ]]; then
    ovmf_code="OVMF_CODE.fd"
    ovmf_vars="OVMF_VARS.fd"
elif [[ -f "firmware/ovmf/OVMF_CODE.fd" && -f "firmware/ovmf/OVMF_VARS.fd" ]]; then
    ovmf_code="firmware/ovmf/OVMF_CODE.fd"
    ovmf_vars="firmware/ovmf/OVMF_VARS.fd"
elif [[ -f "/opt/homebrew/share/qemu/edk2-x86_64-code.fd" && -f "/opt/homebrew/share/qemu/edk2-x86_64-vars.fd" ]]; then
    ovmf_code="/opt/homebrew/share/qemu/edk2-x86_64-code.fd"
    ovmf_vars="/opt/homebrew/share/qemu/edk2-x86_64-vars.fd"
fi

if [[ -n "$ovmf_code" && -n "$ovmf_vars" ]]; then
    ovmf_vars_copy="${OUT_DIR}/OVMF_VARS.fd"
    cp -f "$ovmf_vars" "$ovmf_vars_copy"
    info "OVMF detected: ${ovmf_code}"
    info "OVMF vars copy: ${ovmf_vars_copy}"
fi

qemu_args=()
if [[ -n "$ovmf_code" && -n "$ovmf_vars_copy" ]]; then
    qemu_args+=(
        -machine pc
        -drive "if=pflash,format=raw,readonly=on,file=${ovmf_code}"
        -drive "if=pflash,format=raw,file=${ovmf_vars_copy}"
    )
fi
# Always use EFI.img regardless of OVMF presence
qemu_args+=(
    -drive "format=raw,file=${EFI_IMAGE}"
)
qemu_args+=(
    -display none
    -no-reboot
    -no-shutdown
    -serial "file:${LOG_SERIAL}"
    -debugcon "file:${LOG_DEBUGCON}"
    -global isa-debugcon.iobase=0xe9
)

boot_success=false
early_seen=false
late_seen=false
qemu_pid=""
start_time=$(date +%s)

if [[ -n "$timeout_cmd" ]]; then
    "$timeout_cmd" "$QEMU_TIMEOUT" qemu-system-x86_64 \
        "${qemu_args[@]}" \
        > "$LOG_OUT" 2> "$LOG_ERR" &
else
    qemu-system-x86_64 \
        "${qemu_args[@]}" \
        > "$LOG_OUT" 2> "$LOG_ERR" &
fi

qemu_pid=$!

while kill -0 "$qemu_pid" 2>/dev/null; do
    current_time=$(date +%s)
    if (( current_time - start_time > QEMU_TIMEOUT )); then
        break
    fi

    if [[ -f "$LOG_DEBUGCON" ]]; then
        if grep -q -F "$EARLY_MARKER" "$LOG_DEBUGCON"; then
            early_seen=true
        fi
        if grep -q -F "$MARKER" "$LOG_DEBUGCON"; then
            late_seen=true
        fi
    fi

    if [[ -f "$LOG_SERIAL" ]]; then
        if grep -q -F "$MARKER" "$LOG_SERIAL"; then
            late_seen=true
        fi
    fi

    if [[ "$late_seen" == "true" ]]; then
        boot_success=true
        success "Canonical boot marker detected."
        break
    fi

    sleep 0.5
done

if kill -0 "$qemu_pid" 2>/dev/null; then
    kill "$qemu_pid" 2>/dev/null || true
    wait "$qemu_pid" 2>/dev/null || true
fi

echo ""
echo "================== Phase 4.4 QEMU Boot Audit =================="
echo "Early Marker: ${EARLY_MARKER}"
echo "Late Marker: ${MARKER}"
echo "Timeout: ${QEMU_TIMEOUT}s"
echo "Logs: ${LOG_OUT}, ${LOG_ERR}"
echo "Serial: ${LOG_SERIAL}"
echo "Debugcon: ${LOG_DEBUGCON}"
echo "Result: $([ "$boot_success" == "true" ] && echo "PASS" || echo "FAIL")"
echo "==============================================================="

if [[ "$boot_success" == "true" ]]; then
    exit 0
fi

if [[ "$early_seen" == "true" ]]; then
    error "EARLY marker seen but BOOT_OK not reached (init crash likely)."
else
    error "No EARLY marker found (likely no kernel entry / boot path issue)."
fi
exit 2

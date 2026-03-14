#!/usr/bin/env bash
# Phase 4.4 Syscall Roundtrip Audit Script (deterministic, fail-closed)
# Purpose: Produce audit-grade evidence for syscall roundtrip readiness.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "${ROOT}/tools/lib/ayken_path_contract.sh"
cd "${ROOT}"
ayken_prepare_out_dirs

TIMEOUT=50
MARKER="[U][SYSCALL_OK]"
FALLBACK_MARKER="[[AYKEN_SYSCALL_V2_OK]]"
OUT_DIR=""

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

marker_present_in_file() {
    local path="$1"

    if [[ ! -f "$path" ]]; then
        return 1
    fi

    if grep -a -q -F "$MARKER" "$path" 2>/dev/null; then
        return 0
    fi
    if grep -a -q -F "$FALLBACK_MARKER" "$path" 2>/dev/null; then
        return 0
    fi
    return 1
}

usage() {
    cat <<EOF
Phase 4.4 Syscall Roundtrip Audit (deterministic, fail-closed)

Usage: $0 [OPTIONS]

Options:
  --timeout N       Set timeout in seconds (default: 50)
  --marker TEXT     Canonical marker (default: ${MARKER})
  --out-dir PATH    Output directory for logs (default: out/reports/phase_4_4_closure_YYYY-MM-DD)
  --help            Show this help
EOF
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --timeout)
            TIMEOUT="$2"
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

LOG_OUT="${OUT_DIR}/syscall_output.log"
LOG_ERR="${OUT_DIR}/syscall_error.log"
LOG_ANALYSIS="${OUT_DIR}/syscall_analysis.log"
LOG_QEMU_DEBUG="${OUT_DIR}/syscall_qemu_debug.log"
AUDIT_LOG="${OUT_DIR}/syscall_audit.log"
META_LOG="${OUT_DIR}/syscall_audit_meta.log"
SERIAL_LOG="${OUT_DIR}/syscall_serial.log"
DEBUGCON_LOG="${OUT_DIR}/syscall_debugcon.log"

info "Phase 4.4 Syscall Roundtrip Audit"
info "Timeout: ${TIMEOUT}s"
info "Marker (primary): ${MARKER}"
info "Marker (fallback): ${FALLBACK_MARKER}"
info "Logs: ${LOG_OUT}, ${LOG_ERR}, ${LOG_ANALYSIS}"
info "Serial: ${SERIAL_LOG}, DebugCon: ${DEBUGCON_LOG}"

timeout_cmd="$(detect_timeout_cmd)"
qemu_int_trace_mode="${SYSCALL_QEMU_INT_TRACE:-auto}"
qemu_accel_mode="${SYSCALL_QEMU_ACCEL:-auto}"
watchdog_grace_seconds="${SYSCALL_AUDIT_WRAPPER_GRACE_SECONDS:-10}"
if ! [[ "${watchdog_grace_seconds}" =~ ^[0-9]+$ ]]; then
    echo "Invalid SYSCALL_AUDIT_WRAPPER_GRACE_SECONDS='${watchdog_grace_seconds}' (expected non-negative integer)." >&2
    exit 2
fi
hard_timeout_seconds=$((TIMEOUT + watchdog_grace_seconds))
{
    echo "phase_4_4_syscall_roundtrip_audit"
    echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "host=$(uname -a)"
    echo "git_commit=$(git rev-parse HEAD 2>/dev/null || echo unknown)"
    echo "qemu_version=$(qemu-system-x86_64 --version 2>/dev/null | head -n1 || echo unknown)"
    echo "timeout_cmd=${timeout_cmd:-none}"
    echo "timeout_seconds=${TIMEOUT}"
    echo "marker=${MARKER}"
    echo "fallback_marker=${FALLBACK_MARKER}"
    echo "qemu_int_trace_mode=${qemu_int_trace_mode}"
    echo "qemu_accel_mode=${qemu_accel_mode}"
    echo "watchdog_grace_seconds=${watchdog_grace_seconds}"
    echo "hard_timeout_seconds=${hard_timeout_seconds}"
    echo "command=./tools/validation/syscall_roundtrip_test.sh --timeout ${TIMEOUT} --save-logs"
} > "$META_LOG"

forced_timeout=false
set +e
SYSCALL_SERIAL_LOG="$SERIAL_LOG" SYSCALL_DEBUGCON_LOG="$DEBUGCON_LOG" ./tools/validation/syscall_roundtrip_test.sh --timeout "$TIMEOUT" --save-logs > "$AUDIT_LOG" 2>&1 &
test_pid=$!
start_time=$(date +%s)
while kill -0 "$test_pid" 2>/dev/null; do
    current_time=$(date +%s)
    if (( current_time - start_time > hard_timeout_seconds )); then
        forced_timeout=true
        kill "$test_pid" 2>/dev/null || true
        for _ in $(seq 1 20); do
            if ! kill -0 "$test_pid" 2>/dev/null; then
                break
            fi
            sleep 0.1
        done
        if kill -0 "$test_pid" 2>/dev/null; then
            kill -9 "$test_pid" 2>/dev/null || true
        fi
        wait "$test_pid" 2>/dev/null || true
        break
    fi
    sleep 0.5
done
wait "$test_pid" 2>/dev/null
test_exit=$?
set -e

if [[ "$forced_timeout" == "true" ]]; then
    echo "forced_timeout=true" >> "$META_LOG"
    echo "forced_timeout=true" >> "$AUDIT_LOG"
fi

# Expected log names from syscall_roundtrip_test.sh
SRC_OUT="syscall_roundtrip_output.log"
SRC_ERR="syscall_roundtrip_error.log"
SRC_ANALYSIS="syscall_roundtrip_analysis.log"
SRC_QEMU_DEBUG="qemu_syscall_debug.log"

if [[ -f "$SRC_OUT" ]]; then
    cp -a "$SRC_OUT" "$LOG_OUT"
fi
if [[ -f "$SRC_ERR" ]]; then
    cp -a "$SRC_ERR" "$LOG_ERR"
fi
if [[ -f "$SRC_ANALYSIS" ]]; then
    cp -a "$SRC_ANALYSIS" "$LOG_ANALYSIS"
fi
if [[ -f "$SRC_QEMU_DEBUG" ]]; then
    cp -a "$SRC_QEMU_DEBUG" "$LOG_QEMU_DEBUG"
fi

if [[ ! -f "$LOG_OUT" ]]; then
    error "Syscall output log missing. Checking other logs..."
    # Don't exit immediately, check other logs first
fi

# Check for marker in any available log file
marker_found=false
if [[ -f "$LOG_OUT" && -s "$LOG_OUT" ]] && marker_present_in_file "$LOG_OUT"; then
    marker_found=true
elif marker_present_in_file "$LOG_ERR"; then
    marker_found=true
elif marker_present_in_file "$LOG_ANALYSIS"; then
    marker_found=true
elif marker_present_in_file "$LOG_QEMU_DEBUG"; then
    marker_found=true
elif marker_present_in_file "$AUDIT_LOG"; then
    marker_found=true
elif marker_present_in_file "$SERIAL_LOG"; then
    marker_found=true
elif marker_present_in_file "$DEBUGCON_LOG"; then
    marker_found=true
fi

if [[ "$marker_found" == "true" ]]; then
    success "Canonical syscall marker detected."
    {
        echo "hash_syscall_output=$(sha256sum "$LOG_OUT" 2>/dev/null | awk '{print $1}' || echo 'missing')"
        echo "hash_syscall_error=$(sha256sum "$LOG_ERR" 2>/dev/null | awk '{print $1}' || echo 'missing')"
        echo "hash_syscall_analysis=$(sha256sum "$LOG_ANALYSIS" 2>/dev/null | awk '{print $1}' || echo 'missing')"
        echo "hash_syscall_qemu_debug=$(sha256sum "$LOG_QEMU_DEBUG" 2>/dev/null | awk '{print $1}' || echo 'missing')"
        echo "hash_syscall_audit=$(sha256sum "$AUDIT_LOG" 2>/dev/null | awk '{print $1}' || echo 'missing')"
        echo "hash_syscall_serial=$(sha256sum "$SERIAL_LOG" 2>/dev/null | awk '{print $1}' || echo 'missing')"
        echo "hash_syscall_debugcon=$(sha256sum "$DEBUGCON_LOG" 2>/dev/null | awk '{print $1}' || echo 'missing')"
    } >> "$META_LOG"
    exit 0
fi

warning "Canonical syscall marker not found."
warning "Underlying test exit code: ${test_exit}"
error "Syscall did not reach canonical marker (FAIL)."
exit 2

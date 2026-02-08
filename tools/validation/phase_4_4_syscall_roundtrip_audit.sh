#!/usr/bin/env bash
# Phase 4.4 Syscall Roundtrip Audit Script (deterministic, fail-closed)
# Purpose: Produce audit-grade evidence for syscall roundtrip readiness.

set -euo pipefail

TIMEOUT=50
MARKER="[U][SYSCALL_OK] Phase 4.4 syscall roundtrip ok"
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

usage() {
    cat <<EOF
Phase 4.4 Syscall Roundtrip Audit (deterministic, fail-closed)

Usage: $0 [OPTIONS]

Options:
  --timeout N       Set timeout in seconds (default: 50)
  --marker TEXT     Canonical marker (default: ${MARKER})
  --out-dir PATH    Output directory for logs (default: reports/phase_4_4_closure_YYYY-MM-DD)
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
    OUT_DIR="reports/phase_4_4_closure_${DATE_STAMP}"
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
info "Marker: ${MARKER}"
info "Logs: ${LOG_OUT}, ${LOG_ERR}, ${LOG_ANALYSIS}"
info "Serial: ${SERIAL_LOG}, DebugCon: ${DEBUGCON_LOG}"

timeout_cmd="$(detect_timeout_cmd)"
{
    echo "phase_4_4_syscall_roundtrip_audit"
    echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "host=$(uname -a)"
    echo "git_commit=$(git rev-parse HEAD 2>/dev/null || echo unknown)"
    echo "qemu_version=$(qemu-system-x86_64 --version 2>/dev/null | head -n1 || echo unknown)"
    echo "timeout_cmd=${timeout_cmd:-none}"
    echo "timeout_seconds=${TIMEOUT}"
    echo "marker=${MARKER}"
    echo "command=./tools/validation/syscall_roundtrip_test.sh --timeout ${TIMEOUT} --save-logs"
} > "$META_LOG"

forced_timeout=false
set +e
SYSCALL_SERIAL_LOG="$SERIAL_LOG" SYSCALL_DEBUGCON_LOG="$DEBUGCON_LOG" ./tools/validation/syscall_roundtrip_test.sh --timeout "$TIMEOUT" --save-logs > "$AUDIT_LOG" 2>&1 &
test_pid=$!
start_time=$(date +%s)
while kill -0 "$test_pid" 2>/dev/null; do
    current_time=$(date +%s)
    if (( current_time - start_time > TIMEOUT )); then
        forced_timeout=true
        kill "$test_pid" 2>/dev/null || true
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
    error "Syscall output log missing. Audit FAIL."
    exit 2
fi

if [[ ! -s "$LOG_OUT" ]]; then
    error "Syscall output log is empty (zero detection = FAIL)."
    exit 2
fi

if grep -q -F "$MARKER" "$LOG_OUT" || \
   grep -q -F "$MARKER" "$LOG_ERR" || \
   grep -q -F "$MARKER" "$LOG_ANALYSIS" || \
   grep -q -F "$MARKER" "$LOG_QEMU_DEBUG" || \
   grep -q -F "$MARKER" "$AUDIT_LOG" || \
   grep -q -F "$MARKER" "$SERIAL_LOG" || \
   grep -q -F "$MARKER" "$DEBUGCON_LOG"; then
    success "Canonical syscall marker detected."
    {
        echo "hash_syscall_output=$(sha256sum "$LOG_OUT" | awk '{print $1}')"
        echo "hash_syscall_error=$(sha256sum "$LOG_ERR" 2>/dev/null | awk '{print $1}')"
        echo "hash_syscall_analysis=$(sha256sum "$LOG_ANALYSIS" 2>/dev/null | awk '{print $1}')"
        echo "hash_syscall_qemu_debug=$(sha256sum "$LOG_QEMU_DEBUG" 2>/dev/null | awk '{print $1}')"
        echo "hash_syscall_audit=$(sha256sum "$AUDIT_LOG" | awk '{print $1}')"
        echo "hash_syscall_serial=$(sha256sum "$SERIAL_LOG" 2>/dev/null | awk '{print $1}')"
        echo "hash_syscall_debugcon=$(sha256sum "$DEBUGCON_LOG" 2>/dev/null | awk '{print $1}')"
    } >> "$META_LOG"
    exit 0
fi

warning "Canonical syscall marker not found."
warning "Underlying test exit code: ${test_exit}"
error "Syscall did not reach canonical marker (FAIL)."
exit 2

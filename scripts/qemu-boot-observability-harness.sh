#!/usr/bin/env bash

# qemu-boot-observability-harness.sh
#
# Deterministic, CI-safe, cross-platform QEMU boot observability harness
#
# Properties:
# - No pipes/tee around QEMU
# - stdin detached from TTY
# - per-run isolated OVMF vars
# - per-run isolated evidence directory
# - portable timeout wrapper
# - canonical output publication for CI gate
# - safe for repeated sequential CI invocation
#
# Note:
# - This script is designed for single-invocation CI use.
# - Parallel QEMU runs on macOS/TCG are not treated as a supported contract.
#
# Exit codes:
#   0 = evidence generated successfully
#   1 = hard failure (all channels empty / missing prerequisites)
#   2 = usage/configuration failure

set -u

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEFAULT_EVIDENCE_DIR="$PROJECT_ROOT/evidence/boot-observability"
EVIDENCE_DIR="${EVIDENCE_DIR:-$DEFAULT_EVIDENCE_DIR}"

EFI_IMAGE="${EFI_IMAGE:-$PROJECT_ROOT/EFI.img}"
OVMF_CODE="${OVMF_CODE:-$PROJECT_ROOT/firmware/ovmf/OVMF_CODE.fd}"
OVMF_VARS_TEMPLATE="${OVMF_VARS_TEMPLATE:-$PROJECT_ROOT/firmware/ovmf/OVMF_VARS.fd}"

QEMU_BIN="${QEMU_BIN:-qemu-system-x86_64}"
QEMU_TIMEOUT_SECS="${QEMU_TIMEOUT_SECS:-45}"
QEMU_MEMORY="${QEMU_MEMORY:-256M}"
QEMU_MACHINE="${QEMU_MACHINE:-q35}"
QEMU_CPU="${QEMU_CPU:-qemu64}"

PUBLISH_CANONICAL="${PUBLISH_CANONICAL:-1}"   # 1=copy final artifacts to EVIDENCE_DIR
KEEP_RUN_DIR="${KEEP_RUN_DIR:-0}"             # 1=keep temp run dir for debugging
SYNC_AFTER_QEMU="${SYNC_AFTER_QEMU:-1}"       # 1=run sync after QEMU exits/times out
POST_QEMU_SLEEP_SECS="${POST_QEMU_SLEEP_SECS:-1}"

RED=$'\033[0;31m'
GREEN=$'\033[0;32m'
YELLOW=$'\033[1;33m'
BLUE=$'\033[0;34m'
NC=$'\033[0m'

log_info()  { printf "%s[INFO]%s %s\n" "$GREEN" "$NC" "$*"; }
log_warn()  { printf "%s[WARN]%s %s\n" "$YELLOW" "$NC" "$*"; }
log_error() { printf "%s[ERROR]%s %s\n" "$RED" "$NC" "$*"; }

die() {
  log_error "$1"
  exit "${2:-1}"
}

require_file() {
  local path="$1"
  local label="$2"
  [[ -f "$path" ]] || die "$label not found: $path" 2
}

portable_stat_size() {
  local file="$1"
  if [[ ! -f "$file" ]]; then
    echo "0"
    return 0
  fi
  
  stat -c%s "$file" 2>/dev/null && return 0
  stat -f%z "$file" 2>/dev/null && return 0
  wc -c < "$file" 2>/dev/null | tr -d ' ' && return 0
  
  echo "0"
}

find_timeout_cmd() {
  if command -v timeout >/dev/null 2>&1; then
    echo "timeout"
    return 0
  fi
  if command -v gtimeout >/dev/null 2>&1; then
    echo "gtimeout"
    return 0
  fi
  echo ""
}

run_with_timeout() {
  local duration="$1"
  shift
  
  local timeout_cmd
  timeout_cmd="$(find_timeout_cmd)"
  
  if [[ -n "$timeout_cmd" ]]; then
    # SIGTERM first, SIGKILL later. Foreground mode reduces TTY weirdness.
    "$timeout_cmd" --foreground --signal=TERM --kill-after=5s "${duration}s" "$@"
    return $?
  fi
  
  if command -v python3 >/dev/null 2>&1; then
    python3 - "$duration" "$@" <<'PY'
import os, signal, subprocess, sys, time

duration = int(sys.argv[1])
cmd = sys.argv[2:]

p = subprocess.Popen(cmd, stdin=subprocess.DEVNULL,
                     stdout=subprocess.DEVNULL,
                     stderr=subprocess.DEVNULL)

deadline = time.time() + duration
while True:
    rc = p.poll()
    if rc is not None:
        sys.exit(rc)
    if time.time() >= deadline:
        try:
            p.terminate()
        except ProcessLookupError:
            pass
        try:
            p.wait(timeout=5)
            sys.exit(124)
        except subprocess.TimeoutExpired:
            try:
                p.kill()
            except ProcessLookupError:
                pass
            p.wait()
            sys.exit(124)
    time.sleep(0.1)
PY
    return $?
  fi
  
  die "No timeout implementation available (need timeout/gtimeout or python3)" 2
}

copy_if_exists() {
  local src="$1"
  local dst="$2"
  if [[ -f "$src" ]]; then
    cp -f "$src" "$dst"
  fi
}

# Optional single-invocation lock for canonical CI target.
# Portable and simple: mkdir lock.
acquire_lock() {
  local lock_dir="$1"
  local waited=0
  local max_wait=60
  
  while ! mkdir "$lock_dir" 2>/dev/null; do
    sleep 1
    waited=$((waited + 1))
    if (( waited >= max_wait )); then
      die "Could not acquire harness lock: $lock_dir" 1
    fi
  done
}

release_lock() {
  local lock_dir="$1"
  rmdir "$lock_dir" 2>/dev/null || true
}

log_info "QEMU Boot Observability Test Harness"
log_info "====================================="

require_file "$EFI_IMAGE" "EFI image"
require_file "$OVMF_CODE" "OVMF firmware code"
require_file "$OVMF_VARS_TEMPLATE" "OVMF vars template"

mkdir -p "$EVIDENCE_DIR"

# Canonical shared lock only matters when publishing canonical outputs.
LOCK_DIR="$EVIDENCE_DIR/.harness.lock"
LOCK_ACQUIRED=0
if [[ "$PUBLISH_CANONICAL" == "1" ]]; then
  acquire_lock "$LOCK_DIR"
  LOCK_ACQUIRED=1
fi

RUN_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/ayken-bootobs.XXXXXX")"
RUN_EVIDENCE_DIR="$RUN_ROOT/evidence"
mkdir -p "$RUN_EVIDENCE_DIR"

RUN_OVMF_VARS="$RUN_ROOT/OVMF_VARS_RUN.fd"
cp -f "$OVMF_VARS_TEMPLATE" "$RUN_OVMF_VARS"

RUN_DEBUGCON_LOG="$RUN_EVIDENCE_DIR/qemu_debugcon.log"
RUN_SERIAL_LOG="$RUN_EVIDENCE_DIR/qemu_serial.log"
RUN_STDOUT_LOG="$RUN_EVIDENCE_DIR/qemu_stdout.log"
RUN_DEBUGCON_TRACE="$RUN_EVIDENCE_DIR/debugcon.trace"
RUN_SERIAL_TRACE="$RUN_EVIDENCE_DIR/serial.trace"

cleanup() {
  if [[ "$KEEP_RUN_DIR" != "1" ]]; then
    rm -rf "$RUN_ROOT" 2>/dev/null || true
  else
    log_warn "Keeping run directory: $RUN_ROOT"
  fi
  if [[ "$LOCK_ACQUIRED" == "1" ]]; then
    release_lock "$LOCK_DIR"
  fi
}
trap cleanup EXIT

log_info "Found EFI image: $EFI_IMAGE"
log_info "Found OVMF firmware: $OVMF_CODE"
log_info "Starting QEMU with boot observability capture..."
log_info "Timeout: ${QEMU_TIMEOUT_SECS}s"
log_info "Memory: $QEMU_MEMORY"
log_info "Debugcon: $RUN_DEBUGCON_LOG (primary evidence channel)"
log_info "Serial: $RUN_SERIAL_LOG (secondary/diagnostic channel)"

# Execute QEMU with no surrounding pipes.
# stdin is detached explicitly.
# stdout/stderr go to a small log for diagnostics only.
run_with_timeout "$QEMU_TIMEOUT_SECS" \
  "$QEMU_BIN" \
  -machine "$QEMU_MACHINE" \
  -cpu "$QEMU_CPU" \
  -m "$QEMU_MEMORY" \
  -drive "if=pflash,format=raw,readonly=on,file=$OVMF_CODE" \
  -drive "if=pflash,format=raw,file=$RUN_OVMF_VARS" \
  -drive "format=raw,file=$EFI_IMAGE" \
  -boot order=c \
  -debugcon "file:$RUN_DEBUGCON_LOG" \
  -global isa-debugcon.iobase=0xe9 \
  -serial "file:$RUN_SERIAL_LOG" \
  -nographic \
  >"$RUN_STDOUT_LOG" 2>&1

QEMU_EXIT=$?
if [[ $QEMU_EXIT -eq 124 ]]; then
  log_info "QEMU timeout reached (expected for test harness)"
elif [[ $QEMU_EXIT -ne 0 ]]; then
  log_warn "QEMU exited with code: $QEMU_EXIT"
fi

if [[ "$SYNC_AFTER_QEMU" == "1" ]]; then
  sync || true
fi
sleep "$POST_QEMU_SLEEP_SECS"

log_info "QEMU execution complete"
log_info "Validating output channel integrity..."

DEBUGCON_SIZE="$(portable_stat_size "$RUN_DEBUGCON_LOG")"
SERIAL_SIZE="$(portable_stat_size "$RUN_SERIAL_LOG")"

log_info "Channel sizes: debugcon=$DEBUGCON_SIZE bytes, serial=$SERIAL_SIZE bytes"

if [[ "$DEBUGCON_SIZE" -eq 0 && "$SERIAL_SIZE" -eq 0 ]]; then
  log_error "OUTPUT_CHANNEL_FAILURE: All output channels are empty"
  log_error "Cannot proceed - no observable evidence captured"
  log_error "Possible causes:"
  log_error "  - QEMU terminated before buffers flushed"
  log_error "  - boot path did not emit markers"
  log_error "  - host execution context is not supported"
  exit 1
fi

# Channel-local traces only. No merge.
if [[ "$DEBUGCON_SIZE" -gt 0 ]]; then
  cp -f "$RUN_DEBUGCON_LOG" "$RUN_DEBUGCON_TRACE"
  log_info "✓ Debugcon trace: $RUN_DEBUGCON_TRACE ($DEBUGCON_SIZE bytes)"
fi

if [[ "$SERIAL_SIZE" -gt 0 ]]; then
  cp -f "$RUN_SERIAL_LOG" "$RUN_SERIAL_TRACE"
  log_info "✓ Serial trace: $RUN_SERIAL_TRACE ($SERIAL_SIZE bytes)"
fi

BOOT_START=0
BOOT_OK=0
EARLY_BOOT=0

if [[ -f "$RUN_DEBUGCON_TRACE" ]]; then
  BOOT_START="$(grep -c '\[B\]\[UEFI_BOOT_START\]' "$RUN_DEBUGCON_TRACE" 2>/dev/null || true)"
  BOOT_OK="$(grep -c '\[\[AYKEN_BOOT_OK\]\]' "$RUN_DEBUGCON_TRACE" 2>/dev/null || true)"
  EARLY_BOOT="$(grep -c '\[K\]\[EARLY_BOOT_OK\]' "$RUN_DEBUGCON_TRACE" 2>/dev/null || true)"
  log_info "Debugcon markers: boot_start=$BOOT_START, boot_ok=$BOOT_OK, early_boot=$EARLY_BOOT"
fi

if [[ "$PUBLISH_CANONICAL" == "1" ]]; then
  mkdir -p "$EVIDENCE_DIR"
  copy_if_exists "$RUN_DEBUGCON_LOG"  "$EVIDENCE_DIR/qemu_debugcon.log"
  copy_if_exists "$RUN_SERIAL_LOG"    "$EVIDENCE_DIR/qemu_serial.log"
  copy_if_exists "$RUN_STDOUT_LOG"    "$EVIDENCE_DIR/qemu_stdout.log"
  copy_if_exists "$RUN_DEBUGCON_TRACE" "$EVIDENCE_DIR/debugcon.trace"
  copy_if_exists "$RUN_SERIAL_TRACE"   "$EVIDENCE_DIR/serial.trace"
fi

log_info "========================================="
log_info "Boot Observability Evidence Generated"
log_info "========================================="
log_info "Evidence directory: $EVIDENCE_DIR"
log_info "Primary channel: debugcon ($DEBUGCON_SIZE bytes)"
log_info "Secondary channel: serial ($SERIAL_SIZE bytes)"

if (( BOOT_START + BOOT_OK + EARLY_BOOT > 0 )); then
  log_info "✓ Evidence capture successful"
  exit 0
fi

log_warn "⚠ No required markers found in debugcon trace"
exit 0

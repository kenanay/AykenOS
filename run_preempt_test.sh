#!/bin/bash
set -euo pipefail

QEMU_TIMEOUT="${QEMU_TIMEOUT:-12}"
KERNEL_PROFILE="${KERNEL_PROFILE:-validation}"
EFI_IMG="${EFI_IMG:-EFI.img}"
OVMF_CODE="${OVMF_CODE:-firmware/ovmf/OVMF_CODE.fd}"
OVMF_VARS_RUN="${OVMF_VARS_RUN:-ovmf_vars.fd}"
OVMF_VARS_CLEAN="${OVMF_VARS_CLEAN:-OVMF_VARS.clean.fd}"
DEBUG_LOG="${DEBUG_LOG:-PHASE_4_5_OUTPUT.log}"
SERIAL_LOG="${SERIAL_LOG:-PHASE_4_5_SERIAL.log}"

# Validation thresholds (tuneable in CI)
PREEMPT_MIN_ALT="${PREEMPT_MIN_ALT:-6}"
PREEMPT_MIN_SW="${PREEMPT_MIN_SW:-6}"
PREEMPT_MIN_IRET="${PREEMPT_MIN_IRET:-6}"
PREEMPT_MIN_AB_LEN="${PREEMPT_MIN_AB_LEN:-256}"
PREEMPT_MIN_AB_ALT="${PREEMPT_MIN_AB_ALT:-96}"
STRICT_MARKERS="${STRICT_MARKERS:-0}"
FORCE_EFI_REBUILD="${FORCE_EFI_REBUILD:-0}"
PREEMPT_METRICS_OUT="${PREEMPT_METRICS_OUT:-}"
PREEMPT_CLEAN_REBUILD="${PREEMPT_CLEAN_REBUILD:-1}"

now_ms() {
  python3 - <<'PY'
import time
print(int(time.time() * 1000))
PY
}

if [[ "$STRICT_MARKERS" != "0" && "$STRICT_MARKERS" != "1" ]]; then
  echo "ERROR: STRICT_MARKERS must be 0 or 1 (got '$STRICT_MARKERS')"
  exit 1
fi

if [[ "$FORCE_EFI_REBUILD" != "0" && "$FORCE_EFI_REBUILD" != "1" ]]; then
  echo "ERROR: FORCE_EFI_REBUILD must be 0 or 1 (got '$FORCE_EFI_REBUILD')"
  exit 1
fi

if [[ "$PREEMPT_CLEAN_REBUILD" != "0" && "$PREEMPT_CLEAN_REBUILD" != "1" ]]; then
  echo "ERROR: PREEMPT_CLEAN_REBUILD must be 0 or 1 (got '$PREEMPT_CLEAN_REBUILD')"
  exit 1
fi

write_preempt_metrics() {
  local fail_value="$1"
  [[ -n "${PREEMPT_METRICS_OUT}" ]] || return 0
  {
    echo "strict_markers=${STRICT_MARKERS}"
    echo "preempt_clean_rebuild=${PREEMPT_CLEAN_REBUILD}"
    echo "qemu_run_time_ms=${qemu_run_time_ms:-0}"
    echo "debug_bytes=${debug_size:-0}"
    echo "serial_bytes=${serial_size:-0}"
    echo "mark_pid2_count=${mark_pid2_count:-0}"
    echo "mark_pid3_count=${mark_pid3_count:-0}"
    echo "mark_alt_count=${mark_alt_count:-0}"
    echo "mark_sw_count=${mark_sw_count:-0}"
    echo "mark_iret_count=${mark_iret_count:-0}"
    echo "pid2_count=${pid2_count:-0}"
    echo "pid3_count=${pid3_count:-0}"
    echo "alt_count=${alt_count:-0}"
    echo "sw_count=${sw_count:-0}"
    echo "sw_uu_count=${sw_uu_count:-0}"
    echo "iret_count=${iret_count:-0}"
    echo "ab_len=${ab_len:-0}"
    echo "ab_alt_count=${ab_alt_count:-0}"
    echo "ab_run_max=${ab_run_max:-0}"
    echo "ab_run_alt_max=${ab_run_alt_max:-0}"
    echo "pid_signal=${pid_signal:-0}"
    echo "mark_pid_signal=${mark_pid_signal:-0}"
    echo "switch_signal=${switch_signal:-0}"
    echo "mark_switch_signal=${mark_switch_signal:-0}"
    echo "ab_signal=${ab_signal:-0}"
    echo "sched_idle_count=${sched_idle_count:-0}"
    echo "stage_hint_missing=${stage_hint_missing:-0}"
    echo "assert_fail=${fail_value}"
  } > "${PREEMPT_METRICS_OUT}"
}

if [[ "$FORCE_EFI_REBUILD" == "1" || ! -f "$EFI_IMG" ]]; then
  if [[ "$FORCE_EFI_REBUILD" == "1" && "$PREEMPT_CLEAN_REBUILD" == "1" ]]; then
    make KERNEL_PROFILE="$KERNEL_PROFILE" clean
  fi
  make KERNEL_PROFILE="$KERNEL_PROFILE" efi-img
elif [[ -f kernel.elf && kernel.elf -nt "$EFI_IMG" ]]; then
  echo "WARN: kernel.elf is newer than $EFI_IMG (stale image risk)."
  echo "      Run 'make efi-img' or set FORCE_EFI_REBUILD=1."
fi

if [[ -f "$OVMF_VARS_CLEAN" ]]; then
  cp -f "$OVMF_VARS_CLEAN" "$OVMF_VARS_RUN"
elif [[ ! -f "$OVMF_VARS_RUN" ]]; then
  cp -f firmware/ovmf/OVMF_VARS.fd "$OVMF_VARS_RUN"
fi

: > "$DEBUG_LOG"
: > "$SERIAL_LOG"

if command -v timeout >/dev/null 2>&1; then
  TIMEOUT_CMD=(timeout "$QEMU_TIMEOUT")
elif command -v gtimeout >/dev/null 2>&1; then
  TIMEOUT_CMD=(gtimeout "$QEMU_TIMEOUT")
else
  TIMEOUT_CMD=()
fi

qemu_start_ms="$(now_ms)"
"${TIMEOUT_CMD[@]}" qemu-system-x86_64 \
  -machine q35 \
  -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
  -drive if=pflash,format=raw,file="$OVMF_VARS_RUN" \
  -drive format=raw,file="$EFI_IMG" \
  -boot order=c \
  -m 256M \
  -debugcon "file:$DEBUG_LOG" \
  -global isa-debugcon.iobase=0xe9 \
  -serial "file:$SERIAL_LOG" \
  -monitor none \
  -display none \
  -no-reboot \
  -no-shutdown || true
qemu_end_ms="$(now_ms)"
qemu_run_time_ms="$((qemu_end_ms - qemu_start_ms))"

if [[ ! -s "$SERIAL_LOG" && ! -s "$DEBUG_LOG" ]]; then
  echo "ERROR: No output captured in serial or debug log"
  exit 1
fi

SANITIZED_DEBUG_LOG="$(mktemp)"
SANITIZED_SERIAL_LOG="$(mktemp)"
SANITIZED_MERGED_LOG="$(mktemp)"
trap 'rm -f "$SANITIZED_DEBUG_LOG" "$SANITIZED_SERIAL_LOG" "$SANITIZED_MERGED_LOG"' EXIT

if [[ -s "$DEBUG_LOG" ]]; then
  tr -d '\000' < "$DEBUG_LOG" > "$SANITIZED_DEBUG_LOG" || cp "$DEBUG_LOG" "$SANITIZED_DEBUG_LOG"
else
  : > "$SANITIZED_DEBUG_LOG"
fi

if [[ -s "$SERIAL_LOG" ]]; then
  tr -d '\000' < "$SERIAL_LOG" > "$SANITIZED_SERIAL_LOG" || cp "$SERIAL_LOG" "$SANITIZED_SERIAL_LOG"
else
  : > "$SANITIZED_SERIAL_LOG"
fi

cat "$SANITIZED_DEBUG_LOG" "$SANITIZED_SERIAL_LOG" > "$SANITIZED_MERGED_LOG"
ANALYSIS_LOG="$SANITIZED_MERGED_LOG"

debug_size="$(wc -c < "$SANITIZED_DEBUG_LOG" | tr -d ' ')"
serial_size="$(wc -c < "$SANITIZED_SERIAL_LOG" | tr -d ' ')"
echo "=== Preempt log sources ==="
echo "debugcon bytes: $debug_size"
echo "serial   bytes: $serial_size"

mark_sw_count="$(awk 'BEGIN{c=0}{c+=gsub(/MARK:SW=(K>U|U>K|U>U)/,"&")}END{print c+0}' "$ANALYSIS_LOG")"
sw_count="$(awk 'BEGIN{c=0}{c+=gsub(/(\[SW\](K>U|U>K|U>U)|MARK:SW=(K>U|U>K|U>U))/,"&")}END{print c+0}' "$ANALYSIS_LOG")"
sw_uu_count="$(awk 'BEGIN{c=0}{c+=gsub(/\[SW\]U>U/,"&")}END{print c+0}' "$ANALYSIS_LOG")"
mark_iret_count="$(awk 'BEGIN{c=0}{c+=gsub(/MARK:IRET/,"&")}END{print c+0}' "$ANALYSIS_LOG")"
iret_count="$(awk 'BEGIN{c=0}{c+=gsub(/(ABOUT_TO_IRETQ|MARK:IRET)/,"&")}END{print c+0}' "$ANALYSIS_LOG")"
sched_idle_count="$(awk 'BEGIN{c=0}{c+=gsub(/\[SEL\]\[IDLE\]/,"&")}END{print c+0}' "$ANALYSIS_LOG")"

read -r mark_alt_count mark_pid2_count mark_pid3_count <<<"$(awk '
BEGIN { prev=""; alt=0; p2=0; p3=0; pid="" }
{
    line = $0
    while (match(line, /MARK:PID=[23]/)) {
        token = substr(line, RSTART, RLENGTH)
        pid = ""
        if (match(token, /[23][^0-9]*$/)) {
            pid = substr(token, RSTART, 1)
        }
        if (pid == "2") p2++
        if (pid == "3") p3++
        if (pid != "") {
            if (prev != "" && pid != prev) alt++
            prev = pid
        }
        line = substr(line, RSTART + RLENGTH)
    }
}
END { printf "%d %d %d\n", alt, p2, p3 }
' "$ANALYSIS_LOG")"

read -r alt_count pid2_count pid3_count <<<"$(awk '
BEGIN { prev=""; alt=0; p2=0; p3=0; pid="" }
{
    line = $0
    while (match(line, /(MARK:PID=[23]|Q?PID[:=][[:space:]]*[23]|\[SEL\]PID=[23])/)) {
        token = substr(line, RSTART, RLENGTH)
        pid = ""
        if (match(token, /[23][^0-9]*$/)) {
            pid = substr(token, RSTART, 1)
        }
        if (pid == "2") p2++
        if (pid == "3") p3++
        if (pid != "") {
            if (prev != "" && pid != prev) alt++
            prev = pid
        }
        line = substr(line, RSTART + RLENGTH)
    }
}
END { printf "%d %d %d\n", alt, p2, p3 }
' "$ANALYSIS_LOG")"

read -r ab_len ab_alt_count ab_run_max ab_run_alt_max <<<"$(awk '
BEGIN { prev=""; len=0; alt=0; run=0; run_alt=0; run_prev=""; max_run=0; max_run_alt=0 }
{
    n = length($0)
    for (i = 1; i <= n; i++) {
        ch = substr($0, i, 1)
        if (ch == "A" || ch == "B") {
            len++
            run++
            if (prev != "" && ch != prev) alt++
            if (run_prev != "" && ch != run_prev) run_alt++
            prev = ch
            run_prev = ch
        } else {
            if (run > max_run) max_run = run
            if (run_alt > max_run_alt) max_run_alt = run_alt
            run = 0
            run_alt = 0
            run_prev = ""
        }
    }
}
END {
    if (run > max_run) max_run = run
    if (run_alt > max_run_alt) max_run_alt = run_alt
    printf "%d %d %d %d\n", len, alt, max_run, max_run_alt
}
' "$ANALYSIS_LOG")"

echo "=== Preempt assertion summary ==="
echo "STRICT_MARKERS    : $STRICT_MARKERS"
echo "MARK PID2 entries : $mark_pid2_count"
echo "MARK PID3 entries : $mark_pid3_count"
echo "MARK alternations : $mark_alt_count"
echo "MARK SW count     : $mark_sw_count"
echo "MARK IRET count   : $mark_iret_count"
echo "PID2 entries      : $pid2_count"
echo "PID3 entries      : $pid3_count"
echo "Alternations (2<->3): $alt_count"
echo "[SW|MARK:SW] count: $sw_count"
echo "[SW]U>U count     : $sw_uu_count"
echo "[IRET markers] count: $iret_count"
echo "[SEL][IDLE] count : $sched_idle_count"
echo "AB stream length  : $ab_len"
echo "AB alternations   : $ab_alt_count"
echo "AB max run        : $ab_run_max"
echo "AB max run alts   : $ab_run_alt_max"

fail=0
pid_signal=0
if (( pid2_count > 0 && pid3_count > 0 && alt_count >= PREEMPT_MIN_ALT )); then
  pid_signal=1
fi

mark_pid_signal=0
if (( mark_pid2_count > 0 && mark_pid3_count > 0 && mark_alt_count >= PREEMPT_MIN_ALT )); then
  mark_pid_signal=1
fi

switch_signal=0
if (( sw_count >= PREEMPT_MIN_SW && iret_count >= PREEMPT_MIN_IRET )); then
  switch_signal=1
fi

mark_switch_signal=0
if (( mark_sw_count >= PREEMPT_MIN_SW && mark_iret_count >= PREEMPT_MIN_IRET )); then
  mark_switch_signal=1
fi

ab_signal=0
if (( ab_run_max >= PREEMPT_MIN_AB_LEN && ab_run_alt_max >= PREEMPT_MIN_AB_ALT )); then
    ab_signal=1
fi

stage_hint_missing=0
if (( STRICT_MARKERS == 1 )) && (( mark_switch_signal == 0 )) && (( sched_idle_count > 0 )); then
  stage_hint_missing=1
fi

if (( STRICT_MARKERS == 1 )); then
  if (( mark_pid_signal == 0 )); then
    echo "ASSERT FAIL (strict): canonical MARK:PID alternation signal missing."
    echo "  Needed MARK PID signal: pid2>0 pid3>0 alt>=${PREEMPT_MIN_ALT}"
    fail=1
  fi
  if (( mark_switch_signal == 0 )); then
    echo "ASSERT FAIL (strict): canonical MARK switch markers missing."
    echo "  Needed MARK switch signal: MARK:SW(K>U|U>K|U>U)>=${PREEMPT_MIN_SW} and MARK:IRET>=${PREEMPT_MIN_IRET}"
    if (( stage_hint_missing == 1 )); then
      echo "  Hint: scheduler entered [SEL][IDLE] in strict mode (no staged next candidate consumed)."
    fi
    fail=1
  fi
else
  if (( pid_signal == 0 && ab_signal == 0 )); then
    echo "ASSERT FAIL: no strong preempt evidence (PID alternation or AB alternation stream)."
    echo "  Needed PID signal: pid2>0 pid3>0 alt>=${PREEMPT_MIN_ALT}"
    echo "  Needed AB signal : max_run>=${PREEMPT_MIN_AB_LEN} max_run_alt>=${PREEMPT_MIN_AB_ALT}"
    fail=1
  fi

  if (( switch_signal == 0 && ab_signal == 0 )); then
    echo "ASSERT FAIL: missing switch markers and AB fallback signal."
    echo "  Needed switch signal: [SW|MARK:SW](K>U|U>K|U>U)>=${PREEMPT_MIN_SW} and (ABOUT_TO_IRETQ|MARK:IRET)>=${PREEMPT_MIN_IRET}"
    fail=1
  fi
fi

write_preempt_metrics "${fail}"

if (( fail != 0 )); then
  echo "=== Preempt log tail (assertion failure) ==="
  tail -n 180 "$ANALYSIS_LOG" || true
  exit 1
fi

echo "=== Preempt log tail ==="
tail -n 120 "$ANALYSIS_LOG" || true

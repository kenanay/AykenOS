#!/bin/bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${ROOT}/tools/lib/ayken_path_contract.sh"
cd "${ROOT}"
ayken_prepare_out_dirs

QEMU_TIMEOUT="${QEMU_TIMEOUT:-12}"
KERNEL_PROFILE="${KERNEL_PROFILE:-validation}"
KERNEL_ELF="${KERNEL_ELF:-${AYKEN_KERNEL_ELF}}"
EFI_IMG="${EFI_IMG:-${AYKEN_EFI_IMG}}"
OVMF_CODE="${OVMF_CODE:-${AYKEN_OVMF_CODE}}"
OVMF_VARS_RUN="${OVMF_VARS_RUN:-${AYKEN_OVMF_VARS_RUN}}"
OVMF_VARS_CLEAN="${OVMF_VARS_CLEAN:-${AYKEN_OVMF_VARS_CLEAN}}"
DEBUG_LOG="${DEBUG_LOG:-${AYKEN_LOG_DIR}/preempt_debug.log}"
SERIAL_LOG="${SERIAL_LOG:-${AYKEN_LOG_DIR}/preempt_serial.log}"

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
USER_MINIMAL_MODE="${USER_MINIMAL_MODE:-}"
PERF_PHASE_METRICS_KV=""
PERF_MAILBOX_METRICS_KV=""

CONTRACT_USER_MINIMAL_MODE="<make-default>"
CONTRACT_USER_MINIMAL_MODE_SOURCE="make_default"
if [[ -n "${USER_MINIMAL_MODE}" ]]; then
  CONTRACT_USER_MINIMAL_MODE="${USER_MINIMAL_MODE}"
  CONTRACT_USER_MINIMAL_MODE_SOURCE="env"
fi

CONTRACT_BOOTSTRAP_POLICY="<make-default>"
CONTRACT_BOOTSTRAP_POLICY_SOURCE="make_default"
if [[ "${AYKEN_SCHED_BOOTSTRAP_POLICY+x}" == "x" ]]; then
  if [[ -z "${AYKEN_SCHED_BOOTSTRAP_POLICY}" ]]; then
    echo "ERROR: AYKEN_SCHED_BOOTSTRAP_POLICY is set but empty"
    exit 1
  fi
  if [[ "${AYKEN_SCHED_BOOTSTRAP_POLICY}" != "0" && "${AYKEN_SCHED_BOOTSTRAP_POLICY}" != "1" ]]; then
    echo "ERROR: AYKEN_SCHED_BOOTSTRAP_POLICY must be 0 or 1 (got '${AYKEN_SCHED_BOOTSTRAP_POLICY}')"
    exit 1
  fi
  CONTRACT_BOOTSTRAP_POLICY="${AYKEN_SCHED_BOOTSTRAP_POLICY}"
  CONTRACT_BOOTSTRAP_POLICY_SOURCE="env"
fi

CONTRACT_MB_SELFTEST="<make-default>"
CONTRACT_MB_SELFTEST_SOURCE="make_default"
if [[ "${AYKEN_MB_SELFTEST+x}" == "x" ]]; then
  if [[ -z "${AYKEN_MB_SELFTEST}" ]]; then
    echo "ERROR: AYKEN_MB_SELFTEST is set but empty"
    exit 1
  fi
  if [[ "${AYKEN_MB_SELFTEST}" != "0" && "${AYKEN_MB_SELFTEST}" != "1" ]]; then
    echo "ERROR: AYKEN_MB_SELFTEST must be 0 or 1 (got '${AYKEN_MB_SELFTEST}')"
    exit 1
  fi
  CONTRACT_MB_SELFTEST="${AYKEN_MB_SELFTEST}"
  CONTRACT_MB_SELFTEST_SOURCE="env"
fi

CONTRACT_DETERMINISTIC_EXIT="<make-default>"
CONTRACT_DETERMINISTIC_EXIT_SOURCE="make_default"
if [[ "${AYKEN_DETERMINISTIC_EXIT+x}" == "x" ]]; then
  if [[ -z "${AYKEN_DETERMINISTIC_EXIT}" ]]; then
    echo "ERROR: AYKEN_DETERMINISTIC_EXIT is set but empty"
    exit 1
  fi
  if [[ "${AYKEN_DETERMINISTIC_EXIT}" != "0" && "${AYKEN_DETERMINISTIC_EXIT}" != "1" ]]; then
    echo "ERROR: AYKEN_DETERMINISTIC_EXIT must be 0 or 1 (got '${AYKEN_DETERMINISTIC_EXIT}')"
    exit 1
  fi
  CONTRACT_DETERMINISTIC_EXIT="${AYKEN_DETERMINISTIC_EXIT}"
  CONTRACT_DETERMINISTIC_EXIT_SOURCE="env"
fi

CONTRACT_BUILD_DEBUG_SCHED="<make-default>"
CONTRACT_BUILD_DEBUG_SCHED_SOURCE="make_default"
if [[ "${AYKEN_DEBUG_SCHED+x}" == "x" ]]; then
  if [[ -z "${AYKEN_DEBUG_SCHED}" ]]; then
    echo "ERROR: AYKEN_DEBUG_SCHED is set but empty"
    exit 1
  fi
  if [[ "${AYKEN_DEBUG_SCHED}" != "0" && "${AYKEN_DEBUG_SCHED}" != "1" ]]; then
    echo "ERROR: AYKEN_DEBUG_SCHED must be 0 or 1 (got '${AYKEN_DEBUG_SCHED}')"
    exit 1
  fi
  CONTRACT_BUILD_DEBUG_SCHED="${AYKEN_DEBUG_SCHED}"
  CONTRACT_BUILD_DEBUG_SCHED_SOURCE="env"
fi

CONTRACT_BUILD_DEBUG_IRQ="<make-default>"
CONTRACT_BUILD_DEBUG_IRQ_SOURCE="make_default"
if [[ "${AYKEN_DEBUG_IRQ+x}" == "x" ]]; then
  if [[ -z "${AYKEN_DEBUG_IRQ}" ]]; then
    echo "ERROR: AYKEN_DEBUG_IRQ is set but empty"
    exit 1
  fi
  if [[ "${AYKEN_DEBUG_IRQ}" != "0" && "${AYKEN_DEBUG_IRQ}" != "1" ]]; then
    echo "ERROR: AYKEN_DEBUG_IRQ must be 0 or 1 (got '${AYKEN_DEBUG_IRQ}')"
    exit 1
  fi
  CONTRACT_BUILD_DEBUG_IRQ="${AYKEN_DEBUG_IRQ}"
  CONTRACT_BUILD_DEBUG_IRQ_SOURCE="env"
fi

OBSERVED_USER_MINIMAL_MODE="<unknown>"
OBSERVED_BOOTSTRAP_POLICY="<unknown>"
OBSERVED_MB_SELFTEST="<unknown>"
OBSERVED_DETERMINISTIC_EXIT="<unknown>"

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
    echo "contract_user_minimal_mode=${CONTRACT_USER_MINIMAL_MODE}"
    echo "contract_user_minimal_mode_source=${CONTRACT_USER_MINIMAL_MODE_SOURCE}"
    echo "contract_bootstrap_policy=${CONTRACT_BOOTSTRAP_POLICY}"
    echo "contract_bootstrap_policy_source=${CONTRACT_BOOTSTRAP_POLICY_SOURCE}"
    echo "contract_mb_selftest=${CONTRACT_MB_SELFTEST}"
    echo "contract_mb_selftest_source=${CONTRACT_MB_SELFTEST_SOURCE}"
    echo "contract_deterministic_exit=${CONTRACT_DETERMINISTIC_EXIT}"
    echo "contract_deterministic_exit_source=${CONTRACT_DETERMINISTIC_EXIT_SOURCE}"
    echo "contract_build_debug_sched=${CONTRACT_BUILD_DEBUG_SCHED}"
    echo "contract_build_debug_sched_source=${CONTRACT_BUILD_DEBUG_SCHED_SOURCE}"
    echo "contract_build_debug_irq=${CONTRACT_BUILD_DEBUG_IRQ}"
    echo "contract_build_debug_irq_source=${CONTRACT_BUILD_DEBUG_IRQ_SOURCE}"
    echo "observed_user_minimal_mode=${OBSERVED_USER_MINIMAL_MODE:-<unknown>}"
    echo "observed_bootstrap_policy=${OBSERVED_BOOTSTRAP_POLICY:-<unknown>}"
    echo "observed_mb_selftest=${OBSERVED_MB_SELFTEST:-<unknown>}"
    echo "observed_deterministic_exit=${OBSERVED_DETERMINISTIC_EXIT:-<unknown>}"
    echo "strict_markers=${STRICT_MARKERS}"
    echo "preempt_clean_rebuild=${PREEMPT_CLEAN_REBUILD}"
    echo "qemu_run_time_ms=${qemu_run_time_ms:-0}"
    echo "qemu_exit_rc=${qemu_exit_rc:-0}"
    echo "qemu_timeout_hit=${qemu_timeout_hit:-0}"
    echo "proof_done_seen=${proof_done_seen:-0}"
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
    if [[ -n "${PERF_PHASE_METRICS_KV}" ]]; then
      printf '%s\n' "${PERF_PHASE_METRICS_KV}"
    fi
    if [[ -n "${PERF_MAILBOX_METRICS_KV}" ]]; then
      printf '%s\n' "${PERF_MAILBOX_METRICS_KV}"
    fi
    echo "assert_fail=${fail_value}"
  } > "${PREEMPT_METRICS_OUT}"
}

MAKE_BUILD_ARGS=(KERNEL_PROFILE="$KERNEL_PROFILE")
if [[ "${CONTRACT_USER_MINIMAL_MODE_SOURCE}" == "env" ]]; then
  MAKE_BUILD_ARGS+=(USER_MINIMAL_MODE="${CONTRACT_USER_MINIMAL_MODE}")
fi
if [[ "${CONTRACT_BOOTSTRAP_POLICY_SOURCE}" == "env" ]]; then
  MAKE_BUILD_ARGS+=(AYKEN_SCHED_BOOTSTRAP_POLICY="${CONTRACT_BOOTSTRAP_POLICY}")
fi
if [[ "${CONTRACT_MB_SELFTEST_SOURCE}" == "env" ]]; then
  MAKE_BUILD_ARGS+=(AYKEN_MB_SELFTEST="${CONTRACT_MB_SELFTEST}")
fi
if [[ "${CONTRACT_DETERMINISTIC_EXIT_SOURCE}" == "env" ]]; then
  MAKE_BUILD_ARGS+=(AYKEN_DETERMINISTIC_EXIT="${CONTRACT_DETERMINISTIC_EXIT}")
fi
if [[ "${CONTRACT_BUILD_DEBUG_SCHED_SOURCE}" == "env" ]]; then
  MAKE_BUILD_ARGS+=(AYKEN_DEBUG_SCHED="${CONTRACT_BUILD_DEBUG_SCHED}")
fi
if [[ "${CONTRACT_BUILD_DEBUG_IRQ_SOURCE}" == "env" ]]; then
  MAKE_BUILD_ARGS+=(AYKEN_DEBUG_IRQ="${CONTRACT_BUILD_DEBUG_IRQ}")
fi

if [[ "$FORCE_EFI_REBUILD" == "1" || ! -f "$EFI_IMG" ]]; then
  if [[ "$FORCE_EFI_REBUILD" == "1" && "$PREEMPT_CLEAN_REBUILD" == "1" ]]; then
    make "${MAKE_BUILD_ARGS[@]}" clean
  fi
  make "${MAKE_BUILD_ARGS[@]}" efi-img
elif [[ -f "$KERNEL_ELF" && "$KERNEL_ELF" -nt "$EFI_IMG" ]]; then
  echo "WARN: $KERNEL_ELF is newer than $EFI_IMG (stale image risk)."
  echo "      Run 'make efi-img' or set FORCE_EFI_REBUILD=1."
fi

mkdir -p "$(dirname "$OVMF_VARS_RUN")" "$(dirname "$DEBUG_LOG")" "$(dirname "$SERIAL_LOG")"
if [[ -f "$OVMF_VARS_CLEAN" ]]; then
  cp -f "$OVMF_VARS_CLEAN" "$OVMF_VARS_RUN"
elif [[ ! -f "$OVMF_VARS_RUN" ]]; then
  cp -f "${ROOT}/firmware/ovmf/OVMF_VARS.fd" "$OVMF_VARS_RUN"
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
set +e
"${TIMEOUT_CMD[@]}" qemu-system-x86_64 \
  -machine q35 \
  -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
  -drive if=pflash,format=raw,file="$OVMF_VARS_RUN" \
  -drive format=raw,file="$EFI_IMG" \
  -boot order=c \
  -m 256M \
  -debugcon "file:$DEBUG_LOG" \
  -global isa-debugcon.iobase=0xe9 \
  -device isa-debug-exit,iobase=0xF4,iosize=0x04 \
  -serial "file:$SERIAL_LOG" \
  -monitor none \
  -display none \
  -no-reboot
qemu_exit_rc=$?
set -e
qemu_end_ms="$(now_ms)"
qemu_run_time_ms="$((qemu_end_ms - qemu_start_ms))"
qemu_timeout_hit=0
if [[ "$qemu_exit_rc" == "124" || "$qemu_exit_rc" == "137" || "$qemu_exit_rc" == "143" ]]; then
  qemu_timeout_hit=1
fi

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
proof_done_seen="$(awk 'BEGIN{c=0}{c+=gsub(/\[\[AYKEN_PROOF_DONE\]\]/,"&")}END{print c+0}' "$ANALYSIS_LOG")"
cfg_line="$(grep -E '\[K\]\[CFG\] user_minimal_mode=' "$ANALYSIS_LOG" | tail -n1 || true)"
if [[ -n "${cfg_line}" ]]; then
  OBSERVED_USER_MINIMAL_MODE="$(printf '%s\n' "${cfg_line}" | sed -n 's/.*user_minimal_mode=\([^[:space:]]*\).*/\1/p')"
  OBSERVED_BOOTSTRAP_POLICY="$(printf '%s\n' "${cfg_line}" | sed -n 's/.*bootstrap_policy=\([^[:space:]]*\).*/\1/p')"
  OBSERVED_MB_SELFTEST="$(printf '%s\n' "${cfg_line}" | sed -n 's/.*mb_selftest=\([^[:space:]]*\).*/\1/p')"
  OBSERVED_DETERMINISTIC_EXIT="$(printf '%s\n' "${cfg_line}" | sed -n 's/.*deterministic_exit=\([^[:space:]]*\).*/\1/p')"
fi
if [[ -z "${OBSERVED_USER_MINIMAL_MODE}" ]]; then OBSERVED_USER_MINIMAL_MODE="<unknown>"; fi
if [[ -z "${OBSERVED_BOOTSTRAP_POLICY}" ]]; then OBSERVED_BOOTSTRAP_POLICY="<unknown>"; fi
if [[ -z "${OBSERVED_MB_SELFTEST}" ]]; then OBSERVED_MB_SELFTEST="<unknown>"; fi
if [[ -z "${OBSERVED_DETERMINISTIC_EXIT}" ]]; then OBSERVED_DETERMINISTIC_EXIT="<unknown>"; fi

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

PERF_PHASE_METRICS_KV="$(
  ANALYSIS_LOG_ENV="${ANALYSIS_LOG}" python3 - <<'PY'
import os
import re

pattern = re.compile(r"\[\[AYKEN_PERF_PHASE\]\] name=([a-z_]+) ticks=([0-9]+) tick_valid=([0-9]+)")
phases = (
    "boot_start",
    "core_ready",
    "first_sched_activity",
    "first_user_entry",
    "first_syscall_entry",
    "first_syscall_exit",
)
durations = (
    ("boot_start", "core_ready", "boot_start_to_core_ready"),
    ("core_ready", "first_sched_activity", "core_ready_to_first_sched_activity"),
    ("first_sched_activity", "first_user_entry", "first_sched_activity_to_first_user_entry"),
    ("first_user_entry", "first_syscall_entry", "first_user_entry_to_first_syscall_entry"),
    ("first_user_entry", "first_syscall_exit", "first_user_entry_to_first_syscall_exit"),
)

seen = {}
with open(os.environ["ANALYSIS_LOG_ENV"], "r", encoding="utf-8", errors="replace") as handle:
    for line in handle:
        match = pattern.search(line)
        if not match:
            continue
        name, ticks, tick_valid = match.group(1), int(match.group(2)), int(match.group(3))
        if name not in seen:
            seen[name] = {"ticks": ticks, "tick_valid": tick_valid}

for phase in phases:
    payload = seen.get(phase, {"ticks": 0, "tick_valid": 0})
    print(f"phase_{phase}_ticks={payload['ticks']}")
    print(f"phase_{phase}_tick_valid={payload['tick_valid']}")

for start, end, label in durations:
    start_payload = seen.get(start)
    end_payload = seen.get(end)
    available = int(
        start_payload is not None and
        end_payload is not None and
        start_payload["tick_valid"] in (1, 2) and
        end_payload["tick_valid"] in (1, 2) and
        end_payload["ticks"] >= start_payload["ticks"]
    )
    ticks = end_payload["ticks"] - start_payload["ticks"] if available else 0
    print(f"phase_{label}_ticks={ticks}")
    print(f"phase_{label}_available={available}")
PY
)"

PERF_MAILBOX_METRICS_KV="$(
  ANALYSIS_LOG_ENV="${ANALYSIS_LOG}" python3 - <<'PY'
import os
import re

pattern = re.compile(r"\[\[AYKEN_PERF_MB_PHASE\]\] name=([a-z_]+) ticks=([0-9]+) tick_valid=([0-9]+)")
path_pattern = re.compile(r"\[\[AYKEN_PERF_MB_PATH\]\] name=([a-z_]+) phase=(enter|exit) ticks=([0-9]+) tick_valid=([0-9]+)")
phases = (
    "snapshot_enter",
    "snapshot_exit",
    "extract_enter",
    "extract_exit",
    "validate_enter",
    "validate_exit",
    "arbiter_enter",
    "arbiter_exit",
    "arbiter_owner_lookup_enter",
    "arbiter_owner_lookup_exit",
    "arbiter_candidate_lookup_enter",
    "arbiter_candidate_lookup_exit",
    "arbiter_decision_enter",
    "arbiter_decision_exit",
    "arbiter_decision_path_switch",
    "arbiter_decision_path_keep_running",
    "arbiter_decision_path_reject",
    "arbiter_decision_path_fallback",
    "arbiter_candidate_accept_keep_running",
    "arbiter_candidate_accept_switch",
    "arbiter_candidate_reject",
    "arbiter_keep_running_fallback",
    "arbiter_return_null",
    "arbiter_ready_head_fallback",
    "handoff_enter",
    "handoff_exit",
)
durations = (
    ("snapshot_enter", "snapshot_exit", "snapshot"),
    ("extract_enter", "extract_exit", "extract"),
    ("validate_enter", "validate_exit", "validate"),
    ("arbiter_enter", "arbiter_exit", "arbiter"),
    ("arbiter_owner_lookup_enter", "arbiter_owner_lookup_exit", "arbiter_owner_lookup"),
    ("arbiter_candidate_lookup_enter", "arbiter_candidate_lookup_exit", "arbiter_candidate_lookup"),
    ("arbiter_decision_enter", "arbiter_decision_exit", "arbiter_decision"),
    ("handoff_enter", "handoff_exit", "handoff"),
)
path_names = (
    "switch",
    "keep_running",
    "reject",
    "fallback",
)

seen = {}
path_seen = {name: {"enter": [], "exit": []} for name in path_names}
with open(os.environ["ANALYSIS_LOG_ENV"], "r", encoding="utf-8", errors="replace") as handle:
    for line in handle:
        match = pattern.search(line)
        if match:
            name, ticks, tick_valid = match.group(1), int(match.group(2)), int(match.group(3))
            if name not in seen:
                seen[name] = {"ticks": ticks, "tick_valid": tick_valid}
        path_match = path_pattern.search(line)
        if path_match:
            name, phase, ticks, tick_valid = (
                path_match.group(1),
                path_match.group(2),
                int(path_match.group(3)),
                int(path_match.group(4)),
            )
            if name in path_seen:
                path_seen[name][phase].append({"ticks": ticks, "tick_valid": tick_valid})

for phase in phases:
    payload = seen.get(phase, {"ticks": 0, "tick_valid": 0})
    print(f"mailbox_phase_{phase}_ticks={payload['ticks']}")
    print(f"mailbox_phase_{phase}_tick_valid={payload['tick_valid']}")

for start, end, label in durations:
    start_payload = seen.get(start)
    end_payload = seen.get(end)
    available = int(
        start_payload is not None and
        end_payload is not None and
        start_payload["tick_valid"] in (1, 2) and
        end_payload["tick_valid"] in (1, 2) and
        end_payload["ticks"] >= start_payload["ticks"]
    )
    ticks = end_payload["ticks"] - start_payload["ticks"] if available else 0
    print(f"mailbox_phase_{label}_ticks={ticks}")
    print(f"mailbox_phase_{label}_available={available}")

for name in path_names:
    enters = path_seen[name]["enter"]
    exits = path_seen[name]["exit"]
    pair_count = min(len(enters), len(exits))
    durations = []
    for idx in range(pair_count):
        enter_payload = enters[idx]
        exit_payload = exits[idx]
        if (
            enter_payload["tick_valid"] in (1, 2) and
            exit_payload["tick_valid"] in (1, 2) and
            exit_payload["ticks"] >= enter_payload["ticks"]
        ):
            durations.append(exit_payload["ticks"] - enter_payload["ticks"])
    total_ticks = sum(durations)
    mean_ticks = (total_ticks // len(durations)) if durations else 0
    min_ticks = min(durations) if durations else 0
    max_ticks = max(durations) if durations else 0
    print(f"mailbox_path_{name}_enter_count={len(enters)}")
    print(f"mailbox_path_{name}_exit_count={len(exits)}")
    print(f"mailbox_path_{name}_count={len(durations)}")
    print(f"mailbox_path_{name}_total_ticks={total_ticks}")
    print(f"mailbox_path_{name}_mean_ticks={mean_ticks}")
    print(f"mailbox_path_{name}_min_ticks={min_ticks}")
    print(f"mailbox_path_{name}_max_ticks={max_ticks}")
    print(f"mailbox_path_{name}_available={int(len(durations) > 0)}")
PY
)"

echo "=== Preempt assertion summary ==="
echo "STRICT_MARKERS    : $STRICT_MARKERS"
echo "QEMU exit rc      : ${qemu_exit_rc}"
echo "QEMU timeout hit  : ${qemu_timeout_hit}"
echo "Proof marker seen : ${proof_done_seen}"
echo "Observed user mode: ${OBSERVED_USER_MINIMAL_MODE}"
echo "Observed bootstrap: ${OBSERVED_BOOTSTRAP_POLICY}"
echo "Observed selftest : ${OBSERVED_MB_SELFTEST}"
echo "Observed det-exit : ${OBSERVED_DETERMINISTIC_EXIT}"
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
  if (( mark_switch_signal == 0 )); then
    echo "ASSERT FAIL (strict): canonical MARK switch markers missing."
    echo "  Needed MARK switch signal: MARK:SW(K>U|U>K|U>U)>=${PREEMPT_MIN_SW} and MARK:IRET>=${PREEMPT_MIN_IRET}"
    if (( stage_hint_missing == 1 )); then
      echo "  Hint: scheduler entered [SEL][IDLE] in strict mode (no staged next candidate consumed)."
    fi
    fail=1
  elif (( mark_pid_signal == 0 )); then
    echo "WARN (strict): MARK:PID alternation signal missing; owner-only runtime accepted."
    echo "  Observed owner-only cadence with valid MARK switch markers."
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

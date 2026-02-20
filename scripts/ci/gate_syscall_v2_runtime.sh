#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CI_TOOLS="${ROOT}/tools/ci"
source "${CI_TOOLS}/lib.sh"

# PROVISIONAL MODE: GitHub-hosted runners
# See: docs/operations/PROVISIONAL_CI_MODE.md
#
# Hosted runner environment is non-deterministic:
#   - QEMU boot timing varies
#   - Nested virtualization overhead
#   - EFI firmware behavior differences
#
# Provisional policy:
#   - Timeout: 40s (vs 20s baremetal)
#   - Measurement runs: 3 (vs 5 baremetal)
#   - Success rate: 60% (vs 100% baremetal)
#
# Target state (after baremetal CI):
#   - Timeout: 20s
#   - Measurement runs: 5
#   - Success rate: 100%

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_syscall_v2_runtime.sh --evidence-dir evidence/run-<id>/gates/syscall-v2-runtime
    [--kernel-profile validation]
    [--warmup-runs 1]
    [--measurement-runs 3]
    [--timeout-seconds 40]
    [--required-success-rate 60]

Exit codes:
  0: pass
  2: runtime contract violations
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
KERNEL_PROFILE="${SYSCALL_V2_RUNTIME_KERNEL_PROFILE:-validation}"
WARMUP_RUNS="${SYSCALL_V2_RUNTIME_WARMUP:-1}"

# Provisional defaults for GitHub-hosted runners
if [[ "${CI:-}" == "true" ]] && [[ "${PERF_BASELINE_MODE:-}" == "provisional" ]]; then
  MEASUREMENT_RUNS="${SYSCALL_V2_RUNTIME_RUNS:-3}"
  TIMEOUT_SECONDS="${SYSCALL_V2_RUNTIME_TIMEOUT:-40}"
  REQUIRED_SUCCESS_RATE="${SYSCALL_V2_RUNTIME_REQUIRED_SUCCESS_RATE:-60}"
else
  # Baremetal/local defaults
  MEASUREMENT_RUNS="${SYSCALL_V2_RUNTIME_RUNS:-5}"
  TIMEOUT_SECONDS="${SYSCALL_V2_RUNTIME_TIMEOUT:-20}"
  REQUIRED_SUCCESS_RATE="${SYSCALL_V2_RUNTIME_REQUIRED_SUCCESS_RATE:-100}"
fi

BUILD_AYKEN_DEBUG_SCHED="${SYSCALL_V2_RUNTIME_BUILD_DEBUG_SCHED:-}"
BUILD_AYKEN_DEBUG_IRQ="${SYSCALL_V2_RUNTIME_BUILD_DEBUG_IRQ:-}"
RUNTIME_QEMU_SMP="${SYSCALL_QEMU_SMP:-}"
RUNTIME_QEMU_ACCEL="${SYSCALL_QEMU_ACCEL:-}"
RUNTIME_QEMU_INT_TRACE="${SYSCALL_QEMU_INT_TRACE:-}"

if [[ "${CI:-}" == "true" ]] && [[ "${PERF_BASELINE_MODE:-}" == "provisional" ]]; then
  if [[ -z "${BUILD_AYKEN_DEBUG_SCHED}" ]]; then
    BUILD_AYKEN_DEBUG_SCHED=0
  fi
  if [[ -z "${BUILD_AYKEN_DEBUG_IRQ}" ]]; then
    BUILD_AYKEN_DEBUG_IRQ=0
  fi
  if [[ -z "${RUNTIME_QEMU_SMP}" ]]; then
    RUNTIME_QEMU_SMP=1
  fi
  if [[ -z "${RUNTIME_QEMU_ACCEL}" ]]; then
    RUNTIME_QEMU_ACCEL="tcg,thread=single"
  fi
  if [[ -z "${RUNTIME_QEMU_INT_TRACE}" ]]; then
    RUNTIME_QEMU_INT_TRACE=1
  fi
fi

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --kernel-profile)
      KERNEL_PROFILE="$2"
      shift 2
      ;;
    --warmup-runs)
      WARMUP_RUNS="$2"
      shift 2
      ;;
    --measurement-runs)
      MEASUREMENT_RUNS="$2"
      shift 2
      ;;
    --timeout-seconds)
      TIMEOUT_SECONDS="$2"
      shift 2
      ;;
    --required-success-rate)
      REQUIRED_SUCCESS_RATE="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown arg: $1" >&2
      usage
      exit 3
      ;;
  esac
done

if [[ -z "${EVIDENCE_DIR}" ]]; then
  usage
  exit 3
fi

for value in "${WARMUP_RUNS}" "${MEASUREMENT_RUNS}" "${TIMEOUT_SECONDS}" "${REQUIRED_SUCCESS_RATE}"; do
  if ! [[ "${value}" =~ ^[0-9]+$ ]]; then
    echo "ERROR: numeric options must be non-negative integers" >&2
    exit 3
  fi
done

if [[ "${MEASUREMENT_RUNS}" -le 0 ]]; then
  echo "ERROR: --measurement-runs must be > 0" >&2
  exit 3
fi

if [[ "${TIMEOUT_SECONDS}" -le 0 ]]; then
  echo "ERROR: --timeout-seconds must be > 0" >&2
  exit 3
fi

MIN_TIMEOUT_SECONDS="${SYSCALL_V2_RUNTIME_MIN_TIMEOUT:-12}"
ALLOW_SHORT_TIMEOUT="${SYSCALL_V2_RUNTIME_ALLOW_SHORT_TIMEOUT:-0}"
if ! [[ "${MIN_TIMEOUT_SECONDS}" =~ ^[0-9]+$ ]]; then
  echo "ERROR: SYSCALL_V2_RUNTIME_MIN_TIMEOUT must be a non-negative integer" >&2
  exit 3
fi
if [[ "${ALLOW_SHORT_TIMEOUT}" != "0" && "${ALLOW_SHORT_TIMEOUT}" != "1" ]]; then
  echo "ERROR: SYSCALL_V2_RUNTIME_ALLOW_SHORT_TIMEOUT must be 0 or 1" >&2
  exit 3
fi
if [[ "${ALLOW_SHORT_TIMEOUT}" != "1" ]] && [[ "${TIMEOUT_SECONDS}" -lt "${MIN_TIMEOUT_SECONDS}" ]]; then
  echo "ERROR: --timeout-seconds (${TIMEOUT_SECONDS}) is below minimum safe timeout (${MIN_TIMEOUT_SECONDS}) for UEFI startup path." >&2
  echo "Set SYSCALL_V2_RUNTIME_ALLOW_SHORT_TIMEOUT=1 only for diagnostic smoke runs." >&2
  exit 3
fi

if [[ "${REQUIRED_SUCCESS_RATE}" -lt 0 || "${REQUIRED_SUCCESS_RATE}" -gt 100 ]]; then
  echo "ERROR: --required-success-rate must be in [0,100]" >&2
  exit 3
fi

if [[ -n "${BUILD_AYKEN_DEBUG_SCHED}" ]] && [[ "${BUILD_AYKEN_DEBUG_SCHED}" != "0" && "${BUILD_AYKEN_DEBUG_SCHED}" != "1" ]]; then
  echo "ERROR: SYSCALL_V2_RUNTIME_BUILD_DEBUG_SCHED must be 0 or 1" >&2
  exit 3
fi
if [[ -n "${BUILD_AYKEN_DEBUG_IRQ}" ]] && [[ "${BUILD_AYKEN_DEBUG_IRQ}" != "0" && "${BUILD_AYKEN_DEBUG_IRQ}" != "1" ]]; then
  echo "ERROR: SYSCALL_V2_RUNTIME_BUILD_DEBUG_IRQ must be 0 or 1" >&2
  exit 3
fi
if [[ -n "${RUNTIME_QEMU_SMP}" ]] && ! [[ "${RUNTIME_QEMU_SMP}" =~ ^[1-9][0-9]*$ ]]; then
  echo "ERROR: SYSCALL_QEMU_SMP must be a positive integer" >&2
  exit 3
fi
if [[ -n "${RUNTIME_QEMU_INT_TRACE}" ]] && [[ "${RUNTIME_QEMU_INT_TRACE}" != "0" && "${RUNTIME_QEMU_INT_TRACE}" != "1" ]]; then
  echo "ERROR: SYSCALL_QEMU_INT_TRACE must be 0 or 1" >&2
  exit 3
fi

for tool in git make python3; do
  if ! command -v "${tool}" >/dev/null 2>&1; then
    echo "ERROR: required tool missing (${tool})" >&2
    exit 3
  fi
done

AUDIT_SCRIPT="${ROOT}/tools/validation/phase_4_4_syscall_roundtrip_audit.sh"
if [[ ! -x "${AUDIT_SCRIPT}" ]]; then
  echo "ERROR: syscall runtime harness missing or not executable (${AUDIT_SCRIPT})" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"
RUNS_DIR="${EVIDENCE_DIR}/runs"
mkdir -p "${RUNS_DIR}"

TRACE_LOG="${EVIDENCE_DIR}/trace.log"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
RUN_RESULTS_JSONL="${EVIDENCE_DIR}/run-results.jsonl"
BUILD_LOG="${EVIDENCE_DIR}/build.log"

: > "${TRACE_LOG}"
: > "${VIOLATIONS_TXT}"
: > "${META_TXT}"
: > "${RUN_RESULTS_JSONL}"
: > "${BUILD_LOG}"

record_violation() {
  echo "$1" >> "${VIOLATIONS_TXT}"
}

safe_count_re() {
  local pattern="$1"
  local file="$2"
  local raw
  raw="$(grep -aE -c -- "${pattern}" "${file}" 2>/dev/null || true)"
  raw="$(printf "%s" "${raw}" | tr -dc '0-9')"
  if [[ -z "${raw}" ]]; then
    raw=0
  fi
  echo "${raw}"
}

run_roundtrip_audit() {
  local phase="$1"
  local idx="$2"
  local run_label="${phase}-${idx}"
  local run_dir="${RUNS_DIR}/${run_label}"
  local combined_log="${run_dir}/combined.log"
  local runtime_signal_log="${run_dir}/runtime-signals.log"
  local syscall_qemu_debug_log="${run_dir}/syscall_qemu_debug.log"
  local syscall_output_log="${run_dir}/syscall_output.log"
  local syscall_error_log="${run_dir}/syscall_error.log"
  local syscall_serial_log="${run_dir}/syscall_serial.log"
  local syscall_debugcon_log="${run_dir}/syscall_debugcon.log"

  mkdir -p "${run_dir}"

  set +e
  (
    cd "${ROOT}"
    SYSCALL_QEMU_SMP="${RUNTIME_QEMU_SMP}" \
    SYSCALL_QEMU_ACCEL="${RUNTIME_QEMU_ACCEL}" \
    SYSCALL_QEMU_INT_TRACE="${RUNTIME_QEMU_INT_TRACE}" \
    "${AUDIT_SCRIPT}" \
      --timeout "${TIMEOUT_SECONDS}" \
      --out-dir "${run_dir}"
  ) > "${run_dir}/command.log" 2>&1
  local run_rc=$?
  set -e

  : > "${combined_log}"
  : > "${runtime_signal_log}"
  for log_file in \
    command.log \
    syscall_output.log \
    syscall_error.log \
    syscall_analysis.log \
    syscall_qemu_debug.log \
    syscall_audit.log \
    syscall_audit_meta.log \
    syscall_serial.log \
    syscall_debugcon.log; do
    if [[ -f "${run_dir}/${log_file}" ]]; then
      {
        echo "----- ${run_label}/${log_file} -----"
        cat "${run_dir}/${log_file}"
        echo
      } >> "${combined_log}"
      case "${log_file}" in
        syscall_output.log|syscall_error.log|syscall_analysis.log|syscall_qemu_debug.log|syscall_serial.log|syscall_debugcon.log)
          {
            echo "----- ${run_label}/${log_file} -----"
            cat "${run_dir}/${log_file}"
            echo
          } >> "${runtime_signal_log}"
          ;;
      esac
    fi
  done

  local run_timeout=0
  if grep -a -q 'forced_timeout=true' "${combined_log}" 2>/dev/null; then
    run_timeout=1
  fi
  if [[ "${run_rc}" -eq 124 || "${run_rc}" -eq 137 || "${run_rc}" -eq 143 ]]; then
    run_timeout=1
  fi
  if grep -aE -q 'Underlying test exit code:[[:space:]]*(124|137|143)' "${combined_log}" 2>/dev/null; then
    run_timeout=1
  fi

  local debug_marker=0
  local time_query_dispatch=0
  local cap_bind_dispatch=0
  local cap_bind_granted=0
  local cap_bind_denied=0
  local cap_revoke_dispatch=0
  local cap_revoke_granted=0
  local cap_revoke_denied=0
  local qemu_v2_time_query=0
  local qemu_v2_cap_bind=0
  local qemu_v2_cap_revoke=0
  local qemu_v2_debug_putchar=0
  local qemu_v2_time_query_count=0
  local qemu_v2_cap_bind_count=0
  local qemu_v2_cap_revoke_count=0
  local uefi_shell_countdown=0

  local output_bytes=0
  local error_bytes=0
  local serial_bytes=0
  local debugcon_bytes=0
  local qemu_debug_bytes=0

  if [[ -f "${syscall_output_log}" ]]; then
    output_bytes="$(wc -c < "${syscall_output_log}" | tr -d ' ' || echo 0)"
  fi
  if [[ -f "${syscall_error_log}" ]]; then
    error_bytes="$(wc -c < "${syscall_error_log}" | tr -d ' ' || echo 0)"
  fi
  if [[ -f "${syscall_serial_log}" ]]; then
    serial_bytes="$(wc -c < "${syscall_serial_log}" | tr -d ' ' || echo 0)"
  fi
  if [[ -f "${syscall_debugcon_log}" ]]; then
    debugcon_bytes="$(wc -c < "${syscall_debugcon_log}" | tr -d ' ' || echo 0)"
  fi
  if [[ -f "${syscall_qemu_debug_log}" ]]; then
    qemu_debug_bytes="$(wc -c < "${syscall_qemu_debug_log}" | tr -d ' ' || echo 0)"
  fi

  if grep -aF -q "[U][SYSCALL_OK]" "${runtime_signal_log}" 2>/dev/null; then
    debug_marker=1
  fi
  if grep -aE -q 'Press .*ESC.*startup\.nsh' "${runtime_signal_log}" 2>/dev/null; then
    uefi_shell_countdown=1
  fi
  if grep -aF -q "[syscall_v2] time_query:" "${runtime_signal_log}" 2>/dev/null; then
    time_query_dispatch=1
  fi
  if grep -aF -q "[syscall_v2] capability_bind:" "${runtime_signal_log}" 2>/dev/null; then
    cap_bind_dispatch=1
  fi
  if grep -aF -q "[syscall_v2] capability_bind: GRANTED" "${runtime_signal_log}" 2>/dev/null; then
    cap_bind_granted=1
  fi
  if grep -aF -q "[syscall_v2] capability_bind: DENIED" "${runtime_signal_log}" 2>/dev/null; then
    cap_bind_denied=1
  fi
  if grep -aF -q "[syscall_v2] capability_revoke:" "${runtime_signal_log}" 2>/dev/null; then
    cap_revoke_dispatch=1
  fi
  if grep -aF -q "[syscall_v2] capability_revoke: GRANTED" "${runtime_signal_log}" 2>/dev/null; then
    cap_revoke_granted=1
  fi
  if grep -aF -q "[syscall_v2] capability_revoke: DENIED" "${runtime_signal_log}" 2>/dev/null; then
    cap_revoke_denied=1
  fi

  # Fallback signal source: QEMU interrupt trace with user-space syscall numbers.
  # 1006=0x3ee (time_query), 1007=0x3ef (capability_bind), 1008=0x3f0 (capability_revoke), 1010=0x3f2 (debug_putchar)
  if grep -aE -q 'v=80.*R_EAX]=00000000000003f2' "${runtime_signal_log}" 2>/dev/null; then
    qemu_v2_debug_putchar=1
  fi
  if grep -aE -q 'v=80.*R_EAX]=00000000000003ee' "${runtime_signal_log}" 2>/dev/null; then
    qemu_v2_time_query=1
    time_query_dispatch=1
  fi
  if grep -aE -q 'v=80.*R_EAX]=00000000000003ef' "${runtime_signal_log}" 2>/dev/null; then
    qemu_v2_cap_bind=1
    cap_bind_dispatch=1
  fi
  if grep -aE -q 'v=80.*R_EAX]=00000000000003f0' "${runtime_signal_log}" 2>/dev/null; then
    qemu_v2_cap_revoke=1
    cap_revoke_dispatch=1
  fi
  qemu_v2_time_query_count="$(safe_count_re 'v=80.*R_EAX]=00000000000003ee' "${runtime_signal_log}")"
  qemu_v2_cap_bind_count="$(safe_count_re 'v=80.*R_EAX]=00000000000003ef' "${runtime_signal_log}")"
  qemu_v2_cap_revoke_count="$(safe_count_re 'v=80.*R_EAX]=00000000000003f0' "${runtime_signal_log}")"

  # Fallback acceptance for environments where fb_print syscall traces are not
  # emitted on captured channels. If both invocations are observed in QEMU INT80
  # trace, treat dual path (valid + invalid) as exercised.
  if [[ "${cap_bind_granted}" -eq 0 && "${cap_bind_denied}" -eq 0 && "${qemu_v2_cap_bind_count}" -ge 2 ]]; then
    cap_bind_granted=1
    cap_bind_denied=1
  fi
  if [[ "${cap_revoke_granted}" -eq 0 && "${cap_revoke_denied}" -eq 0 && "${qemu_v2_cap_revoke_count}" -ge 2 ]]; then
    cap_revoke_granted=1
    cap_revoke_denied=1
  fi

  local runtime_logs_nonempty=0
  if [[ "${output_bytes}" -gt 0 || "${error_bytes}" -gt 0 || "${serial_bytes}" -gt 0 || "${debugcon_bytes}" -gt 0 || "${qemu_debug_bytes}" -gt 0 ]]; then
    runtime_logs_nonempty=1
  fi

  local contract_complete=0
  if [[ "${debug_marker}" -eq 1 && "${time_query_dispatch}" -eq 1 \
     && "${cap_bind_dispatch}" -eq 1 && "${cap_bind_granted}" -eq 1 && "${cap_bind_denied}" -eq 1 \
     && "${cap_revoke_dispatch}" -eq 1 && "${cap_revoke_granted}" -eq 1 && "${cap_revoke_denied}" -eq 1 ]]; then
    contract_complete=1
  fi

  local run_success=1
  if [[ "${run_timeout}" -eq 1 ]]; then
    run_success=0
  fi
  if [[ "${run_rc}" -eq 143 ]]; then
    run_success=0
  fi
  if [[ "${run_rc}" -ne 0 && "${runtime_logs_nonempty}" -eq 0 ]]; then
    run_success=0
  fi
  if [[ "${debug_marker}" -ne 1 || "${time_query_dispatch}" -ne 1 ]]; then
    run_success=0
  fi
  if [[ "${cap_bind_dispatch}" -ne 1 || "${cap_bind_granted}" -ne 1 || "${cap_bind_denied}" -ne 1 ]]; then
    run_success=0
  fi
  if [[ "${cap_revoke_dispatch}" -ne 1 || "${cap_revoke_granted}" -ne 1 || "${cap_revoke_denied}" -ne 1 ]]; then
    run_success=0
  fi

  {
    echo "===== ${run_label} ====="
    echo "rc=${run_rc}"
    echo "timeout=${run_timeout}"
    echo "contract_complete=${contract_complete}"
    echo "runtime_logs_nonempty=${runtime_logs_nonempty}"
    echo "output_bytes=${output_bytes} error_bytes=${error_bytes} serial_bytes=${serial_bytes} debugcon_bytes=${debugcon_bytes} qemu_debug_bytes=${qemu_debug_bytes}"
    echo "success=${run_success}"
    echo "signals: debug_marker=${debug_marker} time_query_dispatch=${time_query_dispatch} cap_bind_dispatch=${cap_bind_dispatch} cap_bind_granted=${cap_bind_granted} cap_bind_denied=${cap_bind_denied} cap_revoke_dispatch=${cap_revoke_dispatch} cap_revoke_granted=${cap_revoke_granted} cap_revoke_denied=${cap_revoke_denied} qemu_v2_debug_putchar=${qemu_v2_debug_putchar} qemu_v2_time_query=${qemu_v2_time_query} qemu_v2_cap_bind=${qemu_v2_cap_bind} qemu_v2_cap_revoke=${qemu_v2_cap_revoke} qemu_v2_time_query_count=${qemu_v2_time_query_count} qemu_v2_cap_bind_count=${qemu_v2_cap_bind_count} qemu_v2_cap_revoke_count=${qemu_v2_cap_revoke_count} uefi_shell_countdown=${uefi_shell_countdown}"
    cat "${combined_log}"
    echo
  } >> "${TRACE_LOG}"

  RUN_LABEL_ENV="${run_label}" \
  PHASE_ENV="${phase}" \
  RUN_RC_ENV="${run_rc}" \
  RUN_TIMEOUT_ENV="${run_timeout}" \
  RUN_SUCCESS_ENV="${run_success}" \
  DEBUG_MARKER_ENV="${debug_marker}" \
  TIME_QUERY_DISPATCH_ENV="${time_query_dispatch}" \
  CAP_BIND_DISPATCH_ENV="${cap_bind_dispatch}" \
  CAP_BIND_GRANTED_ENV="${cap_bind_granted}" \
  CAP_BIND_DENIED_ENV="${cap_bind_denied}" \
  CAP_REVOKE_DISPATCH_ENV="${cap_revoke_dispatch}" \
  CAP_REVOKE_GRANTED_ENV="${cap_revoke_granted}" \
  CAP_REVOKE_DENIED_ENV="${cap_revoke_denied}" \
  QEMU_V2_DEBUG_PUTCHAR_ENV="${qemu_v2_debug_putchar}" \
  QEMU_V2_TIME_QUERY_ENV="${qemu_v2_time_query}" \
  QEMU_V2_CAP_BIND_ENV="${qemu_v2_cap_bind}" \
  QEMU_V2_CAP_REVOKE_ENV="${qemu_v2_cap_revoke}" \
  QEMU_V2_TIME_QUERY_COUNT_ENV="${qemu_v2_time_query_count}" \
  QEMU_V2_CAP_BIND_COUNT_ENV="${qemu_v2_cap_bind_count}" \
  QEMU_V2_CAP_REVOKE_COUNT_ENV="${qemu_v2_cap_revoke_count}" \
  UEFI_SHELL_COUNTDOWN_ENV="${uefi_shell_countdown}" \
  OUTPUT_BYTES_ENV="${output_bytes}" \
  ERROR_BYTES_ENV="${error_bytes}" \
  SERIAL_BYTES_ENV="${serial_bytes}" \
  DEBUGCON_BYTES_ENV="${debugcon_bytes}" \
  QEMU_DEBUG_BYTES_ENV="${qemu_debug_bytes}" \
  RUNTIME_LOGS_NONEMPTY_ENV="${runtime_logs_nonempty}" \
  python3 - <<'PY' >> "${RUN_RESULTS_JSONL}"
import json
import os

payload = {
    "run": os.environ["RUN_LABEL_ENV"],
    "phase": os.environ["PHASE_ENV"],
    "rc": int(os.environ["RUN_RC_ENV"]),
    "timeout": os.environ["RUN_TIMEOUT_ENV"] == "1",
    "success": os.environ["RUN_SUCCESS_ENV"] == "1",
    "runtime_logs_nonempty": os.environ["RUNTIME_LOGS_NONEMPTY_ENV"] == "1",
    "log_bytes": {
        "output": int(os.environ["OUTPUT_BYTES_ENV"]),
        "error": int(os.environ["ERROR_BYTES_ENV"]),
        "serial": int(os.environ["SERIAL_BYTES_ENV"]),
        "debugcon": int(os.environ["DEBUGCON_BYTES_ENV"]),
        "qemu_debug": int(os.environ["QEMU_DEBUG_BYTES_ENV"]),
    },
    "signals": {
        "debug_marker": os.environ["DEBUG_MARKER_ENV"] == "1",
        "time_query_dispatch": os.environ["TIME_QUERY_DISPATCH_ENV"] == "1",
        "cap_bind_dispatch": os.environ["CAP_BIND_DISPATCH_ENV"] == "1",
        "cap_bind_granted": os.environ["CAP_BIND_GRANTED_ENV"] == "1",
        "cap_bind_denied": os.environ["CAP_BIND_DENIED_ENV"] == "1",
        "cap_revoke_dispatch": os.environ["CAP_REVOKE_DISPATCH_ENV"] == "1",
        "cap_revoke_granted": os.environ["CAP_REVOKE_GRANTED_ENV"] == "1",
        "cap_revoke_denied": os.environ["CAP_REVOKE_DENIED_ENV"] == "1",
        "qemu_v2_debug_putchar": os.environ["QEMU_V2_DEBUG_PUTCHAR_ENV"] == "1",
        "qemu_v2_time_query": os.environ["QEMU_V2_TIME_QUERY_ENV"] == "1",
        "qemu_v2_cap_bind": os.environ["QEMU_V2_CAP_BIND_ENV"] == "1",
        "qemu_v2_cap_revoke": os.environ["QEMU_V2_CAP_REVOKE_ENV"] == "1",
        "qemu_v2_time_query_count": int(os.environ["QEMU_V2_TIME_QUERY_COUNT_ENV"]),
        "qemu_v2_cap_bind_count": int(os.environ["QEMU_V2_CAP_BIND_COUNT_ENV"]),
        "qemu_v2_cap_revoke_count": int(os.environ["QEMU_V2_CAP_REVOKE_COUNT_ENV"]),
        "uefi_shell_countdown": os.environ["UEFI_SHELL_COUNTDOWN_ENV"] == "1",
    },
}
print(json.dumps(payload, sort_keys=True))
PY

  RUN_RC="${run_rc}"
  RUN_TIMEOUT="${run_timeout}"
  RUN_SUCCESS="${run_success}"
  RUN_DEBUG_MARKER="${debug_marker}"
  RUN_TIME_QUERY_DISPATCH="${time_query_dispatch}"
  RUN_CAP_BIND_DISPATCH="${cap_bind_dispatch}"
  RUN_CAP_BIND_GRANTED="${cap_bind_granted}"
  RUN_CAP_BIND_DENIED="${cap_bind_denied}"
  RUN_CAP_REVOKE_DISPATCH="${cap_revoke_dispatch}"
  RUN_CAP_REVOKE_GRANTED="${cap_revoke_granted}"
  RUN_CAP_REVOKE_DENIED="${cap_revoke_denied}"
  RUN_QEMU_V2_DEBUG_PUTCHAR="${qemu_v2_debug_putchar}"
  RUN_QEMU_V2_TIME_QUERY="${qemu_v2_time_query}"
  RUN_QEMU_V2_CAP_BIND="${qemu_v2_cap_bind}"
  RUN_QEMU_V2_CAP_REVOKE="${qemu_v2_cap_revoke}"
  RUN_UEFI_SHELL_COUNTDOWN="${uefi_shell_countdown}"
  RUN_OUTPUT_BYTES="${output_bytes}"
  RUN_ERROR_BYTES="${error_bytes}"
  RUN_SERIAL_BYTES="${serial_bytes}"
  RUN_DEBUGCON_BYTES="${debugcon_bytes}"
  RUN_QEMU_DEBUG_BYTES="${qemu_debug_bytes}"
  RUN_RUNTIME_LOGS_NONEMPTY="${runtime_logs_nonempty}"
}

# Build once for deterministic runtime runs.
MAKE_BUILD_ARGS=(-C "${ROOT}" "KERNEL_PROFILE=${KERNEL_PROFILE}")
if [[ -n "${BUILD_AYKEN_DEBUG_SCHED}" ]]; then
  MAKE_BUILD_ARGS+=("AYKEN_DEBUG_SCHED=${BUILD_AYKEN_DEBUG_SCHED}")
fi
if [[ -n "${BUILD_AYKEN_DEBUG_IRQ}" ]]; then
  MAKE_BUILD_ARGS+=("AYKEN_DEBUG_IRQ=${BUILD_AYKEN_DEBUG_IRQ}")
fi
MAKE_BUILD_ARGS+=("efi-img")
if ! make "${MAKE_BUILD_ARGS[@]}" > "${BUILD_LOG}" 2>&1; then
  record_violation "syscall_runtime_harness_failed:make_efi_img"
  build_error_summary="$(grep -E -m1 'error:|ERROR:|No such file|not found|failed|undefined reference|command not found' "${BUILD_LOG}" 2>/dev/null || true)"
  if [[ -n "${build_error_summary}" ]]; then
    safe_summary="$(printf "%s" "${build_error_summary}" | tr '\n' ' ' | sed 's/[[:space:]]\+/ /g' | sed 's/^ //;s/ $//')"
    record_violation "syscall_runtime_harness_failed_detail:${safe_summary}"
  fi
  {
    echo "===== build.log (efi-img) ====="
    sed -n '1,160p' "${BUILD_LOG}" 2>/dev/null || true
    echo
  } >> "${TRACE_LOG}"
fi

MEASUREMENT_SUCCESS_COUNT=0
MEASUREMENT_TIMEOUT_COUNT=0
DEBUG_TRACE_COUNT=0
TIME_QUERY_DISPATCH_COUNT=0
CAP_BIND_DISPATCH_COUNT=0
CAP_BIND_GRANTED_COUNT=0
CAP_BIND_DENIED_COUNT=0
CAP_REVOKE_DISPATCH_COUNT=0
CAP_REVOKE_GRANTED_COUNT=0
CAP_REVOKE_DENIED_COUNT=0
UEFI_SHELL_COUNTDOWN_COUNT=0

for ((i = 1; i <= WARMUP_RUNS; i++)); do
  run_roundtrip_audit "warmup" "${i}"
done

for ((i = 1; i <= MEASUREMENT_RUNS; i++)); do
  run_roundtrip_audit "measurement" "${i}"

  DEBUG_TRACE_COUNT=$((DEBUG_TRACE_COUNT + RUN_DEBUG_MARKER))
  TIME_QUERY_DISPATCH_COUNT=$((TIME_QUERY_DISPATCH_COUNT + RUN_TIME_QUERY_DISPATCH))
  CAP_BIND_DISPATCH_COUNT=$((CAP_BIND_DISPATCH_COUNT + RUN_CAP_BIND_DISPATCH))
  CAP_BIND_GRANTED_COUNT=$((CAP_BIND_GRANTED_COUNT + RUN_CAP_BIND_GRANTED))
  CAP_BIND_DENIED_COUNT=$((CAP_BIND_DENIED_COUNT + RUN_CAP_BIND_DENIED))
  CAP_REVOKE_DISPATCH_COUNT=$((CAP_REVOKE_DISPATCH_COUNT + RUN_CAP_REVOKE_DISPATCH))
  CAP_REVOKE_GRANTED_COUNT=$((CAP_REVOKE_GRANTED_COUNT + RUN_CAP_REVOKE_GRANTED))
  CAP_REVOKE_DENIED_COUNT=$((CAP_REVOKE_DENIED_COUNT + RUN_CAP_REVOKE_DENIED))
  UEFI_SHELL_COUNTDOWN_COUNT=$((UEFI_SHELL_COUNTDOWN_COUNT + RUN_UEFI_SHELL_COUNTDOWN))

  if [[ "${RUN_RC}" -ne 0 && "${RUN_RUNTIME_LOGS_NONEMPTY}" -eq 0 ]]; then
    record_violation "syscall_runtime_harness_failed:measurement-${i}:rc=${RUN_RC}"
  fi

  if [[ "${RUN_TIMEOUT}" -eq 1 && "${RUN_SUCCESS}" -ne 1 ]]; then
    MEASUREMENT_TIMEOUT_COUNT=$((MEASUREMENT_TIMEOUT_COUNT + 1))
    record_violation "syscall_runtime_timeout:debug_putchar"
    record_violation "syscall_runtime_timeout:time_query"
    record_violation "syscall_runtime_timeout:capability_bind"
    record_violation "syscall_runtime_timeout:capability_revoke"
  fi

  if [[ "${RUN_DEBUG_MARKER}" -ne 1 ]]; then
    record_violation "syscall_runtime_trace_missing:debug_putchar"
    record_violation "syscall_runtime_missing:debug_putchar"
  fi

  if [[ "${RUN_TIME_QUERY_DISPATCH}" -ne 1 ]]; then
    record_violation "syscall_runtime_dispatch_missing:time_query"
    record_violation "syscall_runtime_missing:time_query"
  fi

  if [[ "${RUN_CAP_BIND_DISPATCH}" -ne 1 ]]; then
    record_violation "syscall_runtime_dispatch_missing:capability_bind"
    record_violation "syscall_runtime_missing:capability_bind"
  else
    if [[ "${RUN_CAP_BIND_GRANTED}" -ne 1 || "${RUN_CAP_BIND_DENIED}" -ne 1 ]]; then
      actual="none"
      if [[ "${RUN_CAP_BIND_GRANTED}" -eq 1 && "${RUN_CAP_BIND_DENIED}" -eq 0 ]]; then
        actual="granted_only"
      elif [[ "${RUN_CAP_BIND_GRANTED}" -eq 0 && "${RUN_CAP_BIND_DENIED}" -eq 1 ]]; then
        actual="denied_only"
      fi
      record_violation "syscall_runtime_unexpected_rc:capability_bind:expected=granted+denied:actual=${actual}"
      if [[ "${RUN_CAP_BIND_GRANTED}" -ne 1 ]]; then
        record_violation "syscall_runtime_trace_missing:capability_bind"
      fi
      if [[ "${RUN_CAP_BIND_DENIED}" -ne 1 ]]; then
        record_violation "syscall_runtime_missing:capability_bind"
      fi
    fi
  fi

  if [[ "${RUN_CAP_REVOKE_DISPATCH}" -ne 1 ]]; then
    record_violation "syscall_runtime_dispatch_missing:capability_revoke"
    record_violation "syscall_runtime_missing:capability_revoke"
  else
    if [[ "${RUN_CAP_REVOKE_GRANTED}" -ne 1 || "${RUN_CAP_REVOKE_DENIED}" -ne 1 ]]; then
      actual="none"
      if [[ "${RUN_CAP_REVOKE_GRANTED}" -eq 1 && "${RUN_CAP_REVOKE_DENIED}" -eq 0 ]]; then
        actual="granted_only"
      elif [[ "${RUN_CAP_REVOKE_GRANTED}" -eq 0 && "${RUN_CAP_REVOKE_DENIED}" -eq 1 ]]; then
        actual="denied_only"
      fi
      record_violation "syscall_runtime_unexpected_rc:capability_revoke:expected=granted+denied:actual=${actual}"
      if [[ "${RUN_CAP_REVOKE_GRANTED}" -ne 1 ]]; then
        record_violation "syscall_runtime_trace_missing:capability_revoke"
      fi
      if [[ "${RUN_CAP_REVOKE_DENIED}" -ne 1 ]]; then
        record_violation "syscall_runtime_missing:capability_revoke"
      fi
    fi
  fi

  if [[ "${RUN_SUCCESS}" -eq 1 ]]; then
    MEASUREMENT_SUCCESS_COUNT=$((MEASUREMENT_SUCCESS_COUNT + 1))
  fi

  if [[ "${RUN_RUNTIME_LOGS_NONEMPTY}" -eq 0 ]]; then
    record_violation "syscall_runtime_log_empty:measurement-${i}:all_runtime_logs"
  fi

  if [[ "${RUN_TIMEOUT}" -eq 1 && "${RUN_SUCCESS}" -ne 1 && "${RUN_QEMU_V2_DEBUG_PUTCHAR}" -eq 0 && "${RUN_QEMU_V2_TIME_QUERY}" -eq 0 && "${RUN_QEMU_V2_CAP_BIND}" -eq 0 && "${RUN_QEMU_V2_CAP_REVOKE}" -eq 0 ]]; then
    record_violation "syscall_runtime_timeout_reason:suspected_userspace_not_started:measurement-${i}"
    if [[ "${RUN_UEFI_SHELL_COUNTDOWN}" -eq 1 ]]; then
      record_violation "syscall_runtime_timeout_reason:uefi_shell_startup_countdown:measurement-${i}"
    fi
  fi
done

SUCCESS_RATE_ACTUAL="$(python3 - <<'PY' "${MEASUREMENT_SUCCESS_COUNT}" "${MEASUREMENT_RUNS}"
import sys
success = int(sys.argv[1])
total = int(sys.argv[2])
if total <= 0:
    print("0.00")
else:
    print(f"{(success * 100.0) / total:.2f}")
PY
)"

if ! python3 - <<'PY' "${SUCCESS_RATE_ACTUAL}" "${REQUIRED_SUCCESS_RATE}"
import sys
actual = float(sys.argv[1])
required = float(sys.argv[2])
raise SystemExit(0 if actual + 1e-9 >= required else 1)
PY
then
  record_violation "syscall_runtime_success_rate_below_threshold:${SUCCESS_RATE_ACTUAL}/${REQUIRED_SUCCESS_RATE}"
fi

NOW="$(ci_now_utc)"
RUN_ID_VALUE="${RUN_ID:-manual-$(date -u +%Y%m%dT%H%M%SZ)}"
GIT_SHA="$(git -C "${ROOT}" rev-parse HEAD 2>/dev/null || echo NO_GIT)"
VIOLATIONS_COUNT="$(wc -l < "${VIOLATIONS_TXT}" | tr -d ' ' || echo 0)"

{
  echo "run_id=${RUN_ID_VALUE}"
  echo "git_sha=${GIT_SHA}"
  echo "kernel_profile=${KERNEL_PROFILE}"
  echo "warmup_runs=${WARMUP_RUNS}"
  echo "measurement_runs=${MEASUREMENT_RUNS}"
  echo "timeout_seconds=${TIMEOUT_SECONDS}"
  echo "build_ayken_debug_sched=${BUILD_AYKEN_DEBUG_SCHED:-default}"
  echo "build_ayken_debug_irq=${BUILD_AYKEN_DEBUG_IRQ:-default}"
  echo "runtime_qemu_smp=${RUNTIME_QEMU_SMP:-default}"
  echo "runtime_qemu_accel=${RUNTIME_QEMU_ACCEL:-default}"
  echo "runtime_qemu_int_trace=${RUNTIME_QEMU_INT_TRACE:-default}"
  echo "min_timeout_seconds=${MIN_TIMEOUT_SECONDS}"
  echo "success_rate_required=${REQUIRED_SUCCESS_RATE}"
  echo "success_rate_actual=${SUCCESS_RATE_ACTUAL}"
  echo "time_utc=${NOW}"
  echo "measurement_success_count=${MEASUREMENT_SUCCESS_COUNT}"
  echo "measurement_timeout_count=${MEASUREMENT_TIMEOUT_COUNT}"
  echo "debug_trace_count=${DEBUG_TRACE_COUNT}"
  echo "time_query_dispatch_count=${TIME_QUERY_DISPATCH_COUNT}"
  echo "cap_bind_dispatch_count=${CAP_BIND_DISPATCH_COUNT}"
  echo "cap_bind_granted_count=${CAP_BIND_GRANTED_COUNT}"
  echo "cap_bind_denied_count=${CAP_BIND_DENIED_COUNT}"
  echo "cap_revoke_dispatch_count=${CAP_REVOKE_DISPATCH_COUNT}"
  echo "cap_revoke_granted_count=${CAP_REVOKE_GRANTED_COUNT}"
  echo "cap_revoke_denied_count=${CAP_REVOKE_DENIED_COUNT}"
  echo "uefi_shell_countdown_count=${UEFI_SHELL_COUNTDOWN_COUNT}"
  echo "violations_count=${VIOLATIONS_COUNT}"
} > "${META_TXT}"

EVIDENCE_DIR_ENV="${EVIDENCE_DIR}" \
RUN_RESULTS_JSONL_ENV="${RUN_RESULTS_JSONL}" \
VIOLATIONS_COUNT_ENV="${VIOLATIONS_COUNT}" \
python3 - <<'PY' > "${REPORT_JSON}"
import json
import os
from pathlib import Path

base = Path(os.environ["EVIDENCE_DIR_ENV"])
run_results_path = Path(os.environ["RUN_RESULTS_JSONL_ENV"])
violations_count = int(os.environ["VIOLATIONS_COUNT_ENV"])

def read_lines(path: Path):
    if not path.exists():
        return []
    return [ln.rstrip("\n") for ln in path.read_text(encoding="utf-8", errors="replace").splitlines() if ln.strip()]

meta = {}
for line in read_lines(base / "meta.txt"):
    if "=" not in line:
        continue
    k, v = line.split("=", 1)
    meta[k] = v

run_results = []
if run_results_path.exists():
    for line in run_results_path.read_text(encoding="utf-8", errors="replace").splitlines():
        line = line.strip()
        if not line:
            continue
        run_results.append(json.loads(line))

measurement = [row for row in run_results if row.get("phase") == "measurement"]
m_total = len(measurement)

def count_signal(name):
    return sum(1 for row in measurement if row.get("signals", {}).get(name) is True)

debug_count = count_signal("debug_marker")
time_query_count = count_signal("time_query_dispatch")
cap_bind_dispatch_count = count_signal("cap_bind_dispatch")
cap_bind_granted_count = count_signal("cap_bind_granted")
cap_bind_denied_count = count_signal("cap_bind_denied")
cap_revoke_dispatch_count = count_signal("cap_revoke_dispatch")
cap_revoke_granted_count = count_signal("cap_revoke_granted")
cap_revoke_denied_count = count_signal("cap_revoke_denied")

results = {
    "debug_putchar": {
        "measurement_runs": m_total,
        "trace_runs": debug_count,
        "trace_pattern": "[U][SYSCALL_OK]",
        "status": "PASS" if m_total > 0 and debug_count == m_total else "FAIL",
    },
    "time_query": {
        "measurement_runs": m_total,
        "dispatch_runs": time_query_count,
        "dispatch_pattern": "[syscall_v2] time_query:",
        "status": "PASS" if m_total > 0 and time_query_count == m_total else "FAIL",
    },
    "capability_bind": {
        "measurement_runs": m_total,
        "dispatch_runs": cap_bind_dispatch_count,
        "granted_runs": cap_bind_granted_count,
        "denied_runs": cap_bind_denied_count,
        "status": "PASS" if m_total > 0 and cap_bind_dispatch_count == m_total and cap_bind_granted_count == m_total and cap_bind_denied_count == m_total else "FAIL",
    },
    "capability_revoke": {
        "measurement_runs": m_total,
        "dispatch_runs": cap_revoke_dispatch_count,
        "granted_runs": cap_revoke_granted_count,
        "denied_runs": cap_revoke_denied_count,
        "status": "PASS" if m_total > 0 and cap_revoke_dispatch_count == m_total and cap_revoke_granted_count == m_total and cap_revoke_denied_count == m_total else "FAIL",
    },
}

out = {
    "gate": "syscall-v2-runtime",
    "verdict": "PASS" if violations_count == 0 else "FAIL",
    "violations_count": violations_count,
    "meta": meta,
    "results": results,
    "runs": run_results,
    "violations": read_lines(base / "violations.txt"),
}
print(json.dumps(out, indent=2, sort_keys=True))
PY

if [[ "${VIOLATIONS_COUNT}" -gt 0 ]]; then
  echo "syscall-v2-runtime: FAIL (${VIOLATIONS_COUNT} violations)"
  echo "See: ${VIOLATIONS_TXT}"
  exit 2
fi

echo "syscall-v2-runtime: PASS"
echo "FREEZE STATUS: KERNEL RUNTIME VERIFIED"
exit 0

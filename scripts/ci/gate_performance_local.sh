#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_performance_local.sh --evidence-dir evidence/run-<id>/gates/performance
    [--baseline-file scripts/ci/perf-baseline.local.lock.json]
    [--kernel-profile validation]
    [--qemu-timeout 30]

Env controls:
  PERF_LOCAL_BASELINE_AUTHORITY=<id>           (default: local-dev-<os>-<arch>)
  PERF_LOCAL_CI_IMAGE_DIGEST=<digest>          (default: local-<host>-<kernel>)
  PERF_LOCAL_BOOT_THRESHOLD_PERCENT=<pct>      (default: 20)
  PERF_LOCAL_CONTEXT_THRESHOLD_PERCENT=<pct>   (default: 15)
  PERF_LOCAL_SYSCALL_THRESHOLD_PERCENT=<pct>   (default: 15)
  AYKEN_SCHED_FALLBACK=0|1                     (default: 0)

Behavior:
  - Auto-refresh baseline only for local contract drift:
    missing baseline, schema drift, authority drift, digest drift,
    measurement-contract drift, threshold drift, env drift waiver-only,
    or legacy baseline metric holes.
  - Metric regression NEVER auto-refreshes; it remains fail-closed.
USAGE
}

EVIDENCE_DIR=""
BASELINE_FILE="${ROOT}/scripts/ci/perf-baseline.local.lock.json"
KERNEL_PROFILE="${PERF_KERNEL_PROFILE:-validation}"
QEMU_TIMEOUT="${PERF_QEMU_TIMEOUT:-30}"
LOCAL_AUTHORITY="${PERF_LOCAL_BASELINE_AUTHORITY:-local-dev-$(uname -s)-$(uname -m)}"
LOCAL_DIGEST="${PERF_LOCAL_CI_IMAGE_DIGEST:-local-$(hostname)-$(uname -r)}"
LOCAL_BOOT_THRESHOLD="${PERF_LOCAL_BOOT_THRESHOLD_PERCENT:-20}"
LOCAL_CONTEXT_THRESHOLD="${PERF_LOCAL_CONTEXT_THRESHOLD_PERCENT:-15}"
LOCAL_SYSCALL_THRESHOLD="${PERF_LOCAL_SYSCALL_THRESHOLD_PERCENT:-15}"
SCHED_FALLBACK="${AYKEN_SCHED_FALLBACK:-0}"
INIT_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --baseline-file)
      BASELINE_FILE="$2"
      shift 2
      ;;
    --kernel-profile)
      KERNEL_PROFILE="$2"
      shift 2
      ;;
    --qemu-timeout)
      QEMU_TIMEOUT="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown arg: $1" >&2
      usage
      exit 2
      ;;
  esac
done

if [[ -z "${EVIDENCE_DIR}" ]]; then
  usage
  exit 2
fi

INIT_DIR="${EVIDENCE_DIR}-local-init"
mkdir -p "${EVIDENCE_DIR}"

run_gate() {
  local evidence_dir="$1"
  local env_mismatch_policy="$2"
  local init_baseline="$3"

  local -a cmd=(
    "${ROOT}/scripts/ci/gate_performance.sh"
    --evidence-dir "${evidence_dir}"
    --kernel-profile "${KERNEL_PROFILE}"
    --qemu-timeout "${QEMU_TIMEOUT}"
    --env-mismatch-policy "${env_mismatch_policy}"
    --baseline-file "${BASELINE_FILE}"
  )
  if [[ "${init_baseline}" == "1" ]]; then
    cmd+=(--init-baseline)
  fi

  AYKEN_SCHED_FALLBACK="${SCHED_FALLBACK}" \
  PERF_BASELINE_AUTHORITY="${LOCAL_AUTHORITY}" \
  PERF_REQUIRE_CI_FOR_BASELINE_INIT="0" \
  PERF_CI_IMAGE_DIGEST="${LOCAL_DIGEST}" \
  PERF_ALLOW_UNTRACKED_BASELINE="1" \
  PERF_BOOT_THRESHOLD_PERCENT="${LOCAL_BOOT_THRESHOLD}" \
  PERF_CONTEXT_THRESHOLD_PERCENT="${LOCAL_CONTEXT_THRESHOLD}" \
  PERF_SYSCALL_THRESHOLD_PERCENT="${LOCAL_SYSCALL_THRESHOLD}" \
  CI="false" \
  "${cmd[@]}"
}

init_baseline() {
  local reason="$1"
  echo "init_local_baseline: ${BASELINE_FILE} (${reason})"
  rm -rf "${INIT_DIR}"
  mkdir -p "${INIT_DIR}"
  if ! run_gate "${INIT_DIR}" "fail" "1" >/dev/null 2>&1; then
    :
  fi
  if [[ ! -f "${BASELINE_FILE}" ]]; then
    echo "performance-local: FAIL (local baseline init failed)" >&2
    exit 2
  fi
}

baseline_requires_refresh_preflight() {
  if [[ ! -f "${BASELINE_FILE}" ]]; then
    echo "missing_baseline"
    return 0
  fi

  BASELINE_FILE_ENV="${BASELINE_FILE}" \
  LOCAL_AUTHORITY_ENV="${LOCAL_AUTHORITY}" \
  LOCAL_DIGEST_ENV="${LOCAL_DIGEST}" \
  LOCAL_BOOT_THRESHOLD_ENV="${LOCAL_BOOT_THRESHOLD}" \
  LOCAL_CONTEXT_THRESHOLD_ENV="${LOCAL_CONTEXT_THRESHOLD}" \
  LOCAL_SYSCALL_THRESHOLD_ENV="${LOCAL_SYSCALL_THRESHOLD}" \
  python3 - <<'PY'
import json
import os
import sys

path = os.environ["BASELINE_FILE_ENV"]
authority = os.environ["LOCAL_AUTHORITY_ENV"]
digest = os.environ["LOCAL_DIGEST_ENV"]
boot = float(os.environ["LOCAL_BOOT_THRESHOLD_ENV"])
context = float(os.environ["LOCAL_CONTEXT_THRESHOLD_ENV"])
syscall = float(os.environ["LOCAL_SYSCALL_THRESHOLD_ENV"])

try:
    payload = json.load(open(path, encoding="utf-8"))
except Exception:
    print("schema_or_parse_drift")
    raise SystemExit(0)

reasons = []
if payload.get("schema_version") != 1:
    reasons.append("schema_drift")
policy = payload.get("policy", {})
env = payload.get("env", {})
marker = policy.get("marker_contract", {})
thresholds = policy.get("thresholds_percent", {})
metrics = payload.get("metrics", {})

if policy.get("baseline_authority") != authority:
    reasons.append("authority_drift")
if env.get("ci_image_digest") != digest:
    reasons.append("digest_drift")
if marker.get("measurement_contract") != "deterministic_preempt_harness":
    reasons.append("measurement_contract_drift")
if thresholds.get("boot_time_ms") != boot:
    reasons.append("boot_threshold_drift")
if thresholds.get("context_switch_latency_ms_proxy") != context:
    reasons.append("context_threshold_drift")
if thresholds.get("syscall_latency_ms_proxy") != syscall:
    reasons.append("syscall_threshold_drift")
if metrics.get("boot_time_ms") is None:
    reasons.append("boot_metric_missing")
if metrics.get("context_switch_latency_ms_proxy") is None:
    reasons.append("context_metric_missing")
if metrics.get("syscall_latency_ms_proxy") is None:
    reasons.append("syscall_metric_missing")

print(",".join(reasons))
PY
}

diff_is_auto_refreshable() {
  local diff_file="$1"
  [[ -f "${diff_file}" ]] || return 1

  local saw_any=0
  while IFS= read -r line; do
    [[ -z "${line}" ]] && continue
    saw_any=1
    case "${line}" in
      env_hash_mismatch_waiver_required:*)
        ;;
      schema_version_mismatch:*)
        ;;
      baseline_authority_mismatch:*)
        ;;
      ci_image_digest_mismatch:*)
        ;;
      marker_contract_mismatch)
        ;;
      measurement_contract_mismatch:*)
        ;;
      metric_missing:*:baseline=None:actual=*)
        ;;
      *)
        return 1
        ;;
    esac
  done < "${diff_file}"

  [[ "${saw_any}" == "1" ]]
}

preflight_reason="$(baseline_requires_refresh_preflight)"
if [[ -n "${preflight_reason}" ]]; then
  init_baseline "${preflight_reason}"
fi

if ! run_gate "${EVIDENCE_DIR}" "waiver" "0"; then
  diff_file="${EVIDENCE_DIR}/baseline.diff.txt"
  if diff_is_auto_refreshable "${diff_file}"; then
    init_baseline "env_or_contract_drift"
    run_gate "${EVIDENCE_DIR}" "waiver" "0"
  else
    echo "performance-local: FAIL (metric regression or non-refreshable drift)" >&2
    exit 2
  fi
fi

if [[ -f "${INIT_DIR}/report.json" ]]; then
  mv -f "${INIT_DIR}/report.json" "${INIT_DIR}/init-report.json"
fi

echo "performance: PASS"
exit 0

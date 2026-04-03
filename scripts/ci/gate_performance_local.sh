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
  PERF_LOCAL_SAMPLE_SIZE=<n>                   (default: 5)
  PERF_LOCAL_WARMUP_RUNS=<n>                   (default: 1)
  PERF_LOCAL_AGGREGATION=median                (default: median)
  PERF_LOCAL_OUTLIER_POLICY=none               (default: none)
  AYKEN_SCHED_FALLBACK=0|1                     (default: 0)

Behavior:
  - Local gate collects multiple provisional samples and compares medians.
  - Auto-refresh is limited to structural baseline drift and pure env drift.
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
LOCAL_SAMPLE_SIZE="${PERF_LOCAL_SAMPLE_SIZE:-5}"
LOCAL_WARMUP_RUNS="${PERF_LOCAL_WARMUP_RUNS:-1}"
LOCAL_AGGREGATION="${PERF_LOCAL_AGGREGATION:-median}"
LOCAL_OUTLIER_POLICY="${PERF_LOCAL_OUTLIER_POLICY:-none}"
SCHED_FALLBACK="${AYKEN_SCHED_FALLBACK:-0}"

SAMPLES_DIR=""
INIT_DIR=""
INIT_SAMPLES_DIR=""
STABLE_METRICS_JSON=""
BASELINE_DIFF_TXT=""
VIOLATIONS_TXT=""
META_TXT=""
REPORT_JSON=""
COMPARE_STATE_JSON=""

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

is_nonnegative_int() {
  [[ "$1" =~ ^[0-9]+$ ]]
}

is_nonnegative_number() {
  [[ "$1" =~ ^[0-9]+([.][0-9]+)?$ ]]
}

if ! is_nonnegative_int "${LOCAL_SAMPLE_SIZE}" || [[ "${LOCAL_SAMPLE_SIZE}" -lt 1 ]]; then
  echo "ERROR: PERF_LOCAL_SAMPLE_SIZE must be integer >= 1" >&2
  exit 2
fi
if ! is_nonnegative_int "${LOCAL_WARMUP_RUNS}"; then
  echo "ERROR: PERF_LOCAL_WARMUP_RUNS must be integer >= 0" >&2
  exit 2
fi
if [[ "${LOCAL_AGGREGATION}" != "median" ]]; then
  echo "ERROR: PERF_LOCAL_AGGREGATION must be 'median'" >&2
  exit 2
fi
if [[ "${LOCAL_OUTLIER_POLICY}" != "none" ]]; then
  echo "ERROR: PERF_LOCAL_OUTLIER_POLICY must be 'none'" >&2
  exit 2
fi
for threshold_value in "${LOCAL_BOOT_THRESHOLD}" "${LOCAL_CONTEXT_THRESHOLD}" "${LOCAL_SYSCALL_THRESHOLD}"; do
  if ! is_nonnegative_number "${threshold_value}"; then
    echo "ERROR: local performance thresholds must be non-negative numbers" >&2
    exit 2
  fi
done

SAMPLES_DIR="${EVIDENCE_DIR}/samples"
INIT_DIR="${EVIDENCE_DIR}-local-init"
INIT_SAMPLES_DIR="${INIT_DIR}/samples"
STABLE_METRICS_JSON="${EVIDENCE_DIR}/stable_metrics.json"
BASELINE_DIFF_TXT="${EVIDENCE_DIR}/baseline.diff.txt"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
COMPARE_STATE_JSON="${EVIDENCE_DIR}/compare_state.json"

mkdir -p "${EVIDENCE_DIR}"

run_gate_sample() {
  local evidence_dir="$1"
  local scratch_baseline="${evidence_dir}/provisional-baseline.lock.json"

  PERF_BASELINE_MODE="provisional" \
  PERF_BASELINE_AUTHORITY="${LOCAL_AUTHORITY}" \
  PERF_REQUIRE_CI_FOR_BASELINE_INIT="0" \
  PERF_CI_IMAGE_DIGEST="${LOCAL_DIGEST}" \
  PERF_ALLOW_UNTRACKED_BASELINE="1" \
  PERF_BOOT_THRESHOLD_PERCENT="${LOCAL_BOOT_THRESHOLD}" \
  PERF_CONTEXT_THRESHOLD_PERCENT="${LOCAL_CONTEXT_THRESHOLD}" \
  PERF_SYSCALL_THRESHOLD_PERCENT="${LOCAL_SYSCALL_THRESHOLD}" \
  AYKEN_SCHED_FALLBACK="${SCHED_FALLBACK}" \
  CI="false" \
  "${ROOT}/scripts/ci/gate_performance.sh" \
    --evidence-dir "${evidence_dir}" \
    --kernel-profile "${KERNEL_PROFILE}" \
    --qemu-timeout "${QEMU_TIMEOUT}" \
    --env-mismatch-policy "waiver" \
    --baseline-file "${scratch_baseline}"
}

collect_samples() {
  local sample_root="$1"
  local total_runs=$((LOCAL_WARMUP_RUNS + LOCAL_SAMPLE_SIZE))
  local idx=1

  rm -rf "${sample_root}"
  mkdir -p "${sample_root}"

  while [[ "${idx}" -le "${total_runs}" ]]; do
    local sample_dir="${sample_root}/sample-${idx}"
    mkdir -p "${sample_dir}"
    if ! run_gate_sample "${sample_dir}" >/dev/null 2>&1; then
      echo "sample_collection_failed:sample-${idx}" >&2
      return 1
    fi
    idx=$((idx + 1))
  done
}

build_stable_metrics_json() {
  local sample_root="$1"
  local output_path="$2"

  SAMPLE_ROOT_ENV="${sample_root}" \
  WARMUP_ENV="${LOCAL_WARMUP_RUNS}" \
  SAMPLE_SIZE_ENV="${LOCAL_SAMPLE_SIZE}" \
  AGGREGATION_ENV="${LOCAL_AGGREGATION}" \
  OUTLIER_POLICY_ENV="${LOCAL_OUTLIER_POLICY}" \
  OUTPUT_PATH_ENV="${output_path}" \
  python3 - <<'PY'
import json
import os
from pathlib import Path

sample_root = Path(os.environ["SAMPLE_ROOT_ENV"])
warmup = int(os.environ["WARMUP_ENV"])
sample_size = int(os.environ["SAMPLE_SIZE_ENV"])
aggregation = os.environ["AGGREGATION_ENV"]
outlier_policy = os.environ["OUTLIER_POLICY_ENV"]
output_path = Path(os.environ["OUTPUT_PATH_ENV"])

sample_dirs = sorted(
    p for p in sample_root.iterdir() if p.is_dir() and p.name.startswith("sample-")
)
usable_dirs = sample_dirs[warmup:]

if len(usable_dirs) != sample_size:
    raise SystemExit(f"sample_count_mismatch:{len(usable_dirs)} != {sample_size}")

reports = []
for sample_dir in usable_dirs:
    report_path = sample_dir / "report.json"
    payload = json.load(report_path.open(encoding="utf-8"))
    if payload.get("verdict") != "PASS":
        raise SystemExit(f"sample_report_not_pass:{sample_dir.name}")
    reports.append(payload)

if not reports:
    raise SystemExit("no_usable_sample_reports")

env = reports[0].get("env", {})
marker_contract = dict(env.get("marker_contract", {}))

def median(values):
    ordered = sorted(float(v) for v in values)
    n = len(ordered)
    mid = n // 2
    if n % 2 == 1:
        return ordered[mid]
    return (ordered[mid - 1] + ordered[mid]) / 2.0

metrics = {}
for key in ("boot_time_ms", "context_switch_latency_ms_proxy", "syscall_latency_ms_proxy"):
    vals = [report.get("results", {}).get(key) for report in reports]
    if any(v is None for v in vals):
        raise SystemExit(f"sample_metric_missing:{key}")
    metrics[key] = {
        "median": median(vals),
        "samples": [float(v) for v in vals],
    }

payload = {
    "sampling": {
        "sample_size": sample_size,
        "warmup_runs": warmup,
        "aggregation": aggregation,
        "outlier_policy": outlier_policy,
        "usable_sample_dirs": [str(p) for p in usable_dirs],
    },
    "env": env,
    "marker_contract": marker_contract,
    "metrics": metrics,
}

output_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
}

write_local_baseline() {
  local stable_metrics_path="$1"
  local output_path="$2"

  STABLE_METRICS_ENV="${stable_metrics_path}" \
  OUTPUT_PATH_ENV="${output_path}" \
  LOCAL_AUTHORITY_ENV="${LOCAL_AUTHORITY}" \
  LOCAL_BOOT_THRESHOLD_ENV="${LOCAL_BOOT_THRESHOLD}" \
  LOCAL_CONTEXT_THRESHOLD_ENV="${LOCAL_CONTEXT_THRESHOLD}" \
  LOCAL_SYSCALL_THRESHOLD_ENV="${LOCAL_SYSCALL_THRESHOLD}" \
  LOCAL_SAMPLE_SIZE_ENV="${LOCAL_SAMPLE_SIZE}" \
  LOCAL_WARMUP_RUNS_ENV="${LOCAL_WARMUP_RUNS}" \
  LOCAL_AGGREGATION_ENV="${LOCAL_AGGREGATION}" \
  LOCAL_OUTLIER_POLICY_ENV="${LOCAL_OUTLIER_POLICY}" \
  GIT_SHA_ENV="$(git -C "${ROOT}" rev-parse HEAD 2>/dev/null || echo NO_GIT)" \
  python3 - <<'PY'
import json
import os
from datetime import datetime, timezone
from pathlib import Path

stable = json.load(open(os.environ["STABLE_METRICS_ENV"], encoding="utf-8"))
payload = {
    "schema_version": 2,
    "created_at_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "git_sha": os.environ["GIT_SHA_ENV"],
    "policy": {
        "baseline_authority": os.environ["LOCAL_AUTHORITY_ENV"],
        "env_mismatch_policy": "waiver",
        "marker_contract": stable.get("marker_contract", {}),
        "thresholds_percent": {
            "boot_time_ms": float(os.environ["LOCAL_BOOT_THRESHOLD_ENV"]),
            "context_switch_latency_ms_proxy": float(os.environ["LOCAL_CONTEXT_THRESHOLD_ENV"]),
            "syscall_latency_ms_proxy": float(os.environ["LOCAL_SYSCALL_THRESHOLD_ENV"]),
        },
        "sampling": {
            "sample_size": int(os.environ["LOCAL_SAMPLE_SIZE_ENV"]),
            "warmup_runs": int(os.environ["LOCAL_WARMUP_RUNS_ENV"]),
            "aggregation": os.environ["LOCAL_AGGREGATION_ENV"],
            "outlier_policy": os.environ["LOCAL_OUTLIER_POLICY_ENV"],
        },
    },
    "env": stable.get("env", {}),
    "metrics": stable.get("metrics", {}),
}

out = Path(os.environ["OUTPUT_PATH_ENV"])
out.parent.mkdir(parents=True, exist_ok=True)
out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
}

write_init_report() {
  local init_reason="$1"
  local stable_metrics_path="$2"

  mkdir -p "${INIT_DIR}"
  STABLE_METRICS_ENV="${stable_metrics_path}" \
  OUTPUT_PATH_ENV="${INIT_DIR}/init-report.json" \
  INIT_REASON_ENV="${init_reason}" \
  python3 - <<'PY'
import json
import os
from pathlib import Path

stable = json.load(open(os.environ["STABLE_METRICS_ENV"], encoding="utf-8"))
payload = {
    "gate": "performance-local-init",
    "verdict": "PASS",
    "baseline_status": "auto_refreshed_structural_drift",
    "auto_refresh_reason": os.environ["INIT_REASON_ENV"],
    "sampling": stable.get("sampling", {}),
    "env": stable.get("env", {}),
    "metrics": stable.get("metrics", {}),
    "violations": [],
    "violations_count": 0,
}
Path(os.environ["OUTPUT_PATH_ENV"]).write_text(
    json.dumps(payload, indent=2, sort_keys=True) + "\n",
    encoding="utf-8",
)
PY
}

baseline_requires_refresh_preflight() {
  if [[ ! -f "${BASELINE_FILE}" ]]; then
    echo "missing_baseline"
    return 0
  fi

  BASELINE_FILE_ENV="${BASELINE_FILE}" \
  LOCAL_BOOT_THRESHOLD_ENV="${LOCAL_BOOT_THRESHOLD}" \
  LOCAL_CONTEXT_THRESHOLD_ENV="${LOCAL_CONTEXT_THRESHOLD}" \
  LOCAL_SYSCALL_THRESHOLD_ENV="${LOCAL_SYSCALL_THRESHOLD}" \
  LOCAL_SAMPLE_SIZE_ENV="${LOCAL_SAMPLE_SIZE}" \
  LOCAL_WARMUP_RUNS_ENV="${LOCAL_WARMUP_RUNS}" \
  LOCAL_AGGREGATION_ENV="${LOCAL_AGGREGATION}" \
  LOCAL_OUTLIER_POLICY_ENV="${LOCAL_OUTLIER_POLICY}" \
  python3 - <<'PY'
import json
import os

path = os.environ["BASELINE_FILE_ENV"]
boot = float(os.environ["LOCAL_BOOT_THRESHOLD_ENV"])
context = float(os.environ["LOCAL_CONTEXT_THRESHOLD_ENV"])
syscall = float(os.environ["LOCAL_SYSCALL_THRESHOLD_ENV"])
sample_size = int(os.environ["LOCAL_SAMPLE_SIZE_ENV"])
warmup = int(os.environ["LOCAL_WARMUP_RUNS_ENV"])
aggregation = os.environ["LOCAL_AGGREGATION_ENV"]
outlier_policy = os.environ["LOCAL_OUTLIER_POLICY_ENV"]

try:
    payload = json.load(open(path, encoding="utf-8"))
except Exception:
    print("schema_or_parse_drift")
    raise SystemExit(0)

reasons = []
if payload.get("schema_version") != 2:
    reasons.append("schema_drift")

policy = payload.get("policy", {})
marker = policy.get("marker_contract", {})
thresholds = policy.get("thresholds_percent", {})
sampling = policy.get("sampling", {})
metrics = payload.get("metrics", {})

if marker.get("measurement_contract") != "deterministic_preempt_harness":
    reasons.append("measurement_contract_drift")
if thresholds.get("boot_time_ms") != boot:
    reasons.append("boot_threshold_drift")
if thresholds.get("context_switch_latency_ms_proxy") != context:
    reasons.append("context_threshold_drift")
if thresholds.get("syscall_latency_ms_proxy") != syscall:
    reasons.append("syscall_threshold_drift")
if sampling.get("sample_size") != sample_size:
    reasons.append("sample_size_drift")
if sampling.get("warmup_runs") != warmup:
    reasons.append("warmup_runs_drift")
if sampling.get("aggregation") != aggregation:
    reasons.append("aggregation_drift")
if sampling.get("outlier_policy") != outlier_policy:
    reasons.append("outlier_policy_drift")

for key in ("boot_time_ms", "context_switch_latency_ms_proxy", "syscall_latency_ms_proxy"):
    value = metrics.get(key)
    if not isinstance(value, dict):
        reasons.append(f"{key}_legacy_metric_shape")
        continue
    if value.get("median") is None:
        reasons.append(f"{key}_median_missing")
    samples = value.get("samples")
    if not isinstance(samples, list) or len(samples) != sample_size:
        reasons.append(f"{key}_samples_missing")

print(",".join(reasons))
PY
}

compare_against_baseline() {
  local baseline_file="$1"
  local stable_metrics_path="$2"
  local report_path="$3"
  local diff_path="$4"
  local violations_path="$5"
  local state_path="$6"

  BASELINE_FILE_ENV="${baseline_file}" \
  STABLE_METRICS_ENV="${stable_metrics_path}" \
  REPORT_PATH_ENV="${report_path}" \
  DIFF_PATH_ENV="${diff_path}" \
  VIOLATIONS_PATH_ENV="${violations_path}" \
  STATE_PATH_ENV="${state_path}" \
  python3 - <<'PY'
import json
import os
from pathlib import Path

baseline = json.load(open(os.environ["BASELINE_FILE_ENV"], encoding="utf-8"))
current = json.load(open(os.environ["STABLE_METRICS_ENV"], encoding="utf-8"))

thresholds = baseline.get("policy", {}).get("thresholds_percent", {})
baseline_metrics = baseline.get("metrics", {})
current_metrics = current.get("metrics", {})
baseline_env = baseline.get("env", {})
current_env = current.get("env", {})

diffs = []
violations = []

def add_diff(line, violation=False):
    diffs.append(line)
    if violation:
        violations.append(f"baseline_diff:{line}")

b_env_hash = baseline_env.get("env_hash")
c_env_hash = current_env.get("env_hash")
if b_env_hash != c_env_hash:
    add_diff(f"env_hash_mismatch_waiver_required: baseline={b_env_hash} actual={c_env_hash}")

b_authority = baseline.get("policy", {}).get("baseline_authority")
c_authority = current_env.get("baseline_authority")
if b_authority != c_authority:
    add_diff(f"baseline_authority_mismatch: baseline={b_authority} actual={c_authority}")

b_digest = baseline_env.get("ci_image_digest")
c_digest = current_env.get("ci_image_digest")
if b_digest != c_digest:
    add_diff(f"ci_image_digest_mismatch: baseline={b_digest} actual={c_digest}")

metric_regressions = []
metrics_summary = {}
for key in ("boot_time_ms", "context_switch_latency_ms_proxy", "syscall_latency_ms_proxy"):
    baseline_value = baseline_metrics.get(key, {})
    current_value = current_metrics.get(key, {})
    baseline_median = baseline_value.get("median")
    current_median = current_value.get("median")
    threshold_percent = float(thresholds.get(key, 0))
    metrics_summary[key] = {
        "baseline_median": baseline_median,
        "current_median": current_median,
        "samples": current_value.get("samples", []),
        "threshold_percent": threshold_percent,
    }
    if baseline_median is None or current_median is None:
        add_diff(f"metric_missing:{key}:baseline={baseline_median}:actual={current_median}", violation=True)
        continue
    max_allowed = float(baseline_median) * (1.0 + threshold_percent / 100.0)
    if float(current_median) > max_allowed:
        diff = (
            f"metric_regression:{key}:baseline={baseline_median}:actual={current_median}:"
            f"threshold_percent={threshold_percent}:max_allowed={max_allowed}"
        )
        add_diff(diff, violation=True)
        metric_regressions.append(diff)

refreshable_env_only = bool(diffs) and not metric_regressions
refreshable_prefixes = (
    "env_hash_mismatch_waiver_required:",
    "baseline_authority_mismatch:",
    "ci_image_digest_mismatch:",
)
if refreshable_env_only:
    for line in diffs:
        if not line.startswith(refreshable_prefixes):
            refreshable_env_only = False
            break

if metric_regressions:
    violations.append(f"baseline_mismatch:{os.environ['BASELINE_FILE_ENV']}")
    decision = "fail"
    verdict = "FAIL"
    baseline_status = "mismatch"
elif refreshable_env_only:
    decision = "refreshable_env_drift"
    verdict = "PASS"
    baseline_status = "env_drift_refreshable"
else:
    decision = "match"
    verdict = "PASS"
    baseline_status = "match"

Path(os.environ["DIFF_PATH_ENV"]).write_text(
    ("\n".join(diffs) + "\n") if diffs else "",
    encoding="utf-8",
)
Path(os.environ["VIOLATIONS_PATH_ENV"]).write_text(
    ("\n".join(violations) + "\n") if violations else "",
    encoding="utf-8",
)

state = {
    "decision": decision,
    "baseline_diff": diffs,
    "violations": violations,
    "refreshable_env_only": refreshable_env_only,
    "metric_regressions": metric_regressions,
}
Path(os.environ["STATE_PATH_ENV"]).write_text(
    json.dumps(state, indent=2, sort_keys=True) + "\n",
    encoding="utf-8",
)

report = {
    "gate": "performance",
    "verdict": verdict,
    "baseline_status": baseline_status,
    "decision": decision,
    "measurement_contract": "deterministic_preempt_harness",
    "sampling": current.get("sampling", {}),
    "baseline": {
        "schema_version": baseline.get("schema_version"),
        "baseline_authority": b_authority,
        "ci_image_digest": b_digest,
        "env_hash": b_env_hash,
    },
    "current": {
        "baseline_authority": c_authority,
        "ci_image_digest": c_digest,
        "env_hash": c_env_hash,
    },
    "metrics": metrics_summary,
    "baseline_diff": diffs,
    "violations": violations,
    "violations_count": len(violations),
}
Path(os.environ["REPORT_PATH_ENV"]).write_text(
    json.dumps(report, indent=2, sort_keys=True) + "\n",
    encoding="utf-8",
)
PY
}

write_structural_refresh_report() {
  local init_reason="$1"
  local stable_metrics_path="$2"

  STABLE_METRICS_ENV="${stable_metrics_path}" \
  OUTPUT_PATH_ENV="${REPORT_JSON}" \
  INIT_REASON_ENV="${init_reason}" \
  python3 - <<'PY'
import json
import os
from pathlib import Path

stable = json.load(open(os.environ["STABLE_METRICS_ENV"], encoding="utf-8"))
metrics = {}
for key, value in stable.get("metrics", {}).items():
    metrics[key] = {
        "baseline_median": value.get("median"),
        "current_median": value.get("median"),
        "samples": value.get("samples", []),
        "threshold_percent": None,
    }

payload = {
    "gate": "performance",
    "verdict": "PASS",
    "baseline_status": "auto_refreshed_structural_drift",
    "auto_refresh_reason": os.environ["INIT_REASON_ENV"],
    "measurement_contract": "deterministic_preempt_harness",
    "sampling": stable.get("sampling", {}),
    "current": {
        "baseline_authority": stable.get("env", {}).get("baseline_authority"),
        "ci_image_digest": stable.get("env", {}).get("ci_image_digest"),
        "env_hash": stable.get("env", {}).get("env_hash"),
    },
    "metrics": metrics,
    "baseline_diff": [],
    "violations": [],
    "violations_count": 0,
}
Path(os.environ["OUTPUT_PATH_ENV"]).write_text(
    json.dumps(payload, indent=2, sort_keys=True) + "\n",
    encoding="utf-8",
)
PY
  : > "${BASELINE_DIFF_TXT}"
  : > "${VIOLATIONS_TXT}"
}

refresh_report_after_env_drift() {
  python3 - <<'PY' "${REPORT_JSON}"
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
payload = json.load(path.open(encoding="utf-8"))
payload["baseline_status"] = "auto_refreshed_env_drift"
payload["auto_refresh_reason"] = "pure_env_drift"
payload["verdict"] = "PASS"
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
}

write_meta() {
  local baseline_status="$1"
  local auto_refresh_reason="${2:-}"
  {
    echo "baseline_file=${BASELINE_FILE}"
    echo "baseline_status=${baseline_status}"
    echo "auto_refresh_reason=${auto_refresh_reason}"
    echo "baseline_authority=${LOCAL_AUTHORITY}"
    echo "ci_image_digest=${LOCAL_DIGEST}"
    echo "kernel_profile=${KERNEL_PROFILE}"
    echo "qemu_timeout=${QEMU_TIMEOUT}"
    echo "sample_size=${LOCAL_SAMPLE_SIZE}"
    echo "warmup_runs=${LOCAL_WARMUP_RUNS}"
    echo "aggregation=${LOCAL_AGGREGATION}"
    echo "outlier_policy=${LOCAL_OUTLIER_POLICY}"
    echo "boot_threshold_percent=${LOCAL_BOOT_THRESHOLD}"
    echo "context_threshold_percent=${LOCAL_CONTEXT_THRESHOLD}"
    echo "syscall_threshold_percent=${LOCAL_SYSCALL_THRESHOLD}"
  } > "${META_TXT}"
}

preflight_reason="$(baseline_requires_refresh_preflight)"
if [[ -n "${preflight_reason}" ]]; then
  collect_samples "${SAMPLES_DIR}"
  build_stable_metrics_json "${SAMPLES_DIR}" "${STABLE_METRICS_JSON}"
  write_local_baseline "${STABLE_METRICS_JSON}" "${BASELINE_FILE}"
  write_init_report "${preflight_reason}" "${STABLE_METRICS_JSON}"
  write_structural_refresh_report "${preflight_reason}" "${STABLE_METRICS_JSON}"
  write_meta "auto_refreshed_structural_drift" "${preflight_reason}"
  echo "performance: PASS"
  exit 0
fi

collect_samples "${SAMPLES_DIR}"
build_stable_metrics_json "${SAMPLES_DIR}" "${STABLE_METRICS_JSON}"
compare_against_baseline \
  "${BASELINE_FILE}" \
  "${STABLE_METRICS_JSON}" \
  "${REPORT_JSON}" \
  "${BASELINE_DIFF_TXT}" \
  "${VIOLATIONS_TXT}" \
  "${COMPARE_STATE_JSON}"

decision="$(jq -r '.decision' "${COMPARE_STATE_JSON}")"
case "${decision}" in
  match)
    write_meta "match"
    echo "performance: PASS"
    exit 0
    ;;
  refreshable_env_drift)
    write_local_baseline "${STABLE_METRICS_JSON}" "${BASELINE_FILE}"
    refresh_report_after_env_drift
    write_meta "auto_refreshed_env_drift" "pure_env_drift"
    echo "performance: PASS"
    exit 0
    ;;
  fail)
    write_meta "fail"
    echo "performance-local: FAIL (metric regression or non-refreshable drift)" >&2
    exit 2
    ;;
  *)
    echo "performance-local: FAIL (unknown compare decision: ${decision})" >&2
    exit 2
    ;;
esac

#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_performance_learning_review.sh \
    --evidence-dir evidence/run-<id>/gates/performance-learning \
    [--source-glob 'out/evidence/run-*/gates/performance/report.json'] \
    [--source-report path/to/report.json ...]

Env controls:
  PERF_LEARNING_SOURCE_GLOB=<glob>  Additional glob of report.json files to review

Behavior:
  - Non-authoritative learning review only
  - Includes PASS performance reports only
  - Requires split metrics to be available
  - Groups runs by a single authority surface
  - Emits history/summary/recommendations JSON files

Exit codes:
  0: review completed (PASS or WARN verdict in report)
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
SOURCE_GLOB="${PERF_LEARNING_SOURCE_GLOB:-}"
declare -a SOURCE_REPORTS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --source-glob)
      SOURCE_GLOB="$2"
      shift 2
      ;;
    --source-report)
      SOURCE_REPORTS+=("$2")
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

if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: required tool missing (python3)" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

REPORTS_JSON="${EVIDENCE_DIR}/input-reports.json"
HISTORY_JSON="${EVIDENCE_DIR}/history.json"
SUMMARY_JSON="${EVIDENCE_DIR}/summary.json"
RECOMMENDATIONS_JSON="${EVIDENCE_DIR}/recommendations.json"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
SOURCE_REPORTS_TXT="${EVIDENCE_DIR}/source-reports.txt"

: > "${SOURCE_REPORTS_TXT}"
if (( ${#SOURCE_REPORTS[@]} > 0 )); then
  for report in "${SOURCE_REPORTS[@]}"; do
    printf '%s\n' "${report}" >> "${SOURCE_REPORTS_TXT}"
  done
fi

python3 - <<'PY' "${SOURCE_GLOB}" "${SOURCE_REPORTS_TXT}" "${REPORTS_JSON}" "${HISTORY_JSON}" "${SUMMARY_JSON}" "${RECOMMENDATIONS_JSON}" "${REPORT_JSON}"
import glob
import json
import math
import os
import statistics
import sys
from pathlib import Path


def percentile(sorted_values: list[float], pct: float) -> float | None:
    if not sorted_values:
        return None
    if len(sorted_values) == 1:
        return sorted_values[0]
    rank = (len(sorted_values) - 1) * pct
    lo = math.floor(rank)
    hi = math.ceil(rank)
    if lo == hi:
        return sorted_values[lo]
    frac = rank - lo
    return sorted_values[lo] + (sorted_values[hi] - sorted_values[lo]) * frac


def median_abs_deviation(values: list[float], median: float) -> float:
    deviations = [abs(v - median) for v in values]
    return statistics.median(deviations) if deviations else 0.0


source_glob = sys.argv[1]
source_reports_txt = Path(sys.argv[2])
reports_json = Path(sys.argv[3])
history_json = Path(sys.argv[4])
summary_json = Path(sys.argv[5])
recommendations_json = Path(sys.argv[6])
report_json = Path(sys.argv[7])
cli_sources = []
if source_reports_txt.exists():
    cli_sources = [line.strip() for line in source_reports_txt.read_text(encoding="utf-8").splitlines() if line.strip()]

source_paths: list[str] = []
if source_glob:
    source_paths.extend(glob.glob(source_glob))
source_paths.extend(cli_sources)
source_paths = [str(Path(p)) for p in source_paths]
source_paths = sorted(dict.fromkeys(source_paths))

reports_json.write_text(json.dumps({"source_reports": source_paths}, indent=2) + "\n", encoding="utf-8")

if not source_paths:
    report_json.write_text(
        json.dumps(
            {
                "gate": "performance-learning-review",
                "verdict": "WARN",
                "reason": "no_source_reports",
                "eligible_run_count": 0,
                "excluded_run_count": 0,
                "output_files": {
                    "history": history_json.name,
                    "summary": summary_json.name,
                    "recommendations": recommendations_json.name,
                },
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )
    history_json.write_text(json.dumps({"schema_version": 1, "authority": None, "runs": []}, indent=2) + "\n", encoding="utf-8")
    summary_json.write_text(json.dumps({"schema_version": 1, "authority": None, "metrics": {}}, indent=2) + "\n", encoding="utf-8")
    recommendations_json.write_text(
        json.dumps({"schema_version": 1, "authority": None, "recommendations": {}}, indent=2) + "\n",
        encoding="utf-8",
    )
    raise SystemExit(0)

eligible_runs = []
excluded_runs = []
authority = None

for source in source_paths:
    try:
        payload = json.loads(Path(source).read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001
        excluded_runs.append({"source": source, "reason": f"unreadable:{exc.__class__.__name__}"})
        continue

    verdict = payload.get("verdict")
    env = payload.get("env", {})
    results = payload.get("results", {})
    meta = payload.get("meta", {})
    source_authority = env.get("baseline_authority")

    if verdict != "PASS":
        excluded_runs.append({"source": source, "reason": f"verdict:{verdict or 'missing'}"})
        continue

    if not source_authority:
        excluded_runs.append({"source": source, "reason": "missing_authority"})
        continue

    if authority is None:
        authority = source_authority
    elif authority != source_authority:
        excluded_runs.append(
            {
                "source": source,
                "reason": f"authority_mismatch:{source_authority}",
            }
        )
        continue

    split_metrics = {
        "entry_latency_ticks": results.get("entry_latency_ticks", {}),
        "syscall_latency_ticks_pure": results.get("syscall_latency_ticks_pure", {}),
        "syscall_gate_return_latency_ticks": results.get("syscall_gate_return_latency_ticks", {}),
    }
    missing_metric = None
    run_metrics: dict[str, float] = {}
    for metric_name, metric_payload in split_metrics.items():
        if not isinstance(metric_payload, dict) or not metric_payload.get("available") or metric_payload.get("ticks", 0) <= 0:
            missing_metric = metric_name
            break
        run_metrics[metric_name] = float(metric_payload["ticks"])

    if missing_metric is not None:
        excluded_runs.append({"source": source, "reason": f"split_metric_missing:{missing_metric}"})
        continue

    eligible_runs.append(
        {
            "run_id": meta.get("run_id", Path(source).parts[-4] if len(Path(source).parts) >= 4 else Path(source).stem),
            "git_sha": meta.get("git_sha", "unknown"),
            "authority": source_authority,
            "source_report": source,
            "metrics": run_metrics,
        }
    )


history_payload = {
    "schema_version": 1,
    "authority": authority,
    "eligibility_policy": {
        "verdict": "PASS",
        "same_authority_required": True,
        "required_split_metrics": [
            "entry_latency_ticks",
            "syscall_latency_ticks_pure",
            "syscall_gate_return_latency_ticks",
        ],
    },
    "runs": eligible_runs,
}
history_json.write_text(json.dumps(history_payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

summary_metrics = {}
recommendations = {}

recommendation_config = {
    "entry_latency_ticks": {"enforcement": "soft", "mad_multiplier": 10.0, "percentile": 0.95},
    "syscall_latency_ticks_pure": {"enforcement": "hard", "mad_multiplier": 6.0, "percentile": 0.95},
    "syscall_gate_return_latency_ticks": {"enforcement": "medium", "mad_multiplier": 8.0, "percentile": 0.95},
}

for metric_name, config in recommendation_config.items():
    values = sorted(float(run["metrics"][metric_name]) for run in eligible_runs if metric_name in run["metrics"])
    if not values:
        continue
    median = statistics.median(values)
    mad = median_abs_deviation(values, median)
    p90 = percentile(values, 0.90)
    p95 = percentile(values, config["percentile"])
    summary_metrics[metric_name] = {
        "sample_count": len(values),
        "median": median,
        "median_abs_deviation": mad,
        "p90": p90,
        "p95": p95,
        "min": min(values),
        "max": max(values),
    }
    recommendation = max(p95 or median, median + (config["mad_multiplier"] * mad))
    recommendations[metric_name] = {
        "enforcement": config["enforcement"],
        "recommended_threshold_ticks": recommendation,
        "based_on": f"max(p95, median + {config['mad_multiplier']:.0f}*MAD)",
        "sample_count": len(values),
    }

summary_payload = {
    "schema_version": 1,
    "authority": authority,
    "metrics": summary_metrics,
}
summary_json.write_text(json.dumps(summary_payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

recommendations_payload = {
    "schema_version": 1,
    "authority": authority,
    "recommendations": recommendations,
}
recommendations_json.write_text(json.dumps(recommendations_payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

report_payload = {
    "gate": "performance-learning-review",
    "verdict": "PASS" if eligible_runs else "WARN",
    "authority": authority,
    "eligible_run_count": len(eligible_runs),
    "excluded_run_count": len(excluded_runs),
    "eligibility_policy": history_payload["eligibility_policy"],
    "output_files": {
        "history": history_json.name,
        "summary": summary_json.name,
        "recommendations": recommendations_json.name,
    },
    "excluded_runs": excluded_runs,
    "summary_preview": summary_metrics,
    "recommendations_preview": recommendations,
    "note": "Learning review is non-authoritative. It does not mutate perf baselines or thresholds.",
}
report_json.write_text(json.dumps(report_payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

VERDICT="$(jq -r '.verdict // "WARN"' "${REPORT_JSON}")"
if [[ "${VERDICT}" == "PASS" ]]; then
  echo "performance-learning-review: PASS"
else
  echo "performance-learning-review: WARN"
fi
echo "See: ${REPORT_JSON}"
exit 0

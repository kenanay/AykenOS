#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_performance_stability.sh \
    --evidence-dir evidence/run-<id>/gates/performance-stability \
    [--source-gate-dir evidence/run-<id>/gates/performance] \
    [--contract-file scripts/ci/perf-stability.contract.json] \
    [--contract-profile local-default]

Behavior:
  - Reads performance-local sampling evidence and evaluates measurement stability separately.
  - Performance median verdict remains authoritative for performance gating.
  - Stability uses contract-driven range/MAD/outlier checks.
  - Range and MAD breaches are fail-closed by default; outlier count is warn-only in the initial contract.
USAGE
}

EVIDENCE_DIR=""
SOURCE_GATE_DIR=""
CONTRACT_FILE="${ROOT}/scripts/ci/perf-stability.contract.json"
CONTRACT_PROFILE="${PERF_STABILITY_PROFILE:-local-default}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --source-gate-dir)
      SOURCE_GATE_DIR="$2"
      shift 2
      ;;
    --contract-file)
      CONTRACT_FILE="$2"
      shift 2
      ;;
    --contract-profile)
      CONTRACT_PROFILE="$2"
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
if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: missing required tool: python3" >&2
  exit 2
fi

mkdir -p "${EVIDENCE_DIR}"
if [[ -z "${SOURCE_GATE_DIR}" ]]; then
  SOURCE_GATE_DIR="$(cd "$(dirname "${EVIDENCE_DIR}")" && pwd)/performance"
fi
SOURCE_GATE_DIR="$(cd "${SOURCE_GATE_DIR}" && pwd)"

REPORT_JSON="${EVIDENCE_DIR}/report.json"
DETAIL_JSON="${EVIDENCE_DIR}/performance_stability_report.json"
RISKS_TXT="${EVIDENCE_DIR}/risks.txt"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

SOURCE_GATE_DIR_ENV="${SOURCE_GATE_DIR}" \
CONTRACT_FILE_ENV="${CONTRACT_FILE}" \
CONTRACT_PROFILE_ENV="${CONTRACT_PROFILE}" \
REPORT_JSON_ENV="${REPORT_JSON}" \
DETAIL_JSON_ENV="${DETAIL_JSON}" \
RISKS_TXT_ENV="${RISKS_TXT}" \
VIOLATIONS_TXT_ENV="${VIOLATIONS_TXT}" \
META_TXT_ENV="${META_TXT}" \
python3 - <<'PY'
import json
import os
import sys
from pathlib import Path

source_gate_dir = Path(os.environ["SOURCE_GATE_DIR_ENV"])
contract_file = Path(os.environ["CONTRACT_FILE_ENV"])
contract_profile = os.environ["CONTRACT_PROFILE_ENV"]
report_json = Path(os.environ["REPORT_JSON_ENV"])
detail_json = Path(os.environ["DETAIL_JSON_ENV"])
risks_txt = Path(os.environ["RISKS_TXT_ENV"])
violations_txt = Path(os.environ["VIOLATIONS_TXT_ENV"])
meta_txt = Path(os.environ["META_TXT_ENV"])

source_report_path = source_gate_dir / "report.json"

def fail(reason: str) -> None:
    payload = {
        "gate": "performance-stability",
        "verdict": "FAIL",
        "stability_status": "runtime_error",
        "reason": reason,
        "contract_file": str(contract_file),
        "contract_profile": contract_profile,
        "source_gate_dir": str(source_gate_dir),
        "source_report_path": str(source_report_path),
        "violations": [reason],
        "violations_count": 1,
        "risks": [],
        "risks_count": 0,
    }
    detail_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    report_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    violations_txt.write_text(reason + "\n", encoding="utf-8")
    risks_txt.write_text("", encoding="utf-8")
    meta_txt.write_text(
        f"source_gate_dir={source_gate_dir}\n"
        f"contract_file={contract_file}\n"
        f"contract_profile={contract_profile}\n"
        f"reason={reason}\n",
        encoding="utf-8",
    )
    print(f"performance-stability: FAIL ({reason})")
    raise SystemExit(2)

if not source_report_path.is_file():
    fail("missing_source_performance_report")
if not contract_file.is_file():
    fail("missing_stability_contract_file")

try:
    source_report = json.loads(source_report_path.read_text(encoding="utf-8"))
except Exception:
    fail("invalid_source_performance_report")

try:
    contract_payload = json.loads(contract_file.read_text(encoding="utf-8"))
except Exception:
    fail("invalid_stability_contract_file")

profile = contract_payload.get("profiles", {}).get(contract_profile)
if not isinstance(profile, dict):
    fail("missing_stability_contract_profile")

if source_report.get("gate") != "performance":
    fail("unexpected_source_gate")
if source_report.get("verdict") != "PASS":
    fail("source_performance_gate_not_pass")

metrics_contract = profile.get("metrics")
if not isinstance(metrics_contract, dict):
    fail("invalid_stability_contract_metrics")

source_metrics = source_report.get("metrics")
if not isinstance(source_metrics, dict):
    fail("missing_source_metrics")

violations = []
risks = []
metrics_summary = {}

def add_finding(line: str, severity: str) -> None:
    if severity == "fail":
        violations.append(line)
    elif severity == "warn":
        risks.append(line)
    else:
        violations.append(f"invalid_severity:{severity}:{line}")

for metric_name, metric_contract in metrics_contract.items():
    metric_payload = source_metrics.get(metric_name)
    if not isinstance(metric_payload, dict):
        fail(f"missing_source_metric:{metric_name}")
    current_stats = metric_payload.get("current_stats")
    if not isinstance(current_stats, dict):
        fail(f"missing_current_stats:{metric_name}")

    current_median = current_stats.get("median")
    current_range_pct = current_stats.get("range_percent_of_median")
    current_mad = current_stats.get("median_abs_deviation")
    current_outliers = current_stats.get("outlier_analysis", {}).get("candidate_count")

    if current_median in (None, 0):
        fail(f"invalid_current_median:{metric_name}")
    if current_range_pct is None:
        fail(f"missing_current_range_percent:{metric_name}")
    if current_mad is None:
        fail(f"missing_current_mad:{metric_name}")
    if current_outliers is None:
        fail(f"missing_current_outlier_count:{metric_name}")

    current_mad_pct = (float(current_mad) / float(current_median)) * 100.0

    metric_findings = []
    range_contract = metric_contract.get("range_percent_of_median", {})
    mad_contract = metric_contract.get("mad_percent_of_median", {})
    outlier_contract = metric_contract.get("outlier_candidate_count", {})

    def evaluate_rule(rule_name: str, actual: float, rule_contract: dict) -> None:
        if not isinstance(rule_contract, dict):
            fail(f"invalid_contract_rule:{metric_name}:{rule_name}")
        maximum = rule_contract.get("max")
        severity = rule_contract.get("severity")
        if maximum is None or severity is None:
            fail(f"incomplete_contract_rule:{metric_name}:{rule_name}")
        breached = float(actual) > float(maximum)
        result = {
            "actual": actual,
            "max": maximum,
            "severity": severity,
            "breached": breached,
        }
        metric_findings.append((rule_name, result))
        if breached:
            add_finding(
                f"{rule_name}:{metric_name}:actual={actual}:max={maximum}",
                severity,
            )

    evaluate_rule("range_percent_of_median", float(current_range_pct), range_contract)
    evaluate_rule("mad_percent_of_median", float(current_mad_pct), mad_contract)
    evaluate_rule("outlier_candidate_count", int(current_outliers), outlier_contract)

    metrics_summary[metric_name] = {
        "current_median": current_median,
        "current_range_percent_of_median": current_range_pct,
        "current_mad": current_mad,
        "current_mad_percent_of_median": current_mad_pct,
        "current_outlier_candidate_count": current_outliers,
        "current_outlier_candidates": current_stats.get("outlier_analysis", {}).get("candidates", []),
        "constraints": {name: result for name, result in metric_findings},
    }

if violations:
    verdict = "FAIL"
    stability_status = "fail"
elif risks:
    verdict = "WARN"
    stability_status = "risk"
else:
    verdict = "PASS"
    stability_status = "match"

payload = {
    "gate": "performance-stability",
    "verdict": verdict,
    "stability_status": stability_status,
    "contract_file": str(contract_file),
    "contract_profile": contract_profile,
    "contract_schema_version": contract_payload.get("schema_version"),
    "contract_description": profile.get("description"),
    "source_gate_dir": str(source_gate_dir),
    "source_report_path": str(source_report_path),
    "measurement_contract": source_report.get("measurement_contract"),
    "sampling": source_report.get("sampling", {}),
    "metrics": metrics_summary,
    "risks": risks,
    "risks_count": len(risks),
    "violations": violations,
    "violations_count": len(violations),
}

detail_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
report_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
risks_txt.write_text(("\n".join(risks) + "\n") if risks else "", encoding="utf-8")
violations_txt.write_text(("\n".join(violations) + "\n") if violations else "", encoding="utf-8")
meta_txt.write_text(
    f"source_gate_dir={source_gate_dir}\n"
    f"contract_file={contract_file}\n"
    f"contract_profile={contract_profile}\n"
    f"stability_status={stability_status}\n",
    encoding="utf-8",
)
print(f"performance-stability: {verdict}")
if verdict == "FAIL":
    raise SystemExit(2)
PY

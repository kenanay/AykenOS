#!/usr/bin/env bash
set -euo pipefail

# Author: Kenan AY

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_gcp_finalization.sh \
    --evidence-dir evidence/run-<id>/gates/gcp-finalization \
    --dlt-evidence evidence/run-<id>/gates/dlt-monotonicity \
    [--previous-gcp evidence/run-<id>/gates/gcp-finalization/gcp_snapshot.json]

Exit codes:
  0: pass
  2: GCP finalization contract failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
DLT_EVIDENCE_DIR=""
PREVIOUS_GCP=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --dlt-evidence)
      DLT_EVIDENCE_DIR="$2"
      shift 2
      ;;
    --previous-gcp)
      PREVIOUS_GCP="$2"
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

if [[ -z "${EVIDENCE_DIR}" || -z "${DLT_EVIDENCE_DIR}" ]]; then
  usage
  exit 3
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: missing required tool: python3" >&2
  exit 3
fi

VALIDATOR="${ROOT}/tools/ci/validate_gcp_finalization.py"
if [[ ! -f "${VALIDATOR}" ]]; then
  echo "ERROR: missing validator: ${VALIDATOR}" >&2
  exit 3
fi

DLT_TRACE_JSONL="${DLT_EVIDENCE_DIR}/ltick_trace.jsonl"
if [[ ! -s "${DLT_TRACE_JSONL}" ]]; then
  echo "ERROR: missing_or_empty:${DLT_TRACE_JSONL}" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

GCP_SNAPSHOT_JSON="${EVIDENCE_DIR}/gcp_snapshot.json"
GCP_RECORD_JSON="${EVIDENCE_DIR}/gcp_record.json"
GCP_CONSISTENCY_REPORT_JSON="${EVIDENCE_DIR}/gcp_consistency_report.json"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

VALIDATOR_ARGS=(
  --dlt-trace-jsonl "${DLT_TRACE_JSONL}"
  --out-gcp-snapshot "${GCP_SNAPSHOT_JSON}"
  --out-gcp-record "${GCP_RECORD_JSON}"
  --out-gcp-consistency-report "${GCP_CONSISTENCY_REPORT_JSON}"
  --out-report "${REPORT_JSON}"
)
if [[ -n "${PREVIOUS_GCP}" ]]; then
  VALIDATOR_ARGS+=(--previous-gcp "${PREVIOUS_GCP}")
fi

set +e
python3 "${VALIDATOR}" "${VALIDATOR_ARGS[@]}"
VALIDATOR_RC=$?
set -e

if [[ ! -f "${REPORT_JSON}" ]]; then
  echo "ERROR: validator did not produce report: ${REPORT_JSON}" >&2
  exit 3
fi
if [[ ! -f "${GCP_SNAPSHOT_JSON}" ]]; then
  echo "ERROR: validator did not produce gcp snapshot: ${GCP_SNAPSHOT_JSON}" >&2
  exit 3
fi
if [[ ! -f "${GCP_RECORD_JSON}" ]]; then
  echo "ERROR: validator did not produce gcp record: ${GCP_RECORD_JSON}" >&2
  exit 3
fi
if [[ ! -f "${GCP_CONSISTENCY_REPORT_JSON}" ]]; then
  echo "ERROR: validator did not produce gcp consistency report: ${GCP_CONSISTENCY_REPORT_JSON}" >&2
  exit 3
fi

python3 - "${REPORT_JSON}" "${VIOLATIONS_TXT}" <<'PY'
import json
import sys

report_path, violations_path = sys.argv[1:3]
with open(report_path, "r", encoding="utf-8") as fh:
    report = json.load(fh)
with open(violations_path, "w", encoding="utf-8") as fh:
    for violation in report.get("violations", []):
        fh.write(f"{violation}\n")
PY

{
  echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "dlt_trace_jsonl=${DLT_TRACE_JSONL}"
  echo "previous_gcp=${PREVIOUS_GCP}"
  echo "validator_rc=${VALIDATOR_RC}"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "gcp-finalization: FAIL (${COUNT} violations)"
  exit 2
fi

echo "gcp-finalization: PASS"
exit 0

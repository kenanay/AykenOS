#!/usr/bin/env bash
set -euo pipefail

# Author: Kenan AY

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_eti_dlt_binding.sh \
    --evidence-dir evidence/run-<id>/gates/eti-dlt-binding \
    --eti-evidence evidence/run-<id>/gates/eti \
    --dlt-evidence evidence/run-<id>/gates/dlt-monotonicity

Exit codes:
  0: pass
  2: ETI-DLT binding contract failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
ETI_EVIDENCE_DIR=""
DLT_EVIDENCE_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --eti-evidence)
      ETI_EVIDENCE_DIR="$2"
      shift 2
      ;;
    --dlt-evidence)
      DLT_EVIDENCE_DIR="$2"
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

if [[ -z "${EVIDENCE_DIR}" || -z "${ETI_EVIDENCE_DIR}" || -z "${DLT_EVIDENCE_DIR}" ]]; then
  usage
  exit 3
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: missing required tool: python3" >&2
  exit 3
fi

VALIDATOR="${ROOT}/tools/ci/validate_eti_dlt_binding.py"
if [[ ! -f "${VALIDATOR}" ]]; then
  echo "ERROR: missing validator: ${VALIDATOR}" >&2
  exit 3
fi

ETI_JSONL="${ETI_EVIDENCE_DIR}/eti_transcript.jsonl"
LTICK_TRACE_JSONL="${DLT_EVIDENCE_DIR}/ltick_trace.jsonl"
if [[ ! -s "${ETI_JSONL}" ]]; then
  echo "ERROR: missing_or_empty:${ETI_JSONL}" >&2
  exit 3
fi
if [[ ! -s "${LTICK_TRACE_JSONL}" ]]; then
  echo "ERROR: missing_or_empty:${LTICK_TRACE_JSONL}" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

BINDING_REPORT_JSON="${EVIDENCE_DIR}/binding_report.json"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

set +e
python3 "${VALIDATOR}" \
  --eti-jsonl "${ETI_JSONL}" \
  --ltick-trace-jsonl "${LTICK_TRACE_JSONL}" \
  --out-binding-report "${BINDING_REPORT_JSON}" \
  --out-report "${REPORT_JSON}"
VALIDATOR_RC=$?
set -e

if [[ ! -f "${REPORT_JSON}" ]]; then
  echo "ERROR: validator did not produce report: ${REPORT_JSON}" >&2
  exit 3
fi
if [[ ! -f "${BINDING_REPORT_JSON}" ]]; then
  echo "ERROR: validator did not produce binding report: ${BINDING_REPORT_JSON}" >&2
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
  echo "eti_jsonl=${ETI_JSONL}"
  echo "ltick_trace_jsonl=${LTICK_TRACE_JSONL}"
  echo "validator_rc=${VALIDATOR_RC}"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "eti-dlt-binding: FAIL (${COUNT} violations)"
  exit 2
fi

echo "eti-dlt-binding: PASS"
exit 0

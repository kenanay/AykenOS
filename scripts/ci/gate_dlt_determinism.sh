#!/usr/bin/env bash
set -euo pipefail

# Author: Kenan AY

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_dlt_determinism.sh \
    --evidence-dir evidence/run-<id>/gates/dlt-determinism \
    --eti-evidence evidence/run-<id>/gates/eti

Exit codes:
  0: pass
  2: DLT determinism contract failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
ETI_EVIDENCE_DIR=""

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

if [[ -z "${EVIDENCE_DIR}" || -z "${ETI_EVIDENCE_DIR}" ]]; then
  usage
  exit 3
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: missing required tool: python3" >&2
  exit 3
fi

VALIDATOR="${ROOT}/tools/ci/validate_dlt_determinism.py"
if [[ ! -f "${VALIDATOR}" ]]; then
  echo "ERROR: missing validator: ${VALIDATOR}" >&2
  exit 3
fi

ETI_JSONL="${ETI_EVIDENCE_DIR}/eti_transcript.jsonl"
if [[ ! -s "${ETI_JSONL}" ]]; then
  echo "ERROR: missing_or_empty:${ETI_JSONL}" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

LTICK_TRACE_A_JSONL="${EVIDENCE_DIR}/ltick_trace_a.jsonl"
LTICK_TRACE_B_JSONL="${EVIDENCE_DIR}/ltick_trace_b.jsonl"
DETERMINISM_REPORT_JSON="${EVIDENCE_DIR}/dlt_determinism_report.json"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

set +e
python3 "${VALIDATOR}" \
  --eti-jsonl "${ETI_JSONL}" \
  --out-ltick-trace-a "${LTICK_TRACE_A_JSONL}" \
  --out-ltick-trace-b "${LTICK_TRACE_B_JSONL}" \
  --out-determinism-report "${DETERMINISM_REPORT_JSON}" \
  --out-report "${REPORT_JSON}"
VALIDATOR_RC=$?
set -e

if [[ ! -f "${REPORT_JSON}" ]]; then
  echo "ERROR: validator did not produce report: ${REPORT_JSON}" >&2
  exit 3
fi
if [[ ! -f "${DETERMINISM_REPORT_JSON}" ]]; then
  echo "ERROR: validator did not produce determinism report: ${DETERMINISM_REPORT_JSON}" >&2
  exit 3
fi
if [[ ! -f "${LTICK_TRACE_A_JSONL}" ]]; then
  echo "ERROR: validator did not produce ltick trace a: ${LTICK_TRACE_A_JSONL}" >&2
  exit 3
fi
if [[ ! -f "${LTICK_TRACE_B_JSONL}" ]]; then
  echo "ERROR: validator did not produce ltick trace b: ${LTICK_TRACE_B_JSONL}" >&2
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
  echo "validator_rc=${VALIDATOR_RC}"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "dlt-determinism: FAIL (${COUNT} violations)"
  exit 2
fi

echo "dlt-determinism: PASS"
exit 0

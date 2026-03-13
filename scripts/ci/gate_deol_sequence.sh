#!/usr/bin/env bash
set -euo pipefail

# Author: Kenan AY

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_deol_sequence.sh \
    --evidence-dir evidence/run-<id>/gates/deol-sequence \
    --ledger-evidence evidence/run-<id>/gates/ledger-v1

Exit codes:
  0: pass
  2: DEOL ordering contract failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
LEDGER_EVIDENCE_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --ledger-evidence)
      LEDGER_EVIDENCE_DIR="$2"
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

if [[ -z "${EVIDENCE_DIR}" || -z "${LEDGER_EVIDENCE_DIR}" ]]; then
  usage
  exit 3
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: missing required tool: python3" >&2
  exit 3
fi

VALIDATOR="${ROOT}/tools/ci/validate_deol_sequence.py"
if [[ ! -f "${VALIDATOR}" ]]; then
  echo "ERROR: missing validator: ${VALIDATOR}" >&2
  exit 3
fi

LEDGER_JSONL="${LEDGER_EVIDENCE_DIR}/decision_ledger.jsonl"

mkdir -p "${EVIDENCE_DIR}"

EVENT_SEQ_JSONL="${EVIDENCE_DIR}/event_seq.jsonl"
SEQUENCE_REPORT_JSON="${EVIDENCE_DIR}/sequence_report.json"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

set +e
python3 "${VALIDATOR}" \
  --ledger-jsonl "${LEDGER_JSONL}" \
  --out-event-seq "${EVENT_SEQ_JSONL}" \
  --out-sequence-report "${SEQUENCE_REPORT_JSON}" \
  --out-report "${REPORT_JSON}"
VALIDATOR_RC=$?
set -e

if [[ ! -f "${REPORT_JSON}" ]]; then
  echo "ERROR: validator did not produce report: ${REPORT_JSON}" >&2
  exit 3
fi
if [[ ! -f "${SEQUENCE_REPORT_JSON}" ]]; then
  echo "ERROR: validator did not produce sequence_report.json: ${SEQUENCE_REPORT_JSON}" >&2
  exit 3
fi
if [[ ! -f "${EVENT_SEQ_JSONL}" ]]; then
  echo "ERROR: validator did not produce event_seq.jsonl: ${EVENT_SEQ_JSONL}" >&2
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
  echo "ledger_jsonl=${LEDGER_JSONL}"
  echo "validator_rc=${VALIDATOR_RC}"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "deol-sequence: FAIL (${COUNT} violations)"
  exit 2
fi

echo "deol-sequence: PASS"
exit 0

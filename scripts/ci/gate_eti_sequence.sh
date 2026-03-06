#!/usr/bin/env bash
set -euo pipefail

# Author: Kenan AY

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_eti_sequence.sh \
    --evidence-dir evidence/run-<id>/gates/eti \
    --phase10a2-evidence evidence/run-<id>/gates/ring3-execution-phase10a2

Exit codes:
  0: pass
  2: ETI sequence contract failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
A2_EVIDENCE_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --phase10a2-evidence)
      A2_EVIDENCE_DIR="$2"
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

if [[ -z "${EVIDENCE_DIR}" || -z "${A2_EVIDENCE_DIR}" ]]; then
  usage
  exit 3
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: missing required tool: python3" >&2
  exit 3
fi

VALIDATOR="${ROOT}/tools/ci/validate_eti_sequence.py"
if [[ ! -f "${VALIDATOR}" ]]; then
  echo "ERROR: missing validator: ${VALIDATOR}" >&2
  exit 3
fi

EVENTS_JSONL="${A2_EVIDENCE_DIR}/events.jsonl"
if [[ ! -s "${EVENTS_JSONL}" ]]; then
  echo "ERROR: missing_or_empty_events:${EVENTS_JSONL}" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

ETI_JSONL="${EVIDENCE_DIR}/eti_transcript.jsonl"
ETI_BIN="${EVIDENCE_DIR}/eti_transcript.bin"
CHAIN_VERIFY_JSON="${EVIDENCE_DIR}/eti_chain_verify.json"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
ETI_DIFF_TXT="${EVIDENCE_DIR}/eti_diff.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

set +e
python3 "${VALIDATOR}" \
  --events "${EVENTS_JSONL}" \
  --out-eti-jsonl "${ETI_JSONL}" \
  --out-eti-bin "${ETI_BIN}" \
  --out-chain-verify "${CHAIN_VERIFY_JSON}" \
  --out-report "${REPORT_JSON}"
VALIDATOR_RC=$?
set -e

if [[ ! -f "${REPORT_JSON}" ]]; then
  echo "ERROR: validator did not produce report: ${REPORT_JSON}" >&2
  exit 3
fi
if [[ ! -f "${ETI_JSONL}" ]]; then
  echo "ERROR: validator did not produce eti transcript jsonl: ${ETI_JSONL}" >&2
  exit 3
fi
if [[ ! -f "${ETI_BIN}" ]]; then
  echo "ERROR: validator did not produce eti transcript bin: ${ETI_BIN}" >&2
  exit 3
fi
if [[ ! -f "${CHAIN_VERIFY_JSON}" ]]; then
  echo "ERROR: validator did not produce chain verify: ${CHAIN_VERIFY_JSON}" >&2
  exit 3
fi

python3 - "${REPORT_JSON}" "${VIOLATIONS_TXT}" "${ETI_DIFF_TXT}" <<'PY'
import json
import sys

report_path, violations_path, diff_path = sys.argv[1:4]
with open(report_path, "r", encoding="utf-8") as fh:
    report = json.load(fh)
with open(violations_path, "w", encoding="utf-8") as fh:
    for violation in report.get("violations", []):
        fh.write(f"{violation}\n")
with open(diff_path, "w", encoding="utf-8") as fh:
    # Bootstrap gate emits this artifact for parity with issue evidence contract.
    for violation in report.get("violations", []):
        fh.write(f"{violation}\n")
PY

{
  echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "events_jsonl=${EVENTS_JSONL}"
  echo "validator_rc=${VALIDATOR_RC}"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "eti-sequence: FAIL (${COUNT} violations)"
  exit 2
fi

echo "eti-sequence: PASS"
exit 0

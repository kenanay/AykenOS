#!/usr/bin/env bash
set -euo pipefail

# Author: Kenan AY

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_replay_determinism.sh \
    --evidence-dir evidence/run-<id>/gates/replay-v1 \
    --abdf-evidence evidence/run-<id>/gates/abdf-snapshot-identity \
    --execution-evidence evidence/run-<id>/gates/execution-identity \
    [--expected-final-state-hash-file <path>]

Exit codes:
  0: pass
  2: replay determinism contract failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
ABDF_EVIDENCE_DIR=""
EXECUTION_EVIDENCE_DIR=""
EXPECTED_FINAL_STATE_HASH_FILE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --abdf-evidence)
      ABDF_EVIDENCE_DIR="$2"
      shift 2
      ;;
    --execution-evidence)
      EXECUTION_EVIDENCE_DIR="$2"
      shift 2
      ;;
    --expected-final-state-hash-file)
      EXPECTED_FINAL_STATE_HASH_FILE="$2"
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

if [[ -z "${EVIDENCE_DIR}" || -z "${ABDF_EVIDENCE_DIR}" || -z "${EXECUTION_EVIDENCE_DIR}" ]]; then
  usage
  exit 3
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: missing required tool: python3" >&2
  exit 3
fi

VALIDATOR="${ROOT}/tools/ci/validate_replay_determinism.py"
if [[ ! -f "${VALIDATOR}" ]]; then
  echo "ERROR: missing validator: ${VALIDATOR}" >&2
  exit 3
fi

ABDF_HASH_FILE="${ABDF_EVIDENCE_DIR}/abdf_snapshot_hash.txt"
BCIB_PLAN_HASH_FILE="${EXECUTION_EVIDENCE_DIR}/bcib_plan_hash.txt"
RECORD_TRACE_JSONL="${EXECUTION_EVIDENCE_DIR}/execution_trace.jsonl"
RECORD_TRACE_HASH_FILE="${EXECUTION_EVIDENCE_DIR}/execution_trace_hash.txt"

if [[ ! -s "${ABDF_HASH_FILE}" ]]; then
  echo "ERROR: missing_or_empty:${ABDF_HASH_FILE}" >&2
  exit 3
fi
if [[ ! -s "${BCIB_PLAN_HASH_FILE}" ]]; then
  echo "ERROR: missing_or_empty:${BCIB_PLAN_HASH_FILE}" >&2
  exit 3
fi
if [[ ! -s "${RECORD_TRACE_JSONL}" ]]; then
  echo "ERROR: missing_or_empty:${RECORD_TRACE_JSONL}" >&2
  exit 3
fi
if [[ ! -s "${RECORD_TRACE_HASH_FILE}" ]]; then
  echo "ERROR: missing_or_empty:${RECORD_TRACE_HASH_FILE}" >&2
  exit 3
fi
if [[ -n "${EXPECTED_FINAL_STATE_HASH_FILE}" && ! -s "${EXPECTED_FINAL_STATE_HASH_FILE}" ]]; then
  echo "ERROR: missing_or_empty:${EXPECTED_FINAL_STATE_HASH_FILE}" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

REPLAY_TRACE_JSONL="${EVIDENCE_DIR}/replay_trace.jsonl"
REPLAY_TRACE_HASH_TXT="${EVIDENCE_DIR}/replay_trace_hash.txt"
REPLAY_REPORT_JSON="${EVIDENCE_DIR}/replay_report.json"
EVENT_DIFF_TXT="${EVIDENCE_DIR}/event_diff.txt"
LTICK_DIFF_TXT="${EVIDENCE_DIR}/ltick_diff.txt"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

VALIDATOR_ARGS=(
  --abdf-hash-file "${ABDF_HASH_FILE}"
  --bcib-plan-hash-file "${BCIB_PLAN_HASH_FILE}"
  --record-trace-jsonl "${RECORD_TRACE_JSONL}"
  --record-trace-hash-file "${RECORD_TRACE_HASH_FILE}"
  --out-replay-trace-jsonl "${REPLAY_TRACE_JSONL}"
  --out-replay-trace-hash-txt "${REPLAY_TRACE_HASH_TXT}"
  --out-replay-report "${REPLAY_REPORT_JSON}"
  --out-event-diff "${EVENT_DIFF_TXT}"
  --out-ltick-diff "${LTICK_DIFF_TXT}"
  --out-report "${REPORT_JSON}"
)
if [[ -n "${EXPECTED_FINAL_STATE_HASH_FILE}" ]]; then
  VALIDATOR_ARGS+=(--expected-final-state-hash-file "${EXPECTED_FINAL_STATE_HASH_FILE}")
fi

set +e
python3 "${VALIDATOR}" "${VALIDATOR_ARGS[@]}"
VALIDATOR_RC=$?
set -e

if [[ ! -f "${REPORT_JSON}" ]]; then
  echo "ERROR: validator did not produce report: ${REPORT_JSON}" >&2
  exit 3
fi
if [[ ! -f "${REPLAY_REPORT_JSON}" ]]; then
  echo "ERROR: validator did not produce replay report: ${REPLAY_REPORT_JSON}" >&2
  exit 3
fi
if [[ ! -f "${REPLAY_TRACE_JSONL}" ]]; then
  echo "ERROR: validator did not produce replay trace: ${REPLAY_TRACE_JSONL}" >&2
  exit 3
fi
if [[ ! -f "${REPLAY_TRACE_HASH_TXT}" ]]; then
  echo "ERROR: validator did not produce replay trace hash: ${REPLAY_TRACE_HASH_TXT}" >&2
  exit 3
fi
if [[ ! -f "${EVENT_DIFF_TXT}" ]]; then
  echo "ERROR: validator did not produce event diff: ${EVENT_DIFF_TXT}" >&2
  exit 3
fi
if [[ ! -f "${LTICK_DIFF_TXT}" ]]; then
  echo "ERROR: validator did not produce ltick diff: ${LTICK_DIFF_TXT}" >&2
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
  echo "abdf_hash_file=${ABDF_HASH_FILE}"
  echo "bcib_plan_hash_file=${BCIB_PLAN_HASH_FILE}"
  echo "record_trace_jsonl=${RECORD_TRACE_JSONL}"
  echo "record_trace_hash_file=${RECORD_TRACE_HASH_FILE}"
  echo "expected_final_state_hash_file=${EXPECTED_FINAL_STATE_HASH_FILE}"
  echo "validator_rc=${VALIDATOR_RC}"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "replay-determinism: FAIL (${COUNT} violations)"
  exit 2
fi

echo "replay-determinism: PASS"
exit 0

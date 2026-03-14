#!/usr/bin/env bash
set -euo pipefail

# Author: Kenan AY

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_bcib_trace_identity.sh \
    --evidence-dir evidence/run-<id>/gates/execution-identity \
    --bcib-plan evidence/run-<id>/execution/plan.bcib \
    --eti-evidence evidence/run-<id>/gates/eti \
    [--expected-plan-hash-file <path>] \
    [--expected-trace-hash-file <path>]

Exit codes:
  0: pass
  2: BCIB/trace identity contract failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
BCIB_PLAN_BIN=""
ETI_EVIDENCE_DIR=""
EXPECTED_PLAN_HASH_FILE=""
EXPECTED_TRACE_HASH_FILE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --bcib-plan)
      BCIB_PLAN_BIN="$2"
      shift 2
      ;;
    --eti-evidence)
      ETI_EVIDENCE_DIR="$2"
      shift 2
      ;;
    --expected-plan-hash-file)
      EXPECTED_PLAN_HASH_FILE="$2"
      shift 2
      ;;
    --expected-trace-hash-file)
      EXPECTED_TRACE_HASH_FILE="$2"
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

if [[ -z "${EVIDENCE_DIR}" || -z "${BCIB_PLAN_BIN}" || -z "${ETI_EVIDENCE_DIR}" ]]; then
  usage
  exit 3
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: missing required tool: python3" >&2
  exit 3
fi

VALIDATOR="${ROOT}/tools/ci/validate_bcib_trace_identity.py"
if [[ ! -f "${VALIDATOR}" ]]; then
  echo "ERROR: missing validator: ${VALIDATOR}" >&2
  exit 3
fi

ETI_JSONL="${ETI_EVIDENCE_DIR}/eti_transcript.jsonl"
if [[ ! -s "${BCIB_PLAN_BIN}" ]]; then
  echo "ERROR: missing_or_empty:${BCIB_PLAN_BIN}" >&2
  exit 3
fi
if [[ ! -s "${ETI_JSONL}" ]]; then
  echo "ERROR: missing_or_empty:${ETI_JSONL}" >&2
  exit 3
fi
if [[ -n "${EXPECTED_PLAN_HASH_FILE}" && ! -s "${EXPECTED_PLAN_HASH_FILE}" ]]; then
  echo "ERROR: missing_or_empty:${EXPECTED_PLAN_HASH_FILE}" >&2
  exit 3
fi
if [[ -n "${EXPECTED_TRACE_HASH_FILE}" && ! -s "${EXPECTED_TRACE_HASH_FILE}" ]]; then
  echo "ERROR: missing_or_empty:${EXPECTED_TRACE_HASH_FILE}" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

PLAN_HASH_TXT="${EVIDENCE_DIR}/bcib_plan_hash.txt"
TRACE_JSONL="${EVIDENCE_DIR}/execution_trace.jsonl"
TRACE_HASH_TXT="${EVIDENCE_DIR}/execution_trace_hash.txt"
TRACE_VERIFY_JSON="${EVIDENCE_DIR}/trace_verify.json"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

VALIDATOR_ARGS=(
  --bcib-plan-bin "${BCIB_PLAN_BIN}"
  --eti-jsonl "${ETI_JSONL}"
  --out-plan-hash-txt "${PLAN_HASH_TXT}"
  --out-execution-trace-jsonl "${TRACE_JSONL}"
  --out-execution-trace-hash-txt "${TRACE_HASH_TXT}"
  --out-trace-verify-json "${TRACE_VERIFY_JSON}"
  --out-report "${REPORT_JSON}"
)
if [[ -n "${EXPECTED_PLAN_HASH_FILE}" ]]; then
  VALIDATOR_ARGS+=(--expected-plan-hash-file "${EXPECTED_PLAN_HASH_FILE}")
fi
if [[ -n "${EXPECTED_TRACE_HASH_FILE}" ]]; then
  VALIDATOR_ARGS+=(--expected-trace-hash-file "${EXPECTED_TRACE_HASH_FILE}")
fi

set +e
python3 "${VALIDATOR}" "${VALIDATOR_ARGS[@]}"
VALIDATOR_RC=$?
set -e

if [[ ! -f "${REPORT_JSON}" ]]; then
  echo "ERROR: validator did not produce report: ${REPORT_JSON}" >&2
  exit 3
fi
if [[ ! -f "${PLAN_HASH_TXT}" ]]; then
  echo "ERROR: validator did not produce plan hash: ${PLAN_HASH_TXT}" >&2
  exit 3
fi
if [[ ! -f "${TRACE_JSONL}" ]]; then
  echo "ERROR: validator did not produce execution trace: ${TRACE_JSONL}" >&2
  exit 3
fi
if [[ ! -f "${TRACE_HASH_TXT}" ]]; then
  echo "ERROR: validator did not produce trace hash: ${TRACE_HASH_TXT}" >&2
  exit 3
fi
if [[ ! -f "${TRACE_VERIFY_JSON}" ]]; then
  echo "ERROR: validator did not produce trace verify report: ${TRACE_VERIFY_JSON}" >&2
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
  echo "bcib_plan_bin=${BCIB_PLAN_BIN}"
  echo "eti_jsonl=${ETI_JSONL}"
  echo "expected_plan_hash_file=${EXPECTED_PLAN_HASH_FILE}"
  echo "expected_trace_hash_file=${EXPECTED_TRACE_HASH_FILE}"
  echo "validator_rc=${VALIDATOR_RC}"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "bcib-trace-identity: FAIL (${COUNT} violations)"
  exit 2
fi

echo "bcib-trace-identity: PASS"
exit 0

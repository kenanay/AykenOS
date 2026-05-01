#!/usr/bin/env bash
set -euo pipefail

# Author: Kenan AY

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_bcib_determinism.sh \
    --evidence-dir evidence/run-<id>/gates/bcib-determinism \
    --run-a-dir evidence/bcib-kernel-determinism/run-1 \
    --run-b-dir evidence/bcib-kernel-determinism/run-2

Exit codes:
  0: pass
  2: BCIB determinism contract failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
RUN_A_DIR=""
RUN_B_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --run-a-dir)
      RUN_A_DIR="$2"
      shift 2
      ;;
    --run-b-dir)
      RUN_B_DIR="$2"
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

if [[ -z "${EVIDENCE_DIR}" || -z "${RUN_A_DIR}" || -z "${RUN_B_DIR}" ]]; then
  usage
  exit 3
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: missing required tool: python3" >&2
  exit 3
fi

VALIDATOR="${ROOT}/tools/ci/validate_bcib_determinism.py"
if [[ ! -f "${VALIDATOR}" ]]; then
  echo "ERROR: missing validator: ${VALIDATOR}" >&2
  exit 3
fi
if [[ ! -d "${RUN_A_DIR}" ]]; then
  echo "ERROR: missing run dir: ${RUN_A_DIR}" >&2
  exit 3
fi
if [[ ! -d "${RUN_B_DIR}" ]]; then
  echo "ERROR: missing run dir: ${RUN_B_DIR}" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

RUN_A_JSON="${EVIDENCE_DIR}/bcib_determinism_run_1.json"
RUN_B_JSON="${EVIDENCE_DIR}/bcib_determinism_run_2.json"
TRACE_RUN_A="${EVIDENCE_DIR}/bcib_determinism_trace_run_1.log"
TRACE_RUN_B="${EVIDENCE_DIR}/bcib_determinism_trace_run_2.log"
RESULT_BIN="${EVIDENCE_DIR}/result.bin"
RESULT_SHA256="${EVIDENCE_DIR}/result.sha256"
RESULT_METADATA="${EVIDENCE_DIR}/result_metadata.json"
COMPARISON_LOG="${EVIDENCE_DIR}/result_sha256_comparison.log"
DETERMINISM_EVIDENCE="${EVIDENCE_DIR}/bcib_kernel_determinism_evidence.json"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

set +e
python3 "${VALIDATOR}" \
  --run-a-dir "${RUN_A_DIR}" \
  --run-b-dir "${RUN_B_DIR}" \
  --out-run-a-json "${RUN_A_JSON}" \
  --out-run-b-json "${RUN_B_JSON}" \
  --out-trace-run-a "${TRACE_RUN_A}" \
  --out-trace-run-b "${TRACE_RUN_B}" \
  --out-result-bin "${RESULT_BIN}" \
  --out-result-sha256 "${RESULT_SHA256}" \
  --out-result-metadata "${RESULT_METADATA}" \
  --out-comparison-log "${COMPARISON_LOG}" \
  --out-determinism-evidence "${DETERMINISM_EVIDENCE}" \
  --out-report "${REPORT_JSON}"
VALIDATOR_RC=$?
set -e

for required in \
  "${RUN_A_JSON}" \
  "${RUN_B_JSON}" \
  "${TRACE_RUN_A}" \
  "${TRACE_RUN_B}" \
  "${RESULT_BIN}" \
  "${RESULT_SHA256}" \
  "${RESULT_METADATA}" \
  "${COMPARISON_LOG}" \
  "${DETERMINISM_EVIDENCE}" \
  "${REPORT_JSON}"
do
  if [[ ! -f "${required}" ]]; then
    echo "ERROR: validator did not produce required output: ${required}" >&2
    exit 3
  fi
done

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
  echo "run_a_dir=${RUN_A_DIR}"
  echo "run_b_dir=${RUN_B_DIR}"
  echo "validator_rc=${VALIDATOR_RC}"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "bcib-determinism: FAIL (${COUNT} violations)"
  exit 2
fi

echo "bcib-determinism: PASS"
echo "DETERMINISM_PASS"
exit 0

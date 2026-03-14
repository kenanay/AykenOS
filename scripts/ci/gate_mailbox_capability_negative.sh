#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_mailbox_capability_negative.sh \
    --evidence-dir evidence/run-<id>/gates/mailbox-cap

Exit codes:
  0: pass
  2: mailbox capability negative contract failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
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
  echo "ERROR: missing required tool: python3" >&2
  exit 3
fi

VALIDATOR="${ROOT}/tools/ci/validate_mailbox_capability_negative.py"
HEADER="${ROOT}/kernel/include/sched_mailbox_abi.h"
SOURCE="${ROOT}/kernel/sched/sched_mailbox.c"

if [[ ! -f "${VALIDATOR}" ]]; then
  echo "ERROR: missing validator: ${VALIDATOR}" >&2
  exit 3
fi
if [[ ! -f "${HEADER}" ]]; then
  echo "ERROR: missing header: ${HEADER}" >&2
  exit 3
fi
if [[ ! -f "${SOURCE}" ]]; then
  echo "ERROR: missing source: ${SOURCE}" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

REPORT_JSON="${EVIDENCE_DIR}/report.json"
NEGATIVE_MATRIX_JSON="${EVIDENCE_DIR}/negative_matrix.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

set +e
python3 "${VALIDATOR}" \
  --header "${HEADER}" \
  --source "${SOURCE}" \
  --out-report "${REPORT_JSON}" \
  --out-matrix "${NEGATIVE_MATRIX_JSON}"
VALIDATOR_RC=$?
set -e

if [[ ! -f "${REPORT_JSON}" ]]; then
  echo "ERROR: validator did not produce report: ${REPORT_JSON}" >&2
  exit 3
fi
if [[ ! -f "${NEGATIVE_MATRIX_JSON}" ]]; then
  echo "ERROR: validator did not produce matrix: ${NEGATIVE_MATRIX_JSON}" >&2
  exit 3
fi

python3 - "${REPORT_JSON}" "${VIOLATIONS_TXT}" <<'PY'
import json
import sys

report_path, violations_path = sys.argv[1:3]
with open(report_path, "r", encoding="utf-8") as fh:
    payload = json.load(fh)
with open(violations_path, "w", encoding="utf-8") as fh:
    for row in payload.get("violations", []):
        fh.write(f"{row}\n")
PY

{
  echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "header=${HEADER}"
  echo "source=${SOURCE}"
  echo "validator_rc=${VALIDATOR_RC}"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "mailbox-capability-negative: FAIL (${COUNT} violations)"
  exit 2
fi

echo "mailbox-capability-negative: PASS"
exit 0

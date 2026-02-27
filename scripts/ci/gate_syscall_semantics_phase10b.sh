#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_syscall_semantics_phase10b.sh \
    --evidence-dir evidence/run-<id>/gates/syscall-semantics-phase10b \
    --phase10a2-evidence evidence/run-<id>/gates/ring3-execution-phase10a2 \
    [--mode positive|negative]

Exit codes:
  0: pass
  2: semantic contract failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
A2_EVIDENCE_DIR=""
MODE="${PHASE10B_MODE:-negative}"

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
    --mode)
      MODE="$2"
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
if [[ "${MODE}" != "positive" && "${MODE}" != "negative" ]]; then
  echo "ERROR: --mode must be positive or negative" >&2
  exit 3
fi
for tool in python3; do
  if ! command -v "${tool}" >/dev/null 2>&1; then
    echo "ERROR: missing required tool: ${tool}" >&2
    exit 3
  fi
done

VALIDATOR="${ROOT}/tools/ci/validate_syscall_semantics_phase10b.py"
if [[ ! -f "${VALIDATOR}" ]]; then
  echo "ERROR: missing validator: ${VALIDATOR}" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

A2_EVENTS="${A2_EVIDENCE_DIR}/events.jsonl"
A2_MARKER_LOG="${A2_EVIDENCE_DIR}/marker.log"

REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

: > "${VIOLATIONS_TXT}"
: > "${META_TXT}"

if [[ ! -s "${A2_EVENTS}" ]]; then
  echo "missing_or_empty_events:${A2_EVENTS}" > "${VIOLATIONS_TXT}"
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "syscall-semantics-phase10b",
  "mode": "${MODE}",
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["missing_or_empty_events"]
}
EOF
  {
    echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "mode=${MODE}"
    echo "phase10a2_events=${A2_EVENTS}"
    echo "phase10a2_marker_log=${A2_MARKER_LOG}"
    echo "validator_rc=2"
  } > "${META_TXT}"
  echo "syscall-semantics-phase10b: FAIL (missing_or_empty_events)"
  exit 2
fi

if [[ ! -s "${A2_MARKER_LOG}" ]]; then
  echo "missing_or_empty_marker_log:${A2_MARKER_LOG}" > "${VIOLATIONS_TXT}"
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "syscall-semantics-phase10b",
  "mode": "${MODE}",
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["missing_or_empty_marker_log"]
}
EOF
  {
    echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "mode=${MODE}"
    echo "phase10a2_events=${A2_EVENTS}"
    echo "phase10a2_marker_log=${A2_MARKER_LOG}"
    echo "validator_rc=2"
  } > "${META_TXT}"
  echo "syscall-semantics-phase10b: FAIL (missing_or_empty_marker_log)"
  exit 2
fi

set +e
python3 "${VALIDATOR}" \
  --events "${A2_EVENTS}" \
  --log "${A2_MARKER_LOG}" \
  --mode "${MODE}" \
  --out "${REPORT_JSON}"
VALIDATOR_RC=$?
set -e

python3 - "${REPORT_JSON}" "${A2_EVENTS}" "${A2_MARKER_LOG}" <<'PY'
import json
import sys

path, events_path, marker_log_path = sys.argv[1:4]
with open(path, "r", encoding="utf-8") as fh:
    row = json.load(fh)
row["phase10a2_events"] = events_path
row["phase10a2_marker_log"] = marker_log_path
with open(path, "w", encoding="utf-8") as fh:
    json.dump(row, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY

python3 - "${REPORT_JSON}" "${VIOLATIONS_TXT}" <<'PY'
import json
import sys

report_path, violations_path = sys.argv[1:3]
with open(report_path, "r", encoding="utf-8") as fh:
    row = json.load(fh)
violations = row.get("violations", [])
with open(violations_path, "w", encoding="utf-8") as fh:
    for item in violations:
        fh.write(f"{item}\n")
PY

{
  echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "mode=${MODE}"
  echo "phase10a2_events=${A2_EVENTS}"
  echo "phase10a2_marker_log=${A2_MARKER_LOG}"
  echo "validator_rc=${VALIDATOR_RC}"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "syscall-semantics-phase10b: FAIL (${COUNT} violations)"
  exit 2
fi

echo "syscall-semantics-phase10b: PASS (mode=${MODE})"
exit 0

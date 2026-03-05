#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_scheduler_mailbox_phase10c.sh \
    --evidence-dir evidence/run-<id>/gates/scheduler-mailbox-phase10c \
    --phase10a2-evidence evidence/run-<id>/gates/ring3-execution-phase10a2 \
    [--require-metadata 0|1] \
    [--c2-strict 0|1] \
    [--c2-owner-set <csv-pids>] \
    [--c2-require-cursor-marker 0|1]

Exit codes:
  0: pass
  2: scheduler mailbox semantic contract failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
A2_EVIDENCE_DIR=""
REQUIRE_METADATA="${PHASE10C_REQUIRE_METADATA:-1}"
C2_STRICT="${PHASE10C_C2_STRICT:-0}"
C2_OWNER_SET="${PHASE10C_C2_OWNER_SET:-2}"
C2_REQUIRE_CURSOR_MARKER="${PHASE10C_C2_REQUIRE_CURSOR_MARKER:-1}"

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
    --require-metadata)
      REQUIRE_METADATA="$2"
      shift 2
      ;;
    --c2-strict)
      C2_STRICT="$2"
      shift 2
      ;;
    --c2-owner-set)
      C2_OWNER_SET="$2"
      shift 2
      ;;
    --c2-require-cursor-marker)
      C2_REQUIRE_CURSOR_MARKER="$2"
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
if ! [[ "${REQUIRE_METADATA}" =~ ^[01]$ ]]; then
  echo "ERROR: --require-metadata must be 0 or 1 (current=${REQUIRE_METADATA})" >&2
  exit 3
fi
if ! [[ "${C2_STRICT}" =~ ^[01]$ ]]; then
  echo "ERROR: --c2-strict must be 0 or 1 (current=${C2_STRICT})" >&2
  exit 3
fi
if ! [[ "${C2_REQUIRE_CURSOR_MARKER}" =~ ^[01]$ ]]; then
  echo "ERROR: --c2-require-cursor-marker must be 0 or 1 (current=${C2_REQUIRE_CURSOR_MARKER})" >&2
  exit 3
fi
if [[ "${C2_STRICT}" == "1" ]]; then
  if [[ -z "${C2_OWNER_SET}" ]]; then
    echo "ERROR: --c2-owner-set is required when --c2-strict=1" >&2
    exit 3
  fi
  if ! [[ "${C2_OWNER_SET}" =~ ^[0-9]+(,[0-9]+)*$ ]]; then
    echo "ERROR: --c2-owner-set must be CSV positive integers (current=${C2_OWNER_SET})" >&2
    exit 3
  fi
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: missing required tool: python3" >&2
  exit 3
fi

VALIDATOR="${ROOT}/tools/ci/validate_scheduler_mailbox_phase10c.py"
SCHED_SOURCE="${ROOT}/kernel/sched/sched.c"
TIMER_SOURCE="${ROOT}/kernel/arch/x86_64/timer.c"
if [[ ! -f "${VALIDATOR}" ]]; then
  echo "ERROR: missing validator: ${VALIDATOR}" >&2
  exit 3
fi
if [[ ! -f "${SCHED_SOURCE}" ]]; then
  echo "ERROR: missing scheduler source: ${SCHED_SOURCE}" >&2
  exit 3
fi
if [[ ! -f "${TIMER_SOURCE}" ]]; then
  echo "ERROR: missing timer source: ${TIMER_SOURCE}" >&2
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
  "gate": "scheduler-mailbox-phase10c",
  "require_metadata": ${REQUIRE_METADATA},
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["missing_or_empty_events"]
}
EOF
{
  echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "phase10a2_events=${A2_EVENTS}"
  echo "phase10a2_marker_log=${A2_MARKER_LOG}"
  echo "require_metadata=${REQUIRE_METADATA}"
  echo "c2_strict=${C2_STRICT}"
  echo "c2_owner_set=${C2_OWNER_SET}"
  echo "c2_require_cursor_marker=${C2_REQUIRE_CURSOR_MARKER}"
  echo "validator_rc=2"
} > "${META_TXT}"
  echo "scheduler-mailbox-phase10c: FAIL (missing_or_empty_events)"
  exit 2
fi

if [[ ! -s "${A2_MARKER_LOG}" ]]; then
  echo "missing_or_empty_marker_log:${A2_MARKER_LOG}" > "${VIOLATIONS_TXT}"
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "scheduler-mailbox-phase10c",
  "require_metadata": ${REQUIRE_METADATA},
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["missing_or_empty_marker_log"]
}
EOF
{
  echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "phase10a2_events=${A2_EVENTS}"
  echo "phase10a2_marker_log=${A2_MARKER_LOG}"
  echo "require_metadata=${REQUIRE_METADATA}"
  echo "c2_strict=${C2_STRICT}"
  echo "c2_owner_set=${C2_OWNER_SET}"
  echo "c2_require_cursor_marker=${C2_REQUIRE_CURSOR_MARKER}"
  echo "validator_rc=2"
} > "${META_TXT}"
  echo "scheduler-mailbox-phase10c: FAIL (missing_or_empty_marker_log)"
  exit 2
fi

set +e
python3 "${VALIDATOR}" \
  --events "${A2_EVENTS}" \
  --log "${A2_MARKER_LOG}" \
  --require-metadata "${REQUIRE_METADATA}" \
  --c2-strict "${C2_STRICT}" \
  --c2-owner-set "${C2_OWNER_SET}" \
  --c2-require-cursor-marker "${C2_REQUIRE_CURSOR_MARKER}" \
  --sched-source "${SCHED_SOURCE}" \
  --timer-source "${TIMER_SOURCE}" \
  --out "${REPORT_JSON}"
VALIDATOR_RC=$?
set -e

python3 - "${REPORT_JSON}" "${A2_EVENTS}" "${A2_MARKER_LOG}" "${REQUIRE_METADATA}" "${C2_STRICT}" "${C2_OWNER_SET}" "${C2_REQUIRE_CURSOR_MARKER}" <<'PY'
import json
import sys

path, events_path, marker_log_path, require_metadata, c2_strict, c2_owner_set, c2_require_cursor_marker = sys.argv[1:8]
with open(path, "r", encoding="utf-8") as fh:
    row = json.load(fh)
row["phase10a2_events"] = events_path
row["phase10a2_marker_log"] = marker_log_path
row["require_metadata"] = int(require_metadata)
row["c2_strict"] = int(c2_strict)
row["c2_owner_set"] = c2_owner_set
row["c2_require_cursor_marker"] = int(c2_require_cursor_marker)
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
  echo "phase10a2_events=${A2_EVENTS}"
  echo "phase10a2_marker_log=${A2_MARKER_LOG}"
  echo "require_metadata=${REQUIRE_METADATA}"
  echo "c2_strict=${C2_STRICT}"
  echo "c2_owner_set=${C2_OWNER_SET}"
  echo "c2_require_cursor_marker=${C2_REQUIRE_CURSOR_MARKER}"
  echo "validator_rc=${VALIDATOR_RC}"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "scheduler-mailbox-phase10c: FAIL (${COUNT} violations)"
  exit 2
fi

echo "scheduler-mailbox-phase10c: PASS"
exit 0

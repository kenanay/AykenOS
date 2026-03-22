#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_syscall_semantics_phase10b.sh \
    --evidence-dir evidence/run-<id>/gates/syscall-semantics-phase10b \
    --phase10a2-evidence evidence/run-<id>/gates/ring3-execution-phase10a2 \
    [--mode positive|negative] \
    [--proof-qemu-timeout <sec>] \
    [--require-colocated-phase10a2]

Exit codes:
  0: pass
  2: semantic contract failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
A2_EVIDENCE_DIR=""
MODE="${PHASE10B_MODE:-negative}"
PROOF_QEMU_TIMEOUT="${PHASE10B_PROOF_QEMU_TIMEOUT:-20}"
REQUIRE_COLOCATED_A2="${PHASE10B_REQUIRE_COLOCATED_A2:-0}"

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
    --proof-qemu-timeout)
      PROOF_QEMU_TIMEOUT="$2"
      shift 2
      ;;
    --require-colocated-phase10a2)
      REQUIRE_COLOCATED_A2="1"
      shift 1
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
if [[ "${REQUIRE_COLOCATED_A2}" != "0" && "${REQUIRE_COLOCATED_A2}" != "1" ]]; then
  echo "ERROR: colocated A2 requirement must be 0 or 1 (current=${REQUIRE_COLOCATED_A2})" >&2
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

SOURCE_GUARD="${ROOT}/scripts/ci/check_phase10b_execution_hardening.sh"
if [[ ! -f "${SOURCE_GUARD}" ]]; then
  echo "ERROR: missing source guard: ${SOURCE_GUARD}" >&2
  exit 3
fi

FAIL_CLOSED_PROOF_GATE="${ROOT}/scripts/ci/gate_execution_fail_closed_proof.sh"
if [[ ! -f "${FAIL_CLOSED_PROOF_GATE}" ]]; then
  echo "ERROR: missing fail-closed proof gate: ${FAIL_CLOSED_PROOF_GATE}" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

A2_EVENTS="${A2_EVIDENCE_DIR}/events.jsonl"
A2_MARKER_LOG="${A2_EVIDENCE_DIR}/marker.log"
SOURCE_GUARD_EVIDENCE_DIR="${EVIDENCE_DIR}/execution-hardening"
FAIL_CLOSED_PROOF_EVIDENCE_DIR="${EVIDENCE_DIR}/fail-closed-proof"
ACTUAL_A2_EVIDENCE_DIR="$(cd "${A2_EVIDENCE_DIR}" 2>/dev/null && pwd || true)"
EXPECTED_COLOCATED_A2_DIR="$(cd "$(dirname "${EVIDENCE_DIR}")" && pwd)/ring3-execution-phase10a2"
PHASE10A2_EVIDENCE_MODE="external_reused_review_evidence"
PHASE10A2_EVIDENCE_SAME_RUN_CANDIDATE=0

if [[ -n "${ACTUAL_A2_EVIDENCE_DIR}" && "${ACTUAL_A2_EVIDENCE_DIR}" == "${EXPECTED_COLOCATED_A2_DIR}" ]]; then
  PHASE10A2_EVIDENCE_MODE="co_located_same_run_candidate"
  PHASE10A2_EVIDENCE_SAME_RUN_CANDIDATE=1
fi

REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

: > "${VIOLATIONS_TXT}"
: > "${META_TXT}"

if [[ "${REQUIRE_COLOCATED_A2}" == "1" && "${PHASE10A2_EVIDENCE_SAME_RUN_CANDIDATE}" -ne 1 ]]; then
  echo "phase10a2_evidence_not_colocated:${A2_EVIDENCE_DIR}" > "${VIOLATIONS_TXT}"
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "syscall-semantics-phase10b",
  "mode": "${MODE}",
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["phase10a2_evidence_not_colocated"],
  "phase10a2_evidence_dir": "${A2_EVIDENCE_DIR}",
  "phase10a2_evidence_mode": "${PHASE10A2_EVIDENCE_MODE}",
  "phase10a2_evidence_same_run_candidate": ${PHASE10A2_EVIDENCE_SAME_RUN_CANDIDATE},
  "phase10a2_expected_colocated_dir": "${EXPECTED_COLOCATED_A2_DIR}",
  "phase10a2_colocated_requirement_enforced": true
}
EOF
  {
    echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "mode=${MODE}"
    echo "phase10a2_events=${A2_EVENTS}"
    echo "phase10a2_marker_log=${A2_MARKER_LOG}"
    echo "phase10a2_evidence_dir=${A2_EVIDENCE_DIR}"
    echo "phase10a2_evidence_mode=${PHASE10A2_EVIDENCE_MODE}"
    echo "phase10a2_evidence_same_run_candidate=${PHASE10A2_EVIDENCE_SAME_RUN_CANDIDATE}"
    echo "phase10a2_expected_colocated_dir=${EXPECTED_COLOCATED_A2_DIR}"
    echo "phase10a2_colocated_requirement_enforced=1"
    echo "validator_rc=2"
  } > "${META_TXT}"
  echo "syscall-semantics-phase10b: FAIL (phase10a2_evidence_not_colocated)"
  exit 2
fi

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
bash "${SOURCE_GUARD}" --evidence-dir "${SOURCE_GUARD_EVIDENCE_DIR}"
SOURCE_GUARD_RC=$?
set -e

if [[ "${SOURCE_GUARD_RC}" -ne 0 ]]; then
  echo "phase10b_execution_hardening_guard_failed:${SOURCE_GUARD_EVIDENCE_DIR}/report.json" > "${VIOLATIONS_TXT}"
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "syscall-semantics-phase10b",
  "mode": "${MODE}",
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["phase10b_execution_hardening_guard_failed"],
  "execution_hardening_report": "${SOURCE_GUARD_EVIDENCE_DIR}/report.json"
}
EOF
  {
    echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "mode=${MODE}"
    echo "phase10a2_events=${A2_EVENTS}"
    echo "phase10a2_marker_log=${A2_MARKER_LOG}"
    echo "source_guard_rc=${SOURCE_GUARD_RC}"
    echo "validator_rc=2"
  } > "${META_TXT}"
  echo "syscall-semantics-phase10b: FAIL (phase10b_execution_hardening_guard_failed)"
  exit 2
fi

set +e
bash "${FAIL_CLOSED_PROOF_GATE}" \
  --evidence-dir "${FAIL_CLOSED_PROOF_EVIDENCE_DIR}" \
  --qemu-timeout "${PROOF_QEMU_TIMEOUT}"
FAIL_CLOSED_PROOF_RC=$?
set -e

if [[ "${FAIL_CLOSED_PROOF_RC}" -ne 0 ]]; then
  echo "phase10b_fail_closed_proof_failed:${FAIL_CLOSED_PROOF_EVIDENCE_DIR}/report.json" > "${VIOLATIONS_TXT}"
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "syscall-semantics-phase10b",
  "mode": "${MODE}",
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["phase10b_fail_closed_proof_failed"],
  "execution_hardening_report": "${SOURCE_GUARD_EVIDENCE_DIR}/report.json",
  "fail_closed_proof_report": "${FAIL_CLOSED_PROOF_EVIDENCE_DIR}/report.json"
}
EOF
  {
    echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "mode=${MODE}"
    echo "phase10a2_events=${A2_EVENTS}"
    echo "phase10a2_marker_log=${A2_MARKER_LOG}"
    echo "source_guard_rc=${SOURCE_GUARD_RC}"
    echo "fail_closed_proof_rc=${FAIL_CLOSED_PROOF_RC}"
    echo "validator_rc=2"
  } > "${META_TXT}"
  echo "syscall-semantics-phase10b: FAIL (phase10b_fail_closed_proof_failed)"
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

python3 - "${REPORT_JSON}" "${A2_EVIDENCE_DIR}" "${PHASE10A2_EVIDENCE_MODE}" "${PHASE10A2_EVIDENCE_SAME_RUN_CANDIDATE}" "${EXPECTED_COLOCATED_A2_DIR}" "${REQUIRE_COLOCATED_A2}" <<'PY'
import json
import sys

report_path, evidence_dir, mode, same_run, expected_colocated_dir, enforced = sys.argv[1:7]
with open(report_path, "r", encoding="utf-8") as fh:
    row = json.load(fh)
row["phase10a2_evidence_dir"] = evidence_dir
row["phase10a2_evidence_mode"] = mode
row["phase10a2_evidence_same_run_candidate"] = int(same_run)
row["phase10a2_expected_colocated_dir"] = expected_colocated_dir
row["phase10a2_colocated_requirement_enforced"] = enforced == "1"
with open(report_path, "w", encoding="utf-8") as fh:
    json.dump(row, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY

python3 - "${REPORT_JSON}" "${SOURCE_GUARD_EVIDENCE_DIR}/report.json" <<'PY'
import json
import sys

report_path, source_guard_path = sys.argv[1:3]
with open(report_path, "r", encoding="utf-8") as fh:
    row = json.load(fh)
row["execution_hardening_report"] = source_guard_path
with open(report_path, "w", encoding="utf-8") as fh:
    json.dump(row, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY

python3 - "${REPORT_JSON}" "${FAIL_CLOSED_PROOF_EVIDENCE_DIR}/report.json" <<'PY'
import json
import sys

report_path, proof_path = sys.argv[1:3]
with open(report_path, "r", encoding="utf-8") as fh:
    row = json.load(fh)
row["fail_closed_proof_report"] = proof_path
with open(report_path, "w", encoding="utf-8") as fh:
    json.dump(row, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY

python3 - "${REPORT_JSON}" "${FAIL_CLOSED_PROOF_EVIDENCE_DIR}" <<'PY'
import json
import sys
from pathlib import Path

report_path, proof_dir = sys.argv[1:3]
proof_dir_path = Path(proof_dir)
with open(report_path, "r", encoding="utf-8") as fh:
    row = json.load(fh)
row["fail_closed_replay_trace_jsonl"] = str(proof_dir_path / "replay_trace.jsonl")
row["fail_closed_replay_trace_hash_file"] = str(proof_dir_path / "replay_trace_hash.txt")
row["fail_closed_replay_report"] = str(proof_dir_path / "replay_report.json")
row["fail_closed_replay_manifest"] = str(proof_dir_path / "replay_manifest.json")
row["fail_closed_final_state_hash_file"] = str(proof_dir_path / "final_state_hash.txt")
row["fail_closed_replay_result_hash_file"] = str(proof_dir_path / "replay_result_hash.txt")
with open(report_path, "w", encoding="utf-8") as fh:
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
  echo "phase10a2_evidence_dir=${A2_EVIDENCE_DIR}"
  echo "phase10a2_evidence_mode=${PHASE10A2_EVIDENCE_MODE}"
  echo "phase10a2_evidence_same_run_candidate=${PHASE10A2_EVIDENCE_SAME_RUN_CANDIDATE}"
  echo "phase10a2_expected_colocated_dir=${EXPECTED_COLOCATED_A2_DIR}"
  echo "phase10a2_colocated_requirement_enforced=${REQUIRE_COLOCATED_A2}"
  echo "source_guard_rc=${SOURCE_GUARD_RC}"
  echo "fail_closed_proof_rc=${FAIL_CLOSED_PROOF_RC}"
  echo "fail_closed_replay_trace_jsonl=${FAIL_CLOSED_PROOF_EVIDENCE_DIR}/replay_trace.jsonl"
  echo "fail_closed_replay_trace_hash_txt=${FAIL_CLOSED_PROOF_EVIDENCE_DIR}/replay_trace_hash.txt"
  echo "fail_closed_replay_report_json=${FAIL_CLOSED_PROOF_EVIDENCE_DIR}/replay_report.json"
  echo "fail_closed_replay_manifest_json=${FAIL_CLOSED_PROOF_EVIDENCE_DIR}/replay_manifest.json"
  echo "fail_closed_final_state_hash_txt=${FAIL_CLOSED_PROOF_EVIDENCE_DIR}/final_state_hash.txt"
  echo "fail_closed_replay_result_hash_txt=${FAIL_CLOSED_PROOF_EVIDENCE_DIR}/replay_result_hash.txt"
  echo "validator_rc=${VALIDATOR_RC}"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "syscall-semantics-phase10b: FAIL (${COUNT} violations)"
  exit 2
fi

echo "syscall-semantics-phase10b: PASS (mode=${MODE})"
exit 0

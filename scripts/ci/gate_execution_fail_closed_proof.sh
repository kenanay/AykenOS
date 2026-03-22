#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_execution_fail_closed_proof.sh \
    --evidence-dir evidence/run-<id>/gates/fail-closed-proof \
    [--qemu-timeout <sec>]

Exit codes:
  0: pass
  1: tooling/infra failure
  2: proof contract failure
  3: usage error
USAGE
}

EVIDENCE_DIR=""
QEMU_TIMEOUT="${PHASE10B_PROOF_QEMU_TIMEOUT:-20}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --qemu-timeout)
      QEMU_TIMEOUT="$2"
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

for tool in python3 make; do
  if ! command -v "${tool}" >/dev/null 2>&1; then
    echo "ERROR: missing required tool: ${tool}" >&2
    exit 1
  fi
done

BOOT_AUDIT="${ROOT}/tools/validation/phase_4_4_qemu_boot_audit.sh"
VALIDATOR="${ROOT}/tools/ci/validate_execution_fail_closed_proof.py"

if [[ ! -x "${BOOT_AUDIT}" ]]; then
  echo "ERROR: missing boot audit script: ${BOOT_AUDIT}" >&2
  exit 1
fi
if [[ ! -f "${VALIDATOR}" ]]; then
  echo "ERROR: missing proof validator: ${VALIDATOR}" >&2
  exit 1
fi

mkdir -p "${EVIDENCE_DIR}"

BUILD_LOG="${EVIDENCE_DIR}/build.log"
BOOT_AUDIT_LOG="${EVIDENCE_DIR}/boot_audit.log"
BOOT_AUDIT_DIR="${EVIDENCE_DIR}/boot-audit"
COMBINED_LOG="${EVIDENCE_DIR}/combined.log"
MARKER_LOG="${EVIDENCE_DIR}/marker.log"
PROOF_JSON="${EVIDENCE_DIR}/proof.json"
REPLAY_TRACE_JSONL="${EVIDENCE_DIR}/replay_trace.jsonl"
REPLAY_TRACE_HASH_TXT="${EVIDENCE_DIR}/replay_trace_hash.txt"
REPLAY_REPORT_JSON="${EVIDENCE_DIR}/replay_report.json"
REPLAY_MANIFEST_JSON="${EVIDENCE_DIR}/replay_manifest.json"
FINAL_STATE_HASH_TXT="${EVIDENCE_DIR}/final_state_hash.txt"
REPLAY_RESULT_HASH_TXT="${EVIDENCE_DIR}/replay_result_hash.txt"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

: > "${BUILD_LOG}"
: > "${BOOT_AUDIT_LOG}"
: > "${COMBINED_LOG}"
: > "${MARKER_LOG}"
: > "${VIOLATIONS_TXT}"
: > "${META_TXT}"

refresh_marker_log() {
  cat "${BOOT_AUDIT_DIR}/qemu_serial.log" "${BOOT_AUDIT_DIR}/qemu_debugcon.log" 2>/dev/null > "${COMBINED_LOG}" || true
  if [[ -s "${BOOT_AUDIT_DIR}/qemu_debugcon.log" ]]; then
    cp -f "${BOOT_AUDIT_DIR}/qemu_debugcon.log" "${MARKER_LOG}"
  else
    cp -f "${COMBINED_LOG}" "${MARKER_LOG}"
  fi
}

wait_for_marker_log() {
  local attempts=20
  local i
  for ((i = 1; i <= attempts; i++)); do
    refresh_marker_log
    if [[ -s "${MARKER_LOG}" ]]; then
      return 0
    fi
    sleep 0.1
  done
  return 1
}

set +e
make -C "${ROOT}" \
  KERNEL_PROFILE=validation \
  USER_MINIMAL_MODE=phase10a2 \
  AYKEN_CR3_PCID=0 \
  AYKEN_MB_SELFTEST=0 \
  AYKEN_GATE4_POLICY_TEST=0 \
  AYKEN_SCHED_BOOTSTRAP_POLICY=0 \
  AYKEN_PHASE10B_FAIL_CLOSED_SELFTEST=1 \
  clean > "${BUILD_LOG}" 2>&1 || true
make -C "${ROOT}" \
  KERNEL_PROFILE=validation \
  USER_MINIMAL_MODE=phase10a2 \
  AYKEN_CR3_PCID=0 \
  AYKEN_MB_SELFTEST=0 \
  AYKEN_GATE4_POLICY_TEST=0 \
  AYKEN_SCHED_BOOTSTRAP_POLICY=0 \
  AYKEN_PHASE10B_FAIL_CLOSED_SELFTEST=1 \
  guard-context-offsets efi-img >> "${BUILD_LOG}" 2>&1
BUILD_RC=$?
set -e

if [[ "${BUILD_RC}" -ne 0 ]]; then
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "phase10b-fail-closed-proof",
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["build_failed"]
}
EOF
  echo "build_failed" > "${VIOLATIONS_TXT}"
  echo "phase10b-fail-closed-proof: INFRA FAIL (build_failed)"
  exit 1
fi

set +e
"${BOOT_AUDIT}" \
  --timeout "${QEMU_TIMEOUT}" \
  --marker "[[P10B_FAIL_CLOSED_END]]" \
  --out-dir "${BOOT_AUDIT_DIR}" > "${BOOT_AUDIT_LOG}" 2>&1
BOOT_AUDIT_RC=$?
set -e

if ! wait_for_marker_log; then
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "phase10b-fail-closed-proof",
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": ["marker_log_empty"]
}
EOF
  echo "marker_log_empty" > "${VIOLATIONS_TXT}"
  echo "phase10b-fail-closed-proof: FAIL (marker_log_empty)"
  exit 2
fi

set +e
python3 "${VALIDATOR}" \
  --log "${MARKER_LOG}" \
  --out "${REPORT_JSON}" \
  --out-proof "${PROOF_JSON}" \
  --out-replay-trace-jsonl "${REPLAY_TRACE_JSONL}" \
  --out-replay-trace-hash-txt "${REPLAY_TRACE_HASH_TXT}" \
  --out-replay-report "${REPLAY_REPORT_JSON}" \
  --out-replay-manifest-json "${REPLAY_MANIFEST_JSON}" \
  --out-final-state-hash-txt "${FINAL_STATE_HASH_TXT}" \
  --out-replay-result-hash-txt "${REPLAY_RESULT_HASH_TXT}"
VALIDATOR_RC=$?
set -e

for required in \
  "${REPORT_JSON}" \
  "${PROOF_JSON}" \
  "${REPLAY_TRACE_JSONL}" \
  "${REPLAY_TRACE_HASH_TXT}" \
  "${REPLAY_REPORT_JSON}" \
  "${REPLAY_MANIFEST_JSON}" \
  "${FINAL_STATE_HASH_TXT}" \
  "${REPLAY_RESULT_HASH_TXT}"
do
  if [[ ! -f "${required}" ]]; then
    echo "ERROR: validator did not produce required artifact: ${required}" >&2
    exit 3
  fi
done

python3 - "${REPORT_JSON}" "${PROOF_JSON}" "${REPLAY_TRACE_JSONL}" "${REPLAY_TRACE_HASH_TXT}" "${REPLAY_REPORT_JSON}" "${REPLAY_MANIFEST_JSON}" "${FINAL_STATE_HASH_TXT}" "${REPLAY_RESULT_HASH_TXT}" "${BOOT_AUDIT_RC}" "${QEMU_TIMEOUT}" <<'PY'
import json
import sys

(
    report_path,
    proof_path,
    replay_trace_path,
    replay_trace_hash_path,
    replay_report_path,
    replay_manifest_path,
    final_state_hash_path,
    replay_result_hash_path,
    boot_audit_rc,
    qemu_timeout,
) = sys.argv[1:11]
with open(report_path, "r", encoding="utf-8") as fh:
    row = json.load(fh)
row["proof_json"] = proof_path
row["replay_trace_jsonl"] = replay_trace_path
row["replay_trace_hash_file"] = replay_trace_hash_path
row["replay_report_json"] = replay_report_path
row["replay_manifest_json"] = replay_manifest_path
row["final_state_hash_file"] = final_state_hash_path
row["replay_result_hash_file"] = replay_result_hash_path
row["boot_audit_exit_code"] = int(boot_audit_rc)
row["qemu_timeout_seconds"] = int(qemu_timeout)
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
  echo "qemu_timeout=${QEMU_TIMEOUT}"
  echo "boot_audit_rc=${BOOT_AUDIT_RC}"
  echo "validator_rc=${VALIDATOR_RC}"
  echo "proof_json=${PROOF_JSON}"
  echo "replay_trace_jsonl=${REPLAY_TRACE_JSONL}"
  echo "replay_trace_hash_txt=${REPLAY_TRACE_HASH_TXT}"
  echo "replay_report_json=${REPLAY_REPORT_JSON}"
  echo "replay_manifest_json=${REPLAY_MANIFEST_JSON}"
  echo "final_state_hash_txt=${FINAL_STATE_HASH_TXT}"
  echo "replay_result_hash_txt=${REPLAY_RESULT_HASH_TXT}"
} > "${META_TXT}"

if [[ "${BOOT_AUDIT_RC}" -ne 0 ]]; then
  echo "phase10b-fail-closed-proof: FAIL (boot_audit_failed)"
  exit 2
fi

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "phase10b-fail-closed-proof: FAIL (${COUNT} violations)"
  exit 2
fi

echo "phase10b-fail-closed-proof: PASS"
exit 0

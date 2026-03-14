#!/usr/bin/env bash
set -euo pipefail

# Author: Kenan AY

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_kpl_proof_verify.sh \
    --evidence-dir evidence/run-<id>/gates/kpl-proof \
    --abdf-evidence evidence/run-<id>/gates/abdf-snapshot-identity \
    --execution-evidence evidence/run-<id>/gates/execution-identity \
    --replay-evidence evidence/run-<id>/gates/replay-v1 \
    --ledger-evidence evidence/run-<id>/gates/ledger-v1 \
    --eti-evidence evidence/run-<id>/gates/eti \
    [--kernel-image-bin <path>] \
    [--config-json <path>] \
    [--in-proof-manifest-json <path>] \
    [--expected-proof-hash-file <path>] \
    [--expected-final-state-hash-file <path>]

Exit codes:
  0: pass
  2: KPL proof manifest contract failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
ABDF_EVIDENCE_DIR=""
EXECUTION_EVIDENCE_DIR=""
REPLAY_EVIDENCE_DIR=""
LEDGER_EVIDENCE_DIR=""
ETI_EVIDENCE_DIR=""
KERNEL_IMAGE_BIN="kernel.elf"
CONFIG_JSON=""
INPUT_PROOF_MANIFEST_JSON=""
EXPECTED_PROOF_HASH_FILE=""
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
    --replay-evidence)
      REPLAY_EVIDENCE_DIR="$2"
      shift 2
      ;;
    --ledger-evidence)
      LEDGER_EVIDENCE_DIR="$2"
      shift 2
      ;;
    --eti-evidence)
      ETI_EVIDENCE_DIR="$2"
      shift 2
      ;;
    --kernel-image-bin)
      KERNEL_IMAGE_BIN="$2"
      shift 2
      ;;
    --config-json)
      CONFIG_JSON="$2"
      shift 2
      ;;
    --in-proof-manifest-json)
      INPUT_PROOF_MANIFEST_JSON="$2"
      shift 2
      ;;
    --expected-proof-hash-file)
      EXPECTED_PROOF_HASH_FILE="$2"
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

if [[ -z "${EVIDENCE_DIR}" || -z "${ABDF_EVIDENCE_DIR}" || -z "${EXECUTION_EVIDENCE_DIR}" || -z "${REPLAY_EVIDENCE_DIR}" || -z "${LEDGER_EVIDENCE_DIR}" || -z "${ETI_EVIDENCE_DIR}" || -z "${CONFIG_JSON}" ]]; then
  usage
  exit 3
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: missing required tool: python3" >&2
  exit 3
fi

VALIDATOR="${ROOT}/tools/ci/validate_kpl_proof_manifest.py"
if [[ ! -f "${VALIDATOR}" ]]; then
  echo "ERROR: missing validator: ${VALIDATOR}" >&2
  exit 3
fi

ABDF_HASH_FILE="${ABDF_EVIDENCE_DIR}/abdf_snapshot_hash.txt"
BCIB_PLAN_HASH_FILE="${EXECUTION_EVIDENCE_DIR}/bcib_plan_hash.txt"
EXECUTION_TRACE_HASH_FILE="${EXECUTION_EVIDENCE_DIR}/execution_trace_hash.txt"
REPLAY_REPORT_JSON="${REPLAY_EVIDENCE_DIR}/replay_report.json"
LEDGER_JSONL="${LEDGER_EVIDENCE_DIR}/decision_ledger.jsonl"
ETI_JSONL="${ETI_EVIDENCE_DIR}/eti_transcript.jsonl"

if [[ ! -s "${ABDF_HASH_FILE}" ]]; then
  echo "ERROR: missing_or_empty:${ABDF_HASH_FILE}" >&2
  exit 3
fi
if [[ ! -s "${BCIB_PLAN_HASH_FILE}" ]]; then
  echo "ERROR: missing_or_empty:${BCIB_PLAN_HASH_FILE}" >&2
  exit 3
fi
if [[ ! -s "${EXECUTION_TRACE_HASH_FILE}" ]]; then
  echo "ERROR: missing_or_empty:${EXECUTION_TRACE_HASH_FILE}" >&2
  exit 3
fi
if [[ ! -s "${REPLAY_REPORT_JSON}" ]]; then
  echo "ERROR: missing_or_empty:${REPLAY_REPORT_JSON}" >&2
  exit 3
fi
if [[ ! -s "${LEDGER_JSONL}" ]]; then
  echo "ERROR: missing_or_empty:${LEDGER_JSONL}" >&2
  exit 3
fi
if [[ ! -s "${ETI_JSONL}" ]]; then
  echo "ERROR: missing_or_empty:${ETI_JSONL}" >&2
  exit 3
fi
if [[ ! -s "${KERNEL_IMAGE_BIN}" ]]; then
  echo "ERROR: missing_or_empty:${KERNEL_IMAGE_BIN}" >&2
  exit 3
fi
if [[ ! -s "${CONFIG_JSON}" ]]; then
  echo "ERROR: missing_or_empty:${CONFIG_JSON}" >&2
  exit 3
fi
if [[ -n "${INPUT_PROOF_MANIFEST_JSON}" && ! -s "${INPUT_PROOF_MANIFEST_JSON}" ]]; then
  echo "ERROR: missing_or_empty:${INPUT_PROOF_MANIFEST_JSON}" >&2
  exit 3
fi
if [[ -n "${EXPECTED_PROOF_HASH_FILE}" && ! -s "${EXPECTED_PROOF_HASH_FILE}" ]]; then
  echo "ERROR: missing_or_empty:${EXPECTED_PROOF_HASH_FILE}" >&2
  exit 3
fi
if [[ -n "${EXPECTED_FINAL_STATE_HASH_FILE}" && ! -s "${EXPECTED_FINAL_STATE_HASH_FILE}" ]]; then
  echo "ERROR: missing_or_empty:${EXPECTED_FINAL_STATE_HASH_FILE}" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

PROOF_MANIFEST_JSON="${EVIDENCE_DIR}/proof_manifest.json"
PROOF_VERIFY_JSON="${EVIDENCE_DIR}/proof_verify.json"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

VALIDATOR_ARGS=(
  --abdf-hash-file "${ABDF_HASH_FILE}"
  --bcib-plan-hash-file "${BCIB_PLAN_HASH_FILE}"
  --execution-trace-hash-file "${EXECUTION_TRACE_HASH_FILE}"
  --replay-report-json "${REPLAY_REPORT_JSON}"
  --ledger-jsonl "${LEDGER_JSONL}"
  --eti-jsonl "${ETI_JSONL}"
  --kernel-image-bin "${KERNEL_IMAGE_BIN}"
  --config-json "${CONFIG_JSON}"
  --out-proof-manifest-json "${PROOF_MANIFEST_JSON}"
  --out-proof-verify-json "${PROOF_VERIFY_JSON}"
  --out-report "${REPORT_JSON}"
)
if [[ -n "${INPUT_PROOF_MANIFEST_JSON}" ]]; then
  VALIDATOR_ARGS+=(--in-proof-manifest-json "${INPUT_PROOF_MANIFEST_JSON}")
fi
if [[ -n "${EXPECTED_PROOF_HASH_FILE}" ]]; then
  VALIDATOR_ARGS+=(--expected-proof-hash-file "${EXPECTED_PROOF_HASH_FILE}")
fi
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
if [[ ! -f "${PROOF_MANIFEST_JSON}" ]]; then
  echo "ERROR: validator did not produce proof manifest: ${PROOF_MANIFEST_JSON}" >&2
  exit 3
fi
if [[ ! -f "${PROOF_VERIFY_JSON}" ]]; then
  echo "ERROR: validator did not produce proof verify: ${PROOF_VERIFY_JSON}" >&2
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
  echo "execution_trace_hash_file=${EXECUTION_TRACE_HASH_FILE}"
  echo "replay_report_json=${REPLAY_REPORT_JSON}"
  echo "ledger_jsonl=${LEDGER_JSONL}"
  echo "eti_jsonl=${ETI_JSONL}"
  echo "kernel_image_bin=${KERNEL_IMAGE_BIN}"
  echo "config_json=${CONFIG_JSON}"
  echo "in_proof_manifest_json=${INPUT_PROOF_MANIFEST_JSON}"
  echo "expected_proof_hash_file=${EXPECTED_PROOF_HASH_FILE}"
  echo "expected_final_state_hash_file=${EXPECTED_FINAL_STATE_HASH_FILE}"
  echo "validator_rc=${VALIDATOR_RC}"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "kpl-proof-verify: FAIL (${COUNT} violations)"
  exit 2
fi

echo "kpl-proof-verify: PASS"
exit 0

#!/usr/bin/env bash
set -euo pipefail

# Author: Kenan AY

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_proof_bundle.sh \
    --evidence-dir evidence/run-<id>/gates/proof-bundle \
    --abdf-evidence evidence/run-<id>/gates/abdf-snapshot-identity \
    --execution-evidence evidence/run-<id>/gates/execution-identity \
    --replay-evidence evidence/run-<id>/gates/replay-v1 \
    --kpl-evidence evidence/run-<id>/gates/kpl-proof \
    --ledger-evidence evidence/run-<id>/gates/ledger-v1 \
    --eti-evidence evidence/run-<id>/gates/eti \
    [--kernel-image-bin <path>] \
    [--summary-json <path>] \
    [--meta-run-json <path>]

Exit codes:
  0: pass
  2: proof bundle portability failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
ABDF_EVIDENCE_DIR=""
EXECUTION_EVIDENCE_DIR=""
REPLAY_EVIDENCE_DIR=""
KPL_EVIDENCE_DIR=""
LEDGER_EVIDENCE_DIR=""
ETI_EVIDENCE_DIR=""
KERNEL_IMAGE_BIN="kernel.elf"
SUMMARY_JSON=""
META_RUN_JSON=""

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
    --kpl-evidence)
      KPL_EVIDENCE_DIR="$2"
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
    --summary-json)
      SUMMARY_JSON="$2"
      shift 2
      ;;
    --meta-run-json)
      META_RUN_JSON="$2"
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

if [[ -z "${EVIDENCE_DIR}" || -z "${ABDF_EVIDENCE_DIR}" || -z "${EXECUTION_EVIDENCE_DIR}" || -z "${REPLAY_EVIDENCE_DIR}" || -z "${KPL_EVIDENCE_DIR}" || -z "${LEDGER_EVIDENCE_DIR}" || -z "${ETI_EVIDENCE_DIR}" ]]; then
  usage
  exit 3
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: missing required tool: python3" >&2
  exit 3
fi

RUN_ROOT="$(cd "$(dirname "${EVIDENCE_DIR}")/.." && pwd)"
if [[ -z "${SUMMARY_JSON}" ]]; then
  SUMMARY_JSON="${RUN_ROOT}/reports/summary.json"
fi
if [[ -z "${META_RUN_JSON}" ]]; then
  META_RUN_JSON="${RUN_ROOT}/meta/run.json"
fi

VALIDATOR="${ROOT}/tools/ci/validate_proof_bundle.py"
VERIFY_SCRIPT="${ROOT}/scripts/ci/verify_proof_bundle.sh"
if [[ ! -f "${VALIDATOR}" ]]; then
  echo "ERROR: missing validator: ${VALIDATOR}" >&2
  exit 3
fi
if [[ ! -f "${VERIFY_SCRIPT}" ]]; then
  echo "ERROR: missing verifier script: ${VERIFY_SCRIPT}" >&2
  exit 3
fi

for required_path in \
  "${ABDF_EVIDENCE_DIR}/abdf_snapshot_hash.txt" \
  "${EXECUTION_EVIDENCE_DIR}/bcib_plan_hash.txt" \
  "${EXECUTION_EVIDENCE_DIR}/execution_trace_hash.txt" \
  "${EXECUTION_EVIDENCE_DIR}/execution_trace.jsonl" \
  "${REPLAY_EVIDENCE_DIR}/replay_trace_hash.txt" \
  "${REPLAY_EVIDENCE_DIR}/replay_trace.jsonl" \
  "${REPLAY_EVIDENCE_DIR}/replay_report.json" \
  "${KPL_EVIDENCE_DIR}/proof_manifest.json" \
  "${KPL_EVIDENCE_DIR}/proof_verify.json" \
  "${KPL_EVIDENCE_DIR}/report.json" \
  "${LEDGER_EVIDENCE_DIR}/decision_ledger.jsonl" \
  "${ETI_EVIDENCE_DIR}/eti_transcript.jsonl" \
  "${KERNEL_IMAGE_BIN}" \
  "${SUMMARY_JSON}" \
  "${META_RUN_JSON}"; do
  if [[ ! -s "${required_path}" ]]; then
    echo "ERROR: missing_or_empty:${required_path}" >&2
    exit 3
  fi
done

mkdir -p "${EVIDENCE_DIR}"

BUNDLE_ROOT="${EVIDENCE_DIR}/proof_bundle"
VERIFY_OUT_DIR="${EVIDENCE_DIR}"
META_TXT="${EVIDENCE_DIR}/meta.txt"

set +e
python3 "${VALIDATOR}" generate \
  --bundle-root "${BUNDLE_ROOT}" \
  --abdf-evidence "${ABDF_EVIDENCE_DIR}" \
  --execution-evidence "${EXECUTION_EVIDENCE_DIR}" \
  --replay-evidence "${REPLAY_EVIDENCE_DIR}" \
  --kpl-evidence "${KPL_EVIDENCE_DIR}" \
  --ledger-evidence "${LEDGER_EVIDENCE_DIR}" \
  --eti-evidence "${ETI_EVIDENCE_DIR}" \
  --kernel-image-bin "${KERNEL_IMAGE_BIN}" \
  --summary-json "${SUMMARY_JSON}" \
  --meta-run-json "${META_RUN_JSON}"
GENERATE_RC=$?
set -e

if [[ "${GENERATE_RC}" -ne 0 ]]; then
  echo "ERROR: proof-bundle generation failed rc=${GENERATE_RC}" >&2
  exit 3
fi

set +e
bash "${VERIFY_SCRIPT}" --bundle-root "${BUNDLE_ROOT}" --out-dir "${VERIFY_OUT_DIR}"
VERIFY_RC=$?
set -e

{
  echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "bundle_root=${BUNDLE_ROOT}"
  echo "run_root=${RUN_ROOT}"
  echo "generate_rc=${GENERATE_RC}"
  echo "verify_rc=${VERIFY_RC}"
  echo "summary_json=${SUMMARY_JSON}"
  echo "meta_run_json=${META_RUN_JSON}"
} > "${META_TXT}"

if [[ "${VERIFY_RC}" -ne 0 ]]; then
  echo "proof-bundle: FAIL"
  exit 2
fi

echo "proof-bundle: PASS"
exit 0

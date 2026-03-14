#!/usr/bin/env bash
set -euo pipefail

# Author: Kenan AY

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_abdf_snapshot_identity.sh \
    --evidence-dir evidence/run-<id>/gates/abdf-snapshot-identity \
    --snapshot-bin evidence/run-<id>/input/snapshot.abdf \
    [--expected-hash-file <path>]

Exit codes:
  0: pass
  2: ABDF snapshot identity contract failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
SNAPSHOT_BIN=""
EXPECTED_HASH_FILE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --snapshot-bin)
      SNAPSHOT_BIN="$2"
      shift 2
      ;;
    --expected-hash-file)
      EXPECTED_HASH_FILE="$2"
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

if [[ -z "${EVIDENCE_DIR}" || -z "${SNAPSHOT_BIN}" ]]; then
  usage
  exit 3
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: missing required tool: python3" >&2
  exit 3
fi

VALIDATOR="${ROOT}/tools/ci/validate_abdf_snapshot_identity.py"
if [[ ! -f "${VALIDATOR}" ]]; then
  echo "ERROR: missing validator: ${VALIDATOR}" >&2
  exit 3
fi

if [[ ! -s "${SNAPSHOT_BIN}" ]]; then
  echo "ERROR: missing_or_empty:${SNAPSHOT_BIN}" >&2
  exit 3
fi
if [[ -n "${EXPECTED_HASH_FILE}" && ! -s "${EXPECTED_HASH_FILE}" ]]; then
  echo "ERROR: missing_or_empty:${EXPECTED_HASH_FILE}" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

ABDF_HASH_TXT="${EVIDENCE_DIR}/abdf_snapshot_hash.txt"
IDENTITY_REPORT_JSON="${EVIDENCE_DIR}/snapshot_identity_report.json"
CONSISTENCY_REPORT_JSON="${EVIDENCE_DIR}/snapshot_identity_consistency.json"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

VALIDATOR_ARGS=(
  --snapshot-bin "${SNAPSHOT_BIN}"
  --out-hash-txt "${ABDF_HASH_TXT}"
  --out-identity-report "${IDENTITY_REPORT_JSON}"
  --out-consistency-report "${CONSISTENCY_REPORT_JSON}"
  --out-report "${REPORT_JSON}"
)
if [[ -n "${EXPECTED_HASH_FILE}" ]]; then
  VALIDATOR_ARGS+=(--expected-hash-file "${EXPECTED_HASH_FILE}")
fi

set +e
python3 "${VALIDATOR}" "${VALIDATOR_ARGS[@]}"
VALIDATOR_RC=$?
set -e

if [[ ! -f "${REPORT_JSON}" ]]; then
  echo "ERROR: validator did not produce report: ${REPORT_JSON}" >&2
  exit 3
fi
if [[ ! -f "${ABDF_HASH_TXT}" ]]; then
  echo "ERROR: validator did not produce hash file: ${ABDF_HASH_TXT}" >&2
  exit 3
fi
if [[ ! -f "${IDENTITY_REPORT_JSON}" ]]; then
  echo "ERROR: validator did not produce identity report: ${IDENTITY_REPORT_JSON}" >&2
  exit 3
fi
if [[ ! -f "${CONSISTENCY_REPORT_JSON}" ]]; then
  echo "ERROR: validator did not produce consistency report: ${CONSISTENCY_REPORT_JSON}" >&2
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
  echo "snapshot_bin=${SNAPSHOT_BIN}"
  echo "expected_hash_file=${EXPECTED_HASH_FILE}"
  echo "validator_rc=${VALIDATOR_RC}"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "abdf-snapshot-identity: FAIL (${COUNT} violations)"
  exit 2
fi

echo "abdf-snapshot-identity: PASS"
exit 0

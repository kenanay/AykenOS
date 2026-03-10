#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_proof_exchange.sh \
    --evidence-dir evidence/run-<id>/gates/proof-exchange

Exit codes:
  0: pass
  2: proof exchange gate failure
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
if ! command -v cargo >/dev/null 2>&1; then
  echo "ERROR: missing required tool: cargo" >&2
  exit 3
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: missing required tool: python3" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

REPORT_JSON="${EVIDENCE_DIR}/report.json"
EXCHANGE_CONTRACT_REPORT_JSON="${EVIDENCE_DIR}/exchange_contract_report.json"
TRANSPORT_MUTATION_MATRIX_JSON="${EVIDENCE_DIR}/transport_mutation_matrix.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

set +e
cargo run --quiet --manifest-path "${ROOT}/ayken-core/Cargo.toml" \
  -p proof-verifier \
  --example phase12_gate_harness \
  -- proof-exchange --out-dir "${EVIDENCE_DIR}"
HARNESS_RC=$?
set -e

if [[ ! -f "${REPORT_JSON}" || ! -f "${EXCHANGE_CONTRACT_REPORT_JSON}" || ! -f "${TRANSPORT_MUTATION_MATRIX_JSON}" ]]; then
  echo "ERROR: proof exchange harness did not produce required outputs" >&2
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
  echo "harness_rc=${HARNESS_RC}"
  echo "evidence_dir=${EVIDENCE_DIR}"
} > "${META_TXT}"

if [[ "${HARNESS_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "proof-exchange: FAIL (${COUNT} violations)"
  exit 2
fi

echo "proof-exchange: PASS"
exit 0

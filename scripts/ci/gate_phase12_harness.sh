#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_phase12_harness.sh \
    --mode <producer-schema|signature-envelope|bundle-v2-schema|bundle-v2-compat|signature-verify|registry-resolution|key-rotation> \
    --evidence-dir evidence/run-<id>/gates/<gate-name>

Exit codes:
  0: pass
  2: gate failure
  3: usage/tooling error
USAGE
}

MODE=""
EVIDENCE_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mode)
      MODE="$2"
      shift 2
      ;;
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

if [[ -z "${MODE}" || -z "${EVIDENCE_DIR}" ]]; then
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

case "${MODE}" in
  producer-schema)
    GATE_NAME="proof-producer-schema"
    REQUIRED_OUTPUTS=("report.json" "producer_schema_report.json" "producer_identity_examples.json")
    ;;
  signature-envelope)
    GATE_NAME="proof-signature-envelope"
    REQUIRED_OUTPUTS=("report.json" "signature_envelope_report.json" "identity_stability_report.json")
    ;;
  bundle-v2-schema)
    GATE_NAME="proof-bundle-v2-schema"
    REQUIRED_OUTPUTS=("report.json" "bundle_schema_report.json")
    ;;
  bundle-v2-compat)
    GATE_NAME="proof-bundle-v2-compat"
    REQUIRED_OUTPUTS=("report.json" "compatibility_report.json")
    ;;
  signature-verify)
    GATE_NAME="proof-signature-verify"
    REQUIRED_OUTPUTS=("report.json" "signature_verify.json" "registry_resolution_report.json")
    ;;
  registry-resolution)
    GATE_NAME="proof-registry-resolution"
    REQUIRED_OUTPUTS=("report.json" "registry_snapshot.json" "registry_resolution_matrix.json")
    ;;
  key-rotation)
    GATE_NAME="proof-key-rotation"
    REQUIRED_OUTPUTS=("report.json" "rotation_matrix.json" "revocation_matrix.json")
    ;;
  *)
    echo "ERROR: unsupported mode: ${MODE}" >&2
    usage
    exit 3
    ;;
esac

mkdir -p "${EVIDENCE_DIR}"

REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

set +e
cargo run --quiet --manifest-path "${ROOT}/ayken-core/Cargo.toml" \
  -p proof-verifier \
  --example phase12_gate_harness \
  -- "${MODE}" --out-dir "${EVIDENCE_DIR}"
HARNESS_RC=$?
set -e

for output in "${REQUIRED_OUTPUTS[@]}"; do
  if [[ ! -f "${EVIDENCE_DIR}/${output}" ]]; then
    echo "ERROR: ${GATE_NAME} harness did not produce required output ${output}" >&2
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
  echo "gate_name=${GATE_NAME}"
  echo "mode=${MODE}"
  echo "harness_rc=${HARNESS_RC}"
  echo "evidence_dir=${EVIDENCE_DIR}"
} > "${META_TXT}"

if [[ "${HARNESS_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "${GATE_NAME}: FAIL (${COUNT} violations)"
  exit 2
fi

echo "${GATE_NAME}: PASS"
exit 0

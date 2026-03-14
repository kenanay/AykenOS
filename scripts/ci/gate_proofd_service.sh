#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_proofd_service.sh \
    --evidence-dir evidence/run-<id>/gates/proofd-service

Exit codes:
  0: pass
  2: proofd service gate failure
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
PROOFD_SERVICE_REPORT_JSON="${EVIDENCE_DIR}/proofd_service_report.json"
PROOFD_RECEIPT_REPORT_JSON="${EVIDENCE_DIR}/proofd_receipt_report.json"
PROOFD_ENDPOINT_CONTRACT_JSON="${EVIDENCE_DIR}/proofd_endpoint_contract.json"
PROOFD_RECEIPT_VERIFICATION_REPORT_JSON="${EVIDENCE_DIR}/proofd_receipt_verification_report.json"
PROOFD_REPEATED_EXECUTION_REPORT_JSON="${EVIDENCE_DIR}/proofd_repeated_execution_report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"
SERVICE_ROOT="${EVIDENCE_DIR}/service-root"
RUN_ID="run-proofd-local-r1"
RUN_DIR="${SERVICE_ROOT}/${RUN_ID}"
STDOUT_LOG="${EVIDENCE_DIR}/proofd.stdout.log"
STDERR_LOG="${EVIDENCE_DIR}/proofd.stderr.log"

rm -rf "${SERVICE_ROOT}"
mkdir -p "${RUN_DIR}"

set +e
cargo run --quiet --manifest-path "${ROOT}/ayken-core/Cargo.toml" \
  -p proof-verifier \
  --example phase12_gate_harness \
  -- cross-node-parity --out-dir "${RUN_DIR}"
HARNESS_RC=$?
set -e

if [[ "${HARNESS_RC}" -ne 0 ]]; then
  cat > "${VIOLATIONS_TXT}" <<'EOF'
proofd-service gate bootstrap failed because the cross-node parity fixture could not be generated
EOF
  cat > "${REPORT_JSON}" <<'EOF'
{"gate":"proofd-service","mode":"phase12_proofd_service_gate_bootstrap","verdict":"FAIL","violations":["cross_node_parity_fixture_generation_failed"],"violations_count":1}
EOF
  exit 2
fi

for artifact in \
  parity_report.json \
  parity_determinism_incidents.json \
  parity_drift_attribution_report.json \
  parity_convergence_report.json \
  failure_matrix.json \
  parity_authority_drift_topology.json \
  parity_authority_suppression_report.json \
  parity_incident_graph.json
do
  cp -f "${RUN_DIR}/${artifact}" "${SERVICE_ROOT}/${artifact}"
done

set +e
cargo run --quiet --manifest-path "${ROOT}/userspace/Cargo.toml" \
  -p proofd \
  --example proofd_gate_harness \
  -- service-contract \
  --evidence-root "${SERVICE_ROOT}" \
  --run-id "${RUN_ID}" \
  --out-dir "${EVIDENCE_DIR}" \
  >"${STDOUT_LOG}" 2>"${STDERR_LOG}"
HARNESS_GATE_RC=$?
set -e

for artifact in \
  "${REPORT_JSON}" \
  "${PROOFD_SERVICE_REPORT_JSON}" \
  "${PROOFD_RECEIPT_REPORT_JSON}" \
  "${PROOFD_ENDPOINT_CONTRACT_JSON}" \
  "${EVIDENCE_DIR}/proofd_verify_request.json" \
  "${EVIDENCE_DIR}/proofd_verify_response.json" \
  "${EVIDENCE_DIR}/proofd_run_manifest.json" \
  "${EVIDENCE_DIR}/replay_boundary_flow_source.json" \
  "${EVIDENCE_DIR}/trust_reuse_flow_source.json" \
  "${PROOFD_RECEIPT_VERIFICATION_REPORT_JSON}" \
  "${PROOFD_REPEATED_EXECUTION_REPORT_JSON}"
do
  if [[ ! -f "${artifact}" ]]; then
    echo "ERROR: proofd service harness did not produce required output: ${artifact}" >&2
    exit 3
  fi
done

{
  echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "harness_rc=${HARNESS_RC}"
  echo "proofd_gate_rc=${HARNESS_GATE_RC}"
  echo "service_root=${SERVICE_ROOT}"
  echo "run_id=${RUN_ID}"
  echo "evidence_dir=${EVIDENCE_DIR}"
} > "${META_TXT}"

if [[ "${HARNESS_GATE_RC}" -ne 0 ]]; then
  echo "proofd-service: FAIL"
  exit 2
fi

echo "proofd-service: PASS"
exit 0

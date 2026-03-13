#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_proofd_observability_boundary.sh \
    --evidence-dir evidence/run-<id>/gates/proofd-observability-boundary \
    [--artifact-root path] \
    [--run-id run-proofd-local-r1]

Exit codes:
  0: pass
  2: proofd observability boundary failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
ARTIFACT_ROOT=""
RUN_ID="run-proofd-local-r1"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --artifact-root)
      ARTIFACT_ROOT="$2"
      shift 2
      ;;
    --run-id)
      RUN_ID="$2"
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

mkdir -p "${EVIDENCE_DIR}"

REPORT_JSON="${EVIDENCE_DIR}/report.json"
BOUNDARY_REPORT_JSON="${EVIDENCE_DIR}/proofd_observability_boundary_report.json"
NEGATIVE_MATRIX_JSON="${EVIDENCE_DIR}/proofd_observability_negative_matrix.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"
STDOUT_LOG="${EVIDENCE_DIR}/proofd.stdout.log"
STDERR_LOG="${EVIDENCE_DIR}/proofd.stderr.log"

BOOTSTRAP_RC=0
if [[ -z "${ARTIFACT_ROOT}" ]]; then
  SERVICE_ROOT="${EVIDENCE_DIR}/service-root"
  RUN_DIR="${SERVICE_ROOT}/${RUN_ID}"
  rm -rf "${SERVICE_ROOT}"
  mkdir -p "${RUN_DIR}"

  set +e
  cargo run --quiet --manifest-path "${ROOT}/ayken-core/Cargo.toml" \
    -p proof-verifier \
    --example phase12_gate_harness \
    -- cross-node-parity --out-dir "${RUN_DIR}"
  BOOTSTRAP_RC=$?
  set -e

  if [[ "${BOOTSTRAP_RC}" -ne 0 ]]; then
    cat > "${VIOLATIONS_TXT}" <<'EOF'
proofd-observability-boundary gate bootstrap failed because the cross-node parity fixture could not be generated
EOF
    cat > "${REPORT_JSON}" <<'EOF'
{"gate":"proofd-observability-boundary","mode":"phase13_proofd_observability_boundary_bootstrap","verdict":"FAIL","violations":["cross_node_parity_fixture_generation_failed"],"violations_count":1}
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
else
  SERVICE_ROOT="${ARTIFACT_ROOT}"
  if [[ ! -d "${SERVICE_ROOT}/${RUN_ID}" ]]; then
    echo "ERROR: artifact root is missing run directory ${SERVICE_ROOT}/${RUN_ID}" >&2
    exit 3
  fi
fi

set +e
cargo run --quiet --manifest-path "${ROOT}/userspace/Cargo.toml" \
  -p proofd \
  --example proofd_gate_harness \
  -- observability-boundary \
  --evidence-root "${SERVICE_ROOT}" \
  --run-id "${RUN_ID}" \
  --out-dir "${EVIDENCE_DIR}" \
  >"${STDOUT_LOG}" 2>"${STDERR_LOG}"
HARNESS_RC=$?
set -e

for artifact in \
  "${REPORT_JSON}" \
  "${BOUNDARY_REPORT_JSON}" \
  "${NEGATIVE_MATRIX_JSON}" \
  "${VIOLATIONS_TXT}"
do
  if [[ ! -f "${artifact}" ]]; then
    echo "ERROR: proofd observability harness did not produce required output: ${artifact}" >&2
    exit 3
  fi
done

{
  echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "bootstrap_rc=${BOOTSTRAP_RC}"
  echo "proofd_gate_rc=${HARNESS_RC}"
  echo "service_root=${SERVICE_ROOT}"
  echo "run_id=${RUN_ID}"
  echo "evidence_dir=${EVIDENCE_DIR}"
} > "${META_TXT}"

if [[ "${HARNESS_RC}" -ne 0 ]]; then
  echo "proofd-observability-boundary: FAIL"
  exit 2
fi

echo "proofd-observability-boundary: PASS"
exit 0

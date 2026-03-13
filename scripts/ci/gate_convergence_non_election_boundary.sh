#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_convergence_non_election_boundary.sh \
    --evidence-dir evidence/run-<id>/gates/convergence-non-election-boundary \
    [--artifact-root /path/to/diagnostics-artifacts]

Exit codes:
  0: pass
  2: convergence non-election boundary failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
ARTIFACT_ROOT=""

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
if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: missing required tool: python3" >&2
  exit 3
fi
if [[ -z "${ARTIFACT_ROOT}" ]] && ! command -v cargo >/dev/null 2>&1; then
  echo "ERROR: missing required tool: cargo" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

REPORT_JSON="${EVIDENCE_DIR}/report.json"
DETAIL_REPORT_JSON="${EVIDENCE_DIR}/convergence_non_election_report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"
BOOTSTRAP_ROOT="${EVIDENCE_DIR}/artifact-root"

HARNESS_RC=0
BOOTSTRAP_MODE="provided_artifact_root"
if [[ -z "${ARTIFACT_ROOT}" ]]; then
  ARTIFACT_ROOT="${BOOTSTRAP_ROOT}"
  BOOTSTRAP_MODE="cross_node_parity_harness"
  rm -rf "${ARTIFACT_ROOT}"
  mkdir -p "${ARTIFACT_ROOT}"
  set +e
  cargo run --quiet --manifest-path "${ROOT}/ayken-core/Cargo.toml" \
    -p proof-verifier \
    --example phase12_gate_harness \
    -- cross-node-parity --out-dir "${ARTIFACT_ROOT}"
  HARNESS_RC=$?
  set -e
  if [[ "${HARNESS_RC}" -ne 0 ]]; then
    cat > "${VIOLATIONS_TXT}" <<'EOF'
artifact_bootstrap_failed:cross_node_parity_harness
EOF
    cat > "${DETAIL_REPORT_JSON}" <<EOF
{"status":"FAIL","mode":"phase13_convergence_non_election_boundary_gate","artifact_root":"${ARTIFACT_ROOT}","violations":["artifact_bootstrap_failed:cross_node_parity_harness"],"violations_count":1}
EOF
    cat > "${REPORT_JSON}" <<EOF
{"gate":"convergence-non-election-boundary","mode":"phase13_convergence_non_election_boundary_gate","verdict":"FAIL","detail_report_path":"convergence_non_election_report.json","violations":["artifact_bootstrap_failed:cross_node_parity_harness"],"violations_count":1}
EOF
    exit 2
  fi
fi

set +e
python3 "${ROOT}/tools/ci/validate_convergence_non_election_boundary.py" \
  --artifact-root "${ARTIFACT_ROOT}" \
  --out-report "${REPORT_JSON}" \
  --out-detail-report "${DETAIL_REPORT_JSON}" \
  --violations-out "${VIOLATIONS_TXT}"
VALIDATOR_RC=$?
set -e

{
  echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "bootstrap_mode=${BOOTSTRAP_MODE}"
  echo "harness_rc=${HARNESS_RC}"
  echo "validator_rc=${VALIDATOR_RC}"
  echo "artifact_root=${ARTIFACT_ROOT}"
  echo "evidence_dir=${EVIDENCE_DIR}"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "convergence-non-election-boundary: FAIL (${COUNT} violations)"
  exit 2
fi

echo "convergence-non-election-boundary: PASS"
exit 0

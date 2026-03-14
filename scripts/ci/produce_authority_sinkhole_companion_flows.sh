#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/produce_authority_sinkhole_companion_flows.sh \
    --evidence-dir evidence/run-<id>/producers/authority-sinkhole-companion-flows \
    --artifact-root /path/to/artifacts \
    [--replay-source /path/to/replay_boundary_flow_source.json] \
    [--trust-source /path/to/trust_reuse_flow_source.json] \
    [--replay-output /path/to/replay_boundary_flow_report.json] \
    [--trust-output /path/to/trust_reuse_flow_report.json]

Exit codes:
  0: producer success
  2: producer failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
ARTIFACT_ROOT=""
REPLAY_SOURCE_PATH=""
TRUST_SOURCE_PATH=""
REPLAY_OUTPUT_PATH=""
TRUST_OUTPUT_PATH=""

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
    --replay-source)
      REPLAY_SOURCE_PATH="$2"
      shift 2
      ;;
    --trust-source)
      TRUST_SOURCE_PATH="$2"
      shift 2
      ;;
    --replay-output)
      REPLAY_OUTPUT_PATH="$2"
      shift 2
      ;;
    --trust-output)
      TRUST_OUTPUT_PATH="$2"
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

if [[ -z "${EVIDENCE_DIR}" || -z "${ARTIFACT_ROOT}" ]]; then
  usage
  exit 3
fi
if ! command -v cargo >/dev/null 2>&1; then
  echo "ERROR: missing required tool: cargo" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

META_TXT="${EVIDENCE_DIR}/meta.txt"
VALIDATOR_ARGS=(
  --artifact-root "${ARTIFACT_ROOT}"
  --output-dir "${EVIDENCE_DIR}"
)
if [[ -n "${REPLAY_SOURCE_PATH}" ]]; then
  VALIDATOR_ARGS+=(--replay-source "${REPLAY_SOURCE_PATH}")
fi
if [[ -n "${TRUST_SOURCE_PATH}" ]]; then
  VALIDATOR_ARGS+=(--trust-source "${TRUST_SOURCE_PATH}")
fi
if [[ -n "${REPLAY_OUTPUT_PATH}" ]]; then
  VALIDATOR_ARGS+=(--replay-output "${REPLAY_OUTPUT_PATH}")
fi
if [[ -n "${TRUST_OUTPUT_PATH}" ]]; then
  VALIDATOR_ARGS+=(--trust-output "${TRUST_OUTPUT_PATH}")
fi

set +e
cargo run --quiet --manifest-path "${ROOT}/ayken-core/Cargo.toml" \
  -p proof-verifier \
  --bin authority-sinkhole-companion-producer \
  -- "${VALIDATOR_ARGS[@]}"
VALIDATOR_RC=$?
set -e

{
  echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "validator_rc=${VALIDATOR_RC}"
  echo "artifact_root=${ARTIFACT_ROOT}"
  echo "evidence_dir=${EVIDENCE_DIR}"
  if [[ -n "${REPLAY_SOURCE_PATH}" ]]; then
    echo "replay_source_path=${REPLAY_SOURCE_PATH}"
  fi
  if [[ -n "${TRUST_SOURCE_PATH}" ]]; then
    echo "trust_source_path=${TRUST_SOURCE_PATH}"
  fi
  if [[ -n "${REPLAY_OUTPUT_PATH}" ]]; then
    echo "replay_output_path=${REPLAY_OUTPUT_PATH}"
  fi
  if [[ -n "${TRUST_OUTPUT_PATH}" ]]; then
    echo "trust_output_path=${TRUST_OUTPUT_PATH}"
  fi
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${EVIDENCE_DIR}/violations.txt" 2>/dev/null || true)"
  echo "authority-sinkhole-companion-flow-producer: FAIL (${COUNT} violations)"
  exit 2
fi

echo "authority-sinkhole-companion-flow-producer: PASS"
exit 0

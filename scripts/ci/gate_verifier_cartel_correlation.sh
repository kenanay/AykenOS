#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_verifier_cartel_correlation.sh \
    --evidence-dir evidence/run-<id>/gates/verifier-cartel-correlation \
    --artifact-root /path/to/cartel-correlation-artifacts \
    [--ledger /path/to/verification_diversity_ledger.json] \
    [--policy /path/to/cartel_correlation_policy.json] \
    [--window-runs N] \
    [--window-seconds SECONDS]

Exit codes:
  0: pass
  2: verifier cartel correlation failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
ARTIFACT_ROOT=""
LEDGER_PATH=""
POLICY_PATH=""
WINDOW_RUNS=""
WINDOW_SECONDS=""

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
    --ledger)
      LEDGER_PATH="$2"
      shift 2
      ;;
    --policy)
      POLICY_PATH="$2"
      shift 2
      ;;
    --window-runs)
      WINDOW_RUNS="$2"
      shift 2
      ;;
    --window-seconds)
      WINDOW_SECONDS="$2"
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
if [[ -n "${LEDGER_PATH}" ]]; then
  VALIDATOR_ARGS+=(--ledger "${LEDGER_PATH}")
fi
if [[ -n "${POLICY_PATH}" ]]; then
  VALIDATOR_ARGS+=(--policy "${POLICY_PATH}")
fi
if [[ -n "${WINDOW_RUNS}" ]]; then
  VALIDATOR_ARGS+=(--window-runs "${WINDOW_RUNS}")
fi
if [[ -n "${WINDOW_SECONDS}" ]]; then
  VALIDATOR_ARGS+=(--window-seconds "${WINDOW_SECONDS}")
fi

set +e
cargo run --quiet --manifest-path "${ROOT}/ayken-core/Cargo.toml" \
  -p proof-verifier \
  --bin verifier-cartel-correlation \
  -- "${VALIDATOR_ARGS[@]}"
VALIDATOR_RC=$?
set -e

{
  echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "validator_rc=${VALIDATOR_RC}"
  echo "artifact_root=${ARTIFACT_ROOT}"
  echo "evidence_dir=${EVIDENCE_DIR}"
  if [[ -n "${LEDGER_PATH}" ]]; then
    echo "ledger_path=${LEDGER_PATH}"
  fi
  if [[ -n "${POLICY_PATH}" ]]; then
    echo "policy_path=${POLICY_PATH}"
  fi
  if [[ -n "${WINDOW_RUNS}" ]]; then
    echo "window_runs=${WINDOW_RUNS}"
  fi
  if [[ -n "${WINDOW_SECONDS}" ]]; then
    echo "window_seconds=${WINDOW_SECONDS}"
  fi
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${EVIDENCE_DIR}/violations.txt" 2>/dev/null || true)"
  echo "verifier-cartel-correlation: FAIL (${COUNT} violations)"
  exit 2
fi

echo "verifier-cartel-correlation: PASS"
exit 0

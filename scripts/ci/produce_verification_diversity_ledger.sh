#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/produce_verification_diversity_ledger.sh \
    --evidence-dir evidence/run-<id>/producers/verification-diversity-ledger \
    --artifact-root /path/to/artifacts \
    [--audit-ledger /path/to/verification_audit_ledger.jsonl] \
    [--binding /path/to/verification_diversity_ledger_binding.json] \
    [--ledger /path/to/verification_diversity_ledger.json]

Exit codes:
  0: producer success
  2: producer failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
ARTIFACT_ROOT=""
AUDIT_LEDGER_PATH=""
BINDING_PATH=""
LEDGER_PATH=""

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
    --audit-ledger)
      AUDIT_LEDGER_PATH="$2"
      shift 2
      ;;
    --binding)
      BINDING_PATH="$2"
      shift 2
      ;;
    --ledger)
      LEDGER_PATH="$2"
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
if [[ -n "${AUDIT_LEDGER_PATH}" ]]; then
  VALIDATOR_ARGS+=(--audit-ledger "${AUDIT_LEDGER_PATH}")
fi
if [[ -n "${BINDING_PATH}" ]]; then
  VALIDATOR_ARGS+=(--binding "${BINDING_PATH}")
fi
if [[ -n "${LEDGER_PATH}" ]]; then
  VALIDATOR_ARGS+=(--ledger "${LEDGER_PATH}")
fi

set +e
cargo run --quiet --manifest-path "${ROOT}/ayken-core/Cargo.toml" \
  -p proof-verifier \
  --bin verification-diversity-ledger-producer \
  -- "${VALIDATOR_ARGS[@]}"
VALIDATOR_RC=$?
set -e

{
  echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "validator_rc=${VALIDATOR_RC}"
  echo "artifact_root=${ARTIFACT_ROOT}"
  echo "evidence_dir=${EVIDENCE_DIR}"
  if [[ -n "${AUDIT_LEDGER_PATH}" ]]; then
    echo "audit_ledger_path=${AUDIT_LEDGER_PATH}"
  fi
  if [[ -n "${BINDING_PATH}" ]]; then
    echo "binding_path=${BINDING_PATH}"
  fi
  if [[ -n "${LEDGER_PATH}" ]]; then
    echo "ledger_path=${LEDGER_PATH}"
  fi
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${EVIDENCE_DIR}/violations.txt" 2>/dev/null || true)"
  echo "verification-diversity-ledger-producer: FAIL (${COUNT} violations)"
  exit 2
fi

echo "verification-diversity-ledger-producer: PASS"
exit 0

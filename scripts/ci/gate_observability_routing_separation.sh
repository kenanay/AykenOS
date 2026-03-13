#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_observability_routing_separation.sh \
    --evidence-dir evidence/run-<id>/gates/observability-routing-separation \
    [--source-root /path/to/source-root] \
    [--scan-root relative/root ...] \
    [--source-path relative/file.rs ...]

Exit codes:
  0: pass
  2: observability routing separation gate failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
SOURCE_ROOT="${ROOT}"
SCAN_ROOT_ARGS=()
SOURCE_PATH_ARGS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --source-root)
      SOURCE_ROOT="$2"
      shift 2
      ;;
    --scan-root)
      SCAN_ROOT_ARGS+=("--scan-root" "$2")
      shift 2
      ;;
    --source-path)
      SOURCE_PATH_ARGS+=("--source-path" "$2")
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

mkdir -p "${EVIDENCE_DIR}"

REPORT_JSON="${EVIDENCE_DIR}/report.json"
DETAIL_REPORT_JSON="${EVIDENCE_DIR}/observability_routing_separation_report.json"
NEGATIVE_MATRIX_JSON="${EVIDENCE_DIR}/observability_routing_negative_matrix.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

VALIDATOR_ARGS=(
  --source-root "${SOURCE_ROOT}"
)
if ((${#SCAN_ROOT_ARGS[@]} > 0)); then
  VALIDATOR_ARGS+=("${SCAN_ROOT_ARGS[@]}")
fi
if ((${#SOURCE_PATH_ARGS[@]} > 0)); then
  VALIDATOR_ARGS+=("${SOURCE_PATH_ARGS[@]}")
fi

set +e
python3 "${ROOT}/tools/ci/validate_observability_routing_separation.py" \
  "${VALIDATOR_ARGS[@]}" \
  --out-report "${REPORT_JSON}" \
  --out-detail-report "${DETAIL_REPORT_JSON}" \
  --out-negative-matrix "${NEGATIVE_MATRIX_JSON}" \
  --violations-out "${VIOLATIONS_TXT}"
VALIDATOR_RC=$?
set -e

{
  echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "validator_rc=${VALIDATOR_RC}"
  echo "source_root=${SOURCE_ROOT}"
  echo "evidence_dir=${EVIDENCE_DIR}"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "observability-routing-separation: FAIL (${COUNT} violations)"
  exit 2
fi

echo "observability-routing-separation: PASS"
exit 0

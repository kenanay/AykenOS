#!/usr/bin/env bash
set -euo pipefail

# Author: Kenan AY

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/verify_proof_bundle.sh \
    --bundle-root <proof-bundle-dir> \
    --out-dir <verification-output-dir>

Exit codes:
  0: pass
  2: proof bundle verification failure
  3: usage/tooling error
USAGE
}

BUNDLE_ROOT=""
OUT_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bundle-root)
      BUNDLE_ROOT="$2"
      shift 2
      ;;
    --out-dir)
      OUT_DIR="$2"
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

if [[ -z "${BUNDLE_ROOT}" || -z "${OUT_DIR}" ]]; then
  usage
  exit 3
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: missing required tool: python3" >&2
  exit 3
fi

VALIDATOR="${ROOT}/tools/ci/validate_proof_bundle.py"
if [[ ! -f "${VALIDATOR}" ]]; then
  echo "ERROR: missing validator: ${VALIDATOR}" >&2
  exit 3
fi
if [[ ! -d "${BUNDLE_ROOT}" ]]; then
  echo "ERROR: missing_bundle_root:${BUNDLE_ROOT}" >&2
  exit 3
fi

mkdir -p "${OUT_DIR}"

BUNDLE_VERIFY_JSON="${OUT_DIR}/bundle_verify.json"
REPORT_JSON="${OUT_DIR}/report.json"
VIOLATIONS_TXT="${OUT_DIR}/violations.txt"
META_TXT="${OUT_DIR}/meta.txt"

set +e
python3 "${VALIDATOR}" verify \
  --bundle-root "${BUNDLE_ROOT}" \
  --out-bundle-verify-json "${BUNDLE_VERIFY_JSON}" \
  --out-report "${REPORT_JSON}"
VALIDATOR_RC=$?
set -e

if [[ ! -f "${REPORT_JSON}" || ! -f "${BUNDLE_VERIFY_JSON}" ]]; then
  echo "ERROR: verifier did not produce required outputs" >&2
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
  echo "bundle_root=${BUNDLE_ROOT}"
  echo "validator_rc=${VALIDATOR_RC}"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "proof-bundle-verify: FAIL (${COUNT} violations)"
  exit 2
fi

echo "proof-bundle-verify: PASS"
exit 0

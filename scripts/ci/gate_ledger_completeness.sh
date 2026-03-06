#!/usr/bin/env bash
set -euo pipefail

# Author: Kenan AY

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_ledger_completeness.sh \
    --evidence-dir evidence/run-<id>/gates/ledger-v1 \
    --phase10a2-evidence evidence/run-<id>/gates/ring3-execution-phase10a2 \
    [--require-eti-binding 0|1] \
    [--eti-events <path>]

Exit codes:
  0: pass
  2: ledger completeness contract failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
A2_EVIDENCE_DIR=""
REQUIRE_ETI_BINDING="${PHASE11_LEDGER_REQUIRE_ETI:-0}"
ETI_EVENTS_PATH="${PHASE11_LEDGER_ETI_EVENTS:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --phase10a2-evidence)
      A2_EVIDENCE_DIR="$2"
      shift 2
      ;;
    --require-eti-binding)
      REQUIRE_ETI_BINDING="$2"
      shift 2
      ;;
    --eti-events)
      ETI_EVENTS_PATH="$2"
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

if [[ -z "${EVIDENCE_DIR}" || -z "${A2_EVIDENCE_DIR}" ]]; then
  usage
  exit 3
fi
if ! [[ "${REQUIRE_ETI_BINDING}" =~ ^[01]$ ]]; then
  echo "ERROR: --require-eti-binding must be 0 or 1 (current=${REQUIRE_ETI_BINDING})" >&2
  exit 3
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: missing required tool: python3" >&2
  exit 3
fi

VALIDATOR="${ROOT}/tools/ci/validate_ledger_completeness.py"
if [[ ! -f "${VALIDATOR}" ]]; then
  echo "ERROR: missing validator: ${VALIDATOR}" >&2
  exit 3
fi

A2_EVENTS="${A2_EVIDENCE_DIR}/events.jsonl"
A2_MARKER_LOG="${A2_EVIDENCE_DIR}/marker.log"

if [[ ! -s "${A2_EVENTS}" ]]; then
  echo "ERROR: missing_or_empty_events:${A2_EVENTS}" >&2
  exit 3
fi
if [[ ! -s "${A2_MARKER_LOG}" ]]; then
  echo "ERROR: missing_or_empty_marker_log:${A2_MARKER_LOG}" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

REPORT_JSON="${EVIDENCE_DIR}/report.json"
LEDGER_JSONL="${EVIDENCE_DIR}/decision_ledger.jsonl"
LEDGER_BIN="${EVIDENCE_DIR}/decision_ledger.bin"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

set +e
VALIDATOR_ARGS=(
  --events "${A2_EVENTS}"
  --log "${A2_MARKER_LOG}"
  --out-report "${REPORT_JSON}"
  --out-ledger-jsonl "${LEDGER_JSONL}"
  --out-ledger-bin "${LEDGER_BIN}"
  --require-eti-binding "${REQUIRE_ETI_BINDING}"
)
if [[ -n "${ETI_EVENTS_PATH}" ]]; then
  VALIDATOR_ARGS+=(--eti-events "${ETI_EVENTS_PATH}")
fi

python3 "${VALIDATOR}" "${VALIDATOR_ARGS[@]}"
VALIDATOR_RC=$?
set -e

if [[ ! -f "${REPORT_JSON}" ]]; then
  echo "ERROR: validator did not produce report: ${REPORT_JSON}" >&2
  exit 3
fi
if [[ ! -f "${LEDGER_JSONL}" ]]; then
  echo "ERROR: validator did not produce ledger jsonl: ${LEDGER_JSONL}" >&2
  exit 3
fi
if [[ ! -f "${LEDGER_BIN}" ]]; then
  echo "ERROR: validator did not produce ledger binary: ${LEDGER_BIN}" >&2
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
  echo "phase10a2_events=${A2_EVENTS}"
  echo "phase10a2_marker_log=${A2_MARKER_LOG}"
  echo "require_eti_binding=${REQUIRE_ETI_BINDING}"
  echo "eti_events=${ETI_EVENTS_PATH:-none}"
  echo "validator_rc=${VALIDATOR_RC}"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "ledger-completeness: FAIL (${COUNT} violations)"
  exit 2
fi

echo "ledger-completeness: PASS"
exit 0

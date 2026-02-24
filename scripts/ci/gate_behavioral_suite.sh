#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_behavioral_suite.sh --evidence-dir evidence/run-<id>/gates/behavioral-suite [--phase <n>] [--suite <path>]

Exit codes:
  0: pass/warn
  1: tooling or infra failure
  2: behavioral proof violations
  3: usage error
USAGE
}

EVIDENCE_DIR=""
PHASE="${BEHAVIORAL_SUITE_PHASE:-5}"
SUITE_FILE="${ROOT}/constitution/behavioral_contracts/suite.json"
KERNEL_PROFILE="${KERNEL_PROFILE:-validation}"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)-$(git -C "${ROOT}" rev-parse --short HEAD 2>/dev/null || echo nogit)}"
DRIFT_HISTORY_ROOT="${DRIFT_HISTORY_ROOT:-${ROOT}/evidence/history}"
DRIFT_AI_POLICY_HASH="${DRIFT_AI_POLICY_HASH:-unknown}"
DRIFT_WORKLOAD_ID="${DRIFT_WORKLOAD_ID:-default}"
if [[ -n "${DRIFT_RUN_CLASS:-}" ]]; then
  DRIFT_RUN_CLASS="${DRIFT_RUN_CLASS}"
elif [[ "${CI:-}" == "true" ]]; then
  DRIFT_RUN_CLASS="ci"
else
  DRIFT_RUN_CLASS="local"
fi
DRIFT_PROFILE_FILE="${DRIFT_PROFILE_FILE:-}"
DRIFT_MARKER_SCHEMA_VERSION="${DRIFT_MARKER_SCHEMA_VERSION:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --phase)
      PHASE="$2"
      shift 2
      ;;
    --suite)
      SUITE_FILE="$2"
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
if [[ ! "${PHASE}" =~ ^[0-9]+$ ]]; then
  echo "ERROR: --phase must be numeric" >&2
  exit 3
fi
if [[ "${KERNEL_PROFILE}" != "validation" ]]; then
  echo "ERROR: behavioral-suite requires KERNEL_PROFILE=validation (current=${KERNEL_PROFILE})" >&2
  exit 2
fi

for tool in python3 git; do
  if ! command -v "${tool}" >/dev/null 2>&1; then
    echo "ERROR: missing required tool: ${tool}" >&2
    exit 1
  fi
done

BOOT_AUDIT_SCRIPT="${ROOT}/tools/validation/phase_4_4_qemu_boot_audit.sh"
EXTRACT_SCRIPT="${ROOT}/tools/ci/extract_markers.py"
SCORE_SCRIPT="${ROOT}/tools/ci/score_behavioral_proofs.py"
if [[ ! -x "${BOOT_AUDIT_SCRIPT}" ]]; then
  echo "ERROR: missing boot audit script: ${BOOT_AUDIT_SCRIPT}" >&2
  exit 1
fi
if [[ ! -f "${EXTRACT_SCRIPT}" ]]; then
  echo "ERROR: missing marker extractor: ${EXTRACT_SCRIPT}" >&2
  exit 1
fi
if [[ ! -f "${SCORE_SCRIPT}" ]]; then
  echo "ERROR: missing behavioral scorer: ${SCORE_SCRIPT}" >&2
  exit 1
fi
if [[ ! -f "${SUITE_FILE}" ]]; then
  echo "ERROR: missing suite file: ${SUITE_FILE}" >&2
  exit 1
fi

mkdir -p "${EVIDENCE_DIR}"

BOOT_AUDIT_DIR="${EVIDENCE_DIR}/boot-audit"
BOOT_LOG="${EVIDENCE_DIR}/boot.log"
COMBINED_LOG="${EVIDENCE_DIR}/combined.log"
EVENTS_JSONL="${EVIDENCE_DIR}/events.jsonl"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
SEED_VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.seed.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"

: > "${BOOT_LOG}"
: > "${COMBINED_LOG}"
: > "${EVENTS_JSONL}"
: > "${VIOLATIONS_TXT}"
: > "${SEED_VIOLATIONS_TXT}"
: > "${META_TXT}"

INFRA_FAILURE=0

mkdir -p "${BOOT_AUDIT_DIR}"
"${BOOT_AUDIT_SCRIPT}" --out-dir "${BOOT_AUDIT_DIR}" > "${BOOT_LOG}" 2>&1 || true

SERIAL_LOG="${BOOT_AUDIT_DIR}/qemu_serial.log"
DEBUGCON_LOG="${BOOT_AUDIT_DIR}/qemu_debugcon.log"
cat "${SERIAL_LOG}" "${DEBUGCON_LOG}" 2>/dev/null > "${COMBINED_LOG}" || true

if [[ ! -s "${COMBINED_LOG}" ]]; then
  echo "combined_log_empty" >> "${SEED_VIOLATIONS_TXT}"
fi

if ! python3 "${EXTRACT_SCRIPT}" --log "${COMBINED_LOG}" --out "${EVENTS_JSONL}"; then
  echo "extract_markers_failed" >> "${SEED_VIOLATIONS_TXT}"
  INFRA_FAILURE=1
fi

set +e
SCORER_CMD=(
  python3 "${SCORE_SCRIPT}"
  --events "${EVENTS_JSONL}"
  --suite "${SUITE_FILE}"
  --phase "${PHASE}"
  --run-id "${RUN_ID}"
  --kernel-profile "${KERNEL_PROFILE}"
  --history-root "${DRIFT_HISTORY_ROOT}"
  --ai-policy-hash "${DRIFT_AI_POLICY_HASH}"
  --workload-id "${DRIFT_WORKLOAD_ID}"
  --run-class "${DRIFT_RUN_CLASS}"
  --seed-violations "${SEED_VIOLATIONS_TXT}"
  --out "${REPORT_JSON}"
  --violations-out "${VIOLATIONS_TXT}"
)
if [[ -n "${DRIFT_PROFILE_FILE}" ]]; then
  SCORER_CMD+=(--drift-profile-file "${DRIFT_PROFILE_FILE}")
fi
if [[ -n "${DRIFT_MARKER_SCHEMA_VERSION}" ]]; then
  SCORER_CMD+=(--marker-schema-version "${DRIFT_MARKER_SCHEMA_VERSION}")
fi
"${SCORER_CMD[@]}"
SCORER_RC=$?
set -e

if [[ "${SCORER_RC}" -eq 1 ]]; then
  INFRA_FAILURE=1
fi

if [[ ! -f "${REPORT_JSON}" ]]; then
  INFRA_FAILURE=1
  cat > "${REPORT_JSON}" <<EOF
{
  "gate": "behavioral-suite",
  "tier": "tier-3-behavioral",
  "run_id": "${RUN_ID}",
  "kernel_profile": "${KERNEL_PROFILE}",
  "phase": "${PHASE}",
  "verdict": "FAIL",
  "violations_count": 1,
  "violations": [
    "report_missing"
  ]
}
EOF
  echo "report_missing" >> "${VIOLATIONS_TXT}"
fi

NOW="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
GIT_SHA="$(git -C "${ROOT}" rev-parse HEAD 2>/dev/null || echo NO_GIT)"
{
  echo "time_utc=${NOW}"
  echo "git_sha=${GIT_SHA}"
  echo "run_id=${RUN_ID}"
  echo "phase=${PHASE}"
  echo "kernel_profile=${KERNEL_PROFILE}"
  echo "suite=${SUITE_FILE}"
  echo "drift_history_root=${DRIFT_HISTORY_ROOT}"
  echo "drift_ai_policy_hash=${DRIFT_AI_POLICY_HASH}"
  echo "drift_workload_id=${DRIFT_WORKLOAD_ID}"
  echo "drift_run_class=${DRIFT_RUN_CLASS}"
  echo "drift_profile_file=${DRIFT_PROFILE_FILE:-auto}"
  echo "scorer_rc=${SCORER_RC}"
  echo "infra_failure=${INFRA_FAILURE}"
} > "${META_TXT}"

VERDICT="$(python3 - "${REPORT_JSON}" <<'PY'
import json
import sys
path = sys.argv[1]
try:
    with open(path, "r", encoding="utf-8", errors="replace") as fh:
        print(str(json.load(fh).get("verdict", "FAIL")))
except Exception:
    print("FAIL")
PY
)"

if [[ "${INFRA_FAILURE}" -eq 1 ]]; then
  echo "behavioral-suite: INFRA FAIL"
  echo "See: ${VIOLATIONS_TXT}"
  exit 1
fi

if [[ "${SCORER_RC}" -eq 2 || "${VERDICT}" == "FAIL" ]]; then
  COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
  echo "behavioral-suite: FAIL (${COUNT} violations)"
  echo "See: ${VIOLATIONS_TXT}"
  exit 2
fi

echo "behavioral-suite: ${VERDICT}"
exit 0

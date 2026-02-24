#!/usr/bin/env bash
# Drift Activation Gate (Requirement Enforcement Only)
# Authority: ARCHITECTURE_FREEZE.md
# Responsibility: Enforce drift blocking activation requirement (Phase >= 9)
# Does NOT perform drift detection or N-run persistence.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CI_TOOLS="${ROOT}/tools/ci"

source "${CI_TOOLS}/lib.sh"
source "${ROOT}/scripts/ci/lib-phase.sh"

usage() {
    cat <<'USAGE'
Usage: scripts/ci/gate_drift_activation.sh --evidence-dir <path>

Responsibility:
  Enforce drift blocking activation requirement (Phase >= 9).
  Does NOT perform drift detection or N-run persistence.
  Drift detection is handled by ci-gate-performance.

Exit codes:
  0: pass/skip
  2: drift activation violations
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --evidence-dir) EVIDENCE_DIR="$2"; shift 2 ;;
        -h|--help) usage; exit 0 ;;
        *) echo "Unknown arg: $1" >&2; usage; exit 3 ;;
    esac
done

if [[ -z "${EVIDENCE_DIR}" ]]; then
    usage
    exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

PHASE_FILE="${ROOT}/docs/roadmap/CURRENT_PHASE"
ACTIVATION_FILE="${ROOT}/constitution/drift_blocking_activation.md"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
META_TXT="${EVIDENCE_DIR}/meta.txt"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"

NOW="$(ci_now_utc)"
GIT_SHA="$(git -C "${ROOT}" rev-parse HEAD 2>/dev/null || echo NO_GIT)"

# Get current phase
if ! CURRENT_PHASE=$(get_current_phase); then
    echo "ERROR: Failed to detect current phase" >&2
    exit 3
fi

# Parse activation state
if [[ ! -f "${ACTIVATION_FILE}" ]]; then
    echo "ERROR: Activation file not found: ${ACTIVATION_FILE}" >&2
    exit 3
fi

ENABLED=$(grep -E "^enabled:\s*(true|false)" "${ACTIVATION_FILE}" | \
          grep -oE "(true|false)" || echo "false")
PHASE_MIN=$(grep -E "^phase_minimum:\s*[0-9]+" "${ACTIVATION_FILE}" | \
            grep -oE "[0-9]+" || echo "9")

# Enforcement logic (activation requirement only)
VERDICT="SKIP"
REASON=""
VIOLATIONS=()

if [[ "${CURRENT_PHASE}" -lt "${PHASE_MIN}" ]]; then
    VERDICT="SKIP"
    REASON="phase_below_minimum"
elif [[ "${ENABLED}" == "false" ]]; then
    VERDICT="FAIL"
    REASON="drift_blocking_required_but_disabled"
    VIOLATIONS+=("phase=${CURRENT_PHASE}:enabled=false:required=true")
else
    VERDICT="PASS"
    REASON="drift_blocking_enabled"
fi

# Write evidence
cat > "${META_TXT}" <<META
time_utc=${NOW}
git_sha=${GIT_SHA}
current_phase=${CURRENT_PHASE}
phase_minimum=${PHASE_MIN}
enabled=${ENABLED}
verdict=${VERDICT}
reason=${REASON}
META

if [[ ${#VIOLATIONS[@]} -gt 0 ]]; then
    printf "%s\n" "${VIOLATIONS[@]}" > "${VIOLATIONS_TXT}"
    VIOLATIONS_STR="${VIOLATIONS[*]}"
else
    : > "${VIOLATIONS_TXT}"
    VIOLATIONS_STR=""
fi

python3 - <<'PY' "${REPORT_JSON}" "${VERDICT}" "${REASON}" "${CURRENT_PHASE}" "${PHASE_MIN}" "${ENABLED}" "${VIOLATIONS_STR}"
import json
import sys

path, verdict, reason, phase, phase_min, enabled = sys.argv[1:7]
violations_str = sys.argv[7] if len(sys.argv) > 7 else ""
violations = [v for v in violations_str.split() if v] if violations_str else []

report = {
    "gate": "drift-activation",
    "verdict": verdict,
    "reason": reason,
    "current_phase": int(phase),
    "phase_minimum": int(phase_min),
    "enabled": enabled == "true",
    "violations": violations,
    "note": "This gate enforces activation requirement only. Drift detection is handled by ci-gate-performance."
}

with open(path, "w", encoding="utf-8") as fh:
    json.dump(report, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY

if [[ "${VERDICT}" == "FAIL" ]]; then
    echo "drift-activation: FAIL (${REASON})"
    exit 2
elif [[ "${VERDICT}" == "SKIP" ]]; then
    echo "drift-activation: SKIP (${REASON})"
    exit 0
else
    echo "drift-activation: PASS"
    exit 0
fi

#!/usr/bin/env bash
# ============================================================================
# Gate-4.5: Decision -> Switch Proof (Isolated)
# ============================================================================
# Purpose:
#   Prove Ring3 publish -> Ring0 validate(ACCEPT) -> scheduler decision ->
#   actual context-switch boundary marker in strict order.
#
# Contract (selftest=0):
#   - exactly 1 target ACCEPT (epoch=1)
#   - exactly 1 arbiter decision marker
#   - exactly 1 ctx-switch proof marker
#   - publish < accept < arbiter < switch
#   - from != to and decision/switch endpoints match
# ============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "${REPO_ROOT}"

RUN_ID="${RUN_ID:-gate45-$(date -u +%Y%m%dT%H%M%SZ)}"
KERNEL_PROFILE="${KERNEL_PROFILE:-validation}"
QEMU_TIMEOUT="${QEMU_TIMEOUT:-20}"
GATE4_BOOTSTRAP_POLICY="${GATE4_BOOTSTRAP_POLICY:-1}"
GATE4_MB_SELFTEST="${GATE4_MB_SELFTEST:-0}"

EVIDENCE_ROOT="evidence/gate-4.5-decision-switch-proof"
EVIDENCE_DIR="${EVIDENCE_ROOT}/${RUN_ID}"
mkdir -p "${EVIDENCE_DIR}"
ln -sfn "${RUN_ID}" "${EVIDENCE_ROOT}/latest"

GATE4_EVIDENCE_DIR="evidence/gate-4-policy-accept/${RUN_ID}"
GATE4_REPORT="${GATE4_EVIDENCE_DIR}/report.json"
DEBUGCON_LOG="${GATE4_EVIDENCE_DIR}/debugcon.log"

LOG_FILE="${EVIDENCE_DIR}/gate45.log"
VIOLATIONS="${EVIDENCE_DIR}/violations.txt"
REPORT_JSON="${EVIDENCE_DIR}/report.json"

: > "${LOG_FILE}"
: > "${VIOLATIONS}"

safe_count_file() {
    local pattern="$1"
    local file="$2"
    local count
    count=$(grep -a -cE "$pattern" "$file" 2>/dev/null || true)
    count=$(printf "%s" "$count" | tr -dc '0-9')
    if [[ -z "$count" ]]; then
        count=0
    fi
    echo "$count"
}

line_of_first() {
    local pattern="$1"
    local file="$2"
    local line
    line=$(grep -a -n -E "$pattern" "$file" 2>/dev/null | head -n1 | cut -d: -f1 || true)
    line=$(printf "%s" "$line" | tr -dc '0-9')
    if [[ -z "$line" ]]; then
        line=0
    fi
    echo "$line"
}

extract_field() {
    local text="$1"
    local key="$2"
    printf "%s" "$text" | sed -n "s/.*${key}=\([0-9][0-9]*\).*/\1/p"
}

echo "== GATE-4.5: DECISION -> SWITCH PROOF =="
echo "run_id: ${RUN_ID}"
echo "kernel_profile: ${KERNEL_PROFILE}"
echo "qemu_timeout: ${QEMU_TIMEOUT}s"
echo "gate4_bootstrap_policy: ${GATE4_BOOTSTRAP_POLICY}"
echo "gate4_mb_selftest: ${GATE4_MB_SELFTEST}"
echo "evidence_dir: ${EVIDENCE_DIR}"

echo "[*] Running Gate-4 prerequisite with AYKEN_GATE45_PROOF=1..."
set +e
RUN_ID="${RUN_ID}" \
KERNEL_PROFILE="${KERNEL_PROFILE}" \
QEMU_TIMEOUT="${QEMU_TIMEOUT}" \
GATE4_BOOTSTRAP_POLICY="${GATE4_BOOTSTRAP_POLICY}" \
GATE4_MB_SELFTEST="${GATE4_MB_SELFTEST}" \
AYKEN_GATE45_PROOF=1 \
bash scripts/ci/gate_4_policy_accept.sh > "${LOG_FILE}" 2>&1
GATE4_RC=$?
set -e

if [[ "${GATE4_RC}" -ne 0 ]]; then
    echo "gate4_prerequisite_failed:rc=${GATE4_RC}" >> "${VIOLATIONS}"
fi
if [[ ! -f "${GATE4_REPORT}" ]]; then
    echo "gate4_report_missing" >> "${VIOLATIONS}"
fi
if [[ ! -f "${DEBUGCON_LOG}" ]]; then
    echo "gate4_debugcon_missing" >> "${VIOLATIONS}"
fi

GATE4_PID=0
if [[ -f "${GATE4_REPORT}" ]]; then
    GATE4_PID=$(python3 - <<'PY' "${GATE4_REPORT}"
import json, sys
p = sys.argv[1]
try:
    data = json.load(open(p, encoding='utf-8'))
    print(int(data.get('gate4_pid', 0) or 0))
except Exception:
    print(0)
PY
)
    GATE4_PID=$(printf "%s" "${GATE4_PID}" | tr -dc '0-9')
    if [[ -z "${GATE4_PID}" ]]; then
        GATE4_PID=0
    fi
fi
if [[ "${GATE4_PID}" -le 0 ]]; then
    echo "gate4_pid_invalid" >> "${VIOLATIONS}"
fi

PUBLISH_LINE=0
ACCEPT_LINE=0
ARBITER_LINE=0
SWITCH_LINE=0
TARGET_ACCEPT_COUNT=0
ARBITER_COUNT=0
SWITCH_COUNT=0
ARBITER_FROM=0
ARBITER_TO=0
ARBITER_EPOCH=0
SWITCH_FROM=0
SWITCH_TO=0

if [[ ! -s "${VIOLATIONS}" ]]; then
    PUBLISH_LINE=$(line_of_first "\\[\\[AYKEN_RING3_PUBLISH\\]\\] pid=${GATE4_PID} epoch=1" "${DEBUGCON_LOG}")
    ACCEPT_LINE=$(line_of_first "\\[\\[AYKEN_SCHED_MB_ACCEPT\\]\\] pid=${GATE4_PID} epoch=1" "${DEBUGCON_LOG}")
    TARGET_ACCEPT_COUNT=$(safe_count_file "\\[\\[AYKEN_SCHED_MB_ACCEPT\\]\\] pid=${GATE4_PID} epoch=1" "${DEBUGCON_LOG}")

    ARBITER_COUNT=$(safe_count_file "\\[\\[AYKEN_SCHED_ARBITER_DECISION\\]\\] from=[0-9]+ to=[0-9]+ epoch=[0-9]+" "${DEBUGCON_LOG}")
    SWITCH_COUNT=$(safe_count_file "\\[\\[AYKEN_CTX_SWITCH\\]\\] from=[0-9]+ to=[0-9]+" "${DEBUGCON_LOG}")

    ARBITER_LINE=$(line_of_first "\\[\\[AYKEN_SCHED_ARBITER_DECISION\\]\\] from=[0-9]+ to=[0-9]+ epoch=[0-9]+" "${DEBUGCON_LOG}")
    SWITCH_LINE=$(line_of_first "\\[\\[AYKEN_CTX_SWITCH\\]\\] from=[0-9]+ to=[0-9]+" "${DEBUGCON_LOG}")

    if [[ "${ARBITER_COUNT}" -ge 1 ]]; then
        ARBITER_ROW=$(grep -a -E "\\[\\[AYKEN_SCHED_ARBITER_DECISION\\]\\] from=[0-9]+ to=[0-9]+ epoch=[0-9]+" "${DEBUGCON_LOG}" | head -n1 || true)
        ARBITER_FROM=$(extract_field "${ARBITER_ROW}" "from")
        ARBITER_TO=$(extract_field "${ARBITER_ROW}" "to")
        ARBITER_EPOCH=$(extract_field "${ARBITER_ROW}" "epoch")
        ARBITER_FROM=${ARBITER_FROM:-0}
        ARBITER_TO=${ARBITER_TO:-0}
        ARBITER_EPOCH=${ARBITER_EPOCH:-0}
    fi
    if [[ "${SWITCH_COUNT}" -ge 1 ]]; then
        SWITCH_ROW=$(grep -a -E "\\[\\[AYKEN_CTX_SWITCH\\]\\] from=[0-9]+ to=[0-9]+" "${DEBUGCON_LOG}" | head -n1 || true)
        SWITCH_FROM=$(extract_field "${SWITCH_ROW}" "from")
        SWITCH_TO=$(extract_field "${SWITCH_ROW}" "to")
        SWITCH_FROM=${SWITCH_FROM:-0}
        SWITCH_TO=${SWITCH_TO:-0}
    fi

    if [[ "${TARGET_ACCEPT_COUNT}" -ne 1 ]]; then
        echo "target_accept_mismatch:count=${TARGET_ACCEPT_COUNT}" >> "${VIOLATIONS}"
    fi
    if [[ "${ARBITER_COUNT}" -ne 1 ]]; then
        echo "arbiter_decision_mismatch:count=${ARBITER_COUNT}" >> "${VIOLATIONS}"
    fi
    if [[ "${SWITCH_COUNT}" -ne 1 ]]; then
        echo "ctx_switch_mismatch:count=${SWITCH_COUNT}" >> "${VIOLATIONS}"
    fi

    if [[ "${PUBLISH_LINE}" -le 0 || "${ACCEPT_LINE}" -le 0 || "${ARBITER_LINE}" -le 0 || "${SWITCH_LINE}" -le 0 ]]; then
        echo "required_marker_missing" >> "${VIOLATIONS}"
    elif [[ ! ( "${PUBLISH_LINE}" -lt "${ACCEPT_LINE}" && "${ACCEPT_LINE}" -lt "${ARBITER_LINE}" && "${ARBITER_LINE}" -lt "${SWITCH_LINE}" ) ]]; then
        echo "marker_order_invalid:publish=${PUBLISH_LINE}:accept=${ACCEPT_LINE}:arbiter=${ARBITER_LINE}:switch=${SWITCH_LINE}" >> "${VIOLATIONS}"
    fi

    if [[ "${ARBITER_COUNT}" -ge 1 && "${SWITCH_COUNT}" -ge 1 ]]; then
        if [[ "${ARBITER_FROM}" -le 0 || "${ARBITER_TO}" -le 0 || "${SWITCH_FROM}" -le 0 || "${SWITCH_TO}" -le 0 ]]; then
            echo "marker_payload_invalid" >> "${VIOLATIONS}"
        fi
        if [[ "${ARBITER_FROM}" -eq "${ARBITER_TO}" ]]; then
            echo "arbiter_noop_forbidden:from=${ARBITER_FROM}:to=${ARBITER_TO}" >> "${VIOLATIONS}"
        fi
        if [[ "${SWITCH_FROM}" -eq "${SWITCH_TO}" ]]; then
            echo "ctx_switch_noop_forbidden:from=${SWITCH_FROM}:to=${SWITCH_TO}" >> "${VIOLATIONS}"
        fi
        if [[ "${ARBITER_FROM}" -ne "${SWITCH_FROM}" || "${ARBITER_TO}" -ne "${SWITCH_TO}" ]]; then
            echo "decision_switch_endpoint_mismatch:arbiter=${ARBITER_FROM}->${ARBITER_TO}:switch=${SWITCH_FROM}->${SWITCH_TO}" >> "${VIOLATIONS}"
        fi
        if [[ "${ARBITER_EPOCH}" -ne 1 ]]; then
            echo "arbiter_epoch_mismatch:epoch=${ARBITER_EPOCH}" >> "${VIOLATIONS}"
        fi
    fi
fi

VIOLATION_COUNT="$(wc -l < "${VIOLATIONS}" | tr -d '[:space:]')"
if [[ "${VIOLATION_COUNT}" -eq 0 ]]; then
    VERDICT="PASS"
    REASON="Gate-4.5 decision-to-switch proof validated"
else
    VERDICT="FAIL"
    REASON="${VIOLATION_COUNT} violations detected"
fi

cat > "${REPORT_JSON}" <<EOF_JSON
{
  "gate": "decision-switch-proof",
  "run_id": "${RUN_ID}",
  "verdict": "${VERDICT}",
  "reason": "${REASON}",
  "kernel_profile": "${KERNEL_PROFILE}",
  "gate4_bootstrap_policy": ${GATE4_BOOTSTRAP_POLICY},
  "gate4_mb_selftest": ${GATE4_MB_SELFTEST},
  "qemu_timeout": ${QEMU_TIMEOUT},
  "gate4_prereq_rc": ${GATE4_RC},
  "gate4_pid": ${GATE4_PID},
  "publish_line": ${PUBLISH_LINE},
  "accept_line": ${ACCEPT_LINE},
  "arbiter_line": ${ARBITER_LINE},
  "switch_line": ${SWITCH_LINE},
  "target_accept_count": ${TARGET_ACCEPT_COUNT},
  "arbiter_count": ${ARBITER_COUNT},
  "switch_count": ${SWITCH_COUNT},
  "arbiter_from": ${ARBITER_FROM},
  "arbiter_to": ${ARBITER_TO},
  "arbiter_epoch": ${ARBITER_EPOCH},
  "switch_from": ${SWITCH_FROM},
  "switch_to": ${SWITCH_TO},
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF_JSON

echo "Report: ${REPORT_JSON}"
cat "${REPORT_JSON}"

if [[ "${VERDICT}" != "PASS" ]]; then
    echo ""
    echo "Violations:"
    cat "${VIOLATIONS}"
    echo ""
    echo "Gate-4 log (tail 60):"
    tail -60 "${LOG_FILE}" || true
    exit 1
fi

echo "[PASS] Gate-4.5: Decision -> Switch proof PASS"

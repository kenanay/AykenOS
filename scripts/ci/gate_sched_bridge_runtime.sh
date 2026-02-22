#!/usr/bin/env bash
set -euo pipefail

echo "== CI GATE SCHED BRIDGE RUNTIME =="

RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)-$(git rev-parse --short HEAD)}"
EVIDENCE_DIR="evidence/run-${RUN_ID}/gates/sched-bridge-runtime"
mkdir -p "${EVIDENCE_DIR}"

LOG_FILE="${EVIDENCE_DIR}/boot.log"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS="${EVIDENCE_DIR}/violations.txt"

: > "${VIOLATIONS}"

# --- QEMU BOOT ---
# Run boot harness and capture output
tools/validation/phase_4_4_qemu_boot_audit.sh > "${LOG_FILE}" 2>&1 || true

# --- Extract actual serial/debugcon logs ---
# The boot harness creates logs in reports/phase_4_4_closure_*/
LATEST_REPORT=$(ls -td reports/phase_4_4_closure_* 2>/dev/null | head -1 || echo "")

if [[ -z "${LATEST_REPORT}" ]]; then
    echo "boot_harness_failed:no_report_directory" >> "${VIOLATIONS}"
    ACCEPT_COUNT=0
    REJECT_COUNT=0
else
    # Check both serial and debugcon logs for markers
    SERIAL_LOG="${LATEST_REPORT}/qemu_serial.log"
    DEBUGCON_LOG="${LATEST_REPORT}/qemu_debugcon.log"
    
    # Combine both logs for marker search
    COMBINED_LOG="${EVIDENCE_DIR}/combined.log"
    cat "${SERIAL_LOG}" "${DEBUGCON_LOG}" 2>/dev/null > "${COMBINED_LOG}" || touch "${COMBINED_LOG}"
    
    # --- Marker extraction ---
    ACCEPT_LINES=$(grep -E "\[\[AYKEN_SCHED_MB_ACCEPT\]\]" "${COMBINED_LOG}" || true)
    REJECT_LINES=$(grep -E "\[\[AYKEN_SCHED_MB_REJECT\]\]" "${COMBINED_LOG}" || true)
    
    ACCEPT_COUNT=$(echo "${ACCEPT_LINES}" | grep -c . || true)
    REJECT_COUNT=$(echo "${REJECT_LINES}" | grep -c . || true)
    
    # --- Fail if no markers at all ---
    if [[ "${ACCEPT_COUNT}" -eq 0 && "${REJECT_COUNT}" -eq 0 ]]; then
        echo "no_markers_detected" >> "${VIOLATIONS}"
    fi
    
    # --- ACCEPT must be exactly 1 ---
    if [[ "${ACCEPT_COUNT}" -ne 1 ]]; then
        echo "invalid_accept_count:expected=1:actual=${ACCEPT_COUNT}" >> "${VIOLATIONS}"
    fi
    
    # --- REJECT must be >= 2 ---
    if [[ "${REJECT_COUNT}" -lt 2 ]]; then
        echo "insufficient_reject_count:expected>=2:actual=${REJECT_COUNT}" >> "${VIOLATIONS}"
    fi
    
    # --- Epoch progression validation ---
    # Extract epoch numbers
    EPOCHS=$(grep -Eo "epoch=[0-9]+" "${COMBINED_LOG}" | cut -d= -f2 || true)
    
    if [[ -z "${EPOCHS}" ]]; then
        echo "epoch_missing" >> "${VIOLATIONS}"
    else
        PREV=""
        for E in ${EPOCHS}; do
            if [[ -n "${PREV}" && "${E}" -lt "${PREV}" ]]; then
                echo "epoch_not_monotonic:prev=${PREV}:current=${E}" >> "${VIOLATIONS}"
            fi
            PREV="${E}"
        done
    fi
fi

# --- Verdict ---
VIOLATION_COUNT=$(grep -c . "${VIOLATIONS}" || true)

if [[ "${VIOLATION_COUNT}" -eq 0 ]]; then
    VERDICT="PASS"
else
    VERDICT="FAIL"
fi

cat > "${REPORT_JSON}" <<EOF
{
  "run_id": "${RUN_ID}",
  "accept_count": ${ACCEPT_COUNT},
  "reject_count": ${REJECT_COUNT},
  "verdict": "${VERDICT}",
  "violations_count": ${VIOLATION_COUNT}
}
EOF

if [[ "${VERDICT}" == "FAIL" ]]; then
    echo "sched-bridge-runtime: FAIL (${VIOLATION_COUNT} violations)"
    echo "See: ${VIOLATIONS}"
    exit 2
fi

echo "sched-bridge-runtime: PASS"
exit 0

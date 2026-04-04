#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo "== CI GATE SCHED BRIDGE RUNTIME =="

# --- FAIL-CLOSED: Validation profile enforcement ---
# Self-test is compile-out in release, so this gate MUST run with validation profile
if [[ "${KERNEL_PROFILE:-}" != "validation" ]]; then
    echo "ERROR: sched-bridge-runtime gate requires KERNEL_PROFILE=validation"
    echo "Current: KERNEL_PROFILE=${KERNEL_PROFILE:-unset}"
    echo "Reason: Self-test markers are validation-only (compile-out in release)"
    exit 2
fi

EXPECTED_USER_MINIMAL_MODE="phase10a2"
OBSERVED_USER_MINIMAL_MODE="${USER_MINIMAL_MODE:-}"
if [[ "${OBSERVED_USER_MINIMAL_MODE}" != "${EXPECTED_USER_MINIMAL_MODE}" ]]; then
    echo "FATAL: sched-bridge-runtime gate invoked with USER_MINIMAL_MODE=${OBSERVED_USER_MINIMAL_MODE:-unset} (expected=${EXPECTED_USER_MINIMAL_MODE})"
    exit 2
fi

RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)-$(git rev-parse --short HEAD)}"
EVIDENCE_DIR="evidence/run-${RUN_ID}/gates/sched-bridge-runtime"
mkdir -p "${EVIDENCE_DIR}"

LOG_FILE="${EVIDENCE_DIR}/boot.log"
BUILD_LOG="${EVIDENCE_DIR}/build.log"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS="${EVIDENCE_DIR}/violations.txt"
BOOT_AUDIT_DIR="${EVIDENCE_DIR}/boot-audit"
ACCEPT_FORMAT_INVALID_TXT="${EVIDENCE_DIR}/accept-format-invalid.txt"
REJECT_FORMAT_INVALID_TXT="${EVIDENCE_DIR}/reject-format-invalid.txt"
RUNTIME_MARKER_CONTRACT_ENFORCE="${RUNTIME_MARKER_CONTRACT_ENFORCE:-1}"

: > "${VIOLATIONS}"
: > "${ACCEPT_FORMAT_INVALID_TXT}"
: > "${REJECT_FORMAT_INVALID_TXT}"
: > "${BUILD_LOG}"

ACCEPT_COUNT=0
REJECT_COUNT=0
ACCEPT_FORMAT_INVALID=0
REJECT_FORMAT_INVALID=0

if [[ "${RUNTIME_MARKER_CONTRACT_ENFORCE}" != "0" && "${RUNTIME_MARKER_CONTRACT_ENFORCE}" != "1" ]]; then
    echo "runtime_marker_contract_enforce_invalid:${RUNTIME_MARKER_CONTRACT_ENFORCE}" >> "${VIOLATIONS}"
    RUNTIME_MARKER_CONTRACT_ENFORCE=1
fi

# --- Deterministic build for this gate ---
MAKE_BUILD_ARGS=(
    -C "${ROOT}"
    "KERNEL_PROFILE=validation"
    "USER_MINIMAL_MODE=${EXPECTED_USER_MINIMAL_MODE}"
    "AYKEN_MB_SELFTEST=1"
    "AYKEN_SCHED_BOOTSTRAP_POLICY=${AYKEN_SCHED_BOOTSTRAP_POLICY:-0}"
)
if ! make "${MAKE_BUILD_ARGS[@]}" clean > "${BUILD_LOG}" 2>&1; then
    echo "build_failed:clean" >> "${VIOLATIONS}"
fi
if ! make "${MAKE_BUILD_ARGS[@]}" efi-img >> "${BUILD_LOG}" 2>&1; then
    echo "build_failed:efi-img" >> "${VIOLATIONS}"
fi
if [[ -s "${VIOLATIONS}" ]]; then
    VIOLATION_COUNT=$(grep -c . "${VIOLATIONS}" || true)
    cat > "${REPORT_JSON}" <<EOF
{
  "run_id": "${RUN_ID}",
  "user_minimal_mode": "${OBSERVED_USER_MINIMAL_MODE}",
  "runtime_marker_contract_enforce": ${RUNTIME_MARKER_CONTRACT_ENFORCE},
  "accept_count": ${ACCEPT_COUNT},
  "reject_count": ${REJECT_COUNT},
  "accept_format_invalid_count": ${ACCEPT_FORMAT_INVALID},
  "reject_format_invalid_count": ${REJECT_FORMAT_INVALID},
  "verdict": "FAIL",
  "violations_count": ${VIOLATION_COUNT}
}
EOF
    echo "sched-bridge-runtime: FAIL (${VIOLATION_COUNT} violations)"
    echo "See: ${VIOLATIONS}"
    exit 2
fi

# --- QEMU BOOT ---
# Run boot harness and capture output
mkdir -p "${BOOT_AUDIT_DIR}"
"${ROOT}/tools/validation/phase_4_4_qemu_boot_audit.sh" --out-dir "${BOOT_AUDIT_DIR}" > "${LOG_FILE}" 2>&1 || true

# --- Extract actual serial/debugcon logs ---
if [[ ! -d "${BOOT_AUDIT_DIR}" ]]; then
    echo "boot_harness_failed:no_report_directory" >> "${VIOLATIONS}"
else
    # Check both serial and debugcon logs for markers
    SERIAL_LOG="${BOOT_AUDIT_DIR}/qemu_serial.log"
    DEBUGCON_LOG="${BOOT_AUDIT_DIR}/qemu_debugcon.log"
    
    # Combine both logs for marker search
    COMBINED_LOG="${EVIDENCE_DIR}/combined.log"
    cat "${SERIAL_LOG}" "${DEBUGCON_LOG}" 2>/dev/null > "${COMBINED_LOG}" || touch "${COMBINED_LOG}"
    
    # --- Marker extraction ---
    ACCEPT_LINES=$(grep -E "\[\[AYKEN_SCHED_MB_ACCEPT\]\]" "${COMBINED_LOG}" || true)
    REJECT_LINES=$(grep -E "\[\[AYKEN_SCHED_MB_REJECT\]\]" "${COMBINED_LOG}" || true)
    
    ACCEPT_COUNT=$(echo "${ACCEPT_LINES}" | grep -c . || true)
    REJECT_COUNT=$(echo "${REJECT_LINES}" | grep -c . || true)

    if [[ "${RUNTIME_MARKER_CONTRACT_ENFORCE}" == "1" ]]; then
        MARKER_REGISTRY="${ROOT}/constitution/runtime_markers.json"
        if [[ ! -f "${MARKER_REGISTRY}" ]]; then
            echo "marker_registry_missing:${MARKER_REGISTRY}" >> "${VIOLATIONS}"
        else
            if ! python3 - "${MARKER_REGISTRY}" "${COMBINED_LOG}" "${ACCEPT_FORMAT_INVALID_TXT}" "${REJECT_FORMAT_INVALID_TXT}" <<'PY'
import json
import re
import sys

registry_path, log_path, accept_out, reject_out = sys.argv[1:5]
with open(registry_path, "r", encoding="utf-8", errors="replace") as fh:
    registry = json.load(fh)

markers = registry.get("runtime_markers")
if not isinstance(markers, list):
    raise SystemExit(2)

pattern_map = {}
for row in markers:
    if isinstance(row, dict):
        name = row.get("name")
        pattern = row.get("pattern")
        if isinstance(name, str) and isinstance(pattern, str):
            pattern_map[name] = re.compile(pattern)

accept_re = pattern_map.get("AYKEN_SCHED_MB_ACCEPT")
reject_re = pattern_map.get("AYKEN_SCHED_MB_REJECT")
if accept_re is None or reject_re is None:
    raise SystemExit(2)

accept_bad = []
reject_bad = []

with open(log_path, "r", encoding="utf-8", errors="replace") as fh:
    for raw in fh:
        line = raw.rstrip("\n")
        if "[[AYKEN_SCHED_MB_ACCEPT]]" in line:
            if not accept_re.fullmatch(line):
                accept_bad.append(line)
        if "[[AYKEN_SCHED_MB_REJECT]]" in line:
            if not reject_re.fullmatch(line):
                reject_bad.append(line)

with open(accept_out, "w", encoding="utf-8") as fh:
    for row in accept_bad:
        fh.write(row + "\n")

with open(reject_out, "w", encoding="utf-8") as fh:
    for row in reject_bad:
        fh.write(row + "\n")

raise SystemExit(0)
PY
            then
                echo "marker_registry_parse_or_contract_error" >> "${VIOLATIONS}"
            fi
        fi
    fi

    ACCEPT_FORMAT_INVALID=$(grep -c . "${ACCEPT_FORMAT_INVALID_TXT}" 2>/dev/null || true)
    REJECT_FORMAT_INVALID=$(grep -c . "${REJECT_FORMAT_INVALID_TXT}" 2>/dev/null || true)
    if [[ "${RUNTIME_MARKER_CONTRACT_ENFORCE}" == "1" && "${ACCEPT_FORMAT_INVALID}" -gt 0 ]]; then
        echo "accept_marker_format_invalid:count=${ACCEPT_FORMAT_INVALID}" >> "${VIOLATIONS}"
    fi
    if [[ "${RUNTIME_MARKER_CONTRACT_ENFORCE}" == "1" && "${REJECT_FORMAT_INVALID}" -gt 0 ]]; then
        echo "reject_marker_format_invalid:count=${REJECT_FORMAT_INVALID}" >> "${VIOLATIONS}"
    fi
    
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
    # Use only authoritative scheduler mailbox ACCEPT/REJECT markers for
    # monotonicity. Broader debug/perf lines also contain epoch= fields and can
    # legitimately reflect internal mailbox observations or selftest traces that
    # are not part of the constitutional runtime marker contract.
    EPOCHS=$(
        grep -a -E "\[\[AYKEN_SCHED_MB_(ACCEPT|REJECT)\]\]" "${COMBINED_LOG}" | \
            grep -a -Eo "epoch=[0-9]+" | cut -d= -f2 || true
    )
    
    if [[ -z "${EPOCHS}" ]]; then
        echo "epoch_missing" >> "${VIOLATIONS}"
    else
        PREV=""
        for E in ${EPOCHS}; do
            if [[ "${E}" -eq 0 ]]; then
                continue
            fi
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
  "runtime_marker_contract_enforce": ${RUNTIME_MARKER_CONTRACT_ENFORCE},
  "accept_count": ${ACCEPT_COUNT},
  "reject_count": ${REJECT_COUNT},
  "accept_format_invalid_count": ${ACCEPT_FORMAT_INVALID},
  "reject_format_invalid_count": ${REJECT_FORMAT_INVALID},
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

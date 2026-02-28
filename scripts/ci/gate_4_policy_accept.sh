#!/usr/bin/env bash
# ============================================================================
# Gate-4: Policy Accept Proof (Isolated)
# ============================================================================
# Purpose:
#   Prove Ring3 policy input -> Ring0 timer-path validation -> ACCEPT marker.
#
# Contract:
#   - Ring3 writes mailbox ABI header + epoch=1
#   - Kernel deterministically seeds proposer/candidate pid fields
#   - Timer IRQ invokes sched_mailbox_validate_ring3(current_proc)
#   - Gate-4 mode (AYKEN_GATE45_PROOF=0): exactly one target ACCEPT (epoch=1)
#   - Gate-4.5 prereq mode (AYKEN_GATE45_PROOF=1): exactly one target ACCEPT
#     (epoch=1) and, with selftest disabled, exactly one total ACCEPT
#   - Kernel fault signatures are absent (PF/PANIC/FATAL)
# ============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "${REPO_ROOT}"

RUN_ID="${RUN_ID:-gate4-$(date -u +%Y%m%dT%H%M%SZ)}"
KERNEL_PROFILE="${KERNEL_PROFILE:-validation}"
QEMU_TIMEOUT="${QEMU_TIMEOUT:-15}"
GATE4_BOOTSTRAP_POLICY="${GATE4_BOOTSTRAP_POLICY:-1}"
GATE4_MB_SELFTEST="${GATE4_MB_SELFTEST:-0}"
AYKEN_GATE45_PROOF="${AYKEN_GATE45_PROOF:-0}"

EVIDENCE_ROOT="evidence/gate-4-policy-accept"
EVIDENCE_DIR="${EVIDENCE_ROOT}/${RUN_ID}"
mkdir -p "${EVIDENCE_DIR}"
ln -sfn "${RUN_ID}" "${EVIDENCE_ROOT}/latest"

BUILD_LOG="${EVIDENCE_DIR}/build.log"
EFI_LOG="${EVIDENCE_DIR}/efi.log"
QEMU_LOG="${EVIDENCE_DIR}/boot.log"
DEBUGCON_LOG="${EVIDENCE_DIR}/debugcon.log"
SERIAL_LOG="${EVIDENCE_DIR}/serial.log"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS="${EVIDENCE_DIR}/violations.txt"
OVMF_VARS_RUN="${EVIDENCE_DIR}/ovmf_vars.fd"

: > "${BUILD_LOG}"
: > "${EFI_LOG}"
: > "${QEMU_LOG}"
: > "${DEBUGCON_LOG}"
: > "${SERIAL_LOG}"
: > "${VIOLATIONS}"

resolve_ovmf_firmware() {
    if [[ -n "${SYSCALL_OVMF_CODE:-}" && -n "${SYSCALL_OVMF_VARS:-}" && \
          -f "${SYSCALL_OVMF_CODE}" && -f "${SYSCALL_OVMF_VARS}" ]]; then
        printf "%s\n%s\n" "${SYSCALL_OVMF_CODE}" "${SYSCALL_OVMF_VARS}"
        return 0
    fi

    local candidates=(
        "/usr/share/OVMF/OVMF_CODE_4M.fd|/usr/share/OVMF/OVMF_VARS_4M.fd"
        "/usr/share/OVMF/OVMF_CODE.fd|/usr/share/OVMF/OVMF_VARS.fd"
        "/usr/share/edk2/ovmf/OVMF_CODE.fd|/usr/share/edk2/ovmf/OVMF_VARS.fd"
        "/usr/share/qemu/OVMF_CODE.fd|/usr/share/qemu/OVMF_VARS.fd"
        "/opt/homebrew/share/qemu/edk2-x86_64-code.fd|/opt/homebrew/share/qemu/edk2-x86_64-vars.fd"
        "firmware/ovmf/OVMF_CODE.fd|firmware/ovmf/OVMF_VARS.fd"
        "OVMF_CODE.fd|OVMF_VARS.fd"
    )

    local entry code vars
    for entry in "${candidates[@]}"; do
        code="${entry%%|*}"
        vars="${entry##*|}"
        if [[ -f "${code}" && -f "${vars}" ]]; then
            printf "%s\n%s\n" "${code}" "${vars}"
            return 0
        fi
    done
    return 1
}

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

run_qemu_once() {
    : > "${QEMU_LOG}"
    : > "${DEBUGCON_LOG}"
    : > "${SERIAL_LOG}"

    set +e
    timeout "${QEMU_TIMEOUT}" qemu-system-x86_64 \
        -machine q35 \
        -cpu qemu64 \
        -m 512M \
        -drive if=pflash,format=raw,readonly=on,file="${OVMF_CODE}" \
        -drive if=pflash,format=raw,file="${OVMF_VARS_RUN}" \
        -drive format=raw,file=EFI.img \
        -debugcon "file:${DEBUGCON_LOG}" \
        -global isa-debugcon.iobase=0xe9 \
        -serial "file:${SERIAL_LOG}" \
        -display none \
        -no-reboot \
        -no-shutdown \
        > "${QEMU_LOG}" 2>&1
    QEMU_EXIT=$?
    set -e
}

echo "== GATE-4: POLICY ACCEPT PROOF =="
echo "run_id: ${RUN_ID}"
echo "kernel_profile: ${KERNEL_PROFILE}"
echo "qemu_timeout: ${QEMU_TIMEOUT}s"
echo "gate4_bootstrap_policy: ${GATE4_BOOTSTRAP_POLICY}"
echo "gate4_mb_selftest: ${GATE4_MB_SELFTEST}"
echo "ayken_gate45_proof: ${AYKEN_GATE45_PROOF}"
echo "evidence_dir: ${EVIDENCE_DIR}"

echo "[*] Cleaning build artifacts for isolated profile flags..."
make KERNEL_PROFILE="${KERNEL_PROFILE}" AYKEN_MB_SELFTEST="${GATE4_MB_SELFTEST}" AYKEN_GATE4_POLICY_TEST=1 AYKEN_GATE45_PROOF="${AYKEN_GATE45_PROOF}" AYKEN_SCHED_BOOTSTRAP_POLICY="${GATE4_BOOTSTRAP_POLICY}" clean >> "${BUILD_LOG}" 2>&1 || true

echo "[*] Building kernel (Gate-4 isolated mode)..."
if ! make KERNEL_PROFILE="${KERNEL_PROFILE}" AYKEN_MB_SELFTEST="${GATE4_MB_SELFTEST}" AYKEN_GATE4_POLICY_TEST=1 AYKEN_GATE45_PROOF="${AYKEN_GATE45_PROOF}" AYKEN_SCHED_BOOTSTRAP_POLICY="${GATE4_BOOTSTRAP_POLICY}" kernel >> "${BUILD_LOG}" 2>&1; then
    echo "build_failed" >> "${VIOLATIONS}"
fi

echo "[*] Creating EFI image..."
if ! make KERNEL_PROFILE="${KERNEL_PROFILE}" AYKEN_MB_SELFTEST="${GATE4_MB_SELFTEST}" AYKEN_GATE4_POLICY_TEST=1 AYKEN_GATE45_PROOF="${AYKEN_GATE45_PROOF}" AYKEN_SCHED_BOOTSTRAP_POLICY="${GATE4_BOOTSTRAP_POLICY}" efi-img > "${EFI_LOG}" 2>&1; then
    echo "efi_image_failed" >> "${VIOLATIONS}"
fi

OVMF_PAIR="$(resolve_ovmf_firmware || true)"
if [[ -z "${OVMF_PAIR}" ]]; then
    echo "ovmf_not_found" >> "${VIOLATIONS}"
else
    OVMF_CODE="$(printf "%s\n" "${OVMF_PAIR}" | sed -n '1p')"
    OVMF_VARS_TEMPLATE="$(printf "%s\n" "${OVMF_PAIR}" | sed -n '2p')"
fi

if [[ ! -f EFI.img ]]; then
    echo "efi_img_missing" >> "${VIOLATIONS}"
fi

VARSTORE_MODE="template_copy"
SHELL_FALLBACK_DETECTED=0
SHELL_FALLBACK_ON_TEMPLATE=0
QEMU_ATTEMPTS=0

if [[ ! -s "${VIOLATIONS}" ]]; then
    echo "[*] Preparing clean varstore..."
    if [[ ! -f "${OVMF_VARS_TEMPLATE}" ]]; then
        echo "ovmf_vars_template_missing" >> "${VIOLATIONS}"
    else
        cp -f "${OVMF_VARS_TEMPLATE}" "${OVMF_VARS_RUN}"
    fi
fi

QEMU_EXIT=-1
if [[ ! -s "${VIOLATIONS}" ]]; then
    echo "[*] Booting kernel (timeout: ${QEMU_TIMEOUT}s)..."
    QEMU_ATTEMPTS=$((QEMU_ATTEMPTS + 1))
    run_qemu_once

    if grep -a -Eiq 'UEFI Interactive Shell|EFI Internal Shell|Boot0006' "${SERIAL_LOG}" "${QEMU_LOG}" 2>/dev/null; then
        SHELL_FALLBACK_DETECTED=1
        SHELL_FALLBACK_ON_TEMPLATE=1
    fi

    if [[ "${SHELL_FALLBACK_DETECTED}" -eq 1 ]]; then
        echo "[*] Shell fallback detected with template varstore; retrying with blank varstore..."
        cp -f "${QEMU_LOG}" "${EVIDENCE_DIR}/boot_attempt_template.log" 2>/dev/null || true
        cp -f "${DEBUGCON_LOG}" "${EVIDENCE_DIR}/debugcon_attempt_template.log" 2>/dev/null || true
        cp -f "${SERIAL_LOG}" "${EVIDENCE_DIR}/serial_attempt_template.log" 2>/dev/null || true

        VARS_SIZE="$(wc -c < "${OVMF_VARS_TEMPLATE}" 2>/dev/null || echo 0)"
        VARS_SIZE="$(printf "%s" "${VARS_SIZE}" | tr -d '[:space:]')"
        if [[ -z "${VARS_SIZE}" || "${VARS_SIZE}" -le 0 ]]; then
            echo "ovmf_vars_size_invalid" >> "${VIOLATIONS}"
        else
            dd if=/dev/zero of="${OVMF_VARS_RUN}" bs=1 count="${VARS_SIZE}" >/dev/null 2>&1
            VARSTORE_MODE="blank_fallback"
            SHELL_FALLBACK_DETECTED=0
            QEMU_ATTEMPTS=$((QEMU_ATTEMPTS + 1))
            run_qemu_once
            cp -f "${QEMU_LOG}" "${EVIDENCE_DIR}/boot_attempt_blank.log" 2>/dev/null || true
            cp -f "${DEBUGCON_LOG}" "${EVIDENCE_DIR}/debugcon_attempt_blank.log" 2>/dev/null || true
            cp -f "${SERIAL_LOG}" "${EVIDENCE_DIR}/serial_attempt_blank.log" 2>/dev/null || true
            if grep -a -Eiq 'UEFI Interactive Shell|EFI Internal Shell|Boot0006' "${SERIAL_LOG}" "${QEMU_LOG}" 2>/dev/null; then
                SHELL_FALLBACK_DETECTED=1
            fi
        fi
    fi
fi

DEBUGCON_BYTES="$(wc -c < "${DEBUGCON_LOG}" 2>/dev/null || echo 0)"
SERIAL_BYTES="$(wc -c < "${SERIAL_LOG}" 2>/dev/null || echo 0)"
QEMU_LOG_BYTES="$(wc -c < "${QEMU_LOG}" 2>/dev/null || echo 0)"
DEBUGCON_BYTES="$(printf "%s" "${DEBUGCON_BYTES}" | tr -d '[:space:]')"
SERIAL_BYTES="$(printf "%s" "${SERIAL_BYTES}" | tr -d '[:space:]')"
QEMU_LOG_BYTES="$(printf "%s" "${QEMU_LOG_BYTES}" | tr -d '[:space:]')"

if [[ "${QEMU_EXIT}" -ne 0 && "${QEMU_EXIT}" -ne 124 && "${QEMU_EXIT}" -ne -1 ]]; then
    echo "qemu_unexpected_exit:${QEMU_EXIT}" >> "${VIOLATIONS}"
fi

if [[ "${SHELL_FALLBACK_DETECTED}" -eq 1 ]]; then
    echo "uefi_shell_fallback_detected" >> "${VIOLATIONS}"
fi

echo "[*] Validating markers..."
BOOT_OK_COUNT="$(safe_count_file "\\[\\[AYKEN_BOOT_OK\\]\\]" "${DEBUGCON_LOG}")"
if [[ "${BOOT_OK_COUNT}" -lt 1 ]]; then
    echo "boot_ok_missing" >> "${VIOLATIONS}"
fi

PRELOAD_MARKER_COUNT="$(safe_count_file "\\[K\\]\\[PHASE10\\] PRELOAD_GATE4_OWNER" "${DEBUGCON_LOG}")"
if [[ "${GATE4_BOOTSTRAP_POLICY}" -eq 0 ]]; then
    if [[ "${PRELOAD_MARKER_COUNT}" -lt 1 ]]; then
        echo "preload_marker_missing_strict" >> "${VIOLATIONS}"
    fi
else
    if [[ "${PRELOAD_MARKER_COUNT}" -ne 0 ]]; then
        echo "preload_marker_unexpected_transitional:count=${PRELOAD_MARKER_COUNT}" >> "${VIOLATIONS}"
    fi
fi

GATE4_PID_MARKER_COUNT="$(safe_count_file "\\[\\[AYKEN_GATE4_PID\\]\\] pid=[0-9]+" "${DEBUGCON_LOG}")"
GATE4_PID="$(grep -a -Eo '\[\[AYKEN_GATE4_PID\]\] pid=[0-9]+' "${DEBUGCON_LOG}" | tail -n1 | sed 's/.*pid=//' || true)"
if [[ -z "${GATE4_PID}" ]]; then
    echo "gate4_pid_missing" >> "${VIOLATIONS}"
fi

TARGET_ACCEPT_COUNT=0
TOTAL_ACCEPT_COUNT=0
NON_TARGET_ACCEPT_COUNT=0
TARGET_PID_MARKER_COUNT=0
RING3_PUBLISH_COUNT=0
RING3_PUBLISH_LINE=0
TARGET_ACCEPT_LINE=0

if [[ -n "${GATE4_PID}" ]]; then
    TARGET_PID_MARKER_COUNT="$(safe_count_file "\\[\\[AYKEN_GATE4_PID\\]\\] pid=${GATE4_PID}" "${DEBUGCON_LOG}")"
    if [[ "${TARGET_PID_MARKER_COUNT}" -lt 1 ]]; then
        echo "gate4_pid_marker_mismatch:pid=${GATE4_PID}" >> "${VIOLATIONS}"
    fi
    TARGET_ACCEPT_COUNT="$(safe_count_file "\\[\\[AYKEN_SCHED_MB_ACCEPT\\]\\] pid=${GATE4_PID} epoch=1" "${DEBUGCON_LOG}")"
    RING3_PUBLISH_COUNT="$(safe_count_file "\\[\\[AYKEN_RING3_PUBLISH\\]\\] pid=${GATE4_PID} epoch=1" "${DEBUGCON_LOG}")"
    if [[ "${RING3_PUBLISH_COUNT}" -lt 1 ]]; then
        echo "ring3_publish_missing:pid=${GATE4_PID}" >> "${VIOLATIONS}"
    fi

    RING3_PUBLISH_LINE="$(grep -a -n -E "\\[\\[AYKEN_RING3_PUBLISH\\]\\] pid=${GATE4_PID} epoch=1" "${DEBUGCON_LOG}" | head -n1 | cut -d: -f1 || true)"
    TARGET_ACCEPT_LINE="$(grep -a -n -E "\\[\\[AYKEN_SCHED_MB_ACCEPT\\]\\] pid=${GATE4_PID} epoch=1" "${DEBUGCON_LOG}" | head -n1 | cut -d: -f1 || true)"
    RING3_PUBLISH_LINE="$(printf "%s" "${RING3_PUBLISH_LINE}" | tr -dc '0-9')"
    TARGET_ACCEPT_LINE="$(printf "%s" "${TARGET_ACCEPT_LINE}" | tr -dc '0-9')"
    if [[ -z "${RING3_PUBLISH_LINE}" ]]; then
        RING3_PUBLISH_LINE=0
    fi
    if [[ -z "${TARGET_ACCEPT_LINE}" ]]; then
        TARGET_ACCEPT_LINE=0
    fi
    if [[ "${RING3_PUBLISH_LINE}" -gt 0 && "${TARGET_ACCEPT_LINE}" -gt 0 && "${RING3_PUBLISH_LINE}" -ge "${TARGET_ACCEPT_LINE}" ]]; then
        echo "ring3_publish_order_invalid:publish_line=${RING3_PUBLISH_LINE}:accept_line=${TARGET_ACCEPT_LINE}" >> "${VIOLATIONS}"
    fi
fi
TOTAL_ACCEPT_COUNT="$(safe_count_file "\\[\\[AYKEN_SCHED_MB_ACCEPT\\]\\]" "${DEBUGCON_LOG}")"
if [[ "${TOTAL_ACCEPT_COUNT}" -ge "${TARGET_ACCEPT_COUNT}" ]]; then
    NON_TARGET_ACCEPT_COUNT=$((TOTAL_ACCEPT_COUNT - TARGET_ACCEPT_COUNT))
fi

if [[ "${AYKEN_GATE45_PROOF}" -eq 1 ]]; then
    if [[ "${TARGET_ACCEPT_COUNT}" -ne 1 ]]; then
        echo "target_accept_mismatch_gate45:pid=${GATE4_PID:-unknown}:count=${TARGET_ACCEPT_COUNT}" >> "${VIOLATIONS}"
    fi
    if [[ "${GATE4_MB_SELFTEST}" -eq 0 && "${TOTAL_ACCEPT_COUNT}" -ne 1 ]]; then
        echo "total_accept_mismatch_gate45_no_selftest:count=${TOTAL_ACCEPT_COUNT}" >> "${VIOLATIONS}"
    fi
else
    if [[ "${GATE4_MB_SELFTEST}" -eq 0 && "${TOTAL_ACCEPT_COUNT}" -ne 1 ]]; then
        echo "total_accept_mismatch_no_selftest:count=${TOTAL_ACCEPT_COUNT}" >> "${VIOLATIONS}"
    fi
    if [[ "${TARGET_ACCEPT_COUNT}" -ne 1 ]]; then
        echo "target_accept_mismatch:pid=${GATE4_PID:-unknown}:count=${TARGET_ACCEPT_COUNT}" >> "${VIOLATIONS}"
    fi
fi

# Gate-4 run must not end in obvious fault signatures.
if grep -a -Eiq 'PF!|PANIC|FATAL|TRIPLE FAULT|ASSERT|KERNEL BUG|GENERAL PROTECTION' \
    "${DEBUGCON_LOG}" "${SERIAL_LOG}" "${QEMU_LOG}" 2>/dev/null; then
    echo "kernel_fault_signature_detected" >> "${VIOLATIONS}"
fi

# Epoch monotonic check on scheduler markers for target PID only.
if [[ -n "${GATE4_PID}" ]]; then
    EPOCHS="$(grep -a -E "\\[\\[AYKEN_SCHED_MB_(ACCEPT|REJECT)\\]\\]" "${DEBUGCON_LOG}" | \
        grep -a -E "pid=${GATE4_PID}([^0-9]|$)" | grep -a -Eo "epoch=[0-9]+" | cut -d= -f2 || true)"
    PREV=""
    for E in ${EPOCHS}; do
        if [[ "${E}" -eq 0 ]]; then
            continue
        fi
        if [[ -n "${PREV}" && "${E}" -lt "${PREV}" ]]; then
            echo "epoch_not_monotonic:prev=${PREV}:current=${E}" >> "${VIOLATIONS}"
            break
        fi
        PREV="${E}"
    done
fi

VIOLATION_COUNT="$(wc -l < "${VIOLATIONS}" | tr -d '[:space:]')"
if [[ "${VIOLATION_COUNT}" -eq 0 ]]; then
    VERDICT="PASS"
    REASON="Gate-4 policy path validated (target ACCEPT matched)"
else
    VERDICT="FAIL"
    REASON="${VIOLATION_COUNT} violations detected"
fi

cat > "${REPORT_JSON}" <<EOF
{
  "gate": "policy-accept",
  "run_id": "${RUN_ID}",
  "verdict": "${VERDICT}",
  "reason": "${REASON}",
  "kernel_profile": "${KERNEL_PROFILE}",
  "gate4_bootstrap_policy": ${GATE4_BOOTSTRAP_POLICY},
  "gate4_mb_selftest": ${GATE4_MB_SELFTEST},
  "ayken_gate45_proof": ${AYKEN_GATE45_PROOF},
  "qemu_timeout": ${QEMU_TIMEOUT},
  "qemu_attempts": ${QEMU_ATTEMPTS},
  "qemu_exit_code": ${QEMU_EXIT},
  "varstore_mode": "${VARSTORE_MODE}",
  "shell_fallback_on_template": ${SHELL_FALLBACK_ON_TEMPLATE},
  "shell_fallback_final": ${SHELL_FALLBACK_DETECTED},
  "boot_ok_count": ${BOOT_OK_COUNT},
  "preload_marker_count": ${PRELOAD_MARKER_COUNT},
  "gate4_pid_marker_count": ${GATE4_PID_MARKER_COUNT},
  "target_pid_marker_count": ${TARGET_PID_MARKER_COUNT},
  "gate4_pid": ${GATE4_PID:-0},
  "ring3_publish_count": ${RING3_PUBLISH_COUNT},
  "ring3_publish_line": ${RING3_PUBLISH_LINE},
  "target_accept_line": ${TARGET_ACCEPT_LINE},
  "target_accept_count": ${TARGET_ACCEPT_COUNT},
  "total_accept_count": ${TOTAL_ACCEPT_COUNT},
  "non_target_accept_count": ${NON_TARGET_ACCEPT_COUNT},
  "debugcon_bytes": ${DEBUGCON_BYTES},
  "serial_bytes": ${SERIAL_BYTES},
  "qemu_log_bytes": ${QEMU_LOG_BYTES},
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo "Report: ${REPORT_JSON}"
cat "${REPORT_JSON}"

if [[ "${VERDICT}" != "PASS" ]]; then
    echo ""
    echo "Violations:"
    cat "${VIOLATIONS}"
    echo ""
    echo "Debugcon (tail 40):"
    tail -40 "${DEBUGCON_LOG}" || true
    echo ""
    echo "Serial (tail 40):"
    tail -40 "${SERIAL_LOG}" || true
    exit 1
fi

echo "[PASS] Gate-4: Policy Accept Proof PASS"

#!/bin/bash
# AykenOS Development Loop
# Fast iteration cycle: smoke → contract → full
#
# Author: Kenan AY — System Architect
#
# Usage:
#   ./scripts/dev_loop.sh smoke      # Quick boot check (5-10s)
#   ./scripts/dev_loop.sh contract   # Runtime contract tests
#   ./scripts/dev_loop.sh full       # Full evidence validation
#
# Exit Status Contract (Subtask 1.3):
#   0 = PASS - All validation checks succeeded
#   1 = FAIL - Validation failure (build, boot, markers, tests)
#   2 = Invalid usage (wrong arguments)
#
# Marker Sequence Guarantee (Subtask 1.1):
#   Required order: [K][EARLY_BOOT_OK] → [K][LATE_INIT_END] → [[AYKEN_BOOT_OK]]
#   Sequence violations result in FAIL (exit 1)
#
# Error Reporting (Subtask 1.2):
#   Clear diagnostic output for all failure modes
#   Last 50 lines of boot log shown on failure
#
# Log Directory Management (Subtask 1.4):
#   Automatic creation of log directory with error handling
#   Proper file lifecycle management

set -euo pipefail

MODE="${1:-smoke}"
QEMU_TIMEOUT_SECONDS="${QEMU_TIMEOUT_SECONDS:-20}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_DIR="out/logs"
BOOT_LOG="$LOG_DIR/boot_watch.log"
DEBUG_LOG="$LOG_DIR/debug_run.log"
QEMU_LOG="$LOG_DIR/qemu_run.log"
OVMF_CODE="${OVMF_CODE:-firmware/ovmf/OVMF_CODE.fd}"
OVMF_VARS_TEMPLATE="${OVMF_VARS_TEMPLATE:-OVMF_VARS.clean.fd}"
OVMF_VARS_RUN="$LOG_DIR/ovmf_vars.fd"
EFI_IMG="${EFI_IMG:-EFI.img}"

cd "$PROJECT_ROOT"

is_positive_int() {
    case "${1:-}" in
        ''|*[!0-9]*)
            return 1
            ;;
        *)
            [ "$1" -gt 0 ]
            ;;
    esac
}

detect_ncpu() {
    local value=""

    if command -v sysctl >/dev/null 2>&1; then
        value="$(sysctl -n hw.ncpu 2>/dev/null || true)"
        if is_positive_int "$value"; then
            printf '%s\n' "$value"
            return 0
        fi
    fi

    if command -v nproc >/dev/null 2>&1; then
        value="$(nproc 2>/dev/null || true)"
        if is_positive_int "$value"; then
            printf '%s\n' "$value"
            return 0
        fi
    fi

    if command -v getconf >/dev/null 2>&1; then
        value="$(getconf _NPROCESSORS_ONLN 2>/dev/null || true)"
        if is_positive_int "$value"; then
            printf '%s\n' "$value"
            return 0
        fi
    fi

    printf '%s\n' 4
}

find_timeout_bin() {
    if [ -n "${TIMEOUT_BIN:-}" ]; then
        printf '%s\n' "$TIMEOUT_BIN"
        return 0
    fi

    if command -v timeout >/dev/null 2>&1; then
        command -v timeout
        return 0
    fi

    if command -v gtimeout >/dev/null 2>&1; then
        command -v gtimeout
        return 0
    fi

    if [ -x /opt/homebrew/bin/timeout ]; then
        printf '%s\n' /opt/homebrew/bin/timeout
        return 0
    fi

    return 1
}

print_usage() {
    echo "Usage: $0 {smoke|contract|full}"
    echo ""
    echo "Modes:"
    echo "  smoke    - Quick build + boot check"
    echo "  contract - Build + boot + runtime contract tests"
    echo "  full     - Build + boot + all evidence tests"
}

case "$MODE" in
    smoke|contract|full)
        ;;
    *)
        print_usage
        exit 2
        ;;
esac

if ! is_positive_int "$QEMU_TIMEOUT_SECONDS"; then
    echo "Invalid QEMU_TIMEOUT_SECONDS: $QEMU_TIMEOUT_SECONDS"
    exit 2
fi

TIMEOUT_BIN="$(find_timeout_bin)" || {
    echo "BOOT FAILED: timeout command not found. Install coreutils or set TIMEOUT_BIN."
    exit 1
}

NCPU="$(detect_ncpu)"

echo "== AykenOS Dev Loop =="
echo "mode: $MODE"
echo "cpus: $NCPU"
echo ""

run_build() {
    echo "[1/3] Build..."
    make -j"$NCPU" all \
        KERNEL_PROFILE=validation \
        AYKEN_VALIDATION=1 \
        AYKEN_VCP_RUNTIME_HOOK_TEST=0 \
        AYKEN_VCP_FAIL_CLOSED_TEST=0 \
        AYKEN_VCP_EVIDENCE_TEST=0 \
        AYKEN_VCP_TRUST_VERIFICATION_TEST=0 \
        AYKEN_MB_SELFTEST=0
    make efi-img \
        KERNEL_PROFILE=validation \
        AYKEN_VALIDATION=1 \
        AYKEN_VCP_RUNTIME_HOOK_TEST=0 \
        AYKEN_VCP_FAIL_CLOSED_TEST=0 \
        AYKEN_VCP_EVIDENCE_TEST=0 \
        AYKEN_VCP_TRUST_VERIFICATION_TEST=0 \
        AYKEN_MB_SELFTEST=0
    echo "✅ Build complete"
    echo ""
}

file_bytes() {
    local file="$1"
    local bytes="0"

    if [ -f "$file" ]; then
        bytes="$(wc -c < "$file" 2>/dev/null || echo 0)"
        bytes="${bytes//[[:space:]]/}"
    fi

    if is_positive_int "$bytes"; then
        printf '%s\n' "$bytes"
    else
        printf '%s\n' 0
    fi
}

show_log_tail() {
    local log_file="$1"

    if [ -s "$log_file" ]; then
        tail -50 "$log_file"
    else
        echo "(log is empty: $log_file)"
    fi
}

prepare_qemu_inputs() {
    if [ ! -f "$OVMF_CODE" ]; then
        echo "❌ BOOT FAILED: OVMF code not found: $OVMF_CODE"
        exit 1
    fi

    if [ ! -f "$OVMF_VARS_TEMPLATE" ]; then
        echo "❌ BOOT FAILED: OVMF vars template not found: $OVMF_VARS_TEMPLATE"
        exit 1
    fi

    if [ ! -f "$EFI_IMG" ]; then
        echo "❌ BOOT FAILED: EFI image not found: $EFI_IMG"
        exit 1
    fi

    # Copy template instead of creating empty file
    cp -f "$OVMF_VARS_TEMPLATE" "$OVMF_VARS_RUN" || {
        echo "❌ BOOT FAILED: Cannot copy OVMF vars template: $OVMF_VARS_TEMPLATE"
        exit 1
    }
}

run_qemu_debugcon_backend() {
    set +e
    "$TIMEOUT_BIN" "$QEMU_TIMEOUT_SECONDS" qemu-system-x86_64 \
        -machine q35 \
        -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
        -drive if=pflash,format=raw,file="$OVMF_VARS_RUN" \
        -drive format=raw,file="$EFI_IMG" \
        -debugcon "file:$DEBUG_LOG" \
        -global isa-debugcon.iobase=0xe9 \
        -nographic \
        -no-reboot \
        > "$QEMU_LOG" 2>&1
    qemu_status=$?
    set -e
}

run_qemu_chardev_backend() {
    set +e
    "$TIMEOUT_BIN" "$QEMU_TIMEOUT_SECONDS" qemu-system-x86_64 \
        -machine q35 \
        -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
        -drive if=pflash,format=raw,file="$OVMF_VARS_RUN" \
        -drive format=raw,file="$EFI_IMG" \
        -chardev file,id=debugcon0,path="$DEBUG_LOG" \
        -device isa-debugcon,iobase=0xe9,chardev=debugcon0 \
        -nographic \
        -no-reboot \
        > "$QEMU_LOG" 2>&1
    qemu_status=$?
    set -e
}

run_qemu_observer() {
    local debug_bytes
    qemu_backend="debugcon"
    qemu_status=0

    prepare_qemu_inputs
    run_qemu_debugcon_backend
    debug_bytes="$(file_bytes "$DEBUG_LOG")"

    if [ "$debug_bytes" -eq 0 ]; then
        qemu_backend="chardev"
        : > "$DEBUG_LOG"
        : > "$QEMU_LOG"
        prepare_qemu_inputs
        run_qemu_chardev_backend
        debug_bytes="$(file_bytes "$DEBUG_LOG")"
    fi

    echo "qemu backend: $qemu_backend"
    echo "qemu exit status: $qemu_status"
    echo "debug log bytes: $debug_bytes"

    if [ "$qemu_status" -ne 0 ] && [ "$qemu_status" -ne 124 ]; then
        echo "❌ BOOT FAILED: QEMU exited unexpectedly (exit $qemu_status)"
        echo ""
        echo "Last 50 lines of QEMU log:"
        show_log_tail "$QEMU_LOG"
        exit 1
    fi

    if [ "$debug_bytes" -eq 0 ]; then
        echo "❌ BOOT FAILED: Kernel debug log is empty after both QEMU debugcon backends"
        echo ""
        echo "Last 50 lines of QEMU log:"
        show_log_tail "$QEMU_LOG"
        exit 1
    fi

    if grep -Eq "UEFI Interactive Shell|Boot[0-9]+ \"EFI Internal Shell\"" "$QEMU_LOG"; then
        echo "qemu note: UEFI shell text observed before kernel markers; marker log remains authoritative"
    fi
}

read_marker_positions() {
    local log_file="$1"

    awk '
        BEGIN {
            early_line = 0
            late_line = 0
            boot_line = 0
            early_count = 0
            late_count = 0
            boot_count = 0
            pre_sequence_boot_count = 0
        }
        index($0, "[K][EARLY_BOOT_OK]") {
            early_count++
            if (early_line == 0) {
                early_line = NR
            }
        }
        index($0, "[K][LATE_INIT_END]") {
            late_count++
            if (early_line > 0 && late_line == 0 && NR > early_line) {
                late_line = NR
            }
        }
        index($0, "[[AYKEN_BOOT_OK]]") {
            boot_count++
            if (early_line == 0) {
                pre_sequence_boot_count++
            }
            if (late_line > 0 && boot_line == 0 && NR > late_line) {
                boot_line = NR
            }
        }
        END {
            printf("early_line=%d\n", early_line)
            printf("late_line=%d\n", late_line)
            printf("boot_line=%d\n", boot_line)
            printf("early_count=%d\n", early_count)
            printf("late_count=%d\n", late_count)
            printf("boot_count=%d\n", boot_count)
            printf("pre_sequence_boot_count=%d\n", pre_sequence_boot_count)
        }
    ' "$log_file"
}

validate_marker_sequence() {
    local log_file="$1"
    local marker_positions
    local key
    local value
    local early_line=0
    local late_line=0
    local boot_line=0
    local early_count=0
    local late_count=0
    local boot_count=0
    local pre_sequence_boot_count=0

    marker_positions="$(read_marker_positions "$log_file")"
    while IFS='=' read -r key value; do
        case "$key" in
            early_line) early_line="$value" ;;
            late_line) late_line="$value" ;;
            boot_line) boot_line="$value" ;;
            early_count) early_count="$value" ;;
            late_count) late_count="$value" ;;
            boot_count) boot_count="$value" ;;
            pre_sequence_boot_count) pre_sequence_boot_count="$value" ;;
        esac
    done <<EOF
$marker_positions
EOF

    if [ "$early_count" -eq 0 ]; then
        echo "❌ BOOT FAILED: [K][EARLY_BOOT_OK] marker not found"
        echo ""
        echo "Expected marker: [K][EARLY_BOOT_OK]"
        echo "This indicates early boot phase did not complete successfully."
        echo ""
        echo "Last 50 lines of kernel log:"
        show_log_tail "$log_file"
        exit 1
    fi

    if [ "$late_count" -eq 0 ]; then
        echo "❌ BOOT FAILED: [K][LATE_INIT_END] marker not found"
        echo ""
        echo "Expected marker: [K][LATE_INIT_END]"
        echo "This indicates late initialization phase did not complete successfully."
        echo ""
        echo "Last 50 lines of kernel log:"
        show_log_tail "$log_file"
        exit 1
    fi

    if [ "$boot_count" -eq 0 ]; then
        echo "❌ BOOT FAILED: [[AYKEN_BOOT_OK]] marker not found"
        echo ""
        echo "Expected marker: [[AYKEN_BOOT_OK]]"
        echo "This indicates full boot sequence did not complete successfully."
        echo ""
        echo "Last 50 lines of kernel log:"
        show_log_tail "$log_file"
        exit 1
    fi

    if [ "$late_line" -eq 0 ]; then
        echo "❌ BOOT FAILED: Marker sequence violation"
        echo ""
        echo "Expected order: [K][EARLY_BOOT_OK] before [K][LATE_INIT_END]"
        echo "No [K][LATE_INIT_END] marker was observed after [K][EARLY_BOOT_OK]."
        echo ""
        echo "Marker counts:"
        echo "  [K][EARLY_BOOT_OK]: $early_count"
        echo "  [K][LATE_INIT_END]: $late_count"
        echo "  [[AYKEN_BOOT_OK]]: $boot_count"
        echo ""
        echo "Last 50 lines of kernel log:"
        show_log_tail "$log_file"
        exit 1
    fi

    if [ "$boot_line" -eq 0 ]; then
        echo "❌ BOOT FAILED: Marker sequence violation"
        echo ""
        echo "Expected order: [K][LATE_INIT_END] before [[AYKEN_BOOT_OK]]"
        echo "No [[AYKEN_BOOT_OK]] marker was observed after [K][LATE_INIT_END]."
        echo ""
        echo "Marker counts:"
        echo "  [K][EARLY_BOOT_OK]: $early_count"
        echo "  [K][LATE_INIT_END]: $late_count"
        echo "  [[AYKEN_BOOT_OK]]: $boot_count"
        echo ""
        echo "Last 50 lines of kernel log:"
        show_log_tail "$log_file"
        exit 1
    fi

    if [ "$pre_sequence_boot_count" -gt 0 ]; then
        echo "   Note: observed $pre_sequence_boot_count pre-sequence BOOT marker(s); canonical post-late BOOT marker used."
    fi

    echo "✅ Smoke boot PASS"
    echo "   Marker sequence validated: EARLY(line $early_line) → LATE(line $late_line) → BOOT_OK(line $boot_line)"
    echo ""
}

run_smoke_boot() {
    echo "[2/3] Smoke boot test..."

    # Subtask 1.4: Log directory management
    if [ ! -d "$LOG_DIR" ]; then
        mkdir -p "$LOG_DIR" || {
            echo "❌ BOOT FAILED: Cannot create log directory: $LOG_DIR"
            exit 1
        }
    fi

    # Clear previous log files
    : > "$BOOT_LOG" || {
        echo "❌ BOOT FAILED: Cannot write to log file: $BOOT_LOG"
        exit 1
    }

    : > "$DEBUG_LOG" || {
        echo "❌ BOOT FAILED: Cannot write to debug log"
        exit 1
    }

    : > "$QEMU_LOG" || {
        echo "❌ BOOT FAILED: Cannot write to QEMU log"
        exit 1
    }

    run_qemu_observer | tee "$BOOT_LOG"
    validate_marker_sequence "$DEBUG_LOG"
}

run_contract_tests() {
    echo "[3/3] Contract tests..."
    
    # Check if test scripts exist
    if [ -f "scripts/test_vcp_runtime_hook.sh" ]; then
        echo "Running VCP runtime hook tests..."
        bash scripts/test_vcp_runtime_hook.sh
    else
        echo "⚠️  scripts/test_vcp_runtime_hook.sh not found, skipping"
    fi
    
    if [ -f "scripts/test_vcp_trust_verification.sh" ]; then
        echo "Running VCP trust verification tests..."
        bash scripts/test_vcp_trust_verification.sh
    else
        echo "⚠️  scripts/test_vcp_trust_verification.sh not found, skipping"
    fi
    
    if [ -f "scripts/test_vcp_fail_closed.sh" ]; then
        echo "Running VCP fail-closed tests..."
        bash scripts/test_vcp_fail_closed.sh
    else
        echo "⚠️  scripts/test_vcp_fail_closed.sh not found, skipping"
    fi
    
    echo "✅ Contract tests complete"
    echo ""
}

run_full_tests() {
    echo "[3/3] Full evidence tests..."
    
    run_contract_tests
    
    if [ -f "scripts/test_vcp_evidence.sh" ]; then
        echo "Running VCP evidence tests..."
        bash scripts/test_vcp_evidence.sh
    else
        echo "⚠️  scripts/test_vcp_evidence.sh not found, skipping"
    fi
    
    echo "✅ Full tests complete"
    echo ""
}

case "$MODE" in
    smoke)
        run_build
        run_smoke_boot
        ;;
    contract)
        run_build
        run_smoke_boot
        run_contract_tests
        ;;
    full)
        run_build
        run_smoke_boot
        run_full_tests
        ;;
esac

echo "✅ PASS: $MODE mode"

# Generate evidence artifacts (runs AFTER validation)
# This is non-authoritative and never affects validation decisions
if [ -f "scripts/generate_evidence.sh" ]; then
    echo ""
    echo "Generating evidence artifacts..."
    RUN_ID=$(bash scripts/generate_evidence.sh 2>&1 | tail -1)
    echo "Evidence generated: $RUN_ID"
fi

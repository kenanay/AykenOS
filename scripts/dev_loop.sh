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
LOG_DIR="out/logs"
BOOT_LOG="$LOG_DIR/boot_watch.log"

# Detect CPU count for parallel builds
if command -v sysctl >/dev/null 2>&1; then
    NCPU=$(sysctl -n hw.ncpu)
elif command -v nproc >/dev/null 2>&1; then
    NCPU=$(nproc)
else
    NCPU=4
fi

echo "== AykenOS Dev Loop =="
echo "mode: $MODE"
echo "cpus: $NCPU"
echo ""

run_build() {
    echo "[1/3] Build..."
    make -j"$NCPU" \
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

run_smoke_boot() {
    echo "[2/3] Smoke boot test..."
    
    # Subtask 1.4: Log directory management
    if [ ! -d "$LOG_DIR" ]; then
        mkdir -p "$LOG_DIR" || {
            echo "❌ BOOT FAILED: Cannot create log directory: $LOG_DIR"
            exit 1
        }
    fi
    
    # Clear previous log file
    : > "$BOOT_LOG" || {
        echo "❌ BOOT FAILED: Cannot write to log file: $BOOT_LOG"
        exit 1
    }
    
    set +e
    timeout "$QEMU_TIMEOUT_SECONDS" make run \
        KERNEL_PROFILE=validation \
        AYKEN_VALIDATION=1 \
        AYKEN_VCP_RUNTIME_HOOK_TEST=0 \
        AYKEN_VCP_FAIL_CLOSED_TEST=0 \
        AYKEN_VCP_EVIDENCE_TEST=0 \
        AYKEN_VCP_TRUST_VERIFICATION_TEST=0 \
        AYKEN_MB_SELFTEST=0 \
        > "$BOOT_LOG" 2>&1
    qemu_status=$?
    set -e
    
    echo "qemu exit status: $qemu_status"
    
    # Subtask 1.2: Error reporting capability - Check for required markers
    local early_found=false
    local late_found=false
    local boot_found=false
    
    if grep -q "\[K\]\[EARLY_BOOT_OK\]" "$BOOT_LOG"; then
        early_found=true
    fi
    
    if grep -q "\[K\]\[LATE_INIT_END\]" "$BOOT_LOG"; then
        late_found=true
    fi
    
    if grep -q "\[\[AYKEN_BOOT_OK\]\]" "$BOOT_LOG"; then
        boot_found=true
    fi
    
    # Check for missing markers with clear diagnostics
    if [ "$early_found" = false ]; then
        echo "❌ BOOT FAILED: [K][EARLY_BOOT_OK] marker not found"
        echo ""
        echo "Expected marker: [K][EARLY_BOOT_OK]"
        echo "This indicates early boot phase did not complete successfully."
        echo ""
        echo "Last 50 lines of boot log:"
        tail -50 "$BOOT_LOG"
        exit 1
    fi
    
    if [ "$late_found" = false ]; then
        echo "❌ BOOT FAILED: [K][LATE_INIT_END] marker not found"
        echo ""
        echo "Expected marker: [K][LATE_INIT_END]"
        echo "This indicates late initialization phase did not complete successfully."
        echo ""
        echo "Last 50 lines of boot log:"
        tail -50 "$BOOT_LOG"
        exit 1
    fi
    
    if [ "$boot_found" = false ]; then
        echo "❌ BOOT FAILED: [[AYKEN_BOOT_OK]] marker not found"
        echo ""
        echo "Expected marker: [[AYKEN_BOOT_OK]]"
        echo "This indicates full boot sequence did not complete successfully."
        echo ""
        echo "Last 50 lines of boot log:"
        tail -50 "$BOOT_LOG"
        exit 1
    fi
    
    # Subtask 1.1: Marker sequence guarantee - Validate correct ordering
    local early_line=$(grep -n "\[K\]\[EARLY_BOOT_OK\]" "$BOOT_LOG" | head -1 | cut -d: -f1)
    local late_line=$(grep -n "\[K\]\[LATE_INIT_END\]" "$BOOT_LOG" | head -1 | cut -d: -f1)
    local boot_line=$(grep -n "\[\[AYKEN_BOOT_OK\]\]" "$BOOT_LOG" | head -1 | cut -d: -f1)
    
    # Validate sequence: EARLY → LATE → BOOT_OK
    if [ "$early_line" -gt "$late_line" ]; then
        echo "❌ BOOT FAILED: Marker sequence violation"
        echo ""
        echo "Expected order: [K][EARLY_BOOT_OK] → [K][LATE_INIT_END] → [[AYKEN_BOOT_OK]]"
        echo "Actual order: [K][EARLY_BOOT_OK] appears AFTER [K][LATE_INIT_END]"
        echo ""
        echo "Line numbers:"
        echo "  [K][EARLY_BOOT_OK]: line $early_line"
        echo "  [K][LATE_INIT_END]: line $late_line"
        echo "  [[AYKEN_BOOT_OK]]: line $boot_line"
        echo ""
        echo "Last 50 lines of boot log:"
        tail -50 "$BOOT_LOG"
        exit 1
    fi
    
    if [ "$late_line" -gt "$boot_line" ]; then
        echo "❌ BOOT FAILED: Marker sequence violation"
        echo ""
        echo "Expected order: [K][EARLY_BOOT_OK] → [K][LATE_INIT_END] → [[AYKEN_BOOT_OK]]"
        echo "Actual order: [K][LATE_INIT_END] appears AFTER [[AYKEN_BOOT_OK]]"
        echo ""
        echo "Line numbers:"
        echo "  [K][EARLY_BOOT_OK]: line $early_line"
        echo "  [K][LATE_INIT_END]: line $late_line"
        echo "  [[AYKEN_BOOT_OK]]: line $boot_line"
        echo ""
        echo "Last 50 lines of boot log:"
        tail -50 "$BOOT_LOG"
        exit 1
    fi
    
    # All checks passed
    echo "✅ Smoke boot PASS"
    echo "   Marker sequence validated: EARLY(line $early_line) → LATE(line $late_line) → BOOT_OK(line $boot_line)"
    echo ""
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
    *)
        echo "Usage: $0 {smoke|contract|full}"
        echo ""
        echo "Modes:"
        echo "  smoke    - Quick build + boot check (5-10s)"
        echo "  contract - Build + boot + runtime contract tests"
        echo "  full     - Build + boot + all evidence tests"
        exit 2
        ;;
esac

echo "✅ PASS: $MODE mode"

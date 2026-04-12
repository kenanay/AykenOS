#!/bin/bash

# Boot Observability Determinism Test
#
# This script verifies that the boot observability harness produces
# non-zero output consistently across multiple runs, testing for
# nondeterministic behavior that would cause CI flakiness.
#
# Author: Kenan AY - Architectural Steward
# Spec: .kiro/specs/boot-chain-observability-restoration/
# Status: Production readiness verification

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Test configuration
NUM_RUNS="${NUM_RUNS:-10}"
PARALLEL_RUNS="${PARALLEL_RUNS:-3}"

log_info "Boot Observability Determinism Test"
log_info "===================================="
log_info "Sequential runs: $NUM_RUNS"
log_info "Parallel runs: $PARALLEL_RUNS"
log_info ""

# Check prerequisites
if [[ ! -f "$PROJECT_ROOT/EFI.img" ]]; then
    log_error "EFI image not found. Run 'make efi-img' first."
    exit 1
fi

HARNESS="$PROJECT_ROOT/scripts/qemu-boot-observability-harness.sh"
if [[ ! -x "$HARNESS" ]]; then
    log_error "Harness not found or not executable: $HARNESS"
    exit 1
fi

# Test 1: Sequential Execution Determinism
log_info "Test 1: Sequential Execution (${NUM_RUNS} runs)"
log_info "----------------------------------------------"

SEQUENTIAL_PASS=0
SEQUENTIAL_FAIL=0

for i in $(seq 1 $NUM_RUNS); do
    log_info "Run $i/$NUM_RUNS..."
    
    if "$HARNESS" > /dev/null 2>&1; then
        DEBUGCON_SIZE=$(stat -c%s "$PROJECT_ROOT/evidence/boot-observability/qemu_debugcon.log" 2>/dev/null || stat -f%z "$PROJECT_ROOT/evidence/boot-observability/qemu_debugcon.log" 2>/dev/null || echo "0")
        SERIAL_SIZE=$(stat -c%s "$PROJECT_ROOT/evidence/boot-observability/qemu_serial.log" 2>/dev/null || stat -f%z "$PROJECT_ROOT/evidence/boot-observability/qemu_serial.log" 2>/dev/null || echo "0")
        
        if [[ $DEBUGCON_SIZE -gt 0 ]] || [[ $SERIAL_SIZE -gt 0 ]]; then
            log_success "  ✓ Run $i: debugcon=${DEBUGCON_SIZE}B, serial=${SERIAL_SIZE}B"
            SEQUENTIAL_PASS=$((SEQUENTIAL_PASS + 1))
        else
            log_error "  ✗ Run $i: ALL CHANNELS ZERO (nondeterministic failure)"
            SEQUENTIAL_FAIL=$((SEQUENTIAL_FAIL + 1))
        fi
    else
        log_error "  ✗ Run $i: Harness failed"
        SEQUENTIAL_FAIL=$((SEQUENTIAL_FAIL + 1))
    fi
done

log_info ""
log_info "Sequential Test Results: $SEQUENTIAL_PASS/$NUM_RUNS PASS, $SEQUENTIAL_FAIL/$NUM_RUNS FAIL"

if [[ $SEQUENTIAL_FAIL -gt 0 ]]; then
    log_error "SEQUENTIAL DETERMINISM FAILURE: System is not reliable"
    log_error "This indicates nondeterministic behavior that will cause CI flakiness"
fi

# Test 2: Parallel Execution Safety
log_info ""
log_info "Test 2: Parallel Execution (${PARALLEL_RUNS} simultaneous runs)"
log_info "----------------------------------------------------------------"

PARALLEL_PASS=0
PARALLEL_FAIL=0

# Create temporary evidence directories for parallel runs
TEMP_EVIDENCE_BASE="$PROJECT_ROOT/evidence/boot-observability-parallel-test"
mkdir -p "$TEMP_EVIDENCE_BASE"

# Launch parallel runs
PIDS=()
for i in $(seq 1 $PARALLEL_RUNS); do
    (
        # Each parallel run uses its own evidence directory and NVRAM
        EVIDENCE_DIR="$TEMP_EVIDENCE_BASE/run-$i"
        mkdir -p "$EVIDENCE_DIR"
        
        # Create per-run NVRAM
        NVRAM_RUN="$PROJECT_ROOT/build/OVMF_VARS_RUN_parallel_$i.fd"
        cp -f "$PROJECT_ROOT/firmware/ovmf/OVMF_VARS.fd" "$NVRAM_RUN"
        
        # Run harness with isolated evidence directory
        cd "$PROJECT_ROOT"
        timeout --signal=SIGINT 45 qemu-system-x86_64 \
            -machine q35 \
            -drive if=pflash,format=raw,readonly=on,file="$PROJECT_ROOT/firmware/ovmf/OVMF_CODE.fd" \
            -drive if=pflash,format=raw,file="$NVRAM_RUN" \
            -drive format=raw,file="$PROJECT_ROOT/EFI.img" \
            -boot order=c \
            -debugcon file:$EVIDENCE_DIR/qemu_debugcon.log \
            -global isa-debugcon.iobase=0xe9 \
            -serial file:$EVIDENCE_DIR/qemu_serial.log \
            -nographic \
            < /dev/null > "$TEMP_EVIDENCE_BASE/run-$i.log" 2>&1
        
        QEMU_EXIT=$?
        
        # Explicit flush
        wait
        sync
        sleep 1
        
        # Clean up NVRAM
        rm -f "$NVRAM_RUN"
        
        # Check if evidence was captured
        DEBUGCON_SIZE=$(stat -c%s "$EVIDENCE_DIR/qemu_debugcon.log" 2>/dev/null || stat -f%z "$EVIDENCE_DIR/qemu_debugcon.log" 2>/dev/null || echo "0")
        SERIAL_SIZE=$(stat -c%s "$EVIDENCE_DIR/qemu_serial.log" 2>/dev/null || stat -f%z "$EVIDENCE_DIR/qemu_serial.log" 2>/dev/null || echo "0")
        
        if [[ $DEBUGCON_SIZE -gt 0 ]] || [[ $SERIAL_SIZE -gt 0 ]]; then
            echo 0 > "$TEMP_EVIDENCE_BASE/run-$i.exit"
        else
            echo 1 > "$TEMP_EVIDENCE_BASE/run-$i.exit"
        fi
    ) &
    PIDS+=($!)
done

# Wait for all parallel runs to complete
log_info "Waiting for $PARALLEL_RUNS parallel runs to complete..."
for pid in "${PIDS[@]}"; do
    wait $pid
done

# Check results
for i in $(seq 1 $PARALLEL_RUNS); do
    EXIT_CODE=$(cat "$TEMP_EVIDENCE_BASE/run-$i.exit" 2>/dev/null || echo "1")
    
    if [[ $EXIT_CODE -eq 0 ]]; then
        DEBUGCON_SIZE=$(stat -c%s "$TEMP_EVIDENCE_BASE/run-$i/qemu_debugcon.log" 2>/dev/null || stat -f%z "$TEMP_EVIDENCE_BASE/run-$i/qemu_debugcon.log" 2>/dev/null || echo "0")
        SERIAL_SIZE=$(stat -c%s "$TEMP_EVIDENCE_BASE/run-$i/qemu_serial.log" 2>/dev/null || stat -f%z "$TEMP_EVIDENCE_BASE/run-$i/qemu_serial.log" 2>/dev/null || echo "0")
        
        if [[ $DEBUGCON_SIZE -gt 0 ]] || [[ $SERIAL_SIZE -gt 0 ]]; then
            log_success "  ✓ Parallel run $i: debugcon=${DEBUGCON_SIZE}B, serial=${SERIAL_SIZE}B"
            PARALLEL_PASS=$((PARALLEL_PASS + 1))
        else
            log_error "  ✗ Parallel run $i: ALL CHANNELS ZERO"
            PARALLEL_FAIL=$((PARALLEL_FAIL + 1))
        fi
    else
        log_error "  ✗ Parallel run $i: Harness failed (exit code $EXIT_CODE)"
        PARALLEL_FAIL=$((PARALLEL_FAIL + 1))
    fi
done

# Clean up parallel test artifacts
rm -rf "$TEMP_EVIDENCE_BASE"

log_info ""
log_info "Parallel Test Results: $PARALLEL_PASS/$PARALLEL_RUNS PASS, $PARALLEL_FAIL/$PARALLEL_RUNS FAIL"

if [[ $PARALLEL_FAIL -gt 0 ]]; then
    log_error "PARALLEL EXECUTION FAILURE: System is not safe for concurrent use"
    log_error "This indicates race conditions or resource contention"
fi

# Final verdict
log_info ""
log_info "========================================="
log_info "Determinism Test Summary"
log_info "========================================="
log_info "Sequential: $SEQUENTIAL_PASS/$NUM_RUNS PASS ($SEQUENTIAL_FAIL failures)"
log_info "Parallel: $PARALLEL_PASS/$PARALLEL_RUNS PASS ($PARALLEL_FAIL failures)"
log_info ""

TOTAL_FAIL=$((SEQUENTIAL_FAIL + PARALLEL_FAIL))

if [[ $TOTAL_FAIL -eq 0 ]]; then
    log_success "✓ PRODUCTION-READY: System is deterministic and reliable"
    log_success "  - Sequential execution: 100% success rate"
    log_success "  - Parallel execution: Safe for concurrent use"
    log_success "  - Ready for CI integration"
    exit 0
else
    log_error "✗ NOT PRODUCTION-READY: Nondeterministic behavior detected"
    log_error "  - Total failures: $TOTAL_FAIL"
    log_error "  - System will cause CI flakiness"
    log_error "  - Further investigation required"
    exit 1
fi

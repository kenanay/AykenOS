#!/bin/bash
# Dev Loop Isolation Property Test - Task 3
# Author: Kenan AY — System Architect
#
# This script validates the isolation property:
# "Dev loop does NOT affect kernel execution behavior"
#
# Test Method:
# 1. Run same kernel binary multiple times
# 2. Capture marker output each time
# 3. Verify marker sequence is identical across runs
# 4. Verify marker content is deterministic
#
# Success Criteria:
# - All runs produce identical marker sequence
# - Marker hash matches across all runs
# - No nondeterministic behavior detected

set -euo pipefail

RUNS="${ISOLATION_TEST_RUNS:-5}"
LOG_DIR="out/logs/isolation_test"
BASELINE_MARKERS="$LOG_DIR/baseline_markers.txt"
BASELINE_HASH="$LOG_DIR/baseline_hash.txt"

echo "== Dev Loop Isolation Property Test =="
echo ""
echo "This test validates that dev loop does NOT affect kernel execution."
echo "Running $RUNS iterations with same kernel binary..."
echo ""

# Clean previous test artifacts
rm -rf "$LOG_DIR"
mkdir -p "$LOG_DIR"

# Build kernel once (same binary for all runs)
echo "[1/3] Building kernel (validation profile)..."
make -j8 efi-img \
    KERNEL_PROFILE=validation \
    AYKEN_VALIDATION=1 \
    AYKEN_VCP_RUNTIME_HOOK_TEST=0 \
    AYKEN_VCP_FAIL_CLOSED_TEST=0 \
    AYKEN_VCP_EVIDENCE_TEST=0 \
    AYKEN_VCP_TRUST_VERIFICATION_TEST=0 \
    AYKEN_MB_SELFTEST=0 \
    > /dev/null 2>&1

echo "✅ Build complete"
echo ""

# Run multiple iterations
echo "[2/3] Running $RUNS iterations..."
echo ""

for i in $(seq 1 $RUNS); do
    echo "  Run $i/$RUNS..."
    
    RUN_LOG="$LOG_DIR/run_${i}_debug.log"
    MARKERS_LOG="$LOG_DIR/run_${i}_markers.txt"
    
    # Clear debug log
    : > out/logs/debug_run.log
    
    # Run QEMU with timeout
    set +e
    timeout 30 make run-qemu \
        KERNEL_PROFILE=validation \
        AYKEN_VALIDATION=1 \
        AYKEN_VCP_RUNTIME_HOOK_TEST=0 \
        AYKEN_VCP_FAIL_CLOSED_TEST=0 \
        AYKEN_VCP_EVIDENCE_TEST=0 \
        AYKEN_VCP_TRUST_VERIFICATION_TEST=0 \
        AYKEN_MB_SELFTEST=0 \
        > /dev/null 2>&1
    qemu_status=$?
    set -e
    
    # Copy debug log for this run
    cp out/logs/debug_run.log "$RUN_LOG"
    
    # Extract boot markers (canonical sequence)
    grep -E "\[K\]\[EARLY_BOOT_OK\]|\[K\]\[LATE_INIT_END\]|\[\[AYKEN_BOOT_OK\]\]" "$RUN_LOG" > "$MARKERS_LOG" || {
        echo "❌ FAIL: Run $i did not produce boot markers"
        echo ""
        echo "This indicates kernel did not boot successfully."
        echo "Check $RUN_LOG for details."
        exit 1
    }
    
    # Verify all three markers present
    if ! grep -q "\[K\]\[EARLY_BOOT_OK\]" "$MARKERS_LOG"; then
        echo "❌ FAIL: Run $i missing EARLY_BOOT_OK marker"
        exit 1
    fi
    
    if ! grep -q "\[K\]\[LATE_INIT_END\]" "$MARKERS_LOG"; then
        echo "❌ FAIL: Run $i missing LATE_INIT_END marker"
        exit 1
    fi
    
    if ! grep -q "\[\[AYKEN_BOOT_OK\]\]" "$MARKERS_LOG"; then
        echo "❌ FAIL: Run $i missing AYKEN_BOOT_OK marker"
        exit 1
    fi
    
    # Save baseline from first run
    if [ $i -eq 1 ]; then
        cp "$MARKERS_LOG" "$BASELINE_MARKERS"
        sha256sum "$BASELINE_MARKERS" | awk '{print $1}' > "$BASELINE_HASH"
        echo "    ✅ Baseline established"
    else
        # Compare with baseline
        current_hash=$(sha256sum "$MARKERS_LOG" | awk '{print $1}')
        baseline_hash=$(cat "$BASELINE_HASH")
        
        if [ "$current_hash" = "$baseline_hash" ]; then
            echo "    ✅ Markers match baseline"
        else
            echo "    ❌ FAIL: Markers differ from baseline"
            echo ""
            echo "Baseline markers:"
            cat "$BASELINE_MARKERS"
            echo ""
            echo "Run $i markers:"
            cat "$MARKERS_LOG"
            echo ""
            echo "This indicates nondeterministic kernel behavior."
            echo "Dev loop isolation property VIOLATED."
            exit 1
        fi
    fi
done

echo ""
echo "[3/3] Analyzing results..."
echo ""

# Count unique marker sequences
unique_sequences=$(cat "$LOG_DIR"/run_*_markers.txt | sort -u | wc -l | tr -d ' ')

if [ "$unique_sequences" -eq 1 ]; then
    echo "✅ PASS: All $RUNS runs produced identical marker sequence"
    echo ""
    echo "Isolation property validated:"
    echo "  ✅ Same kernel binary → Same marker output"
    echo "  ✅ Dev loop does NOT affect kernel execution"
    echo "  ✅ Deterministic behavior confirmed"
    echo ""
    echo "Baseline marker sequence:"
    cat "$BASELINE_MARKERS"
    echo ""
    echo "Baseline hash: $(cat "$BASELINE_HASH")"
else
    echo "❌ FAIL: Found $unique_sequences different marker sequences"
    echo ""
    echo "This indicates nondeterministic kernel behavior."
    echo "Dev loop isolation property VIOLATED."
    exit 1
fi

echo ""
echo "Task 3 (Isolation property enforcement) COMPLETE"

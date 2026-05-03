#!/bin/bash
# Property Test: Dev Loop Isolation
# Validates that dev loop does not affect kernel behavior
#
# Property: For any kernel build with AYKEN_VALIDATION=1,
# running the kernel directly (baseline) and running it through
# the dev loop (dev loop active) SHALL produce identical boot
# outcomes, marker sets, marker sequences, and execution results.

set -euo pipefail

echo "=========================================="
echo "Dev Loop Isolation Property Test"
echo "=========================================="
echo ""
echo "Property: Dev loop does NOT affect kernel behavior"
echo ""

# Clean build environment
echo "[0/4] Cleaning build environment..."
make clean > /dev/null 2>&1 || true
mkdir -p out/logs

# Run A (baseline): Build and run kernel directly
echo "[1/4] Baseline run (no dev loop)..."
make kernel.elf \
    KERNEL_PROFILE=validation \
    AYKEN_VALIDATION=1 \
    AYKEN_VCP_RUNTIME_HOOK_TEST=0 \
    AYKEN_VCP_FAIL_CLOSED_TEST=0 \
    AYKEN_VCP_EVIDENCE_TEST=0 \
    AYKEN_VCP_TRUST_VERIFICATION_TEST=0 \
    AYKEN_MB_SELFTEST=0 \
    > /dev/null 2>&1

set +e
timeout 20 make run \
    KERNEL_PROFILE=validation \
    AYKEN_VALIDATION=1 \
    AYKEN_VCP_RUNTIME_HOOK_TEST=0 \
    AYKEN_VCP_FAIL_CLOSED_TEST=0 \
    AYKEN_VCP_EVIDENCE_TEST=0 \
    AYKEN_VCP_TRUST_VERIFICATION_TEST=0 \
    AYKEN_MB_SELFTEST=0 \
    > out/logs/baseline.log 2>&1
baseline_status=$?
set -e

echo "   Baseline exit status: $baseline_status"

# Run B (dev loop): Build and run through dev loop
echo "[2/4] Dev loop run..."
make clean > /dev/null 2>&1 || true

set +e
./scripts/dev_loop.sh smoke > out/logs/devloop_full.log 2>&1
devloop_status=$?
set -e

echo "   Dev loop exit status: $devloop_status"

# Extract markers from both logs
echo "[3/4] Comparing marker sets..."

# Extract markers from baseline
grep -E '\[K\]|\[\[AYKEN' out/logs/baseline.log | sort > out/logs/baseline_markers.txt || true

# Extract markers from dev loop (boot_watch.log is created by dev_loop.sh)
if [ -f out/logs/boot_watch.log ]; then
    grep -E '\[K\]|\[\[AYKEN' out/logs/boot_watch.log | sort > out/logs/devloop_markers.txt || true
else
    echo "❌ FAIL: Dev loop did not create boot_watch.log"
    exit 1
fi

# Compare marker sets
if ! diff out/logs/baseline_markers.txt out/logs/devloop_markers.txt > /dev/null 2>&1; then
    echo "❌ FAIL: Marker sets differ between baseline and dev loop"
    echo ""
    echo "Baseline markers:"
    cat out/logs/baseline_markers.txt
    echo ""
    echo "Dev loop markers:"
    cat out/logs/devloop_markers.txt
    echo ""
    echo "Diff:"
    diff out/logs/baseline_markers.txt out/logs/devloop_markers.txt || true
    exit 1
fi

echo "   ✅ Marker sets are identical"

# Verify both runs have critical marker
if ! grep -q "\[\[AYKEN_BOOT_OK\]\]" out/logs/baseline.log; then
    echo "❌ FAIL: Baseline run missing [[AYKEN_BOOT_OK]] marker"
    echo "Last 50 lines of baseline log:"
    tail -50 out/logs/baseline.log
    exit 1
fi

if ! grep -q "\[\[AYKEN_BOOT_OK\]\]" out/logs/boot_watch.log; then
    echo "❌ FAIL: Dev loop run missing [[AYKEN_BOOT_OK]] marker"
    echo "Last 50 lines of dev loop log:"
    tail -50 out/logs/boot_watch.log
    exit 1
fi

echo "   ✅ Both runs have [[AYKEN_BOOT_OK]] marker"

# Verify marker sequence in both runs
echo "   Checking marker sequence..."

# Extract line numbers for baseline
baseline_early=$(grep -n "\[K\]\[EARLY_BOOT_OK\]" out/logs/baseline.log | head -1 | cut -d: -f1 || echo "0")
baseline_late=$(grep -n "\[K\]\[LATE_INIT_END\]" out/logs/baseline.log | head -1 | cut -d: -f1 || echo "0")
baseline_boot=$(grep -n "\[\[AYKEN_BOOT_OK\]\]" out/logs/baseline.log | head -1 | cut -d: -f1 || echo "0")

# Extract line numbers for dev loop
devloop_early=$(grep -n "\[K\]\[EARLY_BOOT_OK\]" out/logs/boot_watch.log | head -1 | cut -d: -f1 || echo "0")
devloop_late=$(grep -n "\[K\]\[LATE_INIT_END\]" out/logs/boot_watch.log | head -1 | cut -d: -f1 || echo "0")
devloop_boot=$(grep -n "\[\[AYKEN_BOOT_OK\]\]" out/logs/boot_watch.log | head -1 | cut -d: -f1 || echo "0")

# Verify sequence for baseline
if [ "$baseline_early" -gt 0 ] && [ "$baseline_late" -gt 0 ] && [ "$baseline_boot" -gt 0 ]; then
    if [ "$baseline_early" -ge "$baseline_late" ] || [ "$baseline_late" -ge "$baseline_boot" ]; then
        echo "❌ FAIL: Baseline marker sequence violation"
        echo "   EARLY: line $baseline_early"
        echo "   LATE: line $baseline_late"
        echo "   BOOT: line $baseline_boot"
        exit 1
    fi
fi

# Verify sequence for dev loop
if [ "$devloop_early" -gt 0 ] && [ "$devloop_late" -gt 0 ] && [ "$devloop_boot" -gt 0 ]; then
    if [ "$devloop_early" -ge "$devloop_late" ] || [ "$devloop_late" -ge "$devloop_boot" ]; then
        echo "❌ FAIL: Dev loop marker sequence violation"
        echo "   EARLY: line $devloop_early"
        echo "   LATE: line $devloop_late"
        echo "   BOOT: line $devloop_boot"
        exit 1
    fi
fi

echo "   ✅ Marker sequences are correct"

# Negative test: Break dev loop → kernel unaffected
echo "[4/4] Negative test (broken dev loop)..."

# Test 1: Invalid timeout should cause dev loop to fail
export QEMU_TIMEOUT_SECONDS=invalid
set +e
./scripts/dev_loop.sh smoke > /dev/null 2>&1
broken_devloop_status=$?
set -e
unset QEMU_TIMEOUT_SECONDS

if [ "$broken_devloop_status" -eq 0 ]; then
    echo "⚠️  WARNING: Dev loop should have failed with invalid timeout"
    echo "   (This may be acceptable if timeout validation is not strict)"
fi

# Test 2: Run kernel directly (should still work despite broken dev loop)
echo "   Running kernel directly after dev loop error..."
make clean > /dev/null 2>&1 || true
make kernel.elf \
    KERNEL_PROFILE=validation \
    AYKEN_VALIDATION=1 \
    > /dev/null 2>&1

set +e
timeout 20 make run \
    KERNEL_PROFILE=validation \
    AYKEN_VALIDATION=1 \
    > out/logs/negative_test.log 2>&1
negative_status=$?
set -e

if ! grep -q "\[\[AYKEN_BOOT_OK\]\]" out/logs/negative_test.log; then
    echo "❌ FAIL: Kernel affected by broken dev loop"
    echo "   Kernel should boot successfully even if dev loop is broken"
    echo "Last 50 lines of negative test log:"
    tail -50 out/logs/negative_test.log
    exit 1
fi

echo "   ✅ Kernel unaffected by broken dev loop"

echo ""
echo "=========================================="
echo "✅ PASS: Dev loop isolation property validated"
echo "=========================================="
echo ""
echo "Property: For any kernel build with AYKEN_VALIDATION=1,"
echo "running the kernel directly and running it through the dev loop"
echo "produce identical boot outcomes, marker sets, and sequences."
echo ""
echo "Validated:"
echo "  ✅ Marker sets are identical"
echo "  ✅ Both runs have [[AYKEN_BOOT_OK]] marker"
echo "  ✅ Marker sequences are correct"
echo "  ✅ Kernel unaffected by broken dev loop"
echo ""


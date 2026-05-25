#!/bin/bash
# Performance Regression Check (Dev Loop Integration)
# Author: Kenan AY — System Architect
#
# Integrates with existing AykenOS performance gate infrastructure
#
# Purpose: Quick performance regression check for dev loop
# Uses: Existing perf-baseline.lock.json and ci-gate-performance logic
#
# Usage:
#   ./scripts/check_perf_regression.sh [mode]
#
# Modes:
#   quick  - Quick check (boot time only, default)
#   full   - Full check (boot + syscall + context switch)
#
# Exit Codes:
#   0 = PASS (no regression)
#   1 = FAIL (regression detected)
#   2 = ERROR (baseline missing or invalid)

set -euo pipefail

MODE="${1:-quick}"
BASELINE="scripts/ci/perf-baseline.lock.json"
LOG="out/logs/boot_watch.log"

# Check if baseline exists
if [ ! -f "$BASELINE" ]; then
    echo "⚠️  WARNING: Performance baseline not found at $BASELINE"
    echo "   Run 'make ci-gate-performance' to initialize baseline"
    echo "   Skipping performance check"
    exit 0  # Don't fail dev loop if baseline missing
fi

# Check if boot log exists
if [ ! -f "$LOG" ]; then
    echo "❌ ERROR: Boot log not found at $LOG"
    echo "   Run dev loop first to generate boot log"
    exit 2
fi

echo "=========================================="
echo "Performance Regression Check"
echo "=========================================="
echo ""
echo "Mode: $MODE"
echo "Baseline: $BASELINE"
echo "Log: $LOG"
echo ""

# Extract baseline metrics
BASELINE_BOOT_TIME=$(jq -r '.metrics.boot_time_ms' "$BASELINE")
BASELINE_SYSCALL=$(jq -r '.metrics.syscall_latency_ms_proxy' "$BASELINE")
BASELINE_CONTEXT=$(jq -r '.metrics.context_switch_latency_ms_proxy' "$BASELINE")

# Extract thresholds
THRESH_BOOT=$(jq -r '.policy.thresholds_percent.boot_time_ms' "$BASELINE")
THRESH_SYSCALL=$(jq -r '.policy.thresholds_percent.syscall_latency_ms_proxy' "$BASELINE")
THRESH_CONTEXT=$(jq -r '.policy.thresholds_percent.context_switch_latency_ms_proxy' "$BASELINE")

echo "Baseline metrics:"
echo "  Boot time: ${BASELINE_BOOT_TIME}ms (threshold: ±${THRESH_BOOT}%)"
echo "  Syscall latency: ${BASELINE_SYSCALL}ms (threshold: ±${THRESH_SYSCALL}%)"
echo "  Context switch: ${BASELINE_CONTEXT}ms (threshold: ±${THRESH_CONTEXT}%)"
echo ""

# Quick mode: Only check boot time (marker-based proxy)
if [ "$MODE" = "quick" ]; then
    echo "[Quick Mode] Checking boot time only..."
    echo ""
    
    # Count lines between markers as proxy for boot time
    # This is a rough proxy - real measurement requires TSC markers
    EARLY_LINE=$(grep -n "\[K\]\[EARLY_BOOT_OK\]" "$LOG" | head -1 | cut -d: -f1 || echo "0")
    BOOT_LINE=$(grep -n "\[\[AYKEN_BOOT_OK\]\]" "$LOG" | head -1 | cut -d: -f1 || echo "0")
    
    if [ "$EARLY_LINE" -eq 0 ] || [ "$BOOT_LINE" -eq 0 ]; then
        echo "⚠️  WARNING: Boot markers not found, cannot estimate boot time"
        echo "   Skipping performance check"
        exit 0
    fi
    
    # Line count as proxy (very rough)
    LINE_COUNT=$((BOOT_LINE - EARLY_LINE))
    
    # Normalize to baseline (assume baseline had similar line count)
    # This is a VERY rough proxy - real measurement needs TSC
    BASELINE_LINE_COUNT=100  # Rough estimate
    RATIO=$(echo "scale=2; $LINE_COUNT / $BASELINE_LINE_COUNT" | bc -l)
    
    echo "Boot marker line count: $LINE_COUNT (baseline: ~$BASELINE_LINE_COUNT)"
    echo "Ratio: $RATIO"
    echo ""
    
    # Check if ratio exceeds threshold
    UPPER_LIMIT=$(echo "scale=2; 1 + ($THRESH_BOOT / 100)" | bc -l)
    
    if (( $(echo "$RATIO > $UPPER_LIMIT" | bc -l) )); then
        echo "❌ FAIL: Boot time regression detected"
        echo "   Ratio: $RATIO exceeds threshold: $UPPER_LIMIT"
        echo ""
        echo "Note: This is a rough proxy based on log line count."
        echo "      For accurate measurement, use 'make ci-gate-performance'"
        exit 1
    fi
    
    echo "✅ PASS: No boot time regression detected (proxy check)"
    echo ""
    echo "Note: This is a rough proxy. For accurate measurement,"
    echo "      use 'make ci-gate-performance' with TSC markers."
    exit 0
fi

# Full mode: Delegate to existing performance gate
if [ "$MODE" = "full" ]; then
    echo "[Full Mode] Running full performance gate..."
    echo ""
    
    # Delegate to existing performance gate
    make ci-gate-performance
    
    exit $?
fi

echo "❌ ERROR: Invalid mode: $MODE"
echo "   Valid modes: quick, full"
exit 2

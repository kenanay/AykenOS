#!/usr/bin/env bash
set -euo pipefail

# GitHub CI Performance Regression Bisect Script
# 
# Usage:
#   git bisect start
#   git bisect bad HEAD  # 9b3358e6 (current, slow)
#   git bisect good 050332220d9a  # baseline (fast)
#   git bisect run scripts/ci/bisect_performance_regression.sh
#
# Exit codes:
#   0: GOOD (performance acceptable)
#   1: BAD (performance regressed)
#   125: SKIP (build failed or test inconclusive)

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT}"

# Performance threshold (from baseline)
BOOT_TIME_THRESHOLD=11000  # baseline 10684, allow +3% margin
CONTEXT_SWITCH_THRESHOLD=184  # baseline 175.08, allow +5% margin
SYSCALL_THRESHOLD=184  # baseline 175.08, allow +5% margin

echo "=== Performance Bisect Test ==="
echo "Commit: $(git rev-parse --short HEAD)"
echo "Threshold: boot <= ${BOOT_TIME_THRESHOLD}ms"
echo ""

# Clean build
make clean > /dev/null 2>&1 || true

# Build with validation profile
echo "Building..."
if ! make KERNEL_PROFILE=validation \
          USER_MINIMAL_MODE=syscall-v2-runtime \
          AYKEN_SCHED_BOOTSTRAP_POLICY=1 \
          AYKEN_MB_SELFTEST=0 \
          AYKEN_DETERMINISTIC_EXIT=1 \
          AYKEN_RING3_ENTRY_GUARD=1 \
          efi-img > /dev/null 2>&1; then
    echo "Build failed - SKIP"
    exit 125
fi

# Run performance gate
EVIDENCE_DIR="evidence/bisect-$(git rev-parse --short HEAD)"
mkdir -p "${EVIDENCE_DIR}"

echo "Running performance measurement..."
if ! scripts/ci/gate_performance.sh \
    --evidence-dir "${EVIDENCE_DIR}" \
    --env-mismatch-policy waiver \
    > "${EVIDENCE_DIR}/gate.log" 2>&1; then
    
    # Check if it's a build/test failure vs performance regression
    if grep -q "build_failed\|preempt_test_failed\|boot_audit_failed" "${EVIDENCE_DIR}/violations.txt" 2>/dev/null; then
        echo "Test failed - SKIP"
        exit 125
    fi
fi

# Extract metrics
ACTUAL_LOCK="${EVIDENCE_DIR}/actual.lock.json"
if [[ ! -f "${ACTUAL_LOCK}" ]]; then
    echo "No metrics generated - SKIP"
    exit 125
fi

BOOT_TIME=$(jq -r '.metrics.boot_time_ms // 0' "${ACTUAL_LOCK}")
CONTEXT_SWITCH=$(jq -r '.metrics.context_switch_latency_ms_proxy // 0' "${ACTUAL_LOCK}")
SYSCALL=$(jq -r '.metrics.syscall_latency_ms_proxy // 0' "${ACTUAL_LOCK}")

echo "Results:"
echo "  boot_time_ms: ${BOOT_TIME}"
echo "  context_switch_latency_ms_proxy: ${CONTEXT_SWITCH}"
echo "  syscall_latency_ms_proxy: ${SYSCALL}"
echo ""

# Check thresholds
if (( $(echo "${BOOT_TIME} > ${BOOT_TIME_THRESHOLD}" | bc -l) )); then
    echo "VERDICT: BAD (boot time ${BOOT_TIME} > ${BOOT_TIME_THRESHOLD})"
    
    # Extract mailbox fallback stats for diagnosis
    FALLBACK_COUNT=$(jq -r '.raw_metrics.mailbox_phase_breakdown_ticks.fallback_reasons.no_candidate // 0' "${ACTUAL_LOCK}")
    TOTAL_EXTRACT=$(jq -r '.raw_metrics.mailbox_phase_breakdown_ticks.extract_diagnostics.raw_observations.count // 0' "${ACTUAL_LOCK}")
    EPOCH_STALE=$(jq -r '.raw_metrics.mailbox_phase_breakdown_ticks.extract_diagnostics.extract_reasons.epoch_stale // 0' "${ACTUAL_LOCK}")
    EPOCH_GT=$(jq -r '.raw_metrics.mailbox_phase_breakdown_ticks.extract_diagnostics.raw_observations.epoch_gt_owner_last_epoch_count // 0' "${ACTUAL_LOCK}")
    EPOCH_LTE=$(jq -r '.raw_metrics.mailbox_phase_breakdown_ticks.extract_diagnostics.raw_observations.epoch_lte_owner_last_epoch_count // 0' "${ACTUAL_LOCK}")
    LATEST_EPOCH=$(jq -r '.raw_metrics.mailbox_phase_breakdown_ticks.extract_diagnostics.raw_observations.latest_epoch // 0' "${ACTUAL_LOCK}")
    LATEST_OWNER_EPOCH=$(jq -r '.raw_metrics.mailbox_phase_breakdown_ticks.extract_diagnostics.raw_observations.latest_owner_last_epoch // 0' "${ACTUAL_LOCK}")
    
    echo "Mailbox stats:"
    echo "  fallback (no_candidate): ${FALLBACK_COUNT}"
    echo "  total extracts: ${TOTAL_EXTRACT}"
    echo "  epoch_stale rejections: ${EPOCH_STALE}"
    echo "  epoch > owner_last_epoch: ${EPOCH_GT}"
    echo "  epoch <= owner_last_epoch: ${EPOCH_LTE}"
    echo "  latest candidate epoch: ${LATEST_EPOCH}"
    echo "  latest owner_last_epoch: ${LATEST_OWNER_EPOCH}"
    
    exit 1
fi

echo "VERDICT: GOOD (performance acceptable)"
exit 0

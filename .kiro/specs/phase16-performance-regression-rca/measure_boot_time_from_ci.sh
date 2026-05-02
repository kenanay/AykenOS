#!/usr/bin/env bash
# Extract boot_time_ms from CI-style measurement
# This script runs the actual CI performance gate to get accurate boot_time measurements

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
CONFIG_NAME="${1:-test}"
BCIB_WORKER="${2:-1}"
BOUNDARY="${3:-1}"
PROBE="${4:-1}"
DIAG="${5:-1}"

echo "========================================="
echo "CI-Style Boot Time Measurement"
echo "Configuration: ${CONFIG_NAME}"
echo "  BCIB_WORKER=${BCIB_WORKER}"
echo "  BOUNDARY=${BOUNDARY}"
echo "  PROBE=${PROBE}"
echo "  DIAG=${DIAG}"
echo "========================================="

# Clean build
make -C "${ROOT}" clean > /dev/null 2>&1 || true

# Build with feature toggles
echo "Building kernel..."
if ! make -C "${ROOT}" \
    KERNEL_PROFILE=validation \
    USER_MINIMAL_MODE=syscall-v2-runtime \
    AYKEN_PHASE16_BCIB_WORKER_ENABLE="${BCIB_WORKER}" \
    AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE="${BOUNDARY}" \
    AYKEN_PHASE16_PROBE_VALIDATION_ENABLE="${PROBE}" \
    AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE="${DIAG}" \
    efi-img > /tmp/build_${CONFIG_NAME}.log 2>&1; then
    echo "BUILD FAILED - see /tmp/build_${CONFIG_NAME}.log"
    exit 1
fi

# Run CI performance gate to get boot_time_ms
echo "Running CI performance gate..."
export PERF_BASELINE_ENFORCEMENT=0  # Disable enforcement, just measure
export PERF_DRIFT_ENFORCEMENT=0
export PERF_ENV_MISMATCH_POLICY=waiver

if bash "${ROOT}/scripts/ci/gate_performance.sh" > /tmp/ci_${CONFIG_NAME}.log 2>&1; then
    # Extract boot_time_ms from the report
    REPORT_JSON="${ROOT}/out/reports/perf-gate-$(date +%Y-%m-%d)/report.json"
    if [ -f "${REPORT_JSON}" ]; then
        BOOT_TIME=$(python3 -c "import json; print(json.load(open('${REPORT_JSON}'))['metrics']['boot_time_ms'])" 2>/dev/null || echo "FAILED")
        echo "boot_time_ms: ${BOOT_TIME}"
        echo "${BOOT_TIME}"
    else
        echo "Report not found"
        exit 1
    fi
else
    echo "CI gate failed - see /tmp/ci_${CONFIG_NAME}.log"
    tail -50 /tmp/ci_${CONFIG_NAME}.log
    exit 1
fi

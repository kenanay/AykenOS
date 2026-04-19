#!/usr/bin/env bash
# Get CI artifacts from GitHub Actions run 24634827102
# This will download the authoritative performance test results

set -euo pipefail

RUN_ID="24634827102"
DOWNLOAD_DIR="/tmp/freeze-artifacts-${RUN_ID}"

echo "=== Downloading CI Artifacts from Run ${RUN_ID} ==="
echo "Download directory: ${DOWNLOAD_DIR}"
echo ""

# Download all artifacts from the CI run
gh run download "${RUN_ID}" --dir "${DOWNLOAD_DIR}"

echo ""
echo "=== Artifacts Downloaded ==="
echo ""

# Find the performance evidence directory
PERF_DIR=$(find "${DOWNLOAD_DIR}" -type d -path "*/gates/performance" | head -1)

if [[ -z "${PERF_DIR}" ]]; then
    echo "ERROR: Performance evidence directory not found"
    echo "Available directories:"
    find "${DOWNLOAD_DIR}" -type d | head -20
    exit 1
fi

echo "Performance evidence directory: ${PERF_DIR}"
echo ""

# 1. METRICS - The actual performance numbers
echo "=== 1. PERFORMANCE METRICS ==="
if [[ -f "${PERF_DIR}/report.json" ]]; then
    echo "File: ${PERF_DIR}/report.json"
    echo ""
    jq '.metrics' "${PERF_DIR}/report.json"
    echo ""
else
    echo "WARNING: report.json not found"
fi

# 2. MARKERS - Execution path verification
echo "=== 2. EXECUTION PATH MARKERS ==="
DEBUGCON_LOG="${PERF_DIR}/../boot-audit/qemu_debugcon.log"
if [[ -f "${DEBUGCON_LOG}" ]]; then
    echo "File: ${DEBUGCON_LOG}"
    echo ""
    echo "Checking for Patch C markers:"
    grep -E "SYSCALL_HANDLER_ENTRY|DISPATCH_TO_HARDENED|HARDENED_ENTRY|PATCH_C" "${DEBUGCON_LOG}" || echo "  No Patch C markers found"
    echo ""
    echo "Checking for Ring3 transition markers:"
    grep -c "P10_RING3_ATTEMPT" "${DEBUGCON_LOG}" || echo "  0"
    grep -c "P10_RING3_COMMIT" "${DEBUGCON_LOG}" || echo "  0"
    grep -c "P10_RING3_ENTER" "${DEBUGCON_LOG}" || echo "  0"
    grep -c "P10_PIC_MASK" "${DEBUGCON_LOG}" || echo "  0"
    echo ""
else
    echo "WARNING: qemu_debugcon.log not found"
fi

# 3. IRET CADENCE - The actual measurement basis
echo "=== 3. IRET CADENCE (Measurement Basis) ==="
PREEMPT_LOG="${PERF_DIR}/preempt.analysis.log"
if [[ -f "${PREEMPT_LOG}" ]]; then
    echo "File: ${PREEMPT_LOG}"
    echo ""
    grep "MARK:IRET" "${PREEMPT_LOG}" | head -5
    echo ""
    echo "Total IRET count:"
    grep -c "MARK:IRET" "${PREEMPT_LOG}" || echo "  0"
    echo ""
else
    echo "WARNING: preempt.analysis.log not found"
fi

# 4. FULL REPORT - All details
echo "=== 4. FULL PERFORMANCE REPORT ==="
if [[ -f "${PERF_DIR}/report.json" ]]; then
    echo "File: ${PERF_DIR}/report.json"
    echo ""
    jq '.' "${PERF_DIR}/report.json"
    echo ""
fi

echo "=== DONE ==="
echo ""
echo "Artifact location: ${DOWNLOAD_DIR}"
echo ""
echo "To inspect manually:"
echo "  cd ${DOWNLOAD_DIR}"
echo "  find . -name '*.json' -o -name '*.log'"

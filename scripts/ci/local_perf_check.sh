#!/bin/bash
# Local performance check wrapper
# Uses local baseline with waiver policy for development

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

cd "${REPO_ROOT}"

# Check if local baseline exists
BASELINE_FILE="scripts/ci/perf-baseline.local.lock.json"
if [ ! -f "${BASELINE_FILE}" ]; then
    echo "ERROR: Local baseline not found: ${BASELINE_FILE}"
    echo "Run: ./scripts/ci/local_perf_baseline_init.sh"
    exit 1
fi

# Local authority configuration
export PERF_BASELINE_AUTHORITY="local-dev-$(uname -s)-$(uname -m)"
export PERF_CI_IMAGE_DIGEST="local-$(hostname)-$(uname -r)"
export PERF_REQUIRE_CI_FOR_BASELINE_INIT="0"
export PERF_QEMU_TIMEOUT="30"
export PERF_KERNEL_PROFILE="validation"
export PERF_ENV_MISMATCH_POLICY="waiver"  # Allow env changes in local dev
export CI="false"
export RUN_ID="local-check-$(date +%s)"

echo "=== Local Performance Check ==="
echo "Authority: ${PERF_BASELINE_AUTHORITY}"
echo "Baseline: ${BASELINE_FILE}"
echo "Policy: waiver (env changes allowed)"
echo ""

# Run performance gate
make ci-gate-performance PERF_BASELINE_FILE="${BASELINE_FILE}"

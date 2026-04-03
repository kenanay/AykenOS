#!/bin/bash
# Local performance baseline initialization
# This creates a LOCAL authority baseline for development
# NOT for CI/production use

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

cd "${REPO_ROOT}"

# Local authority configuration
export PERF_BASELINE_AUTHORITY="local-dev-$(uname -s)-$(uname -m)"
export PERF_CI_IMAGE_DIGEST="local-$(hostname)-$(uname -r)"
export PERF_INIT_BASELINE="1"
export PERF_REQUIRE_CI_FOR_BASELINE_INIT="0"  # Allow local init
export PERF_QEMU_TIMEOUT="30"
export PERF_KERNEL_PROFILE="validation"
export PERF_ENV_MISMATCH_POLICY="fail"  # Use fail for local too
export PERF_BOOT_THRESHOLD_PERCENT="20"
export PERF_CONTEXT_THRESHOLD_PERCENT="15"
export PERF_SYSCALL_THRESHOLD_PERCENT="15"
export CI="false"
export RUN_ID="local-dev-$(date +%s)"

BASELINE_FILE="scripts/ci/perf-baseline.local.lock.json"

echo "=== Local Performance Baseline Init ==="
echo "Authority: ${PERF_BASELINE_AUTHORITY}"
echo "Digest: ${PERF_CI_IMAGE_DIGEST}"
echo "Baseline file: ${BASELINE_FILE}"
echo ""

# Run baseline init
set +e
make ci-gate-performance PERF_BASELINE_FILE="${BASELINE_FILE}"
rc=$?
set -e

echo ""
echo "=== Exit Code: ${rc} ==="

if [ "${rc}" -eq 2 ]; then
    echo "✓ Baseline initialized (fail-closed)"
    if [ -f "${BASELINE_FILE}" ]; then
        echo "✓ Baseline file created: ${BASELINE_FILE}"
        echo ""
        echo "=== Baseline Summary ==="
        jq -r '.metrics' "${BASELINE_FILE}" 2>/dev/null || cat "${BASELINE_FILE}"
    fi
elif [ "${rc}" -eq 0 ]; then
    echo "✗ Unexpected success - baseline should not exist yet"
    exit 1
else
    echo "✗ Unexpected exit code: ${rc}"
    exit "${rc}"
fi

echo ""
echo "=== Next Steps ==="
echo "1. Review baseline: cat ${BASELINE_FILE}"
echo "2. Test comparison: make ci-gate-performance-local"
echo "3. This baseline is LOCAL only - do not commit to repo"
echo ""
echo "For CI baseline, use GitHub Actions workflow after billing is resolved."

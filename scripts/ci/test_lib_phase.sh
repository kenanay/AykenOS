#!/usr/bin/env bash
# Unit tests for phase detection library
# Run: ./scripts/ci/test_lib_phase.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "${ROOT}/scripts/ci/lib-phase.sh"

PHASE_FILE="${ROOT}/docs/roadmap/CURRENT_PHASE"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/ayken-phase-test.XXXXXX")"
PHASE_BAK="${TMP_DIR}/CURRENT_PHASE.bak"
PHASE_EXISTED=0

cleanup() {
    if [[ "${PHASE_EXISTED}" -eq 1 ]]; then
        cp -f "${PHASE_BAK}" "${PHASE_FILE}"
    else
        rm -f "${PHASE_FILE}"
    fi
    rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

if [[ -f "${PHASE_FILE}" ]]; then
    PHASE_EXISTED=1
    cp -f "${PHASE_FILE}" "${PHASE_BAK}"
fi

fail() {
    echo "[FAIL] $1" >&2
    exit 1
}

assert_eq() {
    local got="$1"
    local want="$2"
    local msg="$3"
    if [[ "${got}" != "${want}" ]]; then
        fail "${msg} (got='${got}', want='${want}')"
    fi
}

echo "=== lib-phase unit tests ==="

echo "Test 1: parse valid phase file"
echo "CURRENT_PHASE=8" > "${PHASE_FILE}"
phase="$(get_current_phase)"
assert_eq "${phase}" "8" "valid phase parse failed"
echo "[PASS] valid phase parse"

echo "Test 2: missing phase file should fail with rc=3"
rm -f "${PHASE_FILE}"
set +e
out_missing="$(get_current_phase 2>&1)"
rc_missing=$?
set -e
assert_eq "${rc_missing}" "3" "missing phase file rc mismatch"
if [[ "${out_missing}" != *"Phase file not found"* ]]; then
    fail "missing phase file error text mismatch"
fi
echo "[PASS] missing phase file handling"

echo "Test 3: invalid format should fail with rc=3"
echo "CURRENT_PHASE=eight" > "${PHASE_FILE}"
set +e
out_invalid="$(get_current_phase 2>&1)"
rc_invalid=$?
set -e
assert_eq "${rc_invalid}" "3" "invalid phase format rc mismatch"
if [[ "${out_invalid}" != *"Could not parse phase number"* ]]; then
    fail "invalid phase format error text mismatch"
fi
echo "[PASS] invalid format handling"

echo "=== all lib-phase tests passed ==="

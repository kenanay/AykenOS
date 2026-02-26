#!/usr/bin/env bash
# Unit tests for drift allowlist library
# Run: ./scripts/ci/test_drift_allowlist.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "${ROOT}/scripts/ci/lib-drift-allowlist.sh"

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/ayken-drift-allowlist-test.XXXXXX")"
ALLOWLIST_FILE="${TMP_DIR}/allowlist.json"

cleanup() {
    rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

fail() {
    echo "[FAIL] $1" >&2
    exit 1
}

assert_rc() {
    local got="$1"
    local want="$2"
    local msg="$3"
    if [[ "${got}" != "${want}" ]]; then
        fail "${msg} (got=${got}, want=${want})"
    fi
}

write_allowlist() {
    local body="$1"
    printf "%s\n" "${body}" > "${ALLOWLIST_FILE}"
}

echo "=== drift allowlist unit tests ==="

echo "Test 1: valid schema"
write_allowlist '{"version":"1.0","metrics":["syscall_latency_ms_proxy","boot_time_ms"]}'
validate_drift_allowlist "${ALLOWLIST_FILE}"
echo "[PASS] valid schema"

echo "Test 2: missing file should fail"
set +e
validate_drift_allowlist "${TMP_DIR}/missing.json" >/dev/null 2>&1
rc_missing=$?
set -e
assert_rc "${rc_missing}" "3" "missing file rc mismatch"
echo "[PASS] missing file validation"

echo "Test 3: invalid schema should fail"
write_allowlist '{"version":"1.0","metrics":"not-array"}'
set +e
validate_drift_allowlist "${ALLOWLIST_FILE}" >/dev/null 2>&1
rc_invalid=$?
set -e
assert_rc "${rc_invalid}" "3" "invalid schema rc mismatch"
echo "[PASS] invalid schema validation"

echo "Test 4: allowlisted metric detection"
write_allowlist '{"version":"1.0","metrics":["syscall_latency_ms_proxy"]}'
if is_metric_allowlisted "syscall_latency_ms_proxy" "${ALLOWLIST_FILE}"; then
    echo "[PASS] allowlisted metric detected"
else
    fail "allowlisted metric not detected"
fi

echo "Test 5: non-allowlisted metric detection"
if is_metric_allowlisted "boot_time_ms" "${ALLOWLIST_FILE}"; then
    fail "non-allowlisted metric should not match"
else
    echo "[PASS] non-allowlisted metric rejected"
fi

echo "=== all drift allowlist tests passed ==="

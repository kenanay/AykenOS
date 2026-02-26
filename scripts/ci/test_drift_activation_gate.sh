#!/usr/bin/env bash
# Unit/integration tests for drift activation gate
# Run: ./scripts/ci/test_drift_activation_gate.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GATE_SCRIPT="${ROOT}/scripts/ci/gate_drift_activation.sh"
PHASE_FILE="${ROOT}/docs/roadmap/CURRENT_PHASE"
ACTIVATION_FILE="${ROOT}/constitution/drift_blocking_activation.md"

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/ayken-drift-activation-test.XXXXXX")"
PHASE_BAK="${TMP_DIR}/CURRENT_PHASE.bak"
ACT_BAK="${TMP_DIR}/drift_blocking_activation.md.bak"
PHASE_EXISTED=0
ACT_EXISTED=0
LAST_DIR=""
LAST_RC=0

cleanup() {
    if [[ "${PHASE_EXISTED}" -eq 1 ]]; then
        cp -f "${PHASE_BAK}" "${PHASE_FILE}"
    else
        rm -f "${PHASE_FILE}"
    fi

    if [[ "${ACT_EXISTED}" -eq 1 ]]; then
        cp -f "${ACT_BAK}" "${ACTIVATION_FILE}"
    else
        rm -f "${ACTIVATION_FILE}"
    fi

    rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

if [[ -f "${PHASE_FILE}" ]]; then
    PHASE_EXISTED=1
    cp -f "${PHASE_FILE}" "${PHASE_BAK}"
fi
if [[ -f "${ACTIVATION_FILE}" ]]; then
    ACT_EXISTED=1
    cp -f "${ACTIVATION_FILE}" "${ACT_BAK}"
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

assert_contains() {
    local haystack="$1"
    local needle="$2"
    local msg="$3"
    if [[ "${haystack}" != *"${needle}"* ]]; then
        fail "${msg} (missing '${needle}')"
    fi
}

write_phase() {
    local phase="$1"
    echo "CURRENT_PHASE=${phase}" > "${PHASE_FILE}"
}

write_activation() {
    local enabled_line="$1"
    local phase_min_line="$2"
    cat > "${ACTIVATION_FILE}" <<EOF
---
enabled: ${enabled_line}
phase_minimum: ${phase_min_line}
auto_activation_policy: phase_guard
n_run_threshold: 3
---
EOF
}

run_gate() {
    local case_name="$1"
    LAST_DIR="${TMP_DIR}/${case_name}"
    mkdir -p "${LAST_DIR}"

    set +e
    "${GATE_SCRIPT}" --evidence-dir "${LAST_DIR}" > "${LAST_DIR}/stdout.txt" 2> "${LAST_DIR}/stderr.txt"
    LAST_RC=$?
    set -e
}

echo "=== drift-activation gate tests ==="

echo "Test 1 (9.1): phase < 9 => SKIP"
write_phase "8"
write_activation "false" "9"
run_gate "phase_lt_9_skip"
assert_eq "${LAST_RC}" "0" "phase < 9 should not fail"
verdict="$(jq -r '.verdict' "${LAST_DIR}/report.json")"
reason="$(jq -r '.reason' "${LAST_DIR}/report.json")"
assert_eq "${verdict}" "SKIP" "phase < 9 verdict mismatch"
assert_eq "${reason}" "phase_below_minimum" "phase < 9 reason mismatch"
echo "[PASS] phase < 9 skip"

echo "Test 2 (9.2): phase 9 + enabled=false => FAIL"
write_phase "9"
write_activation "false" "9"
run_gate "phase_9_disabled_fail"
assert_eq "${LAST_RC}" "2" "phase 9 disabled must fail"
verdict="$(jq -r '.verdict' "${LAST_DIR}/report.json")"
reason="$(jq -r '.reason' "${LAST_DIR}/report.json")"
assert_eq "${verdict}" "FAIL" "phase 9 disabled verdict mismatch"
assert_eq "${reason}" "drift_blocking_required_but_disabled" "phase 9 disabled reason mismatch"
violations_count="$(jq '.violations | length' "${LAST_DIR}/report.json")"
assert_eq "${violations_count}" "1" "phase 9 disabled should record 1 violation"
echo "[PASS] phase 9 disabled fail"

echo "Test 3 (9.3): phase 9 + enabled=true => PASS"
write_phase "9"
write_activation "true" "9"
run_gate "phase_9_enabled_pass"
assert_eq "${LAST_RC}" "0" "phase 9 enabled should pass"
verdict="$(jq -r '.verdict' "${LAST_DIR}/report.json")"
reason="$(jq -r '.reason' "${LAST_DIR}/report.json")"
assert_eq "${verdict}" "PASS" "phase 9 enabled verdict mismatch"
assert_eq "${reason}" "drift_blocking_enabled" "phase 9 enabled reason mismatch"
echo "[PASS] phase 9 enabled pass"

echo "Test 4 (9.4): missing phase file => error"
rm -f "${PHASE_FILE}"
write_activation "true" "9"
run_gate "missing_phase_error"
assert_eq "${LAST_RC}" "3" "missing phase file should error"
stderr_text="$(cat "${LAST_DIR}/stderr.txt")"
assert_contains "${stderr_text}" "Failed to detect current phase" "missing phase error text mismatch"
echo "[PASS] missing phase error"

echo "Test 5 (9.5): missing activation file => error"
write_phase "9"
rm -f "${ACTIVATION_FILE}"
run_gate "missing_activation_error"
assert_eq "${LAST_RC}" "3" "missing activation file should error"
stderr_text="$(cat "${LAST_DIR}/stderr.txt")"
assert_contains "${stderr_text}" "Activation file not found" "missing activation error text mismatch"
echo "[PASS] missing activation error"

echo "Test 6 (9.6): invalid phase number => error"
echo "CURRENT_PHASE=phase9" > "${PHASE_FILE}"
write_activation "true" "9"
run_gate "invalid_phase_error"
assert_eq "${LAST_RC}" "3" "invalid phase format should error"
stderr_text="$(cat "${LAST_DIR}/stderr.txt")"
assert_contains "${stderr_text}" "Failed to detect current phase" "invalid phase error text mismatch"
echo "[PASS] invalid phase format error"

echo "Test 7 (9.7): invalid activation state => defaults applied"
write_phase "9"
cat > "${ACTIVATION_FILE}" <<'EOF'
---
enabled: maybe
phase_minimum: nope
auto_activation_policy: phase_guard
---
EOF
run_gate "invalid_activation_defaults"
assert_eq "${LAST_RC}" "2" "invalid activation should default to fail at phase 9"
enabled_value="$(jq -r '.enabled' "${LAST_DIR}/report.json")"
phase_min_value="$(jq -r '.phase_minimum' "${LAST_DIR}/report.json")"
assert_eq "${enabled_value}" "false" "invalid enabled should default false"
assert_eq "${phase_min_value}" "9" "invalid phase_minimum should default 9"
echo "[PASS] invalid activation defaults"

echo "Test 8 (6.5): evidence fields present"
write_phase "9"
write_activation "true" "9"
run_gate "evidence_fields"
assert_eq "${LAST_RC}" "0" "evidence test run should pass"
[[ -f "${LAST_DIR}/report.json" ]] || fail "report.json missing"
[[ -f "${LAST_DIR}/meta.txt" ]] || fail "meta.txt missing"
[[ -f "${LAST_DIR}/violations.txt" ]] || fail "violations.txt missing"
meta_text="$(cat "${LAST_DIR}/meta.txt")"
assert_contains "${meta_text}" "time_utc=" "meta missing time_utc"
assert_contains "${meta_text}" "git_sha=" "meta missing git_sha"
assert_contains "${meta_text}" "current_phase=9" "meta missing current_phase"
echo "[PASS] evidence generation"

echo "=== all drift-activation gate tests passed ==="

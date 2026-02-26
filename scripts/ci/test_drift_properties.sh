#!/usr/bin/env bash
# Property-based tests for drift activation/persistence
# Run: ./scripts/ci/test_drift_properties.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GATE_SCRIPT="${ROOT}/scripts/ci/gate_drift_activation.sh"
source "${ROOT}/scripts/ci/lib-drift-persistence.sh"

PHASE_FILE="${ROOT}/docs/roadmap/CURRENT_PHASE"
ACTIVATION_FILE="${ROOT}/constitution/drift_blocking_activation.md"

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/ayken-drift-properties.XXXXXX")"
PHASE_BAK="${TMP_DIR}/CURRENT_PHASE.bak"
ACT_BAK="${TMP_DIR}/drift_blocking_activation.md.bak"
PHASE_EXISTED=0
ACT_EXISTED=0

fail() {
    echo "[FAIL] $1" >&2
    exit 1
}

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

write_phase() {
    local phase="$1"
    echo "CURRENT_PHASE=${phase}" > "${PHASE_FILE}"
}

write_activation() {
    local enabled="$1"
    local phase_min="$2"
    cat > "${ACTIVATION_FILE}" <<EOF
---
enabled: ${enabled}
phase_minimum: ${phase_min}
auto_activation_policy: phase_guard
n_run_threshold: 3
---
EOF
}

run_gate_report_verdict() {
    local run_name="$1"
    local out_dir="${TMP_DIR}/${run_name}"
    mkdir -p "${out_dir}"
    set +e
    "${GATE_SCRIPT}" --evidence-dir "${out_dir}" >/dev/null 2>&1
    rc=$?
    set -e
    verdict="$(jq -r '.verdict' "${out_dir}/report.json")"
    echo "${rc}:${verdict}:${out_dir}"
}

echo "=== drift property tests ==="

echo "Property 12.1: phase-driven enforcement"
for phase in 0 1 4 8 9 10 12; do
    for enabled in true false; do
        for phase_min in 8 9 10; do
            write_phase "${phase}"
            write_activation "${enabled}" "${phase_min}"
            result="$(run_gate_report_verdict "prop_phase_${phase}_${enabled}_${phase_min}")"
            rc="${result%%:*}"
            rest="${result#*:}"
            verdict="${rest%%:*}"

            expected_verdict="PASS"
            expected_rc="0"
            if [[ "${phase}" -lt "${phase_min}" ]]; then
                expected_verdict="SKIP"
                expected_rc="0"
            elif [[ "${enabled}" == "false" ]]; then
                expected_verdict="FAIL"
                expected_rc="2"
            fi

            if [[ "${verdict}" != "${expected_verdict}" || "${rc}" != "${expected_rc}" ]]; then
                fail "phase-driven property mismatch phase=${phase} enabled=${enabled} min=${phase_min} got=${rc}/${verdict} want=${expected_rc}/${expected_verdict}"
            fi
        done
    done
done
echo "[PASS] phase-driven enforcement property"

echo "Property 12.2: explicit activation required at/after minimum phase"
for phase in 9 10 11 12; do
    write_phase "${phase}"
    write_activation "false" "9"
    result="$(run_gate_report_verdict "prop_explicit_${phase}")"
    rc="${result%%:*}"
    rest="${result#*:}"
    verdict="${rest%%:*}"
    if [[ "${rc}" != "2" || "${verdict}" != "FAIL" ]]; then
        fail "explicit activation property mismatch phase=${phase} got=${rc}/${verdict}"
    fi
done
echo "[PASS] explicit activation property"

echo "Property 12.3: evidence immutability (read-only operations keep checksum stable)"
write_phase "9"
write_activation "true" "9"
result="$(run_gate_report_verdict "prop_evidence_immutable")"
out_dir="${result##*:}"
before_report="$(sha256sum "${out_dir}/report.json" | awk '{print $1}')"
before_meta="$(sha256sum "${out_dir}/meta.txt" | awk '{print $1}')"
before_violations="$(sha256sum "${out_dir}/violations.txt" | awk '{print $1}')"
cat "${out_dir}/report.json" >/dev/null
cat "${out_dir}/meta.txt" >/dev/null
cat "${out_dir}/violations.txt" >/dev/null
after_report="$(sha256sum "${out_dir}/report.json" | awk '{print $1}')"
after_meta="$(sha256sum "${out_dir}/meta.txt" | awk '{print $1}')"
after_violations="$(sha256sum "${out_dir}/violations.txt" | awk '{print $1}')"
if [[ "${before_report}" != "${after_report}" || "${before_meta}" != "${after_meta}" || "${before_violations}" != "${after_violations}" ]]; then
    fail "evidence immutability property failed"
fi
echo "[PASS] evidence immutability property"

echo "Property 12.4: N-run persistence counter monotonicity"
DRIFT_STATE_FILE="${TMP_DIR}/drift_state.json"
rm -f "${DRIFT_STATE_FILE}"
unset PERF_AUTHORITY_SALT || true
for n in 1 2 3 4 5; do
    count="$(increment_counter "metric_prop")"
    if [[ "${count}" != "${n}" ]]; then
        fail "N-run monotonicity failed at step ${n} (got ${count})"
    fi
done
if ! check_drift_threshold "metric_prop" 5; then
    fail "N-run threshold should be reached at 5"
fi
echo "[PASS] N-run persistence property"

echo "Property 12.5: authority hash change resets counters"
DRIFT_STATE_FILE="${TMP_DIR}/drift_state_reset.json"
rm -f "${DRIFT_STATE_FILE}"
PERF_AUTHORITY_SALT="salt-a" increment_counter "metric_reset" >/dev/null
PERF_AUTHORITY_SALT="salt-a" increment_counter "metric_reset" >/dev/null
count_before="$(PERF_AUTHORITY_SALT="salt-a" get_counter "metric_reset")"
count_after="$(PERF_AUTHORITY_SALT="salt-b" increment_counter "metric_reset")"
if [[ "${count_before}" != "2" || "${count_after}" != "1" ]]; then
    fail "authority reset property failed (before=${count_before}, after=${count_after})"
fi
echo "[PASS] authority reset property"

echo "=== all drift property tests passed ==="

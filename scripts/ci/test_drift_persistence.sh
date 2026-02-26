#!/usr/bin/env bash
# Unit tests for drift persistence library
# Run: ./scripts/ci/test_drift_persistence.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "${ROOT}/scripts/ci/lib-drift-persistence.sh"

# Test state directory
TEST_STATE_DIR="${ROOT}/.ci-state-test"
DRIFT_STATE_FILE="${TEST_STATE_DIR}/drift_state.json"

# Cleanup
cleanup() {
    rm -rf "${TEST_STATE_DIR}"
}

trap cleanup EXIT

# Test 1: compute_authority_hash is deterministic
test_authority_hash_deterministic() {
    echo "Test 1: Authority hash deterministic"
    
    local hash1
    local hash2
    
    hash1="$(compute_authority_hash)"
    hash2="$(compute_authority_hash)"
    
    if [[ "${hash1}" == "${hash2}" ]]; then
        echo "  ✅ PASS: Hash deterministic (${hash1})"
    else
        echo "  ❌ FAIL: Hash not deterministic"
        exit 1
    fi
}

# Test 2: load_state returns empty state if file missing
test_load_empty_state() {
    echo "Test 2: Load empty state"
    
    cleanup
    mkdir -p "${TEST_STATE_DIR}"
    
    local state
    state="$(load_state)"
    
    local authority_hash
    authority_hash="$(echo "${state}" | jq -r '.authority_hash')"
    
    if [[ "${authority_hash}" == "" ]]; then
        echo "  ✅ PASS: Empty state loaded"
    else
        echo "  ❌ FAIL: State not empty"
        exit 1
    fi
}

# Test 3: save_state and load_state roundtrip
test_save_load_roundtrip() {
    echo "Test 3: Save/load roundtrip"
    
    cleanup
    mkdir -p "${TEST_STATE_DIR}"
    
    local test_state='{"authority_hash":"test123","counters":{"metric1":5}}'
    save_state "${test_state}"
    
    local loaded_state
    loaded_state="$(load_state)"
    
    local counter
    counter="$(echo "${loaded_state}" | jq -r '.counters.metric1')"
    
    if [[ "${counter}" == "5" ]]; then
        echo "  ✅ PASS: Roundtrip successful"
    else
        echo "  ❌ FAIL: Roundtrip failed (got: ${counter})"
        exit 1
    fi
}

# Test 4: increment_counter increments correctly
test_increment_counter() {
    echo "Test 4: Increment counter"
    
    cleanup
    mkdir -p "${TEST_STATE_DIR}"
    
    local count1
    local count2
    local count3
    
    count1="$(increment_counter "test_metric")"
    count2="$(increment_counter "test_metric")"
    count3="$(increment_counter "test_metric")"
    
    if [[ "${count1}" == "1" ]] && [[ "${count2}" == "2" ]] && [[ "${count3}" == "3" ]]; then
        echo "  ✅ PASS: Counter increments (1→2→3)"
    else
        echo "  ❌ FAIL: Counter increment failed (${count1}, ${count2}, ${count3})"
        exit 1
    fi
}

# Test 5: get_counter returns correct value
test_get_counter() {
    echo "Test 5: Get counter"
    
    cleanup
    mkdir -p "${TEST_STATE_DIR}"
    
    increment_counter "metric_a" > /dev/null
    increment_counter "metric_a" > /dev/null
    increment_counter "metric_b" > /dev/null
    
    local count_a
    local count_b
    local count_c
    
    count_a="$(get_counter "metric_a")"
    count_b="$(get_counter "metric_b")"
    count_c="$(get_counter "metric_c")"
    
    if [[ "${count_a}" == "2" ]] && [[ "${count_b}" == "1" ]] && [[ "${count_c}" == "0" ]]; then
        echo "  ✅ PASS: Get counter correct (a=2, b=1, c=0)"
    else
        echo "  ❌ FAIL: Get counter failed (a=${count_a}, b=${count_b}, c=${count_c})"
        exit 1
    fi
}

# Test 6: reset_counters clears all counters
test_reset_counters() {
    echo "Test 6: Reset counters"
    
    cleanup
    mkdir -p "${TEST_STATE_DIR}"
    
    increment_counter "metric1" > /dev/null
    increment_counter "metric2" > /dev/null
    
    reset_counters
    
    local count1
    local count2
    
    count1="$(get_counter "metric1")"
    count2="$(get_counter "metric2")"
    
    if [[ "${count1}" == "0" ]] && [[ "${count2}" == "0" ]]; then
        echo "  ✅ PASS: Counters reset"
    else
        echo "  ❌ FAIL: Counters not reset (${count1}, ${count2})"
        exit 1
    fi
}

# Test 7: authority_changed detects change
test_authority_changed() {
    echo "Test 7: Authority change detection"
    
    cleanup
    mkdir -p "${TEST_STATE_DIR}"
    
    # First run: no state, should detect change
    if authority_changed; then
        echo "  ✅ PASS: Initial authority change detected"
    else
        echo "  ❌ FAIL: Initial authority change not detected"
        exit 1
    fi
    
    # Save current authority
    local current_hash
    current_hash="$(compute_authority_hash)"
    save_state '{"authority_hash":"'${current_hash}'","counters":{}}'
    
    # Second run: same authority, should not detect change
    if ! authority_changed; then
        echo "  ✅ PASS: No authority change detected"
    else
        echo "  ❌ FAIL: False authority change detected"
        exit 1
    fi
}

# Test 8: compatibility aliases behave like primary functions
test_compat_aliases() {
    echo "Test 8: Compatibility aliases"

    cleanup
    mkdir -p "${TEST_STATE_DIR}"

    local state='{"authority_hash":"alias","counters":{"metric_alias":4}}'
    save_drift_state "${state}"

    local loaded
    loaded="$(load_drift_state)"
    local before
    before="$(echo "${loaded}" | jq -r '.counters.metric_alias')"

    if [[ "${before}" != "4" ]]; then
        echo "  ❌ FAIL: Alias load/save mismatch (${before})"
        exit 1
    fi

    local after
    after="$(increment_drift_counter "metric_alias")"
    if [[ "${after}" =~ ^[0-9]+$ ]] && [[ "${after}" -ge 1 ]]; then
        echo "  ✅ PASS: Alias increment works (counter=${after})"
    else
        echo "  ❌ FAIL: Alias increment failed (${after})"
        exit 1
    fi
}

# Test 9: threshold check logic
test_check_drift_threshold() {
    echo "Test 9: Threshold check"

    cleanup
    mkdir -p "${TEST_STATE_DIR}"

    increment_counter "metric_t" > /dev/null
    increment_counter "metric_t" > /dev/null

    if check_drift_threshold "metric_t" 3; then
        echo "  ❌ FAIL: Threshold should not be reached at 2/3"
        exit 1
    fi

    increment_counter "metric_t" > /dev/null

    if check_drift_threshold "metric_t" 3; then
        echo "  ✅ PASS: Threshold reached at 3/3"
    else
        echo "  ❌ FAIL: Threshold should be reached at 3/3"
        exit 1
    fi

    if check_drift_threshold "metric_t" 0; then
        echo "  ❌ FAIL: Invalid threshold should not return success"
        exit 1
    else
        local rc=$?
        if [[ "${rc}" == "3" ]]; then
            echo "  ✅ PASS: Invalid threshold rejected with rc=3"
        else
            echo "  ❌ FAIL: Invalid threshold rc mismatch (${rc})"
            exit 1
        fi
    fi
}

# Run all tests
echo "=== Drift Persistence Library Unit Tests ==="
echo ""

test_authority_hash_deterministic
test_load_empty_state
test_save_load_roundtrip
test_increment_counter
test_get_counter
test_reset_counters
test_authority_changed
test_compat_aliases
test_check_drift_threshold

echo ""
echo "=== All Tests Passed ==="

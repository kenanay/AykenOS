#!/usr/bin/env bash
# Drift Persistence Library (Model A - Minimal)
# Authority: ARCHITECTURE_FREEZE.md
# Responsibility: State management for N-run drift persistence
# Does NOT enforce blocking (handled by performance gate)

# State file location (gitignored, CI artifact only)
DRIFT_STATE_FILE="${ROOT}/.ci-state/drift_state.json"

# Compute authority hash (deterministic, no network)
# Authority = git SHA + toolchain version + QEMU version
compute_authority_hash() {
    local git_sha
    local clang_ver
    local qemu_ver
    
    git_sha="$(git -C "${ROOT}" rev-parse HEAD 2>/dev/null || echo "NO_GIT")"
    clang_ver="$(clang --version 2>/dev/null | head -1 || echo "NO_CLANG")"
    qemu_ver="$(qemu-system-x86_64 --version 2>/dev/null | head -1 || echo "NO_QEMU")"
    
    echo -n "${git_sha}:${clang_ver}:${qemu_ver}" | sha256sum | cut -d' ' -f1
}

# Load drift state from file
# Returns empty state if file missing or authority mismatch
load_state() {
    if [[ ! -f "${DRIFT_STATE_FILE}" ]]; then
        echo '{"authority_hash":"","counters":{}}'
        return
    fi
    
    cat "${DRIFT_STATE_FILE}"
}

# Save drift state to file
save_state() {
    local state="$1"
    
    mkdir -p "$(dirname "${DRIFT_STATE_FILE}")"
    echo "${state}" > "${DRIFT_STATE_FILE}"
}

# Increment drift counter for a metric
# Resets all counters if authority hash changed
# Returns new counter value
increment_counter() {
    local metric="$1"
    local state
    local current_hash
    local stored_hash
    
    state="$(load_state)"
    current_hash="$(compute_authority_hash)"
    stored_hash="$(echo "${state}" | jq -r '.authority_hash // ""')"
    
    # Authority changed → reset all counters
    if [[ "${stored_hash}" != "${current_hash}" ]]; then
        state='{"authority_hash":"'${current_hash}'","counters":{}}'
    fi
    
    # Increment counter
    state="$(echo "${state}" | jq --arg m "${metric}" \
        '.counters[$m] = (.counters[$m] // 0) + 1')"
    
    save_state "${state}"
    
    # Return counter value
    echo "${state}" | jq -r --arg m "${metric}" '.counters[$m]'
}

# Get current counter value for a metric
get_counter() {
    local metric="$1"
    local state
    
    state="$(load_state)"
    echo "${state}" | jq -r --arg m "${metric}" '.counters[$m] // 0'
}

# Reset all counters (authority change or manual reset)
reset_counters() {
    local current_hash
    
    current_hash="$(compute_authority_hash)"
    save_state '{"authority_hash":"'${current_hash}'","counters":{}}'
}

# Check if authority hash changed since last run
authority_changed() {
    local state
    local current_hash
    local stored_hash
    
    state="$(load_state)"
    current_hash="$(compute_authority_hash)"
    stored_hash="$(echo "${state}" | jq -r '.authority_hash // ""')"
    
    if [[ -z "${stored_hash}" ]] || [[ "${stored_hash}" != "${current_hash}" ]]; then
        return 0  # Changed
    else
        return 1  # Not changed
    fi
}

#!/usr/bin/env bash
# Phase detection library for AykenOS CI
# Authority: ARCHITECTURE_FREEZE.md

get_current_phase() {
    local phase_file="${ROOT}/docs/roadmap/CURRENT_PHASE"
    
    if [[ ! -f "${phase_file}" ]]; then
        echo "ERROR: Phase file not found: ${phase_file}" >&2
        return 3
    fi
    
    # Extract phase number from simple format
    # Expected format: "CURRENT_PHASE=8"
    local phase=$(grep -E "^CURRENT_PHASE=[0-9]+$" "${phase_file}" | \
                  cut -d'=' -f2)
    
    if [[ -z "${phase}" ]]; then
        echo "ERROR: Could not parse phase number from ${phase_file}" >&2
        return 3
    fi
    
    echo "${phase}"
}

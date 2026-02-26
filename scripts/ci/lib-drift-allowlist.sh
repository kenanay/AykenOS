#!/usr/bin/env bash
# Drift allowlist helper library
# Authority: ARCHITECTURE_FREEZE.md

validate_drift_allowlist() {
    local allowlist_file="$1"

    if [[ ! -f "${allowlist_file}" ]]; then
        echo "ERROR: drift allowlist file not found: ${allowlist_file}" >&2
        return 3
    fi

    if ! jq -e '
      type == "object" and
      .version == "1.0" and
      (.metrics | type == "array") and
      all(.metrics[]; type == "string" and length > 0)
    ' "${allowlist_file}" >/dev/null 2>&1; then
        echo "ERROR: invalid drift allowlist schema in ${allowlist_file}" >&2
        return 3
    fi

    return 0
}

is_metric_allowlisted() {
    local metric="$1"
    local allowlist_file="$2"

    jq -e --arg metric "${metric}" '.metrics | index($metric) != null' \
        "${allowlist_file}" >/dev/null 2>&1
}

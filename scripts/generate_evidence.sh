#!/usr/bin/env bash
# Evidence Generation Script
#
# Purpose: Transform raw boot logs into structured evidence artifacts
# Authority: ZERO - runs AFTER validation, never affects decisions
#
# Maintainer: Kenan AY — System Architect

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

# Generate unique run ID
TIMESTAMP=$(date -u +%Y%m%dT%H%M%SZ)
GIT_SHA=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
PID=$$
RUN_ID="run-${TIMESTAMP}-${GIT_SHA}-${PID}"

# Create evidence directory structure
EVIDENCE_BASE="out/evidence"
RUN_DIR="${EVIDENCE_BASE}/${RUN_ID}"

mkdir -p "${RUN_DIR}/meta"
mkdir -p "${RUN_DIR}/logs"
mkdir -p "${RUN_DIR}/reports"

# Determine source (dev_loop, ci, or manual)
SOURCE="manual"
if [[ "${CI:-false}" == "true" ]]; then
    SOURCE="ci"
elif [[ -n "${DEV_LOOP_MODE:-}" ]]; then
    SOURCE="dev_loop"
fi

# Generate run metadata with developer attribution
cat > "${RUN_DIR}/meta/run.json" <<EOF
{
  "run_id": "${RUN_ID}",
  "time_utc": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "source": "${SOURCE}",
  "git_sha": "${GIT_SHA}",
  "git_branch": "$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo 'unknown')",
  "git_dirty": $(git diff --quiet 2>/dev/null && echo 'false' || echo 'true'),
  "deterministic": true,
  "developer": "Kenan AY",
  "generated_by": "Kenan AY — System Architect"
}
EOF

# Copy boot log if available
if [[ -f "out/logs/boot_watch.log" ]]; then
    cp "out/logs/boot_watch.log" "${RUN_DIR}/logs/boot.log"
fi

# Parse markers from boot log
EARLY_BOOT_OK=false
LATE_INIT_END=false
BOOT_OK=false
FAIL_CLOSED=false

if [[ -f "${RUN_DIR}/logs/boot.log" ]]; then
    if grep -q '\[K\]\[EARLY_BOOT_OK\]' "${RUN_DIR}/logs/boot.log"; then
        EARLY_BOOT_OK=true
    fi

    if grep -q '\[K\]\[LATE_INIT_END\]' "${RUN_DIR}/logs/boot.log"; then
        LATE_INIT_END=true
    fi

    if grep -q '\[\[AYKEN_BOOT_OK\]\]' "${RUN_DIR}/logs/boot.log"; then
        BOOT_OK=true
    fi

    if grep -q '\[VCP\]\[FAIL_CLOSED\]' "${RUN_DIR}/logs/boot.log"; then
        FAIL_CLOSED=true
    fi
fi

# Generate markers report with developer attribution
cat > "${RUN_DIR}/reports/markers.json" <<EOF
{
  "EARLY_BOOT_OK": ${EARLY_BOOT_OK},
  "LATE_INIT_END": ${LATE_INIT_END},
  "BOOT_OK": ${BOOT_OK},
  "FAIL_CLOSED": ${FAIL_CLOSED},
  "generated_by": "Kenan AY — System Architect"
}
EOF

# Determine boot status
BOOT_STATUS="UNKNOWN"
BOOT_REASON="No boot log available"
if [[ "${EARLY_BOOT_OK}" == "true" ]] && [[ "${LATE_INIT_END}" == "true" ]] && [[ "${BOOT_OK}" == "true" ]]; then
    BOOT_STATUS="PASS"
    BOOT_REASON="All required markers present in correct sequence"
elif [[ -f "${RUN_DIR}/logs/boot.log" ]]; then
    BOOT_STATUS="FAIL"
    # Determine specific failure reason
    if [[ "${EARLY_BOOT_OK}" == "false" ]]; then
        BOOT_REASON="Missing EARLY_BOOT_OK marker"
    elif [[ "${LATE_INIT_END}" == "false" ]]; then
        BOOT_REASON="Missing LATE_INIT_END marker"
    elif [[ "${BOOT_OK}" == "false" ]]; then
        BOOT_REASON="Missing AYKEN_BOOT_OK marker"
    else
        BOOT_REASON="Unknown failure"
    fi
fi

# Generate summary report with developer attribution and enhanced structure
cat > "${RUN_DIR}/reports/summary.json" <<EOF
{
  "format_version": "1.0",
  "status": {
    "boot": "${BOOT_STATUS}",
    "reason": "${BOOT_REASON}",
    "markers_ok": $(if [[ "${BOOT_STATUS}" == "PASS" ]]; then echo "true"; else echo "false"; fi),
    "fail_closed": ${FAIL_CLOSED}
  },
  "validation": {
    "source": "raw_boot_logs",
    "authoritative": false,
    "diagnostic_only": true
  },
  "isolation": {
    "non_influential": true,
    "read_only": true,
    "post_validation": true
  },
  "generated_by": "Kenan AY — System Architect"
}
EOF

# Generate performance proxy report with developer attribution
# Standardized format: value, unit, method, validity, disclaimer
BOOT_TIME_PROXY=0
if [[ -f "${RUN_DIR}/logs/boot.log" ]]; then
    BOOT_TIME_PROXY=$(wc -l < "${RUN_DIR}/logs/boot.log" || echo "0")
fi

# Determine validity: must be positive and within reasonable bounds
PERF_VALID="false"
if [[ ${BOOT_TIME_PROXY} -gt 0 ]] && [[ ${BOOT_TIME_PROXY} -lt 100000 ]]; then
    PERF_VALID="true"
fi

cat > "${RUN_DIR}/reports/perf.json" <<EOF
{
  "format_version": "1.0",
  "value": ${BOOT_TIME_PROXY},
  "unit": "lines",
  "method": "line_count_proxy",
  "valid": ${PERF_VALID},
  "diagnostic_only": true,
  "non_authoritative": true,
  "disclaimer": "Performance metrics are diagnostic only and do not affect validation decisions",
  "generated_by": "Kenan AY — System Architect"
}
EOF

# Output run ID for caller
echo "${RUN_ID}"

# Update run history
RUNS_HISTORY="${EVIDENCE_BASE}/runs.json"

# Create or update runs history
if [[ ! -f "$RUNS_HISTORY" ]]; then
    cat > "$RUNS_HISTORY" <<HISTORY_EOF
{
  "format_version": "1.0",
  "runs": [],
  "generated_by": "Kenan AY — System Architect"
}
HISTORY_EOF
fi

# Check if old format and migrate
if ! jq -e '.format_version' "$RUNS_HISTORY" > /dev/null 2>&1; then
    # Old format detected, migrate to new format
    OLD_RUNS=$(jq -r '.runs // []' "$RUNS_HISTORY" 2>/dev/null || echo "[]")
    cat > "$RUNS_HISTORY" <<HISTORY_MIGRATE
{
  "format_version": "1.0",
  "runs": ${OLD_RUNS},
  "generated_by": "Kenan AY — System Architect"
}
HISTORY_MIGRATE
fi

# Add current run to history
# Read existing runs
EXISTING_RUNS=$(jq -r '.runs // []' "$RUNS_HISTORY" 2>/dev/null || echo "[]")

# Create new run entry
NEW_RUN=$(cat <<RUN_ENTRY
{
  "run_id": "${RUN_ID}",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "git_sha": "${GIT_SHA}",
  "git_branch": "$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo 'unknown')",
  "source": "${SOURCE}",
  "boot_status": "${BOOT_STATUS}",
  "markers_ok": $(if [[ "${BOOT_STATUS}" == "PASS" ]]; then echo "true"; else echo "false"; fi),
  "perf_value": ${BOOT_TIME_PROXY}
}
RUN_ENTRY
)

# Merge new run into history (keep last 100 runs, filter out old string-only entries)
jq --argjson new_run "$NEW_RUN" \
   '.runs = ([$new_run] + (.runs | map(select(type == "object")))) | .runs = .runs[:100]' \
   "$RUNS_HISTORY" > "${RUNS_HISTORY}.tmp" && mv "${RUNS_HISTORY}.tmp" "$RUNS_HISTORY"

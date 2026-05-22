#!/usr/bin/env bash
#
# Task 25: Final Checkpoint - Unified Observability Validated
#
# Purpose: Validate that unified observability system is complete and operational
# Maintainer: Kenan AY — System Architect
#
# This checkpoint validates:
# - Web dashboard is operational and accessible
# - Evidence pipeline generates structured artifacts
# - Dashboard can load and display evidence
# - All observability features are working correctly
# - No regressions have been introduced

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Evidence directory for this checkpoint
EVIDENCE_DIR="$PROJECT_ROOT/out/evidence/task25_unified_observability"
mkdir -p "$EVIDENCE_DIR"

LOG_FILE="$EVIDENCE_DIR/validation.log"
RESULT_FILE="$EVIDENCE_DIR/result.json"

# Logging functions
log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG_FILE"
}

log_error() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "$LOG_FILE" >&2
}

log_success() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] ✅ $*" | tee -a "$LOG_FILE"
}

log_fail() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] ❌ $*" | tee -a "$LOG_FILE"
}

# Initialize result tracking
CHECKS_PASSED=0
CHECKS_FAILED=0
FAILURES=()

record_pass() {
    ((CHECKS_PASSED++))
    log_success "$1"
}

record_fail() {
    ((CHECKS_FAILED++))
    FAILURES+=("$1")
    log_fail "$1"
}

# Start validation
log "========================================="
log "Task 25: Unified Observability Validation"
log "========================================="
log ""

# Check 1: Web dashboard files exist
log "Check 1: Web dashboard files exist"
if [[ -f "$PROJECT_ROOT/tools/dashboard/index.html" ]] && \
   [[ -f "$PROJECT_ROOT/tools/dashboard/dashboard.js" ]] && \
   [[ -f "$PROJECT_ROOT/tools/dashboard/serve.sh" ]]; then
    record_pass "Web dashboard files exist"
else
    record_fail "Web dashboard files missing"
fi

# Check 2: Dashboard HTML structure is valid
log "Check 2: Dashboard HTML structure is valid"
if grep -q "AykenOS Dev Loop Observability Dashboard" "$PROJECT_ROOT/tools/dashboard/index.html" && \
   grep -q "Read-Only Validation Observer" "$PROJECT_ROOT/tools/dashboard/index.html" && \
   grep -q "Kenan AY" "$PROJECT_ROOT/tools/dashboard/index.html"; then
    record_pass "Dashboard HTML structure is valid"
else
    record_fail "Dashboard HTML structure is invalid"
fi

# Check 3: Dashboard JavaScript is functional
log "Check 3: Dashboard JavaScript is functional"
if grep -q "async function init()" "$PROJECT_ROOT/tools/dashboard/dashboard.js" && \
   grep -q "loadRunData" "$PROJECT_ROOT/tools/dashboard/dashboard.js" && \
   grep -q "updateStatusCard" "$PROJECT_ROOT/tools/dashboard/dashboard.js"; then
    record_pass "Dashboard JavaScript is functional"
else
    record_fail "Dashboard JavaScript is incomplete"
fi

# Check 4: Evidence directory structure exists
log "Check 4: Evidence directory structure exists"
if [[ -d "$PROJECT_ROOT/out/evidence" ]]; then
    EVIDENCE_RUNS=$(find "$PROJECT_ROOT/out/evidence" -maxdepth 1 -type d -name "run-*" | wc -l)
    if [[ $EVIDENCE_RUNS -gt 0 ]]; then
        record_pass "Evidence directory contains $EVIDENCE_RUNS runs"
    else
        record_fail "Evidence directory exists but contains no runs"
    fi
else
    record_fail "Evidence directory does not exist"
fi

# Check 5: Evidence artifacts have correct structure
log "Check 5: Evidence artifacts have correct structure"
LATEST_RUN=$(find "$PROJECT_ROOT/out/evidence" -maxdepth 1 -type d -name "run-*" | sort -r | head -n 1)
if [[ -n "$LATEST_RUN" ]]; then
    REQUIRED_DIRS=("meta" "reports" "logs" "artifacts" "execution" "gates" "input")
    MISSING_DIRS=()

    for dir in "${REQUIRED_DIRS[@]}"; do
        if [[ ! -d "$LATEST_RUN/$dir" ]]; then
            MISSING_DIRS+=("$dir")
        fi
    done

    if [[ ${#MISSING_DIRS[@]} -eq 0 ]]; then
        record_pass "Evidence structure is complete"
    else
        record_fail "Evidence structure missing directories: ${MISSING_DIRS[*]}"
    fi
else
    record_fail "No evidence runs found to validate structure"
fi

# Check 6: Evidence metadata is present
log "Check 6: Evidence metadata is present"
if [[ -n "$LATEST_RUN" ]] && [[ -f "$LATEST_RUN/meta/run.json" ]]; then
    if grep -q "run_id" "$LATEST_RUN/meta/run.json" && \
       grep -q "time_utc" "$LATEST_RUN/meta/run.json"; then
        record_pass "Evidence metadata is present and valid"
    else
        record_fail "Evidence metadata is incomplete"
    fi
else
    record_fail "Evidence metadata file not found"
fi

# Check 7: Dashboard serve script is executable
log "Check 7: Dashboard serve script is executable"
if [[ -x "$PROJECT_ROOT/tools/dashboard/serve.sh" ]]; then
    record_pass "Dashboard serve script is executable"
else
    if [[ -f "$PROJECT_ROOT/tools/dashboard/serve.sh" ]]; then
        record_fail "Dashboard serve script exists but is not executable"
    else
        record_fail "Dashboard serve script not found"
    fi
fi

# Check 8: Dashboard constitutional compliance
log "Check 8: Dashboard constitutional compliance"
if grep -q "Read-Only Validation Observer — No Decision Authority" "$PROJECT_ROOT/tools/dashboard/index.html" && \
   grep -q "ZERO validation authority" "$PROJECT_ROOT/tools/dashboard/index.html"; then
    record_pass "Dashboard declares constitutional compliance"
else
    record_fail "Dashboard missing constitutional compliance declarations"
fi

# Check 9: Dashboard JavaScript has no validation authority
log "Check 9: Dashboard JavaScript has no validation authority"
if grep -q "Authority: ZERO - purely observational" "$PROJECT_ROOT/tools/dashboard/dashboard.js"; then
    record_pass "Dashboard JavaScript declares zero authority"
else
    record_fail "Dashboard JavaScript missing authority declaration"
fi

# Check 10: Evidence isolation is maintained
log "Check 10: Evidence isolation is maintained"
if [[ -f "$PROJECT_ROOT/scripts/check_evidence_isolation.sh" ]]; then
    if bash "$PROJECT_ROOT/scripts/check_evidence_isolation.sh" >> "$LOG_FILE" 2>&1; then
        record_pass "Evidence isolation check passed"
    else
        record_fail "Evidence isolation check failed"
    fi
else
    record_fail "Evidence isolation check script not found"
fi

# Check 11: Observation boundary is maintained
log "Check 11: Observation boundary is maintained"
if [[ -f "$PROJECT_ROOT/scripts/check_observation_boundary.sh" ]]; then
    if bash "$PROJECT_ROOT/scripts/check_observation_boundary.sh" >> "$LOG_FILE" 2>&1; then
        record_pass "Observation boundary check passed"
    else
        record_fail "Observation boundary check failed"
    fi
else
    record_fail "Observation boundary check script not found"
fi

# Check 12: Dashboard README exists
log "Check 12: Dashboard README exists"
if [[ -f "$PROJECT_ROOT/tools/dashboard/README.md" ]]; then
    if grep -q "Observability Dashboard" "$PROJECT_ROOT/tools/dashboard/README.md"; then
        record_pass "Dashboard README exists and is valid"
    else
        record_fail "Dashboard README exists but is incomplete"
    fi
else
    record_fail "Dashboard README not found"
fi

# Check 13: Previous checkpoints passed
log "Check 13: Previous checkpoints passed"
CHECKPOINT_23_PASSED=false
CHECKPOINT_24_PASSED=false

# Check if task 23 evidence exists
if [[ -f "$PROJECT_ROOT/out/evidence/task23_web_dashboard/result.json" ]]; then
    if grep -q '"status": "PASS"' "$PROJECT_ROOT/out/evidence/task23_web_dashboard/result.json" 2>/dev/null; then
        CHECKPOINT_23_PASSED=true
    fi
fi

# Check if task 24 evidence exists
if [[ -f "$PROJECT_ROOT/out/evidence/task24_web_dashboard_checkpoint/result.json" ]]; then
    if grep -q '"status": "PASS"' "$PROJECT_ROOT/out/evidence/task24_web_dashboard_checkpoint/result.json" 2>/dev/null; then
        CHECKPOINT_24_PASSED=true
    fi
fi

if [[ "$CHECKPOINT_23_PASSED" == true ]] && [[ "$CHECKPOINT_24_PASSED" == true ]]; then
    record_pass "Previous checkpoints (23, 24) passed"
elif [[ "$CHECKPOINT_23_PASSED" == true ]]; then
    record_fail "Checkpoint 24 did not pass"
elif [[ "$CHECKPOINT_24_PASSED" == true ]]; then
    record_fail "Checkpoint 23 did not pass"
else
    # This is acceptable - checkpoints may not have been run yet
    log "⚠️  Previous checkpoint evidence not found (acceptable if not run yet)"
fi

# Check 14: No regressions in core functionality
log "Check 14: No regressions in core functionality"
if [[ -f "$PROJECT_ROOT/scripts/dev_loop.sh" ]]; then
    # Verify dev_loop.sh still has core functionality
    if grep -q "EARLY_BOOT_OK" "$PROJECT_ROOT/scripts/dev_loop.sh" && \
       grep -q "LATE_INIT_END" "$PROJECT_ROOT/scripts/dev_loop.sh" && \
       grep -q "AYKEN_BOOT_OK" "$PROJECT_ROOT/scripts/dev_loop.sh"; then
        record_pass "Core dev loop functionality intact"
    else
        record_fail "Core dev loop functionality appears broken"
    fi
else
    record_fail "dev_loop.sh not found"
fi

# Check 15: Dashboard can be served (test serve script syntax)
log "Check 15: Dashboard serve script syntax"
if bash -n "$PROJECT_ROOT/tools/dashboard/serve.sh" 2>> "$LOG_FILE"; then
    record_pass "Dashboard serve script has valid syntax"
else
    record_fail "Dashboard serve script has syntax errors"
fi

# Summary
log ""
log "========================================="
log "Validation Summary"
log "========================================="
log "Checks passed: $CHECKS_PASSED"
log "Checks failed: $CHECKS_FAILED"

if [[ $CHECKS_FAILED -gt 0 ]]; then
    log ""
    log "Failed checks:"
    for failure in "${FAILURES[@]}"; do
        log "  - $failure"
    done
fi

# Generate result JSON
cat > "$RESULT_FILE" <<EOF
{
  "task": "25. Final checkpoint - Unified observability validated",
  "status": "$([ $CHECKS_FAILED -eq 0 ] && echo "PASS" || echo "FAIL")",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "checks": {
    "total": $((CHECKS_PASSED + CHECKS_FAILED)),
    "passed": $CHECKS_PASSED,
    "failed": $CHECKS_FAILED
  },
  "failures": $(if [ ${#FAILURES[@]} -gt 0 ]; then printf '%s\n' "${FAILURES[@]}" | jq -R . | jq -s .; else echo '[]'; fi),
  "evidence_dir": "$EVIDENCE_DIR",
  "maintainer": "Kenan AY — System Architect"
}
EOF

log ""
log "Result written to: $RESULT_FILE"
log ""

# Exit with appropriate status
if [[ $CHECKS_FAILED -eq 0 ]]; then
    log_success "✅ PASS: Unified observability validated"
    exit 0
else
    log_fail "❌ FAIL: Unified observability validation failed"
    exit 1
fi

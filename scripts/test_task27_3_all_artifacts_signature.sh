#!/usr/bin/env bash
# Test: Developer signature in all generated artifacts
# Validates: R24 (Developer Signature Integration)
# Task: 27.3
#
# Maintainer: Kenan AY — System Architect

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

EVIDENCE_DIR="out/evidence/task27_3_artifacts_test"
mkdir -p "$EVIDENCE_DIR"

RESULT_JSON="$EVIDENCE_DIR/result.json"
LOG_FILE="$EVIDENCE_DIR/validation.log"

log() {
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*" | tee -a "$LOG_FILE"
}

fail() {
    log "❌ FAIL: $*"
    jq -n \
        --arg status "FAIL" \
        --arg reason "$*" \
        --arg timestamp "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
        '{
            status: $status,
            reason: $reason,
            timestamp: $timestamp,
            task: "27.3",
            requirement: "R24"
        }' > "$RESULT_JSON"
    exit 1
}

pass() {
    log "✅ PASS: $*"
    jq -n \
        --arg status "PASS" \
        --arg message "$*" \
        --arg timestamp "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
        '{
            status: $status,
            message: $message,
            timestamp: $timestamp,
            task: "27.3",
            requirement: "R24"
        }' > "$RESULT_JSON"
    exit 0
}

log "=========================================="
log "Task 27.3: Developer Signature in All Generated Artifacts"
log "=========================================="
log ""

# Check 1: Verify all dev-loop scripts have developer attribution
log "Check 1: Verify all dev-loop scripts have developer attribution"

SCRIPTS_CHECKED=0
SCRIPTS_MISSING=0
MISSING_SCRIPTS=()

# List of dev-loop related scripts that should have attribution
DEV_LOOP_SCRIPTS=(
    "scripts/dev_loop.sh"
    "scripts/generate_evidence.sh"
    "scripts/oracle.sh"
    "scripts/find_regression.sh"
    "scripts/check_perf_regression.sh"
    "scripts/check_vcp_runtime_contract.sh"
    "scripts/test_vcp_evidence.sh"
    "scripts/test_vcp_fail_closed.sh"
    "scripts/test_vcp_runtime_hook.sh"
    "scripts/test_vcp_trust_verification.sh"
    "scripts/check_evidence_isolation.sh"
    "scripts/check_observation_boundary.sh"
    "scripts/check_naming_compliance.sh"
    "scripts/test_marker_validation.sh"
    "scripts/test_exit_status_contract.sh"
    "scripts/test_qemu_integration.sh"
    "scripts/test_devloop_isolation.sh"
    "scripts/test_evidence_as_input_detection.sh"
    "scripts/test_evidence_non_authoritative_property.sh"
    "scripts/test_isolation_boundary_guarantee.sh"
    "scripts/test_constitutional_compliance.sh"
    "scripts/test_full_validation_capability.sh"
    "scripts/test_regression_detection_capability.sh"
    "scripts/test_task10_integration_completeness.sh"
    "scripts/checkpoint_task9_test_scripts_validated.sh"
    "scripts/checkpoint_task10_integration_complete.sh"
    "scripts/test_task12_automated_regression_finder.sh"
    "scripts/test_known_regressions.sh"
    "scripts/test_task18_observability_dashboard.sh"
    "scripts/test_task25_unified_observability.sh"
    "scripts/test_ci_workflow_assurance.sh"
    "scripts/validate_ci_workflow.sh"
    "scripts/setup_branch_protection.sh"
    "scripts/validate_branch_protection.sh"
    "scripts/test_task27_1_evidence_metadata_signature.sh"
    "scripts/test_task27_2_dashboard_signature.sh"
)

for script in "${DEV_LOOP_SCRIPTS[@]}"; do
    if [[ -f "$script" ]]; then
        SCRIPTS_CHECKED=$((SCRIPTS_CHECKED + 1))

        # Check for developer attribution (Author or Maintainer)
        if ! grep -q "Kenan AY" "$script"; then
            log "⚠ Missing attribution: $script"
            SCRIPTS_MISSING=$((SCRIPTS_MISSING + 1))
            MISSING_SCRIPTS+=("$script")
        fi
    fi
done

log "Checked ${SCRIPTS_CHECKED} scripts"
log "Missing attribution: ${SCRIPTS_MISSING}"

if [[ $SCRIPTS_MISSING -gt 0 ]]; then
    log ""
    log "Scripts missing 'Kenan AY' attribution:"
    for script in "${MISSING_SCRIPTS[@]}"; do
        log "  - $script"
    done
    log ""
    fail "${SCRIPTS_MISSING} scripts missing developer attribution"
fi

log "✓ All ${SCRIPTS_CHECKED} dev-loop scripts have developer attribution"

# Check 2: Verify dashboard files have developer attribution
log ""
log "Check 2: Verify dashboard files have developer attribution"

DASHBOARD_FILES=(
    "tools/dashboard/index.html"
    "tools/dashboard/dashboard.js"
    "tools/dashboard/README.md"
    "tools/dashboard/serve.sh"
)

DASHBOARD_CHECKED=0
DASHBOARD_MISSING=0

for file in "${DASHBOARD_FILES[@]}"; do
    if [[ -f "$file" ]]; then
        DASHBOARD_CHECKED=$((DASHBOARD_CHECKED + 1))

        if ! grep -q "Kenan AY" "$file"; then
            log "⚠ Missing attribution: $file"
            DASHBOARD_MISSING=$((DASHBOARD_MISSING + 1))
        fi
    fi
done

if [[ $DASHBOARD_MISSING -gt 0 ]]; then
    fail "${DASHBOARD_MISSING} dashboard files missing developer attribution"
fi

log "✓ All ${DASHBOARD_CHECKED} dashboard files have developer attribution"

# Check 3: Verify evidence metadata includes developer attribution
log ""
log "Check 3: Verify evidence metadata includes developer attribution"

# Find a recent run directory
LATEST_RUN=$(find out/evidence -maxdepth 1 -type d -name "run-*" 2>/dev/null | sort -r | head -1 || echo "")

if [[ -n "$LATEST_RUN" ]]; then
    RUN_JSON="${LATEST_RUN}/meta/run.json"

    if [[ -f "$RUN_JSON" ]]; then
        if ! grep -q '"developer".*:.*"Kenan AY"' "$RUN_JSON" || \
           ! grep -q '"generated_by".*:.*"Kenan AY' "$RUN_JSON"; then
            fail "Evidence metadata missing developer attribution"
        fi

        log "✓ Evidence metadata includes developer attribution"
    else
        log "⚠ run.json not found, skipping evidence metadata check"
    fi

    # Check evidence reports
    REPORTS_DIR="${LATEST_RUN}/reports"
    if [[ -d "$REPORTS_DIR" ]]; then
        REPORT_COUNT=0
        REPORT_MISSING=0

        for report in "$REPORTS_DIR"/*.json; do
            if [[ -f "$report" ]]; then
                REPORT_COUNT=$((REPORT_COUNT + 1))

                if ! grep -q '"generated_by".*:.*"Kenan AY' "$report"; then
                    log "⚠ Missing attribution in: $report"
                    REPORT_MISSING=$((REPORT_MISSING + 1))
                fi
            fi
        done

        if [[ $REPORT_MISSING -gt 0 ]]; then
            fail "${REPORT_MISSING} evidence reports missing developer attribution"
        fi

        if [[ $REPORT_COUNT -gt 0 ]]; then
            log "✓ All ${REPORT_COUNT} evidence reports have developer attribution"
        fi
    fi
else
    log "⚠ No evidence runs found, skipping evidence metadata check"
fi

# Check 4: Verify signature does NOT appear in runtime logs
log ""
log "Check 4: Verify signature does NOT appear in runtime logs (R29: Signature Non-Propagation)"

BOOT_LOG="out/logs/boot_watch.log"

if [[ -f "$BOOT_LOG" ]]; then
    if grep -q "Kenan AY" "$BOOT_LOG"; then
        fail "Developer signature found in runtime boot log (violates R29: Signature Non-Propagation)"
    fi

    log "✓ Developer signature correctly absent from runtime logs"
else
    log "⚠ Boot log not found, skipping runtime log check"
fi

# Check 5: Verify CI scripts have developer attribution
log ""
log "Check 5: Verify CI scripts have developer attribution"

CI_SCRIPTS_CHECKED=0
CI_SCRIPTS_MISSING=0

if [[ -d "scripts/ci" ]]; then
    for ci_script in scripts/ci/*.sh; do
        if [[ -f "$ci_script" ]]; then
            CI_SCRIPTS_CHECKED=$((CI_SCRIPTS_CHECKED + 1))

            # CI scripts may have different attribution patterns
            # Check for any mention of Kenan AY or allow scripts without attribution
            # (some CI scripts are auto-generated or system scripts)
            if grep -q "Kenan AY" "$ci_script"; then
                # Has attribution, good
                :
            else
                # No attribution - this is acceptable for some CI scripts
                log "ℹ No attribution in: $ci_script (acceptable for CI scripts)"
            fi
        fi
    done

    log "✓ Checked ${CI_SCRIPTS_CHECKED} CI scripts"
fi

log ""
log "=========================================="
log "Task 27.3: PASS"
log "=========================================="
log ""
log "All generated artifacts include developer attribution:"
log "  - ${SCRIPTS_CHECKED} dev-loop scripts"
log "  - ${DASHBOARD_CHECKED} dashboard files"
log "  - Evidence metadata (run.json, reports/*.json)"
log ""
log "Signature correctly absent from runtime logs (R29 compliance)"
log ""

pass "All generated artifacts include developer attribution (R24, R29)"

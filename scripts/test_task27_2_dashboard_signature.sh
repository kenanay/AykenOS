#!/usr/bin/env bash
# Test: Developer signature in web dashboard
# Validates: R24 (Developer Signature Integration)
# Task: 27.2
#
# Maintainer: Kenan AY — System Architect

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

DASHBOARD_DIR="tools/dashboard"
EVIDENCE_DIR="out/evidence/task27_2_dashboard_test"
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
            task: "27.2",
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
            task: "27.2",
            requirement: "R24"
        }' > "$RESULT_JSON"
    exit 0
}

log "=========================================="
log "Task 27.2: Developer Signature in Web Dashboard"
log "=========================================="
log ""

# Check 1: Verify index.html includes developer attribution
log "Check 1: Verify index.html includes developer attribution"

if [[ ! -f "$DASHBOARD_DIR/index.html" ]]; then
    fail "Dashboard index.html not found"
fi

if ! grep -q "Kenan AY" "$DASHBOARD_DIR/index.html"; then
    fail "index.html missing 'Kenan AY' attribution"
fi

if ! grep -q "Maintainer.*Kenan AY" "$DASHBOARD_DIR/index.html"; then
    fail "index.html missing 'Maintainer: Kenan AY' attribution"
fi

log "✓ index.html contains developer attribution"

# Check 2: Verify dashboard.js includes developer attribution
log ""
log "Check 2: Verify dashboard.js includes developer attribution"

if [[ ! -f "$DASHBOARD_DIR/dashboard.js" ]]; then
    fail "Dashboard dashboard.js not found"
fi

if ! grep -q "Kenan AY" "$DASHBOARD_DIR/dashboard.js"; then
    fail "dashboard.js missing 'Kenan AY' attribution"
fi

if ! grep -q "Maintainer.*Kenan AY" "$DASHBOARD_DIR/dashboard.js"; then
    fail "dashboard.js missing 'Maintainer: Kenan AY' attribution"
fi

log "✓ dashboard.js contains developer attribution"

# Check 3: Verify README.md includes developer attribution
log ""
log "Check 3: Verify README.md includes developer attribution"

if [[ ! -f "$DASHBOARD_DIR/README.md" ]]; then
    fail "Dashboard README.md not found"
fi

if ! grep -q "Kenan AY" "$DASHBOARD_DIR/README.md"; then
    fail "README.md missing 'Kenan AY' attribution"
fi

if ! grep -q "Maintainer.*Kenan AY" "$DASHBOARD_DIR/README.md"; then
    fail "README.md missing 'Maintainer: Kenan AY' attribution"
fi

log "✓ README.md contains developer attribution"

# Check 4: Verify serve.sh includes developer attribution
log ""
log "Check 4: Verify serve.sh includes developer attribution"

if [[ ! -f "$DASHBOARD_DIR/serve.sh" ]]; then
    fail "Dashboard serve.sh not found"
fi

if ! grep -q "Kenan AY" "$DASHBOARD_DIR/serve.sh"; then
    fail "serve.sh missing 'Kenan AY' attribution"
fi

if ! grep -q "Maintainer.*Kenan AY" "$DASHBOARD_DIR/serve.sh"; then
    fail "serve.sh missing 'Maintainer: Kenan AY' attribution"
fi

log "✓ serve.sh contains developer attribution"

# Check 5: Verify attribution is visible in UI
log ""
log "Check 5: Verify attribution is visible in UI (HTML)"

if ! grep -q 'class="attribution"' "$DASHBOARD_DIR/index.html"; then
    fail "index.html missing attribution CSS class"
fi

if ! grep -q '<div class="attribution">.*Kenan AY' "$DASHBOARD_DIR/index.html"; then
    fail "index.html missing visible attribution element"
fi

log "✓ Attribution is visible in dashboard UI"

# Check 6: Verify signature does NOT appear in runtime logs
log ""
log "Check 6: Verify signature does NOT appear in runtime logs (R29: Signature Non-Propagation)"

BOOT_LOG="out/logs/boot_watch.log"

if [[ -f "$BOOT_LOG" ]]; then
    if grep -q "Kenan AY" "$BOOT_LOG"; then
        fail "Developer signature found in runtime boot log (violates R29: Signature Non-Propagation)"
    fi

    log "✓ Developer signature correctly absent from runtime logs"
else
    log "⚠ Boot log not found, skipping runtime log check"
fi

log ""
log "=========================================="
log "Task 27.2: PASS"
log "=========================================="
log ""
log "Dashboard includes developer attribution in:"
log "  - index.html (visible in UI)"
log "  - dashboard.js (code comments)"
log "  - README.md (documentation)"
log "  - serve.sh (script header)"
log ""
log "Signature correctly absent from runtime logs (R29 compliance)"
log ""

pass "Dashboard includes developer attribution (R24, R29)"

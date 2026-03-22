#!/usr/bin/env bash
# Alias-Aware Address Space Leak Proof Audit Script
# Purpose: Validate alias proof witness in boot log and produce audit evidence
# Authority: Phase 11 Memory Model Verification

set -euo pipefail

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info() { echo -e "${CYAN}ℹ $1${NC}"; }
success() { echo -e "${GREEN}✓ $1${NC}"; }
error() { echo -e "${RED}✗ $1${NC}"; }
warning() { echo -e "${YELLOW}⚠ $1${NC}"; }

usage() {
    cat <<EOF
Alias Proof Audit Script (Phase 11 v1)

Usage: $0 <boot.log> [report.json]

Arguments:
  boot.log      Path to QEMU boot log (debugcon output)
  report.json   Path to output JSON report (optional)

Validates:
  1. [[AYKEN_ALIAS_PROOF_OK]] witness appears exactly 1 time
  2. [[AYKEN_ALIAS_LEAK_DETECTED]] witness appears exactly 0 times
  3. leaked=0 field present and numerically zero
  4. total == verified (numerically equal)
  5. proof_scope=admitted_surface in report.json

Exit codes:
  0 - All checks passed
  1 - One or more checks failed
  2 - Usage error or missing input
EOF
}

if [[ $# -lt 1 ]]; then
    usage
    exit 2
fi

BOOT_LOG="$1"
REPORT_JSON="${2:-}"

if [[ ! -f "$BOOT_LOG" ]]; then
    error "Boot log not found: $BOOT_LOG"
    exit 2
fi

# Determine violations.txt path (same directory as boot.log)
VIOLATIONS_TXT="$(dirname "$BOOT_LOG")/violations.txt"
> "$VIOLATIONS_TXT"  # Clear violations file

# Determine report.json path if not provided
if [[ -z "$REPORT_JSON" ]]; then
    REPORT_JSON="$(dirname "$BOOT_LOG")/report.json"
fi

info "Alias Proof Audit"
info "Boot log: $BOOT_LOG"
info "Report: $REPORT_JSON"
info "Violations: $VIOLATIONS_TXT"
echo ""

# Validation state
VIOLATIONS=0

# Check 1: [[AYKEN_ALIAS_PROOF_OK]] appears exactly 1 time
info "Check 1: [[AYKEN_ALIAS_PROOF_OK]] witness count"
PROOF_OK_COUNT=$(grep -c '\[\[AYKEN_ALIAS_PROOF_OK\]\]' "$BOOT_LOG" || true)
if [[ "$PROOF_OK_COUNT" -eq 1 ]]; then
    success "[[AYKEN_ALIAS_PROOF_OK]] found exactly 1 time"
else
    error "[[AYKEN_ALIAS_PROOF_OK]] found $PROOF_OK_COUNT times (expected: 1)"
    echo "VIOLATION: [[AYKEN_ALIAS_PROOF_OK]] count is $PROOF_OK_COUNT (expected: 1)" >> "$VIOLATIONS_TXT"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Check 2: [[AYKEN_ALIAS_LEAK_DETECTED]] appears exactly 0 times
info "Check 2: [[AYKEN_ALIAS_LEAK_DETECTED]] witness count"
LEAK_DETECTED_COUNT=$(grep -c '\[\[AYKEN_ALIAS_LEAK_DETECTED\]\]' "$BOOT_LOG" || true)
if [[ "$LEAK_DETECTED_COUNT" -eq 0 ]]; then
    success "[[AYKEN_ALIAS_LEAK_DETECTED]] found 0 times (no leaks)"
else
    error "[[AYKEN_ALIAS_LEAK_DETECTED]] found $LEAK_DETECTED_COUNT times (expected: 0)"
    echo "VIOLATION: [[AYKEN_ALIAS_LEAK_DETECTED]] count is $LEAK_DETECTED_COUNT (expected: 0)" >> "$VIOLATIONS_TXT"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Check 3: leaked=0 field present and numerically zero
info "Check 3: leaked=0 field validation"
LEAKED_LINE=$(grep '\[\[AYKEN_ALIAS_PROOF_OK\]\]' "$BOOT_LOG" | head -n1 || true)
if [[ -n "$LEAKED_LINE" ]]; then
    if echo "$LEAKED_LINE" | grep -q 'leaked=0'; then
        # Extract leaked value and verify it's numerically 0
        LEAKED_VALUE=$(echo "$LEAKED_LINE" | sed -n 's/.*leaked=\([0-9]*\).*/\1/p')
        if [[ "$LEAKED_VALUE" -eq 0 ]]; then
            success "leaked=0 field present and numerically zero"
        else
            error "leaked field value is $LEAKED_VALUE (expected: 0)"
            echo "VIOLATION: leaked field value is $LEAKED_VALUE (expected: 0)" >> "$VIOLATIONS_TXT"
            VIOLATIONS=$((VIOLATIONS + 1))
        fi
    else
        error "leaked=0 field not found in [[AYKEN_ALIAS_PROOF_OK]] line"
        echo "VIOLATION: leaked=0 field not found in proof witness" >> "$VIOLATIONS_TXT"
        VIOLATIONS=$((VIOLATIONS + 1))
    fi
else
    warning "Cannot validate leaked=0 (no [[AYKEN_ALIAS_PROOF_OK]] line found)"
fi

# Check 4: total == verified (numerically equal)
info "Check 4: total == verified validation"
if [[ -n "$LEAKED_LINE" ]]; then
    TOTAL_VALUE=$(echo "$LEAKED_LINE" | sed -n 's/.*total=\([0-9]*\).*/\1/p')
    VERIFIED_VALUE=$(echo "$LEAKED_LINE" | sed -n 's/.*verified=\([0-9]*\).*/\1/p')
    
    if [[ -n "$TOTAL_VALUE" && -n "$VERIFIED_VALUE" ]]; then
        if [[ "$TOTAL_VALUE" -eq "$VERIFIED_VALUE" ]]; then
            success "total ($TOTAL_VALUE) == verified ($VERIFIED_VALUE)"
        else
            error "total ($TOTAL_VALUE) != verified ($VERIFIED_VALUE)"
            echo "VIOLATION: total ($TOTAL_VALUE) != verified ($VERIFIED_VALUE)" >> "$VIOLATIONS_TXT"
            VIOLATIONS=$((VIOLATIONS + 1))
        fi
    else
        error "Cannot extract total or verified values from proof witness"
        echo "VIOLATION: total or verified field missing in proof witness" >> "$VIOLATIONS_TXT"
        VIOLATIONS=$((VIOLATIONS + 1))
    fi
else
    warning "Cannot validate total==verified (no [[AYKEN_ALIAS_PROOF_OK]] line found)"
fi

# Generate report.json
info "Generating report.json"
cat > "$REPORT_JSON" <<EOF
{
  "audit_type": "alias_proof",
  "phase": "Phase 11 v1",
  "proof_scope": "admitted_surface",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "boot_log": "$BOOT_LOG",
  "checks": {
    "proof_ok_count": {
      "expected": 1,
      "actual": $PROOF_OK_COUNT,
      "passed": $([ "$PROOF_OK_COUNT" -eq 1 ] && echo "true" || echo "false")
    },
    "leak_detected_count": {
      "expected": 0,
      "actual": $LEAK_DETECTED_COUNT,
      "passed": $([ "$LEAK_DETECTED_COUNT" -eq 0 ] && echo "true" || echo "false")
    },
    "leaked_field": {
      "expected": 0,
      "actual": ${LEAKED_VALUE:-null},
      "passed": $([ -n "${LEAKED_VALUE:-}" ] && [ "$LEAKED_VALUE" -eq 0 ] && echo "true" || echo "false")
    },
    "total_verified_equality": {
      "total": ${TOTAL_VALUE:-null},
      "verified": ${VERIFIED_VALUE:-null},
      "passed": $([ -n "${TOTAL_VALUE:-}" ] && [ -n "${VERIFIED_VALUE:-}" ] && [ "$TOTAL_VALUE" -eq "$VERIFIED_VALUE" ] && echo "true" || echo "false")
    }
  },
  "violations_count": $VIOLATIONS,
  "violations_file": "$VIOLATIONS_TXT",
  "verdict": "$([ $VIOLATIONS -eq 0 ] && echo "PASS" || echo "FAIL")"
}
EOF

# Check 5: proof_scope=admitted_surface in report.json
info "Check 5: proof_scope=admitted_surface in report.json"
if grep -q '"proof_scope": "admitted_surface"' "$REPORT_JSON"; then
    success "proof_scope=admitted_surface present in report.json"
else
    error "proof_scope=admitted_surface not found in report.json"
    echo "VIOLATION: proof_scope=admitted_surface not found in report.json" >> "$VIOLATIONS_TXT"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

echo ""
echo "================== Alias Proof Audit Summary =================="
echo "Boot log: $BOOT_LOG"
echo "Report: $REPORT_JSON"
echo "Violations: $VIOLATIONS"
echo "Verdict: $([ $VIOLATIONS -eq 0 ] && echo "PASS" || echo "FAIL")"
echo "==============================================================="

if [[ $VIOLATIONS -eq 0 ]]; then
    success "All checks passed - alias proof validated"
    exit 0
else
    error "$VIOLATIONS violation(s) detected - see $VIOLATIONS_TXT"
    exit 1
fi

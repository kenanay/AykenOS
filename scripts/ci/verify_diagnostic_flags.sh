#!/usr/bin/env bash
#
# AykenOS Diagnostic Flags Verification
#
# Purpose: Verify that diagnostic flags like AYKEN_RING3_FETCH_PROBE are
# disabled in production builds. This prevents the false blocker that occurred
# in Task 1 from recurring.
#
# Expected Behavior:
# - Production builds: AYKEN_RING3_FETCH_PROBE=0 or undefined
# - Diagnostic builds: AYKEN_RING3_FETCH_PROBE=1 allowed (explicit opt-in)
# - CI gate enforces production flag values
#
# Spec: scheduler-primary-regression-rca
# Task: 2 - Preservation property tests

set -euo pipefail

echo "============================================================"
echo " DIAGNOSTIC FLAGS VERIFICATION"
echo " Spec: scheduler-primary-regression-rca"
echo " Task: 2 - Preservation property tests"
echo "============================================================"
echo

# Check if we're in a production build context
# Production: no explicit diagnostic flags set
# Diagnostic: explicit AYKEN_RING3_FETCH_PROBE=1 or similar

PRODUCTION_MODE=1

# Check environment variables
if [[ "${AYKEN_RING3_FETCH_PROBE:-0}" == "1" ]]; then
    echo "⚠️  WARNING: AYKEN_RING3_FETCH_PROBE=1 detected"
    echo "   This is a diagnostic flag that should NOT be set in production"
    PRODUCTION_MODE=0
fi

if [[ "${AYKEN_RING3_SECOND_CANONICAL_PROBE:-0}" == "1" ]]; then
    echo "⚠️  WARNING: AYKEN_RING3_SECOND_CANONICAL_PROBE=1 detected"
    echo "   This is a diagnostic flag that should NOT be set in production"
    PRODUCTION_MODE=0
fi

if [[ "${AYKEN_RING3_FRESH_FRAME_PROBE:-0}" == "1" ]]; then
    echo "⚠️  WARNING: AYKEN_RING3_FRESH_FRAME_PROBE=1 detected"
    echo "   This is a diagnostic flag that should NOT be set in production"
    PRODUCTION_MODE=0
fi

if [[ "${AYKEN_SYSCALL_DIAGNOSTIC_MARKERS_ENABLE:-0}" == "1" ]]; then
    echo "⚠️  WARNING: AYKEN_SYSCALL_DIAGNOSTIC_MARKERS_ENABLE=1 detected"
    echo "   Syscall diagnostic markers are opt-in and must stay out of production performance runs"
    PRODUCTION_MODE=0
fi

# Check Makefile for diagnostic flag definitions in default/production targets
if grep -E "^(all|efi-img|kernel\.elf):.*AYKEN_RING3_FETCH_PROBE=1" Makefile 2>/dev/null; then
    echo "⚠️  WARNING: AYKEN_RING3_FETCH_PROBE=1 found in production Makefile target"
    echo "   Production builds should not define this flag"
    PRODUCTION_MODE=0
fi

if grep -E "^AYKEN_SYSCALL_DIAGNOSTIC_MARKERS_ENABLE[[:space:]]*\\?=[[:space:]]*1" Makefile 2>/dev/null; then
    echo "⚠️  WARNING: AYKEN_SYSCALL_DIAGNOSTIC_MARKERS_ENABLE defaults to 1"
    echo "   Syscall diagnostic markers must be explicit opt-in"
    PRODUCTION_MODE=0
fi

echo
echo "============================================================"
echo " RESULT"
echo "============================================================"

if [[ $PRODUCTION_MODE -eq 1 ]]; then
    echo "✅ PASS: Production mode verified"
    echo
    echo "Property Verified:"
    echo "  → AYKEN_RING3_FETCH_PROBE is disabled (0 or undefined)"
    echo "  → AYKEN_SYSCALL_DIAGNOSTIC_MARKERS_ENABLE defaults to disabled"
    echo "  → No diagnostic flags detected in production build"
    echo "  → False blocker from Task 1 cannot recur"
    exit 0
else
    echo "❌ FAIL: Diagnostic flags detected in production context"
    echo
    echo "Action Required:"
    echo "  → Set AYKEN_RING3_FETCH_PROBE=0 or leave undefined"
    echo "  → Remove diagnostic flag definitions from production Makefile"
    echo "  → Use diagnostic flags only in explicit diagnostic builds"
    exit 1
fi

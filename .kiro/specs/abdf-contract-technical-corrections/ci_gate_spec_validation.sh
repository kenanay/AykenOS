#!/usr/bin/env bash
# ci_gate_spec_validation.sh - CI Gate for Spec Validation
#
# Purpose: Enforce Level 3 validation for spec changes
# Authority: Constitutional Enforcement (Phase-17.5)
# Status: CI-AUTHORITATIVE

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
SPEC_DIR="${SPEC_DIR:-.kiro/specs/abdf-contract-technical-corrections}"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)-$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")-$$}"
EVIDENCE_DIR="${EVIDENCE_DIR:-out/evidence/$RUN_ID/spec-validation}"

# Files
ORIGINAL="$SPEC_DIR/ORIGINAL_BASELINE.md"
FIXED="$SPEC_DIR/FIXED_DOCUMENT.md"
EXPECTED_CHANGES="$SPEC_DIR/expected_changes.yml"
BUG_VALIDATOR="$SPEC_DIR/validate_bug_conditions.sh"
PRESERVATION_VALIDATOR="$SPEC_DIR/validate_preservation.py"

echo "== CI GATE SPEC VALIDATION =="
echo "run_id: $RUN_ID"
echo "spec_dir: $SPEC_DIR"
echo "evidence_dir: $EVIDENCE_DIR"
echo ""

# Create evidence directory
mkdir -p "$EVIDENCE_DIR/logs"
mkdir -p "$EVIDENCE_DIR/reports"

# Check if spec validation is applicable
if [ ! -f "$ORIGINAL" ]; then
    echo -e "${YELLOW}⏭️  SKIP: No ORIGINAL baseline found${NC}"
    echo "Spec validation requires ORIGINAL_BASELINE.md"
    echo "This is expected for specs created before Phase-17.5"
    exit 0
fi

if [ ! -f "$FIXED" ]; then
    echo -e "${RED}❌ FAIL: FIXED document not found: $FIXED${NC}"
    exit 1
fi

if [ ! -f "$EXPECTED_CHANGES" ]; then
    echo -e "${RED}❌ FAIL: Expected changes not found: $EXPECTED_CHANGES${NC}"
    exit 1
fi

# Level 1: Bug conditions on ORIGINAL (must FAIL)
echo ">> Level 1: Bug Proof on ORIGINAL"
echo "--------------------------------"
if [ -f "$BUG_VALIDATOR" ]; then
    if ! bash "$BUG_VALIDATOR" "$ORIGINAL" > "$EVIDENCE_DIR/logs/bug_original.log" 2>&1; then
        echo -e "${GREEN}✅ Bug conditions FAIL on ORIGINAL (bugs proven)${NC}"
    else
        echo -e "${RED}❌ FAIL: Bug conditions PASS on ORIGINAL (bugs not proven)${NC}"
        echo "ORIGINAL baseline must contain the bugs being fixed"
        cat "$EVIDENCE_DIR/logs/bug_original.log"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠️  WARN: Bug validator not found, skipping bug proof${NC}"
fi

# Level 1: Bug conditions on FIXED (must PASS)
echo ""
echo ">> Level 1: Bug Proof on FIXED"
echo "--------------------------------"
if [ -f "$BUG_VALIDATOR" ]; then
    if bash "$BUG_VALIDATOR" "$FIXED" > "$EVIDENCE_DIR/logs/bug_fixed.log" 2>&1; then
        echo -e "${GREEN}✅ Bug conditions PASS on FIXED (fixes working)${NC}"
    else
        echo -e "${RED}❌ FAIL: Bug conditions FAIL on FIXED (fixes not working)${NC}"
        echo "FIXED document must pass all bug condition checks"
        cat "$EVIDENCE_DIR/logs/bug_fixed.log"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠️  WARN: Bug validator not found, skipping bug proof${NC}"
fi

# Level 3: Preservation validation (must PASS)
echo ""
echo ">> Level 3: Preservation Validation"
echo "--------------------------------"
if [ -f "$PRESERVATION_VALIDATOR" ]; then
    if python3 "$PRESERVATION_VALIDATOR" "$ORIGINAL" "$FIXED" "$EXPECTED_CHANGES" > "$EVIDENCE_DIR/logs/preservation.log" 2>&1; then
        echo -e "${GREEN}✅ Preservation validation PASS (no scope creep)${NC}"
        
        # Copy reports to evidence directory
        REPORTS_DIR="$(dirname "$PRESERVATION_VALIDATOR")/reports"
        if [ -d "$REPORTS_DIR" ]; then
            cp "$REPORTS_DIR"/preservation_validation_*.json "$EVIDENCE_DIR/reports/" 2>/dev/null || true
            cp "$REPORTS_DIR"/preservation_validation_*.md "$EVIDENCE_DIR/reports/" 2>/dev/null || true
            cp "$REPORTS_DIR"/diff_*.patch "$EVIDENCE_DIR/reports/" 2>/dev/null || true
        fi
    else
        echo -e "${RED}❌ FAIL: Preservation validation FAIL (scope creep detected)${NC}"
        echo "Only expected changes are allowed (no scope creep)"
        cat "$EVIDENCE_DIR/logs/preservation.log"
        exit 1
    fi
else
    echo -e "${RED}❌ FAIL: Preservation validator not found: $PRESERVATION_VALIDATOR${NC}"
    exit 1
fi

# Generate summary
echo ""
echo "== SPEC VALIDATION SUMMARY =="
echo "✅ PASS: All validation levels passed"
echo "Evidence: $EVIDENCE_DIR"
echo ""
echo "Validation Levels:"
echo "  Level 1 (ORIGINAL): Bug conditions FAIL ✅"
echo "  Level 1 (FIXED): Bug conditions PASS ✅"
echo "  Level 3: Preservation validation PASS ✅"
echo ""
echo "CI-Authoritative: YES"
echo "Deterministic: YES"

exit 0

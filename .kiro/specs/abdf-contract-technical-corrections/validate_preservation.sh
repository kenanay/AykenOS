#!/usr/bin/env bash
# validate_preservation.sh - Phase-17.5 Preservation Validation (ALPHA)
# 
# Purpose: Verify that ONLY expected changes were made (no scope creep)
# Authority: Constitutional enforcement for spec validation
# Status: ALPHA - Initial implementation, NOT CI-authoritative
#
# KNOWN LIMITATIONS:
# - YAML parsing is regex-based (fragile to format changes)
# - Section matching is heuristic (may miss context-only changes)
# - No diff hunk → section resolver (coarse-grained detection)
# - No fixture-based PASS/FAIL tests (false positive/negative risk)
# - Line count calculation may be inaccurate in edge cases
#
# CI-AUTHORITATIVE STATUS: NOT YET
# This script requires hardening before becoming CI-authoritative:
# - Fixture-based validation (intentional PASS/FAIL cases)
# - Diff hunk → section mapping
# - Robust YAML parsing (or Python-based parser)
# - False positive/negative testing
#
# Usage:
#   ./validate_preservation.sh ORIGINAL_FILE FIXED_FILE EXPECTED_CHANGES_YML
#
# Exit Codes:
#   0 - PASS (only expected changes found)
#   1 - FAIL (unexpected changes or missing expected changes)
#   2 - ERROR (invalid arguments or file not found)

set -euo pipefail

# ============================================================================
# CONFIGURATION
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TIMESTAMP="$(date -u +%Y%m%d_%H%M%S)"
REPORT_DIR="${SCRIPT_DIR}/reports"
REPORT_FILE="${REPORT_DIR}/preservation_validation_${TIMESTAMP}.md"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ============================================================================
# ARGUMENT VALIDATION
# ============================================================================

if [[ $# -ne 3 ]]; then
    echo -e "${RED}ERROR: Invalid arguments${NC}" >&2
    echo "Usage: $0 ORIGINAL_FILE FIXED_FILE EXPECTED_CHANGES_YML" >&2
    echo "" >&2
    echo "Example:" >&2
    echo "  $0 ORIGINAL_ABDF.md _ayken/specs/ABDF_HARDWARE_CONTRACT.md expected_changes.yml" >&2
    exit 2
fi

ORIGINAL_FILE="$1"
FIXED_FILE="$2"
EXPECTED_CHANGES_YML="$3"

# Validate files exist
if [[ ! -f "$ORIGINAL_FILE" ]]; then
    echo -e "${RED}ERROR: ORIGINAL file not found: $ORIGINAL_FILE${NC}" >&2
    exit 2
fi

if [[ ! -f "$FIXED_FILE" ]]; then
    echo -e "${RED}ERROR: FIXED file not found: $FIXED_FILE${NC}" >&2
    exit 2
fi

if [[ ! -f "$EXPECTED_CHANGES_YML" ]]; then
    echo -e "${RED}ERROR: Expected changes file not found: $EXPECTED_CHANGES_YML${NC}" >&2
    exit 2
fi

# ============================================================================
# HELPER FUNCTIONS
# ============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $*"
}

# Generate hash for file
generate_hash() {
    local file="$1"
    shasum -a 256 "$file" | awk '{print $1}'
}

# ============================================================================
# DIFF GENERATION
# ============================================================================

log_info "Generating unified diff..."

DIFF_FILE="${REPORT_DIR}/diff_${TIMESTAMP}.patch"
mkdir -p "$REPORT_DIR"

# Generate unified diff (suppress error if files are identical)
if diff -u "$ORIGINAL_FILE" "$FIXED_FILE" > "$DIFF_FILE" 2>/dev/null; then
    log_warning "Files are identical - no changes detected"
    DIFF_EMPTY=true
else
    DIFF_EMPTY=false
fi

# Count changed sections (heuristic: count @@ markers in diff)
if [[ "$DIFF_EMPTY" == true ]]; then
    CHANGED_SECTIONS=0
    ADDED_LINES=0
    REMOVED_LINES=0
else
    CHANGED_SECTIONS=$(grep -c "^@@" "$DIFF_FILE" || echo "0")
    # Count added/removed lines (exclude diff metadata lines)
    ADDED_LINES=$(grep "^+" "$DIFF_FILE" | grep -v "^+++" | wc -l | tr -d ' ')
    REMOVED_LINES=$(grep "^-" "$DIFF_FILE" | grep -v "^---" | wc -l | tr -d ' ')
fi

log_info "Diff statistics:"
log_info "  Changed sections: $CHANGED_SECTIONS"
log_info "  Added lines: $ADDED_LINES"
log_info "  Removed lines: $REMOVED_LINES"

# ============================================================================
# EXPECTED CHANGES PARSING
# ============================================================================

log_info "Parsing expected changes from $EXPECTED_CHANGES_YML..."

# Extract expected change sections from fixes array
EXPECTED_SECTIONS=()
EXPECTED_FIX_IDS=()

while IFS= read -r line; do
    # Skip comments and empty lines
    [[ "$line" =~ ^[[:space:]]*# ]] && continue
    [[ -z "$line" ]] && continue
    
    # Extract fix ID
    if [[ "$line" =~ ^[[:space:]]*-[[:space:]]*id:[[:space:]]*(.+) ]]; then
        EXPECTED_FIX_IDS+=("${BASH_REMATCH[1]}")
    fi
    
    # Extract section name
    if [[ "$line" =~ ^[[:space:]]*section:[[:space:]]*\"(.+)\" ]]; then
        EXPECTED_SECTIONS+=("${BASH_REMATCH[1]}")
    fi
done < "$EXPECTED_CHANGES_YML"

log_info "Expected fixes: ${#EXPECTED_FIX_IDS[@]}"
log_info "Expected change sections: ${#EXPECTED_SECTIONS[@]}"

# ============================================================================
# WHITELIST VALIDATION
# ============================================================================

log_info "Validating changes against whitelist..."

VALIDATION_PASSED=true
MATCHED_SECTIONS=0
UNMATCHED_CHANGES=()

# If no changes expected and no changes found, that's a PASS
if [[ ${#EXPECTED_SECTIONS[@]} -eq 0 && "$DIFF_EMPTY" == true ]]; then
    log_success "No changes expected, no changes found - PASS"
    VALIDATION_PASSED=true
elif [[ "$DIFF_EMPTY" == true && ${#EXPECTED_SECTIONS[@]} -gt 0 ]]; then
    log_error "Expected changes but files are identical"
    VALIDATION_PASSED=false
else
    # Check if diff contains expected sections
    for section in "${EXPECTED_SECTIONS[@]}"; do
        # Remove emoji and normalize section name for matching
        section_normalized=$(echo "$section" | sed 's/[^a-zA-Z0-9 ]//g')
        
        if grep -q "$section_normalized" "$DIFF_FILE" || grep -q "$section" "$DIFF_FILE"; then
            log_success "Expected change found: $section"
            ((MATCHED_SECTIONS++))
        else
            log_warning "Expected change NOT found: $section"
        fi
    done
    
    # Verify all expected changes were found
    if [[ $MATCHED_SECTIONS -lt ${#EXPECTED_SECTIONS[@]} ]]; then
        VALIDATION_PASSED=false
    fi
fi

# ============================================================================
# UNEXPECTED CHANGE DETECTION
# ============================================================================

log_info "Checking for unexpected changes..."

# Extract preservation sections from YAML
PRESERVED_SECTIONS=()
while IFS= read -r line; do
    if [[ "$line" =~ ^[[:space:]]*-[[:space:]]*section:[[:space:]]*\"(.+)\" ]]; then
        # Check if we're in preservation block (after "preservation:" line)
        PRESERVED_SECTIONS+=("${BASH_REMATCH[1]}")
    fi
done < <(sed -n '/^preservation:/,/^[a-z_]*:/p' "$EXPECTED_CHANGES_YML" | grep -v "^preservation:" | grep -v "^[a-z_]*:")

log_info "Preserved sections: ${#PRESERVED_SECTIONS[@]}"

UNEXPECTED_CHANGES=0

for section in "${PRESERVED_SECTIONS[@]}"; do
    # Remove emoji and normalize
    section_normalized=$(echo "$section" | sed 's/[^a-zA-Z0-9 ]//g')
    
    # Check if section appears in CHANGED lines (not context)
    # Only check lines starting with + or - (actual changes, not context)
    if grep "^[+-]" "$DIFF_FILE" | grep -q "$section_normalized" || \
       grep "^[+-]" "$DIFF_FILE" | grep -q "$section"; then
        # Verify this is not an expected change
        EXPECTED=false
        for expected_section in "${EXPECTED_SECTIONS[@]}"; do
            expected_normalized=$(echo "$expected_section" | sed 's/[^a-zA-Z0-9 ]//g')
            if [[ "$section_normalized" == *"$expected_normalized"* ]] || [[ "$expected_normalized" == *"$section_normalized"* ]]; then
                EXPECTED=true
                break
            fi
        done
        
        if [[ "$EXPECTED" == false ]]; then
            log_error "Unexpected change in preserved section: $section"
            UNMATCHED_CHANGES+=("$section")
            ((UNEXPECTED_CHANGES++))
            VALIDATION_PASSED=false
        fi
    fi
done

# ============================================================================
# GENERATE REPORT
# ============================================================================

log_info "Generating validation report..."

cat > "$REPORT_FILE" <<EOF
# Preservation Validation Report (ALPHA)

**Generated**: $(date -u +"%Y-%m-%d %H:%M:%S UTC")  
**Script**: validate_preservation.sh  
**Version**: 1.0.0-alpha (Phase-17.5)  
**Status**: ALPHA - Not CI-authoritative

⚠️ **KNOWN LIMITATIONS**:
- YAML parsing is regex-based (fragile)
- Section matching is heuristic (may miss changes)
- No diff hunk → section resolver
- No fixture-based validation
- Requires hardening before CI-authoritative use

---

## Input Files

- **ORIGINAL**: \`$ORIGINAL_FILE\`
  - Hash: $(generate_hash "$ORIGINAL_FILE")
- **FIXED**: \`$FIXED_FILE\`
  - Hash: $(generate_hash "$FIXED_FILE")
- **Expected Changes**: \`$EXPECTED_CHANGES_YML\`

---

## Diff Statistics

- **Changed Sections**: $CHANGED_SECTIONS
- **Added Lines**: $ADDED_LINES
- **Removed Lines**: $REMOVED_LINES
- **Diff File**: \`$DIFF_FILE\`

---

## Whitelist Validation

**Expected Changes**: ${#EXPECTED_SECTIONS[@]}  
**Matched Sections**: $MATCHED_SECTIONS

EOF

if [[ $MATCHED_SECTIONS -eq ${#EXPECTED_SECTIONS[@]} ]]; then
    echo "✅ **Status**: ALL expected changes found" >> "$REPORT_FILE"
else
    echo "⚠️ **Status**: Some expected changes NOT found" >> "$REPORT_FILE"
    VALIDATION_PASSED=false
fi

cat >> "$REPORT_FILE" <<EOF

### Expected Change Sections

EOF

if [[ ${#EXPECTED_SECTIONS[@]} -eq 0 ]]; then
    echo "- (No expected changes)" >> "$REPORT_FILE"
else
    for section in "${EXPECTED_SECTIONS[@]}"; do
        section_normalized=$(echo "$section" | sed 's/[^a-zA-Z0-9 ]//g')
        if [[ "$DIFF_EMPTY" == true ]]; then
            echo "- ❌ \`$section\` (NOT FOUND - files identical)" >> "$REPORT_FILE"
        elif grep -q "$section_normalized" "$DIFF_FILE" || grep -q "$section" "$DIFF_FILE"; then
            echo "- ✅ \`$section\`" >> "$REPORT_FILE"
        else
            echo "- ❌ \`$section\` (NOT FOUND)" >> "$REPORT_FILE"
        fi
    done
fi

cat >> "$REPORT_FILE" <<EOF

---

## Preservation Validation

**Preserved Sections**: ${#PRESERVED_SECTIONS[@]}  
**Unexpected Changes**: $UNEXPECTED_CHANGES

EOF

if [[ $UNEXPECTED_CHANGES -eq 0 ]]; then
    echo "✅ **Status**: NO unexpected changes detected" >> "$REPORT_FILE"
else
    echo "❌ **Status**: Unexpected changes detected in preserved sections" >> "$REPORT_FILE"
fi

cat >> "$REPORT_FILE" <<EOF

### Preserved Sections Check

EOF

for section in "${PRESERVED_SECTIONS[@]}"; do
    section_normalized=$(echo "$section" | sed 's/[^a-zA-Z0-9 ]//g')
    
    # Check if section appears in CHANGED lines only
    if grep "^[+-]" "$DIFF_FILE" | grep -q "$section_normalized" || \
       grep "^[+-]" "$DIFF_FILE" | grep -q "$section"; then
        # Check if expected
        EXPECTED=false
        for expected_section in "${EXPECTED_SECTIONS[@]}"; do
            expected_normalized=$(echo "$expected_section" | sed 's/[^a-zA-Z0-9 ]//g')
            if [[ "$section_normalized" == *"$expected_normalized"* ]] || [[ "$expected_normalized" == *"$section_normalized"* ]]; then
                EXPECTED=true
                break
            fi
        done
        
        if [[ "$EXPECTED" == true ]]; then
            echo "- ⚠️ \`$section\` (changed - expected)" >> "$REPORT_FILE"
        else
            echo "- ❌ \`$section\` (changed - UNEXPECTED)" >> "$REPORT_FILE"
        fi
    else
        echo "- ✅ \`$section\` (unchanged)" >> "$REPORT_FILE"
    fi
done

# ============================================================================
# FINAL VERDICT
# ============================================================================

cat >> "$REPORT_FILE" <<EOF

---

## Final Verdict

EOF

if [[ "$VALIDATION_PASSED" == true ]]; then
    cat >> "$REPORT_FILE" <<EOF
✅ **PASS**: Preservation validation successful

- All expected changes found
- No unexpected changes detected
- Preserved sections remain unchanged

**Conclusion**: The transformation satisfies preservation requirements.
EOF
    log_success "Preservation validation PASSED"
    EXIT_CODE=0
else
    cat >> "$REPORT_FILE" <<EOF
❌ **FAIL**: Preservation validation failed

**Issues**:
EOF
    
    if [[ $MATCHED_SECTIONS -lt ${#EXPECTED_SECTIONS[@]} ]]; then
        echo "- Some expected changes were NOT found" >> "$REPORT_FILE"
    fi
    
    if [[ $UNEXPECTED_CHANGES -gt 0 ]]; then
        echo "- $UNEXPECTED_CHANGES unexpected changes detected in preserved sections" >> "$REPORT_FILE"
    fi
    
    cat >> "$REPORT_FILE" <<EOF

**Conclusion**: The transformation does NOT satisfy preservation requirements.

**Action Required**: Review diff and verify changes are intentional.
EOF
    log_error "Preservation validation FAILED"
    EXIT_CODE=1
fi

cat >> "$REPORT_FILE" <<EOF

---

## Evidence

- **Report**: \`$REPORT_FILE\`
- **Diff**: \`$DIFF_FILE\`
- **Timestamp**: $TIMESTAMP

---

**Validation Level**: Level 3 (Complete Audit Trail) - ALPHA  
**Authority**: Constitutional Enforcement (Phase-17.5)  
**CI-Authoritative**: ❌ NOT YET (requires hardening)

**Next Steps for CI-Authoritative Status**:
1. Add fixture-based PASS/FAIL tests
2. Implement diff hunk → section resolver
3. Replace regex YAML parsing with robust parser
4. Test false positive/negative scenarios
5. Add integration tests with real spec examples

EOF

# ============================================================================
# OUTPUT
# ============================================================================

echo ""
log_info "Validation complete"
log_info "Report: $REPORT_FILE"
log_info "Diff: $DIFF_FILE"
echo ""

if [[ "$VALIDATION_PASSED" == true ]]; then
    log_success "✅ PRESERVATION VALIDATION PASSED"
else
    log_error "❌ PRESERVATION VALIDATION FAILED"
    echo ""
    log_info "Review the report for details: $REPORT_FILE"
fi

exit $EXIT_CODE

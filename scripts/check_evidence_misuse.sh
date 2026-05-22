#!/usr/bin/env bash
# Evidence Misuse Guard
#
# Purpose: Detect patterns where evidence artifacts are used as validation input
# Authority: Governance enforcement
#
# Maintainer: Kenan AY — System Architect

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

VIOLATIONS=0

echo "========================================"
echo "Evidence Misuse Guard"
echo "========================================"
echo ""

# Check 1: Validation scripts must not read evidence artifacts
echo "Check 1: Validation scripts isolation"
echo "--------------------------------------"

VALIDATION_SCRIPTS=(
    "scripts/dev_loop.sh"
    "scripts/oracle.sh"
    "scripts/check_vcp_runtime_contract.sh"
)

for script in "${VALIDATION_SCRIPTS[@]}"; do
    if [[ ! -f "$script" ]]; then
        continue
    fi

    # Check for evidence directory reads
    if grep -q "out/evidence" "$script" 2>/dev/null; then
        # Allow only evidence generation (writing), not reading for validation
        if grep -q "out/evidence.*summary.json\|out/evidence.*markers.json\|out/evidence.*perf.json" "$script" 2>/dev/null; then
            # Check if it's reading (cat, grep, source) vs writing (echo, cat >)
            if grep -E "(cat|grep|source|jq).*out/evidence.*(summary|markers|perf)\.json" "$script" 2>/dev/null; then
                echo -e "${RED}❌ VIOLATION: $script reads evidence artifacts${NC}"
                VIOLATIONS=$((VIOLATIONS + 1))
            fi
        fi
    fi
done

if [[ $VIOLATIONS -eq 0 ]]; then
    echo -e "${GREEN}✓ No validation scripts read evidence artifacts${NC}"
fi

echo ""

# Check 2: Evidence artifacts must not be used in conditional logic
echo "Check 2: Evidence in conditional logic"
echo "--------------------------------------"

CONDITIONAL_VIOLATIONS=0

for script in "${VALIDATION_SCRIPTS[@]}"; do
    if [[ ! -f "$script" ]]; then
        continue
    fi

    # Check for evidence in if statements
    if grep -E "if.*out/evidence.*(summary|markers|perf)\.json" "$script" 2>/dev/null; then
        echo -e "${RED}❌ VIOLATION: $script uses evidence in conditional logic${NC}"
        CONDITIONAL_VIOLATIONS=$((CONDITIONAL_VIOLATIONS + 1))
    fi
done

if [[ $CONDITIONAL_VIOLATIONS -eq 0 ]]; then
    echo -e "${GREEN}✓ No evidence artifacts in conditional logic${NC}"
else
    VIOLATIONS=$((VIOLATIONS + CONDITIONAL_VIOLATIONS))
fi

echo ""

# Check 3: Evidence must not affect exit status
echo "Check 3: Evidence affecting exit status"
echo "--------------------------------------"

EXIT_VIOLATIONS=0

for script in "${VALIDATION_SCRIPTS[@]}"; do
    if [[ ! -f "$script" ]]; then
        continue
    fi

    # Check for evidence affecting exit
    if grep -E "exit.*out/evidence|out/evidence.*exit" "$script" 2>/dev/null; then
        echo -e "${RED}❌ VIOLATION: $script exit status depends on evidence${NC}"
        EXIT_VIOLATIONS=$((EXIT_VIOLATIONS + 1))
    fi
done

if [[ $EXIT_VIOLATIONS -eq 0 ]]; then
    echo -e "${GREEN}✓ Evidence does not affect exit status${NC}"
else
    VIOLATIONS=$((VIOLATIONS + EXIT_VIOLATIONS))
fi

echo ""

# Check 4: Dashboard must not write to validation inputs
echo "Check 4: Dashboard write isolation"
echo "--------------------------------------"

DASHBOARD_FILES=(
    "tools/dashboard/dashboard.js"
    "tools/dashboard/index.html"
)

DASHBOARD_VIOLATIONS=0

for file in "${DASHBOARD_FILES[@]}"; do
    if [[ ! -f "$file" ]]; then
        continue
    fi

    # Check for actual write operations (not just the word "write" in text)
    # Look for fetch with POST/PUT/DELETE methods or file write operations
    if grep -E "(fetch.*method.*['\"]POST|fetch.*method.*['\"]PUT|fetch.*method.*['\"]DELETE|XMLHttpRequest.*open.*POST|XMLHttpRequest.*open.*PUT)" "$file" 2>/dev/null; then
        echo -e "${RED}❌ VIOLATION: $file attempts to write to validation inputs${NC}"
        DASHBOARD_VIOLATIONS=$((DASHBOARD_VIOLATIONS + 1))
    fi
done

if [[ $DASHBOARD_VIOLATIONS -eq 0 ]]; then
    echo -e "${GREEN}✓ Dashboard is read-only${NC}"
else
    VIOLATIONS=$((VIOLATIONS + DASHBOARD_VIOLATIONS))
fi

echo ""

# Check 5: Evidence generation must happen after validation
echo "Check 5: Evidence generation timing"
echo "--------------------------------------"

# Check dev_loop.sh for proper ordering
if [[ -f "scripts/dev_loop.sh" ]]; then
    # Extract line numbers for validation and evidence generation
    VALIDATION_LINE=$(grep -n "Validation complete\|PASS\|FAIL" scripts/dev_loop.sh | head -1 | cut -d: -f1 || echo "0")
    EVIDENCE_LINE=$(grep -n "generate_evidence.sh" scripts/dev_loop.sh | head -1 | cut -d: -f1 || echo "0")

    if [[ $EVIDENCE_LINE -gt 0 ]] && [[ $VALIDATION_LINE -gt 0 ]]; then
        if [[ $EVIDENCE_LINE -lt $VALIDATION_LINE ]]; then
            echo -e "${RED}❌ VIOLATION: Evidence generation before validation${NC}"
            VIOLATIONS=$((VIOLATIONS + 1))
        else
            echo -e "${GREEN}✓ Evidence generation after validation${NC}"
        fi
    else
        echo -e "${YELLOW}⚠ Cannot verify evidence generation timing${NC}"
    fi
else
    echo -e "${YELLOW}⚠ dev_loop.sh not found${NC}"
fi

echo ""

# Summary
echo "========================================"
echo "Evidence Misuse Guard Summary"
echo "========================================"
echo ""

if [[ $VIOLATIONS -eq 0 ]]; then
    echo -e "${GREEN}✅ PASS: No evidence misuse detected${NC}"
    echo ""
    echo "Evidence integrity verified:"
    echo "  - Validation scripts do not read evidence"
    echo "  - Evidence not used in conditional logic"
    echo "  - Evidence does not affect exit status"
    echo "  - Dashboard is read-only"
    echo "  - Evidence generated after validation"
    echo ""
    exit 0
else
    echo -e "${RED}❌ FAIL: $VIOLATIONS evidence misuse violation(s) detected${NC}"
    echo ""
    echo "Evidence integrity compromised:"
    echo "  - Evidence artifacts used as validation input"
    echo "  - Violates R26 (Direct Observation Source Constraint)"
    echo "  - Violates R27 (Evidence State Isolation)"
    echo ""
    exit 1
fi

#!/usr/bin/env bash
# CI Gate: Execution Marker Validation Isolation
# 
# PURPOSE:
# Ensure sandbox marker validation code does NOT leak into production.
# 
# RULES:
# - execution_marker_validation.c is SANDBOX ONLY
# - execution_slot.c must NOT call marker validation (yet)
# - No phase17 naming in kernel code
# - Production code protection maintained

set -euo pipefail

RUN_ID=$(date -u +%Y%m%dT%H%M%SZ)-$(git rev-parse --short HEAD)-$$
EVIDENCE_DIR="out/evidence/run-$RUN_ID/gates/execution-marker-isolation"

mkdir -p "$EVIDENCE_DIR"

echo "== CI GATE EXECUTION MARKER ISOLATION =="
echo "run_id: $RUN_ID"

VIOLATIONS=()

# Check 1: execution_slot.c does NOT call marker validation
echo ""
echo "Check 1: execution_slot.c isolation"
if grep -q "execution_marker_validate" kernel/sys/execution_slot.c 2>/dev/null; then
    echo "  ❌ FAIL: execution_slot.c calls marker validation"
    VIOLATIONS+=("execution_slot.c:calls_marker_validation")
else
    echo "  ✅ PASS: execution_slot.c clean"
fi

# Check 2: No marker validation leakage outside sandbox
echo ""
echo "Check 2: Marker validation leakage check"
LEAK_FILES=$(grep -r "execution_marker_validate" kernel/ 2>/dev/null | \
    grep -v "execution_marker_validation.c" | \
    grep -v "execution_marker_validation.h" | \
    grep -v "\.o" | \
    grep -v "Binary file" | \
    cut -d: -f1 | sort -u || true)

if [ -n "$LEAK_FILES" ]; then
    echo "  ❌ FAIL: Marker validation leaked to:"
    echo "$LEAK_FILES" | while read -r file; do
        echo "    - $file"
        VIOLATIONS+=("leakage:$file")
    done
else
    echo "  ✅ PASS: No leakage detected"
fi

# Check 3: No phase17 naming in kernel code (except docs)
echo ""
echo "Check 3: Phase17 naming leak check"
PHASE17_LEAKS=$(grep -r "phase17\|PHASE17" kernel/ 2>/dev/null | \
    grep -v "docs/" | \
    grep -v ".o:" | \
    cut -d: -f1 | sort -u || true)

if [ -n "$PHASE17_LEAKS" ]; then
    echo "  ❌ FAIL: Phase17 naming found in:"
    echo "$PHASE17_LEAKS" | while read -r file; do
        echo "    - $file"
        VIOLATIONS+=("phase17_naming:$file")
    done
else
    echo "  ✅ PASS: No phase17 naming in kernel"
fi

# Check 4: execution_slot.c exists and protected
echo ""
echo "Check 4: Production code protection"
if [ ! -f "kernel/sys/execution_slot.c" ]; then
    echo "  ❌ FAIL: execution_slot.c missing"
    VIOLATIONS+=("execution_slot.c:missing")
else
    LINES=$(wc -l < kernel/sys/execution_slot.c)
    if [ "$LINES" -lt 1500 ]; then
        echo "  ❌ FAIL: execution_slot.c too small ($LINES lines)"
        VIOLATIONS+=("execution_slot.c:line_count_low:$LINES")
    else
        echo "  ✅ PASS: execution_slot.c protected ($LINES lines)"
    fi
fi

# Check 5: Marker validation files exist
echo ""
echo "Check 5: Sandbox files present"
if [ ! -f "kernel/include/execution_marker_validation.h" ]; then
    echo "  ❌ FAIL: Header missing"
    VIOLATIONS+=("header:missing")
else
    echo "  ✅ PASS: Header present"
fi

if [ ! -f "kernel/sys/execution_marker_validation.c" ]; then
    echo "  ❌ FAIL: Implementation missing"
    VIOLATIONS+=("implementation:missing")
else
    echo "  ✅ PASS: Implementation present"
fi

if [ ! -f "tests/unit/execution_marker_validation_test.c" ]; then
    echo "  ❌ FAIL: Test missing"
    VIOLATIONS+=("test:missing")
else
    echo "  ✅ PASS: Test present"
fi

# Generate report
VIOLATIONS_COUNT=${#VIOLATIONS[@]}

if [ "$VIOLATIONS_COUNT" -eq 0 ]; then
    VERDICT="PASS"
    VIOLATIONS_JSON="[]"
else
    VERDICT="FAIL"
    VIOLATIONS_JSON="[$(printf '"%s",' "${VIOLATIONS[@]}" | sed 's/,$//')]"
fi

cat > "$EVIDENCE_DIR/report.json" <<EOF
{
  "gate": "execution-marker-isolation",
  "verdict": "$VERDICT",
  "run_id": "$RUN_ID",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "commit": "$(git rev-parse HEAD)",
  "violations": $VIOLATIONS_JSON,
  "violations_count": $VIOLATIONS_COUNT,
  "checks": {
    "execution_slot_isolation": "$(grep -q "execution_marker_validate" kernel/sys/execution_slot.c 2>/dev/null && echo "fail" || echo "pass")",
    "no_leakage": "$([ -z "$LEAK_FILES" ] && echo "pass" || echo "fail")",
    "no_phase17_naming": "$([ -z "$PHASE17_LEAKS" ] && echo "pass" || echo "fail")",
    "production_protected": "$([ -f kernel/sys/execution_slot.c ] && [ $(wc -l < kernel/sys/execution_slot.c) -ge 1500 ] && echo "pass" || echo "fail")",
    "sandbox_files_present": "$([ -f kernel/include/execution_marker_validation.h ] && [ -f kernel/sys/execution_marker_validation.c ] && [ -f tests/unit/execution_marker_validation_test.c ] && echo "pass" || echo "fail")"
  },
  "meta": {
    "purpose": "prevent_sandbox_code_leaking_to_production",
    "scope": "execution_marker_validation_isolation"
  }
}
EOF

echo ""
if [ "$VERDICT" = "PASS" ]; then
    echo "✅ PASS: Execution Marker Isolation Gate"
    echo "Evidence: $EVIDENCE_DIR/report.json"
    exit 0
else
    echo "❌ FAIL: Execution Marker Isolation Gate"
    echo "Violations:"
    printf '  - %s\n' "${VIOLATIONS[@]}"
    echo "Evidence: $EVIDENCE_DIR/report.json"
    exit 1
fi

#!/bin/bash
# CI Gate: Execution Slot Integrity
# 
# Purpose: Prevent accidental overwrite of production execution_slot.c
# 
# This gate was added after a critical incident where prototype code
# accidentally overwrote 1910 lines of production kernel code.

set -e

RUN_ID=$(date -u +%Y%m%dT%H%M%SZ)-$(git rev-parse --short HEAD)-$$
EVIDENCE_DIR="out/evidence/run-$RUN_ID/gates/execution-slot-integrity"

mkdir -p "$EVIDENCE_DIR"

echo "== CI GATE EXECUTION SLOT INTEGRITY =="
echo "run_id: $RUN_ID"

EXECUTION_SLOT_C="kernel/sys/execution_slot.c"
EXECUTION_SLOT_H="kernel/include/execution_slot.h"

# Check files exist
if [ ! -f "$EXECUTION_SLOT_C" ]; then
    echo "ERROR: $EXECUTION_SLOT_C not found"
    exit 1
fi

if [ ! -f "$EXECUTION_SLOT_H" ]; then
    echo "ERROR: $EXECUTION_SLOT_H not found"
    exit 1
fi

# Line count check (production file should be substantial)
MIN_LINES_C=1500
MIN_LINES_H=100

LINES_C=$(wc -l < "$EXECUTION_SLOT_C")
LINES_H=$(wc -l < "$EXECUTION_SLOT_H")

echo "Line counts:"
echo "  $EXECUTION_SLOT_C: $LINES_C lines (minimum: $MIN_LINES_C)"
echo "  $EXECUTION_SLOT_H: $LINES_H lines (minimum: $MIN_LINES_H)"

if [ "$LINES_C" -lt "$MIN_LINES_C" ]; then
    echo "ERROR: $EXECUTION_SLOT_C has only $LINES_C lines (expected >= $MIN_LINES_C)"
    echo "This may indicate accidental overwrite with prototype code"
    exit 1
fi

if [ "$LINES_H" -lt "$MIN_LINES_H" ]; then
    echo "ERROR: $EXECUTION_SLOT_H has only $LINES_H lines (expected >= $MIN_LINES_H)"
    echo "This may indicate accidental overwrite with prototype code"
    exit 1
fi

# Critical marker check
CRITICAL_MARKERS=(
    "g_execution_slots"
    "execution_slot_prepare_result_locked"
    "AYKEN_BCIB_STUB_RESULT_VALUE_U64"
    "execution_slot_debugcon_write"
    "AYKEN_MAX_EXECUTION_SLOTS"
)

echo ""
echo "Checking critical markers in $EXECUTION_SLOT_C:"

MISSING_MARKERS=()
for marker in "${CRITICAL_MARKERS[@]}"; do
    if ! grep -q "$marker" "$EXECUTION_SLOT_C"; then
        echo "  ❌ MISSING: $marker"
        MISSING_MARKERS+=("$marker")
    else
        echo "  ✅ FOUND: $marker"
    fi
done

if [ ${#MISSING_MARKERS[@]} -gt 0 ]; then
    echo ""
    echo "ERROR: Critical markers missing from $EXECUTION_SLOT_C"
    echo "Missing markers: ${MISSING_MARKERS[*]}"
    echo "This indicates the file may have been overwritten"
    exit 1
fi

# Check for prototype indicators (should NOT be present)
PROTOTYPE_INDICATORS=(
    "malloc"
    "printf"
    "fprintf"
    "exit("
    "HELLO_BCIB_EXECUTION"
)

echo ""
echo "Checking for prototype code indicators (should NOT be present):"

FOUND_INDICATORS=()
for indicator in "${PROTOTYPE_INDICATORS[@]}"; do
    if grep -q "$indicator" "$EXECUTION_SLOT_C"; then
        echo "  ⚠️  FOUND: $indicator"
        FOUND_INDICATORS+=("$indicator")
    else
        echo "  ✅ NOT FOUND: $indicator"
    fi
done

if [ ${#FOUND_INDICATORS[@]} -gt 0 ]; then
    echo ""
    echo "WARNING: Prototype code indicators found in $EXECUTION_SLOT_C"
    echo "Found indicators: ${FOUND_INDICATORS[*]}"
    echo "This may indicate prototype code was merged into production"
    # Don't fail yet, but warn
fi

# Generate evidence report
cat > "$EVIDENCE_DIR/report.json" <<EOF
{
  "gate": "execution-slot-integrity",
  "verdict": "PASS",
  "run_id": "$RUN_ID",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "commit": "$(git rev-parse HEAD)",
  "violations": [],
  "violations_count": 0,
  "checks": {
    "line_count_c": {
      "actual": $LINES_C,
      "minimum": $MIN_LINES_C,
      "status": "PASS"
    },
    "line_count_h": {
      "actual": $LINES_H,
      "minimum": $MIN_LINES_H,
      "status": "PASS"
    },
    "critical_markers": {
      "expected": ${#CRITICAL_MARKERS[@]},
      "found": $((${#CRITICAL_MARKERS[@]} - ${#MISSING_MARKERS[@]})),
      "missing": [$(printf '"%s",' "${MISSING_MARKERS[@]}" | sed 's/,$//')]
    },
    "prototype_indicators": {
      "found": [$(printf '"%s",' "${FOUND_INDICATORS[@]}" | sed 's/,$//')]
    }
  },
  "meta": {
    "purpose": "prevent_production_code_overwrite",
    "incident_reference": "commit_b3e2aee7_overwrite_recovery"
  }
}
EOF

echo ""
echo "✅ PASS: Execution Slot Integrity Gate"
echo "Evidence: $EVIDENCE_DIR/report.json"

exit 0

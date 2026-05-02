#!/usr/bin/env bash
# validate_bug_conditions.sh - Validate bug conditions before and after fixes
# Usage: ./validate_bug_conditions.sh <ORIGINAL|FIXED> <document_path>

set -euo pipefail

MODE="${1:-}"
DOC_PATH="${2:-}"

if [[ -z "$MODE" || -z "$DOC_PATH" ]]; then
    echo "Usage: $0 <ORIGINAL|FIXED> <document_path>"
    exit 1
fi

if [[ ! -f "$DOC_PATH" ]]; then
    echo "ERROR: Document not found: $DOC_PATH"
    exit 1
fi

REPORT_DIR=".kiro/specs/abdf-contract-technical-corrections/reports"
mkdir -p "$REPORT_DIR"

REPORT_FILE="$REPORT_DIR/bug_condition_$(echo "$MODE" | tr '[:upper:]' '[:lower:]')_$(date +%Y%m%d_%H%M%S).md"

echo "# Bug Condition Validation Report - $MODE" > "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "**Date**: $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$REPORT_FILE"
echo "**Mode**: $MODE" >> "$REPORT_FILE"
echo "**Document**: $DOC_PATH" >> "$REPORT_FILE"
echo "**Document Hash**: $(shasum -a 256 "$DOC_PATH" | cut -d' ' -f1)" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

PASS_COUNT=0
FAIL_COUNT=0

# Bug 1: String Pool null-termination
echo "## Bug 1: String Pool Representation" >> "$REPORT_FILE"
if grep -q "UTF-8 Data (null-terminated)" "$DOC_PATH"; then
    echo "- ❌ FOUND: null-terminated representation (BUG PRESENT)" >> "$REPORT_FILE"
    if [[ "$MODE" == "ORIGINAL" ]]; then
        ((PASS_COUNT++))
        echo "- ✅ Expected for ORIGINAL (bug should exist)" >> "$REPORT_FILE"
    else
        ((FAIL_COUNT++))
        echo "- 🔴 FAIL: Bug still present in FIXED document" >> "$REPORT_FILE"
    fi
else
    echo "- ✅ NOT FOUND: null-terminated representation" >> "$REPORT_FILE"
    if [[ "$MODE" == "FIXED" ]]; then
        if grep -q "UTF-8 Data (offset + length)" "$DOC_PATH"; then
            ((PASS_COUNT++))
            echo "- ✅ PASS: offset+length representation present" >> "$REPORT_FILE"
        else
            ((FAIL_COUNT++))
            echo "- 🔴 FAIL: Neither null-terminated nor offset+length found" >> "$REPORT_FILE"
        fi
    else
        ((FAIL_COUNT++))
        echo "- 🔴 FAIL: Bug not present in ORIGINAL (cannot prove bug existed)" >> "$REPORT_FILE"
    fi
fi
echo "" >> "$REPORT_FILE"

# Bug 2: Checksum scope undefined
echo "## Bug 2: Checksum Scope Definition" >> "$REPORT_FILE"
if grep -F "checksum: u64, // XXH3 hash" "$DOC_PATH" | grep -v "XXH3-64" > /dev/null 2>&1; then
    echo "- ❌ FOUND: Undefined checksum scope (BUG PRESENT)" >> "$REPORT_FILE"
    if [[ "$MODE" == "ORIGINAL" ]]; then
        ((PASS_COUNT++))
        echo "- ✅ Expected for ORIGINAL (bug should exist)" >> "$REPORT_FILE"
    else
        ((FAIL_COUNT++))
        echo "- 🔴 FAIL: Bug still present in FIXED document" >> "$REPORT_FILE"
    fi
else
    echo "- ✅ NOT FOUND: Undefined checksum scope" >> "$REPORT_FILE"
    if [[ "$MODE" == "FIXED" ]]; then
        if grep -F "XXH3-64 hash of bytes [64..total_size)" "$DOC_PATH" > /dev/null 2>&1; then
            ((PASS_COUNT++))
            echo "- ✅ PASS: Checksum scope defined" >> "$REPORT_FILE"
        else
            ((FAIL_COUNT++))
            echo "- 🔴 FAIL: Checksum scope still undefined" >> "$REPORT_FILE"
        fi
    else
        ((FAIL_COUNT++))
        echo "- 🔴 FAIL: Bug not present in ORIGINAL (cannot prove bug existed)" >> "$REPORT_FILE"
    fi
fi
echo "" >> "$REPORT_FILE"

# Bug 3: GPU directly mappable without fallback
echo "## Bug 3: GPU Zero-Copy Overpromise" >> "$REPORT_FILE"
if grep -F "GPU buffer data is **directly mappable** to GPU memory." "$DOC_PATH"; then
    echo "- ❌ FOUND: GPU directly mappable without fallback (BUG PRESENT)" >> "$REPORT_FILE"
    if [[ "$MODE" == "ORIGINAL" ]]; then
        ((PASS_COUNT++))
        echo "- ✅ Expected for ORIGINAL (bug should exist)" >> "$REPORT_FILE"
    else
        ((FAIL_COUNT++))
        echo "- 🔴 FAIL: Bug still present in FIXED document" >> "$REPORT_FILE"
    fi
else
    echo "- ✅ NOT FOUND: GPU directly mappable without fallback" >> "$REPORT_FILE"
    if [[ "$MODE" == "FIXED" ]]; then
        if grep -F "designed for **direct mapping to GPU memory** as an optimization target" "$DOC_PATH"; then
            ((PASS_COUNT++))
            echo "- ✅ PASS: GPU mapping as optimization target with fallback" >> "$REPORT_FILE"
        else
            ((FAIL_COUNT++))
            echo "- 🔴 FAIL: GPU mapping clarification not found" >> "$REPORT_FILE"
        fi
    else
        ((FAIL_COUNT++))
        echo "- 🔴 FAIL: Bug not present in ORIGINAL (cannot prove bug existed)" >> "$REPORT_FILE"
    fi
fi
echo "" >> "$REPORT_FILE"

# Bug 4: Immutability scope ambiguous
echo "## Bug 4: Immutability Scope Ambiguity" >> "$REPORT_FILE"
if grep -F "This contract is **immutable** in Phase 1" "$DOC_PATH" && ! grep -q "Immutable Core Contract" "$DOC_PATH"; then
    echo "- ❌ FOUND: Ambiguous immutability scope (BUG PRESENT)" >> "$REPORT_FILE"
    if [[ "$MODE" == "ORIGINAL" ]]; then
        ((PASS_COUNT++))
        echo "- ✅ Expected for ORIGINAL (bug should exist)" >> "$REPORT_FILE"
    else
        ((FAIL_COUNT++))
        echo "- 🔴 FAIL: Bug still present in FIXED document" >> "$REPORT_FILE"
    fi
else
    echo "- ✅ NOT FOUND: Ambiguous immutability scope" >> "$REPORT_FILE"
    if [[ "$MODE" == "FIXED" ]]; then
        if grep -q "Immutable Core Contract" "$DOC_PATH" && grep -q "Versioned Extensions" "$DOC_PATH"; then
            ((PASS_COUNT++))
            echo "- ✅ PASS: Immutability scope separated (core vs extensions)" >> "$REPORT_FILE"
        else
            ((FAIL_COUNT++))
            echo "- 🔴 FAIL: Immutability scope separation not found" >> "$REPORT_FILE"
        fi
    else
        ((FAIL_COUNT++))
        echo "- 🔴 FAIL: Bug not present in ORIGINAL (cannot prove bug existed)" >> "$REPORT_FILE"
    fi
fi
echo "" >> "$REPORT_FILE"

# Bug 5: Missing static assertions
echo "## Bug 5: SegmentEntry Static Assertions Missing" >> "$REPORT_FILE"
if ! grep -F "Compile-Time Validation" "$DOC_PATH" > /dev/null 2>&1; then
    echo "- ❌ NOT FOUND: Compile-Time Validation section (BUG PRESENT)" >> "$REPORT_FILE"
    if [[ "$MODE" == "ORIGINAL" ]]; then
        ((PASS_COUNT++))
        echo "- ✅ Expected for ORIGINAL (bug should exist)" >> "$REPORT_FILE"
    else
        ((FAIL_COUNT++))
        echo "- 🔴 FAIL: Bug still present in FIXED document" >> "$REPORT_FILE"
    fi
else
    echo "- ✅ FOUND: Compile-Time Validation section" >> "$REPORT_FILE"
    if [[ "$MODE" == "FIXED" ]]; then
        if grep -F "const _: () = assert!(core::mem::size_of::<SegmentEntry>() == 32)" "$DOC_PATH" > /dev/null 2>&1; then
            ((PASS_COUNT++))
            echo "- ✅ PASS: Static assertions present" >> "$REPORT_FILE"
        else
            ((FAIL_COUNT++))
            echo "- 🔴 FAIL: Static assertions not found" >> "$REPORT_FILE"
        fi
    else
        ((FAIL_COUNT++))
        echo "- 🔴 FAIL: Bug not present in ORIGINAL (cannot prove bug existed)" >> "$REPORT_FILE"
    fi
fi
echo "" >> "$REPORT_FILE"

# Bug 6: Alignment conflation
echo "## Bug 6: Alignment Requirements Conflation" >> "$REPORT_FILE"
if grep -q "All offsets MUST be 64B aligned" "$DOC_PATH" && ! grep -q "ABDF segment start offsets MUST be 64B aligned" "$DOC_PATH"; then
    echo "- ❌ FOUND: Alignment conflation (BUG PRESENT)" >> "$REPORT_FILE"
    if [[ "$MODE" == "ORIGINAL" ]]; then
        ((PASS_COUNT++))
        echo "- ✅ Expected for ORIGINAL (bug should exist)" >> "$REPORT_FILE"
    else
        ((FAIL_COUNT++))
        echo "- 🔴 FAIL: Bug still present in FIXED document" >> "$REPORT_FILE"
    fi
else
    echo "- ✅ NOT FOUND: Alignment conflation" >> "$REPORT_FILE"
    if [[ "$MODE" == "FIXED" ]]; then
        if grep -q "ABDF segment start offsets MUST be 64B aligned" "$DOC_PATH" && grep -q "mmap base alignment is an OS/runtime concern" "$DOC_PATH"; then
            ((PASS_COUNT++))
            echo "- ✅ PASS: Alignment requirements separated" >> "$REPORT_FILE"
        else
            ((FAIL_COUNT++))
            echo "- 🔴 FAIL: Alignment separation not found" >> "$REPORT_FILE"
        fi
    else
        ((FAIL_COUNT++))
        echo "- 🔴 FAIL: Bug not present in ORIGINAL (cannot prove bug existed)" >> "$REPORT_FILE"
    fi
fi
echo "" >> "$REPORT_FILE"

# Bug 7: ABDF-BCIB boundary contract missing
echo "## Bug 7: ABDF-BCIB Boundary Contract Missing" >> "$REPORT_FILE"
if ! grep -q "ABDF-BCIB Integration Contract" "$DOC_PATH"; then
    echo "- ❌ NOT FOUND: ABDF-BCIB Integration Contract (BUG PRESENT)" >> "$REPORT_FILE"
    if [[ "$MODE" == "ORIGINAL" ]]; then
        ((PASS_COUNT++))
        echo "- ✅ Expected for ORIGINAL (bug should exist)" >> "$REPORT_FILE"
    else
        ((FAIL_COUNT++))
        echo "- 🔴 FAIL: Bug still present in FIXED document" >> "$REPORT_FILE"
    fi
else
    echo "- ✅ FOUND: ABDF-BCIB Integration Contract" >> "$REPORT_FILE"
    if [[ "$MODE" == "FIXED" ]]; then
        if grep -q "Pointer-Free Guarantee" "$DOC_PATH" && grep -q "Stable Identifiers Only" "$DOC_PATH"; then
            ((PASS_COUNT++))
            echo "- ✅ PASS: ABDF-BCIB boundary contract present" >> "$REPORT_FILE"
        else
            ((FAIL_COUNT++))
            echo "- 🔴 FAIL: ABDF-BCIB boundary contract incomplete" >> "$REPORT_FILE"
        fi
    else
        ((FAIL_COUNT++))
        echo "- 🔴 FAIL: Bug not present in ORIGINAL (cannot prove bug existed)" >> "$REPORT_FILE"
    fi
fi
echo "" >> "$REPORT_FILE"

# Summary
echo "## Summary" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "- **Total Checks**: 7" >> "$REPORT_FILE"
echo "- **Passed**: $PASS_COUNT" >> "$REPORT_FILE"
echo "- **Failed**: $FAIL_COUNT" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

if [[ "$MODE" == "ORIGINAL" ]]; then
    if [[ $PASS_COUNT -eq 7 ]]; then
        echo "✅ **RESULT**: All 7 bugs PRESENT in ORIGINAL (expected)" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        echo "Bug proof successful. Proceed to Task 2 (baseline capture)." >> "$REPORT_FILE"
        exit 0
    else
        echo "❌ **RESULT**: FAIL - Not all bugs present in ORIGINAL" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        echo "Cannot prove bugs existed. Do not proceed to fixes." >> "$REPORT_FILE"
        exit 1
    fi
else
    if [[ $PASS_COUNT -eq 7 ]]; then
        echo "✅ **RESULT**: All 7 bugs FIXED (expected)" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        echo "Bug fixes verified. Proceed to Task 3.9 (preservation validation)." >> "$REPORT_FILE"
        exit 0
    else
        echo "❌ **RESULT**: FAIL - Not all bugs fixed" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        echo "Fixes incomplete. Do not proceed to preservation validation." >> "$REPORT_FILE"
        exit 1
    fi
fi

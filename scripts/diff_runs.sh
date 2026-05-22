#!/usr/bin/env bash
# Run Diff Engine
#
# Purpose: Compare two validation runs for diagnostic analysis
# Authority: ZERO - purely observational, no validation authority
#
# Maintainer: Kenan AY — System Architect

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Usage
if [[ $# -lt 2 ]]; then
    echo "Usage: $0 <run-id-1> <run-id-2>"
    echo ""
    echo "Compare two validation runs for diagnostic analysis."
    echo ""
    echo "Example:"
    echo "  $0 run-20260508T120000Z-abc123-1234 run-20260508T130000Z-def456-5678"
    echo ""
    exit 1
fi

RUN1="$1"
RUN2="$2"

EVIDENCE_BASE="out/evidence"
RUN1_DIR="${EVIDENCE_BASE}/${RUN1}"
RUN2_DIR="${EVIDENCE_BASE}/${RUN2}"

# Verify runs exist
if [[ ! -d "$RUN1_DIR" ]]; then
    echo -e "${RED}Error: Run 1 not found: $RUN1${NC}"
    exit 1
fi

if [[ ! -d "$RUN2_DIR" ]]; then
    echo -e "${RED}Error: Run 2 not found: $RUN2${NC}"
    exit 1
fi

echo "========================================"
echo "Run Diff Engine"
echo "========================================"
echo ""
echo -e "${BLUE}Run 1:${NC} $RUN1"
echo -e "${BLUE}Run 2:${NC} $RUN2"
echo ""

# Load run data
load_json() {
    local file="$1"
    if [[ -f "$file" ]]; then
        cat "$file"
    else
        echo "{}"
    fi
}

RUN1_META=$(load_json "${RUN1_DIR}/meta/run.json")
RUN2_META=$(load_json "${RUN2_DIR}/meta/run.json")

RUN1_SUMMARY=$(load_json "${RUN1_DIR}/reports/summary.json")
RUN2_SUMMARY=$(load_json "${RUN2_DIR}/reports/summary.json")

RUN1_MARKERS=$(load_json "${RUN1_DIR}/reports/markers.json")
RUN2_MARKERS=$(load_json "${RUN1_DIR}/reports/markers.json")

RUN1_PERF=$(load_json "${RUN1_DIR}/reports/perf.json")
RUN2_PERF=$(load_json "${RUN2_DIR}/reports/perf.json")

# Compare metadata
echo "========================================"
echo "Metadata Comparison"
echo "========================================"
echo ""

compare_field() {
    local label="$1"
    local val1="$2"
    local val2="$3"

    printf "%-20s" "$label:"
    if [[ "$val1" == "$val2" ]]; then
        echo -e "${GREEN}✓ Same${NC} ($val1)"
    else
        echo -e "${YELLOW}△ Different${NC}"
        echo "  Run 1: $val1"
        echo "  Run 2: $val2"
    fi
}

RUN1_TIME=$(echo "$RUN1_META" | jq -r '.time_utc // "unknown"')
RUN2_TIME=$(echo "$RUN2_META" | jq -r '.time_utc // "unknown"')
compare_field "Timestamp" "$RUN1_TIME" "$RUN2_TIME"

RUN1_SHA=$(echo "$RUN1_META" | jq -r '.git_sha // "unknown"')
RUN2_SHA=$(echo "$RUN2_META" | jq -r '.git_sha // "unknown"')
compare_field "Git SHA" "$RUN1_SHA" "$RUN2_SHA"

RUN1_BRANCH=$(echo "$RUN1_META" | jq -r '.git_branch // "unknown"')
RUN2_BRANCH=$(echo "$RUN2_META" | jq -r '.git_branch // "unknown"')
compare_field "Git Branch" "$RUN1_BRANCH" "$RUN2_BRANCH"

RUN1_SOURCE=$(echo "$RUN1_META" | jq -r '.source // "unknown"')
RUN2_SOURCE=$(echo "$RUN2_META" | jq -r '.source // "unknown"')
compare_field "Source" "$RUN1_SOURCE" "$RUN2_SOURCE"

echo ""

# Compare boot status
echo "========================================"
echo "Boot Status Comparison"
echo "========================================"
echo ""

RUN1_BOOT=$(echo "$RUN1_SUMMARY" | jq -r '.status.boot // .boot // "UNKNOWN"')
RUN2_BOOT=$(echo "$RUN2_SUMMARY" | jq -r '.status.boot // .boot // "UNKNOWN"')

printf "%-20s" "Boot Status:"
if [[ "$RUN1_BOOT" == "$RUN2_BOOT" ]]; then
    if [[ "$RUN1_BOOT" == "PASS" ]]; then
        echo -e "${GREEN}✓ Both PASS${NC}"
    elif [[ "$RUN1_BOOT" == "FAIL" ]]; then
        echo -e "${RED}✗ Both FAIL${NC}"
    else
        echo -e "${YELLOW}? Both UNKNOWN${NC}"
    fi
else
    echo -e "${YELLOW}△ Different${NC}"
    echo "  Run 1: $RUN1_BOOT"
    echo "  Run 2: $RUN2_BOOT"

    # Show reasons if available
    RUN1_REASON=$(echo "$RUN1_SUMMARY" | jq -r '.status.reason // "No reason provided"')
    RUN2_REASON=$(echo "$RUN2_SUMMARY" | jq -r '.status.reason // "No reason provided"')
    echo "  Run 1 Reason: $RUN1_REASON"
    echo "  Run 2 Reason: $RUN2_REASON"
fi

echo ""

# Compare markers
echo "========================================"
echo "Marker Comparison"
echo "========================================"
echo ""

compare_marker() {
    local marker="$1"
    local val1="$2"
    local val2="$3"

    printf "%-20s" "$marker:"
    if [[ "$val1" == "$val2" ]]; then
        if [[ "$val1" == "true" ]]; then
            echo -e "${GREEN}✓ Both present${NC}"
        else
            echo -e "${RED}✗ Both absent${NC}"
        fi
    else
        echo -e "${YELLOW}△ Different${NC}"
        if [[ "$val1" == "true" ]]; then
            echo "  Run 1: ✓ Present"
        else
            echo "  Run 1: ✗ Absent"
        fi
        if [[ "$val2" == "true" ]]; then
            echo "  Run 2: ✓ Present"
        else
            echo "  Run 2: ✗ Absent"
        fi
    fi
}

RUN1_EARLY=$(echo "$RUN1_MARKERS" | jq -r '.EARLY_BOOT_OK // false')
RUN2_EARLY=$(echo "$RUN2_MARKERS" | jq -r '.EARLY_BOOT_OK // false')
compare_marker "EARLY_BOOT_OK" "$RUN1_EARLY" "$RUN2_EARLY"

RUN1_LATE=$(echo "$RUN1_MARKERS" | jq -r '.LATE_INIT_END // false')
RUN2_LATE=$(echo "$RUN2_MARKERS" | jq -r '.LATE_INIT_END // false')
compare_marker "LATE_INIT_END" "$RUN1_LATE" "$RUN2_LATE"

RUN1_BOOT_OK=$(echo "$RUN1_MARKERS" | jq -r '.BOOT_OK // false')
RUN2_BOOT_OK=$(echo "$RUN2_MARKERS" | jq -r '.BOOT_OK // false')
compare_marker "BOOT_OK" "$RUN1_BOOT_OK" "$RUN2_BOOT_OK"

RUN1_FAIL_CLOSED=$(echo "$RUN1_MARKERS" | jq -r '.FAIL_CLOSED // false')
RUN2_FAIL_CLOSED=$(echo "$RUN2_MARKERS" | jq -r '.FAIL_CLOSED // false')
compare_marker "FAIL_CLOSED" "$RUN1_FAIL_CLOSED" "$RUN2_FAIL_CLOSED"

echo ""

# Compare performance
echo "========================================"
echo "Performance Comparison"
echo "========================================"
echo ""

RUN1_PERF_VAL=$(echo "$RUN1_PERF" | jq -r '.value // .boot_time_proxy // 0')
RUN2_PERF_VAL=$(echo "$RUN2_PERF" | jq -r '.value // .boot_time_proxy // 0')

printf "%-20s" "Performance Value:"
if [[ "$RUN1_PERF_VAL" == "$RUN2_PERF_VAL" ]]; then
    echo -e "${GREEN}✓ Same${NC} ($RUN1_PERF_VAL)"
else
    DIFF=$((RUN2_PERF_VAL - RUN1_PERF_VAL))
    PERCENT=0
    if [[ $RUN1_PERF_VAL -gt 0 ]]; then
        PERCENT=$(awk "BEGIN {printf \"%.1f\", ($DIFF / $RUN1_PERF_VAL) * 100}")
    fi

    echo -e "${YELLOW}△ Different${NC}"
    echo "  Run 1: $RUN1_PERF_VAL"
    echo "  Run 2: $RUN2_PERF_VAL"
    echo "  Delta: $DIFF ($PERCENT%)"

    if [[ $DIFF -gt 0 ]]; then
        echo -e "  ${YELLOW}⚠ Run 2 slower${NC}"
    else
        echo -e "  ${GREEN}✓ Run 2 faster${NC}"
    fi
fi

echo ""

# Log diff summary
echo "========================================"
echo "Log Diff Summary"
echo "========================================"
echo ""

if [[ -f "${RUN1_DIR}/logs/boot.log" ]] && [[ -f "${RUN2_DIR}/logs/boot.log" ]]; then
    RUN1_LINES=$(wc -l < "${RUN1_DIR}/logs/boot.log")
    RUN2_LINES=$(wc -l < "${RUN2_DIR}/logs/boot.log")

    echo "Log line count:"
    echo "  Run 1: $RUN1_LINES lines"
    echo "  Run 2: $RUN2_LINES lines"
    echo "  Delta: $((RUN2_LINES - RUN1_LINES)) lines"
    echo ""

    # Show first difference
    echo "First difference:"
    diff -u "${RUN1_DIR}/logs/boot.log" "${RUN2_DIR}/logs/boot.log" | head -20 || true
else
    echo -e "${YELLOW}⚠ Boot logs not available for comparison${NC}"
fi

echo ""

# Disclaimer
echo "========================================"
echo "Disclaimer"
echo "========================================"
echo ""
echo -e "${YELLOW}⚠ This diff is for diagnostic purposes only.${NC}"
echo ""
echo "Evidence artifacts are non-authoritative and do not affect"
echo "validation decisions. All validation uses raw boot logs only."
echo ""
echo "Maintainer: Kenan AY — System Architect"
echo ""

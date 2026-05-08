#!/bin/bash
# Oracle Script: Deterministic Validation Check
# Author: Kenan AY — System Architect
# Returns: 0 (PASS) or non-zero (FAIL)
#
# Purpose: Provides a deterministic validation check for git bisect.
# This script is used by find_regression.sh to automatically identify
# which commit broke the system.
#
# Output Format:
#   [ORACLE][PASS] - Validation succeeded
#   [ORACLE][FAIL] REASON=<reason> - Validation failed
#
# Usage:
#   ./scripts/oracle.sh
#
# Exit Codes:
#   0 = PASS (system validates successfully)
#   1 = FAIL (validation failed)
#   2 = ERROR (script error, not validation failure)

set -euo pipefail

# Run smoke test (fast validation)
# For more thorough validation, change to: ./scripts/dev_loop.sh contract

set +e
validation_output=$(./scripts/dev_loop.sh smoke 2>&1)
validation_status=$?
set -e

if [ "$validation_status" -eq 0 ]; then
    echo "[ORACLE][PASS]"
    exit 0
else
    # Extract failure reason from validation output
    reason="unknown"
    
    if echo "$validation_output" | grep -q "Build failed"; then
        reason="build_failure"
    elif echo "$validation_output" | grep -q "Boot timeout"; then
        reason="boot_timeout"
    elif echo "$validation_output" | grep -q "Missing marker"; then
        reason="missing_marker"
    elif echo "$validation_output" | grep -q "Marker sequence"; then
        reason="marker_sequence_violation"
    elif echo "$validation_output" | grep -q "Test failed"; then
        reason="test_failure"
    fi
    
    echo "[ORACLE][FAIL] REASON=$reason"
    exit 1
fi

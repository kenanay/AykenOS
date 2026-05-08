#!/bin/bash
# Automated Regression Finder
# Author: Kenan AY — System Architect
# Uses git bisect to find the first commit that broke the system
#
# Purpose: Automatically identifies which commit introduced a regression
# by binary searching through commit history using the oracle script.
#
# Usage:
#   ./scripts/find_regression.sh <last-known-good-commit> [bad-commit=HEAD]
#
# Example:
#   ./scripts/find_regression.sh a1b2c3d
#   ./scripts/find_regression.sh a1b2c3d HEAD
#
# Output:
#   - Bisect log showing tested commits
#   - First bad commit that caused regression
#   - Individual test logs in out/logs/bisect/<commit>.log

set -euo pipefail

GOOD_COMMIT="${1:-}"
BAD_COMMIT="${2:-HEAD}"

if [ -z "$GOOD_COMMIT" ]; then
    echo "Usage: $0 <last-known-good-commit> [bad-commit=HEAD]"
    echo ""
    echo "Example:"
    echo "  $0 a1b2c3d        # Find regression between a1b2c3d and HEAD"
    echo "  $0 a1b2c3d HEAD   # Same as above (explicit)"
    echo ""
    exit 2
fi

LOG_DIR="out/logs/bisect"
mkdir -p "$LOG_DIR"

echo "=========================================="
echo "Automated Regression Finder"
echo "=========================================="
echo ""
echo "Good commit: $GOOD_COMMIT"
echo "Bad commit:  $BAD_COMMIT"
echo ""
echo "This will use git bisect to find the first commit that broke the system."
echo "Each commit will be tested using: ./scripts/oracle.sh"
echo ""

# Verify commits exist
if ! git rev-parse "$GOOD_COMMIT" >/dev/null 2>&1; then
    echo "❌ ERROR: Good commit '$GOOD_COMMIT' not found"
    exit 2
fi

if ! git rev-parse "$BAD_COMMIT" >/dev/null 2>&1; then
    echo "❌ ERROR: Bad commit '$BAD_COMMIT' not found"
    exit 2
fi

# Start bisect
echo "[1/3] Starting git bisect..."
git bisect start "$BAD_COMMIT" "$GOOD_COMMIT"

# Define oracle runner function
run_oracle() {
    local commit
    commit=$(git rev-parse --short HEAD)
    local log="$LOG_DIR/$commit.log"
    
    echo "[TEST] Testing commit $commit..."
    
    # Run oracle and capture result
    set +e
    ./scripts/oracle.sh >"$log" 2>&1
    status=$?
    set -e
    
    if [ "$status" -eq 0 ]; then
        echo "       ✅ PASS"
        return 0
    else
        echo "       ❌ FAIL"
        return 1
    fi
}

# Export function for git bisect run
export -f run_oracle
export LOG_DIR

# Run bisect
echo ""
echo "[2/3] Running bisect (this may take a few minutes)..."
echo ""

# Use bash -c to invoke exported function (macOS compatibility)
git bisect run bash -c 'run_oracle'

echo ""
echo "[3/3] Bisect complete!"
echo ""

# Display results
echo "=========================================="
echo "RESULT"
echo "=========================================="
echo ""

# Show bisect log
echo "Bisect log:"
git bisect log || true
echo ""

# Show first bad commit
echo "First bad commit:"
git bisect visualize --oneline -1 || git log --oneline -1 "$(git bisect view)" 2>/dev/null || true
echo ""

# Reset bisect
git bisect reset

echo "=========================================="
echo "Individual test logs saved to: $LOG_DIR"
echo "=========================================="
echo ""
echo "To view a specific commit's log:"
echo "  cat $LOG_DIR/<commit>.log"
echo ""

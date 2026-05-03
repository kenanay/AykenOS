#!/usr/bin/env bash
# Author: Kenan AY
# Role: Developer / Architect / Designer / Implementer
#
# Ayken - Naming Convention Enforcement
#
# CRITICAL RULE:
# - "ayken" is the canonical system identifier
# - "aykenos" is FORBIDDEN in new code
# - "phase-*" naming is FORBIDDEN in new file/directory structures
#
# This script scans modified files and FAILS if:
# - New usage of "aykenos" is detected
# - New usage of "phase-*" in file paths is detected
#
# Legacy usage is allowed but marked as deprecated.

set -euo pipefail

echo "== CHECK: Naming Convention Compliance =="

ROOT_DIR="$(git rev-parse --show-toplevel)"
cd "$ROOT_DIR"

FAIL=0
DEPRECATED_COUNT=0

# Get modified files (compared to main/master branch)
if git rev-parse --verify main >/dev/null 2>&1; then
  BASE_BRANCH="main"
elif git rev-parse --verify master >/dev/null 2>&1; then
  BASE_BRANCH="master"
else
  echo "⚠ Warning: No main/master branch found, checking all files"
  BASE_BRANCH=""
fi

if [ -n "$BASE_BRANCH" ]; then
  MODIFIED_FILES=$(git diff --name-only "$BASE_BRANCH"...HEAD 2>/dev/null || git diff --name-only --cached 2>/dev/null || true)
else
  MODIFIED_FILES=$(git ls-files 2>/dev/null || find . -type f -not -path '*/\.git/*' -not -path '*/out/*' -not -path '*/node_modules/*')
fi

if [ -z "$MODIFIED_FILES" ]; then
  echo "✔ No modified files to check"
  exit 0
fi

echo "Checking modified files for naming violations..."

# Check for "aykenos" in new code
echo ""
echo "Checking for forbidden term: 'aykenos'..."

while IFS= read -r file; do
  [ -f "$file" ] || continue
  
  # Skip binary files, images, and generated files
  case "$file" in
    *.png|*.jpg|*.jpeg|*.gif|*.pdf|*.bin|*.o|*.elf|out/*|node_modules/*|.git/*)
      continue
      ;;
  esac
  
  matches=$(grep -nHi "aykenos" "$file" 2>/dev/null || true)
  
  if [ -n "$matches" ]; then
    echo "❌ VIOLATION: 'aykenos' found in: $file"
    echo "$matches"
    FAIL=1
  fi
done <<< "$MODIFIED_FILES"

# Check for "phase-*" in file paths
echo ""
echo "Checking for forbidden naming pattern: 'phase-*' in paths..."

while IFS= read -r file; do
  if echo "$file" | grep -qE "phase-[0-9]"; then
    echo "❌ VIOLATION: 'phase-*' naming in path: $file"
    FAIL=1
  fi
done <<< "$MODIFIED_FILES"

# Check legacy usage (informational)
echo ""
echo "Checking for deprecated legacy usage..."

LEGACY_FILES=$(git ls-files | grep -E "aykenos|phase-" || true)

if [ -n "$LEGACY_FILES" ]; then
  DEPRECATED_COUNT=$(echo "$LEGACY_FILES" | wc -l | tr -d ' ')
  echo "⚠ Deprecated: $DEPRECATED_COUNT files with legacy naming"
  echo "$LEGACY_FILES" | head -n 10
  if [ "$DEPRECATED_COUNT" -gt 10 ]; then
    echo "... and $((DEPRECATED_COUNT - 10)) more"
  fi
fi

# Verify canonical usage
echo ""
echo "Verifying canonical identifier usage..."

AYKEN_USAGE=$(grep -RIn "ayken" scripts tools .kiro 2>/dev/null | grep -v "aykenos" | wc -l | tr -d ' ')

echo "✔ Canonical 'ayken' usage: $AYKEN_USAGE occurrences"

# Final decision
if [ "$FAIL" -ne 0 ]; then
  echo ""
  echo "🚨 CRITICAL FAILURE: Naming convention violated"
  echo ""
  echo "Rules:"
  echo "  - Use 'ayken' (canonical identifier)"
  echo "  - Do NOT use 'aykenos' in new code"
  echo "  - Do NOT use 'phase-*' in new file/directory names"
  echo ""
  echo "Fix:"
  echo "  - Replace 'aykenos' with 'ayken'"
  echo "  - Rename files/directories to remove 'phase-*' pattern"
  echo ""
  echo "Constitutional Reference:"
  echo "  See .kiro/specs/dev-loop-boot-monitoring/DEV_LOOP_CONSTITUTION.md"
  echo "  Section 10: Naming Law"
  echo ""
  echo "Requirements Reference:"
  echo "  Requirement 25: Naming Convention Enforcement"
  echo "  Requirement 30: Naming Enforcement Scope"
  echo ""
  exit 1
fi

echo ""
echo "✅ PASS: Naming convention compliance verified"

if [ "$DEPRECATED_COUNT" -gt 0 ]; then
  echo "⚠ Note: $DEPRECATED_COUNT legacy files remain (allowed but deprecated)"
fi

exit 0

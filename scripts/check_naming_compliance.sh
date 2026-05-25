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
FORBIDDEN_PATTERN="ayken""os"
CANONICAL_EVIDENCE_DIR="${NAMING_COMPLIANCE_EVIDENCE_DIR:-${ROOT_DIR}/out/evidence/naming-compliance-standalone}"

# The canonical gate resolves PR/push base-vs-head ranges in clean CI
# checkouts and applies the reviewed scope/allowlist contract. Keep this
# legacy entry point for local staged/untracked feedback only.
echo "Running canonical diff-aware naming gate..."
if ! ./scripts/ci/check_naming_convention.sh --evidence-dir "$CANONICAL_EVIDENCE_DIR"; then
  echo "CRITICAL FAILURE: canonical naming convention gate rejected this change"
  exit 1
fi

TRACKED_FILES=$(
  {
    git diff --name-only HEAD 2>/dev/null || true
    git diff --name-only --cached 2>/dev/null || true
  } | sort -u
)
UNTRACKED_FILES=$(git ls-files --others --exclude-standard 2>/dev/null || true)
MODIFIED_FILES=$(printf '%s\n%s\n' "$TRACKED_FILES" "$UNTRACKED_FILES" | sed '/^$/d' | sort -u)

if [ -z "$MODIFIED_FILES" ]; then
  echo "✔ No modified files to check"
  exit 0
fi

echo "Checking modified files for naming violations..."

is_skipped_file() {
  case "$1" in
    *.png|*.jpg|*.jpeg|*.gif|*.pdf|*.bin|*.o|*.elf|out/*|node_modules/*|.git/*)
      return 0
      ;;
  esac
  return 1
}

is_untracked_file() {
  printf '%s\n' "$UNTRACKED_FILES" | grep -Fxq "$1"
}

# Check for "aykenos" in new code
echo ""
echo "Checking for forbidden term: 'aykenos'..."

while IFS= read -r file; do
  [ -f "$file" ] || continue
  
  if is_skipped_file "$file"; then
    continue
  fi
  
  if is_untracked_file "$file"; then
    matches=$(grep -nHi "$FORBIDDEN_PATTERN" "$file" 2>/dev/null || true)
  else
    added_lines=$(
      {
        git diff --cached -U0 -- "$file" 2>/dev/null || true
        git diff -U0 -- "$file" 2>/dev/null || true
      } | awk '/^\+[^+]/ { print substr($0, 2) }'
    )
    matches=$(printf '%s\n' "$added_lines" | grep -nHi "$FORBIDDEN_PATTERN" 2>/dev/null || true)
  fi
  
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

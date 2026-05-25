#!/usr/bin/env bash
# Author: Kenan AY
# Role: Developer / Architect / Designer / Implementer
#
# Ayken - Normative Spec Purity Enforcement
#
# This check protects the normative requirement and task contracts. Design and
# governance documents intentionally describe enforcement mechanisms and are
# reviewed as architecture documentation instead of being scanned here.

set -euo pipefail

echo "== CHECK: Normative Spec Purity =="

ROOT_DIR="$(git rev-parse --show-toplevel)"
cd "$ROOT_DIR"

FAIL=0
SPEC_FILES=(
  ".kiro/specs/dev-loop-boot-monitoring/requirements.md"
  ".kiro/specs/dev-loop-boot-monitoring/tasks.md"
)

FORBIDDEN_PATTERNS=(
  '^```(bash|sh|shell|python|javascript|js|json|yaml|yml|makefile|c|cpp|rust)([[:space:]]|$)'
  '^[[:space:]]*(\$[[:space:]]+)?(grep|sed|awk|make|git|python3?|bash|cargo|npm)[[:space:]]+[-[:alnum:]./]'
  '^[[:space:]]*[{[]["'\'']?[[:alnum:]_]+["'\'']?[[:space:]]*:'
)

for file in "${SPEC_FILES[@]}"; do
  if [ ! -f "$file" ]; then
    echo "VIOLATION: Missing normative spec file: $file"
    FAIL=1
    continue
  fi

  echo "Checking: $file"
  for pattern in "${FORBIDDEN_PATTERNS[@]}"; do
    matches="$(grep -nE "$pattern" "$file" 2>/dev/null || true)"
    if [ -n "$matches" ]; then
      echo "VIOLATION: Implementation syntax found in normative spec: $file"
      echo "$matches"
      FAIL=1
    fi
  done
done

if [ "$FAIL" -ne 0 ]; then
  echo ""
  echo "CRITICAL FAILURE: Normative specification purity violated"
  echo "Requirements and tasks describe behavior and capability only."
  echo "Place implementation instructions in design or implementation guides."
  exit 1
fi

echo "PASS: Normative specification purity enforced"
exit 0

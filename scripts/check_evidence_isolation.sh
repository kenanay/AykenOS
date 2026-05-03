#!/usr/bin/env bash
# Author: Kenan AY
# Role: Developer / Architect / Designer / Implementer
#
# Ayken - Evidence Isolation Enforcement
#
# CRITICAL RULE:
# Evidence (out/evidence/) MUST NEVER be used as input for validation decisions.
#
# This script scans the repository and FAILS if:
# - out/evidence is used in decision logic (if/while/grep conditions)
# - evidence is parsed or reused as input
#
# Allowed:
# - cat/echo for display only
# - dashboard / tools (read-only visualization)
#
# Forbidden:
# - if grep ... out/evidence
# - parsing evidence for decisions
# - feeding evidence into validation logic

set -euo pipefail

echo "== CHECK: Evidence Isolation =="

ROOT_DIR="$(git rev-parse --show-toplevel)"
cd "$ROOT_DIR"

FAIL=0

# Validation scripts to scan (CRITICAL - these must NOT use evidence)
VALIDATION_SCRIPTS=(
  "scripts/dev_loop.sh"
  "scripts/oracle.sh"
  "scripts/find_regression.sh"
  "scripts/test_devloop_isolation.sh"
  "scripts/test_vcp_runtime_hook.sh"
  "scripts/test_vcp_trust_verification.sh"
  "scripts/test_vcp_fail_closed.sh"
  "scripts/test_vcp_evidence.sh"
  "scripts/check_perf_regression.sh"
)

# Safe zones (allowed to use evidence)
SAFE_ZONES=(
  "tools/web"
  "tools/debug"
  "tools/verification"
  "scripts/ci"
  "scripts/generate_evidence.sh"
  "scripts/check_evidence_isolation.sh"
  "scripts/check_observation_boundary.sh"
)

# Forbidden patterns (decision usage)
FORBIDDEN_PATTERNS=(
  "if .*out/evidence"
  "while .*out/evidence"
  "grep .*out/evidence.*|"
  "awk .*out/evidence"
  "sed .*out/evidence"
)

echo "Scanning validation scripts for illegal evidence usage..."

for script in "${VALIDATION_SCRIPTS[@]}"; do
  [ -f "$script" ] || continue
  
  echo "Checking: $script"
  
  for pattern in "${FORBIDDEN_PATTERNS[@]}"; do
    matches=$(grep -nE "$pattern" "$script" 2>/dev/null || true)
    
    if [ -n "$matches" ]; then
      echo "❌ VIOLATION: Forbidden evidence usage in validation script"
      echo "  File: $script"
      echo "$matches"
      FAIL=1
    fi
  done
done

# Additional strict rule: evidence used in validation scripts
echo "Checking validation scripts for evidence assignment..."

for script in "${VALIDATION_SCRIPTS[@]}"; do
  [ -f "$script" ] || continue
  
  matches=$(grep -nE "=.*out/evidence" "$script" 2>/dev/null || true)
  
  if [ -n "$matches" ]; then
    echo "❌ VIOLATION: Evidence used in assignment in validation script"
    echo "  File: $script"
    echo "$matches"
    FAIL=1
  fi
done

# Check for evidence in validation scripts (oracle, dev_loop)
echo "Checking validation scripts for any evidence directory usage..."

for script in "${VALIDATION_SCRIPTS[@]}"; do
  if [ -f "$script" ]; then
    matches=$(grep -nE "out/evidence" "$script" || true)
    if [ -n "$matches" ]; then
      echo "❌ VIOLATION: Evidence directory used in validation script: $script"
      echo "$matches"
      FAIL=1
    fi
  fi
done

# Allow safe usage check (informational)
echo "Checking safe usage (read-only)..."

SAFE_MATCHES=$(grep -RIn "out/evidence" scripts tools 2>/dev/null | grep -E "cat|echo|ls" | grep -v "check_evidence_isolation.sh" || true)

if [ -n "$SAFE_MATCHES" ]; then
  echo "✔ Safe read-only usage detected:"
  echo "$SAFE_MATCHES"
fi

# Final decision
if [ "$FAIL" -ne 0 ]; then
  echo ""
  echo "🚨 CRITICAL FAILURE: Evidence isolation violated"
  echo ""
  echo "Rule:"
  echo "  Evidence MUST NOT be used as input to validation logic"
  echo ""
  echo "Fix:"
  echo "  Use raw logs (out/logs) for validation"
  echo "  Use evidence only for visualization"
  echo ""
  echo "Constitutional Reference:"
  echo "  See .kiro/specs/dev-loop-boot-monitoring/DEV_LOOP_CONSTITUTION.md"
  echo "  Section 5: Evidence Law"
  echo "  Section 6: Observation Source Constraint"
  echo ""
  exit 1
fi

echo "✅ PASS: Evidence isolation enforced"
exit 0

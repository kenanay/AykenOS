#!/usr/bin/env bash
# Author: Kenan AY
# Role: Developer / Architect / Designer / Implementer
#
# Ayken - Observation Boundary Enforcement
#
# CRITICAL RULE:
# Validation decisions MUST ONLY use raw boot logs (out/logs/).
# Derived artifacts (out/evidence/) MUST NEVER be used as validation input.
#
# This script scans validation scripts and FAILS if:
# - Validation reads from out/evidence/ instead of out/logs/
# - Evidence artifacts (summary.json, markers.json, perf.json) are used in decisions
# - Historical runs (history.json) affect current validation
#
# Allowed:
# - Reading from out/logs/ for validation
# - Generating evidence AFTER validation
# - Visualization reading evidence (tools/web/)
#
# Forbidden:
# - if grep ... out/evidence/
# - Validation depending on previous runs
# - Evidence as input to oracle/validation logic

set -euo pipefail

echo "== CHECK: Observation Boundary =="

ROOT_DIR="$(git rev-parse --show-toplevel)"
cd "$ROOT_DIR"

FAIL=0

# Validation scripts that MUST NOT read evidence
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

echo "Checking validation scripts for observation boundary violations..."

for script in "${VALIDATION_SCRIPTS[@]}"; do
  if [ ! -f "$script" ]; then
    continue
  fi
  
  echo "Checking: $script"
  
  # Check for evidence directory usage
  matches=$(grep -nE "out/evidence" "$script" || true)
  
  if [ -n "$matches" ]; then
    echo "❌ VIOLATION: Validation script reads from out/evidence/"
    echo "  File: $script"
    echo "$matches"
    FAIL=1
  fi
  
  # Check for evidence artifact usage
  for artifact in "summary.json" "markers.json" "perf.json" "history.json"; do
    matches=$(grep -nE "$artifact" "$script" || true)
    
    if [ -n "$matches" ]; then
      echo "❌ VIOLATION: Validation script uses evidence artifact: $artifact"
      echo "  File: $script"
      echo "$matches"
      FAIL=1
    fi
  done
  
  # Verify raw log usage (should be present)
  log_usage=$(grep -nE "out/logs/boot_watch.log" "$script" || true)
  
  if [ -z "$log_usage" ] && [ -f "$script" ]; then
    # Check if this is a validation script that should use logs
    if [[ "$script" == *"dev_loop"* ]] || [[ "$script" == *"oracle"* ]]; then
      echo "⚠ Warning: Validation script may not be reading raw logs"
      echo "  File: $script"
    fi
  fi
done

# Check evidence generator runs AFTER validation
echo ""
echo "Checking evidence generation order..."

if [ -f "scripts/dev_loop.sh" ]; then
  # Verify generate_evidence.sh is called at the end
  generate_line=$(grep -n "generate_evidence.sh" scripts/dev_loop.sh | tail -n 1 || true)
  validation_line=$(grep -n "AYKEN_BOOT_OK" scripts/dev_loop.sh | head -n 1 || true)
  
  if [ -n "$generate_line" ] && [ -n "$validation_line" ]; then
    gen_num=$(echo "$generate_line" | cut -d: -f1)
    val_num=$(echo "$validation_line" | cut -d: -f1)
    
    if [ "$gen_num" -lt "$val_num" ]; then
      echo "❌ VIOLATION: Evidence generation before validation"
      echo "  Evidence must be generated AFTER validation decisions"
      FAIL=1
    else
      echo "✔ Evidence generation after validation: correct order"
    fi
  fi
fi

# Check for history.json usage in validation
echo ""
echo "Checking for historical run dependencies..."

HISTORY_USAGE=$(grep -RnE "history\.json" scripts/*.sh 2>/dev/null | grep -v "generate_evidence.sh" | grep -v "check_observation_boundary.sh" || true)

if [ -n "$HISTORY_USAGE" ]; then
  echo "❌ VIOLATION: Validation depends on historical runs"
  echo "$HISTORY_USAGE"
  FAIL=1
fi

# Verify safe zones (tools/web can read evidence)
echo ""
echo "Verifying safe zones (visualization only)..."

WEB_USAGE=$(grep -RnE "out/evidence" tools/web 2>/dev/null || true)

if [ -n "$WEB_USAGE" ]; then
  echo "✔ Safe zone: tools/web/ reads evidence (visualization only)"
fi

# Final decision
if [ "$FAIL" -ne 0 ]; then
  echo ""
  echo "🚨 CRITICAL FAILURE: Observation boundary violated"
  echo ""
  echo "Rule:"
  echo "  Validation MUST use ONLY raw logs (out/logs/)"
  echo "  Evidence (out/evidence/) MUST NOT be validation input"
  echo ""
  echo "Fix:"
  echo "  - Replace evidence reads with raw log reads"
  echo "  - Move evidence generation to AFTER validation"
  echo "  - Remove historical run dependencies"
  echo ""
  echo "Constitutional Reference:"
  echo "  See .kiro/specs/dev-loop-boot-monitoring/DEV_LOOP_CONSTITUTION.md"
  echo "  Section 6: Observation Source Constraint"
  echo "  Section 7: State Isolation Law"
  echo ""
  echo "Requirements Reference:"
  echo "  Requirement 26: Direct Observation Source Constraint"
  echo "  Requirement 27: Evidence State Isolation"
  echo ""
  exit 1
fi

echo ""
echo "✅ PASS: Observation boundary enforced"
echo "  - Validation uses raw logs only"
echo "  - Evidence generated after validation"
echo "  - No historical run dependencies"
exit 0

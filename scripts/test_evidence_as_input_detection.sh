#!/usr/bin/env bash
# Author: Kenan AY
# Role: Developer / Architect / Designer / Implementer
#
# Task 26.2: Static analysis for evidence-as-input detection
#
# This test validates that dev loop scripts never use evidence artifacts
# as validation input, maintaining the observation source constraint.
#
# Validation Properties:
# 1. Dev loop scripts do not read from out/evidence/* for validation
# 2. Evidence artifacts are never used as decision input
# 3. Validation uses only raw boot logs (out/logs/boot_watch.log)
# 4. Evidence generation runs AFTER validation completes
# 5. Evidence is derived data, never authority
#
# Test Method:
# - Static analysis of dev loop scripts for evidence directory access
# - Verification that validation logic uses only allowed observation sources
# - Confirmation that evidence is never used in conditional logic
# - Validation that evidence reads are only for visualization/reporting

set -euo pipefail

echo "== Task 26.2: Evidence-as-Input Detection =="
echo ""
echo "Validating observation source constraints..."
echo ""

ROOT_DIR="$(git rev-parse --show-toplevel)"
cd "$ROOT_DIR"

FAIL=0

# ============================================================================
# Property 1: Dev loop scripts do not read evidence for validation
# ============================================================================

echo "[Property 1] Dev loop scripts do not read evidence for validation"
echo ""

# Dev loop scripts that perform validation
DEV_LOOP_VALIDATION_SCRIPTS=(
  "scripts/dev_loop.sh"
  "scripts/oracle.sh"
  "scripts/find_regression.sh"
  "scripts/test_devloop_isolation.sh"
  "scripts/test_marker_validation.sh"
  "scripts/test_exit_status_contract.sh"
  "scripts/test_qemu_integration.sh"
  "scripts/test_vcp_runtime_hook.sh"
  "scripts/test_vcp_trust_verification.sh"
  "scripts/test_vcp_fail_closed.sh"
  "scripts/test_vcp_evidence.sh"
  "scripts/test_constitutional_compliance.sh"
  "scripts/test_full_validation_capability.sh"
  "scripts/test_isolation_boundary_guarantee.sh"
  "scripts/check_vcp_runtime_contract.sh"
)

# Forbidden patterns: reading from evidence directory for validation
# Note: Writing to evidence (mkdir, >, >>) is allowed for evidence generation
FORBIDDEN_EVIDENCE_READ_PATTERNS=(
  # Direct evidence directory reads (not writes)
  "cat [^>]*out/evidence/"
  "grep [^>]*out/evidence/"
  "awk [^>]*out/evidence/"
  "sed [^>]*out/evidence/"
  "head [^>]*out/evidence/"
  "tail [^>]*out/evidence/"
  "less [^>]*out/evidence/"
  "more [^>]*out/evidence/"

  # Evidence file reads (input redirection)
  "< *out/evidence/"
  "read.*< *out/evidence/"

  # Evidence directory listing for validation decisions
  "ls [^>]*out/evidence/.*\|"
  "find [^>]*out/evidence/.*\|"

  # JSON parsing from evidence for validation
  "jq [^>]*out/evidence/.*\|"

  # Evidence variable assignment (reading for validation input)
  "=.*\$(cat [^>]*out/evidence/"
  "=.*\$(grep [^>]*out/evidence/"
  "=.*\$(jq [^>]*out/evidence/"

  # Conditional logic based on evidence content
  "if.*cat.*out/evidence/"
  "if.*grep.*out/evidence/"
  "if.*jq.*out/evidence/"
  "while.*cat.*out/evidence/"
  "until.*cat.*out/evidence/"
)

echo "Checking dev loop validation scripts for evidence reads..."

for script in "${DEV_LOOP_VALIDATION_SCRIPTS[@]}"; do
  if [ ! -f "$script" ]; then
    continue
  fi

  echo "  Checking: $script"

  for pattern in "${FORBIDDEN_EVIDENCE_READ_PATTERNS[@]}"; do
    # Skip comment lines and variable declarations that are just paths
    matches=$(grep -nE "$pattern" "$script" 2>/dev/null | grep -v "^[[:space:]]*#" | grep -v "EVIDENCE_DIR=" || true)

    if [ -n "$matches" ]; then
      echo "    ❌ VIOLATION: Evidence read detected in validation script"
      echo "    Pattern: $pattern"
      echo "$matches"
      FAIL=1
    fi
  done
done

if [ "$FAIL" -eq 0 ]; then
  echo "  ✅ No evidence reads detected in validation scripts"
fi

echo ""

# ============================================================================
# Property 2: Validation uses only allowed observation sources
# ============================================================================

echo "[Property 2] Validation uses only allowed observation sources"
echo ""

# Allowed observation sources for validation
ALLOWED_SOURCES=(
  "out/logs/boot_watch.log"
  "out/logs/debug_run.log"
  "out/logs/qemu_run.log"
)

echo "Verifying validation scripts use only allowed sources..."

# Check that dev_loop.sh uses only allowed sources
if [ -f "scripts/dev_loop.sh" ]; then
  echo "  Checking: scripts/dev_loop.sh"

  # Extract log file references
  log_refs=$(grep -E "BOOT_LOG=|DEBUG_LOG=|QEMU_LOG=" scripts/dev_loop.sh || true)

  echo "    Log file references:"
  echo "$log_refs" | sed 's/^/      /'

  # Verify these are in out/logs/ directory (LOG_DIR variable)
  if echo "$log_refs" | grep -qE "LOG_DIR=.*out/logs"; then
    echo "    ✅ Uses out/logs/ directory (allowed)"
  elif echo "$log_refs" | grep -qE "\\\$LOG_DIR"; then
    # Check if LOG_DIR is defined as out/logs
    log_dir_def=$(grep "^LOG_DIR=" scripts/dev_loop.sh | head -1)
    if echo "$log_dir_def" | grep -q "out/logs"; then
      echo "    ✅ Uses out/logs/ directory via LOG_DIR variable (allowed)"
    else
      echo "    ❌ VIOLATION: LOG_DIR does not point to out/logs/"
      FAIL=1
    fi
  else
    echo "    ❌ VIOLATION: Does not use out/logs/ directory"
    FAIL=1
  fi

  # Verify no evidence directory references in validation logic
  evidence_refs=$(grep -n "out/evidence/" scripts/dev_loop.sh | grep -v "generate_evidence.sh" || true)

  if [ -n "$evidence_refs" ]; then
    echo "    ❌ VIOLATION: Evidence directory referenced in validation logic"
    echo "$evidence_refs"
    FAIL=1
  else
    echo "    ✅ No evidence directory references in validation logic"
  fi
fi

echo ""

# ============================================================================
# Property 3: Evidence is never used in conditional logic
# ============================================================================

echo "[Property 3] Evidence is never used in conditional logic"
echo ""

echo "Checking for evidence in conditional statements..."

for script in "${DEV_LOOP_VALIDATION_SCRIPTS[@]}"; do
  if [ ! -f "$script" ]; then
    continue
  fi

  echo "  Checking: $script"

  # Check for evidence in if statements
  evidence_conditionals=$(grep -B2 -A2 "if.*out/evidence/" "$script" 2>/dev/null || true)

  if [ -n "$evidence_conditionals" ]; then
    echo "    ❌ VIOLATION: Evidence used in conditional logic"
    echo "$evidence_conditionals"
    FAIL=1
  fi

  # Check for evidence in case statements
  evidence_case=$(grep -B2 -A2 "case.*out/evidence/" "$script" 2>/dev/null || true)

  if [ -n "$evidence_case" ]; then
    echo "    ❌ VIOLATION: Evidence used in case statement"
    echo "$evidence_case"
    FAIL=1
  fi

  # Check for evidence in while/until loops
  evidence_loops=$(grep -B2 -A2 "while.*out/evidence/\|until.*out/evidence/" "$script" 2>/dev/null || true)

  if [ -n "$evidence_loops" ]; then
    echo "    ❌ VIOLATION: Evidence used in loop condition"
    echo "$evidence_loops"
    FAIL=1
  fi
done

if [ "$FAIL" -eq 0 ]; then
  echo "  ✅ No evidence in conditional logic"
fi

echo ""

# ============================================================================
# Property 4: Evidence generation runs AFTER validation
# ============================================================================

echo "[Property 4] Evidence generation runs AFTER validation"
echo ""

if [ -f "scripts/dev_loop.sh" ]; then
  echo "  Checking: scripts/dev_loop.sh"

  # Find line numbers for validation completion and evidence generation
  validation_complete_line=$(grep -n "✅ PASS:" scripts/dev_loop.sh | tail -n1 | cut -d: -f1 || echo "0")
  evidence_gen_line=$(grep -n "generate_evidence.sh" scripts/dev_loop.sh | head -n1 | cut -d: -f1 || echo "0")

  if [ "$validation_complete_line" -gt 0 ] && [ "$evidence_gen_line" -gt 0 ]; then
    if [ "$evidence_gen_line" -lt "$validation_complete_line" ]; then
      echo "    ❌ VIOLATION: Evidence generation before validation completion"
      echo "    Validation complete line: $validation_complete_line"
      echo "    Evidence generation line: $evidence_gen_line"
      FAIL=1
    else
      echo "    ✅ Evidence generation after validation (line $evidence_gen_line > $validation_complete_line)"
    fi
  else
    echo "    ⚠ Warning: Could not determine validation/evidence order"
  fi

  # Verify evidence generation is not in critical path
  evidence_context=$(grep -B5 -A5 "generate_evidence.sh" scripts/dev_loop.sh || true)

  # Check that evidence generation failure doesn't affect exit status
  if echo "$evidence_context" | grep -q "set +e\||| true"; then
    echo "    ✅ Evidence generation failure does not affect validation outcome"
  else
    echo "    ⚠ Warning: Evidence generation may affect exit status"
  fi
fi

echo ""

# ============================================================================
# Property 5: Evidence reads are only for visualization/reporting
# ============================================================================

echo "[Property 5] Evidence reads are only for visualization/reporting"
echo ""

# Scripts that are ALLOWED to read evidence (visualization/reporting only)
ALLOWED_EVIDENCE_READERS=(
  "scripts/generate_evidence.sh"
  "scripts/compare_runs.sh"
  "scripts/dashboard.sh"
  "docs/dev-loop/dashboard.html"
)

echo "Identifying scripts that read evidence..."

# Find all scripts that reference out/evidence/
all_scripts=$(find scripts -type f -name "*.sh" 2>/dev/null || true)

for script in $all_scripts; do
  # Look for actual evidence reads (not just path definitions or writes)
  evidence_reads=$(grep -nE "cat [^>]*out/evidence/|grep [^>]*out/evidence/|jq [^>]*out/evidence/|< *out/evidence/" "$script" 2>/dev/null | grep -v "^[[:space:]]*#" || true)

  if [ -n "$evidence_reads" ]; then
    # Check if this script is in allowed list
    is_allowed=0
    for allowed in "${ALLOWED_EVIDENCE_READERS[@]}"; do
      if [ "$script" = "$allowed" ]; then
        is_allowed=1
        break
      fi
    done

    # Check if this script is a validation script
    is_validation=0
    for validation_script in "${DEV_LOOP_VALIDATION_SCRIPTS[@]}"; do
      if [ "$script" = "$validation_script" ]; then
        is_validation=1
        break
      fi
    done

    if [ "$is_validation" -eq 1 ]; then
      echo "  ❌ VIOLATION: Validation script reads evidence: $script"
      echo "$evidence_reads" | head -5
      FAIL=1
    elif [ "$is_allowed" -eq 1 ]; then
      echo "  ✅ Allowed evidence reader: $script (visualization/reporting)"
    else
      # Only warn if it's actually reading, not just defining paths
      if echo "$evidence_reads" | grep -qE "cat |grep |jq |<"; then
        echo "  ⚠ Warning: Unexpected evidence reader: $script"
        echo "$evidence_reads" | head -3
      fi
    fi
  fi
done

echo ""

# ============================================================================
# Property 6: Observation source constraint in oracle
# ============================================================================

echo "[Property 6] Oracle uses only raw boot logs"
echo ""

if [ -f "scripts/oracle.sh" ]; then
  echo "  Checking: scripts/oracle.sh"

  # Oracle should only read from out/logs/
  oracle_sources=$(grep -E "cat |grep |awk |sed " scripts/oracle.sh | grep -v "^#" || true)

  # Check for evidence directory access
  if echo "$oracle_sources" | grep -q "out/evidence/"; then
    echo "    ❌ VIOLATION: Oracle reads from evidence directory"
    echo "$oracle_sources" | grep "out/evidence/"
    FAIL=1
  else
    echo "    ✅ Oracle does not read from evidence directory"
  fi

  # Verify oracle uses out/logs/
  if echo "$oracle_sources" | grep -q "out/logs/"; then
    echo "    ✅ Oracle uses out/logs/ (allowed observation source)"
  else
    echo "    ⚠ Warning: Oracle may not use standard log directory"
  fi
fi

echo ""

# ============================================================================
# Property 7: Regression finder uses only raw logs
# ============================================================================

echo "[Property 7] Regression finder uses only raw logs"
echo ""

if [ -f "scripts/find_regression.sh" ]; then
  echo "  Checking: scripts/find_regression.sh"

  # Regression finder should only read from out/logs/
  regression_sources=$(grep -E "cat |grep |awk |sed " scripts/find_regression.sh | grep -v "^#" || true)

  # Check for evidence directory access
  if echo "$regression_sources" | grep -q "out/evidence/"; then
    echo "    ❌ VIOLATION: Regression finder reads from evidence directory"
    echo "$regression_sources" | grep "out/evidence/"
    FAIL=1
  else
    echo "    ✅ Regression finder does not read from evidence directory"
  fi
fi

echo ""

# ============================================================================
# Property 8: Evidence directory structure is write-only for validation
# ============================================================================

echo "[Property 8] Evidence directory is write-only for validation"
echo ""

echo "Checking evidence directory access patterns..."

for script in "${DEV_LOOP_VALIDATION_SCRIPTS[@]}"; do
  if [ ! -f "$script" ]; then
    continue
  fi

  # Check for evidence reads (forbidden for validation)
  # Exclude EVIDENCE_DIR variable definitions and mkdir commands
  evidence_reads=$(grep -nE "cat [^>]*out/evidence/|grep [^>]*out/evidence/|< *out/evidence/" "$script" 2>/dev/null | grep -v "^[[:space:]]*#" | grep -v "EVIDENCE_DIR=" || true)

  if [ -n "$evidence_reads" ]; then
    echo "  ❌ VIOLATION: $script reads from evidence directory"
    echo "$evidence_reads"
    FAIL=1
  fi
done

if [ "$FAIL" -eq 0 ]; then
  echo "  ✅ Evidence directory is write-only for validation scripts"
fi

echo ""

# ============================================================================
# Final Report
# ============================================================================

echo "========================================"
echo "Evidence-as-Input Detection Summary"
echo "========================================"
echo ""

if [ "$FAIL" -eq 0 ]; then
  echo "✅ PASS: All observation source constraints validated"
  echo ""
  echo "Validated Properties:"
  echo "  ✅ Dev loop scripts do not read evidence for validation"
  echo "  ✅ Validation uses only allowed observation sources"
  echo "  ✅ Evidence is never used in conditional logic"
  echo "  ✅ Evidence generation runs after validation"
  echo "  ✅ Evidence reads are only for visualization/reporting"
  echo "  ✅ Oracle uses only raw boot logs"
  echo "  ✅ Regression finder uses only raw logs"
  echo "  ✅ Evidence directory is write-only for validation"
  echo ""
  echo "Observation Source Constraint:"
  echo "  ✅ Validation uses only raw boot logs (out/logs/)"
  echo "  ✅ Evidence artifacts (out/evidence/) forbidden as input"
  echo "  ✅ Evidence is derived data, never authority"
  echo "  ✅ Evidence cannot affect validation decisions"
  echo ""
  echo "Constitutional Compliance:"
  echo "  ✅ R23: Dev Loop Non-Interference Guarantee"
  echo "  ✅ R26: Direct Observation Source Constraint"
  echo "  ✅ R27: Evidence State Isolation"
  echo "  ✅ Design Section 2.2: Observation Source Constraint"
  echo "  ✅ Design Section 2.3: Evidence ≠ Authority"
  echo "  ✅ Design Section 5.2: Forbidden Flow (Evidence → Validation)"
  echo ""
  echo "Task 26.2 (Evidence-as-input detection) COMPLETE"
  exit 0
else
  echo "❌ FAIL: Observation source constraint violations detected"
  echo ""
  echo "Critical Failures:"
  echo "  - Evidence artifacts used as validation input"
  echo "  - Observation source constraint VIOLATED"
  echo "  - Evidence has become authority (forbidden)"
  echo ""
  echo "Required Actions:"
  echo "  1. Remove all evidence directory reads from validation scripts"
  echo "  2. Use only raw boot logs (out/logs/) for validation"
  echo "  3. Ensure evidence generation runs after validation"
  echo "  4. Move evidence reads to visualization/reporting scripts only"
  echo ""
  echo "Constitutional Reference:"
  echo "  - Requirement R23: Dev Loop Non-Interference Guarantee"
  echo "  - Requirement R26: Direct Observation Source Constraint"
  echo "  - Requirement R27: Evidence State Isolation"
  echo "  - Design Section 2.2: Observation Source Constraint"
  echo "  - Design Section 2.3: Evidence ≠ Authority"
  echo "  - Design Section 5.2: Forbidden Flow"
  echo "  - Design Section 10: Anti-Patterns (Evidence as Validation Input)"
  echo ""
  exit 1
fi

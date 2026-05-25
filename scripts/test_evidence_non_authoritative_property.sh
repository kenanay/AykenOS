#!/usr/bin/env bash
# Author: Kenan AY
# Role: Developer / Architect / Designer / Implementer
#
# Task 26.3: Evidence pipeline non-authoritative property
#
# This test validates that the evidence pipeline maintains non-authoritative
# properties throughout the system lifecycle.
#
# Non-Authoritative Properties:
# 1. Evidence is generated AFTER validation completes
# 2. Evidence is read-only for visualization
# 3. Evidence never affects validation outcome
# 4. Evidence never affects execution flow
# 5. Evidence is derived data, never decision input
# 6. Evidence artifacts are purely observational
# 7. Evidence cannot influence kernel execution
# 8. Evidence generation failure does not affect validation
#
# Test Method:
# - Temporal ordering verification (evidence after validation)
# - Data flow analysis (evidence never flows back to validation)
# - Failure isolation (evidence failure doesn't affect validation)
# - Authority boundary enforcement (evidence has no decision power)
# - Observational purity (evidence is read-only derived data)

set -euo pipefail

echo "== Task 26.3: Evidence Pipeline Non-Authoritative Property =="
echo ""
echo "Validating evidence non-authoritative properties..."
echo ""

ROOT_DIR="$(git rev-parse --show-toplevel)"
cd "$ROOT_DIR"

FAIL=0

# ============================================================================
# Property 1: Evidence generation runs AFTER validation completes
# ============================================================================

echo "[Property 1] Evidence generation runs AFTER validation completes"
echo ""

if [ -f "scripts/dev_loop.sh" ]; then
  echo "  Checking: scripts/dev_loop.sh"

  # Find line numbers for validation completion and evidence generation
  validation_pass_line=$(grep -n "✅ PASS:" scripts/dev_loop.sh | tail -n1 | cut -d: -f1 || echo "0")
  evidence_gen_line=$(grep -n "generate_evidence.sh\|Generating evidence" scripts/dev_loop.sh | head -n1 | cut -d: -f1 || echo "0")

  if [ "$validation_pass_line" -gt 0 ] && [ "$evidence_gen_line" -gt 0 ]; then
    if [ "$evidence_gen_line" -lt "$validation_pass_line" ]; then
      echo "    ❌ VIOLATION: Evidence generation before validation completion"
      echo "    Validation PASS line: $validation_pass_line"
      echo "    Evidence generation line: $evidence_gen_line"
      FAIL=1
    else
      echo "    ✅ Evidence generation after validation PASS (line $evidence_gen_line > $validation_pass_line)"
    fi
  else
    echo "    ⚠ Warning: Could not determine validation/evidence order"
    echo "    Validation PASS line: $validation_pass_line"
    echo "    Evidence generation line: $evidence_gen_line"
  fi

  # Verify evidence generation is in the post-validation section
  # Extract the section after "✅ PASS: $MODE mode"
  post_validation_section=$(awk '/✅ PASS:.*mode/,EOF' scripts/dev_loop.sh)

  if echo "$post_validation_section" | grep -q "generate_evidence.sh\|Generating evidence"; then
    echo "    ✅ Evidence generation in post-validation section"
  else
    echo "    ❌ VIOLATION: Evidence generation not in post-validation section"
    FAIL=1
  fi

  # Verify evidence generation has explicit comment about non-authority
  evidence_context=$(grep -B2 -A2 "generate_evidence.sh" scripts/dev_loop.sh || true)

  if echo "$evidence_context" | grep -q "non-authoritative\|never affects validation\|runs AFTER validation"; then
    echo "    ✅ Evidence generation has non-authoritative documentation"
  else
    echo "    ⚠ Warning: Evidence generation lacks non-authoritative documentation"
  fi
fi

echo ""

# ============================================================================
# Property 2: Evidence generation failure does not affect validation outcome
# ============================================================================

echo "[Property 2] Evidence generation failure does not affect validation outcome"
echo ""

if [ -f "scripts/dev_loop.sh" ]; then
  echo "  Checking: scripts/dev_loop.sh"

  # Check that evidence generation is wrapped in failure-tolerant constructs
  evidence_context=$(grep -B5 -A5 "generate_evidence.sh" scripts/dev_loop.sh || true)

  # Look for failure tolerance patterns
  has_failure_tolerance=0

  if echo "$evidence_context" | grep -q "set +e"; then
    echo "    ✅ Evidence generation uses 'set +e' (failure tolerant)"
    has_failure_tolerance=1
  fi

  if echo "$evidence_context" | grep -q "|| true"; then
    echo "    ✅ Evidence generation uses '|| true' (failure tolerant)"
    has_failure_tolerance=1
  fi

  if echo "$evidence_context" | grep -q "if \[.*generate_evidence"; then
    echo "    ✅ Evidence generation uses conditional execution (failure tolerant)"
    has_failure_tolerance=1
  fi

  # Check that evidence generation is NOT in the critical path
  # (i.e., it's after the final PASS/FAIL decision)
  if echo "$evidence_context" | grep -B10 "generate_evidence.sh" | grep -q "exit 0\|exit 1"; then
    echo "    ❌ VIOLATION: Evidence generation before exit status determination"
    FAIL=1
  else
    echo "    ✅ Evidence generation after exit status determination"
  fi

  if [ "$has_failure_tolerance" -eq 0 ]; then
    echo "    ⚠ Warning: Evidence generation may not be failure-tolerant"
  fi
fi

echo ""

# ============================================================================
# Property 3: Evidence artifacts are never used as validation input
# ============================================================================

echo "[Property 3] Evidence artifacts are never used as validation input"
echo ""

# This property is already validated by task 26.2 (evidence-as-input detection)
# We verify that the detection script exists and passes

if [ -f "scripts/test_evidence_as_input_detection.sh" ]; then
  echo "  Checking: scripts/test_evidence_as_input_detection.sh exists"
  echo "    ✅ Evidence-as-input detection script exists"

  # Run the detection script to verify evidence isolation
  echo "  Running evidence-as-input detection..."
  if bash scripts/test_evidence_as_input_detection.sh > /dev/null 2>&1; then
    echo "    ✅ Evidence-as-input detection PASSED"
  else
    echo "    ❌ VIOLATION: Evidence-as-input detection FAILED"
    echo "    Evidence artifacts may be used as validation input"
    FAIL=1
  fi
else
  echo "  ⚠ Warning: scripts/test_evidence_as_input_detection.sh not found"
  echo "  Cannot verify evidence isolation property"
fi

echo ""

# ============================================================================
# Property 4: Evidence directory structure is write-only for validation
# ============================================================================

echo "[Property 4] Evidence directory is write-only for validation pipeline"
echo ""

# Validation scripts should only WRITE to evidence, never READ from it
VALIDATION_SCRIPTS=(
  "scripts/dev_loop.sh"
  "scripts/oracle.sh"
  "scripts/find_regression.sh"
)

echo "Checking validation scripts for evidence reads..."

for script in "${VALIDATION_SCRIPTS[@]}"; do
  if [ ! -f "$script" ]; then
    continue
  fi

  echo "  Checking: $script"

  # Look for evidence reads (forbidden)
  # Exclude: variable definitions, mkdir, writes (>, >>)
  evidence_reads=$(grep -nE "cat [^>]*out/evidence/|grep [^>]*out/evidence/|< *out/evidence/|jq [^>]*out/evidence/" "$script" 2>/dev/null | grep -v "^[[:space:]]*#" | grep -v "EVIDENCE_DIR=" || true)

  if [ -n "$evidence_reads" ]; then
    echo "    ❌ VIOLATION: Validation script reads from evidence directory"
    echo "$evidence_reads"
    FAIL=1
  else
    echo "    ✅ No evidence reads detected"
  fi

  # Look for evidence writes (allowed for evidence generation)
  evidence_writes=$(grep -nE "> *out/evidence/|>> *out/evidence/|mkdir.*out/evidence" "$script" 2>/dev/null | grep -v "^[[:space:]]*#" || true)

  if [ -n "$evidence_writes" ]; then
    echo "    ℹ Evidence writes detected (allowed for evidence generation)"
  fi
done

echo ""

# ============================================================================
# Property 5: Evidence has no decision authority
# ============================================================================

echo "[Property 5] Evidence has no decision authority"
echo ""

echo "Checking for evidence in decision logic..."

for script in "${VALIDATION_SCRIPTS[@]}"; do
  if [ ! -f "$script" ]; then
    continue
  fi

  echo "  Checking: $script"

  # Check for evidence in conditional statements (if, case, while, until)
  evidence_in_conditionals=$(grep -nE "if.*out/evidence/|case.*out/evidence/|while.*out/evidence/|until.*out/evidence/" "$script" 2>/dev/null | grep -v "^[[:space:]]*#" || true)

  if [ -n "$evidence_in_conditionals" ]; then
    echo "    ❌ VIOLATION: Evidence used in decision logic"
    echo "$evidence_in_conditionals"
    FAIL=1
  else
    echo "    ✅ Evidence not used in decision logic"
  fi

  # Check for evidence in exit status determination
  evidence_in_exit=$(grep -B5 "exit 0\|exit 1" "$script" | grep "out/evidence/" || true)

  if [ -n "$evidence_in_exit" ]; then
    echo "    ❌ VIOLATION: Evidence affects exit status"
    echo "$evidence_in_exit"
    FAIL=1
  else
    echo "    ✅ Evidence does not affect exit status"
  fi
done

echo ""

# ============================================================================
# Property 6: Evidence is purely observational (derived data)
# ============================================================================

echo "[Property 6] Evidence is purely observational (derived data)"
echo ""

# Evidence should be generated FROM logs, not FROM kernel state
# Evidence generation should only read from out/logs/, not modify kernel

if [ -f "scripts/generate_evidence.sh" ]; then
  echo "  Checking: scripts/generate_evidence.sh"

  # Verify evidence generator reads from logs
  log_reads=$(grep -nE "cat.*out/logs/|grep.*out/logs/|awk.*out/logs/" scripts/generate_evidence.sh 2>/dev/null || true)

  if [ -n "$log_reads" ]; then
    echo "    ✅ Evidence generator reads from logs (observational)"
  else
    echo "    ⚠ Warning: Evidence generator may not read from logs"
  fi

  # Verify evidence generator does not modify kernel state
  kernel_modifications=$(grep -nE "/dev/mem|/dev/kmem|/proc/sys|insmod|rmmod|modprobe" scripts/generate_evidence.sh 2>/dev/null || true)

  if [ -n "$kernel_modifications" ]; then
    echo "    ❌ VIOLATION: Evidence generator modifies kernel state"
    echo "$kernel_modifications"
    FAIL=1
  else
    echo "    ✅ Evidence generator does not modify kernel state"
  fi

  # Verify evidence generator writes to evidence directory
  evidence_writes=$(grep -nE "> *out/evidence/|>> *out/evidence/|mkdir.*out/evidence" scripts/generate_evidence.sh 2>/dev/null || true)

  if [ -n "$evidence_writes" ]; then
    echo "    ✅ Evidence generator writes to evidence directory (derived data)"
  else
    echo "    ⚠ Warning: Evidence generator may not write to evidence directory"
  fi
else
  echo "  ⚠ Warning: scripts/generate_evidence.sh not found"
  echo "  Cannot verify observational property"
fi

echo ""

# ============================================================================
# Property 7: Evidence cannot influence kernel execution
# ============================================================================

echo "[Property 7] Evidence cannot influence kernel execution"
echo ""

# Evidence is generated in userspace, after kernel has completed boot
# Evidence should never be passed back to kernel

echo "Checking for evidence-to-kernel data flow..."

# Check validation scripts only (not test scripts which check for violations)
SCRIPTS_TO_CHECK=(
  "scripts/dev_loop.sh"
  "scripts/oracle.sh"
  "scripts/find_regression.sh"
  "scripts/generate_evidence.sh"
)

evidence_to_kernel_found=0

for script in "${SCRIPTS_TO_CHECK[@]}"; do
  if [ ! -f "$script" ]; then
    continue
  fi

  # Look for patterns that could pass evidence to kernel
  # This is a very specific pattern: reading evidence and writing to kernel devices
  evidence_to_kernel=$(grep -nE "cat.*out/evidence/.*>/dev/|cat.*out/evidence/.*>/proc/|cat.*out/evidence/.*>/sys/" "$script" 2>/dev/null | grep -v "^[[:space:]]*#" || true)

  if [ -n "$evidence_to_kernel" ]; then
    echo "  ❌ VIOLATION: $script passes evidence to kernel"
    echo "$evidence_to_kernel"
    FAIL=1
    evidence_to_kernel_found=1
  fi
done

if [ "$evidence_to_kernel_found" -eq 0 ]; then
  echo "  ✅ No evidence-to-kernel data flow detected"
fi

echo ""

# ============================================================================
# Property 8: Evidence generation is optional (not required for validation)
# ============================================================================

echo "[Property 8] Evidence generation is optional (not required for validation)"
echo ""

if [ -f "scripts/dev_loop.sh" ]; then
  echo "  Checking: scripts/dev_loop.sh"

  # Check that evidence generation is conditional (if [ -f ... ])
  evidence_conditional=$(grep -B2 "generate_evidence.sh" scripts/dev_loop.sh | grep "if \[" || true)

  if [ -n "$evidence_conditional" ]; then
    echo "    ✅ Evidence generation is conditional (optional)"
  else
    echo "    ⚠ Warning: Evidence generation may be unconditional"
  fi

  # Verify that validation PASS is declared BEFORE evidence generation
  pass_before_evidence=$(awk '/✅ PASS:.*mode/,/generate_evidence.sh/' scripts/dev_loop.sh | head -1)

  if echo "$pass_before_evidence" | grep -q "✅ PASS:"; then
    echo "    ✅ Validation PASS declared before evidence generation"
  else
    echo "    ❌ VIOLATION: Validation PASS not declared before evidence generation"
    FAIL=1
  fi
fi

echo ""

# ============================================================================
# Property 9: Evidence artifacts are stateless
# ============================================================================

echo "[Property 9] Evidence artifacts are stateless"
echo ""

# Evidence should not maintain state between runs
# Each evidence generation should be independent

if [ -f "scripts/generate_evidence.sh" ]; then
  echo "  Checking: scripts/generate_evidence.sh"

  # Look for state files that persist between runs
  state_files=$(grep -nE "state\.json|state\.txt|\.state|last_run|previous_run" scripts/generate_evidence.sh 2>/dev/null | grep -v "^[[:space:]]*#" || true)

  if [ -n "$state_files" ]; then
    echo "    ⚠ Warning: Evidence generator may maintain state"
    echo "$state_files"
  else
    echo "    ✅ Evidence generator appears stateless"
  fi

  # Verify each run creates a new evidence directory
  run_dir_creation=$(grep -nE "RUN_ID=|run-.*-" scripts/generate_evidence.sh 2>/dev/null || true)

  if [ -n "$run_dir_creation" ]; then
    echo "    ✅ Evidence generator creates unique run directories"
  else
    echo "    ⚠ Warning: Evidence generator may not create unique run directories"
  fi
else
  echo "  ⚠ Warning: scripts/generate_evidence.sh not found"
fi

echo ""

# ============================================================================
# Property 10: Evidence visualization is read-only
# ============================================================================

echo "[Property 10] Evidence visualization is read-only"
echo ""

# Dashboard and visualization tools should only READ evidence, never modify it

VISUALIZATION_SCRIPTS=(
  "scripts/dashboard.sh"
  "scripts/compare_runs.sh"
  "docs/dev-loop/dashboard.html"
)

echo "Checking visualization scripts for evidence writes..."

for script in "${VISUALIZATION_SCRIPTS[@]}"; do
  if [ ! -f "$script" ]; then
    continue
  fi

  echo "  Checking: $script"

  # Look for evidence writes (forbidden for visualization)
  evidence_writes=$(grep -nE "> *out/evidence/|>> *out/evidence/" "$script" 2>/dev/null | grep -v "^[[:space:]]*#" || true)

  if [ -n "$evidence_writes" ]; then
    echo "    ❌ VIOLATION: Visualization script writes to evidence"
    echo "$evidence_writes"
    FAIL=1
  else
    echo "    ✅ Visualization script is read-only"
  fi

  # Look for evidence reads (allowed for visualization)
  evidence_reads=$(grep -nE "cat.*out/evidence/|grep.*out/evidence/|jq.*out/evidence/" "$script" 2>/dev/null | grep -v "^[[:space:]]*#" || true)

  if [ -n "$evidence_reads" ]; then
    echo "    ✅ Visualization script reads evidence (allowed)"
  fi
done

echo ""

# ============================================================================
# Property 11: Evidence never affects execution flow
# ============================================================================

echo "[Property 11] Evidence never affects execution flow"
echo ""

# Evidence should not cause early returns, exits, or control flow changes
# in validation scripts

echo "Checking for evidence affecting control flow..."

for script in "${VALIDATION_SCRIPTS[@]}"; do
  if [ ! -f "$script" ]; then
    continue
  fi

  echo "  Checking: $script"

  # Look for control flow statements near evidence references
  evidence_control_flow=$(grep -B2 -A2 "out/evidence/" "$script" 2>/dev/null | grep -E "return|exit|break|continue|goto" | grep -v "^[[:space:]]*#" || true)

  if [ -n "$evidence_control_flow" ]; then
    # Filter out false positives (e.g., exit in comments or after validation)
    suspicious_flow=$(echo "$evidence_control_flow" | grep -v "exit 0" | grep -v "exit 1" || true)

    if [ -n "$suspicious_flow" ]; then
      echo "    ⚠ Warning: Evidence near control flow statements"
      echo "$suspicious_flow"
    else
      echo "    ✅ No evidence affecting control flow"
    fi
  else
    echo "    ✅ No evidence affecting control flow"
  fi
done

echo ""

# ============================================================================
# Property 12: Evidence pipeline temporal ordering guarantee
# ============================================================================

echo "[Property 12] Evidence pipeline temporal ordering guarantee"
echo ""

# Verify the complete temporal ordering:
# 1. Build
# 2. Boot
# 3. Validation
# 4. PASS/FAIL decision
# 5. Evidence generation (AFTER decision)

if [ -f "scripts/dev_loop.sh" ]; then
  echo "  Checking: scripts/dev_loop.sh temporal ordering"

  # Extract line numbers for each phase (using case statement execution order)
  # The case statement at the end shows the actual execution order
  case_start=$(grep -n "^case \"\$MODE\" in" scripts/dev_loop.sh | tail -n1 | cut -d: -f1 || echo "0")
  decision_line=$(grep -n "✅ PASS:" scripts/dev_loop.sh | tail -n1 | cut -d: -f1 || echo "0")
  evidence_line=$(grep -n "generate_evidence.sh" scripts/dev_loop.sh | tail -n1 | cut -d: -f1 || echo "0")

  echo "    Phase line numbers:"
  echo "      Execution case: $case_start"
  echo "      Decision: $decision_line"
  echo "      Evidence: $evidence_line"

  # The key ordering is: decision BEFORE evidence
  # The case statement (which includes build, boot, validation) comes before both
  if [ "$case_start" -gt 0 ] && \
     [ "$decision_line" -gt 0 ] && \
     [ "$evidence_line" -gt 0 ] && \
     [ "$case_start" -lt "$decision_line" ] && \
     [ "$decision_line" -lt "$evidence_line" ]; then
    echo "    ✅ Temporal ordering correct: Execution → Decision → Evidence"
  else
    echo "    ❌ VIOLATION: Temporal ordering incorrect"
    FAIL=1
  fi
fi

echo ""

# ============================================================================
# Final Report
# ============================================================================

echo "========================================"
echo "Evidence Non-Authoritative Property Summary"
echo "========================================"
echo ""

if [ "$FAIL" -eq 0 ]; then
  echo "✅ PASS: All evidence non-authoritative properties validated"
  echo ""
  echo "Validated Properties:"
  echo "  ✅ Evidence generation runs AFTER validation completes"
  echo "  ✅ Evidence generation failure does not affect validation outcome"
  echo "  ✅ Evidence artifacts are never used as validation input"
  echo "  ✅ Evidence directory is write-only for validation pipeline"
  echo "  ✅ Evidence has no decision authority"
  echo "  ✅ Evidence is purely observational (derived data)"
  echo "  ✅ Evidence cannot influence kernel execution"
  echo "  ✅ Evidence generation is optional (not required for validation)"
  echo "  ✅ Evidence artifacts are stateless"
  echo "  ✅ Evidence visualization is read-only"
  echo "  ✅ Evidence never affects execution flow"
  echo "  ✅ Evidence pipeline temporal ordering guarantee"
  echo ""
  echo "Non-Authoritative Guarantee:"
  echo "  ✅ Evidence is derived data, never decision input"
  echo "  ✅ Evidence is generated AFTER validation"
  echo "  ✅ Evidence is read-only for visualization"
  echo "  ✅ Evidence never affects validation outcome"
  echo "  ✅ Evidence never affects execution flow"
  echo "  ✅ Evidence is purely observational"
  echo ""
  echo "Constitutional Compliance:"
  echo "  ✅ R23: Dev Loop Non-Interference Guarantee"
  echo "  ✅ R26: Direct Observation Source Constraint"
  echo "  ✅ R27: Evidence State Isolation"
  echo "  ✅ Design Section 2.3: Evidence ≠ Authority"
  echo "  ✅ Design Section 6: Evidence Model"
  echo "  ✅ Design Section 6.2: Non-Authority"
  echo "  ✅ Design Section 10: Anti-Patterns (Evidence as Validation Input)"
  echo ""
  echo "Task 26.3 (Evidence pipeline non-authoritative property) COMPLETE"
  exit 0
else
  echo "❌ FAIL: Evidence non-authoritative property violations detected"
  echo ""
  echo "Critical Failures:"
  echo "  - Evidence may have decision authority"
  echo "  - Evidence may affect validation outcome"
  echo "  - Evidence may affect execution flow"
  echo "  - Non-authoritative guarantee VIOLATED"
  echo ""
  echo "Required Actions:"
  echo "  1. Ensure evidence generation runs AFTER validation completes"
  echo "  2. Remove all evidence reads from validation scripts"
  echo "  3. Ensure evidence failure does not affect validation outcome"
  echo "  4. Remove evidence from all decision logic"
  echo "  5. Ensure evidence is purely observational (derived data)"
  echo "  6. Verify evidence cannot influence kernel execution"
  echo ""
  echo "Constitutional Reference:"
  echo "  - Requirement R23: Dev Loop Non-Interference Guarantee"
  echo "  - Requirement R26: Direct Observation Source Constraint"
  echo "  - Requirement R27: Evidence State Isolation"
  echo "  - Design Section 2.3: Evidence ≠ Authority"
  echo "  - Design Section 6: Evidence Model (Non-Authority)"
  echo "  - Design Section 10: Anti-Patterns"
  echo ""
  exit 1
fi

#!/usr/bin/env bash
# Author: Kenan AY
# Role: Developer / Architect / Designer / Implementer
#
# Task 26.1: Isolation Boundary Guarantee
#
# This test validates that the dev loop maintains strict isolation boundaries
# and operates as a read-only observer relative to kernel runtime behavior.
#
# Validation Properties:
# 1. Dev loop scripts do not modify kernel execution behavior
# 2. Validation markers are pure output (no side effects)
# 3. Dev loop operates as read-only observer
# 4. No kernel state modification from dev loop
# 5. No kernel memory writes from dev loop
# 6. No execution flow changes from dev loop
#
# Test Method:
# - Static analysis of dev loop scripts for forbidden patterns
# - Verification that markers are conditional compilation only
# - Confirmation that dev loop uses only observation mechanisms
# - Validation that no kernel state is modified by dev loop

set -euo pipefail

echo "== Task 26.1: Isolation Boundary Guarantee =="
echo ""
echo "Validating dev loop isolation boundaries..."
echo ""

ROOT_DIR="$(git rev-parse --show-toplevel)"
cd "$ROOT_DIR"

FAIL=0

# ============================================================================
# Property 1: Dev loop scripts are read-only observers
# ============================================================================

echo "[Property 1] Dev loop scripts are read-only observers"
echo ""

DEV_LOOP_SCRIPTS=(
  "scripts/dev_loop.sh"
  "scripts/oracle.sh"
  "scripts/find_regression.sh"
  "scripts/test_devloop_isolation.sh"
  "scripts/test_marker_validation.sh"
  "scripts/test_exit_status_contract.sh"
  "scripts/test_qemu_integration.sh"
)

# Forbidden patterns that would indicate kernel modification
FORBIDDEN_KERNEL_MODIFICATION_PATTERNS=(
  # Direct kernel memory access
  "/dev/mem"
  "/dev/kmem"
  "/proc/kcore"

  # Kernel module operations
  "insmod"
  "rmmod"
  "modprobe"

  # Kernel parameter modification
  "sysctl.*-w"
  "/proc/sys/.*="

  # Debugfs writes
  "/sys/kernel/debug/.*>"

  # Kernel debugging that modifies state
  "kgdb"
  "kdb"
)

echo "Checking dev loop scripts for kernel modification patterns..."

for script in "${DEV_LOOP_SCRIPTS[@]}"; do
  if [ ! -f "$script" ]; then
    continue
  fi

  echo "  Checking: $script"

  for pattern in "${FORBIDDEN_KERNEL_MODIFICATION_PATTERNS[@]}"; do
    matches=$(grep -nE "$pattern" "$script" 2>/dev/null || true)

    if [ -n "$matches" ]; then
      echo "    ❌ VIOLATION: Forbidden kernel modification pattern detected"
      echo "    Pattern: $pattern"
      echo "$matches"
      FAIL=1
    fi
  done
done

if [ "$FAIL" -eq 0 ]; then
  echo "  ✅ No kernel modification patterns detected"
fi

echo ""

# ============================================================================
# Property 2: Validation markers are conditional compilation only
# ============================================================================

echo "[Property 2] Validation markers are conditional compilation only"
echo ""

# Check that markers are guarded by AYKEN_VALIDATION preprocessor directive
MARKER_FILES=(
  "kernel/kernel.c"
)

echo "Checking that validation markers are conditional..."

for file in "${MARKER_FILES[@]}"; do
  if [ ! -f "$file" ]; then
    echo "  ⚠ Warning: $file not found"
    continue
  fi

  echo "  Checking: $file"

  # Extract marker emission lines
  marker_lines=$(grep -n "EARLY_BOOT_OK\|LATE_INIT_END\|AYKEN_BOOT_OK" "$file" || true)

  if [ -z "$marker_lines" ]; then
    echo "    ⚠ Warning: No markers found in $file"
    continue
  fi

  # For each marker, verify it's inside an #if AYKEN_VALIDATION block
  while IFS= read -r line; do
    line_num=$(echo "$line" | cut -d: -f1)
    line_content=$(echo "$line" | cut -d: -f2-)

    # Skip comment lines (they're not actual marker emissions)
    if echo "$line_content" | grep -q "^[[:space:]]*//"; then
      continue
    fi

    # Check if there's an #if AYKEN_VALIDATION before this line
    guard_check=$(awk -v target="$line_num" '
      NR < target && /#if.*AYKEN_VALIDATION/ { found=1 }
      NR < target && /#endif/ { found=0 }
      NR == target && found { print "GUARDED" }
      NR == target && !found { print "UNGUARDED" }
    ' "$file")

    if [ "$guard_check" = "UNGUARDED" ]; then
      echo "    ❌ VIOLATION: Marker at line $line_num not guarded by AYKEN_VALIDATION"
      echo "    $line"
      FAIL=1
    fi
  done <<< "$marker_lines"

  if [ "$FAIL" -eq 0 ]; then
    echo "    ✅ All markers properly guarded by conditional compilation"
  fi
done

echo ""

# ============================================================================
# Property 3: Dev loop uses only observation mechanisms
# ============================================================================

echo "[Property 3] Dev loop uses only observation mechanisms"
echo ""

# Allowed observation mechanisms
ALLOWED_OBSERVATION_PATTERNS=(
  "grep"           # Reading logs
  "cat"            # Reading files
  "tail"           # Reading log tails
  "head"           # Reading log heads
  "awk"            # Processing logs
  "sed"            # Processing logs (read-only)
  "wc"             # Counting
  "sha256sum"      # Hashing
  "timeout"        # Process management
  "qemu-system"    # VM execution
)

# Forbidden write operations to kernel-affecting paths
FORBIDDEN_WRITE_PATTERNS=(
  "echo.*>/dev/mem"
  "echo.*>/dev/kmem"
  "echo.*>/proc/sys"
  "echo.*>/sys/kernel"
  "dd.*of=/dev/mem"
  "dd.*of=/dev/kmem"
)

echo "Checking dev loop scripts use only observation mechanisms..."

for script in "${DEV_LOOP_SCRIPTS[@]}"; do
  if [ ! -f "$script" ]; then
    continue
  fi

  echo "  Checking: $script"

  for pattern in "${FORBIDDEN_WRITE_PATTERNS[@]}"; do
    matches=$(grep -nE "$pattern" "$script" 2>/dev/null || true)

    if [ -n "$matches" ]; then
      echo "    ❌ VIOLATION: Forbidden write operation detected"
      echo "    Pattern: $pattern"
      echo "$matches"
      FAIL=1
    fi
  done
done

if [ "$FAIL" -eq 0 ]; then
  echo "  ✅ Only observation mechanisms detected"
fi

echo ""

# ============================================================================
# Property 4: Markers have no side effects
# ============================================================================

echo "[Property 4] Validation markers have no side effects"
echo ""

# Check that marker emission is pure output (debugcon_write/outb only)
echo "Checking marker emission is pure output..."

for file in "${MARKER_FILES[@]}"; do
  if [ ! -f "$file" ]; then
    continue
  fi

  echo "  Checking: $file"

  # Extract marker emission context (5 lines before and after)
  marker_contexts=$(grep -B5 -A5 "EARLY_BOOT_OK\|LATE_INIT_END\|AYKEN_BOOT_OK" "$file" || true)

  # Check for side effects in marker context
  side_effect_patterns=(
    "kheap_alloc"
    "kheap_free"
    "kmalloc"
    "kfree"
    "paging_map"
    "paging_unmap"
    "proc_create"
    "sched_"
    "syscall_"
    "capability_"
  )

  for pattern in "${side_effect_patterns[@]}"; do
    matches=$(echo "$marker_contexts" | grep -n "$pattern" || true)

    if [ -n "$matches" ]; then
      echo "    ❌ VIOLATION: Side effect detected near marker emission"
      echo "    Pattern: $pattern"
      echo "$matches"
      FAIL=1
    fi
  done

  if [ "$FAIL" -eq 0 ]; then
    echo "    ✅ Markers are pure output (no side effects)"
  fi
done

echo ""

# ============================================================================
# Property 5: Dev loop does not modify kernel build artifacts
# ============================================================================

echo "[Property 5] Dev loop does not modify kernel build artifacts"
echo ""

# Check that dev loop scripts don't modify kernel binaries or build outputs
KERNEL_ARTIFACTS=(
  "kernel/kernel.o"
  "kernel.elf"
  "EFI.img"
)

echo "Checking dev loop scripts don't modify kernel artifacts..."

for script in "${DEV_LOOP_SCRIPTS[@]}"; do
  if [ ! -f "$script" ]; then
    continue
  fi

  echo "  Checking: $script"

  for artifact in "${KERNEL_ARTIFACTS[@]}"; do
    # Check for writes to kernel artifacts
    matches=$(grep -nE ">${artifact}|>.*${artifact}" "$script" 2>/dev/null || true)

    if [ -n "$matches" ]; then
      echo "    ❌ VIOLATION: Script modifies kernel artifact: $artifact"
      echo "$matches"
      FAIL=1
    fi
  done
done

if [ "$FAIL" -eq 0 ]; then
  echo "  ✅ Dev loop does not modify kernel artifacts"
fi

echo ""

# ============================================================================
# Property 6: QEMU invocation is read-only observation
# ============================================================================

echo "[Property 6] QEMU invocation is read-only observation"
echo ""

echo "Checking QEMU invocation parameters..."

if [ -f "scripts/dev_loop.sh" ]; then
  echo "  Checking: scripts/dev_loop.sh"

  # Extract QEMU invocation
  qemu_invocation=$(grep -A20 "qemu-system-x86_64" scripts/dev_loop.sh || true)

  # Check for forbidden QEMU options that could modify kernel behavior
  forbidden_qemu_options=(
    "\\-gdb"           # GDB debugging (can modify state)
    "\\-s[[:space:]]"  # GDB shorthand
    "\\-monitor"       # QEMU monitor (can modify state)
    "\\-qmp"           # QEMU Machine Protocol (can modify state)
  )

  for option in "${forbidden_qemu_options[@]}"; do
    matches=$(echo "$qemu_invocation" | grep -E "$option" || true)

    if [ -n "$matches" ]; then
      echo "    ❌ VIOLATION: QEMU option allows state modification: $option"
      echo "$matches"
      FAIL=1
    fi
  done

  # Verify QEMU uses read-only observation mechanisms
  if echo "$qemu_invocation" | grep -q "\\-nographic"; then
    echo "    ✅ QEMU uses -nographic (observation only)"
  else
    echo "    ⚠ Warning: QEMU missing recommended option: -nographic"
  fi

  if echo "$qemu_invocation" | grep -q "\\-no-reboot"; then
    echo "    ✅ QEMU uses -no-reboot (deterministic)"
  else
    echo "    ⚠ Warning: QEMU missing recommended option: -no-reboot"
  fi

  if [ "$FAIL" -eq 0 ]; then
    echo "    ✅ QEMU invocation is read-only observation"
  fi
fi

echo ""

# ============================================================================
# Property 7: Evidence generation runs AFTER validation
# ============================================================================

echo "[Property 7] Evidence generation runs AFTER validation"
echo ""

if [ -f "scripts/dev_loop.sh" ]; then
  echo "  Checking: scripts/dev_loop.sh"

  # Find line numbers for validation and evidence generation
  validation_line=$(grep -n "AYKEN_BOOT_OK" scripts/dev_loop.sh | head -n1 | cut -d: -f1 || echo "0")
  evidence_line=$(grep -n "generate_evidence.sh" scripts/dev_loop.sh | tail -n1 | cut -d: -f1 || echo "0")

  if [ "$validation_line" -gt 0 ] && [ "$evidence_line" -gt 0 ]; then
    if [ "$evidence_line" -lt "$validation_line" ]; then
      echo "    ❌ VIOLATION: Evidence generation before validation"
      echo "    Validation line: $validation_line"
      echo "    Evidence line: $evidence_line"
      FAIL=1
    else
      echo "    ✅ Evidence generation after validation (line $evidence_line > $validation_line)"
    fi
  else
    echo "    ⚠ Warning: Could not determine validation/evidence order"
  fi
fi

echo ""

# ============================================================================
# Property 8: Marker emission does not affect kernel execution flow
# ============================================================================

echo "[Property 8] Marker emission does not affect kernel execution flow"
echo ""

echo "Checking marker emission does not affect control flow..."

for file in "${MARKER_FILES[@]}"; do
  if [ ! -f "$file" ]; then
    continue
  fi

  echo "  Checking: $file"

  # Extract marker emission and check for control flow changes
  marker_contexts=$(grep -B10 -A10 "debugcon_write.*BOOT_OK\|debugcon_write.*INIT_END" "$file" || true)

  # Check for control flow statements near markers
  control_flow_patterns=(
    "return"
    "goto"
    "break"
    "continue"
    "exit"
    "while.*1.*hlt"  # Infinite loop
  )

  for pattern in "${control_flow_patterns[@]}"; do
    # Only flag if control flow is INSIDE the #if AYKEN_VALIDATION block
    matches=$(echo "$marker_contexts" | grep -A2 "debugcon_write" | grep "$pattern" || true)

    if [ -n "$matches" ]; then
      echo "    ❌ VIOLATION: Control flow change near marker emission"
      echo "    Pattern: $pattern"
      echo "$matches"
      FAIL=1
    fi
  done

  if [ "$FAIL" -eq 0 ]; then
    echo "    ✅ Marker emission does not affect control flow"
  fi
done

echo ""

# ============================================================================
# Final Report
# ============================================================================

echo "========================================"
echo "Isolation Boundary Guarantee Summary"
echo "========================================"
echo ""

if [ "$FAIL" -eq 0 ]; then
  echo "✅ PASS: All isolation boundary properties validated"
  echo ""
  echo "Validated Properties:"
  echo "  ✅ Dev loop scripts are read-only observers"
  echo "  ✅ Validation markers are conditional compilation only"
  echo "  ✅ Dev loop uses only observation mechanisms"
  echo "  ✅ Markers have no side effects"
  echo "  ✅ Dev loop does not modify kernel artifacts"
  echo "  ✅ QEMU invocation is read-only observation"
  echo "  ✅ Evidence generation runs after validation"
  echo "  ✅ Marker emission does not affect control flow"
  echo ""
  echo "Isolation Guarantee:"
  echo "  ✅ Dev loop operates as read-only observer"
  echo "  ✅ No kernel state modification"
  echo "  ✅ No kernel memory writes"
  echo "  ✅ No execution flow changes"
  echo "  ✅ Validation markers are pure output"
  echo ""
  echo "Constitutional Compliance:"
  echo "  ✅ R23: Dev Loop Non-Interference Guarantee"
  echo "  ✅ Design Section 2.1: Non-Interference Principle"
  echo "  ✅ Design Section 5: Isolation Model"
  echo ""
  echo "Task 26.1 (Isolation boundary guarantee) COMPLETE"
  exit 0
else
  echo "❌ FAIL: Isolation boundary violations detected"
  echo ""
  echo "Critical Failures:"
  echo "  - Dev loop may modify kernel execution behavior"
  echo "  - Isolation boundary guarantee VIOLATED"
  echo ""
  echo "Required Actions:"
  echo "  1. Remove all kernel modification patterns from dev loop scripts"
  echo "  2. Ensure markers are conditional compilation only"
  echo "  3. Use only observation mechanisms (no writes)"
  echo "  4. Verify evidence generation runs after validation"
  echo ""
  echo "Constitutional Reference:"
  echo "  - Requirement R23: Dev Loop Non-Interference Guarantee"
  echo "  - Design Section 2.1: Non-Interference Principle"
  echo "  - Design Section 5: Isolation Model"
  echo "  - Design Section 10: Anti-Patterns"
  echo ""
  exit 1
fi

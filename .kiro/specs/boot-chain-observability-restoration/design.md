# Boot Chain Observability Restoration Bugfix Design

**Author**: Kenan AY - Architectural Steward  
**Created**: 2026-04-11  
**Status**: Implementation Ready

## Overview

The boot chain observability evidence pipeline in AykenOS has regressed, preventing QEMU/kernel trace capture for boot-trace dependent validations. Historical evidence confirms boot markers were previously captured (`ApqiggggIBK0[K][EARLY_BOOT_OK]`), indicating this is a **regression in the evidence capture path** rather than a fundamental debugcon failure.

The root cause is **EVIDENCE PIPELINE FAILURE** - a combination of harness integrity issues (destructive `sort` operation), QEMU configuration drift, and capture path regression. This is NOT "debugcon doesn't work" - it's a **non-architectural bugfix** to restore evidence collection under architectural freeze constraints.

This design follows the bug condition methodology: identify inputs that trigger evidence pipeline failure (C), define expected observable behavior (P), and preserve existing boot functionality (¬C). The fix is scoped as evidence pipeline integrity repair, not a Phase-16 blocker or architectural change.

## Glossary

- **Bug_Condition (C)**: The condition that triggers observability failure - when boot markers are emitted but no output channel captures them
- **Property (P)**: The desired behavior for C(X) - at least one output channel SHALL capture deterministic boot markers in correct order
- **Preservation**: Existing boot functionality (kernel initialization, subsystem setup, Phase-16 tests) that must remain unchanged
- **Output Channel**: A mechanism for capturing boot trace data (debugcon port 0xE9, serial port 0x3F8, or UEFI Print)
- **Debugcon**: QEMU's debug console (port 0xE9) - primary output channel
- **Serial**: COM1 serial port (port 0x3F8) - fallback output channel
- **UEFI Print**: UEFI firmware console output - bootloader-only channel
- **Marker**: Deterministic boot trace identifier (e.g., `[[AYKEN_BOOT_OK]]`, `[B][UEFI_BOOT_START]`)
- **Fail-Closed Proof**: Security property requiring marker order preservation (no sort allowed)
- **Channel Integrity**: Proof that an output channel works before relying on it for verification
- **efi_main**: Bootloader entry point in `bootloader/efi/efi_main.c`
- **kmain_real**: C kernel entry function in `kernel/kernel.c` (NOT the actual entry point - see entry stub)
- **ayken_jump_to_kernel**: Bootloader function that transfers control to kernel
- **QEMU Harness**: Test script that launches QEMU and captures output logs

## Bug Details

### Bug Condition

The bug manifests when the bootloader and kernel emit markers to output channels (debugcon port 0xE9, serial port 0x3F8) but no output is captured in QEMU log files. The system cannot prove kernel entry occurred, blocking Phase-16 verification.

**Formal Specification:**
```
FUNCTION isBugCondition(input)
  INPUT: input of type BootExecution
  OUTPUT: boolean
  
  // Two distinct failure modes:
  // 1. HARD FAILURE: All channels zero (complete observability loss)
  // 2. INSUFFICIENT EVIDENCE: Boot completed but required kernel markers absent
  
  hard_failure := (input.debugcon_log_size == 0 
                   AND input.serial_log_size == 0 
                   AND input.uefi_output_size == 0)
                  AND input.qemu_boot_completed == true
  
  insufficient_evidence := (input.qemu_boot_completed == true)
                           AND (input.debugcon_log_size > 0 OR input.serial_log_size > 0 OR input.uefi_output_size > 0)
                           AND (input.kernel_marker_present == false)
  
  RETURN hard_failure OR insufficient_evidence
END FUNCTION
```

### Examples

- **Bootloader Start**: Bootloader calls `debugcon_write("[B][UEFI_BOOT_START]")` but debugcon log remains 0 bytes
- **Kernel Entry**: Kernel executes `outb(0xE9, 'K')` at entry point but no 'K' appears in any log
- **Marker Emission**: Code contains `debugcon_write("[[AYKEN_BOOT_OK]]")` but grep finds no matches in QEMU output
- **Edge Case**: QEMU runs with `-debugcon file:log.txt` but file is created empty (channel misconfigured)

### Root Cause Analysis

Based on historical evidence showing boot markers were previously captured, the most likely issues are:

1. **Evidence Pipeline Integrity Failure (MOST CRITICAL)**
   - Harness scripts use destructive `sort` operation: `cat "$DEBUGCON_LOG" "$SERIAL_LOG" | sort`
   - `sort` destroys marker order, breaking fail-closed proof (order = security property)
   - This is NOT a symptom - it's **direct evidence tampering**
   - Even if output exists, sorted trace cannot prove deterministic execution order
   - **PRIORITY 1**: Remove sort, preserve append-order trace

2. **QEMU Configuration Drift (LIKELY)**
   - QEMU flags may have changed since last successful capture
   - `-debugcon file:$LOG -global isa-debugcon.iobase=0xe9` may not be active
   - OVMF combination may have changed
   - stdout/stderr redirect may be swallowing logs
   - **PRIORITY 2**: Validate QEMU launch configuration

3. **Bootloader Marker Path Regression (POSSIBLE)**
   - Marker code in `efi_main.c` may not be reaching build
   - Optimization/link stage may be dropping marker functions
   - Marker code may execute before `InitializeLib` completes
   - **PRIORITY 3**: Verify marker code in bootloader build

4. **Kernel Entry Path Regression (LESS LIKELY given historical evidence)**
   - Bootloader jump may be failing
   - Entry stub may not be executing
   - Entry stub vs `kmain_real` distinction may be unclear (entry stub is actual entry, not C function)
   - **PRIORITY 4**: Verify kernel entry path

## Expected Behavior

### Preservation Requirements

**Unchanged Behaviors:**
- Kernel initialization (paging, heap, memory management) must continue to work exactly as before
- Existing Phase-16 tests must continue to execute without breaking due to boot chain changes
- Build system must continue to produce valid ELF and EFI binaries
- QEMU must continue to boot without hanging or crashing

**Scope:**
All inputs that do NOT involve boot observability evidence collection (normal kernel execution, subsystem initialization, Ring3 process creation) should be completely unaffected by this fix. This includes:
- Kernel subsystem initialization (scheduler, syscalls, capabilities)
- Memory management operations (physical memory, paging, heap)
- Process creation and Ring3 transitions
- Existing validation tests (ELF parser, alias proof, Phase-10 prerequisites)

**Architectural Freeze Compliance:**
This fix MUST remain within non-architectural bugfix boundaries:
- NO new syscalls, execution layers, or contracts
- NO changes to Ring0/Ring3 boundary, BCIB/CLI contracts, or kernel policy
- NO phase transition claims or Phase-16 architectural expansion
- ONLY evidence pipeline repair, harness integrity fixes, and capture path restoration

## Correctness Properties

Property 1: Bug Condition - Output Channel Integrity

_For any_ boot execution where the bootloader and kernel emit markers to output channels, at least one channel (debugcon, serial, or UEFI console) SHALL capture those markers in a readable log file, proving that the output channel works before any kernel-level verification claims are made.

**Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7**

Property 2: Preservation - Boot Functionality Unchanged

_For any_ kernel initialization sequence that does NOT involve boot observability markers, the fixed code SHALL produce exactly the same behavior as the original code, preserving all subsystem initialization, memory management, and process creation functionality.

**Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6**

## Fix Implementation

### Changes Required

Assuming our root cause analysis is correct (evidence pipeline failure + QEMU config drift + capture path regression):

**Phase 1: Prove Channel Works (Minimal Reproducible Test)**

**File**: `bootloader/efi/efi_main.c`

**Function**: `efi_main`

**Specific Changes**:
1. **Add Direct Port Write Test**: Immediately after `InitializeLib`, write single character 'B' directly to port 0xE9 (bypass UEFI Print)
   - Use inline assembly: `__asm__ volatile("outb %0, %1" : : "a"('B'), "Nd"(0xE9));`
   - This proves bootloader can write to debugcon before any complex operations
   
2. **Add Serial Port Test**: Write 'B' to COM1 (port 0x3F8) as fallback channel
   - **CRITICAL**: Initialize COM1 deterministically BEFORE write
   - Use polling mode (NO interrupts)
   - Safe FIFO config: disable interrupts, set baud rate, enable FIFO
   - Poll TX empty bit before write to prevent silent fail
   - **RATIONALE**: Non-deterministic serial init causes silent fail (debugcon fail + serial init fail = 0 byte log)
   - Write test character to prove serial works

3. **Keep UEFI Print**: Maintain existing `Print(L"...")` calls for firmware console visibility

**File**: `kernel/arch/x86_64/entry.S` (or equivalent entry stub)

**Function**: Real kernel entry point (NOT `kmain_real`)

**Specific Changes**:
1. **Add Entry Stub Test**: At the very first instruction of kernel entry stub, write 'K' to port 0xE9
   - Use assembly (NOT C): `mov al, 'K'; out 0xE9, al`
   - **RATIONALE**: C function `kmain_real` is NOT deterministic entry point
   - Compiler can optimize, add prologue, or reorder instructions
   - Stack setup before C code may cause undefined behavior
   - Real entry stub (_start / kernel_entry) is deterministic and compiler-independent
   - This proves **actual kernel entry** was reached, not just C function

2. **Add Serial Fallback**: Write 'K' to COM1 (port 0x3F8) immediately after debugcon write
   - Use assembly: `mov al, 'K'; mov dx, 0x3F8; out dx, al`
   - **NOTE**: This is **best-effort fallback** without deterministic serial init
   - **Authoritative kernel-first proof**: debugcon byte + later initialized serial path
   - Entry stub serial write may not appear if serial init incomplete
   - Ensures at least one channel captures kernel entry marker

**Phase 2: Add Deterministic Markers**

**File**: `bootloader/efi/efi_main.c`

**Specific Changes**:
1. **Bootloader Start Marker**: After InitializeLib, emit `[B][UEFI_BOOT_START]\n` to both debugcon and serial
2. **Kernel ELF Loaded Marker**: After successful `elf_load_kernel`, emit `[B][KERNEL_ELF_LOADED]\n`
3. **Jump Marker**: Before `ayken_jump_to_kernel`, emit `[B][JUMP_NOW]\n`
4. **Dual-Channel Emission**: Every marker goes to BOTH debugcon (port 0xE9) AND serial (port 0x3F8)

**File**: `kernel/kernel.c`

**Specific Changes**:
1. **Keep Existing Markers**: Preserve `[[AYKEN_BOOT_OK]]` and `[K][EARLY_BOOT_OK]` markers
2. **Add Dual-Channel Helper**: Create `dual_channel_write(const char *s)` that writes to both debugcon and serial
3. **Replace Single-Channel Calls**: Update all `debugcon_write()` calls to use `dual_channel_write()`

**Phase 3: Fix Harness (CRITICAL for Fail-Closed Proof)**

**File**: `scripts/qemu-fail-closed-proof-harness.sh`

**Specific Changes**:
1. **Remove Cross-Channel Merge**: Do NOT concatenate debugcon and serial logs
   - Current: `cat "$DEBUGCON_LOG" "$SERIAL_LOG" > "$TRACE"` creates fake temporal ordering
   - Correct: Keep separate traces: `TRACE_DEBUGCON=debugcon.trace`, `TRACE_SERIAL=serial.trace`
   - Analysis: `grep marker $TRACE_DEBUGCON || grep marker $TRACE_SERIAL`
   - **RATIONALE**: Debugcon and serial are NOT on same time axis; merge creates false ordering
   - Fail-closed systems require **channel-local truth**, not cross-channel merge

2. **Add Channel Integrity Validation (HARD FAIL)**:
   - Before analyzing markers, verify at least one log file is non-empty
   - **RULE**: IF debugcon == 0 AND serial == 0 AND uefi == 0 → HARD FAIL → STOP ALL VALIDATION
   - Fail with "OUTPUT_CHANNEL_FAILURE" and exit immediately
   - This makes observability failure explicit rather than silent
   - **CI GATE**: `stat -c%s "$DEBUGCON_LOG"` must be > 0 OR serial > 0 OR uefi output exists

3. **Strengthen Regression Lock**:
   - FORBIDDEN operations: `sort`, `uniq`, `reorder`, `grep -o` (order loss), `awk` reorder, multiline buffer
   - REQUIRED: Raw append-order trace only
   - **CI GATE**: Any forbidden operation → FAIL

4. **Add Minimal Test Mode**: Support `-debugcon stdio` for immediate visual feedback during development
   - **DIAGNOSTIC MODE ONLY**: stdio-based capture is NOT authoritative evidence
   - **AUTHORITATIVE MODE**: file-based capture (`-debugcon file:$LOG`) for CI gates
   - stdio and file capture may have different behavior (buffering, timing)
   - Use stdio for development/debugging, file for production evidence
   - Allows developer to see output in real-time before switching to file capture

**File**: `scripts/qemu-runtime-bridge-proof-harness.sh`

**Specific Changes**:
1. **Remove Sort**: Same as fail-closed harness - preserve marker order
2. **Add Channel Validation**: Verify output channels work before analyzing markers

**Phase 4: QEMU Configuration Validation**

**File**: `Makefile` (or wherever QEMU is invoked)

**Specific Changes**:
1. **Verify QEMU Flags**: Ensure `-debugcon file:$LOG -global isa-debugcon.iobase=0xe9` is present
2. **Add Serial Capture**: Add `-serial file:$SERIAL_LOG` to capture COM1 output
3. **Use Minimal Test Config**: For initial testing, use `-debugcon stdio` to see immediate output

## Testing Strategy

### Validation Approach

The testing strategy follows a three-phase approach: first, prove output channels work with minimal tests; second, verify deterministic markers appear in correct order; third, validate that fail-closed proof is preserved.

### Exploratory Bug Condition Checking

**Goal**: Surface counterexamples that demonstrate the bug BEFORE implementing the fix. Confirm or refute the root cause analysis. If we refute, we will need to re-hypothesize.

**Test Plan**: Run QEMU with minimal test configuration (`-debugcon stdio`) and observe whether single-character writes appear. Run on UNFIXED code to understand root cause.

**Test Cases**:
1. **Bootloader Debugcon Test**: Add `outb('B', 0xE9)` at start of `efi_main`, run QEMU with `-debugcon stdio` (diagnostic mode) (will fail if debugcon misconfigured)
2. **Kernel Entry Test**: Add `outb('K', 0xE9)` at start of **entry stub** (NOT `kmain_real`), check if 'K' appears (will fail if kernel entry not reached)
3. **Serial Fallback Test**: Write 'B' to COM1 in bootloader, check serial log (will fail if serial also broken)
4. **UEFI Print Test**: Verify existing `Print(L"...")` calls appear in QEMU stdout (may succeed even if debugcon fails)

**Expected Counterexamples**:
- Debugcon log remains 0 bytes even with direct port writes
- Possible causes: OVMF doesn't route port 0xE9, QEMU debugcon device not active, output buffered and never flushed

### Fix Checking

**Goal**: Verify that for all inputs where the bug condition holds (boot execution with marker emission), the fixed system produces observable output in at least one channel.

**Pseudocode:**
```
FOR ALL boot_execution WHERE isBugCondition(boot_execution) DO
  result := run_qemu_with_fixed_code(boot_execution)
  ASSERT (result.debugcon_log_size > 0 OR result.serial_log_size > 0)
  ASSERT result.contains_marker("[B][UEFI_BOOT_START]")
  ASSERT result.contains_marker("[[AYKEN_BOOT_OK]]")
  ASSERT result.marker_order_preserved == true
END FOR
```

### Preservation Checking

**Goal**: Verify that for all inputs where the bug condition does NOT hold (normal kernel execution without observability requirements), the fixed code produces the same result as the original code.

**Pseudocode:**
```
FOR ALL kernel_operation WHERE NOT isBugCondition(kernel_operation) DO
  ASSERT original_kernel(kernel_operation) = fixed_kernel(kernel_operation)
END FOR
```

**Testing Approach**: Property-based testing is recommended for preservation checking because:
- It generates many test cases automatically across the input domain
- It catches edge cases that manual unit tests might miss
- It provides strong guarantees that behavior is unchanged for all non-observability operations

**Test Plan**: Run existing Phase-16 validation tests (ELF parser, alias proof, Phase-10 prerequisites) on UNFIXED code to capture baseline behavior, then verify same tests pass on FIXED code with identical results.

**Test Cases**:
1. **Kernel Initialization Preservation**: Verify paging, heap, memory management work identically after fix
2. **Subsystem Initialization Preservation**: Verify scheduler, syscalls, capabilities initialize correctly
3. **Phase-16 Test Preservation**: Verify existing validation tests (ELF parser, alias proof) produce same results
4. **Build System Preservation**: Verify ELF and EFI binaries are byte-identical (except for marker code changes)

### Unit Tests

- Test direct port write to debugcon (port 0xE9) in bootloader context
- Test direct port write to serial (port 0x3F8) in bootloader context
- Test kernel entry marker emission at naked entry point
- Test dual-channel write helper function
- Test harness marker extraction without sort

### Property-Based Tests

- Generate random boot sequences and verify at least one channel captures markers
- Generate random marker emission patterns and verify order preservation in harness
- Test that all non-observability kernel operations produce identical results across many scenarios

### Integration Tests

- Test full boot flow with QEMU `-debugcon stdio` (visual confirmation)
- Test full boot flow with QEMU `-debugcon file:log.txt` (file capture)
- Test fail-closed proof harness with fixed marker order preservation
- Test runtime-bridge proof harness with fixed marker order preservation
- Test that Phase-16 verification tasks can now locate kernel trace markers

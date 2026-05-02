# Implementation Plan

**Author**: Kenan AY - Architectural Steward  
**Created**: 2026-04-11  
**Status**: Ready for Implementation

## Overview

This implementation plan restores boot chain observability evidence pipeline following the 4-block strategy: (1) Evidence Pipeline Repair, (2) Channel Proof, (3) Deterministic Boot Marker Chain, (4) Regression Lock. The fix is scoped as a non-architectural bugfix under architectural freeze constraints.

**Historical Context:** Boot markers were previously captured (`ApqiggggIBK0[K][EARLY_BOOT_OK]`), confirming this is a regression in evidence capture path, not a fundamental debugcon failure.

**Root Cause Priority:**
1. Evidence pipeline integrity failure (harness `sort` destroys marker order)
2. QEMU configuration drift (debugcon/serial capture path)
3. Bootloader marker path regression (marker code not reaching build)
4. Kernel entry path regression (less likely given historical evidence)

**Architectural Freeze Compliance:** NO new syscalls, execution layers, or contracts. ONLY evidence pipeline repair.

## Tasks

- [x] 1. Block 1: Evidence Pipeline Repair (PRIORITY 1 - CRITICAL)
  - **GOAL**: Fix harness integrity BEFORE any code changes
  - **RATIONALE**: Cross-channel merge and `sort` are direct evidence tampering, not symptoms
  - Fix `scripts/qemu-fail-closed-proof-harness.sh`:
    - **Remove cross-channel merge**: Do NOT concatenate debugcon and serial logs
    - Current: `cat "$DEBUGCON_LOG" "$SERIAL_LOG" > "$TRACE"` creates fake temporal ordering
    - Correct: Keep separate traces: `TRACE_DEBUGCON=debugcon.trace`, `TRACE_SERIAL=serial.trace`
    - Analysis: `grep marker $TRACE_DEBUGCON || grep marker $TRACE_SERIAL`
    - **RATIONALE**: Debugcon and serial are NOT on same time axis; merge creates false ordering
    - Fail-closed systems require **channel-local truth**, not cross-channel merge
    - **Add HARD FAIL rule**: IF debugcon == 0 AND serial == 0 AND uefi == 0 → HARD FAIL → STOP ALL VALIDATION
    - Fail with "OUTPUT_CHANNEL_FAILURE" and exit immediately
    - **CI GATE**: `stat -c%s "$DEBUGCON_LOG"` must be > 0 OR serial > 0 OR uefi output exists
    - **Strengthen regression lock**: FORBIDDEN operations: `sort`, `uniq`, `reorder`, `grep -o`, `awk` reorder, multiline buffer
  - Fix `scripts/qemu-runtime-bridge-proof-harness.sh`:
    - Apply same fixes: no cross-channel merge, channel-local analysis, HARD FAIL rule
  - Validate QEMU launch configuration:
    - Verify `-debugcon file:$LOG -global isa-debugcon.iobase=0xe9` is active
    - Verify `-serial file:$SERIAL_LOG` captures COM1 output
    - Check for stdout/stderr redirect that may swallow logs
  - **EXPECTED OUTCOME**: Harness preserves raw append-order trace per channel, validates channel integrity with HARD FAIL
  - **ARCHITECTURAL FREEZE**: No code changes, only harness/config repair
  - _Requirements: 2.6, 2.7, 3.7_

- [x] 2. Block 2: Bootloader Execution Proof (PRIORITY 2) ✅ COMPLETED
  - **GOAL**: Establish that efi_main() executes BEFORE testing port capture
  - **STATUS**: ✅ EXECUTION PROVEN - efi_main() executes, debugcon works
  - **Evidence**: Debugcon successfully captures bootloader output (1476+ bytes confirmed):
    - `B[B][UEFI_BOOT_START] efi_main entry`
    - `[B][INIT_LIB_OK]`
    - `[B][MEMMAP] status=0x0000000000000000`
    - `[B][GOP] status=0x0000000000000000`
    - `[B][ELF_MAGIC_OK]`
    - `[B][KERNEL_OPEN_OK]`
    - `[B][ELF_HDR_READ_OK]`
  - **Working Test**: `test_uefi_console_capture.sh` successfully captures debugcon output
  - **Key Finding**: OVMF does NOT route ConOut to serial, but debugcon (port 0xE9) works perfectly
  - **Acceptance Criteria MET**:
    - [✓] Test 1: BOOTX64.EFI structure valid (PE32+ header confirmed)
    - [✓] Test 3: Debugcon capture working (32KB+ output with markers)
    - [✓] efi_main() execution proven beyond doubt
  - **Conclusion**: Block 2 complete. Execution proof established. Ready for Block 3.
  - **ARCHITECTURAL FREEZE**: Minimal port writes only, no new contracts
  - _Requirements: 2.1, 2.4, 2.7, 3.7_

- [x] 3. Block 3: Deterministic Boot Marker Chain (PRIORITY 3)
  - **GOAL**: Restore full marker chain AFTER channel proof succeeds
  - **RATIONALE**: Semantic markers only make sense if channels work
  - Bootloader markers (`bootloader/efi/efi_main.c`):
    - After InitializeLib: emit "[B][UEFI_BOOT_START]\n" to debugcon and serial
    - After `elf_load_kernel`: emit "[B][KERNEL_ELF_LOADED]\n"
    - Before `ayken_jump_to_kernel`: emit "[B][JUMP_NOW]\n"
  - Kernel markers (`kernel/kernel.c`):
    - Keep existing "[[AYKEN_BOOT_OK]]" and "[K][EARLY_BOOT_OK]" markers
    - Create `dual_channel_write(const char *s)` helper for debugcon + serial
    - Replace single-channel `debugcon_write()` calls with `dual_channel_write()`
  - Channel strategy:
    - Debugcon (port 0xE9) - primary
    - Serial (COM1 port 0x3F8) - duplicate/fallback
    - UEFI Print - **ground truth fallback** (bootloader-only)
  - **UEFI Print Role**: If debugcon=FAIL AND serial=FAIL BUT uefi=OK → problem is QEMU capture path, NOT bootloader execution
  - **Test Matrix**:
    - UEFI=OK, debugcon=FAIL → capture bug (QEMU config drift)
    - UEFI=FAIL → bootloader execution bug (handoff failure)
  - Marker helper requirements:
    - Pure, small, side-effect free
    - NO touching paging, ELF mapping, higher-half flow
  - **EXPECTED OUTCOME**: Full marker chain appears in correct order
  - **ARCHITECTURAL FREEZE**: Marker emission only, no new execution layers
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 3.7_

- [x] 4. Block 4: Regression Lock (PRIORITY 4)
  - **GOAL**: Prevent future evidence pipeline regressions
  - **RATIONALE**: Lock down evidence integrity requirements with CI gates
  - Create QEMU boot audit script with CI gates:
    - **CI GATE**: Marker absent → FAIL
    - **CI GATE**: Marker order broken → FAIL
    - **CI GATE**: Zero-byte log (debugcon AND serial AND uefi all 0) → HARD FAIL
    - **CI GATE**: `stat -c%s "$DEBUGCON_LOG"` must be > 0 OR serial > 0 OR uefi output exists
    - **CI GATE**: Forbidden operations detected → FAIL
  - Forbidden operations (explicit list):
    - `sort`, `uniq`, `reorder` - destroys temporal order
    - `grep -o` - loses line context and order
    - `awk` with reorder logic - creates fake ordering
    - Multiline buffer operations - can reorder or drop lines
    - Cross-channel concatenation - creates fake temporal ordering
  - Document evidence pipeline requirements:
    - Raw append-order trace mandatory (per channel)
    - Channel-local detection (no fake temporal ordering across channels)
    - At least one channel must capture markers (HARD FAIL if all zero)
    - UEFI Print as ground truth fallback for bootloader execution
  - **EXPECTED OUTCOME**: Automated regression detection with CI enforcement
  - **ARCHITECTURAL FREEZE**: Audit/validation only, no architectural changes
  - _Requirements: 2.6, 2.7, 3.7_

- [x] 5. Write bug condition exploration test
  - **Property 1: Bug Condition** - Evidence Pipeline Integrity Failure
  - **CRITICAL**: This test MUST FAIL on unfixed code - failure confirms the bug exists
  - **DO NOT attempt to fix the test or the code when it fails**
  - Test that QEMU boot with marker emission produces non-empty debugcon OR serial logs
  - Verify bootloader marker "[B][UEFI_BOOT_START]" appears in at least one output channel
  - Verify kernel marker "[[AYKEN_BOOT_OK]]" appears in at least one output channel
  - Verify marker order is preserved (no sort operation in harness)
  - Run test on UNFIXED code
  - **EXPECTED OUTCOME**: Test FAILS (confirms evidence pipeline regression)
  - Document counterexamples: which channels are empty, which markers are missing, whether sort is present
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6_

- [x] 6. Write preservation property tests (BEFORE implementing fix)
  - **Property 2: Preservation** - Boot Functionality Unchanged
  - **IMPORTANT**: Follow observation-first methodology
  - Test kernel initialization (paging, heap, memory management) works identically
  - Test subsystem initialization (scheduler, syscalls, capabilities) works correctly
  - Test existing validation tests (ELF parser, alias proof) produce same results
  - Test build system produces valid ELF and EFI binaries
  - Verify architectural freeze compliance: NO new syscalls, layers, or contracts
  - Run tests on UNFIXED code
  - **EXPECTED OUTCOME**: Tests PASS (confirms baseline behavior to preserve)
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7_

- [x] 7. Verify bug condition exploration test now passes
  - **Property 1: Expected Behavior** - Evidence Pipeline Integrity Restored
  - **IMPORTANT**: Re-run the SAME test from task 5 - do NOT write a new test
  - Run bug condition exploration test from step 5
  - **EXPECTED OUTCOME**: Test PASSES (confirms evidence pipeline works)
  - Verify debugcon OR serial log is non-empty
  - Verify "[B][UEFI_BOOT_START]" appears in logs
  - Verify "[[AYKEN_BOOT_OK]]" appears in logs
  - Verify marker order is preserved (no sort)
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7_

- [x] 8. Verify preservation tests still pass
  - **Property 2: Preservation** - Boot Functionality Unchanged
  - **IMPORTANT**: Re-run the SAME tests from task 6 - do NOT write new tests
  - Run preservation property tests from step 6
  - **EXPECTED OUTCOME**: Tests PASS (confirms no regressions)
  - Verify kernel initialization works identically
  - Verify subsystem initialization unchanged
  - Verify existing validation tests produce same results
  - Verify architectural freeze compliance maintained

- [x] 9. Checkpoint - Ensure all tests pass
  - Verify bug condition test passes (evidence pipeline works)
  - Verify preservation tests pass (no regressions, freeze compliance)
  - Verify boot-trace dependent validations can now locate kernel trace markers
  - **CRITICAL**: Review `.kiro/specs/boot-chain-observability-restoration/design.md` for implementation guidance
  - Ensure all design requirements are met (channel-local analysis, entry stub, deterministic serial init, HARD FAIL rule)
  - **VERIFICATION**: Confirm all spec documents (bugfix.md, design.md, tasks.md) authored by Kenan AY are followed
  - Ensure all tests pass, ask the user if questions arise

---

**Spec Completion**  
**Author**: Kenan AY - Architectural Steward  
**Date**: 2026-04-11  
**Status**: Implementation Ready - All requirements, design, and tasks finalized

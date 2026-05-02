# Userspace Payload Authority Drift Bugfix Design

**Author**: Kenan AY - Architectural Steward  
**Created**: 2026-04-11  
**Status**: Implementation Ready

## Overview

The AykenOS kernel build pipeline exhibits authority drift between the requested userspace payload mode and the actually embedded/booted payload. The build system has multiple authority sources (USER_MINIMAL_MODE, MINIMAL_MODE, embedded header generation) that fall back to defaults silently, causing execution proofs to run against the wrong payload. This makes userspace execution debugging results unreliable.

The root cause is **AUTHORITY FRAGMENTATION** - the build chain lacks a single source of authority for payload selection, with no deterministic locking mechanism or hash verification at build-time or boot-time. The fix establishes a single authority chain: build system selection → embedded header generation → boot verification, with hard fail on any mismatch.

This design follows the bug condition methodology: identify inputs that trigger authority drift (C), define expected deterministic behavior (P), and preserve existing build functionality (¬C). The fix is scoped as build chain integrity repair under architectural freeze constraints.

## Glossary

- **Bug_Condition (C)**: The condition that triggers authority drift - when the build system compiles with a requested payload mode but embeds a different payload
- **Property (P)**: The desired behavior for C(X) - the build system SHALL enforce `build_selected_mode == embedded_elf.h_SHA == boot_emitted_mode_hash` as a correctness invariant
- **Preservation**: Existing build functionality (phase10a2 default, kernel compilation, EFI image generation) that must remain unchanged
- **Authority Source**: A mechanism that determines which userspace payload is embedded (USER_MINIMAL_MODE, MINIMAL_MODE, embedded header)
- **Single Source of Authority**: The build system's USER_MINIMAL_MODE variable as the sole determinant of payload selection
- **Mode Authority Invariant**: `build_selected_mode == embedded_elf_mode == boot_emitted_mode` - mode string must be consistent across build/embed/boot
- **Payload Integrity Invariant**: `expected_payload_sha == embedded_elf_sha == boot_emitted_sha` - payload hash must match across build/embed/boot
- **USER_MINIMAL_MODE**: Makefile variable that selects userspace payload mode (phase10a2, entry-proof, runtime-bridge-test)
- **MINIMAL_MODE**: Userspace Makefile variable that receives USER_MINIMAL_MODE value
- **embedded_elf.h**: Auto-generated C header containing embedded payload bytes and SHA256 hash
- **tools/embed_elf.py**: Python script that generates embedded_elf.h from userspace ELF
- **DAYKEN_USER_MINIMAL_MODE_STRING**: C preprocessor define emitted to build log
- **Boot Marker**: Debugcon output emitted by kernel at boot to verify embedded payload

## Bug Details

### Bug Condition

The bug manifests when the build system compiles the kernel with a requested userspace payload mode (entry-proof or runtime-bridge) but the build log shows `DAYKEN_USER_MINIMAL_MODE_STRING="phase10a2"` (default) and the embedded payload hash does not match the requested payload ELF hash. The system cannot prove which payload was actually embedded into the kernel, blocking execution proof validation.

**Formal Specification:**
```
FUNCTION isBugCondition(input)
  INPUT: input of type BuildExecution
  OUTPUT: boolean
  
  // Authority drift occurs when manifest mode != embedded mode
  // OR when embedded hash != manifest payload hash
  // OR when boot markers don't match manifest
  
  // AUTHORITY SOURCES (manifest-based, NOT log-based)
  manifest_mode := extract_mode_from_manifest(input.manifest_json)
  manifest_payload_sha := extract_payload_sha_from_manifest(input.manifest_json)
  embedded_mode := extract_mode_from_embedded_elf_h(input.embedded_elf_h)
  embedded_sha := extract_sha_from_embedded_elf_h(input.embedded_elf_h)
  boot_emitted_mode := extract_mode_from_boot_log(input.boot_log)
  boot_emitted_sha := extract_sha_from_boot_log(input.boot_log)
  
  // Three distinct failure modes (manifest-based authority):
  // 1. Manifest mode != embedded mode (embed drift)
  // 2. Embedded hash != manifest payload hash (payload integrity failure)
  // 3. Boot markers missing or don't match manifest (boot verification failure)
  
  manifest_mode_mismatch := (manifest_mode != embedded_mode)
  hash_mismatch := (embedded_sha != manifest_payload_sha)
  boot_mode_mismatch := (boot_emitted_mode == "" OR boot_emitted_mode != manifest_mode)
  boot_sha_mismatch := (boot_emitted_sha == "" OR boot_emitted_sha != embedded_sha)
  
  RETURN manifest_mode_mismatch OR hash_mismatch OR boot_mode_mismatch OR boot_sha_mismatch
END FUNCTION
```

### Examples

- **Entry-Proof Build**: User runs `USER_MINIMAL_MODE=entry-proof make efi-img` but manifest shows `selected_mode: "phase10a2"` and embedded_elf.h contains phase10a2 payload hash
- **Runtime-Bridge Build**: User runs `USER_MINIMAL_MODE=runtime-bridge-test make efi-img` but embedded_elf.h SHA256 doesn't match manifest payload_sha256
- **Boot Verification**: Kernel boots but boot log doesn't contain `[K][PAYLOAD_MODE=entry-proof]` marker or `[K][PAYLOAD_SHA=...]` marker
- **Edge Case**: Manifest missing or malformed - build should HARD FAIL before kernel compile completes

### Root Cause Analysis

Based on the bug description and build system analysis, the most likely issues are:

1. **Authority Fragmentation (MOST CRITICAL)**
   - Multiple authority sources: USER_MINIMAL_MODE (Makefile), MINIMAL_MODE (userspace Makefile), embedded header generation
   - No single source of authority for payload selection
   - Silent fallback to default phase10a2 when authority sources disagree
   - **PRIORITY 1**: Establish USER_MINIMAL_MODE as single source of authority

2. **Missing Hash Verification (LIKELY)**
   - embedded_elf.h contains SHA256 hash but no build-time verification
   - No check that embedded hash matches requested payload ELF hash
   - Build succeeds even when wrong payload is embedded
   - **PRIORITY 2**: Add build-time hash verification with hard fail on mismatch

3. **Missing Boot Verification (LIKELY)**
   - Kernel boots but doesn't emit mode/hash marker to debugcon
   - No way to verify which payload was actually embedded at runtime
   - Execution proofs run against unknown payload
   - **PRIORITY 3**: Add boot-time mode/hash marker emission

4. **Silent Fallback Behavior (POSSIBLE)**
   - Build system falls back to default phase10a2 without error
   - No explicit failure when USER_MINIMAL_MODE is invalid or missing
   - Makes authority drift invisible until execution proof fails
   - **PRIORITY 4**: Add explicit validation of USER_MINIMAL_MODE value

## Expected Behavior

### Preservation Requirements

**Unchanged Behaviors:**
- Kernel compilation with phase10a2 payload (default) must continue to work exactly as before
- Existing CI gates must continue to pass without breaking due to payload authority changes
- Build system must continue to produce valid ELF and EFI binaries
- QEMU must continue to boot without hanging or crashing

**Scope:**
All inputs that do NOT involve non-default payload selection (normal kernel builds with phase10a2) should be completely unaffected by this fix. This includes:
- Default builds without USER_MINIMAL_MODE set
- Existing validation tests that use phase10a2
- CI gates that rely on default payload behavior
- QEMU boot flow with phase10a2 payload

**Architectural Freeze Compliance:**
This fix MUST remain within non-architectural bugfix boundaries:
- NO new syscalls, execution layers, or contracts
- NO changes to Ring0/Ring3 boundary, BCIB/CLI contracts, or kernel policy
- NO phase transition claims or architectural expansion
- ONLY build chain integrity repair and authority verification

## Correctness Properties

Property 1: Bug Condition - Payload Authority Determinism

_For any_ build execution where a userspace payload mode is requested via USER_MINIMAL_MODE, the build system SHALL enforce two correctness invariants:

1. **Mode Authority Invariant**: `build_selected_mode == embedded_elf_mode == boot_emitted_mode`
2. **Payload Integrity Invariant**: `expected_payload_sha == embedded_elf_sha == boot_emitted_sha`

These invariants ensure that the requested payload is deterministically embedded and verifiable at boot.

**Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8**

Property 2: Preservation - Default Build Behavior Unchanged

_For any_ build execution that does NOT specify a non-default payload mode (phase10a2 default), the fixed code SHALL produce exactly the same behavior as the original code, preserving all existing build functionality, CI gates, and QEMU boot flow.

**Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7**

## Fix Implementation

### Changes Required

Assuming our root cause analysis is correct (authority fragmentation + missing hash verification + missing boot verification):

**Phase 1: Establish Single Source of Authority**

**File**: `Makefile`

**Section**: User binary embedding (lines 760-780)

**Specific Changes**:
1. **Add USER_MINIMAL_MODE Validation**: Before building userspace payload, validate that USER_MINIMAL_MODE is one of the allowed values
   - Allowed values: `phase10a2`, `entry-proof`, `runtime-bridge-test`
   - If USER_MINIMAL_MODE is set but invalid, HARD FAIL with explicit error message
   - If USER_MINIMAL_MODE is unset, default to `phase10a2` (preserve existing behavior)
   - **RATIONALE**: Makes authority drift visible at build time instead of execution time

2. **Strengthen Mode Stamp Dependency**: Ensure USER_MINIMAL_MODE_STAMP triggers rebuild when mode changes
   - Current: `.mode.$(USER_MINIMAL_EFFECTIVE_MODE)` stamp file
   - Add: Explicit dependency chain from stamp → userspace ELF → embedded header → kernel objects
   - **RATIONALE**: Prevents stale embedded headers from previous builds

3. **Generate Build Manifest (Authority)**: After kernel compilation, generate machine-readable manifest
   - Create `out/build/payload_manifest.json` with:
     ```json
     {
       "selected_mode": "entry-proof",
       "payload_sha256": "abc123...",
       "embedded_header_sha256": "abc123...",
       "build_timestamp": "2026-04-12T22:36:00Z"
     }
     ```
   - Manifest is AUTHORITATIVE source for verification
   - Build log verification is DIAGNOSTIC only (helpful but not authoritative)
   - **RATIONALE**: Machine-readable manifest is more robust than log parsing

**Phase 2: Add Build-Time Hash Verification**

**File**: `Makefile`

**Target**: `$(EMBEDDED_ELF_HEADER)` (line 780)

**Specific Changes**:
1. **Compute Expected Hash**: Before generating embedded_elf.h, compute SHA256 of requested payload ELF
   - Use `sha256sum` or `shasum -a 256` depending on platform
   - Store in temporary variable `EXPECTED_PAYLOAD_HASH`

2. **Verify Embedded Hash**: After generating embedded_elf.h, extract embedded hash and compare
   - Extract `embedded_elf_sha256` from generated header
   - Compare with `EXPECTED_PAYLOAD_HASH`
   - HARD FAIL if mismatch detected with explicit error message showing both hashes
   - **RATIONALE**: Catches embed_elf.py failures or wrong payload selection
   - **IMPORTANT**: Hash verification runs for ALL builds (including default phase10a2)
   - No special cases - same verification logic for all modes

3. **Add Hash Verification Target**: Create explicit `verify-payload-hash` target
   - Can be run independently to verify embedded header integrity
   - Used by CI gates to validate payload authority
   - **RATIONALE**: Makes hash verification explicit and testable

**File**: `tools/embed_elf.py`

**Function**: `main`

**Specific Changes**:
1. **Add Mode Metadata**: Include requested mode in generated header
   - Add `--mode` command-line argument
   - Emit `static const char embedded_elf_mode[] = "mode";` in header
   - **RATIONALE**: Allows build-time verification of mode propagation

2. **Add Verification Helper**: Emit hash verification function in header
   - Generate `static inline int verify_embedded_elf_hash(const char *expected)` function
   - Returns 0 if hash matches, -1 if mismatch
   - **RATIONALE**: Allows kernel code to verify embedded payload at runtime

**Phase 3: Add Boot-Time Verification**

**File**: `kernel/kernel.c`

**Function**: `kmain_real` (or early boot initialization)

**Specific Changes**:
1. **Emit Boot Marker**: After kernel initialization, emit mode/hash marker to debugcon
   - Format: `[K][PAYLOAD_MODE=mode][PAYLOAD_SHA=hash]`
   - Use `debugcon_write` to emit marker
   - Emit immediately after `[[AYKEN_BOOT_OK]]` marker
   - **RATIONALE**: Provides authoritative evidence of embedded payload at boot

2. **Add Hash Verification (Observable Mismatch - Phase A)**: Before emitting marker, verify embedded hash
   - Call `verify_embedded_elf_hash` with expected hash (if available)
   - If verification fails, emit `[K][PAYLOAD_HASH_MISMATCH]` marker (DO NOT halt yet)
   - **RATIONALE**: Makes mismatch observable in boot logs for debugging
   - **NOTE**: Phase B (hard fail/halt) will be added after evidence chain stabilizes

3. **Add Mode String Verification (Observable Mismatch - Phase A)**: Verify DAYKEN_USER_MINIMAL_MODE_STRING matches embedded mode
   - Compare compile-time define with embedded_elf_mode
   - If mismatch, emit `[K][PAYLOAD_MODE_MISMATCH]` marker (DO NOT halt yet)
   - **RATIONALE**: Makes mismatch observable in boot logs for debugging
   - **NOTE**: Phase B (hard fail/halt) will be added after evidence chain stabilizes

**File**: `kernel/include/embedded_elf.h`

**Generated Content**: (via tools/embed_elf.py changes)

**Specific Changes**:
1. **Add Mode Constant**: Include mode string in generated header
   - `static const char embedded_elf_mode[] = "phase10a2";`
   - Generated from `--mode` argument to embed_elf.py

2. **Add Verification Function**: Include hash verification helper
   - `static inline int verify_embedded_elf_hash(const char *expected)`
   - Compares `embedded_elf_sha256` with expected hash

**Phase 4: Update Build Harnesses**

**File**: `scripts/qemu-entry-proof-harness.sh`

**Specific Changes**:
1. **Verify Build Manifest (Authority)**: After build, verify machine-readable manifest
   - Read `out/build/payload_manifest.json`
   - Verify `selected_mode == "entry-proof"`
   - Verify `payload_sha256 == embedded_header_sha256`
   - HARD FAIL if manifest missing or values mismatch
   - **RATIONALE**: Manifest is authoritative, more robust than log parsing

2. **Verify Build Log (Diagnostic)**: Optionally check build log for diagnostic evidence
   - `grep "DAYKEN_USER_MINIMAL_MODE_STRING=\"entry-proof\"" build.log`
   - If not found, emit WARNING (not HARD FAIL)
   - **RATIONALE**: Log verification is helpful diagnostic but not authoritative

3. **Verify Boot Marker**: After QEMU run, verify boot log contains mode/hash marker
   - `grep "\[K\]\[PAYLOAD_MODE=entry-proof\]" debugcon.log`
   - HARD FAIL if not found

**File**: `scripts/qemu-runtime-bridge-proof-harness.sh`

**Specific Changes**:
1. **Verify Build Manifest (Authority)**: Same as entry-proof harness, verify `selected_mode == "runtime-bridge-test"`
2. **Verify Build Log (Diagnostic)**: Same as entry-proof harness, emit WARNING if log check fails
3. **Verify Boot Marker**: Same as entry-proof harness, verify `[K][PAYLOAD_MODE=runtime-bridge-test]`

**File**: `scripts/ci/gate_ring3_execution_phase10a2.sh`

**Specific Changes**:
1. **Verify Build Manifest (Authority)**: Verify `selected_mode == "phase10a2"`
2. **Verify Build Log (Diagnostic)**: Verify build log contains `DAYKEN_USER_MINIMAL_MODE_STRING="phase10a2"` (WARNING if missing)
3. **Verify Boot Marker**: Verify boot log contains `[K][PAYLOAD_MODE=phase10a2]`

## Testing Strategy

### Validation Approach

The testing strategy follows a three-phase approach: first, surface counterexamples that demonstrate authority drift on unfixed code; second, verify the fix enforces the correctness invariant; third, validate that default build behavior is preserved.

### Exploratory Bug Condition Checking

**Goal**: Surface counterexamples that demonstrate the bug BEFORE implementing the fix. Confirm or refute the root cause analysis. If we refute, we will need to re-hypothesize.

**Test Plan**: Build kernel with entry-proof and runtime-bridge payloads on UNFIXED code, examine build logs and embedded headers to observe authority drift.

**Test Cases**:
1. **Entry-Proof Manifest Test**: Run `USER_MINIMAL_MODE=entry-proof make efi-img`, check if manifest shows `selected_mode: "entry-proof"` (will fail on unfixed code - shows phase10a2)
2. **Runtime-Bridge Hash Test**: Run `USER_MINIMAL_MODE=runtime-bridge-test make efi-img`, verify manifest `payload_sha256` matches embedded_elf_sha256 (will fail on unfixed code - hash mismatch)
3. **Boot Marker Test**: Boot kernel with entry-proof payload, check if debugcon log contains both `[K][PAYLOAD_MODE=entry-proof]` AND `[K][PAYLOAD_SHA=...]` (will fail on unfixed code - markers not emitted)
4. **Invalid Mode Test**: Run `USER_MINIMAL_MODE=invalid make efi-img`, check if build fails (will fall back silently on unfixed code)
5. **Manifest Missing Test**: Delete manifest, verify build fails before kernel compile completes (will succeed on unfixed code - no manifest check)

**Expected Counterexamples**:
- Build log shows `DAYKEN_USER_MINIMAL_MODE_STRING="phase10a2"` even when entry-proof requested
- Embedded hash matches phase10a2 payload even when runtime-bridge requested
- Boot log does not contain payload mode/hash marker
- Invalid mode falls back to default without error

### Fix Checking

**Goal**: Verify that for all inputs where the bug condition holds (non-default payload requested), the fixed system enforces the correctness invariant.

**Pseudocode:**
```
FOR ALL build_execution WHERE isBugCondition(build_execution) DO
  result := run_build_with_fixed_code(build_execution)
  
  // Verify manifest authority (not log parsing)
  ASSERT result.manifest.selected_mode == result.requested_mode
  ASSERT result.manifest.payload_sha256 == result.manifest.embedded_header_sha256
  
  // Verify embedded header matches manifest
  ASSERT result.embedded_mode == result.manifest.selected_mode
  ASSERT result.embedded_sha == result.manifest.payload_sha256
  
  // Verify boot markers match manifest (both mode AND sha required)
  ASSERT result.boot_emitted_mode == result.manifest.selected_mode
  ASSERT result.boot_emitted_sha == result.embedded_sha
  
  // Verify correctness invariants
  ASSERT result.manifest.selected_mode == result.embedded_mode == result.boot_emitted_mode
  ASSERT result.manifest.payload_sha256 == result.embedded_sha == result.boot_emitted_sha
END FOR
```

### Preservation Checking

**Goal**: Verify that for all inputs where the bug condition does NOT hold (default phase10a2 builds), the fixed code produces the same result as the original code.

**Pseudocode:**
```
FOR ALL build_execution WHERE NOT isBugCondition(build_execution) DO
  ASSERT original_build(build_execution) = fixed_build(build_execution)
END FOR
```

**Testing Approach**: Property-based testing is recommended for preservation checking because:
- It generates many test cases automatically across the input domain
- It catches edge cases that manual unit tests might miss
- It provides strong guarantees that behavior is unchanged for all default builds

**Test Plan**: Run existing CI gates (ring3-execution-phase10a2, fail-closed proof, runtime marker contract) on UNFIXED code to capture baseline behavior, then verify same gates pass on FIXED code with identical results.

**Test Cases**:
1. **Default Build Preservation**: Build kernel without USER_MINIMAL_MODE set, verify build log, embedded hash, and boot marker match unfixed code
2. **CI Gate Preservation**: Run all existing CI gates on fixed code, verify they pass with same results as unfixed code
3. **QEMU Boot Preservation**: Boot kernel with default payload, verify boot flow is identical to unfixed code
4. **Build Artifact Preservation**: Verify kernel.elf and BOOTX64.EFI are byte-identical (except for mode/hash marker code changes)

### Unit Tests

- Test USER_MINIMAL_MODE validation logic (valid values, invalid values, unset)
- Test mode stamp dependency chain (rebuild when mode changes)
- Test build log mode extraction and verification
- Test embedded hash extraction and verification
- Test boot marker emission and parsing
- Test hash verification function in embedded_elf.h

### Property-Based Tests

- Generate random payload mode selections and verify correctness invariant holds
- Generate random build configurations and verify preservation for default mode
- Test that all non-default payload builds produce deterministic embedded headers
- Test that all boot flows emit correct mode/hash markers

### Integration Tests

- Test full build flow with entry-proof payload (build → verify hash → boot → verify marker)
- Test full build flow with runtime-bridge payload (build → verify hash → boot → verify marker)
- Test full build flow with default phase10a2 payload (verify preservation)
- Test CI gate integration (verify all gates pass with fixed code)
- Test harness integration (verify entry-proof and runtime-bridge harnesses work with fixed code)

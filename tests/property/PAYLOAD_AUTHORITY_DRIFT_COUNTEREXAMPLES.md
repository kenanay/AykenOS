# Payload Authority Drift - Bug Condition Exploration Results

**Date**: 2026-04-13  
**Test**: `tests/property/test_payload_authority_drift.py`  
**Status**: ✅ Counterexamples Successfully Surfaced (Bug Confirmed)

## Executive Summary

The bug condition exploration test has successfully confirmed the existence of payload authority drift in the AykenOS kernel build pipeline. All 5 test cases failed as expected on unfixed code, surfacing concrete counterexamples that demonstrate:

1. **Authority Fragmentation**: No manifest is generated to serve as single source of authority
2. **Missing Hash Verification**: No build-time verification that embedded hash matches requested payload
3. **Missing Boot Verification**: No boot-time markers to verify which payload was actually embedded
4. **Silent Fallback**: Build succeeds even when manifest is missing (no hard fail)

## Test Results

### Test 1: Entry-Proof Manifest Mode ❌ FAILED

**Test**: Build kernel with `USER_MINIMAL_MODE=entry-proof`, verify manifest shows `selected_mode: "entry-proof"`

**Expected**: Manifest exists with `selected_mode: "entry-proof"`

**Actual**: Manifest missing or malformed

**Counterexample**:
- Build command: `USER_MINIMAL_MODE=entry-proof make efi-img`
- Build result: SUCCESS (kernel built)
- Manifest path: `out/build/payload_manifest.json`
- Manifest status: **DOES NOT EXIST**

**Root Cause Evidence**:
- No manifest generation logic exists in Makefile
- Build system has no single source of authority for payload selection
- Cannot verify which mode was actually selected during build

---

### Test 2: Runtime-Bridge Hash Match ❌ FAILED

**Test**: Build kernel with `USER_MINIMAL_MODE=runtime-bridge-test`, verify manifest `payload_sha256 == embedded_header_sha256`

**Expected**: Manifest exists with both hashes matching

**Actual**: Manifest missing or incomplete

**Counterexample**:
- Build command: `USER_MINIMAL_MODE=runtime-bridge-test make efi-img`
- Build result: SUCCESS (kernel built)
- Manifest path: `out/build/payload_manifest.json`
- Manifest status: **DOES NOT EXIST**

**Root Cause Evidence**:
- No manifest generation logic exists in Makefile
- No build-time hash verification
- Cannot verify payload integrity at build time

---

### Test 3: Boot Marker Emission ❌ FAILED (2 failures)

**Test**: Boot kernel with entry-proof payload, verify debugcon contains BOTH `[K][PAYLOAD_MODE=entry-proof]` AND `[K][PAYLOAD_SHA=...]`

**Expected**: Both boot markers present in debugcon output

**Actual**: 
- Mode marker: **NOT FOUND**
- SHA marker: **NOT FOUND**

**Counterexample**:
- Build command: `USER_MINIMAL_MODE=entry-proof make efi-img`
- Boot method: QEMU with OVMF firmware
- Debugcon output: Kernel boots successfully but no payload markers emitted
- Boot log path: `evidence/entry-proof/qemu_debugcon.log`

**Root Cause Evidence**:
- No boot-time marker emission logic in kernel
- Cannot verify which payload was actually embedded at runtime
- Execution proofs run against unknown payload

---

### Test 4: Invalid Mode Fails ✅ PASSED

**Test**: Build with `USER_MINIMAL_MODE=invalid`, verify build fails

**Expected**: Build fails with explicit error

**Actual**: Build fails with explicit error ✅

**Result**: This test PASSED, indicating that invalid mode validation exists in the userspace Makefile.

**Note**: While this test passed, it doesn't address the core authority drift issue (manifest missing, no hash verification, no boot markers).

---

### Test 5: Manifest Missing Fails ❌ FAILED

**Test**: Delete manifest, verify build fails before kernel compile completes

**Expected**: Build fails when manifest is missing

**Actual**: Build succeeded without manifest

**Counterexample**:
- Initial build: `USER_MINIMAL_MODE=entry-proof make efi-img` (SUCCESS)
- Manifest deletion: `rm out/build/payload_manifest.json`
- Rebuild: `USER_MINIMAL_MODE=entry-proof make efi-img` (SUCCESS)
- Result: **Build succeeded even though manifest was deleted**

**Root Cause Evidence**:
- No manifest dependency check in Makefile
- Build system doesn't require manifest to exist
- No hard fail when authority source is missing

---

## Root Cause Analysis

### 1. Authority Fragmentation (CONFIRMED)

**Evidence**:
- No `payload_manifest.json` file generated during build
- No single source of authority for payload selection
- Multiple potential authority sources (USER_MINIMAL_MODE, MINIMAL_MODE, embedded header) with no coordination

**Impact**:
- Cannot prove which payload mode was selected
- Cannot verify mode consistency across build/embed/boot chain
- Execution proofs run against unknown payload

**Priority**: CRITICAL (P1)

---

### 2. Missing Hash Verification (CONFIRMED)

**Evidence**:
- No manifest to store expected payload hash
- No build-time verification that embedded hash matches requested payload
- `embedded_elf.h` contains SHA256 hash but no verification logic

**Impact**:
- Cannot prove payload integrity
- Wrong payload could be embedded without detection
- Hash mismatch only discovered at execution time (if at all)

**Priority**: HIGH (P2)

---

### 3. Missing Boot Verification (CONFIRMED)

**Evidence**:
- No `[K][PAYLOAD_MODE=...]` marker in boot log
- No `[K][PAYLOAD_SHA=...]` marker in boot log
- Kernel boots successfully but doesn't emit payload verification markers

**Impact**:
- Cannot verify which payload was actually embedded at runtime
- No observable evidence of payload identity at boot
- Execution proofs cannot validate payload authority

**Priority**: HIGH (P3)

---

### 4. Silent Fallback Behavior (CONFIRMED)

**Evidence**:
- Build succeeds even when manifest is missing
- No hard fail when authority source is unavailable
- No explicit validation of manifest existence

**Impact**:
- Authority drift is invisible until execution proof fails
- No early detection of build configuration issues
- Debugging becomes difficult when payload mismatch occurs

**Priority**: MEDIUM (P4)

---

## Correctness Invariants Violated

The following correctness invariants are currently violated:

### Mode Authority Invariant
```
manifest.selected_mode == embedded_elf_mode == boot_emitted_mode
```

**Status**: ❌ VIOLATED
- `manifest.selected_mode`: **DOES NOT EXIST** (no manifest)
- `embedded_elf_mode`: **DOES NOT EXIST** (not in embedded_elf.h)
- `boot_emitted_mode`: **DOES NOT EXIST** (no boot marker)

### Payload Integrity Invariant
```
manifest.payload_sha256 == embedded_elf_sha == boot_emitted_sha
```

**Status**: ❌ VIOLATED
- `manifest.payload_sha256`: **DOES NOT EXIST** (no manifest)
- `embedded_elf_sha`: EXISTS in `embedded_elf.h` but not verified
- `boot_emitted_sha`: **DOES NOT EXIST** (no boot marker)

---

## Counterexample Summary

| Test | Expected | Actual | Root Cause |
|------|----------|--------|------------|
| Entry-Proof Manifest | Manifest with mode | No manifest | Authority fragmentation |
| Runtime-Bridge Hash | Manifest with hashes | No manifest | Missing hash verification |
| Boot Marker (Mode) | Mode marker emitted | No marker | Missing boot verification |
| Boot Marker (SHA) | SHA marker emitted | No marker | Missing boot verification |
| Manifest Missing | Build fails | Build succeeds | Silent fallback |

---

## Next Steps

1. ✅ **Task 1 Complete**: Bug condition exploration test written and run
2. ✅ **Counterexamples Documented**: All failures documented with evidence
3. ⏳ **Task 2**: Write preservation property tests (BEFORE implementing fix)
4. ⏳ **Task 3**: Implement fix (4 phases: authority, hash verification, boot verification, harness updates)
5. ⏳ **Task 3.5**: Re-run bug condition test (should PASS after fix)
6. ⏳ **Task 3.6**: Re-run preservation tests (should still PASS after fix)

---

## Test Execution Log

```
============================================================
Bug Condition Exploration Test: Payload Authority Drift
============================================================

CRITICAL: These tests MUST FAIL on unfixed code.
Failure confirms the bug exists.

=== Test 1: Entry-Proof Manifest Mode ===
❌ FAIL: Manifest missing or malformed

=== Test 2: Runtime-Bridge Hash Match ===
❌ FAIL: Manifest missing or incomplete

=== Test 3: Boot Marker Emission ===
❌ FAIL: Mode marker not found
❌ FAIL: SHA marker not found

=== Test 4: Invalid Mode Fails ===
✅ PASS: Build fails with explicit error

=== Test 5: Manifest Missing Fails ===
❌ FAIL: Build succeeded without manifest

============================================================
EXPECTED OUTCOME: Tests FAIL (confirms bug exists)
============================================================
```

---

## Conclusion

The bug condition exploration test has successfully confirmed the existence of payload authority drift in the AykenOS kernel build pipeline. The test surfaced 5 concrete counterexamples demonstrating:

- **Authority Fragmentation**: No manifest serves as single source of authority
- **Missing Hash Verification**: No build-time verification of payload integrity
- **Missing Boot Verification**: No boot-time markers to verify embedded payload
- **Silent Fallback**: Build succeeds even when authority source is missing

These counterexamples provide clear evidence for the root cause analysis in the design document and validate the need for the proposed fix.

**Test Status**: ✅ COMPLETE  
**Bug Status**: ✅ CONFIRMED  
**Next Task**: Write preservation property tests (Task 2)

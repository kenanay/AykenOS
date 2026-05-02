# Phase-17 Runtime Failure Injection Plan

**Date**: 2026-05-02  
**Branch**: `phase17-marker-validation-guard`  
**Status**: Pre-Production Testing Required  
**Authority**: Kenan AY - Architectural Steward

---

## ⚠️ CRITICAL SAFETY REQUIREMENTS

**Mandate from Architectural Steward (2026-05-02)**:

### 1. Test-Only Guard (Production Sızma Prevention)
- **Requirement**: Injection harness MUST be test-only
- **Implementation**: Top-level guard `AYKEN_PHASE17_MARKER_INJECTION_TEST=1`
- **Rationale**: Prevent ANY injection code from compiling into production
- **Enforcement**: Fail-closed — no guard = no injection code

### 2. Explicit Validation (No "Fail = Pass" Logic)
- **Requirement**: Tests must explicitly validate expected error codes and state transitions
- **Implementation**: `grep` for specific `MARKER_ERROR_*` codes and `EXEC_SLOT_FAILED` state
- **Rationale**: Kernel crash vs. correct validation failure must be distinguishable
- **Enforcement**: Test fails if expected error code not found (even if execution fails)

**Merge Gate**: These requirements are NON-NEGOTIABLE. PR will be rejected if violated.

---

## Purpose

Validate Phase-17 Step 5 marker validation under **adversarial conditions** before production merge.

**Goal**: Prove validation correctly rejects all invalid marker sequences in runtime.

---

## Test Matrix

### Test 1: Invalid Order Injection
**Scenario**: Markers captured out of order  
**Injection Point**: `execution_slot_marker_capture_locked()`  
**Expected Sequence**: `[0,1,2,3,4]`  
**Injected Sequence**: `[0,2,1,3,4]`

**Expected Behavior**:
- Layer 2 validation fails: `marker_sequence[1] = 2 != 1`
- Returns: `MARKER_ERROR_INVALID_ORDER`
- State transition blocked: `EXEC_SLOT_FAILED`

**Test Command**:
```bash
AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1 \
AYKEN_MARKER_INJECT_INVALID_ORDER=1 \
make qemu-test
```

**Success Criteria**:
- ✅ Validation detects invalid order
- ✅ Execution fails with `MARKER_ERROR_INVALID_ORDER`
- ✅ No hash prepared (pre-commit guard blocks)
- ✅ Slot transitions to `EXEC_SLOT_FAILED`

---

### Test 2: Duplicate Marker Injection
**Scenario**: Same marker captured twice  
**Injection Point**: `execution_slot_marker_capture_locked()`  
**Expected Sequence**: `[0,1,2,3,4]`  
**Injected Sequence**: `[0,1,1,2,3]`

**Expected Behavior**:
- Layer 2 validation fails: `marker_sequence[2] = 1 != 2`
- Returns: `MARKER_ERROR_INVALID_ORDER`
- State transition blocked

**Test Command**:
```bash
AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1 \
AYKEN_MARKER_INJECT_DUPLICATE=1 \
make qemu-test
```

**Success Criteria**:
- ✅ Validation detects duplicate
- ✅ Execution fails with `MARKER_ERROR_INVALID_ORDER`
- ✅ Bitmap check may also fail (bit pattern mismatch)

---

### Test 3: Missing Marker Injection
**Scenario**: Marker skipped in sequence  
**Injection Point**: `execution_slot_marker_capture_locked()`  
**Expected Sequence**: `[0,1,2,3,4]`  
**Injected Sequence**: `[0,1,3,4]` (marker 2 missing)

**Expected Behavior**:
- Layer 1 validation fails: `marker_count = 4 != 5`
- Returns: `MARKER_ERROR_INVALID_ORDER`
- State transition blocked

**Test Command**:
```bash
AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1 \
AYKEN_MARKER_INJECT_MISSING=1 \
make qemu-test
```

**Success Criteria**:
- ✅ Validation detects missing marker
- ✅ Count check fails immediately
- ✅ Execution fails with `MARKER_ERROR_INVALID_ORDER`

---

### Test 4: Overflow Injection
**Scenario**: More than 7 markers captured  
**Injection Point**: `execution_slot_marker_capture_locked()`  
**Expected Count**: 5 markers  
**Injected Count**: 8 markers (overflow)

**Expected Behavior**:
- Capture phase detects overflow: `marker_count >= 7`
- Sets: `marker_error_code = MARKER_ERROR_OVERFLOW`
- Validation reads error code immediately
- Returns: `MARKER_ERROR_OVERFLOW`

**Test Command**:
```bash
AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1 \
AYKEN_MARKER_INJECT_OVERFLOW=1 \
make qemu-test
```

**Success Criteria**:
- ✅ Capture phase sets `marker_error_code = 3`
- ✅ Validation detects error code
- ✅ Execution fails with `MARKER_ERROR_OVERFLOW`
- ✅ No buffer overflow (bounds-safe)

---

### Test 5: Stale Buffer Data Injection (Layer 4)
**Scenario**: Valid markers but garbage in unused buffer space  
**Injection Point**: After validation, before cleanup  
**Expected Buffer**: `[0,1,2,3,4,0,0]`  
**Injected Buffer**: `[0,1,2,3,4,X,Y]` (X,Y = stale data)

**Expected Behavior**:
- Layer 4 validation fails: `marker_sequence[5] != 0`
- Returns: `MARKER_ERROR_INVALID_ORDER`
- Memory hygiene enforced

**Test Command**:
```bash
AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1 \
AYKEN_MARKER_INJECT_STALE_DATA=1 \
make qemu-test
```

**Success Criteria**:
- ✅ Layer 4 detects stale data
- ✅ Validation fails with `MARKER_ERROR_INVALID_ORDER`
- ✅ Temporal safety guaranteed

---

### Test 6: Corrupted Bitmap Injection
**Scenario**: Valid sequence but bitmap mismatch  
**Injection Point**: `execution_slot_marker_capture_locked()`  
**Expected Bitmap**: `0x1F` (bits 0-4 set)  
**Injected Bitmap**: `0x3F` (bit 5 also set)

**Expected Behavior**:
- Layer 3 validation fails: `marker_bitmap != 0x1F`
- Returns: `MARKER_ERROR_INVALID_ORDER`
- Bitmap integrity enforced

**Test Command**:
```bash
AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1 \
AYKEN_MARKER_INJECT_CORRUPT_BITMAP=1 \
make qemu-test
```

**Success Criteria**:
- ✅ Layer 3 detects bitmap corruption
- ✅ Validation fails with `MARKER_ERROR_INVALID_ORDER`
- ✅ Extra marker bits rejected

---

### Test 7: Race Condition Simulation (Partial Write)
**Scenario**: Marker capture interrupted mid-sequence  
**Injection Point**: `execution_slot_marker_capture_locked()`  
**Expected Sequence**: `[0,1,2,3,4]`  
**Injected Sequence**: `[0,1,2]` (partial write)

**Expected Behavior**:
- Layer 1 validation fails: `marker_count = 3 != 5`
- Returns: `MARKER_ERROR_INVALID_ORDER`
- Partial writes rejected

**Test Command**:
```bash
AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1 \
AYKEN_MARKER_INJECT_PARTIAL_WRITE=1 \
make qemu-test
```

**Success Criteria**:
- ✅ Count check detects partial write
- ✅ Validation fails immediately
- ✅ No partial state committed

---

## Implementation Strategy

### Phase 1: Injection Harness (Test-Only)

**CRITICAL REQUIREMENT #1**: Injection code MUST be test-only. Use top-level guard to prevent production sızma:

```c
// kernel/sys/execution_marker_injection.h
#ifndef EXECUTION_MARKER_INJECTION_H
#define EXECUTION_MARKER_INJECTION_H

// ⚠️ TOP-LEVEL GUARD: ONLY enabled in test builds
// This prevents ANY injection code from compiling into production
#if defined(AYKEN_PHASE17_MARKER_INJECTION_TEST) && (AYKEN_PHASE17_MARKER_INJECTION_TEST == 1)

#include "execution_slot.h"

// Individual test flags (enabled one at a time)
#if defined(AYKEN_MARKER_INJECT_INVALID_ORDER) && (AYKEN_MARKER_INJECT_INVALID_ORDER == 1)
void inject_invalid_order(exec_slot_t *slot);
#endif

#if defined(AYKEN_MARKER_INJECT_DUPLICATE) && (AYKEN_MARKER_INJECT_DUPLICATE == 1)
void inject_duplicate(exec_slot_t *slot);
#endif

#if defined(AYKEN_MARKER_INJECT_MISSING) && (AYKEN_MARKER_INJECT_MISSING == 1)
void inject_missing(exec_slot_t *slot);
#endif

#if defined(AYKEN_MARKER_INJECT_OVERFLOW) && (AYKEN_MARKER_INJECT_OVERFLOW == 1)
void inject_overflow(exec_slot_t *slot);
#endif

#if defined(AYKEN_MARKER_INJECT_STALE_DATA) && (AYKEN_MARKER_INJECT_STALE_DATA == 1)
void inject_stale_data(exec_slot_t *slot);
#endif

#if defined(AYKEN_MARKER_INJECT_CORRUPT_BITMAP) && (AYKEN_MARKER_INJECT_CORRUPT_BITMAP == 1)
void inject_corrupt_bitmap(exec_slot_t *slot);
#endif

#if defined(AYKEN_MARKER_INJECT_PARTIAL_WRITE) && (AYKEN_MARKER_INJECT_PARTIAL_WRITE == 1)
void inject_partial_write(exec_slot_t *slot);
#endif

#endif // AYKEN_PHASE17_MARKER_INJECTION_TEST

#endif // EXECUTION_MARKER_INJECTION_H
```

Create `kernel/sys/execution_marker_injection.c`:

```c
#include "execution_marker_injection.h"

// ⚠️ CRITICAL: All injection code guarded by test-only flag
#if defined(AYKEN_PHASE17_MARKER_INJECTION_TEST) && (AYKEN_PHASE17_MARKER_INJECTION_TEST == 1)

#if defined(AYKEN_MARKER_INJECT_INVALID_ORDER) && (AYKEN_MARKER_INJECT_INVALID_ORDER == 1)
void inject_invalid_order(exec_slot_t *slot) {
    // Swap markers 1 and 2 to create invalid sequence
    if (slot->marker_count >= 3) {
        uint8_t temp = slot->marker_sequence[1];
        slot->marker_sequence[1] = slot->marker_sequence[2];
        slot->marker_sequence[2] = temp;
    }
}
#endif

#if defined(AYKEN_MARKER_INJECT_DUPLICATE) && (AYKEN_MARKER_INJECT_DUPLICATE == 1)
void inject_duplicate(exec_slot_t *slot) {
    // Duplicate marker 1 at position 2
    if (slot->marker_count >= 3) {
        slot->marker_sequence[2] = slot->marker_sequence[1];
    }
}
#endif

#if defined(AYKEN_MARKER_INJECT_MISSING) && (AYKEN_MARKER_INJECT_MISSING == 1)
void inject_missing(exec_slot_t *slot) {
    // Remove marker 2 by shifting sequence
    if (slot->marker_count >= 5) {
        slot->marker_sequence[2] = slot->marker_sequence[3];
        slot->marker_sequence[3] = slot->marker_sequence[4];
        slot->marker_count = 4;
    }
}
#endif

#if defined(AYKEN_MARKER_INJECT_OVERFLOW) && (AYKEN_MARKER_INJECT_OVERFLOW == 1)
void inject_overflow(exec_slot_t *slot) {
    // Force overflow condition
    slot->marker_count = 8;
    slot->marker_error_code = MARKER_ERROR_OVERFLOW;
}
#endif

#if defined(AYKEN_MARKER_INJECT_STALE_DATA) && (AYKEN_MARKER_INJECT_STALE_DATA == 1)
void inject_stale_data(exec_slot_t *slot) {
    // Valid markers but garbage in unused buffer space
    if (slot->marker_count == 5) {
        slot->marker_sequence[5] = 0xAA;
        slot->marker_sequence[6] = 0xBB;
    }
}
#endif

#if defined(AYKEN_MARKER_INJECT_CORRUPT_BITMAP) && (AYKEN_MARKER_INJECT_CORRUPT_BITMAP == 1)
void inject_corrupt_bitmap(exec_slot_t *slot) {
    // Valid sequence but corrupted bitmap
    slot->marker_bitmap = 0x3F; // Extra bit set
}
#endif

#if defined(AYKEN_MARKER_INJECT_PARTIAL_WRITE) && (AYKEN_MARKER_INJECT_PARTIAL_WRITE == 1)
void inject_partial_write(exec_slot_t *slot) {
    // Simulate interrupted capture
    slot->marker_count = 3;
}
#endif

#endif // AYKEN_PHASE17_MARKER_INJECTION_TEST
```

### Phase 2: Injection Points
Add hooks in `execution_slot_prepare_hash_locked()`:

```c
#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
    // ⚠️ Injection hooks (test-only, guarded by AYKEN_PHASE17_MARKER_INJECTION_TEST)
    #if defined(AYKEN_PHASE17_MARKER_INJECTION_TEST) && (AYKEN_PHASE17_MARKER_INJECTION_TEST == 1)
        #ifdef AYKEN_MARKER_INJECT_INVALID_ORDER
            inject_invalid_order(slot);
        #endif
        #ifdef AYKEN_MARKER_INJECT_DUPLICATE
            inject_duplicate(slot);
        #endif
        #ifdef AYKEN_MARKER_INJECT_MISSING
            inject_missing(slot);
        #endif
        #ifdef AYKEN_MARKER_INJECT_OVERFLOW
            inject_overflow(slot);
        #endif
        #ifdef AYKEN_MARKER_INJECT_STALE_DATA
            inject_stale_data(slot);
        #endif
        #ifdef AYKEN_MARKER_INJECT_CORRUPT_BITMAP
            inject_corrupt_bitmap(slot);
        #endif
        #ifdef AYKEN_MARKER_INJECT_PARTIAL_WRITE
            inject_partial_write(slot);
        #endif
    #endif
    
    /* Pre-commit guard: validate markers before hash preparation */
    if (execution_slot_validate_markers_locked(slot) != 0) {
        return -1;
    }
#endif
```

### Phase 3: Test Automation

**CRITICAL REQUIREMENT #2**: Tests must explicitly validate expected error codes and state transitions, not just "fail = pass".

Create `tests/phase17_marker_injection_suite.sh`:

```bash
#!/bin/bash
set -e

echo "=== Phase-17 Marker Validation Injection Tests ==="
echo "⚠️  Test Philosophy: Explicit validation of error codes and state transitions"
echo ""

EVIDENCE_DIR="out/evidence/phase17-injection-tests"
mkdir -p "$EVIDENCE_DIR"

# Test 1: Invalid Order
echo ">> Test 1: Invalid Order Injection"
AYKEN_PHASE17_MARKER_INJECTION_TEST=1 \
AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1 \
AYKEN_MARKER_INJECT_INVALID_ORDER=1 \
make qemu-test-headless > "$EVIDENCE_DIR/test1_invalid_order.log" 2>&1 || true

# ✅ EXPLICIT VALIDATION: Check for expected error code
if grep -q "MARKER_ERROR_INVALID_ORDER" "$EVIDENCE_DIR/test1_invalid_order.log" && \
   grep -q "EXEC_SLOT_FAILED" "$EVIDENCE_DIR/test1_invalid_order.log"; then
    echo "✅ PASS: Invalid order correctly rejected with expected error code"
else
    echo "❌ FAIL: Expected MARKER_ERROR_INVALID_ORDER and EXEC_SLOT_FAILED"
    exit 1
fi

# Test 2: Duplicate Marker
echo ">> Test 2: Duplicate Marker Injection"
AYKEN_PHASE17_MARKER_INJECTION_TEST=1 \
AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1 \
AYKEN_MARKER_INJECT_DUPLICATE=1 \
make qemu-test-headless > "$EVIDENCE_DIR/test2_duplicate.log" 2>&1 || true

if grep -q "MARKER_ERROR_INVALID_ORDER" "$EVIDENCE_DIR/test2_duplicate.log" && \
   grep -q "EXEC_SLOT_FAILED" "$EVIDENCE_DIR/test2_duplicate.log"; then
    echo "✅ PASS: Duplicate marker correctly rejected"
else
    echo "❌ FAIL: Expected MARKER_ERROR_INVALID_ORDER and EXEC_SLOT_FAILED"
    exit 1
fi

# Test 3: Missing Marker
echo ">> Test 3: Missing Marker Injection"
AYKEN_PHASE17_MARKER_INJECTION_TEST=1 \
AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1 \
AYKEN_MARKER_INJECT_MISSING=1 \
make qemu-test-headless > "$EVIDENCE_DIR/test3_missing.log" 2>&1 || true

if grep -q "MARKER_ERROR_INVALID_ORDER" "$EVIDENCE_DIR/test3_missing.log" && \
   grep -q "EXEC_SLOT_FAILED" "$EVIDENCE_DIR/test3_missing.log"; then
    echo "✅ PASS: Missing marker correctly rejected"
else
    echo "❌ FAIL: Expected MARKER_ERROR_INVALID_ORDER and EXEC_SLOT_FAILED"
    exit 1
fi

# Test 4: Overflow
echo ">> Test 4: Overflow Injection"
AYKEN_PHASE17_MARKER_INJECTION_TEST=1 \
AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1 \
AYKEN_MARKER_INJECT_OVERFLOW=1 \
make qemu-test-headless > "$EVIDENCE_DIR/test4_overflow.log" 2>&1 || true

if grep -q "MARKER_ERROR_OVERFLOW" "$EVIDENCE_DIR/test4_overflow.log" && \
   grep -q "EXEC_SLOT_FAILED" "$EVIDENCE_DIR/test4_overflow.log"; then
    echo "✅ PASS: Overflow correctly rejected with expected error code"
else
    echo "❌ FAIL: Expected MARKER_ERROR_OVERFLOW and EXEC_SLOT_FAILED"
    exit 1
fi

# Test 5: Stale Buffer Data
echo ">> Test 5: Stale Buffer Data Injection"
AYKEN_PHASE17_MARKER_INJECTION_TEST=1 \
AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1 \
AYKEN_MARKER_INJECT_STALE_DATA=1 \
make qemu-test-headless > "$EVIDENCE_DIR/test5_stale_data.log" 2>&1 || true

if grep -q "MARKER_ERROR_INVALID_ORDER" "$EVIDENCE_DIR/test5_stale_data.log" && \
   grep -q "EXEC_SLOT_FAILED" "$EVIDENCE_DIR/test5_stale_data.log"; then
    echo "✅ PASS: Stale buffer data correctly rejected (Layer 4)"
else
    echo "❌ FAIL: Expected MARKER_ERROR_INVALID_ORDER and EXEC_SLOT_FAILED"
    exit 1
fi

# Test 6: Corrupted Bitmap
echo ">> Test 6: Corrupted Bitmap Injection"
AYKEN_PHASE17_MARKER_INJECTION_TEST=1 \
AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1 \
AYKEN_MARKER_INJECT_CORRUPT_BITMAP=1 \
make qemu-test-headless > "$EVIDENCE_DIR/test6_corrupt_bitmap.log" 2>&1 || true

if grep -q "MARKER_ERROR_INVALID_ORDER" "$EVIDENCE_DIR/test6_corrupt_bitmap.log" && \
   grep -q "EXEC_SLOT_FAILED" "$EVIDENCE_DIR/test6_corrupt_bitmap.log"; then
    echo "✅ PASS: Corrupted bitmap correctly rejected (Layer 3)"
else
    echo "❌ FAIL: Expected MARKER_ERROR_INVALID_ORDER and EXEC_SLOT_FAILED"
    exit 1
fi

# Test 7: Partial Write
echo ">> Test 7: Partial Write Injection"
AYKEN_PHASE17_MARKER_INJECTION_TEST=1 \
AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1 \
AYKEN_MARKER_INJECT_PARTIAL_WRITE=1 \
make qemu-test-headless > "$EVIDENCE_DIR/test7_partial_write.log" 2>&1 || true

if grep -q "MARKER_ERROR_INVALID_ORDER" "$EVIDENCE_DIR/test7_partial_write.log" && \
   grep -q "EXEC_SLOT_FAILED" "$EVIDENCE_DIR/test7_partial_write.log"; then
    echo "✅ PASS: Partial write correctly rejected (Layer 1)"
else
    echo "❌ FAIL: Expected MARKER_ERROR_INVALID_ORDER and EXEC_SLOT_FAILED"
    exit 1
fi

echo ""
echo "=== All 7 injection tests completed successfully ==="
echo "Evidence stored in: $EVIDENCE_DIR"
```

---

## Success Criteria (Overall)

### Validation Layer Tests
- ✅ Layer 1 (Count): Rejects missing/extra markers
- ✅ Layer 2 (Sequence): Rejects invalid order/duplicates
- ✅ Layer 3 (Bitmap): Rejects corrupted bitmaps
- ✅ Layer 4 (Hygiene): Rejects stale buffer data

### System Behavior Tests
- ✅ Pre-commit guard blocks invalid executions
- ✅ State transitions to `EXEC_SLOT_FAILED` on validation failure
- ✅ No hash prepared for invalid marker sequences
- ✅ Error codes propagate correctly

### Safety Properties
- ✅ No buffer overflows (bounds-safe capture)
- ✅ No partial state commits (atomic validation)
- ✅ No temporal safety violations (hygiene enforced)
- ✅ Deterministic failure behavior (same input → same output)

---

## Timeline

1. **Day 1**: Implement injection harness
2. **Day 2**: Add injection points and test automation
3. **Day 3**: Run full test suite and collect evidence
4. **Day 4**: Document results and update completion report
5. **Day 5**: Submit PR with test evidence

---

## Evidence Collection

For each test, collect:
- ✅ QEMU console output (validation failure messages)
- ✅ Execution slot state dump (marker_count, marker_sequence, marker_bitmap)
- ✅ Error code returned (`MARKER_ERROR_*`)
- ✅ Final slot state (`EXEC_SLOT_FAILED`)
- ✅ Pre-ci gate results (all gates must still pass)

Store evidence in: `out/evidence/phase17-injection-tests/`

### Verification: Test-Only Guard Effectiveness

**Critical Check**: Verify injection code does NOT compile into production builds.

```bash
# Build production kernel (without test flag)
make clean
make kernel.elf

# Verify NO injection symbols exist
objdump -t out/build/kernel.elf | grep -i inject

# Expected output: NOTHING (empty)
# If ANY injection symbols found → FAIL (production contamination)
```

**Success Criteria**:
- ✅ Production build contains ZERO injection symbols
- ✅ Test build (with `AYKEN_PHASE17_MARKER_INJECTION_TEST=1`) contains injection symbols
- ✅ Guard is fail-closed (no flag = no code)

---

## Production Readiness Checklist

### Core Requirements
- [ ] All 7 injection tests pass with explicit error code validation
- [ ] Test-only guard (`AYKEN_PHASE17_MARKER_INJECTION_TEST=1`) verified in all injection code
- [ ] Evidence collected and documented (stored in `out/evidence/phase17-injection-tests/`)
- [ ] Pre-ci gates pass with injection harness enabled

### Validation Requirements
- [ ] Each test explicitly validates expected `MARKER_ERROR_*` code
- [ ] Each test explicitly validates `EXEC_SLOT_FAILED` state transition
- [ ] No "fail = pass" logic (kernel crash vs. correct failure distinguishable)

### Safety Requirements
- [ ] Injection code CANNOT compile without `AYKEN_PHASE17_MARKER_INJECTION_TEST=1`
- [ ] Production build verified to have ZERO injection code (objdump check)
- [ ] No production path contamination (code review verified)

### CI/Review Requirements
- [ ] Remote CI passes (mandatory)
- [ ] Code review approved
- [ ] Architectural steward sign-off

**Only after ALL checkboxes**: Merge to main.

**Merge Rejection Criteria**:
- ❌ Any injection code without test-only guard
- ❌ Any test using "fail = pass" logic
- ❌ Missing explicit error code validation
- ❌ Remote CI not executed

---

## Notes

- **Injection harness is test-only**: Never enabled in production builds
- **Fail-closed semantics**: All tests expect validation to reject invalid inputs
- **Determinism**: Same injection → same validation failure (reproducible)
- **Constitutional compliance**: All tests must pass pre-ci gates

---

**Prepared by**: Kenan AY - Architectural Steward  
**Date**: 2026-05-02  
**Status**: Ready for implementation

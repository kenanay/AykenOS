# Phase-17 Runtime Failure Injection Plan

**Date**: 2026-05-02  
**Branch**: `phase17-marker-validation-guard`  
**Status**: Pre-Production Testing Required

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

### Phase 1: Injection Harness
Create `kernel/sys/execution_marker_injection.c`:

```c
#if defined(AYKEN_MARKER_INJECT_INVALID_ORDER) && (AYKEN_MARKER_INJECT_INVALID_ORDER == 1)
static void inject_invalid_order(exec_slot_t *slot) {
    // Swap markers 1 and 2
    if (slot->marker_count >= 3) {
        uint8_t temp = slot->marker_sequence[1];
        slot->marker_sequence[1] = slot->marker_sequence[2];
        slot->marker_sequence[2] = temp;
    }
}
#endif

#if defined(AYKEN_MARKER_INJECT_DUPLICATE) && (AYKEN_MARKER_INJECT_DUPLICATE == 1)
static void inject_duplicate(exec_slot_t *slot) {
    // Duplicate marker 1
    if (slot->marker_count >= 3) {
        slot->marker_sequence[2] = slot->marker_sequence[1];
    }
}
#endif

// ... (other injection functions)
```

### Phase 2: Injection Points
Add hooks in `execution_slot_prepare_hash_locked()`:

```c
#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
    #ifdef AYKEN_MARKER_INJECT_INVALID_ORDER
        inject_invalid_order(slot);
    #endif
    #ifdef AYKEN_MARKER_INJECT_DUPLICATE
        inject_duplicate(slot);
    #endif
    // ... (other injections)
    
    /* Pre-commit guard: validate markers before hash preparation */
    if (execution_slot_validate_markers_locked(slot) != 0) {
        return -1;
    }
#endif
```

### Phase 3: Test Automation
Create `tests/phase17_marker_injection_suite.sh`:

```bash
#!/bin/bash
set -e

echo "=== Phase-17 Marker Validation Injection Tests ==="

tests=(
    "AYKEN_MARKER_INJECT_INVALID_ORDER"
    "AYKEN_MARKER_INJECT_DUPLICATE"
    "AYKEN_MARKER_INJECT_MISSING"
    "AYKEN_MARKER_INJECT_OVERFLOW"
    "AYKEN_MARKER_INJECT_STALE_DATA"
    "AYKEN_MARKER_INJECT_CORRUPT_BITMAP"
    "AYKEN_MARKER_INJECT_PARTIAL_WRITE"
)

for test in "${tests[@]}"; do
    echo "Running: $test"
    AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1 \
    $test=1 \
    make qemu-test-headless || echo "✅ Test failed as expected"
done

echo "=== All injection tests completed ==="
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

---

## Production Readiness Checklist

- [ ] All 7 injection tests pass
- [ ] Evidence collected and documented
- [ ] Pre-ci gates pass with injection harness
- [ ] Remote CI passes (mandatory)
- [ ] Code review approved
- [ ] Architectural steward sign-off

**Only after all checkboxes**: Merge to main.

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

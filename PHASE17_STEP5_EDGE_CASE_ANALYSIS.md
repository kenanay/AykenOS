# Phase 17 Step 5 - Edge Case Analysis

**Date**: 2026-05-02  
**Author**: Kiro (AI Assistant)  
**Purpose**: Kernel-grade validation semantics for marker sequence validation

---

## Critical Question

**If marker sequence is: `START → OUTPUT → VERIFY → COMPLETE` (wrong order), what happens?**

**Answer**: System MUST fail at validation point in `execution_slot_prepare_hash_locked()`

**Fail Point**: After `MARKER_VERIFY_PASS`, before `MARKER_RESULT_OK`

**State Transition**: `EXEC_SLOT_RUNNING` → `EXEC_SLOT_FAILED`

**Error Code**: `marker_error_code = MARKER_ERR_ORDER` (value: 2)

**Control Flow**: `return -EINVAL` (caller handles failed state)

---

## Error Code Semantics

### Current Implementation (PR #133)
```c
// Implicit error codes (magic numbers)
marker_error_code = 3;  // Overflow
```

### Required for Step 5 (Explicit Semantics)
```c
typedef enum {
    MARKER_ERR_NONE = 0,      // No error
    MARKER_ERR_COUNT = 1,     // Wrong marker count
    MARKER_ERR_ORDER = 2,     // Wrong sequence order
    MARKER_ERR_OVERFLOW = 3,  // Marker buffer overflow
    MARKER_ERR_DUPLICATE = 4, // Duplicate marker (optional)
} marker_error_code_t;
```

**Rationale**: Explicit enum for CI evidence, debug, reproducibility

---

## Architectural Layers

### Layer Separation (Critical)

```
┌─────────────────────────────────────────┐
│ CAPTURE LAYER (PR #133)                 │
│ - Writes markers to sequence            │
│ - Signals errors via marker_error_code  │
│ - NO validation, NO enforcement         │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│ VALIDATION LAYER (Step 5)               │
│ - Decides: valid or invalid             │
│ - Sets: marker_error_code (diagnostic)  │
│ - Returns: 0 (valid) or -EINVAL         │
│ - NO state mutation                     │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│ ENFORCEMENT LAYER (Caller)              │
│ - Enforces: fail-closed semantics       │
│ - Mutates: slot->state = FAILED         │
│ - Controls: execution flow              │
└─────────────────────────────────────────┘
```

### Responsibility Matrix

| Layer | Writes Data | Validates | Sets Error Code | Mutates State | Returns Status |
|-------|-------------|-----------|-----------------|---------------|----------------|
| Capture | ✅ | ❌ | ✅ (overflow) | ❌ | void |
| Validation | ❌ | ✅ | ✅ (count/order) | ❌ | int |
| Enforcement | ❌ | ❌ | ❌ | ✅ | int |

### Why This Matters

**Single Responsibility**:
- Each layer has ONE job
- No overlap, no confusion
- Clear ownership

**Future-Proof**:
- Validation logic can be reused
- No hidden side effects
- Easy to test in isolation

**Debug-Friendly**:
- State changes happen in ONE place
- Error codes trace back to source
- No race conditions on state

**Constitutional**:
- Validation = measurement
- Enforcement = action
- Clean separation of concerns

---

## Edge Case Matrix

### 1. Duplicate Marker

**Scenario**: Same marker captured twice
```
Sequence: [START, START, OUTPUT, ...]
```

**Detection Point**: `execution_slot_marker_capture_locked()` (optional) OR `execution_slot_validate_markers_locked()` (required)

**Current Behavior**: 
- Capture: Increments count, appends to sequence (no duplicate check)
- Validation: Detects wrong order (START appears twice, COMPLETE missing)

**Fail Semantics**:
- **Error Code**: `MARKER_ERR_ORDER` (sequence validation catches it)
- **State**: `EXEC_SLOT_FAILED`
- **Fail Point**: `execution_slot_validate_markers_locked()`
- **Return**: `-EINVAL`

**Optional Enhancement** (NOT for Phase 17):
- Add duplicate detection in capture layer
- Set `marker_error_code = MARKER_ERR_DUPLICATE`
- Fail-fast at capture time

**Decision**: Phase 17 uses sequence validation only (simpler, sufficient)

---

### 2. Missing Marker

**Scenario**: Required marker not captured
```
Expected: [START, OUTPUT, COMPLETE, VERIFY_START, VERIFY_PASS]
Actual:   [START, OUTPUT, COMPLETE, VERIFY_START]
                                                 ^^^ VERIFY_PASS missing
```

**Detection Point**: `execution_slot_validate_markers_locked()`

**Fail Semantics**:
- **Error Code**: `MARKER_ERR_ORDER` (sequence[3] != VERIFY_PASS)
- **State**: `EXEC_SLOT_FAILED`
- **Fail Point**: Validation loop at index 3
- **Return**: `-EINVAL`

**Alternative Detection** (count check):
```c
if (slot->marker_count != EXPECTED_COUNT) {
    marker_error_code = MARKER_ERR_COUNT;
    // ...
}
```

**Decision**: Both checks required (count + sequence)

---

### 3. Wrong Order

**Scenario**: Markers captured in incorrect sequence
```
Expected: [START, OUTPUT, COMPLETE, VERIFY_START, VERIFY_PASS]
Actual:   [START, COMPLETE, OUTPUT, VERIFY_START, VERIFY_PASS]
                  ^^^^^^^^^ swapped
```

**Detection Point**: `execution_slot_validate_markers_locked()`

**Fail Semantics**:
- **Error Code**: `MARKER_ERR_ORDER`
- **State**: `EXEC_SLOT_FAILED`
- **Fail Point**: Validation loop at index 1 (sequence[1] != OUTPUT)
- **Return**: `-EINVAL`

**Critical**: This is the PRIMARY validation check

---

### 4. Extra Marker (Overflow)

**Scenario**: More than 7 markers captured
```
Capture attempts: 8 markers
Buffer capacity: 7 markers
```

**Detection Point**: `execution_slot_marker_capture_locked()` (already implemented in PR #133)

**Current Behavior**:
```c
if (slot->marker_count < 7) {
    slot->marker_sequence[slot->marker_count] = marker;
    slot->marker_count++;
} else {
    slot->marker_error_code = 3;  // Overflow
}
```

**Fail Semantics**:
- **Error Code**: `MARKER_ERR_OVERFLOW` (value: 3)
- **State**: Set to `EXEC_SLOT_FAILED` in Step 5 validation
- **Fail Point**: `execution_slot_validate_markers_locked()` hard guard
- **Return**: `-EINVAL`

**Validation Guard** (Step 5):
```c
if (slot->marker_error_code != MARKER_ERR_NONE) {
    slot->state = EXEC_SLOT_FAILED;
    return -EINVAL;
}
```

**Critical**: Fail-fast already implemented in capture, fail-closed enforced in validation

---

### 5. Partial Sequence (Early Exit)

**Scenario**: Execution aborted before all markers captured
```
Expected: 5 pre-commit markers
Actual:   3 markers [START, OUTPUT, COMPLETE]
Reason:   Execution failed/aborted before validation
```

**Detection Point**: `execution_slot_validate_markers_locked()`

**Fail Semantics**:
- **Error Code**: `MARKER_ERR_COUNT`
- **State**: `EXEC_SLOT_FAILED`
- **Fail Point**: Count check
- **Return**: `-EINVAL`

**Special Case**: If execution already failed (e.g., `EXEC_SLOT_FAILED` from other error), validation may not run

**Guard**: Validation only runs on `EXEC_SLOT_RUNNING` → `EXEC_SLOT_COMPLETED` path

**Decision**: Count check catches this

---

## Validation Integration Point

### Exact Location
```c
int execution_slot_prepare_hash_locked(exec_slot_t *slot)
{
    // ... existing validation ...
    
    /* MARKER VALIDATION CHECKPOINT (PRE-COMMIT GUARD) */
#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
    /* Validation decides, enforcement acts (single responsibility) */
    if (execution_slot_validate_markers_locked(slot) != 0) {
        slot->state = EXEC_SLOT_FAILED;  // ← ENFORCEMENT: Single point of state mutation
        return -EINVAL;
    }
#endif
    
    /* Capture RESULT_OK marker AFTER validation passes */
#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
    execution_slot_marker_capture_locked(slot, MARKER_RESULT_OK);
#endif
    
    // ... continue with hash preparation ...
}
```

### Timing
- **After**: `MARKER_VERIFY_PASS` captured
- **Validation**: Checks 5 pre-commit markers
- **After Validation**: `MARKER_RESULT_OK` captured
- **Rationale**: Validation is the publish boundary guard, RESULT_OK confirms publish

### Marker Lifecycle
```
1. EXEC_START          → captured at transition to RUNNING
2. OUTPUT_WRITTEN      → captured after output write
3. COMPLETE_OK         → captured at transition to COMPLETED
4. VERIFY_START        → captured before validation
5. VERIFY_PASS         → captured after validation
   ↓
   [VALIDATION CHECKPOINT] ← Step 5 validates markers 1-5
   ↓
6. RESULT_OK           → captured AFTER validation passes
7. WAIT_OK             → captured in userspace (not validated in Phase 17)
```

---

## Fail Semantics Contract

### State Transition Rules

**Valid Path**:
```
EXEC_SLOT_RUNNING → validation PASS → EXEC_SLOT_COMPLETED
```

**Invalid Path**:
```
EXEC_SLOT_RUNNING → validation FAIL → EXEC_SLOT_FAILED
```

### Critical Requirements

1. **State Change MUST Precede Return**
```c
// ✅ CORRECT
slot->state = EXEC_SLOT_FAILED;
return -EINVAL;

// ❌ WRONG (ghost success)
return -EINVAL;
// (state unchanged = execution appears successful)
```

2. **No Soft Fail**
```c
// ❌ FORBIDDEN
if (validation_failed) {
    log_error();
    continue;  // CONSTITUTIONAL VIOLATION
}

// ✅ REQUIRED
if (validation_failed) {
    slot->state = EXEC_SLOT_FAILED;
    return -EINVAL;
}
```

3. **No Flag Bypass**
```c
// ❌ FORBIDDEN
#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
    if (validation_failed) {
        log_warning();  // Soft fail when flag ON
    }
#endif
// Continue execution regardless

// ✅ REQUIRED
#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
    if (validation_failed) {
        slot->state = EXEC_SLOT_FAILED;
        return -EINVAL;  // Hard fail when flag ON
    }
#endif
```

---

## Validation Function Specification

### Function Signature
```c
static int execution_slot_validate_markers_locked(exec_slot_t *slot);
```

### Implementation Pattern
```c
static int execution_slot_validate_markers_locked(exec_slot_t *slot)
{
    const uint8_t EXPECTED_COUNT = 5;
    
    /* Expected pre-commit marker sequence (RESULT_OK and WAIT_OK excluded) */
    static const uint8_t expected_seq[EXPECTED_COUNT] = {
        MARKER_EXEC_START,           // 0
        MARKER_EXEC_OUTPUT_WRITTEN,  // 1
        MARKER_EXEC_COMPLETE_OK,     // 2
        MARKER_VERIFY_START,         // 3
        MARKER_VERIFY_PASS,          // 4
    };
    
    /* HARD GUARD: Check for capture errors FIRST */
    if (slot->marker_error_code != MARKER_ERR_NONE) {
        return -EINVAL;  // Validation decides, caller enforces
    }
    
    /* DEFENSIVE: Overflow protection (redundant with capture, but explicit) */
    if (slot->marker_count > EXPECTED_COUNT) {
        slot->marker_error_code = MARKER_ERR_OVERFLOW;
        return -EINVAL;
    }
    
    /* COUNT CHECK: Verify exactly 5 pre-commit markers */
    if (slot->marker_count != EXPECTED_COUNT) {
        slot->marker_error_code = MARKER_ERR_COUNT;
        return -EINVAL;
    }
    
    /* SEQUENCE CHECK: Verify correct order */
    for (uint8_t i = 0; i < EXPECTED_COUNT; i++) {
        if (slot->marker_sequence[i] != expected_seq[i]) {
            slot->marker_error_code = MARKER_ERR_ORDER;
            return -EINVAL;
        }
    }
    
    return 0;  /* Valid sequence - caller decides enforcement */
}
```

### Architectural Separation

**Validation Layer** (this function):
- ✅ Decides: valid or invalid
- ✅ Sets: `marker_error_code` (diagnostic)
- ✅ Returns: success (0) or failure (-EINVAL)
- ❌ Does NOT: mutate `slot->state`

**Enforcement Layer** (caller):
- ✅ Enforces: fail-closed semantics
- ✅ Mutates: `slot->state = EXEC_SLOT_FAILED`
- ✅ Controls: execution flow

**Why This Separation Matters**:
- Clear ownership: validation measures, enforcement acts
- No double responsibility
- Future-proof: validation logic can be reused without side effects
- Debug-friendly: state changes happen in one place

### Error Handling
- **Capture errors**: Detected by hard guard (overflow from PR #133)
- **Count errors**: Detected by count check
- **Order errors**: Detected by sequence loop
- **All errors**: Set `marker_error_code`, return `-EINVAL`
- **State enforcement**: Caller sets `EXEC_SLOT_FAILED` (single responsibility)

---

## Phase Scope Boundaries

### Phase 17 (Current)
- ✅ 5 pre-commit markers (0-4)
- ✅ Fixed sequence
- ✅ Strict validation
- ✅ RESULT_OK captured AFTER validation (marker 5)
- ❌ NO WAIT_OK validation (marker 6 is userspace)
- ❌ NO optional markers
- ❌ NO extensibility

### Phase 18 (Future)
- Optional markers
- Extensible sequences
- Userspace marker validation
- Dynamic validation rules

**Critical**: Do NOT add Phase 18 features to Phase 17

---

## Testing Requirements

### Build Verification
```bash
# Flag OFF (default)
make clean && make kernel
# Expected: Compiles, no marker code active

# Flag ON (validation enabled)
make clean && make kernel AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1
# Expected: Compiles, validation active
```

### Functional Testing (Manual)
1. **Valid sequence (5 markers)**: Execution succeeds, RESULT_OK captured
2. **Wrong count**: `EXEC_SLOT_FAILED`
3. **Wrong order**: `EXEC_SLOT_FAILED`
4. **Overflow**: `EXEC_SLOT_FAILED`
5. **Partial sequence**: `EXEC_SLOT_FAILED`

### CI Gates
- All gates must pass with flag ON
- Pre-ci discipline: local verification before PR

---

## Summary

**Edge Cases Covered**: 5/5 ✅
- Duplicate marker → Caught by sequence validation
- Missing marker → Caught by count/sequence validation
- Wrong order → Caught by sequence validation
- Extra marker (overflow) → Caught by hard guard + defensive check
- Partial sequence → Caught by count validation

**Validation Scope**: 5 pre-commit markers ✅
- Validates: START, OUTPUT, COMPLETE, VERIFY_START, VERIFY_PASS
- Excludes: RESULT_OK (captured AFTER validation)
- Excludes: WAIT_OK (userspace, not validated in Phase 17)

**Architectural Separation**: Kernel-grade ✅
- **Capture**: Writes data, signals errors
- **Validation**: Decides validity, sets error codes
- **Enforcement**: Mutates state, controls flow
- **Single Responsibility**: Each layer has ONE job

**Fail Semantics**: Fail-closed ✅
- Validation returns: `-EINVAL` (no state mutation)
- Enforcement sets: `slot->state = EXEC_SLOT_FAILED`
- Control flow: `return -EINVAL`
- No soft fail, no bypass, no ghost success

**Integration Point**: Verified ✅
- Location: `execution_slot_prepare_hash_locked()`
- Timing: After VERIFY_PASS, validates 5 markers, then captures RESULT_OK
- Guard: Pre-commit boundary (publish guard)
- State mutation: Single point in caller

**Ready for Implementation**: ✅

---

**Next Steps**:
1. Wait for PR #133 remote CI → PASS
2. Merge PR #133
3. Pull main
4. Implement Step 5 per this specification
5. Test with flag ON and flag OFF
6. Create PR for Step 5

# Phase 17 Step 5 - Terminal Semantics Analysis

**Date**: 2026-05-02  
**Author**: Kiro (AI Assistant)  
**Purpose**: Pre-implementation analysis for Step 5 validation guard

---

## Critical Questions Answered

### 1. Is EXEC_SLOT_FAILED a terminal state?

**YES** ✅

**Evidence**:
```c
// kernel/sys/execution_slot.c:1615
int execution_slot_state_is_terminal(exec_slot_state_t state)
{
    return state == EXEC_SLOT_COMPLETED ||
           state == EXEC_SLOT_FAILED ||      // ← TERMINAL
           state == EXEC_SLOT_TIMEOUT ||
           state == EXEC_SLOT_ABORTED ||
           state == EXEC_SLOT_RESULT_MAPPED;
}
```

### 2. Can scheduler retry FAILED slots?

**NO** ❌

**Evidence**:
```c
// kernel/sys/execution_slot.c:614
static int execution_slot_can_transition(exec_slot_state_t from, exec_slot_state_t to)
{
    // ...
    case EXEC_SLOT_FAILED:
    case EXEC_SLOT_TIMEOUT:
    case EXEC_SLOT_ABORTED:
    default:
        return 0;  // ← NO transitions allowed FROM failed state
}
```

**State Machine Contract**:
- `EXEC_SLOT_FAILED` → **NO outgoing transitions**
- Once FAILED, slot is **immutable**
- Scheduler cannot pick up FAILED slots (pickup requires `EXEC_SLOT_READY → EXEC_SLOT_RUNNING`)

### 3. How does userspace see FAILED state?

**Returns `ESYS_V2_CONTEXT_ERROR`** ✅

**Evidence**:
```c
// kernel/sys/syscall_v2.c:1035
case EXEC_SLOT_FAILED:
case EXEC_SLOT_ABORTED:
    execution_slot_exit_critical(&slot_guard);
    return ESYS_V2_CONTEXT_ERROR;  // ← Userspace error code
```

**Userspace Contract**:
- `sys_v2_wait_execution()` returns error code
- NO result mapping occurs
- Execution is **definitively failed**

### 4. What happens to resources when slot transitions to FAILED?

**All resources released immediately** ✅

**Evidence**:
```c
// kernel/sys/execution_slot.c:755
if (next_state == EXEC_SLOT_FAILED ||
    next_state == EXEC_SLOT_TIMEOUT ||
    next_state == EXEC_SLOT_ABORTED) {
    execution_slot_release_bcib_backing_locked(slot);
    execution_slot_release_result_backing_locked(slot);
    execution_slot_release_output_backing_locked(slot);
    execution_slot_release_hash_backing_locked(slot);
}
```

**Resource Cleanup**:
- BCIB frames released
- Result frames released
- Output frames released
- Hash frames released
- Deadline cleared
- Target process latch cleared
- Waiters woken up

### 5. Does EXEC_SLOT_FLAG_TERMINAL exist?

**NO** ❌

**Evidence**: Searched entire codebase - no such flag exists.

**Existing Model**: State-based terminal checking via `execution_slot_state_is_terminal()`

---

## Step 5 Implementation Decisions

### ✅ DECISION 1: NO flag needed

**Rationale**:
- `EXEC_SLOT_FAILED` is already recognized as terminal
- State machine enforces immutability
- Adding flag would be redundant and risky

**Pattern**:
```c
slot->state = EXEC_SLOT_FAILED;  // Sufficient - already terminal
return -EINVAL;
```

### ✅ DECISION 2: Fail-closed enforcement point

**Location**: `execution_slot_prepare_hash_locked()`

**Timing**: After `MARKER_VERIFY_PASS`, before `MARKER_RESULT_OK`

**Rationale**:
- Validation happens AFTER output verification
- BEFORE result preparation
- Natural checkpoint in lifecycle

### ✅ DECISION 3: Validation scope

**Markers validated**: 6 kernel markers (0-5)

**Excluded**: `MARKER_WAIT_OK` (marker 6) - userspace only

**Expected sequence**:
```c
static const uint8_t expected_seq[6] = {
    MARKER_EXEC_START,           // 0
    MARKER_EXEC_OUTPUT_WRITTEN,  // 1
    MARKER_EXEC_COMPLETE_OK,     // 2
    MARKER_VERIFY_START,         // 3
    MARKER_VERIFY_PASS,          // 4
    MARKER_RESULT_OK             // 5
};
```

### ✅ DECISION 4: Error code semantics

**Capture layer (PR #133)**: Signals errors via `marker_error_code`

**Validation layer (Step 5)**: Enforces fail-closed via state transition

**Error codes**:
- `0` = No error
- `1` = Wrong count
- `2` = Wrong order
- `3` = Marker overflow (from PR #133)

### ✅ DECISION 5: Fail-closed pattern

**CORRECT**:
```c
if (validation_failed) {
    slot->state = EXEC_SLOT_FAILED;
    return -EINVAL;
}
```

**INCORRECT** (ghost success):
```c
if (validation_failed) {
    return -EINVAL;  // ← State unchanged = execution appears successful
}
```

---

## Step 5 Validation Guard Specification

### Function Signature
```c
static int execution_slot_validate_markers_locked(exec_slot_t *slot)
```

### Implementation Pattern
```c
static int execution_slot_validate_markers_locked(exec_slot_t *slot)
{
    const uint8_t EXPECTED_COUNT = 6;
    
    static const uint8_t expected_seq[EXPECTED_COUNT] = {
        MARKER_EXEC_START,
        MARKER_EXEC_OUTPUT_WRITTEN,
        MARKER_EXEC_COMPLETE_OK,
        MARKER_VERIFY_START,
        MARKER_VERIFY_PASS,
        MARKER_RESULT_OK
    };
    
    /* HARD GUARD: Check for capture errors FIRST */
    if (slot->marker_error_code != 0) {
        slot->state = EXEC_SLOT_FAILED;
        return -EINVAL;
    }
    
    /* Check count */
    if (slot->marker_count != EXPECTED_COUNT) {
        slot->marker_error_code = 1;
        slot->state = EXEC_SLOT_FAILED;
        return -EINVAL;
    }
    
    /* Check sequence order */
    for (uint8_t i = 0; i < EXPECTED_COUNT; i++) {
        if (slot->marker_sequence[i] != expected_seq[i]) {
            slot->marker_error_code = 2;
            slot->state = EXEC_SLOT_FAILED;
            return -EINVAL;
        }
    }
    
    return 0;  /* Valid sequence */
}
```

### Integration Point
```c
int execution_slot_prepare_hash_locked(exec_slot_t *slot)
{
    // ... existing validation ...
    
#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
    /* Validate marker sequence before preparing result */
    if (execution_slot_validate_markers_locked(slot) != 0) {
        return -1;  /* Slot already transitioned to FAILED */
    }
#endif
    
    /* Capture RESULT_OK marker after validation passes */
#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
    execution_slot_marker_capture_locked(slot, MARKER_RESULT_OK);
#endif
    
    // ... continue with hash preparation ...
}
```

---

## Architectural Guarantees

### ✅ Fail-Fast (PR #133)
- Capture stops after first error
- `marker_error_code` signals problem
- Deterministic trace (no garbage)

### ✅ Fail-Closed (Step 5)
- Invalid marker sequence → `EXEC_SLOT_FAILED`
- State transition is **irreversible**
- Scheduler cannot retry
- Userspace sees `ESYS_V2_CONTEXT_ERROR`
- Resources cleaned up immediately

### ✅ No Ghost Success
- Validation failure ALWAYS transitions state
- NO "validation failed but execution succeeded"
- Execution contract violation = execution failure

---

## Testing Requirements

### Build Verification
- Flag OFF: `make clean && make kernel` → PASS
- Flag ON: `make clean && make kernel AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1` → PASS

### CI Gates
- All gates must pass with flag ON
- Pre-ci discipline: local verification before PR

### Functional Testing
- Valid sequence → execution succeeds
- Invalid count → `EXEC_SLOT_FAILED`
- Invalid order → `EXEC_SLOT_FAILED`
- Capture error → `EXEC_SLOT_FAILED`

---

## Summary

**Terminal Semantics**: ✅ VERIFIED  
**State Machine Contract**: ✅ ENFORCED  
**Resource Cleanup**: ✅ AUTOMATIC  
**Userspace Visibility**: ✅ CLEAR ERROR  
**No Flag Needed**: ✅ CONFIRMED  

**Ready for Step 5 implementation**: ✅

---

**Next Steps**:
1. Wait for PR #133 remote CI → PASS
2. Merge PR #133
3. Pull main
4. Implement Step 5 validation guard per this specification
5. Test with flag ON and flag OFF
6. Create PR for Step 5

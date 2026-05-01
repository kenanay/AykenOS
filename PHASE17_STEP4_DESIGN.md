# Phase 17 Step 4: Add Marker Call Sites

## Objective
Add `execution_slot_marker_capture_locked()` calls at canonical lifecycle points.

## Canonical Marker Sequence
```
MARKER_EXEC_START           → Execution begins
MARKER_EXEC_OUTPUT_WRITTEN  → Output written
MARKER_EXEC_COMPLETE_OK     → Execution completed
MARKER_VERIFY_START         → Verification begins
MARKER_VERIFY_PASS          → Verification passed
MARKER_RESULT_OK            → Result ready
MARKER_WAIT_OK              → Wait completed
```

## Call Site Mapping

### 1. MARKER_EXEC_START
**Location:** `execution_slot_transition_locked()`
- **When:** Transition to `EXEC_SLOT_EXECUTING`
- **Rationale:** Execution phase begins

### 2. MARKER_EXEC_OUTPUT_WRITTEN
**Location:** `execution_slot_write_output_v1_locked()`
- **When:** After successful output write
- **Rationale:** Output data committed

### 3. MARKER_EXEC_COMPLETE_OK
**Location:** `execution_slot_finish_locked()`
- **When:** Transition to `EXEC_SLOT_EXECUTED`
- **Rationale:** Execution phase complete

### 4. MARKER_VERIFY_START
**Location:** `execution_slot_validate_output_locked()`
- **When:** Before validation begins
- **Rationale:** Verification phase begins

### 5. MARKER_VERIFY_PASS
**Location:** `execution_slot_validate_output_locked()`
- **When:** After successful validation
- **Rationale:** Verification passed

### 6. MARKER_RESULT_OK
**Location:** `execution_slot_prepare_result_locked()`
- **When:** After result preparation
- **Rationale:** Result ready for consumption

### 7. MARKER_WAIT_OK
**Location:** `execution_slot_finish_locked()`
- **When:** Transition to `EXEC_SLOT_COMPLETED`
- **Rationale:** Full lifecycle complete

## Implementation Rules

### 1. Placement
- Call AFTER state transition succeeds
- Call BEFORE returning from function
- Call inside `#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE` guard

### 2. Pattern
```c
#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
    execution_slot_marker_capture_locked(slot, MARKER_XXX);
#endif
```

### 3. NO Validation
- NO error checking
- NO return value
- NO failure path
- Validation deferred to Step 5

## Constitutional Compliance
- MEMORY.CONTRACT.VIOLATION=PASS (no unsafe ops)
- Phase: P4.4 (Development)

## Dependencies
- Step 2: Header definitions (merged)
- Step 3: Helper function (merged)

## Next Step
- Step 5: Add pre-commit validation guard

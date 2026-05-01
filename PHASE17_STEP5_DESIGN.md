# Phase 17 Step 5: Add Pre-Commit Validation Guard

## ⚠️ CRITICAL: System Behavior Change
**This is NOT a harmless step. Kernel behavior changes here.**

- Fail-closed becomes active
- Scheduler can be affected
- Determinism must be preserved
- Evidence chain must remain valid

## 🔴 4 Critical Decisions (Non-Negotiable)

### 1. 📍 VALIDATION EXACT CALL POINT

**ONLY ONE correct location:**
```c
execution_slot_prepare_hash_locked()
```

**Why this location:**
- ✅ After VERIFY_PASS
- ✅ Before RESULT_OK
- ✅ Rollback still possible
- ✅ Publish not yet done

**❌ WRONG locations:**
- `execution_slot_finish_locked()` → too late
- `execution_slot_transition_locked()` → state corruption risk
- Any other location → breaks evidence chain

### 2. ⚠️ FAIL SEMANTICS (MOST CRITICAL)

**❌ NEVER do this:**
```c
return execution_slot_finish_locked(...);  // RECURSION = STATE CORRUPTION
```

**✅ CORRECT model:**
```c
slot->state = EXEC_SLOT_FAILED;
slot->flags |= EXEC_SLOT_FLAG_TERMINAL;
slot->marker_error_code = error;
return -EINVAL;
```

**Why:**
- ✔ Direct state change
- ✔ Deterministic
- ✔ Side-effects controlled
- ✔ No recursion

### 3. 🧬 VALIDATION MODEL (3 Phases)

**Phase 1 (Step 4 - Current):**
- Incremental capture
- NO ordering check yet
- Write-only operations

**Phase 2 (Step 5 - This Step):**
- FULL SEQUENCE CHECK
- Pre-commit guard
- Expected sequence:
  ```
  EXEC_START → OUTPUT_WRITTEN → EXEC_COMPLETE_OK → 
  VERIFY_START → VERIFY_PASS → RESULT_OK
  ```
- ⚠️ WAIT_OK NOT included (userspace side)

**Phase 3 (Optional - Document Only):**
- Post-commit sanity check
- 👉 DO NOT implement in Step 5
- 👉 Only document for future

### 4. 🧠 VALIDATION SCOPE (CRITICAL DISTINCTION)

**❌ WRONG:**
```c
// Kernel validates all 7 markers
expected_mask = 0b01111111;  // WRONG - includes WAIT_OK
```

**✅ CORRECT:**
```c
// Kernel validates only 6 markers (0-5)
// WAIT_OK (marker 6) is userspace side
#define EXPECTED_KERNEL_MARKER_COUNT 6
#define EXPECTED_KERNEL_MARKER_MASK  0b00111111  // bits 0-5
```

**Marker Ownership:**
| Marker | Owner | Validated By |
|--------|-------|--------------|
| 0-5 | Kernel | Kernel (Step 5) |
| WAIT_OK | Userspace | NOT in Step 5 |

**⚠️ Missing this = system lockup**

## 📋 Implementation Skeleton

### Validation Function
```c
#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE

#define EXPECTED_KERNEL_MARKER_COUNT 6
#define EXPECTED_KERNEL_MARKER_MASK  0b00111111

/*
 * execution_slot_validate_markers_locked - Pre-commit marker validation
 * 
 * @slot: Execution slot
 * 
 * Validates that all required kernel-side markers were captured in correct sequence.
 * 
 * Expected sequence:
 *   MARKER_EXEC_START (0)
 *   MARKER_EXEC_OUTPUT_WRITTEN (1)
 *   MARKER_EXEC_COMPLETE_OK (2)
 *   MARKER_VERIFY_START (3)
 *   MARKER_VERIFY_PASS (4)
 *   MARKER_RESULT_OK (5)
 * 
 * WAIT_OK (6) is userspace-side and NOT validated here.
 * 
 * Returns:
 *   0 on success
 *   -EINVAL on validation failure
 */
static int execution_slot_validate_markers_locked(exec_slot_t *slot)
{
    if (!slot || !slot->in_use) {
        return -EINVAL;
    }

    /* Check marker count */
    if (slot->marker_count != EXPECTED_KERNEL_MARKER_COUNT) {
        return -EINVAL;
    }

    /* Check marker bitmap */
    if (slot->marker_bitmap != EXPECTED_KERNEL_MARKER_MASK) {
        return -EINVAL;
    }

    return 0;
}

#endif /* AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE */
```

### Call Site (in execution_slot_prepare_hash_locked)
```c
int execution_slot_prepare_hash_locked(exec_slot_t *slot)
{
    /* ... existing validation ... */

#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
    /* Pre-commit marker validation guard */
    int marker_validation_result = execution_slot_validate_markers_locked(slot);
    if (marker_validation_result != 0) {
        /* FAIL-CLOSED: Direct state change, NO recursion */
        slot->state = EXEC_SLOT_FAILED;
        slot->flags |= EXEC_SLOT_FLAG_TERMINAL;
        slot->marker_error_code = marker_validation_result;
        return -EINVAL;
    }
#endif

    /* ... continue with hash preparation ... */
}
```

## 🚨 BIGGEST TRAP (DO NOT DO THIS)

**❌ "Validation failed but continue anyway"**

This breaks:
- Determinism
- Evidence chain
- Constitutional guarantees

**Fail-closed = NO negotiation**

## 🧭 Implementation Checklist

### Before Implementation
- [ ] PR #131 CI PASS
- [ ] PR #131 merged
- [ ] Flag ON: `-DAYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1`
- [ ] Evidence validation:
  - [ ] `marker_count` increases
  - [ ] `marker_bitmap` populates
  - [ ] Sequence is logical

### During Implementation
- [ ] Add validation function with correct mask
- [ ] Add call site in `execution_slot_prepare_hash_locked()`
- [ ] Use direct state change (NO recursion)
- [ ] Guard with `#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE`

### After Implementation
- [ ] Local CI: ALL GATES PASS
- [ ] Remote CI: ALL GATES PASS
- [ ] Evidence: validation triggers correctly
- [ ] Evidence: fail-closed works on invalid sequence

## Constitutional Compliance
- MEMORY.CONTRACT.VIOLATION=PASS (no unsafe ops)
- DETERMINISM.GLOBAL=PASS (no global state mutation)
- Phase: P4.4 (Development)

## Dependencies
- Step 2: Header definitions (merged)
- Step 3: Helper function (merged)
- Step 4: Call sites (pending merge)
- Evidence run with flag ON

## ⚠️ FINAL WARNING

**DO NOT implement Step 5 alone.**

Why:
- State machine is sensitive
- Fail path is critical
- Small error = systemic failure

**Correct approach:**
1. Wait for PR #131 merge
2. Enable flag and validate evidence
3. Implement together with review

## Next Steps After Step 5
- Step 6: Enable flag in default build
- Step 7: Add unit tests
- Step 8: Integration testing

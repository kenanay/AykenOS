# Phase-17 Execution Marker Validation Integration Design

**Authority:** Kenan AY - Architectural Steward  
**Status:** DESIGN ONLY (Implementation NOT Started)  
**Phase:** 17  
**Date:** 2026-05-01  
**Purpose:** Define integration points for marker validation into production execution pipeline

---

## 1. Purpose

Integrate the execution marker validation sandbox module into the production execution pipeline in a **controlled, feature-flag gated, and fail-closed** manner.

**This document is NOT code integration.** It defines integration points, failure semantics, and rollout strategy.

---

## 2. Current State

### Foundation Complete (PR #126 Merged)
- ✅ `kernel/include/execution_marker_validation.h` - Pure interface
- ✅ `kernel/sys/execution_marker_validation.c` - Deterministic validation logic
- ✅ `tests/unit/execution_marker_validation_test.c` - 13/13 PASS
- ✅ `scripts/ci/ci-gate-execution-marker-isolation.sh` - Isolation gate
- ✅ `scripts/ci/ci-gate-execution-slot-integrity.sh` - Protection gate

### NOT Yet Done (Intentional)
- ❌ No calls into `execution_slot.c`
- ❌ Runtime marker validation NOT active
- ❌ BCIB execution behavior unchanged
- ❌ No feature flag defined

**Rationale:** Foundation must be proven isolated before integration.

---

## 3. Non-Goals (Out of Scope)

Phase-17 does **NOT** include:

- ❌ BCIB interpreter implementation
- ❌ AI runtime integration
- ❌ Semantic output validation
- ❌ `execution_slot.c` refactoring
- ❌ State machine modification
- ❌ Performance optimization (proof before optimization)

**Why Out of Scope?**
> These features either break determinism or belong to Phase-18+

---

## 4. Feature Flag

### Proposed Flag

```c
AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
```

**Default:** `0` (OFF)

**Semantics:**
- `OFF` → Existing behavior preserved exactly (no runtime overhead)
- `ON` → Marker sequence capture + validation active

**Build Integration:**
```c
// kernel/include/execution_slot.h
#ifndef AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
#define AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE 0
#endif
```

**Makefile:**
```makefile
# Default: OFF
AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE ?= 0

KERNEL_DEFINES += -DAYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=$(AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE)
```

---

## 5. Canonical Marker Sequence

### Immutable Order

```
MARKER_EXEC_START           (0)
MARKER_EXEC_OUTPUT_WRITTEN  (1)
MARKER_EXEC_COMPLETE_OK     (2)
MARKER_VERIFY_START         (3)
MARKER_VERIFY_PASS          (4)
MARKER_RESULT_OK            (5)
MARKER_WAIT_OK              (6)
```

**Rule:** This order is **IMMUTABLE**. Changes require spec update.

---

## 6. State Machine Integration Points

### Current Production State Machine

```
EXEC_SLOT_CREATED
    ↓
EXEC_SLOT_READY (enqueued)
    ↓
EXEC_SLOT_RUNNING (picked up by scheduler)
    ↓
EXEC_SLOT_COMPLETED (execution finished)
    ↓
EXEC_SLOT_RESULT_MAPPED (result published to userspace)
```

### Marker Emission Points

| Marker | Production Call Site | State | Function |
|--------|---------------------|-------|----------|
| `EXEC_START` | `execution_slot_pickup_locked()` | `EXEC_SLOT_RUNNING` | Scheduler pickup |
| `EXEC_OUTPUT_WRITTEN` | `execution_slot_write_output_v()` | `EXEC_SLOT_RUNNING` | Output buffer write |
| `EXEC_COMPLETE_OK` | `execution_slot_finish_locked()` | `EXEC_SLOT_RUNNING → COMPLETED` | Execution complete |
| `VERIFY_START` | `execution_slot_prepare_result_locked()` | `EXEC_SLOT_COMPLETED` | Verification start |
| `VERIFY_PASS` | `execution_slot_prepare_hash_locked()` | `EXEC_SLOT_COMPLETED` | Hash computed |
| `RESULT_OK` | `execution_slot_record_result_mapping_locked()` | `EXEC_SLOT_RESULT_MAPPED` | Result published |
| `WAIT_OK` | Userspace `wait_result` syscall | N/A | Userspace retrieval |

**Critical Rule:**
> Markers are emitted **after** the corresponding operation succeeds, NOT before.

---

---

## 7.1. Return Value Enforcement (CRITICAL)

### Mandatory Check Rule

**ALL marker emission call sites MUST check return value.**

**Correct Pattern:**
```c
int result = execution_slot_emit_marker_locked(slot, MARKER_EXEC_START);
if (result != 0) {
    // Handle failure: set terminal state, emit evidence
    slot->state = EXEC_SLOT_FAILED;
    slot->flags |= EXEC_SLOT_FLAG_TERMINAL;
    slot->marker_error_code = result;
    execution_slot_emit_marker_validation_failure(slot, result);
    return -EINVAL;
}
```

**FORBIDDEN Pattern:**
```c
// ❌ WRONG: Unchecked call
execution_slot_emit_marker_locked(slot, MARKER_EXEC_START);
// Execution continues even if validation failed → SILENT CORRUPTION
```

### Enforcement Strategy

**Code Review:**
- Every PR adding marker emission MUST show return value check
- Reviewer MUST verify no unchecked calls

**CI Gate (Future):**
- Static analysis: detect unchecked `execution_slot_emit_marker_locked()` calls
- Fail CI if unchecked call found

**Rationale:**
> Unchecked return value = validation bypass = silent corruption = determinism violation

**Impact of Violation:**
- Validation fails but execution continues
- Invalid marker sequence reaches userspace
- Determinism guarantee broken
- CI evidence invalid

**Rule:**
> **NO EXCEPTIONS.** Every call site checks return value. Period.

---

## 7.2. Validation Ordering: Strict Sequential

### Phase 1 (Incremental): Strict Sequential Check

**Rule:** `marker == last_marker + 1` (exact next marker)

**Enforces:**
- Exact sequential order
- No gaps
- No backward markers
- No duplicate markers

**Catches:**
- Gap (e.g., 0 → 2, missing 1)
- Backward (e.g., 0 → 2 → 1)
- Duplicate (e.g., 0 → 1 → 1)

**Example:**
```c
// Valid: 0 → 1 → 2 → 3 → 4 → 5 → 6 (exact sequence)
// Invalid: 0 → 2 (gap, missing 1)
// Invalid: 0 → 1 → 0 (backward)
// Invalid: 0 → 1 → 1 (duplicate)
```

**Implementation:**
```c
// Check strict sequential ordering
if (slot->last_marker != MARKER_INVALID) {
    if (marker != slot->last_marker + 1) {
        return -EINVAL;  // Not exact next marker
    }
}
```

---

### Phase 2 (Pre-Commit): Verified Prefix Check

**Rule:** Markers through `MARKER_VERIFY_PASS` are present, exact count

**Enforces:**
- Verified prefix is complete before result mapping
- Prefix bitmap matches `MARKER_VERIFY_PREFIX_MASK`
- Prefix count matches `MARKER_VERIFY_PREFIX_COUNT`
- Last marker is `MARKER_VERIFY_PASS`

**Example:**
```c
// Valid before result mapping: bitmap = 0b00011111 (markers 0..4 present)
// Invalid before result mapping: bitmap = 0b00001111 (VERIFY_PASS missing)
```

---

### Phase 3 (Publish Boundary): Result Prefix Check

**Rule:** `MARKER_RESULT_OK` is emitted only after result mapping succeeds.

**Enforces:**
- Verified prefix is still intact before mapping
- `RESULT_OK` is the exact next marker after successful mapping
- Mapping is not attempted from invalid marker state

**Example:**
```c
// Valid before RESULT_OK: bitmap = 0b00011111 (markers 0..4 present)
// Valid after RESULT_OK:  bitmap = 0b00111111 (markers 0..5 present)
```

---

### Phase 4 (Wait Path): Full Sequence Check

**Rule:** Full canonical sequence is validated only after `MARKER_WAIT_OK`.

**Enforces:**
- Complete sequence is checked after the final marker can exist
- Full bitmap matches `MARKER_MASK`
- Full count matches `MARKER_COUNT`
- No missing markers before successful `wait_result` return

**Example:**
```c
// Valid after WAIT_OK: bitmap = 0b01111111 (all 7 markers present)
// Invalid after WAIT_OK: bitmap = 0b00111111 (WAIT_OK missing)
```

**Critical Rule:**
> `MARKER_WAIT_OK` is emitted in the userspace `wait_result` syscall. Therefore, any check requiring all 7 markers MUST run in the wait path, not in `execution_slot_prepare_hash_locked()`.

---

### Rationale for Strict Sequential

**Why strict sequential in Phase 1?**
- **Header contract alignment**: `execution_marker_validate_transition()` enforces strict sequential
- **Determinism guarantee**: Exact sequence required for correctness
- **Fail-fast**: Errors detected immediately at emission time
- **CI alignment**: Existing tests assume strict sequential

**Why prefix validation in Phase 2?**
- Pre-commit guard: Markers through `VERIFY_PASS` must be complete before publish
- Correct timing: `RESULT_OK` and `WAIT_OK` cannot exist before result mapping and wait
- Fail-closed: Invalid pre-publish state is rejected before result mapping
- Header alignment: Prefixes are valid inputs to `execution_marker_validate()`

**Why full validation in Phase 4?**
- Complete sequence can only exist after `WAIT_OK`
- Final evidence proves execution integrity before successful wait return
- Missing result or wait markers are caught at the first point where they can be required

**Rule:**
> Phase 1 = strict sequential (fail-fast). Phase 2 = verified prefix (fail-closed before mapping). Phase 3 = result boundary guard. Phase 4 = full sequence after `WAIT_OK` (fail-closed before successful wait return).

---

### Future Extension: Monotonic Ordering (Phase-18+)

**Deferred to Phase-18:**
- Monotonic ordering (`marker > last_marker`)
- Optional markers support
- Flexible evolution

**Rationale for Deferral:**
- Current header contract is strict sequential
- API change required for monotonic support
- Migration path needs design
- Phase-17 focuses on determinism proof, not flexibility

**Migration Path (Future):**
1. Update `execution_marker_validate_transition()` contract
2. Add optional marker support to header
3. Update CI tests for monotonic validation
4. Migrate Phase 1 to monotonic check

**Current Decision:**
> Phase-17 uses **strict sequential** for every emitted transition. Monotonic is future work.

---

## 7. Validation Strategy (Staged)

### Phase 1: Incremental Validation (Per-Marker)

**Location:** Every marker emission site

**Timing:** Immediately after marker emission

**Purpose:** Fail-fast detection of transition violations

**Logic:**
```c
static inline int execution_slot_emit_marker_locked(exec_slot_t *slot,
                                                    execution_marker_t marker)
{
#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
    // Validate transition
    if (slot->last_marker != MARKER_INVALID) {
        marker_validation_result_t result;
        result = execution_marker_validate_transition(slot->last_marker, marker);
        
        if (result != MARKER_VALIDATION_OK) {
            // Fail-fast: invalid transition detected
            execution_slot_emit_marker_validation_failure(slot, result);
            slot->state = EXEC_SLOT_FAILED;
            slot->flags |= EXEC_SLOT_FLAG_TERMINAL;
            slot->marker_error_code = result;
            return -EINVAL;
        }
    }
    
    // Check duplicate
    if (slot->marker_bitmap & (1 << marker)) {
        execution_slot_emit_marker_validation_failure(slot, MARKER_VALIDATION_DUPLICATE);
        slot->state = EXEC_SLOT_FAILED;
        slot->flags |= EXEC_SLOT_FLAG_TERMINAL;
        slot->marker_error_code = MARKER_VALIDATION_DUPLICATE;
        return -EINVAL;
    }
    
    // Record marker
    slot->marker_bitmap |= (1 << marker);
    slot->last_marker = marker;
    slot->marker_count++;
#endif
    return 0;
}
```

**Rationale:**
1. **Fail-fast**: Errors detected immediately, not post-mortem
2. **Deterministic**: Transition validation at emission time
3. **Debuggable**: Clear failure point in execution trace
4. **Safe**: No half-committed state possible

---

### Phase 2: Verified Prefix Validation (Pre-Commit)

**Location:** `execution_slot_prepare_hash_locked()` (after hash computation)

**Timing:** After VERIFY_PASS, before result mapping

**Purpose:** Final pre-publish guard for markers that can exist before result mapping

**Logic:**
```c
static int execution_slot_prepare_hash_locked(exec_slot_t *slot)
{
    // ... existing hash computation ...
    
#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
    // Verified prefix validation (markers 0..4)
    marker_validation_result_t validation_result;
    
    if (slot->marker_bitmap != MARKER_VERIFY_PREFIX_MASK ||
        slot->marker_count != MARKER_VERIFY_PREFIX_COUNT ||
        slot->last_marker != MARKER_VERIFY_PASS) {
        execution_slot_emit_marker_validation_failure(slot,
                                                      MARKER_VALIDATION_MISSING);
        slot->state = EXEC_SLOT_FAILED;
        slot->flags |= EXEC_SLOT_FLAG_TERMINAL;
        slot->marker_error_code = MARKER_VALIDATION_MISSING;
        execution_slot_release_hash_backing_locked(slot);
        return -EINVAL;
    }

    // Reconstruct marker array from bitmap
    execution_marker_t markers[MARKER_COUNT];
    uint8_t count = 0;
    for (uint8_t i = 0; i < MARKER_COUNT; i++) {
        if (slot->marker_bitmap & (1 << i)) {
            markers[count++] = i;
        }
    }
    
    validation_result = execution_marker_validate(markers, count);
    
    if (validation_result != MARKER_VALIDATION_OK) {
        // Fail-closed: do NOT proceed to mapping
        execution_slot_emit_marker_validation_failure(slot, validation_result);
        slot->state = EXEC_SLOT_FAILED;
        slot->flags |= EXEC_SLOT_FLAG_TERMINAL;
        slot->marker_error_code = validation_result;
        execution_slot_release_hash_backing_locked(slot);
        return -EINVAL;
    }
#endif

    // ... existing hash finalization ...
    return 0;
}
```

**Rationale:**
1. **Pre-commit guard**: Validation before result mapping (rollback still possible)
2. **Correct timing**: Only markers through `VERIFY_PASS` can exist at this point
3. **Safe failure point**: Hash can be released, no userspace visibility yet
4. **Terminal state**: Direct state set, no `finish_locked()` recursion

---

### Phase 3: Result Mapping Guard (Publish Boundary)

**Location:** `execution_slot_record_result_mapping_locked()`

**Timing:** Before and after result mapping

**Purpose:** Guard the publish boundary and emit `RESULT_OK` only after mapping succeeds

**Logic:**
```c
int execution_slot_record_result_mapping_locked(exec_slot_t *slot,
                                                uint64_t mapped_result_va,
                                                uint64_t mapped_hash_va,
                                                uint64_t map_flags)
{
    // ... existing validation ...

#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
    // Pre-publish guard: RESULT_OK cannot be emitted safely unless the
    // verified prefix is still intact.
    if (slot->marker_bitmap != MARKER_VERIFY_PREFIX_MASK ||
        slot->marker_count != MARKER_VERIFY_PREFIX_COUNT ||
        slot->last_marker != MARKER_VERIFY_PASS) {
        execution_slot_emit_marker_validation_failure(slot,
                                                      MARKER_VALIDATION_MISSING);
        slot->state = EXEC_SLOT_FAILED;
        slot->flags |= EXEC_SLOT_FLAG_TERMINAL;
        slot->marker_error_code = MARKER_VALIDATION_MISSING;
        return -EINVAL;
    }
#endif

    // ... existing mapping logic ...

#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
    int result = execution_slot_emit_marker_locked(slot, MARKER_RESULT_OK);
    if (result != 0) {
        // The pre-publish guard should make this impossible under the slot lock.
        execution_slot_runtime_panic("marker_result_emit_invariant_violation",
                                     slot,
                                     slot->state,
                                     EXEC_SLOT_FAILED);
    }
#endif

    return 0;
}
```

**Rationale:**
1. **Pre-publish guard**: Invalid prefix state cannot be mapped
2. **Marker timing**: `RESULT_OK` is emitted after mapping succeeds
3. **Invariant panic**: Failure after the pre-publish guard indicates a logic bug

---

### Phase 4: Full Sequence Validation (Wait Path)

**Location:** Userspace `wait_result` syscall path

**Timing:** After `WAIT_OK` is emitted, before returning wait success

**Purpose:** Final guard for the complete canonical marker sequence

**Logic:**
```c
static int execution_slot_wait_result_locked(exec_slot_t *slot)
{
    // ... existing wait/result retrieval logic ...

#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
    if (slot->marker_bitmap != MARKER_RESULT_PREFIX_MASK ||
        slot->marker_count != MARKER_RESULT_PREFIX_COUNT ||
        slot->last_marker != MARKER_RESULT_OK) {
        execution_slot_emit_marker_validation_failure(slot,
                                                      MARKER_VALIDATION_MISSING);
        slot->state = EXEC_SLOT_FAILED;
        slot->flags |= EXEC_SLOT_FLAG_TERMINAL;
        slot->marker_error_code = MARKER_VALIDATION_MISSING;
        return -EINVAL;
    }

    int result = execution_slot_emit_marker_locked(slot, MARKER_WAIT_OK);
    if (result != 0) {
        return -EINVAL;
    }

    if (slot->marker_bitmap != MARKER_MASK ||
        slot->marker_count != MARKER_COUNT) {
        execution_slot_emit_marker_validation_failure(slot,
                                                      MARKER_VALIDATION_MISSING);
        slot->state = EXEC_SLOT_FAILED;
        slot->flags |= EXEC_SLOT_FLAG_TERMINAL;
        slot->marker_error_code = MARKER_VALIDATION_MISSING;
        return -EINVAL;
    }
#endif

    return 0;
}
```

**Rationale:**
1. **Correct timing**: The full 7-marker sequence can only exist after `WAIT_OK`
2. **Fail-closed wait**: Invalid final sequence returns failure instead of wait success
3. **Evidence completeness**: Final evidence includes `RESULT_OK` and `WAIT_OK`

---

### Validation Model Summary

| Phase | Location | Timing | Purpose | Failure Mode |
|-------|----------|--------|---------|--------------|
| **1. Incremental** | Marker emission | Per-marker | Fail-fast transition check | `EXEC_SLOT_FAILED` (direct) |
| **2. Verified Prefix** | `prepare_hash_locked()` | Pre-commit | Guard markers 0..4 before mapping | `EXEC_SLOT_FAILED` (direct) |
| **3. Result Boundary** | `record_result_mapping_locked()` | Publish boundary | Guard prefix, emit `RESULT_OK` | `EXEC_SLOT_FAILED` or `panic()` |
| **4. Full Sequence** | `wait_result` syscall | Before wait success | Validate all 7 markers | `EXEC_SLOT_FAILED` (direct) |

**Critical Rule:**
> Normal validation failures use **direct state assignment**. NO `finish_locked()` recursion. Panic is reserved for impossible invariant violations after a preflight guard.

---

## 8. Failure Semantics (Fail-Closed)

### Flag OFF (Default)

```c
// No validation, no overhead
// Existing behavior preserved exactly
```

### Flag ON + Validation PASS

```c
// Normal execution continues
// Result published to userspace
// No evidence generated (validation is transparent)
```

### Flag ON + Validation FAIL

**Phase 1 Failure (Incremental):**
```c
// Marker emission fails
int result = execution_slot_emit_marker_locked(slot, marker);
if (result != 0) {
    // CRITICAL: Caller MUST check return value
    slot->state = EXEC_SLOT_FAILED;
    slot->flags |= EXEC_SLOT_FLAG_TERMINAL;
    slot->marker_error_code = result;  // Record failure reason
    execution_slot_emit_marker_validation_failure(slot, result);
    return -EINVAL;
}
```

**Phase 2 Failure (Pre-Commit Prefix):**
```c
// Hash preparation / verified-prefix validation fails
execution_slot_prepare_hash_locked(slot);
// Returns: -EINVAL

// Internal handling:
slot->state = EXEC_SLOT_FAILED;
slot->flags |= EXEC_SLOT_FLAG_TERMINAL;
slot->marker_error_code = validation_result;  // Record failure reason
execution_slot_release_hash_backing_locked(slot);
execution_slot_emit_marker_validation_failure(slot, result);
return -EINVAL;
```

**Phase 3 Failure (Result Boundary):**
```c
// Pre-publish prefix guard fails before mapping
slot->state = EXEC_SLOT_FAILED;
slot->flags |= EXEC_SLOT_FLAG_TERMINAL;
slot->marker_error_code = validation_result;
execution_slot_emit_marker_validation_failure(slot, result);
return -EINVAL;
```

**Phase 3 Invariant Violation (Post-Mapping Emit):**
```c
// Should NEVER happen (invariant violation)
execution_slot_runtime_panic("marker_result_emit_invariant_violation",
                             slot,
                             slot->state,
                             EXEC_SLOT_FAILED);
```

**Phase 4 Failure (Wait Path Full Sequence):**
```c
// Full sequence validation fails before successful wait_result return
slot->state = EXEC_SLOT_FAILED;
slot->flags |= EXEC_SLOT_FLAG_TERMINAL;
slot->marker_error_code = validation_result;
execution_slot_emit_marker_validation_failure(slot, result);
return -EINVAL;
```

**Critical Rules:**

1. **NO `finish_locked()` recursion**
   - Direct state assignment: `slot->state = EXEC_SLOT_FAILED`
   - Avoids double-finish and state corruption

2. **Terminal flag set**
   - `slot->flags |= EXEC_SLOT_FLAG_TERMINAL`
   - Prevents scheduler retry

3. **Error code recorded**
   - `slot->marker_error_code = validation_result`
   - Scheduler visibility, CI evidence, debug

4. **Evidence emission**
   - `execution_slot_emit_marker_validation_failure(slot, result)`
   - Debugcon marker for CI/debug analysis

5. **Resource cleanup**
   - Phase 2: Release hash backing on prefix validation failure
   - Phase 3: No result buffer publish when pre-publish guard fails
   - Phase 4: No successful `wait_result` return when full sequence validation fails

6. **Return value enforcement (CRITICAL)**
   - **ALL call sites MUST check return value**
   - Failure to check = silent corruption
   - Design requirement: No unchecked calls

**Failure Codes:**

| Validation Result | Slot State | Terminal | Error Code | Evidence Marker |
|-------------------|------------|----------|------------|-----------------|
| `MARKER_VALIDATION_INVALID_ORDER` | `EXEC_SLOT_FAILED` | YES | `MARKER_ERROR_INVALID_ORDER` | `[[MARKER_VALIDATION_FAIL]] reason=INVALID_ORDER` |
| `MARKER_VALIDATION_MISSING` | `EXEC_SLOT_FAILED` | YES | `MARKER_ERROR_MISSING` | `[[MARKER_VALIDATION_FAIL]] reason=MISSING` |
| `MARKER_VALIDATION_DUPLICATE` | `EXEC_SLOT_FAILED` | YES | `MARKER_ERROR_DUPLICATE` | `[[MARKER_VALIDATION_FAIL]] reason=DUPLICATE` |
| `MARKER_VALIDATION_OUT_OF_BOUNDS` | `EXEC_SLOT_FAILED` | YES | `MARKER_ERROR_OUT_OF_BOUNDS` | `[[MARKER_VALIDATION_FAIL]] reason=OUT_OF_BOUNDS` |

**Evidence Format:**
```
[[MARKER_VALIDATION_FAIL]] exec_id=<ID> generation=<GEN> reason=<REASON> error_code=<CODE> bitmap=0b<BITMAP> last=<LAST> count=<COUNT>
```

**Example:**
```
[[MARKER_VALIDATION_FAIL]] exec_id=123 generation=1 reason=INVALID_ORDER error_code=1 bitmap=0b0001111 last=3 count=4
```

**Rule:**
> Validation failure = **permanent failure** (no retry, no recovery, terminal state)

---

## 9. Data Structure (Bitmap Design)

### Marker Capture Buffer

**Location:** Inside `exec_slot_t` structure

**Proposed Addition:**
```c
// kernel/include/execution_slot.h
typedef struct exec_slot {
    // ... existing fields ...

#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
    uint8_t marker_bitmap;      // 7 bits (one per marker)
    uint8_t last_marker;        // Last emitted marker (for ordering)
    uint8_t marker_count;       // Total markers emitted
    uint8_t marker_error_code;  // Validation failure reason (if failed)
#endif
} exec_slot_t;
```

**Rationale:**

**marker_error_code Addition:**
1. **Scheduler visibility**: Failure reason accessible to scheduler
2. **CI evidence**: Validation failure code in slot state
3. **Debug**: Clear failure reason without parsing logs
4. **Telemetry**: Future observability integration

**Values:**
```c
#define MARKER_ERROR_NONE                0
#define MARKER_ERROR_INVALID_ORDER       1
#define MARKER_ERROR_MISSING             2
#define MARKER_ERROR_DUPLICATE           3
#define MARKER_ERROR_OUT_OF_BOUNDS       4
```

**Rationale:**

**Bitmap Advantages:**
1. **O(1) duplicate detection**: `if (bitmap & (1 << marker))`
2. **O(1) presence check**: Single bit test
3. **Compact**: 1 byte for 7 markers (vs 7 bytes for array)
4. **Deterministic**: No iteration needed for validation
5. **Debug-friendly**: Easy to visualize (0b0111111 = all markers present)

**last_marker Advantages:**
1. **Ordering enforcement**: Incremental transition validation
2. **Fail-fast**: Detect out-of-order immediately
3. **No array scan**: Direct comparison

**Memory Impact:**
- Flag OFF: 0 bytes (compile-time removed)
- Flag ON: 4 bytes per slot (bitmap + last + count + padding)
- **Savings**: 5 bytes per slot vs array design (9 bytes → 4 bytes)

**Initialization:**
```c
slot->marker_bitmap = 0;
slot->last_marker = MARKER_INVALID;  // 0xFF
slot->marker_count = 0;
slot->marker_error_code = MARKER_ERROR_NONE;
```

**Constants (Future-Proof):**
```c
#define MARKER_INVALID 0xFF
#define MARKER_MASK ((1u << MARKER_COUNT) - 1u)  // 0b01111111 for 7 markers
#define MARKER_VERIFY_PREFIX_COUNT ((uint8_t)(MARKER_VERIFY_PASS + 1u))
#define MARKER_VERIFY_PREFIX_MASK ((uint8_t)((1u << MARKER_VERIFY_PREFIX_COUNT) - 1u))
#define MARKER_RESULT_PREFIX_COUNT ((uint8_t)(MARKER_RESULT_OK + 1u))
#define MARKER_RESULT_PREFIX_MASK ((uint8_t)((1u << MARKER_RESULT_PREFIX_COUNT) - 1u))
```

**Marker Emission:**
```c
// Check duplicate
if (slot->marker_bitmap & (1 << marker)) {
    return -EINVAL;  // Duplicate detected
}

// Check strict sequential ordering (if not first marker)
// Phase 1: Strict sequential (exact next marker)
if (slot->last_marker != MARKER_INVALID) {
    if (marker != slot->last_marker + 1) {
        return -EINVAL;  // Not exact next marker (gap or backward)
    }
}

// Record marker
slot->marker_bitmap |= (1 << marker);
slot->last_marker = marker;
slot->marker_count++;
```

**Pre-Commit Prefix Validation:**
```c
// Check markers through VERIFY_PASS before result mapping
if (slot->marker_bitmap != MARKER_VERIFY_PREFIX_MASK ||
    slot->marker_count != MARKER_VERIFY_PREFIX_COUNT ||
    slot->last_marker != MARKER_VERIFY_PASS) {
    return -EINVAL;  // Missing or invalid verified prefix
}
```

**Full Sequence Validation:**
```c
// Check all markers after WAIT_OK, before successful wait_result return
if (slot->marker_bitmap != MARKER_MASK ||
    slot->marker_count != MARKER_COUNT ||
    slot->last_marker != MARKER_WAIT_OK) {
    return -EINVAL;  // Missing or invalid final sequence
}
```

**Rationale for Strict Sequential:**
- **Header contract alignment**: `execution_marker_validate_transition()` enforces strict sequential
- **Determinism guarantee**: Exact sequence required
- **Fail-fast**: Errors detected immediately
- **Consistency**: Same transition rule at every marker emission

**Debug Output:**
```c
// Human-readable marker state
debugcon_printf("marker_bitmap=0b%07b last=%d count=%d\n",
                slot->marker_bitmap,
                slot->last_marker,
                slot->marker_count);
```

---

## 10. Performance Model

### STRICT Mode (CI/Debug)

```c
#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
    // Incremental validation (per-marker)
    // Verified prefix validation (pre-commit)
    // Full sequence validation (wait path)
    // Evidence generation on failure
#endif
```

**Overhead (Per Execution):**

**Phase 1 (Incremental - 7 markers):**
- Transition check: ~5 cycles per marker
- Duplicate check: ~3 cycles per marker (bitmap test)
- Bitmap update: ~2 cycles per marker
- **Subtotal**: ~70 cycles (7 markers × 10 cycles)

**Phase 2 (Verified Prefix - once):**
- Prefix bitmap check: ~4 cycles
- Count + last check: ~4 cycles
- **Subtotal**: ~8 cycles

**Phase 3 (Result Boundary - once):**
- Prefix guard: ~8 cycles
- RESULT_OK emission: ~10 cycles
- **Subtotal**: ~18 cycles

**Phase 4 (Full Sequence - once):**
- Result prefix guard: ~8 cycles
- WAIT_OK emission: ~10 cycles
- Full bitmap/count check: ~6 cycles
- **Subtotal**: ~24 cycles

**Total Overhead**: ~120 cycles per execution

**Comparison to Array Design:**
- Array design: higher and scales with marker array scans
- Bitmap design: O(1) checks at each boundary
- **Improvement**: deterministic fixed-cost validation

**Acceptable for:** CI, debug builds, validation runs

---

### RELAXED Mode (Production - Future)

```c
#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE && !AYKEN_STRICT_VALIDATION
    // Capture disabled OR lightweight sanity checks only
    // No dynamic allocation
    // No logging
    // No system time
#endif
```

**Overhead:** ~0 cycles (compile-time removed or minimal)

**Rule:**
> Phase-17 uses **STRICT mode only**. RELAXED mode is Phase-18+.

---

### Memory Overhead

**Per Slot:**
- Flag OFF: 0 bytes
- Flag ON: 4 bytes (bitmap + last + count + padding)

**Total (64 slots):**
- Flag OFF: 0 bytes
- Flag ON: 256 bytes

**Cache Impact:** Negligible (4 bytes fits in single cache line)

---

## 11. Integration Rollout Plan (Revised)

### Step 1: Design PR (Current)

**Deliverable:** This document only

**No code changes**

**Merge Criteria:**
- Design review approved
- Integration points validated (incremental + prefix + result boundary + wait path)
- Failure semantics agreed (no `finish_locked()` recursion)
- Data structure approved (bitmap design)

---

### Step 2: Feature Flag + Data Structure

**Changes:**
- Add `AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE` to `Makefile`
- Add flag to `kernel/include/execution_slot.h`
- Add bitmap fields to `exec_slot_t` (guarded by flag)
- Default: `0` (OFF)

**Merge Criteria:**
- Flag OFF: kernel build PASS
- Flag OFF: all tests PASS
- Flag OFF: ci-freeze PASS
- No runtime behavior change
- Struct size unchanged (flag OFF)

---

### Step 3: Marker Emission Helper (Incremental Validation)

**Changes:**
- Add `execution_slot_emit_marker_locked()` helper
- Implements Phase 1 validation (transition + duplicate check)
- Unit test for marker emission

**Merge Criteria:**
- Flag OFF: no behavior change
- Flag ON: marker emission works
- Flag ON: duplicate detection works
- Flag ON: transition validation works
- No integration into production code yet

---

### Step 4: Marker Emission Integration

**Changes:**
- Add `execution_slot_emit_marker_locked()` calls to production code
- Guarded by `#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE`
- All 7 marker emission points

**Merge Criteria:**
- Flag OFF: regression test PASS (no behavior change)
- Flag ON: markers captured correctly
- Flag ON: incremental validation works
- QEMU/debugcon evidence shows marker sequence

---

### Step 5: Verified Prefix Validation (Pre-Commit)

**Changes:**
- Add Phase 2 prefix validation to `execution_slot_prepare_hash_locked()`
- Validate markers through `MARKER_VERIFY_PASS`
- Fail-closed on validation failure (direct state set)
- Evidence generation on failure

**Merge Criteria:**
- Flag OFF: regression test PASS
- Flag ON + valid verified prefix: execution proceeds to mapping
- Flag ON + invalid verified prefix: execution fails before mapping (EXEC_SLOT_FAILED)
- Flag ON: no `finish_locked()` recursion
- Evidence generated on failure

---

### Step 6: Result Boundary Guard

**Changes:**
- Add pre-publish prefix guard to `execution_slot_record_result_mapping_locked()`
- Emit `MARKER_RESULT_OK` after mapping succeeds
- Panic only if post-mapping marker emission violates the preflight invariant

**Merge Criteria:**
- Flag OFF: regression test PASS
- Flag ON: invalid prefix cannot be mapped
- Flag ON: `RESULT_OK` emitted after successful mapping
- Flag ON: panic path is unreachable in valid execution

---

### Step 7: Wait Path Full Sequence Validation

**Changes:**
- Emit `MARKER_WAIT_OK` in the userspace `wait_result` syscall path
- Validate full 7-marker sequence after `WAIT_OK`
- Fail before successful wait return on invalid final sequence

**Merge Criteria:**
- Flag OFF: regression test PASS
- Flag ON + valid sequence: wait returns success
- Flag ON + invalid final sequence: wait returns failure and slot is terminal
- Evidence generated on failure

---

### Step 8: CI Gate Addition

**Changes:**
- Add `ci-gate-execution-marker-runtime` gate
- Validate marker order in kernel output
- Detect validation failures

**Merge Criteria:**
- Gate PASS with flag ON
- Gate detects marker order violations
- Evidence format validated

---

## 12. Required Gates

### Existing Gates (Must Continue to PASS)

- ✅ `ci-gate-execution-slot-integrity` - Production code protection
- ✅ `ci-gate-execution-marker-isolation` - Sandbox isolation
- ✅ `ci-freeze` - Full freeze validation

### New Gate (Step 8)

**Gate:** `ci-gate-execution-marker-runtime`

**Purpose:** Validate marker order in kernel execution output

**Checks:**
1. Marker sequence present in debugcon output
2. Marker order matches canonical sequence
3. No missing markers
4. No duplicate markers
5. No out-of-bounds markers

**Evidence:**
```
out/evidence/run-<RUN_ID>/gates/execution-marker-runtime/
├── report.json
├── marker_sequence.txt
└── violations.txt (if any)
```

---

## 13. Merge Criteria

### Every PR Must Satisfy

**Flag OFF (Default):**
- ✅ Kernel build PASS
- ✅ Existing boot/runtime behavior unchanged
- ✅ ci-freeze PASS
- ✅ No execution_slot.c refactoring
- ✅ No deletion-heavy diffs

**Flag ON (Validation):**
- ✅ Marker capture works
- ✅ Validation logic correct
- ✅ Fail-closed on invalid sequence
- ✅ Evidence generated

**Gates:**
- ✅ execution-slot-integrity PASS
- ✅ execution-marker-isolation PASS
- ✅ Remote CI PASS

---

## 14. Risk Mitigation (Revised)

### Risk 1: Accidental Production Code Overwrite

**Mitigation:**
- `ci-gate-execution-slot-integrity` enforced
- Line count minimum (1500+ lines)
- Critical marker presence check
- Prototype indicator detection

**Incident Reference:** Commit b3e2aee7 (1910 lines overwritten)

**Status:** ✅ Mitigated

---

### Risk 2: State Machine Corruption

**Original Risk:** `finish_locked()` recursion causing double-finish

**Mitigation:**
- **NO `finish_locked()` calls in validation failure path**
- Direct state assignment: `slot->state = EXEC_SLOT_FAILED`
- Terminal flag set: `slot->flags |= EXEC_SLOT_FLAG_TERMINAL`
- Clear failure semantics documented

**Status:** ✅ Mitigated (design corrected)

---

### Risk 3: Half-Committed State

**Original Risk:** Validation after mapping allows partial publish

**Mitigation:**
- **Pre-commit prefix validation moved to `prepare_hash_locked()`**
- Markers through `VERIFY_PASS` are validated BEFORE result mapping
- Rollback possible on prefix failure (hash backing can be released)
- Result boundary guard prevents invalid prefix state from being mapped
- Full sequence validation runs after `WAIT_OK`, before successful wait return

**Status:** ✅ Mitigated (design corrected)

---

### Risk 4: Determinism Violation

**Mitigation:**
- No dynamic allocation in validation
- No I/O in validation
- No system time in validation
- Pure function validation logic
- Bitmap design: O(1) deterministic operations

**Status:** ✅ Mitigated

---

### Risk 5: Performance Regression

**Mitigation:**
- Flag OFF: zero overhead (compile-time removed)
- Flag ON: STRICT mode only (CI/debug)
- Bitmap design: O(1) boundary checks
- RELAXED mode deferred to Phase-18

**Status:** ✅ Mitigated

---

### Risk 6: Post-Mortem Validation (Fail-Slow)

**Original Risk:** Full sequence validation only at end (errors detected late)

**Mitigation:**
- **Incremental validation (Phase 1)**: Fail-fast per-marker
- Transition validation at emission time
- Duplicate detection immediate
- Pre-commit prefix validation before result mapping
- Full sequence validation at the first valid point after `WAIT_OK`

**Status:** ✅ Mitigated (design corrected)

---

### Risk 7: Concurrency / Race Conditions

**Mitigation:**
- All validation under `execution_slot_guard_t` (interrupts disabled)
- Bitmap operations atomic (single byte)
- No shared state between slots
- Per-slot validation state

**Status:** ✅ Mitigated

---

## 15. Evidence Requirements

### Per Execution (Flag ON)

```
evidence/run-<RUN_ID>/
├── marker_sequence.txt       (captured markers)
├── marker_validation.json    (validation result)
└── execution_trace.txt       (state transitions)
```

**Format:**
```json
{
  "execution_id": 123,
  "generation": 1,
  "marker_count": 7,
  "markers": [
    "EXEC_START",
    "EXEC_OUTPUT_WRITTEN",
    "EXEC_COMPLETE_OK",
    "VERIFY_START",
    "VERIFY_PASS",
    "RESULT_OK",
    "WAIT_OK"
  ],
  "validation_result": "MARKER_VALIDATION_OK"
}
```

---

## 16. Testing Strategy

### Unit Tests (Userspace)

**Location:** `tests/unit/execution_marker_validation_test.c`

**Status:** ✅ 13/13 PASS (already merged)

**Coverage:**
- Valid full sequence
- Valid partial sequence
- Missing markers
- Gap in sequence
- Duplicate markers
- Out-of-bounds markers
- Marker name validation

---

### Integration Tests (Kernel)

**Location:** `tests/integration/execution_marker_integration_test.c` (future)

**Coverage:**
- Flag OFF: no behavior change
- Flag ON + valid sequence: execution succeeds
- Flag ON + invalid sequence: execution fails
- Marker emission at correct points
- Evidence generation

---

### Regression Tests

**Purpose:** Ensure flag OFF preserves existing behavior

**Tests:**
- Kernel boot
- BCIB execution (stub)
- Result buffer publish
- Userspace wait_result
- Slot lifecycle

**Criteria:** Bit-identical behavior with flag OFF

---

## 17. Documentation Updates

### Required Updates

1. **IMPLEMENTATION_RULES.md**
   - Add marker validation rules
   - Add feature flag semantics
   - Add failure semantics

2. **GATE_VALIDATION_SCOPE.md**
   - Add `ci-gate-execution-marker-runtime` scope
   - Update gate dependency graph

3. **ARCHITECTURE_FREEZE.md**
   - Add Phase-17 marker validation milestone
   - Update integration status

---

## 18. Open Questions

### Q1: Should marker validation be synchronous or asynchronous?

**Answer:** Synchronous (Phase-17). Asynchronous validation is Phase-18+.

**Rationale:** Fail-closed requires immediate feedback.

---

### Q2: Should validation failure trigger panic or graceful failure?

**Answer:** Graceful failure (`EXEC_SLOT_FAILED`). Panic only for invariant violations.

**Rationale:** Validation failure is a runtime error, not a kernel bug.

---

### Q3: Should marker capture be always-on or feature-flag gated?

**Answer:** Feature-flag gated. Default OFF.

**Rationale:** Zero overhead for production until proven.

---

## 19. Success Criteria (Revised)

### Phase-17 Integration Complete When:

✅ Feature flag defined (default OFF)  
✅ Bitmap data structure implemented (4 bytes per slot)  
✅ Incremental validation implemented (Phase 1: fail-fast)  
✅ Marker emission integrated (guarded, per-marker validation)  
✅ Verified prefix validation implemented (Phase 2: pre-commit)
✅ Result boundary guard implemented (Phase 3: publish boundary)
✅ Full sequence validation implemented (Phase 4: wait path)
✅ CI gate added (`ci-gate-execution-marker-runtime`)  
✅ Flag OFF: regression tests PASS  
✅ Flag ON: incremental validation PASS  
✅ Flag ON: pre-commit prefix validation PASS
✅ Flag ON: full sequence validation PASS  
✅ Evidence generated on failure  
✅ NO `finish_locked()` recursion  
✅ Remote CI PASS  

---

## 20. Final Rule (Revised)

**Integration Principle:**
> Add, don't replace. Guard, don't assume. Fail-fast, then fail-closed.

**Validation Principle:**
> Incremental first (fail-fast). Verified prefix before mapping. Full sequence after `WAIT_OK`. Panic only for impossible invariants.

**Failure Principle:**
> Direct state assignment. No recursion. Terminal immediately.

**Merge Principle:**
> Flag OFF = no change. Flag ON = proven correct at each phase.

**Evidence Principle:**
> Capture everything. Validate deterministically. Fail loudly.

**Design Principle:**
> Bitmap over array. O(1) over O(n). Validate each boundary at the first point where its markers can exist.

---

## 21. Design Revision Summary

### Critical Corrections Made

**1. Validation Strategy:**
- ❌ **Old**: Single post-commit validation
- ✅ **New**: Staged validation (incremental + prefix + result boundary + wait full sequence)

**2. Primary Call Site:**
- ❌ **Old**: `execution_slot_record_result_mapping_locked()` (post-commit)
- ✅ **New**: Prefix guard at `execution_slot_prepare_hash_locked()` and full guard at `wait_result`

**3. Failure Handling:**
- ❌ **Old**: `return execution_slot_finish_locked(slot, EXEC_SLOT_FAILED)` (recursion risk)
- ✅ **New**: Direct state assignment + terminal flag (no recursion)

**4. Data Structure:**
- ❌ **Old**: Array design (9 bytes, O(n) validation)
- ✅ **New**: Bitmap design (4 bytes, O(1) validation)

**5. Validation Model:**
- ❌ **Old**: Post-mortem (fail-slow)
- ✅ **New**: Incremental (fail-fast) + boundary guards (fail-closed)

### Minor Revisions (v2.1)

**6. Marker Mask Future-Proofing:**
- ❌ **Old**: `if (bitmap != 0b01111111)` (hardcoded)
- ✅ **New**: `if (bitmap != MARKER_MASK)` (computed from MARKER_COUNT)

**7. Incremental Validation Ordering:**
- ❌ **Old**: Monotonic (`marker > last_marker`, allows gaps)
- ✅ **New**: Strict sequential (`marker == last_marker + 1`)

**8. Error Code Field:**
- ❌ **Old**: `reserved_marker` (unused padding)
- ✅ **New**: `marker_error_code` (validation failure reason)

**9. Return Value Enforcement:**
- ❌ **Old**: Implicit requirement
- ✅ **New**: Explicit mandatory rule with enforcement strategy

**10. Validation Ordering Clarification:**
- ❌ **Old**: Monotonic Phase 1, exact Phase 2
- ✅ **New**: Strict sequential for every emitted transition (header contract alignment)

**Rationale:** `execution_marker_validate_transition()` enforces strict sequential. Monotonic ordering deferred to Phase-18 (requires API change).

### Timing Corrections (v2.1.2)

**11. Full Sequence Validation Timing:**
- ❌ **Old**: Full 7-marker validation in `execution_slot_prepare_hash_locked()`
- ✅ **New**: Prefix validation in `execution_slot_prepare_hash_locked()`, full validation after `WAIT_OK`

**12. Active Error Codes:**
- ❌ **Old**: `MARKER_VALIDATION_NON_MONOTONIC` listed as active despite absent header contract
- ✅ **New**: Active failure codes match `execution_marker_validation.h`

**Rationale:** `RESULT_OK` and `WAIT_OK` cannot exist at the pre-commit hash stage. Full sequence validation must run at the first point where all markers can exist: after `WAIT_OK` in the wait path.

### Design Improvements

**Performance:**
- O(1) boundary checks
- Fixed-cost validation in STRICT mode
- 56% smaller memory footprint (4 bytes vs 9 bytes)

**Safety:**
- Fail-fast detection (per-marker)
- Pre-commit prefix validation (rollback possible)
- Full sequence validation before successful wait return
- No recursion risk (direct state assignment)
- Return value enforcement (no silent corruption)

**Future-Proofing:**
- Computed marker mask (adapts to MARKER_COUNT changes)
- Strict sequential (header contract alignment)
- Error code field (scheduler visibility, telemetry)
- Monotonic ordering (Phase-18 future extension)

**Debuggability:**
- Bitmap visualization (0b0111111)
- Clear failure point (marker emission site)
- Evidence includes bitmap state + error code
- Scheduler-visible failure reason

---

**Prepared By:** Kenan AY - Architectural Steward  
**Date:** 01 May 2026  
**Version:** 2.1.2 (Validation Timing Alignment)
**Status:** DESIGN ONLY (Implementation NOT Started)

**Revision History:**
- v1.0 (2026-05-01): Initial design
- v2.0 (2026-05-01): Critical corrections (validation strategy, call site, failure handling, data structure)
- v2.1 (2026-05-01): Minor revisions (marker mask, error code, return value enforcement)
- v2.1.1 (2026-05-01): Header contract alignment (strict sequential transitions, monotonic deferred)
- v2.1.2 (2026-05-01): Validation timing alignment (pre-commit prefix, wait-path full sequence)

**© 2026 Kenan AY - AykenOS Project**


---

## 22. Implementation Progress

### ✅ Completed Steps

#### Step 1: Design Document (PR #127)
- **Status:** ✅ MERGED (commit 99b7c80d)
- **Date:** 2026-05-01
- **Version:** 2.1.2 (Validation Timing Aligned)
- **CI:** ALL PASS (10/10 gates + freeze)
- **Deliverable:** `docs/specs/phase17-execution-pipeline/INTEGRATION_DESIGN.md`

#### Step 2: Feature Flag + Bitmap Structure (PR #128)
- **Status:** ✅ MERGED (commit 8cc00e5f)
- **Date:** 2026-05-01
- **Changes:**
  - Added `AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE` flag (default: 0)
  - Added bitmap fields to `exec_slot_t` (4 bytes, guarded)
  - Added flag validation (0 or 1 only)
  - Added flag to build stamp
- **Verification:**
  - Flag OFF: no behavior change, struct size unchanged
  - Flag ON: struct +4 bytes, fields unused
- **CI:** ALL PASS (10/10 gates + freeze)
- **Files:** `Makefile`, `kernel/include/execution_slot.h`

---

### 🔄 In Progress

#### Step 3: Marker Capture Helper
- **Status:** 🔄 IN PROGRESS
- **Branch:** `phase17-marker-capture-helper`
- **Objective:** Add write-only capture helper (NO validation, NO call sites)
- **Scope:**
  - Add `execution_slot_marker_capture_locked()` to `kernel/sys/execution_slot.c`
  - Pure write: `marker_bitmap`, `last_marker`, `marker_count`
  - **NO validation logic**
  - **NO error handling**
  - **NO return value**
  - **NO call sites** (Step 4)
- **Implementation:**
  ```c
  #if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
  static inline void execution_slot_marker_capture_locked(
      exec_slot_t *slot,
      execution_marker_t marker)
  {
      // Pure write - no validation, no checks, no failure
      slot->marker_bitmap |= (uint8_t)(1u << marker);
      slot->last_marker = (uint8_t)marker;
      slot->marker_count++;
  }
  #endif
  ```
- **Expected:**
  - Flag OFF: helper not compiled
  - Flag ON: helper compiled but unused
  - No behavior change (helper not called)
  - CI: ALL PASS

---

### 🔜 Upcoming Steps

#### Step 4: Marker Emission Integration
- **Objective:** Add call sites to production code
- **Scope:**
  - Add guarded calls to 7 emission points
  - **Return value checks mandatory** (all call sites)
  - Evidence generation on failure
- **Call Sites:**
  1. `execution_slot_pickup_locked()` → EXEC_START
  2. `execution_slot_write_output_v()` → EXEC_OUTPUT_WRITTEN
  3. `execution_slot_finish_locked()` → EXEC_COMPLETE_OK
  4. `execution_slot_prepare_result_locked()` → VERIFY_START
  5. `execution_slot_prepare_hash_locked()` → VERIFY_PASS
  6. `execution_slot_record_result_mapping_locked()` → RESULT_OK
  7. Userspace `wait_result` syscall → WAIT_OK
- **Verification:**
  - Flag OFF: regression test PASS
  - Flag ON: markers captured correctly
  - QEMU/debugcon evidence shows marker sequence

#### Step 5: Full Sequence Validation (Pre-Commit)
- **Objective:** Add validation to `prepare_hash_locked()`
- **Scope:**
  - Prefix validation (markers 0-4 present)
  - Fail-closed on invalid sequence
  - Direct state set (no `finish_locked()` recursion)
  - Error code recorded
- **Verification:**
  - Flag OFF: regression test PASS
  - Flag ON + valid: execution succeeds
  - Flag ON + invalid: execution fails (EXEC_SLOT_FAILED)

#### Step 6: Sanity Check (Post-Commit)
- **Objective:** Add invariant check to `record_result_mapping_locked()`
- **Scope:**
  - Check marker count == MARKER_COUNT
  - Panic on violation (should never happen)
- **Verification:**
  - Flag OFF: regression test PASS
  - Flag ON: sanity check never triggers

#### Step 7: CI Gate
- **Objective:** Add `ci-gate-execution-marker-runtime`
- **Scope:**
  - Validate marker order in kernel output
  - Detect validation failures
  - Evidence format validation
- **Verification:**
  - Gate PASS with flag ON
  - Gate detects marker order violations

---

### 📊 Progress Summary

| Step | Status | PR | Commit | Date |
|------|--------|-----|--------|------|
| 1. Design | ✅ MERGED | #127 | 99b7c80d | 2026-05-01 |
| 2. Flag + Bitmap | ✅ MERGED | #128 | 8cc00e5f | 2026-05-01 |
| 3. Capture Helper | 🔄 IN PROGRESS | TBD | TBD | TBD |
| 4. Emission Integration | 🔜 TODO | TBD | TBD | TBD |
| 5. Validation (Pre-Commit) | 🔜 TODO | TBD | TBD | TBD |
| 6. Sanity Check | 🔜 TODO | TBD | TBD | TBD |
| 7. CI Gate | 🔜 TODO | TBD | TBD | TBD |

---

**Implementation Status:** Step 3 IN PROGRESS  
**Last Updated:** 2026-05-01  
**Next Milestone:** Marker capture helper (write-only, no validation)

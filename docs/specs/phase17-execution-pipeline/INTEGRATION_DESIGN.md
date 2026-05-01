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

## 7. Validation Call Site

### Primary Validation Point

**Location:** `execution_slot_record_result_mapping_locked()`

**Timing:** After result mapping, before returning success to userspace

**Rationale:**
1. Full marker sequence is complete
2. No partial execution validation (avoids false failures)
3. Userspace publish boundary is clear
4. Verification already passed (VERIFY_PASS marker present)

### Validation Logic (Pseudocode)

```c
int execution_slot_record_result_mapping_locked(exec_slot_t *slot,
                                                uint64_t mapped_result_va,
                                                uint64_t mapped_hash_va,
                                                uint64_t map_flags)
{
    // ... existing validation ...

#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
    // Validate marker sequence
    marker_validation_result_t validation_result;
    validation_result = execution_marker_validate(
        slot->markers,
        slot->marker_count
    );

    if (validation_result != MARKER_VALIDATION_OK) {
        // Fail-closed: do NOT publish result
        execution_slot_emit_marker_validation_failure(slot, validation_result);
        return execution_slot_finish_locked(slot, EXEC_SLOT_FAILED);
    }
#endif

    // ... existing mapping logic ...
    return 0;
}
```

### Secondary Validation Point (Future)

**Location:** `execution_slot_prepare_result_locked()` (before VERIFY_PASS)

**Purpose:** Early detection of marker order violations

**Timing:** Phase-17.1 (after primary validation proven)

---

## 8. Failure Semantics

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

**Behavior:**
1. Result buffer **NOT published** to userspace
2. Slot transitioned to `EXEC_SLOT_FAILED`
3. Evidence marker written to debugcon
4. Scheduler does **NOT** retry (terminal failure)

**Failure Codes:**

| Validation Result | Slot State | Evidence Marker |
|-------------------|------------|-----------------|
| `MARKER_VALIDATION_INVALID_ORDER` | `EXEC_SLOT_FAILED` | `[[MARKER_VALIDATION_FAIL]] reason=INVALID_ORDER` |
| `MARKER_VALIDATION_MISSING` | `EXEC_SLOT_FAILED` | `[[MARKER_VALIDATION_FAIL]] reason=MISSING` |
| `MARKER_VALIDATION_DUPLICATE` | `EXEC_SLOT_FAILED` | `[[MARKER_VALIDATION_FAIL]] reason=DUPLICATE` |
| `MARKER_VALIDATION_OUT_OF_BOUNDS` | `EXEC_SLOT_FAILED` | `[[MARKER_VALIDATION_FAIL]] reason=OUT_OF_BOUNDS` |

**Evidence Format:**
```
[[MARKER_VALIDATION_FAIL]] exec_id=<ID> generation=<GEN> reason=<REASON> expected=<EXPECTED> actual=<ACTUAL>
```

**Rule:**
> Validation failure = **permanent failure** (no retry, no recovery)

---

## 9. Data Structure

### Marker Capture Buffer

**Location:** Inside `exec_slot_t` structure

**Proposed Addition:**
```c
// kernel/include/execution_slot.h
typedef struct exec_slot {
    // ... existing fields ...

#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
    execution_marker_t markers[MARKER_COUNT];
    uint8_t marker_count;
    uint8_t marker_validation_enabled;
    uint8_t reserved_marker[2];
#endif
} exec_slot_t;
```

**Rules:**
- Fixed-size array (no dynamic allocation)
- Bounded by `MARKER_COUNT` (7 markers)
- Zero-initialized on slot allocation
- Cleared on slot release

**Memory Impact:**
- Flag OFF: 0 bytes (compile-time removed)
- Flag ON: 9 bytes per slot (7 markers + 1 count + 2 reserved)

---

## 10. Performance Model

### STRICT Mode (CI/Debug)

```c
#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
    // Full marker capture
    // Full sequence validation
    // Evidence generation on failure
#endif
```

**Overhead:**
- Marker emission: ~10 cycles per marker (7 markers = 70 cycles)
- Validation: ~50 cycles (linear scan)
- Total: ~120 cycles per execution

**Acceptable for:** CI, debug builds, validation runs

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

## 11. Integration Rollout Plan

### Step 1: Design PR (Current)

**Deliverable:** This document only

**No code changes**

**Merge Criteria:**
- Design review approved
- Integration points validated
- Failure semantics agreed

---

### Step 2: Feature Flag Definition

**Changes:**
- Add `AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE` to `Makefile`
- Add flag to `kernel/include/execution_slot.h`
- Default: `0` (OFF)

**Merge Criteria:**
- Flag OFF: kernel build PASS
- Flag OFF: all tests PASS
- Flag OFF: ci-freeze PASS
- No runtime behavior change

---

### Step 3: Marker Capture Helper

**Changes:**
- Add `execution_slot_emit_marker_locked()` helper
- Add marker buffer to `exec_slot_t` (guarded by flag)
- Unit test for marker capture

**Merge Criteria:**
- Flag OFF: no behavior change
- Flag ON: marker capture works
- No validation logic yet (capture only)

---

### Step 4: Marker Emission Integration

**Changes:**
- Add `execution_slot_emit_marker_locked()` calls to production code
- Guarded by `#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE`

**Merge Criteria:**
- Flag OFF: regression test PASS (no behavior change)
- Flag ON: markers captured correctly
- QEMU/debugcon evidence shows marker sequence

---

### Step 5: Validation Integration

**Changes:**
- Add `execution_marker_validate()` call to `execution_slot_record_result_mapping_locked()`
- Fail-closed on validation failure

**Merge Criteria:**
- Flag OFF: regression test PASS
- Flag ON + valid sequence: execution succeeds
- Flag ON + invalid sequence: execution fails (EXEC_SLOT_FAILED)
- Evidence generated on failure

---

### Step 6: CI Gate Addition

**Changes:**
- Add `ci-gate-execution-marker-runtime` gate
- Validate marker order in kernel output

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

### New Gate (Step 6)

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

## 14. Risk Mitigation

### Risk 1: Accidental Production Code Overwrite

**Mitigation:**
- `ci-gate-execution-slot-integrity` enforced
- Line count minimum (1500+ lines)
- Critical marker presence check
- Prototype indicator detection

**Incident Reference:** Commit b3e2aee7 (1910 lines overwritten)

---

### Risk 2: State Machine Corruption

**Mitigation:**
- No state machine changes
- Marker emission AFTER operation success
- Validation AFTER full sequence complete
- Fail-closed on validation failure

---

### Risk 3: Determinism Violation

**Mitigation:**
- No dynamic allocation in validation
- No I/O in validation
- No system time in validation
- Pure function validation logic

---

### Risk 4: Performance Regression

**Mitigation:**
- Flag OFF: zero overhead (compile-time removed)
- Flag ON: STRICT mode only (CI/debug)
- RELAXED mode deferred to Phase-18

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

## 19. Success Criteria

### Phase-17 Integration Complete When:

✅ Feature flag defined (default OFF)  
✅ Marker capture implemented (guarded)  
✅ Marker emission integrated (guarded)  
✅ Validation integrated (fail-closed)  
✅ CI gate added (`ci-gate-execution-marker-runtime`)  
✅ Flag OFF: regression tests PASS  
✅ Flag ON: marker validation PASS  
✅ Evidence generated on failure  
✅ Remote CI PASS  

---

## 20. Final Rule

**Integration Principle:**
> Add, don't replace. Guard, don't assume. Fail-closed, don't ignore.

**Merge Principle:**
> Flag OFF = no change. Flag ON = proven correct.

**Evidence Principle:**
> Capture everything. Validate deterministically. Fail loudly.

---

**Prepared By:** Kenan AY - Architectural Steward  
**Date:** 01 May 2026  
**Version:** 1.0  
**Status:** DESIGN ONLY (Implementation NOT Started)

**© 2026 Kenan AY - AykenOS Project**

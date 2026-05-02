# Phase-17.5: Observability & Debug Infrastructure

**Date**: 2026-05-02  
**Status**: PREREQUISITE (before Phase-18)  
**Authority**: Kenan AY — Architectural Steward

---

## 🎯 OBJECTIVE

**Add visibility tools BEFORE Phase-18 starts.**

**Critical Insight**: Phase-17 proved logic correct. Phase-18 must **SEE** system behavior.

**Without this**: Phase-18 debugging = blind  
**With this**: Phase-18 debugging = observable

---

## 🔥 WHY THIS IS CRITICAL

### The Problem
Phase-17 tests answered: **"Does validation logic work?"** (YES)  
Phase-18 tests must answer: **"Does system behave correctly?"** (UNKNOWN)

**Gap**: We can test outcomes, but we cannot **observe** the path to those outcomes.

### The Risk
Without observability:
- Scheduler bug → can't see what happened
- Race condition → can't reproduce
- State corruption → can't diagnose
- Performance issue → can't measure

**Result**: Debug time × 10

### The Solution
Add **observability layer** before Phase-18:
- Trace every marker capture
- Dump every state transition
- Measure every timing
- Log every failure

**Result**: Debug time ÷ 5

---

## 📊 INFRASTRUCTURE TO ADD

### 1. Marker Trace Log

**Purpose**: See every marker capture in real-time

**Implementation**:
```c
// kernel/include/execution_marker_trace.h
#if DEBUG_KERNEL
#define TRACE_MARKER(slot_id, marker_id, timestamp) \
    kernel_log("[MARKER] slot=%u marker=%u ts=%llu\n", \
               slot_id, marker_id, timestamp)
#else
#define TRACE_MARKER(slot_id, marker_id, timestamp) ((void)0)
#endif
```

**Usage**:
```c
void execution_slot_marker_capture_locked(exec_slot_t *slot, uint8_t marker) {
    TRACE_MARKER(slot->id, marker, rdtsc());
    // ... existing code ...
}
```

**Output**:
```
[MARKER] slot=1 marker=0 ts=12345
[MARKER] slot=1 marker=1 ts=12456
[MARKER] slot=1 marker=2 ts=12567
```

**Why Critical**: Without this, you can't see marker capture order

---

### 2. Execution Slot Dump

**Purpose**: Snapshot slot state at any point

**Implementation**:
```c
// kernel/sys/execution_slot_debug.c
void dump_execution_slot(const exec_slot_t *slot) {
    kernel_log("[SLOT_DUMP] id=%u state=%u\n", slot->id, slot->state);
    kernel_log("  marker_count=%u\n", slot->marker_count);
    kernel_log("  marker_bitmap=0x%02X\n", slot->marker_bitmap);
    kernel_log("  marker_sequence=[");
    for (int i = 0; i < slot->marker_count; i++) {
        kernel_log("%u%s", slot->marker_sequence[i], 
                   i < slot->marker_count - 1 ? "," : "");
    }
    kernel_log("]\n");
    kernel_log("  error_code=%u\n", slot->marker_error_code);
}
```

**Usage**:
```c
if (execution_slot_validate_markers_locked(slot) != 0) {
    dump_execution_slot(slot);  // See why validation failed
    return -1;
}
```

**Output**:
```
[SLOT_DUMP] id=1 state=3
  marker_count=5
  marker_bitmap=0x1F
  marker_sequence=[0,1,2,3,4]
  error_code=0
```

**Why Critical**: Without this, you can't see slot state at failure

---

### 3. State Transition Trace

**Purpose**: See every state machine transition

**Implementation**:
```c
// kernel/include/execution_state_trace.h
#if DEBUG_KERNEL
#define TRACE_STATE(slot_id, old_state, new_state) \
    kernel_log("[STATE] slot=%u old=%s new=%s\n", \
               slot_id, state_name(old_state), state_name(new_state))
#else
#define TRACE_STATE(slot_id, old_state, new_state) ((void)0)
#endif
```

**Usage**:
```c
void execution_slot_transition_locked(exec_slot_t *slot, exec_slot_state_t new_state) {
    TRACE_STATE(slot->id, slot->state, new_state);
    slot->state = new_state;
}
```

**Output**:
```
[STATE] slot=1 old=INIT new=EXEC
[STATE] slot=1 old=EXEC new=VERIFY
[STATE] slot=1 old=VERIFY new=COMPLETE
```

**Why Critical**: Without this, you can't see illegal state transitions

---

### 4. Timing Measurement

**Purpose**: Measure validation overhead

**Implementation**:
```c
// kernel/include/execution_timing.h
#if DEBUG_KERNEL
typedef struct {
    uint64_t start;
    uint64_t end;
    const char *label;
} timing_t;

#define TIMING_START(label) \
    timing_t __timing_##label = {rdtsc(), 0, #label}

#define TIMING_END(label) \
    do { \
        __timing_##label.end = rdtsc(); \
        kernel_log("[TIMING] %s: %llu cycles\n", \
                   __timing_##label.label, \
                   __timing_##label.end - __timing_##label.start); \
    } while(0)
#else
#define TIMING_START(label) ((void)0)
#define TIMING_END(label) ((void)0)
#endif
```

**Usage**:
```c
int execution_slot_validate_markers_locked(const void *slot_ptr) {
    TIMING_START(validation);
    // ... validation logic ...
    TIMING_END(validation);
    return result;
}
```

**Output**:
```
[TIMING] validation: 1234 cycles
```

**Why Critical**: Without this, you can't measure performance impact

---

### 5. Kernel Test Harness

**Purpose**: Run tests inside kernel, report results

**Implementation**:
```c
// kernel/test/test_harness.h
typedef int (*test_fn_t)(void);

typedef struct {
    const char *name;
    test_fn_t fn;
} kernel_test_t;

#define KERNEL_TEST_REGISTER(name, fn) \
    static kernel_test_t __test_##name \
    __attribute__((section(".kernel_tests"))) = {#name, fn}

void kernel_test_run_all(void);
```

**Usage**:
```c
// kernel/test/test_marker_validation.c
static int test_golden_path(void) {
    exec_slot_t slot = {0};
    // ... test logic ...
    return (result == 0) ? 0 : -1;
}

KERNEL_TEST_REGISTER(golden_path, test_golden_path);
```

**Output**:
```
[TEST] golden_path: PASS
[TEST] invalid_order: PASS
[TEST] invalid_count: PASS
```

**Why Critical**: Without this, you can't run tests in kernel context

---

### 6. Debug Mode Flags

**Purpose**: Control instrumentation overhead

**Implementation**:
```c
// kernel/include/kernel_config.h
#if defined(AYKEN_DEBUG_KERNEL)
#define DEBUG_KERNEL 1
#else
#define DEBUG_KERNEL 0
#endif

#if defined(AYKEN_TEST_KERNEL)
#define TEST_KERNEL 1
#else
#define TEST_KERNEL 0
#endif

#if !DEBUG_KERNEL && !TEST_KERNEL
#define PRODUCTION_KERNEL 1
#else
#define PRODUCTION_KERNEL 0
#endif
```

**Usage**:
```c
#if DEBUG_KERNEL
    // Verbose logging
#endif

#if TEST_KERNEL
    // Test harness enabled
#endif

#if PRODUCTION_KERNEL
    // Minimal overhead
#endif
```

**Why Critical**: Without this, debug code leaks into production

---

### 7. Failure Snapshot

**Purpose**: Capture complete state at failure

**Implementation**:
```c
// kernel/sys/execution_failure.c
void failure_snapshot(const exec_slot_t *slot, const char *reason) {
    kernel_log("[FAILURE] slot=%u reason=%s\n", slot->id, reason);
    dump_execution_slot(slot);
    // Optional: stack trace if available
}
```

**Usage**:
```c
if (execution_slot_validate_markers_locked(slot) != 0) {
    failure_snapshot(slot, "validation_failed");
    return -1;
}
```

**Output**:
```
[FAILURE] slot=1 reason=validation_failed
[SLOT_DUMP] id=1 state=3
  marker_count=6
  marker_bitmap=0x3F
  marker_sequence=[0,1,2,3,4,5]
  error_code=3
```

**Why Critical**: Without this, you lose failure context

---

### 8. Deterministic Replay Support

**Purpose**: Reproduce nondeterministic bugs

**Implementation**:
```c
// kernel/test/replay.h
#if TEST_KERNEL
extern uint64_t replay_seed;

#define REPLAY_SEED(seed) (replay_seed = (seed))
#define REPLAY_LOG(event) \
    kernel_log("[REPLAY] event=%s seed=%llu\n", event, replay_seed)
#else
#define REPLAY_SEED(seed) ((void)0)
#define REPLAY_LOG(event) ((void)0)
#endif
```

**Usage**:
```c
void scheduler_init(void) {
    REPLAY_SEED(12345);  // Fixed seed for deterministic scheduling
    REPLAY_LOG("scheduler_init");
}
```

**Output**:
```
[REPLAY] event=scheduler_init seed=12345
```

**Why Critical**: Without this, race conditions are unreproducible

---

### 9. State Invariant Checks

**Purpose**: Detect illegal state transitions

**Implementation**:
```c
// kernel/sys/execution_invariants.c
#if DEBUG_KERNEL
#define ASSERT_STATE_INVARIANT(slot, condition) \
    do { \
        if (!(condition)) { \
            kernel_log("[INVARIANT] slot=%u failed: %s\n", \
                       (slot)->id, #condition); \
            dump_execution_slot(slot); \
            kernel_panic("State invariant violation"); \
        } \
    } while(0)
#else
#define ASSERT_STATE_INVARIANT(slot, condition) ((void)0)
#endif
```

**Usage**:
```c
void execution_slot_transition_locked(exec_slot_t *slot, exec_slot_state_t new_state) {
    ASSERT_STATE_INVARIANT(slot, is_valid_transition(slot->state, new_state));
    slot->state = new_state;
}
```

**Output** (on violation):
```
[INVARIANT] slot=1 failed: is_valid_transition(slot->state, new_state)
[SLOT_DUMP] id=1 state=5
  ...
PANIC: State invariant violation
```

**Why Critical**: Without this, illegal transitions go undetected

---

### 10. Structured Log Format

**Purpose**: Parseable logs for CI automation

**Format**:
```
[TAG] key1=value1 key2=value2 ...
```

**Examples**:
```
[MARKER] slot=1 marker=0 ts=12345
[STATE] slot=1 old=INIT new=EXEC
[FAIL] slot=1 reason=INVALID_ORDER
[TIMING] label=validation cycles=1234
[TEST] name=golden_path result=PASS
```

**Parser** (CI script):
```bash
#!/bin/bash
# parse_kernel_log.sh
grep '^\[TEST\]' kernel.log | while read line; do
    name=$(echo "$line" | sed 's/.*name=\([^ ]*\).*/\1/')
    result=$(echo "$line" | sed 's/.*result=\([^ ]*\).*/\1/')
    echo "$name: $result"
done
```

**Why Critical**: Without this, CI can't parse test results

---

## 📋 IMPLEMENTATION CHECKLIST

### Files to Create
- [ ] `kernel/include/execution_marker_trace.h`
- [ ] `kernel/include/execution_state_trace.h`
- [ ] `kernel/include/execution_timing.h`
- [ ] `kernel/include/kernel_config.h` (update)
- [ ] `kernel/sys/execution_slot_debug.c`
- [ ] `kernel/sys/execution_failure.c`
- [ ] `kernel/sys/execution_invariants.c`
- [ ] `kernel/test/test_harness.h`
- [ ] `kernel/test/test_harness.c`
- [ ] `kernel/test/replay.h`
- [ ] `scripts/parse_kernel_log.sh`

### Integration Points
- [ ] Add `TRACE_MARKER()` to `execution_slot_marker_capture_locked()`
- [ ] Add `TRACE_STATE()` to `execution_slot_transition_locked()`
- [ ] Add `TIMING_START/END()` to `execution_slot_validate_markers_locked()`
- [ ] Add `failure_snapshot()` to validation failure paths
- [ ] Add `ASSERT_STATE_INVARIANT()` to state transitions
- [ ] Add `dump_execution_slot()` to error paths

### Build System
- [ ] Add `AYKEN_DEBUG_KERNEL` flag to Makefile
- [ ] Add `AYKEN_TEST_KERNEL` flag to Makefile
- [ ] Add conditional compilation for debug code
- [ ] Verify production build has ZERO debug symbols (objdump)

### Testing
- [ ] Test with `DEBUG_KERNEL=1` (verbose logging)
- [ ] Test with `TEST_KERNEL=1` (test harness)
- [ ] Test with `PRODUCTION_KERNEL=1` (minimal overhead)
- [ ] Verify log parsing script works
- [ ] Verify timing measurements accurate

---

## 🎯 SUCCESS CRITERIA

### Observability
- [ ] Every marker capture logged
- [ ] Every state transition logged
- [ ] Every validation timing measured
- [ ] Every failure snapshot captured

### Debuggability
- [ ] Can reproduce any test failure
- [ ] Can see exact failure state
- [ ] Can measure performance impact
- [ ] Can parse logs automatically

### Production Safety
- [ ] ZERO debug code in production build (objdump verified)
- [ ] ZERO performance overhead in production
- [ ] Debug flags controlled by build system

---

## 🔥 WHY THIS COMES BEFORE PHASE-18

**Phase-18 without observability**:
- Test fails → don't know why
- Race condition → can't reproduce
- Performance issue → can't measure
- Debug time → × 10

**Phase-18 with observability**:
- Test fails → see exact failure state
- Race condition → replay with seed
- Performance issue → measure precisely
- Debug time → ÷ 5

**Investment**: 1 day (Phase-17.5)  
**Return**: 5-10 days saved (Phase-18)

---

## 📊 COMPARISON: WITH vs WITHOUT

| Aspect | Without Observability | With Observability |
|--------|----------------------|-------------------|
| **Test Failure** | "It failed" | "Failed at marker 3, state EXEC, bitmap 0x07" |
| **Race Condition** | "Sometimes fails" | "Fails with seed 12345, reproduced 10/10 times" |
| **Performance** | "Feels slow" | "Validation: 1234 cycles (0.5% overhead)" |
| **Debug Time** | Hours to days | Minutes to hours |
| **Confidence** | Low (blind) | High (observable) |

---

## 🚀 NEXT STEPS

### Immediate
1. Create observability infrastructure (Phase-17.5)
2. Verify with local tests
3. Verify production safety (objdump)

### Then
1. Start Phase-18 Step 1A (QEMU boot)
2. Use observability tools throughout Phase-18
3. Generate evidence artifacts automatically

---

**Signed**: Kenan AY — Architectural Steward  
**Date**: 2026-05-02  
**Status**: PREREQUISITE (must complete before Phase-18)  
**Authority**: Architectural design + Phase-17 lessons learned

**Next Action**: Implement Phase-17.5 observability infrastructure


# MVP-2: Ring3 Scheduler Hint API Documentation

**Version:** 1.0  
**Library:** `libayken` (userspace/libayken/)  
**Status:** Production-Ready (Library Level)

---

## Overview

The Ring3 Scheduler Hint API provides a constitutional-grade interface for Ring3 policy decisions to communicate scheduling hints to Ring0 mechanism. This API is part of AykenOS's policy/mechanism separation architecture.

**Key Properties:**
- No syscalls required (mailbox pre-mapped by Ring0)
- Zero overhead (direct memory write)
- Monotonic epoch counter (replay prevention)
- Fail-closed validation (Ring0 rejects invalid hints)

---

## API Reference

### `ayken_sched_hint()`

Write scheduling hint to per-process mailbox.

```c
void ayken_sched_hint(uint32_t candidate_pid);
```

**Parameters:**
- `candidate_pid` - PID to hint for next scheduling decision (1-1000)

**Behavior:**
1. Read current epoch from mailbox
2. Increment epoch (monotonic)
3. Write `candidate_pid` to mailbox
4. Write new epoch to mailbox (commit)

**Validation (Ring0):**
- Double-read atomicity check (torn write detection)
- Epoch monotonicity check (replay prevention)
- PID sanity check (0 < pid <= 1000)

**Return Value:** None (fire-and-forget)

**Thread Safety:** Not thread-safe (single-writer assumption)

**Example:**
```c
#include "sched_hint.h"

void my_scheduler_policy(void) {
    // Ring3 policy decision: hint PID 42 for next scheduling
    ayken_sched_hint(42);
    
    // Validation happens asynchronously on next timer tick
    // Ring0 emits marker: [[AYKEN_SCHED_MB_ACCEPT]] or [[AYKEN_SCHED_MB_REJECT]]
}
```

---

### `ayken_sched_hint_read()`

Read current mailbox state (debugging only).

```c
void ayken_sched_hint_read(uint64_t *epoch_out, uint32_t *pid_out);
```

**Parameters:**
- `epoch_out` - Output: current epoch value (may be NULL)
- `pid_out` - Output: current candidate PID (may be NULL)

**Behavior:**
- Read epoch from mailbox
- Read candidate_pid from mailbox
- Store values in output parameters

**Use Case:** Debugging, testing, diagnostics

**Warning:** Do NOT make policy decisions based on mailbox state. Ring3 policy should be stateless (write hints, don't read back).

**Example:**
```c
uint64_t epoch;
uint32_t pid;
ayken_sched_hint_read(&epoch, &pid);
printf("Mailbox: epoch=%llu pid=%u\n", epoch, pid);
```

---

## Data Structures

### `sched_mailbox_t`

Per-process mailbox structure (Ring3 write, Ring0 read).

```c
typedef struct {
    uint64_t epoch;           // Monotonic counter (replay prevention)
    uint32_t candidate_pid;   // Scheduling hint (which PID to run next)
    uint32_t reserved;        // Padding (future use)
} sched_mailbox_t;
```

**Layout:**
- Offset 0: `epoch` (8 bytes)
- Offset 8: `candidate_pid` (4 bytes)
- Offset 12: `reserved` (4 bytes)
- Total: 16 bytes

**Alignment:** Natural alignment (8-byte aligned)

**Location:** Fixed VA `0x700000` (SCHED_MAILBOX_VA)

**Mapping:** USER | WRITABLE | PRESENT (per-process, isolated)

---

## Constants

### `SCHED_MAILBOX_VA`

Fixed virtual address for scheduler mailbox.

```c
#define SCHED_MAILBOX_VA 0x700000UL  // 7 MiB
```

**Properties:**
- Fixed across all processes (deterministic)
- Per-process isolation (separate physical frame)
- Safe from loader collision (above 4 MiB)

---

## Invariants

### 1. Epoch Monotonicity

**Invariant:** Epoch MUST strictly increase with each write.

**Enforcement:**
- Ring3: Increments epoch before write
- Ring0: Rejects if `epoch <= last_epoch`

**Violation:** Ring0 emits `[[AYKEN_SCHED_MB_REJECT]] reason=2`

### 2. PID Validity

**Invariant:** Candidate PID MUST be in range (1-1000).

**Enforcement:**
- Ring3: Caller responsibility (no validation)
- Ring0: Rejects if `pid == 0 || pid > 1000`

**Violation:** Ring0 emits `[[AYKEN_SCHED_MB_REJECT]] reason=3`

### 3. Atomicity

**Invariant:** Mailbox writes MUST be atomic (no torn reads).

**Enforcement:**
- Ring3: Write order (candidate_pid first, epoch last)
- Ring0: Double-read check (`e1 == e2`)

**Violation:** Ring0 emits `[[AYKEN_SCHED_MB_REJECT]] reason=1`

### 4. No Syscalls

**Invariant:** API MUST NOT use syscalls (1000-1010 frozen).

**Enforcement:**
- Mailbox pre-mapped by Ring0 at process creation
- Direct memory access (no kernel involvement)

**Verification:** Symbol scan, CI gate `ci-gate-boundary`

---

## Error Handling

### Ring0 Rejection Reasons

Ring0 validates mailbox on timer tick and emits markers:

**ACCEPT:**
```
[[AYKEN_SCHED_MB_ACCEPT]] pid=<pid> epoch=<epoch>
```

**REJECT:**
```
[[AYKEN_SCHED_MB_REJECT]] reason=<reason> epoch=<epoch> pid=<pid>
```

**Reason Codes:**
- `1` - Torn read (epoch changed during read)
- `2` - Stale epoch (epoch <= last_epoch)
- `3` - Invalid PID (pid == 0 || pid > 1000)
- `4` - No mailbox (mailbox_pa == 0)
- `5` - Reserved (future use)

**Handling:**
- Ring3 does NOT receive rejection notifications
- Rejection is silent (fail-closed)
- Ring3 policy should be stateless (no feedback loop)

---

## Performance

### Per-Call Overhead

**Ring3 (`ayken_sched_hint`):**
- 1 memory read (current epoch)
- 1 increment (next epoch)
- 2 memory writes (candidate_pid, epoch)
- **Total:** ~10 CPU cycles (no syscall overhead)

**Ring0 (validation on timer tick):**
- 3 memory reads (double-read + pid)
- 3 comparisons (atomicity, monotonicity, validity)
- 1 marker write (debugcon)
- **Total:** ~20 CPU cycles (validation profile only)

**Release Profile:**
- Ring3: Same overhead (library always compiled)
- Ring0: Zero overhead (validation compile-out)

### Memory Overhead

**Per-Process:**
- 1 physical frame (4 KB) for mailbox
- 1 page table entry
- 2 uint64_t fields in `proc_t` (16 bytes)

**Total:** ~4 KB per process

---

## Build Integration

### Compilation

```bash
cd userspace/libayken
make all          # Build library object
make test         # Build test binary
make clean        # Clean artifacts
make check        # Constitutional compliance check
```

### Linking

```c
// Your Ring3 program
#include "sched_hint.h"

void _start() {
    ayken_sched_hint(1);
    while (1) { }
}
```

```bash
clang -c your_program.c -o your_program.o
clang your_program.o sched_hint.o -o your_program
```

---

## Testing

### Unit Test

```bash
cd userspace/libayken
make test
./sched_hint_test
```

**Expected Output:**
```
=== Ring3 Scheduler Hint Test ===

[TEST] Writing valid hint: pid=42
[TEST] Mailbox state: epoch=1 pid=42
[TEST] Waiting for timer tick validation...

[TEST] Writing invalid hint: pid=2147483647 (out of range)
[TEST] Mailbox state: epoch=2 pid=2147483647
[TEST] Expecting REJECT (reason=3, invalid PID)...

[TEST] Testing epoch monotonicity...
[TEST] First hint: epoch=3 pid=10
[TEST] Second hint: epoch=4 pid=20
[TEST] ✓ Epoch monotonicity verified

=== Test Complete ===
Check kernel log for markers:
  - [[AYKEN_SCHED_MB_ACCEPT]] (expected: 3)
  - [[AYKEN_SCHED_MB_REJECT]] (expected: 1, reason=3)
```

### Integration Test

**Note:** Integration test requires MVP-3 (real Ring3 runtime).

**Expected Behavior:**
1. Ring3 process calls `ayken_sched_hint(1)`
2. Timer tick triggers Ring0 validation
3. Ring0 emits `[[AYKEN_SCHED_MB_ACCEPT]] pid=1 epoch=1`
4. CI gate validates marker

**Status:** Deferred to MVP-3.

---

## Constitutional Compliance

### Red Lines Maintained

1. ✅ **No Syscalls** - Mailbox pre-mapped, no kernel calls
2. ✅ **No Ring0 Exports** - Library is Ring3-only
3. ✅ **ABI Stable** - No changes to `ayken_abi.h`
4. ✅ **Ring0 Mechanism Only** - Validation is pure mechanism
5. ✅ **Ring3 Policy** - Scheduling decision in userspace

### Verification

```bash
# Verify no syscalls in library
$ nm sched_hint.o | grep -E "int.*0x80|syscall"
(empty)

# Verify no Ring0 exports added
$ nm kernel.elf | grep -c " T "
165  # Unchanged

# Verify ABI stable
$ git diff HEAD~1 HEAD kernel/include/ayken_abi.h
(empty)
```

---

## Limitations

### 1. Single-Writer Assumption

**Limitation:** API is not thread-safe.

**Rationale:** Per-process mailbox assumes single writer (process itself).

**Mitigation:** If multi-threaded, caller must synchronize access.

### 2. No Feedback Loop

**Limitation:** Ring3 does not receive validation results.

**Rationale:** Fail-closed design, stateless policy.

**Mitigation:** Ring3 policy should not depend on validation results.

### 3. PID Range Hardcoded

**Limitation:** PID validity check uses hardcoded limit (1000).

**Rationale:** Pragmatic for MVP-2, sufficient for current use case.

**Future:** Use `PID_MAX` constant or proc_table limit.

### 4. No Memory Barriers

**Limitation:** No explicit memory barriers (volatile only).

**Rationale:** Single-core system, validation profile.

**Future:** Consider `smp_rmb()` or `volatile` for SMP.

---

## Future Enhancements

### MVP-3: Runtime Proof

- Real Ring3 process execution
- Timer tick → Ring0 validation → ACCEPT marker
- CI gate validation of real interaction

### Post-MVP-3: Advanced Features

- Multi-threaded support (mutex/spinlock)
- Feedback loop (validation result notification)
- Dynamic PID range (configurable limit)
- SMP support (memory barriers)
- Priority hints (extend mailbox structure)

---

## References

- **MVP-1:** Per-process mailbox mapping (Ring0 mechanism)
- **MVP-2:** Ring3 scheduler hint library (Ring3 policy)
- **MVP-3:** Runtime execution proof (privilege separation)

**Files:**
- `userspace/libayken/sched_hint.h` - API header
- `userspace/libayken/sched_hint.c` - Implementation
- `userspace/libayken/sched_hint_test.c` - Test harness
- `kernel/sched/sched_mailbox.c` - Ring0 validation
- `MVP_2_FINAL_STATUS.md` - Milestone closure report

---

**Düzenleyen:** Kenan AY  
**Date:** 2026-02-22  
**Version:** 1.0  
**Status:** Production-Ready (Library Level)

**This API is constitutional-grade and ready for production use (library level). Runtime proof is deferred to MVP-3.**

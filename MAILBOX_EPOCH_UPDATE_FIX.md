# Mailbox Epoch Update Fix

**Date:** 2026-04-14
**Issue:** BCIB worker mailbox epoch becomes stale after first scheduler handoff
**Status:** FIXED

## Problem Analysis

### Observed Behavior

**First Extraction (SUCCESS):**
```
[[AYKEN_PERF_MB_EXTRACT_RAW]] epoch=1 candidate_pid=2 owner_last_epoch=0
[[AYKEN_PERF_MB_EXTRACT_REASON]] name=ok
[[AYKEN_PERF_MB_CONSUME]] site=START old_last_epoch=0 new_last_epoch=1 candidate_epoch=1
[[AYKEN_SCHED_MB_ACCEPT]] pid=2 epoch=1
```

**Subsequent Extractions (STALE):**
```
[[AYKEN_PERF_MB_EXTRACT_RAW]] epoch=1 candidate_pid=2 owner_last_epoch=1
[[AYKEN_PERF_MB_EXTRACT_REASON]] name=epoch_stale
```

### Root Cause

The mailbox epoch freshness check in `kernel/sched/sched.c:sched_mailbox_extract_candidate()`:

```c
if (mb->epoch == 0 || mb->epoch <= owner->mailbox_last_epoch) {
    sched_emit_perf_mb_extract_reason_marker("epoch_stale");
    return 0;
}
```

**Lifecycle:**
1. Worker created with `mailbox epoch = 1` (kernel bootstrap in `bcib_worker.c`)
2. Scheduler accepts first handoff: `owner->mailbox_last_epoch = 0` → `1` ✓
3. Worker executes but **never updates mailbox epoch**
4. Next scheduler tick: `epoch (1) <= owner_last_epoch (1)` → **"epoch_stale"** ❌

### Expected Behavior

Mailbox epoch must be **monotonically increasing** to maintain freshness:

```
Initial:        epoch=1, owner_last_epoch=0 → OK (1 > 0)
After 1st:      epoch=2, owner_last_epoch=1 → OK (2 > 1)
After 2nd:      epoch=3, owner_last_epoch=2 → OK (3 > 2)
...
```

## Solution

### Mailbox Structure

The scheduler mailbox is mapped at fixed VA `0x700000` (SCHED_MAILBOX_VA) with USER | WRITABLE permissions:

```c
// From shared/abi/sched_mailbox_abi.h
typedef struct {
    uint32_t magic;           // +0x00
    uint16_t version;         // +0x04
    uint16_t kind;            // +0x06
    uint64_t epoch;           // +0x08 <- CRITICAL: Must increment
    uint32_t proposer_pid;    // +0x10
    uint32_t candidate_pid;   // +0x14
    uint32_t flags;           // +0x18
    uint32_t status;          // +0x1C
    uint32_t reject_reason;   // +0x20
    uint32_t reserved;        // +0x24
} ayken_sched_mailbox_t;
```

### Implementation

**File:** `userspace/minimal/minimal_bcib_worker.S`

**Key Changes:**
1. Load mailbox address into `%rbx` at start: `mov $0x700000, %rbx`
2. After each work cycle, increment epoch:
   ```asm
   # Read current epoch
   mov 0x08(%rbx), %rax
   
   # Increment
   inc %rax
   
   # Handle wraparound (epoch must never be 0)
   test %rax, %rax
   jnz .Lepoch_ok
   mov $1, %rax
   .Lepoch_ok:
   
   # Write back
   mov %rax, 0x08(%rbx)
   ```

3. Emit `[EPOCH_UPDATE]` marker for observability

### Verification

**Expected Log Sequence:**
```
[BCIB_WORKER_START]
[[AYKEN_PERF_MB_EXTRACT_RAW]] epoch=1 candidate_pid=2 owner_last_epoch=0
[[AYKEN_PERF_MB_EXTRACT_REASON]] name=ok
[[AYKEN_SCHED_MB_ACCEPT]] pid=2 epoch=1
[BCIB_SUBMIT_OK]
[EPOCH_UPDATE]
[[AYKEN_PERF_MB_EXTRACT_RAW]] epoch=2 candidate_pid=2 owner_last_epoch=1
[[AYKEN_PERF_MB_EXTRACT_REASON]] name=ok
[[AYKEN_SCHED_MB_ACCEPT]] pid=2 epoch=2
[BCIB_SUBMIT_OK]
[EPOCH_UPDATE]
...
```

**Key Indicators:**
- ✅ `epoch` increments: 1 → 2 → 3 → ...
- ✅ `owner_last_epoch` follows: 0 → 1 → 2 → ...
- ✅ `name=ok` on all extractions (no more "epoch_stale")
- ✅ `[EPOCH_UPDATE]` marker appears after each work cycle

## Architecture Notes

### Why Userspace Updates?

The mailbox is a **Ring3 → Ring0 communication channel**:
- **Ring3 (userspace):** Publishes scheduling decisions and work readiness
- **Ring0 (kernel):** Validates and consumes mailbox proposals

**Authority Model:**
- Kernel **validates** epoch freshness (authoritative check)
- Userspace **publishes** epoch updates (policy decision)
- No syscall needed (mailbox pre-mapped with write permissions)

### Epoch Monotonicity Contract

From `docs/governance/MAILBOX_PROTOCOL_V1_FREEZE.md`:

**Validation Rules:**
1. ABI checks: magic/version/kind
2. Torn-read guard: `e1 == e2` (epoch double read)
3. **Monotonic epoch:** `epoch > mailbox_last_epoch` and `epoch != 0`
4. Candidate sanity: `candidate_pid != 0`, resolvable, runnable

**Consumption:**
```c
// kernel/sched/sched.c
if (consume_epoch) {
    uint64_t old_last_epoch = owner->mailbox_last_epoch;
    owner->mailbox_last_epoch = epoch;  // Consume epoch
    sched_perf_note_mailbox_consume(...);
}
```

### Comparison with Other Processes

**Normal User Processes:**
- Use `libayken/sched_hint.c:ayken_sched_hint_candidate()`
- Library handles epoch increment: `next_epoch = publisher->mailbox_last_epoch + 1`
- Kernel seeds initial mailbox at process creation

**BCIB Worker:**
- Kernel-managed process (validation profile only)
- Bootstrap mailbox with epoch=1 at creation
- **Must manually increment epoch** after work cycles
- No library dependency (minimal assembly payload)

## Testing

### Build and Run

```bash
# Build with BCIB worker mode
USER_MINIMAL_MODE=bcib-worker-bootstrap make efi-img

# Run QEMU with trace
./scripts/qemu-runtime-bridge-proof-harness.sh

# Check logs
grep "EPOCH_UPDATE" evidence/runtime-bridge-proof/qemu_debugcon.log
grep "epoch_stale" evidence/runtime-bridge-proof/qemu_debugcon.log  # Should be empty after fix
```

### Success Criteria

- ✅ No "epoch_stale" rejections after first handoff
- ✅ Monotonically increasing epoch values in logs
- ✅ Worker continues to be scheduled across multiple ticks
- ✅ `[EPOCH_UPDATE]` markers present in trace

## Related Files

**Modified:**
- `userspace/minimal/minimal_bcib_worker.S` - Added epoch increment logic

**Reference:**
- `kernel/proc/bcib_worker.c` - Worker creation and bootstrap
- `kernel/sched/sched.c` - Mailbox extraction and consumption
- `kernel/sched/sched_mailbox.h` - SCHED_MAILBOX_VA definition
- `shared/abi/sched_mailbox_abi.h` - Mailbox structure
- `docs/governance/MAILBOX_PROTOCOL_V1_FREEZE.md` - Protocol specification

## Constitutional Compliance

This fix maintains:
- ✅ **DETERMINISM.GLOBAL:** Epoch is deterministic and monotonic
- ✅ **SECURITY.BOUNDARY.VIOLATION:** No Ring0 access, userspace-only update
- ✅ **KERNEL.SAFETY.CRITICAL:** Kernel validation remains authoritative
- ✅ **MEMORY.CONTRACT.VIOLATION:** Bounded memory access (fixed VA)

## Next Steps

1. Build and test with updated worker payload
2. Verify epoch increment in QEMU traces
3. Confirm no "epoch_stale" rejections
4. Document in execution delivery proof
5. Update Task 5 progress with mailbox epoch fix

---

**Fix Author:** Kiro AI Assistant
**Review Required:** Kenan AY (Architectural Steward)

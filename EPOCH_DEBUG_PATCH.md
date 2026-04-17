# Epoch Debug Patch

## Purpose

After bisect identifies the regression commit, apply this patch to diagnose epoch progression issue.

## Suspected Root Causes

Based on evidence:
- `epoch_gt_owner_last_epoch_count: 1` (only 1 progression)
- `epoch_lte_owner_last_epoch_count: 61` (61 stuck)
- `epoch_stale: 61` (all rejected as stale)

**Hypothesis:**
1. Producer not incrementing epoch
2. Consumer overshooting owner_last_epoch
3. Epoch comparison logic broken

## Debug Patch

Add to `kernel/sched/sched.c` in mailbox extract path:

```c
// In mailbox_extract_candidate() or equivalent

static void debug_epoch_mismatch(uint64_t candidate_epoch, 
                                  uint64_t owner_last_epoch,
                                  uint64_t candidate_pid) {
    static int debug_count = 0;
    if (debug_count < 5) {  // Limit spam
        printk("[EPOCH_DEBUG] candidate_pid=%lu candidate_epoch=%lu owner_last_epoch=%lu\n",
               candidate_pid, candidate_epoch, owner_last_epoch);
        debug_count++;
    }
}

// Before epoch staleness check:
if (candidate_epoch <= owner_last_epoch) {
    debug_epoch_mismatch(candidate_epoch, owner_last_epoch, candidate_pid);
    // ... existing rejection logic ...
}
```

## Expected Debug Output Patterns

### Pattern A: Producer Stuck (Most Likely)
```
[EPOCH_DEBUG] candidate_pid=2 candidate_epoch=1 owner_last_epoch=1
[EPOCH_DEBUG] candidate_pid=2 candidate_epoch=1 owner_last_epoch=1
[EPOCH_DEBUG] candidate_pid=2 candidate_epoch=1 owner_last_epoch=1
```
**Diagnosis:** Producer not incrementing epoch
**Fix location:** Mailbox write path (where epoch is assigned)

### Pattern B: Consumer Overshoot
```
[EPOCH_DEBUG] candidate_pid=2 candidate_epoch=1 owner_last_epoch=2
[EPOCH_DEBUG] candidate_pid=2 candidate_epoch=2 owner_last_epoch=3
[EPOCH_DEBUG] candidate_pid=2 candidate_epoch=3 owner_last_epoch=4
```
**Diagnosis:** Consumer incrementing owner_last_epoch too much
**Fix location:** After mailbox consume (owner_last_epoch update)

### Pattern C: Initial Epoch Wrong
```
[EPOCH_DEBUG] candidate_pid=2 candidate_epoch=0 owner_last_epoch=1
[EPOCH_DEBUG] candidate_pid=2 candidate_epoch=0 owner_last_epoch=1
```
**Diagnosis:** Candidate starts at 0, owner starts at 1
**Fix location:** Initialization (proc creation or mailbox init)

## Code Areas to Inspect

### 1. Producer Side (Mailbox Write)

```c
// kernel/proc/proc.c or kernel/sched/sched.c
// When writing to mailbox:

// CORRECT:
mailbox->epoch = ++proc->current_epoch;  // Increment before write

// WRONG:
mailbox->epoch = proc->current_epoch;  // No increment - STUCK!

// WRONG:
mailbox->epoch = 0;  // Hardcoded - STUCK!
```

### 2. Consumer Side (After Accept)

```c
// kernel/sched/sched.c
// After accepting candidate:

// CORRECT:
owner->last_epoch = candidate_epoch;  // Match candidate

// WRONG:
owner->last_epoch = candidate_epoch + 1;  // Overshoot!

// WRONG:
owner->last_epoch = global_epoch;  // Wrong source!
```

### 3. Initialization

```c
// kernel/proc/proc.c - proc_create()

// CORRECT:
proc->current_epoch = 0;
proc->last_epoch = 0;  // Both start at 0

// WRONG:
proc->current_epoch = 0;
proc->last_epoch = 1;  // Mismatch - immediate stale!
```

## Bisect + Debug Workflow

### Step 1: Run Bisect
```bash
git bisect start
git bisect bad 9b3358e6
git bisect good 050332220d9a
git bisect run scripts/ci/bisect_performance_regression.sh
```

### Step 2: Identify First Bad Commit
```bash
# Bisect will output:
# abc1234567890abcdef is the first bad commit

git show abc1234567890abcdef
```

### Step 3: Apply Debug Patch
```bash
# Checkout the bad commit
git checkout abc1234567890abcdef

# Apply debug patch to kernel/sched/sched.c
# (Add debug_epoch_mismatch() as shown above)

# Rebuild and test
make clean
make KERNEL_PROFILE=validation USER_MINIMAL_MODE=syscall-v2-runtime efi-img

# Run with debug output
mkdir -p evidence/epoch-debug
scripts/ci/gate_performance.sh --evidence-dir evidence/epoch-debug

# Check QEMU log for [EPOCH_DEBUG] markers
grep "EPOCH_DEBUG" evidence/epoch-debug/preempt.log
```

### Step 4: Analyze Pattern
```bash
# Pattern A (producer stuck):
# → Fix mailbox write path

# Pattern B (consumer overshoot):
# → Fix owner_last_epoch update

# Pattern C (init mismatch):
# → Fix proc initialization
```

### Step 5: Compare with Parent Commit
```bash
# Checkout parent (last good)
git checkout abc1234567890abcdef^

# Check epoch logic differences
git diff abc1234567890abcdef^ abc1234567890abcdef -- kernel/sched/sched.c kernel/proc/proc.c

# Look for changes in:
# - mailbox->epoch assignment
# - owner->last_epoch update
# - proc->current_epoch initialization
```

## Expected Fix Examples

### Fix A: Producer Stuck
```c
// BEFORE (broken):
void mailbox_write(proc_t *proc, ...) {
    mailbox->epoch = proc->current_epoch;  // No increment!
}

// AFTER (fixed):
void mailbox_write(proc_t *proc, ...) {
    mailbox->epoch = ++proc->current_epoch;  // Increment!
}
```

### Fix B: Consumer Overshoot
```c
// BEFORE (broken):
void mailbox_consume(proc_t *owner, uint64_t candidate_epoch) {
    owner->last_epoch = candidate_epoch + 1;  // Overshoot!
}

// AFTER (fixed):
void mailbox_consume(proc_t *owner, uint64_t candidate_epoch) {
    owner->last_epoch = candidate_epoch;  // Match!
}
```

### Fix C: Comparison Logic
```c
// BEFORE (broken):
if (candidate_epoch <= owner_last_epoch) {  // Too strict!
    reject_as_stale();
}

// AFTER (fixed):
if (candidate_epoch < owner_last_epoch) {  // Correct!
    reject_as_stale();
}
```

## Verification After Fix

```bash
# Apply fix
# Rebuild
make clean
make KERNEL_PROFILE=validation USER_MINIMAL_MODE=syscall-v2-runtime efi-img

# Test
mkdir -p evidence/fix-verification
scripts/ci/gate_performance.sh --evidence-dir evidence/fix-verification

# Check mailbox stats
jq '.raw_metrics.mailbox_phase_breakdown_ticks.extract_diagnostics' \
   evidence/fix-verification/actual.lock.json
```

**Expected after fix:**
```json
{
  "extract_reasons": {
    "ok": 60,           // Most accepted
    "epoch_stale": 1    // Minimal stale
  },
  "raw_observations": {
    "epoch_gt_owner_last_epoch_count": 60,  // Epoch progressing!
    "epoch_lte_owner_last_epoch_count": 1   // Minimal stuck
  }
}
```

**Performance after fix:**
- `boot_time_ms`: ~10700 (baseline)
- `context_switch_latency_ms_proxy`: ~175 (baseline)
- `syscall_latency_ms_proxy`: ~175 (baseline)

## Critical Notes

- Debug output limited to 5 prints to avoid log spam
- Focus on epoch values, not just comparison result
- Compare parent vs regression commit for epoch logic changes
- Fix should be 1-5 lines (increment, assignment, or comparison)
- After fix, epoch should progress normally (60+ accepted, 0-1 stale)

## Timeline

- Bisect: ~2 hours
- Debug patch + analysis: 30 min
- Fix: 15 min
- Verification: 15 min
- **Total: ~3 hours**

## Success Criteria

✅ Epoch progresses normally (epoch_gt_owner_last_epoch_count: 60+)
✅ Minimal stale rejections (epoch_stale: 0-1)
✅ Minimal fallback (no_candidate: 0-5)
✅ Performance returns to baseline
✅ All CI gates pass

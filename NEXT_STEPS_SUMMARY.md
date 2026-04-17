# Next Steps Summary - Performance Regression Resolution

## Current Status

✅ **Root cause identified:** Epoch progression broken (61/61 stuck)
✅ **Bisect infrastructure ready:** Automated script with epoch diagnostics
✅ **Debug strategy prepared:** Epoch debug patch for post-bisect analysis
✅ **All local CI gates passing:** System is functional, just slow

## Critical Evidence

```json
{
  "epoch_gt_owner_last_epoch_count": 1,   // Only 1 progression
  "epoch_lte_owner_last_epoch_count": 61, // 61 stuck
  "extract_reasons": {
    "ok": 1,           // Only 1 accepted
    "epoch_stale": 61  // 61 rejected as stale
  },
  "fallback_reasons": {
    "no_candidate": 61  // Every tick falls back
  }
}
```

**Translation:** Scheduler finds candidates but rejects them all as "epoch stale" because epoch is not progressing.

## Root Cause Hypothesis (Refined)

**Most likely (60%):** Producer not incrementing epoch
```c
// BROKEN:
mailbox->epoch = proc->current_epoch;  // No increment!

// CORRECT:
mailbox->epoch = ++proc->current_epoch;  // Increment!
```

**Likely (30%):** Consumer overshooting owner_last_epoch
```c
// BROKEN:
owner->last_epoch = candidate_epoch + 1;  // Overshoot!

// CORRECT:
owner->last_epoch = candidate_epoch;  // Match!
```

**Possible (10%):** Comparison logic too strict
```c
// BROKEN:
if (candidate_epoch <= owner_last_epoch) {  // Too strict!

// CORRECT:
if (candidate_epoch < owner_last_epoch) {  // Correct!
```

**Note:** Evidence strongly suggests epoch progression issue (not comparison), so focus on epoch write/update points first.

## Immediate Next Steps

### Step 1: Run Bisect (GitHub CI or Local Linux x86_64)

```bash
git bisect start
git bisect bad 9b3358e6  # current HEAD (61/61 stuck)
git bisect good 050332220d9a  # baseline (normal scheduling)
git bisect run scripts/ci/bisect_performance_regression.sh
```

**Expected output:**
```
Bisecting: 47 revisions left to test after this (roughly 6 steps)
... (7 iterations) ...
abc1234567890abcdef is the first bad commit
```

**Timeline:** ~2 hours (7 steps × 15 min)

### Step 2: Analyze First Bad Commit

```bash
# Show the regression commit
git show abc1234567890abcdef

# Look for changes in:
# - kernel/sched/sched.c (mailbox extract/consume)
# - kernel/proc/proc.c (mailbox write/epoch)
# - Any epoch assignment or comparison
```

### Step 3: Apply Debug Patch (If Needed)

```bash
# Checkout bad commit
git checkout abc1234567890abcdef

# Add debug output to kernel/sched/sched.c:
# (See EPOCH_DEBUG_PATCH.md for details)

# Rebuild and check debug output
make clean && make KERNEL_PROFILE=validation USER_MINIMAL_MODE=syscall-v2-runtime efi-img
mkdir -p evidence/epoch-debug
scripts/ci/gate_performance.sh --evidence-dir evidence/epoch-debug
grep "EPOCH_DEBUG" evidence/epoch-debug/preempt.log
```

**Expected patterns:**
- Pattern A: `candidate_epoch=1 owner_last_epoch=1` (producer stuck)
- Pattern B: `candidate_epoch=1 owner_last_epoch=2` (consumer overshoot)
- Pattern C: `candidate_epoch=0 owner_last_epoch=1` (init mismatch)

### Step 4: Fix the Issue

Based on debug pattern, apply appropriate fix:

**Fix A (Producer):**
```c
// Add increment to mailbox write
mailbox->epoch = ++proc->current_epoch;
```

**Fix B (Consumer):**
```c
// Remove overshoot from owner update
owner->last_epoch = candidate_epoch;  // Not +1
```

**Fix C (Comparison):**
```c
// Fix comparison operator
if (candidate_epoch < owner_last_epoch) {  // Not <=
```

### Step 5: Verify Fix

```bash
# Rebuild with fix
make clean && make KERNEL_PROFILE=validation USER_MINIMAL_MODE=syscall-v2-runtime efi-img

# Test
mkdir -p evidence/fix-verification
scripts/ci/gate_performance.sh --evidence-dir evidence/fix-verification

# Check mailbox stats
jq '.raw_metrics.mailbox_phase_breakdown_ticks.extract_diagnostics' \
   evidence/fix-verification/actual.lock.json
```

**Success criteria:**
```json
{
  "extract_reasons": {
    "ok": 60,           // Most accepted (not 1)
    "epoch_stale": 1    // Minimal stale (not 61)
  },
  "raw_observations": {
    "epoch_gt_owner_last_epoch_count": 60,  // Progressing (not 1)
    "epoch_lte_owner_last_epoch_count": 1   // Minimal stuck (not 61)
  }
}
```

**Performance:**
- `boot_time_ms`: ~10700 (baseline 10684)
- `context_switch_latency_ms_proxy`: ~175 (baseline 175.08)
- `syscall_latency_ms_proxy`: ~175 (baseline 175.08)

### Step 6: Commit and Push

```bash
# Commit the fix
git add kernel/sched/sched.c  # or relevant file
git commit -m "fix(sched): Fix epoch progression in mailbox path

Root cause: [Producer not incrementing / Consumer overshooting / etc]

Evidence:
- Before: epoch_stale=61, no_candidate=61 (61/61 fallback)
- After: epoch_stale=1, no_candidate=0-5 (normal scheduling)

Performance impact:
- boot_time_ms: 12197 → 10700 (back to baseline)
- context_switch: 201.93 → 175 (back to baseline)
- syscall: 201.93 → 175 (back to baseline)

Fixes: #[issue_number]"

# Push to branch
git push origin fix/epoch-progression-regression

# Create PR
# CI will verify performance returns to baseline
```

## Files Created

1. `scripts/ci/bisect_performance_regression.sh` - Automated bisect with epoch diagnostics
2. `BISECT_REGRESSION_GUIDE.md` - Detailed bisect instructions
3. `EPOCH_DEBUG_PATCH.md` - Debug patch and analysis guide
4. `REGRESSION_ROOT_CAUSE_ANALYSIS.md` - Root cause analysis
5. `GITHUB_CI_PERF_ANALYSIS_PLAN.md` - Alternative approaches
6. `NEXT_STEPS_SUMMARY.md` - This file

## Timeline Estimate

- Bisect: ~2 hours (7 steps)
- Analysis: 30 min (inspect commit)
- Debug (if needed): 30 min (apply patch, analyze output)
- Fix: 15 min (1-5 line change)
- Verification: 15 min (rebuild + test)
- **Total: ~3.5 hours**

## Critical Rules

✅ Run bisect in GitHub CI or Linux x86_64 (authoritative environment)
✅ Focus on epoch progression, not just comparison
✅ Compare parent vs regression commit for epoch logic changes
✅ Verify epoch_gt_owner_last_epoch_count increases after fix
✅ Verify performance returns to baseline after fix
❌ Don't update baseline - this is a bug, not a feature cost
❌ Don't accept 61/61 fallback as normal
❌ Don't use local macOS results for decisions

## Expected Outcome

- Bisect identifies commit with broken epoch logic
- Fix is likely 1-5 lines (increment, assignment, or comparison)
- Epoch progression should return to normal (60+ accepted, 0-1 stale)
- Fallback should drop from 61 → 0-5
- Performance should return to baseline range
- Verify with authoritative metrics before closing

**Important:** Don't assume fix will perfectly restore baseline. Verify these three metrics:
1. `no_candidate`: 61 → single digit
2. `epoch_stale`: 61 → single digit  
3. Performance: within baseline threshold range

If metrics don't improve as expected, deeper analysis may be needed.

## Key Insight

**This is NOT a performance optimization problem.**
**This is a scheduler correctness bug.**

The scheduler is working correctly in terms of marker chain and execution flow, but the mailbox epoch logic is broken, causing it to reject all valid candidates and fall back every tick.

Fix the epoch logic → performance returns to baseline automatically.

## What NOT To Do

❌ Don't add performance counters yet (not needed, root cause is clear)
❌ Don't optimize hot paths (not the problem)
❌ Don't update baseline (hiding the bug)
❌ Don't accept 61/61 fallback as "new normal"
❌ Don't blame Phase 16 features (they're not the cause)

## What TO Do

✅ Run bisect to find broken commit
✅ Inspect epoch logic changes in that commit
✅ Fix epoch increment/assignment/comparison
✅ Verify epoch progression returns to normal
✅ Verify performance returns to baseline
✅ Commit fix with evidence in commit message

## Ready to Execute

All infrastructure is ready. Next command:

```bash
git bisect start
git bisect bad 9b3358e6
git bisect good 050332220d9a
git bisect run scripts/ci/bisect_performance_regression.sh
```

Then follow steps 2-6 above.

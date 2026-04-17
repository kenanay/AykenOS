# Performance Regression Root Cause Analysis

## Executive Summary

**Problem:** +14-15% performance regression across all metrics
**Root Cause:** Scheduler fallback path abuse (61/61 ticks)
**Solution:** Bisect to find broken mailbox/epoch logic, then fix

## Critical Evidence

### GitHub CI Metrics (Authoritative)

| Metric | Baseline | Current | Regression |
|--------|----------|---------|------------|
| Boot time | 10684ms | 12197ms | +14% |
| Context switch | 175.08ms | 201.93ms | +15% |
| Syscall | 175.08ms | 201.93ms | +15% |

### Mailbox Fallback Pattern (Smoking Gun)

```json
{
  "fallback_reasons": {
    "no_candidate": 61,
    "owner_mismatch": 0,
    "candidate_proc_missing": 0,
    "candidate_proc_not_schedulable": 0
  },
  "extract_diagnostics": {
    "raw_observations": {
      "count": 62,
      "candidate_pid_nonzero_count": 62,
      "epoch_gt_owner_last_epoch_count": 1,
      "epoch_lte_owner_last_epoch_count": 61
    },
    "extract_reasons": {
      "ok": 1,
      "epoch_stale": 61
    }
  }
}
```

**Translation:**
- 62 mailbox extracts attempted
- 61 rejected as "epoch_stale"
- 1 accepted
- Result: 61/61 fallback to idle/retry path

## Root Cause Hypothesis

**Epoch staleness logic is broken:**

```c
// Suspected broken logic:
if (candidate_epoch <= owner_last_epoch) {
    // Reject as STALE
    // This condition is TOO STRICT or owner_last_epoch is WRONG
}
```

**Why this causes +15% regression:**
- Normal path: Find candidate → switch (fast)
- Fallback path: No candidate → fallback logic → retry (slow)
- 61/61 fallback = every tick pays fallback overhead

## Why Local Tests Were Misleading

Local macOS tests showed "features OFF = slower" but:
- Different environment (Darwin ARM64 vs Linux x86_64)
- Different compiler (Apple clang vs Ubuntu clang)
- Different QEMU (10.2.0 vs 8.2.2)
- **Not authoritative for baseline comparison**

GitHub CI is authoritative - same environment as baseline.

## Bisect Strategy

### Commands

```bash
# Start bisect
git bisect start
git bisect bad 9b3358e6  # current (61/61 fallback)
git bisect good 050332220d9a  # baseline (normal scheduling)

# Automated bisect
git bisect run scripts/ci/bisect_performance_regression.sh
```

### Expected Result

```
abc1234567890abcdef is the first bad commit
commit abc1234567890abcdef
Date: 2026-04-XX

    [Scheduler/mailbox/epoch logic change]
    
    Changes:
    - kernel/sched/sched.c (epoch comparison logic)
    - OR kernel/proc/proc.c (candidate visibility)
    - OR mailbox extract/validate chain
```

### Timeline

- Bisect: ~7 steps × 15 min = ~2 hours
- Analysis: 30 min
- Fix: 30 min
- Verification: 15 min
- **Total: ~3 hours**

## Suspected Code Areas

### 1. Epoch Staleness Check (Most Likely)

```c
// kernel/sched/sched.c - mailbox_extract_candidate()

// BEFORE (working):
if (candidate_epoch < owner_last_epoch) {
    // Reject only if strictly less
}

// AFTER (broken):
if (candidate_epoch <= owner_last_epoch) {
    // Reject if less than OR EQUAL
    // This rejects valid candidates!
}
```

### 2. Owner Last Epoch Update (Possible)

```c
// kernel/sched/sched.c - after consume

// BEFORE (working):
owner->last_epoch = candidate_epoch;  // Update after accept

// AFTER (broken):
owner->last_epoch = candidate_epoch + 1;  // Off by one!
// Next candidate with same epoch is rejected as stale
```

### 3. Candidate Epoch Assignment (Possible)

```c
// kernel/proc/proc.c - when writing to mailbox

// BEFORE (working):
mailbox->epoch = current_epoch;

// AFTER (broken):
mailbox->epoch = current_epoch - 1;  // Off by one!
// Candidate appears stale immediately
```

## Resolution Options

### Option A: Revert Regression Commit (Fast)

```bash
git revert <bad_commit>
make clean && make KERNEL_PROFILE=validation USER_MINIMAL_MODE=syscall-v2-runtime efi-img
scripts/ci/gate_performance.sh --evidence-dir evidence/revert-test

# Expected:
# - fallback_reasons.no_candidate: 0-5 (normal)
# - boot_time_ms: ~10700 (baseline)
# - context_switch: ~175 (baseline)
```

### Option B: Fix Epoch Logic (Correct)

```bash
# Identify broken condition
# Fix comparison operator or epoch update
# Test with same commands as Option A
```

### Option C: Accept Fallback (Only if Justified)

```bash
# Document why 61/61 fallback is intentional
# Example: New safety check that rejects candidates
# Update baseline via authorized workflow
# Requires strong justification
```

## Verification Criteria

After fix/revert:

✅ **Mailbox stats:**
- `fallback_reasons.no_candidate`: 0-5 (not 61)
- `extract_reasons.ok`: 60+ (not 1)
- `extract_reasons.epoch_stale`: 0-5 (not 61)

✅ **Performance:**
- `boot_time_ms`: 10600-10800 (baseline range)
- `context_switch_latency_ms_proxy`: 170-180 (baseline range)
- `syscall_latency_ms_proxy`: 170-180 (baseline range)

✅ **CI gates:**
- Performance gate: PASS
- All other gates: PASS

## Key Insights

1. **Not a feature overhead problem** - it's a logic bug
2. **Uniform regression** - because fallback affects all paths
3. **61/61 pattern** - scheduler never finds valid candidates
4. **Epoch staleness** - most likely culprit based on extract_reasons
5. **Fix is simple** - likely one-line comparison operator or off-by-one

## Files Created

- `scripts/ci/bisect_performance_regression.sh` - Automated bisect script
- `BISECT_REGRESSION_GUIDE.md` - Detailed bisect instructions
- `REGRESSION_ROOT_CAUSE_ANALYSIS.md` - This file
- `GITHUB_CI_PERF_ANALYSIS_PLAN.md` - Alternative approaches
- `kernel/perf/perf_diag.{h,c}` - Measurement infrastructure (for future use)

## Next Action

**Run bisect in GitHub CI:**

```bash
git bisect start
git bisect bad 9b3358e6
git bisect good 050332220d9a
git bisect run scripts/ci/bisect_performance_regression.sh
```

Then analyze the first bad commit's mailbox/epoch changes.

## Expected Outcome

- Bisect identifies commit with broken epoch logic
- Fix comparison operator or epoch update
- Fallback drops from 61 → 0-5
- Performance returns to baseline
- No baseline update needed (regression fixed)

## Critical Rule

**Do NOT update baseline until:**
- Bisect identifies regression commit
- Root cause is understood
- Either fixed OR justified as intentional
- Mailbox fallback pattern is explained

This is a bug, not a feature cost. Fix the bug, don't hide it with baseline update.

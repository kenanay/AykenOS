# Bisect Results Analysis - Critical Findings

## Executive Summary

**CRITICAL DISCOVERY:** The 61/61 mailbox fallback pattern has been present since BEFORE the baseline was established. This is NOT a recent regression - it's either:
1. Expected behavior that was always there
2. A bug that was introduced before mailbox stats were added
3. A misinterpretation of what "61/61 fallback" means

## Bisect Results

### Command Executed
```bash
git bisect start
git bisect bad 9b3358e6  # current HEAD
git bisect good 050332220d9a  # baseline
git bisect run scripts/ci/bisect_performance_regression.sh
```

### First Bad Commit Identified
```
commit 71a2ef0a51742f45ab8d763bb6dcbf67f4b6e1e1
Author: Kenan AY <kenanay34@gmail.com>
Date:   Fri Apr 10 00:08:15 2026 +0300

    perf(baseline): update lock for gha-ubuntu24-20260406.80.1-X64 
    [authorized CI init run 24213352466]
```

### Critical Finding

**This commit is a BASELINE UPDATE, not a code change!**

- Parent commit: `050332220d9a` (the "good" baseline)
- This commit: `71a2ef0a` (baseline update)
- **They are consecutive commits** (no code changes between them)

### Mailbox Stats Comparison

#### Baseline (050332220d9a) - "GOOD" commit
```json
{
  "fallback_reasons": {
    "no_candidate": 61
  },
  "extract_diagnostics": {
    "extract_reasons": {
      "ok": 1,
      "epoch_stale": 61
    },
    "raw_observations": {
      "epoch_gt_owner_last_epoch_count": 1,
      "epoch_lte_owner_last_epoch_count": 61
    }
  }
}
```

#### Updated Baseline (71a2ef0a) - "BAD" commit
```json
{
  "fallback_reasons": {
    "no_candidate": 61
  },
  "extract_diagnostics": {
    "extract_reasons": {
      "ok": 1,
      "epoch_stale": 61
    },
    "raw_observations": {
      "epoch_gt_owner_last_epoch_count": 1,
      "epoch_lte_owner_last_epoch_count": 61
    }
  }
}
```

**IDENTICAL PATTERNS!** Both show 61/61 fallback.

### Bisect Performance Results

All tested commits showed similar patterns:

| Commit | Boot Time | Fallback | Epoch Stale | Pattern |
|--------|-----------|----------|-------------|---------|
| 2f871def | 13928ms | 61 | 61 | 61/61 |
| c17bafcf | 12361ms | 61 | 61 | 61/61 |
| bc94a7b3 | 11791ms | 61 | 61 | 61/61 |
| d748433a | 11784ms | 61 | 61 | 61/61 |
| 7bc69062 | 11759ms | 61 | 61 | 61/61 |
| 71a2ef0a | 11786ms | 61 | 61 | 61/61 |

**All commits show the same 61/61 pattern!**

## Historical Analysis

### When Was Mailbox Stats Added?

Checked commit history for `fallback_reasons` in baseline:
```bash
git log --oneline --all -S "fallback_reasons" -- scripts/ci/perf-baseline.lock.json
```

Result: `8a4d4c8e ci(perf): renew baseline for runner image update`

### Mailbox Stats in First Baseline

```json
{
  "fallback_reasons": {
    "no_candidate": 61
  }
}
```

**The 61/61 pattern was present from the FIRST baseline that included mailbox stats!**

## Implications

### Option 1: This is Expected Behavior

The 61/61 fallback pattern might be NORMAL for the current workload:
- 61 timer ticks
- 1 successful candidate extraction
- 61 fallbacks due to epoch staleness
- This could be the intended behavior for the deterministic preempt harness

**Evidence:**
- Pattern present since mailbox stats were added
- Pattern present in "good" baseline (050332220d9a)
- Pattern consistent across all commits

### Option 2: Bug Introduced Before Mailbox Stats

The epoch progression bug might have been introduced BEFORE mailbox stats were added to the baseline:
- Bug introduced in earlier commit
- Mailbox stats added later
- Baseline captured the already-buggy behavior
- We've been measuring against a buggy baseline

**Evidence:**
- No "good" baseline exists with different pattern
- Need to check commits before mailbox stats were added

### Option 3: Misinterpretation

The "61/61 fallback" might not mean what we think:
- Could be normal for 61-tick workload
- "epoch_stale" might not indicate a bug
- Fallback path might be the expected path for this scenario

## Performance Variation Analysis

### Baseline vs Current

| Metric | Baseline (050332220d9a) | Current (9b3358e6) | Delta |
|--------|-------------------------|---------------------|-------|
| Boot time | 10684ms | 12197ms (GH CI) | +14% |
| Context switch | 175.08ms | 201.93ms (GH CI) | +15% |
| Syscall | 175.08ms | 201.93ms (GH CI) | +15% |
| Fallback pattern | 61/61 | 61/61 | Same |

**Key observation:** Performance degraded but fallback pattern stayed the same!

### Bisect Results

All commits between baseline and current show:
- Boot time: 11700-13900ms (all above threshold)
- Fallback pattern: 61/61 (all identical)

**This suggests the performance variation is NOT caused by the fallback pattern!**

## Possible Explanations

### 1. Environment Changes

- GitHub Actions runner image updated
- QEMU version changed (8.2.2 patch level)
- Compiler optimizations changed
- System load/noise increased

### 2. Cumulative Overhead

- Multiple small changes accumulated
- Each change added small overhead
- Total overhead crossed threshold
- But fallback pattern remained constant

### 3. Measurement Noise

- Performance measurements have variance
- Baseline might have been "lucky" measurement
- Current measurements might be "unlucky"
- Need more samples to establish true baseline

### 4. Hidden Bug

- Bug exists but is not visible in mailbox stats
- Affects performance but not fallback pattern
- Could be in other parts of the system
- Mailbox stats are red herring

## Recommended Next Steps

### Step 1: Verify Baseline Measurement

Re-run baseline commit (050332220d9a) in current GitHub CI environment:

```bash
git checkout 050332220d9a
make clean && make KERNEL_PROFILE=validation USER_MINIMAL_MODE=syscall-v2-runtime efi-img
# Run in GitHub CI, not local
```

**Expected outcomes:**
- If performance is still ~10684ms → environment changed
- If performance is now ~12000ms → baseline was lucky/environment changed
- If performance varies widely → measurement noise

### Step 2: Check Commits Before Mailbox Stats

Find when mailbox stats were added and check earlier commits:

```bash
git log --oneline --all -- kernel/sched/sched.c | grep -i "mailbox\|epoch"
```

Look for commits that:
- Added epoch logic
- Modified mailbox extract/validate
- Changed candidate selection

### Step 3: Understand 61/61 Pattern

Analyze the code to understand if 61/61 is expected:

```c
// kernel/sched/sched.c
// Why are 61 candidates rejected as epoch_stale?
// Is this the intended behavior for 61-tick workload?
```

Questions to answer:
- What is the expected epoch progression for 61 ticks?
- Should we see 61 successful extractions or 1?
- Is the fallback path the "normal" path for this workload?

### Step 4: Profile Other Hotspots

If 61/61 is normal, look elsewhere for performance regression:
- Boot phase timing
- Interrupt handling
- Memory allocation
- Context switch overhead
- Syscall gate overhead

### Step 5: Statistical Analysis

Run multiple measurements to establish confidence intervals:
- 10 runs of baseline commit
- 10 runs of current commit
- Calculate mean, stddev, confidence intervals
- Determine if difference is statistically significant

## Conclusion

**The bisect revealed that the 61/61 mailbox fallback pattern is NOT a recent regression.**

This pattern has been present since:
1. The baseline was established (050332220d9a)
2. Mailbox stats were first added to the baseline
3. Possibly even earlier (need to check pre-stats commits)

**The performance regression (+14-15%) exists, but it's NOT explained by the fallback pattern alone.**

Either:
- The 61/61 pattern is normal and we need to look elsewhere
- The bug is older than we thought and the baseline is wrong
- The performance variation is due to environment changes
- We're misinterpreting what the mailbox stats mean

**Next action:** Consult with Kenan AY (architectural authority) to understand:
1. Is 61/61 fallback expected for this workload?
2. When was epoch logic introduced?
3. What should "normal" mailbox stats look like?
4. Should we re-establish the baseline?


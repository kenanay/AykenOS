# Performance Regression Bisect Guide

## Critical Finding

**Mailbox fallback pattern detected:**
- `fallback_reasons.no_candidate = 61`
- `total extracts = 62`
- **61/61 fallback = scheduler ALWAYS falls back**

This explains uniform +15% regression across all metrics.

## Root Cause Hypothesis

Scheduler is not finding valid candidates, forcing fallback path every tick:
- Mailbox epoch logic changed
- Candidate visibility broken
- Owner/candidate matching broken
- Snapshot/extract/validate chain broken

## Bisect Strategy

### Step 1: Start Bisect (GitHub CI or Local Linux x86_64)

```bash
git bisect start
git bisect bad 9b3358e6  # current HEAD (slow, 61/61 fallback)
git bisect good 050332220d9a  # baseline (fast, normal scheduling)
```

### Step 2: Automated Bisect Run

```bash
git bisect run scripts/ci/bisect_performance_regression.sh
```

This will:
- Test ~7 commits (log2(95) ≈ 6.6)
- Build each commit
- Run performance gate
- Check: `boot_time_ms <= 11000` (baseline 10684 + 3% margin)
- Extract mailbox fallback stats
- Report GOOD/BAD/SKIP

### Step 3: Bisect Output

Expected output:
```
Bisecting: 47 revisions left to test after this (roughly 6 steps)
[commit_hash] commit message
=== Performance Bisect Test ===
Commit: abc1234
Threshold: boot <= 11000ms

Building...
Running performance measurement...
Results:
  boot_time_ms: 10800
  context_switch_latency_ms_proxy: 177.2
  syscall_latency_ms_proxy: 177.2

VERDICT: GOOD (performance acceptable)

... (repeat for ~7 commits) ...

abc1234567890abcdef is the first bad commit
commit abc1234567890abcdef
Author: ...
Date: ...

    [commit message - likely scheduler/mailbox change]
```

### Step 4: Analyze First Bad Commit

```bash
# Show the regression commit
git bisect log
git show HEAD

# Look for changes in:
# - kernel/sched/sched.c (mailbox logic)
# - kernel/sched/mailbox.c (if exists)
# - kernel/proc/proc.c (candidate visibility)
# - Any epoch/candidate/owner logic
```

### Step 5: Targeted Analysis

Once regression commit is found, compare parent vs regression:

```bash
GOOD_COMMIT=$(git rev-parse HEAD^)  # parent (last good)
BAD_COMMIT=$(git rev-parse HEAD)    # regression commit

# Build both and extract mailbox stats
for commit in ${GOOD_COMMIT} ${BAD_COMMIT}; do
    git checkout ${commit}
    make clean && make KERNEL_PROFILE=validation USER_MINIMAL_MODE=syscall-v2-runtime efi-img
    
    mkdir -p evidence/analysis-${commit}
    scripts/ci/gate_performance.sh --evidence-dir evidence/analysis-${commit}
    
    echo "=== Commit ${commit} ==="
    jq '.raw_metrics.mailbox_phase_breakdown_ticks.fallback_reasons' \
       evidence/analysis-${commit}/actual.lock.json
done
```

Expected difference:
- **GOOD commit:** `no_candidate: 0-5` (normal scheduling)
- **BAD commit:** `no_candidate: 61` (always fallback)

## Suspected Code Areas

Based on mailbox fallback pattern, likely changes in:

### 1. Epoch Logic
```c
// kernel/sched/sched.c
if (candidate_epoch <= owner_last_epoch) {
    // STALE - reject candidate
    // If this logic changed, all candidates might be rejected
}
```

### 2. Candidate Visibility
```c
// kernel/proc/proc.c
if (proc->state != PROC_STATE_READY) {
    // NOT SCHEDULABLE
    // If state machine changed, candidates might be invisible
}
```

### 3. Mailbox Extract
```c
// kernel/sched/sched.c - mailbox_extract_candidate()
// If snapshot/extract/validate chain broken:
// - Always returns NULL
// - Forces fallback every tick
```

### 4. Arbiter Decision
```c
// kernel/sched/sched.c - sched_arbiter_decision()
// If candidate acceptance logic changed:
// - Always rejects valid candidates
// - Forces fallback path
```

## Performance Impact Calculation

**Fallback path overhead:**
- Normal scheduling: candidate found → direct switch
- Fallback path: no candidate → fallback logic → idle/retry

**Measured impact:**
- Boot: +14% (10684 → 12197ms)
- Context switch: +15% (175.08 → 201.93ms)
- Syscall: +15% (175.08 → 201.93ms)

**Root cause:** 61/61 fallback = every tick pays fallback overhead

## Expected Resolution

Once regression commit is identified:

### Option A: Revert the commit
```bash
git revert <bad_commit>
# Test: fallback should drop to 0-5
# Performance should return to baseline
```

### Option B: Fix the logic
```bash
# Identify broken condition in mailbox/epoch/candidate logic
# Fix the condition
# Test: fallback should drop to 0-5
# Performance should return to baseline
```

### Option C: Accept with justification
```bash
# If fallback is intentional (e.g., new safety check)
# Document why 61/61 fallback is acceptable
# Update baseline via authorized workflow
```

## Bisect Script Details

**Thresholds:**
- `BOOT_TIME_THRESHOLD=11000` (baseline 10684 + 3%)
- `CONTEXT_SWITCH_THRESHOLD=184` (baseline 175.08 + 5%)
- `SYSCALL_THRESHOLD=184` (baseline 175.08 + 5%)

**Exit codes:**
- `0`: GOOD (performance acceptable)
- `1`: BAD (performance regressed)
- `125`: SKIP (build failed or test inconclusive)

**Mailbox stats extraction:**
```bash
jq '.raw_metrics.mailbox_phase_breakdown_ticks.fallback_reasons.no_candidate' \
   evidence/bisect-<commit>/actual.lock.json
```

## Timeline Estimate

- Bisect: ~7 steps × 15 min = ~2 hours (GitHub CI)
- Analysis: 30 min (compare good vs bad commit)
- Fix/revert: 30 min
- Verification: 15 min
- **Total: ~3 hours to resolution**

## Critical Rules

- ✅ Run bisect in GitHub CI (authoritative environment)
- ✅ Focus on mailbox fallback stats
- ✅ Compare parent vs regression commit
- ✅ Look for epoch/candidate/visibility changes
- ❌ Don't update baseline before understanding root cause
- ❌ Don't accept 61/61 fallback without justification

## Next Steps After Bisect

1. Identify regression commit
2. Analyze mailbox/scheduler changes
3. Understand why candidates are rejected
4. Fix logic OR revert commit OR justify fallback
5. Verify fallback drops to 0-5
6. Verify performance returns to baseline
7. Update baseline via authorized workflow (if needed)

## Key Insight

**This is not a feature overhead problem.**
**This is a scheduler logic bug causing fallback path abuse.**

Fix the logic → performance returns to baseline.

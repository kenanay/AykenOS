# CI Reproduction Checklist - 30 Second Diagnosis

## Objective

Verify if performance regression is due to environment drift or code regression.

## Test Setup

**Commit:** `9b3358e6` (current HEAD)  
**Previous Run:** 24535837417 (failed with ~19% regression)  
**New Run:** [TO BE FILLED]

## Hypothesis

**Environment Drift:** GitHub Actions runner image changed from 80.1 to 86.1, causing uniform ~19% performance regression.

## Verification Steps

### Step 1: Trigger Fresh CI Run

```bash
# Push current HEAD to trigger CI
git push origin phase14-closure-final

# Or create empty commit to retrigger
git commit --allow-empty -m "ci: reproduce performance regression"
git push origin phase14-closure-final
```

### Step 2: Wait for CI Completion

Monitor: https://github.com/kenanay/AykenOS/actions

### Step 3: Extract Evidence (30 seconds)

Once CI completes, run:

```bash
# Get run ID
RUN_ID="[NEW_RUN_ID]"

# Extract key metrics
gh run view ${RUN_ID} --log | grep -A 50 "== CI GATE PERFORMANCE ==" | grep -E "boot_time_ms|context_switch|syscall_latency|ci_image_digest|env_hash"
```

### Step 4: Compare Metrics (30 seconds)

Fill this table:

| Metric | Run 1 (24535837417) | Run 2 (NEW) | Delta | Status |
|--------|---------------------|-------------|-------|--------|
| ci_image_digest | gha-ubuntu24-20260413.86.1-X64 | ??? | ??? | ??? |
| env_hash | edbe5bed... | ??? | ??? | ??? |
| boot_time_ms | 12713 | ??? | ??? | ??? |
| context_switch | 208.21 | ??? | ??? | ??? |
| syscall | 208.21 | ??? | ??? | ??? |

### Step 5: Decision Matrix (30 seconds)

#### Scenario A: Metrics Match (Environment Drift CONFIRMED)

**Criteria:**
- ✅ ci_image_digest: SAME (both 86.1)
- ✅ env_hash: SAME
- ✅ boot_time_ms: Within ±5% (12100-13300ms)
- ✅ context_switch: Within ±5% (197-218ms)
- ✅ syscall: Within ±5% (197-218ms)

**Conclusion:** Environment drift is root cause.

**Action:**
```bash
# Update baseline for new environment
git checkout -b fix/baseline-update-runner-86.1
# Trigger baseline update workflow
git push origin fix/baseline-update-runner-86.1
# Add label: baseline-update
```

**Commit Message:**
```
perf(baseline): update for runner image gha-ubuntu24-20260413.86.1-X64

Root cause: GitHub Actions runner image updated from 80.1 to 86.1
Impact: ~19% uniform performance regression across all metrics
Evidence: Reproduced in runs 24535837417 and [NEW_RUN_ID]

Metrics (new baseline):
- boot_time_ms: ~12700 (was 10684)
- context_switch: ~208ms (was 175ms)
- syscall: ~208ms (was 175ms)

Decision: Accept environment drift, establish new baseline
Authority: Kenan AY (architectural steward)
```

---

#### Scenario B: Metrics Vary (Nondeterminism DETECTED)

**Criteria:**
- ✅ ci_image_digest: SAME (both 86.1)
- ✅ env_hash: SAME
- ❌ boot_time_ms: Varies >10% (e.g., 11000 vs 13000)
- ❌ context_switch: Varies >10%
- ❌ syscall: Varies >10%

**Conclusion:** Measurement has high variance (nondeterminism).

**Action:**
```bash
# Run statistical analysis
# Need 10+ samples to establish confidence intervals
for i in {1..10}; do
  git commit --allow-empty -m "ci: sample $i for statistical analysis"
  git push origin phase14-closure-final
done

# Collect all measurements
# Calculate mean, stddev, confidence intervals
# Determine if regression is statistically significant
```

**Next Steps:**
1. Collect 10 samples
2. Calculate statistics
3. If variance is high → investigate nondeterminism
4. If variance is low → use mean as new baseline

---

#### Scenario C: Metrics Improved (Code Regression FIXED)

**Criteria:**
- ✅ ci_image_digest: SAME (both 86.1)
- ✅ env_hash: SAME
- ✅ boot_time_ms: Back to baseline (~10700ms)
- ✅ context_switch: Back to baseline (~175ms)
- ✅ syscall: Back to baseline (~175ms)

**Conclusion:** Previous run was anomaly, or code was fixed.

**Action:**
```bash
# Verify baseline is still valid
# No action needed if metrics match baseline
# CI should pass
```

**Investigation:**
- Why did previous run show regression?
- Was it a transient issue?
- Check GitHub Actions status page for incidents

---

#### Scenario D: Metrics Worse (Code Regression CONFIRMED)

**Criteria:**
- ✅ ci_image_digest: SAME (both 86.1)
- ✅ env_hash: SAME
- ❌ boot_time_ms: Worse than Run 1 (>13300ms)
- ❌ context_switch: Worse than Run 1 (>218ms)
- ❌ syscall: Worse than Run 1 (>218ms)

**Conclusion:** Code regression exists AND environment drifted.

**Action:**
```bash
# Bisect to find code regression
git bisect start
git bisect bad HEAD
git bisect good 050332220d9a

# But use NEW environment baseline for comparison
# Modify bisect script to use ~12700ms as threshold
git bisect run scripts/ci/bisect_with_new_baseline.sh
```

**Next Steps:**
1. Find regression commit
2. Fix code regression
3. Then update baseline for environment drift

---

## Quick Reference

### Environment Drift Indicators

✅ **Strong Evidence:**
- ci_image_digest changed
- env_hash unchanged
- Uniform regression across all metrics (~same %)
- Reproducible in multiple runs

❌ **Weak Evidence:**
- Only one measurement
- High variance between runs
- Non-uniform regression (some metrics up, some down)

### Code Regression Indicators

✅ **Strong Evidence:**
- ci_image_digest unchanged
- env_hash unchanged
- Specific metric regressed (not uniform)
- Bisect identifies specific commit

❌ **Weak Evidence:**
- Uniform regression (suggests system-wide issue)
- Cannot reproduce
- Bisect points to baseline update commit

## Expected Outcome

**Most Likely:** Scenario A (Environment Drift Confirmed)

**Evidence:**
- ci_image_digest already confirmed different
- Uniform ~19% regression pattern
- Bisect showed all commits slow
- 61/61 mailbox pattern in baseline

**Confidence:** 90%

**Timeline:**
- Fresh CI run: 2-3 minutes
- Evidence extraction: 30 seconds
- Decision: 30 seconds
- Baseline update PR: 5 minutes
- **Total: ~10 minutes**

## Fallback Plan

If Scenario B (high variance) occurs:

1. Run 10 samples (30 minutes)
2. Calculate statistics
3. If stddev > 10% of mean → investigate nondeterminism
4. If stddev < 10% of mean → use mean as baseline

## Critical Rules

1. ✅ **DO** reproduce before updating baseline
2. ✅ **DO** document evidence in commit message
3. ✅ **DO** get architectural authority approval (Kenan AY)
4. ❌ **DON'T** update baseline without reproduction
5. ❌ **DON'T** assume environment drift without verification
6. ❌ **DON'T** ignore high variance (nondeterminism is a bug)

## Success Criteria

**Baseline Update is Valid if:**
- ✅ Reproduced in 2+ runs
- ✅ Variance < 10%
- ✅ Environment change documented
- ✅ Architectural authority approved
- ✅ Commit message includes evidence

**Baseline Update is Invalid if:**
- ❌ Only one measurement
- ❌ High variance (>10%)
- ❌ No environment change
- ❌ No architectural authority approval


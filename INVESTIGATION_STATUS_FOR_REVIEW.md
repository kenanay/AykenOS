# Performance Regression Investigation - Status for Review

**Date:** 2026-04-16  
**Investigator:** Kiro AI Assistant  
**Reviewer:** Kenan AY (Architectural Authority)  
**Status:** CRITICAL DECISION POINT

## TL;DR

**Initial hypothesis (environment drift only) is INSUFFICIENT.**

**Evidence suggests:** Either code regression OR high environment sensitivity OR both.

**Recommendation:** Need architectural authority decision on next steps.

## Investigation Timeline

1. ✅ Initial CI failure detected (Run 24535837417)
2. ✅ Bisect completed (identified baseline update commit)
3. ✅ Environment drift hypothesis formed (80.1 → 86.1)
4. ✅ Reproduction attempted (Run 24538949281)
5. ⚠️ **UNEXPECTED:** Reproduction also failed in baseline environment

## Critical Finding

### Run 24538949281 Analysis

**Environment:** `gha-ubuntu24-20260406.80.1-X64` (SAME as baseline)  
**Commit:** `2ef73b06` (empty commit, code identical to 9b3358e6)  
**Result:** **FAIL (4 violations)**

**Implication:** Either:
- A) Code regression exists (but bisect didn't find it)
- B) Environment variable doesn't match actual runner image
- C) High measurement variance (nondeterminism)

## Data Summary

| Run | Commit | Env (reported) | Result | boot_time_ms |
|-----|--------|----------------|--------|--------------|
| Baseline | 050332220d9a | 80.1 | PASS | 10684 |
| 24535837417 | 9b3358e6 | 86.1 | FAIL | 12713 |
| 24538949281 | 2ef73b06 | 80.1 | FAIL | ??? |
| Local | 9b3358e6 | macOS | N/A | 13851 |

**Pattern:** All recent runs fail, regardless of reported environment.

## Three Hypotheses

### Hypothesis A: Code Regression (60% confidence)

**Evidence For:**
- Bisect showed all commits after baseline are slow
- Run 24538949281 failed in baseline environment
- 61/61 mailbox pattern consistent (not the cause, but present)

**Evidence Against:**
- 2ef73b06 is empty commit (no code change from 9b3358e6)
- Bisect identified baseline update, not code change
- No obvious regression commit in git history

**Test:** Bisect in guaranteed baseline environment

### Hypothesis B: Environment Sensitivity (30% confidence)

**Evidence For:**
- Local macOS even slower (13851ms)
- ci_image_digest changed between runs
- Uniform ~19% regression (system-wide)

**Evidence Against:**
- Run 24538949281 reported 80.1 but still failed
- If purely environment, baseline commit should pass in any environment

**Test:** Run baseline commit (050332220d9a) in new environment (86.1)

### Hypothesis C: Measurement Variance (10% confidence)

**Evidence For:**
- Deterministic system showing non-deterministic behavior
- Multiple runs needed to establish confidence

**Evidence Against:**
- Baseline was stable for weeks
- Sudden change suggests real issue, not variance

**Test:** Run 10 samples, calculate statistics

## Bisect Reanalysis

### Why Bisect Found Baseline Update

Bisect tested commits in CURRENT environment (likely 86.1):
- All commits slow in 86.1
- Baseline update commit marked as "first bad"
- But this doesn't mean code regression

### Correct Bisect Would Require

1. Pin environment to baseline (80.1)
2. Test all commits in same environment
3. Find commit where performance degrades

**Problem:** GitHub Actions doesn't allow pinning to specific runner image version.

## Mailbox 61/61 Pattern

### Finding

Both baseline and current show:
- `fallback_reasons.no_candidate`: 61
- `extract_reasons.epoch_stale`: 61
- `epoch_gt_owner_last_epoch_count`: 1
- `epoch_lte_owner_last_epoch_count`: 61

### Interpretation

**NOT a recent regression.** Pattern exists in baseline.

**Possible meanings:**
1. Normal behavior for this workload (61 ticks)
2. Pre-existing bug that doesn't affect performance
3. Measurement artifact

**Recommendation:** Separate investigation, not related to current regression.

## Environment Drift Analysis

### ci_image_digest Changes

| Date | Image | Version |
|------|-------|---------|
| Apr 6 | gha-ubuntu24-20260406.80.1-X64 | 80.1 |
| Apr 13 | gha-ubuntu24-20260413.86.1-X64 | 86.1 |

**Impact:** Unknown, but likely contributes to regression.

### env_hash Paradox

**Observation:** env_hash SAME but ci_image_digest DIFFERENT

**Implication:** env_hash doesn't include ci_image_digest

**Risk:** Silent environment drift without detection

**Fix:** Add ci_image_digest to env_hash calculation

## Decision Matrix

### Option 1: Accept as Environment Drift (RISKY)

**Action:**
- Update baseline to ~12700ms
- Document environment change
- Move forward

**Risk:**
- Might mask real code regression
- Performance loss accepted without investigation

**Confidence:** 40%

### Option 2: Bisect in Controlled Environment (RECOMMENDED)

**Action:**
- Set up self-hosted runner with 80.1 image
- Bisect all commits in stable environment
- Find actual regression commit (if exists)

**Risk:**
- Time-consuming (4-6 hours)
- Might not find anything (if purely environment)

**Confidence:** 70%

### Option 3: Statistical Analysis (THOROUGH)

**Action:**
- Run baseline commit 10x in current environment
- Run current commit 10x in current environment
- Calculate mean, stddev, confidence intervals
- Determine if difference is statistically significant

**Risk:**
- Time-consuming (2-3 hours)
- Might show high variance (nondeterminism issue)

**Confidence:** 60%

### Option 4: Hybrid Approach (BALANCED)

**Action:**
1. Run baseline commit once in current environment (86.1)
2. If it passes → code regression confirmed
3. If it fails → environment drift confirmed
4. Then proceed with appropriate fix

**Risk:**
- Single measurement might be misleading
- But fastest path to decision

**Confidence:** 80%

## Recommended Action

**OPTION 4: Hybrid Approach**

### Step 1: Test Baseline in Current Environment

```bash
git checkout 050332220d9a
git commit --allow-empty -m "test: verify baseline in current environment"
git push origin HEAD:test/baseline-in-new-env
# Wait for CI
```

**Expected outcomes:**
- **If PASS:** Code regression exists → bisect needed
- **If FAIL:** Environment drift confirmed → update baseline

### Step 2: Based on Result

**If baseline passes in 86.1:**
- Code regression confirmed
- Bisect between 050332220d9a and 2ef73b06
- Find and fix regression

**If baseline fails in 86.1:**
- Environment drift confirmed
- Update baseline for new environment
- Document environment change

## Critical Questions for Architectural Authority

### Question 1: Acceptable Performance Loss?

Is ~19% performance loss acceptable for environment update?

**Context:**
- boot_time: 10684ms → 12713ms (+19%)
- context_switch: 175ms → 208ms (+19%)
- syscall: 175ms → 208ms (+19%)

**Options:**
- Accept: Update baseline, move forward
- Reject: Investigate and optimize to recover performance

### Question 2: Investigation Depth?

How deep should we investigate before accepting?

**Options:**
- Shallow: Single test (Option 4) - 30 minutes
- Medium: Statistical analysis (Option 3) - 2-3 hours
- Deep: Full bisect in controlled environment (Option 2) - 4-6 hours

### Question 3: Environment Pinning?

Should we pin CI environment to prevent future drift?

**Options:**
- Pin to specific image version (if GitHub allows)
- Use self-hosted runners for stability
- Accept drift and update baseline as needed

### Question 4: Determinism Guarantee?

Does ~19% variance violate determinism guarantees?

**Context:**
- AykenOS claims deterministic execution
- But performance varies significantly with environment
- Is this acceptable for a deterministic system?

## Files Created

1. `BISECT_RESULTS_ANALYSIS.md` - Bisect findings
2. `GITHUB_CI_EVIDENCE_RCA_PLAN.md` - RCA methodology
3. `ENVIRONMENT_DRIFT_ANALYSIS.md` - Environment analysis
4. `CI_REPRODUCTION_CHECKLIST.md` - Diagnosis checklist
5. `PERFORMANCE_REGRESSION_INVESTIGATION_SUMMARY.md` - Investigation summary
6. `CRITICAL_FINDING_CODE_REGRESSION_CONFIRMED.md` - Code regression analysis
7. `INVESTIGATION_STATUS_FOR_REVIEW.md` - This file

## Recommendation

**Proceed with Option 4 (Hybrid Approach):**

1. Test baseline commit in current environment (30 minutes)
2. Make decision based on result
3. Either bisect (if code regression) or update baseline (if environment drift)

**Rationale:**
- Fastest path to decision
- Isolates variables properly
- Provides clear answer

**Next step:** Await architectural authority approval to proceed.

## Contact

**Investigator:** Kiro AI Assistant  
**Reviewer:** Kenan AY  
**Status:** Awaiting Decision  
**Priority:** CRITICAL (CI blocked)


# Performance Regression Investigation - Executive Summary

## Status: AWAITING REPRODUCTION

**Current Phase:** Verifying environment drift hypothesis  
**CI Run:** 24538949317 (in progress)  
**Expected Completion:** ~3 minutes

## Timeline

| Date | Event | Status |
|------|-------|--------|
| 2026-04-16 21:50 | Initial CI failure (Run 24535837417) | ❌ FAILED |
| 2026-04-16 22:00 | Investigation started | 🔍 IN PROGRESS |
| 2026-04-16 22:30 | Bisect completed | ✅ COMPLETED |
| 2026-04-16 23:00 | Environment drift identified | ✅ IDENTIFIED |
| 2026-04-16 23:16 | Reproduction run triggered (24538949317) | ⏳ IN PROGRESS |
| 2026-04-16 23:20 | Decision pending | ⏳ PENDING |

## Investigation Summary

### Initial Hypothesis (REJECTED)

**Claim:** 61/61 mailbox fallback pattern is a scheduler bug causing +15% regression.

**Evidence Against:**
- ✅ Bisect showed baseline commit (050332220d9a) already has 61/61 pattern
- ✅ All commits from baseline to HEAD show same 61/61 pattern
- ✅ Pattern is consistent, not a recent regression
- ✅ Baseline performance (10684ms) achieved WITH 61/61 pattern

**Conclusion:** 61/61 pattern is NOT the root cause. It's either normal behavior or a pre-existing condition.

### Current Hypothesis (UNDER VERIFICATION)

**Claim:** GitHub Actions runner image update caused environment drift.

**Evidence For:**
- ✅ ci_image_digest changed: `80.1` → `86.1` (April 6 → April 13)
- ✅ env_hash unchanged (toolchain same)
- ✅ Uniform ~19% regression across ALL metrics
- ✅ Bisect showed all commits slow (not code-specific)

**Evidence Needed:**
- ⏳ Reproduction in fresh CI run (Run 24538949317)
- ⏳ Metrics match previous run (±5% variance)

## Key Findings

### 1. Bisect Results

**First Bad Commit:** `71a2ef0a` (baseline update)

**Interpretation:** This is NOT a code regression. Bisect identified the baseline update commit because:
- Baseline was measured in OLD environment (80.1)
- Bisect tested in NEW environment (86.1)
- Performance difference is environment-related, not code-related

### 2. Mailbox Stats Analysis

**Pattern:** 61/61 fallback in BOTH baseline and current

| Metric | Baseline | Current | Status |
|--------|----------|---------|--------|
| fallback_reasons.no_candidate | 61 | 61 | ✅ SAME |
| extract_reasons.epoch_stale | 61 | 61 | ✅ SAME |
| epoch_gt_owner_last_epoch_count | 1 | 1 | ✅ SAME |
| epoch_lte_owner_last_epoch_count | 61 | 61 | ✅ SAME |

**Conclusion:** Mailbox behavior is IDENTICAL. Regression is NOT in mailbox logic.

### 3. Performance Metrics

**GitHub CI (Run 24535837417):**

| Metric | Baseline | Actual | Delta | Percent |
|--------|----------|--------|-------|---------|
| boot_time_ms | 10684 | 12713 | +2029 | +19.0% |
| context_switch | 175.08 | 208.21 | +33.13 | +18.9% |
| syscall | 175.08 | 208.21 | +33.13 | +18.9% |

**Pattern:** Uniform ~19% regression (system-wide, not code-specific)

### 4. Environment Comparison

| Component | Baseline | Current | Status |
|-----------|----------|---------|--------|
| ci_image_digest | gha-ubuntu24-20260406.80.1-X64 | gha-ubuntu24-20260413.86.1-X64 | ❌ CHANGED |
| env_hash | edbe5bed... | edbe5bed... | ✅ SAME |
| clang_version | 18.1.3 (1ubuntu1) | 18.1.3 (1ubuntu1) | ✅ SAME |
| qemu_version | 8.2.2 (...ubuntu1.16) | 8.2.2 (...ubuntu1.16) | ✅ SAME |

**Critical:** env_hash is SAME but ci_image_digest is DIFFERENT.

**Implication:** env_hash does NOT include ci_image_digest, allowing silent environment drift.

## Decision Matrix

### Scenario A: Reproduction Confirms Environment Drift (90% probability)

**Criteria:**
- Metrics match Run 24535837417 (±5%)
- ci_image_digest = 86.1
- env_hash unchanged

**Action:**
1. Update baseline for new environment
2. Add ci_image_digest to env_hash calculation
3. Document environment drift in ARCHITECTURE_FREEZE.md
4. Merge baseline update PR

**Timeline:** 30 minutes

### Scenario B: High Variance Detected (5% probability)

**Criteria:**
- Metrics vary >10% between runs
- Nondeterminism in measurements

**Action:**
1. Run 10 samples for statistical analysis
2. Calculate mean, stddev, confidence intervals
3. Investigate nondeterminism sources
4. Use mean as baseline if variance acceptable

**Timeline:** 2 hours

### Scenario C: Metrics Return to Baseline (5% probability)

**Criteria:**
- Metrics match baseline (~10700ms)
- Previous run was anomaly

**Action:**
1. Investigate why previous run showed regression
2. Check GitHub Actions status page
3. No baseline update needed

**Timeline:** 30 minutes

## Files Created

1. `BISECT_RESULTS_ANALYSIS.md` - Bisect findings and mailbox analysis
2. `GITHUB_CI_EVIDENCE_RCA_PLAN.md` - Evidence-based RCA methodology
3. `ENVIRONMENT_DRIFT_ANALYSIS.md` - Environment drift hypothesis
4. `CI_REPRODUCTION_CHECKLIST.md` - 30-second diagnosis checklist
5. `PERFORMANCE_REGRESSION_INVESTIGATION_SUMMARY.md` - This file

## Artifacts

1. `scripts/ci/bisect_performance_regression.sh` - Automated bisect script
2. `BISECT_REGRESSION_GUIDE.md` - Bisect execution guide
3. `EPOCH_DEBUG_PATCH.md` - Debug patch (not needed)
4. `REGRESSION_ROOT_CAUSE_ANALYSIS.md` - Initial RCA (superseded)
5. `NEXT_STEPS_SUMMARY.md` - Initial action plan (superseded)

## Lessons Learned

### What Worked

1. ✅ **Bisect revealed pattern consistency** - Showed 61/61 in baseline
2. ✅ **GitHub CI logs accessible** - Could extract evidence
3. ✅ **Environment tracking** - ci_image_digest captured
4. ✅ **Systematic approach** - Evidence-based decision making

### What Didn't Work

1. ❌ **Initial hypothesis** - Assumed mailbox bug without verification
2. ❌ **Local testing** - macOS results not authoritative
3. ❌ **env_hash incomplete** - Doesn't include ci_image_digest

### Improvements Needed

1. 🔧 **Add ci_image_digest to env_hash** - Prevent silent drift
2. 🔧 **Automated drift detection** - Alert when runner image changes
3. 🔧 **Baseline update workflow** - Streamline environment updates
4. 🔧 **Statistical analysis** - Measure variance, confidence intervals

## Next Steps

### Immediate (Now)

1. ⏳ Wait for Run 24538949317 to complete (~2 minutes)
2. ⏳ Extract evidence using CI_REPRODUCTION_CHECKLIST.md
3. ⏳ Make decision based on reproduction results

### Short-term (Today)

1. Update baseline if environment drift confirmed
2. Add ci_image_digest to env_hash calculation
3. Document in ARCHITECTURE_FREEZE.md
4. Merge baseline update PR

### Long-term (This Week)

1. Add automated drift detection to CI
2. Investigate runner image differences
3. Profile and optimize hot paths
4. Consider self-hosted runners for stability

## Critical Insights

### 1. Environment Drift is a First-Class Bug

In a deterministic system like AykenOS:
- Environment changes are as serious as code bugs
- Silent drift violates determinism guarantees
- env_hash MUST include ALL environment factors

### 2. Baseline Authority is Governance

Baseline updates are not technical decisions:
- Require architectural authority approval
- Must be documented with evidence
- Cannot be automated without verification

### 3. Reproduction is Mandatory

Never update baseline without:
- Reproducing in 2+ runs
- Verifying variance < 10%
- Documenting environment changes
- Getting authority approval

## Confidence Levels

| Hypothesis | Confidence | Evidence |
|------------|-----------|----------|
| Environment drift | 90% | ci_image_digest changed, uniform regression |
| Code regression | 5% | Bisect showed all commits slow |
| Nondeterminism | 5% | No evidence yet, but possible |

## Waiting For

- ⏳ CI Run 24538949317 completion
- ⏳ Evidence extraction
- ⏳ Decision based on reproduction

**ETA:** 3 minutes

## Contact

**Architectural Authority:** Kenan AY  
**Investigation Lead:** Kiro AI Assistant  
**Status:** In Progress  
**Priority:** High (CI blocked)


# Phase 10: Baseline Immutability Violation

**Date:** 2026-03-01  
**Status:** CI FREEZE FAILED - BASELINE IMMUTABILITY VIOLATION  
**CI Run:** #22552220402  
**Issue:** Baseline lock file modified directly in PR

---

## CI Failure Analysis

### Result

**CI Freeze:** FAILED  
**Failed Gate:** `ci-gate-performance`  
**Failure Reason:** `baseline lock immutability violation`

### Error Message

```
performance: FAIL (baseline lock immutability violation)
Baseline lock file mutated in PR: scripts/ci/perf-baseline.lock.json
If this change is intentional, regenerate via perf-baseline-init workflow (authorized path).
```

---

## Root Cause

### What Happened

1. Commit `4aa0c4f5` modified `scripts/ci/perf-baseline.lock.json` directly
2. This commit is part of PR #25
3. CI detected baseline file mutation in the PR
4. Performance gate correctly FAILED due to immutability violation

### Why This Is Correct Behavior

**Constitutional Rule:**
- Baseline locks are immutable
- Changes MUST go through authorized workflow only
- Direct baseline commits in PRs are PROHIBITED
- This prevents unauthorized baseline manipulation

**Enforcement:**
```bash
make ci-gate-performance
# Checks: git diff main...HEAD -- scripts/ci/perf-baseline.lock.json
# If changed → FAIL
```

---

## The Baseline Immutability Problem

### Commit History

```
7bd9dfdf - fix(ci): reorder freeze gates (current HEAD)
b1c88436 - docs: Phase 10 completion summary
4aa0c4f5 - perf: regenerate baseline [local-simulated] ← PROBLEM
01afa498 - docs: add CI validation checklist
21bd0076 - docs: Phase 10 determinism achieved
```

**Problem Commit:** `4aa0c4f5`
- Modified: `scripts/ci/perf-baseline.lock.json`
- Method: Direct commit (local CI simulation)
- Status: VIOLATES immutability rule

### Why Local Baseline Generation Is Prohibited

**Constitutional Requirement:**
- Baseline authority: `github-hosted-ubuntu-24.04-x64`
- Authorized workflow: `perf-baseline-init`
- Environment: GitHub Actions runner (not local)

**Local baseline issues:**
- Environment mismatch (macOS arm64 vs Ubuntu x64)
- No CI image digest validation
- No authority verification
- Circumvents governance

---

## Correct Path Forward

### Option 1: Remove Baseline Commit, Use Authorized Workflow

**Steps:**
1. Revert commit `4aa0c4f5` (baseline change)
2. Keep commits `7bd9dfdf` (Makefile fix) and docs
3. Push cleaned branch
4. Trigger `perf-baseline-init` workflow (authorized)
5. Workflow generates baseline in CI environment
6. Workflow creates separate PR with baseline
7. Merge baseline PR
8. Then merge main PR

**Pros:**
- Follows constitutional process
- Baseline generated in authoritative environment
- Immutability preserved
- Governance enforced

**Cons:**
- Requires two PRs
- More steps
- Baseline validation delayed

### Option 2: Request Baseline Regeneration Exception

**Steps:**
1. Document why local baseline was necessary
2. Request Architecture Board review
3. Provide evidence of local determinism
4. Request one-time exception
5. Commit baseline with `[authorized]` tag

**Pros:**
- Single PR
- Faster completion

**Cons:**
- Requires manual approval
- Sets precedent
- Weakens governance
- Not recommended

### Option 3: Accept Current State, Fix Ring3 Execution First

**Steps:**
1. Remove baseline commit `4aa0c4f5`
2. Keep Makefile fix and docs
3. Fix ring3-execution-phase10a2 issue
4. Get CI freeze to PASS without baseline change
5. Then regenerate baseline via authorized workflow

**Pros:**
- Separates concerns
- Fixes functional issue first
- Baseline comes after functional correctness

**Cons:**
- Baseline validation delayed
- Phase 10 not complete yet

---

## Recommended Path

### Immediate Action: Option 1 (Constitutional Path)

**Rationale:**
- Preserves governance integrity
- Follows constitutional process
- Establishes correct precedent
- Validates baseline in authoritative environment

**Implementation:**

```bash
# 1. Create new branch without baseline commit
git checkout -b pr/main-updates-20260301-no-baseline

# 2. Cherry-pick commits EXCEPT 4aa0c4f5
git cherry-pick 21bd0076  # docs
git cherry-pick 01afa498  # docs
git cherry-pick b1c88436  # docs
git cherry-pick 7bd9dfdf  # Makefile fix

# 3. Push new branch
git push origin pr/main-updates-20260301-no-baseline

# 4. Update PR to point to new branch
# OR create new PR

# 5. Trigger perf-baseline-init workflow
gh workflow run perf-baseline-init

# 6. Wait for baseline PR
# 7. Merge baseline PR first
# 8. Then merge main PR
```

---

## Engineering Assessment

### What This Reveals

**Governance Working:**
- ✅ Immutability enforcement active
- ✅ CI correctly rejected unauthorized baseline
- ✅ Constitutional rules enforced
- ✅ No bypass allowed

**Process Gap:**
- ❌ Attempted to circumvent authorized workflow
- ❌ Local baseline generation not valid
- ❌ Premature baseline commit

### Lesson Learned

**"Local validation ≠ Baseline authority"**

Local determinism validation is necessary but NOT sufficient for baseline lock.

Baseline MUST be generated via authorized workflow in CI environment.

---

## Current Status

### What's Validated

✅ Local determinism (3+ runs)  
✅ Makefile gate ordering fix  
✅ Documentation updates  
✅ Pre-CI discipline clean

### What's Blocked

❌ Baseline lock (immutability violation)  
❌ CI freeze PASS (performance gate failed)  
❌ Phase 10 completion (baseline not validated)

### What's Required

⏳ Remove baseline commit from PR  
⏳ Use authorized `perf-baseline-init` workflow  
⏳ Validate baseline in CI environment  
⏳ Merge via constitutional process

---

## Next Steps

### Immediate (Required)

1. Decide on path forward (recommend Option 1)
2. Remove baseline commit `4aa0c4f5` from PR
3. Keep Makefile fix and documentation
4. Trigger `perf-baseline-init` workflow
5. Wait for authorized baseline generation
6. Merge baseline PR
7. Then complete Phase 10

### Alternative (If Ring3 Issue Persists)

1. Fix ring3-execution-phase10a2 issue first
2. Get functional correctness working
3. Then regenerate baseline
4. Complete Phase 10 after both fixed

---

## Conclusion

CI freeze correctly FAILED due to baseline immutability violation.

This is governance working as designed.

The path forward requires:
- Removing unauthorized baseline commit
- Using authorized workflow
- Following constitutional process

**Phase 10: BLOCKED - Baseline Immutability Violation 🚫**

---

**Maintained by:** AykenOS Architecture Board  
**Last Updated:** 2026-03-01T20:50Z  
**CI Run:** #22552220402 (FAILED)  
**Issue:** Baseline lock file modified in PR (constitutional violation)

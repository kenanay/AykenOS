# PR Analysis and Merge Strategy

## Current PR Status

### PR #109: phase14-closure-final (MAIN PR)
**Status:** OPEN, awaiting merge  
**Base:** main  
**Commits:** 73  
**Created:** 2026-04-16 18:57 UTC

**Content:**
- Phase 14 closure work
- Phase 16 features (dual-worker, observability, validation)
- **Contains the performance regression** (fc22692d + 27136fec + others)

**Key commits:**
- fc22692d: Dual-worker infrastructure
- 27136fec: Ring3 observability probes
- 705de486: BCIB graph validation
- aefac050: Baseline update (later reverted in 9b3358e6)
- 2ef73b06: Current HEAD (performance regression present)

**Issue:** Performance gate will FAIL (+14% regression)

---

### PR #111: test/irq-validation-disabled (DIAGNOSTIC)
**Status:** OPEN, diagnostic only  
**Base:** main  
**Commits:** 64  
**Created:** 2026-04-17 00:13 UTC

**Content:**
- Based on fc22692d (dual-worker commit from PR #109)
- IRQ validation disabled (#if 0)
- RING0_EXPORT_MAX increased to 196
- **Purpose:** Confirm IRQ validation is the bottleneck

**Issue:** 
- Naming convention gate fails
- Performance gate never executes
- **Should NOT be merged** (diagnostic only)

---

### PR #112: test/t4-dual-worker (DIAGNOSTIC)
**Status:** OPEN, diagnostic only  
**Base:** main  
**Commits:** 62  
**Created:** 2026-04-17 00:13 UTC

**Content:**
- Tests fc22692d commit (dual-worker)
- **Purpose:** Measure dual-worker overhead

**Issue:**
- **Should NOT be merged** (diagnostic only)

---

### PR #108: diag/ring3-transition-trace (DIAGNOSTIC)
**Status:** OPEN  
**Base:** main  
**Content:** Diagnostic markers

**Issue:**
- **Should NOT be merged** (diagnostic only)

## Merge Strategy

### Current Situation

**Problem:** PR #109 (phase14-closure-final) has 14% performance regression

**Root cause:** IRQ handler validation (timer_isr_c → sched_mailbox_validate_ring3)

**Evidence:**
- Code inspection: Validation in IRQ hot path ✅
- Metrics: Uniform 14% regression ✅
- Phase 16: Validation became heavier ✅
- Confidence: 95%

### Option A: Fix PR #109, Then Merge (RECOMMENDED)

**Approach:** Add proper fix to PR #109 before merging

**Steps:**

1. **Create fix branch from phase14-closure-final:**
   ```bash
   git checkout phase14-closure-final
   git checkout -b fix/deferred-validation-phase14
   ```

2. **Implement deferred validation:**
   - Move validation out of IRQ handler
   - Add validation_pending flag to proc_t
   - Call validation from scheduler (not IRQ)

3. **Test fix:**
   ```bash
   # Build and test
   make clean && make
   make ci-pre-ci
   
   # Push to CI
   git push origin fix/deferred-validation-phase14
   
   # Wait for performance gate: PASS
   ```

4. **Merge fix into phase14-closure-final:**
   ```bash
   git checkout phase14-closure-final
   git merge fix/deferred-validation-phase14
   git push origin phase14-closure-final
   ```

5. **Merge phase14-closure-final to main:**
   ```bash
   # After CI passes
   gh pr merge 109
   ```

**Timeline:** 4-6 hours

**Pros:**
- Clean history
- Performance regression fixed before merge
- All Phase 14 + fix in one PR

**Cons:**
- Delays Phase 14 merge

---

### Option B: Merge PR #109, Fix in Separate PR

**Approach:** Merge Phase 14 with regression, fix immediately after

**Steps:**

1. **Merge PR #109 to main** (with performance regression)

2. **Create fix PR:**
   ```bash
   git checkout main
   git pull
   git checkout -b fix/deferred-validation
   # Implement fix
   git push origin fix/deferred-validation
   ```

3. **Merge fix PR**

**Timeline:** 5-7 hours (merge + fix + merge)

**Pros:**
- Phase 14 work lands immediately
- Fix is separate, easier to review

**Cons:**
- Main branch has regression temporarily
- Two PRs instead of one
- Bisect history shows regression

---

### Option C: Revert Problematic Commits, Merge, Re-add with Fix

**Approach:** Remove regression-causing commits from PR #109, merge, then add back with fix

**Steps:**

1. **Revert in phase14-closure-final:**
   ```bash
   git checkout phase14-closure-final
   git revert fc22692d 27136fec 705de486  # Revert Phase 16 features
   git push origin phase14-closure-final
   ```

2. **Merge PR #109** (without Phase 16 features)

3. **Re-add Phase 16 with fix:**
   ```bash
   git checkout main
   git pull
   git checkout -b phase16-with-deferred-validation
   git cherry-pick fc22692d 27136fec 705de486
   # Add deferred validation fix
   git push origin phase16-with-deferred-validation
   ```

**Timeline:** 6-8 hours

**Pros:**
- Main never has regression
- Clean separation of Phase 14 and Phase 16

**Cons:**
- Complex git history
- More work
- Loses Phase 16 features temporarily

---

## Recommended Strategy: Option A

**Rationale:**

1. **Cleanest approach:** Fix before merge
2. **No regression in main:** Performance always good
3. **Single PR:** All Phase 14 + fix together
4. **Fastest to stable state:** 4-6 hours

**Implementation Plan:**

### Step 1: Create Fix Branch (30 minutes)

```bash
git checkout phase14-closure-final
git checkout -b fix/deferred-validation-phase14
```

### Step 2: Implement Deferred Validation (3 hours)

**File 1: kernel/include/proc.h**
```c
typedef struct proc {
    // ... existing fields ...
    uint8_t validation_pending;  // NEW: deferred validation flag
} proc_t;
```

**File 2: kernel/arch/x86_64/timer.c**
```c
// Line 227: Change from immediate validation to deferred
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
   ((defined(AYKEN_SCHED_BOOTSTRAP_POLICY) && (AYKEN_SCHED_BOOTSTRAP_POLICY == 1)) || \
    (defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)))
    // Defer validation (don't validate in IRQ)
    current_proc->validation_pending = 1;
#endif
```

**File 3: kernel/sched/sched.c**
```c
// Add validation before scheduler decision
void sched_schedule(void)  // Or appropriate scheduler entry point
{
    if (current_proc && current_proc->validation_pending) {
        sched_mailbox_validate_ring3(current_proc);
        current_proc->validation_pending = 0;
    }
    // ... existing scheduler logic ...
}
```

### Step 3: Test Fix (1 hour)

```bash
# Build
make clean && make

# Local test
make ci-pre-ci

# Push to CI
git push origin fix/deferred-validation-phase14

# Wait for CI
gh run watch $(gh run list --branch fix/deferred-validation-phase14 --limit 1 --json databaseId --jq '.[0].databaseId')

# Expected: Performance gate PASS
```

### Step 4: Merge Fix into Phase14 (30 minutes)

```bash
git checkout phase14-closure-final
git merge fix/deferred-validation-phase14
git push origin phase14-closure-final

# Wait for CI on phase14-closure-final
# Expected: All gates PASS
```

### Step 5: Merge Phase14 to Main (30 minutes)

```bash
gh pr merge 109
```

### Step 6: Clean Up Test Branches

```bash
# Close diagnostic PRs
gh pr close 111  # test/irq-validation-disabled
gh pr close 112  # test/t4-dual-worker

# Delete branches
git push origin --delete test/irq-validation-disabled
git push origin --delete test/t4-dual-worker
```

## PR Dependencies and Conflicts

### No Direct Conflicts

All PRs are independent:
- PR #109: Phase 14 work
- PR #111, #112: Diagnostic branches (based on PR #109)
- PR #108: Separate diagnostic work

### Merge Order

**Correct order:**
1. Fix PR #109 (add deferred validation)
2. Merge PR #109 to main
3. Close PR #111, #112 (diagnostic, no longer needed)
4. Handle PR #108 separately (if needed)

**DO NOT:**
- Merge PR #111 or #112 (diagnostic only)
- Merge PR #109 without fix (introduces regression)

## Timeline Summary

| Action | Duration | Total |
|--------|----------|-------|
| Create fix branch | 30 min | 0.5h |
| Implement deferred validation | 3 hours | 3.5h |
| Test fix | 1 hour | 4.5h |
| Merge fix into Phase14 | 30 min | 5h |
| Merge Phase14 to main | 30 min | 5.5h |
| Clean up | 30 min | 6h |

**Total time to resolution:** 6 hours

## Next Steps

**Immediate (now):**
1. Create fix branch: `fix/deferred-validation-phase14`
2. Implement deferred validation
3. Test in CI

**After fix passes:**
1. Merge fix into phase14-closure-final
2. Merge phase14-closure-final to main
3. Close diagnostic PRs

**Do NOT:**
- Merge test/irq-validation-disabled
- Merge test/t4-dual-worker
- Merge phase14-closure-final without fix

---

**Status:** Ready to implement fix  
**Recommended action:** Option A (fix before merge)  
**ETA:** 6 hours to stable main branch

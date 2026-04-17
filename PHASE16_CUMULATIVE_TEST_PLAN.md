# Phase 16 Cumulative Overhead Test Plan

## Critical Insight

**This is NOT a single-commit regression.**  
**This is a PHASE regression - accumulated overhead from multiple features.**

Kenan AY's analysis (aefac050):
```
Root cause: Accumulated overhead from Phase 16 features
```

The word "accumulated" is key - this is cumulative cost, not a single bug.

## Test Strategy: Measure Cumulative Impact

Instead of finding "the regression commit", measure overhead contribution of each major feature.

### Test Points

| Test | Commit | Description | Expected Result |
|------|--------|-------------|-----------------|
| **T0** | 050332220d9a | Baseline (known good) | PASS (10684ms) |
| **T1** | 81bc4240^ | Before Phase 16 starts | PASS (~10700ms) |
| **T2** | 1dd6b95c | After boot observability | PASS or slight degrade |
| **T3** | 705de486 | After BCIB graph validation | Possible FAIL |
| **T4** | fc22692d | After dual-worker | Likely FAIL |
| **T5** | 27136fec | After Ring3 observability | Definite FAIL |
| **T6** | bc3c7b04 | Before baseline update attempt | FAIL (12197ms) |

### Expected Overhead Breakdown

Based on feature scope and Kenan's analysis:

| Feature | Commit | Estimated Overhead | Cumulative |
|---------|--------|-------------------|------------|
| Baseline | 050332220d9a | 0% | 10684ms |
| Boot observability | 1dd6b95c | +2% | 10898ms |
| BCIB graph validation | 705de486 | +4% | 11334ms |
| Dual-worker infrastructure | fc22692d | +5% | 11901ms |
| Ring3 observability probes | 27136fec | +3% | 12258ms |
| **Total Phase 16** | | **+14%** | **12197ms** |

**Threshold:** 11752ms (10684 * 1.10)

**Expected FAIL point:** Between fc22692d and 27136fec (when cumulative exceeds threshold)

## Test Execution Plan

### Phase 1: Confirm Phase 16 Boundary (20 minutes)

**Test T1: Before Phase 16**

```bash
# Test commit before Phase 16 work started
git log --oneline 050332220d9a..81bc4240 | tail -1
# Get parent of Phase 16 start
git checkout 81bc4240^

# Push to CI
git push -f origin HEAD:test/before-phase16

# Expected: PASS (confirms Phase 16 is the problem area)
```

**Decision:**
- If PASS → Phase 16 is the problem ✅
- If FAIL → regression is earlier (unlikely)

### Phase 2: Measure Dual-Worker Impact (20 minutes)

**Test T4: Dual-worker commit**

```bash
git checkout fc22692d
git push -f origin HEAD:test/dual-worker

# Expected: FAIL or borderline (cumulative ~11900ms)
```

**Decision:**
- If FAIL → dual-worker pushed over threshold
- If PASS → later features pushed over threshold

### Phase 3: Measure Observability Impact (20 minutes)

**Test T5: Ring3 observability**

```bash
git checkout 27136fec
git push -f origin HEAD:test/ring3-observability

# Expected: FAIL (cumulative ~12200ms)
```

**Decision:**
- Confirms observability adds final overhead

### Phase 4: Isolate Validation Impact (optional, 20 minutes)

**Test T3: BCIB graph validation**

```bash
git checkout 705de486
git push -f origin HEAD:test/bcib-validation

# Expected: PASS or borderline
```

**Decision:**
- Measures validation overhead contribution

## Analysis Framework

### Overhead Attribution

After tests complete, calculate per-feature overhead:

```python
# Example calculation
baseline = 10684  # T0
before_phase16 = 10700  # T1 (estimated)
after_validation = 11300  # T3 (estimated)
after_dual_worker = 11900  # T4 (estimated)
after_observability = 12200  # T5 (measured)

# Per-feature overhead
validation_overhead = after_validation - before_phase16  # ~600ms (5.6%)
dual_worker_overhead = after_dual_worker - after_validation  # ~600ms (5.3%)
observability_overhead = after_observability - after_dual_worker  # ~300ms (2.5%)

# Total Phase 16 overhead
total_overhead = after_observability - baseline  # ~1516ms (14.2%)
```

### Hot Path Analysis

From logs, we know the overhead is in:

```
timer_validate_irq → scheduler → mailbox → context_switch
```

**Evidence:**
- context_switch_latency: 175ms → 201ms (+15%)
- syscall_latency: 175ms → 201ms (+15%)
- boot_time: 10684ms → 12197ms (+14%)

**Interpretation:** Uniform regression across all metrics = system-wide overhead in hot path

**Likely culprits:**
1. **Timer IRQ path:** Additional validation/observability in IRQ handler
2. **Scheduler path:** Dual-worker decision logic
3. **Mailbox path:** BCIB graph validation on every consume
4. **Context switch:** Ring3 observability probes on transitions

## Remediation Strategy

### Option A: Selective Feature Flags (RECOMMENDED)

**Approach:** Make Phase 16 features configurable

```c
// kernel/config.h
#define AYKEN_DUAL_WORKER 1        // Can disable for prod
#define AYKEN_RING3_OBSERVABILITY 0  // Disable by default
#define AYKEN_BCIB_VALIDATION 1     // Keep (security critical)
#define AYKEN_BOOT_OBSERVABILITY 1  // Keep (diagnostic value)
```

**Impact:**
- Disable Ring3 observability: -3% (300ms)
- Disable dual-worker (if not needed): -5% (600ms)
- **Total savings: -8%** → boot_time ~11600ms (within threshold)

**Timeline:** 2 days
- Day 1: Add feature flags, test
- Day 2: Profile-specific configs, CI validation

### Option B: Optimize Hot Paths

**Approach:** Keep all features, reduce overhead

**Targets:**
1. **Timer IRQ path:** Reduce validation overhead
   - Cache validation results
   - Lazy validation (not every IRQ)
   - Optimize validation algorithm

2. **Scheduler path:** Optimize dual-worker logic
   - Fast path for single-worker case
   - Reduce decision complexity
   - Cache worker state

3. **Observability probes:** Reduce probe overhead
   - Conditional compilation
   - Sampling (not every transition)
   - Batch probe data

**Timeline:** 1 week
- Days 1-2: Profile and identify bottlenecks
- Days 3-5: Implement optimizations
- Days 6-7: Test and validate

**Expected savings:** 5-8% (600-900ms)

### Option C: Accept Regression, Update Baseline

**Approach:** Accept Phase 16 cost, update baseline

**Rationale:**
- Phase 16 features provide critical functionality
- Overhead is acceptable for feature value
- Optimization can be deferred

**Action:**
```bash
# Update baseline to bc3c7b04
git checkout bc3c7b04
./scripts/ci/perf-baseline-init.sh

# Document in commit message:
# - Phase 16 feature list
# - Performance cost per feature
# - Justification for acceptance
# - Future optimization plan
```

**Timeline:** 1 hour

**Risk:** Performance debt accumulates, future regressions harder to detect

### Option D: Hybrid Approach (BEST)

**Approach:** Quick wins + long-term optimization

**Phase 1 (This week):**
1. Disable Ring3 observability in prod profile (-3%)
2. Add fast path for single-worker scheduler (-2%)
3. **Total: -5%** → boot_time ~11600ms ✅ PASS

**Phase 2 (Next sprint):**
1. Profile hot paths systematically
2. Optimize timer IRQ validation
3. Optimize BCIB graph validation
4. **Target: -5%** → boot_time ~11000ms

**Phase 3 (Future):**
1. Architectural improvements
2. Self-hosted runners (controlled environment)
3. Performance regression prevention

**Timeline:**
- Week 1: Quick wins (5% reduction)
- Week 2-3: Systematic optimization (5% reduction)
- Month 2: Architectural improvements

## Decision Matrix

| Approach | Timeline | Risk | Performance Gain | Feature Impact |
|----------|----------|------|------------------|----------------|
| Feature Flags | 2 days | Low | 8% | Some features disabled |
| Optimize | 1 week | Medium | 5-8% | All features kept |
| Accept | 1 hour | Low | 0% | All features kept |
| Hybrid | 1 week + ongoing | Low | 10% | Minimal impact |

**Recommendation:** Hybrid approach
- Immediate: Feature flags for non-critical features
- Short-term: Optimize hot paths
- Long-term: Architectural improvements

## Test Commands

### Environment Check (CRITICAL)

```bash
# ONLY accept results from 80.1 environment
gh api repos/kenanay/AykenOS/actions/jobs/{JOB_ID}/logs 2>&1 | \
  grep "perf_ci_image_digest"

# Expected: gha-ubuntu24-20260406.80.1-X64
# If different: SKIP result
```

### Test Execution

```bash
# Test T1: Before Phase 16
git checkout $(git log --oneline 050332220d9a..81bc4240 | tail -1 | awk '{print $1}')^
git push -f origin HEAD:test/t1-before-phase16

# Test T4: Dual-worker
git checkout fc22692d
git push -f origin HEAD:test/t4-dual-worker

# Test T5: Ring3 observability
git checkout 27136fec
git push -f origin HEAD:test/t5-ring3-observability

# Monitor
gh run watch $(gh run list --branch test/t1-before-phase16 --limit 1 --json databaseId --jq '.[0].databaseId')
```

### Result Collection

```bash
# For each test, extract metrics
RUN_ID=24539434904
JOB_ID=$(gh api repos/kenanay/AykenOS/actions/runs/$RUN_ID/jobs --jq '.jobs[] | select(.name == "freeze") | .id')

# Check environment
gh api repos/kenanay/AykenOS/actions/jobs/$JOB_ID/logs 2>&1 | \
  grep "perf_ci_image_digest"

# Check result
gh api repos/kenanay/AykenOS/actions/runs/$RUN_ID/jobs --jq \
  '.jobs[] | select(.name == "freeze") | .conclusion'
```

## Timeline

**Test execution:** 1-2 hours (3-4 tests × 20 minutes)  
**Analysis:** 30 minutes  
**Decision:** 30 minutes  
**Implementation:** 2 days - 1 week (depending on approach)

**Total time to resolution:** 3-8 days

## Confidence Level

**Phase 16 is the cause:** 95% confidence  
**Cumulative overhead model:** 90% confidence  
**Overhead breakdown estimates:** 70% confidence (will be refined by tests)

## Next Action

**Execute Test T1 (before Phase 16):**

```bash
# Find commit before Phase 16
BEFORE_PHASE16=$(git log --oneline 050332220d9a..81bc4240 | tail -1 | awk '{print $1}')^

# Checkout and test
git checkout $BEFORE_PHASE16
git push -f origin HEAD:test/t1-before-phase16

# Monitor
gh run watch $(gh run list --branch test/t1-before-phase16 --limit 1 --json databaseId --jq '.[0].databaseId')
```

**Expected result:** PASS (confirms Phase 16 boundary)

---

**Status:** READY FOR CUMULATIVE TESTING  
**Model:** Accumulated overhead (not single commit)  
**Priority:** HIGH  
**Estimated resolution:** 3-8 days

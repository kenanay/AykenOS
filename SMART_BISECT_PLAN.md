# Smart Bisect Plan - Performance Regression

## Root Cause Already Identified

**Kenan AY's commit message (aefac050):**
```
Root cause: Accumulated overhead from Phase 16 features:
- Dual-worker infrastructure (BCIB + USER workers)
- Ring3 observability probes
- BCIB graph validation
- Enhanced boundary enforcement
```

**Regression magnitude:**
- boot_time_ms: 10684 → 12197 (+14%)
- context_switch_latency_ms_proxy: 175.08 → 201.93 (+15%)
- syscall_latency_ms_proxy: 175.08 → 201.93 (+15%)

## Top Suspect Commits

Based on commit messages and timing, these are the likely culprits:

### 1. fc22692d - Dual-worker infrastructure (PRIMARY SUSPECT)
```
Task 5: Add dual-worker infrastructure (BCIB + USER)
```
**Why suspect:** Adds entire dual-worker system, likely adds overhead to scheduler/context switch path

**Test priority:** HIGH

### 2. 27136fec - Ring3 observability probes (SECONDARY SUSPECT)
```
Enable Ring3 observability probes for dual-worker runtime validation
```
**Why suspect:** Observability probes in hot path (Ring3 transitions) add latency

**Test priority:** HIGH

### 3. 705de486 - BCIB graph validation (TERTIARY SUSPECT)
```
Task 5: Remove NULL workaround, implement real BCIB graph validation
```
**Why suspect:** Graph validation on every operation adds overhead

**Test priority:** MEDIUM

### 4. d9261b6b - BCIB context propagation
```
Task 5: Fix BCIB context propagation to boundary enforcement
```
**Why suspect:** Context propagation in syscall/boundary path

**Test priority:** MEDIUM

### 5. fbcc7464 - Scheduler and IRQ0 debug markers
```
Phase 3A: Add scheduler and IRQ0 debug markers
```
**Why suspect:** Debug markers in scheduler hot path

**Test priority:** LOW (likely small overhead)

## Smart Bisect Strategy

Instead of blind bisect (7 iterations), test suspects directly:

### Phase 1: Test Top 3 Suspects (3 CI runs)

```bash
# Test 1: Before dual-worker
git checkout fc22692d^  # Parent of dual-worker commit
# Push to CI, check performance
# Expected: PASS (if dual-worker is the culprit)

# Test 2: After dual-worker, before observability
git checkout 27136fec^  # Parent of observability commit
# Push to CI, check performance
# Expected: FAIL if dual-worker caused it, PASS if observability caused it

# Test 3: After observability, before graph validation
git checkout 705de486^  # Parent of graph validation commit
# Push to CI, check performance
# Expected: Narrows down to specific feature
```

### Phase 2: Binary Search Within Suspect Range (if needed)

If Phase 1 shows regression is gradual (multiple commits), bisect within that range.

### Phase 3: Detailed Analysis

Once regression commit found:
1. Review code changes
2. Identify hot path modifications
3. Profile specific functions
4. Determine if overhead is:
   - Necessary (feature cost)
   - Optimizable (implementation inefficiency)
   - Removable (debug code left in)

## Expected Outcomes

### Scenario A: Single Commit Regression

**Most likely:** fc22692d (dual-worker infrastructure)

**Evidence:**
- Largest feature addition
- Affects scheduler/context switch path
- Timing matches regression magnitude

**Action:** 
- Review dual-worker implementation
- Identify optimization opportunities
- Consider feature flags for optional overhead

### Scenario B: Accumulated Regression

**Possible:** Multiple commits each add 2-3% overhead

**Evidence:**
- Kenan's message says "accumulated overhead"
- Multiple Phase 16 features

**Action:**
- Profile each feature's contribution
- Optimize highest-impact features
- Consider architectural changes

### Scenario C: Debug Code Left In

**Possible:** Debug markers not properly gated

**Evidence:**
- fbcc7464 adds scheduler/IRQ0 debug markers
- b2c073b8 adds diagnostic markers (later reverted)

**Action:**
- Ensure debug code is behind feature flags
- Remove or optimize debug paths

## Bisect Commands

### Manual Bisect (if needed)

```bash
# Start bisect
git bisect start
git bisect bad bc3c7b04  # Last commit before baseline update
git bisect good 050332220d9a  # Known good baseline

# For each bisect step:
# 1. Push to test branch
git push -f origin HEAD:test/bisect-step

# 2. Wait for CI (check environment!)
gh run list --branch test/bisect-step --limit 1

# 3. Check performance gate
gh api repos/kenanay/AykenOS/actions/runs/{RUN_ID}/jobs \
  --jq '.jobs[] | select(.name == "freeze") | .conclusion'

# 4. Mark result
git bisect good  # if PASS
git bisect bad   # if FAIL
git bisect skip  # if wrong environment (not 80.1)
```

### Automated Bisect Script

```bash
#!/bin/bash
# bisect_smart.sh

GOOD="050332220d9a"
BAD="bc3c7b04"
ENV_REQUIRED="gha-ubuntu24-20260406.80.1-X64"

git bisect start "$BAD" "$GOOD"

while true; do
  COMMIT=$(git rev-parse HEAD)
  echo "Testing commit: $COMMIT"
  
  # Push to CI
  git push -f origin HEAD:test/bisect-step
  
  # Wait for run to start
  sleep 10
  
  # Get run ID
  RUN_ID=$(gh run list --branch test/bisect-step --limit 1 --json databaseId --jq '.[0].databaseId')
  
  # Wait for completion
  gh run watch "$RUN_ID"
  
  # Check environment
  ENV=$(gh api repos/kenanay/AykenOS/actions/jobs/$(gh api repos/kenanay/AykenOS/actions/runs/$RUN_ID/jobs --jq '.jobs[0].id')/logs 2>&1 | grep "perf_ci_image_digest" | awk '{print $2}')
  
  if [[ "$ENV" != "$ENV_REQUIRED" ]]; then
    echo "Wrong environment: $ENV (expected $ENV_REQUIRED)"
    git bisect skip
    continue
  fi
  
  # Check result
  RESULT=$(gh api repos/kenanay/AykenOS/actions/runs/$RUN_ID/jobs --jq '.jobs[] | select(.name == "freeze") | .conclusion')
  
  if [[ "$RESULT" == "success" ]]; then
    echo "PASS - marking good"
    git bisect good
  else
    echo "FAIL - marking bad"
    git bisect bad
  fi
  
  # Check if done
  if git bisect log | grep -q "is the first bad commit"; then
    echo "Bisect complete!"
    git bisect log
    break
  fi
done
```

## Timeline Estimate

### Smart approach (test suspects):
- 3 CI runs × 20 minutes = **60 minutes**
- Analysis: 30 minutes
- **Total: 1.5 hours**

### Full bisect:
- 7 CI runs × 20 minutes = **140 minutes**
- Analysis: 30 minutes
- **Total: 3 hours**

### Recommendation:
**Start with smart approach** - test fc22692d first. If that's the culprit (90% probability), you're done in 20 minutes.

## Critical Rules

### Environment Enforcement

**ONLY accept results from:**
```
perf_ci_image_digest: gha-ubuntu24-20260406.80.1-X64
```

**Any other environment = SKIP**

### Validation

Before marking good/bad, verify:
1. CI run completed successfully
2. Environment matches 80.1
3. Performance gate executed (not skipped)
4. Result is deterministic (not flaky)

## Next Steps

**Option 1: Smart Test (Recommended)**
```bash
# Test the primary suspect
git checkout fc22692d^
git push -f origin HEAD:test/suspect-dual-worker
# Wait for CI, check result
```

**Option 2: Full Bisect**
```bash
git bisect start bc3c7b04 050332220d9a
# Follow manual bisect process
```

**Option 3: Accept Regression**
```bash
# If Phase 16 features are necessary and overhead is acceptable:
# Update baseline to bc3c7b04
# Document performance cost of Phase 16 features
# Plan optimization work for future
```

## Recommendation

Based on evidence, I recommend:

1. **Test fc22692d first** (20 minutes)
   - If PASS before, FAIL after → dual-worker is the culprit
   - If FAIL before → test earlier commits

2. **If dual-worker is culprit:**
   - Review implementation for optimization opportunities
   - Profile dual-worker overhead
   - Consider if overhead is acceptable for feature value

3. **If not dual-worker:**
   - Test observability probes (27136fec)
   - Then graph validation (705de486)

4. **Document findings:**
   - Performance cost per feature
   - Optimization opportunities
   - Trade-off analysis (feature value vs performance cost)

---

**Priority:** HIGH  
**Estimated time:** 1.5-3 hours  
**Confidence:** 90% that fc22692d is the primary culprit

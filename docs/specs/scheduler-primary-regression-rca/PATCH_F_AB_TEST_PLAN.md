# Patch F: Boundary Enforcement A/B Test Plan

**Date**: 2026-04-19  
**Commit Base**: 12316741 (Patch E)  
**Branch**: test/boundary-enforcement-ab-test  
**Objective**: Establish causality between Phase 16 boundary enforcement and performance regression

## Hypothesis

The ~18% performance regression originates from Phase 16 boundary enforcement subsystem, specifically its impact on the entry window (24.3M ticks, 80% of total latency).

**Current Evidence** (correlational, not causal):
- Phase 16 enabled → regression present
- Entry window dominates latency (80%)
- Pure syscall improved 30% with Patch E, but entry window increased 10%
- IRET count stable at 61 (scheduler cadence unchanged)

**What We Don't Know** (need causality):
- Does boundary enforcement directly cause entry window inflation?
- Or is entry window inflation from other Ring3 transition overhead?
- How much of the regression is attributable to boundary enforcement vs other factors?

## Experimental Design

### Run A: Baseline (Current State)
```
AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE=1
```
- All other flags unchanged
- Commit: 12316741 (Patch E)
- Environment: Authoritative GitHub CI (Linux x86_64)

### Run B: Treatment (Boundary Enforcement Disabled)
```
AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE=0
```
- All other flags unchanged
- Same commit: 12316741 (Patch E)
- Same environment: Authoritative GitHub CI (Linux x86_64)

### Control Variables (MUST NOT CHANGE)
- `AYKEN_SYSCALL_DIAGNOSTIC_MARKERS_ENABLE=0` (same in both runs)
- `AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE=1` (same in both runs)
- `AYKEN_RING3_FETCH_PROBE=0` (same in both runs)
- All Ring3 transition flags (same in both runs)
- Kernel profile: release (same in both runs)

## Metrics to Compare

### Primary Metrics (Constitutional Compliance)
| Metric | Baseline (Run A) | Threshold | Expected Run B |
|--------|------------------|-----------|----------------|
| boot_time_ms | 12714ms | ≤11752ms | ? |
| syscall_latency_ms_proxy | 204.90ms | ≤183.84ms | ? |
| context_switch_latency_ms_proxy | 204.90ms | ≤183.84ms | ? |

### Diagnostic Metrics (Root Cause Isolation)
| Metric | Run A (Patch E) | Expected Run B | Delta Interpretation |
|--------|-----------------|----------------|----------------------|
| entry_latency_ticks | 24,343,714 | ? | If ↓↓ → boundary enforcement inflates entry |
| syscall_latency_ticks_pure | 6,225,818 | ? | If ↓ → boundary enforcement in syscall body |
| preempt_iret_count | 61 | 61 | Should be stable (deterministic test) |

## Interpretation Matrix

### Scenario 1: Metrics Approach Baseline
```
boot_time_ms: 12714 → ~11000ms (-13%)
syscall_latency_ms_proxy: 204.90 → ~180ms (-12%)
entry_latency_ticks: 24.3M → ~18M (-26%)
```

**Conclusion**: Boundary enforcement is the primary root cause  
**Next Action**: Redesign boundary enforcement for lower overhead  
**Options**:
- Move enforcement checks to boot-time validation
- Cache enforcement results per process
- Optimize enforcement hot-path further
- Consider enforcement architecture redesign

### Scenario 2: Metrics Unchanged
```
boot_time_ms: 12714 → 12700ms (-0.1%)
syscall_latency_ms_proxy: 204.90 → 204.5ms (-0.2%)
entry_latency_ticks: 24.3M → 24.0M (-1%)
```

**Conclusion**: Boundary enforcement is NOT the primary cause  
**Next Action**: Profile entry window components  
**Focus Areas**:
- Ring3 transition overhead (CR3 pivot, page table operations)
- Frame validation overhead
- Text walk proof overhead
- Scheduler mailbox operations in entry path
- Other Phase 16 components (BCIB worker, probe validation)

### Scenario 3: Syscall Improves, Boot Doesn't
```
boot_time_ms: 12714 → 12500ms (-1.7%)
syscall_latency_ms_proxy: 204.90 → 185ms (-9.7%)
entry_latency_ticks: 24.3M → 20M (-18%)
syscall_latency_ticks_pure: 6.2M → 4M (-35%)
```

**Conclusion**: Two separate cost centers  
**Next Action**: Investigate both paths  
**Analysis**:
- Boundary enforcement affects syscall path (measurable)
- Boot regression has additional source (entry/init overhead)
- Need separate optimization strategies

### Scenario 4: Entry Window Improves, Pure Syscall Unchanged
```
entry_latency_ticks: 24.3M → 18M (-26%)
syscall_latency_ticks_pure: 6.2M → 6.0M (-3%)
```

**Conclusion**: Boundary enforcement primarily affects entry window, not syscall body  
**Next Action**: Profile boundary enforcement's entry-side impact  
**Focus**: How does boundary enforcement inflate Ring3 transition overhead?

## Implementation

### Step 1: Document Current State (Run A)
Already documented in `PATCH_E_CI_VERDICT.md`:
- syscall_latency_ms_proxy: 204.90ms
- boot_time_ms: 12714ms
- entry_latency_ticks: 24,343,714
- syscall_latency_ticks_pure: 6,225,818
- preempt_iret_count: 61

### Step 2: Modify Build Flag
```bash
# In Makefile, change:
AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE ?= 0
```

### Step 3: Verify Build
```bash
make clean
make kernel.elf
make ci-gate-pre-ci
```

### Step 4: Push to CI
```bash
git add Makefile
git commit -m "test: disable boundary enforcement for A/B test (Run B)"
git push origin test/boundary-enforcement-ab-test
```

### Step 5: Download Artifacts
```bash
# After CI completes
bash docs/specs/scheduler-primary-regression-rca/GET_CI_ARTIFACTS.sh <run_id>
```

### Step 6: Compare Results
Create `PATCH_F_AB_TEST_RESULTS.md` with side-by-side comparison

## Success Criteria

This is a DIAGNOSTIC test, not a fix. Success means:
- ✅ Both runs complete successfully in authoritative CI
- ✅ IRET count remains stable (61 in both runs)
- ✅ Clear delta in metrics between Run A and Run B
- ✅ Causal relationship established (or refuted)

## Risk Assessment

### Low Risk
- Boundary enforcement is a Phase 16 feature with kill switch
- Disabling it for diagnostic purposes is architecturally safe
- This is a test branch, not production
- Can revert immediately after measurement

### Architectural Compliance
- This test does NOT violate AykenOS principles
- We are measuring mechanism cost, not bypassing validation
- Boundary enforcement can be redesigned if proven costly
- Constitutional compliance requires performance within thresholds

## Timeline

1. **Now**: Document test plan (this file)
2. **Next**: Modify Makefile, commit, push
3. **Wait**: CI completes (~15 minutes)
4. **Then**: Download artifacts, analyze results
5. **Finally**: Document findings, determine next action

## Notes

- This test answers: "Is boundary enforcement the root cause?"
- It does NOT answer: "How do we fix it?" (that comes after)
- Clean A/B design: single variable changed, all else constant
- Authoritative environment: GitHub CI Linux x86_64 only
- IRET count stability is critical validation metric

---

**Status**: READY TO EXECUTE  
**Next Action**: Modify Makefile and push to CI

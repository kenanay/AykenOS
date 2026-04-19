# Patch F: Boundary Enforcement A/B Test - Status

**Date**: 2026-04-19  
**Branch**: test/boundary-enforcement-ab-test  
**Commit**: a0a41125  
**Status**: ⏳ WAITING FOR CI

## Objective

Establish causal relationship between Phase 16 boundary enforcement and the ~18% performance regression.

## Test Design

Clean A/B test with single variable changed:

### Run A (Baseline - Already Complete)
- **CI Run**: 24636726029
- **Commit**: 12316741 (Patch E)
- **Config**: `AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE=1`
- **Results**:
  - boot_time_ms: 12714ms (FAIL, need ≤11752ms)
  - syscall_latency_ms_proxy: 204.90ms (FAIL, need ≤183.84ms)
  - entry_latency_ticks: 24,343,714 (80% of total)
  - syscall_latency_ticks_pure: 6,225,818 (20% of total)
  - preempt_iret_count: 61 (stable)

### Run B (Treatment - In Progress)
- **CI Run**: TBD (waiting for completion)
- **Commit**: a0a41125 (this commit)
- **Config**: `AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE=0`
- **Results**: Pending

## Hypothesis

Boundary enforcement subsystem causes entry window inflation:
- Entry window: 24.3M ticks (80% of total latency)
- Pure syscall: 6.2M ticks (20% of total latency)
- Hypothesis: Disabling enforcement will reduce entry window significantly

## Expected Outcomes

### Scenario 1: Metrics Approach Baseline (Boundary Enforcement is Root Cause)
```
boot_time_ms: 12714 → ~11000ms (-13%)
syscall_latency_ms_proxy: 204.90 → ~180ms (-12%)
entry_latency_ticks: 24.3M → ~18M (-26%)
```
**Conclusion**: Boundary enforcement is primary cause  
**Next Action**: Redesign boundary enforcement for lower overhead

### Scenario 2: Metrics Unchanged (Boundary Enforcement NOT Root Cause)
```
boot_time_ms: 12714 → 12700ms (-0.1%)
syscall_latency_ms_proxy: 204.90 → 204.5ms (-0.2%)
entry_latency_ticks: 24.3M → 24.0M (-1%)
```
**Conclusion**: Entry window has other bottlenecks  
**Next Action**: Profile Ring3 transition components (CR3 pivot, page tables, frame validation)

### Scenario 3: Syscall Improves, Boot Doesn't (Two Cost Centers)
```
boot_time_ms: 12714 → 12500ms (-1.7%)
syscall_latency_ms_proxy: 204.90 → 185ms (-9.7%)
entry_latency_ticks: 24.3M → 20M (-18%)
```
**Conclusion**: Two separate cost centers  
**Next Action**: Investigate both boundary enforcement and boot/entry overhead

## Control Variables (Unchanged)

All other flags remain constant between Run A and Run B:
- `AYKEN_SYSCALL_DIAGNOSTIC_MARKERS_ENABLE=0`
- `AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE=1`
- `AYKEN_RING3_FETCH_PROBE=0`
- All Ring3 transition flags
- Kernel profile: release

## Timeline

1. ✅ **2026-04-19 19:15**: Test plan documented
2. ✅ **2026-04-19 19:16**: Makefile modified, committed
3. ✅ **2026-04-19 19:17**: Pushed to GitHub, CI triggered
4. ⏳ **Next**: Wait for CI completion (~15 minutes)
5. **Then**: Download artifacts, analyze results
6. **Finally**: Document findings, determine next action

## CI Monitoring

Check CI status:
```bash
gh run list --branch test/boundary-enforcement-ab-test
```

Download artifacts after completion:
```bash
bash docs/specs/scheduler-primary-regression-rca/GET_CI_ARTIFACTS.sh <run_id>
```

## Key Metrics to Extract

From `gates/performance/report.json`:
- `boot_time_ms`
- `syscall_latency_ms_proxy`
- `context_switch_latency_ms_proxy`
- `entry_latency_ticks`
- `syscall_latency_ticks_pure`
- `preempt_iret_count` (should remain 61)

## Risk Assessment

- ✅ Low risk: Boundary enforcement has kill switch
- ✅ Diagnostic only: Test branch, not production
- ✅ Reversible: Can revert immediately
- ✅ Architecturally compliant: Measuring mechanism cost, not bypassing validation

## References

- Test Plan: `PATCH_F_AB_TEST_PLAN.md`
- Patch E Results: `PATCH_E_CI_VERDICT.md`
- Patch C Analysis: `CI_RUN_24634827102_FINAL_VERDICT.md`

---

**Next Update**: After CI completion with Run B results

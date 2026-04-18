# Performance Regression Investigation Status

**Prepared by**: Kenan AY - Architectural Steward  
**Date**: 2026-04-18  
**Status**: Active Investigation - New Spec Opened

## Current Situation

A primary performance regression persists in authoritative GitHub CI despite fixing a secondary snapshot duplication issue:

- boot_time: 12,176ms vs baseline 10,684ms (+1,492ms, +14%)
- syscall_latency: 195.7ms vs 175ms (+11.8%)
- context_switch_latency: 195.7ms vs 175ms (+11.8%)

## Investigation History

### Phase 1: Initial Investigation (CLOSED)
**Spec**: `.kiro/specs/phase16-performance-regression-rca`  
**Status**: SECONDARY ISSUE RESOLVED (PRIMARY REGRESSION UNRESOLVED)  
**Date**: 2026-04-16 to 2026-04-18

**Findings**:
- ✅ BCIB execution engine, boundary enforcement, observability probes, and trace markers have zero overhead
- ✅ Stale epoch short-circuit is working correctly
- ✅ Snapshot duplication fix (commit 31a33246) reduced overhead by ~14ms
- ❌ Primary regression (+1,492ms) persists after snapshot fix

**Authoritative CI Evidence**: GitHub Actions Run 24610398801

**Conclusion**: Snapshot duplication was a real but secondary issue. Primary regression source remains unidentified.

### Phase 2: Primary Regression Investigation (ACTIVE)
**Spec**: `.kiro/specs/scheduler-primary-regression-rca`  
**Status**: REQUIREMENTS AND DESIGN COMPLETE  
**Date**: 2026-04-18 onwards

**Approach**: Surgical 4-probe measurement strategy to identify exact syscall path segment causing regression

**Measurement Points**:
1. EVT_SYSCALL_GATE_ENTER - Syscall entry point
2. EVT_SYSCALL_BODY_ENTER - Kernel handler start
3. EVT_SYSCALL_GATE_RETURN - Syscall return start
4. EVT_CONTEXT_SWITCH_COMPLETE - Context switch and restore complete

**Derived Metrics**:
- ENTRY_COST = BODY_ENTER - GATE_ENTER (entry overhead)
- KERNEL_COST = GATE_RETURN - BODY_ENTER (kernel execution)
- RETURN_COST = SWITCH_COMPLETE - GATE_RETURN (return + context switch + restore)
- TOTAL_COST = SWITCH_COMPLETE - GATE_ENTER (total roundtrip)

**Expected Outcome**: Single CI run will identify which segment (entry/kernel/return) is causing the regression

**Primary Hypothesis** (70% confidence): RETURN_COST inflated due to context switch / return boundary overhead

**Critical Evidence**: syscall_latency and context_switch_latency regressed by IDENTICAL amounts (+11.8%), indicating shared overhead in return-to-user + context switch path

**5 Hotspots to Investigate** (if RETURN_COST inflated):
1. CR3 / address space switch (unnecessary writes, TLB flush)
2. Low-level register restore (double restore, redundancy)
3. Return-to-user frame (IRET prep, validation)
4. Scheduler handoff (bookkeeping, state publish)
5. IRQ/barrier restore (excessive CLI/STI, memory barriers)

## Confirmed Non-Root Causes

The following have been verified as NOT causing the primary regression:

1. ❌ BCIB execution engine features
2. ❌ Boundary enforcement subsystem
3. ❌ Observability probes and trace markers
4. ❌ Stale epoch short-circuit failure
5. ❌ Snapshot duplication (fixed but minimal impact)

## Constitutional Compliance

**Threshold**: boot_time ≤ 11,752ms (baseline + 10%)  
**Current**: boot_time = 12,176ms  
**Violation**: +424ms above threshold (+3.6% additional)

**Authority**: Performance regression violates constitutional 10% threshold and requires resolution before merge approval.

## Verification Authority

**Authoritative Environment**: GitHub CI Linux x86_64  
**Invalid Environments**: Local Darwin/arm64 (env_hash_mismatch)

All performance measurements must be verified in authoritative GitHub CI environment for constitutional compliance.

## Next Steps

1. Implement 4-probe measurement infrastructure in syscall + context switch path
2. Run single CI build with probes enabled
3. Analyze ENTRY_COST, KERNEL_COST, RETURN_COST to identify inflated segment
4. If RETURN_COST inflated (70% confidence), investigate 5 hotspots in priority order
5. Apply targeted optimization following AykenOS architectural constraints
6. Verify constitutional compliance restored (boot_time ≤ 11,752ms)
7. Remove measurement probes

## AykenOS Architectural Constraints

All performance fixes must adhere to these immutable principles:

1. **MECHANISM ≠ POLICY**: Optimize execution, never bypass validation or boundary checks
2. **SHORTCUT ≠ SKIP**: Eliminate redundancy, preserve semantic equivalence
3. **OBSERVABILITY PRESERVED**: Reduce cost, maintain trace identity and event emission
4. **DETERMINISM MANDATORY**: No branch-based fast paths, no nondeterministic behavior
5. **BOUNDARY & SECURITY IMMUTABLE**: Never remove boundary_validate, syscall guards, or capability checks
6. **LAYER DISCIPLINE**: Fix within appropriate layer, no cross-layer violations
7. **STRUCTURE PRESERVED**: No inline sprawl, maintain abstraction and readability
8. **ELIMINATE REDUNDANCY, NOT RESPONSIBILITY**: Same behavior, lower cost

## Related Documentation

- `.kiro/specs/scheduler-primary-regression-rca/bugfix.md` - Requirements document
- `.kiro/specs/scheduler-primary-regression-rca/design.md` - Technical design
- `.kiro/specs/scheduler-primary-regression-rca/tasks.md` - Implementation plan
- `.kiro/specs/phase16-performance-regression-rca/INVESTIGATION_SUMMARY.md` - Previous investigation summary
- `PERFORMANCE_REGRESSION_INVESTIGATION_SUMMARY.md` - Historical investigation (superseded)
- `PERF_REGRESSION_ANALYSIS_STATUS.md` - Historical analysis (superseded)
- `REGRESSION_ROOT_CAUSE_ANALYSIS.md` - Historical RCA (superseded)

## Key Architectural Decisions

1. **Minimal instrumentation**: Only 3 probes, no tick accumulation, no global counters
2. **Single CI run**: Identify root cause in one authoritative measurement
3. **Surgical optimization**: Fix only the identified segment, preserve all other behavior
4. **Constitutional enforcement**: No baseline update until regression is eliminated

## Timeline

- **Phase 1 Investigation**: 2026-04-16 to 2026-04-18 (2 days)
- **Phase 2 Spec Creation**: 2026-04-18 (completed)
- **Phase 2 Implementation**: TBD (estimated 4-6 hours)
- **Phase 2 Verification**: TBD (estimated 1-2 hours)

---

**Architectural Authority**: Kenan AY  
**Investigation Lead**: Kiro AI Assistant  
**Priority**: High (CI blocked, constitutional violation)

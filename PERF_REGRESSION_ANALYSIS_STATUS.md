# Performance Regression Analysis - Current Status

## Problem Statement

GitHub Actions CI is failing due to performance regression:
- Boot time: 10684ms → 12197ms (+14% regression)
- Context switch latency: 175.08ms → 201.93ms (+15% regression)
- Syscall latency: 175.08ms → 201.93ms (+15% regression)

Baseline from 2026-04-09 (commit 050332220d9a), 95 commits behind current HEAD.

## Critical Understanding

### What We Know
1. **System is functionally correct**: IRQ/scheduler/context-switch chain proven working
2. **Uniform regression**: +14-15% across all metrics suggests common hot-path overhead
3. **Not a runtime bug**: This is performance overhead, not broken functionality
4. **Baseline lock is protected**: PR cannot modify baseline (intentional security mechanism)

### What We Don't Know Yet
1. Which Phase 16 feature(s) caused the regression
2. How much each feature contributes to the overhead
3. Whether the overhead is acceptable or needs optimization

## Approach: Measure First, Decide Later

### Phase 1: Measurement Infrastructure (COMPLETED)

Created lightweight performance diagnostic system:

**Files Created:**
- `kernel/perf/perf_diag.h` - Measurement API and macros
- `kernel/perf/perf_diag.c` - TSC accumulation implementation
- `PERF_MEASUREMENT_PLAN.md` - Strategy document
- `PERF_INTEGRATION_GUIDE.md` - Integration instructions

**Design Principles:**
- Accumulate TSC measurements without per-call log spam
- Single summary emission at deterministic exit
- Profile-controlled (validation profile only)
- Zero overhead when disabled

**Measurement Points:**
- IRQ handler entry/exit
- Scheduler dispatch entry/exit
- Context switch wrapper
- Syscall gate entry/exit
- Mailbox extract/validate
- Boundary enforcement checks
- BCIB validation checks

### Phase 2: Integration (NEXT STEP)

Integrate measurement points into hot paths:

1. `kernel/arch/x86_64/timer.c` - IRQ handler
2. `kernel/sched/sched.c` - Scheduler and mailbox operations
3. `kernel/sys/syscall_v2_hardened.c` - Syscall gate
4. `kernel/sys/boundary_enforcement.c` - Boundary checks
5. `kernel/main.c` - Boot markers
6. Deterministic exit path - Summary emission

See `PERF_INTEGRATION_GUIDE.md` for detailed integration instructions.

### Phase 3: Experiment Matrix (AFTER INTEGRATION)

Run 4 configurations to isolate overhead:

**Run A - Current (Baseline)**
- All features enabled
- Reference measurement

**Run B - Observability Reduced**
- Debug markers OFF
- Probes OFF
- Goal: Measure log/probe overhead

**Run C - BCIB/Dual-Worker Minimal**
- No BCIB worker bootstrap
- No dual-worker infrastructure
- Goal: Measure worker overhead

**Run D - Validation Minimal**
- Minimum enforcement only
- Expensive validation disabled
- Goal: Measure enforcement overhead

### Phase 4: Analysis and Decision

Compare average TSC per operation across configurations:

| Scenario | Interpretation |
|----------|---------------|
| A → B large drop | Observability/probe overhead dominant |
| B → C large drop | Dual-worker/BCIB infrastructure dominant |
| C → D large drop | Boundary/validation overhead dominant |
| All small drops | Cumulative hot-path bloat |

Then decide:
- **Optimize**: If overhead is unacceptable and can be reduced
- **Accept**: If overhead is acceptable cost of Phase 16 features

### Phase 5: Baseline Update (FINAL STEP)

Only after understanding root cause:
1. Document which feature(s) caused regression
2. Document measured overhead per feature
3. Document decision (optimize or accept)
4. Update baseline via authorized workflow (`perf-baseline-init`)

## Suspected Culprits

Based on commit history and code inspection:

1. **Hot-path marker emission** - `outb(0xE9, ...)` calls are expensive
2. **Mailbox extract validation** - Snapshot + barrier + retry on every tick
3. **Boundary/BCIB checks** - Common path overhead (both syscall and switch regressed)
4. **Dual-worker scheduling** - Ready/running arbitration complexity
5. **Scheduler instrumentation** - `sched_emit_*` marker family

## Why Not Update Baseline Now?

Updating baseline without understanding root cause would:
- Hide the problem instead of solving it
- Violate AykenOS CI integrity principles
- Lose visibility into performance characteristics
- Make future regressions harder to detect

The baseline lock file mutation guard in PR is intentional - it forces conscious decision-making about performance changes.

## Current Blockers

None. Infrastructure is ready for integration.

## Next Actions

1. **Integrate measurement points** (see `PERF_INTEGRATION_GUIDE.md`)
2. **Update Makefile** with `AYKEN_PERF_DIAG` build flag
3. **Run experiment matrix** (4 configurations)
4. **Analyze results** with comparison script
5. **Identify root cause** from per-feature breakdown
6. **Decide**: optimize or accept
7. **Update baseline** via authorized workflow with justification

## Timeline Estimate

- Integration: 2-3 hours (straightforward macro insertion)
- Experiment runs: 1 hour (4 QEMU runs)
- Analysis: 30 minutes (parse and compare)
- Decision: Depends on findings
- Baseline update: 15 minutes (authorized workflow)

Total: ~4 hours to root cause identification

## References

- `PERF_MEASUREMENT_PLAN.md` - Detailed strategy
- `PERF_INTEGRATION_GUIDE.md` - Integration instructions
- `scripts/ci/perf-baseline.lock.json` - Current baseline
- `scripts/ci/gate_performance.sh` - Performance gate logic
- Baseline commit: 050332220d9a (2026-04-09)
- Current HEAD: 9b3358e6 (95 commits ahead)

## Key Takeaway

**Measure, understand, then decide.** 

The infrastructure is ready. The next step is integration and measurement, not baseline updates.

# Performance Measurement Plan - Phase 16 Regression Root Cause Analysis

## Executive Summary

Performance regression detected: +14-15% across all metrics (boot, context_switch, syscall).
Baseline from 2026-04-09 (commit 050332220d9a), 95 commits behind current HEAD.

**Critical Rule**: DO NOT update baseline until root cause is measured and understood.

## Current State

- System is functionally correct (IRQ/scheduler/context-switch proven working)
- Uniform regression (+14-15%) suggests common hot-path overhead
- Baseline lock file mutation in PR is intentionally blocked
- CI correctly rejects performance threshold violations

## Measurement Strategy

### 1. Lightweight Performance Counter Infrastructure

Create a single diagnostic structure to accumulate TSC measurements without log spam:

```c
typedef struct {
    // Boot phase
    uint64_t boot_start_tsc;
    uint64_t first_sched_tsc;
    uint64_t first_user_entry_tsc;
    
    // IRQ path
    uint64_t irq_count;
    uint64_t irq_total_tsc;
    
    // Scheduler path
    uint64_t sched_count;
    uint64_t sched_total_tsc;
    
    // Context switch path
    uint64_t switch_count;
    uint64_t switch_total_tsc;
    
    // Syscall path
    uint64_t syscall_count;
    uint64_t syscall_total_tsc;
    
    // Mailbox operations
    uint64_t mailbox_extract_count;
    uint64_t mailbox_extract_total_tsc;
    
    // Boundary enforcement
    uint64_t boundary_check_count;
    uint64_t boundary_check_total_tsc;
    
    // BCIB validation
    uint64_t bcib_check_count;
    uint64_t bcib_check_total_tsc;
} ayken_perf_diag_t;
```

### 2. Measurement Points

Add entry/exit TSC capture at:
- `timer_isr_c()` - IRQ handler
- `sched_yield_irq()` / scheduler dispatch
- `context_switch()` call site
- Syscall gate entry/exit
- Mailbox extract/validate path
- Boundary enforcement validate path
- BCIB validation path

**Rule**: Accumulate only, no per-call marker emission.

### 3. Single Summary Marker

At deterministic exit, emit one block:

```
[[AYKEN_PERF_DIAG]] irq_avg=<tsc>
[[AYKEN_PERF_DIAG]] sched_avg=<tsc>
[[AYKEN_PERF_DIAG]] switch_avg=<tsc>
[[AYKEN_PERF_DIAG]] syscall_avg=<tsc>
[[AYKEN_PERF_DIAG]] mailbox_extract_avg=<tsc>
[[AYKEN_PERF_DIAG]] boundary_avg=<tsc>
[[AYKEN_PERF_DIAG]] bcib_avg=<tsc>
```

### 4. Experiment Matrix

Run 4 configurations in authorized workflow:

**Run A - Current (Baseline)**
- All features enabled
- Reference measurement

**Run B - Observability Reduced**
- Ring3 observability probes OFF
- Extra trace markers OFF
- Functional contract unchanged
- Goal: Measure log/probe overhead

**Run C - BCIB/Dual-Worker Minimal**
- syscall-v2-runtime only
- No BCIB worker bootstrap
- No dual-worker infrastructure
- Goal: Measure worker infrastructure overhead

**Run D - Validation Minimal**
- Minimum enforcement only
- Expensive validation branches disabled
- Goal: Measure enforcement/check overhead

### 5. Decision Matrix

| Scenario | Interpretation |
|----------|---------------|
| A → B large drop | Observability/probe overhead dominant |
| B → C large drop | Dual-worker/BCIB infrastructure dominant |
| C → D large drop | Boundary/validation overhead dominant |
| All small drops | Cumulative hot-path bloat (no single culprit) |

## Suspected Hot-Path Culprits

Based on commit history and code inspection:

1. **Hot-path marker emission** - `outb(0xE9, ...)` calls are expensive
2. **Mailbox extract validation** - Snapshot + barrier + retry on every tick
3. **Boundary/BCIB common checks** - Syscall and switch proxy both regressed similarly
4. **Dual-worker scheduling** - Ready/running arbitration complexity
5. **Scheduler instrumentation** - `sched_emit_*` marker family

## Implementation Files

### Core Infrastructure
- `kernel/perf/perf_diag.h` - Structure definition
- `kernel/perf/perf_diag.c` - Accumulation and summary emission

### Measurement Points
- `kernel/arch/x86_64/timer.c` - IRQ entry/exit
- `kernel/sched/sched.c` - Scheduler dispatch, mailbox operations
- `kernel/arch/x86_64/context_switch.asm` - Context switch wrapper
- `kernel/sys/syscall_v2_hardened.c` - Syscall gate
- `kernel/sys/boundary_enforcement.c` - Boundary checks
- (BCIB validation path TBD based on architecture)

### Build Control
- Add `AYKEN_PERF_DIAG=1` build flag
- Profile-controlled (validation profile only)
- Zero overhead when disabled

## Next Steps

1. Implement perf counter infrastructure
2. Add measurement points to hot paths
3. Create experiment configurations
4. Run 4-configuration matrix in CI
5. Analyze per-feature overhead
6. Either optimize expensive feature OR consciously accept overhead
7. Update baseline via authorized workflow with justification

## Non-Goals

- Do NOT add new log spam
- Do NOT change functional behavior
- Do NOT update baseline before measurement
- Do NOT treat this as optimization work (measurement only)

## Success Criteria

- Per-feature average cost table generated
- Root cause of +15% regression identified
- Conscious decision: optimize vs accept
- Baseline update with full justification

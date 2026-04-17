# Quick Start: Performance Regression Analysis

## TL;DR

Performance regressed +14-15% across all metrics. System works correctly, but slower.

**DO NOT** update baseline yet. First measure which Phase 16 features caused it.

## What Was Created

1. **Measurement infrastructure** - `kernel/perf/perf_diag.{h,c}`
2. **Strategy document** - `PERF_MEASUREMENT_PLAN.md`
3. **Integration guide** - `PERF_INTEGRATION_GUIDE.md`
4. **Status summary** - `PERF_REGRESSION_ANALYSIS_STATUS.md`

## Quick Integration Checklist

### 1. Add Measurement Points (5 minutes each)

```c
#include "perf/perf_diag.h"

// IRQ handler (kernel/arch/x86_64/timer.c)
void timer_isr_c(void) {
    PERF_DIAG_IRQ_START();
    // ... existing code ...
    PERF_DIAG_IRQ_END();
}

// Scheduler (kernel/sched/sched.c)
void sched_yield_irq(void) {
    PERF_DIAG_SCHED_START();
    // ... existing code ...
    PERF_DIAG_SCHED_END();
}

// Context switch (before context_switch() call)
PERF_DIAG_SWITCH_START();
context_switch(old, new);
PERF_DIAG_SWITCH_END();

// Syscall gate (kernel/sys/syscall_v2_hardened.c)
uint64_t syscall_v2_hardened_dispatch(...) {
    PERF_DIAG_SYSCALL_START();
    // ... existing code ...
    PERF_DIAG_SYSCALL_END();
}

// Mailbox extract (kernel/sched/sched.c)
static int mailbox_extract_candidate(...) {
    PERF_DIAG_MAILBOX_EXTRACT_START();
    // ... existing code ...
    PERF_DIAG_MAILBOX_EXTRACT_END();
}

// Boundary check (kernel/sys/boundary_enforcement.c)
int boundary_check_bcib_submission_path(...) {
    PERF_DIAG_BOUNDARY_CHECK_START();
    // ... existing code ...
    PERF_DIAG_BOUNDARY_CHECK_END();
}

// Boot markers (kernel/main.c)
void kernel_main(void) {
    PERF_DIAG_INIT();
    PERF_DIAG_MARK_BOOT_START();
    // ...
}

// First user entry
PERF_DIAG_MARK_FIRST_USER_ENTRY();

// Deterministic exit
void deterministic_exit(void) {
    PERF_DIAG_EMIT_SUMMARY();
    // ...
}
```

### 2. Update Makefile (2 minutes)

```makefile
# Add to kernel/Makefile or main Makefile
ifeq ($(KERNEL_PROFILE),validation)
  ifeq ($(AYKEN_PERF_DIAG),1)
    CFLAGS += -DAYKEN_PERF_DIAG
  endif
endif

KERNEL_SOURCES += kernel/perf/perf_diag.c
```

### 3. Run Experiments (1 hour total)

```bash
# Run A - Current (all features)
make clean && make KERNEL_PROFILE=validation AYKEN_PERF_DIAG=1 efi-img
./run_qemu.sh > logs/run_a.log 2>&1

# Run B - Observability reduced
make clean && make KERNEL_PROFILE=validation AYKEN_PERF_DIAG=1 \
    AYKEN_DEBUG_SCHED=0 AYKEN_DEBUG_IRQ=0 efi-img
./run_qemu.sh > logs/run_b.log 2>&1

# Run C - BCIB minimal
make clean && make KERNEL_PROFILE=validation AYKEN_PERF_DIAG=1 \
    AYKEN_SCHED_BOOTSTRAP_POLICY=0 efi-img
./run_qemu.sh > logs/run_c.log 2>&1

# Run D - Validation minimal
make clean && make KERNEL_PROFILE=validation AYKEN_PERF_DIAG=1 \
    AYKEN_RING3_ENTRY_GUARD=0 AYKEN_MB_SELFTEST=0 efi-img
./run_qemu.sh > logs/run_d.log 2>&1
```

### 4. Analyze Results (10 minutes)

```bash
# Extract summaries
grep -A 50 "AYKEN_PERF_DIAG_SUMMARY" logs/run_a.log > results/run_a.txt
grep -A 50 "AYKEN_PERF_DIAG_SUMMARY" logs/run_b.log > results/run_b.txt
grep -A 50 "AYKEN_PERF_DIAG_SUMMARY" logs/run_c.log > results/run_c.txt
grep -A 50 "AYKEN_PERF_DIAG_SUMMARY" logs/run_d.log > results/run_d.txt

# Compare (manual or script)
# Look for largest drops in avg_tsc metrics
```

### 5. Decision Matrix

| A → B Drop | B → C Drop | C → D Drop | Culprit |
|------------|------------|------------|---------|
| Large | Small | Small | Observability/probes |
| Small | Large | Small | BCIB/dual-worker |
| Small | Small | Large | Boundary/validation |
| Small | Small | Small | Cumulative bloat |

## Expected Output

```
[[AYKEN_PERF_DIAG_SUMMARY]]
irq_count=61
irq_avg_tsc=20238162
sched_count=62
sched_avg_tsc=37833530
switch_count=61
switch_avg_tsc=56668672
syscall_count=61
syscall_avg_tsc=74883772
mailbox_extract_count=62
mailbox_extract_avg_tsc=9159518
boundary_check_count=1
boundary_check_avg_tsc=123456
[[AYKEN_PERF_DIAG_END]]
```

## After Analysis

1. **Document findings** - Which feature(s) caused regression
2. **Decide** - Optimize or accept
3. **Update baseline** - Via authorized workflow with justification

## Key Files

- `kernel/perf/perf_diag.h` - Measurement API
- `kernel/perf/perf_diag.c` - Implementation
- `PERF_INTEGRATION_GUIDE.md` - Detailed instructions
- `PERF_MEASUREMENT_PLAN.md` - Full strategy

## Critical Rules

- ✅ Measure first, decide later
- ✅ Use authorized workflow for baseline update
- ❌ Do NOT update baseline without understanding root cause
- ❌ Do NOT treat this as optimization work (measurement only)

## Estimated Time

- Integration: 30-45 minutes
- Experiments: 1 hour
- Analysis: 15 minutes
- **Total: ~2 hours to root cause**

## Questions?

See detailed guides:
- `PERF_INTEGRATION_GUIDE.md` - How to integrate
- `PERF_MEASUREMENT_PLAN.md` - Why and what
- `PERF_REGRESSION_ANALYSIS_STATUS.md` - Current state

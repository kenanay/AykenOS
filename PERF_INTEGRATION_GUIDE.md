# Performance Diagnostic Integration Guide

## Overview

This guide explains how to integrate the lightweight performance measurement infrastructure into AykenOS hot paths to identify the root cause of the Phase 16 performance regression.

## Files Created

1. `kernel/perf/perf_diag.h` - Header with measurement API and macros
2. `kernel/perf/perf_diag.c` - Implementation with TSC accumulation
3. `PERF_MEASUREMENT_PLAN.md` - Strategy and experiment matrix
4. `PERF_INTEGRATION_GUIDE.md` - This file

## Integration Points

### 1. Timer IRQ Handler (`kernel/arch/x86_64/timer.c`)

```c
#include "perf/perf_diag.h"

void timer_isr_c(void) {
    PERF_DIAG_IRQ_START();
    
    // Existing IRQ handler code...
    
    PERF_DIAG_IRQ_END();
}
```

### 2. Scheduler Dispatch (`kernel/sched/sched.c`)

```c
#include "perf/perf_diag.h"

// In sched_yield_irq() or main scheduler dispatch:
void sched_yield_irq(void) {
    PERF_DIAG_SCHED_START();
    
    // Existing scheduler code...
    
    PERF_DIAG_SCHED_END();
}

// Mark first scheduler activity:
void sched_init(void) {
    // ... existing init ...
    PERF_DIAG_MARK_FIRST_SCHED();
}
```

### 3. Context Switch (`kernel/sched/sched.c` or wrapper)

```c
// Before context_switch() call:
PERF_DIAG_SWITCH_START();
context_switch(old_proc, new_proc);
PERF_DIAG_SWITCH_END();
```

### 4. Syscall Gate (`kernel/sys/syscall_v2_hardened.c`)

```c
#include "perf/perf_diag.h"

uint64_t syscall_v2_hardened_dispatch(...) {
    PERF_DIAG_SYSCALL_START();
    
    // Existing syscall dispatch...
    
    PERF_DIAG_SYSCALL_END();
    return result;
}
```

### 5. Mailbox Extract (`kernel/sched/sched.c`)

```c
// In mailbox extract/validate path:
static int mailbox_extract_candidate(...) {
    PERF_DIAG_MAILBOX_EXTRACT_START();
    
    // Existing mailbox logic...
    
    PERF_DIAG_MAILBOX_EXTRACT_END();
    return result;
}
```

### 6. Boundary Enforcement (`kernel/sys/boundary_enforcement.c`)

```c
#include "perf/perf_diag.h"

int boundary_check_bcib_submission_path(...) {
    PERF_DIAG_BOUNDARY_CHECK_START();
    
    // Existing boundary check...
    
    PERF_DIAG_BOUNDARY_CHECK_END();
    return result;
}
```

### 7. Boot Markers (`kernel/main.c` or equivalent)

```c
#include "perf/perf_diag.h"

void kernel_main(void) {
    PERF_DIAG_INIT();
    PERF_DIAG_MARK_BOOT_START();
    
    // ... boot sequence ...
}

// At first Ring3 entry:
void first_user_entry_point(void) {
    PERF_DIAG_MARK_FIRST_USER_ENTRY();
    // ...
}
```

### 8. Deterministic Exit (`kernel/exit.c` or shutdown path)

```c
void deterministic_exit(void) {
    PERF_DIAG_EMIT_SUMMARY();
    
    // ... existing exit logic ...
}
```

## Build System Integration

### Makefile Changes

Add to `Makefile` or `kernel/Makefile`:

```makefile
# Performance diagnostic flag (validation profile only)
ifeq ($(KERNEL_PROFILE),validation)
  ifeq ($(AYKEN_PERF_DIAG),1)
    CFLAGS += -DAYKEN_PERF_DIAG
  endif
endif

# Add perf_diag.c to kernel sources
KERNEL_SOURCES += kernel/perf/perf_diag.c
```

### Build Command

```bash
# Enable performance diagnostics
make KERNEL_PROFILE=validation AYKEN_PERF_DIAG=1 efi-img

# Disable (default)
make KERNEL_PROFILE=validation efi-img
```

## Experiment Configurations

### Run A - Current Baseline (All Features)

```bash
make clean
make KERNEL_PROFILE=validation \
     AYKEN_PERF_DIAG=1 \
     USER_MINIMAL_MODE=syscall-v2-runtime \
     AYKEN_SCHED_BOOTSTRAP_POLICY=1 \
     AYKEN_DETERMINISTIC_EXIT=1 \
     AYKEN_RING3_ENTRY_GUARD=1 \
     efi-img
```

### Run B - Observability Reduced

```bash
make clean
make KERNEL_PROFILE=validation \
     AYKEN_PERF_DIAG=1 \
     USER_MINIMAL_MODE=syscall-v2-runtime \
     AYKEN_SCHED_BOOTSTRAP_POLICY=1 \
     AYKEN_DETERMINISTIC_EXIT=1 \
     AYKEN_RING3_ENTRY_GUARD=1 \
     AYKEN_DEBUG_SCHED=0 \
     AYKEN_DEBUG_IRQ=0 \
     AYKEN_RING3_FETCH_PROBE=0 \
     AYKEN_RING3_POST_CR3_TEXT_PROBE=0 \
     efi-img
```

### Run C - BCIB/Dual-Worker Minimal

```bash
make clean
make KERNEL_PROFILE=validation \
     AYKEN_PERF_DIAG=1 \
     USER_MINIMAL_MODE=syscall-v2-runtime \
     AYKEN_SCHED_BOOTSTRAP_POLICY=0 \
     AYKEN_DETERMINISTIC_EXIT=1 \
     AYKEN_RING3_ENTRY_GUARD=1 \
     efi-img
```

### Run D - Validation Minimal

```bash
make clean
make KERNEL_PROFILE=validation \
     AYKEN_PERF_DIAG=1 \
     USER_MINIMAL_MODE=syscall-v2-runtime \
     AYKEN_SCHED_BOOTSTRAP_POLICY=1 \
     AYKEN_DETERMINISTIC_EXIT=1 \
     AYKEN_RING3_ENTRY_GUARD=0 \
     AYKEN_MB_SELFTEST=0 \
     efi-img
```

## Output Analysis

### Expected Output Format

```
[[AYKEN_PERF_DIAG_SUMMARY]]
boot_start_tsc=25161394233
first_sched_tsc=25283844915
first_user_entry_tsc=25303281549
irq_count=61
irq_total_tsc=1234567890
irq_avg_tsc=20238162
sched_count=62
sched_total_tsc=2345678901
sched_avg_tsc=37833530
switch_count=61
switch_total_tsc=3456789012
switch_avg_tsc=56668672
syscall_count=61
syscall_total_tsc=4567890123
syscall_avg_tsc=74883772
mailbox_extract_count=62
mailbox_extract_total_tsc=567890123
mailbox_extract_avg_tsc=9159518
boundary_check_count=1
boundary_check_total_tsc=123456
boundary_check_avg_tsc=123456
bcib_check_count=0
bcib_check_total_tsc=0
bcib_check_avg_tsc=0
[[AYKEN_PERF_DIAG_END]]
```

### Analysis Script

Create `scripts/analyze_perf_diag.py`:

```python
#!/usr/bin/env python3
import sys
import re

def parse_perf_diag(log_file):
    with open(log_file, 'r') as f:
        content = f.read()
    
    # Extract summary block
    match = re.search(r'\[\[AYKEN_PERF_DIAG_SUMMARY\]\](.*?)\[\[AYKEN_PERF_DIAG_END\]\]', 
                      content, re.DOTALL)
    if not match:
        print("No performance diagnostic summary found")
        return None
    
    summary = match.group(1)
    metrics = {}
    
    for line in summary.strip().split('\n'):
        if '=' in line:
            key, value = line.split('=', 1)
            metrics[key.strip()] = int(value.strip())
    
    return metrics

def compare_runs(baseline, experiment):
    print(f"{'Metric':<30} {'Baseline':<15} {'Experiment':<15} {'Delta':<10} {'%':<10}")
    print("-" * 80)
    
    for key in sorted(baseline.keys()):
        if key.endswith('_avg_tsc'):
            base_val = baseline.get(key, 0)
            exp_val = experiment.get(key, 0)
            
            if base_val > 0:
                delta = exp_val - base_val
                pct = (delta / base_val) * 100
                print(f"{key:<30} {base_val:<15} {exp_val:<15} {delta:<10} {pct:>6.2f}%")

if __name__ == '__main__':
    if len(sys.argv) != 3:
        print("Usage: analyze_perf_diag.py <baseline_log> <experiment_log>")
        sys.exit(1)
    
    baseline = parse_perf_diag(sys.argv[1])
    experiment = parse_perf_diag(sys.argv[2])
    
    if baseline and experiment:
        compare_runs(baseline, experiment)
```

## Validation Steps

1. Build with `AYKEN_PERF_DIAG=1`
2. Run QEMU with timeout
3. Extract QEMU log
4. Verify `[[AYKEN_PERF_DIAG_SUMMARY]]` block present
5. Parse metrics
6. Compare across experiment configurations

## Critical Notes

- **Zero overhead when disabled**: `AYKEN_PERF_DIAG` not defined = no-op macros
- **Profile-controlled**: Only enable in validation profile
- **Single summary**: No per-call log spam
- **TSC-based**: Cycle-accurate measurement
- **Thread-safe**: Uses thread-local storage for nested calls

## Next Steps

1. Integrate measurement points into hot paths
2. Update Makefile with build flag
3. Run 4-configuration experiment matrix
4. Analyze per-feature overhead
5. Identify regression culprit
6. Decide: optimize or accept
7. Update baseline via authorized workflow

## Troubleshooting

### No summary output
- Check `AYKEN_PERF_DIAG` is defined during build
- Verify `perf_diag_emit_summary()` is called at exit
- Check QEMU log for marker output

### Zero counts
- Verify measurement points are reached
- Check TSC read is working (`rdtsc` instruction)
- Ensure start/end pairs are balanced

### Unexpected values
- TSC frequency varies by CPU
- Compare relative differences, not absolute values
- Use average TSC per operation for comparison

## References

- `PERF_MEASUREMENT_PLAN.md` - Overall strategy
- `scripts/ci/perf-baseline.lock.json` - Current baseline
- `scripts/ci/gate_performance.sh` - Performance gate logic

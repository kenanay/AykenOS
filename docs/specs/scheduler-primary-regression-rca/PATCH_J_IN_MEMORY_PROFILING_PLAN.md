# Patch J: In-Memory Profiling Plan

## Context
A/B testing methods (Patch I-A to I-C2) have hit structural limits due to governance and architectural boundaries (e.g., `ENTRY_GUARD` and `PCID` locks, `SKIP_CR3_PIVOT` page fault). To identify the remaining ~9.5% unexplained performance regression in the Ring3 entry window, we need to transition from indirect A/B testing to direct, low-overhead in-memory profiling.

## Objective
Implement an in-memory profiling system that avoids the I/O cost of `debugcon` writes while maintaining strict architectural isolation. This will allow us to accurately measure the time spent in different segments of the Ring3 entry path.

## Design
1. **Fixed Memory Buffer**: 
   - A `16 * 1024` byte buffer (`entry_diag_buffer`) is allocated in the `.data` section to store up to 1024 samples.
   - Each sample contains a phase ID and a `TSC` timestamp.

2. **Low-Overhead Recording**:
   - `RECORD_DIAG_TSC` macro is implemented to capture `RDTSC` (with `LFENCE`) directly into the buffer.
   - Designed to avoid clobbering user registers or triggering faults in the entry path.

3. **Injection Points**:
   - Phase 1: `P10_RING3_COMMIT`
   - Phase 2: `P10_CR3_SWITCH` start
   - Phase 3: `P10_CR3_SWITCH` end
   - Phase 4: `P10_RING3_ENTER`
   - Phase 5: `FIRST_FETCH_OK`
   - Phase 6: `FIRST_KERNEL_REENTRY`

4. **Delayed Dump**:
   - Dump logic (`entry_diag_dump`) is deferred until `timer_maybe_exit_on_proof_done()` when the preempt contract is successfully concluded.

## Validation Criteria
- Minimal observer effect on entry metrics compared to the baseline.
- Expected to dump diagnostic phase data exactly once before QEMU exit.
- `preempt_iret_count` == 61.

## Next Steps
1. Commit the changes.
2. Push to branch and wait for CI to run.
3. Extract `ENTRY_DIAG_SAMPLE` logs to analyze precise segment costs.

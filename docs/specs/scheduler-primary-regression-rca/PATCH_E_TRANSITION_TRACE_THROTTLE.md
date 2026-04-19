# Patch E: Ring3 Transition Trace Throttle

## Status

Implemented locally after ci-freeze run `24635638004` showed Patch D did not move the enforced metrics.

## Authoritative CI Evidence

Run: `24635638004`

Metrics:

- `syscall_latency_ms_proxy`: `208.196721ms`
- `context_switch_latency_ms_proxy`: `208.196721ms`
- `boot_time_ms`: `12712ms`

Build evidence confirmed:

- `AYKEN_SYSCALL_DIAGNOSTIC_MARKERS_ENABLE=0`
- No syscall diagnostic marker flood from Patch C/D remained in the performance build.

Artifact evidence showed the remaining measured-path debugcon load came from Ring3 transition proof markers emitted on every return to user mode:

- `P10_TEXT_ROOT_PROOF`
- `P10_ROOT_FRAME_WITNESS`
- `P10_TEXT_FRAME_WITNESS`
- `P10_RING3_FRAME_PROOF`
- `P10_RING3_ATTEMPT`
- `P10_RING3_COMMIT`
- `P10_RING3_ENTER`
- `PIC_MASK`

The enforced proxy metrics are still based on `preempt_qemu_run_time_ms / MARK:IRET count`, so per-transition debugcon proof spam directly slows the measured run even when pure syscall ticks are only informational.

## Fix

Patch E keeps the canonical preempt cadence markers per-transition and makes verbose Ring3 transition proof output one-shot:

- `sched_emit_ring3_frame_proof()` now emits once.
- `sched_emit_pre_dispatch_text_walk_proof()` now emits once.
- `ring3_enter_iretq` now emits verbose P10/PIC transition markers once.

Preserved per-transition markers:

- `MARK:SW`
- `MARK:IRET`
- `[[AYKEN_PERF_PHASE]]`
- `[[AYKEN_PERF_MB_*]]`

## Local Verification

`make kernel.elf` passes.

Local `make ci-gate-performance` still fails on Darwin/arm64 due expected environment and baseline authority mismatch, but it verifies the marker contract remains intact and the heavy proof markers are throttled:

- `P10_TEXT_ROOT_PROOF`: 1 occurrence
- `P10_RING3_FRAME_PROOF`: 1 occurrence
- `P10_RING3_ATTEMPT`: 1 occurrence
- `PIC_MASK`: 2 occurrences
- `MARK:SW`: present at cadence
- `MARK:IRET`: present at cadence

Local debugcon size dropped into the `48KB` class after throttling; the failing CI artifact before Patch E was `186508` debugcon bytes.

## Expected CI Outcome

The next `ci-freeze` run must confirm whether removing per-transition proof spam is sufficient to move the enforced wall-time proxy metrics back under:

- `syscall_latency_ms_proxy <= 183.83606535ms`
- `context_switch_latency_ms_proxy <= 183.83606535ms`
- `boot_time_ms <= 11752.4ms`

If this does not pass, the next focus is the measured preempt run wall-clock model and remaining per-tick mailbox markers, not syscall enforcement path execution.

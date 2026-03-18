# Progress Log

**Status:** Active log for execution-path hardening work
**Scope:** Phase 10-B / 10-C runtime stabilization

## Purpose

This file is the running memory for implementation progress under:

- `requirements.md`
- `design.md`
- `tasks.md`

Every completed implementation slice should be recorded here in the same change
set as the code when feasible.

## Entry Template

```text
Date:
Completed Slice:
Touched Code Paths:
Touched Docs:
Validation:
Impact:
Notes:
```

## Entries

### 2026-03-18

Completed Slice:
- Initial execution-path hardening spec set created
- Concurrency/serialization, stable wait identity, worker pickup model, result mapping permissions, and exit queue cleanup constraints added

Touched Code Paths:
- none yet (documentation-only)

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- no code validation run; documentation update only

Impact:
- preserved a repo-grounded execution-path plan without changing active roadmap authority

Notes:
- Official roadmap truth remains Phase-13 boundary hardening.
- This spec set exists as a local execution-reality stabilization plan and does not override roadmap authority.

### 2026-03-18

Completed Slice:
- Added initial kernel `execution_slot` runtime skeleton
- Added bounded per-`context_id` execution queue storage
- Added interrupt-disabled execution-table critical section helpers for the current single-core bring-up
- Wired passive `execution_slots_init()` into `kernel_late_init()`

Touched Code Paths:
- `kernel/include/execution_slot.h`
- `kernel/sys/execution_slot.c`
- `kernel/kernel.c`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel` passed
- unrelated pre-existing warnings observed in `kernel/proc/proc.c`, `kernel/arch/x86_64/timer.c`, and `kernel/arch/x86_64/interrupts.c`

Impact:
- established a concrete execution-slot truth anchor without altering boot protocol or early bring-up

Notes:
- This slice adds passive execution-state infrastructure only; no syscall behavior changed yet.
- Boot protocol and early bring-up were intentionally left untouched.

### 2026-03-18

Completed Slice:
- Froze `sys_v2_time_query()` query-type contract in the shared ABI
- Replaced dummy time return values with PIT-backed monotonic ticks and uptime milliseconds
- Tightened validation to reject unknown time query types and assert monotonic nondecreasing behavior

Touched Code Paths:
- `shared/abi/syscall_v2.h`
- `kernel/arch/x86_64/timer.h`
- `kernel/arch/x86_64/timer.c`
- `kernel/sys/syscall_v2.c`
- `kernel/tests/validation/phase2_validation_test.c`
- `kernel/tests/validation/syscall_count_test.c`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`

Validation:
- `make kernel` passed
- unrelated pre-existing warnings remain in `kernel/arch/x86_64/timer.c`

Impact:
- replaced a fake time syscall with a real monotonic authority developers can build semantics around

Notes:
- Timeout authority is still a design/runtime constraint; timer IRQ deadline progression is not wired yet.
- This slice intentionally stops at real time authority and does not yet modify submit/wait/exit semantics.

### 2026-03-19

Completed Slice:
- Split syscall documentation into ABI/migration truth and runtime-reality truth
- Reduced overstated "completed/fully operational" language in the active transition guide
- Added an explicit maturity map for incomplete execution-lifecycle syscalls

Touched Code Paths:
- none (documentation-only)

Touched Docs:
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/syscall_transition_guide.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- no code validation run; documentation update only

Impact:
- removed false "fully operational" interpretations from the active syscall guide
- aligned ABI planning with actual runtime maturity so higher-level design choices do not build on placeholder semantics

Notes:
- `time_query` is now treated as operational in runtime reality.
- `submit_execution`, `wait_result`, `interrupt_return`, `exit`, `map_memory`, and `unmap_memory` remain classified as ABI-stable but semantically incomplete.

### 2026-03-19

Completed Slice:
- Added explicit non-guarantees to the runtime reality guide
- Added a critical warning against building higher-level logic on incomplete syscalls
- Upgraded the progress log template to record impact per slice

Touched Code Paths:
- none (documentation-only)

Touched Docs:
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- no code validation run; documentation update only

Impact:
- closed the remaining ambiguity about what the current kernel does not promise
- made future progress entries carry engineering significance, not just edit history

Notes:
- documentation now explicitly warns against treating incomplete syscall surfaces as a stable runtime substrate.

### 2026-03-19

Completed Slice:
- Bound `sys_v2_submit_execution()` to the kernel `execution_slot` table
- Replaced ad hoc execution ID allocation with kernel-owned slot ID authority
- Added validation coverage for `READY` slot creation and target queue insertion

Touched Code Paths:
- `kernel/include/execution_slot.h`
- `kernel/sys/execution_slot.c`
- `kernel/sys/syscall_v2.c`
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel` passed
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c` passed
- pre-existing sign-compare warnings remain in `kernel/tests/validation/phase2_validation_test.c`

Impact:
- turned `submit_execution` into the first syscall that actively uses the execution-slot runtime model
- removed duplicate execution ID authority from `syscall_v2.c` and anchored future worker/wait work on one kernel-owned state surface

Notes:
- BCIB backing copy and live target-context validation remain pending.
- Worker pickup, completion, timeout, and wait semantics are still not implemented.

### 2026-03-19

Completed Slice:
- Revised `phase2_validation_test.c` output to distinguish semantic checks from interface-shape checks
- Removed overstated "complete/operational" success messaging from the legacy validation snapshot
- Aligned validation summary text with the current syscall runtime reality

Touched Code Paths:
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c` passed
- pre-existing sign-compare warnings remain in `kernel/tests/validation/phase2_validation_test.c`

Impact:
- closed the remaining truth-surface gap between runtime documentation and validation output
- reduced the risk of reading placeholder-success tests as proof of full execution lifecycle behavior

Notes:
- this slice changes messaging only; it does not change syscall semantics or runtime maturity.

### 2026-03-19

Completed Slice:
- Added schedule-entry worker pickup that transitions queued `READY` slots to `RUNNING`
- Made `wait_result` reflect slot ownership and nonterminal `RESOURCE_BUSY` state instead of unconditional success
- Tightened validation snapshot expectations around `submit_execution` and `wait_result`

Touched Code Paths:
- `kernel/include/proc.h`
- `kernel/include/execution_slot.h`
- `kernel/sys/execution_slot.c`
- `kernel/sched/sched.c`
- `kernel/sys/syscall_v2.c`
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel` passed
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c` passed
- pre-existing sign-compare warnings remain in `kernel/tests/validation/phase2_validation_test.c`

Impact:
- moved the execution path one step closer to a real lifecycle without faking completion
- removed another placeholder-success surface from validation and runtime behavior

Notes:
- worker pickup currently records `RUNNING` state only; userspace delivery/completion is still pending.
- `wait_result` is still non-blocking until wake/timeout authority is implemented.
- the current scheduler hookup permits one active execution per user process until a later slice clears `active_execution_id`.

### 2026-03-19

Completed Slice:
- Wired finite-timeout `wait_result()` blocking to `proc_block_current(&slot->wait_key)`
- Added bounded timer-IRQ timeout scanning that transitions overdue slots to `TIMEOUT` and wakes waiters
- Cleared worker `active_execution_id` on timeout-driven terminalization

Touched Code Paths:
- `kernel/include/execution_slot.h`
- `kernel/sys/execution_slot.c`
- `kernel/arch/x86_64/timer.h`
- `kernel/arch/x86_64/timer.c`
- `kernel/sys/syscall_v2.c`
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel` passed
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c` passed
- pre-existing sign-compare warnings remain in `kernel/tests/validation/phase2_validation_test.c`

Impact:
- moved `wait_result` from pure polling semantics to a real timeout-driven block/wake path without faking completion
- added the first kernel terminalization path that releases worker execution latches

Notes:
- completion, result delivery, and repeated successful wait semantics are still not implemented.
- the current timeout path can retire queued or running work, but there is still no authoritative success-completion handoff from userspace execution.
- the current timeout deadline is slot-scoped; multi-waiter timeout semantics are not yet frozen.

### 2026-03-19

Completed Slice:
- Added a standalone execution inbox minimal spec for the next delivery/completion slice
- Froze the fixed-VA, kernel-write, user-read-only inbox contract as separate from scheduler mailbox
- Updated tasks to make execution inbox projection the next concrete implementation target

Touched Code Paths:
- none (documentation-only)

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/execution-inbox-minimal-spec.md`
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- no code validation run; documentation update only

Impact:
- removed ambiguity around how worker-visible execution delivery can land without violating the scheduler mailbox boundary
- made the next runtime slice concrete enough to implement without inventing a second control plane

Notes:
- the inbox remains explicitly non-authoritative; kernel queue truth is unchanged.
- completion handoff is still a later slice and is not implied by this spec.

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

### 2026-03-19

Completed Slice:
- Hardened the execution inbox spec with explicit worker latch, overwrite, and publish-order rules
- Recorded payload-window mapping lifecycle and clarified that v1 delivery is polling-visible, not interrupt-driven
- Mirrored the new latch/publication constraints back into the main requirements/design set

Touched Code Paths:
- none (documentation-only)

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/execution-inbox-minimal-spec.md`
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- no code validation run; documentation update only

Impact:
- removed the silent-overwrite and implicit-latch ambiguity before kernel execution inbox implementation
- made the delivery contract explicit enough to implement without inventing hidden queue or ownership semantics

Notes:
- the commit-point rule now depends on a full publish barrier before `delivery_seq` is advanced.
- completion and result ownership remain separate follow-on slices.

### 2026-03-19

Completed Slice:
- Added a file/function-level implementation plan for execution inbox bring-up
- Anchored the next mapping slice to the real `proc_create_user_process()` flow instead of an abstract init path
- Linked worker-pickup tasks to the implementation plan so the next code slice has one canonical checklist

Touched Code Paths:
- none (documentation-only)

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/execution-inbox-implementation-plan.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- no code validation run; documentation update only

Impact:
- converted the execution inbox work from architecture intent into a local, code-anchored kernel plan
- reduced the risk of drifting away from the actual `proc_create_user_process()` and page-flag surfaces while landing `6.4` and `6.5`

Notes:
- the plan is intentionally gated by kernel-owned BCIB backing and real target-context validation.
- at the time of this planning slice, `kernel/include/mm.h` still lacked an explicit NX-style flag in the common mapping header path.

### 2026-03-19

Completed Slice:
- Widened execution-slot BCIB backing from a single placeholder reference to bounded kernel-owned frame metadata
- Made `sys_v2_submit_execution()` validate live `PROC_TYPE_USER` targets and copy accepted BCIB payloads into kernel-owned backing
- Added validation coverage for copy ownership, oversize fail-closed behavior, and invalid target-context rejection

Touched Code Paths:
- `kernel/include/execution_slot.h`
- `kernel/sys/execution_slot.c`
- `kernel/sys/syscall_v2.c`
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/execution-inbox-minimal-spec.md`
- `docs/specs/phase10b-execution-path-hardening/execution-inbox-implementation-plan.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel` passed
- `clang --target=x86_64-elf -ffreestanding -fno-stack-protector -fno-pic -m64 -Wall -Wextra -Ikernel -Ikernel/include -Ishared -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o` passed
- pre-existing `-Wsign-compare` warnings remain in `kernel/tests/validation/phase2_validation_test.c`

Impact:
- made execution inbox publish prerequisites true on the kernel-owned submission side instead of leaving delivery dependent on placeholder payload state
- moved `submit_execution` from slot-only anchoring to bounded ownership transfer, which makes later fixed-VA delivery honest

Notes:
- process-init inbox/payload mapping and schedule-entry descriptor publish are still separate follow-on slices.
- at the time of this submission-backing slice, `kernel/include/mm.h` still lacked an explicit NX-style flag in the common mapping header path.

### 2026-03-19

Completed Slice:
- Added explicit execution-inbox ABI constants and descriptor shape for fixed-VA delivery surfaces
- Mapped per-process execution inbox and payload windows during `proc_create_user_process()`
- Exposed NX/read-only mapping control and added a publish-safety precheck for future scheduler descriptor publication
- Added post-map verification so execution delivery surfaces fail closed instead of leaving half-mapped worker state behind

Touched Code Paths:
- `shared/abi/execution_inbox_abi.h`
- `kernel/include/execution_inbox_abi.h`
- `kernel/include/mm.h`
- `kernel/mm/paging.c`
- `kernel/include/proc.h`
- `kernel/proc/proc.c`
- `kernel/include/execution_slot.h`
- `kernel/sys/execution_slot.c`
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/execution-inbox-minimal-spec.md`
- `docs/specs/phase10b-execution-path-hardening/execution-inbox-implementation-plan.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel` passed
- `clang --target=x86_64-elf -ffreestanding -fno-stack-protector -fno-pic -m64 -Wall -Wextra -Ikernel -Ikernel/include -Ishared -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o` passed
- pre-existing `-Wsign-compare` warnings remain in `kernel/tests/validation/phase2_validation_test.c`
- pre-existing warnings remain in `kernel/proc/proc.c`, `kernel/arch/x86_64/timer.c`, and `kernel/arch/x86_64/interrupts.c`

Impact:
- made the delivery surface real without yet turning it into an authority plane
- removed the last major excuse for postponing descriptor publish ordering, because worker processes now have dedicated RO/NX inbox and payload windows

Notes:
- schedule-entry descriptor publish is still pending.
- completion/result ownership is still a later slice.

### 2026-03-19

Completed Slice:
- Published execution payload and descriptor snapshots from the scheduler pickup hook into worker inbox surfaces
- Enforced commit-point ordering `payload -> descriptor -> barrier -> delivery_seq`
- Aborted publish failures fail-closed instead of leaving a `RUNNING` slot with a stale worker latch

Touched Code Paths:
- `kernel/include/execution_slot.h`
- `kernel/sys/execution_slot.c`
- `kernel/sched/sched.h`
- `kernel/sched/sched.c`
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/execution-inbox-minimal-spec.md`
- `docs/specs/phase10b-execution-path-hardening/execution-inbox-implementation-plan.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel` passed
- `clang --target=x86_64-elf -ffreestanding -fno-stack-protector -fno-pic -m64 -Wall -Wextra -Ikernel -Ikernel/include -Ishared -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o` passed
- pre-existing `-Wsign-compare` warnings remain in `kernel/tests/validation/phase2_validation_test.c`
- pre-existing warnings remain in `kernel/proc/proc.c`, `kernel/arch/x86_64/timer.c`, and `kernel/arch/x86_64/interrupts.c`

Impact:
- turned the execution inbox from a mapped but empty surface into a real delivery projection with an explicit userspace commit point
- removed the last structural blocker before authoritative completion handoff work

Notes:
- completion/result ownership is still a later slice.
- deterministic multi-item pickup validation is still pending.

### 2026-03-19

Completed Slice:
- Replaced stale positive `submit_execution()` checks that used fixed non-live context IDs in validation
- Bound BCIB validation submission paths to a live user worker created through `ensure_validation_worker_proc()`
- Kept the validation snapshot aligned with the new live-target requirement without changing runtime behavior

Touched Code Paths:
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `clang --target=x86_64-elf -ffreestanding -fno-stack-protector -fno-pic -m64 -Wall -Wextra -Ikernel -Ikernel/include -Ishared -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o` passed
- pre-existing `-Wsign-compare` warnings remain in `kernel/tests/validation/phase2_validation_test.c`

Impact:
- removed the last stale validation path that could read hardcoded invalid context IDs as proof of successful execution submission
- kept the validation truth surface consistent with the live-target enforcement now required by `sys_v2_submit_execution()`

Notes:
- this slice changes validation inputs only; it does not change scheduler, submit, wait, or completion semantics.

### 2026-03-19

Completed Slice:
- Added a forward-only execution-path naming freeze under governance
- Added a diff-scoped naming-convention CI gate with explicit scope, deny, and legacy-allow files
- Wired the naming gate into freeze targets without forcing a repo-wide rename or global grep policy

Touched Code Paths:
- `scripts/ci/check_naming_convention.sh`
- `scripts/ci/naming-convention-scope.regex`
- `scripts/ci/naming-convention-deny.regex`
- `scripts/ci/naming-convention-legacy-allow.regex`
- `Makefile`

Touched Docs:
- `docs/governance/NAMING_CONVENTION_V1.md`
- `docs/governance/README.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `./scripts/ci/check_naming_convention.sh --evidence-dir /tmp/ayken-naming-gate` passed

Impact:
- froze future execution-path naming without destabilizing already-landed ABI and legacy scheduler/process surfaces
- turned naming guidance into a scoped enforcement rule that skips cleanly when no execution-path diff is in scope

Notes:
- the gate is intentionally diff-scoped and allowlist-backed to avoid repo-wide false positives.
- frozen existing terms remain stable; the rule applies to new execution-path additions only.

### 2026-03-19

Completed Slice:
- Softened the naming convention from hard positive prescription to guidance plus negative enforcement
- Added a decision table that separates execution-model naming from real OS/runtime primitive naming
- Restricted `LEGACY:` escape handling to comment-form annotations instead of any arbitrary added line

Touched Code Paths:
- `scripts/ci/check_naming_convention.sh`

Touched Docs:
- `docs/governance/NAMING_CONVENTION_V1.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `./scripts/ci/check_naming_convention.sh --evidence-dir /tmp/ayken-naming-gate` passed

Impact:
- kept the naming system from hardening into architecture-forcing dogma while preserving real negative enforcement
- made the execution model vocabulary clearer without treating semantically correct OS/runtime terms as forbidden everywhere

Notes:
- positive naming remains guidance, not a mandatory exact-token rule.
- `LEGACY:` now exempts only comment-form annotations.

### 2026-03-19

Completed Slice:
- Added an explicit completion-handoff decision surface for the missing `RUNNING -> COMPLETED/FAILED` half of the lifecycle
- Recorded that a dedicated completion syscall is the preferred technical answer while `interrupt_return` and scheduler-side implicit completion remain rejected
- Synced tasks, design, requirements, and runtime truth docs so completion work is now blocked on explicit ABI/governance ratification instead of ad hoc implementation

Touched Code Paths:
- none (documentation-only)

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/completion-handoff-decision.md`
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- no code validation run; documentation update only

Impact:
- removed ambiguity about what the correct completion authority surface should be
- prevented completion work from drifting into `interrupt_return`, scheduler side effects, or implicit inbox semantics

Notes:
- this slice does not ratify a new syscall number by itself.
- completion implementation remains intentionally blocked until the ABI/governance decision is made explicit.

### 2026-03-19

Completed Slice:
- Added a governance-facing ABI exception candidate for explicit completion handoff
- Froze the proposed minimal syscall shape as `sys_v2_complete_execution(execution_id, completion_code)` without bundling result ownership
- Linked the runtime decision surface to a concrete governance ratification document

Touched Code Paths:
- none (documentation-only)

Touched Docs:
- `docs/governance/ABI_EXCEPTION_COMPLETION_HANDOFF.md`
- `docs/governance/README.md`
- `docs/specs/phase10b-execution-path-hardening/completion-handoff-decision.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- no code validation run; documentation update only

Impact:
- moved completion handoff from a local design preference into an explicit ABI-exception governance candidate
- made the next runtime blocker legible as a ratification step instead of an implicit implementation argument

Notes:
- this slice still does not assign a final syscall number.
- the first completion landing remains intentionally result-free.

### 2026-03-19

Completed Slice:
- Strengthened the completion ABI exception candidate with explicit rejection consequences
- Added a recommended syscall numbering policy that prefers a single-surface v2 extension at `1011`
- Made the ratification argument harder to dismiss as optional or degradable

Touched Code Paths:
- none (documentation-only)

Touched Docs:
- `docs/governance/ABI_EXCEPTION_COMPLETION_HANDOFF.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- no code validation run; documentation update only

Impact:
- made it explicit that rejecting the exception preserves a broken lifecycle rather than a merely degraded mode
- narrowed number-policy ambiguity before ABI ratification discussion begins

Notes:
- `1011` is now the preferred ratification target, not an active assigned syscall number.

### 2026-03-19

Completed Slice:
- Added an explicit completion authority model to the ABI exception candidate
- Bound completion eligibility to the worker-owned `active_execution_id` latch
- Synced requirements, design, tasks, and runtime truth so caller authority is no longer implicit

Touched Code Paths:
- none (documentation-only)

Touched Docs:
- `docs/governance/ABI_EXCEPTION_COMPLETION_HANDOFF.md`
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- no code validation run; documentation update only

Impact:
- removed ambiguity about who is allowed to close a `RUNNING` execution slot
- made foreign or stale completion injection an explicitly fail-closed condition

Notes:
- completion remains blocked on ABI ratification; this slice only freezes the authority model.

### 2026-03-19

Completed Slice:
- Added explicit timeout-vs-completion arbitration rules to the completion governance candidate
- Froze `first terminal state wins` as the required lifecycle rule for completion landing
- Synced requirements, design, tasks, and runtime truth so terminal-state races are no longer implicit

Touched Code Paths:
- none (documentation-only)

Touched Docs:
- `docs/governance/ABI_EXCEPTION_COMPLETION_HANDOFF.md`
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- no code validation run; documentation update only

Impact:
- removed ambiguity around the `TIMEOUT` versus `COMPLETED/FAILED` race before code lands
- made dual-terminal-state outcomes explicitly forbidden at the spec/governance layer

Notes:
- this slice freezes arbitration semantics but does not implement the completion syscall.

### 2026-03-19

Completed Slice:
- Added a deterministic completion return contract to the governance candidate
- Synced requirements, design, tasks, and runtime truth so completion behavior is specified at both state and return-surface level
- Extended the completion task set to require explicit return-code tests

Touched Code Paths:
- none (documentation-only)

Touched Docs:
- `docs/governance/ABI_EXCEPTION_COMPLETION_HANDOFF.md`
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- no code validation run; documentation update only

Impact:
- removed ambiguity about how completion failure modes must surface back to the caller
- made completion determinism apply to both slot state and syscall return behavior

Notes:
- completion remains ratification-blocked; this slice only freezes the return contract.

### 2026-03-19

Completed Slice:
- Clarified that the current completion ABI depends on monotonic non-reused `execution_id` allocation rather than generation-bearing completion handles
- Marked slot `generation` as internal wait-key lifetime metadata, not part of the first completion surface
- Synced requirements, design, tasks, and runtime truth to make the no-reuse invariant explicit before code lands

Touched Code Paths:
- none (documentation-only)

Touched Docs:
- `docs/governance/ABI_EXCEPTION_COMPLETION_HANDOFF.md`
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- no code validation run; documentation update only

Impact:
- removed ambiguity about whether completion correctness depends on slot generation in the first ABI landing
- made the monotonic non-reuse execution ID invariant explicit before implementation begins

Notes:
- this slice does not add wrap handling in code yet; it freezes the contract that code must satisfy.

### 2026-03-19

Completed Slice:
- Landed `SYS_V2_COMPLETE_EXECUTION` as the ratified explicit completion surface at public number `1011`
- Implemented latch-bound `RUNNING -> COMPLETED/FAILED` terminalization with deterministic return codes and first-terminal-state-wins timeout arbitration
- Updated validation coverage for completion authority, double completion rejection, timeout-vs-completion ordering, latch clear, and monotonic non-reused execution IDs

Touched Code Paths:
- `shared/abi/syscall_v2.h`
- `kernel/sys/syscall_v2.h`
- `kernel/include/sys_v2_abi_lock.h`
- `kernel/sys/syscall_v2.c`
- `kernel/sys/execution_slot.c`
- `kernel/sys/syscall.c`
- `kernel/kernel.c`
- `kernel/tests/validation/phase2_validation_test.c`
- `kernel/tests/validation/syscall_count_test.c`
- `kernel/tests/validation/syscall_count_test.h`
- `scripts/ci/gate_abi.sh`

Touched Docs:
- `docs/governance/ABI_EXCEPTION_COMPLETION_HANDOFF.md`
- `docs/governance/README.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`
- `docs/specs/phase10b-execution-path-hardening/completion-handoff-decision.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o`
- `clang --target=x86_64-elf ... -c kernel/tests/validation/syscall_count_test.c -o /tmp/syscall_count_test.o`
- `make ci-gate-abi`

Impact:
- turned completion from a ratified contract into a live kernel surface without routing authority through scheduler or inbox side channels
- closed the missing success/failure half of the execution lifecycle while keeping result ownership explicitly out of the first landing

Notes:
- `make ci-gate-abi` reported `PASS (SKIP no ABI-affecting changes)` in standalone mode; the ABI surface itself was still updated in source and validation came from build plus focused compiles.
- result ownership, repeated successful `wait_result()` semantics, and `exit()` teardown remain follow-on work.

### 2026-03-19

Completed Slice:
- Landed initial completed-result ownership in `wait_result()` using a minimal kernel-owned receipt page
- Mapped successful results into the owner address space as read-only and non-executable
- Froze repeated successful `wait_result()` behavior to deterministic same-VA replay
- Removed stale completion-blocked language from the design spec now that explicit completion is live

Touched Code Paths:
- `kernel/include/execution_slot.h`
- `kernel/sys/execution_slot.c`
- `kernel/sys/syscall_v2.c`
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`

Validation:
- `make kernel`
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o`

Impact:
- turned `wait_result()` success from placeholder status `0` into a real mapped-result VA contract
- made result ownership deterministic without widening the ABI again
- closed the design-doc drift where completion was still described as ratification-blocked after code had already landed

Notes:
- the current result payload is a minimal kernel-owned completion receipt page, not full BCIB output materialization
- direct blocked-wait wake coverage is still missing from validation; current coverage proves timeout wake, stale execution identity rejection, and repeated same-VA replay

### 2026-03-19

Completed Slice:
- Fixed result-mapping rollback to unmap the target process PML4 instead of the active kernel root
- Reclassified the design spec from draft wording to an active implementation reference now that completion and initial result ownership have landed

Touched Code Paths:
- `kernel/include/mm.h`
- `kernel/mm/paging.c`
- `kernel/sys/syscall_v2.c`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`

Impact:
- removed a foreign-PML4 rollback bug from the first result-ownership landing
- made the remaining result lifecycle risk explicit: VA disposal is now clearly an `exit()` blocker rather than a hidden mapping bug

Notes:
- per-slot result VA selection still depends on Task 10 teardown to stay clean across long-lived slot reuse

### 2026-03-19

Completed Slice:
- Landed the first real `sys_v2_exit()` teardown path with zombie transition, owned/targeted non-terminal slot abort, result revoke, delivery-surface revoke, scheduler detachment, and no-return switch-away
- Added helper-level validation for exit teardown side effects instead of faking a direct return from `sys_v2_exit()`
- Reclassified runtime/docs truth so `exit` is now partially real rather than still described as a pure placeholder

Touched Code Paths:
- `kernel/include/execution_slot.h`
- `kernel/include/mm.h`
- `kernel/include/proc.h`
- `kernel/mm/paging.c`
- `kernel/proc/proc.c`
- `kernel/sched/sched.c`
- `kernel/sched/sched.h`
- `kernel/sys/execution_slot.c`
- `kernel/sys/syscall_v2.c`
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o`
- `git diff --check`

Impact:
- turned `exit()` from a yield-loop placeholder into a real lifecycle disposal surface for zombie transition, slot abort, result revoke, and scheduler detachment
- closed the main runtime hole that let result VA reuse remain an unbounded lifecycle leak
- kept truth surfaces honest by leaving generic map-ledger cleanup and direct blocked-wait wake proof explicitly incomplete

Notes:
- helper-level validation proves teardown side effects without invoking the no-return `sys_v2_exit()` path directly inside the snapshot harness
- generic map-ledger revoke is still pending because `map_memory` / `unmap_memory` remain placeholder lifecycle surfaces
- the live no-return exit path defers freeing the current `rsp0` backing and scheduler-owner mailbox backing; those remain future reap work rather than synchronous teardown

### 2026-03-20

Completed Slice:
- Added a syscall-entry owner-exit guard so the scheduler owner process now fails closed on `sys_v2_exit()` until an explicit handoff protocol exists
- Added validation coverage proving owner exit denial is deterministic and side-effect free
- Synced runtime/docs truth so owner-exit denial is treated as a constitutional boundary rather than an incidental bug workaround

Touched Code Paths:
- `kernel/sys/syscall_v2.c`
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o`
- `git diff --check`

Impact:
- closed the highest-risk scheduler authority hole before starting generic mapping ledger work
- made owner exit an explicit constitutional deny instead of letting teardown begin on an authority surface that still owns successor selection

Notes:
- owner exit currently returns `ESYS_V2_PERMISSION_DENIED` and emits a distinct kernel log line
- this is a temporary fail-closed boundary until a separate owner handoff protocol is designed and ratified

### 2026-03-20

Completed Slice:
- Landed the first explicit generic mapping ledger for `map_memory()` / `unmap_memory()`
- Replaced placeholder mapping success paths with real caller-root page-table mutation, capability-bound ledger recording, and caller-owned span unmap
- Extended `exit()` surface cleanup so generic ledger-backed mappings are revoked alongside result and delivery/mailbox surfaces
- Added validation coverage for page-table effects, ledger cleanup, duplicate-map rejection, foreign-unmap denial, and exit-time generic revoke

Touched Code Paths:
- `kernel/include/proc.h`
- `kernel/proc/proc.c`
- `shared/abi/capability.h`
- `kernel/sys/capability_manager.c`
- `kernel/sys/syscall_v2.c`
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o`
- `git diff --check`

Impact:
- turned `map_memory()` / `unmap_memory()` from placeholder reachability surfaces into real explicit mapping primitives with page-table side effects and capability-backed ownership checks
- closed the remaining generic mapping gap in `exit()` so result and explicit user mappings now tear down under one deterministic lifecycle path
- moved the next runtime bottlenecks away from mapping truth and back onto blocked-wait proof, IRQ-timeout proof, and fuller process-memory disposal

Notes:
- the generic ledger currently covers only explicit `map_memory()` mappings; result, inbox, payload-window, and mailbox surfaces still keep their dedicated lifecycle paths
- full process-memory teardown remains incomplete because text, stack, page-table hierarchy, scheduler-owner mailbox handoff, and current-rsp0 deferred reap are still outside this slice

### 2026-03-20

Completed Slice:
- Added a direct blocked-wait wake harness for `sys_v2_wait_result()` using real scheduler block/requeue behavior
- Proved that a waiter blocked on the canonical slot `wait_key` survives a spurious wake attempt on a different key and only resumes after the matching terminal wake path fires
- Synced runtime truth so blocked-wait proof is no longer listed as the primary missing wait-result validation gap

Touched Code Paths:
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o`
- `git diff --check`

Impact:
- upgraded `wait_result()` proof from terminal-state observation to direct scheduler-mediated block/wake evidence
- added spurious-wake resistance coverage at the wait-key identity layer, reducing the risk of silent stale or foreign wake regressions
- narrowed the next runtime-proof bottleneck to IRQ-driven timeout evidence rather than generic wait-path uncertainty

Notes:
- the direct harness currently proves blocked wait release through the abort path; timeout wake still needs its separate IRQ-driven proof slice

### 2026-03-20

Completed Slice:
- Added a direct IRQ-driven timeout harness for `sys_v2_wait_result()` using the real timer ISR entry point
- Proved that scheduler yields alone do not advance timeout terminalization before the IRQ path runs
- Closed Task 9.4 and moved the next validation bottleneck from timeout proof to fuller process-memory teardown

Touched Code Paths:
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o`
- `git diff --check`

Impact:
- upgraded timeout validation from helper-level terminal-state observation to a direct timer-ISR proof
- demonstrated that timeout progression is not syscall-spin-driven by showing scheduler yields do not release the waiter before the IRQ path fires
- cleared the last major proof gap in the core execution lifecycle, leaving fuller process-memory teardown and full BCIB output materialization as the next substantive runtime debts

Notes:
- the timeout harness intentionally keeps the target executor non-runnable so the proof isolates IRQ timeout authority from worker pickup behavior

### 2026-03-20

Completed Slice:
- Removed residual runtime-truth wording drift from `SYSCALL_RUNTIME_REALITY.md`
- Updated the doc timestamp and narrowed the remaining gaps from generic `exit` missing language to the real open items: full process-memory disposal and full BCIB output materialization

Touched Code Paths:
- none (docs-only slice)

Touched Docs:
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- not rerun (docs-only wording sync)

Impact:
- kept runtime truth aligned with the actual post-Task-11 kernel state instead of letting stale “exit teardown missing” language survive after teardown became partially real
- narrowed the visible remaining blockers to the real proof and disposal gaps

Notes:
- this slice intentionally changed wording only; no kernel behavior changed

### 2026-03-20

Completed Slice:
- Extended `exit` teardown from explicit-surface revoke to fuller lower-half user-memory destruction
- Added deferred reap for the active process root PML4 and current `rsp0` backing on later safe scheduler paths
- Made `proc_create_user_process()` fail-closed with transactional cleanup instead of leaking partial user address spaces

Touched Code Paths:
- `kernel/include/mm/user_as.h`
- `kernel/mm/user_as.c`
- `kernel/include/proc.h`
- `kernel/proc/proc.c`
- `kernel/sched/sched.c`
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o`
- `git diff --check`

Impact:
- moved `exit` teardown beyond explicit mapping revoke by reclaiming user text, stack, canary, and lower-half page tables
- replaced permanently leaked active-process root/rsp0 state with deferred reap on later safe scheduler paths
- converted partial user-process construction failures from silent address-space leaks into transactional cleanup

Notes:
- non-owner processes now destroy remaining lower-half user memory during teardown, but scheduler-owner mailbox handoff and direct no-return `sys_v2_exit()` runtime proof remain separate follow-on work

### 2026-03-20

Completed Slice:
- Hardened `proc_create_user_process()` against null kernel-stack allocation during Ring3 `rsp0` bring-up

Touched Code Paths:
- `kernel/proc/proc.c`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `git diff --check`

Impact:
- closed a real fail-closed gap in the user-process bring-up path by rejecting null `kmalloc(4096)` before publishing `rsp0`
- clarified that the earlier teardown slice did not leave the specific post-`rsp0` rollback leak window that would have existed if `rsp0` were published after the supervisor-stack mapping loop

Notes:
- this was a narrow hardening follow-up; the next substantive runtime feature work remains full BCIB output materialization

### 2026-03-20

Completed Slice:
- Replaced the minimal completed-result receipt placeholder with full bounded kernel-owned BCIB byte materialization
- Mapped successful `wait_result()` ownership as a multi-page RO/NX result window with deterministic same-VA replay
- Extended exit-side result revoke to clear the full fixed result window instead of only the first page

Touched Code Paths:
- `kernel/include/execution_slot.h`
- `kernel/sys/execution_slot.c`
- `kernel/sys/syscall_v2.c`
- `kernel/proc/proc.c`
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o`

Impact:
- removed the last receipt-page placeholder from successful `wait_result()` by reusing the slot-owned bounded BCIB backing as result backing
- kept the ABI stable while upgrading result ownership from one-page receipt semantics to full multi-page payload semantics
- made exit-side result cleanup match the full fixed result window instead of only the first mapped page

Notes:
- this slice does not introduce a distinct executor-written output plane; the mapped bytes are the kernel-owned BCIB backing currently preserved by the completed slot
- the next substantive runtime debts are now direct no-return `sys_v2_exit()` proof and scheduler-owner handoff/reap

### 2026-03-20

Completed Slice:
- Added a direct non-owner `sys_v2_exit()` no-return runtime harness to the validation snapshot
- Proved that real `sys_v2_exit()` switches away without returning, zombifies the caller, destroys lower-half mappings, and completes deferred reap on a later safe drain

Touched Code Paths:
- `kernel/sched/sched.h`
- `kernel/sched/sched.c`
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o`
- `git diff --check`

Impact:
- closed the last major proof gap around `sys_v2_exit()` for ordinary non-owner processes by validating a real no-return switch-away path instead of only helper-level teardown effects
- shifted the remaining exit-side runtime debt from generic teardown correctness to scheduler-owner handoff/reap

Notes:
- the harness uses a validation-only forced-successor seam inside `sched_exit_current()` to make the direct no-return proof safe and deterministic
- this is not a general scheduler-owner handoff mechanism; owner exit remains fail-closed until a separate protocol is designed

### 2026-03-20

Completed Slice:
- Isolated scheduler-owner handoff/reap as a separate governance-controlled follow-on
- Froze the minimum acceptable owner-transfer and reap semantics in a review candidate instead of silently mutating mailbox v1/C1

Touched Code Paths:
- none (docs-only slice)

Touched Docs:
- `docs/governance/SCHEDULER_OWNER_HANDOFF_REAP_CANDIDATE.md`
- `docs/governance/README.md`
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- docs-only; no behavior change

Impact:
- prevented scheduler-owner lifecycle closure from drifting into an unratified runtime hack
- made the remaining top-priority debt explicit: owner handoff/reap ratification first, then runtime proof, then userspace ABI correction

Notes:
- mailbox v1/C1 remains the only active normative owner-authority model
- owner exit stays fail-closed until the candidate is ratified into an authorized runtime mechanism

### 2026-03-20

Completed Slice:
- Compared the two remaining valid ratification paths for scheduler-owner handoff/reap
- Narrowed the next governance decision to either a mailbox-v1 transfer exception or a promoted mailbox-v2/C2 owner-transfer path

Touched Code Paths:
- none (docs-only slice)

Touched Docs:
- `docs/governance/SCHEDULER_OWNER_HANDOFF_RATIFICATION_OPTIONS.md`
- `docs/governance/README.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- docs-only; no behavior change

Impact:
- removed ambiguity about what “ratification” now means for owner handoff/reap
- made the recommended path explicit: prefer a narrow mailbox-v1 transfer exception unless the project is ready to promote C2 authority now

Notes:
- this slice does not choose or ratify a path by itself
- owner exit remains fail-closed until one of the two paths is selected explicitly

### 2026-03-20

Completed Slice:
- Drafted the narrow mailbox-v1 owner-transfer exception as the preferred ratification core for scheduler-owner handoff/reap

Touched Code Paths:
- none (docs-only slice)

Touched Docs:
- `docs/governance/MAILBOX_V1_OWNER_TRANSFER_EXCEPTION.md`
- `docs/governance/README.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- docs-only; no behavior change

Impact:
- turned the recommended Option A path from a comparison result into a concrete ratification candidate
- froze the narrowest governance surface that can close owner lifecycle continuity without prematurely promoting mailbox v2/C2

Notes:
- this slice still does not ratify transfer behavior; owner exit remains fail-closed
- the next governance action is now explicit: approve or reject `docs/governance/MAILBOX_V1_OWNER_TRANSFER_EXCEPTION.md`

### 2026-03-20

Completed Slice:
- Tightened the owner-transfer ratification candidate by making the commit point explicit

Touched Code Paths:
- none (docs-only slice)

Touched Docs:
- `docs/governance/MAILBOX_V1_OWNER_TRANSFER_EXCEPTION.md`
- `docs/governance/SCHEDULER_OWNER_HANDOFF_REAP_CANDIDATE.md`
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- docs-only; no behavior change

Impact:
- removed the last major ambiguity around owner-transfer atomics by binding commit to the scheduler dispatch boundary
- made it explicit that authority swap is not allowed to happen opportunistically in syscall bodies or timer validation paths

Notes:
- this slice still does not ratify or activate owner transfer behavior
- owner exit remains fail-closed until the transfer exception is explicitly approved

### 2026-03-20

Completed Slice:
- Ratified the narrow mailbox-v1 owner-transfer exception and landed the first runtime proof slice
- Added runtime active-owner tracking plus dispatch-boundary-only owner commit under a pending transfer request
- Proved successor-authority mailbox application through a narrow validation harness without overstating full old-owner lifecycle closure

Touched Code Paths:
- `kernel/sched/sched.c`
- `kernel/sched/sched.h`
- `kernel/proc/proc.c`
- `kernel/sys/syscall_v2.c`
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/governance/MAILBOX_V1_OWNER_TRANSFER_EXCEPTION.md`
- `docs/governance/SCHEDULER_OWNER_HANDOFF_REAP_CANDIDATE.md`
- `docs/governance/README.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o`
- `git diff --check`

Impact:
- turned owner transfer from a ratification-ready candidate into the active narrow governance exception
- proved that owner authority can commit only at a scheduler dispatch boundary and that the successor mailbox becomes authoritative after commit
- eliminated the remaining ambiguity between “ratified mechanism exists” and “full owner lifecycle closure is already proven”

Notes:
- this slice does not yet prove old-owner no-return exit/reap follow-through after the authority commit
- the active-owner `sys_v2_exit()` deny remains fail-closed in production runtime until that follow-on proof lands

### 2026-03-22

Completed Slice:
- Closed `11A.4.2` with a narrow runtime proof for old-owner no-return exit/reap follow-through after dispatch-boundary owner commit
- Proved that successor authority remains sole and active while the retired old owner takes the no-return exit path and deferred reap completes
- Added direct validation coverage for stale old-owner mailbox publish rejection after commit

Touched Code Paths:
- `kernel/sched/sched.c`
- `kernel/sched/sched.h`
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o`
- `git diff --check`

Impact:
- completed the narrow owner-handoff runtime proof promised by the ratified mailbox-v1 transfer exception
- moved the remaining top-level runtime debt from owner authority continuity to userspace `context_id` ABI correctness
- showed that old-owner mailbox authority stays fail-closed after commit while successor authority drives scheduling and reap

Notes:
- this remains a narrow, validation-seamed proof for the ratified mailbox-v1 transfer exception, not a general mailbox-v2/C2 promotion
- active-owner `sys_v2_exit()` still denies when no ratified transfer request has committed

### 2026-03-21

Completed Slice:
- Closed Task 12 by fixing `userspace/bcib-runtime` to pass a real target `context_id`
- Removed local userspace `execution_id` fabrication and treated the syscall return as the authoritative kernel-owned `execution_id`
- Added unit-test coverage that directly proves wrapper/kernel agreement on submit argument meaning

Touched Code Paths:
- `userspace/bcib-runtime/src/executor.rs`
- `userspace/bcib-runtime/src/bin/dispatcher.rs`
- `userspace/bcib-runtime/examples/submit_execution_demo.rs`

Touched Docs:
- `userspace/bcib-runtime/ARCHITECTURE.md`
- `userspace/bcib-runtime/SUBMIT_EXECUTION_IMPLEMENTATION.md`
- `docs/development/BCIB_SUBMISSION_PROTOCOL.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `cargo test -p bcib-runtime`

Impact:
- aligned the canonical userspace wrapper with the kernel ABI contract for `submit_execution(..., context_id)`
- closed the last remaining userspace ABI reinterpretation gap in the execution path
- moved the remaining follow-on work from ABI correctness to optional output-plane evolution and broader end-to-end proof

Notes:
- runtime kernel behavior did not change in this slice; only the userspace wrapper and its tests moved
- the dispatcher/example path now requires `AYKEN_TARGET_CONTEXT_ID` when actual syscall submission is requested

### 2026-03-21

Completed Slice:
- Replaced the old placeholder Phase 2 integration test with a semantic end-to-end execution scenario
- Closed `13.1`, `13.2`, and `13.3` by proving `map -> submit -> pickup -> complete -> wait -> exit` in one validation flow
- Added explicit assertions for foreign wait denial, deterministic same-VA replay, result revoke on exit, generic mapping revoke on exit, and slot release after owner exit

Touched Code Paths:
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o`
- `make kernel`
- `git diff --check`

Impact:
- turned the old Phase 2 integration placeholder into a real semantic proof package
- connected submit/pickup/complete/wait/exit into one lifecycle scenario instead of relying only on isolated surface checks
- moved the remaining validation-quality debt down to the negative timeout/abort scenario

Notes:
- `13.4` remains open; timeout/wake proof exists, but the dedicated negative timeout-to-cleanup scenario is still separate work
- this slice did not change kernel runtime semantics; it upgraded validation quality and truth-surface accuracy

### 2026-03-21

Completed Slice:
- Closed `13.4` and completed Task 13 with a negative semantic timeout lifecycle harness
- Proved `submit -> pickup -> running -> timeout IRQ -> wake -> repeated timeout -> foreign wait denial -> cleanup`
- Extended validation summary and runtime truth so negative-path proof is no longer described as pending

Touched Code Paths:
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o`
- `make kernel`
- `git diff --check`

Impact:
- completed the validation-quality checklist for Phase 10-B/10-C runtime stabilization
- added direct proof that the timeout failure path stays deterministic, owner-bound, and cleanly reclaimable after wake
- removed the last major proof gap between happy-path lifecycle validation and negative-path cleanup validation

Notes:
- this slice still does not introduce a distinct executor-written output plane
- runtime semantics did not change; only proof coverage and truth-surface accuracy changed

### 2026-03-21

Completed Slice:
- Froze the first distinct execution output-plane candidate without widening the syscall surface
- Added a shared output-window ABI header and a minimal spec for fixed-VA output, completion-time validation, and frozen result publication
- Added Task 15 and explicitly closed `15.1` so future runtime work can land in bounded slices instead of reopening the already-closed execution lifecycle contract

Touched Code Paths:
- `shared/abi/execution_output_abi.h`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/execution-output-minimal-spec.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- no behavior change; no kernel/runtime validation rerun
- `git diff --check`

Impact:
- turned the future "distinct output plane" work into a frozen minimal contract instead of an open-ended idea
- kept the next semantic expansion bounded to fixed-VA output, completion-time validation, and frozen publish without a new write syscall

Notes:
- runtime behavior is unchanged in this slice
- the current kernel still re-exposes the completed slot's kernel-owned BCIB backing on successful waits

### 2026-03-21

Completed Slice:
- Landed `15.2` and `15.3` by adding a fixed writable/NX worker output window and binding slot-owned output backing at pickup time
- Kept that backing slot-owned rather than worker-owned, and zero-filled it before execution begins
- Added validation that the output window is distinct from the input payload backing and that terminalization clears the worker binding

Touched Code Paths:
- `kernel/include/execution_slot.h`
- `kernel/include/proc.h`
- `kernel/include/execution_output_abi.h`
- `kernel/proc/proc.c`
- `kernel/sys/execution_slot.c`
- `kernel/sched/sched.c`
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o`
- `git diff --check`

Impact:
- turned the frozen output-plane candidate into a real runtime surface without changing submit/complete/wait ABI
- established slot-owned writable/NX output backing as the canonical worker-facing output surface for the next completion-time validation slice

Notes:
- `wait_result()` still publishes the preserved slot-owned BCIB backing
- output-header validation and frozen output publication remain open under `15.4` and `15.5`

### 2026-03-21

Completed Slice:
- Hardened the worker output-window bind path so stale/double bind attempts fail closed instead of silently rebinding
- Strengthened validation from first-page distinctness to full-frame-set non-alias checks between output, payload, and BCIB backing
- Froze the `15.4` fail-closed completion contract for invalid output metadata

Touched Code Paths:
- `kernel/proc/proc.c`
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/execution-output-minimal-spec.md`
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o`
- `git diff --check`

Impact:
- removed the silent rebind path from the new output-window surface before any future multi-path evolution
- raised the proof level for slot-owned output isolation from single-page sampling to full-frame-set coverage

Notes:
- runtime behavior for result publication is unchanged
- `15.4` and `15.5` remain the next actual semantic landing slices

### 2026-03-21

Completed Slice:
- Closed `15.4` by validating output header magic, ABI version, and bounded `bytes_written` during `complete_execution(COMPLETED)`
- Wired invalid output metadata to fail-closed terminalization as `FAILED` with latch clear and waiter wake rather than silent acceptance
- Extended validation so success completions now write a valid output header and malformed output requests are observed as deterministic `INVALID_STATE`

Touched Code Paths:
- `kernel/include/execution_slot.h`
- `kernel/sys/execution_slot.c`
- `kernel/sys/syscall_v2.c`
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/execution-output-minimal-spec.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o`
- `git diff --check`

Impact:
- converted the previously frozen fail-closed output-validation contract into real runtime behavior
- made successful completion depend on valid output metadata instead of accepting arbitrary writable scratch bytes

Notes:
- `wait_result()` still publishes the preserved BCIB backing rather than frozen output bytes
- `15.5` and `15.6` remain open

### 2026-03-21

Completed Slice:
- Closed `15.5` by switching result preparation from preserved BCIB backing to the slot-owned frozen validated output backing
- Closed `15.6` by extending semantic validation across valid output publication, invalid magic, invalid ABI version, overflowing `bytes_written`, same-VA replay, and cleanup-sensitive output/result teardown
- Updated runtime truth so successful waits now explicitly publish frozen output bytes rather than reusing BCIB input bytes
- Added trailing-byte sealing so bytes beyond the declared logical output size inside the mapped result frame span are zeroed before publication

Touched Code Paths:
- `kernel/sys/execution_slot.c`
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/execution-output-minimal-spec.md`
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o`
- `git diff --check`

Impact:
- completed the first distinct output-plane landing without widening the syscall surface
- moved result truth from reused BCIB input bytes to frozen validated executor output bytes

Notes:
- QEMU/runtime boot validation remains out of scope for this slice

### 2026-03-21

Completed Slice:
- Opened the docs-first `Structured Output Minimal Spec` as the next semantic layer above the landed raw output plane
- Froze the kernel boundary for that future layer: payload remains opaque, kernel enforces only structure/version/kind/bounds, and semantic interpretation stays in userland
- Added a dedicated task track for structured output so future widening stays additive instead of mutating the landed raw output contract in place

Touched Code Paths:
- none (docs-only slice)

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/structured-output-minimal-spec.md`
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- not rerun (docs-only slice; runtime behavior unchanged)
- `git diff --check`

Impact:
- moved the next semantic widening from ad hoc discussion into a bounded, repo-tracked contract
- locked the "kernel enforces structure, userland owns meaning" rule before any structured runtime code lands

Notes:
- no runtime or ABI behavior changed in this slice

### 2026-03-22

Completed Slice:
- stabilized the canonical Phase10-A2 runtime gate by moving the QEMU stop condition from early boot completion to the final canonical A2 marker `P10_RING3_USER_CODE`, which removed the false-positive evidence cut
- root-caused the remaining fresh-run crash to user CR3 visibility of low-half kmalloc/proc metadata and fixed it by mirroring the current kernel heap supervisor-only into user PML4 roots
- fixed mixed low-half page-table trees so parent entries upgrade to `USER=1` when user leaves share a subtree with supervisor-only kernel leaves, while kernel heap leaf PTEs remain supervisor-only
- reran the official `make ci-gate-syscall-semantics-phase10b` chain successfully with fresh same-run Phase10-A2 evidence and the nested fail-closed replay proof artifacts copied into top-level reports

Touched Code Paths:
- `scripts/ci/gate_ring3_execution_phase10a2.sh`
- `kernel/include/mm.h`
- `kernel/mm/kheap.c`
- `kernel/mm/paging.c`
- `kernel/mm/user_as.c`
- `Makefile`

Touched Docs:
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `USER_MINIMAL_MODE=phase10a2 KERNEL_PROFILE=validation AYKEN_C2_STRICT_MARKERS=0 AYKEN_MB_SELFTEST=1 AYKEN_GATE4_POLICY_TEST=0 AYKEN_SCHED_BOOTSTRAP_POLICY=0 AYKEN_CR3_PCID=0 bash scripts/ci/gate_ring3_execution_phase10a2.sh --evidence-dir /tmp/phase10a2-fresh --qemu-timeout 35`
- `make ci-gate-syscall-semantics-phase10b`
- `git diff --check`

Impact:
- removed the stale claim that official Phase10-B closure was blocked by fresh Phase10-A2 instability; the canonical same-run chain now passes locally again
- converted the low-half kmalloc visibility problem from implicit behavior into explicit temporary compatibility scaffolding, which keeps the debt technical and bounded rather than architectural and hidden

Notes:
- the supervisor-only heap mirror is an explicit temporary bridge, not a new architectural destination; the follow-on cleanup is to move kmalloc/proc metadata fully out of the low half

### 2026-03-22

Multi-Exit Proof Evolution Summary:
- started from a single-witness terminal exit proof lane bound to one authoritative `exit_pid`
- widened the runtime proof from a single PTE observation to global lower-half cleanup counts (`lower_half_roots`, `lower_half_leaves`, `lower_half_user_leaves`) at `exit_teardown_post`
- added a dedicated multi-exit lineage lane so every authoritative `[[AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_LINEAGE]]` witness gets its own nested scaffold proof
- normalized the lineage artifact schema to canonical `armed_rows` / `lineage_rows` names so gate output, review tooling, and manual inspection share one truth surface
- parameterized the validation-only workload with `AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT` (default `2`) and revalidated the lane with `N=3`
- added a dedicated interleaving proof lane that pre-creates the full exit set before first teardown, then requires canonical `prepared -> armed -> lineage` ordering plus per-lineage global lower-half cleanup under overlap pressure
- current closure line: deterministic parametric `N`-exit lineage coverage and overlap-pressure interleaving proof are landed; alias-aware full address-space leak proof remains follow-on work

Completed Slice:
- added a dedicated `ci-gate-low-half-kheap-exit-proof` workload that drives a non-owner Ring3 process through `syscall -> exit` and now observes the terminal `exit_teardown_pre/post` runtime proof slice under QEMU
- fixed the two runtime authority gaps that the exit-proof workload exposed: scheduler owner-mailbox reads now snapshot safely across CR3 boundaries, and current-process exit teardown now runs its user-root dismantle path under kernel CR3 instead of faulting on self-owned page-table walks
- made the validation-only exit selftest resilient to PID1 re-entry by turning it into an explicit armed/completed state machine instead of assuming a single kernel-thread stack continuation
- taught the exit-proof gate to pin the nested scaffold proof to the explicit `exit_pid` carried by `[[AYKEN_LOW_HALF_KHEAP_EXIT_SELFTEST_OK]]`, so future duplicate-phase / multi-exit workloads do not rely on heuristic PID auto-selection
- extended the report surface so exit-proof evidence now records all observed runtime PIDs, all terminal-slice PIDs, and fails if any unexpected terminal witness appears outside the authoritative selected exit PID
- extended the runtime proof marker with global lower-half teardown counts (`lower_half_roots`, `lower_half_leaves`, `lower_half_user_leaves`) and taught the nested scaffold gate plus dedicated exit-proof gate to require all three to collapse to zero at `exit_teardown_post`
- added a dedicated `ci-gate-low-half-kheap-multi-exit-proof` lane that drives a deterministic parametric `N`-exit validation workload (default `N=2`), emits authoritative `[[AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_LINEAGE]]` markers, and runs one nested scaffold proof per enumerated `exit_pid`
- added a dedicated `ci-gate-low-half-kheap-interleaving-proof` lane that pre-creates the full validation-only exit set before first teardown, proves canonical `prepared -> armed -> lineage` ordering for every slot, and runs one nested scaffold proof per enumerated `exit_pid` under overlap pressure
- added the `terminal_lineage` scaffold phase profile so multi-exit coverage can require `create + exit_teardown_pre/post + global lower-half cleanup` per exit lineage without over-claiming timer/syscall coverage for every exit worker
- normalized `lineage_contract.json` to canonical `armed_rows` / `lineage_rows` field names so review tooling and manual inspection do not drift from the gate schema

Touched Code Paths:
- `kernel/proc/proc.c`
- `kernel/sched/sched.c`
- `kernel/mm/paging.c`
- `kernel/ring3_jump.c`
- `scripts/ci/gate_low_half_kheap_exit_proof.sh`
- `scripts/ci/gate_low_half_kheap_multi_exit_proof.sh`
- `scripts/ci/gate_low_half_kheap_interleaving_proof.sh`
- `scripts/ci/gate_low_half_kheap_scaffold.sh`
- `Makefile`

Touched Docs:
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`
- `docs/specs/phase11-verification-substrate/requirements.md`

Validation:
- `make kernel KERNEL_PROFILE=validation`
- `make ci-gate-low-half-kheap-exit-proof RUN_ID=local-low-half-kheap-exit-proof`
- `make ci-gate-low-half-kheap-exit-proof RUN_ID=local-low-half-kheap-exit-proof-global-leak`
- `make ci-gate-low-half-kheap-multi-exit-proof RUN_ID=local-low-half-kheap-multi-exit-proof-v7`
- `make ci-gate-low-half-kheap-multi-exit-proof RUN_ID=local-low-half-kheap-multi-exit-proof-n3 AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT=3`
- `make ci-gate-low-half-kheap-interleaving-proof RUN_ID=local-low-half-kheap-interleaving-proof-n3 AYKEN_LOW_HALF_KHEAP_INTERLEAVING_PROOF_COUNT=3`
- `make ci-gate-low-half-kheap-exit-proof RUN_ID=local-low-half-kheap-exit-proof-regression`
- `git diff --check`

Impact:
- promoted the low-half scaffold proof from "multi-point with optional terminal slice" to a QEMU-backed terminal teardown proof with deterministic artifact production
- converted two user-CR3-sensitive paging/scheduler assumptions into explicit kernel-CR3 authority transitions, which removes hidden runtime dependence on low identity aliases during owner-mailbox reads and self-teardown
- gave Phase10 a dedicated terminal teardown proof for the low-half scaffold debt instead of relying on incidental teardown coverage in unrelated workloads
- improved debt observability and terminal proof quality without changing the fact that the low-half scaffold remains an active debt until higher-half migration lands
- made the dedicated exit-proof gate explicitly single-witness and fail-closed on stray terminal witnesses, which keeps future multi-exit widening from silently degrading into heuristic coverage

Notes:
- the terminal exit proof is validation-only and does not change the normal Phase10-A2 workload; it exists to prove the teardown invariant without widening the baseline A2 scenario

### 2026-03-22

Completed Slice:
- synchronized the public truth surface around the low-half heap compatibility bridge so headers and Phase10B spec docs no longer imply that user address-space creation only copies the kernel half
- made the temporary scaffold explicit in the requirements/design layer to keep the debt visible and bounded

Touched Code Paths:
- `kernel/include/mm/user_as.h`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `git diff --check`

Impact:
- reduced documentation drift around the current memory model so reviews do not confuse the temporary supervisor-only heap mirror with the target architecture

Notes:
- no runtime behavior changed in this slice

### 2026-03-22

Completed Slice:
- added a dedicated low-half kernel-heap scaffold CI gate for Phase10 visibility mode and a hard-fail Phase11 closure gate that forbids the scaffold entirely
- bound the current scaffold debt to explicit repo-enforced truth instead of documentation-only reminders
- made Phase11 closure criteria fail-closed on `ci-gate-no-low-half-kernel-dependency`
- strengthened the scaffold gate from positive fragment checks into a semantic truth-integrity gate with contradictory-truth detection and an `AYKEN_KHEAP_START`/`KERNEL_VIRT_BASE` address-model anchor
- added same-run Phase10-A2 runtime proof consumption so the scaffold gate now validates the actual user-root PTE for `AYKEN_KHEAP_START` from QEMU marker evidence instead of relying on static truth alone
- widened that runtime proof from one snapshot into a multi-point temporal slice over `create`, `timer_irq`, and `syscall_entry`, with optional `exit_teardown_pre/post` terminal validation when the workload actually traverses teardown
- taught the scaffold gate to verify temporal invariants (`user=0` at every observed phase, strict proof ordering, single-root consistency, and no low-half reversion) instead of only a last-snapshot PTE read

Touched Code Paths:
- `kernel/include/mm.h`
- `kernel/include/proc.h`
- `kernel/proc/proc.c`
- `kernel/sys/syscall.c`
- `kernel/arch/x86_64/timer.c`
- `scripts/ci/gate_low_half_kheap_scaffold.sh`
- `Makefile`

Touched Docs:
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`
- `docs/specs/phase11-verification-substrate/requirements.md`
- `docs/specs/phase11-verification-substrate/tasks.md`

Validation:
- `make kernel KERNEL_PROFILE=validation`
- `make ci-gate-low-half-kheap-scaffold RUN_ID=local-low-half-kheap-runtime-proof`
- `make ci-gate-low-half-kheap-scaffold RUN_ID=local-low-half-kheap-guard`
- `bash scripts/ci/gate_low_half_kheap_scaffold.sh --evidence-dir /tmp/phase10-low-half-kheap-scaffold --phase10a2-evidence out/evidence/run-local-low-half-kheap-runtime-proof/gates/ring3-execution-phase10a2 --mode allow`
- `bash scripts/ci/gate_low_half_kheap_scaffold.sh --evidence-dir /tmp/phase11-low-half-kheap-runtime-block --phase10a2-evidence out/evidence/run-local-low-half-kheap-runtime-proof/gates/ring3-execution-phase10a2 --mode forbid` (expected fail)
- `bash -n scripts/ci/gate_low_half_kheap_scaffold.sh`
- `git diff --check`

Impact:
- turned the temporary low-half heap mirror from a narrative warning into an explicit CI-tracked compatibility scaffold
- reserved Phase11 closure for the cleaned memory model instead of allowing the scaffold to silently normalize
- made false "scaffold removed" drift harder by rejecting contradictory current-truth statements and by checking whether the kernel heap still lives in the low half
- upgraded the gate from static truth enforcement to runtime-backed truth enforcement by requiring the same-run A2 boot evidence to prove the current user-root mapping state directly
- moved the scaffold proof from single-point observation toward temporal runtime truth, so Phase10 now measures the memory-model invariant across the actual syscall/IRQ lifecycle instead of only at process creation

Notes:
- the new Phase10 visibility gate is intentionally PASS-with-scaffold while the Phase11 closure gate is intentionally FAIL-with-scaffold

### 2026-03-22

Completed Slice:
- froze the first minimal replayable evidence format for the landed fail-closed proof slice so the normalized replay artifact set is now explicit and versioned instead of remaining an implicit debug transcript convention
- surfaced the fail-closed replay artifact bundle directly from the Phase 10-B semantic gate and official `make ci-gate-syscall-semantics-phase10b` reports so same-run proof consumers do not need to infer paths manually
- documented the normalization rule that replay identity is derived from ordered transition rows plus normalized local ticks, not raw boot-time execution IDs or absolute tick values

Touched Code Paths:
- `scripts/ci/gate_execution_fail_closed_proof.sh`
- `scripts/ci/gate_syscall_semantics_phase10b.sh`
- `tools/ci/validate_execution_fail_closed_proof.py`
- `Makefile`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/fail-closed-replay-minimal-spec.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `python3 -m py_compile tools/ci/validate_execution_fail_closed_proof.py tools/ci/validate_syscall_semantics_phase10b.py`
- `bash -n scripts/ci/gate_execution_fail_closed_proof.sh scripts/ci/gate_syscall_semantics_phase10b.sh`
- `bash scripts/ci/gate_execution_fail_closed_proof.sh --evidence-dir /tmp/phase10b-fail-closed-proof-replay --qemu-timeout 20`
- `bash scripts/ci/gate_syscall_semantics_phase10b.sh --evidence-dir /tmp/phase10b-colocated-run/gates/syscall-semantics-phase10b --phase10a2-evidence /tmp/phase10b-colocated-run/gates/ring3-execution-phase10a2 --mode negative --proof-qemu-timeout 20 --require-colocated-phase10a2`
- `make ci-gate-syscall-semantics-phase10b`
- `git diff --check`

Impact:
- converted the landed fail-closed proof export from "narrow proof engine with ad hoc artifacts" into a frozen replay-evidence contract that future Phase 10-B / Phase 11 work must extend additively
- reduced the risk that replay proof work drifts into log-shaped technical debt before multi-execution proof widening lands

Notes:
- this freeze is intentionally narrow: it stabilizes the current single-slice fail-closed replay contract, not the future multi-execution proof surface

### 2026-03-21

Completed Slice:
- added runtime proof export for fail-closed execution-slot panics so the authoritative invalid-transition path now emits a debugcon transcript, invariant verdict, per-slot trace rows, and a SHA-256 transcript hash before halting
- added a dedicated validation-only boot selftest that deterministically drives `CREATED -> READY` and then forces an invalid authoritative transition attempt to prove the fail-closed path under QEMU
- added an adversarial multi-execution validation slice covering repeated owner `wait_result` replay floods, stale/unknown execution-ID rejection, double-finalize rejection, and pickup-vs-exit collision handling
- integrated the new QEMU proof slice into the canonical Phase 10-B syscall semantics gate so closure evidence now combines source guard policy with runtime-observed fail-closed proof
- tightened the canonical Phase 10-B gate so the official CI path requires a co-located same-run Phase10-A2 evidence directory, while standalone review invocations now report when they are consuming external reused evidence

Touched Code Paths:
- `kernel/include/execution_slot.h`
- `kernel/sys/execution_slot.c`
- `kernel/kernel.c`
- `kernel/tests/validation/phase2_validation_test.c`
- `scripts/ci/gate_execution_fail_closed_proof.sh`
- `scripts/ci/gate_syscall_semantics_phase10b.sh`
- `tools/ci/validate_execution_fail_closed_proof.py`
- `Makefile`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `bash scripts/ci/check_phase10b_execution_hardening.sh --evidence-dir /tmp/phase10b-hardening-guard`
- `bash scripts/ci/gate_execution_fail_closed_proof.sh --evidence-dir /tmp/phase10b-fail-closed-proof --qemu-timeout 20`
- `bash scripts/ci/gate_syscall_semantics_phase10b.sh --evidence-dir /tmp/phase10b-main-gate --phase10a2-evidence evidence/run-local-p10-b-fix-serial/gates/ring3-execution-phase10a2 --mode negative --proof-qemu-timeout 20`
- `python3 -m py_compile tools/ci/validate_execution_fail_closed_proof.py tools/ci/validate_syscall_semantics_phase10b.py`
- `bash -n scripts/ci/gate_execution_fail_closed_proof.sh scripts/ci/gate_syscall_semantics_phase10b.sh`
- `git diff --check`

Impact:
- moved Phase 10-B closure evidence beyond source-level hardening into a runtime-observed proof transcript for the authoritative panic path
- added direct adversarial coverage for multi-execution collision and replay pressure rather than relying only on single-slice lifecycle tests

Notes:
- the new runtime proof slice is intentionally narrow and deterministic; it proves fail-closed export on one authoritative invalid transition, not full replay portability yet

### 2026-03-21

Completed Slice:
- Closed `2.5` with direct monotonic forward-progress validation by proving `sys_v2_time_query(TIME_QUERY_MONOTONIC)` advances after a real timer IRQ tick
- Closed `3.5` by tightening blocked-wait coverage from "different wait object" to explicit stale-generation rejection against the slot-backed wait key
- Closed `4.2` / `4.3` with a source-level hardening guard that proves execution-side files do not depend on scheduler mailbox ABI tokens, scheduler mailbox files do not depend on execution delivery/result ABI tokens, and the core mutation sites still carry the common execution-slot critical section discipline
- Closed `6.6` with direct FIFO pickup and no-mailbox-reuse validation over two queued executions targeting the same worker
- Reconciled the Phase 10-B task checklist with the now-landed validation/guard coverage

Touched Code Paths:
- `kernel/tests/validation/phase2_validation_test.c`
- `scripts/ci/check_phase10b_execution_hardening.sh`
- `scripts/ci/gate_syscall_semantics_phase10b.sh`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `cargo test -p bcib-runtime`
- `bash scripts/ci/check_phase10b_execution_hardening.sh --evidence-dir /tmp/phase10b-hardening-guard`
- `bash scripts/ci/gate_syscall_semantics_phase10b.sh --evidence-dir /tmp/phase10b-review-gate --phase10a2-evidence evidence/run-local-p10-b-fix-serial/gates/ring3-execution-phase10a2 --mode negative`
- `git diff --check`

Impact:
- converted several remaining Phase 10-B closure doubts from manual code reading into explicit automated proof surfaces
- narrowed the remaining open closure work to illegal cross-state mutation assertions rather than pickup/time/boundary ambiguity

Notes:
- this slice tightens proof quality and source guarding; it does not widen the runtime ABI

### 2026-03-21

Completed Slice:
- Converted the core runtime paths from post-fact transition checking to fail-closed transition enforcement by routing authoritative lifecycle mutations through explicit `require_*` wrappers
- Kept intentional negative validation intact by leaving the raw transition helpers available for tests while forbidding direct runtime use through the Phase 10-B source guard
- Strengthened invariant checking with result/hash mapping coherence for `COMPLETED` and `RESULT_MAPPED` slots

Touched Code Paths:
- `kernel/include/execution_slot.h`
- `kernel/sys/execution_slot.c`
- `kernel/sys/syscall_v2.c`
- `kernel/sched/sched.c`
- `scripts/ci/check_phase10b_execution_hardening.sh`

Touched Docs:
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `bash scripts/ci/check_phase10b_execution_hardening.sh --evidence-dir /tmp/phase10b-hardening-guard`
- `git diff --check`

Impact:
- shifted the execution lifecycle from "trace detects illegal state drift" toward "runtime halts on impossible transition attempts" for authoritative paths
- locked the fail-closed wrapper pattern into CI so future changes cannot silently fall back to permissive raw helpers

Notes:
- raw transition helpers remain intentionally callable from validation code that proves negative paths without halting the kernel

### 2026-03-21

Completed Slice:
- Added the first closure-proof layer above the existing runtime validation by recording per-slot transition traces with actor and timestamp metadata
- Added a global execution-slot invariant checker for duplicate live IDs, one-`RUNNING`-slot-per-worker, coherent trace ordering, and immutable post-terminal states
- Added direct validation that the core lifecycle emits the expected `submit -> pickup -> complete -> wait_result` trace sequence without widening the syscall ABI
- Clarified the Phase 10-B task sheet status to distinguish checklist completion from still-pending formal closure work

Touched Code Paths:
- `kernel/include/execution_slot.h`
- `kernel/sys/execution_slot.c`
- `kernel/sys/syscall_v2.c`
- `kernel/sched/sched.c`
- `kernel/arch/x86_64/timer.c`
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `git diff --check`

Impact:
- moved Phase 10-B proof quality from isolated semantic checks toward trace-backed lifecycle evidence
- created a reusable kernel-internal surface for future adversarial and replay-oriented closure work

Notes:
- this slice keeps the proof layer kernel-internal; userspace ABI and syscall numbers are unchanged

### 2026-03-21

Completed Slice:
- Closed the remaining `Task 1.4` proof gap by adding direct negative validation for illegal execution-slot cross-state mutation attempts
- Closed parent `Task 1` after proving the slot transition helpers reject stale `expected_from` mismatches, backward rewrites, illegal terminal shortcuts, and post-terminal overwrite attempts without mutating state
- Marked the canonical Phase 10-B hardening checklist complete now that the last open serialization item has runtime proof coverage

Touched Code Paths:
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `git diff --check`

Impact:
- moved the final remaining execution-slot serialization concern from manual reasoning into an explicit fail-closed test surface
- removed the last open item from the canonical Phase 10-B hardening task list

Notes:
- this slice does not widen the runtime ABI; it only strengthens negative lifecycle proof coverage

### 2026-03-21

Completed Slice:
- Closed `16.4` by landing additive structured-output v2 header support on top of the raw output plane without widening the syscall surface
- Closed `16.5` by adding semantic validation for backward-compatible raw fallback, known structured `RAW`/`BLOB` kinds, unknown kind fail-closed behavior, and structured version mismatch
- Updated runtime truth so the current output plane is explicitly dual-header (`raw v1` + `structured v2`) while payload meaning remains userland-owned

Touched Code Paths:
- `shared/abi/execution_output_structured_abi.h`
- `kernel/include/execution_output_structured_abi.h`
- `kernel/include/execution_slot.h`
- `kernel/sys/execution_slot.c`
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/structured-output-minimal-spec.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o`
- `git diff --check`

Impact:
- established the first semantic result layer without turning the kernel into a parser
- kept raw output fully backward-compatible while adding bounded typed header enforcement

Notes:
- runtime still treats all payload bytes as opaque regardless of the declared structured kind

### 2026-03-21

Completed Slice:
- Landed the minimal result-hash integrity layer (`Task 17.4`/`17.5`) on top of the already-frozen raw+structured result plane
- `complete_execution(COMPLETED)` now freezes a SHA-256 digest over the exact logical published result bytes and stores it in a slot-owned kernel-backed sidecar page
- `wait_result()` now maps that sidecar read-only/NX at a deterministic hash VA alongside the existing result mapping and replays both VAs deterministically on repeated successful waits
- Exit cleanup now revokes the hash sidecar with the result mapping
- Validation now proves raw-v1 and structured-v2 digest publication, exact-byte coverage, replay stability, and revoke-on-cleanup behavior

Touched Code Paths:
- `shared/abi/execution_result_hash_abi.h`
- `kernel/include/execution_result_hash_abi.h`
- `kernel/include/sha256.h`
- `kernel/lib/sha256.c`
- `kernel/include/execution_slot.h`
- `kernel/include/proc.h`
- `kernel/sys/execution_slot.c`
- `kernel/sys/syscall_v2.c`
- `kernel/proc/proc.c`
- `kernel/tests/validation/phase2_validation_test.c`

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/result-hash-minimal-spec.md`
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/development/SYSCALL_TRANSITION_GUIDE.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- `make kernel`
- `clang --target=x86_64-elf ... -c kernel/tests/validation/phase2_validation_test.c -o /tmp/phase2_validation_test.o`
- `git diff --check`

Impact:
- execution results now carry a deterministic owner-visible integrity anchor without changing the existing syscall return contract
- the kernel hashes exact frozen bytes while preserving the "hash bytes, not meaning" boundary

Notes:
- QEMU/runtime validation was not rerun in this slice
- the current integrity layer is fixed to SHA-256 and remains additive

### 2026-03-21

Completed Slice:
- Opened the docs-first `Result Hash Minimal Spec` as the next integrity layer above the landed raw+structured result plane
- Froze the hash subject as the exact logical published result bytes (`result_size`) so future integrity work cannot drift into hashing padding, worker scratch bytes, or payload semantics
- Added a dedicated task track for result hashing so any future integrity anchor remains additive and does not widen the current execution syscall surface by accident

Touched Code Paths:
- none (docs-only slice)

Touched Docs:
- `docs/specs/phase10b-execution-path-hardening/result-hash-minimal-spec.md`
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`
- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/progress.md`

Validation:
- not rerun (docs-only slice; runtime behavior unchanged)
- `git diff --check`

Impact:
- moved the next integrity widening from loose discussion into a bounded, repo-tracked contract
- locked the "hash bytes, not meaning" rule before any result-hash runtime code lands

Notes:
- no runtime or ABI behavior changed in this slice

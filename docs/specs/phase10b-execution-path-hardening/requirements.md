# Requirements

**Status:** Draft (local execution-reality stabilization plan; does not override roadmap authority)
**Phase:** Phase 10-B / Phase 10-C runtime stabilization
**Last Updated:** 2026-03-22

## 1. Purpose

This document defines the minimum kernel/runtime requirements needed to make the
execution-centric syscall path behave like a real execution lifecycle instead of
an interface with placeholder semantics.

This document is subordinate to:

- `docs/constitution/PHASE_0_FOUNDATIONAL_OATH.md`
- `docs/operations/RUNTIME_INTEGRATION_GUARDRAILS.md`
- `ARCHITECTURE_FREEZE.md`

This document does **not** change the official roadmap truth surface that says
Phase-13 boundary hardening is the active governance workstream. Its purpose is
to prevent runtime architecture drift while local kernel execution semantics are
still incomplete.

## 2. Scope

In scope:

- `kernel/sys/syscall_v2.c`
- `kernel/mm/*`
- `kernel/proc/*`
- `kernel/sched/*`
- `kernel/arch/x86_64/timer.c`
- `userspace/bcib-runtime/*`

Out of scope:

- Phase-13 `proofd` authority/observability semantics
- scheduler policy changes in Ring3
- new POSIX-like syscall surfaces
- ABI widening beyond the frozen v2 syscall range

## 3. Repo Anchors

The following repo truths constrain this plan:

- Ring0 is mechanism-only; Ring3 is policy.
- The canonical runtime path is `semantic-cli -> bcib-runtime -> ayken-core/bcib -> kernel syscalls -> kernel`.
- Scheduler mailbox is a policy bridge and MUST remain separate from execution dispatch.
- Timer ticks are the only current monotonic in-kernel time source.
- `proc_block_current(wait_obj)` / `proc_wake_waiters(wait_obj)` already define the canonical block/wake mechanism.
- Kernel heap and kmalloc-backed proc metadata now live in the higher half, and
  user CR3 roots MUST consume them only through the copied kernel-half entries.
- Any reintroduction of a dedicated low-half kernel-heap mirror into user CR3
  roots MUST fail closed under `ci-gate-no-low-half-kernel-dependency`.

## 4. Requirements

### Requirement 1: Monotonic Time Authority

`sys_v2_time_query()` MUST be treated as a first-class part of the execution
lifecycle, not a helper syscall.

The implementation MUST satisfy all of the following:

- derive time from a single monotonic kernel authority
- remain valid across preemption and context switch
- provide the timeout basis for `wait_result`
- provide a stable deadline basis for execution-slot timeout handling
- freeze `query_type` semantics so userspace and validation agree on units

The initial authority source MUST be the PIT-backed monotonic tick counter in
`kernel/arch/x86_64/timer.c`.

Initial frozen `sys_v2_time_query()` contract:

- `TIME_QUERY_MONOTONIC = 0` returns raw monotonic PIT ticks
- `TIME_QUERY_UPTIME = 1` returns uptime in milliseconds derived from PIT ticks
- unknown `query_type` values MUST fail closed with `ESYS_V2_INVALID_PARAM`

### Requirement 2: Execution Slot Model

Kernel execution state MUST be represented by an explicit kernel-owned
`execution_slot` model.

Each slot MUST contain at least:

- `execution_id`
- `owner_pid`
- `target_context_id`
- kernel-owned BCIB buffer reference
- status
- creation tick
- deadline tick
- result backing reference
- error code
- waiter count or equivalent waiter bookkeeping

The minimum required state set is:

- `CREATED`
- `READY`
- `RUNNING`
- `COMPLETED`
- `FAILED`
- `TIMEOUT`
- `RESULT_MAPPED`
- `ABORTED`

### Requirement 3: Execution Slot Serialization

Execution-slot state transitions MUST be serialized.

At minimum, the following mutation sites MUST use the same serialization
discipline:

- `sys_v2_submit_execution()`
- worker pickup / completion path
- `sys_v2_wait_result()`
- timer IRQ timeout progression
- `sys_v2_exit()`

The current repo does not expose a general spinlock primitive in the kernel
tree. Therefore the initial implementation MUST choose one concrete mechanism
and use it consistently:

- first landing: a bounded global execution-table critical section guarded by
  interrupt-disabled entry/exit on the current single-core runtime

If SMP or parallel kernel mutation is introduced later, this contract MUST be
upgraded to a real lock primitive without changing slot semantics.

### Requirement 4: Result Ownership Contract

Execution results MUST be kernel-owned until explicitly mapped into a caller
address space.

The minimum result contract is:

- result backing is produced in kernel-owned memory
- the first successful `wait_result` MUST materialize the full bounded
  kernel-owned validated output header plus payload bytes currently attached to
  the completed slot
- the first materialization contract now uses the distinct executor-written
  output plane validated during `complete_execution`; the mapped bytes are the
  frozen slot-owned output backing rather than the original BCIB input
- first successful `wait_result` maps result into caller address space
- slot transitions from `COMPLETED` to `RESULT_MAPPED`
- repeated `wait_result` behavior MUST be explicit and deterministic:
  either return the same mapped VA or fail with an explicit consumed-state error
- result mapping MUST be read-only and non-executable in user space
- `exit` MUST revoke all result mappings owned by the exiting process

### Requirement 5: Timeout Authority

Timeout resolution MUST have one explicit authority.

The chosen authority for the initial implementation MUST be:

- timer IRQ path performs deadline progression and timeout state transition

This means:

- slot timeout decisions MUST be driven from monotonic tick state
- `wait_result` MUST NOT implement timeout by busy-spin loops
- scheduler-side ad hoc polling MUST NOT become the timeout authority

The initial implementation MAY scan the bounded execution-slot table in the
timer IRQ path. Unbounded or dynamically growing IRQ-path scans are forbidden.
If slot pressure grows beyond the bounded table model, a dedicated timeout index
structure becomes mandatory.

### Requirement 6: Dispatch Boundary Separation

Execution dispatch MUST remain separate from scheduler arbitration.

This means:

- scheduler mailbox remains dedicated to scheduling authority hints
- execution submission MUST use a distinct kernel-owned queue surface keyed by
  `context_id`
- if a userspace-visible execution inbox is added, it MUST be a kernel-written,
  read-only projection of that queue rather than a peer authority surface
- scheduler mailbox ABI MUST NOT be reused to carry BCIB payloads or execution result data

This requirement exists to preserve the Ring0/Ring3 authority split and avoid
reintroducing a mixed control-plane surface inside the kernel.

The initial projection contract is further constrained by:

- `docs/specs/phase10b-execution-path-hardening/execution-inbox-minimal-spec.md`

### Requirement 7: Worker Pickup Model

The initial worker execution model MUST be explicit.

Initial version:

- execution descriptors are queued in a kernel-owned per-`context_id` queue
- the target worker checks for queued execution work on schedule entry
- userspace polling loops are not the authoritative dispatch mechanism
- each worker has exactly one kernel-owned active execution latch guarding
  delivery publication
- a userspace-visible execution inbox, if present, MUST NOT be overwritten while
  that latch remains set

This means the kernel queue is authoritative and any userspace-visible inbox is
only a delivery projection, not the source of truth.

### Requirement 8: Blocking Wait Semantics

`sys_v2_wait_result()` MUST use the existing process block/wake path.

The implementation MUST:

- block on a stable wait object derived from the execution slot
- wake through `proc_wake_waiters(wait_obj)` or equivalent canonical path
- return immediately if slot is already terminal
- fail closed on invalid or foreign execution IDs
- preserve a single terminal outcome when timeout and explicit completion race;
  the first successful terminal transition MUST win and any later attempt MUST
  fail-closed without overwriting terminal state

`wait_obj` identity MUST remain stable for the slot lifetime. Raw recyclable
slot pointers are forbidden. The initial implementation MUST use an embedded
wait-key identity based on stable slot metadata such as:

- `execution_id`
- generation counter

or an equivalent dedicated wait structure whose address remains stable until the
slot is fully retired.

### Requirement 9: Explicit Mapping Primitive

`sys_v2_map_memory()` and `sys_v2_unmap_memory()` MUST remain explicit mapping
primitives under the frozen ABI:

- `map_memory(virt_addr, phys_addr, flags)`
- `unmap_memory(virt_addr, size)`

They MUST NOT silently become allocators.

Mapping state MUST be tracked in a process-local mapping ledger. Each ledger
entry MUST contain at least:

- `owner_pid`
- `user_va`
- `phys_addr` or backing frame reference
- flags
- capability/token binding
- local `map_id` or syscall sequence identifier

The ledger exists for both cleanup and capability-backed enforcement.

At minimum, kernel-mediated mapping lifecycle operations MUST validate owner and
capability binding:

- create
- unmap
- revoke on exit
- result remap into user space

This requirement does not add a new dynamic page-fault mediation layer in the
initial version; process isolation remains CR3/page-table based.

### Requirement 10: Exit Lifecycle

`sys_v2_exit()` MUST terminate the process lifecycle instead of looping on
`sched_yield()`.

At minimum, exit MUST:

- transition process state to `PROC_ZOMBIE`
- abort or fail any owned non-terminal execution slots
- revoke owned result mappings
- revoke owned map ledger entries
- wake waiters blocked on aborted/failed slots
- remove the process from all scheduler queues
- trigger an immediate context switch away from the exiting process
- ensure the exiting task is not selected again as runnable

Until an explicit scheduler-owner handoff protocol exists, the scheduler owner
process MUST fail closed on `sys_v2_exit()` at syscall entry.

That owner-exit deny path MUST:

- return `ESYS_V2_PERMISSION_DENIED`
- emit an explicit log/marker for diagnostics
- avoid starting any slot abort, revoke, queue removal, or switch-away side
  effects

Future scheduler-owner handoff/reap work MUST remain a separate
governance-controlled slice rather than a silent extension of the current exit
path. The review context remains:

- `docs/governance/SCHEDULER_OWNER_HANDOFF_REAP_CANDIDATE.md`

The governing ratified surface for that work is the narrow mailbox-v1 transfer
exception:

- `docs/governance/MAILBOX_V1_OWNER_TRANSFER_EXCEPTION.md`

Any future owner-transfer authority commit MUST occur only at the scheduler
dispatch boundary while scheduling is paused for the current CPU.
Runtime proof is now landed for dispatch-boundary owner commit,
successor-authority mailbox application, and old-owner no-return exit/reap
follow-through under successor authority via validation-safe scheduler seams.

### Requirement 11: ABI Consistency

The v2 syscall ABI contract MUST remain internally consistent.

In particular:

- `sys_v2_submit_execution(..., context_id)` third argument is `context_id`
- userspace wrappers MUST NOT reinterpret that third argument as `execution_id`
- kernel-owned `execution_id` allocation remains the syscall return value
- `execution_id` MUST remain strictly monotonic and non-reused within a boot
  session; allocation wrap-around MUST fail-closed rather than reusing an old ID
- if authoritative completion requires a dedicated ABI entry point, that entry
  point MUST be ratified explicitly before implementation rather than silently
  repurposing an existing syscall
- once that completion entry point is ratified, only the target executor
  process that currently owns `active_execution_id` for the matching
  `execution_id` MAY invoke it; foreign or stale completion attempts MUST
  fail-closed
- slot `generation` remains internal lifetime metadata for wait-key stability
  and does not become part of the first completion ABI
- the completion entry point MUST expose deterministic return codes for success,
  invalid state, permission denial, and invalid/stale execution identity

### Requirement 12: Semantic Validation

Placeholder-success tests are insufficient for this scope.

The implementation MUST add semantic validation for:

- real mapping presence/absence in process page tables
- execution slot creation and terminal transitions
- blocking wakeup on completion
- timeout wakeup
- zombie cleanup behavior
- userspace wrapper ABI consistency

### Requirement 13: Documentation Sync

Implementation work under this plan MUST be documented as it lands.

At minimum, each completed runtime slice MUST update the relevant documents in
the same change set:

- this spec set under `docs/specs/phase10b-execution-path-hardening/`
- any touched authority-adjacent runtime docs when behavior changes materially
- test or gate docs when validation semantics change

The goal is to prevent code/docs drift during staged kernel runtime bring-up.

The active execution-path progress log for this plan is:

- `docs/specs/phase10b-execution-path-hardening/progress.md`

### Requirement 14: Distinct Output Plane Upgrade

The first distinct output-plane upgrade MUST use a bounded fixed-VA output
window rather than a new per-write syscall.

That first upgrade MUST:

- preserve the existing `submit_execution`, `complete_execution`, and
  `wait_result` syscall surface
- give the executor a fixed writable/NX output window
- keep the output backing kernel-owned and slot-owned
- validate a versioned output header during `complete_execution()`
- freeze the validated output backing before `wait_result()` publication
- zero-seal any bytes past the declared logical result size inside the mapped
  result frame span before publication
- keep owner-visible result publication read-only/NX
- keep repeated successful waits deterministic
- if `COMPLETED` is requested with invalid output metadata, fail closed by
  terminalizing as `FAILED`, clearing the latch, waking waiters, and returning
  `ESYS_V2_INVALID_STATE`

The frozen candidate for that first landing is:

- `docs/specs/phase10b-execution-path-hardening/execution-output-minimal-spec.md`

### Requirement 15: Structured Output Semantic Layer

If the output plane widens from "safe bounded bytes" to "typed semantic
results", the first structured-output landing MUST remain additive on top of
the landed raw output contract.

That first semantic layer MUST:

- preserve the existing raw output/result publication path as a backward-
  compatible fallback
- introduce a distinct typed structured-output header rather than overloading
  the existing raw header
- keep `kind` as a closed, bounded set
- validate only header structure, known version, known `kind`, and bounds
- treat payload bytes as opaque regardless of the declared `kind`
- explicitly delegate all payload parsing and semantic interpretation to
  userland
- fail closed on unknown `kind`, version mismatch, malformed header, or invalid
  bounds

That first semantic layer MUST NOT:

- introduce kernel-side payload parsing
- introduce schema negotiation or dynamic typing
- mutate the landed raw output-plane contract in place

The active minimal contract for that semantic layer is:

- `docs/specs/phase10b-execution-path-hardening/structured-output-minimal-spec.md`

The first landed structured layer now uses:

- raw v1 output header as the backward-compatible fallback
- additive structured v2 output header with a closed initial `kind` set
  (`RAW`, `BLOB`)

### Requirement 16: Result Hash Integrity Anchor

If the result model widens from "frozen published bytes" to "frozen published
bytes plus integrity anchor", the first result-hash landing MUST remain
additive on top of the landed raw+structured result plane.

That first integrity layer MUST:

- preserve the existing `submit_execution`, `complete_execution`, and
  `wait_result` syscall surface
- compute one fixed digest over exactly the logical published result bytes
  (`result_size`)
- compute that digest during successful completion-time freeze instead of
  lazily during `wait_result()`
- exclude zero-sealed padding beyond `result_size` from the digest subject
- keep the digest kernel-owned and slot-owned until explicitly mapped
- preserve raw v1 and structured v2 result compatibility
- keep the hash surface read-only/NX in user space
- keep repeated successful waits deterministic for both result publication and
  digest publication

That first integrity layer MUST NOT:

- introduce algorithm negotiation
- parse payload semantics while hashing
- reinterpret output `kind`
- mutate the landed result bytes in place

The active minimal contract for that integrity layer is:

- `docs/specs/phase10b-execution-path-hardening/result-hash-minimal-spec.md`

## 5. Non-Goals

This plan does not authorize:

- policy migration back into Ring0
- replacing scheduler mailbox with execution dispatch
- introducing distributed verification semantics into kernel runtime work
- changing the frozen syscall number range without an explicit completion-handoff
  ratification

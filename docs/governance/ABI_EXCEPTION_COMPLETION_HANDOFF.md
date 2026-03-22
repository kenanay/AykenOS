# ABI Exception: Completion Handoff

Status: RATIFIED (narrow ABI exception accepted 2026-03-19)  
Scope: execution-path lifecycle closure only

## 1. Problem

The current execution lifecycle is structurally incomplete.

What exists today:

- `submit_execution()` can create kernel-owned `READY` work
- schedule-entry pickup can transition `READY -> RUNNING`
- `wait_result()` can block and wake on timeout
- timer IRQ can terminalize overdue work as `TIMEOUT`

What is still missing:

- an authoritative `RUNNING -> COMPLETED`
- an authoritative `RUNNING -> FAILED`

This means the runtime can start work and time it out, but cannot yet close a
successful or explicit failed execution lifecycle.

## 2. Why Existing Syscalls Cannot Be Reused

### 2.1 `sys_v2_interrupt_return`

Rejected.

Reason:

- it is already tied to IRQ-exit semantics in the runtime state machine
- repurposing it would overload an unrelated surface and create semantic drift

### 2.2 `sys_v2_submit_execution`

Rejected.

Reason:

- submission and completion are opposite lifecycle edges
- combining them would create ambiguous authority and validation semantics

### 2.3 Scheduler-Side Implicit Completion

Rejected.

Reason:

- scheduler behavior must not become hidden completion authority
- completion must remain explicit, auditable, and state-checked

## 3. Why A New Syscall Is Required

The kernel needs one explicit completion authority surface so it can:

- validate that the slot is still `RUNNING`
- validate caller authority against the active execution latch
- serialize terminal state mutation under the execution-slot lock discipline
- clear `active_execution_id`
- wake blocked waiters deterministically

Without an explicit kernel entry point, completion semantics either become
implicit or leak through unrelated channels.

## 4. Proposed Surface Shape

The proposed first-landing shape is intentionally minimal:

```text
sys_v2_complete_execution(execution_id, completion_code)
```

Initial scope:

- `execution_id`
- terminal completion code (`COMPLETED` or `FAILED`)

Initial non-goals:

- result pointer transfer
- result ownership transfer
- repeated wait semantics
- multi-waiter refinements

Those remain later slices.

## 5. Why The ABI Exception Is Safe

This exception is narrow:

- exactly one new completion-oriented syscall surface
- no renumbering or semantic mutation of existing v2 syscalls
- no reuse of unrelated entry points
- no mailbox ABI mutation
- no scheduler authority drift

The intent is to add the missing lifecycle half, not to reopen the ABI broadly.

## 6. Why Not Adding It Is Worse

Not adding a dedicated completion surface pushes the runtime toward bad
alternatives:

- hidden completion channels
- IRQ semantic overloading
- scheduler-coupled completion inference
- incomplete `wait_result()` semantics
- permanent timeout-only terminalization

Those outcomes are worse for determinism, boundary clarity, and Ring0/Ring3
separation than one explicit narrow ABI exception.

## 7. Ratification Record

This document records the ratified exception and its chosen numbering path.

Ratification confirmed all of the following:

1. one dedicated completion syscall is allowed as an exception to the current
   frozen v2 range
2. `interrupt_return` remains IRQ-only
3. completion remains distinct from scheduler mailbox and execution inbox
4. result ownership is not bundled into the first landing

## 8. Failure If Rejected

If this exception is rejected:

- execution lifecycle remains permanently incomplete
- system correctness depends on timeout-only terminalization
- successful execution cannot be represented
- `wait_result()` semantics remain structurally partial

This is not a degraded mode.

This is a broken lifecycle.

## 9. Syscall Number Strategy

Three numbering strategies exist:

### 9.1 Option A: v2 Single-Surface Extension

Selected.

```text
1011 -> sys_v2_complete_execution
```

Why this is preferred:

- exactly one new surface
- existing v2 numbering stays intact
- no semantic mutation of existing syscall meanings
- the exception remains obvious and auditable

### 9.2 Option B: Separate v2 Extension Range

Possible, but not preferred.

Example:

```text
1100+ -> v2 extension zone
```

Why this is weaker:

- larger policy surface than needed
- invites future unrelated widening under the same escape hatch

### 9.3 Option C: Start v3

Rejected for now.

Why this is excessive:

- the problem is one missing lifecycle surface, not a whole-interface redesign
- v3 would create more migration overhead than the runtime defect justifies

Ratification selected Option A.

## 10. Completion Authority Model

Only the worker process that currently owns the execution latch
(`active_execution_id`) may call `sys_v2_complete_execution()` for that
execution.

The kernel must validate all of the following before terminalizing the slot:

- the slot is still in `RUNNING`
- the caller context matches the worker process that owns the active execution
  latch
- the supplied `execution_id` matches the caller's current active execution
  latch

Any mismatch MUST fail-closed.

This authority model exists to:

- prevent third-party completion injection
- preserve single-writer lifecycle closure
- align completion authority with execution ownership

This contract relies on the current execution identity invariant:

- `execution_id` is kernel-assigned, 64-bit, and strictly monotonic within a
  boot session
- `execution_id` is not recycled on slot reuse
- wrap-around is forbidden; if monotonic allocation can no longer provide a
  fresh non-zero ID, allocation MUST fail-closed

`generation` remains kernel-internal slot lifetime metadata for wait-key
stability. It is not part of the first completion ABI surface.

## 11. Completion vs Timeout Arbitration

If a slot is concurrently targeted by:

- timer-driven `TIMEOUT`
- worker-driven `COMPLETED`
- worker-driven `FAILED`

the kernel MUST serialize terminalization under the execution-slot lock and
enforce the following rule:

- the first successful terminal transition wins
- any subsequent terminalization attempt MUST fail-closed without overwriting
  the existing terminal state

Allowed outcomes:

- `RUNNING -> COMPLETED`
- `RUNNING -> FAILED`
- `RUNNING -> TIMEOUT`

Forbidden outcomes:

- dual terminal states for one `execution_id`
- overwriting an already terminal slot

## 12. Completion Return Contract

The first landing of `sys_v2_complete_execution()` MUST expose deterministic
return semantics:

- return `0` on successful terminalization
- return `ESYS_V2_INVALID_STATE` if the slot is valid but no longer `RUNNING`
- return `ESYS_V2_PERMISSION_DENIED` if the caller does not own the matching
  `active_execution_id` latch
- return `ESYS_V2_INVALID_ID` if the supplied `execution_id` is invalid or
  stale

This return contract is part of the lifecycle guarantee. Terminal state
determinism without return-surface determinism is insufficient.

## 13. Post-Ratification Landing Order

Once ratified, the next implementation order should be:

1. add `sys_v2_complete_execution`
2. implement `RUNNING -> COMPLETED/FAILED`
3. clear `active_execution_id`
4. wake waiters
5. stop

Result delivery and ownership remain separate follow-on work.

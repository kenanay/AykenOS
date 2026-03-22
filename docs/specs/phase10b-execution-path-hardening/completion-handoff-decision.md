# Completion Handoff Decision

**Status:** Resolved (ratified and implemented on 2026-03-19)  
**Scope:** Phase 10-B / 10-C execution-path hardening  
**Authority Level:** Local runtime decision surface, not roadmap authority

## 1. Purpose

This document froze the preferred shape of execution completion handoff before
implementation and now records the resolved local decision.

Its job is to answer one question:

How does a `RUNNING` execution become authoritatively `COMPLETED` or `FAILED`
without violating the current Ring0/Ring3 boundary?

## 2. Current Problem

The kernel now supports:

- kernel-owned execution slots
- `READY -> RUNNING` pickup on schedule entry
- finite timeout waits
- timer-driven timeout terminalization
- worker-latch clearing on timeout

What it still lacks is an authoritative success/failure completion path.

Current gap:

```text
submit_execution -> READY -> RUNNING -> TIMEOUT
```

Missing path:

```text
RUNNING -> COMPLETED / FAILED
```

This means the current runtime can start work and time it out, but cannot yet
close the lifecycle on successful or explicit failed execution.

## 3. Repo Constraints

The completion design is constrained by existing repo truths:

- Ring0 is mechanism-only; Ring3 is policy/execution logic.
- `wait_result()` already expects terminal slot states but no completion entry
  point exists yet.
- `sys_v2_interrupt_return()` is currently reserved for IRQ-exit semantics and
  must not be repurposed as execution completion.
- the v2 syscall range was `1000-1010` and frozen before the ratified
  completion exception extended it to `1011`.

These constraints rule out several superficially convenient shortcuts.

## 4. Decision

The preferred completion model is:

- userspace execution logic decides when a `RUNNING` execution has finished
- kernel remains the only authority allowed to mutate slot state
- completion crosses the boundary through an explicit kernel entry point

In short:

- execution authority lives in userspace
- state authority lives in kernel

## 5. Preferred Interface

The preferred interface is a dedicated completion syscall surface:

```text
sys_v2_complete_execution(execution_id, completion_code, ...)
```

The exact syscall number is intentionally not assigned in this document.

Reason:

- the current v2 range is already full
- assigning a new number would be an ABI/governance action, not just a local
  implementation detail

The minimum first-landing contract is intentionally narrow:

- identify the `execution_id`
- identify terminal outcome (`COMPLETED` or `FAILED`)
- perform no result-ownership transfer yet

Result ownership remains a later slice.

## 6. Why A Dedicated Entry Point

An explicit completion entry point is preferred because it preserves the current
architecture rules:

- no scheduler-side implicit state mutation
- no hidden control path through the execution inbox
- no IRQ path semantic overload
- no fake completion inferred from userspace return behavior

This makes completion explicit, auditable, and serializable with the existing
execution-slot critical section.

## 7. Required Kernel Checks

Any completion handoff implementation must validate all of the following under
the same execution-slot serialization discipline already used for submit/wait/
timeout:

- the slot exists
- the slot is currently `RUNNING`
- the caller is the current active executor for that slot
- `active_execution_id` matches the submitted `execution_id`
- the slot has not already been terminalized

On successful validation, the completion path must atomically:

1. transition `RUNNING -> COMPLETED` or `RUNNING -> FAILED`
2. clear the worker `active_execution_id`
3. wake waiters on `&slot->wait_key`

Double-completion and foreign-completion attempts must fail closed.

## 8. Explicitly Rejected Alternatives

### 8.1 Reusing `interrupt_return`

Rejected.

Reason:

- `interrupt_return` belongs to IRQ-exit semantics in the runtime state machine
- reusing it for execution completion would create semantic drift

### 8.2 Reusing `submit_execution`

Rejected.

Reason:

- submission and completion are opposite lifecycle edges
- overloading one syscall with both meanings would make validation and ABI usage
  ambiguous

### 8.3 Implicit Scheduler Completion

Rejected.

Reason:

- completion must not be inferred from scheduler behavior or userspace return
- the scheduler should not become the hidden authority for execution success

### 8.4 Reverse Shared-Memory Completion Channel

Rejected for the first landing.

Reason:

- it would introduce a second authority surface before the basic lifecycle is
  closed
- it is more complex than the current runtime needs

## 9. ABI and Governance Implication

This decision does **not** itself widen the ABI.

It does, however, conclude that a dedicated completion syscall is the correct
technical answer. Therefore one explicit ABI/governance decision is required
before implementation:

- either ratify a one-surface exception to the frozen v2 range
- or ratify a different explicit ABI entry point with equivalent semantics

Until such ratification exists, completion handoff should not be implemented by
repurposing an unrelated syscall.

The ratified governance record is:

- `docs/governance/ABI_EXCEPTION_COMPLETION_HANDOFF.md`

## 10. Implementation Order Consequence

Once ratified, the correct runtime order is:

1. land the explicit completion entry point
2. transition `RUNNING -> COMPLETED/FAILED`
3. clear `active_execution_id` on completion
4. wake waiters on completion/failure
5. land result ownership and repeated `wait_result()` semantics

This keeps lifecycle closure ahead of result-delivery complexity.

## 11. Resolution Outcome

The ratified implementation landed as:

- `SYS_V2_COMPLETE_EXECUTION` at public number `1011`
- explicit `RUNNING -> COMPLETED/FAILED` terminalization
- latch-bound caller authority validation
- first-terminal-state-wins arbitration against timeout

Result ownership remains follow-on work.

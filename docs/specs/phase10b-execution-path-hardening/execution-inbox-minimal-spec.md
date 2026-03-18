# Execution Inbox Minimal Spec

**Status:** Draft (implementation-targeted local runtime spec)
**Phase:** Phase 10-B / Phase 10-C execution path hardening
**Last Updated:** 2026-03-19

## 1. Purpose

This document defines the minimum userspace-visible execution inbox needed to
turn schedule-entry worker pickup into an actual delivery surface without
reusing the scheduler mailbox ABI.

The goal is narrow:

- give the target worker a deterministic, kernel-written view of the current
  execution descriptor
- preserve the kernel queue as the sole source of truth
- make a later completion handoff possible without widening the syscall ABI

This document does **not** authorize a new control plane.

## 1.1 Publish Preconditions

The first execution inbox implementation MUST NOT land before all of the
following are true:

- `submit_execution` has copied BCIB bytes into kernel-owned backing
- slot metadata can describe enough backing frames to populate the bounded
  payload window
- `context_id` is validated against a live target user worker

Current codebase note:

- the current single `bcib_phys` field in `exec_slot_t` is not sufficient for
  the 16 KiB payload window defined below
- before inbox publish lands, slot backing metadata MUST widen to a bounded
  frame list or an equivalent kernel-owned payload representation
- oversize BCIB payloads MUST fail closed rather than partially publishing

## 2. Non-Negotiable Rules

- the kernel execution queue remains authoritative
- the execution inbox is a projection only
- userspace MUST NOT publish work into the execution inbox
- Ring0 MUST NOT read the inbox as an authority surface
- scheduler mailbox and execution inbox MUST use different physical pages and
  different fixed virtual addresses
- the execution inbox mapping MUST be user-readable, non-executable, and
  non-writable from userspace
- each worker MUST have exactly one `active_execution_id` latch in kernel space
- inbox publication MUST NOT proceed while that worker latch is already set

## 3. Fixed VA Layout

Initial fixed virtual addresses:

- `SCHED_MAILBOX_VA = 0x700000` stays reserved for scheduler policy hints
- `EXECUTION_INBOX_VA = 0x701000` is the one-page execution descriptor surface
- `EXECUTION_PAYLOAD_VA = 0x702000` is the start of the worker BCIB payload
  window

Initial payload window:

- `EXECUTION_PAYLOAD_WINDOW_SIZE = 0x4000` (16 KiB, 4 pages)

Initial size rule:

- if `graph_size > EXECUTION_PAYLOAD_WINDOW_SIZE`, `submit_execution` MUST fail
  closed until a larger kernel-backed payload projection is implemented

This keeps the first delivery slice bounded and deterministic.

Payload mapping lifecycle:

- the execution inbox page and payload window MUST be mapped during process
  initialization
- those mappings MUST remain stable for the worker lifetime
- process exit MUST revoke both mappings
- if the shared paging flag surface does not yet expose NX control for this
  mapping path, that flag exposure MUST be added before landing execution inbox
  publish; executable user mapping is not an acceptable fallback

## 4. Authority Model

The execution authority chain is:

```text
kernel execution queue (truth)
  -> schedule-entry pickup
  -> execution inbox projection
  -> userspace worker reads snapshot
  -> future completion handoff
  -> execution_slot terminal transition
```

The inbox is never the truth source.

It is only a delivery snapshot for the currently active execution owned by the
scheduled worker.

Initial latch rule:

- the kernel worker latch (`active_execution_id`) is the publication guard
- if the latch is non-zero, Ring0 MUST NOT publish a new inbox snapshot for that
  worker
- one-deep inbox behavior is therefore enforced by kernel latch state, not by
  userspace discipline

## 5. Inbox Descriptor

Initial one-page descriptor:

```c
typedef struct ayken_execution_inbox_v1 {
    uint32_t magic;              // 'AXIB'
    uint16_t version;            // 1
    uint16_t state;              // EMPTY or READY
    uint64_t delivery_seq;       // incremented by kernel on each new delivery
    uint64_t execution_id;       // authoritative kernel execution_id
    uint64_t target_context_id;  // worker pid/context the slot targets
    uint64_t bcib_user_va;       // fixed == EXECUTION_PAYLOAD_VA
    uint64_t bcib_size;          // number of readable bytes in payload window
    uint64_t bcib_window_size;   // fixed == EXECUTION_PAYLOAD_WINDOW_SIZE
    uint64_t flags;              // reserved, must be zero in v1
    uint64_t reserved[6];
} ayken_execution_inbox_v1_t;
```

Initial state values:

- `AXIB_STATE_EMPTY = 0`
- `AXIB_STATE_READY = 1`

Notes:

- the inbox is intentionally one-deep in the initial version
- this matches the current single `active_execution_id` latch per worker
- queue depth remains a kernel concern, not an inbox concern
- overwrite is forbidden while the worker latch remains set

## 6. Kernel Write Contract

When a worker is scheduled and `READY -> RUNNING` pickup succeeds, Ring0 MUST:

1. ensure the worker has no active execution latch
2. copy or map the kernel-owned BCIB backing into the fixed payload window
3. write descriptor fields except `delivery_seq`
4. set `state = AXIB_STATE_READY`
5. enforce a full publish barrier
6. publish the new `delivery_seq` last

The publish order matters:

- payload window first
- descriptor content second
- publish barrier third
- `delivery_seq` last

This makes `delivery_seq` the userspace-visible commit point.

Kernel MUST ensure full memory ordering before publishing `delivery_seq`.

Kernel MUST NOT overwrite an inbox snapshot while the worker still has a
non-zero `active_execution_id` latch.

Ring0 MAY reuse the same physical payload frames across deliveries, but the
worker-visible snapshot MUST be fully overwritten before `delivery_seq`
advances.

## 7. Userspace Read Contract

Userspace worker behavior in v1:

1. read `delivery_seq`
2. if unchanged, do nothing
3. if changed and `state == AXIB_STATE_READY`, read `execution_id`,
   `bcib_user_va`, and `bcib_size`
4. treat the payload window as immutable read-only BCIB bytes
5. begin BCIB execution only after observing the new committed sequence

Userspace MUST NOT:

- write inbox descriptor fields
- write BCIB payload bytes in the mapped window
- treat the inbox as a completion or acknowledgment channel
- make scheduling decisions from inbox content

Userspace polling strategy is outside the kernel contract, but workers MUST NOT
assume interrupt-driven delivery in v1.

## 8. Boundary Rules

Forbidden designs:

- user writes inbox -> kernel reads inbox
- scheduler mailbox reused for execution payloads
- execution inbox reused for scheduling hints
- result ownership encoded by mutating the inbox from userspace

Required separation:

- scheduler mailbox = Ring3 policy -> Ring0 scheduling hints
- execution inbox = Ring0 delivery -> Ring3 worker snapshot

## 9. Completion Implication

This spec does **not** define successful completion itself.

It only defines the missing delivery half needed before a completion handoff can
be authoritative.

The next completion slice MUST add:

- a kernel-recognized completion entry point tied to `active_execution_id`
- `RUNNING -> COMPLETED` or `RUNNING -> FAILED` transitions under the
  execution-slot critical section
- latch clearing on successful completion, not only timeout
- waiter wake on completion/failure

## 10. Initial Validation Targets

The first execution inbox slice should prove:

- scheduler mailbox and execution inbox use different fixed VAs and different
  physical frames
- schedule-entry pickup publishes a descriptor into the execution inbox
- the published descriptor matches the authoritative slot `execution_id`
- userspace cannot be required as an authority source for pickup
- oversize BCIB submissions fail closed instead of partially publishing

## 11. Out of Scope

This minimal spec does not yet define:

- result ownership mapping
- repeated successful `wait_result()` semantics
- multi-worker shared execution queues
- multi-waiter timeout semantics
- SMP-safe locking beyond the current interrupt-disabled single-core model

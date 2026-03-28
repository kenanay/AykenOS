# Execution Inbox Implementation Plan

**Status:** Historical implementation plan (the delivery slice landed on 2026-03-19; retained for traceability)
**Phase:** Phase 10-B / Phase 10-C execution path hardening
**Last Updated:** 2026-03-25

This document is intentionally retained as the historical file/function plan
for the already-landed execution inbox delivery slice. References below to the
"next" slice or to missing completion/result ownership reflect pre-landing
planning state and are not the current runtime truth surface.

## 1. Goal

This plan captured how to land the first real execution delivery surface
without:

- reusing the scheduler mailbox
- widening the syscall ABI
- publishing from userspace back into Ring0

This plan covers only the delivery half:

- kernel-owned BCIB backing
- fixed-VA inbox/payload mapping
- schedule-entry descriptor publish

It does **not** include successful completion or result ownership.

## 2. Required Order

Implementation order for the next slice:

1. widen kernel-owned BCIB backing representation
2. validate real `context_id -> target user worker`
3. map execution inbox and payload window during process init
4. publish picked-up descriptors on schedule entry
5. only then start the completion handoff slice

`6.5 publish` MUST NOT land before `5.2` is true.

## 2.1 Local Code Anchors

This plan is grounded in the current kernel layout:

- `kernel/proc/proc.c:425+` already performs user-process bring-up in one place:
  image load, user stack mapping, `RING3_CANARY_ADDR`, then
  `SCHED_MAILBOX_VA = 0x700000`
- `kernel/include/proc.h` currently tracks `active_execution_id` and
  `mailbox_pa`, and now also carries execution-inbox and payload-window
  bookkeeping
- `kernel/include/execution_slot.h` now carries bounded `bcib_frames[]` plus
  `bcib_frame_count` for the 16 KiB payload window
- `kernel/sys/syscall_v2.c` now validates live `PROC_TYPE_USER` targets and
  copies bounded BCIB payloads into kernel-owned backing before `READY`
- `kernel/include/mm.h` currently exports `PRESENT`, `WRITABLE`, `USER`, and
  `GLOBAL`, and now exposes explicit NX/read-only mapping control for this path

This means the next code slice should reuse the existing process-init mapping
path rather than inventing a new bring-up path.

## 3. File-by-File Plan

### 3.1 `kernel/include/execution_slot.h`

Current landed baseline:

- bounded `bcib_frames[]`
- `bcib_frame_count`
- `bcib_size`

Remaining follow-up:

- keep slot ownership and wait-key semantics unchanged while publish prechecks
  are added

Preferred first version:

```c
#define AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES 4u

uint64_t bcib_frames[AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES];
uint32_t bcib_frame_count;
```

Reason:

- bounded frame metadata keeps the first landing simple and fail-closed

### 3.2 `kernel/sys/execution_slot.c`

Required changes:

- zero and release the widened BCIB backing metadata
- add a helper that proves whether a slot is publishable into the inbox
- keep publish preconditions centralized in execution-slot logic where possible

Suggested helper shape:

```c
int execution_slot_can_publish_locked(const exec_slot_t *slot);
```

This helper should fail if:

- slot is not `RUNNING`
- BCIB backing is incomplete
- payload size exceeds the bounded window

### 3.3 `kernel/include/proc.h`

Current landed baseline:

- `execution_inbox_pa`
- `execution_payload_pas[4]`
- `execution_delivery_seq`

These fields are mechanism state only.

They do not make the inbox authoritative.

### 3.4 `kernel/proc/proc.c`

Current landed baseline:

- fixed-VA execution delivery mapping now happens in `proc_create_user_process()`
  after scheduler mailbox mapping and before user entry state is finalized

Add a focused helper:

```c
static int proc_map_execution_delivery_surfaces(proc_t *p, uint64_t user_pml4);
```

Responsibilities:

- allocate one physical frame for `EXECUTION_INBOX_VA`
- allocate four physical frames for `EXECUTION_PAYLOAD_VA`
- zero all frames
- map inbox page user-readable and non-writable
- map payload window user-readable and non-writable
- store physical frame references in `proc_t`
- fail closed and clean up on partial allocation failure

Important:

- this helper is process-init plumbing only
- it must not publish any execution descriptor
- it must not depend on scheduler mailbox state

### 3.5 `kernel/include/mm.h` and paging flag surface

Current landed baseline:

- the common paging header now exposes explicit NX-style control and read-only
  mapping control usable by the user mapping path

No fallback to executable user mapping is allowed.

### 3.6 `kernel/sys/syscall_v2.c`

Current landed baseline:

- validates `context_id` against a live `PROC_TYPE_USER` target
- rejects `graph_size > EXECUTION_PAYLOAD_WINDOW_SIZE`
- copies BCIB bytes into kernel-owned backing frames

Remaining work before delivery publish lands:

- keep backing metadata authoritative for later inbox publish
- avoid widening semantics beyond bounded-window delivery in this slice

This is the gate that makes inbox publish honest.

### 3.7 `kernel/sched/sched.c`

Current landed baseline:

- the existing pickup hook point now publishes into the worker execution inbox
- payload bytes are written first
- descriptor fields are written second
- a full barrier is issued before `delivery_seq`
- publish failure aborts the picked-up slot fail-closed rather than leaving a
  latched half-published worker

Remaining follow-up:

- expand validation toward deterministic multi-item pickup order
- keep mailbox boundary checks explicit as delivery logic grows

Suggested helper shape:

```c
static int sched_publish_execution_delivery(proc_t *worker, const exec_slot_t *slot);
```

Publication must fail closed if:

- worker already has an active execution latch
- slot backing is incomplete
- worker delivery surfaces are missing

### 3.8 `userspace/bcib-runtime/src/executor.rs`

Do **not** change this in the same slice unless kernel-side context validation is
already ready.

Follow-on change:

- stop passing locally allocated execution IDs as `context_id`
- pass the real worker context ID instead

This is the first userspace ABI correction after kernel delivery surfaces land.

## 4. Validation Checklist

Minimum validation for the delivery slice:

- `make kernel`
- focused compile of `kernel/tests/validation/phase2_validation_test.c`
- new validation that inbox and mailbox have different fixed VAs and different
  backing frames
- new validation that oversize BCIB submission fails closed
- new validation that schedule-entry pickup publishes the authoritative
  `execution_id` into the inbox commit-point contract

## 5. Explicit Non-Goals

This slice does not yet implement:

- successful completion handoff
- `RUNNING -> COMPLETED`
- result ownership mapping
- repeated successful `wait_result()` semantics
- slot reap/release policy after terminal completion

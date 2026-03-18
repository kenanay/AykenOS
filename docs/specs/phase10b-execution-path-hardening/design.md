# Design Document

**Status:** Draft (local execution-reality stabilization plan; not roadmap authority)
**Phase:** Phase 10-B / Phase 10-C runtime stabilization

## 1. Overview

The current repo already contains most of the mechanism pieces needed for a
real execution lifecycle:

- mechanism-only scheduler
- process block/wake path
- per-process scheduler mailbox
- user process page-table roots
- monotonic PIT tick source

What is missing is the binding layer that turns:

- `map_memory`
- `submit_execution`
- `wait_result`
- `time_query`
- `exit`

into one coherent execution path.

This design keeps Ring0 mechanism-only and reuses the existing scheduler and
timer machinery instead of inventing a new runtime substrate.

## 2. Existing Repo Reality

### 2.1 Mechanisms Already Present

- `proc_block_current(wait_obj)` and `proc_wake_waiters(wait_obj)` already form
  the canonical block/wake contract.
- `sched_block_current()` already moves the current process into blocked state
  and switches via the mailbox-first scheduler.
- `timer.c` already maintains a monotonic `tick_count`.
- user processes already have dedicated `cr3`, user stack mapping, and a fixed
  scheduler mailbox VA.

### 2.2 Incomplete Areas

- `sys_v2_map_memory()` and `sys_v2_unmap_memory()` still return placeholder success.
- `sys_v2_submit_execution()` allocates an ID but does not create an execution lifecycle.
- `sys_v2_wait_result()` does not block.
- `sys_v2_time_query()` returns a dummy value.
- `sys_v2_exit()` never terminates the process.

## 3. Architecture Rules

The following rules are non-negotiable in this design:

- Ring0 remains mechanism-only.
- Ring3 remains policy-only.
- scheduler mailbox and execution dispatch MUST be separate surfaces.
- timeout authority comes from one monotonic source.
- execution state is kernel-owned until explicitly exposed through mapping.

### 3.1 Concurrency and Serialization Reality

The current kernel tree exposes interrupt enable/disable patterns but does not
yet expose a general spinlock primitive. That matters because execution-slot
state is touched from:

- syscall context
- timer IRQ context
- worker completion path
- exit/teardown path

Therefore the first implementation will not assume lock-free correctness.

Initial locking plan:

- one bounded global execution-table critical section
- entered with interrupts disabled
- exited by restoring interrupts
- all slot state mutation sites use that same discipline

This is sufficient for the current single-core runtime bring-up. It is not an
SMP-final design.

## 4. Execution Chain

The minimum real execution path is:

```text
time_query
  -> monotonic kernel tick contract

map_memory
  -> explicit user VA mapping
  -> mapping ledger entry

submit_execution
  -> kernel-owned BCIB copy
  -> execution_slot create
  -> target execution inbox enqueue

wait_result
  -> terminal? map result and return
  -> not terminal? block on execution_slot
  -> timeout/deadline? wake with timeout

exit
  -> zombie
  -> abort owned slots
  -> revoke mappings/results
  -> wake waiters
```

## 5. Components

### 5.1 Monotonic Time Contract

The first implementation uses `tick_count` as the only timeout authority.

`sys_v2_time_query()` exposes a deterministic monotonic value derived from that
counter under a frozen two-value contract:

- `TIME_QUERY_MONOTONIC = 0` returns raw monotonic PIT ticks
- `TIME_QUERY_UPTIME = 1` returns uptime milliseconds derived from PIT ticks

Any other `query_type` fails closed with `ESYS_V2_INVALID_PARAM`.

Timeout progression for execution slots happens in the timer IRQ path, not in
`wait_result` spin loops.

The first version may scan the bounded execution-slot table in IRQ context. This
is acceptable only while the table remains small and statically bounded.

### 5.2 Execution Slot Table

Introduce a kernel-owned execution-slot table with a fixed upper bound.

Suggested slot shape:

```c
typedef enum {
    EXEC_SLOT_CREATED = 0,
    EXEC_SLOT_READY,
    EXEC_SLOT_RUNNING,
    EXEC_SLOT_COMPLETED,
    EXEC_SLOT_FAILED,
    EXEC_SLOT_TIMEOUT,
    EXEC_SLOT_RESULT_MAPPED,
    EXEC_SLOT_ABORTED,
} exec_slot_state_t;

typedef struct execution_wait_key {
    uint64_t execution_id;
    uint64_t generation;
} execution_wait_key_t;

typedef struct exec_slot {
    uint64_t execution_id;
    uint64_t generation;
    uint64_t owner_pid;
    uint64_t target_context_id;
    uint64_t created_tick;
    uint64_t deadline_tick;
    exec_slot_state_t state;
    uint64_t bcib_phys;
    uint64_t bcib_size;
    uint64_t result_phys;
    uint64_t result_size;
    uint64_t mapped_result_va;
    uint64_t result_map_flags;
    uint32_t error_code;
    execution_wait_key_t wait_key;
} exec_slot_t;
```

Notes:

- `wait_obj` must point to `&slot->wait_key`, not to recyclable slot memory
  interpreted as opaque identity.
- BCIB backing is kernel-owned after submission.
- `mapped_result_va` freezes deterministic repeated `wait_result` behavior.
- `generation` prevents stale wake / stale wait ambiguity across slot reuse.
- `result_map_flags` should freeze read-only + non-executable user mapping.

### 5.3 State Transition Serialization

Every transition in the slot table is serialized by the same execution-table
critical section.

That includes:

- `CREATED -> READY`
- `READY -> RUNNING`
- `RUNNING -> COMPLETED`
- `RUNNING -> FAILED`
- `RUNNING -> TIMEOUT`
- `ANY NONTERM -> ABORTED`
- `COMPLETED -> RESULT_MAPPED`

### 5.4 Execution Inbox Boundary

Execution dispatch MUST NOT reuse `sched_mailbox`.

Reason:

- scheduler mailbox carries policy hints
- execution submission carries BCIB payload ownership and completion semantics

These are separate authority surfaces.

Chosen initial model:

- kernel-owned bounded queue keyed by `target_context_id`

If a userspace-visible execution inbox page is later used, it is only a
projection of the authoritative kernel queue.

### 5.5 Worker Pickup Model

The worker execution model is explicit in the first version:

- `submit_execution` appends a descriptor to the kernel queue for
  `target_context_id`
- when the target worker is scheduled, Ring0 checks the queue on schedule entry
- if work exists, Ring0 publishes the next descriptor to the worker delivery
  surface before returning to userspace

This avoids undefined "worker somehow picks slot" behavior.

### 5.6 Mapping Ledger

Add a process-local mapping ledger for user-visible mappings created by
`sys_v2_map_memory()`.

Suggested entry shape:

```c
typedef struct proc_mapping_entry {
    uint64_t map_id;
    uint64_t owner_pid;
    uint64_t user_va;
    uint64_t phys_addr;
    uint64_t flags;
    uint64_t capability_id;
    uint64_t created_tick;
} proc_mapping_entry_t;
```

This ledger supports:

- `unmap_memory`
- `exit` cleanup
- future capability-backed audits
- owner/capability validation for kernel-mediated remap and revoke paths

### 5.7 Result Ownership

Result memory stays kernel-owned until `wait_result` succeeds.

Initial contract:

- terminal `COMPLETED` slot maps result into caller address space
- slot transitions to `RESULT_MAPPED`
- repeated `wait_result` returns the same `mapped_result_va`
- mapped result is read-only and non-executable in user space
- `exit` revokes that mapping and marks the slot terminal for cleanup

This avoids ambiguous multi-consumer semantics in the first real version.

## 6. Syscall Behavior

### 6.1 `sys_v2_time_query`

- validate pointer
- return monotonic tick-backed value
- never return dummy constants

### 6.2 `sys_v2_map_memory`

- validate alignment and current process
- validate memory capability against requested backing
- map pages into `current_proc->context.cr3`
- record ledger entries

### 6.3 `sys_v2_unmap_memory`

- validate alignment and size
- remove ledger entries over the requested span
- unmap from the current process root only for entries owned by the caller

### 6.4 `sys_v2_submit_execution`

- validate BCIB pointer, size, and target context
- copy BCIB into kernel-owned backing
- allocate `execution_id`
- create slot in `CREATED`, then `READY`
- enqueue descriptor into execution inbox for `target_context_id`
- return `execution_id`

### 6.5 Worker Schedule-Entry Pickup

- on schedule entry, check authoritative kernel queue for `current_proc->pid`
- if a descriptor exists, publish it to the worker delivery surface
- transition slot `READY -> RUNNING` under execution-table serialization
- userspace worker begins BCIB execution only after this handoff

### 6.6 `sys_v2_wait_result`

- resolve slot by `execution_id`
- reject foreign or missing slot
- if `COMPLETED` or `RESULT_MAPPED`, return mapped result VA
- if `FAILED`, `TIMEOUT`, or `ABORTED`, return explicit error
- otherwise set `wait_obj = &slot->wait_key` and block

### 6.7 `sys_v2_exit`

- transition current process to `PROC_ZOMBIE`
- abort all non-terminal owned slots
- revoke map ledger entries
- revoke result mappings
- wake blocked waiters
- remove the process from ready and blocked scheduler queues
- switch away immediately without requeueing the exiting process

## 7. State Transition Table

```text
CREATED       -> READY           on kernel-owned BCIB copy success
CREATED       -> FAILED          on validation/copy failure
READY         -> RUNNING         on schedule-entry worker pickup
READY         -> ABORTED         on owner exit or target teardown
RUNNING       -> COMPLETED       on result ready
RUNNING       -> FAILED          on execution error
RUNNING       -> TIMEOUT         on deadline expiry
RUNNING       -> ABORTED         on owner exit or target teardown
COMPLETED     -> RESULT_MAPPED   on first successful wait_result
RESULT_MAPPED -> RESULT_MAPPED   on repeated deterministic wait_result
ANY NONTERM   -> ABORTED         on forced teardown
```

## 8. Implementation Order

Code should land in this order:

1. execution-table serialization model
2. time contract and timeout authority
3. execution-slot data model
4. submit + kernel queue creation
5. worker schedule-entry pickup
6. wait lifecycle
7. exit cleanup
8. map/unmap ledger
9. userspace ABI correction for `context_id`

This order intentionally prioritizes lifecycle authority before page-table work.

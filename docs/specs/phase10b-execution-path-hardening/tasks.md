# Implementation Tasks

**Status:** Draft
**Scope:** Phase 10-B / 10-C execution path hardening

## Tasks

- [ ] 1. Freeze the execution-slot serialization model
  - [x] 1.1 Define one concrete execution-table critical section discipline
  - [ ] 1.2 Ensure submit, wait, timeout, worker completion, and exit all use the same discipline
  - [x] 1.3 Record the current single-core assumption explicitly in code comments or docs where needed
  - [ ] 1.4 Add tests or assertions for illegal cross-state mutation sequences
  - Reference: Requirements 2, 3

- [ ] 2. Freeze the monotonic time contract
  - [x] 2.1 Define the exact `sys_v2_time_query()` unit and `query_type` behavior
  - [x] 2.2 Replace dummy time return values with PIT-backed monotonic data
  - [x] 2.3 Define timeout authority explicitly as timer IRQ path
  - [x] 2.4 Bound the initial IRQ-path timeout scan to the static slot table
  - [ ] 2.5 Add semantic tests for monotonicity and non-zero forward progress
  - Reference: Requirements 4, 5

- [ ] 3. Introduce the kernel execution-slot model
  - [x] 3.1 Add `exec_slot_state_t` with `RESULT_MAPPED` and `ABORTED`
  - [x] 3.2 Add kernel-owned execution-slot storage with bounded capacity
  - [x] 3.3 Add stable wait-key identity with generation tracking
  - [x] 3.4 Add helpers for slot lookup, allocation, and terminal-state checks
  - [ ] 3.5 Add tests for state creation, invalid ID handling, and stale generation rejection
  - Reference: Requirements 2, 4, 8

- [ ] 4. Separate execution dispatch from scheduler mailbox
  - [x] 4.1 Define a kernel-owned per-`context_id` execution queue
  - [ ] 4.2 Ensure no BCIB payload or execution result flows through scheduler mailbox ABI
  - [ ] 4.3 Add a source-level guard test or code review note for mailbox boundary separation
  - Reference: Requirements 6, 7

- [ ] 5. Implement real `sys_v2_submit_execution()`
  - [ ] 5.1 Validate BCIB pointer, size, and target context
  - [ ] 5.2 Copy BCIB into kernel-owned backing
  - [x] 5.3 Allocate `execution_id` in kernel
  - [x] 5.4 Create slot state transitions `CREATED -> READY`
  - [x] 5.5 Enqueue target execution descriptor into the kernel-owned execution queue
  - [ ] 5.6 Add tests for slot creation, copy ownership, and invalid target context
  - Reference: Requirements 2, 3, 6, 11

- [ ] 6. Implement worker pickup on schedule entry
  - [x] 6.1 Define the exact hook point where the target worker checks its authoritative execution queue
  - [x] 6.2 Transition `READY -> RUNNING` under execution-table serialization
  - [x] 6.3 Freeze the minimal execution inbox projection contract before implementation
  - [ ] 6.4 Map a kernel-written, user-read-only execution inbox at a fixed VA distinct from scheduler mailbox
  - [ ] 6.5 Publish picked-up execution descriptors into the inbox commit-point contract
  - [ ] 6.6 Add tests for deterministic pickup order and no-mailbox reuse
  - Reference: Requirements 6, 7

- [ ] 7. Implement blocking `sys_v2_wait_result()`
  - [x] 7.1 Resolve slot ownership and reject invalid or foreign execution IDs
  - [x] 7.2 Block using `proc_block_current(&slot->wait_key)` when slot is not terminal
  - [ ] 7.3 Wake via `proc_wake_waiters(&slot->wait_key)` on completion, failure, timeout, or abort
  - [ ] 7.4 Map completed result into caller address space on first successful wait
  - [ ] 7.5 Make result mapping read-only and non-executable
  - [ ] 7.6 Freeze repeated `wait_result` semantics to deterministic same-VA replay
  - [ ] 7.7 Add tests for block/wake, timeout wake, stale wait-key rejection, and repeated wait behavior
  - Reference: Requirements 4, 8

- [ ] 8. Add timeout progression in timer IRQ path
  - [x] 8.1 Scan active execution slots against monotonic deadline ticks
  - [x] 8.2 Transition overdue slots to `TIMEOUT`
  - [x] 8.3 Wake all waiters on timed-out slots
  - [ ] 8.4 Add tests proving timeout is IRQ-driven rather than syscall-spin-driven
  - Reference: Requirement 5

- [ ] 9. Implement real `sys_v2_exit()`
  - [ ] 9.1 Transition exiting process to `PROC_ZOMBIE`
  - [ ] 9.2 Abort non-terminal owned execution slots
  - [ ] 9.3 Revoke result mappings and map ledger entries
  - [ ] 9.4 Wake waiters blocked on aborted slots
  - [ ] 9.5 Remove the process from all scheduler queues
  - [ ] 9.6 Trigger immediate context switch away from the exiting process
  - [ ] 9.7 Add tests for zombie transition and cleanup side effects
  - Reference: Requirement 10

- [ ] 10. Implement explicit mapping ledger for `map_memory` / `unmap_memory`
  - [ ] 10.1 Add process-local mapping ledger entries with capability binding
  - [ ] 10.2 Implement real page mapping into `current_proc->context.cr3`
  - [ ] 10.3 Implement span unmapping only for caller-owned mappings
  - [ ] 10.4 Validate owner and capability binding on mapping lifecycle operations
  - [ ] 10.5 Add tests for page-table effects, ledger cleanup, and capability enforcement
  - Reference: Requirement 9

- [ ] 11. Correct userspace ABI usage
  - [ ] 11.1 Update `userspace/bcib-runtime` so `submit_execution(..., context_id)` passes a real context ID
  - [ ] 11.2 Keep kernel-owned `execution_id` generation as the returned value
  - [ ] 11.3 Add tests for wrapper/kernel agreement on argument meaning
  - Reference: Requirement 11

- [ ] 12. Raise runtime validation quality
  - [ ] 12.1 Replace placeholder-success assertions in validation tests with semantic assertions
  - [ ] 12.2 Add an end-to-end scenario: submit -> pickup -> wait -> exit
  - [ ] 12.3 Add an extended end-to-end scenario: map -> submit -> wait -> exit
  - [ ] 12.4 Add a negative scenario: timeout -> wake -> abort cleanup
  - [ ] 12.5 Add docs drift follow-up once implementation lands
  - Reference: Requirement 12

- [ ] 13. Keep docs synchronized while implementation lands
  - [x] 13.1 Update `progress.md` whenever a task or subtask is completed
  - [x] 13.2 Update this `tasks.md` checklist in the same change set as code
  - [x] 13.3 If runtime behavior or validation semantics materially change, update the relevant runtime docs in the same change set
  - Reference: Requirement 13

## Progress Update Rule

When implementation starts, completed work must be recorded in:

- `docs/specs/phase10b-execution-path-hardening/progress.md`

Minimum log fields per entry:

- date
- completed slice
- touched code paths
- touched docs
- validation run or explicit note that validation is pending

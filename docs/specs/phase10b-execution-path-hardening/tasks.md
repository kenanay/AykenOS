# Implementation Tasks

**Status:** Checklist complete; higher-half kheap and no-scaffold dependency removal landed; formal closure proof still in progress
**Scope:** Phase 10-B / 10-C execution path hardening

Current proof surfaces:
- source guard for serialization / mailbox-boundary / fail-closed wrapper discipline
- QEMU-backed fail-closed runtime proof export for invalid authoritative transitions
- frozen minimal replay evidence format for the fail-closed proof slice (`replay_trace.jsonl`, `replay_trace_hash.txt`, `replay_report.json`, `replay_manifest.json`, `final_state_hash.txt`, `replay_result_hash.txt`)
- adversarial multi-execution validation for replay floods, double finalize rejection, and pickup-vs-exit collision handling
- `ci-gate-no-low-half-kernel-dependency` now hard-fails any reintroduction of a low-half kernel-heap mirror and is backed by same-run runtime page-table proof for `AYKEN_KHEAP_START` across `create`, `timer_irq`, `syscall_entry`, and terminal `exit_teardown_pre/post` when present
- `ci-gate-low-half-kheap-exit-proof` remains as a validation-only exit workload that binds nested proof selection to the authoritative `exit_pid` carried by the single `[[AYKEN_LOW_HALF_KHEAP_EXIT_SELFTEST_OK]]` witness and proves terminal lower-half cleanup under QEMU without requiring a live low-half kheap scaffold
- `ci-gate-low-half-kheap-multi-exit-proof` remains as a validation-only parametric `N`-exit workload (default `N=2`, override via `AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT`) that enumerates authoritative `[[AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_LINEAGE]]` witnesses for every `exit_pid` and proves global lower-half cleanup for each enumerated exit lineage
- `ci-gate-low-half-kheap-interleaving-proof` remains as a validation-only overlap-pressure lane where every `exit_pid` is prepared before the first teardown begins, requires canonical `prepared -> armed -> lineage` ordering for every slot, and proves global lower-half cleanup even when future exits are already live
- official `ci-gate-syscall-semantics-phase10b` now requires co-located same-run Phase10-A2 evidence; ad-hoc standalone gate use may still report external review-evidence mode explicitly

Minimal replay spec:
- `docs/specs/phase10b-execution-path-hardening/fail-closed-replay-minimal-spec.md`

Current regression-specific closure checklist:
- `docs/specs/phase10b-execution-path-hardening/closure-checklist-runtime-first-authority-second.md`

Post-runtime-blocker hardening contract:
- `docs/specs/phase10b-execution-path-hardening/ring3-transition-minimal-secure-paging-contract.md`

## Tasks

- [x] 1. Freeze the execution-slot serialization model
  - [x] 1.1 Define one concrete execution-table critical section discipline
  - [x] 1.2 Ensure submit, wait, timeout, worker completion, and exit all use the same discipline
  - [x] 1.3 Record the current single-core assumption explicitly in code comments or docs where needed
  - [x] 1.4 Add tests or assertions for illegal cross-state mutation sequences
  - Reference: Requirements 2, 3

- [x] 2. Freeze the monotonic time contract
  - [x] 2.1 Define the exact `sys_v2_time_query()` unit and `query_type` behavior
  - [x] 2.2 Replace dummy time return values with PIT-backed monotonic data
  - [x] 2.3 Define timeout authority explicitly as timer IRQ path
  - [x] 2.4 Bound the initial IRQ-path timeout scan to the static slot table
  - [x] 2.5 Add semantic tests for monotonicity and non-zero forward progress
  - Reference: Requirements 1, 5

- [x] 3. Introduce the kernel execution-slot model
  - [x] 3.1 Add `exec_slot_state_t` with `RESULT_MAPPED` and `ABORTED`
  - [x] 3.2 Add kernel-owned execution-slot storage with bounded capacity
  - [x] 3.3 Add stable wait-key identity with generation tracking
  - [x] 3.4 Add helpers for slot lookup, allocation, and terminal-state checks
  - [x] 3.5 Add tests for state creation, invalid ID handling, and stale generation rejection
  - Reference: Requirements 2, 4, 8

- [x] 4. Separate execution dispatch from scheduler mailbox
  - [x] 4.1 Define a kernel-owned per-`context_id` execution queue
  - [x] 4.2 Ensure no BCIB payload or execution result flows through scheduler mailbox ABI
  - [x] 4.3 Add a source-level guard test or code review note for mailbox boundary separation
  - Reference: Requirements 6, 7
  - Source guard: `scripts/ci/check_phase10b_execution_hardening.sh`

- [x] 5. Implement real `sys_v2_submit_execution()`
  - [x] 5.1 Validate BCIB pointer, size, and target context
  - [x] 5.2 Copy BCIB into kernel-owned backing with enough slot metadata to populate the bounded payload window
  - [x] 5.3 Allocate `execution_id` in kernel
  - [x] 5.4 Create slot state transitions `CREATED -> READY`
  - [x] 5.5 Enqueue target execution descriptor into the kernel-owned execution queue
  - [x] 5.6 Add tests for slot creation, copy ownership, and invalid target context
  - Reference: Requirements 2, 3, 6, 11

- [x] 6. Implement worker pickup on schedule entry
  - [x] 6.1 Define the exact hook point where the target worker checks its authoritative execution queue
  - [x] 6.2 Transition `READY -> RUNNING` under execution-table serialization
  - [x] 6.3 Freeze the minimal execution inbox projection contract before implementation
  - [x] 6.4 Map a kernel-written, user-read-only execution inbox at a fixed VA distinct from scheduler mailbox
  - [x] 6.5 Publish picked-up execution descriptors into the inbox commit-point contract after kernel-owned BCIB backing lands
  - [x] 6.6 Add tests for deterministic pickup order and no-mailbox reuse
  - Reference: Requirements 6, 7
  - Implementation plan: `docs/specs/phase10b-execution-path-hardening/execution-inbox-implementation-plan.md`

- [x] 7. Add authoritative completion handoff
  - [x] 7.1 Ratify the preferred completion entry point before code lands
  - [x] 7.2 Keep completion distinct from `interrupt_return`, scheduler pickup, and execution inbox projection
  - [x] 7.3 Add a dedicated explicit completion surface for `execution_id` terminalization
  - [x] 7.4 Reject any completion attempt whose caller does not own the matching `active_execution_id` latch for the `RUNNING` slot
  - [x] 7.5 Serialize timeout-vs-completion races so the first terminal state wins and later attempts fail-closed
  - [x] 7.6 Transition `RUNNING -> COMPLETED/FAILED` under execution-table serialization
  - [x] 7.7 Clear `active_execution_id` on successful completion/failure terminalization
  - [x] 7.8 Wake via `proc_wake_waiters(&slot->wait_key)` on completion or failure
  - [x] 7.9 Return deterministic completion status codes for success, invalid state, permission denial, and invalid/stale `execution_id`
  - [x] 7.10 Preserve monotonic non-reused `execution_id` allocation semantics and fail-closed on wrap risk
  - [x] 7.11 Add tests for foreign completion rejection, double-completion rejection, timeout-vs-completion arbitration, latch clear, return-surface determinism, and no-ID-reuse assumptions
  - Reference: Requirements 3, 8, 11
  - Decision: `docs/specs/phase10b-execution-path-hardening/completion-handoff-decision.md`
  - Ratification: `docs/governance/ABI_EXCEPTION_COMPLETION_HANDOFF.md`

- [x] 8. Implement blocking `sys_v2_wait_result()`
  - [x] 8.1 Resolve slot ownership and reject invalid or foreign execution IDs
  - [x] 8.2 Block using `proc_block_current(&slot->wait_key)` when slot is not terminal
  - [x] 8.3 Wake via `proc_wake_waiters(&slot->wait_key)` on completion, failure, timeout, or abort
  - [x] 8.4 Map completed result into caller address space on first successful wait
  - [x] 8.5 Make result mapping read-only and non-executable
  - [x] 8.6 Freeze repeated `wait_result` semantics to deterministic same-VA replay
  - [x] 8.6.1 Replace the minimal receipt placeholder with full bounded kernel-owned output-byte materialization
  - [x] 8.7 Add tests for block/wake, timeout wake, stale wait-key rejection, and repeated wait behavior
    - [x] 8.7.1 Keep stale execution-identity rejection and repeated same-VA replay covered in the validation snapshot
    - [x] 8.7.2 Add a direct blocked-wait wake harness instead of relying only on terminal-state observation
  - Reference: Requirements 4, 8

- [x] 9. Add timeout progression in timer IRQ path
  - [x] 9.1 Scan active execution slots against monotonic deadline ticks
  - [x] 9.2 Transition overdue slots to `TIMEOUT`
  - [x] 9.3 Wake all waiters on timed-out slots
  - [x] 9.4 Add tests proving timeout is IRQ-driven rather than syscall-spin-driven
  - Reference: Requirement 5

- [x] 10. Implement real `sys_v2_exit()`
  - [x] 10.0 Fail closed for scheduler-owner exit until explicit owner handoff exists
  - [x] 10.1 Transition exiting process to `PROC_ZOMBIE`
  - [x] 10.2 Abort non-terminal owned execution slots
  - [x] 10.3 Revoke result mappings, explicit map ledger entries, and remaining user lower-half memory
    - [x] 10.3.1 Destroy user text, stack, canary, and lower-half page-table hierarchy during teardown
    - [x] 10.3.2 Defer current root-PML4 and current `rsp0` backing reap until a safe later scheduler drain point
  - [x] 10.4 Wake waiters blocked on aborted slots
  - [x] 10.5 Remove the process from all scheduler queues
  - [x] 10.6 Trigger immediate context switch away from the exiting process
  - [x] 10.7 Add tests for zombie transition and cleanup side effects
    - [x] 10.7.1 Cover helper-level zombie transition, slot abort, result revoke, and ownership release in the validation snapshot
    - [x] 10.7.2 Add a direct no-return `sys_v2_exit()` harness only if the validation framework grows one safely
  - Reference: Requirement 10

- [x] 11. Implement explicit mapping ledger for `map_memory` / `unmap_memory`
  - [x] 11.1 Add process-local mapping ledger entries with capability binding
  - [x] 11.2 Implement real page mapping into `current_proc->context.cr3`
  - [x] 11.3 Implement span unmapping only for caller-owned mappings
  - [x] 11.4 Validate owner and capability binding on mapping lifecycle operations
  - [x] 11.5 Add tests for page-table effects, ledger cleanup, and capability enforcement
  - Reference: Requirement 9

- [x] 11A. Design scheduler-owner handoff / reap before relaxing owner-exit deny
  - [x] 11A.0 Compare the two valid ratification paths: narrow mailbox-v1 exception vs promoted mailbox-v2/C2 owner-transfer path
  - [x] 11A.1 Freeze why mailbox v1/C1 cannot safely absorb runtime owner exit today
  - [x] 11A.2 Define the minimum atomic authority-transfer and reap semantics in a review candidate
  - [x] 11A.2.1 Draft the narrow mailbox-v1 owner-transfer exception candidate
  - [x] 11A.3 Ratify the narrow mailbox-v1 owner-transfer exception before changing `sys_v2_exit()` owner behavior
  - [x] 11A.4 Add narrow runtime proof for owner handoff/reap after ratification
  - [x] 11A.4.1 Prove dispatch-boundary owner commit and successor-authority mailbox application
  - [x] 11A.4.2 Prove old-owner no-return exit/reap follow-through after the authority commit
  - Reference: Requirement 10
  - Candidate: `docs/governance/SCHEDULER_OWNER_HANDOFF_REAP_CANDIDATE.md`
  - Comparison: `docs/governance/SCHEDULER_OWNER_HANDOFF_RATIFICATION_OPTIONS.md`
  - Ratified surface: `docs/governance/MAILBOX_V1_OWNER_TRANSFER_EXCEPTION.md`

- [x] 12. Correct userspace ABI usage
  - [x] 12.1 Update `userspace/bcib-runtime` so `submit_execution(..., context_id)` passes a real context ID
  - [x] 12.2 Keep kernel-owned `execution_id` generation as the returned value
  - [x] 12.3 Add tests for wrapper/kernel agreement on argument meaning
  - Reference: Requirement 11

- [x] 13. Raise runtime validation quality
  - [x] 13.1 Replace placeholder-success assertions in validation tests with semantic assertions
  - [x] 13.2 Add an end-to-end scenario: submit -> pickup -> wait -> exit
  - [x] 13.3 Add an extended end-to-end scenario: map -> submit -> wait -> exit
  - [x] 13.4 Add a negative scenario: timeout -> wake -> abort cleanup
  - [x] 13.5 Add docs drift follow-up once implementation lands
  - Reference: Requirement 12

- [x] 14. Keep docs synchronized while implementation lands
  - [x] 14.1 Update `progress.md` whenever a task or subtask is completed
  - [x] 14.2 Update this `tasks.md` checklist in the same change set as code
  - [x] 14.3 If runtime behavior or validation semantics materially change, update the relevant runtime docs in the same change set
  - Reference: Requirement 13

- [x] 15. Land the first distinct execution output plane
  - [x] 15.1 Freeze the fixed-VA output window ABI and header contract
  - [x] 15.2 Map a writable/NX fixed output window into executing workers
  - [x] 15.3 Keep output backing slot-owned and zero-fill it on pickup
  - [x] 15.4 Validate output header and bounds during `complete_execution()`
  - [x] 15.5 Publish frozen output backing through `wait_result()` instead of reusing the input BCIB bytes
  - [x] 15.6 Add semantic validation for valid output, invalid header, overflow, replay, and cleanup
  - Reference: Requirement 14
  - Spec: `docs/specs/phase10b-execution-path-hardening/execution-output-minimal-spec.md`

- [x] 16. Freeze the first minimal structured output semantic layer
  - [x] 16.1 Add a docs-first minimal structured output spec with a distinct typed header
  - [x] 16.2 Freeze the kernel boundary: opaque payload, structure/bounds enforcement only, userland-owned semantic interpretation
  - [x] 16.3 Freeze backward compatibility: raw output remains valid fallback, structured output is additive
  - [x] 16.4 Land minimal runtime support for a distinct structured-output header without widening the current syscall surface
  - [x] 16.5 Add semantic validation for known/unknown kind handling and backward-compatible raw fallback
  - Reference: Requirement 15
  - Spec: `docs/specs/phase10b-execution-path-hardening/structured-output-minimal-spec.md`

- [x] 17. Freeze the first minimal result-hash integrity layer
  - [x] 17.1 Add a docs-first minimal result-hash spec with a fixed digest subject and algorithm
  - [x] 17.2 Freeze the kernel boundary: hash frozen published bytes only; payload meaning remains userland-owned
  - [x] 17.3 Freeze backward compatibility: raw v1 and structured v2 stay unchanged; hash remains additive
  - [x] 17.4 Land minimal runtime support for a deterministic owner-visible hash sidecar without widening the current syscall surface
  - [x] 17.5 Add semantic validation for digest stability, exact-byte coverage, and revoke-on-cleanup behavior
  - Reference: Requirement 16
  - Spec: `docs/specs/phase10b-execution-path-hardening/result-hash-minimal-spec.md`

## Progress Update Rule

When implementation starts, completed work must be recorded in:

- `docs/specs/phase10b-execution-path-hardening/progress.md`

Minimum log fields per entry:

- date
- completed slice
- touched code paths
- touched docs
- validation run or explicit note that validation is pending

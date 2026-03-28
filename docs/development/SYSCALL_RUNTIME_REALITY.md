# AykenOS Syscall Runtime Reality
This document is subordinate to PHASE 0 - FOUNDATIONAL OATH. In case of
conflict, Phase 0 prevails.

**Status:** Current kernel behavior map
**Updated:** 2026-03-22

## Purpose

This file describes what the current kernel actually does at runtime for each
v2 syscall surface.

Use this document for:

- runtime truth
- maturity classification
- distinguishing semantic tests from interface-only checks

This file does not replace the ABI guide. For numbering and migration intent,
see `docs/development/SYSCALL_TRANSITION_GUIDE.md`.

## Runtime State Summary

The repository has completed the v2 ABI transition and the core execution-path
runtime contract.

Current high-level state:

- numbering and dispatch range are frozen
- `time_query` is real
- explicit generic `map_memory` / `unmap_memory` backed by a process-local
  capability ledger is real
- explicit completion handoff is real
- `wait_result()` now publishes frozen validated output bytes rather than BCIB
  input bytes
- the landed output plane accepts raw v1 and additive structured v2 headers
  while keeping payload semantics opaque to the kernel
- the landed result-hash layer adds a deterministic SHA-256 owner-visible
  sidecar over the exact frozen published result bytes
- `exit` teardown now revokes result and hash publication alongside explicit
  mappings and remaining lower-half user memory

The main follow-on work is no longer execution-path closure. It is future
semantic widening or integrity/proof expansion above the landed lifecycle,
output, structured-output, and result-hash layers.

## Maturity Matrix

| Syscall | ABI Status | Runtime Status | Notes |
|---|---|---|---|
| `map_memory` | stable | operational | validates alignment/current process, checks a bound memory capability, writes a real user PTE into the caller root, and records a process-local generic mapping ledger entry |
| `unmap_memory` | stable | operational | validates caller ownership and capability binding across the requested span, removes real PTEs from the caller root, and clears matching generic ledger entries |
| `switch_context` | stable | more mature | real process/context switch path exists |
| `submit_execution` | stable | incomplete | validates a live user target, copies bounded BCIB bytes into kernel-owned backing, creates a `READY` execution slot and queue entry, and schedule-entry pickup plus explicit completion now connect submission to terminal slot closure; successful waits now materialize the full bounded frozen output header plus payload bytes carried by the completed slot, and `exit` teardown now performs lower-half user-memory destruction with deferred active-root reap |
| `wait_result` | stable | more mature | validates ownership/state, can block on finite waits, now has direct blocked-wait/wake validation coverage for the canonical `wait_key` path, observes explicit completion/failure/timeout, maps a read-only/NX full bounded frozen output result window plus a deterministic read-only/NX hash sidecar on first successful wait, and replays the same result VA and hash VA on repeated success |
| `interrupt_return` | stable | incomplete | placeholder handler |
| `time_query` | stable | operational | PIT-backed monotonic ticks and uptime milliseconds |
| `capability_bind` | stable | more mature | capability manager-backed |
| `capability_revoke` | stable | more mature | capability manager-backed |
| `exit` | stable | incomplete | transitions to `PROC_ZOMBIE`, aborts owned/targeted non-terminal slots, revokes result mappings plus generic ledger-backed explicit mappings, destroys remaining user text/stack/canary and lower-half page tables, detaches scheduler bookkeeping, and switches away without return; active root-PML4/current-`rsp0` reap is deferred to a later safe scheduler path, and scheduler-owner exit is fail-closed until handoff exists |
| `debug_putchar` | stable | operational | real debug heartbeat path |
| `complete_execution` | stable | operational | explicit latch-bound completion surface with deterministic return codes, first-terminal-state-wins arbitration, fail-closed output-header/bounds validation for `COMPLETED` requests, and freeze-time SHA-256 hashing over the exact published result bytes |

## Current Truth by Area

### ABI Truth

These statements are currently safe:

- the v2 range is `1000-1011`
- the surface contains 12 syscalls total
- `SYS_V2_COMPLETE_EXECUTION` is included in that count
- `TIME_QUERY_MONOTONIC = 0`
- `TIME_QUERY_UPTIME = 1`

### Execution Reality Truth

These statements are currently safe:

- there is now an `execution_slot` data model in kernel space
- `submit_execution()` now anchors kernel-owned `READY` slots into that model and copies accepted BCIB bytes into kernel-owned bounded backing
- user processes now pre-map fixed execution inbox/payload windows at dedicated VAs separate from the scheduler mailbox
- schedule-entry worker pickup can advance queued work to `RUNNING` and publish a committed execution descriptor into the worker inbox
- the current pickup path allows only one active execution per user process until a terminal path clears the latch
- `submit_execution()` now rejects oversize BCIB payloads and non-live/non-user target contexts fail-closed
- execution inbox surfaces are mapped read-only and NX, and descriptor publish now follows the commit-point rule `payload -> descriptor -> barrier -> delivery_seq`
- `wait_result` no longer returns unconditional success and can block on finite timeout waits
- `wait_result(timeout_ms > 0)` can now observe explicit completion/failure as well as timeout
- the canonical blocked waiter path now has direct validation coverage for real
  `proc_block_current()` / `proc_wake_waiters()` release instead of relying only
  on terminal-state observation
- timeout progression now has direct validation coverage through the timer IRQ
  entry point rather than only helper-level timeout scans
- the first successful `wait_result()` on a `COMPLETED` slot now maps the full
  bounded frozen validated output header plus payload bytes into the owner
  address space as a read-only/NX result window
- repeated successful `wait_result()` calls now deterministically replay the
  same mapped result VA
- schedule-entry pickup now binds a slot-owned writable/NX output window at
  `EXECUTION_OUTPUT_VA` into the running worker and zero-fills it before user
  execution begins
- successful completion and timeout terminalization now clear that worker
  output-window binding when the active execution latch is released
- `complete_execution(COMPLETED)` now validates output magic, ABI version, and
  bounded `bytes_written`; invalid output metadata fail-closes as
  `FAILED + latch clear + waiter wake + ESYS_V2_INVALID_STATE`
- `complete_execution(COMPLETED)` now also accepts the structured v2 output
  header as an additive path, enforcing only known `kind`, known version, and
  bounded `bytes_written` while treating payload bytes as opaque
- the current result materialization publishes the slot-owned frozen validated
  output backing rather than reusing the original BCIB input bytes, and bytes
  past the declared logical result size inside the mapped frame span are
  zero-sealed before publication
- the completion path now also computes one SHA-256 digest over exactly those
  frozen published bytes and stores it in a slot-owned kernel-backed sidecar
  page
- the first successful `wait_result()` now maps that hash sidecar read-only/NX
  at a deterministic fixed hash VA, and repeated successful waits replay the
  same result VA and the same hash VA
- `sys_v2_exit()` now performs a real no-return teardown path for zombie
  transition, slot abort, result revoke, delivery-surface revoke, scheduler
  detachment, and switch-away
- `sys_v2_exit()` teardown now destroys the remaining lower-half user mappings
  and page-table hierarchy after explicit result, delivery, and generic-map
  revokes run
- when the exiting task is still the active process, the root PML4 frame and
  current `rsp0` backing now enter a deferred reap queue and are reclaimed on a
  later safe scheduler path instead of remaining permanently leaked
- direct runtime validation now proves the non-owner `sys_v2_exit()` no-return
  path through a validation-only forced-successor seam; this is not yet a
  general scheduler-owner handoff mechanism
- the scheduler owner process still cannot exit through `sys_v2_exit()` as the
  current active owner; that deny path remains fail-closed until the ratified
  handoff path is activated for production runtime
- narrow runtime proof now exists for dispatch-boundary owner commit and
  successor-authority mailbox application through validation-safe scheduler
  hooks, plus old-owner no-return exit/reap follow-through once authority has
  committed to the successor
- `map_memory()` now installs real caller-owned user mappings and records them
  in a process-local generic ledger
- `unmap_memory()` now removes only caller-owned ledger-backed explicit
  mappings and validates the original memory capability binding before unmapping
- `userspace/bcib-runtime` now requires an explicit target `context_id` for
  `submit_execution(...)`, passes that ID as syscall arg3, and treats the
  returned kernel value as the authoritative `execution_id`
- timer IRQ now performs a bounded slot scan that can transition overdue queued or running work to `TIMEOUT` and wake waiters
- `sys_v2_complete_execution()` now closes `RUNNING` work through an explicit dedicated kernel surface
- only the executor process that currently owns the matching `active_execution_id` latch may close the `RUNNING` slot
- timeout and explicit completion now share one terminalization lock discipline; the first successful terminal state wins
- completion now uses deterministic return codes for success, invalid state, permission denial, and invalid/stale execution ID
- the current completion model depends on monotonic non-reused `execution_id`
  allocation; slot `generation` remains internal wait-key metadata
- the ratified governance record for that exception is
  `docs/governance/ABI_EXCEPTION_COMPLETION_HANDOFF.md`

### Time Truth

These statements are currently safe:

- monotonic time comes from PIT-backed `tick_count`
- `sys_v2_time_query()` is no longer a dummy syscall
- timeout authority is specified to belong to the timer IRQ path
- timer IRQ deadline progression is now wired as a bounded static-slot scan

## Explicit Non-Guarantees

The current kernel does **not** guarantee:

- scheduler-owner handoff semantics
- indefinite wait behavior
- per-waiter timeout semantics; the current deadline is slot-scoped
- any implicit completion path through scheduler return, inbox side channels, or
  `interrupt_return`

## Test Meaning

Current test signals need careful interpretation.

### Mostly ABI or Interface Shape

- syscall count / range validation
- placeholder-success validation for incomplete syscalls

These prove:

- numbering is correct
- dispatch is reachable
- handlers return expected status shapes

These do **not** prove:

- lifecycle correctness
- indefinite-wait semantics
- full hardware interrupt delivery beyond the timer ISR entry point

### More Semantic Today

- `time_query` monotonic nondecreasing checks
- `time_query` non-zero forward progress after a direct timer-IRQ tick
- capability bind/revoke behavior
- switch-context error handling
- deterministic FIFO pickup order and no-mailbox-reuse proof for execution inbox delivery
- stale-generation wait-key rejection on blocked `wait_result`
- direct blocked-wait / canonical wake proof for `wait_result`
- direct IRQ-driven timeout proof for `wait_result`
- explicit completion authority, return codes, and first-terminal-state-wins checks
- direct non-owner `sys_v2_exit()` no-return switch-away proof with deferred
  reap follow-through via a validation-only forced-successor seam
- narrow scheduler-owner handoff proof for dispatch-boundary owner commit,
  successor-authority mailbox application, and old-owner no-return
  exit/reap follow-through under successor authority
- direct semantic end-to-end validation for `submit -> pickup -> complete ->
  wait -> exit`, including repeated same-VA replay, foreign wait denial, and
  generic mapping revoke on exit
- direct negative lifecycle validation for `submit -> pickup -> running ->
  timeout IRQ -> wake -> repeated timeout -> foreign wait denial -> cleanup`
- source-level guard for shared execution-slot serialization discipline and
  scheduler-mailbox / execution-surface separation
- direct negative state-machine validation for illegal `CREATED`, `READY`,
  `RUNNING`, `COMPLETED`, and terminal cross-state mutation attempts
- per-slot transition tracing with actor/timestamp capture for the core
  `submit -> pickup -> complete -> wait_result` lifecycle chain
- global invariant checking for duplicate IDs, per-worker `RUNNING` uniqueness,
  trace-order coherence, and immutable post-terminal states
- fail-closed runtime transition/finish enforcement on the authoritative
  submit, pickup, complete, result-map, timeout, and exit-abort paths
- runtime proof export for fail-closed transition panics: debugcon transcript,
  invariant verdict, per-slot trace rows, SHA-256 transcript hash, and a
  frozen normalized replay artifact set (`replay_trace.jsonl`,
  `replay_trace_hash.txt`, `replay_report.json`, `replay_manifest.json`,
  `final_state_hash.txt`, `replay_result_hash.txt`)
- kernel heap now lives in the higher half, and user CR3 roots no longer carry
  a dedicated low-half kernel-heap mirror; kmalloc-backed `proc_t` metadata and
  kernel stacks are now reached only through the copied kernel-half mappings
- `ci-gate-no-low-half-kernel-dependency` is now the hard fail-closed guard
  against reintroducing any low-half kheap scaffold into user CR3 roots
- same-run runtime proof for `AYKEN_KHEAP_START` still spans
  `create -> timer_irq -> syscall_entry`, but now proves a higher-half,
  supervisor-only mapping rather than a bounded low-half mirror
- the validation-only `ci-gate-low-half-kheap-exit-proof`,
  `ci-gate-low-half-kheap-multi-exit-proof`, and
  `ci-gate-low-half-kheap-interleaving-proof` lanes remain as teardown and
  lineage regression surfaces; they still require lower-half cleanup counts to
  collapse at `exit_teardown_post`, but they no longer imply a live low-half
  kheap dependency in the steady-state runtime
- adversarial multi-execution validation covering replay floods, double
  finalize rejection, stale/unknown wait rejection, and pickup-vs-exit
  collision handling

## Recommended Read Order

To understand current status without mixing target architecture and current
behavior, read in this order:

1. `docs/development/SYSCALL_TRANSITION_GUIDE.md`
2. `docs/development/SYSCALL_RUNTIME_REALITY.md`
3. `docs/specs/phase10b-execution-path-hardening/requirements.md`
4. `docs/specs/phase10b-execution-path-hardening/tasks.md`

## Next Runtime Priorities

The next follow-on work should remain:

1. widen result semantics only above the landed result-hash layer in `docs/specs/phase10b-execution-path-hardening/result-hash-minimal-spec.md`
2. keep any future integrity widening additive: hash exact frozen published bytes, not semantic meaning
3. keep the landed fail-closed replay v1 format frozen and widen proof coverage additively from `docs/specs/phase10b-execution-path-hardening/fail-closed-replay-minimal-spec.md` only when a new lifecycle surface appears
4. treat proofd / zero-copy verification as follow-on work above the landed structured semantic boundary and the landed result-hash anchor
5. keep `ci-gate-no-low-half-kernel-dependency` green and do not reintroduce any low-half kheap mirror into user CR3 roots before Phase 11 closure

The frozen minimal candidate for that first output-plane landing is:

- `docs/specs/phase10b-execution-path-hardening/execution-output-minimal-spec.md`

The active minimal candidate for the current semantic layer is:

- `docs/specs/phase10b-execution-path-hardening/structured-output-minimal-spec.md`

The active minimal contract for the current integrity layer is:

- `docs/specs/phase10b-execution-path-hardening/result-hash-minimal-spec.md`

The frozen minimal contract for the current fail-closed replay evidence slice is:

- `docs/specs/phase10b-execution-path-hardening/fail-closed-replay-minimal-spec.md`

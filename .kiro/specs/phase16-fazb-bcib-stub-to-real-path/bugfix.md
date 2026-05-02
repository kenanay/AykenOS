# Bugfix Requirements Document

## Introduction

The full BCIB stub-to-real closure goal is still valid, but the live proof/test
runtime is no longer blocked by `ctx=2` queue creation or binding. Fresh QEMU
evidence now shows the real submit, queue-create, dequeue/pickup, wait, and
result chain in one run:

- `[SUBMIT_BIND]`
- `[QUEUE_CREATE]`
- `[DEQUEUE_HIT]`
- `[PICKUP]`
- `[RESULT_VA]`
- `[WAIT_OK]`
- `[RESULT_OK]`

The bug that actually blocked closure was narrower: the worker could return
from `SYS_V2_SUBMIT_EXECUTION` or `SYS_V2_WAIT_RESULT` and then be preempted
before the first userspace instruction retired. The proof/test closure now uses
short proc-local post-syscall guards on those return paths, and the fresh gate
classifies the run as `end_to_end_completion`.

This bugfix addresses the stub-to-real path closure for the execution engine, ensuring deterministic result delivery without introducing new architectural work or directory refactoring.

## Runtime Status Note (2026-04-23)

- Ring3 handoff and first retirement are proven.
- The execution-slot queue path is also proven in the active proof/test lane:
  `submit -> queue create -> enqueue -> dequeue -> pickup -> wait -> result`.
- Repeated early `[DEQUEUE_MISS] reason=queue_not_found ctx=2` lines are now
  understood as empty-registry probes that can happen before userspace reaches
  submit. They are not the live blocker in the validated path.
- The live defect that mattered was post-syscall first-user-retirement
  starvation after successful `submit` and `wait_result` returns.
- The fresh gate
  `scripts/ci-gate-bcib-post-syscall-e2e.sh`
  plus
  `scripts/validate_bcib_post_syscall_e2e.py`
  now validates `result=PASS`, `proof_level=end_to_end_completion`, and `pf=0`
  for the proof/test BCIB worker path.
- Scope note: this is a proof/test closure claim, not a blanket production or
  governance closure claim for all Phase-16 runtime variants.

## Bug Analysis

The numbered defects below describe the narrowed bug that actually blocked the
proof/test closure path before the current fix.

### Current Behavior Before Fix (Now Closed in Proof/Test)

1.1 WHEN `sys_v2_submit_execution` returned successfully to userspace THEN the
worker could be preempted at the first post-return instruction
(`rip=0x400013B`) before that instruction retired

1.2 WHEN `sys_v2_wait_result` returned successfully to userspace THEN the
worker could be preempted at the first post-return instruction
(`rip=0x4000194`) before the result-validation path retired

1.3 WHEN those post-return retirements did not happen THEN scheduler fallback
logic kept the worker runnable, but userspace made no forward progress and the
kernel could emit misleading early empty-queue probes

1.4 WHEN the worker finally reached submit under entry masking or guard help
THEN queue creation, dequeue, and pickup all worked, proving the queue path was
not the live blocker

### Primary Root Causes (Ordered by Likelihood)

**Root Cause 1: Post-submit first-user-retirement starvation**
- `submit` succeeded, but the return to userspace could be preempted before
  `test %rax, %rax` at `0x400013B` retired
- This prevented deterministic progress into the rest of the worker path

**Root Cause 2: Post-wait first-user-retirement starvation**
- `wait_result` succeeded, but the return to userspace could be preempted
  before the worker consumed the mapped result and emitted final markers
- This produced flake-like behavior, including missing `[RESULT_OK]`

**Root Cause 3: Queue miss interpreted as primary failure**
- Early `[DEQUEUE_MISS] reason=queue_not_found ctx=2` occurred before submit in
  some runs and looked like a queue/binding defect
- Fresh proof/test evidence later proved the queue path was healthy once submit
  was actually reached

### Current Evidence from Fresh Gate

- ✅ Real submit binding observed: `[SUBMIT_BIND]`
- ✅ Queue instantiated on demand: `[QUEUE_CREATE]`
- ✅ Pickup path proven: `[DEQUEUE_HIT]` then `[PICKUP]`
- ✅ Submit-return guard armed and deferred in the fresh trace
- ✅ Wait-return guard armed and deferred in the fresh trace
- ✅ Result VA mapped: `[RESULT_VA] 0x0000000000706000`
- ✅ Wait returns successfully: `[WAIT_OK]`
- ✅ End-to-end completion observed: `[RESULT_OK]`
- ✅ Machine verdict:
  `evidence/bcib-post-syscall-e2e/bcib_post_syscall_e2e_evidence.json`
  reports `result=PASS`, `proof_level=end_to_end_completion`, `pf=0`

**Conclusion:** The live blocker is no longer queue creation/binding for
`ctx=2`. In the proof/test BCIB worker lane, the submit -> queue -> pickup ->
wait -> result chain is now proven end-to-end. The defect that mattered was
post-syscall first-user-retirement starvation, and that defect is closed in the
fresh gated run.

### Expected Behavior (Correct)

2.1 WHEN `sys_v2_submit_execution` receives a BCIB graph THEN the system SHALL validate target_context_id matches executable owner and current context

2.2 WHEN target_context_id validation fails THEN the system SHALL return `ESYS_V2_CONTEXT_ERROR` and emit `[SUBMIT_REJECTED]` marker

2.3 WHEN target_context_id validation succeeds THEN the system SHALL allocate execution slot and emit `[SUBMIT_ACCEPTED]` marker

2.4 WHEN execution slot transitions READY → RUNNING THEN the system SHALL emit `[PICKUP]` marker with slot_id, target_context_id, and current_context_id

2.5 WHEN proof-test mode is enabled AND slot is RUNNING THEN the system SHALL auto-complete execution and emit `[AUTO_COMPLETE_PROOF]` marker

2.6 WHEN execution completes THEN the system SHALL validate output_size > 0 before transitioning to COMPLETED

2.7 WHEN RUNNING → COMPLETED transition occurs THEN the system SHALL populate result buffer, compute SHA-256 fingerprint, and emit `[COMPLETION_OK]` marker

2.8 WHEN `sys_v2_wait_result` is called THEN the system SHALL validate result_size > 0 and hash_size > 0 before mapping buffers

2.9 WHEN result buffer validation fails THEN the system SHALL return `ESYS_V2_CONTEXT_ERROR` and emit `[WAIT_REJECTED]` marker

2.10 WHEN result buffer validation succeeds THEN the system SHALL map buffers to userspace and emit `[WAIT_OK]` marker

2.11 WHEN worker reads mapped result/hash buffers AND fingerprint matches expected value THEN the system SHALL emit `[RESULT_OK]` marker

2.12 WHEN the full execution ownership chain completes THEN the system SHALL have emitted all markers: `[SUBMIT_ACCEPTED]` → `[PICKUP]` → `[AUTO_COMPLETE_PROOF]` → `[COMPLETION_OK]` → `[WAIT_OK]` → `[RESULT_OK]`

### Unchanged Behavior (Regression Prevention)

3.1 WHEN Ring3 worker bootstrap executes THEN the system SHALL CONTINUE TO reach syscall 1003/1004 successfully

3.2 WHEN BCIB worker initialization occurs THEN the system SHALL CONTINUE TO produce `[BCIB_WORKER_START]` marker

3.3 WHEN submission is accepted THEN the system SHALL CONTINUE TO produce `[SUBMIT_OK]` marker (legacy compatibility marker for existing log parsers)

3.4 WHEN submission is accepted THEN the system SHALL ALSO produce `[SUBMIT_ACCEPTED]` marker (authoritative marker with context validation details)

3.5 WHEN wait operation completes THEN the system SHALL CONTINUE TO produce `[WAIT_OK]` marker

3.6 WHEN execution engine operates THEN the system SHALL CONTINUE TO maintain memory safety and capability boundaries per NON_OVERRIDABLE rules

3.7 WHEN execution state transitions occur THEN the system SHALL CONTINUE TO maintain deterministic behavior per DETERMINISM.GLOBAL constitutional rules

**Marker Authority:**
- `[SUBMIT_OK]` = legacy compatibility marker (preserved for existing CI/log parsers)
- `[SUBMIT_ACCEPTED]` = authoritative marker with validation details (new, required for closure proof)

## Result Contract (MANDATORY)

4.1 WHEN execution completes THEN the kernel SHALL produce a result buffer

4.2 The result buffer SHALL contain:
- status code (SUCCESS/FAIL)
- result length
- result payload (opaque)
- fingerprint (64-bit or 128-bit deterministic hash)

4.3 The fingerprint SHALL be computed deterministically from:
- BCIB graph input
- execution output
- execution context (excluding non-deterministic fields)

4.4 WHEN identical BCIB graph is executed multiple times THEN fingerprint SHALL be identical

4.5 WHEN fingerprint matches expected value THEN system SHALL emit `[RESULT_OK]`

4.6 WHEN fingerprint does not match THEN system SHALL emit `[RESULT_MISMATCH]`

## Execution Ownership Chain Contract (MANDATORY)

5.1 Execution ownership chain SHALL consist of six validated links:
- Link 1: Submit with valid target_context_id
- Link 2: Scheduler pickup (READY → RUNNING)
- Link 3: Execution completion (auto-complete in proof-test mode)
- Link 4: Result buffer population with fingerprint
- Link 5: Wait validation before mapping
- Link 6: Fingerprint verification

5.2 WHEN any link in the chain fails THEN the system SHALL fail-closed (return error, do NOT produce false success markers)

5.3 WHEN all links succeed THEN the system SHALL emit complete marker chain: `[SUBMIT_ACCEPTED]` → `[PICKUP]` → `[AUTO_COMPLETE_PROOF]` → `[COMPLETION_OK]` → `[WAIT_OK]` → `[RESULT_OK]`

5.4 WHEN any marker is missing THEN the execution ownership chain is incomplete and the bug is NOT fixed

5.5 Partial closure is NOT acceptable - all six links must be proven before claiming bugfix completion

## Closure Integrity Rule (MANDATORY)

5.6 IF any marker in the chain is missing THEN:
- CI MUST fail
- Closure claim MUST be rejected
- Implementation MUST be considered incomplete

5.7 Missing marker = hard fail, no partial closure claim allowed

5.8 Marker chain integrity is non-negotiable for production deployment

## Wait Semantics

6.1 `sys_v2_wait_result` SHALL block until completion OR timeout

6.2 Blocking SHALL be implemented via:
- scheduler yield (NOT busy-wait)
- deterministic wakeup on completion

6.3 WHEN execution completes THEN waiting thread SHALL be resumed

6.4 WHEN timeout occurs THEN system SHALL return TIMEOUT status

## Determinism Constraints

7.1 Execution SHALL NOT depend on:
- wall clock time
- interrupt timing
- scheduler order

7.2 Execution SHALL depend ONLY on:
- BCIB graph input
- ABDF data

7.3 Kernel SHALL ensure deterministic ordering of execution steps

7.4 Result fingerprint SHALL be reproducible across identical inputs

## Output Determinism Constraint (MANDATORY)

7.5 Execution output SHALL be deterministic

7.6 The output buffer MUST NOT depend on:
- execution timing
- scheduling order
- uninitialized memory
- non-deterministic data sources

7.7 Output MUST be derived ONLY from:
- BCIB graph input
- ABDF data

## Fingerprint Authority Rule (MANDATORY)

7.8 Fingerprint computation is kernel-owned

7.9 Userspace MUST NOT compute or override fingerprint

7.10 Kernel is the single source of truth for result verification

7.11 Fingerprint SHALL be computed as: SHA256(BCIB_size || OUTPUT_size || BCIB_data || OUTPUT_data)

## Result Buffer Layout (MANDATORY)

8.1 Result buffer SHALL be structured as:
- offset 0: `magic` (u32) - 0x54554F41 ('AOUT')
- offset 4: `abi_version` (u32) - 1
- offset 8: `flags` (u32) - reserved
- offset 12: `reserved0` (u32) - reserved
- offset 16: `bytes_written` (u64) - actual output size
- offset 24: `reserved[3]` (u64 × 3) - reserved for future use

8.2 Hash buffer SHALL be structured as:
- offset 0: `magic` (u32) - 0x48534541 ('AESH')
- offset 4: `abi_version` (u32) - 1
- offset 8: `algorithm` (u32) - AYKEN_RESULT_HASH_ALG_SHA256
- offset 12: `flags` (u32) - reserved
- offset 16: `hashed_size` (u64) - bcib_size + output_size
- offset 24: `digest[32]` (u8 × 32) - SHA-256 result
- offset 56: `reserved[16]` (u8 × 16) - reserved for future use

8.3 Layout SHALL be fixed and ABI-compatible across kernel and userspace

8.4 Alignment SHALL be 8-byte aligned for all u64 fields

8.5 Kernel and userspace MUST interpret buffer identically (no endianness conversion)

8.6 Buffer size SHALL be validated before mapping to userspace


---

**Authority:** Kenan AY - Architectural Steward  
**Status:** Requirements Complete - Ready for Design Phase  
**Closure Condition:** `[RESULT_OK]` marker observed in QEMU logs with deterministic fingerprint match

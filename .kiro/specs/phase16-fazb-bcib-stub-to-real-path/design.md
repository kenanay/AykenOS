# BCIB Stub-to-Real Path Closure Bugfix Design

## Overview

The end-state goal remains BCIB stub-to-real closure through deterministic
result delivery, but the live proof/test runtime is no longer blocked by
`ctx=2` queue creation or binding. Fresh QEMU evidence now shows real
`submit -> queue create -> dequeue/pickup -> wait -> result` completion in one
run, and the gate classifies that run as `end_to_end_completion`.

This design therefore applies to the narrowed defect that mattered during
closure: userspace could return successfully from `SYS_V2_SUBMIT_EXECUTION` or
`SYS_V2_WAIT_RESULT` and still fail to retire the first instruction after the
syscall return. The accepted proof/test design adds a short proc-local
post-syscall reschedule guard at those two return sites.

The fix focuses on the critical path: submit → execute → wait → result verification. No new files, no refactoring, no scheduler regression investigation.

## Runtime Status Note (2026-04-23)

- Ring3 execution and first retirement are proven.
- The execution-slot queue path is also proven in the active proof/test lane:
  `[SUBMIT_BIND]`, `[QUEUE_CREATE]`, `[ENQUEUE_BIND]`, `[DEQUEUE_HIT]`,
  `[PICKUP]`, `[WAIT_OK]`, and `[RESULT_OK]` can occur in one run.
- The queue miss storm is now understood as an early empty-registry probe that
  can happen before userspace reaches submit. It is not the live blocker in the
  validated proof/test path.
- The live defect that mattered was post-syscall first-user-retirement
  starvation on the submit-return and wait-return edges.
- The fresh gate
  `scripts/ci-gate-bcib-post-syscall-e2e.sh`
  plus
  `scripts/validate_bcib_post_syscall_e2e.py`
  now report `result=PASS`, `proof_level=end_to_end_completion`, and `pf=0`.
- Scope note: this is a proof/test closure, not a blanket production closure
  for all Phase-16 runtime variants.

## Glossary

- **Bug_Condition (C)**: `submit` or `wait_result` succeeds, but the first
  userspace instruction after the syscall return fails to retire deterministically
- **Property (P)**: Execution SHALL produce `[RESULT_OK]` marker with deterministic fingerprint match
- **Preservation**: All existing behavior (Ring3 bootstrap, syscall markers, memory safety) must remain unchanged
- **Execution Slot**: Kernel data structure tracking BCIB execution lifecycle (CREATED→QUEUED→RUNNING→COMPLETED)
- **Result Buffer**: Memory region containing execution output with status, length, payload, and fingerprint
- **Fingerprint**: SHA-256 hash computed deterministically from BCIB graph input + execution output
- **sys_v2_submit_execution**: Syscall 1003 - allocates execution slot and enqueues BCIB graph
- **sys_v2_wait_result**: Syscall 1004 - blocks until completion and returns result buffer VA

## Bug Details

### Bug Condition

The bug manifests when the BCIB worker returns successfully from
`SYS_V2_SUBMIT_EXECUTION` or `SYS_V2_WAIT_RESULT`, but the first userspace
instruction after that return does not retire before the next reschedule
decision. In that state the worker looks alive to the scheduler, but it does
not make deterministic forward progress. Earlier logs could then surface empty-
queue probes and other misleading symptoms even though the queue/binding path
itself was healthy once submit was actually reached.

**Formal Specification:**
```
FUNCTION isBugCondition(input)
  INPUT: input of type ExecutionFlow
  OUTPUT: boolean
  
  RETURN (input.submit_status == SUBMIT_OK
          AND input.post_submit_first_retirement == NOT_OBSERVED)
         OR
         (input.wait_status == WAIT_OK
          AND input.post_wait_first_retirement == NOT_OBSERVED)
END FUNCTION
```

### Examples

- **Example 1**: BCIB worker submits minimal graph → syscall 1003 returns → the
  timer preempts the worker at `rip=0x400013B` before `test %rax, %rax`
  retires → scheduler keeps the worker runnable but forward progress stalls
- **Example 2**: BCIB worker reaches `wait_result` → syscall 1004 returns
  successfully → the timer preempts the worker at `rip=0x4000194` before the
  result-consumption path retires → `[RESULT_OK]` never appears in that run
- **Example 3**: Early scheduler probes emit `[DEQUEUE_MISS] reason=queue_not_found ctx=2`
  before the worker reaches submit → queue absence is misdiagnosed as the
  primary blocker even though a guarded run later proves `[QUEUE_CREATE]` and
  `[DEQUEUE_HIT]`
- **Edge Case**: BCIB worker submits graph → execution times out → wait returns TIMEOUT status → worker handles timeout gracefully (expected behavior, not a bug)

## Expected Behavior

### Preservation Requirements

**Unchanged Behaviors:**
- Once richer bootstrap is restored after the proof lane, Ring3 worker bootstrap must continue to reach syscall 1003/1004 successfully
- Once richer bootstrap is restored after the proof lane, `[BCIB_WORKER_START]` marker must continue to be emitted during worker initialization
- `[SUBMIT_OK]` marker must continue to be emitted when submission is accepted
- `[WAIT_OK]` marker must continue to be emitted when wait operation completes
- Memory safety and capability boundaries per NON_OVERRIDABLE rules must be maintained
- Deterministic behavior per DETERMINISM.GLOBAL constitutional rules must be maintained

**Scope:**
All inputs that do NOT involve the result buffer population and fingerprint computation should be completely unaffected by this fix. This includes:
- Execution slot allocation and state transitions
- BCIB graph validation and storage
- Timeout handling and IRQ-driven terminalization
- Process exit cleanup and resource release

## Current Runtime Truth (2026-04-23)

The lower-level Ring3 entry problem is closed, and the queue/binding hypothesis
is also closed for the proof/test worker path. The fresh gated run shows:

- submit accepted far enough to emit `[SUBMIT_BIND]`
- queue instantiation via `[QUEUE_CREATE]`
- dequeue/pickup via `[DEQUEUE_HIT]` and `[PICKUP]`
- successful wait via `[WAIT_OK]`
- result validation via `[RESULT_OK]`
- no page faults (`pf=0`)

The defect that actually prevented closure was the lack of a protected
post-syscall retirement window after successful submit and wait returns.

## Accepted Closure Mechanism

The proof/test implementation uses a short proc-local post-syscall guard:

1. Arm a pending guard on successful `sys_v2_submit_execution` return
2. Arm a pending guard on successful `sys_v2_wait_result` return
3. Finalize the guard in the syscall-return path with the actual user return RIP
4. On timer IRQ, defer reschedule briefly while the guarded userspace return RIP
   remains inside the expected post-syscall retirement window
5. Disarm once RIP advances beyond the guarded site or the worker re-enters the
   kernel

Important implementation details:

- Guard state is proc-local and one-shot
- The wait-return guard keeps holding across the BCIB helper range so it does
  not disarm inside `emit_cstr` / result-stage helper code
- The current proof/test guard budget is `8`
- Fresh-gate validation is done by
  `scripts/ci-gate-bcib-post-syscall-e2e.sh`
  and
  `scripts/validate_bcib_post_syscall_e2e.py`

## Historical Note

The detailed queue-focused patch plan below is retained as design history. It
was useful to prove that queue creation and binding were not the live blocker,
but it should not be read as the current runtime diagnosis for the validated
proof/test path.

## Correctness Properties

Property 1: Bug Condition - Post-Syscall First-Retirement Closure

_For any_ execution where `submit` or `wait_result` succeeds in the proof/test
BCIB worker lane, the fixed runtime SHALL preserve a short post-syscall
retirement window long enough for the first userspace instruction after the
return to retire, allowing the worker to advance to the next deterministic
state and ultimately emit `[RESULT_OK]`.

**Validates: Requirements 2.4, 2.8, 2.10, 2.11, 5.1, 5.3, 7.1, 7.3**

Property 2: Preservation - Existing Execution Flow Behavior

_For any_ execution flow that does NOT involve result buffer population (slot allocation, state transitions, timeout handling, process exit), the fixed code SHALL produce exactly the same behavior as the original code, preserving all existing functionality for Ring3 bootstrap, syscall markers, memory safety, and deterministic execution.

**Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6**

## Historical Closure Strategy

### Six-Patch Fail-Closed Closure Plan

The original plan followed a strict fail-closed approach: each patch validated
one link in the execution ownership chain. If any link failed, the system would
stop immediately rather than producing false success markers.

**Execution Order:** Ownership → Pickup → Auto-Complete → Completion → Wait Validation → Result Verification

**Success Criteria:** All six links must be proven before `[RESULT_OK]` can be emitted:
1. ✅ Submit accepted with valid target
2. ✅ Pickup marker present
3. ✅ Auto-complete/proof complete marker present
4. ✅ Result/hash sizes non-zero
5. ✅ Wait success after validation
6. ✅ Fingerprint match → `[RESULT_OK]`

---

### Patch 1: Validate target_context_id at Submit (Fail-Closed Ownership)

**Goal:** Prevent wrong-target submissions from appearing successful

**File:** `kernel/sys/syscall_v2.c` (submit validation)

**Changes:**
1. During `sys_v2_submit_execution`, validate `target_context_id`:
   - Check context exists in scheduler domain
   - Check context is runnable (active in scheduler, not suspended/terminated)
   - Check context matches executable owner
   - In proof-test mode, check context matches expected test context
2. If validation fails:
   - Do NOT return `SUBMIT_OK`
   - Return fail-closed error: `ESYS_V2_CONTEXT_ERROR`
   - Log: `[SUBMIT_REJECTED] invalid_target_context reason=<context_not_found|context_not_runnable|owner_mismatch>`
3. If validation succeeds:
   - Continue with existing submit flow
   - Emit both markers for compatibility:
     - `[SUBMIT_OK]` (legacy compatibility)
     - `[SUBMIT_ACCEPTED] target_context=%llu owner_pid=%llu` (authoritative)

**Rationale:** Current logs show `target_context_id = 3` but scheduler never picks up. This patch ensures wrong-target submissions fail immediately rather than creating orphaned execution slots. The `context_is_runnable` check prevents submitting to inactive contexts that would cause perpetual `no_candidate` loops.

**Validation:**
- Before: Submit succeeds but execution never picked up
- After: Submit fails immediately if target context is wrong or not runnable
- Preservation: Valid submissions continue to work, both markers emitted

---

### Patch 2: Add Pickup Observation Point (Fail-Closed Pickup Proof)

**Goal:** Prove execution slot enters pickup chain

**File:** `kernel/sched/sched.c` (scheduler pickup logic)

**Changes:**
1. In `execution_slot_pickup_locked` (or equivalent), add deterministic marker:
   ```c
   klog("[PICKUP] slot=%llu target_ctx=%llu current_ctx=%llu state=%s->%s reason=ctx_match\n",
        slot->execution_id, slot->target_context_id, current_context_id,
        state_to_string(old_state), state_to_string(new_state));
   ```
2. Marker MUST appear when slot transitions READY → RUNNING
3. Marker MUST include `reason=ctx_match` to prove why this slot was picked
4. If marker never appears, pickup chain is broken

**Rationale:** Logs show continuous `no_candidate` but no pickup markers. This patch makes pickup observable and debuggable. The `reason` field is critical for future debugging - it proves the scheduler picked this slot because target_context_id matched current_context_id.

**Validation:**
- Before: No pickup markers, unclear if pickup happens
- After: `[PICKUP]` marker proves slot entered RUNNING state with explicit reason
- Preservation: Pickup logic unchanged, only adds logging

---

### Patch 3: Bind Auto-Complete to Pickup (Fail-Closed Proof Path)

**Goal:** In proof-test mode, auto-complete execution after pickup

**File:** `kernel/sched/sched.c` (post-pickup helper)

**Changes:**
1. Create helper: `execution_slot_auto_complete_proof(exec_slot_t *slot)`
2. Call ONLY when:
   - `AYKEN_PHASE16_BCIB_PROOF_TEST=1` is set
   - Slot state is RUNNING (after pickup)
3. Helper behavior:
   - Write minimal stub output to output window
   - If output write fails:
     - Transition slot to FAILED state
     - Log: `[AUTO_COMPLETE_FAILED] slot=%llu reason=output_write_failed`
     - Return error (do NOT proceed to completion)
   - If output write succeeds:
     - Call internal completion path
     - Log: `[AUTO_COMPLETE_PROOF] slot=%llu output_size=%llu`
4. Do NOT call from submit - only from post-pickup

**Rationale:** Minimal worker does `submit + wait` but never calls `complete_execution`. Proof path needs kernel-side auto-complete, but only after pickup to maintain ownership model. Output write failure MUST be handled explicitly to prevent silent corruption.

**Validation:**
- Before: Execution hangs in RUNNING state
- After: Proof-test executions auto-complete after pickup, or fail cleanly if output write fails
- Preservation: Non-proof executions unchanged

---

### Patch 4: Lock Completion to Output Ready (Fail-Closed Completion)

**Goal:** Prevent completion without valid output

**File:** `kernel/sys/syscall_v2.c` (completion handler)

**Changes:**
1. During RUNNING → COMPLETED transition, ALL operations MUST occur under `execution_slot_lock`:
   - Validate `slot->output_size > 0`
   - Validate output window frames are written
   - Call `execution_slot_prepare_result_locked()` (which performs):
     - Copy output window → result window
     - Compute SHA-256 fingerprint
     - Populate result buffer header
     - Populate hash buffer header
   - Transition slot state to COMPLETED
   - Wakeup waiting thread
2. Lock boundary (CRITICAL for atomicity):
   ```c
   execution_slot_enter_critical(&slot_guard, slot);
   // ALL validation, copy, hash, state transition here
   execution_slot_exit_critical(&slot_guard);
   ```
3. If validation fails:
   - Do NOT transition to COMPLETED
   - Do NOT wakeup waiting thread
   - Log: `[COMPLETION_BLOCKED] reason=no_output`
4. If validation succeeds:
   - Transition to COMPLETED
   - Wakeup waiting thread
   - Log: `[COMPLETION_OK] slot=%llu output_size=%llu`

**Rationale:** Current logs show `[WAIT_OK]` but worker crashes after. This suggests wait returns before result is ready. This patch ensures completion is atomic under a single lock, preventing race conditions where wait observes COMPLETED state but result buffer is not yet populated.

**Validation:**
- Before: Wait returns but result buffer is empty (race condition)
- After: Wait only returns when result is fully prepared (atomic under lock)
- Preservation: Completion logic unchanged, only adds validation and explicit lock boundary

---

### Patch 5: Fail-Closed Wait Result Validation

**Goal:** Prevent wait from mapping empty/invalid result buffers

**File:** `kernel/sys/syscall_v2.c` (`sys_v2_wait_result`)

**Changes:**
1. Before mapping result buffer to userspace, validate:
   - `slot->result_size > 0`
   - `slot->hash_size > 0`
   - Result buffer magic is valid
   - Hash buffer magic is valid
2. If validation fails:
   - Do NOT map buffers
   - Do NOT emit `[WAIT_OK]`
   - Return fail-closed error: `ESYS_V2_CONTEXT_ERROR`
   - Log: `[WAIT_REJECTED] reason=invalid_result`
3. After successful map, validate kernel-side alias:
   - Read back mapped VA from kernel side
   - Verify magic fields are readable
4. **CRITICAL TIMING REQUIREMENT:** `[WAIT_OK]` marker MUST be emitted AFTER:
   - Result buffer validation passes
   - Mapping succeeds
   - Kernel-side alias validation passes
5. Emit `[WAIT_OK]` only after ALL validations pass

**Rationale:** Current logs show `[WAIT_OK]` followed by crash at F%. This suggests userspace receives invalid VA. This patch converts late crash into early error. The timing requirement is MANDATORY - emitting `[WAIT_OK]` before validation creates false success that leads to userspace crashes.

**Validation:**
- Before: Wait succeeds but userspace faults on access
- After: Wait fails cleanly if result is invalid, `[WAIT_OK]` only after proven safe
- Preservation: Valid wait operations unchanged

---

### Patch 6: Result Verification with Deterministic Fingerprint

**Goal:** Emit `[RESULT_OK]` only when full chain is proven

**File:** `userspace/minimal/minimal_bcib_worker.S` or `kernel/sys/syscall_v2.c`

**Changes:**
1. Expected digest generation (DO NOT hardcode directly):
   - **Option A (Recommended):** Build-time script generates expected digest:
     ```bash
     # tools/generate_expected_digest.sh
     # Computes SHA-256 from minimal BCIB graph + stub output
     # Outputs C header with digest constant
     ```
   - **Option B:** Document reproducible computation method:
     ```
     Input: minimal_bcib_graph.bin (fixed)
     Input: stub_output.bin (fixed)
     Command: sha256sum <(cat bcib_size output_size minimal_bcib_graph.bin stub_output.bin)
     Output: expected_digest.txt
     ```
   - Store in version control: `expected_digest.txt` or generated header
2. After wait succeeds, worker reads:
   - Mapped result buffer
   - Mapped hash buffer
3. Worker compares `hash_buffer->digest` with expected digest
4. If match:
   - Emit `[RESULT_OK]`
5. If mismatch:
   - Emit `[RESULT_MISMATCH] expected=<hex> actual=<hex>`
6. Update Makefile verification to check for `[RESULT_OK]`

**Rationale:** This is the final proof that the entire chain works. Do NOT implement this until Patches 1-5 are proven. Expected digest MUST be reproducible - if BCIB graph changes, digest must be regenerated automatically or with documented script. Hardcoding without reproducibility causes test breakage when BCIB evolves.

**Validation:**
- Before: `[RESULT_MISMATCH]` due to incomplete chain
- After: `[RESULT_OK]` proves full execution ownership chain
- Preservation: All existing markers continue to work

---

### Implementation Order (STRICT)

1. **Patch 1** (ownership) → Verify submit validation works
2. **Patch 2** (pickup) → Verify pickup markers appear
3. **Patch 3** (auto-complete) → Verify proof executions complete
4. **Patch 4** (completion) → Verify completion is atomic
5. **Patch 5** (wait validation) → Verify wait fails cleanly on invalid result
6. **Patch 6** (result verification) → Verify `[RESULT_OK]` appears

**DO NOT skip ahead.** Each patch depends on the previous one being proven. If any patch fails, stop and debug before continuing.

### Fail-Closed Principle

At every stage, the system must fail cleanly rather than produce false success:
- Wrong target → Submit fails (not: submit succeeds but execution orphaned)
- No pickup → Execution times out (not: execution appears running but never completes)
- No output → Completion blocked (not: completion succeeds but result empty)
- Invalid result → Wait fails (not: wait succeeds but userspace crashes)
- Wrong fingerprint → Mismatch marker (not: false OK marker)

This approach ensures that when `[RESULT_OK]` finally appears, it represents a fully validated execution chain.

## Data Structures

### Execution Slot (Existing)

```c
typedef struct exec_slot {
    uint8_t in_use;
    uint64_t execution_id;
    uint64_t generation;
    uint64_t owner_pid;
    uint64_t target_context_id;
    uint64_t created_tick;
    uint64_t deadline_tick;
    exec_slot_state_t state;  // CREATED→READY→RUNNING→COMPLETED
    
    // BCIB graph storage
    uint64_t bcib_frames[AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES];
    uint64_t bcib_size;
    
    // Result storage
    uint64_t result_frames[AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES];
    uint64_t result_size;
    
    // Output storage
    uint64_t output_frames[AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES];
    uint64_t output_size;
    
    // Hash storage
    uint64_t hash_frame;
    uint64_t hash_size;
    uint64_t hashed_size;
    
    // Mapping state
    uint64_t mapped_result_va;
    uint64_t mapped_hash_va;
    
    execution_wait_key_t wait_key;
} exec_slot_t;
```

### Result Buffer (Existing ABI)

```c
typedef struct ayken_execution_output_v1 {
    uint32_t magic;           // 0x54554F41 ('AOUT')
    uint32_t abi_version;     // 1
    uint32_t flags;
    uint32_t reserved0;
    uint64_t bytes_written;   // Actual output size
    uint64_t reserved[3];
} ayken_execution_output_v1_t;
```

### Hash Buffer (Existing ABI)

```c
typedef struct ayken_execution_result_hash_v1 {
    uint32_t magic;           // 0x48534541 ('AESH')
    uint32_t abi_version;     // 1
    uint32_t algorithm;       // AYKEN_RESULT_HASH_ALG_SHA256
    uint32_t flags;
    uint64_t hashed_size;     // bcib_size + output_size
    uint8_t digest[32];       // SHA-256 result
    uint8_t reserved[16];
} ayken_execution_result_hash_v1_t;
```

## State Machine

### Execution Slot Lifecycle

```
CREATED (allocation)
    ↓ (BCIB graph stored)
READY (enqueued)
    ↓ (worker pickup)
RUNNING (execution in progress)
    ↓ (completion)
COMPLETED (result ready)
    ↓ (wait maps result)
RESULT_MAPPED (result delivered)
    ↓ (process exit cleanup)
[released]
```

**Critical Transition**: RUNNING → COMPLETED
- This transition MUST trigger result buffer population
- This transition MUST trigger fingerprint computation
- This transition MUST be atomic (no partial results)

## Data Flow

### Submit Path (Existing - No Changes)

```
1. Ring3 BCIB worker calls syscall 1003
2. sys_v2_submit_execution validates parameters
3. execution_slot_alloc_locked allocates slot
4. execution_slot_store_bcib_locked copies BCIB graph
5. Slot transitions CREATED → READY
6. execution_slot_enqueue_locked adds to queue
7. Returns execution_id to worker
```

### Execute Path (Proof-Test Mode - Required for Bugfix)

```
1. Ring3 BCIB worker calls syscall 1003 (submit)
2. Kernel validates target_context_id
3. Kernel allocates execution slot
4. Kernel stores BCIB graph
5. Slot transitions CREATED → READY
6. Scheduler picks up slot (READY → RUNNING)
7. **[NEW]** Kernel emits `[PICKUP]` marker
8. **[NEW]** In proof-test mode: kernel auto-completes execution
9. **[NEW]** Kernel writes minimal stub output to output window
10. **[NEW]** Kernel calls internal completion path
11. **[NEW]** Kernel emits `[AUTO_COMPLETE_PROOF]` marker
12. Kernel transitions slot RUNNING → COMPLETED
13. **[NEW]** Kernel validates output_size > 0
14. **[NEW]** Kernel calls execution_slot_prepare_result_locked
15. **[NEW]** Kernel copies output window → result window
16. **[NEW]** Kernel computes SHA-256 fingerprint
17. **[NEW]** Kernel populates result buffer header
18. **[NEW]** Kernel populates hash buffer header
19. **[NEW]** Kernel emits `[COMPLETION_OK]` marker
20. Kernel wakes waiting thread
```

### Wait Path (Enhanced - Fail-Closed Validation)

```
1. Ring3 BCIB worker calls syscall 1004 (wait)
2. sys_v2_wait_result validates execution_id
3. Loop: check slot state
4. If COMPLETED or RESULT_MAPPED:
   a. **[NEW]** Validate result_size > 0
   b. **[NEW]** Validate hash_size > 0
   c. **[NEW]** Validate result buffer magic is valid
   d. **[NEW]** Validate hash buffer magic is valid
   e. If validation fails:
      - Return ESYS_V2_CONTEXT_ERROR
      - Emit `[WAIT_REJECTED]` marker
   f. sys_v2_map_result_for_wait_locked maps buffers
   g. **[NEW]** Validate kernel-side alias (read back mapped VA)
   h. **[NEW]** Emit `[WAIT_OK]` marker only after all validations pass
   i. Returns result_va to worker
5. If RUNNING/READY/CREATED:
   a. Set deadline_tick (timeout)
   b. proc_block_current (yield to scheduler)
   c. Loop back to step 3
6. If TIMEOUT/FAILED/ABORTED:
   a. Return error code
```

## Fingerprint Computation Algorithm

### Deterministic Hash Function

```
FUNCTION compute_result_fingerprint(slot)
  INPUT: slot of type exec_slot_t*
  OUTPUT: SHA-256 digest (32 bytes)
  
  // Initialize SHA-256 context
  sha256_ctx = sha256_init()
  
  // Hash BCIB size (8 bytes, little-endian)
  sha256_update(sha256_ctx, &slot->bcib_size, 8)
  
  // Hash OUTPUT size (8 bytes, little-endian)
  sha256_update(sha256_ctx, &slot->output_size, 8)
  
  // Hash BCIB graph data (deterministic)
  FOR each frame IN slot->bcib_frames[0..bcib_frame_count-1] DO
    page_data = physical_to_virtual(frame)
    sha256_update(sha256_ctx, page_data, PAGE_SIZE)
  END FOR
  
  // Hash execution output data (deterministic)
  FOR each frame IN slot->output_frames[0..output_frame_count-1] DO
    page_data = physical_to_virtual(frame)
    sha256_update(sha256_ctx, page_data, PAGE_SIZE)
  END FOR
  
  // Finalize hash
  digest = sha256_final(sha256_ctx)
  
  // Store in hash buffer
  hash_buffer = physical_to_virtual(slot->hash_frame)
  hash_buffer->magic = AYKEN_EXECUTION_RESULT_HASH_MAGIC
  hash_buffer->abi_version = AYKEN_EXECUTION_RESULT_HASH_VERSION
  hash_buffer->algorithm = AYKEN_RESULT_HASH_ALG_SHA256
  hash_buffer->hashed_size = slot->bcib_size + slot->output_size
  memcpy(hash_buffer->digest, digest, 32)
  
  RETURN digest
END FUNCTION
```

**Authority Rule**: Fingerprint computation is kernel-owned. Userspace MUST NOT compute or override fingerprint. Kernel is the single source of truth for result verification.

### Determinism Guarantees

- **No wall clock dependency**: Uses only BCIB graph and output data
- **No interrupt timing dependency**: Computation is atomic within critical section
- **No scheduler order dependency**: Hash is computed before wakeup
- **Reproducible**: Identical BCIB graph + output → identical fingerprint
- **Size-prefixed**: Hash includes BCIB_size and OUTPUT_size to prevent collision
- **Kernel authority**: Only kernel computes fingerprint, userspace cannot override

## Fail-Closed Error Paths

### Result Buffer Validation Failure

```
IF slot->result_size == 0 OR slot->hash_size == 0 THEN
  // Result buffer not populated - fail closed
  execution_slot_exit_critical(&slot_guard)
  RETURN ESYS_V2_CONTEXT_ERROR
END IF
```

### Fingerprint Computation Failure

```
IF sha256_compute_failed THEN
  // Cannot guarantee determinism - fail closed
  slot->state = EXEC_SLOT_FAILED
  execution_slot_exit_critical(&slot_guard)
  RETURN ESYS_V2_CONTEXT_ERROR
END IF
```

### Memory Mapping Failure

```
IF sys_v2_map_result_for_wait_locked fails THEN
  // Cannot deliver result - fail closed
  execution_slot_exit_critical(&slot_guard)
  RETURN ESYS_V2_RESOURCE_BUSY
END IF
```

## Testing Strategy

### Validation Approach

The testing strategy follows a two-phase approach: first, surface counterexamples that demonstrate the bug on unfixed code, then verify the fix works correctly and preserves existing behavior.

### Exploratory Bug Condition Checking

**Goal**: Surface counterexamples that demonstrate the bug BEFORE implementing the fix. Confirm or refute the root cause analysis. If we refute, we will need to re-hypothesize.

**Test Plan**: Write tests that simulate the full BCIB execution flow (submit → wait → result verification) and observe the result buffer state. Run these tests on the UNFIXED code to observe `[RESULT_MISMATCH]` marker and understand the root cause.

**Test Cases**:
1. **Minimal BCIB Graph Test**: Submit minimal graph → wait for completion → inspect result buffer (will show empty buffer on unfixed code)
2. **Result Buffer Inspection Test**: Submit graph → wait → read result_va memory → verify magic/version fields (will show zeros on unfixed code)
3. **Fingerprint Inspection Test**: Submit graph → wait → read hash_va memory → verify digest field (will show zeros on unfixed code)
4. **Marker Observation Test**: Run full BCIB worker flow → observe QEMU logs (will show `[RESULT_MISMATCH]` on unfixed code)

**Expected Counterexamples**:
- Result buffer `magic` field is zero (not `0x54554F41`)
- Hash buffer `digest` field is all zeros
- `[RESULT_MISMATCH]` marker emitted in QEMU logs
- Possible causes: result buffer not populated, fingerprint not computed, completion handler incomplete

### Fix Checking

**Goal**: Verify that for all inputs where the bug condition holds, the fixed function produces the expected behavior.

**Pseudocode:**
```
FOR ALL input WHERE isBugCondition(input) DO
  result := execute_bcib_flow_fixed(input)
  ASSERT result.marker == "RESULT_OK"
  ASSERT result.result_buffer.magic == AYKEN_EXECUTION_OUTPUT_MAGIC
  ASSERT result.hash_buffer.magic == AYKEN_EXECUTION_RESULT_HASH_MAGIC
  ASSERT result.hash_buffer.digest != all_zeros
END FOR
```

### Preservation Checking

**Goal**: Verify that for all inputs where the bug condition does NOT hold, the fixed function produces the same result as the original function.

**Pseudocode:**
```
FOR ALL input WHERE NOT isBugCondition(input) DO
  ASSERT execute_bcib_flow_original(input) = execute_bcib_flow_fixed(input)
END FOR
```

**Testing Approach**: Property-based testing is recommended for preservation checking because:
- It generates many test cases automatically across the input domain
- It catches edge cases that manual unit tests might miss
- It provides strong guarantees that behavior is unchanged for all non-buggy inputs

**Test Plan**: Observe behavior on UNFIXED code first for slot allocation, state transitions, and timeout handling, then write property-based tests capturing that behavior.

**Test Cases**:
1. **Slot Allocation Preservation**: Observe that slot allocation works correctly on unfixed code, then write test to verify this continues after fix
2. **State Transition Preservation**: Observe that CREATED→READY→RUNNING transitions work correctly on unfixed code, then write test to verify this continues after fix
3. **Timeout Handling Preservation**: Observe that timeout IRQ-driven terminalization works correctly on unfixed code, then write test to verify this continues after fix
4. **Process Exit Cleanup Preservation**: Observe that resource cleanup works correctly on unfixed code, then write test to verify this continues after fix

### Unit Tests

- Test result buffer population with various output sizes
- Test fingerprint computation with known BCIB graph + output data
- Test fail-closed paths (empty result buffer, hash computation failure)
- Test that existing slot allocation and state transitions continue to work

### Property-Based Tests

- Generate random BCIB graphs and verify fingerprint is deterministic (same input → same hash)
- Generate random execution flows and verify preservation of slot allocation behavior
- Test that all non-result-buffer operations continue to work across many scenarios

### Integration Tests

- Test full BCIB worker flow with result verification
- Test that `[RESULT_OK]` marker is emitted after fix
- Test that fingerprint matches expected value for minimal BCIB graph
- Test that timeout handling continues to work correctly

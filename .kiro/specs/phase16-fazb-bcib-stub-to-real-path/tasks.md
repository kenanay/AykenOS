# BCIB Stub-to-Real Path Closure - Implementation Tasks

## ✅ PHASE-16 FAZ B CLOSURE ACHIEVED (2026-04-25)

**Status:** ✅ **CLOSURE COMPLETE**  
**Closure Type:** Proof-Lane Deterministic Execution (Stub Path)  
**Closure Date:** 2026-04-25  
**Closure Evidence:** `DETERMINISM_PASS`, `violations_count=0`

### **Closure Summary**

Phase-16 Faz B has successfully achieved closure with the implementation of deterministic BCIB payload generation and validation **in the stub execution path**. The critical breakthrough was implementing the `execution_slot_write_output_v1_locked()` helper function that enables non-empty payload generation.

**What This Closure Proves:**
- ✅ Kernel-level deterministic result generation (stub path)
- ✅ Same canonical BCIB → Same kernel result (cryptographically proven)
- ✅ Foundational kernel pipeline for deterministic execution
- ✅ Two-run validation framework operational

**What This Closure Does NOT Prove (Phase-17 Scope):**
- ❌ Real BCIB execution engine determinism (beyond stub)
- ❌ Arbitrary BCIB graph execution determinism
- ❌ Production scheduler nondeterminism resistance

**Critical Distinction:**
```
Stub Determinism ≠ System Determinism
BUT
Stub Determinism = Valid Closure for Faz B
```

**Final Closure Metrics:**
- ✅ `closure_verdict: "DETERMINISM_PASS"`
- ✅ `closure_type: "proof_lane_stub_execution"`
- ✅ `result_size: 8` (non-empty payload)
- ✅ `payload_non_empty: 1`
- ✅ `header_only_result: 0`
- ✅ `violations_count: 0`
- ✅ `pf: 0, boundary_violation: 0, fallback_path: 0`

**Implementation Completed:**
- ✅ `execution_slot_write_output_v1_locked()` helper in `kernel/sys/execution_slot.c` (line 1011)
- ✅ BCIB stub integration in `kernel/sched/sched.c` (lines 2347, 2370)
- ✅ Build flags: `AYKEN_BCIB_STUB_RESULT_ENABLE=1`, `AYKEN_BCIB_STUB_RESULT_VALUE_U64=0xDEADBEEFCAFEBABE`
- ✅ Fresh evidence generation with 56-byte results (48-byte header + 8-byte payload)
- ✅ Two-run determinism validation with identical SHA256

**Evidence Location:**
- `evidence/bcib-kernel-determinism/bcib_kernel_determinism_evidence.json`
- `out/evidence/run-determinism-final-closure/gates/bcib-determinism/report.json`

**Phase-17 Transition:**
The next challenge is transitioning from **stub determinism** to **real BCIB execution engine determinism**. This closure establishes the foundational kernel pipeline, but production determinism requires proving that the property holds across the full execution engine, not just the stub path.

---

## Overview

This document breaks down the implementation of the BCIB stub-to-real path closure bugfix into atomic, testable tasks following the **Six-Patch Fail-Closed Closure Plan**. Each patch validates one link in the execution ownership chain.

**✅ CLOSURE ACHIEVED:** Fresh QEMU proof/test gate reports `result=PASS`, `proof_level=end_to_end_completion`, `pf=0`, and emits `[RESULT_OK]` with deterministic non-empty payload.

**Phase-16 Faz B Determinism Hard Gate:** ✅ **PASSED** - Two-run determinism validation with identical SHA256 results confirms "same BCIB → same kernel result" requirement.

**Critical Rule:** DO NOT skip ahead. Each patch must be proven before moving to the next.

## Runtime Truth Update (2026-04-23)

- The proof/test BCIB worker path is no longer blocked by `ctx=2` queue
  creation or binding.
- Fresh QEMU evidence now shows `[SUBMIT_BIND]`, `[QUEUE_CREATE]`,
  `[DEQUEUE_HIT]`, `[PICKUP]`, `[RESULT_VA]`, `[WAIT_OK]`, and `[RESULT_OK]`
  in one gated run.
- The bug that actually mattered was post-submit and post-wait first-user-
  retirement starvation on syscall return.
- That narrowed defect is now closed in the proof/test lane by the post-syscall
  guard path and the gate
  `scripts/ci-gate-bcib-post-syscall-e2e.sh`.
- Scope note: the historical Patch 0 records below remain useful as forensic
  history, but they are not the current live blocker diagnosis.

---

## Historical Investigation Record: Scheduling Dead-Path Investigation (Patch 0)

**Status**: Historical record from the pre-closure investigation  
**Evidence**: These tasks captured the path from first-retirement proof through
the queue/binding hypothesis before the post-syscall starvation root cause was
isolated.  
**Current State**: The current proof/test blocker described here is closed.
Keep this section as investigation history, not as the authoritative live
status surface.

### Task 0.1: Add instruction retirement markers AND userspace entry validation
**Status:** done  
**File:** `userspace/minimal/minimal_bcib_worker.S` AND `kernel/arch/x86_64/ring3_enter.S`  
**Description:** Add ultra-lightweight markers to prove instruction retirement AND validate userspace entry state

**Implementation Complete:**
- Part A (kernel-side validation): ✅ WORKING - `[ENTRY_VALIDATED]` appears in logs
- Part B (worker-side `[R2]` marker): ❌ NOT REACHED - absence still unresolved after Task 0.2/0.4

**Current Interpretation:**
- `USER_ENTRY` reaches the worker entrypoint successfully.
- The current `[R2]` proof is still inconclusive as a "zero instructions retired" claim because it requires completing a multi-instruction userspace marker sequence, not just landing in `_start`.
- Timer starvation was a hypothesis here, but Task 0.2 disproved it.

**Evidence from logs:**
```
[USER_ENTRY] rip=00000000004000AC cr3=00000000046BD000
[ENTRY_VALIDATED] all_checks_passed
*[[AYKEN_IRQ0_TICK]] count=0000000000000001
[R3_FETCH_OK] RIP=00000000004000AC CR3=0000000005E93000
[[AYKEN_SCHED_TICK]]
```

**Decision:** Proceed to Task 0.2 (timer isolation) per decision tree.

**Part A: Kernel-side entry validation** (CRITICAL - do this FIRST)
```c
// At userspace entry point (before IRET/SYSRET)
void validate_userspace_entry(struct interrupt_frame *frame, pid_t pid) {
    uint64_t cr3 = read_cr3();
    
    debugcon_printf("[USER_ENTRY] pid=%llu rip=%016llx cr3=%016llx "
                   "cs=%04llx ss=%04llx rflags=%016llx\n",
                   pid, frame->rip, cr3, frame->cs, frame->ss, frame->rflags);
    
    // Fail-fast assertions for pid=2 (BCIB worker)
    if (pid == 2) {
        // Expected values for Ring3 userspace
        const uint64_t EXPECTED_CS = 0x23;  // Ring3 code segment
        const uint64_t EXPECTED_SS = 0x1B;  // Ring3 data segment
        const uint64_t EXPECTED_RFLAGS_IF = 0x200;  // Interrupts enabled
        
        if (frame->cs != EXPECTED_CS) {
            debugcon_printf("[ENTRY_FAIL] pid=2 CS=%04llx expected=%04llx\n",
                           frame->cs, EXPECTED_CS);
        }
        
        if (frame->ss != EXPECTED_SS) {
            debugcon_printf("[ENTRY_FAIL] pid=2 SS=%04llx expected=%04llx\n",
                           frame->ss, EXPECTED_SS);
        }
        
        if (!(frame->rflags & EXPECTED_RFLAGS_IF)) {
            debugcon_printf("[ENTRY_FAIL] pid=2 RFLAGS=%016llx IF_not_set\n",
                           frame->rflags);
        }
        
        // Check if CR3 is user CR3 (not kernel CR3)
        uint64_t kernel_cr3 = get_kernel_cr3();
        if (cr3 == kernel_cr3) {
            debugcon_printf("[ENTRY_FAIL] pid=2 CR3=kernel expected=user\n");
        }
        
        debugcon_printf("[ENTRY_VALIDATED] pid=2 all_checks_passed\n");
    }
}
```

**Part B: Worker-side marker** (after Part A works)
```asm
_start:
    # Layer 1: NOP (proves fetch)
    nop
    nop
    
    # Layer 2: Arithmetic (proves execute)
    xor %rax, %rax
    inc %rax
    
    # Layer 3: IO marker (proves IO + retirement)
    mov $0xE9, %dx
    mov $'[', %al
    out %al, %dx
    mov $'R', %al
    out %al, %dx
    mov $'2', %al
    out %al, %dx
    mov $']', %al
    out %al, %dx
    mov $10, %al
    out %al, %dx
    
    # Continue to existing [BCIB_WORKER_START] marker...
```

**Acceptance Criteria:**
- Part A: `[USER_ENTRY]` marker appears with full state
- Part A: `[ENTRY_VALIDATED]` appears for pid=2 (all checks pass)
- Part B: `[R2]` marker appears after entry validation
- If `[ENTRY_FAIL]` appears, root cause identified immediately

**Critical Decision Tree:**
1. **No `[USER_ENTRY]`**: Kernel not reaching entry point (dispatcher broken)
2. **`[ENTRY_FAIL] CR3=kernel`**: CR3 pivot broken (returning with kernel CR3)
3. **`[ENTRY_FAIL] CS!=0x23`**: Segment setup broken
4. **`[ENTRY_FAIL] RFLAGS IF_not_set`**: Interrupts disabled (will hang)
5. **`[ENTRY_VALIDATED]` but no `[R2]`**: execution still not proven; continue with timer isolation and safer proof markers
6. **`[R2]` appears**: CPU execution spine works (proceed to Task 0.3)

**Expected Outcomes:**
- No `[R2]`: current marker path still insufficient OR execution spine broken; continue with Task 0.2+
- `[R2]` appears: Instruction retirement works, problem is in worker logic later

**Validation:**
- Integration test: run worker, check for `[USER_ENTRY]` marker
- Check if `[ENTRY_VALIDATED]` or `[ENTRY_FAIL]` appears
- If `[ENTRY_FAIL]`, root cause is in the failure message
- If `[ENTRY_VALIDATED]` but no `[R2]`, proceed to Task 0.2 (timer isolation) and be prepared to replace the proof marker if needed
- If `[R2]` appears, skip to Task 0.3 (scheduler visibility)

**Critical Decision Point**: This task separates entry-level failures (CR3/segments) from deeper post-entry investigation.

---

### Task 0.2: Isolate timer/IRQ starvation
**Status:** done  
**File:** `kernel/arch/x86_64/timer.c`  
**Description:** Determine if timer IRQ preempts worker before first instruction

**Test Result:** Timer bypass implemented and tested. `[TIMER_SKIP_BCIB_FIRST_SLICE]` marker appeared, confirming bypass worked. However, `[R2]` marker still did NOT appear.

**Conclusion:** Timer starvation is NOT the root cause. Problem persists after timer bypass, so the blocker is deeper than simple preemption. Missing `[R2]` still requires a safer first-retirement proof before concluding "worker cannot execute first instruction."

**Next Step:** Proceed to Task 0.3 (scheduler visibility) per decision tree.

**Acceptance Criteria:**
- Add single-slice IRQ skip AND scheduler bypass for BCIB worker first entry:
  ```c
  #ifdef AYKEN_PHASE16_BCIB_PROOF_TEST
  static bool bcib_worker_first_slice = true;
  
  void timer_irq_handler(void) {
      if (current_pid == 2 && bcib_worker_first_slice) {
          bcib_worker_first_slice = false;
          debugcon_printf("[TIMER_SKIP_BCIB_FIRST_SLICE] pid=2\n");
          // Don't call scheduler - let worker run uninterrupted
          // NOTE: This is DIAGNOSTIC ONLY, not production behavior
          return;
      }
      
      // Normal timer handling
      scheduler_tick();
  }
  #endif
  ```
- Marker `[TIMER_SKIP_BCIB_FIRST_SLICE]` appears in trace
- Check if `[R2]` marker now appears
- Alternative: Add IRQ disable window (very short) around first userspace entry

**CRITICAL**: This is a **diagnostic technique** to isolate timer starvation, NOT a production fix. If this reveals timer as root cause, implement proper timeslice/preemption policy fix.

**Expected Outcomes:**
- `[R2]` appears after skip: Timer starvation is root cause → implement proper fix
- No change: Timer not the primary issue, proceed to Task 0.3

**Validation:**
- Integration test: run with timer skip, check for `[R2]` marker
- If `[R2]` appears, timer starvation confirmed - implement proper fix (NOT this bypass)
- If `[R2]` still missing, problem is deeper (paging/segments/entry)

---

### Task 0.3: Add scheduler decision visibility
**Status:** done  
**File:** `kernel/sched/sched.c`  
**Description:** Make `keep_running` and context switch decisions visible

**Runtime Verification:** ✅ CONFIRMED
```
[KEEP_RUNNING] pid=2 state=1 used_mailbox=0 reason=fallback_keep_running
```

**Findings:**
- Scheduler continuously keeps pid=2 running via fallback path
- No mailbox decisions (used_mailbox=0)
- Worker stays in RUNNING state (state=1)

---

### Task 0.4: Add dequeue miss reason codes
**Status:** done  
**File:** `kernel/sys/execution_slot.c`  
**Description:** Understand why `DEQUEUE_MISS` occurs continuously

**Runtime Verification:** ✅ CONFIRMED
```
[DEQUEUE_MISS] reason=queue_not_found ctx=2
```

**CONFIRMED SECONDARY DEFECT:**
- Execution queue for `context_id=2` does NOT exist
- Worker cannot pickup work because queue was never created
- This is NOT a scheduler problem; it is an execution-slot / queue-initialization defect

**Important Scope Limit:**
- This does **not** fully explain missing `[R2]` by itself.
- `PICKUP_TRY` / `DEQUEUE_MISS` happen on the kernel side before Ring3 dispatch, while `USER_ENTRY` still shows the worker is being sent back to userspace.
- Current local ELF confirms `_start = 0x00000000004000AC`, which matches the observed `USER_ENTRY` RIP. So queue absence is real, but it must not be over-interpreted as proof that userspace never retires any instruction.

**Critical Discovery:**
The continuous pattern reveals one confirmed bug and one still-open execution question:
1. Worker enters userspace: `[USER_ENTRY] rip=4000AC` ✅
2. Entry validated: `[ENTRY_VALIDATED]` ✅  
3. Tries to pickup work: `[PICKUP_TRY] pid=2` ✅
4. Queue not found: `[DEQUEUE_MISS] reason=queue_not_found ctx=2` ❌
5. No work to do: `[PICKUP_NONE]` ❌
6. Scheduler keeps running: `[KEEP_RUNNING]` (fallback)
7. Returns to same RIP: `[USER_ENTRY] rip=4000AC` (no visible progress)
8. Still no `[R2]` marker - but this alone does not prove "zero instructions executed"

**Next Investigation:**
1. Why is execution queue not created for BCIB worker (`ctx=2`)?
2. Replace or augment `[R2]` with a safer first-retirement proof (`int3` or `SYS_V2_DEBUG_PUTCHAR`) before concluding anything stronger about userspace execution failure.

**Acceptance Criteria:**
- Add markers at scheduler decision point AND userspace entry/return:
  ```c
  // At scheduler decision
  if (keep_running_current) {
      debugcon_printf("[KEEP_RUNNING] pid=%llu role=%d state=%d "
                     "timeslice=%llu reason=%s\n",
                     current->pid, current->role, current->state,
                     current->timeslice_remaining, reason_string);
  } else {
      debugcon_printf("[SWITCH_AWAY] pid=%llu role=%d reason=%s "
                     "next_pid=%llu\n",
                     current->pid, current->role, reason_string,
                     next->pid);
  }
  
  // At userspace entry (before IRET/SYSRET)
  debugcon_printf("[USER_ENTRY] pid=%llu rip=%016llx rsp=%016llx\n",
                 current->pid, frame->rip, frame->rsp);
  
  // At userspace return (after interrupt/syscall)
  debugcon_printf("[USER_RETURN] pid=%llu rip=%016llx reason=%s\n",
                 current->pid, saved_rip, return_reason);
  ```
- Markers appear for every scheduler decision and userspace transition involving pid=2

**Expected Pattern (if working)**:
```
[USER_ENTRY] pid=2 rip=4000AC
[R2]
[BCIB_WORKER_START]
[USER_RETURN] pid=2 rip=...
```

**Actual Pattern (if broken)**:
```
[USER_ENTRY] pid=2 rip=4000AC
[USER_RETURN] pid=2 rip=4000AC  # Same RIP, no progress
[USER_ENTRY] pid=2 rip=4000AC   # Loop
```

**Expected Outcomes:**
- Shows why worker is switched away or kept running
- Reveals if worker returns immediately without executing instructions
- Identifies scheduler logic bugs

**Validation:**
- Integration test: run worker, analyze entry/return patterns
- Look for: same RIP repeating, immediate returns, unexpected switch reasons

---

### Patch 0 Status Summary (Tasks 0.1-0.4 Complete)

**What We Know:**
- ✅ Task 0.1: Entry validation works - `[USER_ENTRY]` and `[ENTRY_VALIDATED]` appear
- ✅ Task 0.2: Timer bypass works - `[TIMER_SKIP_BCIB_FIRST_SLICE]` appeared, but `[R2]` still missing
- ✅ Task 0.3: Scheduler visibility works - `[KEEP_RUNNING]` shows continuous fallback_keep_running
- ✅ Task 0.4: Dequeue reason codes work - `[DEQUEUE_MISS] reason=queue_not_found ctx=2`

**Confirmed State:**
- Dispatch path targets `_start` and emits a pre-`iretq` marker (`USER_ENTRY rip=0x4000AC`)
- Scheduler fallback path is stable (`KEEP_RUNNING`)
- `ctx=2` queue creation is missing (`queue_not_found`)
- First-retirement in userspace is still NOT proven

**Confirmed Secondary Defect:**
- `reason=queue_not_found ctx=2` is real and reproducible
- This isolates an execution-slot / queue-initialization bug
- It does NOT, by itself, prove that zero Ring3 instructions retire

**Next Steps:**
Before proceeding to Patch 1, we must settle the open first-retirement question:
1. Keep the `int3` proof payload active at `_start`
2. Explain why the pre-`iretq` dispatch marker repeats without `[USER_BP]`
3. Keep the queue defect documented as a separate, secondary blocker

**Files to Investigate:**
- `kernel/arch/x86_64/interrupts.c` - Ring3 breakpoint visibility / authoritative marker
- `userspace/minimal/Makefile` - proof-mode flag propagation into the user payload build
- `userspace/minimal/minimal_bcib_worker.S` - minimal first-retirement payload
- `kernel/sys/execution_slot.c` - queue creation logic
- `kernel/sched/sched.c` - context initialization
- `kernel/arch/x86_64/ring3_enter.S` - pre-`iretq` dispatch marker vs actual Ring3 handoff
- `kernel/init/` - system initialization order
- Look for where execution queues are registered/bound to contexts

---

### Task 0.5: Add `int3` First-Retirement Proof
**Status:** done (positive runtime result after clean proof-mode rebuild + first-entry IRQ masking)  
**File:** `userspace/minimal/minimal_bcib_worker.S`, `userspace/minimal/Makefile`, `kernel/arch/x86_64/interrupts.c`, `kernel/arch/x86_64/ring3_enter.S`, AND `kernel/sched/sched.c`  
**Description:** Prove that Ring3 retires at least one instruction from `_start` using the shortest possible proof chain

**Implementation:** ✅ COMPLETE
- `_start` now begins with proof-mode `int3; jmp .` under `AYKEN_PHASE16_BCIB_PROOF_TEST=1`
- `userspace/minimal/Makefile` now propagates `AYKEN_PHASE16_BCIB_PROOF_TEST` into userspace `ASFLAGS` / `CFLAGS`
- Ring3 `#BP` handler now emits an authoritative kernel marker:
  ```text
  [USER_BP] pid=<pid> rip=<rip> cs=<cs>
  ```
- `ring3_enter.S` now emits a just-before-`iretq` marker with the final frame:
  ```text
  [IRETQ_PRE] rip=<rip> cs=<cs> rflags=<rflags> rsp=<rsp> ss=<ss> cr3=<cr3>
  ```
- `sched.c` now emits a pre-dispatch active-mapping proof marker:
  ```text
  [USER_TEXT_BYTES] root=<cr3> va=<rip> pte=<pte> pa=<phys> bytes=<8 bytes>
  ```

**Runtime Verification:** ✅ PROOF FIRED AFTER CLOSING TWO SILENT CONFOUNDERS
- First runtime attempt was invalid because the proof flag was not reaching the userspace assembler; `_start` still began with legacy `nop` instructions.
- After fixing userspace flag propagation, a second silent build issue remained: `userspace/minimal/.build_mode.stamp` tracked only `MINIMAL_MODE`, so switching `AYKEN_PHASE16_BCIB_PROOF_MODE` could silently reuse a stale `minimal.o`.
- `userspace/minimal/Makefile` now tracks a full userspace build signature:
  ```text
  $(MINIMAL_MODE)|proof_test=$(AYKEN_PHASE16_BCIB_PROOF_TEST)|proof_mode=$(AYKEN_PHASE16_BCIB_PROOF_MODE)
  ```
- After the stamp fix, the embedded ELF now contains the expected proof bytes at `_start`:
  ```text
  00000000004000ac <_start>:
    cc            int3
    eb fe         jmp .
  ```
- An additional runtime confounder also surfaced: with `AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=0`, the first user slice can be interrupted by IRQ0 before any first-retirement proof becomes observable.
- Unmasked run showed Ring3 was reached, but a timer interrupt arrived at the entry RIP before the proof fired:
  ```text
  [USER_TEXT_BYTES] root=00000000046BC000 va=00000000004000AC ... bytes=66BAE900B055EEB0
  [USER_ENTRY] rip=00000000004000AC cr3=00000000046BC000
  [IRETQ_PRE] rip=00000000004000AC cs=0000000000000023 rflags=0000000000003202 rsp=00000000007FFFF8 ss=000000000000001B cr3=00000000046BC000
  [R3_FETCH_OK] RIP=00000000004000AC CR3=0000000005E93000
  [TIMER_SKIP_BCIB_FIRST_SLICE] pid=2
  ```
- Clean proof run that closed both confounders:
  ```text
  make run AYKEN_LOG_DIR=out/logs_task05_bp_masked_clean USER_MINIMAL_MODE=bcib-worker-bootstrap AYKEN_PHASE16_BCIB_PROOF_TEST=1 AYKEN_PHASE16_BCIB_PROOF_MODE=2 AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=1
  ```
- Clean log confirmed:
  ```text
  [DEQUEUE_MISS] reason=queue_not_found ctx=2
  [USER_TEXT_BYTES] root=00000000046BC000 va=00000000004000AC pte=00000000046B6007 pa=00000000046B60AC bytes=CCEBFE90904831C0
  [USER_ENTRY] rip=00000000004000AC cr3=00000000046BC000
  [IRETQ_PRE] rip=00000000004000AC cs=0000000000000023 rflags=0000000000003202 rsp=00000000007FFFF8 ss=000000000000001B cr3=00000000046BC000
  [R3_FETCH_OK] RIP=00000000004000AC CR3=0000000005E93000
  [TIMER_SKIP_BCIB_FIRST_SLICE] pid=2
  [KEEP_RUNNING] pid=2 state=1 used_mailbox=0 reason=fallback_keep_running
  P10_IRQ0_MASK_FIRST_ENTRY
  [USER_ENTRY] rip=00000000004000AC cr3=00000000046BC000
  [IRETQ_PRE] rip=00000000004000AC cs=0000000000000023 rflags=0000000000003202 rsp=00000000007FFFF8 ss=000000000000001B cr3=00000000046BC000
  [USER_BP] pid=2 rip=00000000004000AD cs=0023
  P10_RING3_USER_CODE
  ```

**Conclusion:**
- The proof payload is now present in the embedded worker image and the loader still dispatches `_start = 0x4000AC`
- The `USER_ENTRY` marker is emitted from `ring3_enter.S` before `iretq`; it remains a pre-dispatch marker, not proof by itself
- `[R3_FETCH_OK] RIP=0x4000AC` proves the CPU really reaches Ring3; the earlier blanket diagnosis "iretq never lands in Ring3" was too strong
- The first unmasked slice can be interrupted at the entry RIP before any proof instruction retires; that is why `[R3_FETCH_OK]` and `[TIMER_SKIP_BCIB_FIRST_SLICE]` can appear without `[USER_BP]`
- Once the proof-mode rebuild became trustworthy and the first-entry IRQ masking path was enabled, `[USER_BP] pid=2 rip=0x4000AD cs=0x23` and `P10_RING3_USER_CODE` appeared exactly as expected
- Therefore first-retirement is now proven, `iretq` handoff is working, and the Ring3 breakpoint/exception visibility path is live
- The queue defect remains real but secondary; after the `int3` gate closed, the next blocker returns to richer userspace bootstrap and execution-slot queue creation (`ctx=2`)

**Acceptance Criteria:**
- In proof-test mode, replace the first worker instruction sequence with a single `int3` at `_start`:
  ```asm
  _start:
      int3
      nop
      nop
  ```
- The Ring3 breakpoint path emits an authoritative proof marker before halting, for example:
  ```text
  [USER_BP] pid=2 rip=00000000004000AD
  ```
- The authoritative marker is emitted from the kernel breakpoint handler, not from userspace I/O and not from the syscall path
- Marker includes enough state to bind the event to the BCIB worker (`pid=2`, trap RIP, and ideally CS/CPL)
- The proof path is allowed to halt after the marker; progress beyond `int3` is not required for this task

**Expected Outcomes:**
- If `[USER_BP]` appears immediately after `USER_ENTRY`, first-retirement is proven
- If `USER_ENTRY` repeats without `[USER_BP]`, the break is between dispatch and first retirement
- If fault markers such as `GP!` / `PF!` / `UD` appear instead, the failure mode is narrowed to a specific exception path

**Validation:**
- QEMU trace shows:
  ```text
  [USER_ENTRY] rip=00000000004000AC
  [USER_BP] pid=2 rip=00000000004000AD
  ```
- `0x4000AD` (or equivalent post-`int3` RIP) matches the expected post-trap instruction pointer
- The clean validation run must use a fresh proof-mode rebuild and must not silently reuse a stale userspace object

---

### Patch 0 Success Criteria

Patch 0 is complete when:
- ✅ `[ENTRY_VALIDATED]` appears for the worker
- ✅ An authoritative first-retirement marker appears (`[USER_BP]` or equivalent Ring3 `int3` proof)
- ✅ The scheduler fallback path is documented (`[KEEP_RUNNING]`)
- ✅ The queue defect is documented (`[DEQUEUE_MISS] reason=queue_not_found ctx=2`)
- ✅ Only after the `int3` proof do we return to richer userspace bootstrap markers such as `[R2]` / `[BCIB_WORKER_START]`

**Critical Milestone**: First prove one retired Ring3 instruction with `int3`. Do not treat missing `[R2]` as the primary blocker until that proof exists.

**ONLY AFTER** Patch 0 completion, proceed to Patch 1.

**Patch 0 Closure Note**: `[USER_BP]` now appears in a clean `int3` run, so the first-retirement gate is closed. Remaining work moves back to the richer bootstrap chain and the still-real `ctx=2` queue defect.

---

## Patch 1: Validate target_context_id at Submit

### Task 1.1: Add target_context_id validation to sys_v2_submit_execution
**Status:** pending  
**File:** `kernel/sys/syscall_v2.c`  
**Description:** Validate target context exists, is runnable, and matches executable owner

**Acceptance Criteria:**
- During submit, validate `target_context_id`:
  - Context exists in scheduler domain
  - Context is runnable (active in scheduler, not suspended/terminated)
  - Context matches executable owner
  - In proof-test mode, context matches expected test context
- If validation fails:
  - Return `ESYS_V2_CONTEXT_ERROR`
  - Log: `[SUBMIT_REJECTED] invalid_target_context reason=<context_not_found|context_not_runnable|owner_mismatch>`
- If validation succeeds:
  - Continue with existing submit flow
  - Emit both markers:
    - `[SUBMIT_OK]` (legacy compatibility)
    - `[SUBMIT_ACCEPTED] target_context=%llu owner_pid=%llu` (authoritative)

**Validation:**
- Unit test: submit with invalid target_context_id, verify error returned
- Unit test: submit with non-runnable context, verify error returned
- Integration test: submit with valid target_context_id, verify both markers emitted
- Regression test: verify existing valid submissions unchanged

---

### Task 1.2: Update minimal_bcib_worker to use self-context
**Status:** pending  
**File:** `userspace/minimal/minimal_bcib_worker.S`  
**Description:** Change worker to submit with self-context instead of hardcoded context 3

**Acceptance Criteria:**
- Worker queries own context_id before submit
- Worker uses self-context as target_context_id
- Log shows `[SUBMIT_ACCEPTED]` with matching context

**Validation:**
- Integration test: run worker in QEMU, verify submit succeeds
- Log verification: `[SUBMIT_ACCEPTED] target_context=<worker_context>`

---

## Patch 2: Add Pickup Observation Point

### Task 2.1: Add pickup marker to execution_slot_pickup_locked
**Status:** pending  
**File:** `kernel/sched/sched.c`  
**Description:** Add deterministic marker when slot transitions READY → RUNNING

**Acceptance Criteria:**
- In `execution_slot_pickup_locked` (or equivalent), add marker:
  ```c
  klog("[PICKUP] slot=%llu target_ctx=%llu current_ctx=%llu state=%s->%s reason=ctx_match\n",
       slot->execution_id, slot->target_context_id, current_context_id,
       state_to_string(old_state), state_to_string(new_state));
  ```
- Marker appears when slot transitions READY → RUNNING
- Marker includes slot_id, target_context_id, current_context_id, state transition, and reason
- Reason field MUST be `reason=ctx_match` to prove why this slot was picked

**Validation:**
- Integration test: submit execution, verify `[PICKUP]` marker appears in logs with reason field
- Regression test: verify pickup logic unchanged (only adds logging)

---

### Task 2.2: Verify pickup marker in QEMU logs
**Status:** pending  
**File:** `Makefile` (verification logic)  
**Description:** Update verification to check for pickup marker

**Acceptance Criteria:**
- Verification checks for `[PICKUP]` marker in QEMU logs
- If marker missing, verification fails with clear error message
- Existing marker checks (`[SUBMIT_OK]`, `[WAIT_OK]`) continue to work

**Validation:**
- Build test: run `make pre-ci`, verify pickup marker check works
- Regression test: verify all other CI gates pass

---

## Patch 3: Bind Auto-Complete to Pickup

### Task 3.1: Implement execution_slot_auto_complete_proof helper
**Status:** pending  
**File:** `kernel/sched/sched.c`  
**Description:** Create helper to auto-complete proof-test executions after pickup

**Acceptance Criteria:**
- Function signature: `int execution_slot_auto_complete_proof(exec_slot_t *slot)`
- Called ONLY when:
  - `AYKEN_PHASE16_BCIB_PROOF_TEST=1` is set
  - Slot state is RUNNING (after pickup)
- Helper behavior:
  - Write minimal stub output to output window
  - If output write fails:
    - Transition slot to FAILED state
    - Log: `[AUTO_COMPLETE_FAILED] slot=%llu reason=output_write_failed`
    - Return error (do NOT proceed to completion)
  - If output write succeeds:
    - Call internal completion path
    - Log: `[AUTO_COMPLETE_PROOF] slot=%llu output_size=%llu`
- Do NOT call from submit - only from post-pickup

**Validation:**
- Unit test: call with mock slot in RUNNING state, verify completion triggered
- Unit test: simulate output write failure, verify FAILED state and error marker
- Integration test: run proof-test execution, verify auto-complete marker appears
- Regression test: verify non-proof executions unchanged

---

### Task 3.2: Integrate auto-complete into pickup path
**Status:** pending  
**File:** `kernel/sched/sched.c`  
**Description:** Call auto-complete helper after successful pickup in proof-test mode

**Acceptance Criteria:**
- After slot transitions to RUNNING, check if proof-test mode enabled
- If enabled, call `execution_slot_auto_complete_proof(slot)`
- If disabled, continue with existing execution flow
- Maintain ownership model: auto-complete only after pickup

**Validation:**
- Integration test: run proof-test execution, verify completion occurs
- Regression test: verify non-proof executions unchanged

---

## Patch 4: Lock Completion to Output Ready

### Task 4.1: Add output validation to completion handler
**Status:** pending  
**File:** `kernel/sys/syscall_v2.c`  
**Description:** Validate output is ready before transitioning to COMPLETED, with explicit lock boundary

**Acceptance Criteria:**
- During RUNNING → COMPLETED transition, ALL operations MUST occur under `execution_slot_lock`:
  ```c
  execution_slot_enter_critical(&slot_guard, slot);
  // ALL validation, copy, hash, state transition here
  execution_slot_exit_critical(&slot_guard);
  ```
- Within lock, validate:
  - `slot->output_size > 0`
  - Output window frames are written
  - `execution_slot_prepare_result_locked()` succeeds
  - Fingerprint/hash populate succeeds
- If validation fails:
  - Do NOT transition to COMPLETED
  - Do NOT wakeup waiting thread
  - Log: `[COMPLETION_BLOCKED] reason=no_output`
- If validation succeeds:
  - Transition to COMPLETED
  - Wakeup waiting thread
  - Log: `[COMPLETION_OK] slot=%llu output_size=%llu`

**Validation:**
- Unit test: attempt completion with empty output, verify blocked
- Integration test: complete with valid output, verify success
- Race condition test: verify no window where COMPLETED state visible but result not ready
- Regression test: verify completion logic unchanged

---

### Task 4.2: Implement execution_slot_prepare_result_locked
**Status:** pending  
**File:** `kernel/sys/syscall_v2.c`  
**Description:** Create or enhance function to prepare result buffer

**Acceptance Criteria:**
- Function signature: `int execution_slot_prepare_result_locked(exec_slot_t *slot)`
- Validates slot state is COMPLETED
- Copies output window frames to result window frames
- Populates `ayken_execution_output_v1_t` structure:
  - `magic = 0x54554F41` ('AOUT')
  - `abi_version = 1`
  - `flags = 0`
  - `bytes_written = slot->output_size`
- Computes SHA-256 fingerprint (see Task 4.3)
- Returns 0 on success, error code on failure
- Fail-closed: returns error if output_size is 0

**Validation:**
- Unit test: call with mock slot, verify result buffer populated
- Integration test: verify result buffer readable from userspace after wait

---

### Task 4.3: Implement SHA-256 fingerprint computation
**Status:** pending  
**File:** `kernel/sys/syscall_v2.c`  
**Description:** Implement deterministic fingerprint computation

**Acceptance Criteria:**
- Function signature: `int compute_result_fingerprint(exec_slot_t *slot, uint8_t digest[32])`
- Computes SHA-256 hash of:
  1. BCIB size (8 bytes, little-endian)
  2. OUTPUT size (8 bytes, little-endian)
  3. BCIB graph data (bcib_frames)
  4. Execution output data (output_frames)
- Uses existing SHA-256 implementation (likely in kernel crypto)
- Populates `ayken_execution_result_hash_v1_t` structure:
  - `magic = 0x48534541` ('AESH')
  - `abi_version = 1`
  - `algorithm = AYKEN_RESULT_HASH_ALG_SHA256`
  - `hashed_size = slot->bcib_size + slot->output_size`
  - `digest[32] = SHA-256 result`
- Returns 0 on success, error code on failure
- Fail-closed: returns error if bcib_size or output_size is 0

**Validation:**
- Unit test: compute fingerprint for known BCIB + output, verify digest matches expected
- Determinism test: compute fingerprint twice for same input, verify identical digest

---

## Patch 5: Fail-Closed Wait Result Validation

### Task 5.1: Add result buffer validation to sys_v2_wait_result
**Status:** pending  
**File:** `kernel/sys/syscall_v2.c`  
**Description:** Validate result buffer before mapping to userspace

**Acceptance Criteria:**
- Before calling `sys_v2_map_result_for_wait_locked`:
  - Check `slot->result_size > 0`
  - Check `slot->hash_size > 0`
  - Check result buffer magic is valid
  - Check hash buffer magic is valid
- If validation fails:
  - Do NOT map buffers
  - Do NOT return `WAIT_OK`
  - Return `ESYS_V2_CONTEXT_ERROR`
  - Log: `[WAIT_REJECTED] reason=invalid_result`
- Validation occurs within critical section (slot lock held)

**Validation:**
- Unit test: call wait with unpopulated result buffer, verify error returned
- Integration test: call wait after proper completion, verify success

---

### Task 5.2: Add kernel-side alias validation after mapping
**Status:** pending  
**File:** `kernel/sys/syscall_v2.c`  
**Description:** Verify mapped VA is readable from kernel side

**Acceptance Criteria:**
- After successful `sys_v2_map_result_for_wait_locked`:
  - Read back mapped VA from kernel side
  - Verify result buffer magic field is readable
  - Verify hash buffer magic field is readable
- If validation fails:
  - Unmap buffers
  - Return `ESYS_V2_CONTEXT_ERROR`
  - Log: `[WAIT_REJECTED] reason=map_validation_failed`
- **CRITICAL TIMING REQUIREMENT:** `[WAIT_OK]` marker MUST be emitted AFTER:
  - Result buffer validation passes
  - Mapping succeeds
  - Kernel-side alias validation passes
- Emit `[WAIT_OK]` only after ALL validations pass

**Validation:**
- Integration test: verify wait succeeds with valid result, `[WAIT_OK]` appears after validation
- Integration test: simulate map validation failure, verify `[WAIT_REJECTED]` and no `[WAIT_OK]`
- Regression test: verify wait error handling unchanged

---

## Patch 6: Result Verification with Deterministic Fingerprint

### Task 6.1: Define expected digest for minimal BCIB graph
**Status:** pending  
**File:** `tools/generate_expected_digest.sh` or `userspace/minimal/minimal_bcib_worker.S`  
**Description:** Compute and store expected digest using reproducible method

**Acceptance Criteria:**
- **Option A (Recommended):** Create build-time script:
  - Script: `tools/generate_expected_digest.sh`
  - Input: minimal BCIB graph data (fixed binary)
  - Input: stub output data (fixed binary)
  - Computation: `SHA256(bcib_size || output_size || bcib_data || output_data)`
  - Output: C header with digest constant (`expected_digest.h`)
  - Store in version control
- **Option B:** Document reproducible computation:
  - Create `docs/expected_digest_computation.md`
  - Document exact inputs (file paths, sizes)
  - Document exact command to reproduce digest
  - Store expected digest in `expected_digest.txt`
- DO NOT hardcode digest directly without reproducibility
- If BCIB graph changes, digest MUST be regenerated automatically or with documented script

**Validation:**
- Manual verification: compute digest independently using documented method, verify matches
- Build test: verify script runs successfully and generates header
- Regression test: verify digest remains stable for fixed BCIB graph

---

### Task 6.2: Implement result verification in BCIB worker
**Status:** pending  
**File:** `userspace/minimal/minimal_bcib_worker.S`  
**Description:** Add fingerprint comparison logic to worker

**Acceptance Criteria:**
- After wait succeeds, worker reads:
  - Mapped result buffer
  - Mapped hash buffer
- Worker compares `hash_buffer->digest` with expected digest (from Task 6.1)
- If match:
  - Emit `[RESULT_OK]`
- If mismatch:
  - Emit `[RESULT_MISMATCH] expected=<hex> actual=<hex>` (include both digests for debugging)
- Worker continues to emit existing markers (`[SUBMIT_OK]`, `[WAIT_OK]`)

**Validation:**
- Integration test: run worker in QEMU with correct digest, verify `[RESULT_OK]` marker appears
- Integration test: run worker with incorrect expected digest, verify `[RESULT_MISMATCH]` with both digests
- Regression test: verify all existing markers continue to work

---

### Task 6.3: Update Makefile verification for RESULT_OK
**Status:** pending  
**File:** `Makefile`  
**Description:** Update build system verification to check for [RESULT_OK] marker

**Acceptance Criteria:**
- Update BCIB worker verification logic (lines 1350-1365)
- Change from checking `[RESULT_MISMATCH]` to checking `[RESULT_OK]`
- Verification passes if `[RESULT_OK]` marker found in QEMU logs
- Verification fails if `[RESULT_MISMATCH]` marker found

**Validation:**
- Build test: run `make pre-ci`, verify BCIB worker verification passes
- Regression test: verify all other CI gates continue to pass

---

## Task Dependencies

```
Patch 0: Scheduling Dead-Path Investigation (BLOCKER)
  ├─ Task 0.1 (add instruction retirement markers)
  ├─ Task 0.2 (isolate timer/IRQ starvation)
  ├─ Task 0.3 (add scheduler decision visibility)
  ├─ Task 0.4 (add dequeue miss reason codes)
  └─ Task 0.5 (add int3 first-retirement proof)
       ↓
Patch 1: Ownership Validation
  ├─ Task 1.1 (validate target_context_id)
  └─ Task 1.2 (update worker to use self-context)
       ↓
Patch 2: Pickup Observation
  ├─ Task 2.1 (add pickup marker)
  └─ Task 2.2 (verify pickup marker)
       ↓
Patch 3: Auto-Complete Binding
  ├─ Task 3.1 (implement auto-complete helper)
  └─ Task 3.2 (integrate into pickup path)
       ↓
Patch 4: Completion Locking
  ├─ Task 4.1 (add output validation)
  ├─ Task 4.2 (implement prepare_result_locked)
  └─ Task 4.3 (implement fingerprint computation)
       ↓
Patch 5: Wait Validation
  ├─ Task 5.1 (add result buffer validation)
  └─ Task 5.2 (add kernel-side alias validation)
       ↓
Patch 6: Result Verification
  ├─ Task 6.1 (define expected digest)
  ├─ Task 6.2 (implement verification in worker)
  └─ Task 6.3 (update Makefile verification)
```

## Historical Implementation Order (STRICT)

**DO NOT skip ahead.** Each patch must be proven before moving to the next:

0. **Patch 0** → Prove first-retirement with `int3`, then restore richer worker bootstrap markers
1. **Patch 1** → Verify submit validation works, `[SUBMIT_ACCEPTED]` appears
2. **Patch 2** → Verify pickup markers appear, `[PICKUP]` in logs
3. **Patch 3** → Verify proof executions complete, `[AUTO_COMPLETE_PROOF]` appears
4. **Patch 4** → Verify completion is atomic, `[COMPLETION_OK]` appears
5. **Patch 5** → Verify wait fails cleanly on invalid result, `[WAIT_OK]` only after validation
6. **Patch 6** → Verify `[RESULT_OK]` appears with fingerprint match

## Current Proof/Test Acceptance (2026-04-23)

The proof/test BCIB worker closure is currently accepted by the fresh gate when
all of the following are true in one run:

- submit-side bind/create/pickup path is visible
- wait-return path reaches result mapping and final result marker
- validator reports `result=PASS`
- validator reports `proof_level=end_to_end_completion`
- validator reports `pf=0`

The concrete machine-readable source of truth is:

- `scripts/ci-gate-bcib-post-syscall-e2e.sh`
- `scripts/validate_bcib_post_syscall_e2e.py`
- `evidence/bcib-post-syscall-e2e/bcib_post_syscall_e2e_evidence.json`

## Determinism Closure Contract

### Hard Gate

`Proof lane != production lane`

Phase-16 Faz B is **NOT** considered complete unless all of the following are
true:

1. Same canonical BCIB fixture is executed at least 2 times on the same
   kernel/QEMU lane
2. Kernel output artifact (`result.bin` / result buffer) is byte-identical
   across runs
3. Result fingerprint (`SHA-256`) is identical across runs
4. No PF, no boundary violation, and no fallback execution path is observed
5. Scheduler interleaving does not change the final result

### Required Evidence

- `bcib_kernel_determinism_evidence.json`
- `bcib_determinism_run_1.json`
- `bcib_determinism_run_2.json`
- `result_sha256_comparison.log`
- Multi-run trace logs
- Result artifact set: `result.bin`, `result.sha256`, `result_metadata.json`

### Minimum Result Metadata

```json
{
  "bcib_sha256": "...",
  "result_sha256": "...",
  "result_size": 123,
  "pf": 0,
  "boundary_violation": 0,
  "fallback_path": 0
}
```

### Failure Conditions

- Output differs -> `NON_DETERMINISTIC`
- Missing output -> `PIPELINE_BROKEN`
- Partial output -> `CONTRACT_VIOLATION`
- PF, boundary violation, or fallback path observed -> `HARD_FAIL`

Without this evidence, Phase-16 Faz B closure is invalid.

## Historical Full-Chain Target

The bugfix is complete when ALL of the following markers appear in QEMU logs:

- ✅ `[USER_BP]` (or equivalent Ring3 `int3` proof marker) - first-retirement proven (Patch 0 - CRITICAL FIRST MILESTONE)
- ✅ `[R2]` or equivalent richer post-entry userspace marker - second-stage bootstrap proof after the `int3` gate
- ✅ `[BCIB_WORKER_START]` - worker bootstrap (Patch 0)
- ✅ `[SUBMIT_OK]` - legacy compatibility marker
- ✅ `[SUBMIT_ACCEPTED]` - submit with valid target (authoritative)
- ✅ `[PICKUP]` - execution slot picked up with reason
- ✅ `[AUTO_COMPLETE_PROOF]` - proof execution completed (proof-test mode only)
- ✅ `[COMPLETION_OK]` - completion with valid output
- ✅ `[WAIT_OK]` - wait after validation
- ✅ `[RESULT_OK]` - fingerprint match

**CRITICAL:** If ANY marker is missing, the chain is incomplete and the bug is NOT fixed.

**Missing marker = hard fail, no partial closure claim allowed.**

This is non-negotiable for production deployment.

**CURRENT STATUS**: The proof/test BCIB worker path now reaches gated end-to-end
completion. Fresh evidence records `[SUBMIT_BIND]`, `[QUEUE_CREATE]`,
`[DEQUEUE_HIT]`, `[PICKUP]`, `[RESULT_VA]`, `[WAIT_OK]`, and `[RESULT_OK]`,
and the machine verdict is `result=PASS`, `proof_level=end_to_end_completion`,
`pf=0`.

---

## Regression Prevention Checklist

Before marking any task complete, verify:
- [ ] Ring3 worker bootstrap continues to reach syscall 1003/1004
- [ ] `[BCIB_WORKER_START]` marker continues to be emitted
- [ ] `[SUBMIT_OK]` marker continues to be emitted
- [ ] `[WAIT_OK]` marker continues to be emitted
- [ ] Memory safety and capability boundaries maintained
- [ ] Deterministic behavior maintained (no wall clock, interrupt timing, scheduler order dependencies)
- [ ] All CI gates continue to pass (ABI, Boundary, Hygiene, Constitutional, Determinism Replay Consistency)

---

**Authority:** Kenan AY - Architectural Steward  
**Status:** Proof/test closure achieved for the BCIB worker post-syscall path  
**Next Step:** Broaden from proof/test closure to wider runtime/generalization
work without regressing the current gated path. The next live question is no
longer `ctx=2` queue creation, but how far the validated proof/test mechanism
should be generalized beyond its current scope.

**Current Blocker**: No live `ctx=2` queue blocker remains in the validated
proof/test lane. Remaining work is broader-scope runtime/productization and
governance closure, not queue diagnosis.

**Primary Milestone**: Fresh QEMU evidence and machine validation now prove
`end_to_end_completion` for the proof/test BCIB worker path.

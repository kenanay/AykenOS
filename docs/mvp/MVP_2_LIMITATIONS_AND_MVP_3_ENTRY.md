# MVP-2: Limitations and MVP-3 Entry Criteria

**Date:** 2026-02-22  
**MVP-2 Status:** ✅ COMPLETE (Library Implementation)  
**MVP-3 Status:** ⏳ NOT STARTED (Runtime Proof Required)

---

## Executive Summary

MVP-2 successfully implements a constitutional-grade Ring3 scheduler hint library. However, **runtime execution proof is incomplete**. This document defines the limitations of MVP-2 and the entry/exit criteria for MVP-3.

**Key Limitation:** No ACCEPT marker from real Ring3 process execution.

**MVP-3 Goal:** Prove policy/mechanism separation at runtime level.

---

## MVP-2 Limitations

### 1. No Runtime Execution Proof

**Issue:** Ring3 library is implemented but not executed in real Ring3 process.

**Evidence:**
```
[SEL]PID=1 ST=0 RIP=@02000018 F780 FULL=8000F780
[SEL]PID=2 ST=0 RIP=@020010F8 0000 FULL=00400000
```

**Analysis:**
- Ring3 process created (PID 2) ✅
- Scheduler selected PID 2 ✅
- But no context switch marker ❌
- No timer tick validation marker ❌
- No ACCEPT marker ❌

**Impact:**
- Library-level proof complete ✅
- Runtime-level proof incomplete ❌

**Root Cause (Hypothesis):**
- Timer interrupt not enabled, or
- Scheduler switch path incomplete, or
- Validation hook not called

**Resolution:** MVP-3 debug session required.

---

### 2. Simulation Test Only

**Current Validation:**
- Ring3 simulation test in kernel (validation profile)
- Simulates `ayken_sched_hint()` behavior
- Validates Ring0 double-read atomicity

**Limitation:**
- Not real Ring3 execution
- No privilege separation proof
- Pragmatic for library validation, insufficient for runtime proof

**Example (Simulation Test):**
```c
// kernel/sched/sched_mailbox.c
void sched_mailbox_test_ring3_simulation(proc_t *proc) {
    // Simulate Ring3 ayken_sched_hint(42)
    mb->candidate_pid = 42;
    mb->epoch = next_epoch;
    
    // Trigger validation
    sched_mailbox_validate_ring3(proc);
}
```

**Result:**
```
[[AYKEN_SCHED_MB_ACCEPT]] pid=1 epoch=1
[[AYKEN_SCHED_MB_REJECT]] reason=4 epoch=1 pid=1
[[AYKEN_SCHED_MB_REJECT]] reason=5 epoch=2 pid=2147483647
```

**Conclusion:** Simulation works, but real Ring3 execution does not.

---

### 3. Timer Tick / Scheduler Switch

**Observed Behavior:**
- Scheduler selected PID 2
- But no context switch occurred
- No timer tick validation triggered

**Log Evidence:**
```
AAA[K][ABOUT_TO_SCHED]
BBBS12[Q]1
[SEL]PID=1 ST=0 RIP=@02000018 F780 FULL=8000F780
[SEL]PID=2 ST=0 RIP=@020010F8 0000 FULL=00400000
```

**Missing:**
- Context switch marker (`[SW]`)
- Timer tick marker
- IRET marker
- ACCEPT marker

**Hypothesis:**
1. **Timer Interrupt Not Enabled**
   - Timer init completed: `[TMR][UNMSK][IRQ0_ON][TMR_OK]`
   - But no tick markers in log
   - Possible: Interrupt flag not set, or PIC mask issue

2. **Scheduler Switch Incomplete**
   - Scheduler selected PID 2
   - But `switch_to_first()` or `sched_yield()` not called
   - Possible: Init process blocked before scheduler started

3. **Validation Hook Not Called**
   - Timer tick may be firing
   - But validation hook not invoked
   - Possible: `current_proc` not set correctly

**Resolution:** Requires systematic debug (MVP-3 scope).

---

### 4. No CI Gate for Real Ring3 Execution

**Current CI Gate:** `ci-gate-sched-bridge-runtime`

**Current Behavior:**
- Runs kernel in validation profile
- Checks for ACCEPT/REJECT markers
- Validates marker format

**Limitation:**
- Only validates simulation test markers
- Does not validate real Ring3 execution markers

**Required for MVP-3:**
- Extend gate to run real Ring3 process
- Validate ACCEPT marker from real Ring3 execution
- Fail if no ACCEPT marker found

---

## MVP-3 Entry Criteria

### Minimum Requirements

1. **Real Ring3 Process Execution**
   - Ring3 process created ✅ (already working)
   - Scheduler selects Ring3 process ✅ (already working)
   - Context switch to Ring3 process ❌ (needs debug)
   - Ring3 code executes ❌ (needs debug)

2. **Timer Tick Validation**
   - Timer interrupt fires ❌ (needs debug)
   - Validation hook called ❌ (needs debug)
   - Mailbox validated ❌ (needs debug)

3. **ACCEPT Marker Emitted**
   - Ring3 writes mailbox ❌ (needs Ring3 execution)
   - Ring0 validates mailbox ❌ (needs timer tick)
   - ACCEPT marker emitted ❌ (needs validation)

4. **CI Gate Validation**
   - CI gate runs real Ring3 process ❌ (needs implementation)
   - CI gate validates ACCEPT marker ❌ (needs implementation)
   - CI gate PASS ❌ (needs ACCEPT marker)

### Entry Checklist

- [ ] Debug timer interrupt (why no tick markers?)
- [ ] Debug scheduler switch (why no context switch?)
- [ ] Debug validation hook (why not called?)
- [ ] Implement real Ring3 test process
- [ ] Extend CI gate for real Ring3 execution
- [ ] Validate ACCEPT marker in CI gate

---

## MVP-3 Exit Criteria

### Success Criteria

1. **Real Ring3 Execution**
   - ✅ Ring3 process created
   - ✅ Scheduler selects Ring3 process
   - ✅ Context switch to Ring3 process
   - ✅ Ring3 code executes (calls `ayken_sched_hint()`)

2. **Timer Tick Validation**
   - ✅ Timer interrupt fires
   - ✅ Validation hook called
   - ✅ Mailbox validated (double-read, epoch, PID)

3. **ACCEPT Marker**
   - ✅ Ring3 writes mailbox (epoch advances, PID valid)
   - ✅ Ring0 validates mailbox (all checks pass)
   - ✅ ACCEPT marker emitted: `[[AYKEN_SCHED_MB_ACCEPT]] pid=<pid> epoch=<epoch>`

4. **CI Gate**
   - ✅ CI gate runs real Ring3 process
   - ✅ CI gate validates ACCEPT marker
   - ✅ CI gate PASS (deterministic)

5. **Evidence**
   - ✅ Evidence stored in `evidence/` directory
   - ✅ ACCEPT marker in log
   - ✅ All CI gates PASS (including sched-bridge-runtime)

### Exit Checklist

- [ ] ACCEPT marker emitted from real Ring3 process
- [ ] CI gate `ci-gate-sched-bridge-runtime` PASS
- [ ] Evidence stored: `evidence/run-<RUN_ID>/gates/sched-bridge-runtime/`
- [ ] All 4 pre-CI discipline gates PASS
- [ ] Documentation updated (MVP-3 final status)

---

## Debug Strategy for MVP-3

### Phase 1: Timer Tick Debug

**Goal:** Understand why timer tick markers are missing.

**Steps:**
1. Verify timer interrupt enabled (PIC mask, IF flag)
2. Add timer tick marker in `timer_isr_c()`
3. Check if timer interrupt fires
4. Verify interrupt handler called

**Expected Output:**
```
[TIMER_TICK] count=1
[TIMER_TICK] count=2
[TIMER_TICK] count=3
```

**Success Criteria:** Timer tick markers appear in log.

---

### Phase 2: Scheduler Switch Debug

**Goal:** Understand why context switch doesn't occur.

**Steps:**
1. Verify `sched_start()` calls `switch_to_first()`
2. Add marker before/after `switch_to_first()`
3. Verify `current_proc` set correctly
4. Check if IRET executed

**Expected Output:**
```
[SCHED_START] before switch_to_first
[SWITCH_TO_FIRST] entry
[IRET] to Ring3
```

**Success Criteria:** Context switch markers appear, Ring3 code executes.

---

### Phase 3: Validation Hook Debug

**Goal:** Understand why validation hook not called.

**Steps:**
1. Verify timer tick calls validation hook
2. Add marker in `sched_mailbox_validate_ring3()`
3. Verify `current_proc` has mailbox
4. Check validation logic

**Expected Output:**
```
[VALIDATE_RING3] proc=<pid> mailbox_pa=<pa>
[VALIDATE_RING3] epoch=<epoch> pid=<pid>
[[AYKEN_SCHED_MB_ACCEPT]] pid=<pid> epoch=<epoch>
```

**Success Criteria:** Validation markers appear, ACCEPT marker emitted.

---

### Phase 4: CI Gate Extension

**Goal:** Extend CI gate to validate real Ring3 execution.

**Steps:**
1. Modify `scripts/ci/gate_sched_bridge_runtime.sh`
2. Run kernel with real Ring3 test process
3. Parse log for ACCEPT marker from real Ring3 execution
4. Fail if no ACCEPT marker found

**Expected Output:**
```
== CI GATE SCHED BRIDGE RUNTIME ==
run_id: <RUN_ID>
real_ring3_execution: PASS
accept_marker_found: YES
summary: evidence/run-<RUN_ID>/reports/summary.json
OK: evidence at evidence/run-<RUN_ID>
```

**Success Criteria:** CI gate PASS with real Ring3 ACCEPT marker.

---

## Architectural Proof Levels

### Level 1: Library Proof (MVP-2) ✅

**Proves:**
- Ring3 library exists
- API is callable
- No syscalls required
- No Ring0 exports added
- ABI stable

**Does NOT Prove:**
- Runtime execution
- Privilege separation
- Real Ring3 → Ring0 interaction

**Status:** COMPLETE

---

### Level 2: Runtime Proof (MVP-3) ⏳

**Proves:**
- Real Ring3 process execution
- Ring3 → Mailbox → Ring0 validation
- Timer tick → ACCEPT marker
- Privilege separation at runtime

**Requires:**
- ACCEPT marker from real Ring3 process
- CI gate validation
- Evidence stored

**Status:** NOT STARTED

---

### Level 3: Production Proof (Post-MVP-3) 🔮

**Proves:**
- Multi-process scheduling
- Concurrent mailbox access
- Stress testing
- Performance validation

**Requires:**
- Multiple Ring3 processes
- High-frequency hints
- Load testing
- Performance baselines

**Status:** FUTURE WORK

---

## Conclusion

MVP-2 is **library-complete** but **runtime-incomplete**. The Ring3 scheduler hint library is production-ready at the library level, but runtime execution proof is missing.

**MVP-3 Entry:** Debug timer tick, scheduler switch, and validation hook.

**MVP-3 Exit:** ACCEPT marker from real Ring3 process, CI gate PASS.

**Honest Assessment:**
- Library implementation: Production-grade ✅
- API design: Constitutional-grade ✅
- Runtime proof: Incomplete ❌

**Next Steps:**
1. Debug timer tick (Phase 1)
2. Debug scheduler switch (Phase 2)
3. Debug validation hook (Phase 3)
4. Extend CI gate (Phase 4)
5. Validate ACCEPT marker (Exit Criteria)

---

**Düzenleyen:** Kenan AY  
**Date:** 2026-02-22  
**Status:** MVP-2 Complete, MVP-3 Entry Criteria Defined

**This document defines the boundary between MVP-2 (library) and MVP-3 (runtime). No false claims, no shortcuts, no compromises.**

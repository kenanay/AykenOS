# Fail-Closed Proof Validation: Production Hardening Checklist

## Status

Task 11.2 implementation is **SUBSTANTIALLY COMPLETE** but **NOT PRODUCTION-READY**.

The gate foundation is solid, but production closure requires hardening against edge cases, multi-run scenarios, and real kernel trace validation.

## Critical Hardening Items

### 1. Multi-Run / Multi-Sequence Correlation

**Current Issue**: Validator uses first marker match. If trace contains multiple proof attempts or interleaved runs, correlation can be incorrect.

**Required Fix**:
- Parse trace into "proof run" blocks (bounded by run markers or clear boundaries)
- Correlate BEFORE → ENTER → KILL within each block
- Validate target block specifically (not just "first match")
- Reject traces with ambiguous block boundaries

**Acceptance Criteria**:
- Trace with 3 sequential runs: validator identifies correct run
- Trace with interleaved logs: validator rejects or isolates target run
- Trace with incomplete run + complete run: validator finds complete run

**Priority**: HIGH (prevents false positives)

---

### 2. Real Determinism Validation

**Current Issue**: Single trace validates "bounded" but not "deterministic". One run cannot prove determinism.

**Required Fix**:
- Run same scenario N times (N ≥ 3)
- Measure ENTER→KILL window variance
- Require variance within acceptable bounds (e.g., ±2 lines)
- Validate error code consistency across runs
- Validate marker sequence consistency across runs

**Acceptance Criteria**:
- 5 runs of same scenario: window size variance < 2 lines
- 5 runs: same error code every time
- 5 runs: same marker sequence every time

**Priority**: HIGH (determinism is a core requirement)

---

### 3. Positive Scheduler Removal Marker

**Current Issue**: Validator uses negative check ("no logs after kill"). This is valuable but not sufficient. Silence could mean success OR incomplete trace.

**Required Fix**:
- Define explicit scheduler removal marker (e.g., `[[AYKEN_SCHED_SLOT_INVALIDATED]]`)
- Kernel must emit this marker when process is removed from scheduler
- Validator must search for this marker positively
- If marker missing, validation fails (even if negative checks pass)

**Acceptance Criteria**:
- Kernel emits `SCHED_SLOT_INVALIDATED` after `BOUNDARY_KILL`
- Validator searches for this marker
- Trace without marker: FAIL (even if no continuation logs)

**Priority**: MEDIUM (strengthens proof, but negative check is still valuable)

---

### 4. Run/Context ID Tracking

**Current Issue**: Validator uses process_id only. PIDs can be reused. In complex traces, PID alone may not uniquely identify execution context.

**Required Fix**:
- Extend marker format to include `context_id` or `execution_id`
- Validator correlates markers using both PID and context_id
- Prevents false correlation when PID is reused

**Acceptance Criteria**:
- Markers include `pid=N context_id=X`
- Validator extracts and validates both
- Trace with PID reuse: validator correctly distinguishes contexts

**Priority**: MEDIUM (defense against PID reuse edge case)

---

### 5. Stricter Marker Format Validation

**Current Issue**: Regex is permissive. Partial markers or malformed markers may pass.

**Required Fix**:
- Define canonical marker schema:
  ```
  MARKER_NAME pid=N context_id=X error=CODE timestamp=T
  ```
- Validator enforces mandatory fields
- Missing field → marker invalid → validation fails
- No "best effort" parsing

**Acceptance Criteria**:
- Marker missing `pid`: FAIL
- Marker missing `context_id`: FAIL
- Marker with extra fields: PASS (forward compatibility)

**Priority**: MEDIUM (prevents ambiguous markers)

---

### 6. Single Source of Truth Enforcement

**Current Issue**: Bash gate and Python validator had duplicated logic (now fixed in this refactoring).

**Status**: ✅ COMPLETE (this refactoring)

**Validation**:
- Bash gate is orchestration only (~100 lines)
- Python validator is authoritative (~400 lines)
- No logic duplication

---

### 7. Failure Code Taxonomy

**Current Issue**: Violations were strings. CI needs structured codes.

**Status**: ✅ COMPLETE (this refactoring)

**Validation**:
- Failure codes defined in `FailureCode` class
- JSON includes `failure_code` field
- CI can parse and route based on code

---

### 8. QEMU Harness as Proof Producer

**Current Issue**: Harness is trace collector, not proof producer. It doesn't enforce proof structure.

**Required Fix**:
- Harness validates BCIB role is active
- Harness confirms forbidden syscall is attempted
- Harness emits run markers (start/end of proof attempt)
- Harness normalizes timeout behavior
- Harness validates trace completeness before exit

**Acceptance Criteria**:
- Harness fails if BCIB role not loaded
- Harness fails if forbidden syscall not attempted
- Harness emits `[[PROOF_RUN_START]]` and `[[PROOF_RUN_END]]`
- Validator uses these markers for block isolation

**Priority**: MEDIUM (strengthens input quality)

---

### 9. Golden + Adversarial Trace Suite

**Current Issue**: Only 2 example traces (valid, invalid). Not enough to stress-test gate.

**Required Fix**:
Create comprehensive trace suite:
- `golden_single_run.log`: Perfect valid trace
- `adversarial_wrong_pid.log`: BEFORE/ENTER/KILL from different PIDs
- `adversarial_double_kill.log`: Two BOUNDARY_KILL markers
- `adversarial_kill_then_exit.log`: KILL followed by SYSCALL_EXIT
- `adversarial_kill_then_same_pid_log.log`: KILL followed by same-PID log
- `adversarial_missing_enter.log`: BEFORE and KILL but no ENTER
- `adversarial_delayed_kill.log`: ENTER→KILL window > 10 lines
- `adversarial_mixed_multi_run.log`: Multiple interleaved proof attempts

**Acceptance Criteria**:
- Gate PASS on golden trace
- Gate FAIL on all adversarial traces with correct failure code

**Priority**: HIGH (validates gate robustness)

---

### 10. Real Kernel Scenario Validation

**Current Issue**: Gate tested on synthetic traces. Real closure requires real kernel scenarios.

**Required Fix**:
Run gate against real QEMU traces from:
- **Task 3**: Invalid BCIB entry rejection (no context/slot/memory allocated)
- **Task 5**: Runtime_Bridge forbidden syscall submission
- **Task 6**: BCIB sandbox violation (forbidden syscall execution)
- **Task 10**: Fail-closed lifecycle enforcement

**Acceptance Criteria**:
- Task 3 QEMU trace: gate PASS, proves no allocation after rejection
- Task 5 QEMU trace: gate PASS, proves bridge path enforcement
- Task 6 QEMU trace: gate PASS, proves sandbox kill
- Task 10 QEMU trace: gate PASS, proves fail-closed termination

**Priority**: CRITICAL (this is the actual production closure requirement)

---

## Hardening Priority Order

1. **Item 10**: Real kernel scenario validation (CRITICAL - this is the actual requirement)
2. **Item 1**: Multi-run correlation (HIGH - prevents false positives)
3. **Item 2**: Real determinism validation (HIGH - core requirement)
4. **Item 9**: Golden + adversarial trace suite (HIGH - validates robustness)
5. **Item 3**: Positive scheduler removal marker (MEDIUM - strengthens proof)
6. **Item 4**: Run/context ID tracking (MEDIUM - edge case defense)
7. **Item 5**: Stricter marker format (MEDIUM - prevents ambiguity)
8. **Item 8**: QEMU harness as proof producer (MEDIUM - input quality)

Items 6 and 7 are already complete (this refactoring).

---

## Recommended Next Steps

### Immediate (This Session)
1. ✅ Refactor bash gate to orchestration-only (DONE)
2. ✅ Standardize failure codes in Python validator (DONE)
3. ✅ Update documentation with hardening status (DONE)
4. Create this hardening checklist (DONE)

### Next Session
1. Implement Item 9: Golden + adversarial trace suite
2. Validate gate against all adversarial traces
3. Fix any issues discovered

### Following Sessions
1. Implement Item 1: Multi-run correlation
2. Implement Item 2: Real determinism validation
3. Implement Item 3: Positive scheduler removal marker
4. Run gate against real Task 3/5/6/10 QEMU traces (Item 10)

---

## Definition of "Production-Ready"

Task 11.2 can be marked **PRODUCTION-READY** when:

1. ✅ Gate foundation implemented (DONE)
2. ✅ Orchestration/validation separation (DONE)
3. ✅ Failure code taxonomy (DONE)
4. ⏳ Golden + adversarial trace suite passes (Item 9)
5. ⏳ Multi-run correlation implemented (Item 1)
6. ⏳ Real determinism validation implemented (Item 2)
7. ⏳ Gate passes on real Task 3/5/6/10 QEMU traces (Item 10)

Current status: **3/7 complete** → **GATE FOUNDATION READY, PRODUCTION HARDENING PENDING**

---

## Authority

This checklist represents the gap between "implementation landed" and "production-ready" as identified by Kenan AY (Architectural Steward) on 2026-04-11.

The gate is usable as a foundation but requires hardening before production deployment.

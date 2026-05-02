# Phase-17 Injection Implementation Checklist

**Date**: 2026-05-02  
**Branch**: `phase17-marker-validation-guard`  
**Next Phase**: Runtime Failure Injection

---

## Implementation Tasks

### Task 1: Create Injection Header
**File**: `kernel/sys/execution_marker_injection.h`

**Requirements**:
- [ ] Add top-level guard: `#if defined(AYKEN_PHASE17_MARKER_INJECTION_TEST) && (AYKEN_PHASE17_MARKER_INJECTION_TEST == 1)`
- [ ] Include `execution_slot.h`
- [ ] Declare 7 injection functions (each with individual flag guard):
  - [ ] `inject_invalid_order()`
  - [ ] `inject_duplicate()`
  - [ ] `inject_missing()`
  - [ ] `inject_overflow()`
  - [ ] `inject_stale_data()`
  - [ ] `inject_corrupt_bitmap()`
  - [ ] `inject_partial_write()`
- [ ] Close top-level guard

**Reference**: See `PHASE17_RUNTIME_FAILURE_INJECTION_PLAN.md` Section "Implementation Strategy → Phase 1"

---

### Task 2: Create Injection Implementation
**File**: `kernel/sys/execution_marker_injection.c`

**Requirements**:
- [ ] Include `execution_marker_injection.h`
- [ ] Add top-level guard (same as header)
- [ ] Implement 7 injection functions:
  - [ ] `inject_invalid_order()` — swap markers[1] and markers[2]
  - [ ] `inject_duplicate()` — set markers[2] = markers[1]
  - [ ] `inject_missing()` — shift sequence, set count = 4
  - [ ] `inject_overflow()` — set count = 8, error_code = OVERFLOW
  - [ ] `inject_stale_data()` — set markers[5] = 0xAA, markers[6] = 0xBB
  - [ ] `inject_corrupt_bitmap()` — set bitmap = 0x3F
  - [ ] `inject_partial_write()` — set count = 3
- [ ] Close top-level guard

**Reference**: See `PHASE17_RUNTIME_FAILURE_INJECTION_PLAN.md` Section "Implementation Strategy → Phase 1"

---

### Task 3: Add Injection Points
**File**: `kernel/sys/execution_slot.c`

**Location**: Inside `execution_slot_prepare_hash_locked()`, before validation call

**Requirements**:
- [ ] Add nested guard structure:
  ```c
  #if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
      #if defined(AYKEN_PHASE17_MARKER_INJECTION_TEST) && (AYKEN_PHASE17_MARKER_INJECTION_TEST == 1)
          // injection hooks
      #endif
      
      /* Pre-commit guard: validate markers */
      if (execution_slot_validate_markers_locked(slot) != 0) {
          return -1;
      }
  #endif
  ```
- [ ] Add 7 conditional injection calls (each with individual flag)
- [ ] Include `execution_marker_injection.h` at top of file (with guard)

**Reference**: See `PHASE17_RUNTIME_FAILURE_INJECTION_PLAN.md` Section "Implementation Strategy → Phase 2"

---

### Task 4: Create Test Automation Script
**File**: `tests/phase17_marker_injection_suite.sh`

**Requirements**:
- [ ] Add shebang and `set -e`
- [ ] Create evidence directory: `out/evidence/phase17-injection-tests`
- [ ] Implement 7 test cases:
  - [ ] Test 1: Invalid Order
  - [ ] Test 2: Duplicate Marker
  - [ ] Test 3: Missing Marker
  - [ ] Test 4: Overflow
  - [ ] Test 5: Stale Buffer Data
  - [ ] Test 6: Corrupted Bitmap
  - [ ] Test 7: Partial Write
- [ ] Each test must:
  - [ ] Set `AYKEN_PHASE17_MARKER_INJECTION_TEST=1`
  - [ ] Set `AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=1`
  - [ ] Set individual injection flag
  - [ ] Run `make qemu-test-headless`
  - [ ] Capture output to evidence file
  - [ ] Use `grep` to validate expected error code
  - [ ] Use `grep` to validate `EXEC_SLOT_FAILED` state
  - [ ] Exit with error if validation fails
- [ ] Make script executable: `chmod +x tests/phase17_marker_injection_suite.sh`

**Reference**: See `PHASE17_RUNTIME_FAILURE_INJECTION_PLAN.md` Section "Implementation Strategy → Phase 3"

---

### Task 5: Update Build System (if needed)
**File**: `Makefile` or build configuration

**Requirements**:
- [ ] Verify `AYKEN_PHASE17_MARKER_INJECTION_TEST` can be passed to compiler
- [ ] Verify `AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE` is already supported
- [ ] Add `qemu-test-headless` target if not exists (for automated testing)

---

### Task 6: Run Tests and Collect Evidence

**Requirements**:
- [ ] Run test suite: `./tests/phase17_marker_injection_suite.sh`
- [ ] Verify all 7 tests pass with explicit validation
- [ ] Collect evidence files in `out/evidence/phase17-injection-tests/`
- [ ] Verify each log contains:
  - [ ] Expected `MARKER_ERROR_*` code
  - [ ] `EXEC_SLOT_FAILED` state transition
  - [ ] No kernel panics or crashes

---

### Task 7: Verify Production Safety

**Requirements**:
- [ ] Build production kernel: `make clean && make kernel.elf`
- [ ] Run objdump: `objdump -t out/build/kernel.elf | grep -i inject`
- [ ] Verify output is EMPTY (no injection symbols)
- [ ] Document result in evidence

---

### Task 8: Run Pre-CI Gates

**Requirements**:
- [ ] Run: `make pre-ci`
- [ ] Verify all gates pass:
  - [ ] ABI Gate
  - [ ] Boundary Gate
  - [ ] Hygiene Gate
  - [ ] Constitutional Gate
  - [ ] Determinism Replay Consistency Gate
- [ ] Collect evidence in `out/evidence/`

---

### Task 9: Update Documentation

**Files to Update**:
- [ ] `PHASE17_STEP5_COMPLETION_REPORT.md`
  - [ ] Add "Runtime Failure Injection" section
  - [ ] Document all 7 test results
  - [ ] Add production safety verification
- [ ] `PHASE17_STEP5_VALIDATION_PROOF.md`
  - [ ] Add "Adversarial Testing" section
  - [ ] Reference injection test evidence

---

### Task 10: Prepare for Remote CI

**Requirements**:
- [ ] Commit all changes with descriptive message
- [ ] Push branch to remote: `git push origin phase17-marker-validation-guard`
- [ ] Run remote CI (mandatory before merge)
- [ ] Wait for CI results
- [ ] Address any CI failures

---

### Task 11: Submit PR

**Requirements**:
- [ ] Create PR with title: "Phase-17 Step 5: Marker Validation Guard + Injection Tests"
- [ ] PR description must include:
  - [ ] Summary of validation implementation
  - [ ] Summary of injection test results
  - [ ] Link to evidence directory
  - [ ] Production safety verification
  - [ ] Pre-ci gate results
  - [ ] Remote CI results
- [ ] Request review from architectural steward
- [ ] Wait for sign-off

---

## Success Criteria

### Code Quality
- ✅ All injection code behind test-only guard
- ✅ No production path contamination
- ✅ All 7 injection functions implemented correctly

### Test Quality
- ✅ All 7 tests pass with explicit validation
- ✅ Each test validates specific error code
- ✅ Each test validates state transition
- ✅ No "fail = pass" logic

### Safety
- ✅ Production build has ZERO injection symbols
- ✅ Test-only guard verified with objdump
- ✅ All pre-ci gates pass

### Documentation
- ✅ Evidence collected for all tests
- ✅ Completion report updated
- ✅ Validation proof updated

### CI/Review
- ✅ Remote CI passes
- ✅ Code review approved
- ✅ Architectural steward sign-off

---

## Timeline Estimate

- **Day 1**: Tasks 1-5 (implementation)
- **Day 2**: Tasks 6-8 (testing and verification)
- **Day 3**: Tasks 9-10 (documentation and CI)
- **Day 4**: Task 11 (PR submission and review)
- **Day 5**: Address review feedback, merge

**Total**: 5 days to production-ready merge

---

## Notes

- **Test-only guard is NON-NEGOTIABLE**: Any injection code without guard will be rejected
- **Explicit validation is NON-NEGOTIABLE**: Any "fail = pass" logic will be rejected
- **Remote CI is MANDATORY**: No merge without remote CI pass
- **Architectural steward sign-off is REQUIRED**: No merge without approval

---

**Prepared by**: Kiro (AI Assistant)  
**Date**: 2026-05-02  
**Status**: Ready for Implementation

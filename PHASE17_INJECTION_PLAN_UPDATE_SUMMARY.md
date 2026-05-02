# Phase-17 Injection Plan Update Summary

**Date**: 2026-05-02  
**Authority**: Kenan AY - Architectural Steward  
**Status**: Critical Safety Requirements Added

---

## Updates Applied

### 1. Critical Safety Requirements Section (NEW)
Added top-level warning section with two non-negotiable mandates:

#### Mandate #1: Test-Only Guard
- **Guard**: `AYKEN_PHASE17_MARKER_INJECTION_TEST=1`
- **Purpose**: Prevent production sızma (contamination)
- **Enforcement**: Fail-closed — no guard = no injection code compiles

#### Mandate #2: Explicit Validation
- **Requirement**: Tests must validate specific error codes and state transitions
- **Purpose**: Distinguish kernel crash from correct validation failure
- **Enforcement**: Test fails if expected `MARKER_ERROR_*` not found

---

## Implementation Strategy Updates

### Phase 1: Injection Harness
**Changes**:
- Added top-level `#if defined(AYKEN_PHASE17_MARKER_INJECTION_TEST)` guard to header
- All injection functions now wrapped in test-only guard
- Complete implementations for all 7 injection functions:
  * `inject_invalid_order()` — swap markers 1 and 2
  * `inject_duplicate()` — duplicate marker 1
  * `inject_missing()` — remove marker 2
  * `inject_overflow()` — force count = 8
  * `inject_stale_data()` — add garbage at positions 5-6
  * `inject_corrupt_bitmap()` — set extra bit (0x3F)
  * `inject_partial_write()` — truncate to count = 3

### Phase 2: Injection Points
**Changes**:
- Added nested guard structure:
  ```c
  #if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
      #if defined(AYKEN_PHASE17_MARKER_INJECTION_TEST) && (AYKEN_PHASE17_MARKER_INJECTION_TEST == 1)
          // injection hooks here
      #endif
  #endif
  ```
- Ensures injection code ONLY compiles when both flags set

### Phase 3: Test Automation
**Changes**:
- Replaced simple loop with explicit validation per test
- Each test now:
  1. Runs with `AYKEN_PHASE17_MARKER_INJECTION_TEST=1` flag
  2. Captures output to evidence file
  3. Uses `grep` to validate expected error code
  4. Uses `grep` to validate `EXEC_SLOT_FAILED` state
  5. Fails if expected patterns NOT found (even if execution fails)

**Example**:
```bash
if grep -q "MARKER_ERROR_INVALID_ORDER" "$LOG" && \
   grep -q "EXEC_SLOT_FAILED" "$LOG"; then
    echo "✅ PASS"
else
    echo "❌ FAIL: Expected error code not found"
    exit 1
fi
```

---

## Production Readiness Checklist Updates

### Added Sections:
1. **Core Requirements** — basic test execution
2. **Validation Requirements** — explicit error code checks
3. **Safety Requirements** — test-only guard verification
4. **CI/Review Requirements** — remote CI and sign-off

### New Merge Rejection Criteria:
- ❌ Any injection code without test-only guard
- ❌ Any test using "fail = pass" logic
- ❌ Missing explicit error code validation
- ❌ Remote CI not executed

---

## Evidence Collection Updates

### Added: Test-Only Guard Verification
**New Check**:
```bash
# Build production kernel
make clean && make kernel.elf

# Verify NO injection symbols
objdump -t out/build/kernel.elf | grep -i inject
# Expected: NOTHING (empty output)
```

**Purpose**: Prove injection code does NOT contaminate production builds

---

## Key Differences from Previous Version

| Aspect | Before | After |
|--------|--------|-------|
| **Test Guard** | Individual flags only | Top-level + individual flags |
| **Test Logic** | "fail = pass" | Explicit error code validation |
| **Production Safety** | Implicit | Explicit objdump verification |
| **Merge Criteria** | Basic checklist | Detailed rejection criteria |
| **Evidence** | Test logs only | Logs + production build verification |

---

## Constitutional Compliance

### NON_OVERRIDABLE Rules
- ✅ `KERNEL.SAFETY.CRITICAL` — test-only guard prevents production risk
- ✅ `SECURITY.BOUNDARY.VIOLATION` — no injection code in production path
- ✅ `CONSTITUTIONAL.ENFORCEMENT.BYPASS` — explicit validation prevents bypass

### Phase Matrix (P4.4 Dev)
- ✅ `ERROR.PANIC` — tests validate controlled failure, not panic
- ✅ `MEMORY.CONTRACT.VIOLATION` — bounds-safe injection functions
- ✅ `DETERMINISM.GLOBAL` — no global state mutations in tests

---

## Next Steps

### Immediate (Day 1-2):
1. Implement `kernel/sys/execution_marker_injection.h`
2. Implement `kernel/sys/execution_marker_injection.c`
3. Add injection points to `execution_slot_prepare_hash_locked()`
4. Create `tests/phase17_marker_injection_suite.sh`

### Validation (Day 3):
1. Run all 7 injection tests
2. Collect evidence in `out/evidence/phase17-injection-tests/`
3. Verify production build has ZERO injection symbols

### Pre-Merge (Day 4-5):
1. Run remote CI (mandatory)
2. Update `PHASE17_STEP5_COMPLETION_REPORT.md` with test results
3. Submit PR with full evidence package
4. Await architectural steward sign-off

---

## Architectural Steward Notes

**Hüküm (Ruling)**:
- Step 5 core implementation: ✅ CLOSED
- Production merge gate: ⏳ PENDING injection evidence
- Test-only guard: 🔒 NON-NEGOTIABLE
- Explicit validation: 🔒 NON-NEGOTIABLE

**Rationale**:
> "Injection hook production path'e sızmamalı. Sadece test profiliyle derlenmeli."
> "Test 'fail oldu = geçti' mantığıyla bırakılmamalı. Kernel crash ile doğru fail aynı görünür."

These are not suggestions — they are **constitutional requirements** for Phase-17 completion.

---

**Prepared by**: Kiro (AI Assistant)  
**Reviewed by**: Kenan AY - Architectural Steward  
**Date**: 2026-05-02  
**Status**: Ready for Implementation

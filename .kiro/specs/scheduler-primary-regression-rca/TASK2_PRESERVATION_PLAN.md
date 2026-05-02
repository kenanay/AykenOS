# Task 2: Preservation Property Tests - Implementation Plan

**Date**: 2026-04-19  
**Spec**: scheduler-primary-regression-rca  
**Task**: 2 - Write preservation property tests (BEFORE implementing fix)

## Overview

Task 2 creates a comprehensive test suite to ensure that any optimization to the first-syscall init path preserves all existing behavior. These tests run on UNFIXED code first to establish baseline behavior, then continue to pass after Task 3 optimization.

## Test Categories

### Category 1: Boundary Init Behavior (NEW - Based on Task 1 Findings)

**Test 1.1: boundary_init_done Idempotency**
- **Purpose**: Verify that `boundary_init_done` flag prevents repeated initialization
- **Method**: Execute multiple syscalls, verify init path taken only once
- **Success Criteria**: 
  - First syscall: `DIAG_BOUNDARY_INIT_ENTER` marker present
  - Second syscall: `DIAG_BOUNDARY_INIT_SKIPPED` marker present
  - Third+ syscalls: `DIAG_BOUNDARY_INIT_SKIPPED` marker present
- **Artifact**: `scripts/ci/test_boundary_init_idempotency.py`

**Test 1.2: Anchored Sequence Detection**
- **Purpose**: Verify test syscall (1010) detection works correctly
- **Method**: Execute anchored syscalls (S/A/B/C), verify markers
- **Success Criteria**:
  - `DIAG_TEST_ANCHOR_SET` marker appears for syscall 1010
  - `DIAG_ANCHORED_SEQ_*` markers track sequence correctly
  - Non-anchored syscalls do NOT trigger anchor markers
- **Artifact**: `scripts/ci/test_anchored_sequence.py`

**Test 1.3: Skip Path Performance Guarantee**
- **Purpose**: Verify second syscall is significantly faster than first
- **Method**: Measure kernel cost for first vs second syscall
- **Success Criteria**:
  - Second syscall kernel cost < first syscall kernel cost * 0.5 (at least 50% faster)
  - Skip path marker present on second syscall
- **Artifact**: `scripts/ci/test_skip_path_performance.py`

**Test 1.4: Normalized Syscall Numbering**
- **Purpose**: Prevent regression of syscall normalization bug
- **Method**: Execute syscall 1010, verify it's checked as normalized (10)
- **Success Criteria**:
  - Anchored syscall detection uses normalized number (syscall_num - SYS_V2_BASE)
  - Test anchor set correctly for syscall 1010
- **Artifact**: Code review + regression test in `test_anchored_sequence.py`

**Test 1.5: Fetch-Probe Diagnostic Isolation**
- **Purpose**: Verify `AYKEN_RING3_FETCH_PROBE` is disabled in production builds
- **Method**: Check build flags, verify probe code is not compiled
- **Success Criteria**:
  - Production builds: `AYKEN_RING3_FETCH_PROBE=0` or undefined
  - Diagnostic builds: `AYKEN_RING3_FETCH_PROBE=1` allowed
  - CI gate enforces production flag value
- **Artifact**: `scripts/ci/verify_diagnostic_flags.sh`

### Category 2: Existing Preservation Tests (From Original Task 2)

**Test 2.1: Observability Infrastructure Zero Overhead**
- **Purpose**: Verify BCIB execution engine, boundary enforcement, probes, and markers have zero overhead
- **Method**: Measure overhead of observability infrastructure across many operations
- **Success Criteria**: Overhead ≤ 1% of operation cost
- **Artifact**: Existing test infrastructure (verified in phase16)

**Test 2.2: Stale Epoch Short-Circuit**
- **Purpose**: Verify stale epoch detection short-circuits correctly
- **Method**: Test stale epoch cases, verify early exit
- **Success Criteria**: Stale epoch path exits before expensive operations
- **Artifact**: Existing test infrastructure (verified in phase16)

**Test 2.3: Snapshot Operations Optimized**
- **Purpose**: Verify snapshot operations use single-snapshot path
- **Method**: Test snapshot operations, verify no duplication
- **Success Criteria**: Single snapshot per operation (commit 31a33246)
- **Artifact**: Existing test infrastructure (verified in phase16)

**Test 2.4: Non-Syscall Kernel Subsystems**
- **Purpose**: Verify non-syscall kernel operations maintain performance
- **Method**: Test memory management, device drivers, scheduler (non-syscall)
- **Success Criteria**: Performance within baseline thresholds
- **Artifact**: Existing test infrastructure

**Test 2.5: Syscall Functional Correctness**
- **Purpose**: Verify syscalls produce correct results
- **Method**: Execute various syscalls, verify return values and side effects
- **Success Criteria**: All syscalls behave correctly
- **Artifact**: Existing syscall test suite

**Test 2.6: Context Switch Functional Correctness**
- **Purpose**: Verify context switches work correctly
- **Method**: Test context switch behavior, verify state preservation
- **Success Criteria**: Context switches preserve all state correctly
- **Artifact**: Existing context switch test suite

## Implementation Strategy

### Phase 1: New Tests (Category 1)

1. **Create test harness infrastructure**
   - Extend `scripts/qemu-second-syscall-proof-harness.sh` for general use
   - Create reusable analysis functions in `scripts/ci/analyze_syscall_regression.py`

2. **Implement Test 1.1: boundary_init_done Idempotency**
   - Reuse existing `minimal_second_syscall_proof.S` payload
   - Create analyzer script to verify init/skip markers
   - Run in CI, verify PASS on unfixed code

3. **Implement Test 1.2: Anchored Sequence Detection**
   - Reuse existing anchored sequence infrastructure
   - Verify marker emission for test syscalls
   - Run in CI, verify PASS on unfixed code

4. **Implement Test 1.3: Skip Path Performance Guarantee**
   - Measure kernel cost deltas from existing markers
   - Calculate performance improvement (should be ~65%)
   - Run in CI, verify PASS on unfixed code

5. **Implement Test 1.4: Normalized Syscall Numbering**
   - Code review: verify normalization happens before anchor check
   - Add regression test to anchored sequence test
   - Run in CI, verify PASS on unfixed code

6. **Implement Test 1.5: Fetch-Probe Diagnostic Isolation**
   - Create build flag verification script
   - Check CI build configuration
   - Add CI gate to enforce production flag values

### Phase 2: Verify Existing Tests (Category 2)

1. **Audit existing test coverage**
   - Review phase16 test artifacts
   - Confirm tests 2.1-2.6 already exist and pass
   - Document test locations and CI integration

2. **Run existing tests on unfixed code**
   - Execute full test suite in authoritative CI
   - Verify all preservation tests PASS
   - Document baseline behavior

### Phase 3: CI Integration

1. **Create CI gate: `ci-gate-preservation-tests`**
   - Run all Category 1 tests
   - Run all Category 2 tests
   - Fail if any test fails
   - Generate preservation report

2. **Add to Makefile**
   - Target: `ci-gate-preservation-tests`
   - Dependencies: build artifacts, test infrastructure
   - Output: `out/evidence/preservation-tests/report.txt`

3. **Document expected behavior**
   - Create `TASK2_PRESERVATION_BASELINE.md`
   - Document all test results on unfixed code
   - Establish preservation contract for Task 3

## Success Criteria

Task 2 is COMPLETE when:

1. ✅ All Category 1 tests (1.1-1.5) implemented and passing on unfixed code
2. ✅ All Category 2 tests (2.1-2.6) verified and passing on unfixed code
3. ✅ CI gate `ci-gate-preservation-tests` integrated and passing
4. ✅ Preservation baseline documented in `TASK2_PRESERVATION_BASELINE.md`
5. ✅ All tests run in authoritative GitHub CI (Linux x86_64)

## Architectural Compliance

- ✅ Tests run on UNFIXED code first (observation-first methodology)
- ✅ Tests use existing marker infrastructure (minimal new instrumentation)
- ✅ Tests are deterministic and reproducible
- ✅ Tests verify behavior, not implementation details
- ✅ Tests establish preservation contract for optimization phase

## Artifacts

- `scripts/ci/test_boundary_init_idempotency.py` - Test 1.1
- `scripts/ci/test_anchored_sequence.py` - Test 1.2
- `scripts/ci/test_skip_path_performance.py` - Test 1.3
- `scripts/ci/verify_diagnostic_flags.sh` - Test 1.5
- `TASK2_PRESERVATION_BASELINE.md` - Baseline behavior documentation
- `Makefile` target: `ci-gate-preservation-tests`

## Timeline

- Phase 1 (New Tests): ~2-3 hours
- Phase 2 (Verify Existing): ~1 hour
- Phase 3 (CI Integration): ~1 hour
- Total: ~4-5 hours

## Next Steps After Task 2

Once Task 2 is COMPLETE and all preservation tests PASS on unfixed code:

1. Proceed to Task 3: Optimize init path
2. Run preservation tests after optimization
3. Verify all tests still PASS (preservation guaranteed)
4. Proceed to Task 3.2-3.4: Verify fix and remove probes

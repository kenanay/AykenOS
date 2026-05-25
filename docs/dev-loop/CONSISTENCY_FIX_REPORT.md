# Dev Loop Spec Consistency Fix Report

**Date**: 2026-05-03  
**Author**: Kenan AY  
**Status**: COMPLETED

---

## Executive Summary

The dev-loop-boot-monitoring specification contained critical inconsistencies that would have caused confusion during implementation. All identified issues have been systematically corrected across requirements, design, tasks, and governance documents.

---

## Critical Issues Fixed

### 1. Task Numbering Conflict ✅ FIXED

**Problem**: Task 26 was used twice:
- First for "Dev loop non-interference enforcement"
- Then again for "Evidence integrity hardening"

**Impact**: Task tracking would be impossible, CI governance references would be ambiguous.

**Solution**:
- Task 26-29: Dev loop non-interference enforcement (governance)
- Task 30-32: Evidence integrity hardening (observability)
- Updated all cross-references in Notes section

---

### 2. Kernel Code Modification Contradiction ✅ FIXED

**Problem**: Design stated "no kernel code modifications" but Task 5 explicitly adds kernel markers.

**Impact**: Confusion about isolation boundary, potential constitutional violation concerns.

**Solution**:
- Clarified: "No kernel code modifications **beyond validation-only marker emission**"
- Added explicit note in Task 5.2: "This is the ONLY kernel code modification"
- Updated design isolation section with precise language:
  - "No runtime behavior change beyond conditional marker emission to serial output"
  - "Markers are pure output with zero side effects"

---

### 3. Naming Convention Self-Violation ✅ FIXED

**Problem**: Requirements 25 and 30 forbid "aykenos" and mandate "ayken", but documents themselves used "AykenOS" throughout.

**Impact**: Spec violates its own rules, CI naming checks would fail on spec files.

**Solution**:
- **Established naming policy**:
  - `ayken` = canonical for code artifacts, file names, CI components
  - `AykenOS` = permitted ONLY in project-level documentation (README, manifests, architectural docs)
  - `aykenos` (lowercase) = forbidden in all new code
- Updated Requirement 25 with clear policy statement
- Updated Requirement 30 with naming scope clarification
- Updated DEV_LOOP_CONSTITUTION.md Section 10 with exception
- Updated GOVERNANCE.md with naming rules
- Updated Task 28.1 with exception handling logic
- Changed "AykenOS kernel development" → "Ayken kernel development" in design.md

---

### 4. Marker Validation Severity Inconsistency ✅ FIXED

**Problem**: 
- Requirements 1 and 17 imply all three markers are mandatory
- Design said `[K][EARLY_BOOT_OK]` and `[K][LATE_INIT_END]` are "WARNING if missing"

**Impact**: Unclear whether missing markers should FAIL or WARN, leading to weak validation.

**Solution**:
- **All three markers are now REQUIRED in validation profile**:
  - `[[AYKEN_BOOT_OK]]` = CRITICAL (always required)
  - `[K][EARLY_BOOT_OK]` = REQUIRED (FAIL if missing)
  - `[K][LATE_INIT_END]` = REQUIRED (FAIL if missing)
- Updated design.md validation logic to FAIL on missing markers
- Updated design.md architecture diagram
- Updated design.md error handling section
- Updated Task 1.1 with explicit FAIL requirement

---

### 5. Evidence Generation Exit Status Risk ✅ FIXED

**Problem**: Task 21.4 said "evidence generation call at end of dev_loop" without specifying that it must not affect validation exit status.

**Impact**: Evidence generation failure could override validation PASS, violating non-interference.

**Solution**:
- Added **CRITICAL** constraint to Task 21.4:
  ```bash
  validation_status=$?
  generate_evidence || true
  exit "$validation_status"
  ```
- Evidence generation MUST NOT affect validation exit status
- Added Requirements 26.1, 26.2 cross-reference

---

### 6. Requirement 20 Naming Mismatch ✅ FIXED

**Problem**: Requirement 20 titled "Evidence Directory Management" but content was about `out/logs/` (not evidence).

**Impact**: Confusing terminology, evidence vs logs boundary unclear.

**Solution**:
- Renamed Requirement 20 → "Log Directory Management"
- Evidence directory management is covered by Requirements 26-27 (evidence layer)
- Updated Task 1.4 description to match

---

### 7. Task 29 Outdated Requirement Range ✅ FIXED

**Problem**: Task 29 checkpoint said "verify all requirements (1-25)" but requirements now go to 30.

**Impact**: Incomplete validation, requirements 26-30 would be skipped.

**Solution**:
- Updated Task 29 → "verify all requirements (1-30)"

---

## Document-by-Document Changes

### requirements.md
- ✅ Requirement 20: Renamed to "Log Directory Management"
- ✅ Requirement 25: Added naming policy with AykenOS exception
- ✅ Requirement 30: Clarified naming enforcement scope with exception

### design.md
- ✅ Overview: Changed "AykenOS kernel development" → "Ayken kernel development"
- ✅ Architecture diagram: Updated marker validation severity (WARNING → REQUIRED)
- ✅ Marker validation logic: Changed to FAIL on missing markers
- ✅ Error handling: Updated "Missing Warning Marker" → FAIL behavior
- ✅ Isolation guarantee: Clarified "no kernel code changes beyond validation-only marker emission"

### tasks.md
- ✅ Task 1.1: Added explicit FAIL requirement for all markers
- ✅ Task 1.4: Renamed "evidence directory" → "log directory"
- ✅ Task 5: Added note about ONLY kernel code modification
- ✅ Task 21.4: Added CRITICAL constraint for exit status preservation
- ✅ Task 26-29: Renumbered to avoid conflict (now governance tasks)
- ✅ Task 28.1: Added AykenOS exception for documentation
- ✅ Task 28.2: Added exception documentation requirement
- ✅ Task 29: Updated requirement range (1-25 → 1-30)
- ✅ Task 30-32: New numbering for evidence integrity hardening
- ✅ Notes: Updated task references and naming policy

### GOVERNANCE.md
- ✅ Overview: Kept "Ayken dev loop" (correct usage)
- ✅ Naming rules: Added AykenOS exception for documentation

### DEV_LOOP_CONSTITUTION.md
- ✅ Section 10: Added naming policy with AykenOS exception

### CI_INTEGRATION.md
- ✅ Architecture diagram: No changes needed (already correct)

---

## Validation

All changes have been applied and cross-checked:

- ✅ No task number conflicts
- ✅ Kernel code modification policy is clear and consistent
- ✅ Naming convention has explicit exception for documentation
- ✅ Marker validation severity is consistent (all REQUIRED)
- ✅ Evidence generation cannot affect exit status
- ✅ Requirement 20 name matches content
- ✅ Task 29 references correct requirement range (1-30)
- ✅ All cross-references updated

---

## Implementation Readiness

The specification is now **READY FOR IMPLEMENTATION** with:

1. **Clear task numbering**: 1-32, no conflicts
2. **Consistent isolation boundary**: Validation-only markers, no runtime behavior change
3. **Explicit naming policy**: ayken (code), AykenOS (docs only), aykenos (forbidden)
4. **Strict marker validation**: All three markers REQUIRED in validation profile
5. **Protected exit status**: Evidence generation cannot override validation result
6. **Accurate terminology**: Log vs evidence directories clearly distinguished
7. **Complete requirement coverage**: Tasks reference requirements 1-30

---

## Recommendations for Implementation

1. **Start with Task 1-6**: Core dev loop and marker emission
2. **Verify isolation early**: Run Task 3 (isolation property test) as soon as markers are in place
3. **Enforce naming from day 1**: Implement Task 28 (naming compliance) before writing new scripts
4. **Protect exit status**: When implementing Task 21.4, use the exact pattern specified
5. **Test marker validation**: Ensure all three markers are REQUIRED, not optional

---

## Constitutional Compliance

All fixes maintain constitutional compliance:

- ✅ **Non-interference**: Evidence generation cannot affect validation
- ✅ **Observation boundary**: Logs are authority, evidence is derived
- ✅ **Naming law**: Clear policy with documented exception
- ✅ **Isolation guarantee**: Kernel changes limited to validation-only markers

---

**End of Report**

---

**Signature**:  
Kenan AY — System Architect  
Date: 2026-05-03

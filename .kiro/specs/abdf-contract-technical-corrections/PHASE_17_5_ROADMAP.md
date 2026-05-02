# Phase-17.5: Spec Validation Infrastructure

**Status**: 🟡 IN PROGRESS  
**Authority**: Constitutional Enforcement  
**Priority**: MANDATORY (blocks future spec merges)

---

## Mission

Build complete validation infrastructure to prevent validation failures like the one encountered in ABDF Contract Technical Corrections spec.

---

## Problem Statement

**Current State** (ABDF spec):
- ❌ Task 1 not executed (no ORIGINAL baseline)
- ❌ PRESERVATION_BASELINE.md corrupt (contains FIXED, not UNFIXED)
- ❌ No transformation proof (ORIGINAL → FIXED)
- ❌ Manual verification overrode script FAIL

**Root Cause**:
- No enforcement of ORIGINAL baseline capture
- No automated preservation validation
- No CI gate for spec validation
- Process violations not blocked

**Impact**:
- Cannot prove bugs existed in ORIGINAL
- Cannot prove only 7 changes made
- Cannot verify transformation correctness
- Validation gaps hidden until self-review

---

## Requirements

### R1: ORIGINAL Baseline Enforcement
- [ ] ORIGINAL baseline MUST be captured before any fixes
- [ ] ORIGINAL hash MUST be recorded
- [ ] Bug validation MUST FAIL on ORIGINAL (proves bugs exist)
- [ ] Task 1 execution MUST be verified before Task 2

### R2: Preservation Validation
- [x] `validate_preservation.sh` - Diff-based validation
- [x] Whitelist-based change verification
- [x] Unexpected change detection
- [x] CI-compatible output (exit codes, reports)

### R3: CI Gate Integration
- [ ] Add spec validation gate to CI pipeline
- [ ] Block merge if validation fails
- [ ] Generate evidence artifacts
- [ ] Integrate with existing pre-ci workflow

### R4: Level 3 Validation Enforcement
- [ ] Make Level 3 validation mandatory for all specs
- [ ] Enforce ORIGINAL → FIXED transformation proof
- [ ] Enforce preservation proof (only expected changes)
- [ ] Enforce complete audit trail

### R5: Process Compliance
- [ ] Script FAIL cannot be overridden by manual verification
- [ ] Validation gaps must block merge
- [ ] "Bug kanıtlanmadan fix yapılmaz" enforced by tooling
- [ ] Self-review checklist integrated with validation

---

## Deliverables

### ✅ Phase 1 Complete (ALPHA)

1. **validate_preservation.sh** (ALPHA)
   - Initial implementation complete
   - NOT CI-authoritative yet
   - Requires hardening (see Phase 1.5)
   - Location: `.kiro/specs/abdf-contract-technical-corrections/validate_preservation.sh`

2. **expected_changes.yml** (Enhanced)
   - Preservation rules defined
   - Expected changes specified
   - Validation rules documented
   - CI integration configured
   - Location: `.kiro/specs/abdf-contract-technical-corrections/expected_changes.yml`

3. **Validation Infrastructure Template**
   - Bug condition validation
   - Preservation validation (ALPHA)
   - Evidence generation
   - Report templates
   - Location: `.kiro/specs/abdf-contract-technical-corrections/`

### ⏳ Phase 1.5: Hardening (NEW - CRITICAL)

**Purpose**: Make validate_preservation.sh CI-authoritative

1. **Fixture-Based Validation**
   - Create test fixtures (PASS cases)
   - Create test fixtures (FAIL cases)
   - Test false positive scenarios
   - Test false negative scenarios
   - Target: `.kiro/specs/abdf-contract-technical-corrections/test_fixtures/`

2. **Diff Hunk → Section Resolver**
   - Parse diff hunk line ranges
   - Map line ranges to document sections
   - Accurate section change detection
   - Target: `validate_preservation.sh` enhancement

3. **Robust YAML Parsing**
   - Replace regex-based parsing
   - Use Python script or yq tool
   - Handle complex YAML structures
   - Target: `parse_expected_changes.py` or yq integration

4. **Integration Tests**
   - Test with real spec examples
   - Verify PASS/FAIL behavior
   - Document edge cases
   - Target: `.kiro/specs/abdf-contract-technical-corrections/tests/`

### ⏳ Phase 2: CI Integration

5. **CI Gate: Spec Validation**
   - Add to `Makefile` as `ci-gate-spec-validation`
   - Integrate with `pre-ci` workflow
   - Generate evidence artifacts
   - Block on validation failure
   - Target: `Makefile`, `_ayken/ci/`

5. **Spec Validation Workflow**
   - Enforce Task 1 execution (ORIGINAL baseline)
   - Enforce bug proof on ORIGINAL (must FAIL)
   - Enforce bug proof on FIXED (must PASS)
   - Enforce preservation validation (must PASS)
   - Target: `.kiro/specs/SPEC_WORKFLOW.md`

6. **generate_final_report.sh**
   - Combine bug validation + preservation validation
   - Generate complete audit trail
   - Include transformation proof
   - Include preservation proof
   - Target: `.kiro/specs/abdf-contract-technical-corrections/generate_final_report.sh`

7. **Documentation**
   - Update spec workflow documentation
   - Add validation examples
   - Document CI integration
   - Create troubleshooting guide
   - Target: `_ayken/docs/SPEC_VALIDATION.md`

---

## Implementation Plan

### ✅ Phase 1: Validation Scripts - ALPHA COMPLETE

**Status**: 🟡 ALPHA (not CI-authoritative)

1. **validate_preservation.sh** (ALPHA Implementation)
   - ✅ Diff generation (ORIGINAL → FIXED)
   - ✅ Whitelist validation (expected_changes.yml)
   - ✅ Unexpected change detection
   - ✅ CI-compatible output (exit codes, evidence)
   - ✅ Evidence generation
   - ❌ Fixture-based PASS/FAIL tests
   - ❌ Diff hunk → section resolver
   - ❌ Robust YAML parsing
   - ❌ False positive/negative testing
   - **Status**: ALPHA - Initial implementation, requires hardening
   - **Location**: `.kiro/specs/abdf-contract-technical-corrections/validate_preservation.sh`

2. **Hardening Required for CI-Authoritative Status**:
   - [ ] Add fixture-based validation (intentional PASS/FAIL cases)
   - [ ] Implement diff hunk → section mapping
   - [ ] Replace regex YAML parsing with robust parser (or Python script)
   - [ ] Test false positive scenarios (preserved section in diff context)
   - [ ] Test false negative scenarios (unexpected change missed)
   - [ ] Integration tests with real spec examples
   - **Target**: Week 2 (before CI Integration)

### Phase 1.5: Hardening ⏳ CRITICAL (NEW)
- [ ] Create fixture-based PASS/FAIL tests
- [ ] Implement diff hunk → section resolver
- [ ] Replace regex YAML parsing with robust parser
- [ ] Test false positive/negative scenarios
- [ ] Integration tests with real specs
- [ ] Achieve CI-authoritative status
- **Target**: Week 2 (BLOCKS Phase 2)

### Phase 2: CI Integration ⏳ BLOCKED
**Blocked by**: Phase 1.5 hardening (CI-authoritative status required)
- [ ] Create `ci-gate-spec-validation` target
- [ ] Add to `pre-ci` workflow
- [ ] Generate evidence artifacts
- [ ] Test with existing specs
- **Target**: Week 3 (after Phase 1.5 complete)

### Phase 3: Process Enforcement ⏳ PENDING
- [ ] Make Level 3 validation mandatory
- [ ] Block merge on validation failure
- [ ] Integrate with spec workflow
- [ ] Update documentation

### Phase 4: Validation Dashboard ⏳ FUTURE
- [ ] Create validation status dashboard
- [ ] Track validation coverage
- [ ] Monitor validation failures
- [ ] Generate compliance reports

---

## Success Criteria

### Immediate (Phase-17.5)
- 🟡 `validate_preservation.sh` ALPHA complete (not CI-authoritative)
- ⏳ Hardening required (Phase 1.5)
- ⏳ CI gate integration (blocked by Phase 1.5)
- ⏳ Level 3 validation enforced
- ⏳ Documentation complete

### Long-term (All Specs)
- ⏳ 100% of specs achieve Level 3 validation
- ⏳ Zero validation failures in CI
- ⏳ Complete audit trail for all specs
- ⏳ Validation dashboard operational

---

## Validation Levels

### Level 0: No Validation ❌
- Manual inspection only
- No automated checks
- No evidence trail
- **Status**: DEPRECATED

### Level 1: FIXED-State Verification 🟡
- Automated bug absence check
- Automated fix presence check
- Evidence: bug_condition_fixed_*.md
- **Limitation**: No transformation proof
- **Status**: MINIMUM (current ABDF spec)

### Level 2: Transformation Proof 🟢
- ORIGINAL baseline captured
- Bug proof on ORIGINAL (FAIL expected)
- Bug proof on FIXED (PASS expected)
- Evidence: bug_condition_original_*.md + bug_condition_fixed_*.md
- **Limitation**: No preservation proof
- **Status**: TARGET (next spec)

### Level 3: Complete Validation 🟢🟢
- Level 2 + Preservation proof
- Diff validation (ORIGINAL → FIXED)
- Whitelist-based change verification
- Evidence: full validation report
- **Provides**: Complete audit trail
- **Status**: MANDATORY (Phase-17.5 complete)

---

## Timeline

- **Week 1**: ✅ Validation scripts (ALPHA COMPLETE)
- **Week 2**: ⏳ Hardening (Phase 1.5 - CRITICAL)
- **Week 3**: ⏳ CI integration (blocked by Week 2)
- **Week 4**: ⏳ Process enforcement
- **Week 5**: ⏳ Documentation + rollout

---

## Commitment

**This validation failure will NOT be repeated.**

Future specs MUST:
- ✅ Capture ORIGINAL baseline BEFORE fixes
- ✅ Prove bugs exist in ORIGINAL (validation FAIL)
- ✅ Prove bugs fixed in FIXED (validation PASS)
- ✅ Prove only expected changes made (preservation PASS)
- ✅ Achieve Level 3 validation before merge

**No exceptions. No manual overrides. No shortcuts.**

---

## References

- ABDF Validation Failure: `.kiro/specs/abdf-contract-technical-corrections/VALIDATION_FAILURE_ANALYSIS.md`
- Validation README: `.kiro/specs/abdf-contract-technical-corrections/VALIDATION_README.md`
- Corrected Status: `.kiro/specs/abdf-contract-technical-corrections/CORRECTED_STATUS.md`
- Self-Review Checklist: `.kiro/specs/abdf-contract-technical-corrections/SELF_REVIEW_CHECKLIST.md`

---

**Phase Status**: 🟡 IN PROGRESS  
**Next Action**: CI gate integration  
**Owner**: Kiro + Kenan AY (Architectural Review)

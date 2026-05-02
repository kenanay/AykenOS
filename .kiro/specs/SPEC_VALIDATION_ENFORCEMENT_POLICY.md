# Spec Validation Enforcement Policy

**Authority**: Constitutional Enforcement (Phase-17.5)  
**Status**: MANDATORY - CI-AUTHORITATIVE  
**Owner**: Kenan AY - Architectural Steward

---

## Policy Statement

All specification changes MUST achieve Level 3 validation before merge.

**No exceptions. No manual overrides. No shortcuts.**

---

## Enforcement Mechanism

### CI Gate: Spec Validation

**Location**: `.github/workflows/ci-freeze.yml`

**Trigger**: Every CI run (pull request, push to main)

**Behavior**: FAIL-CLOSED (strict enforcement)

---

## Validation Levels

### Level 0: No Validation ❌ PROHIBITED
- Manual inspection only
- No automated checks
- No evidence trail
- **Status**: DEPRECATED - NOT ALLOWED

### Level 1: FIXED-State Verification 🟡 INSUFFICIENT
- Automated bug absence check
- Automated fix presence check
- Evidence: `bug_condition_fixed_*.md`
- **Limitation**: No transformation proof
- **Status**: INSUFFICIENT - NOT ALLOWED for new specs

### Level 2: Transformation Proof 🟡 INSUFFICIENT
- ORIGINAL baseline captured
- Bug proof on ORIGINAL (FAIL expected)
- Bug proof on FIXED (PASS expected)
- Evidence: `bug_condition_original_*.md` + `bug_condition_fixed_*.md`
- **Limitation**: No preservation proof
- **Status**: INSUFFICIENT - NOT ALLOWED for new specs

### Level 3: Complete Validation 🟢 MANDATORY
- Level 2 + Preservation proof
- Diff validation (ORIGINAL → FIXED)
- Whitelist-based change verification
- Evidence: full validation report (markdown + JSON)
- **Provides**: Complete audit trail
- **Status**: MANDATORY - ENFORCED BY CI ✅

---

## Enforcement Rules

### Rule 1: Validation Infrastructure REQUIRED

**Policy**: All specs MUST have validation infrastructure

**Check**:
```bash
if [ ! -f "$spec_dir/ci_gate_spec_validation.sh" ]; then
  echo "❌ FAIL: No validation infrastructure"
  exit 1
fi
```

**Consequence**: CI FAIL → merge blocked

**Rationale**: No spec without validation capability

---

### Rule 2: ORIGINAL Baseline REQUIRED (New Specs)

**Policy**: All new specs MUST capture ORIGINAL baseline before fixes

**Check**:
```bash
if [ ! -f "$spec_dir/ORIGINAL_BASELINE.md" ]; then
  # Legacy exception (pre-Phase-17.5 specs)
  echo "⚠️  LEGACY SPEC: Allowed but logged"
else
  # Run Level 3 validation
  make ci-gate-spec-validation
fi
```

**Consequence**: 
- New specs without ORIGINAL → CI FAIL → merge blocked
- Legacy specs (pre-Phase-17.5) → logged but allowed

**Rationale**: Transformation proof required for new specs

---

### Rule 3: Validation MUST Pass

**Policy**: Validation failure blocks merge

**Check**:
```bash
if ! make ci-gate-spec-validation; then
  echo "❌ Validation FAIL"
  exit 1
fi
```

**Consequence**: CI FAIL → merge blocked

**Rationale**: Only validated changes allowed

---

### Rule 4: CI-Authoritative Assertion

**Policy**: Validator MUST assert CI-authoritative status

**Check**:
```python
if os.getenv('CI') == 'true':
    if not report_json.get('ci_authoritative', False):
        sys.exit(2)
    if not report_json.get('deterministic', False):
        sys.exit(2)
```

**Consequence**: Assertion failure → CI FAIL → merge blocked

**Rationale**: Validator must prove it's CI-authoritative

---

### Rule 5: Evidence MUST Be Generated

**Policy**: All validation runs MUST generate evidence artifacts

**Check**:
```yaml
- name: Upload Spec Validation Evidence
  if: ${{ always() }}
  uses: actions/upload-artifact@v4
```

**Consequence**: Evidence uploaded even on failure

**Rationale**: "No evidence = no truth"

---

## Fail-Closed Behavior

### Scenario 1: Spec Without Validation Infrastructure

**Input**: Spec exists, no `ci_gate_spec_validation.sh`

**Behavior**: ❌ CI FAIL

**Message**:
```
❌ FAIL: No validation infrastructure found
   Required: .kiro/specs/my-spec/ci_gate_spec_validation.sh
   Policy: All specs MUST have validation infrastructure
```

**Rationale**: No silent skip - validation infrastructure mandatory

---

### Scenario 2: Validation Failure

**Input**: Validation runs, returns exit code 1

**Behavior**: ❌ CI FAIL

**Message**:
```
❌ Validation FAIL: my-spec
   Policy: Validation failure blocks merge
```

**Rationale**: Only validated changes allowed

---

### Scenario 3: CI-Authoritative Assertion Failure

**Input**: Validator passes but JSON report has `ci_authoritative: false`

**Behavior**: ❌ CI FAIL

**Message**:
```
❌ CI-AUTHORITATIVE ASSERTION FAILED
Validator claims to be CI-authoritative but JSON report says otherwise
```

**Rationale**: Validator must prove its authority

---

### Scenario 4: Legacy Spec (Pre-Cutoff)

**Input**: Spec created before 2026-05-02 19:00:00, no `ORIGINAL_BASELINE.md`

**Behavior**: ⚠️  LOGGED but ALLOWED

**Message**:
```
⚠️  LEGACY SPEC: No ORIGINAL baseline (pre-Phase-17.5)
   Spec: my-spec
   Created: 2026-04-15T10:30:00Z
   Status: Allowed (legacy exception)
   Reason: Created before Phase-17.5 cutoff
```

**Rationale**: Pre-cutoff specs grandfathered (cannot retroactively capture ORIGINAL)

---

### Scenario 5: Legacy Freeze Violation (Post-Cutoff)

**Input**: Spec created after 2026-05-02 19:00:00, no `ORIGINAL_BASELINE.md`

**Behavior**: ❌ CI FAIL

**Message**:
```
❌ LEGACY FREEZE VIOLATION
   Spec: my-spec
   Created: 2026-05-03T10:00:00Z
   Cutoff: 2026-05-02 19:00:00
   Policy: Specs created after Phase-17.5 MUST have ORIGINAL baseline
   Required: .kiro/specs/my-spec/ORIGINAL_BASELINE.md
```

**Rationale**: Legacy freeze prevents abuse - new specs have no excuse

---

### Scenario 6: All Specs Legacy

**Input**: Multiple specs, all legacy (no ORIGINAL)

**Behavior**: ⚠️  LOGGED but ALLOWED

**Message**:
```
⚠️  All specs are legacy (no validation performed)
   Note: Future specs MUST have validation infrastructure
```

**Rationale**: System not broken by legacy specs, but future specs enforced

---

### Scenario 7: No Specs Directory

**Input**: `.kiro/specs/` does not exist

**Behavior**: ✅ PASS (validation not applicable)

**Message**:
```
✅ No specs directory found - validation not applicable
```

**Rationale**: Validation only applies when specs exist

---

## Legacy Exception Policy

### Definition

**Legacy Spec**: Spec created before Phase-17.5 cutoff (2026-05-02 19:00:00 UTC)

**Identifier**: 
1. Missing `ORIGINAL_BASELINE.md` AND
2. First commit before Phase-17.5 cutoff

### Legacy Freeze

**Cutoff Date**: 2026-05-02 19:00:00 UTC  
**Cutoff Commit**: 500ed7b3

**Policy**: No new legacy exceptions after cutoff

**Enforcement**:
```bash
spec_first_commit=$(git log --follow --format=%aI --reverse -- "$spec_dir" | head -1)
spec_epoch=$(date -u -d "$spec_first_commit" +%s)

if [ "$spec_epoch" -gt "$PHASE_17_5_CUTOFF_EPOCH" ]; then
  # Created AFTER cutoff → VIOLATION
  echo "❌ LEGACY FREEZE VIOLATION"
  exit 1
else
  # Created BEFORE cutoff → LEGACY EXCEPTION
  echo "⚠️  LEGACY SPEC: Allowed (pre-Phase-17.5)"
fi
```

### Behavior

**Pre-Cutoff Specs** (created before 2026-05-02 19:00:00):
- ⚠️  Logged (not silent)
- ✅ Allowed (legacy exception)
- 📊 Counted in summary
- 🔒 Frozen (no new exceptions)

**Post-Cutoff Specs** (created after 2026-05-02 19:00:00):
- ❌ MUST have ORIGINAL baseline
- ❌ No legacy exception
- ❌ CI FAIL if missing ORIGINAL
- 🔒 Enforced (legacy freeze active)

### Rationale

- Pre-Phase-17.5 specs cannot retroactively capture ORIGINAL
- Blocking them would break existing work
- Logging ensures visibility
- **Legacy freeze prevents abuse**: New specs have no excuse

### Future

- New specs (post-cutoff) MUST have ORIGINAL
- No new legacy exceptions (frozen)
- Existing legacy specs remain allowed
- Legacy freeze prevents loophole

---

## Validation Requirements

### For New Specs (Post-Phase-17.5)

**MANDATORY**:
1. ✅ `ci_gate_spec_validation.sh` (validation infrastructure)
2. ✅ `ORIGINAL_BASELINE.md` (captured BEFORE fixes)
3. ✅ `FIXED_DOCUMENT.md` (after fixes applied)
4. ✅ `expected_changes.yml` (whitelist of expected changes)
5. ✅ `validate_bug_conditions.sh` (Level 1 validation)
6. ✅ `validate_preservation.py` (Level 3 validation)

**VALIDATION FLOW**:
1. Bug proof on ORIGINAL (must FAIL - proves bugs exist)
2. Bug proof on FIXED (must PASS - proves bugs fixed)
3. Preservation validation (must PASS - proves no scope creep)

**EVIDENCE**:
- Markdown reports (human-readable)
- JSON reports (machine-readable)
- Diff patches
- Validation logs

---

## CI Integration

### Workflow: ci-freeze.yml

**Step 1: Spec Validation Gate**
- Runs after `make ci-freeze`
- Auto-discovers specs
- Validates each spec independently
- FAIL → pipeline FAIL

**Step 2: Evidence Upload**
- Uploads artifacts (always, even on failure)
- 30-day retention
- Includes reports, diffs, logs

**Step 3: Merge Decision**
- PASS → CI continues → merge allowed
- FAIL → CI fails → merge blocked

---

## Enforcement Guarantees

### 1. Merge Blocking ✅
- Validation FAIL → CI FAIL
- CI FAIL → GitHub blocks merge
- No manual override possible

### 2. Evidence Trail ✅
- All runs generate evidence
- Evidence uploaded to GitHub artifacts
- 30-day retention
- Available even on failure

### 3. Fail-Closed ✅
- No silent skips
- No graceful degradation
- Validation infrastructure mandatory
- Assertion failures block merge

### 4. Deterministic ✅
- Canonical section ID system
- Same input → same output (always)
- Path-independent execution
- 100% test coverage

### 5. CI-Authoritative ✅
- Validator asserts its authority
- CI verifies assertion
- False claims block merge

---

## Commitment

**This validation failure will NOT be repeated.**

Future specs MUST:
- ✅ Capture ORIGINAL baseline BEFORE fixes
- ✅ Prove bugs exist in ORIGINAL (validation FAIL)
- ✅ Prove bugs fixed in FIXED (validation PASS)
- ✅ Prove only expected changes made (preservation PASS)
- ✅ Achieve Level 3 validation before merge

**Enforcement**: CI-AUTHORITATIVE (blocking)

**No exceptions. No manual overrides. No shortcuts.**

---

## References

- **Phase-17.5 Summary**: `.kiro/specs/abdf-contract-technical-corrections/PHASE_17_5_SUMMARY.md`
- **CI-Authoritative Status**: `.kiro/specs/abdf-contract-technical-corrections/PHASE_17_5_CI_AUTHORITATIVE.md`
- **Python Validator**: `.kiro/specs/abdf-contract-technical-corrections/validate_preservation.py`
- **CI Gate Script**: `.kiro/specs/abdf-contract-technical-corrections/ci_gate_spec_validation.sh`
- **CI Workflow**: `.github/workflows/ci-freeze.yml`

---

**Policy Status**: ACTIVE - CI-AUTHORITATIVE ✅  
**Enforcement**: FAIL-CLOSED (strict)  
**Authority**: Constitutional (Phase-17.5)  
**Owner**: Kenan AY - Architectural Steward


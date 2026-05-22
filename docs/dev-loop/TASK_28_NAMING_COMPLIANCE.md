# Task 28: Naming Convention Compliance Enforcement

**Author**: Kenan AY — System Architect
**Status**: Complete
**Date**: 2026-05-08

---

## Overview

Task 28 implements automated naming convention compliance enforcement across the Ayken codebase. This ensures consistent use of canonical terminology and prevents architectural drift through naming violations.

---

## Requirements

### R25: Naming Convention Enforcement
The system SHALL enforce consistent naming conventions across artifacts.

### R30: Naming Enforcement Scope
Naming conventions SHALL apply across all system layers.

---

## Implementation

### Sub-task 28.1: Naming Compliance Check Capability

**Script**: `scripts/check_naming_compliance.sh`

**Purpose**: Detect and prevent naming convention violations in modified files.

**Checks**:
1. **Forbidden Term Detection**: Identifies usage of "aykenos" in new code
2. **Path Pattern Detection**: Identifies "phase-*" naming in file paths
3. **Canonical Usage Verification**: Confirms proper use of "ayken"
4. **Legacy Usage Tracking**: Reports deprecated naming (informational)

**Exit Codes**:
- `0`: PASS - No violations detected
- `1`: FAIL - Violations found

**Enforcement Rules**:
- ✅ Use "ayken" (canonical identifier)
- ❌ Do NOT use "aykenos" in new code
- ❌ Do NOT use "phase-*" in new file/directory names
- ⚠️ Legacy usage allowed but deprecated

**Example Output**:
```
== CHECK: Naming Convention Compliance ==
Checking modified files for naming violations...

Checking for forbidden term: 'aykenos'...
❌ VIOLATION: 'aykenos' found in: src/example.rs

Checking for forbidden naming pattern: 'phase-*' in paths...
❌ VIOLATION: 'phase-*' naming in path: docs/phase-1/spec.md

🚨 CRITICAL FAILURE: Naming convention violated
```

---

### Sub-task 28.2: Naming Compliance CI Integration

**Workflow**: `.github/workflows/governance-naming-compliance.yml`

**Purpose**: Automatically enforce naming conventions in CI pipeline.

**Triggers**:
- Push to `main` or `master` branches
- Pull requests targeting `main` or `master`

**Behavior**:
1. Checkout repository with full history
2. Install optional tools (ripgrep for performance)
3. Execute naming compliance check
4. Upload failure artifacts for debugging
5. Block merge on violations

**Parallel Execution**: Runs alongside other governance checks:
- Evidence isolation check
- Observation boundary check
- Naming compliance check (this task)

**Failure Handling**:
- Any governance check failure blocks merge
- Artifacts uploaded for debugging
- Clear error messages with remediation steps

---

## Advanced CI Integration

### Enhanced Naming Convention Check

**Script**: `scripts/ci/check_naming_convention.sh`

**Purpose**: Production-grade naming enforcement with evidence generation.

**Features**:
- Diff-based analysis (only checks changed lines)
- Regex-based pattern matching
- Legacy allowlist support
- Evidence artifact generation
- JSON report output

**Configuration Files**:

1. **`scripts/ci/naming-convention-scope.regex`**
   - Defines which files are subject to naming enforcement
   - Example: `^kernel/include/execution_.*\.h$`

2. **`scripts/ci/naming-convention-deny.regex`**
   - Patterns forbidden in new code
   - Example: `\bworker\b`, `\bthread\b`, `\btask\b`, `\bjob\b`

3. **`scripts/ci/naming-convention-legacy-allow.regex`**
   - Frozen legacy files exempt from enforcement
   - Example: `^kernel/proc/proc\.c$`

**Evidence Artifacts**:
- `changed-files.txt` - All modified files
- `scoped-files.txt` - Files subject to enforcement
- `allowlisted-files.txt` - Legacy files exempt
- `hits.txt` - Detected violations
- `violations.txt` - Violation details
- `report.json` - Structured report
- `diff.patch` - Git diff for analysis

**Exit Codes**:
- `0`: PASS or SKIP (no scoped changes)
- `2`: FAIL (violations detected)
- `3`: ERROR (tooling/usage error)

---

## Constitutional Compliance

### Naming Law (Section 10)

**Canonical Identifier**: "ayken"
- System name: Ayken
- Codebase prefix: `ayken_*`
- Module naming: `ayken-*`

**Forbidden Terms**:
- "aykenos" (deprecated legacy name)
- "phase-*" (architectural drift indicator)

**Rationale**:
- Consistent terminology prevents confusion
- Canonical naming enables tooling
- Prevents architectural drift
- Maintains professional identity

---

## Governance Integration

### Parallel Enforcement

All governance checks run in parallel:
```
┌─────────────────────────────────────┐
│     Governance Enforcement          │
├─────────────────────────────────────┤
│  ✓ Evidence Isolation               │
│  ✓ Observation Boundary             │
│  ✓ Naming Compliance (Task 28)      │
└─────────────────────────────────────┘
         ↓
    Any Failure → Block Merge
```

### Governance Summary

**Workflow**: `.github/workflows/governance-summary.yml`

Provides overview of all governance enforcement mechanisms:
- Lists all active checks
- Verifies scripts exist
- Confirms constitutional documentation
- References requirements

---

## Testing

### Test Script

**Location**: `scripts/test_task28_naming_compliance.sh`

**Coverage**:
1. **Sub-task 28.1 Tests**:
   - Script existence and executability
   - Violation detection logic
   - Exit code contract
   - Developer signature presence

2. **Sub-task 28.2 Tests**:
   - CI workflow existence
   - Trigger configuration
   - Script execution
   - Branch targeting
   - Artifact upload on failure

3. **Advanced Integration Tests**:
   - Enhanced CI script presence
   - Regex configuration files
   - Evidence generation

4. **Functional Tests**:
   - Violation detection accuracy
   - Clean state handling

5. **Constitutional Compliance**:
   - Requirement references
   - Constitutional authority

**Execution**:
```bash
./scripts/test_task28_naming_compliance.sh
```

**Expected Output**:
```
✅ Task 28: Naming Convention Compliance Enforcement - COMPLETE

Validated:
  ✓ 28.1: Naming compliance check capability
  ✓ 28.2: Naming compliance CI integration

Requirements Satisfied:
  ✓ R25: Naming Convention Enforcement
  ✓ R30: Naming Enforcement Scope

Constitutional Compliance:
  ✓ Naming Law (Section 10)
  ✓ Governance Enforcement
```

---

## Usage

### Local Development

**Check naming compliance**:
```bash
./scripts/check_naming_compliance.sh
```

**Fix violations**:
1. Replace "aykenos" with "ayken"
2. Rename files to remove "phase-*" pattern
3. Re-run check to verify

### CI Pipeline

**Automatic enforcement**:
- Runs on every push and PR
- Blocks merge on violations
- Provides clear error messages

**Bypass** (not recommended):
- No bypass mechanism (intentional)
- Violations must be fixed
- Constitutional requirement

---

## Architecture

### Design Principles

1. **Non-Interference**: Check is read-only, doesn't modify code
2. **Determinism**: Same input → same output
3. **Fail-Fast**: Violations detected immediately
4. **Clear Feedback**: Actionable error messages
5. **Constitutional Authority**: Backed by governance model

### Isolation Model

```
Modified Files → Naming Check → PASS/FAIL
                      ↓
                 No Side Effects
                 No Code Modification
                 Pure Validation
```

### Integration Model

```
Developer Commit
    ↓
Git Hook (optional)
    ↓
CI Trigger
    ↓
Governance Checks (parallel)
    ├─ Evidence Isolation
    ├─ Observation Boundary
    └─ Naming Compliance ← Task 28
    ↓
All Pass → Merge Allowed
Any Fail → Merge Blocked
```

---

## Maintenance

### Adding New Forbidden Terms

1. Edit `scripts/check_naming_compliance.sh`
2. Add pattern to detection logic
3. Update error messages
4. Add test case
5. Update documentation

### Exempting Legacy Files

1. Document reason for exemption
2. Add to legacy allowlist (if using advanced CI)
3. Mark as deprecated
4. Plan migration path

### Updating CI Workflow

1. Edit `.github/workflows/governance-naming-compliance.yml`
2. Test locally first
3. Verify parallel execution
4. Confirm artifact upload
5. Update governance summary

---

## References

- **Requirements**: `.kiro/specs/dev-loop-boot-monitoring/requirements.md`
- **Design**: `.kiro/specs/dev-loop-boot-monitoring/design.md` (Section 9)
- **Tasks**: `.kiro/specs/dev-loop-boot-monitoring/tasks.md` (Task 28)
- **Constitution**: `.kiro/specs/dev-loop-boot-monitoring/DEV_LOOP_CONSTITUTION.md` (Section 10)
- **Test**: `scripts/test_task28_naming_compliance.sh`

---

## Completion Criteria

- [x] 28.1: Naming compliance check capability implemented
- [x] 28.2: Naming compliance CI integration complete
- [x] Test script created and passing
- [x] Documentation complete
- [x] Constitutional compliance verified
- [x] Requirements R25 and R30 satisfied

---

**Status**: ✅ COMPLETE
**Validated**: 2026-05-08
**Maintainer**: Kenan AY — System Architect

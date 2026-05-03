# Governance Enforcement System

**Author**: Kenan AY  
**Role**: System Architect / Developer / Designer / Implementer  
**Status**: Active Enforcement

---

## Overview

The Ayken dev loop governance system ensures architectural boundaries are preserved through automated enforcement. This system prevents gradual drift toward "tool-driven runtime" by maintaining strict separation between observation, validation, and derived data.

---

## Enforcement Architecture

```
┌─────────────────────────────────────────────────────────┐
│              CONSTITUTIONAL LAYER                        │
│         (DEV_LOOP_CONSTITUTION.md)                      │
│  Defines immutable rules and authority model            │
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────┴────────────────────────────────┐
│              REQUIREMENTS LAYER                          │
│         (requirements.md: Req 26-30)                    │
│  Specifies acceptance criteria for compliance           │
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────┴────────────────────────────────┐
│              ENFORCEMENT LAYER                           │
│         (Automated CI checks)                           │
│  Verifies compliance on every commit/PR                 │
└─────────────────────────────────────────────────────────┘
```

---

## Enforcement Mechanisms

### 1. Evidence Isolation Check

**Script**: `scripts/check_evidence_isolation.sh`  
**CI Workflow**: `.github/workflows/governance-evidence-isolation.yml`

**Purpose**: Ensures validation scripts NEVER read from `out/evidence/` directory.

**What it checks**:
- Validation scripts do not reference `out/evidence/`
- Evidence artifacts (summary.json, markers.json, perf.json) not used in validation
- Evidence generation happens AFTER validation decisions

**Forbidden Pattern**:
```bash
# ❌ FORBIDDEN
if grep "PASS" out/evidence/latest/summary.json; then
  validation_decision="PASS"  # evidence as authority!
fi
```

**Correct Pattern**:
```bash
# ✅ CORRECT
if grep "BOOT_OK" out/logs/boot_watch.log; then
  validation_decision="PASS"  # direct observation
fi
```

**Constitutional Reference**: Section 5 (Evidence Law), Section 6 (Observation Source Constraint)

---

### 2. Observation Boundary Check

**Script**: `scripts/check_observation_boundary.sh`  
**CI Workflow**: `.github/workflows/governance-observation-boundary.yml`

**Purpose**: Ensures validation decisions use ONLY raw boot logs.

**What it checks**:
- Validation scripts read from `out/logs/` only
- Evidence generation order (must be after validation)
- No historical run dependencies (history.json not used for validation)
- Safe zones verified (tools/web can read evidence for visualization)

**Critical Boundary**:
```
Validation → reads → out/logs/boot_watch.log ✅
Validation → reads → out/evidence/          ❌
```

**Constitutional Reference**: Section 6 (Observation Source Constraint), Section 7 (State Isolation Law)

---

### 3. Naming Compliance Check

**Script**: `scripts/check_naming_compliance.sh`  
**CI Workflow**: `.github/workflows/governance-naming-compliance.yml`

**Purpose**: Enforces naming conventions across the codebase.

**What it checks**:
- New code does not use "aykenos" (canonical: "ayken")
- New paths do not use "phase-*" naming
- Only modified files are checked (legacy usage allowed)

**Rules**:
- ✅ Canonical (code artifacts): `ayken`
- ✅ Project name (documentation only): `AykenOS`
- ❌ Forbidden (in code/CI/files): `aykenos` (lowercase)
- ❌ Forbidden: `phase-*` (in new paths)

**Constitutional Reference**: Section 10 (Naming Law)

---

### 4. Spec Purity Check

**Script**: `scripts/check_spec_purity.sh`  
**CI Workflow**: `.github/workflows/governance-spec-purity.yml`

**Purpose**: Ensures specification documents contain only normative content, not implementation details.

**What it checks**:
- Spec files do not contain code snippets (bash, python, javascript)
- Spec files do not contain command examples (grep, make, git)
- Spec files do not contain tool-specific instructions
- Spec files do not contain JSON/YAML schemas

**Forbidden Patterns in Spec**:
```bash
# ❌ FORBIDDEN in spec files
grep -E "pattern"
make -j4 build
python3 script.py
const foo = () => {}
{"key": "value"}
```

**Allowed Patterns in Spec**:
```markdown
# ✅ ALLOWED in spec files
The system SHALL validate markers
Validation uses grep to search logs
Evidence is structured as JSON
```

**Constitutional Reference**: Section 11 (Spec Purity Rule)

---

## CI Integration

**Implementation Guide**: For detailed CI setup instructions, see `docs/dev-loop/CI_INTEGRATION.md`

### Parallel Execution

All governance checks run in parallel for fast feedback:

```
┌─────────────────────────────────────────────────────────┐
│                    GitHub Actions                        │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────────┐  ┌──────────────────┐            │
│  │ Evidence         │  │ Observation      │            │
│  │ Isolation        │  │ Boundary         │            │
│  │ ✅ PASS          │  │ ✅ PASS          │            │
│  └──────────────────┘  └──────────────────┘            │
│                                                          │
│  ┌──────────────────┐  ┌──────────────────┐            │
│  │ Naming           │  │ Governance       │            │
│  │ Compliance       │  │ Summary          │            │
│  │ ✅ PASS          │  │ ✅ PASS          │            │
│  └──────────────────┘  └──────────────────┘            │
└─────────────────────────────────────────────────────────┘
```

### Workflow Files

| Workflow | Purpose | Trigger |
|----------|---------|---------|
| `governance-evidence-isolation.yml` | Evidence boundary enforcement | push, PR |
| `governance-observation-boundary.yml` | Validation source constraint | push, PR |
| `governance-naming-compliance.yml` | Naming convention enforcement | push, PR |
| `governance-spec-purity.yml` | Spec purity enforcement | push, PR |
| `governance-summary.yml` | Overview and verification | push, PR |

---

## Local Development

### Running Checks Locally

```bash
# Evidence isolation
./scripts/check_evidence_isolation.sh

# Observation boundary
./scripts/check_observation_boundary.sh

# Naming compliance
./scripts/check_naming_compliance.sh

# Spec purity
./scripts/check_spec_purity.sh
```

### Makefile Integration

```bash
# Run all governance checks
make ci-gate-governance

# Individual checks
make ci-gate-evidence-isolation
make ci-gate-observation-boundary
make ci-gate-naming-compliance
make ci-gate-spec-purity
```

---

## Violation Handling

### Severity Levels

All governance violations are **CRITICAL** and cause immediate CI failure.

| Violation Type | Severity | Action |
|----------------|----------|--------|
| Evidence used as validation input | CRITICAL | CI FAIL |
| Validation reads from evidence | CRITICAL | CI FAIL |
| Naming convention violated | CRITICAL | CI FAIL |
| Observation boundary breached | CRITICAL | CI FAIL |
| Spec contains implementation details | CRITICAL | CI FAIL |

### Error Messages

All enforcement scripts provide:
- Clear violation description
- Constitutional reference
- Requirements reference
- Fix instructions

Example:
```
🚨 CRITICAL FAILURE: Evidence isolation violated

Rule:
  Evidence MUST NOT be used as input to validation logic

Fix:
  Use raw logs (out/logs) for validation
  Use evidence only for visualization

Constitutional Reference:
  See .kiro/specs/dev-loop-boot-monitoring/DEV_LOOP_CONSTITUTION.md
  Section 5: Evidence Law
  Section 6: Observation Source Constraint
```

---

## Maintenance

### Adding New Checks

1. Create enforcement script in `scripts/`
2. Add constitutional rule to `DEV_LOOP_CONSTITUTION.md`
3. Add requirement to `requirements.md`
4. Create CI workflow in `.github/workflows/`
5. Update this document

### Modifying Existing Checks

All changes to governance enforcement require:
1. Architectural review (Kenan AY)
2. Constitutional amendment (if rules change)
3. Requirements update
4. CI workflow update
5. Documentation update

---

## Constitutional Compliance

This governance system enforces:

- **Requirement 26**: Direct Observation Source Constraint
- **Requirement 27**: Evidence State Isolation
- **Requirement 28**: Dev Loop Scope Limitation
- **Requirement 29**: Signature Non-Propagation
- **Requirement 30**: Naming Enforcement Scope

See `DEV_LOOP_CONSTITUTION.md` for complete constitutional framework.

---

## Monitoring

### CI Dashboard

View governance status:
- GitHub Actions → Workflows → Filter by "Governance"
- All checks must pass for merge

### Artifacts

On failure, CI uploads:
- Violation details
- Script output
- Relevant file excerpts

Retention: 7 days

---

## FAQ

**Q: Why are governance checks separate workflows?**  
A: Isolation, parallelism, and clear failure attribution. Each check is an independent enforcement layer.

**Q: Can I disable a check temporarily?**  
A: No. Governance checks are constitutional and cannot be bypassed.

**Q: What if I need to use evidence in validation?**  
A: This violates the authority model. Evidence is derived data, not authority. Use raw logs instead.

**Q: How do I fix a naming violation?**  
A: Replace "aykenos" with "ayken" in modified files. Legacy usage is allowed but deprecated.

**Q: What if a check has false positives?**  
A: Report to architectural steward (Kenan AY) for review. Checks are designed to be strict.

---

**End of Governance Documentation**

---

**Signature**:  
Kenan AY — System Architect  
Date: 2026-05-03

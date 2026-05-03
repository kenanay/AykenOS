# Dev Loop Constitution

**Author**: Kenan AY  
**Role**: System Architect / Developer / Designer / Implementer  
**Status**: Constitutional Rule Set (Immutable)  
**Authority**: Architectural Steward

---

## 1. Purpose

The dev loop exists solely as a **validation observer**.

It MUST NOT:
- Control execution
- Influence runtime
- Act as authority

---

## 2. Fundamental Separation

The system is divided into four layers:

1. **Runtime** (kernel)
2. **Observation** (logs)
3. **Derived Data** (evidence)
4. **Visualization** (dashboard)

### STRICT RULE

```
Runtime → Logs → Evidence → UI
```

### NEVER

```
Evidence → Validation  ❌
Evidence → Runtime     ❌
UI → Execution         ❌
```

---

## 3. Authority Model

**Authority** is defined as:
- PASS/FAIL decision
- Execution control
- Runtime behavior influence

### RULE

| Component | Authority Status |
|-----------|-----------------|
| Dev loop  | ✅ Authority (validation only) |
| Evidence  | ❌ NOT authority |
| Dashboard | ❌ NOT authority |

**Violation = Constitutional Failure**

---

## 4. Non-Interference Law

### Dev loop SHALL

- ❌ NOT write to kernel state
- ❌ NOT alter execution flow
- ❌ NOT inject decisions
- ❌ NOT influence determinism

### Dev loop SHALL be

→ **Read-only observer**

---

## 5. Evidence Law

**Evidence** is defined as:
- Derived data
- Post-validation artifacts

### RULES

1. Evidence SHALL NOT affect decisions
2. Evidence SHALL NOT be reused as input
3. Evidence SHALL be immutable
4. Evidence SHALL be non-authoritative

### Forbidden Pattern

```bash
# ❌ FORBIDDEN
if grep "BOOT_OK" out/evidence/latest/reports/summary.json; then
  validation_decision="PASS"  # evidence as authority!
fi
```

### Correct Pattern

```bash
# ✅ CORRECT
if grep "BOOT_OK" out/logs/boot_watch.log; then
  validation_decision="PASS"  # direct observation
fi
# evidence is generated AFTER decision, never used as input
```

---

## 6. Observation Source Constraint

### Direct Observation Rule

Validation decisions SHALL ONLY use:
- Raw boot logs (`out/logs/`)
- Direct kernel output
- Unprocessed runtime data

Validation decisions SHALL NEVER use:
- Parsed artifacts (`out/evidence/`)
- Derived data (summary.json, markers.json, perf.json)
- Historical runs (history.json)

### Enforcement

Static analysis SHALL detect:
- Reading from `out/evidence/` in validation scripts
- Using evidence as input to oracle/validation logic
- Evidence contamination patterns

---

## 7. State Isolation Law

### Evidence State Rule

1. `history.json` SHALL be visualization-only
2. Historical runs SHALL NOT affect current validation
3. No decision SHALL depend on previous runs
4. Evidence SHALL be immutable per run
5. Derived data SHALL NOT re-enter validation pipeline

### Statelessness Property

Same input → Same validation output (deterministic)

---

## 8. Scope Limitation Law

### Dev Loop Scope

Dev loop SHALL remain a **validation tool**, not a system orchestrator.

### Forbidden Behaviors

- ❌ Controlling kernel execution logic
- ❌ Modifying runtime behavior
- ❌ Introducing execution policies
- ❌ Managing kernel state
- ❌ Acting as runtime controller

### Allowed Behaviors

- ✅ Observe
- ✅ Validate
- ✅ Report

---

## 9. Signature Law

**Developer signature**:
- Is metadata
- Is informational
- Has zero system authority

### MUST NOT

- ❌ Appear in runtime logs
- ❌ Affect execution
- ❌ Propagate into logic

### MUST

- ✅ Exist only in: `meta.json`, dashboard footer, script headers
- ✅ Remain purely informational

---

## 10. Naming Law

### Canonical Identifier

→ **ayken** ✅ (for code artifacts, file names, CI components)

### Project-Level Name

→ **AykenOS** ✅ (permitted ONLY in project README, manifests, architectural documents)

### Forbidden

→ **aykenos** (lowercase, new usage) ❌  
→ **phase-\*** naming ❌

### Enforcement

- CI SHALL fail on naming violations in code artifacts
- Partial compliance = violation
- Legacy usage allowed but deprecated
- "AykenOS" permitted only in high-level documentation, not in code/CI/file names

---

## 11. Violation Severity

All violations are **CRITICAL**:

| Violation Type | Severity | Action |
|----------------|----------|--------|
| Non-interference breach | CRITICAL | System MUST fail immediately |
| Evidence used as input | CRITICAL | System MUST fail immediately |
| Runtime influenced by dev loop | CRITICAL | System MUST fail immediately |
| Signature leakage | CRITICAL | System MUST fail immediately |
| Naming violation (new code) | CRITICAL | CI MUST fail |

---

## 12. Enforcement Mechanisms

The system SHALL enforce through:

1. **Static analysis** (`check_observation_boundary.sh`)
   - Detects evidence-as-input patterns
   - Scans validation scripts for forbidden patterns

2. **CI validation** (`.github/workflows/devloop-ci.yml`)
   - Runs naming compliance check
   - Runs observation boundary check
   - Runs evidence isolation check

3. **Isolation property tests** (`test_devloop_isolation.sh`)
   - Verifies dev loop doesn't affect kernel behavior
   - Compares baseline vs dev loop runs

4. **Evidence misuse detection** (`check_evidence_isolation.sh`)
   - Verifies evidence boundary
   - Detects evidence contamination

---

## 13. Architectural Data Flow

```
┌─────────────────────────────────────────────────────────┐
│                      RUNTIME LAYER                       │
│                    (kernel execution)                    │
└────────────────────────┬────────────────────────────────┘
                         │ writes (debugcon/serial)
                         ↓
┌─────────────────────────────────────────────────────────┐
│                   OBSERVATION LAYER                      │
│                  (out/logs/boot_watch.log)              │
└────────────┬────────────────────────────┬───────────────┘
             │ reads (validation)         │ reads (evidence gen)
             ↓                            ↓
┌────────────────────────┐    ┌──────────────────────────┐
│   VALIDATION LAYER     │    │   DERIVED DATA LAYER     │
│   (dev_loop.sh)        │    │   (out/evidence/)        │
│   PASS/FAIL decision   │    │   summary.json           │
│   ✅ AUTHORITY         │    │   markers.json           │
└────────────────────────┘    │   perf.json              │
                              │   history.json           │
                              │   ❌ NOT AUTHORITY       │
                              └──────────┬───────────────┘
                                         │ reads (visualization)
                                         ↓
                              ┌──────────────────────────┐
                              │  VISUALIZATION LAYER     │
                              │  (tools/web/index.html)  │
                              │  Dashboard (read-only)   │
                              │  ❌ NOT AUTHORITY        │
                              └──────────────────────────┘
```

### Critical Boundaries

1. **Validation → Evidence**: One-way only (validation never reads evidence)
2. **Evidence → Dashboard**: Read-only (dashboard never writes)
3. **Dashboard → Runtime**: No connection (dashboard never affects kernel)

---

## 14. Final Principle

> **Dev loop observes.**  
> **It does not decide reality.**  
> **It only reports it.**

---

## 15. Amendment Process

This constitution is **IMMUTABLE** without architectural review.

Any proposed changes MUST:
1. Be reviewed by Architectural Steward (Kenan AY)
2. Maintain non-interference guarantee
3. Preserve observation-validation boundary
4. Not introduce authority to evidence or dashboard

---

## 16. Compliance Verification

### Daily Checks

```bash
# Verify observation boundary
./scripts/check_observation_boundary.sh

# Verify evidence isolation
./scripts/check_evidence_isolation.sh

# Verify naming compliance
./scripts/check_naming_compliance.sh
```

### CI Enforcement

All checks MUST pass before merge.

---

**End of Constitution**

---

**Signature**:  
Kenan AY — System Architect  
Date: 2026-05-03

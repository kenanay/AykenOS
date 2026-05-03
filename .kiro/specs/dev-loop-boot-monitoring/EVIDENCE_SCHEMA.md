# Evidence Schema Specification

**Authority**: Kenan AY — System Architect  
**Status**: Normative  
**Version**: 1.0.0  
**Last Updated**: 2026-05-03

---

## Purpose

This document defines the **evidence schema** for the Development Loop & Boot Monitoring System. Evidence artifacts are **read-only observability outputs** that MUST NOT influence validation decisions.

---

## Constitutional Principles

### NON_AUTHORITY Principle

**Evidence artifacts are NON-AUTHORITATIVE.**

- Evidence = observation output
- Evidence ≠ validation input
- Validation decisions MUST use only raw boot logs

**Violation = Constitutional Breach**

---

### Determinism Guarantee

**Evidence generation MUST be deterministic.**

- Same input → same evidence
- No timestamps in validation logic
- No random generation
- No global state

**Violation = Determinism Breach**

---

### State Isolation

**Evidence artifacts MUST remain stateless.**

- No persistent state
- No cross-run dependencies
- Each run = independent

**Violation = State Coupling**

---

## Schema Structure

### Required Files

Every evidence run MUST produce:

```
out/evidence/run-{timestamp}/
├── meta.json           # Run metadata
├── logs/
│   └── boot.log        # Raw boot log (source of truth)
└── reports/
    ├── summary.json    # High-level summary
    ├── markers.json    # Marker presence
    └── perf.json       # Performance proxy
```

---

## Schema Definitions

### meta.json

**Purpose**: Run metadata and attribution

**Required Fields**:
- `schema_version` (string): Schema version (semver)
- `run_id` (string): Unique run identifier
- `timestamp` (string): ISO 8601 timestamp
- `source` (string): Evidence source (`dev_loop`, `ci`, `manual`)
- `deterministic` (boolean): Determinism guarantee flag
- `author` (string): Developer attribution
- `role` (array): Developer roles
- `signature_type` (string): Signature mechanism

**Example**:
```json
{
  "schema_version": "1.0.0",
  "run_id": "run-20260503-154530",
  "timestamp": "2026-05-03T15:45:30Z",
  "source": "dev_loop",
  "deterministic": true,
  "author": "Kenan AY",
  "role": ["developer", "architect"],
  "signature_type": "digital_meta"
}
```

---

### summary.json

**Purpose**: High-level validation summary

**Required Fields**:
- `boot` (string): Boot status (`PASS`, `FAIL`)
- `markers_ok` (boolean): Marker presence status
- `fail_closed` (boolean): Fail-closed detection
- `perf_regression` (boolean): Performance regression flag
- `timestamp` (string): ISO 8601 timestamp
- `run_id` (string): Run identifier
- `source` (string): Evidence source
- `deterministic` (boolean): Determinism flag

**Example**:
```json
{
  "boot": "PASS",
  "markers_ok": true,
  "fail_closed": false,
  "perf_regression": false,
  "timestamp": "2026-05-03T15:45:30Z",
  "run_id": "run-20260503-154530",
  "source": "dev_loop",
  "deterministic": true
}
```

---

### markers.json

**Purpose**: Boot marker presence

**Required Fields**:
- `EARLY_BOOT_OK` (boolean): Early boot marker
- `LATE_INIT_END` (boolean): Late init marker
- `BOOT_OK` (boolean): Boot complete marker
- `FAIL_CLOSED` (boolean): Fail-closed marker

**Example**:
```json
{
  "EARLY_BOOT_OK": true,
  "LATE_INIT_END": true,
  "BOOT_OK": true,
  "FAIL_CLOSED": false
}
```

---

### perf.json

**Purpose**: Performance proxy metrics

**Required Fields**:
- `boot_time_proxy` (number): Proxy metric value
- `method` (string): Measurement method
- `valid` (boolean): Validity flag
- `disclaimer` (string): Measurement disclaimer
- `unit` (string): Measurement unit

**Example**:
```json
{
  "boot_time_proxy": 1234,
  "method": "marker_delta",
  "valid": true,
  "disclaimer": "Proxy metric based on marker line count, NOT TSC-based measurement",
  "unit": "line_count"
}
```

---

## Validation Rules

### Schema Compliance

**All evidence artifacts MUST:**
- Conform to JSON Schema
- Include all required fields
- Use correct types
- Pass strict validation

**Violation = Schema Breach**

---

### NON_AUTHORITY Enforcement

**Evidence artifacts MUST NOT:**
- Influence validation decisions
- Be used as validation input
- Replace raw boot logs
- Contain authoritative data

**Violation = Authority Breach**

---

### Determinism Enforcement

**Evidence generation MUST:**
- Produce identical output for identical input
- Avoid non-deterministic operations
- Use deterministic timestamps only in metadata
- Maintain reproducibility

**Violation = Determinism Breach**

---

## Versioning Policy

### Schema Version Format

**Format**: `MAJOR.MINOR.PATCH` (semver)

**Rules**:
- MAJOR: Breaking changes
- MINOR: Backward-compatible additions
- PATCH: Backward-compatible fixes

---

### Compatibility Guarantee

**Validators MUST:**
- Support current schema version
- Reject unknown schema versions
- Provide clear error messages
- Fail closed on schema mismatch

---

## CI Enforcement

### Validation Contract

**CI MUST:**
- Validate all evidence artifacts
- Fail on schema violations
- Fail on missing required fields
- Fail on type mismatches

**Exit Status**:
- `0`: Valid evidence
- `1`: Schema violation
- `2`: Missing files

---

### Fail-Closed Guarantee

**On validation failure:**
- CI MUST fail
- No merge allowed
- Clear error reporting
- No silent failures

---

## References

- **Requirements**: `.kiro/specs/dev-loop-boot-monitoring/requirements.md`
- **Design**: `.kiro/specs/dev-loop-boot-monitoring/design.md`
- **Tasks**: `.kiro/specs/dev-loop-boot-monitoring/tasks.md`
- **JSON Schema**: `tools/evidence/evidence-schema.json`
- **Validator**: `tools/evidence/validate-evidence.sh`

---

**Maintainer**: Kenan AY — System Architect  
**Last Updated**: 2026-05-03

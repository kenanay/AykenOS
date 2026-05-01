# Design Document: AykenOS Verification Layer

---

**Document Metadata**
- **Author**: Kenan AY
- **Role**: Architectural Steward & Lead Designer
- **Date**: 2026-04-25
- **Version**: 1.0
- **Status**: Production-Ready (Phase-17 Approved)
- **Project**: AykenOS Verification Layer (tools-verification-layer)
- **Phase**: Phase-17 Production Integration
- **Design Principle**: "Verification reads. It does not mutate."

---

## Overview

The AykenOS Verification Layer is a minimal working truth engine that validates system stability through evidence-driven, non-invasive verification. This design implements a manifest-driven gate execution system with deterministic, topologically-sorted execution order.

**Design Principles:**
- **Non-invasive**: Observer pattern - reads system state, never mutates
- **Evidence-driven**: No claim without machine-readable proof
- **Deterministic**: Same inputs → same outputs, always
- **Minimal**: MVP scope for Phase-17, framework features deferred

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Verification Layer                       │
│                                                              │
│  ┌────────────┐      ┌──────────────┐      ┌─────────────┐ │
│  │  Manifest  │─────▶│ Orchestrator │─────▶│   Report    │ │
│  │   (JSON)   │      │ (run_all.sh) │      │   (JSON)    │ │
│  └────────────┘      └──────┬───────┘      └─────────────┘ │
│                             │                               │
│                             ▼                               │
│                    ┌─────────────────┐                      │
│                    │  Gate Executor  │                      │
│                    └────────┬────────┘                      │
│                             │                               │
│              ┌──────────────┼──────────────┐               │
│              ▼              ▼              ▼               │
│         ┌────────┐     ┌────────┐     ┌────────┐          │
│         │ Gate 1 │     │ Gate 2 │     │ Gate N │          │
│         └───┬────┘     └───┬────┘     └───┬────┘          │
│             │              │              │               │
│             ▼              ▼              ▼               │
│       ┌──────────┐   ┌──────────┐   ┌──────────┐         │
│       │ Evidence │   │ Evidence │   │ Evidence │         │
│       └──────────┘   └──────────┘   └──────────┘         │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐ │
│  │              Validator (Python)                       │ │
│  │  • Schema validation                                  │ │
│  │  • Evidence integrity verification                    │ │
│  │  • Marker contract validation                         │ │
│  └──────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Component Architecture

```
tools/verification/
├── run_all.sh              # Orchestrator (bash)
├── manifest.json           # Gate configuration
├── schemas/
│   ├── manifest.schema.json
│   ├── report.schema.json
│   └── evidence.schema.json
├── validators/
│   ├── validate_manifest.py
│   ├── validate_evidence.py
│   └── validate_report.py
├── adapters/
│   ├── make_gate_adapter.sh
│   └── evidence_adapter.py
└── README.md

out/evidence/verification/
├── {run_id}/
│   ├── report.json
│   └── gates/
│       ├── boot_integrity/
│       ├── ring3_runtime/
│       └── bcib_determinism/
└── latest -> {run_id}/     # Symlink
```

## Component Design

### 1. Orchestrator (run_all.sh)

**Responsibility:** Sequential gate execution with dependency resolution

**Algorithm:**
```bash
1. Generate unique run_id (ISO 8601 timestamp)
2. Validate manifest.json against schema
3. Parse manifest and extract gates
4. Build dependency graph
5. Detect circular dependencies (fail if found)
6. Topologically sort gates
7. Filter gates by performance tier
8. For each gate in sorted order:
   a. Check dependencies (skip if dependency failed)
   b. Validate command against allowlist
   c. Set environment variables:
      - AYKEN_RUN_ID=${run_id}
      - AYKEN_EVIDENCE_DIR=out/evidence/verification/${run_id}/gates/${gate_id}/attempt-1/
   d. Write gate status: RUNNING (atomic)
   e. Execute gate command with timeout
   f. Capture raw_exit_code and raw output
   g. Locate evidence file at deterministic path
   h. Validate evidence integrity (including run_id match)
   i. Validate evidence schema
   j. Validate marker contracts (via validator only)
   k. Validate raw_exit_code consistency (exit != 0 AND verdict == PASS → FAIL)
   l. Validate raw_verdict == adapter_verdict
   m. Determine final verdict
   n. Write gate status: PASS/FAIL/ERROR/TIMEOUT (atomic)
   o. Record result
9. Generate report with canonical evidence hash
10. Create latest symlink
11. Exit with appropriate status code
```

**CRITICAL: Evidence Path Enforcement**
- Gates MUST write evidence to AYKEN_EVIDENCE_DIR
- Path is deterministic: out/evidence/verification/${run_id}/gates/${gate_id}/attempt-1/
- Multiple attempts increment attempt number (prevents overwrite)
- Orchestrator reads latest attempt only
- This prevents race conditions and evidence corruption

**CRITICAL: Atomic Gate Status**
- Gate status transitions: NOT_STARTED → RUNNING → PASS/FAIL/ERROR/TIMEOUT
- Status updates are atomic (write to temp file, then rename)
- Mid-run crash leaves status as RUNNING (never partial PASS)
- Orchestrator detects RUNNING status and marks as ERROR on restart

**Key Functions:**
- `validate_manifest()` - Call Python validator
- `parse_manifest()` - Extract gate definitions
- `build_dependency_graph()` - Create adjacency list
- `topological_sort()` - Kahn's algorithm
- `filter_by_tier()` - Apply performance tier filter
- `execute_gate()` - Run gate command with timeout and environment variables
- `validate_evidence()` - Call Python validator (includes marker validation)
- `generate_report()` - Aggregate results to JSON with canonical hash

**No Parallel Execution:** Sequential only in MVP

**CRITICAL: Marker Validation Boundary**
- Orchestrator does NOT validate markers directly
- Marker validation is ONLY done by Python validator
- Orchestrator only reads validator result

### 2. Manifest (manifest.json)

**Schema:**
```json
{
  "version": 1,
  "mode": "verification_layer",
  "default_tier": "standard",
  "gates": [
    {
      "id": "string (snake_case, no phase numbers)",
      "command": "string (make ci-gate-* pattern)",
      "evidence": "string (path relative to project root)",
      "required_verdict": "PASS|FAIL",
      "blocking": "boolean",
      "performance_tier": "fast|standard|heavy",
      "timeout": "number (seconds, optional, default 300)",
      "determinism_level": "artifact|trace|marker|scheduling-independent",
      "allowed_determinism_levels": ["array of allowed levels for this gate"],
      "required_markers": ["array of strings (optional)"],
      "forbidden_markers": ["array of strings (optional)"],
      "depends_on": ["array of gate IDs (optional)"],
      "expected_invariants": ["array of invariant names (optional)"],
      "build_fingerprint_required": "boolean (optional, default false)"
    }
  ]
}
```

**Validation Rules:**
- Gate IDs must be unique
- Gate IDs must not contain "phase" followed by digits
- Commands must match allowlist pattern
- Evidence paths must be under out/evidence/
- depends_on must reference existing gate IDs
- No circular dependencies
- expected_invariants enforces correctness checks (deterministic ≠ correct)
- build_fingerprint_required prevents binary drift false determinism
- allowed_determinism_levels prevents wrong level selection (e.g., bcib_determinism MUST use "artifact")

### 3. Validator (Python)

**Modules:**

#### validate_manifest.py
```python
def validate_manifest(manifest_path: str) -> ValidationResult:
    """Validate manifest against schema and business rules"""
    # Load manifest
    # Validate JSON schema
    # Check gate ID uniqueness
    # Check for phase numbers in IDs
    # Validate command patterns
    # Check dependency references
    # Detect circular dependencies
    return ValidationResult(valid=bool, errors=list)
```

#### validate_evidence.py
```python
def validate_evidence(evidence_path: str, gate_config: dict, run_id: str, command: str) -> ValidationResult:
    """Validate evidence integrity and schema conformance"""
    # Load evidence JSON
    # Validate JSON schema
    # Check file hash integrity
    # Validate run_id matches current run (CRITICAL)
    # Validate command_fingerprint = SHA256(command) (CRITICAL)
    # Validate timestamp (must be from current run)
    # Validate source_gate_id matches expected gate
    # Check marker contracts (required/forbidden) - ONLY place markers validated
    # Validate determinism_level field
    # Validate determinism_level is in allowed_determinism_levels (if specified)
    # Enforce determinism scope constraints:
    #   - artifact → artifact_hash REQUIRED
    #   - trace → trace_hash REQUIRED
    #   - marker → marker_sequence REQUIRED
    # Validate adapter output has no new semantic fields (CRITICAL)
    #   - IF adapter_output_fields ⊄ raw_source_fields → FAIL
    #   - Adapter can only extract, not create data
    # Validate raw_exit_code consistency (CRITICAL)
    #   - IF raw_exit_code != 0 AND verdict == PASS → FAIL
    #   - Prevents adapter from hiding failures
    # Validate raw_verdict == verdict (CRITICAL)
    #   - Adapter cannot change verdict
    #   - Prevents truth distortion
    # Validate expected_invariants if specified in gate config
    #   - Check invariant_checks field in evidence
    #   - IF any invariant FAIL → gate FAIL (deterministic but wrong)
    # Validate build_fingerprint if specified (kernel + toolchain + build_flags hash)
    #   - Prevents binary drift causing false determinism
    return ValidationResult(valid=bool, verdict=str, errors=list)
```

**CRITICAL: Single Source of Truth for Marker Validation**
- Marker validation happens ONLY in this validator
- Orchestrator never validates markers directly
- Prevents double validation and inconsistency

**CRITICAL: Adapter Manipulation Prevention**
- Validator enforces: adapter_output_fields ⊆ raw_source_fields
- Adapter cannot create new data, only extract
- Prevents silent manipulation

**CRITICAL: Deterministic But Wrong Prevention**
- Validates expected_invariants from gate config
- Deterministic output ≠ correct output
- Invariant failures cause gate FAIL

**CRITICAL: Truth Distortion Prevention**
- Validates raw_exit_code consistency
- Validates raw_verdict == adapter_verdict
- Prevents adapter from changing FAIL to PASS

#### validate_report.py
```python
def validate_report(report_path: str) -> ValidationResult:
    """Validate final report against schema"""
    # Load report JSON
    # Validate JSON schema
    # Check mutation field is false
    # Validate verdict counts match gate results
    return ValidationResult(valid=bool, errors=list)
```

**Key Features:**
- JSON schema validation using jsonschema library
- Input sanitization to prevent injection
- Path validation to prevent directory traversal
- No raw log parsing - only structured JSON
- **Adapter output validation:** Ensures adapters do NOT introduce new semantic fields
- **Single source of truth:** Marker validation happens ONLY here

### 4. Evidence Format

**Schema:**
```json
{
  "gate_id": "string",
  "run_id": "string (REQUIRED - must match current verification run)",
  "timestamp": "ISO 8601 string",
  "verdict": "PASS|FAIL|SKIPPED|ERROR|TIMEOUT",
  "determinism_level": "artifact|trace|marker|scheduling-independent",
  "marker_sequence": ["array of markers in execution order"],
  "trace_hash": "string (SHA256 of execution trace)",
  "artifact_hash": "string (SHA256 of produced artifact)",
  "build_fingerprint": "string (SHA256 of kernel + toolchain + build_flags, optional)",
  "raw_exit_code": "number (REQUIRED - actual gate command exit code)",
  "raw_log_hash": "string (SHA256 of raw gate output)",
  "raw_verdict": "string (REQUIRED - verdict from raw gate output)",
  "invariant_checks": [
    {
      "name": "string",
      "result": "PASS|FAIL",
      "details": "string"
    }
  ],
  "integrity": {
    "file_hash": "SHA256 of this evidence file",
    "source_gate_id": "string",
    "command_fingerprint": "SHA256 of command string (REQUIRED)",
    "schema_version": "1.0"
  },
  "details": {
    "command": "string",
    "exit_code": "number",
    "duration_ms": "number",
    "timeout": "boolean"
  }
}
```

**CRITICAL: run_id Coupling**
- Every evidence file MUST include run_id matching the current verification run
- Validator MUST reject evidence with mismatched run_id
- This prevents reading stale evidence from previous runs

**CRITICAL: Command Fingerprint**
- Evidence MUST include command_fingerprint = SHA256(command string)
- Validator MUST verify fingerprint matches expected command
- Prevents wrong script producing valid-looking evidence

**CRITICAL: Determinism Scope Enforcement**
- If determinism_level = "artifact" → artifact_hash REQUIRED
- If determinism_level = "trace" → trace_hash REQUIRED
- If determinism_level = "marker" → marker_sequence REQUIRED
- Validator enforces these constraints

**CRITICAL: Build Fingerprint (Binary Drift Prevention)**
- Optional build_fingerprint = SHA256(kernel binary + toolchain version + build_flags)
- Includes AYKEN_* config flags that affect behavior
- Prevents same gate with different builds producing different results
- Ensures determinism is not false due to binary drift

**CRITICAL: Invariant Checks (Deterministic But Wrong Prevention)**
- invariant_checks validate correctness, not just determinism
- Deterministic output ≠ correct output
- Validator fails gate if any invariant fails

**CRITICAL: Raw Exit Code Enforcement (Truth Distortion Prevention)**
- raw_exit_code REQUIRED - actual gate command exit code
- raw_verdict REQUIRED - verdict from raw gate output before adapter
- Validator enforces: IF raw_exit_code != 0 AND verdict == PASS → FAIL
- Validator enforces: adapter_verdict MUST == raw_verdict
- Prevents adapter from changing FAIL to PASS

### 5. Report Format

**Schema:**
```json
{
  "run_id": "string",
  "timestamp": "ISO 8601 string",
  "status": "PASS|FAIL",
  "mode": "verification_layer",
  "mutation": false,
  "tier": "fast|standard|heavy",
  "gates_checked": "number",
  "gates_passed": "number",
  "gates_failed": "number",
  "gates_skipped": "number",
  "gates_error": "number",
  "gates_timeout": "number",
  "gates": {
    "gate_id": {
      "verdict": "PASS|FAIL|SKIPPED|ERROR|TIMEOUT",
      "blocking": "boolean",
      "determinism_level": "string",
      "evidence_path": "string"
    }
  },
  "determinism_summary": {
    "artifact": "number",
    "trace": "number",
    "marker": "number",
    "scheduling-independent": "number"
  },
  "evidence_hash": "SHA256 canonical hash of all evidence"
}
```

**CRITICAL: Canonical Evidence Hash**
- Hash computation must be deterministic
- Algorithm:
  1. Sort evidence files by gate_id (lexicographic)
  2. For each file: compute SHA256(file_content)
  3. Concatenate hashes in sorted order
  4. Compute final SHA256(concatenated_hashes)
- Same evidence → same hash, always

## Execution Model

### Gate Execution Flow

```
┌─────────────────────────────────────────────────────────────┐
│ 1. Manifest Validation                                       │
│    • Load manifest.json                                      │
│    • Validate schema                                         │
│    • Check business rules                                    │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. Dependency Resolution                                     │
│    • Build dependency graph                                  │
│    • Detect circular dependencies                            │
│    • Topological sort                                        │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. Tier Filtering                                            │
│    • Apply performance tier filter                           │
│    • Select gates for execution                              │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. Sequential Gate Execution (for each gate)                │
│    ┌──────────────────────────────────────────────────────┐ │
│    │ 4.1 Dependency Check                                  │ │
│    │     • Check if dependencies passed                    │ │
│    │     • Skip if dependency failed                       │ │
│    └────────────────┬─────────────────────────────────────┘ │
│                     ▼                                        │
│    ┌──────────────────────────────────────────────────────┐ │
│    │ 4.2 Command Validation                                │ │
│    │     • Validate against allowlist                      │ │
│    │     • Reject if not repository-local                  │ │
│    └────────────────┬─────────────────────────────────────┘ │
│                     ▼                                        │
│    ┌──────────────────────────────────────────────────────┐ │
│    │ 4.3 Gate Execution                                    │ │
│    │     • Execute command with timeout                    │ │
│    │     • Capture exit code                               │ │
│    │     • Record duration                                 │ │
│    └────────────────┬─────────────────────────────────────┘ │
│                     ▼                                        │
│    ┌──────────────────────────────────────────────────────┐ │
│    │ 4.4 Evidence Location                                 │ │
│    │     • Locate evidence file                            │ │
│    │     • Mark ERROR if missing                           │ │
│    └────────────────┬─────────────────────────────────────┘ │
│                     ▼                                        │
│    ┌──────────────────────────────────────────────────────┐ │
│    │ 4.5 Evidence Integrity Verification                   │ │
│    │     • Validate file hash                              │ │
│    │     • Check timestamp (current run)                   │ │
│    │     • Verify source_gate_id                           │ │
│    │     • Mark ERROR if integrity fails                   │ │
│    └────────────────┬─────────────────────────────────────┘ │
│                     ▼                                        │
│    ┌──────────────────────────────────────────────────────┐ │
│    │ 4.6 Evidence Schema Validation                        │ │
│    │     • Validate against evidence.schema.json           │ │
│    │     • Mark ERROR if invalid                           │ │
│    └────────────────┬─────────────────────────────────────┘ │
│                     ▼                                        │
│    ┌──────────────────────────────────────────────────────┐ │
│    │ 4.7 Marker Contract Validation                        │ │
│    │     • Check required_markers present                  │ │
│    │     • Check forbidden_markers absent                  │ │
│    │     • Mark FAIL if contract violated                  │ │
│    └────────────────┬─────────────────────────────────────┘ │
│                     ▼                                        │
│    ┌──────────────────────────────────────────────────────┐ │
│    │ 4.8 Verdict Determination                             │ │
│    │     • Compare evidence verdict to required_verdict    │ │
│    │     • Determine final gate verdict                    │ │
│    │     • Record result                                   │ │
│    └──────────────────────────────────────────────────────┘ │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 5. Report Generation                                         │
│    • Aggregate all gate results                              │
│    • Calculate overall status                                │
│    • Generate determinism summary                            │
│    • Compute evidence hash                                   │
│    • Write report.json                                       │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 6. Symlink Update                                            │
│    • Create/update latest symlink                            │
│    • Point to current run_id                                 │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 7. Exit                                                      │
│    • Exit 0 if PASS or shadow_mode                           │
│    • Exit 1 if FAIL and hard_gate mode                       │
└─────────────────────────────────────────────────────────────┘
```

### Dependency Resolution Algorithm

**Topological Sort (Kahn's Algorithm):**
```
1. Build adjacency list from depends_on fields
2. Calculate in-degree for each gate
3. Initialize queue with gates having in-degree 0
4. While queue not empty:
   a. Dequeue gate
   b. Add to sorted list
   c. For each dependent gate:
      - Decrement in-degree
      - If in-degree becomes 0, enqueue
5. If sorted list size < total gates:
   - Circular dependency detected
   - FAIL with error
6. Return sorted list
```

### Verdict Determination Logic

```
IF gate command failed to execute:
    verdict = ERROR
ELSE IF gate timed out:
    verdict = TIMEOUT
ELSE IF evidence file missing:
    verdict = ERROR
ELSE IF evidence integrity check failed:
    verdict = ERROR
ELSE IF evidence schema validation failed:
    verdict = ERROR
ELSE IF required marker missing:
    verdict = FAIL
ELSE IF forbidden marker present:
    verdict = FAIL
ELSE IF evidence verdict != required_verdict:
    verdict = FAIL
ELSE:
    verdict = PASS
```

### Overall Status Determination

```
overall_status = PASS

FOR each gate:
    IF gate.blocking AND gate.verdict IN [FAIL, ERROR, TIMEOUT]:
        overall_status = FAIL
        
RETURN overall_status
```

## Security Model

### Trust Boundaries

```
┌─────────────────────────────────────────────────────────────┐
│                    Untrusted Zone                            │
│  • Gate commands (repository-local only)                     │
│  • Evidence files (integrity verified)                       │
│  • Manifest (schema validated)                               │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼ Validation
┌─────────────────────────────────────────────────────────────┐
│                    Trusted Zone                              │
│  • Orchestrator (run_all.sh)                                 │
│  • Validators (Python)                                       │
│  • Schemas (JSON Schema)                                     │
└─────────────────────────────────────────────────────────────┘
```

### Security Constraints

1. **Command Execution:**
   - Allowlist: `make ci-gate-*` pattern only
   - No arbitrary commands
   - No user-injected commands
   - Repository-local only

2. **Evidence Integrity:**
   - File hash validation
   - Timestamp validation (current run only)
   - Source gate validation
   - Schema conformance

3. **Input Sanitization:**
   - Path validation (no directory traversal)
   - JSON sanitization (prevent injection)
   - Command validation (allowlist only)

4. **Isolation:**
   - No root privileges required
   - No network access
   - No file access outside project directory
   - Evidence written to isolated directory

## Data Flow

### Manifest → Execution → Report

```
manifest.json
    │
    ├─ validate_manifest.py
    │       │
    │       ▼
    │   [VALID]
    │       │
    ▼       ▼
run_all.sh ──┐
    │        │
    │        ├─ Parse gates
    │        ├─ Build dependency graph
    │        ├─ Topological sort
    │        └─ Filter by tier
    │
    ├─ For each gate:
    │   │
    │   ├─ Execute command ──▶ Gate produces evidence
    │   │                           │
    │   ├─ Locate evidence ◀────────┘
    │   │       │
    │   │       ▼
    │   ├─ validate_evidence.py
    │   │       │
    │   │       ├─ Schema validation
    │   │       ├─ Integrity check
    │   │       └─ Marker validation
    │   │       │
    │   │       ▼
    │   └─ Determine verdict
    │
    ├─ Aggregate results
    │       │
    │       ▼
    ├─ Generate report.json
    │       │
    │       ▼
    └─ validate_report.py
            │
            ▼
        [REPORT VALID]
            │
            ▼
    Create latest symlink
            │
            ▼
        Exit with status
```

## Integration Points

### 1. Makefile Integration

**Target Definition:**
```makefile
.PHONY: verify-system
verify-system:
	@echo "Running AykenOS Verification Layer..."
	@bash tools/verification/run_all.sh --tier=standard --mode=hard_gate
	@echo "Verification complete. Report: out/evidence/verification/latest/report.json"

.PHONY: verify-fast
verify-fast:
	@bash tools/verification/run_all.sh --tier=fast --mode=hard_gate

.PHONY: verify-heavy
verify-heavy:
	@bash tools/verification/run_all.sh --tier=heavy --mode=hard_gate

.PHONY: verify-shadow
verify-shadow:
	@bash tools/verification/run_all.sh --tier=standard --mode=shadow
```

### 2. CI Integration

**GitHub Actions Example:**
```yaml
# Stage 1: Shadow Mode (non-blocking)
- name: Verification Layer (Shadow)
  run: make verify-shadow
  continue-on-error: true

# Stage 2: Hard Gate (blocking)
- name: Verification Layer (Hard Gate)
  run: make verify-system
```

### 3. Existing Gate Integration

**Adapter Pattern:**
```bash
# Existing gate: make ci-gate-boot-observability
# Produces: evidence/boot-observability/report.json

# Adapter: tools/verification/adapters/make_gate_adapter.sh
# Reads: evidence/boot-observability/report.json
# Writes to: ${AYKEN_EVIDENCE_DIR}/report.json
# Extracts (not transforms): marker_sequence, trace_hash, etc.
```

**CRITICAL: Adapter Boundary Enforcement**
- Adapters MUST NOT introduce new semantic fields
- Adapters MUST map directly to raw output
- Adapters are pass-through extractors only
- Validator enforces this constraint

## MVP Deliverables (Phase-17)

### Required Files

1. **tools/verification/manifest.json**
   - At least 3 gates defined
   - boot_integrity, ring3_runtime, bcib_determinism

2. **tools/verification/run_all.sh**
   - Orchestrator implementation
   - Dependency resolution
   - Sequential execution
   - Report generation

3. **tools/verification/validators/validate_evidence.py**
   - Evidence schema validation
   - Integrity verification
   - Marker contract validation

4. **tools/verification/validators/validate_manifest.py**
   - Manifest schema validation
   - Business rule validation

5. **tools/verification/validators/validate_report.py**
   - Report schema validation

6. **tools/verification/schemas/**
   - manifest.schema.json
   - evidence.schema.json
   - report.schema.json

7. **tools/verification/adapters/make_gate_adapter.sh**
   - Minimal adapter for existing gates

8. **tools/verification/README.md**
   - Usage documentation
   - Architecture overview
   - Troubleshooting guide

9. **Makefile target: verify-system**

10. **At least 1 constitutional rule gate**
    - DETERMINISM.GLOBAL or MEMORY.CONTRACT.VIOLATION

### Deferred Post-Phase-17

- Parser/pretty-printer framework
- Advanced archival (keep only basic symlink)
- Large adapter framework
- Parallel execution support
- Advanced reporting features

## Non-Functional Requirements

### Performance

- **Fast tier:** < 30 seconds total
- **Standard tier:** < 5 minutes total
- **Heavy tier:** < 30 minutes total
- **Default timeout:** 300 seconds per gate

### Reliability

- **Deterministic:** Same inputs → same outputs
- **Idempotent:** Multiple runs → same result
- **Atomic:** Run ID prevents race conditions

### Maintainability

- **Simple:** Bash + Python, no complex frameworks
- **Transparent:** Manifest-driven, easy to understand
- **Extensible:** Add gates by editing manifest

### Security

- **Isolated:** No system modification
- **Validated:** All inputs validated
- **Minimal privilege:** No root required

## Constraints and Assumptions

### Constraints

1. **No parallel execution in MVP**
2. **No raw log parsing by verification layer**
3. **No mutation of system under test**
4. **Repository-local commands only**
5. **Evidence must be JSON**
6. **Evidence path is deterministic and enforced**
7. **run_id must match in evidence**
8. **Marker validation only in validator**
9. **Adapters cannot introduce semantic fields**
10. **Evidence hash must be canonical**

### Assumptions

1. **Gates respect AYKEN_EVIDENCE_DIR environment variable**
2. **Gates include AYKEN_RUN_ID in evidence**
3. **Make targets follow ci-gate-* pattern**
4. **Evidence includes required fields**
5. **System has bash and Python 3.7+**
6. **out/evidence/ directory is writable**

## Error Handling

### Error Categories

1. **Manifest Errors:**
   - Invalid JSON
   - Schema validation failure
   - Circular dependencies
   - Invalid gate references

2. **Execution Errors:**
   - Command not found
   - Command execution failure
   - Timeout exceeded
   - Evidence file missing

3. **Validation Errors:**
   - Evidence schema invalid
   - Integrity check failed
   - Marker contract violated
   - Timestamp mismatch

4. **System Errors:**
   - Out of disk space
   - Permission denied
   - Python not available

### Error Reporting

All errors include:
- Error category
- Gate ID (if applicable)
- Detailed message
- Suggested remediation

## Testing Strategy

### Unit Tests

- Manifest validation logic
- Dependency resolution algorithm
- Verdict determination logic
- Evidence validation logic

### Integration Tests

- End-to-end gate execution
- Dependency chain execution
- Timeout handling
- Error scenarios

### Validation Tests

- Schema validation
- Marker contract validation
- Integrity verification

## Critical Design Decisions Summary

### 1. Gate → Evidence Coupling (ENFORCED)
- Every evidence MUST include run_id
- Validator rejects mismatched run_id
- Prevents stale evidence reads

### 2. Evidence Path Determinism (ENFORCED)
- Path: out/evidence/verification/${run_id}/gates/${gate_id}/attempt-N/
- Set via AYKEN_EVIDENCE_DIR environment variable
- Multiple attempts increment N (prevents overwrite)
- Orchestrator reads latest attempt only

### 3. Marker Validation Single Source (ENFORCED)
- Marker validation happens ONLY in Python validator
- Orchestrator never validates markers directly
- Prevents double validation and inconsistency

### 4. Canonical Evidence Hash (ENFORCED)
- Sort evidence files by gate_id
- Compute SHA256 per file
- Concatenate and hash again
- Deterministic: same evidence → same hash

### 5. Adapter Boundary (ENFORCED)
- Adapters MUST NOT introduce new semantic fields
- Adapters are pass-through extractors only
- Validator enforces this constraint

### 6. Command Fingerprint Verification (ENFORCED)
- Evidence MUST include command_fingerprint = SHA256(command)
- Validator verifies fingerprint matches expected command
- Prevents wrong script producing valid-looking evidence

### 7. Determinism Scope Enforcement (ENFORCED)
- artifact level → artifact_hash REQUIRED
- trace level → trace_hash REQUIRED
- marker level → marker_sequence REQUIRED
- Validator enforces based on declared determinism_level

### 8. Evidence Overwrite Prevention (ENFORCED)
- Multiple gate executions use attempt-N directories
- Latest attempt is read
- Prevents race conditions and data corruption

### 9. Adapter Field Subset Enforcement (ENFORCED)
- Validator enforces: adapter_output_fields ⊆ raw_source_fields
- Adapter cannot create new semantic data
- Prevents silent manipulation of evidence

### 10. Binary Drift Detection (ENFORCED)
- Optional build_fingerprint in evidence
- SHA256(kernel binary + toolchain version)
- Prevents false determinism from binary changes

### 11. Invariant-Based Correctness (ENFORCED)
- expected_invariants in manifest
- Validates correctness, not just determinism
- Deterministic but wrong → FAIL

### 12. Raw Exit Code Enforcement (ENFORCED)
- raw_exit_code REQUIRED in evidence
- IF raw_exit_code != 0 AND verdict == PASS → FAIL
- Prevents adapter from hiding failures

### 13. Raw Verdict Preservation (ENFORCED)
- raw_verdict REQUIRED in evidence
- Adapter verdict MUST == raw_verdict
- Prevents truth distortion

### 14. Atomic Gate Status (ENFORCED)
- Gate status: NOT_STARTED → RUNNING → PASS/FAIL/ERROR/TIMEOUT
- Atomic updates (temp file + rename)
- Mid-run crash → status remains RUNNING → marked ERROR on restart

### 15. Determinism Level Validation (ENFORCED)
- allowed_determinism_levels in manifest
- Prevents wrong level selection (e.g., bcib must use "artifact")
- Validator enforces level is in allowed list

## Architectural Principle

**Verification Layer = Orchestrator + Validator**
**Gate = Black Box**

The verification layer does NOT understand gate behavior.
The verification layer ONLY validates evidence.

This separation ensures the system scales.

## Conclusion

This design implements a minimal working truth engine for AykenOS verification. It prioritizes:

1. **Correctness** over features
2. **Simplicity** over sophistication
3. **Evidence** over assumptions
4. **Determinism** over flexibility

With the 5 critical fixes applied, the system is production-ready for Phase-17 and provides a solid foundation for future enhancements.

**This system prevents AykenOS from deceiving itself.**

---

## Document Approval

**Designed by**: Kenan AY - Architectural Steward & Lead Designer  
**Date**: 2026-04-25  
**Status**: Approved for Phase-17 Implementation

**Architectural Principle**: "Verification Layer = Orchestrator + Validator. Gate = Black Box."

**Signature**: This document represents the authoritative design specification for the AykenOS Verification Layer. All implementation must conform to this design.

---

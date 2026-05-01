# Verification Adapters

This directory contains adapters that transform existing gate outputs into verification evidence format.

## Overview

**Zero-Trust Adapter Architecture**: Bash delegates ALL logic to Python.

Adapters are **pass-through extractors only**. They MUST NOT:
- Transform or normalize data semantically
- **Determine verdicts** (verdict determination is validator's responsibility)
- **Generate fake data** (e.g., fake markers when none exist)
- Introduce new semantic fields not present in raw output
- Alter the meaning of any extracted data

### Critical Architecture Principle

**Single Source of Truth: Python**

```
Bash Adapter (make_gate_adapter.sh)
    ↓ (validates inputs only)
Python Helper (evidence_adapter.py)
    ↓ (ALL logic here)
Evidence JSON
```

**Why this architecture?**
- **No fragile JSON construction** (Python handles all JSON)
- **No bash verdict logic** (authority violation)
- **No duplicate logic** (single source of truth)
- **Deterministic** (Python canonical JSON)
- **Testable** (Python unit tests)

**Adapter extracts. Validator validates. Orchestrator decides.**

- **Adapter**: Reads raw output, extracts structured data, computes hashes
- **Validator**: Validates evidence schema, integrity, marker contracts, determinism
- **Orchestrator**: Determines final gate verdict based on validator output

**WRONG**: Adapter determines verdict by checking exit code or grepping for "PASS"
**RIGHT**: Adapter extracts verdict from structured output, or FAILS if no structured verdict found

## Files

### evidence_adapter.py

Python helper module providing utility functions for evidence generation:

**Primary Function: `generate` (CLI)**
```bash
python3 evidence_adapter.py generate \
  --gate-id "boot_integrity" \
  --run-id "2026-04-25T12:00:00Z" \
  --command "make ci-gate-boot" \
  --exit-code 0 \
  --duration-ms 1000 \
  --determinism-level "marker" \
  --raw-output "/path/to/raw/output.json" \
  --output "/path/to/evidence/report.json" \
  [--build-fingerprint-required]
```

This is the **SINGLE SOURCE OF TRUTH** for evidence generation. All logic is here.

**Utility Functions:**
- `compute_sha256(data)` - Compute SHA256 hash of a string
- `compute_file_hash(file_path)` - Compute SHA256 hash of a file
- `compute_command_fingerprint(command)` - Compute command fingerprint
- `compute_canonical_evidence_hash(evidence)` - Compute canonical hash excluding file_hash
- `generate_evidence_structure(...)` - Generate complete evidence JSON structure
- `write_evidence(evidence, output_path)` - Write evidence to file

**Command-line usage:**
```bash
# Compute SHA256 of string
python3 evidence_adapter.py hash "test string"

# Compute SHA256 of file
python3 evidence_adapter.py file_hash /path/to/file

# Compute command fingerprint
python3 evidence_adapter.py command_fingerprint "make ci-gate-test"

# Generate evidence (primary use case)
python3 evidence_adapter.py generate [args...]
```

### make_gate_adapter.sh

**CRITICAL: This is a THIN WRAPPER around evidence_adapter.py**

Bash adapter that validates inputs and delegates ALL logic to Python.

**Architecture:**
```
make_gate_adapter.sh:
  1. Validate arguments
  2. Validate environment (AYKEN_RUN_ID, AYKEN_EVIDENCE_DIR)
  3. Delegate to Python: python3 evidence_adapter.py generate [args]
  4. Exit with Python's exit code
```

**NO logic in bash** - only input validation and delegation.

**Required environment variables:**
- `AYKEN_RUN_ID` - Verification run identifier (ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ)
- `AYKEN_EVIDENCE_DIR` - Directory where evidence will be written

**Usage:**
```bash
AYKEN_RUN_ID="2026-04-25T10:30:00Z" \
AYKEN_EVIDENCE_DIR="out/evidence/verification/2026-04-25T10:30:00Z/gates/test_gate/attempt-1" \
bash make_gate_adapter.sh \
  --gate-id "test_gate" \
  --command "make ci-gate-test" \
  --exit-code 0 \
  --duration-ms 1000 \
  --determinism-level "marker" \
  --raw-output "/path/to/raw/output.txt" \
  [--build-fingerprint-required]
```

**Arguments:**
- `--gate-id` - Gate identifier (required)
- `--command` - Command that was executed (required)
- `--exit-code` - Command exit code (required)
- `--duration-ms` - Execution duration in milliseconds (required)
- `--determinism-level` - Determinism scope: artifact, trace, marker, or scheduling-independent (required)
- `--raw-output` - Path to raw gate output file (required)
- `--build-fingerprint-required` - Include build fingerprint (optional)

**Output:**
- Writes evidence JSON to `${AYKEN_EVIDENCE_DIR}/report.json`
- Evidence conforms to `tools/verification/schemas/evidence.schema.json`
- Exits with 0 on success, 1 on failure

## Determinism Levels

The adapter supports four determinism levels:

1. **artifact** - Requires `artifact_hash` (SHA256 of produced artifact)
2. **trace** - Requires `trace_hash` (SHA256 of execution trace)
3. **marker** - Requires `marker_sequence` (array of markers in execution order)
4. **scheduling-independent** - No additional hash required

## Evidence Format

Generated evidence includes:

- **Core fields**: gate_id, run_id, timestamp, verdict, determinism_level
- **Truth preservation**: raw_exit_code, raw_verdict (must equal verdict)
- **Adapter validation**: raw_source_fields, adapter_output_fields (subset constraint)
- **Integrity**: file_hash, command_fingerprint, source_gate_id, schema_version
- **Details**: command, exit_code, duration_ms, timeout
- **Optional**: marker_sequence, trace_hash, artifact_hash, build_fingerprint

## Critical Constraints

### Verdict Determination (CRITICAL)
**Adapter does NOT determine verdict.** This is the validator's responsibility.

- **Raw output MUST be valid JSON** (adapter fails if not JSON)
- Adapter extracts `raw_verdict` from structured output (REQUIRED)
- If raw output has no verdict field, adapter FAILS with clear error
- Validator determines final verdict based on:
  - Exit code
  - Marker contracts
  - Determinism requirements
  - Invariant checks

**Anti-pattern**: `if exit_code == 0 then verdict = PASS` ← WRONG
**Correct pattern**: Extract verdict from structured output, or FAIL if not found

### No Fake Data Generation (CRITICAL)
**Adapter must NOT generate fake data.**

- **Raw output must be valid JSON** (fail if not JSON)
- If marker-level determinism but no markers found → FAIL (don't generate fake markers)
- If artifact-level determinism but no artifact found → FAIL (don't use log hash as artifact hash)
- If structured output missing → FAIL (don't invent data)

**Anti-pattern**: `marker_sequence: ["PASS"]` when no markers exist ← WRONG
**Correct pattern**: Fail with clear error message

### Dynamic Field Extraction (CRITICAL)
**`raw_source_fields` must reflect actual raw output.**

- If raw output is JSON: extract actual field names
- If raw output is text: list what was actually parsed
- `adapter_output_fields` must be true subset of `raw_source_fields`

**Anti-pattern**: Hardcoded `["exit_code", "output_text", "markers"]` ← WRONG
**Correct pattern**: Dynamically extract from actual raw output

### Canonical Hash Consistency (CRITICAL)
**Hash computation must be identical to validator.**

The adapter uses Python's `json.dumps(sort_keys=True, separators=(',', ':'))` to compute canonical hash, matching the validator's implementation exactly.

**Anti-pattern**: Using external JSON tools with different canonicalization ← May differ from Python
**Correct pattern**: Use same Python code as validator for canonical JSON

### Artifact vs Trace Hash (CRITICAL)
**artifact_hash ≠ trace_hash ≠ log_hash**

- **artifact_hash**: SHA256 of produced binary/image (e.g., kernel.elf)
- **trace_hash**: SHA256 of execution trace (structured event sequence)
- **log_hash**: SHA256 of raw text output

**Anti-pattern**: `artifact_hash = log_hash` ← WRONG (conflates artifact and trace)
**Correct pattern**: Hash the actual artifact file, fail if not found
Every evidence file MUST include `run_id` matching the current verification run. The validator will reject evidence with mismatched run_id.

### Command Fingerprint
Evidence MUST include `command_fingerprint = SHA256(command)`. The validator verifies this matches the expected command.

### Truth Preservation
- `raw_verdict` MUST equal `verdict` (adapter cannot change verdict)
- `raw_exit_code` MUST be preserved
- Validator enforces: IF raw_exit_code != 0 AND verdict == PASS THEN FAIL

### Adapter Output Validation
- `adapter_output_fields` MUST be subset of `raw_source_fields`
- Adapter cannot introduce new semantic fields
- Validator enforces this constraint

## Example: Integrating an Existing Gate

```bash
#!/usr/bin/env bash
# Example: Wrapping an existing gate

set -euo pipefail

# Run existing gate
START_TIME=$(date +%s%3N)
make ci-gate-boot-observability > /tmp/gate-output.txt 2>&1
EXIT_CODE=$?
END_TIME=$(date +%s%3N)
DURATION_MS=$((END_TIME - START_TIME))

# Generate evidence using adapter
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "${SCRIPT_DIR}/make_gate_adapter.sh" \
  --gate-id "boot_integrity" \
  --command "make ci-gate-boot-observability" \
  --exit-code "${EXIT_CODE}" \
  --duration-ms "${DURATION_MS}" \
  --determinism-level "trace" \
  --raw-output "/tmp/gate-output.txt"

exit ${EXIT_CODE}
```

## Requirements

- Python 3.7+
- bash
- Standard Unix utilities (grep, date, etc.)

**Note**: No external JSON tools required - all JSON processing handled by Python.

## See Also

- `tools/verification/schemas/evidence.schema.json` - Evidence schema definition
- `tools/verification/validators/validate_evidence.py` - Evidence validator
- `tools/verification/README.md` - Main verification layer documentation

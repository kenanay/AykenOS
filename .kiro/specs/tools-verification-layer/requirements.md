# Requirements Document: AykenOS Verification Layer

---

**Document Metadata**
- **Author**: Kenan AY
- **Role**: Architectural Steward
- **Date**: 2026-04-25
- **Version**: 1.0
- **Status**: Production-Ready (Phase-17 Approved)
- **Project**: AykenOS Verification Layer (tools-verification-layer)
- **Phase**: Phase-17 Production Integration

---

## Introduction

The AykenOS Verification Layer is a system-wide, non-invasive, evidence-driven verification infrastructure that validates AykenOS stability through measurable and repeatable proofs. This layer operates as an independent observer that transforms phase-based development into system-based assurance, enforcing the principle "No Evidence = No Truth" across all system components.

**Core Principle:** Verification Layer verifies AykenOS; it does not repair, mutate, patch, rewrite, reconfigure, or normalize AykenOS.

The verification layer must validate whether AykenOS is truly stable without interfering with system operation, providing deterministic and repeatable evidence for all stability claims.

## Glossary

- **Verification_Layer**: The independent system that validates AykenOS stability through evidence collection and analysis
- **Gate**: A named verification checkpoint that validates a specific system property (e.g., boot_integrity, ring3_runtime)
- **Evidence**: Machine-readable artifacts (JSON reports, logs, traces) that prove a system property
- **Manifest**: The JSON configuration file that defines all gates, their commands, and evidence requirements
- **Adapter**: A script that transforms existing test outputs into verification evidence format
- **Validator**: A tool that verifies evidence format and content against schemas
- **Verdict**: The result of a gate check (PASS, FAIL, SKIPPED, ERROR, TIMEOUT)
- **Blocking_Gate**: A gate that must pass for the verification to succeed
- **Performance_Tier**: Classification of gates by execution time (fast, standard, heavy)
- **Shadow_Mode**: CI execution mode where failures are logged but do not block builds
- **Hard_Gate**: CI execution mode where failures block builds and deployments
- **Constitutional_Rule**: A NON_OVERRIDABLE or Phase Matrix rule that must be enforced
- **Evidence_Directory**: The isolated output directory (out/evidence/) where all verification artifacts are written
- **Mutation**: Any modification to kernel source files, git state, or system configuration
- **Report**: The aggregated JSON output summarizing all gate results
- **Marker**: A specific string pattern in execution output that indicates a system state or event (e.g., "[[AYKEN_BOOT_OK]]", "[USER_BP]")
- **Forbidden_Marker**: A marker that indicates failure or violation (e.g., "PF!", "PANIC", "BOUNDARY_KILL")
- **Determinism_Level**: The scope at which determinism is guaranteed (artifact, trace, marker, scheduling-independent)
- **Determinism**: Same input + same binary + same environment → same observable output. Observable output MUST be explicitly declared per gate (artifact hash, trace sequence, marker sequence, or scheduling-independent property)

## Requirements

### Requirement 1: Non-Invasive Operation

**User Story:** As a system architect, I want the verification layer to operate without modifying the system, so that verification does not affect the behavior being verified.

#### Acceptance Criteria

1. THE Verification_Layer SHALL NOT modify any kernel source files
2. THE Verification_Layer SHALL NOT modify any git repository state
3. THE Verification_Layer SHALL NOT patch or inject code into the system under test
4. THE Verification_Layer SHALL write all outputs exclusively to the Evidence_Directory
5. WHEN the Verification_Layer executes, THE system behavior SHALL remain identical to execution without verification
6. THE Verification_Layer SHALL operate in read-only mode for all system files outside the Evidence_Directory

### Requirement 2: Evidence-Driven Validation

**User Story:** As a quality engineer, I want every validation to produce verifiable evidence, so that all stability claims are backed by measurable proof.

#### Acceptance Criteria

1. WHEN a Gate executes, THE Verification_Layer SHALL produce machine-readable Evidence in JSON format
2. THE Verification_Layer SHALL reject any Gate that does not produce Evidence
3. FOR ALL Evidence files, THE Validator SHALL verify conformance to the evidence schema
4. THE Evidence SHALL include timestamp, gate identifier, verdict, supporting data, marker_sequence, trace_hash, artifact_hash, invariant_checks, and determinism_level
5. WHEN Evidence is missing or invalid, THE Verification_Layer SHALL report the Gate as FAIL
6. THE Verification_Layer SHALL store all Evidence in the Evidence_Directory with unique identifiers

### Requirement 3: Manifest-Based Configuration

**User Story:** As a developer, I want to configure all gates in a single manifest file, so that the verification system is transparent and maintainable.

#### Acceptance Criteria

1. THE Verification_Layer SHALL read gate definitions from tools/verification/manifest.json
2. WHEN the Manifest is parsed, THE Validator SHALL verify conformance to manifest.schema.json
3. FOR ALL Gates in the Manifest, THE Verification_Layer SHALL validate that required fields (id, command, evidence, required_verdict, blocking, determinism_level) are present
4. THE Manifest SHALL support gate properties: id, command, evidence, required_verdict, blocking, performance_tier, timeout, determinism_level, required_markers, forbidden_markers, depends_on
5. WHEN a Gate id contains phase numbers (phase10, phase16), THE Verification_Layer SHALL reject the Manifest with a descriptive error
6. THE Verification_Layer SHALL use descriptive gate names (boot_integrity, ring3_runtime, syscall_contract, boundary_enforcement, observability_contract, closure_integrity, bcib_determinism)

### Requirement 4: Deterministic Execution

**User Story:** As a CI engineer, I want verification results to be deterministic and repeatable, so that failures can be reliably reproduced and debugged.

#### Acceptance Criteria

1. WHEN the Verification_Layer executes with identical inputs, THE results SHALL be identical
2. THE Verification_Layer SHALL NOT depend on system time for verification logic
3. THE Verification_Layer SHALL NOT use unseeded random number generation
4. WHEN a Gate fails, THE Evidence SHALL contain sufficient information to reproduce the failure
5. THE Verification_Layer SHALL execute Gates in a deterministic, topologically sorted order based on dependencies
6. Parallel execution is FORBIDDEN in MVP
7. THE Report SHALL include a deterministic hash of all Evidence for reproducibility verification

### Requirement 5: Gate Execution and Verdict Determination

**User Story:** As a verification operator, I want the system to execute filtered gates and determine pass/fail status, so that I can assess overall system stability.

#### Acceptance Criteria

1. WHEN the Verification_Layer runs, THE system SHALL execute filtered Gates based on performance tier and dependencies
2. FOR ALL Gates selected for execution, THE Verification_Layer SHALL execute the specified command
3. Gate commands MUST be predefined, repository-local make targets or scripts
4. Arbitrary command execution is FORBIDDEN
5. User-injected commands are FORBIDDEN
6. THE Verification_Layer SHALL validate commands against an allowlist pattern (e.g., 'make ci-gate-*')
7. WHEN a Gate has depends_on specified, THE Verification_Layer SHALL execute dependency gates first
8. WHEN a Gate command completes, THE Verification_Layer SHALL locate the Evidence file at the specified path
9. THE Verification_Layer SHALL compare the Evidence verdict against the required_verdict field
10. WHEN the Evidence verdict matches required_verdict, THE Gate SHALL be marked as PASS
11. WHEN the Evidence verdict does not match required_verdict, THE Gate SHALL be marked as FAIL
12. Gate verdict SHALL support: PASS, FAIL, SKIPPED, ERROR, TIMEOUT
13. PASS indicates gate executed successfully and met all criteria
14. FAIL indicates gate executed but did not meet criteria
15. SKIPPED indicates gate not executed due to dependency failure or tier filtering
16. ERROR indicates gate command failed to execute or produced invalid evidence
17. TIMEOUT indicates gate exceeded timeout limit
18. WHEN a Gate command fails to execute, THE Gate SHALL be marked as ERROR
19. WHEN Evidence is missing or invalid, THE Gate SHALL be marked as ERROR

### Requirement 6: Blocking and Non-Blocking Gates

**User Story:** As a release manager, I want to distinguish between critical gates that block releases and informational gates, so that I can enforce quality thresholds appropriately.

#### Acceptance Criteria

1. THE Manifest SHALL specify a blocking property (true or false) for each Gate
2. WHEN a Blocking_Gate fails, THE Verification_Layer SHALL set the overall status to FAIL
3. WHEN a non-blocking gate fails, THE Verification_Layer SHALL record the failure but not affect overall status
4. THE Report SHALL distinguish between blocking and non-blocking gate failures
5. WHEN all Blocking_Gates pass, THE overall status SHALL be PASS regardless of non-blocking gate results

### Requirement 7: Performance Tier Classification

**User Story:** As a developer, I want gates classified by execution time, so that I can run fast checks frequently and heavy checks less often.

#### Acceptance Criteria

1. THE Manifest SHALL support performance_tier values: fast, standard, heavy
2. THE Verification_Layer SHALL support filtering gates by performance_tier
3. WHEN invoked with --tier=fast, THE Verification_Layer SHALL execute only gates marked as fast
4. WHEN invoked with --tier=standard, THE Verification_Layer SHALL execute fast and standard gates
5. WHEN invoked with --tier=heavy, THE Verification_Layer SHALL execute all gates
6. THE Report SHALL indicate which performance_tier was executed

### Requirement 8: Report Generation and Format

**User Story:** As a CI system, I want a structured JSON report of all verification results, so that I can parse and act on the results programmatically.

#### Acceptance Criteria

1. WHEN verification completes, THE Verification_Layer SHALL generate a Report in JSON format
2. THE Report SHALL include fields: status, mode, mutation, gates_checked, gates_passed, gates_failed, gates_skipped, gates_error, gates_timeout, gates
3. THE Report SHALL set mutation to false to indicate no system modification occurred
4. THE Report SHALL set mode to "verification_layer"
5. FOR ALL Gates, THE Report SHALL include the gate id and verdict (PASS, FAIL, SKIPPED, ERROR, TIMEOUT) in the gates object
6. THE Verification_Layer SHALL write the Report to out/evidence/verification/{run_id}/report.json
7. THE Validator SHALL verify Report conformance to report.schema.json

### Requirement 9: Schema Validation

**User Story:** As a quality engineer, I want all evidence and reports validated against schemas, so that data format is consistent and machine-parseable.

#### Acceptance Criteria

1. THE Verification_Layer SHALL provide manifest.schema.json defining the Manifest format
2. THE Verification_Layer SHALL provide report.schema.json defining the Report format
3. THE Validator SHALL validate the Manifest against manifest.schema.json before execution
4. THE Validator SHALL validate the Report against report.schema.json after generation
5. WHEN schema validation fails, THE Validator SHALL output descriptive error messages indicating the validation failure location
6. THE schemas SHALL be versioned to support evolution

### Requirement 10: Adapter System for Existing Tests

**User Story:** As a test engineer, I want to reuse existing test infrastructure, so that I don't need to rewrite tests to produce verification evidence.

#### Acceptance Criteria

1. THE Verification_Layer SHALL support Adapters that transform existing test outputs into Evidence format
2. THE Adapter SHALL accept existing test output as input
3. THE Adapter SHALL produce Evidence conforming to the evidence schema
4. THE Adapter SHALL preserve the original test verdict (pass/fail) in the Evidence
5. THE Verification_Layer SHALL execute Adapters as part of the gate command pipeline
6. Adapters MAY NOT alter, normalize, or reinterpret test results
7. Adapters MUST be pass-through extractors only
8. Adapters SHALL extract structured data from raw output without semantic transformation

### Requirement 11: Makefile Integration

**User Story:** As a developer, I want to run verification through make, so that it integrates with existing build workflows.

#### Acceptance Criteria

1. THE Verification_Layer SHALL provide a make target: make verify-system
2. WHEN make verify-system executes, THE Verification_Layer SHALL run all gates at the standard performance tier
3. THE make target SHALL output the Report path on completion
4. WHEN verification fails, THE make target SHALL exit with a non-zero status code
5. WHEN verification passes, THE make target SHALL exit with status code 0
6. THE make target SHALL support TIER variable to override performance tier (e.g., make verify-system TIER=fast)

### Requirement 12: CI Integration Stages

**User Story:** As a DevOps engineer, I want to integrate verification into CI gradually, so that I can validate the system before enforcing hard gates.

#### Acceptance Criteria

1. THE Verification_Layer SHALL support shadow_mode where failures are logged but do not block CI
2. WHEN running in shadow_mode, THE Verification_Layer SHALL always exit with status code 0
3. WHEN running in hard_gate mode, THE Verification_Layer SHALL exit with non-zero status on failure
4. THE CI configuration SHALL support continue-on-error flag for shadow_mode
5. THE Report SHALL indicate which mode was used (shadow or hard_gate)

### Requirement 13: Constitutional Rule Enforcement

**User Story:** As a system architect, I want verification to enforce constitutional rules, so that NON_OVERRIDABLE principles are validated system-wide.

#### Acceptance Criteria

1. THE Verification_Layer SHALL include gates that validate NON_OVERRIDABLE rules
2. THE Verification_Layer SHALL include a gate for DETERMINISM.GLOBAL validation
3. THE Verification_Layer SHALL include a gate for MEMORY.CONTRACT.VIOLATION validation
4. THE Verification_Layer SHALL include a gate for KERNEL.SAFETY.CRITICAL validation
5. THE Verification_Layer SHALL include a gate for SECURITY.BOUNDARY.VIOLATION validation
6. WHEN a Constitutional_Rule gate fails, THE gate SHALL be marked as blocking
7. THE Evidence for constitutional rule gates SHALL include specific violation locations and descriptions

### Requirement 14: Directory Structure and Organization

**User Story:** As a maintainer, I want a clear directory structure, so that the verification system is easy to navigate and extend.

#### Acceptance Criteria

1. THE Verification_Layer SHALL organize files under tools/verification/
2. THE directory structure SHALL include: README.md, manifest.json, run_all.sh, schemas/, validators/, reports/, adapters/
3. THE schemas/ directory SHALL contain manifest.schema.json and report.schema.json
4. THE validators/ directory SHALL contain validate_report.py and validate_evidence.py
5. THE adapters/ directory SHALL contain make_gate_adapter.sh and evidence_adapter.py
6. THE reports/ directory SHALL contain a .gitkeep file to preserve the directory in git
7. THE Evidence_Directory (out/evidence/) SHALL be excluded from git via .gitignore

### Requirement 15: Command-Line Interface

**User Story:** As an operator, I want a simple command-line interface to run verification, so that I can execute verification manually or in scripts.

#### Acceptance Criteria

1. THE Verification_Layer SHALL provide a run_all.sh script as the primary entry point
2. THE run_all.sh script SHALL accept --tier argument to filter by performance tier
3. THE run_all.sh script SHALL accept --mode argument to select shadow or hard_gate mode
4. THE run_all.sh script SHALL accept --manifest argument to specify a custom manifest path
5. WHEN run_all.sh completes, THE script SHALL output the Report path
6. THE run_all.sh script SHALL exit with status code 0 on success, non-zero on failure (unless in shadow mode)
7. THE run_all.sh script SHALL display a summary of gates passed and failed

### Requirement 16: Timeout Handling

**User Story:** As a CI engineer, I want gates to timeout if they hang, so that verification does not block indefinitely.

#### Acceptance Criteria

1. THE Manifest SHALL support an optional timeout field (in seconds) for each Gate
2. WHEN a Gate timeout is specified, THE Verification_Layer SHALL terminate the gate command if it exceeds the timeout
3. WHEN a Gate times out, THE Gate SHALL be marked as TIMEOUT. WHEN a blocking gate times out, THE overall status SHALL be FAIL
4. THE Evidence SHALL indicate when a timeout occurred
5. WHEN no timeout is specified, THE Gate SHALL run without time limit
6. THE default timeout SHALL be 300 seconds (5 minutes)

### Requirement 17: Descriptive Gate Naming

**User Story:** As a developer, I want gate names to describe what they verify, so that I can understand the verification scope without reading implementation details.

#### Acceptance Criteria

1. THE Manifest SHALL use descriptive gate names that indicate the system property being verified
2. THE Verification_Layer SHALL support gate names: boot_integrity, ring3_runtime, syscall_contract, boundary_enforcement, observability_contract, closure_integrity, bcib_determinism
3. WHEN a gate name contains "phase" followed by digits, THE Manifest validator SHALL reject the Manifest
4. THE gate names SHALL use snake_case convention
5. THE README.md SHALL document the meaning of each gate name

### Requirement 18: Evidence Archival and History

**User Story:** As a quality analyst, I want historical evidence preserved, so that I can analyze trends and regressions over time.

#### Acceptance Criteria

1. THE Verification_Layer SHALL write Evidence to out/evidence/verification/{run_id}/ using a unique run_id for each verification run
2. THE Verification_Layer SHALL create a symlink from out/evidence/verification/latest/ to the most recent run_id directory
3. Direct writes to the latest/ directory are FORBIDDEN
4. THE run_id SHALL use ISO 8601 format (YYYY-MM-DDTHH-MM-SS) or UUID format
5. WHEN multiple verification runs execute concurrently, THE unique run_id SHALL prevent race conditions

### Requirement 19: Error Reporting and Diagnostics

**User Story:** As a developer, I want clear error messages when verification fails, so that I can quickly identify and fix issues.

#### Acceptance Criteria

1. WHEN a Gate fails, THE Verification_Layer SHALL output the gate id and failure reason
2. WHEN Evidence is missing, THE error message SHALL include the expected Evidence path
3. WHEN a command fails, THE error message SHALL include the command, exit code, and stderr output
4. WHEN schema validation fails, THE error message SHALL include the schema path and validation error details
5. THE Verification_Layer SHALL support a --verbose flag for detailed diagnostic output
6. WHEN --verbose is enabled, THE Verification_Layer SHALL log each gate execution step

### Requirement 20: MVP Scope Boundary

**User Story:** As a project manager, I want the minimum viable verification layer completed before Phase-17, so that subsequent phases have verification infrastructure in place.

#### Acceptance Criteria

1. THE Verification_Layer SHALL deliver tools/verification/manifest.json before Phase-17
2. THE Verification_Layer SHALL deliver tools/verification/run_all.sh before Phase-17
3. THE Verification_Layer SHALL deliver tools/verification/validators/validate_evidence.py before Phase-17
4. THE Verification_Layer SHALL deliver the make verify-system target before Phase-17
5. THE Verification_Layer SHALL produce out/evidence/verification/{run_id}/report.json on execution. The latest/report.json path SHALL resolve through symlink to the latest run_id directory. Direct writes to latest/ are forbidden
6. THE minimum viable system SHALL include at least 3 working gates with evidence generation
7. THE minimum viable system SHALL validate at least one NON_OVERRIDABLE constitutional rule
8. THE Verification_Layer SHALL defer parser and pretty-printer framework until after Phase-17
9. THE Verification_Layer SHALL defer advanced archival features until after Phase-17
10. THE Verification_Layer SHALL defer large adapter framework until after Phase-17

### Requirement 21: Documentation and README

**User Story:** As a new contributor, I want comprehensive documentation, so that I can understand and extend the verification system.

#### Acceptance Criteria

1. THE Verification_Layer SHALL provide a README.md in tools/verification/
2. THE README.md SHALL document the purpose and architecture of the verification layer
3. THE README.md SHALL provide examples of running verification with different tiers and modes
4. THE README.md SHALL document how to add new gates to the Manifest
5. THE README.md SHALL document how to write adapters for existing tests
6. THE README.md SHALL document the evidence format and schema requirements
7. THE README.md SHALL include a troubleshooting section for common issues

### Requirement 22: Security and Isolation

**User Story:** As a security engineer, I want the verification layer to operate with minimal privileges, so that it cannot compromise system security.

#### Acceptance Criteria

1. THE Verification_Layer SHALL NOT require root or elevated privileges
2. THE Verification_Layer SHALL NOT access files outside the project directory
3. THE Verification_Layer SHALL NOT make network requests
4. THE Verification_Layer SHALL NOT execute arbitrary code from Evidence files
5. Gate commands MUST be predefined, repository-local make targets or scripts
6. Arbitrary command execution is FORBIDDEN
7. User-injected commands are FORBIDDEN
8. THE Verification_Layer SHALL validate commands against an allowlist pattern (e.g., 'make ci-gate-*')
9. WHEN parsing Evidence, THE Validator SHALL sanitize inputs to prevent injection attacks
10. THE Verification_Layer SHALL validate all file paths to prevent directory traversal attacks

### Requirement 23: Marker Contract Support

**User Story:** As a verification engineer, I want gates to validate required and forbidden markers in execution output, so that I can verify system state transitions and detect failures.

#### Acceptance Criteria

1. THE Manifest SHALL support required_markers field containing an array of marker strings
2. THE Manifest SHALL support forbidden_markers field containing an array of marker strings
3. WHEN a Gate specifies required_markers, THE Verification_Layer SHALL verify all required markers appear in the execution output or evidence
4. WHEN a Gate specifies forbidden_markers, THE Verification_Layer SHALL verify no forbidden markers appear in the execution output or evidence
5. WHEN a required marker is missing, THE Gate SHALL be marked as FAIL
6. WHEN a forbidden marker is present, THE Gate SHALL be marked as FAIL
7. THE Evidence SHALL include marker_sequence field containing all markers found in execution order
8. THE Verification_Layer SHALL support marker patterns including: boot markers ("[[AYKEN_BOOT_OK]]"), runtime markers ("[USER_BP]", "P10_RING3_USER_CODE"), and failure markers ("PF!", "PANIC", "GP!", "UD", "BOUNDARY_KILL")
9. THE Verification_Layer SHALL NOT parse raw logs directly for marker extraction
10. THE Verification_Layer SHALL validate markers only from structured Evidence produced by gates
11. Marker extraction is the responsibility of the gate or its adapter

### Requirement 24: Determinism Level Declaration

**User Story:** As a verification engineer, I want each gate to declare its determinism scope, so that I can understand what level of reproducibility is guaranteed.

#### Acceptance Criteria

1. THE Manifest SHALL require a determinism_level field for each Gate
2. THE determinism_level field SHALL accept values: artifact, trace, marker, scheduling-independent
3. WHEN determinism_level is "artifact", THE Gate SHALL guarantee identical artifacts (binaries, images) across runs with identical inputs
4. WHEN determinism_level is "trace", THE Gate SHALL guarantee identical execution traces across runs with identical inputs
5. WHEN determinism_level is "marker", THE Gate SHALL guarantee identical marker sequences across runs with identical inputs
6. WHEN determinism_level is "scheduling-independent", THE Gate SHALL guarantee correctness independent of scheduling order
7. THE Evidence SHALL include determinism_level field matching the gate's declared level
8. THE Report SHALL summarize gates by determinism_level
9. Determinism is defined as: Same input + same binary + same environment → same observable output
10. Observable output MUST be explicitly declared per gate (artifact hash, trace sequence, marker sequence, or scheduling-independent property)

### Requirement 25: Gate Dependency Support

**User Story:** As a verification engineer, I want gates to declare dependencies on other gates, so that verification executes in the correct order.

#### Acceptance Criteria

1. THE Manifest SHALL support an optional depends_on field containing an array of gate IDs
2. WHEN a Gate specifies depends_on, THE Verification_Layer SHALL execute all dependency gates before executing the dependent gate
3. WHEN a dependency gate fails and is blocking, THE Verification_Layer SHALL skip the dependent gate
4. WHEN a dependency gate is skipped, THE dependent gate SHALL also be skipped
5. THE Verification_Layer SHALL detect circular dependencies and reject the Manifest with a descriptive error
6. THE Report SHALL indicate which gates were skipped due to dependency failures

### Requirement 26: Evidence Integrity Verification

**User Story:** As a security engineer, I want evidence integrity verified before trusting verification results, so that the verification layer cannot be compromised by malicious or corrupted evidence.

#### Acceptance Criteria

1. THE Verification_Layer SHALL NOT trust evidence blindly
2. FOR ALL Gates where blocking is true, THE Verification_Layer SHALL verify evidence integrity
3. THE Verification_Layer SHALL validate evidence file hash for integrity and validate evidence content against expected schema
4. THE Verification_Layer SHALL validate evidence timestamp is from current run
5. THE Verification_Layer SHALL validate evidence source matches expected gate command
6. THE Verification_Layer SHALL validate evidence schema conformance
7. WHEN evidence fails integrity checks, THE Gate SHALL be marked as ERROR
8. THE Evidence SHALL include integrity metadata: file_hash, timestamp, source_gate_id, schema_version
9. THE Report SHALL indicate which gates failed due to integrity violations


## Appendix: Manifest Example

The following example demonstrates the revised manifest structure with marker contracts, determinism levels, and gate dependencies:

```json
{
  "version": 1,
  "mode": "verification_layer",
  "default_tier": "standard",
  "gates": [
    {
      "id": "boot_integrity",
      "command": "make ci-gate-boot-observability",
      "evidence": "out/evidence/boot-observability/report.json",
      "required_verdict": "PASS",
      "blocking": true,
      "performance_tier": "fast",
      "determinism_level": "trace",
      "required_markers": ["[[AYKEN_BOOT_OK]]"],
      "forbidden_markers": ["UEFI Interactive Shell", "PANIC", "PF!"]
    },
    {
      "id": "ring3_runtime",
      "command": "make ci-gate-ring3-first-retire",
      "evidence": "out/evidence/ring3-runtime/report.json",
      "required_verdict": "PASS",
      "blocking": true,
      "performance_tier": "standard",
      "determinism_level": "marker",
      "required_markers": ["[USER_BP]", "P10_RING3_USER_CODE"],
      "forbidden_markers": ["GP!", "PF!", "UD"],
      "depends_on": ["boot_integrity"]
    },
    {
      "id": "bcib_determinism",
      "command": "make ci-gate-bcib-determinism",
      "evidence": "out/evidence/run-determinism-final-closure/gates/bcib-determinism/report.json",
      "required_verdict": "PASS",
      "required_closure_verdict": "DETERMINISM_PASS",
      "blocking": true,
      "performance_tier": "heavy",
      "determinism_level": "artifact",
      "required_fields": {
        "payload_non_empty": 1,
        "header_only_result": 0,
        "violations_count": 0
      },
      "forbidden_markers": ["PF!", "BOUNDARY_VIOLATION", "fallback_path=1"]
    }
  ]
}
```

This manifest demonstrates:
- **Marker contracts**: Required and forbidden markers for each gate
- **Determinism levels**: artifact, trace, and marker-level determinism
- **Gate dependencies**: ring3_runtime depends on boot_integrity
- **Performance tiers**: fast, standard, and heavy classification
- **Blocking gates**: All gates are blocking in this example

---

## Document Approval

**Prepared by**: Kenan AY - Architectural Steward  
**Date**: 2026-04-25  
**Status**: Approved for Phase-17 Implementation

**Signature**: This document represents the authoritative requirements specification for the AykenOS Verification Layer. All implementation must conform to these requirements.

---

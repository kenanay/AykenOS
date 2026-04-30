# Implementation Plan: AykenOS Verification Layer MVP

---

**Document Metadata**
- **Author**: Kenan AY
- **Role**: Architectural Steward & Implementation Lead
- **Date**: 2026-04-25
- **Version**: 1.0
- **Status**: Implementation-Ready
- **Project**: AykenOS Verification Layer (tools-verification-layer)
- **Phase**: Pre-Phase-17 (MVP)
- **Implementation Approach**: Bottom-up (schemas → validators → orchestrator → integration)

---

## Overview

This implementation plan delivers a minimal working truth engine for AykenOS verification. The system validates system stability through evidence-driven, non-invasive verification with manifest-driven gate execution. Implementation follows a bottom-up approach: schemas → validators → orchestrator → adapters → integration.

**Core Technologies:**
- Bash for orchestration (run_all.sh)
- Python 3.7+ for validation
- JSON for all structured data
- Make for build integration

**Critical Constraints:**
- No parallel execution (sequential only)
- No raw log parsing (structured JSON only)
- No mutation of system under test
- Evidence path enforced via AYKEN_EVIDENCE_DIR
- run_id coupling enforced in all evidence
- Command fingerprint verification required
- Marker validation only in Python validator

## Tasks

- [x] 1. Create directory structure and JSON schemas
  - Create tools/verification/ directory structure
  - Create schemas/ subdirectory for JSON schemas
  - Create validators/ subdirectory for Python validators
  - Create adapters/ subdirectory for gate adapters
  - Write manifest.schema.json defining gate configuration format
  - Write evidence.schema.json defining evidence format with integrity fields
  - Write report.schema.json defining aggregated report format
  - Create .gitkeep in out/evidence/verification/ to preserve directory
  - Add out/evidence/ to .gitignore
  - CRITICAL: Schema enforces determinism requirements (artifact→artifact_hash, trace→trace_hash, marker→marker_sequence via allOf)
  - CRITICAL: Schema enforces adapter manipulation prevention (raw_source_fields, adapter_output_fields both required)
  - CRITICAL: Schema enforces run_id format (ISO 8601 pattern: YYYY-MM-DDTHH:MM:SSZ)
  - CRITICAL: Schema enforces truth preservation (raw_exit_code, raw_verdict required)
  - CRITICAL: Schema documents 8 validator requirements in description (canonical hash excluding file_hash field, command fingerprint, subset checking, exit code enforcement, determinism_level match, build_fingerprint enforcement, required_verdict=FAIL handling)
  - CRITICAL: Report schema includes evidence_files array for hash chain verification
  - CRITICAL: Gate ID phase number rejection (pattern: not phase[0-9])
  - CRITICAL: marker_sequence minItems=1 for marker-level determinism
  - CRITICAL: additionalProperties=false at root level (only true in details)
  - CRITICAL: file_hash computed as canonical JSON excluding integrity.file_hash field (prevents circular dependency)
  - _Requirements: 2.2, 2.4, 9.1, 9.2, 14.1, 14.2, 14.3, 14.4, 14.7_
  - Create .gitkeep in out/evidence/verification/ to preserve directory
  - Add out/evidence/ to .gitignore
  - _Requirements: 2.2, 2.4, 9.1, 9.2, 14.1, 14.2, 14.3, 14.4, 14.7_

- [x] 2. Implement Python validators
  - [x] 2.1 Implement validate_manifest.py
    - Write manifest validator with JSON schema validation
    - Implement gate ID uniqueness check
    - Implement phase number detection (reject "phase" + digits in gate IDs)
    - Implement command allowlist validation (make ci-gate-* pattern)
    - Implement dependency reference validation
    - Implement circular dependency detection using graph traversal
    - Add descriptive error messages for all validation failures
    - _Requirements: 3.2, 3.3, 3.5, 5.6, 9.4, 17.3, 25.5_

  - [x] 2.2 Implement validate_evidence.py
    - Write evidence validator with JSON schema validation
    - Implement file hash integrity verification (canonical_evidence_hash = sha256(JSON excluding integrity.file_hash field) - CRITICAL: file cannot contain its own hash)
    - Implement run_id matching validation (CRITICAL: reject mismatched run_id)
    - Implement command_fingerprint verification (sha256(details.command) == integrity.command_fingerprint)
    - Implement timestamp validation (must be from current run)
    - Implement source_gate_id validation
    - Implement marker contract validation (required_markers and forbidden_markers)
    - Implement determinism scope enforcement (artifact→artifact_hash, trace→trace_hash, marker→marker_sequence)
    - Implement determinism_level match validation (CRITICAL: evidence.determinism_level MUST equal manifest.determinism_level)
    - Implement adapter output validation (CRITICAL: adapter_output_fields ⊆ raw_source_fields, no new semantic fields)
    - Implement raw_exit_code enforcement (CRITICAL: IF raw_exit_code != 0 AND verdict == PASS THEN FAIL)
    - Implement raw_verdict preservation (CRITICAL: raw_verdict MUST equal verdict, adapter cannot change verdict)
    - Implement expected_invariants validation (if specified in gate config, check invariant_checks in evidence)
    - Implement build_fingerprint validation (CRITICAL: IF manifest.build_fingerprint_required THEN evidence.build_fingerprint MUST exist)
    - Implement required_verdict=FAIL handling (CRITICAL: IF manifest.required_verdict == FAIL THEN gate passes when evidence.verdict == FAIL)
    - Add input sanitization to prevent injection attacks
    - Add path validation to prevent directory traversal
    - Return ValidationResult with verdict and error details
    - _Requirements: 2.3, 2.4, 2.5, 9.5, 22.9, 22.10, 23.3, 23.4, 23.5, 23.6, 24.7, 26.3, 26.4, 26.5, 26.6_

  - [x] 2.3 Implement validate_report.py
    - Write report validator with JSON schema validation
    - Implement mutation field validation (must be false)
    - Implement verdict count validation (match gate results)
    - Add descriptive error messages
    - _Requirements: 8.3, 8.4, 9.7_

- [x] 3. Implement bash orchestrator (run_all.sh)
  - [x] 3.1 Implement core orchestrator structure
    - Write run_all.sh script with command-line argument parsing
    - Implement --tier argument (fast, standard, heavy)
    - Implement --mode argument (shadow, hard_gate)
    - Implement --manifest argument for custom manifest path
    - Implement --verbose flag for diagnostic output
    - Generate unique run_id using ISO 8601 timestamp format
    - Set up evidence directory structure: out/evidence/verification/${run_id}/gates/
    - _Requirements: 4.1, 7.3, 7.4, 7.5, 12.1, 12.2, 12.3, 15.1, 15.2, 15.3, 15.4, 18.1, 18.4, 19.6_

  - [x] 3.2 Implement manifest validation and parsing
    - Call validate_manifest.py before execution
    - Parse manifest.json and extract gate definitions
    - Exit with descriptive error if manifest validation fails
    - _Requirements: 3.1, 3.2, 9.4_

  - [x] 3.3 Implement dependency resolution
    - Build dependency graph from depends_on fields
    - Implement topological sort using Kahn's algorithm
    - Detect circular dependencies (fail if found)
    - _Requirements: 4.5, 25.2, 25.5_

  - [x] 3.4 Implement tier filtering
    - Filter gates by performance_tier based on --tier argument
    - fast tier: execute only "fast" gates
    - standard tier: execute "fast" and "standard" gates
    - heavy tier: execute all gates
    - _Requirements: 7.2, 7.3, 7.4, 7.5_

  - [x] 3.5 Implement sequential gate execution
    - For each gate in topologically sorted order:
    - Check dependencies (skip if dependency not PASS)
    - Validate command against allowlist (make ci-gate-* pattern)
    - Validate command has no shell metacharacters (defense in depth)
    - Find or create attempt directory atomically (race-safe)
    - Set environment variables: AYKEN_RUN_ID, AYKEN_EVIDENCE_DIR
    - Execute gate command with timeout (default 300 seconds) using array execution
    - Capture exit code and duration
    - Handle timeout by terminating command and marking as TIMEOUT
    - Write atomic gate status at each transition
    - _Requirements: 4.2, 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 16.2, 16.3, 16.6, 25.2, 25.3_

  - [x] 3.6 Implement evidence validation and verdict determination
    - Locate evidence file at deterministic path (AYKEN_EVIDENCE_DIR/report.json)
    - No legacy fallback (enforces run_id isolation)
    - Create temporary gate config JSON for validator
    - Call validate_evidence.py with run_id and command for verification
    - Parse JSON output from validator (detect parse failures as ERROR)
    - Extract gate_pass, verdict, and valid fields from validator JSON
    - Enforce gate PASS requirements: (exit_code == 0) AND (gate_pass == true)
    - Mark ERROR if evidence missing, validation fails, or command failed
    - Mark FAIL if validator gate_pass == false
    - Mark SKIPPED if dependency not PASS
    - Determine final gate verdict (PASS, FAIL, SKIPPED, ERROR, TIMEOUT)
    - Record result with blocking status
    - Write atomic gate status
    - _Requirements: 2.1, 2.2, 2.5, 5.8, 5.9, 5.10, 5.11, 5.18, 5.19, 26.2, 26.7_

  - [x] 3.7 Implement report generation
    - Aggregate all gate results
    - Calculate overall status (PASS if all blocking gates pass)
    - Count gates by verdict (passed, failed, skipped, error, timeout)
    - Generate determinism summary (count by determinism_level)
    - For each gate, find latest attempt directory and build evidence path
    - Collect evidence file paths relative to report directory (gates/.../attempt-N/report.json)
    - Compute canonical evidence hash (canonical JSON excluding integrity.file_hash, sorted by gate_id)
    - Build gates object with actual attempt-based evidence paths
    - Write report.json to out/evidence/verification/${run_id}/
    - Call validate_report.py to verify report
    - _Requirements: 4.3, 6.2, 6.3, 6.5, 8.1, 8.2, 8.6, 24.8_

  - [x] 3.8 Implement symlink management and exit handling
    - Create/update latest symlink pointing to current run_id
    - Output report path to stdout
    - Display summary of gates passed and failed
    - Exit with status 0 if PASS or shadow_mode
    - Exit with status 1 if FAIL and hard_gate mode
    - _Requirements: 11.3, 11.4, 11.5, 12.2, 12.3, 15.5, 15.6, 15.7, 18.2_

- [x] 4. Checkpoint - Verify core infrastructure
  - Ensure all tests pass, ask the user if questions arise.

- [x] 5. Implement minimal adapter for existing gates
  - [x] 5.1 Create make_gate_adapter.sh
    - Write bash adapter that reads existing test output
    - Extract verdict from existing format
    - Read AYKEN_RUN_ID and AYKEN_EVIDENCE_DIR from environment
    - Generate evidence JSON with all required fields
    - Include run_id from environment (CRITICAL)
    - Compute and include command_fingerprint (SHA256 of command)
    - Include integrity metadata (file_hash, source_gate_id, schema_version)
    - Include build_fingerprint if required (SHA256 of kernel + toolchain)
    - Write evidence to AYKEN_EVIDENCE_DIR/attempt-1/
    - Ensure adapter is pass-through only (CRITICAL: no semantic transformation, only extraction)
    - Ensure adapter_output_fields ⊆ raw_source_fields (validator will enforce)
    - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.6, 10.7, 10.8_

  - [x] 5.2 Create evidence_adapter.py (Python helper)
    - Write Python helper for evidence generation
    - Implement functions for computing file hashes
    - Implement function for computing command fingerprint
    - Implement function for generating evidence JSON structure
    - Ensure no semantic transformation of input data
    - _Requirements: 10.1, 10.3, 10.6, 10.7_

- [x] 6. Create manifest.json with 3 MVP gates
  - [x] 6.1 Define boot_integrity gate
    - Set id: "boot_integrity"
    - Set command: "make ci-gate-boot-observability"
    - Set evidence path
    - Set required_verdict: "PASS"
    - Set blocking: true
    - Set performance_tier: "fast"
    - Set determinism_level: "trace"
    - Set required_markers: ["[[AYKEN_BOOT_OK]]"]
    - Set forbidden_markers: ["UEFI Interactive Shell", "PANIC", "PF!"]
    - _Requirements: 3.1, 3.4, 17.2, 23.2, 23.3, 24.1, 24.2_

  - [x] 6.2 Define ring3_runtime gate
    - Set id: "ring3_runtime"
    - Set command: "make ci-gate-ring3-first-retire"
    - Set evidence path
    - Set required_verdict: "PASS"
    - Set blocking: true
    - Set performance_tier: "standard"
    - Set determinism_level: "marker"
    - Set required_markers: ["[USER_BP]", "P10_RING3_USER_CODE"]
    - Set forbidden_markers: ["GP!", "PF!", "UD"]
    - Set depends_on: ["boot_integrity"]
    - _Requirements: 3.1, 3.4, 17.2, 23.2, 23.3, 24.1, 24.2, 25.1, 25.2_

  - [x] 6.3 Define bcib_determinism gate
    - Set id: "bcib_determinism"
    - Set command: "make ci-gate-bcib-determinism"
    - Set evidence path
    - Set required_verdict: "PASS"
    - Set blocking: true
    - Set performance_tier: "heavy"
    - Set determinism_level: "artifact"
    - Set timeout: 600 (10 minutes for heavy gate)
    - Set forbidden_markers: ["PF!", "BOUNDARY_VIOLATION"]
    - _Requirements: 3.1, 3.4, 16.1, 17.2, 23.2, 23.3, 24.1, 24.2_

  - [x] 6.4 Add constitutional rule gate (DETERMINISM.GLOBAL or MEMORY.CONTRACT.VIOLATION)
    - Choose one NON_OVERRIDABLE rule to validate
    - Set blocking: true (constitutional rules are always blocking)
    - Set appropriate markers for violation detection
    - Document which constitutional rule is being validated
    - _Requirements: 13.1, 13.2, 13.3, 13.6_

- [x] 7. Integrate with Makefile
  - [x] 7.1 Add verify-system target
    - Create make verify-system target
    - Call bash tools/verification/run_all.sh --tier=standard --mode=hard_gate
    - Output report path on completion
    - Support TIER variable override (make verify-system TIER=fast)
    - _Requirements: 11.1, 11.2, 11.3, 11.6_

  - [x] 7.2 Add convenience targets
    - Create make verify-fast target (tier=fast)
    - Create make verify-heavy target (tier=heavy)
    - Create make verify-shadow target (shadow mode)
    - _Requirements: 11.1, 12.1_

- [x] 8. Create documentation
  - [x] 8.1 Write tools/verification/README.md
    - Document purpose and architecture overview
    - Provide usage examples for different tiers and modes
    - Document how to add new gates to manifest
    - Document how to write adapters for existing tests
    - Document evidence format and schema requirements
    - Include troubleshooting section for common issues
    - Document marker contract system
    - Document determinism levels
    - _Requirements: 21.1, 21.2, 21.3, 21.4, 21.5, 21.6, 21.7_

  - [x] 8.2 Add inline documentation
    - Add comments to run_all.sh explaining key functions
    - Add docstrings to Python validators
    - Add schema descriptions in JSON schemas
    - _Requirements: 21.2_

- [x] 9. Shadow mode testing and validation
  - [x] 9.1 Test shadow mode execution
    - Run make verify-shadow
    - Verify all gates execute
    - Verify failures are logged but don't block (exit 0)
    - Verify report.json is generated correctly
    - Verify latest symlink is created
    - _Requirements: 12.1, 12.2, 18.2_

  - [x] 9.2 Test tier filtering
    - Run make verify-fast and verify only fast gates execute
    - Run make verify-system and verify fast+standard gates execute
    - Run make verify-heavy and verify all gates execute
    - _Requirements: 7.3, 7.4, 7.5_

  - [x] 9.3 Test dependency resolution
    - Verify ring3_runtime executes after boot_integrity
    - Verify dependent gate skips if dependency fails
    - Test circular dependency detection with invalid manifest
    - _Requirements: 25.2, 25.3, 25.5_

  - [x] 9.4 Test error handling
    - Test missing evidence file scenario
    - Test invalid evidence schema scenario
    - Test command timeout scenario
    - Test marker contract violation scenario
    - Verify descriptive error messages for each case
    - _Requirements: 19.1, 19.2, 19.3, 19.4_

- [x] 10. Checkpoint - Verify shadow mode works correctly
  - Ensure all tests pass, ask the user if questions arise.

- [x] 11. Transition to hard gate mode
  - [x] 11.1 Update CI configuration for hard gate
    - Add verification step to CI pipeline
    - Configure make verify-system as blocking step
    - Ensure failures block builds and deployments
    - _Requirements: 12.3_

  - [x] 11.2 Validate hard gate behavior
    - Trigger intentional gate failure
    - Verify CI build fails (exit 1)
    - Verify report indicates failure
    - Verify blocking gate failures affect overall status
    - _Requirements: 6.2, 11.4, 12.3_

  - [x] 11.3 Document transition process
    - Update README with hard gate activation instructions
    - Document rollback procedure if issues arise
    - Document monitoring and alerting recommendations
    - _Requirements: 21.2_

- [x] 12. Final validation and cleanup
  - [x] 12.1 Verify all MVP deliverables
    - Confirm tools/verification/manifest.json exists with 3+ gates
    - Confirm tools/verification/run_all.sh exists and is executable
    - Confirm tools/verification/validators/validate_evidence.py exists
    - Confirm tools/verification/schemas/ contains all 3 schemas
    - Confirm make verify-system target works
    - Confirm at least 1 constitutional rule gate is defined
    - _Requirements: 20.1, 20.2, 20.3, 20.4, 20.5, 20.6, 20.7_

  - [x] 12.2 Run full verification suite
    - Execute make verify-system with all gates
    - Verify report.json is valid and complete
    - Verify evidence integrity for all gates
    - Verify canonical evidence hash is deterministic
    - _Requirements: 4.1, 4.3, 8.1, 26.2_

  - [x] 12.3 Security validation
    - Verify no root privileges required
    - Verify no file access outside project directory
    - Verify command allowlist enforcement
    - Verify input sanitization in validators
    - Verify path validation prevents directory traversal
    - _Requirements: 22.1, 22.2, 22.8, 22.9, 22.10_

  - [x] 12.4 Clean up temporary files and finalize
    - Remove any test artifacts
    - Verify .gitignore excludes out/evidence/
    - Ensure all documentation is complete
    - _Requirements: 14.7_

- [x] 13. Final checkpoint - Complete MVP delivery
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- All tasks reference specific requirements for traceability
- Implementation follows bottom-up approach: schemas → validators → orchestrator → adapters → integration
- Shadow mode testing precedes hard gate transition for safe rollout
- Checkpoints ensure incremental validation at key milestones
- Security constraints are validated throughout implementation
- Constitutional rule enforcement is integrated from the start
- No parallel execution in MVP (deferred post-Phase-17)
- No advanced archival features in MVP (deferred post-Phase-17)
- Parser/pretty-printer framework deferred post-Phase-17

---

## Document Approval

**Implementation Plan by**: Kenan AY - Architectural Steward & Implementation Lead  
**Date**: 2026-04-25  
**Status**: ✅ **COMPLETED** - MVP Successfully Delivered

**Implementation Constraint**: All tasks must maintain consistency with requirements.md and design.md. Cross-reference both documents during implementation.

**Completion Summary**: 
- ✅ All 13 tasks completed successfully
- ✅ Verification Layer MVP fully operational
- ✅ Evidence chain integrity verified
- ✅ Trust layer established with canonical hash validation
- ✅ Fail-closed behavior confirmed
- ✅ Constitutional rule enforcement active
- ✅ Full system verification: `make verify-system` → PASS

**Final Verification Results**:
- Fast tier: `make verify-fast` → 1 gate → PASS
- Standard tier: `make verify-system` → 3 gates → PASS  
- Evidence files: ✅ Non-empty, properly linked
- Hash chain: ✅ Canonical, deterministic
- Dependency chain: ✅ boot_integrity → ring3_runtime → determinism_global_enforcement

**Signature**: This document represents the completed implementation of the AykenOS Verification Layer MVP. The system is production-ready for Phase 17+ integration.

---

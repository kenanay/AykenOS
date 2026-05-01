#!/usr/bin/env python3
"""
Evidence Validator for AykenOS Verification Layer

Validates evidence files against schema and integrity requirements:
- JSON schema conformance
- File hash integrity verification (canonical hash excluding integrity.file_hash)
- run_id matching validation
- Command fingerprint verification
- Timestamp validation
- Source gate ID validation
- Marker contract validation
- Determinism scope enforcement
- Adapter output validation (no new semantic fields)
- Raw exit code enforcement
- Raw verdict preservation
- Expected invariants validation
- Build fingerprint validation

Author: Kenan AY - Architectural Steward
Date: 2026-04-25
"""

import json
import sys
import hashlib
import re
from pathlib import Path
from typing import Dict, List, Optional, Set
from dataclasses import dataclass
from datetime import datetime, timedelta

try:
    import jsonschema
    from jsonschema import validate, ValidationError
except ImportError:
    print("ERROR: jsonschema library not found. Install with: pip3 install jsonschema", file=sys.stderr)
    sys.exit(1)


@dataclass
class ValidationResult:
    """Result of evidence validation"""
    valid: bool
    verdict: Optional[str]
    gate_pass: bool = False  # CRITICAL: Whether gate passes (separate from evidence verdict)
    errors: List[str] = None
    warnings: List[str] = None
    
    def __post_init__(self):
        if self.errors is None:
            self.errors = []
        if self.warnings is None:
            self.warnings = []


class EvidenceValidator:
    """Validates verification evidence against schema and integrity requirements"""
    
    def __init__(self, schema_path: Optional[Path] = None):
        """
        Initialize validator with schema
        
        Args:
            schema_path: Path to evidence.schema.json (auto-detected if None)
        """
        if schema_path is None:
            # Auto-detect schema path relative to this script
            validator_dir = Path(__file__).parent
            schema_path = validator_dir.parent / "schemas" / "evidence.schema.json"
        
        self.schema_path = Path(schema_path)
        self.schema = self._load_schema()
    
    def _load_schema(self) -> dict:
        """Load JSON schema from file"""
        try:
            with open(self.schema_path, 'r') as f:
                return json.load(f)
        except FileNotFoundError:
            raise FileNotFoundError(f"Schema file not found: {self.schema_path}")
        except json.JSONDecodeError as e:
            raise ValueError(f"Invalid JSON in schema file: {e}")
    
    def validate_evidence(
        self,
        evidence_path: Path,
        gate_config: dict,
        run_id: str,
        command: str
    ) -> ValidationResult:
        """
        Validate evidence against schema and integrity requirements
        
        Args:
            evidence_path: Path to evidence JSON file
            gate_config: Gate configuration from manifest
            run_id: Current verification run ID
            command: Expected command string
            
        Returns:
            ValidationResult with validation status, verdict, and errors
        """
        errors = []
        warnings = []
        
        # Load evidence
        try:
            with open(evidence_path, 'r') as f:
                evidence = json.load(f)
        except FileNotFoundError:
            return ValidationResult(
                valid=False,
                verdict="ERROR",
                gate_pass=False,
                errors=[f"Evidence file not found: {evidence_path}"]
            )
        except json.JSONDecodeError as e:
            return ValidationResult(
                valid=False,
                verdict="ERROR",
                gate_pass=False,
                errors=[f"Invalid JSON in evidence: {e}"]
            )
        
        # Validate JSON schema
        try:
            validate(instance=evidence, schema=self.schema)
        except ValidationError as e:
            errors.append(f"Schema validation failed: {e.message}")
            errors.append(f"  Path: {' -> '.join(str(p) for p in e.path)}")
            return ValidationResult(valid=False, verdict="ERROR", gate_pass=False, errors=errors)
        
        # CRITICAL: Validate run_id matches current run
        evidence_run_id = evidence.get("run_id")
        if evidence_run_id != run_id:
            errors.append(
                f"run_id mismatch: evidence has '{evidence_run_id}' "
                f"but current run is '{run_id}'. "
                f"This prevents reading stale evidence from previous runs. "
                f"CRITICAL: All evidence in a verification run MUST share the same run_id."
            )
        
        # Validate run_id format (ISO 8601)
        run_id_pattern = re.compile(r'^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$')
        if not run_id_pattern.match(evidence_run_id):
            errors.append(
                f"Invalid run_id format: '{evidence_run_id}'. "
                f"Must be ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ"
            )
        
        # CRITICAL: Validate command fingerprint
        command_fingerprint_errors = self._validate_command_fingerprint(evidence, command)
        errors.extend(command_fingerprint_errors)
        
        # Validate file hash integrity
        file_hash_errors = self._validate_file_hash(evidence_path, evidence)
        errors.extend(file_hash_errors)
        
        # Validate timestamp (must be from current run)
        timestamp_errors = self._validate_timestamp(evidence, run_id)
        errors.extend(timestamp_errors)
        
        # Validate source_gate_id
        gate_id = gate_config.get("id")
        source_gate_id = evidence.get("integrity", {}).get("source_gate_id")
        if source_gate_id != gate_id:
            errors.append(
                f"source_gate_id mismatch: evidence has '{source_gate_id}' "
                f"but expected '{gate_id}'"
            )
        
        # Validate marker contracts
        marker_errors = self._validate_marker_contracts(evidence, gate_config)
        errors.extend(marker_errors)
        
        # Validate determinism scope enforcement
        determinism_errors = self._validate_determinism_scope(evidence, gate_config)
        errors.extend(determinism_errors)
        
        # Note: _validate_determinism_scope may add warnings, but we need to capture them
        # For now, determinism warnings are added to errors list if critical
        
        # CRITICAL: Validate adapter output (no new semantic fields)
        adapter_errors = self._validate_adapter_output(evidence)
        errors.extend(adapter_errors)
        
        # CRITICAL: Validate raw_exit_code consistency
        exit_code_errors = self._validate_raw_exit_code(evidence)
        errors.extend(exit_code_errors)
        
        # CRITICAL: Validate raw_verdict preservation
        verdict_errors = self._validate_raw_verdict(evidence)
        errors.extend(verdict_errors)
        
        # Validate expected invariants
        invariant_errors = self._validate_expected_invariants(evidence, gate_config)
        errors.extend(invariant_errors)
        
        # Validate build fingerprint if required
        build_fp_errors = self._validate_build_fingerprint(evidence, gate_config)
        errors.extend(build_fp_errors)
        
        # CRITICAL: Validate required_verdict enforcement
        required_verdict_errors = self._validate_required_verdict(evidence, gate_config)
        errors.extend(required_verdict_errors)
        
        # Validate path to prevent directory traversal
        path_errors = self._validate_path_safety(evidence_path)
        errors.extend(path_errors)
        
        # Determine final verdict and gate_pass
        verdict = evidence.get("verdict", "ERROR")
        
        # CRITICAL: gate_pass semantics (separate from validation errors)
        # Gate passes if:
        # 1. No validation errors (evidence is valid)
        # 2. Evidence verdict matches required_verdict EXACTLY
        # 
        # Edge cases:
        # - SKIPPED: gate did not execute → gate_pass=False (explicit fail)
        # - TIMEOUT: gate exceeded time limit → gate_pass=False (explicit fail)
        # - ERROR: gate execution failed → gate_pass=False (explicit fail)
        # - Validation errors: evidence invalid → gate_pass=False, verdict=ERROR
        
        gate_pass = False
        required_verdict = gate_config.get("required_verdict", "PASS")
        
        if len(errors) == 0:
            # No validation errors → check verdict semantics
            if required_verdict == "PASS":
                # Normal case: gate expects PASS
                gate_pass = (verdict == "PASS")
            elif required_verdict == "FAIL":
                # Negative test: gate expects FAIL
                gate_pass = (verdict == "FAIL")
            else:
                # Invalid required_verdict (should be caught earlier)
                gate_pass = False
                errors.append(f"Invalid required_verdict: '{required_verdict}'")
            
            # CRITICAL: SKIPPED/TIMEOUT/ERROR are explicit failures
            # Even if no validation errors, these verdicts mean gate did not complete
            if verdict in ["SKIPPED", "TIMEOUT", "ERROR"]:
                gate_pass = False
        else:
            # Validation errors → gate cannot pass
            verdict = "ERROR"
            gate_pass = False
        
        return ValidationResult(
            valid=len(errors) == 0,
            verdict=verdict,
            gate_pass=gate_pass,
            errors=errors,
            warnings=warnings
        )
    
    def _validate_command_fingerprint(self, evidence: dict, command: str) -> List[str]:
        """
        CRITICAL: Validate command_fingerprint = SHA256(command)
        Prevents wrong script producing valid-looking evidence
        """
        errors = []
        
        expected_fingerprint = hashlib.sha256(command.encode('utf-8')).hexdigest()
        actual_fingerprint = evidence.get("integrity", {}).get("command_fingerprint")
        
        if actual_fingerprint != expected_fingerprint:
            errors.append(
                f"command_fingerprint mismatch: "
                f"expected '{expected_fingerprint}' (SHA256 of '{command}') "
                f"but evidence has '{actual_fingerprint}'. "
                f"This prevents wrong script producing valid-looking evidence."
            )
        
        return errors
    
    def _validate_file_hash(self, evidence_path: Path, evidence: dict) -> List[str]:
        """
        Validate file hash integrity
        CRITICAL: Canonical hash computed excluding integrity.file_hash field
        """
        errors = []
        
        # Compute canonical hash (excluding integrity.file_hash field)
        evidence_copy = json.loads(json.dumps(evidence))  # Deep copy
        if "integrity" in evidence_copy and "file_hash" in evidence_copy["integrity"]:
            del evidence_copy["integrity"]["file_hash"]
        
        # Compute SHA256 of canonical JSON (sorted keys for determinism)
        canonical_json = json.dumps(evidence_copy, sort_keys=True, separators=(',', ':'))
        computed_hash = hashlib.sha256(canonical_json.encode('utf-8')).hexdigest()
        
        declared_hash = evidence.get("integrity", {}).get("file_hash")
        
        if declared_hash != computed_hash:
            errors.append(
                f"file_hash integrity check failed: "
                f"declared '{declared_hash}' but computed '{computed_hash}'. "
                f"Evidence may have been tampered with."
            )
        
        return errors
    
    def _validate_timestamp(self, evidence: dict, run_id: str) -> List[str]:
        """
        Validate timestamp is from current run
        CRITICAL: Tight tolerance prevents stale evidence acceptance
        """
        errors = []
        
        try:
            # Parse run_id timestamp (ISO 8601 format)
            run_timestamp = datetime.fromisoformat(run_id.replace('Z', '+00:00'))
            
            # Parse evidence timestamp
            evidence_timestamp_str = evidence.get("timestamp", "")
            evidence_timestamp = datetime.fromisoformat(
                evidence_timestamp_str.replace('Z', '+00:00')
            )
            
            # CRITICAL: Evidence timestamp must be very close to run timestamp
            # CRITICAL: Timestamp must equal run_id exactly (deterministic)
            # No tolerance needed since adapter uses run_id as timestamp
            if evidence_timestamp_str != run_id:
                errors.append(
                    f"timestamp mismatch: evidence timestamp '{evidence_timestamp_str}' "
                    f"must equal run_id '{run_id}' exactly. "
                    f"Adapter must use run_id as timestamp for determinism."
                )
        except (ValueError, AttributeError) as e:
            errors.append(f"Invalid timestamp format: {e}")
        
        return errors
    
    def _validate_marker_contracts(self, evidence: dict, gate_config: dict) -> List[str]:
        """
        Validate marker contracts (required_markers and forbidden_markers)
        SINGLE SOURCE OF TRUTH for marker validation
        
        CRITICAL: For marker-level determinism:
        - required_markers must appear as ordered subsequence
        - forbidden_markers must not appear
        - Duplicates are allowed unless manifest specifies unique_marker_sequence
        """
        errors = []
        
        required_markers = gate_config.get("required_markers", [])
        forbidden_markers = gate_config.get("forbidden_markers", [])
        marker_sequence = evidence.get("marker_sequence", [])
        determinism_level = evidence.get("determinism_level")
        unique_marker_sequence = gate_config.get("unique_marker_sequence", False)
        
        # Check required markers (must appear as ordered subsequence)
        if required_markers:
            # Find required markers in order
            marker_idx = 0
            for required_marker in required_markers:
                found = False
                while marker_idx < len(marker_sequence):
                    if marker_sequence[marker_idx] == required_marker:
                        found = True
                        marker_idx += 1
                        break
                    marker_idx += 1
                
                if not found:
                    errors.append(
                        f"Required marker '{required_marker}' not found in marker_sequence "
                        f"or appears out of order. "
                        f"Required markers must appear as ordered subsequence. "
                        f"Gate contract violated."
                    )
        
        # Check forbidden markers (absence)
        for forbidden_marker in forbidden_markers:
            if forbidden_marker in marker_sequence:
                errors.append(
                    f"Forbidden marker '{forbidden_marker}' found in marker_sequence. "
                    f"Gate contract violated."
                )
        
        # CRITICAL: For marker-level determinism with unique_marker_sequence flag
        # Only check duplicates if explicitly requested in manifest
        if determinism_level == "marker" and unique_marker_sequence and marker_sequence:
            seen = set()
            duplicates = []
            for marker in marker_sequence:
                if marker in seen:
                    duplicates.append(marker)
                seen.add(marker)
            
            if duplicates:
                errors.append(
                    f"Duplicate markers found in marker_sequence: {list(set(duplicates))}. "
                    f"Gate config requires unique_marker_sequence=true. "
                    f"For this gate, same input → same marker order, no duplicates."
                )
        
        return errors
    
    def _validate_determinism_scope(self, evidence: dict, gate_config: dict) -> List[str]:
        """
        Validate determinism scope enforcement
        - artifact → artifact_hash REQUIRED
        - trace → trace_hash REQUIRED
        - marker → marker_sequence REQUIRED
        
        CRITICAL: Also validate that fields match determinism_level
        (e.g., marker_sequence should not be present if determinism_level != marker)
        """
        errors = []
        
        manifest_level = gate_config.get("determinism_level")
        evidence_level = evidence.get("determinism_level")
        
        # CRITICAL: determinism_level must match
        if evidence_level != manifest_level:
            errors.append(
                f"determinism_level mismatch: "
                f"manifest specifies '{manifest_level}' "
                f"but evidence has '{evidence_level}'"
            )
        
        # Validate required fields based on determinism level
        if evidence_level == "artifact":
            if "artifact_hash" not in evidence:
                errors.append(
                    f"determinism_level is 'artifact' but artifact_hash is missing"
                )
            # CRITICAL: marker_sequence should not be present for artifact-level
            if "marker_sequence" in evidence and evidence.get("marker_sequence"):
                errors.append(
                    f"marker_sequence present but determinism_level is 'artifact'. "
                    f"This field is not used for artifact-level determinism. "
                    f"Schema violation: data is semantically invalid."
                )
        
        if evidence_level == "trace":
            if "trace_hash" not in evidence:
                errors.append(
                    f"determinism_level is 'trace' but trace_hash is missing"
                )
            # CRITICAL: marker_sequence should not be present for trace-level
            if "marker_sequence" in evidence and evidence.get("marker_sequence"):
                errors.append(
                    f"marker_sequence present but determinism_level is 'trace'. "
                    f"This field is not used for trace-level determinism. "
                    f"Schema violation: data is semantically invalid."
                )
        
        if evidence_level == "marker":
            if "marker_sequence" not in evidence:
                errors.append(
                    f"determinism_level is 'marker' but marker_sequence is missing"
                )
            # CRITICAL: Empty marker_sequence is invalid for marker-level
            elif not evidence.get("marker_sequence"):
                errors.append(
                    f"determinism_level is 'marker' but marker_sequence is empty. "
                    f"Schema requires minItems=1 for marker_sequence."
                )
        
        # Validate allowed_determinism_levels if specified
        allowed_levels = gate_config.get("allowed_determinism_levels", [])
        if allowed_levels and evidence_level not in allowed_levels:
            errors.append(
                f"determinism_level '{evidence_level}' is not in allowed levels: {allowed_levels}"
            )
        
        return errors
    
    def _validate_adapter_output(self, evidence: dict) -> List[str]:
        """
        CRITICAL: Validate adapter_output_fields ⊆ raw_source_fields
        Prevents adapter from introducing new semantic fields
        """
        errors = []
        
        raw_source_fields = set(evidence.get("raw_source_fields", []))
        adapter_output_fields = set(evidence.get("adapter_output_fields", []))
        
        # Check subset relationship
        new_fields = adapter_output_fields - raw_source_fields
        if new_fields:
            errors.append(
                f"Adapter introduced new semantic fields: {sorted(new_fields)}. "
                f"Adapter can only extract, not create data. "
                f"adapter_output_fields must be subset of raw_source_fields."
            )
        
        return errors
    
    def _validate_raw_exit_code(self, evidence: dict) -> List[str]:
        """
        CRITICAL: IF raw_exit_code != 0 AND verdict == PASS THEN FAIL
        Prevents adapter from hiding failures
        """
        errors = []
        
        raw_exit_code = evidence.get("raw_exit_code")
        verdict = evidence.get("verdict")
        
        if raw_exit_code != 0 and verdict == "PASS":
            errors.append(
                f"raw_exit_code is {raw_exit_code} (non-zero) but verdict is PASS. "
                f"Adapter cannot hide failures. "
                f"Non-zero exit code must result in FAIL verdict."
            )
        
        return errors
    
    def _validate_raw_verdict(self, evidence: dict) -> List[str]:
        """
        CRITICAL: raw_verdict MUST equal verdict
        Prevents adapter from changing verdict (truth distortion)
        """
        errors = []
        
        raw_verdict = evidence.get("raw_verdict")
        verdict = evidence.get("verdict")
        
        if raw_verdict != verdict:
            errors.append(
                f"raw_verdict is '{raw_verdict}' but verdict is '{verdict}'. "
                f"Adapter cannot change verdict. "
                f"This prevents truth distortion."
            )
        
        return errors
    
    def _validate_expected_invariants(self, evidence: dict, gate_config: dict) -> List[str]:
        """
        Validate expected invariants if specified in gate config
        Deterministic output ≠ correct output
        """
        errors = []
        
        expected_invariants = gate_config.get("expected_invariants", [])
        if not expected_invariants:
            return errors
        
        invariant_checks = evidence.get("invariant_checks", [])
        invariant_results = {check["name"]: check["result"] for check in invariant_checks}
        
        # Check all expected invariants are present and passed
        for invariant_name in expected_invariants:
            if invariant_name not in invariant_results:
                errors.append(
                    f"Expected invariant '{invariant_name}' not found in invariant_checks"
                )
            elif invariant_results[invariant_name] != "PASS":
                errors.append(
                    f"Invariant '{invariant_name}' failed. "
                    f"Deterministic but wrong. Gate must FAIL."
                )
        
        return errors
    
    def _validate_build_fingerprint(self, evidence: dict, gate_config: dict) -> List[str]:
        """
        CRITICAL: IF manifest.build_fingerprint_required THEN evidence.build_fingerprint MUST exist
        Prevents binary drift causing false determinism
        """
        errors = []
        
        build_fingerprint_required = gate_config.get("build_fingerprint_required", False)
        build_fingerprint = evidence.get("build_fingerprint")
        
        if build_fingerprint_required and not build_fingerprint:
            errors.append(
                f"build_fingerprint is required by gate config but missing in evidence. "
                f"This prevents binary drift causing false determinism."
            )
        
        return errors
    
    def _validate_required_verdict(self, evidence: dict, gate_config: dict) -> List[str]:
        """
        CRITICAL: Validate required_verdict is present and valid
        
        Note: This only validates the field exists and is valid.
        Gate pass/fail logic is handled separately in validate_evidence()
        to avoid conflating evidence verdict with validation errors.
        """
        errors = []
        
        required_verdict = gate_config.get("required_verdict", "PASS")
        actual_verdict = evidence.get("verdict")
        
        # Validate required_verdict is valid
        if required_verdict not in ["PASS", "FAIL"]:
            errors.append(
                f"Invalid required_verdict in gate config: '{required_verdict}'. "
                f"Must be PASS or FAIL."
            )
        
        # Validate actual_verdict is valid
        valid_verdicts = ["PASS", "FAIL", "SKIPPED", "ERROR", "TIMEOUT"]
        if actual_verdict not in valid_verdicts:
            errors.append(
                f"Invalid verdict in evidence: '{actual_verdict}'. "
                f"Must be one of: {valid_verdicts}"
            )
        
        # Note: Verdict matching is checked in validate_evidence() for gate_pass
        # This keeps validation errors separate from gate pass/fail semantics
        
        return errors
    
    def _validate_path_safety(self, evidence_path: Path) -> List[str]:
        """
        Validate path to prevent directory traversal attacks
        CRITICAL: Use resolve() to prevent bypass
        """
        errors = []
        
        try:
            # Resolve to absolute path
            resolved_path = evidence_path.resolve()
            
            # Get project root (3 levels up from validators/)
            validator_dir = Path(__file__).parent
            project_root = validator_dir.parent.parent.parent
            
            # Check if resolved path is within project
            try:
                resolved_path.relative_to(project_root)
            except ValueError:
                errors.append(
                    f"Unsafe path detected: '{evidence_path}' resolves to '{resolved_path}' "
                    f"which is outside project root '{project_root}'. "
                    f"Directory traversal attack prevented."
                )
            
            # Additional check: must be under out/evidence/
            if not str(resolved_path).endswith(('.json', '.txt', '.log')):
                errors.append(
                    f"Invalid evidence file extension: '{evidence_path}'. "
                    f"Only .json, .txt, .log files allowed."
                )
                
        except Exception as e:
            errors.append(f"Path validation failed: {e}")
        
        return errors


def main():
    """Command-line interface for evidence validation"""
    if len(sys.argv) < 5:
        print("Usage: validate_evidence.py <evidence.json> <gate_config.json> <run_id> <command>", file=sys.stderr)
        print("", file=sys.stderr)
        print("Validates AykenOS verification evidence against schema and integrity requirements.", file=sys.stderr)
        print("", file=sys.stderr)
        print("Arguments:", file=sys.stderr)
        print("  evidence.json    - Path to evidence file", file=sys.stderr)
        print("  gate_config.json - Path to gate configuration (extracted from manifest)", file=sys.stderr)
        print("  run_id           - Current verification run ID", file=sys.stderr)
        print("  command          - Expected command string", file=sys.stderr)
        sys.exit(1)
    
    evidence_path = Path(sys.argv[1])
    gate_config_path = Path(sys.argv[2])
    run_id = sys.argv[3]
    command = sys.argv[4]
    
    # Load gate config
    try:
        with open(gate_config_path, 'r') as f:
            gate_config = json.load(f)
    except Exception as e:
        print(f"ERROR: Failed to load gate config: {e}", file=sys.stderr)
        sys.exit(1)
    
    # Initialize validator
    try:
        validator = EvidenceValidator()
    except Exception as e:
        print(f"ERROR: Failed to initialize validator: {e}", file=sys.stderr)
        sys.exit(1)
    
    # Validate evidence
    result = validator.validate_evidence(evidence_path, gate_config, run_id, command)
    
    # CRITICAL: Output JSON for machine parsing (orchestrator needs this)
    result_json = {
        "valid": result.valid,
        "verdict": result.verdict,
        "gate_pass": result.gate_pass,
        "errors": result.errors,
        "warnings": result.warnings
    }
    
    # Output JSON to stdout for orchestrator
    print(json.dumps(result_json, indent=2))
    
    # CRITICAL: Exit code based on gate_pass (not just validation)
    if result.gate_pass:
        sys.exit(0)
    else:
        # Evidence is valid but gate did not pass, OR evidence invalid
        sys.exit(1)


if __name__ == "__main__":
    main()

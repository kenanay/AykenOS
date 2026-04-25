#!/usr/bin/env python3
"""
Report Validator for AykenOS Verification Layer

Validates verification report against schema and business rules:
- JSON schema conformance
- Mutation field validation (must be false)
- Verdict count validation (match gate results)
- Evidence hash verification (canonical hash of all evidence)
- Descriptive error messages

Author: Kenan AY - Architectural Steward
Date: 2026-04-25
"""

import json
import sys
import hashlib
from pathlib import Path
from typing import List, Optional
from dataclasses import dataclass

try:
    import jsonschema
    from jsonschema import validate, ValidationError
except ImportError:
    print("ERROR: jsonschema library not found. Install with: pip3 install jsonschema", file=sys.stderr)
    sys.exit(1)


@dataclass
class ValidationResult:
    """Result of report validation"""
    valid: bool
    errors: List[str]
    warnings: List[str] = None
    
    def __post_init__(self):
        if self.warnings is None:
            self.warnings = []


class ReportValidator:
    """Validates verification report against schema and business rules"""
    
    def __init__(self, schema_path: Optional[Path] = None):
        """
        Initialize validator with schema
        
        Args:
            schema_path: Path to report.schema.json (auto-detected if None)
        """
        if schema_path is None:
            # Auto-detect schema path relative to this script
            validator_dir = Path(__file__).parent
            schema_path = validator_dir.parent / "schemas" / "report.schema.json"
        
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
    
    def validate_report(self, report_path: Path) -> ValidationResult:
        """
        Validate report against schema and business rules
        
        Args:
            report_path: Path to report.json
            
        Returns:
            ValidationResult with validation status and errors
        """
        errors = []
        warnings = []
        
        # Load report
        try:
            with open(report_path, 'r') as f:
                report = json.load(f)
        except FileNotFoundError:
            return ValidationResult(
                valid=False,
                errors=[f"Report file not found: {report_path}"]
            )
        except json.JSONDecodeError as e:
            return ValidationResult(
                valid=False,
                errors=[f"Invalid JSON in report: {e}"]
            )
        
        # Validate JSON schema
        try:
            validate(instance=report, schema=self.schema)
        except ValidationError as e:
            errors.append(f"Schema validation failed: {e.message}")
            errors.append(f"  Path: {' -> '.join(str(p) for p in e.path)}")
            return ValidationResult(valid=False, errors=errors)
        
        # Business rule validations
        errors.extend(self._validate_mutation_field(report))
        errors.extend(self._validate_verdict_counts(report))
        errors.extend(self._validate_determinism_summary(report))
        errors.extend(self._validate_evidence_files_consistency(report))
        errors.extend(self._validate_evidence_hash(report, report_path))
        
        return ValidationResult(
            valid=len(errors) == 0,
            errors=errors,
            warnings=warnings
        )
    
    def _validate_mutation_field(self, report: dict) -> List[str]:
        """
        Validate mutation field is false
        Verification layer must not mutate the system
        """
        errors = []
        
        mutation = report.get("mutation")
        if mutation is not False:
            errors.append(
                f"mutation field must be false (verification layer must not mutate system), "
                f"but found: {mutation}"
            )
        
        return errors
    
    def _validate_verdict_counts(self, report: dict) -> List[str]:
        """
        Validate verdict counts match gate results
        Ensures report consistency
        """
        errors = []
        
        # Extract counts from report
        gates_checked = report.get("gates_checked", 0)
        gates_passed = report.get("gates_passed", 0)
        gates_failed = report.get("gates_failed", 0)
        gates_skipped = report.get("gates_skipped", 0)
        gates_error = report.get("gates_error", 0)
        gates_timeout = report.get("gates_timeout", 0)
        
        # Count verdicts from gates object
        gates = report.get("gates", {})
        actual_counts = {
            "PASS": 0,
            "FAIL": 0,
            "SKIPPED": 0,
            "ERROR": 0,
            "TIMEOUT": 0
        }
        
        for gate_id, gate_result in gates.items():
            verdict = gate_result.get("verdict")
            if verdict in actual_counts:
                actual_counts[verdict] += 1
            else:
                errors.append(
                    f"Gate '{gate_id}' has invalid verdict: {verdict}"
                )
        
        # Validate total count
        total_actual = sum(actual_counts.values())
        if gates_checked != total_actual:
            errors.append(
                f"gates_checked ({gates_checked}) does not match "
                f"total gate results ({total_actual})"
            )
        
        # Validate individual counts
        if gates_passed != actual_counts["PASS"]:
            errors.append(
                f"gates_passed ({gates_passed}) does not match "
                f"actual PASS count ({actual_counts['PASS']})"
            )
        
        if gates_failed != actual_counts["FAIL"]:
            errors.append(
                f"gates_failed ({gates_failed}) does not match "
                f"actual FAIL count ({actual_counts['FAIL']})"
            )
        
        if gates_skipped != actual_counts["SKIPPED"]:
            errors.append(
                f"gates_skipped ({gates_skipped}) does not match "
                f"actual SKIPPED count ({actual_counts['SKIPPED']})"
            )
        
        if gates_error != actual_counts["ERROR"]:
            errors.append(
                f"gates_error ({gates_error}) does not match "
                f"actual ERROR count ({actual_counts['ERROR']})"
            )
        
        if gates_timeout != actual_counts["TIMEOUT"]:
            errors.append(
                f"gates_timeout ({gates_timeout}) does not match "
                f"actual TIMEOUT count ({actual_counts['TIMEOUT']})"
            )
        
        return errors
    
    def _validate_determinism_summary(self, report: dict) -> List[str]:
        """
        Validate determinism summary matches gate determinism levels
        Ensures report consistency
        """
        errors = []
        
        # Extract determinism summary from report
        determinism_summary = report.get("determinism_summary", {})
        summary_artifact = determinism_summary.get("artifact", 0)
        summary_trace = determinism_summary.get("trace", 0)
        summary_marker = determinism_summary.get("marker", 0)
        summary_scheduling = determinism_summary.get("scheduling-independent", 0)
        
        # Count determinism levels from gates object
        gates = report.get("gates", {})
        actual_counts = {
            "artifact": 0,
            "trace": 0,
            "marker": 0,
            "scheduling-independent": 0
        }
        
        for gate_id, gate_result in gates.items():
            determinism_level = gate_result.get("determinism_level")
            if determinism_level in actual_counts:
                actual_counts[determinism_level] += 1
            else:
                errors.append(
                    f"Gate '{gate_id}' has invalid determinism_level: {determinism_level}"
                )
        
        # Validate counts
        if summary_artifact != actual_counts["artifact"]:
            errors.append(
                f"determinism_summary.artifact ({summary_artifact}) does not match "
                f"actual artifact-level gate count ({actual_counts['artifact']})"
            )
        
        if summary_trace != actual_counts["trace"]:
            errors.append(
                f"determinism_summary.trace ({summary_trace}) does not match "
                f"actual trace-level gate count ({actual_counts['trace']})"
            )
        
        if summary_marker != actual_counts["marker"]:
            errors.append(
                f"determinism_summary.marker ({summary_marker}) does not match "
                f"actual marker-level gate count ({actual_counts['marker']})"
            )
        
        if summary_scheduling != actual_counts["scheduling-independent"]:
            errors.append(
                f"determinism_summary.scheduling-independent ({summary_scheduling}) does not match "
                f"actual scheduling-independent gate count ({actual_counts['scheduling-independent']})"
            )
        
        return errors
    
    def _validate_evidence_files_consistency(self, report: dict) -> List[str]:
        """
        CRITICAL: Validate evidence_files ↔ gates consistency
        This prevents integrity break where report claims different files than gates reference
        
        Rule: set(evidence_files) MUST equal set(gate evidence paths)
        """
        errors = []
        
        evidence_files = set(report.get("evidence_files", []))
        gates = report.get("gates", {})
        
        # Collect all evidence paths from gates
        gate_evidence_paths = set()
        for gate_id, gate_result in gates.items():
            evidence_path = gate_result.get("evidence_path")
            if evidence_path:
                gate_evidence_paths.add(evidence_path)
        
        # CRITICAL: Sets must match exactly
        if evidence_files != gate_evidence_paths:
            missing_in_files = gate_evidence_paths - evidence_files
            extra_in_files = evidence_files - gate_evidence_paths
            
            if missing_in_files:
                errors.append(
                    f"evidence_files missing paths referenced in gates: {sorted(missing_in_files)}. "
                    f"Integrity break: report claims different files than gates reference."
                )
            
            if extra_in_files:
                errors.append(
                    f"evidence_files contains paths not referenced in gates: {sorted(extra_in_files)}. "
                    f"Integrity break: report claims files that no gate produced."
                )
        
        return errors
    
    def _validate_evidence_hash(self, report: dict, report_path: Path) -> List[str]:
        """
        CRITICAL: Validate evidence_hash is correct canonical hash
        This is the CORE of evidence chain integrity
        
        Algorithm (per design spec):
        1. Sort evidence files by gate_id (lexicographic)
        2. For each file: compute SHA256(file_content)
        3. Concatenate hashes in sorted order
        4. Compute final SHA256(concatenated_hashes)
        
        Same evidence → same hash, always
        """
        errors = []
        
        evidence_files = report.get("evidence_files", [])
        declared_hash = report.get("evidence_hash")
        gates = report.get("gates", {})
        
        if not evidence_files:
            errors.append(
                "evidence_files array is empty. "
                "Cannot compute canonical evidence hash."
            )
            return errors
        
        # Compute canonical hash
        try:
            # Get report directory (evidence files are relative to this)
            report_dir = report_path.parent
            
            # Build gate_id → evidence_path mapping
            gate_evidence_map = {}
            for gate_id, gate_result in gates.items():
                evidence_path = gate_result.get("evidence_path")
                if evidence_path:
                    gate_evidence_map[gate_id] = evidence_path
            
            # Sort by gate_id (lexicographic) per design spec
            sorted_gate_ids = sorted(gate_evidence_map.keys())
            
            # Compute hash for each evidence file in gate_id order
            file_hashes = []
            for gate_id in sorted_gate_ids:
                evidence_file = gate_evidence_map[gate_id]
                
                # CRITICAL: Validate path safety before accessing
                path_errors = self._validate_evidence_path_safety(evidence_file, report_dir)
                if path_errors:
                    errors.extend(path_errors)
                    continue
                
                evidence_path = report_dir / evidence_file
                
                if not evidence_path.exists():
                    errors.append(
                        f"Evidence file not found for gate '{gate_id}': {evidence_file}. "
                        f"Cannot compute canonical hash."
                    )
                    continue
                
                try:
                    # CRITICAL: Compute canonical JSON hash (same as orchestrator)
                    # Algorithm: SHA256(canonical JSON excluding integrity.file_hash)
                    with open(evidence_path, 'r') as f:
                        evidence = json.load(f)
                    
                    # Remove integrity.file_hash for canonical hash
                    if 'integrity' in evidence and 'file_hash' in evidence['integrity']:
                        del evidence['integrity']['file_hash']
                    
                    # Compute SHA256 of canonical JSON (sorted keys for determinism)
                    canonical_json = json.dumps(evidence, sort_keys=True, separators=(',', ':'))
                    file_hash = hashlib.sha256(canonical_json.encode('utf-8')).hexdigest()
                    file_hashes.append(file_hash)
                    
                except json.JSONDecodeError as e:
                    errors.append(
                        f"Invalid JSON in evidence file for gate '{gate_id}' at '{evidence_file}': {e}"
                    )
                except Exception as e:
                    errors.append(
                        f"Failed to read evidence file for gate '{gate_id}' at '{evidence_file}': {e}"
                    )
            
            # If any file failed, cannot validate hash
            if errors:
                return errors
            
            # Concatenate hashes and compute final hash
            concatenated = ''.join(file_hashes)
            computed_hash = hashlib.sha256(concatenated.encode('utf-8')).hexdigest()
            
            # Validate against declared hash
            if computed_hash != declared_hash:
                errors.append(
                    f"evidence_hash mismatch: "
                    f"declared '{declared_hash}' but computed '{computed_hash}'. "
                    f"Evidence chain integrity violated. "
                    f"This is the CORE trust anchor - if this fails, nothing can be trusted. "
                    f"Hash computed from {len(file_hashes)} evidence files sorted by gate_id."
                )
        
        except Exception as e:
            errors.append(f"Evidence hash validation failed: {e}")
        
        return errors
    
    def _validate_evidence_path_safety(self, evidence_file: str, report_dir: Path) -> List[str]:
        """
        CRITICAL: Validate evidence path to prevent directory traversal
        Must be called before accessing any evidence file from report
        """
        errors = []
        
        try:
            # Resolve to absolute path
            evidence_path = (report_dir / evidence_file).resolve()
            
            # Check if resolved path is within report directory
            try:
                evidence_path.relative_to(report_dir.resolve())
            except ValueError:
                errors.append(
                    f"Unsafe evidence path: '{evidence_file}' resolves to '{evidence_path}' "
                    f"which is outside report directory '{report_dir}'. "
                    f"Directory traversal attack prevented."
                )
            
            # Additional check: must be under gates/ subdirectory
            if not any(part == "gates" for part in evidence_path.parts):
                errors.append(
                    f"Invalid evidence path: '{evidence_file}'. "
                    f"Evidence must be under gates/ subdirectory."
                )
        
        except Exception as e:
            errors.append(f"Evidence path validation failed for '{evidence_file}': {e}")
        
        return errors


def main():
    """Command-line interface for report validation"""
    if len(sys.argv) < 2:
        print("Usage: validate_report.py <report.json>", file=sys.stderr)
        print("", file=sys.stderr)
        print("Validates AykenOS verification report against schema and business rules.", file=sys.stderr)
        sys.exit(1)
    
    report_path = Path(sys.argv[1])
    
    # Initialize validator
    try:
        validator = ReportValidator()
    except Exception as e:
        print(f"ERROR: Failed to initialize validator: {e}", file=sys.stderr)
        sys.exit(1)
    
    # Validate report
    result = validator.validate_report(report_path)
    
    # Output results
    if result.valid:
        print(f"✓ Report validation PASSED: {report_path}")
        sys.exit(0)
    else:
        print(f"✗ Report validation FAILED: {report_path}", file=sys.stderr)
        print("", file=sys.stderr)
        print("Errors:", file=sys.stderr)
        for error in result.errors:
            print(f"  - {error}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()

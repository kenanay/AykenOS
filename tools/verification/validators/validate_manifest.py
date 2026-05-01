#!/usr/bin/env python3
"""
Manifest Validator for AykenOS Verification Layer

Validates manifest.json against schema and business rules:
- JSON schema conformance
- Gate ID uniqueness
- Phase number detection (reject "phase" + digits)
- Command allowlist validation
- Dependency reference validation
- Circular dependency detection

Author: Kenan AY - Architectural Steward
Date: 2026-04-25
"""

import json
import sys
import re
from pathlib import Path
from typing import Dict, List, Set, Tuple, Optional
from dataclasses import dataclass

try:
    import jsonschema
    from jsonschema import validate, ValidationError
except ImportError:
    print("ERROR: jsonschema library not found. Install with: pip3 install jsonschema", file=sys.stderr)
    sys.exit(1)


@dataclass
class ValidationResult:
    """Result of manifest validation"""
    valid: bool
    errors: List[str]
    warnings: List[str] = None
    
    def __post_init__(self):
        if self.warnings is None:
            self.warnings = []


class ManifestValidator:
    """Validates verification manifest against schema and business rules"""
    
    def __init__(self, schema_path: Optional[Path] = None):
        """
        Initialize validator with schema
        
        Args:
            schema_path: Path to manifest.schema.json (auto-detected if None)
        """
        if schema_path is None:
            # Auto-detect schema path relative to this script
            validator_dir = Path(__file__).parent
            schema_path = validator_dir.parent / "schemas" / "manifest.schema.json"
        
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
    
    def validate_manifest(self, manifest_path: Path) -> ValidationResult:
        """
        Validate manifest against schema and business rules
        
        Args:
            manifest_path: Path to manifest.json
            
        Returns:
            ValidationResult with validation status and errors
        """
        errors = []
        warnings = []
        
        # Load manifest
        try:
            with open(manifest_path, 'r') as f:
                manifest = json.load(f)
        except FileNotFoundError:
            return ValidationResult(valid=False, errors=[f"Manifest file not found: {manifest_path}"])
        except json.JSONDecodeError as e:
            return ValidationResult(valid=False, errors=[f"Invalid JSON in manifest: {e}"])
        
        # Validate JSON schema
        try:
            validate(instance=manifest, schema=self.schema)
        except ValidationError as e:
            errors.append(f"Schema validation failed: {e.message}")
            errors.append(f"  Path: {' -> '.join(str(p) for p in e.path)}")
            return ValidationResult(valid=False, errors=errors)
        
        # Extract gates for business rule validation
        gates = manifest.get("gates", [])
        
        # Business rule validations
        errors.extend(self._validate_gate_id_uniqueness(gates))
        errors.extend(self._validate_phase_numbers(gates))
        errors.extend(self._validate_command_allowlist(gates))
        errors.extend(self._validate_dependency_references(gates))
        
        circular_dep_errors = self._validate_circular_dependencies(gates)
        errors.extend(circular_dep_errors)
        
        # Additional validations
        errors.extend(self._validate_determinism_levels(gates))
        errors.extend(self._validate_required_closure_verdict(gates))
        
        return ValidationResult(
            valid=len(errors) == 0,
            errors=errors,
            warnings=warnings
        )
    
    def _validate_gate_id_uniqueness(self, gates: List[dict]) -> List[str]:
        """Validate that all gate IDs are unique"""
        errors = []
        gate_ids = [gate.get("id") for gate in gates]
        seen = set()
        duplicates = set()
        
        for gate_id in gate_ids:
            if gate_id in seen:
                duplicates.add(gate_id)
            seen.add(gate_id)
        
        if duplicates:
            errors.append(f"Duplicate gate IDs found: {', '.join(sorted(duplicates))}")
        
        return errors
    
    def _validate_phase_numbers(self, gates: List[dict]) -> List[str]:
        """Validate that gate IDs do not contain phase numbers"""
        errors = []
        phase_pattern = re.compile(r'phase\d+')
        
        for gate in gates:
            gate_id = gate.get("id", "")
            if phase_pattern.search(gate_id):
                errors.append(
                    f"Gate ID '{gate_id}' contains phase number. "
                    f"Use descriptive names instead (e.g., 'boot_integrity', 'ring3_runtime')"
                )
        
        return errors
    
    def _validate_command_allowlist(self, gates: List[dict]) -> List[str]:
        """Validate that commands match allowlist pattern"""
        errors = []
        allowlist_pattern = re.compile(r'^make ci-gate-')
        
        for gate in gates:
            gate_id = gate.get("id", "")
            command = gate.get("command", "")
            
            if not allowlist_pattern.match(command):
                errors.append(
                    f"Gate '{gate_id}' has invalid command '{command}'. "
                    f"Commands must match pattern: 'make ci-gate-*'"
                )
        
        return errors
    
    def _validate_dependency_references(self, gates: List[dict]) -> List[str]:
        """Validate that all dependency references point to existing gates"""
        errors = []
        gate_ids = {gate.get("id") for gate in gates}
        
        for gate in gates:
            gate_id = gate.get("id", "")
            depends_on = gate.get("depends_on", [])
            
            for dep_id in depends_on:
                if dep_id not in gate_ids:
                    errors.append(
                        f"Gate '{gate_id}' depends on non-existent gate '{dep_id}'"
                    )
        
        return errors
    
    def _validate_circular_dependencies(self, gates: List[dict]) -> List[str]:
        """Detect circular dependencies using graph traversal"""
        errors = []
        
        # Build adjacency list
        graph: Dict[str, List[str]] = {}
        for gate in gates:
            gate_id = gate.get("id", "")
            depends_on = gate.get("depends_on", [])
            graph[gate_id] = depends_on
        
        # Detect cycles using DFS
        visited = set()
        rec_stack = set()
        
        def has_cycle(node: str, path: List[str]) -> Optional[List[str]]:
            """DFS to detect cycle, returns cycle path if found"""
            visited.add(node)
            rec_stack.add(node)
            path.append(node)
            
            for neighbor in graph.get(node, []):
                if neighbor not in visited:
                    cycle = has_cycle(neighbor, path.copy())
                    if cycle:
                        return cycle
                elif neighbor in rec_stack:
                    # Found cycle
                    cycle_start = path.index(neighbor)
                    return path[cycle_start:] + [neighbor]
            
            rec_stack.remove(node)
            return None
        
        # Check each gate for cycles
        for gate_id in graph:
            if gate_id not in visited:
                cycle = has_cycle(gate_id, [])
                if cycle:
                    cycle_str = " -> ".join(cycle)
                    errors.append(
                        f"Circular dependency detected: {cycle_str}"
                    )
                    break  # Report first cycle found
        
        return errors
    
    def _validate_determinism_levels(self, gates: List[dict]) -> List[str]:
        """Validate determinism level constraints"""
        errors = []
        
        for gate in gates:
            gate_id = gate.get("id", "")
            determinism_level = gate.get("determinism_level")
            allowed_levels = gate.get("allowed_determinism_levels", [])
            
            # If allowed_determinism_levels is specified, check constraint
            if allowed_levels and determinism_level not in allowed_levels:
                errors.append(
                    f"Gate '{gate_id}' has determinism_level '{determinism_level}' "
                    f"but only {allowed_levels} are allowed for this gate"
                )
        
        return errors
    
    def _validate_required_closure_verdict(self, gates: List[dict]) -> List[str]:
        """
        Validate required_closure_verdict field if present
        This field is used for determinism gates that check closure properties
        """
        errors = []
        
        valid_closure_verdicts = [
            "DETERMINISM_PASS",
            "DETERMINISM_FAIL",
            "CLOSURE_PASS",
            "CLOSURE_FAIL"
        ]
        
        for gate in gates:
            gate_id = gate.get("id", "")
            required_closure_verdict = gate.get("required_closure_verdict")
            
            # If specified, validate it's a known closure verdict
            if required_closure_verdict:
                if required_closure_verdict not in valid_closure_verdicts:
                    errors.append(
                        f"Gate '{gate_id}' has invalid required_closure_verdict: "
                        f"'{required_closure_verdict}'. "
                        f"Valid values: {valid_closure_verdicts}"
                    )
                
                # If closure verdict is specified, determinism_level should be artifact
                determinism_level = gate.get("determinism_level")
                if determinism_level != "artifact":
                    errors.append(
                        f"Gate '{gate_id}' has required_closure_verdict but "
                        f"determinism_level is '{determinism_level}'. "
                        f"Closure verdicts require determinism_level='artifact'."
                    )
        
        return errors


def main():
    """Command-line interface for manifest validation"""
    if len(sys.argv) < 2:
        print("Usage: validate_manifest.py <manifest.json>", file=sys.stderr)
        print("", file=sys.stderr)
        print("Validates AykenOS verification manifest against schema and business rules.", file=sys.stderr)
        sys.exit(1)
    
    manifest_path = Path(sys.argv[1])
    
    # Initialize validator
    try:
        validator = ManifestValidator()
    except Exception as e:
        print(f"ERROR: Failed to initialize validator: {e}", file=sys.stderr)
        sys.exit(1)
    
    # Validate manifest
    result = validator.validate_manifest(manifest_path)
    
    # Output results
    if result.valid:
        print(f"✓ Manifest validation PASSED: {manifest_path}")
        sys.exit(0)
    else:
        print(f"✗ Manifest validation FAILED: {manifest_path}", file=sys.stderr)
        print("", file=sys.stderr)
        print("Errors:", file=sys.stderr)
        for error in result.errors:
            print(f"  - {error}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()

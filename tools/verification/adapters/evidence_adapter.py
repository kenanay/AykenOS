#!/usr/bin/env python3
"""
Evidence Adapter Helper - Python utilities for evidence generation

This module provides helper functions for generating verification evidence
from existing gate outputs. It handles hash computation, evidence structure
generation, and ensures pass-through extraction without semantic transformation.

CRITICAL: This adapter is a pass-through extractor only. It MUST NOT:
- Transform or normalize data semantically
- Change verdicts or exit codes
- Introduce new semantic fields not present in raw output
- Alter the meaning of any extracted data

Requirements: 10.1, 10.3, 10.6, 10.7
"""

import hashlib
import json
import sys
import os
from typing import Dict, Any, List, Optional
from datetime import datetime


def compute_sha256(data: str) -> str:
    """
    Compute SHA256 hash of a string.
    
    Args:
        data: String to hash
        
    Returns:
        Hexadecimal SHA256 hash (64 characters)
    """
    return hashlib.sha256(data.encode('utf-8')).hexdigest()


def compute_file_hash(file_path: str) -> str:
    """
    Compute SHA256 hash of a file.
    
    Args:
        file_path: Path to file
        
    Returns:
        Hexadecimal SHA256 hash (64 characters)
    """
    sha256_hash = hashlib.sha256()
    try:
        with open(file_path, 'rb') as f:
            for byte_block in iter(lambda: f.read(4096), b""):
                sha256_hash.update(byte_block)
        return sha256_hash.hexdigest()
    except FileNotFoundError:
        return ""


def compute_command_fingerprint(command: str) -> str:
    """
    Compute command fingerprint (SHA256 of command string).
    
    This fingerprint ensures evidence came from the expected command.
    
    Args:
        command: Command string that was executed
        
    Returns:
        SHA256 hash of command string
        
    Requirements: 10.7
    """
    return compute_sha256(command)


def compute_canonical_evidence_hash(evidence: Dict[str, Any]) -> str:
    """
    Compute canonical hash of evidence excluding integrity.file_hash field.
    
    CRITICAL: The file_hash field cannot be included in its own computation.
    This function creates a copy of the evidence, removes integrity.file_hash,
    and computes the hash of the canonical JSON representation.
    
    Args:
        evidence: Evidence dictionary
        
    Returns:
        SHA256 hash of canonical evidence (excluding file_hash)
        
    Requirements: 10.6
    """
    # Create deep copy to avoid mutating original
    evidence_copy = json.loads(json.dumps(evidence))
    
    # Remove file_hash from integrity section if present
    if 'integrity' in evidence_copy and 'file_hash' in evidence_copy['integrity']:
        del evidence_copy['integrity']['file_hash']
    
    # Compute hash of canonical JSON (sorted keys, no whitespace)
    canonical_json = json.dumps(evidence_copy, sort_keys=True, separators=(',', ':'))
    return compute_sha256(canonical_json)


def generate_evidence_structure(
    gate_id: str,
    run_id: str,
    command: str,
    exit_code: int,
    duration_ms: int,
    verdict: str,
    raw_verdict: str,
    determinism_level: str,
    raw_source_fields: List[str],
    adapter_output_fields: List[str],
    marker_sequence: Optional[List[str]] = None,
    trace_hash: Optional[str] = None,
    artifact_hash: Optional[str] = None,
    build_fingerprint: Optional[str] = None,
    raw_log_hash: Optional[str] = None,
    invariant_checks: Optional[List[Dict[str, Any]]] = None,
    timeout: bool = False,
    additional_details: Optional[Dict[str, Any]] = None
) -> Dict[str, Any]:
    """
    Generate evidence JSON structure conforming to evidence.schema.json.
    
    CRITICAL: This function performs pass-through extraction only.
    - raw_verdict MUST equal verdict (no transformation)
    - adapter_output_fields MUST be subset of raw_source_fields
    - No semantic transformation of any field
    
    Args:
        gate_id: Gate identifier
        run_id: Verification run identifier (ISO 8601 format)
        command: Command that was executed
        exit_code: Command exit code
        duration_ms: Execution duration in milliseconds
        verdict: Gate verdict (PASS, FAIL, SKIPPED, ERROR, TIMEOUT)
        raw_verdict: Original verdict from raw output (must equal verdict)
        determinism_level: Determinism scope (artifact, trace, marker, scheduling-independent)
        raw_source_fields: Fields present in raw gate output
        adapter_output_fields: Fields in adapter output (must be subset of raw_source_fields)
        marker_sequence: Markers found in execution order (required for marker-level)
        trace_hash: SHA256 of execution trace (required for trace-level)
        artifact_hash: SHA256 of artifact (required for artifact-level)
        build_fingerprint: SHA256 of kernel + toolchain + build_flags (optional)
        raw_log_hash: SHA256 of raw gate output (optional)
        invariant_checks: Correctness invariant results (optional)
        timeout: Whether execution timed out
        additional_details: Additional details to include in details section
        
    Returns:
        Evidence dictionary conforming to schema
        
    Requirements: 10.1, 10.3, 10.6, 10.7
    """
    # Validate raw_verdict equals verdict (truth preservation)
    if raw_verdict != verdict:
        raise ValueError(
            f"CRITICAL: raw_verdict ({raw_verdict}) must equal verdict ({verdict}). "
            "Adapter cannot change verdict."
        )
    
    # Validate adapter_output_fields is subset of raw_source_fields
    if not set(adapter_output_fields).issubset(set(raw_source_fields)):
        extra_fields = set(adapter_output_fields) - set(raw_source_fields)
        raise ValueError(
            f"CRITICAL: adapter_output_fields contains fields not in raw_source_fields: {extra_fields}. "
            "Adapter cannot introduce new semantic fields."
        )
    
    # Generate timestamp from run_id (deterministic)
    # CRITICAL: Use run_id as timestamp source for determinism
    timestamp = run_id
    
    # Compute command fingerprint
    command_fingerprint = compute_command_fingerprint(command)
    
    # Build evidence structure
    evidence = {
        "gate_id": gate_id,
        "run_id": run_id,
        "timestamp": timestamp,
        "verdict": verdict,
        "determinism_level": determinism_level,
        "raw_exit_code": exit_code,
        "raw_verdict": raw_verdict,
        "raw_source_fields": raw_source_fields,
        "adapter_output_fields": adapter_output_fields,
        "integrity": {
            "source_gate_id": gate_id,
            "command_fingerprint": command_fingerprint,
            "schema_version": "1.0"
        },
        "details": {
            "command": command,
            "exit_code": exit_code,
            "duration_ms": duration_ms,
            "timeout": timeout
        }
    }
    
    # Add optional fields based on determinism level
    if marker_sequence is not None:
        evidence["marker_sequence"] = marker_sequence
    
    if trace_hash is not None:
        evidence["trace_hash"] = trace_hash
    
    if artifact_hash is not None:
        evidence["artifact_hash"] = artifact_hash
    
    if build_fingerprint is not None:
        evidence["build_fingerprint"] = build_fingerprint
    
    if raw_log_hash is not None:
        evidence["raw_log_hash"] = raw_log_hash
    
    if invariant_checks is not None:
        evidence["invariant_checks"] = invariant_checks
    
    # Add additional details if provided
    if additional_details:
        evidence["details"].update(additional_details)
    
    # Compute canonical file hash (excluding integrity.file_hash itself)
    file_hash = compute_canonical_evidence_hash(evidence)
    evidence["integrity"]["file_hash"] = file_hash
    
    return evidence


def write_evidence(evidence: Dict[str, Any], output_path: str) -> None:
    """
    Write evidence to JSON file with proper formatting.
    
    Args:
        evidence: Evidence dictionary
        output_path: Path to write evidence file
    """
    with open(output_path, 'w') as f:
        json.dump(evidence, f, indent=2, sort_keys=True)


def generate_from_cli():
    """
    Generate evidence from command-line arguments.
    
    This is the SINGLE SOURCE OF TRUTH for evidence generation.
    Bash adapter MUST use this, not duplicate logic.
    """
    import argparse
    
    parser = argparse.ArgumentParser(description='Generate verification evidence')
    parser.add_argument('--gate-id', required=True, help='Gate identifier')
    parser.add_argument('--run-id', required=True, help='Verification run ID')
    parser.add_argument('--command', required=True, help='Command executed')
    parser.add_argument('--exit-code', type=int, required=True, help='Exit code')
    parser.add_argument('--duration-ms', type=int, required=True, help='Duration in ms')
    parser.add_argument('--determinism-level', required=True, 
                       choices=['artifact', 'trace', 'marker', 'scheduling-independent'],
                       help='Determinism level')
    parser.add_argument('--raw-output', required=True, help='Path to raw output file')
    parser.add_argument('--output', required=True, help='Path to write evidence JSON')
    parser.add_argument('--build-fingerprint-required', action='store_true',
                       help='Require build fingerprint')
    
    args = parser.parse_args()
    
    # Read raw output
    try:
        with open(args.raw_output, 'r') as f:
            raw_content = f.read()
    except Exception as e:
        print(f"ERROR: Cannot read raw output: {e}", file=sys.stderr)
        sys.exit(1)
    
    # Try to parse as JSON (REQUIRED for structured verdict)
    raw_data = None
    try:
        raw_data = json.loads(raw_content)
    except json.JSONDecodeError as e:
        print(f"ERROR: Raw output is not valid JSON", file=sys.stderr)
        print(f"ERROR: Raw output file: {args.raw_output}", file=sys.stderr)
        print(f"ERROR: JSON parse error: {e}", file=sys.stderr)
        print(f"ERROR: Gate must produce structured JSON output", file=sys.stderr)
        sys.exit(1)
    
    # Extract verdict (PASS-THROUGH ONLY)
    # CRITICAL: No UNKNOWN verdict allowed in truth engine
    # If structured verdict not found, adapter MUST fail
    verdict = None
    raw_verdict = None
    if raw_data and isinstance(raw_data, dict) and 'verdict' in raw_data:
        verdict = raw_data['verdict']
        raw_verdict = raw_data['verdict']
    
    # CRITICAL: Fail if no structured verdict found
    if verdict is None:
        print(f"ERROR: No structured verdict found in raw output", file=sys.stderr)
        print(f"ERROR: Raw output file: {args.raw_output}", file=sys.stderr)
        print(f"ERROR: Gate must produce structured JSON with 'verdict' field", file=sys.stderr)
        print(f"ERROR: Truth engine does not accept UNKNOWN verdicts", file=sys.stderr)
        sys.exit(1)
    
    # Extract markers (PASS-THROUGH ONLY)
    marker_sequence = None
    if raw_data and isinstance(raw_data, dict) and 'markers' in raw_data:
        marker_sequence = raw_data['markers']
    
    # Extract invariant_checks (PASS-THROUGH ONLY)
    invariant_checks = None
    if raw_data and isinstance(raw_data, dict) and 'invariant_checks' in raw_data:
        invariant_checks = raw_data['invariant_checks']
    
    # CRITICAL: Validate determinism requirements
    if args.determinism_level == 'marker':
        if not marker_sequence or len(marker_sequence) == 0:
            print(f"ERROR: marker-level determinism requires markers", file=sys.stderr)
            print(f"ERROR: Raw output: {args.raw_output}", file=sys.stderr)
            print(f"ERROR: Gate must produce structured output with 'markers' field", file=sys.stderr)
            sys.exit(1)
    
    # Compute hashes
    raw_log_hash = compute_file_hash(args.raw_output)
    
    trace_hash = None
    if args.determinism_level == 'trace':
        trace_hash = raw_log_hash
    
    artifact_hash = None
    if args.determinism_level == 'artifact':
        # CRITICAL: Hash actual artifact, not log
        # For MVP, try common locations
        artifact_candidates = ['out/kernel.elf', 'out/EFI.img']
        for candidate in artifact_candidates:
            if os.path.exists(candidate):
                artifact_hash = compute_file_hash(candidate)
                break
        
        if not artifact_hash:
            print(f"ERROR: artifact-level determinism requires artifact file", file=sys.stderr)
            print(f"ERROR: Tried: {artifact_candidates}", file=sys.stderr)
            sys.exit(1)
    
    build_fingerprint = None
    if args.build_fingerprint_required:
        # Compute build fingerprint
        kernel_binary = "out/kernel.elf"
        if os.path.exists(kernel_binary):
            build_fingerprint = compute_file_hash(kernel_binary)
        else:
            print(f"WARNING: Build fingerprint required but kernel not found: {kernel_binary}", file=sys.stderr)
    
    # Extract raw source fields
    raw_source_fields = ["exit_code", "raw_output_text"]
    if raw_data and isinstance(raw_data, dict):
        raw_source_fields = list(raw_data.keys()) + ["exit_code"]
        raw_source_fields = sorted(set(raw_source_fields))
    
    # Determine adapter output fields
    adapter_output_fields = ["exit_code", "verdict"]  # verdict is guaranteed to exist
    if marker_sequence:
        adapter_output_fields.append("markers")
    if invariant_checks:
        adapter_output_fields.append("invariant_checks")
    
    # Generate evidence
    try:
        evidence = generate_evidence_structure(
            gate_id=args.gate_id,
            run_id=args.run_id,
            command=args.command,
            exit_code=args.exit_code,
            duration_ms=args.duration_ms,
            verdict=verdict,
            raw_verdict=raw_verdict,
            determinism_level=args.determinism_level,
            raw_source_fields=raw_source_fields,
            adapter_output_fields=adapter_output_fields,
            marker_sequence=marker_sequence,
            trace_hash=trace_hash,
            artifact_hash=artifact_hash,
            build_fingerprint=build_fingerprint,
            raw_log_hash=raw_log_hash,
            invariant_checks=invariant_checks
        )
        
        # Write evidence
        write_evidence(evidence, args.output)
        
        print(f"Evidence written to: {args.output}")
        print(f"Gate: {args.gate_id}")
        print(f"Verdict: {verdict}")
        print(f"Run ID: {args.run_id}")
        
    except ValueError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"ERROR: Failed to generate evidence: {e}", file=sys.stderr)
        sys.exit(1)


def main():
    """
    Command-line interface for evidence generation.
    
    Usage:
        evidence_adapter.py <function> [args...]
        
    Functions:
        hash <string>              - Compute SHA256 of string
        file_hash <path>           - Compute SHA256 of file
        command_fingerprint <cmd>  - Compute command fingerprint
        generate [args]            - Generate evidence (primary function)
    """
    if len(sys.argv) < 2:
        print("Usage: evidence_adapter.py <function> [args...]", file=sys.stderr)
        print("Functions: hash, file_hash, command_fingerprint, generate", file=sys.stderr)
        sys.exit(1)
    
    function = sys.argv[1]
    
    if function == "generate":
        # Remove 'generate' from argv so argparse works
        sys.argv.pop(1)
        generate_from_cli()
    
    elif function == "hash":
        if len(sys.argv) != 3:
            print("Usage: evidence_adapter.py hash <string>", file=sys.stderr)
            sys.exit(1)
        print(compute_sha256(sys.argv[2]))
    
    elif function == "file_hash":
        if len(sys.argv) != 3:
            print("Usage: evidence_adapter.py file_hash <path>", file=sys.stderr)
            sys.exit(1)
        result = compute_file_hash(sys.argv[2])
        if result:
            print(result)
        else:
            print(f"Error: File not found: {sys.argv[2]}", file=sys.stderr)
            sys.exit(1)
    
    elif function == "command_fingerprint":
        if len(sys.argv) != 3:
            print("Usage: evidence_adapter.py command_fingerprint <command>", file=sys.stderr)
            sys.exit(1)
        print(compute_command_fingerprint(sys.argv[2]))
    
    else:
        print(f"Error: Unknown function: {function}", file=sys.stderr)
        print("Available functions: hash, file_hash, command_fingerprint, generate", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()

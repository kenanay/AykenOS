#!/usr/bin/env python3

"""
Fail-Closed Marker Validation Script (AUTHORITATIVE VALIDATION LOGIC)

This is the SINGLE SOURCE OF TRUTH for fail-closed proof validation.
All validation logic resides here. The bash gate is orchestration only.

Validates QEMU kernel trace for fail-closed enforcement proof.
Implements sophisticated marker analysis with process identity tracking,
execution window validation, and negative guarantee checking.

Requirements: 16.1-16.15
"""

import sys
import re
import json
import os
from dataclasses import dataclass
from typing import Optional, List, Dict
from datetime import datetime
from pathlib import Path


@dataclass
class Marker:
    """Represents a kernel trace marker"""
    name: str
    line_number: int
    line_content: str
    process_id: Optional[int] = None
    timestamp: Optional[str] = None


# Standardized failure codes for CI integration
class FailureCode:
    """Standardized failure codes for precise CI reporting"""
    QEMU_TRACE_MISSING = "QEMU_TRACE_MISSING"
    INCOMPLETE_MARKER_FLOW = "INCOMPLETE_MARKER_FLOW"
    MARKER_SEQUENCE_OUT_OF_ORDER = "MARKER_SEQUENCE_OUT_OF_ORDER"
    PROCESS_IDENTITY_MISMATCH = "PROCESS_IDENTITY_MISMATCH"
    MULTIPLE_KILLS_DETECTED = "MULTIPLE_KILLS_DETECTED"
    ZERO_KILLS_DETECTED = "ZERO_KILLS_DETECTED"
    CONTINUATION_AFTER_KILL = "CONTINUATION_AFTER_KILL"
    UNBOUNDED_EXECUTION_WINDOW = "UNBOUNDED_EXECUTION_WINDOW"
    HARD_STOP_FAILED = "HARD_STOP_FAILED"
    PROCESS_ID_EXTRACTION_FAILED = "PROCESS_ID_EXTRACTION_FAILED"


class FailClosedProofValidator:
    """Validates fail-closed enforcement proof from QEMU kernel trace"""
    
    def __init__(self, trace_file: str):
        self.trace_file = trace_file
        self.trace_lines: List[str] = []
        self.violations: List[Dict] = []  # Changed to list of dicts with code + message
        self.warnings: List[str] = []
        self.failure_code: Optional[str] = None  # Primary failure code for CI
        
        # Markers
        self.marker_before: Optional[Marker] = None
        self.marker_enter: Optional[Marker] = None
        self.marker_kill: Optional[Marker] = None
        
        # Evidence output path (same directory as trace)
        trace_path = Path(trace_file)
        self.evidence_dir = trace_path.parent
        self.evidence_file = self.evidence_dir / "failclosed_proof_evidence.json"
        
    def add_violation(self, code: str, message: str):
        """Add a violation with standardized code"""
        self.violations.append({"code": code, "message": message})
        if not self.failure_code:  # Set first violation as primary failure code
            self.failure_code = code
    
    def load_trace(self) -> bool:
        """Load kernel trace file"""
        try:
            with open(self.trace_file, 'r', encoding='utf-8', errors='ignore') as f:
                self.trace_lines = f.readlines()
            print(f"[INFO] Loaded {len(self.trace_lines)} lines from trace")
            return True
        except FileNotFoundError:
            print(f"[ERROR] Trace file not found: {self.trace_file}")
            return False
        except Exception as e:
            print(f"[ERROR] Failed to load trace: {e}")
            return False
    
    def extract_process_id(self, line: str) -> Optional[int]:
        """Extract process ID from marker line"""
        # Try pid=N pattern
        match = re.search(r'pid=(\d+)', line)
        if match:
            return int(match.group(1))
        
        # Try Process N pattern
        match = re.search(r'Process (\d+)', line)
        if match:
            return int(match.group(1))
        
        return None
    
    def find_marker(self, pattern: str, marker_name: str) -> Optional[Marker]:
        """Find first occurrence of marker in trace"""
        for i, line in enumerate(self.trace_lines, start=1):
            if re.search(pattern, line):
                marker = Marker(
                    name=marker_name,
                    line_number=i,
                    line_content=line.strip(),
                    process_id=self.extract_process_id(line)
                )
                print(f"[INFO] Found {marker_name} at line {i}")
                if marker.process_id:
                    print(f"[INFO]   Process ID: {marker.process_id}")
                return marker
        return None
    
    def count_markers(self, pattern: str) -> int:
        """Count occurrences of marker pattern"""
        count = 0
        for line in self.trace_lines:
            if re.search(pattern, line):
                count += 1
        return count
    
    def validate_canonical_flow(self) -> bool:
        """Validate canonical marker flow exists and is ordered"""
        print("\n[TEST 1] Validating canonical marker flow...")
        
        # Find required markers
        self.marker_before = self.find_marker(
            r'BCIB_FORBIDDEN_BEFORE',
            'BCIB_FORBIDDEN_BEFORE'
        )
        self.marker_enter = self.find_marker(
            r'\[\[AYKEN_SYSCALL_ENTER\]\]',
            'AYKEN_SYSCALL_ENTER'
        )
        self.marker_kill = self.find_marker(
            r'\[\[AYKEN_BOUNDARY_KILL\]\]',
            'AYKEN_BOUNDARY_KILL'
        )
        
        # Check all markers present
        if not self.marker_before:
            self.add_violation(
                FailureCode.INCOMPLETE_MARKER_FLOW,
                "Missing required marker: BCIB_FORBIDDEN_BEFORE"
            )
            return False
        if not self.marker_enter:
            self.add_violation(
                FailureCode.INCOMPLETE_MARKER_FLOW,
                "Missing required marker: [[AYKEN_SYSCALL_ENTER]]"
            )
            return False
        if not self.marker_kill:
            self.add_violation(
                FailureCode.INCOMPLETE_MARKER_FLOW,
                "Missing required marker: [[AYKEN_BOUNDARY_KILL]]"
            )
            return False
        
        # Validate sequence order
        if not (self.marker_before.line_number < self.marker_enter.line_number < self.marker_kill.line_number):
            self.add_violation(
                FailureCode.MARKER_SEQUENCE_OUT_OF_ORDER,
                f"Marker sequence out of order: "
                f"BEFORE={self.marker_before.line_number}, "
                f"ENTER={self.marker_enter.line_number}, "
                f"KILL={self.marker_kill.line_number}"
            )
            return False
        
        print("[PASS] Canonical marker flow present and ordered")
        return True
    
    def validate_process_identity(self) -> bool:
        """Validate all markers belong to same process"""
        print("\n[TEST 2] Validating process identity consistency...")
        
        if not all([self.marker_before, self.marker_enter, self.marker_kill]):
            print("[SKIP] Cannot validate - markers missing")
            return False
        
        pid_before = self.marker_before.process_id
        pid_enter = self.marker_enter.process_id
        pid_kill = self.marker_kill.process_id
        
        if not all([pid_before, pid_enter, pid_kill]):
            self.add_violation(
                FailureCode.PROCESS_ID_EXTRACTION_FAILED,
                "Cannot extract process IDs from all markers - "
                "markers must include process_id for validation"
            )
            return False
        
        if not (pid_before == pid_enter == pid_kill):
            self.add_violation(
                FailureCode.PROCESS_IDENTITY_MISMATCH,
                f"Process identity mismatch: "
                f"BEFORE={pid_before}, ENTER={pid_enter}, KILL={pid_kill}. "
                f"All markers must belong to the SAME process. "
                f"This prevents exploit: Process A killed, Process B logs, gate incorrectly passes"
            )
            return False
        
        print(f"[PASS] Process identity consistent: pid={pid_before}")
        return True
    
    def validate_single_kill(self) -> bool:
        """Validate exactly one BOUNDARY_KILL marker"""
        print("\n[TEST 3] Validating single kill guarantee...")
        
        kill_count = self.count_markers(r'\[\[AYKEN_BOUNDARY_KILL\]\]')
        
        if kill_count == 0:
            self.add_violation(
                FailureCode.ZERO_KILLS_DETECTED,
                "Zero BOUNDARY_KILL markers - enforcement failed"
            )
            return False
        elif kill_count > 1:
            self.add_violation(
                FailureCode.MULTIPLE_KILLS_DETECTED,
                f"Multiple BOUNDARY_KILL markers found: {kill_count}. "
                f"This indicates unstable system, double execution, or race condition"
            )
            return False
        
        print(f"[PASS] Single kill guarantee: exactly 1 BOUNDARY_KILL")
        return True
    
    def validate_bounded_window(self, max_lines: int = 10) -> bool:
        """Validate execution window between ENTER and KILL is bounded"""
        print("\n[TEST 4] Validating bounded execution window...")
        
        if not (self.marker_enter and self.marker_kill):
            print("[SKIP] Cannot validate - markers missing")
            return False
        
        window_size = self.marker_kill.line_number - self.marker_enter.line_number
        
        print(f"[INFO] Execution window: {window_size} lines")
        
        if window_size > max_lines:
            self.add_violation(
                FailureCode.UNBOUNDED_EXECUTION_WINDOW,
                f"Execution window too large: {window_size} lines (max: {max_lines}). "
                f"This indicates system hang or delayed enforcement"
            )
            return False
        
        print(f"[PASS] Bounded execution window: {window_size} lines")
        return True
    
    def validate_negative_guarantees(self) -> bool:
        """Validate no continuation markers after BOUNDARY_KILL"""
        print("\n[TEST 5] Validating negative guarantees...")
        
        if not self.marker_kill:
            print("[SKIP] Cannot validate - KILL marker missing")
            return False
        
        # Get all lines after KILL marker
        after_kill_lines = self.trace_lines[self.marker_kill.line_number:]
        
        violations_found = False
        
        # Check for BCIB_FORBIDDEN_AFTER
        forbidden_after_count = sum(
            1 for line in after_kill_lines if 'BCIB_FORBIDDEN_AFTER' in line
        )
        if forbidden_after_count > 0:
            self.add_violation(
                FailureCode.CONTINUATION_AFTER_KILL,
                f"BCIB_FORBIDDEN_AFTER found after kill ({forbidden_after_count} times) - "
                f"execution continued"
            )
            violations_found = True
        
        # Check for SYSCALL_EXIT
        syscall_exit_count = sum(
            1 for line in after_kill_lines if '[[AYKEN_SYSCALL_EXIT]]' in line
        )
        if syscall_exit_count > 0:
            self.add_violation(
                FailureCode.CONTINUATION_AFTER_KILL,
                f"[[AYKEN_SYSCALL_EXIT]] found after kill ({syscall_exit_count} times) - "
                f"syscall returned instead of terminating"
            )
            violations_found = True
        
        # Check for SCHED_RESUME
        sched_resume_count = sum(
            1 for line in after_kill_lines if '[[AYKEN_SCHED_RESUME]]' in line
        )
        if sched_resume_count > 0:
            self.add_violation(
                FailureCode.CONTINUATION_AFTER_KILL,
                f"[[AYKEN_SCHED_RESUME]] found after kill ({sched_resume_count} times) - "
                f"process was rescheduled"
            )
            violations_found = True
        
        if violations_found:
            return False
        
        print("[PASS] No continuation markers after kill")
        return True
    
    def validate_hard_stop(self) -> bool:
        """Validate no logs from same process after BOUNDARY_KILL"""
        print("\n[TEST 6] Validating hard stop guarantee...")
        
        if not (self.marker_kill and self.marker_before):
            print("[SKIP] Cannot validate - markers missing")
            return False
        
        if not self.marker_before.process_id:
            print("[SKIP] Cannot validate - process ID not available")
            return False
        
        pid = self.marker_before.process_id
        after_kill_lines = self.trace_lines[self.marker_kill.line_number:]
        
        process_logs_after_kill = sum(
            1 for line in after_kill_lines if f'pid={pid}' in line or f'Process {pid}' in line
        )
        
        if process_logs_after_kill > 0:
            self.add_violation(
                FailureCode.HARD_STOP_FAILED,
                f"Process logs found after kill: {process_logs_after_kill} occurrences. "
                f"Process was not properly terminated - hard stop failed"
            )
            return False
        
        print("[PASS] No process logs after kill - hard stop verified")
        return True
    
    def validate_deterministic_error(self) -> bool:
        """Check for deterministic error code in trace"""
        print("\n[TEST 7] Validating deterministic error code...")
        
        error_patterns = [
            r'BCIB_ERR_\w+',
            r'BOUNDARY_ERR_\w+',
            r'ABDF_ERR_\w+'
        ]
        
        for line in self.trace_lines:
            for pattern in error_patterns:
                match = re.search(pattern, line)
                if match:
                    error_code = match.group(0)
                    print(f"[INFO] Deterministic error code found: {error_code}")
                    return True
        
        self.warnings.append("No deterministic error code found in trace")
        print("[WARN] No deterministic error code found")
        return True  # Warning, not failure
    
    def generate_report(self) -> Dict:
        """Generate validation report"""
        return {
            "gate": "ci-gate-fail-closed-proof",
            "timestamp": datetime.utcnow().isoformat() + "Z",
            "result": "PASS" if not self.violations else "FAIL",
            "failure_code": self.failure_code,  # Primary failure code for CI
            "violations_detected": len(self.violations),
            "warnings": len(self.warnings),
            "trace_file": self.trace_file,
            "trace_lines": len(self.trace_lines),
            "canonical_marker_flow": {
                "BCIB_FORBIDDEN_BEFORE": {
                    "found": self.marker_before is not None,
                    "line": self.marker_before.line_number if self.marker_before else None,
                    "process_id": self.marker_before.process_id if self.marker_before else None
                },
                "AYKEN_SYSCALL_ENTER": {
                    "found": self.marker_enter is not None,
                    "line": self.marker_enter.line_number if self.marker_enter else None,
                    "process_id": self.marker_enter.process_id if self.marker_enter else None
                },
                "AYKEN_BOUNDARY_KILL": {
                    "found": self.marker_kill is not None,
                    "line": self.marker_kill.line_number if self.marker_kill else None,
                    "process_id": self.marker_kill.process_id if self.marker_kill else None
                }
            },
            "violations": self.violations,
            "warnings": self.warnings,
            "requirements_validated": [
                "16.1: NON_OVERRIDABLE kernel-level validation",
                "16.2: Canonical marker flow enforcement",
                "16.3: BOUNDARY_KILL before scheduler removal",
                "16.4: Negative guarantee validation",
                "16.5-16.8: Process identity and single kill",
                "16.9-16.15: Bounded window and hard stop"
            ],
            "constitutional_compliance": {
                "KERNEL_SAFETY_CRITICAL": "enforced",
                "SECURITY_BOUNDARY_VIOLATION": "enforced"
            }
        }
    
    def run_validation(self) -> bool:
        """Run all validation tests"""
        print("=" * 60)
        print("Fail-Closed Proof Validation (AUTHORITATIVE)")
        print("=" * 60)
        
        if not self.load_trace():
            return False
        
        # Run all validation tests
        tests = [
            self.validate_canonical_flow,
            self.validate_process_identity,
            self.validate_single_kill,
            self.validate_bounded_window,
            self.validate_negative_guarantees,
            self.validate_hard_stop,
            self.validate_deterministic_error
        ]
        
        all_passed = True
        for test in tests:
            if not test():
                all_passed = False
        
        # Generate report
        report = self.generate_report()
        
        print("\n" + "=" * 60)
        print("Validation Summary")
        print("=" * 60)
        print(f"Result: {report['result']}")
        print(f"Violations: {report['violations_detected']}")
        print(f"Warnings: {report['warnings']}")
        if report['failure_code']:
            print(f"Primary Failure Code: {report['failure_code']}")
        
        if self.violations:
            print("\nViolations:")
            for i, violation in enumerate(self.violations, 1):
                print(f"  {i}. [{violation['code']}] {violation['message']}")
        
        if self.warnings:
            print("\nWarnings:")
            for i, warning in enumerate(self.warnings, 1):
                print(f"  {i}. {warning}")
        
        # Save report to evidence directory (authoritative location)
        with open(self.evidence_file, 'w') as f:
            json.dump(report, f, indent=2)
        print(f"\nEvidence saved: {self.evidence_file}")
        
        return all_passed


def main():
    if len(sys.argv) < 2:
        print("Usage: validate_fail_closed_markers.py <trace_file>")
        sys.exit(1)
    
    trace_file = sys.argv[1]
    validator = FailClosedProofValidator(trace_file)
    
    success = validator.run_validation()
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()

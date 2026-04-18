#!/usr/bin/env python3
"""
AykenOS Boundary Init Idempotency Test

Purpose: Verify that boundary_init_done flag prevents repeated initialization
across multiple syscalls. This is the CRITICAL preservation test for Task 2.

Expected Behavior (on unfixed code):
- First anchored syscall: DIAG_BOUNDARY_INIT_ENTER marker present
- Second anchored syscall: DIAG_BOUNDARY_INIT_SKIPPED marker present
- Third+ anchored syscalls: DIAG_BOUNDARY_INIT_SKIPPED marker present

Spec: scheduler-primary-regression-rca
Task: 2 - Preservation property tests
"""

import sys
import re
from pathlib import Path

def parse_debugcon_log(log_path):
    """Parse debugcon log and extract boundary init markers per syscall."""
    syscalls = []
    current_syscall = None
    
    with open(log_path, 'r', encoding='utf-8', errors='ignore') as f:
        for line in f:
            line = line.strip()
            
            # Detect syscall entry
            if '[[AYKEN_SYSCALL_ENTER]]' in line:
                if current_syscall:
                    syscalls.append(current_syscall)
                current_syscall = {
                    'init_enter': False,
                    'init_skipped': False,
                    'init_done': False
                }
            
            if not current_syscall:
                continue
            
            # Track boundary init markers
            if 'DIAG_BOUNDARY_INIT_ENTER' in line:
                current_syscall['init_enter'] = True
            elif 'DIAG_BOUNDARY_INIT_SKIPPED' in line:
                current_syscall['init_skipped'] = True
            elif 'DIAG_BOUNDARY_INIT_DONE' in line:
                current_syscall['init_done'] = True
    
    # Add last syscall
    if current_syscall:
        syscalls.append(current_syscall)
    
    return syscalls

def verify_idempotency(syscalls):
    """Verify boundary_init_done idempotency property."""
    if len(syscalls) < 2:
        return False, f"Need at least 2 syscalls, found {len(syscalls)}"
    
    # First syscall MUST take init path
    first = syscalls[0]
    if not first['init_enter']:
        return False, "First syscall did NOT take init path (missing DIAG_BOUNDARY_INIT_ENTER)"
    if not first['init_done']:
        return False, "First syscall did NOT complete init (missing DIAG_BOUNDARY_INIT_DONE)"
    if first['init_skipped']:
        return False, "First syscall incorrectly took skip path"
    
    # All subsequent syscalls MUST take skip path
    for i, syscall in enumerate(syscalls[1:], start=2):
        if not syscall['init_skipped']:
            return False, f"Syscall #{i} did NOT take skip path (missing DIAG_BOUNDARY_INIT_SKIPPED)"
        if syscall['init_enter']:
            return False, f"Syscall #{i} incorrectly took init path"
    
    return True, f"Idempotency verified: 1st init, {len(syscalls)-1} skip"

def main():
    if len(sys.argv) < 2:
        print("Usage: test_boundary_init_idempotency.py <debugcon.log>")
        sys.exit(1)
    
    log_path = Path(sys.argv[1])
    if not log_path.exists():
        print(f"ERROR: Log file not found: {log_path}")
        sys.exit(1)
    
    print("=" * 60)
    print("BOUNDARY INIT IDEMPOTENCY TEST")
    print("Spec: scheduler-primary-regression-rca")
    print("Task: 2 - Preservation property tests")
    print("=" * 60)
    print()
    
    syscalls = parse_debugcon_log(log_path)
    print(f"Total syscalls detected: {len(syscalls)}")
    print()
    
    if len(syscalls) < 2:
        print("❌ FAIL: Need at least 2 syscalls for idempotency test")
        sys.exit(1)
    
    # Show per-syscall breakdown
    print("Syscall Breakdown:")
    print("-" * 60)
    for i, syscall in enumerate(syscalls, start=1):
        status = "INIT" if syscall['init_enter'] else "SKIP"
        print(f"  Syscall #{i}: {status}")
        print(f"    init_enter:   {syscall['init_enter']}")
        print(f"    init_skipped: {syscall['init_skipped']}")
        print(f"    init_done:    {syscall['init_done']}")
    print()
    
    # Verify idempotency property
    success, message = verify_idempotency(syscalls)
    
    print("=" * 60)
    print("RESULT")
    print("=" * 60)
    if success:
        print(f"✅ PASS: {message}")
        print()
        print("Property Verified:")
        print("  → boundary_init_done flag prevents repeated initialization")
        print("  → First anchored syscall takes init path")
        print("  → Subsequent anchored syscalls take skip path")
        sys.exit(0)
    else:
        print(f"❌ FAIL: {message}")
        sys.exit(1)

if __name__ == '__main__':
    main()

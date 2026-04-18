#!/usr/bin/env python3
"""
AykenOS Skip Path Performance Test

Purpose: Verify that second syscall is significantly faster than first syscall
due to boundary_init skip path. This is a ratio-based preservation test.

Expected Behavior:
- Second anchored syscall kernel cost < first syscall kernel cost * 0.5
- Skip path marker present on second syscall
- Performance improvement is at least 50% (current evidence: 64.8%)

Spec: scheduler-primary-regression-rca
Task: 2 - Preservation property tests
"""

import sys
import re
from pathlib import Path

def parse_debugcon_log(log_path):
    """Parse debugcon log and extract kernel cost per syscall."""
    syscalls = []
    current_syscall = None
    timestamps = {}
    
    with open(log_path, 'r', encoding='utf-8', errors='ignore') as f:
        for line in f:
            line = line.strip()
            
            # Detect syscall entry
            if '[[AYKEN_SYSCALL_ENTER]]' in line:
                if current_syscall:
                    syscalls.append(current_syscall)
                current_syscall = {
                    'anchor_set': False,
                    'init_skipped': False,
                    'kernel_cost': None
                }
                timestamps = {}
            
            if not current_syscall:
                continue
            
            # Track anchor marker
            if 'DIAG_TEST_ANCHOR_SET' in line:
                current_syscall['anchor_set'] = True
            
            # Track skip marker
            if 'DIAG_BOUNDARY_INIT_SKIPPED' in line:
                current_syscall['init_skipped'] = True
            
            # Extract timestamps for kernel cost calculation
            # DIAG_KERNEL_HANDLER_ENTRY and DIAG_SYSCALL_RANGE_CHECK_DONE
            match = re.search(r'DIAG_KERNEL_HANDLER_ENTRY.*tsc=(\d+)', line)
            if match:
                timestamps['kernel_entry'] = int(match.group(1))
            
            match = re.search(r'DIAG_SYSCALL_RANGE_CHECK_DONE.*tsc=(\d+)', line)
            if match:
                timestamps['kernel_exit'] = int(match.group(1))
            
            # Calculate kernel cost if we have both timestamps
            if 'kernel_entry' in timestamps and 'kernel_exit' in timestamps:
                current_syscall['kernel_cost'] = timestamps['kernel_exit'] - timestamps['kernel_entry']
    
    # Add last syscall
    if current_syscall:
        syscalls.append(current_syscall)
    
    return syscalls

def verify_skip_path_performance(syscalls):
    """Verify skip path performance improvement."""
    # Filter to anchored syscalls with kernel cost
    anchored = [s for s in syscalls if s['anchor_set'] and s['kernel_cost'] is not None]
    
    if len(anchored) < 2:
        return False, f"Need at least 2 anchored syscalls with kernel cost, found {len(anchored)}"
    
    first = anchored[0]
    second = anchored[1]
    
    # Verify second syscall took skip path
    if not second['init_skipped']:
        return False, "Second anchored syscall did NOT take skip path"
    
    # Calculate performance improvement
    first_cost = first['kernel_cost']
    second_cost = second['kernel_cost']
    improvement = (first_cost - second_cost) / first_cost * 100
    
    # Verify at least 50% improvement
    if improvement < 50.0:
        return False, f"Skip path improvement {improvement:.1f}% < 50% threshold"
    
    return True, f"Skip path {improvement:.1f}% faster (first: {first_cost:,} ticks, second: {second_cost:,} ticks)"

def main():
    if len(sys.argv) < 2:
        print("Usage: test_skip_path_performance.py <debugcon.log>")
        sys.exit(1)
    
    log_path = Path(sys.argv[1])
    if not log_path.exists():
        print(f"ERROR: Log file not found: {log_path}")
        sys.exit(1)
    
    print("=" * 60)
    print("SKIP PATH PERFORMANCE TEST")
    print("Spec: scheduler-primary-regression-rca")
    print("Task: 2 - Preservation property tests")
    print("=" * 60)
    print()
    
    syscalls = parse_debugcon_log(log_path)
    print(f"Total syscalls detected: {len(syscalls)}")
    
    anchored = [s for s in syscalls if s['anchor_set'] and s['kernel_cost'] is not None]
    print(f"Anchored syscalls with kernel cost: {len(anchored)}")
    print()
    
    if len(anchored) < 2:
        print("❌ FAIL: Need at least 2 anchored syscalls with kernel cost")
        sys.exit(1)
    
    # Show per-syscall breakdown
    print("Anchored Syscall Performance:")
    print("-" * 60)
    for i, syscall in enumerate(anchored[:3], start=1):  # Show first 3
        path = "SKIP" if syscall['init_skipped'] else "INIT"
        print(f"  Syscall #{i}: {path}")
        print(f"    kernel_cost: {syscall['kernel_cost']:,} ticks")
    print()
    
    # Verify skip path performance property
    success, message = verify_skip_path_performance(syscalls)
    
    print("=" * 60)
    print("RESULT")
    print("=" * 60)
    if success:
        print(f"✅ PASS: {message}")
        print()
        print("Property Verified:")
        print("  → Second syscall is at least 50% faster than first")
        print("  → Skip path performance guarantee maintained")
        sys.exit(0)
    else:
        print(f"❌ FAIL: {message}")
        sys.exit(1)

if __name__ == '__main__':
    main()

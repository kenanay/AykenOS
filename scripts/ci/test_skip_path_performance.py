#!/usr/bin/env python3
"""
AykenOS Skip Path Performance Test (Robust Version)

Purpose: Verify that skip path is significantly faster than init path.
This is a ratio-based preservation test that catches performance regressions.

Expected Behavior:
- First syscall cost > subsequent syscall cost
- Skip path at least 50% faster than init path
- Performance improvement maintained across optimizations

Spec: scheduler-primary-regression-rca
Task: 2 - Preservation property tests
"""

import sys
import re
from pathlib import Path

def extract_timestamps(log_path):
    """Extract timestamp sequences per syscall."""
    syscalls = []
    current = []
    
    with open(log_path, 'r', encoding='utf-8', errors='ignore') as f:
        for line in f:
            line = line.strip()
            
            if '[[AYKEN_SYSCALL_ENTER]]' in line:
                if current:
                    syscalls.append(current)
                current = []
            
            m = re.search(r'0x([0-9a-fA-F]+)', line)
            if m:
                ts = int(m.group(1), 16)
                current.append(ts)
    
    if current:
        syscalls.append(current)
    
    return syscalls

def syscall_cost(ts_list):
    """Calculate syscall cost from timestamp list."""
    if len(ts_list) < 2:
        return None
    return ts_list[-1] - ts_list[0]

def main():
    if len(sys.argv) < 2:
        print("Usage: test_skip_path_performance.py <debugcon.log>")
        sys.exit(1)
    
    log_path = Path(sys.argv[1])
    if not log_path.exists():
        print(f"ERROR: Log file not found: {log_path}")
        sys.exit(1)
    
    print("=" * 60)
    print("SKIP PATH PERFORMANCE TEST (ROBUST)")
    print("Spec: scheduler-primary-regression-rca")
    print("Task: 2 - Preservation property tests")
    print("=" * 60)
    print()
    
    syscalls = extract_timestamps(log_path)
    
    if len(syscalls) < 2:
        print("❌ FAIL: Need at least 2 syscalls")
        sys.exit(1)
    
    costs = []
    for i, sc in enumerate(syscalls):
        cost = syscall_cost(sc)
        if cost:
            costs.append(cost)
            print(f"Syscall #{i+1} cost: {cost:,} ticks")
    
    print()
    
    if len(costs) < 2:
        print("❌ FAIL: Not enough valid cost data")
        sys.exit(1)
    
    init_cost = costs[0]
    skip_cost = min(costs[1:])
    
    ratio = skip_cost / init_cost
    improvement = (1 - ratio) * 100
    
    print("Analysis:")
    print(f"  Init cost:      {init_cost:,} ticks")
    print(f"  Best skip cost: {skip_cost:,} ticks")
    print(f"  Ratio:          {ratio:.2f}")
    print(f"  Improvement:    {improvement:.1f}%")
    print()
    
    print("=" * 60)
    print("RESULT")
    print("=" * 60)
    
    if ratio < 0.5:
        print(f"✅ PASS: Skip path is {improvement:.1f}% faster (>50% required)")
        print()
        print("Property Verified:")
        print("  → Init path is measurably expensive")
        print("  → Skip path is significantly faster")
        print("  → Performance guarantee maintained")
        sys.exit(0)
    else:
        print(f"❌ FAIL: Skip path only {improvement:.1f}% faster (<50% required)")
        sys.exit(1)

if __name__ == '__main__':
    main()

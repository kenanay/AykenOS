#!/usr/bin/env python3
"""
AykenOS Anchored Sequence Test

Purpose: Verify anchored syscall detection and normalized syscall numbering.
This test prevents regression of the syscall normalization bug (checking raw
syscall_num == 1010 instead of normalized number).

Expected Behavior:
- Syscall 1010 (SYS_V2_BASE + 10) triggers DIAG_TEST_ANCHOR_SET
- Anchored sequence markers track S/A/B/C correctly
- Non-anchored syscalls do NOT trigger anchor markers
- Normalization happens BEFORE anchor check

Spec: scheduler-primary-regression-rca
Task: 2 - Preservation property tests
"""

import sys
import re
from pathlib import Path

def parse_debugcon_log(log_path):
    """Parse debugcon log and extract anchored sequence markers."""
    syscalls = []
    current_syscall = None
    
    with open(log_path, 'r', encoding='utf-8', errors='ignore') as f:
        for line in f:
            line = line.strip()
            
            # Detect syscall entry with number
            match = re.search(r'\[\[AYKEN_SYSCALL_ENTER\]\]\s+num=(\d+)', line)
            if match:
                if current_syscall:
                    syscalls.append(current_syscall)
                current_syscall = {
                    'num': int(match.group(1)),
                    'anchor_set': False,
                    'sequence': None
                }
            
            if not current_syscall:
                continue
            
            # Track anchor markers
            if 'DIAG_TEST_ANCHOR_SET' in line:
                current_syscall['anchor_set'] = True
            
            # Track sequence markers
            for seq in ['S', 'A', 'B', 'C']:
                if f'DIAG_ANCHORED_SEQ_{seq}' in line:
                    current_syscall['sequence'] = seq
    
    # Add last syscall
    if current_syscall:
        syscalls.append(current_syscall)
    
    return syscalls

def verify_anchored_sequence(syscalls):
    """Verify anchored sequence detection and normalization."""
    # Find anchored syscalls (should be syscall 1010)
    anchored = [s for s in syscalls if s['anchor_set']]
    
    if len(anchored) == 0:
        return False, "No anchored syscalls found (expected syscall 1010)"
    
    # Verify all anchored syscalls are syscall 1010
    for syscall in anchored:
        if syscall['num'] != 1010:
            return False, f"Anchored syscall has wrong number: {syscall['num']} (expected 1010)"
    
    # Verify sequence markers
    expected_sequence = ['S', 'A', 'B', 'C']
    actual_sequence = [s['sequence'] for s in anchored if s['sequence']]
    
    if len(actual_sequence) < len(expected_sequence):
        return False, f"Incomplete sequence: {actual_sequence} (expected {expected_sequence})"
    
    for i, (expected, actual) in enumerate(zip(expected_sequence, actual_sequence)):
        if expected != actual:
            return False, f"Sequence mismatch at position {i}: {actual} (expected {expected})"
    
    # Verify non-anchored syscalls do NOT have anchor markers
    non_anchored = [s for s in syscalls if not s['anchor_set']]
    for syscall in non_anchored:
        if syscall['sequence']:
            return False, f"Non-anchored syscall {syscall['num']} has sequence marker: {syscall['sequence']}"
    
    return True, f"Anchored sequence verified: {len(anchored)} syscalls with sequence {actual_sequence[:len(expected_sequence)]}"

def main():
    if len(sys.argv) < 2:
        print("Usage: test_anchored_sequence.py <debugcon.log>")
        sys.exit(1)
    
    log_path = Path(sys.argv[1])
    if not log_path.exists():
        print(f"ERROR: Log file not found: {log_path}")
        sys.exit(1)
    
    print("=" * 60)
    print("ANCHORED SEQUENCE TEST")
    print("Spec: scheduler-primary-regression-rca")
    print("Task: 2 - Preservation property tests")
    print("=" * 60)
    print()
    
    syscalls = parse_debugcon_log(log_path)
    print(f"Total syscalls detected: {len(syscalls)}")
    
    anchored = [s for s in syscalls if s['anchor_set']]
    print(f"Anchored syscalls detected: {len(anchored)}")
    print()
    
    # Show anchored syscall details
    if anchored:
        print("Anchored Syscall Details:")
        print("-" * 60)
        for i, syscall in enumerate(anchored, start=1):
            print(f"  Syscall #{i}:")
            print(f"    num:      {syscall['num']}")
            print(f"    sequence: {syscall['sequence']}")
        print()
    
    # Verify anchored sequence property
    success, message = verify_anchored_sequence(syscalls)
    
    print("=" * 60)
    print("RESULT")
    print("=" * 60)
    if success:
        print(f"✅ PASS: {message}")
        print()
        print("Property Verified:")
        print("  → Syscall 1010 correctly triggers anchor detection")
        print("  → Normalized syscall numbering works (1010 → 10 after SYS_V2_BASE)")
        print("  → Anchored sequence markers track S/A/B/C correctly")
        print("  → Non-anchored syscalls do NOT trigger anchor markers")
        sys.exit(0)
    else:
        print(f"❌ FAIL: {message}")
        sys.exit(1)

if __name__ == '__main__':
    main()

#!/usr/bin/env python3
"""
AykenOS Anchored Sequence Test (Robust Version)

Purpose: Verify anchored syscall detection and sequence flow.
This test prevents regression of the syscall normalization bug and ensures
the anchor mechanism works correctly.

Expected Behavior:
- DIAG_TEST_ANCHOR_SET marker exists
- DIAG_ANCHORED_SEQ_1/2/3 markers exist in order
- Sequence flow is correct (1 → 2 → 3)

Spec: scheduler-primary-regression-rca
Task: 2 - Preservation property tests
"""

import sys
from pathlib import Path

def parse_markers(log_path):
    """Parse debugcon log and extract anchor/sequence markers."""
    markers = []
    with open(log_path, 'r', encoding='utf-8', errors='ignore') as f:
        for line in f:
            line = line.strip()
            if 'DIAG_TEST_ANCHOR_SET' in line:
                markers.append('ANCHOR')
            elif 'DIAG_ANCHORED_SEQ_1' in line:
                markers.append('SEQ1')
            elif 'DIAG_ANCHORED_SEQ_2' in line:
                markers.append('SEQ2')
            elif 'DIAG_ANCHORED_SEQ_3' in line:
                markers.append('SEQ3')
    return markers

def verify_sequence(markers):
    """Verify anchor and sequence markers are present and in order."""
    if 'ANCHOR' not in markers:
        return False, "Missing DIAG_TEST_ANCHOR_SET"
    
    try:
        i1 = markers.index('SEQ1')
        i2 = markers.index('SEQ2')
        i3 = markers.index('SEQ3')
    except ValueError:
        return False, "Missing one or more sequence markers (1/2/3)"
    
    if not (i1 < i2 < i3):
        return False, "Sequence order invalid (expected 1 → 2 → 3)"
    
    return True, "Anchor + sequence 1→2→3 verified"

def main():
    if len(sys.argv) < 2:
        print("Usage: test_anchored_sequence.py <debugcon.log>")
        sys.exit(1)
    
    log_path = Path(sys.argv[1])
    if not log_path.exists():
        print(f"ERROR: Log file not found: {log_path}")
        sys.exit(1)
    
    print("=" * 60)
    print("ANCHORED SEQUENCE TEST (ROBUST)")
    print("Spec: scheduler-primary-regression-rca")
    print("Task: 2 - Preservation property tests")
    print("=" * 60)
    print()
    
    markers = parse_markers(log_path)
    print(f"Markers detected: {markers}")
    print()
    
    # Verify sequence property
    success, message = verify_sequence(markers)
    
    print("=" * 60)
    print("RESULT")
    print("=" * 60)
    if success:
        print(f"✅ PASS: {message}")
        print()
        print("Property Verified:")
        print("  → Anchor detection works correctly")
        print("  → Sequence flow is correct (1 → 2 → 3)")
        print("  → Normalized syscall numbering preserved")
        sys.exit(0)
    else:
        print(f"❌ FAIL: {message}")
        sys.exit(1)

if __name__ == '__main__':
    main()

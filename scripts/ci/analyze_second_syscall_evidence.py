#!/usr/bin/env python3
"""
AykenOS Second Syscall Evidence Analyzer

Purpose: Verify that boundary_init_done flag works correctly by analyzing
debugcon log for init path vs skip path markers across multiple syscalls.

Expected evidence:
  1st syscall: DIAG_BOUNDARY_INIT_ENTER (init path taken)
  2nd syscall: DIAG_BOUNDARY_INIT_SKIPPED (skip path taken)
  3rd syscall: DIAG_BOUNDARY_INIT_SKIPPED (skip path taken)

Exit codes:
  0 - PASS: Evidence confirms correct behavior
  1 - FAIL: Evidence shows incorrect behavior (flag broken)
  2 - INCONCLUSIVE: Insufficient evidence
"""

import sys
from pathlib import Path

def parse_syscall_evidence(log_lines):
    """Parse debugcon log to extract anchored syscall sequence
    
    This approach uses kernel-side anchored sequence markers that are emitted
    after the test anchor ('S') is detected. This ensures the sequence is
    robust against early boot syscalls.
    
    Expected markers:
      DIAG_TEST_ANCHOR_SET - anchor syscall detected
      DIAG_ANCHORED_SEQ_1 - first syscall after anchor (A)
      DIAG_ANCHORED_SEQ_2 - second syscall after anchor (B)
      DIAG_ANCHORED_SEQ_3 - third syscall after anchor (C)
    """
    
    syscalls = {}  # seq_num -> {'init': bool, 'skip': bool, 'kernel_entry': timestamp, 'range_check_done': timestamp}
    current_seq = None
    anchor_seen = False
    
    for line in log_lines:
        line = line.strip()
        
        # Detect anchor
        if 'DIAG_TEST_ANCHOR_SET' in line:
            anchor_seen = True
            continue
        
        if not anchor_seen:
            continue
        
        # Detect anchored sequence markers
        if 'DIAG_ANCHORED_SEQ_1' in line:
            current_seq = 1
            if current_seq not in syscalls:
                syscalls[current_seq] = {'init': False, 'skip': False}
        elif 'DIAG_ANCHORED_SEQ_2' in line:
            current_seq = 2
            if current_seq not in syscalls:
                syscalls[current_seq] = {'init': False, 'skip': False}
        elif 'DIAG_ANCHORED_SEQ_3' in line:
            current_seq = 3
            if current_seq not in syscalls:
                syscalls[current_seq] = {'init': False, 'skip': False}
        
        # Detect init/skip markers within current syscall
        if current_seq is not None:
            if 'DIAG_BOUNDARY_INIT_ENTER' in line:
                syscalls[current_seq]['init'] = True
            elif 'DIAG_BOUNDARY_INIT_SKIPPED' in line:
                syscalls[current_seq]['skip'] = True
            
            # Optional: capture timestamps for performance comparison
            if 'DIAG_KERNEL_HANDLER_ENTRY' in line:
                parts = line.split()
                if len(parts) >= 2:
                    try:
                        syscalls[current_seq]['kernel_entry'] = int(parts[1], 16)
                    except ValueError:
                        pass
            elif 'DIAG_SYSCALL_RANGE_CHECK_DONE' in line:
                parts = line.split()
                if len(parts) >= 2:
                    try:
                        syscalls[current_seq]['range_check_done'] = int(parts[1], 16)
                    except ValueError:
                        pass
    
    return syscalls

def analyze_evidence(syscalls):
    """Analyze syscall evidence and determine PASS/FAIL/INCONCLUSIVE"""
    
    print("\n" + "="*60)
    print("SECOND SYSCALL EVIDENCE ANALYSIS")
    print("Spec: scheduler-primary-regression-rca")
    print("Purpose: Verify boundary_init_done flag behavior")
    print("="*60)
    
    if not syscalls:
        print("\n❌ INCONCLUSIVE: No anchored syscall sequence found")
        print("   → Test anchor may not have been detected")
        print("   → Check for DIAG_TEST_ANCHOR_SET in log")
        return 2
    
    print(f"\nDetected {len(syscalls)} anchored syscall(s):")
    for seq_num in sorted(syscalls.keys()):
        info = syscalls[seq_num]
        init_status = "✓ INIT" if info['init'] else "✗ no init"
        skip_status = "✓ SKIP" if info['skip'] else "✗ no skip"
        print(f"  Syscall {seq_num}: {init_status}, {skip_status}")
        
        # Check for conflicting markers
        if info['init'] and info['skip']:
            print(f"              ⚠ CONFLICT: Both init and skip markers present")
        
        # Optional: show kernel cost if timestamps available
        if 'kernel_entry' in info and 'range_check_done' in info:
            kernel_cost = info['range_check_done'] - info['kernel_entry']
            print(f"              Kernel cost: {kernel_cost:,} ticks")
    
    # Verification logic
    print("\n" + "="*60)
    print("VERIFICATION")
    print("="*60)
    
    # Check if we have at least 2 syscalls
    if len(syscalls) < 2:
        print("\n❌ INCONCLUSIVE: Need at least 2 anchored syscalls for evidence")
        print(f"   → Only {len(syscalls)} syscall(s) detected after anchor")
        return 2
    
    # Check 1st anchored syscall (A)
    if 1 not in syscalls:
        print("\n❌ INCONCLUSIVE: 1st anchored syscall not detected")
        return 2
    
    first = syscalls[1]
    
    if first['init'] and first['skip']:
        print("\n❌ FAIL: 1st syscall has conflicting markers")
        print("   → Both INIT and SKIP markers present (logic error)")
        return 1
    
    if not first['init']:
        print("\n❌ FAIL: 1st syscall did NOT take init path")
        print("   → Expected: DIAG_BOUNDARY_INIT_ENTER")
        print("   → This indicates init path is never taken (critical bug)")
        return 1
    
    if first['skip']:
        print("\n❌ FAIL: 1st syscall took skip path (should be init path)")
        print("   → Expected: DIAG_BOUNDARY_INIT_ENTER only")
        print("   → Got: Both init and skip markers (logic error)")
        return 1
    
    print("\n✓ 1st anchored syscall correctly took init path")
    
    # Check 2nd anchored syscall (B)
    if 2 not in syscalls:
        print("\n❌ INCONCLUSIVE: 2nd anchored syscall not detected")
        return 2
    
    second = syscalls[2]
    
    if second['init'] and second['skip']:
        print("\n❌ FAIL: 2nd syscall has conflicting markers")
        print("   → Both INIT and SKIP markers present (logic error)")
        return 1
    
    if second['init']:
        print("\n❌ FAIL: 2nd syscall took init path (should be skip path)")
        print("   → Expected: DIAG_BOUNDARY_INIT_SKIPPED")
        print("   → Got: DIAG_BOUNDARY_INIT_ENTER")
        print("   → Flag persistence failure observed")
        print("   → Init path repeats across observed syscalls")
        print("   → This may explain part of the regression")
        return 1
    
    if not second['skip']:
        print("\n❌ FAIL: 2nd syscall did NOT take skip path")
        print("   → Expected: DIAG_BOUNDARY_INIT_SKIPPED")
        print("   → Neither init nor skip marker found (instrumentation issue)")
        return 1
    
    print("✓ 2nd anchored syscall correctly took skip path")
    
    # Optional: check 3rd syscall if available
    if 3 in syscalls:
        third = syscalls[3]
        if third['init'] and third['skip']:
            print("\n⚠ WARNING: 3rd syscall has conflicting markers")
        elif third['init']:
            print("\n⚠ WARNING: 3rd syscall took init path (unexpected)")
        elif third['skip']:
            print("✓ 3rd syscall correctly took skip path")
        else:
            print("⚠ WARNING: 3rd syscall path unclear")
    
    # Performance comparison (if timestamps available)
    if 'kernel_entry' in syscalls[1] and 'range_check_done' in syscalls[1] and \
       'kernel_entry' in syscalls[2] and 'range_check_done' in syscalls[2]:
        cost1 = syscalls[1]['range_check_done'] - syscalls[1]['kernel_entry']
        cost2 = syscalls[2]['range_check_done'] - syscalls[2]['kernel_entry']
        
        print("\n" + "="*60)
        print("PERFORMANCE COMPARISON")
        print("="*60)
        print(f"\n  1st syscall kernel cost: {cost1:,} ticks (init path)")
        print(f"  2nd syscall kernel cost: {cost2:,} ticks (skip path)")
        
        if cost2 < cost1:
            reduction = cost1 - cost2
            reduction_pct = (reduction / cost1) * 100
            print(f"\n  ✓ Skip path is faster: -{reduction:,} ticks (-{reduction_pct:.1f}%)")
            print("    → This confirms init path is expensive and skip path works")
        else:
            increase = cost2 - cost1
            increase_pct = (increase / cost1) * 100
            print(f"\n  ⚠ Skip path is NOT faster: +{increase:,} ticks (+{increase_pct:.1f}%)")
            print("    → This suggests skip path may have other overhead")
    
    print("\n" + "="*60)
    print("CONCLUSION")
    print("="*60)
    print("\n✅ PASS: boundary_init_done flag works correctly")
    print("   → 1st anchored syscall takes init path")
    print("   → 2nd anchored syscall takes skip path")
    print("   → Flag behavior confirmed; first-syscall init is not repeated")
    print("\n   → Next: Proceed to Task 2 (preservation tests)")
    print("="*60 + "\n")
    
    return 0

def main():
    if len(sys.argv) < 2:
        print("Usage: analyze_second_syscall_evidence.py <debugcon.log>")
        sys.exit(2)
    
    log_file = Path(sys.argv[1])
    
    if not log_file.exists():
        print(f"ERROR: Log file not found: {log_file}")
        sys.exit(2)
    
    # Parse log
    log_lines = log_file.read_text(errors='ignore').splitlines()
    syscalls = parse_syscall_evidence(log_lines)
    
    # Analyze and exit with appropriate code
    exit_code = analyze_evidence(syscalls)
    sys.exit(exit_code)

if __name__ == "__main__":
    main()

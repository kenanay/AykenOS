#!/usr/bin/env python3
"""
AykenOS Enforcement Hot-Path Micro-Profile Analyzer

Purpose: Measure per-syscall enforcement overhead by analyzing hot-path markers.

Target markers (per syscall):
  DIAG_HOT_CTX_TYPE_ENTER / DONE - boundary_set_context_type() cost
  DIAG_HOT_VALIDATE_SYSCALL_ENTER / DONE - boundary_validate_syscall() cost
  DIAG_HOT_BYPASS_CHECK_ENTER / DONE - boundary_detect_bridge_bypass() cost
  DIAG_HOT_BCIB_SUBMIT_ENTER / DONE - boundary_check_bcib_submission_path() cost

Exit codes:
  0 - PASS: Analysis complete with evidence
  1 - FAIL: Insufficient evidence
"""

import sys
from pathlib import Path

def parse_timestamp(line):
    """Extract RDTSC timestamp from marker line"""
    parts = line.split()
    if len(parts) >= 2:
        try:
            return int(parts[1], 16)
        except ValueError:
            pass
    return None

def parse_hotpath_evidence(log_lines):
    """Parse debugcon log to extract hot-path timing evidence
    
    Returns dict of syscalls with hot-path segment costs:
      {
        1: {
          'ctx_type': cost_ticks,
          'validate_syscall': cost_ticks,
          'bypass_check': cost_ticks,
          'bcib_submit': cost_ticks (if present),
          'total_hotpath': sum_ticks
        },
        ...
      }
    """
    
    syscalls = {}
    current_seq = None
    anchor_seen = False
    
    # Temporary storage for enter timestamps
    ctx_type_enter = None
    validate_enter = None
    bypass_enter = None
    bcib_enter = None
    
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
                syscalls[current_seq] = {}
        elif 'DIAG_ANCHORED_SEQ_2' in line:
            current_seq = 2
            if current_seq not in syscalls:
                syscalls[current_seq] = {}
        elif 'DIAG_ANCHORED_SEQ_3' in line:
            current_seq = 3
            if current_seq not in syscalls:
                syscalls[current_seq] = {}
        
        if current_seq is None:
            continue
        
        # Parse hot-path markers
        if 'DIAG_HOT_CTX_TYPE_ENTER' in line:
            ctx_type_enter = parse_timestamp(line)
        elif 'DIAG_HOT_CTX_TYPE_DONE' in line and ctx_type_enter is not None:
            done_ts = parse_timestamp(line)
            if done_ts is not None:
                syscalls[current_seq]['ctx_type'] = done_ts - ctx_type_enter
            ctx_type_enter = None
        
        elif 'DIAG_HOT_VALIDATE_SYSCALL_ENTER' in line:
            validate_enter = parse_timestamp(line)
        elif 'DIAG_HOT_VALIDATE_SYSCALL_DONE' in line and validate_enter is not None:
            done_ts = parse_timestamp(line)
            if done_ts is not None:
                syscalls[current_seq]['validate_syscall'] = done_ts - validate_enter
            validate_enter = None
        
        elif 'DIAG_HOT_BYPASS_CHECK_ENTER' in line:
            bypass_enter = parse_timestamp(line)
        elif 'DIAG_HOT_BYPASS_CHECK_DONE' in line and bypass_enter is not None:
            done_ts = parse_timestamp(line)
            if done_ts is not None:
                syscalls[current_seq]['bypass_check'] = done_ts - bypass_enter
            bypass_enter = None
        
        elif 'DIAG_HOT_BCIB_SUBMIT_ENTER' in line:
            bcib_enter = parse_timestamp(line)
        elif 'DIAG_HOT_BCIB_SUBMIT_DONE' in line and bcib_enter is not None:
            done_ts = parse_timestamp(line)
            if done_ts is not None:
                syscalls[current_seq]['bcib_submit'] = done_ts - bcib_enter
            bcib_enter = None
    
    # Calculate total hot-path cost for each syscall
    for seq, data in syscalls.items():
        total = 0
        if 'ctx_type' in data:
            total += data['ctx_type']
        if 'validate_syscall' in data:
            total += data['validate_syscall']
        if 'bypass_check' in data:
            total += data['bypass_check']
        if 'bcib_submit' in data:
            total += data['bcib_submit']
        data['total_hotpath'] = total
    
    return syscalls

def analyze_hotpath(syscalls):
    """Analyze hot-path evidence and report findings"""
    
    print("\n" + "="*60)
    print("ENFORCEMENT HOT-PATH MICRO-PROFILE")
    print("Spec: scheduler-primary-regression-rca")
    print("Purpose: Identify per-syscall enforcement bottleneck")
    print("="*60)
    
    if not syscalls:
        print("\n❌ FAIL: No anchored syscall sequence found")
        return 1
    
    print(f"\nDetected {len(syscalls)} anchored syscall(s)")
    print("\n" + "="*60)
    print("HOT-PATH SEGMENT BREAKDOWN")
    print("="*60)
    
    # Aggregate statistics
    ctx_type_costs = []
    validate_costs = []
    bypass_costs = []
    bcib_costs = []
    total_costs = []
    
    for seq in sorted(syscalls.keys()):
        data = syscalls[seq]
        print(f"\nSyscall #{seq}:")
        
        if 'ctx_type' in data:
            print(f"  ctx_type:         {data['ctx_type']:>10,} ticks")
            ctx_type_costs.append(data['ctx_type'])
        else:
            print(f"  ctx_type:         {'N/A':>10}")
        
        if 'validate_syscall' in data:
            print(f"  validate_syscall: {data['validate_syscall']:>10,} ticks")
            validate_costs.append(data['validate_syscall'])
        else:
            print(f"  validate_syscall: {'N/A':>10}")
        
        if 'bypass_check' in data:
            print(f"  bypass_check:     {data['bypass_check']:>10,} ticks")
            bypass_costs.append(data['bypass_check'])
        else:
            print(f"  bypass_check:     {'N/A':>10}")
        
        if 'bcib_submit' in data:
            print(f"  bcib_submit:      {data['bcib_submit']:>10,} ticks")
            bcib_costs.append(data['bcib_submit'])
        
        if 'total_hotpath' in data:
            print(f"  TOTAL HOT-PATH:   {data['total_hotpath']:>10,} ticks")
            total_costs.append(data['total_hotpath'])
    
    # Summary statistics
    print("\n" + "="*60)
    print("AGGREGATE STATISTICS (across all syscalls)")
    print("="*60)
    
    def print_stats(name, costs):
        if not costs:
            print(f"\n{name}: N/A")
            return
        avg = sum(costs) / len(costs)
        min_cost = min(costs)
        max_cost = max(costs)
        print(f"\n{name}:")
        print(f"  Min:  {min_cost:>10,} ticks")
        print(f"  Max:  {max_cost:>10,} ticks")
        print(f"  Avg:  {avg:>10,.0f} ticks")
    
    print_stats("ctx_type", ctx_type_costs)
    print_stats("validate_syscall", validate_costs)
    print_stats("bypass_check", bypass_costs)
    if bcib_costs:
        print_stats("bcib_submit", bcib_costs)
    print_stats("TOTAL HOT-PATH", total_costs)
    
    # Identify bottleneck
    print("\n" + "="*60)
    print("BOTTLENECK ANALYSIS")
    print("="*60)
    
    if total_costs:
        avg_total = sum(total_costs) / len(total_costs)
        
        segments = []
        if ctx_type_costs:
            avg_ctx = sum(ctx_type_costs) / len(ctx_type_costs)
            pct = (avg_ctx / avg_total) * 100
            segments.append(('ctx_type', avg_ctx, pct))
        
        if validate_costs:
            avg_val = sum(validate_costs) / len(validate_costs)
            pct = (avg_val / avg_total) * 100
            segments.append(('validate_syscall', avg_val, pct))
        
        if bypass_costs:
            avg_byp = sum(bypass_costs) / len(bypass_costs)
            pct = (avg_byp / avg_total) * 100
            segments.append(('bypass_check', avg_byp, pct))
        
        if bcib_costs:
            avg_bcib = sum(bcib_costs) / len(bcib_costs)
            pct = (avg_bcib / avg_total) * 100
            segments.append(('bcib_submit', avg_bcib, pct))
        
        # Sort by percentage descending
        segments.sort(key=lambda x: x[2], reverse=True)
        
        print("\nHot-path segments ranked by cost:")
        for name, cost, pct in segments:
            marker = "🔥" if pct > 30 else "⚠️" if pct > 15 else "✓"
            print(f"  {marker} {name:20} {cost:>10,.0f} ticks ({pct:>5.1f}%)")
        
        # Identify primary bottleneck
        if segments:
            primary = segments[0]
            print(f"\n🎯 PRIMARY BOTTLENECK: {primary[0]} ({primary[2]:.1f}% of hot-path)")
    
    print("\n" + "="*60)
    print("CONCLUSION")
    print("="*60)
    
    if total_costs:
        avg_total = sum(total_costs) / len(total_costs)
        print(f"\n✅ PASS: Hot-path micro-profile complete")
        print(f"   → Average hot-path cost: {avg_total:,.0f} ticks per syscall")
        print(f"   → Measured across {len(syscalls)} syscall(s)")
        print(f"\nNext steps:")
        print(f"  1. Optimize primary bottleneck segment")
        print(f"  2. Consider caching/fast-path for high-cost operations")
        print(f"  3. Re-measure after optimization")
        return 0
    else:
        print(f"\n❌ FAIL: Insufficient hot-path evidence")
        return 1

def main():
    if len(sys.argv) < 2:
        print("Usage: analyze_enforcement_hotpath.py <debugcon.log>")
        return 2
    
    log_path = Path(sys.argv[1])
    if not log_path.exists():
        print(f"Error: Log file not found: {log_path}")
        return 2
    
    with open(log_path, 'r', errors='ignore') as f:
        log_lines = f.readlines()
    
    syscalls = parse_hotpath_evidence(log_lines)
    return analyze_hotpath(syscalls)

if __name__ == '__main__':
    sys.exit(main())

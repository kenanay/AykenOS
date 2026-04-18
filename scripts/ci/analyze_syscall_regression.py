#!/usr/bin/env python3
"""
AykenOS Scheduler Primary Regression RCA - Granular Kernel Segment Analysis

**CRITICAL**: This script analyzes EXISTING + DIAGNOSTIC markers to identify
which sub-segment within KERNEL_COST is causing the +115.6% regression.

**Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6**

Property 1: Bug Condition - Kernel Sub-Segment Regression Detection

This script uses markers to calculate kernel sub-segment costs:
- FIRST_SYSCALL_ENTRY: Syscall handler start (AYKEN_SYSCALL_ENTER)
- DIAG_KERNEL_HANDLER_ENTRY: Handler entry point
- DIAG_CONTEXT_DETECTION_DONE: Context detection complete
- DIAG_BOUNDARY_INIT_DONE: Boundary init complete
- DIAG_CONTEXT_REGISTRATION_DONE: Context registration complete
- DIAG_BOUNDARY_VALIDATE_DONE: Boundary validation complete
- DIAG_BRIDGE_BYPASS_CHECK_DONE: Bridge bypass check complete
- DIAG_BCIB_SUBMISSION_CHECK_DONE: BCIB submission check complete
- DIAG_SYSCALL_RANGE_CHECK_DONE: Syscall range check complete
- FIRST_SYSCALL_EXIT: Syscall handler end (AYKEN_SYSCALL_RETURN)

Derived Kernel Sub-Segment Metrics:
- CONTEXT_DETECTION_COST = CONTEXT_DETECTION_DONE - KERNEL_HANDLER_ENTRY
- BOUNDARY_INIT_COST = BOUNDARY_INIT_DONE - CONTEXT_DETECTION_DONE
- CONTEXT_REGISTRATION_COST = CONTEXT_REGISTRATION_DONE - BOUNDARY_INIT_DONE
- BOUNDARY_VALIDATE_COST = BOUNDARY_VALIDATE_DONE - CONTEXT_REGISTRATION_DONE
- BRIDGE_BYPASS_CHECK_COST = BRIDGE_BYPASS_CHECK_DONE - BOUNDARY_VALIDATE_DONE
- BCIB_SUBMISSION_CHECK_COST = BCIB_SUBMISSION_CHECK_DONE - BRIDGE_BYPASS_CHECK_DONE
- SYSCALL_RANGE_CHECK_COST = SYSCALL_RANGE_CHECK_DONE - BCIB_SUBMISSION_CHECK_DONE
- DISPATCH_AND_HANDLER_COST = SYSCALL_EXIT - SYSCALL_RANGE_CHECK_DONE

Expected Outcome: Identify which kernel sub-segment(s) are inflated
"""

import json
import sys
from pathlib import Path

# Baseline metrics from scripts/ci/perf-baseline.lock.json
BASELINE_METRICS = {
    "first_user_entry_to_first_syscall_gate_entry": 21735640,  # ticks
    "first_syscall_gate_entry_to_first_syscall_entry": 1915974,  # ticks (ENTRY segment)
    "first_syscall_gate_entry_to_first_syscall_exit": 2926525,  # ticks (ENTRY + KERNEL)
    "first_user_entry_to_first_syscall_entry": 23651614,  # ticks
    "first_user_entry_to_first_syscall_exit": 24662165,  # ticks
}

# Derived baseline segment costs
BASELINE_ENTRY_COST = 1915974  # ticks
BASELINE_KERNEL_COST = 2926525 - 1915974  # = 1010551 ticks
# Calculate RETURN_COST from total gate cost minus entry and kernel
baseline_total_gate = BASELINE_METRICS["first_syscall_gate_entry_to_first_syscall_exit"]
BASELINE_RETURN_COST = baseline_total_gate - BASELINE_ENTRY_COST - BASELINE_KERNEL_COST

# Constitutional thresholds: boot=10%, syscall/context=5%
BOOT_THRESHOLD_PCT = 10.0
SYSCALL_THRESHOLD_PCT = 5.0
CONTEXT_THRESHOLD_PCT = 5.0

def load_preempt_metrics(metrics_file):
    """Load metrics from preempt test output"""
    metrics = {}
    with open(metrics_file, 'r') as f:
        for line in f:
            line = line.strip()
            if '=' in line:
                key, value = line.split('=', 1)
                metrics[key] = value
    return metrics

def parse_debugcon_log_for_diagnostic_markers(log_file):
    """Parse debugcon log to extract diagnostic marker timestamps"""
    markers = {}
    
    if not Path(log_file).exists():
        return markers
    
    # Track init path vs skip path
    init_entered = False
    init_skipped = False
    
    with open(log_file, 'r') as f:
        for line in f:
            line = line.strip()
            
            # Look for diagnostic markers with timestamps
            # Format: DIAG_MARKER_NAME 0xHEXTIMESTAMP
            if 'DIAG_' in line:
                parts = line.split()
                if len(parts) >= 2:
                    marker_name = parts[0]
                    timestamp_str = parts[1]
                    
                    try:
                        # Parse hex timestamp
                        timestamp = int(timestamp_str, 16)
                        
                        # Map marker names to keys
                        if marker_name == 'DIAG_KERNEL_HANDLER_ENTRY':
                            markers['kernel_handler_entry'] = timestamp
                        elif marker_name == 'DIAG_CONTEXT_DETECTION_DONE':
                            markers['context_detection_done'] = timestamp
                        elif marker_name == 'DIAG_BOUNDARY_INIT_ENTER':
                            markers['boundary_init_enter'] = timestamp
                            init_entered = True
                        elif marker_name == 'DIAG_BOUNDARY_ENFORCE_INIT_DONE':
                            markers['boundary_enforce_init_done'] = timestamp
                        elif marker_name == 'DIAG_MATRIX_VALIDATE_DONE':
                            markers['matrix_validate_done'] = timestamp
                        elif marker_name == 'DIAG_BOUNDARY_INIT_FLAG_SET':
                            markers['boundary_init_flag_set'] = timestamp
                        elif marker_name == 'DIAG_BOUNDARY_INIT_SKIPPED':
                            markers['boundary_init_skipped'] = timestamp
                            init_skipped = True
                        elif marker_name == 'DIAG_BOUNDARY_INIT_DONE':
                            markers['boundary_init_done'] = timestamp
                        elif marker_name == 'DIAG_CONTEXT_REGISTRATION_DONE':
                            markers['context_registration_done'] = timestamp
                        elif marker_name == 'DIAG_BOUNDARY_VALIDATE_DONE':
                            markers['boundary_validate_done'] = timestamp
                        elif marker_name == 'DIAG_BRIDGE_BYPASS_CHECK_DONE':
                            markers['bridge_bypass_check_done'] = timestamp
                        elif marker_name == 'DIAG_BCIB_SUBMISSION_CHECK_DONE':
                            markers['bcib_submission_check_done'] = timestamp
                        elif marker_name == 'DIAG_SYSCALL_RANGE_CHECK_DONE':
                            markers['syscall_range_check_done'] = timestamp
                    except ValueError:
                        # Skip malformed timestamps
                        pass
    
    # Store init path information
    markers['_init_path_taken'] = init_entered
    markers['_init_path_skipped'] = init_skipped
    
    return markers

def calculate_kernel_subsegment_costs(diagnostic_markers):
    """Calculate kernel sub-segment costs from diagnostic marker timestamps
    
    CRITICAL: Avoid double-counting. _boundary_init_total_for_display is excluded
    from the subsegment sum since its components are already included individually.
    
    NOTE: Marker emission overhead is included in measurements (timestamp taken
    before marker write, so next segment includes previous marker's write cost).
    """
    if not diagnostic_markers or len(diagnostic_markers) < 2:
        return None
    
    # Check init path vs skip path
    init_path_taken = diagnostic_markers.get('_init_path_taken', False)
    init_path_skipped = diagnostic_markers.get('_init_path_skipped', False)
    
    subsegments = {}
    
    # Context detection (always present)
    if 'kernel_handler_entry' in diagnostic_markers and 'context_detection_done' in diagnostic_markers:
        subsegments['context_detection'] = diagnostic_markers['context_detection_done'] - diagnostic_markers['kernel_handler_entry']
    
    # Boundary init - different calculation based on path taken
    if init_path_taken:
        # Init path: measure individual components
        if 'context_detection_done' in diagnostic_markers and 'boundary_init_enter' in diagnostic_markers:
            subsegments['boundary_init_overhead'] = diagnostic_markers['boundary_init_enter'] - diagnostic_markers['context_detection_done']
        
        if 'boundary_init_enter' in diagnostic_markers and 'boundary_enforce_init_done' in diagnostic_markers:
            subsegments['boundary_enforce_init'] = diagnostic_markers['boundary_enforce_init_done'] - diagnostic_markers['boundary_init_enter']
        
        if 'boundary_enforce_init_done' in diagnostic_markers and 'matrix_validate_done' in diagnostic_markers:
            subsegments['matrix_validate'] = diagnostic_markers['matrix_validate_done'] - diagnostic_markers['boundary_enforce_init_done']
        
        if 'matrix_validate_done' in diagnostic_markers and 'boundary_init_flag_set' in diagnostic_markers:
            subsegments['boundary_init_flag_overhead'] = diagnostic_markers['boundary_init_flag_set'] - diagnostic_markers['matrix_validate_done']
        
        # Store total init cost separately for display only (NOT included in subsegment sum)
        if 'context_detection_done' in diagnostic_markers and 'boundary_init_flag_set' in diagnostic_markers:
            subsegments['_boundary_init_total_for_display'] = diagnostic_markers['boundary_init_flag_set'] - diagnostic_markers['context_detection_done']
    
    elif init_path_skipped:
        # Skip path: measure fast path overhead
        if 'context_detection_done' in diagnostic_markers and 'boundary_init_skipped' in diagnostic_markers:
            subsegments['boundary_init_skip_overhead'] = diagnostic_markers['boundary_init_skipped'] - diagnostic_markers['context_detection_done']
    
    # Context registration (always present)
    if init_path_taken and 'boundary_init_flag_set' in diagnostic_markers and 'context_registration_done' in diagnostic_markers:
        subsegments['context_registration'] = diagnostic_markers['context_registration_done'] - diagnostic_markers['boundary_init_flag_set']
    elif init_path_skipped and 'boundary_init_skipped' in diagnostic_markers and 'context_registration_done' in diagnostic_markers:
        subsegments['context_registration'] = diagnostic_markers['context_registration_done'] - diagnostic_markers['boundary_init_skipped']
    
    # Remaining segments (always present)
    if 'context_registration_done' in diagnostic_markers and 'boundary_validate_done' in diagnostic_markers:
        subsegments['boundary_validate'] = diagnostic_markers['boundary_validate_done'] - diagnostic_markers['context_registration_done']
    
    if 'boundary_validate_done' in diagnostic_markers and 'bridge_bypass_check_done' in diagnostic_markers:
        subsegments['bridge_bypass_check'] = diagnostic_markers['bridge_bypass_check_done'] - diagnostic_markers['boundary_validate_done']
    
    if 'bridge_bypass_check_done' in diagnostic_markers and 'bcib_submission_check_done' in diagnostic_markers:
        subsegments['bcib_submission_check'] = diagnostic_markers['bcib_submission_check_done'] - diagnostic_markers['bridge_bypass_check_done']
    
    if 'bcib_submission_check_done' in diagnostic_markers and 'syscall_range_check_done' in diagnostic_markers:
        subsegments['syscall_range_check'] = diagnostic_markers['syscall_range_check_done'] - diagnostic_markers['bcib_submission_check_done']
    
    return subsegments

def extract_tick_value(metrics, key):
    """Extract tick value from metrics, handling 'true'/'false' validity"""
    tick_key = f"phase_{key}_ticks"
    valid_key = f"phase_{key}_tick_valid"
    
    if tick_key not in metrics:
        return None
    
    if valid_key in metrics and metrics[valid_key] == 'false':
        return None
    
    try:
        return int(metrics[tick_key])
    except (ValueError, KeyError):
        return None

def calculate_segment_costs(metrics):
    """Calculate syscall path segment costs from existing markers"""
    
    # Extract phase timestamps
    first_user_entry = extract_tick_value(metrics, "first_user_entry")
    first_syscall_gate_entry = extract_tick_value(metrics, "first_syscall_gate_entry")
    first_syscall_entry = extract_tick_value(metrics, "first_syscall_entry")
    first_syscall_exit = extract_tick_value(metrics, "first_syscall_exit")
    first_syscall_gate_return = extract_tick_value(metrics, "first_syscall_gate_return")
    
    if None in [first_user_entry, first_syscall_gate_entry, first_syscall_entry, 
                first_syscall_exit, first_syscall_gate_return]:
        print("ERROR: Missing required phase timestamps")
        print(f"  first_user_entry: {first_user_entry}")
        print(f"  first_syscall_gate_entry: {first_syscall_gate_entry}")
        print(f"  first_syscall_entry: {first_syscall_entry}")
        print(f"  first_syscall_exit: {first_syscall_exit}")
        print(f"  first_syscall_gate_return: {first_syscall_gate_return}")
        return None
    
    # Calculate segment costs
    entry_cost = first_syscall_entry - first_syscall_gate_entry
    kernel_cost = first_syscall_exit - first_syscall_entry
    return_cost = first_syscall_gate_return - first_syscall_exit
    total_cost = first_syscall_gate_return - first_syscall_gate_entry
    
    return {
        "entry_cost": entry_cost,
        "kernel_cost": kernel_cost,
        "return_cost": return_cost,
        "total_cost": total_cost,
    }

def analyze_regression(segment_costs, baseline_metrics, diagnostic_markers=None):
    """Analyze which segment(s) exceed baseline by >10%"""
    
    print("\n" + "="*60)
    print("SYSCALL REGRESSION ANALYSIS (DIAGNOSTIC)")
    print("Spec: scheduler-primary-regression-rca")
    print("Task: 1 - Bug Condition Exploration (Granular)")
    print("="*60)
    print("\n⚠ IMPORTANT: This is a DIAGNOSTIC measurement")
    print("⚠ Local Darwin/arm64 runs have env_hash_mismatch")
    print("⚠ Only authoritative GitHub CI (ubuntu-24.04-x64) is baseline-enforced")
    print("⚠ Constitutional thresholds: boot=10%, syscall=5%, context=5%")
    print("="*60)
    
    print("\nMeasured Segment Costs (TSC ticks):")
    print(f"  ENTRY_COST:  {segment_costs['entry_cost']:,}")
    print(f"  KERNEL_COST: {segment_costs['kernel_cost']:,}")
    print(f"  RETURN_COST: {segment_costs['return_cost']:,}")
    print(f"  TOTAL_COST:  {segment_costs['total_cost']:,}")
    
    print("\nBaseline Segment Costs (TSC ticks):")
    print(f"  ENTRY_COST:  {BASELINE_ENTRY_COST:,}")
    print(f"  KERNEL_COST: {BASELINE_KERNEL_COST:,}")
    print(f"  RETURN_COST: {BASELINE_RETURN_COST:,}")
    
    # Calculate regression percentages
    entry_regression_pct = ((segment_costs['entry_cost'] - BASELINE_ENTRY_COST) / BASELINE_ENTRY_COST) * 100
    kernel_regression_pct = ((segment_costs['kernel_cost'] - BASELINE_KERNEL_COST) / BASELINE_KERNEL_COST) * 100
    return_regression_pct = ((segment_costs['return_cost'] - BASELINE_RETURN_COST) / BASELINE_RETURN_COST) * 100
    
    print("\nRegression Analysis:")
    print(f"  ENTRY_COST:  {entry_regression_pct:+.1f}% vs baseline")
    print(f"  KERNEL_COST: {kernel_regression_pct:+.1f}% vs baseline")
    print(f"  RETURN_COST: {return_regression_pct:+.1f}% vs baseline")
    
    # Identify inflated segments (>5% threshold for syscall/context)
    inflated_segments = []
    
    if entry_regression_pct > SYSCALL_THRESHOLD_PCT:
        print(f"\n  ⚠ ENTRY_COST INFLATED: {entry_regression_pct:+.1f}% (threshold: +{SYSCALL_THRESHOLD_PCT}%)")
        inflated_segments.append("ENTRY")
        print("    → Investigate: register save, stack setup, initial validation")
    elif entry_regression_pct < -SYSCALL_THRESHOLD_PCT:
        print(f"\n  ✓ ENTRY_COST IMPROVED: {entry_regression_pct:+.1f}%")
    else:
        print(f"\n  ✓ ENTRY_COST STABLE: {entry_regression_pct:+.1f}%")
    
    if kernel_regression_pct > SYSCALL_THRESHOLD_PCT:
        print(f"\n  ⚠ KERNEL_COST INFLATED: {kernel_regression_pct:+.1f}% (threshold: +{SYSCALL_THRESHOLD_PCT}%)")
        inflated_segments.append("KERNEL")
        print("    → Kernel sub-segment analysis:")
        
        # Calculate and show sub-segment costs if diagnostic markers available
        if diagnostic_markers and isinstance(list(diagnostic_markers.values())[0], int):
            subsegments = calculate_kernel_subsegment_costs(diagnostic_markers)
            
            # Show init path information
            init_path_taken = diagnostic_markers.get('_init_path_taken', False)
            init_path_skipped = diagnostic_markers.get('_init_path_skipped', False)
            
            print(f"\n    Init Path Analysis:")
            if init_path_taken:
                print("      ✓ INIT PATH TAKEN (boundary_init_done was 0)")
                print("      → This is the FIRST syscall or flag was reset")
            elif init_path_skipped:
                print("      ✓ INIT PATH SKIPPED (boundary_init_done was 1)")
                print("      → This is a SUBSEQUENT syscall, fast path taken")
            else:
                print("      ⚠ Could not determine init path (missing markers)")
            
            if subsegments:
                print("\n    Kernel Sub-Segment Costs (TSC ticks):")
                
                # Calculate total EXCLUDING _boundary_init_total_for_display (display-only metric)
                total_subsegment = sum(cost for name, cost in subsegments.items() 
                                      if not name.startswith('_'))
                
                # Sort by cost (descending), excluding display-only metrics from main list
                sorted_subsegments = sorted(
                    [(name, cost) for name, cost in subsegments.items() if not name.startswith('_')],
                    key=lambda x: x[1], 
                    reverse=True
                )
                
                for name, cost in sorted_subsegments:
                    pct = (cost / total_subsegment) * 100 if total_subsegment > 0 else 0
                    
                    # Highlight init-specific segments
                    if 'init' in name.lower() or 'enforce' in name.lower() or 'matrix' in name.lower():
                        print(f"      {name:30s}: {cost:10,} ticks ({pct:5.1f}%) 🔥")
                    else:
                        print(f"      {name:30s}: {cost:10,} ticks ({pct:5.1f}%)")
                
                print(f"\n      {'TOTAL (measured segments)':30s}: {total_subsegment:10,} ticks")
                
                # Show boundary_init_total separately if available (for comparison)
                if '_boundary_init_total_for_display' in subsegments:
                    init_total = subsegments['_boundary_init_total_for_display']
                    init_pct = (init_total / total_subsegment) * 100 if total_subsegment > 0 else 0
                    print(f"      {'(boundary_init aggregate)':30s}: {init_total:10,} ticks ({init_pct:5.1f}%) [display only]")
                
                # Identify hotspots (>20% of kernel cost)
                print("\n    🔥 HOTSPOTS (>20% of kernel cost):")
                hotspots_found = False
                for name, cost in sorted_subsegments:
                    pct = (cost / total_subsegment) * 100 if total_subsegment > 0 else 0
                    if pct > 20:
                        print(f"      ⚠ {name}: {pct:.1f}% of kernel cost")
                        hotspots_found = True
                
                if not hotspots_found:
                    print("      (no single sub-segment >20%)")
                
                # Show init breakdown if init path was taken
                if init_path_taken:
                    print("\n    Boundary Init Breakdown (first syscall only):")
                    if 'boundary_enforce_init' in subsegments:
                        print(f"      boundary_enforce_init():                   {subsegments['boundary_enforce_init']:10,} ticks")
                    if 'matrix_validate' in subsegments:
                        print(f"      syscall_enforcement_validate_matrix():     {subsegments['matrix_validate']:10,} ticks")
                    if '_boundary_init_total_for_display' in subsegments:
                        print(f"      TOTAL INIT COST (aggregate):               {subsegments['_boundary_init_total_for_display']:10,} ticks")
                    print("\n      ⚠ NOTE: This measurement is from FIRST syscall only")
                    print("      ⚠ No evidence yet whether subsequent syscalls skip this path")
            else:
                print("    ⚠ Could not calculate sub-segment costs (missing markers)")
        else:
            print("\n    Diagnostic markers detected:")
            if diagnostic_markers:
                for marker, present in diagnostic_markers.items():
                    status = "✓" if present else "✗"
                    print(f"      {status} {marker}")
            
            print("\n    → Run CI with timestamped markers to collect sub-segment data")
        
        print("\n    Suspected hotspots (based on diagnostic markers):")
        print("      1. boundary_enforce_init() - boundary enforcement initialization")
        print("      2. syscall_enforcement_validate_matrix() - matrix validation")
        print("      3. Context detection and role mapping")
        print("      4. Boundary validation checks")
        print("      5. Syscall dispatch switch statement")
    elif kernel_regression_pct < -SYSCALL_THRESHOLD_PCT:
        print(f"\n  ✓ KERNEL_COST IMPROVED: {kernel_regression_pct:+.1f}%")
    else:
        print(f"\n  ✓ KERNEL_COST STABLE: {kernel_regression_pct:+.1f}%")
    
    # RETURN_COST analysis
    if return_regression_pct > SYSCALL_THRESHOLD_PCT:
        print(f"\n  ⚠ RETURN_COST INFLATED: {return_regression_pct:+.1f}% (threshold: +{SYSCALL_THRESHOLD_PCT}%)")
        inflated_segments.append("RETURN")
        print("    → Investigate: context restore, IRET preparation, return validation")
    elif return_regression_pct < -SYSCALL_THRESHOLD_PCT:
        print(f"\n  ✓ RETURN_COST IMPROVED: {return_regression_pct:+.1f}%")
    else:
        print(f"\n  ✓ RETURN_COST STABLE: {return_regression_pct:+.1f}%")
    
    print("\n" + "="*60)
    print("CONCLUSION")
    print("="*60)
    
    if inflated_segments:
        print(f"Inflated segments: {', '.join(inflated_segments)}")
        print("\n**EXPECTED OUTCOME**: Test FAILS (confirms bug exists)")
        print("**NEXT STEP**: Collect granular sub-segment timing data from CI")
        
        if "KERNEL" in inflated_segments:
            print("\n**CRITICAL**: KERNEL_COST regression is SEVERE")
            print("This indicates syscall handler execution is significantly slower")
            print("\n**ACTION REQUIRED**:")
            print("  1. Run instrumented kernel in CI")
            print("  2. Analyze debugcon log for diagnostic marker timing")
            print("  3. Identify which sub-segment(s) are inflated")
            print("  4. Apply surgical optimization to identified hotspot(s)")
    else:
        print("No clear segment inflation - further investigation needed")
        print("**NEXT STEP**: Review baseline assumptions and measurement methodology")
    
    print("="*60 + "\n")
    
    return inflated_segments

def main():
    if len(sys.argv) < 2:
        print("Usage: analyze_syscall_regression.py <preempt_metrics.txt> [debugcon.log]")
        sys.exit(1)
    
    metrics_file = sys.argv[1]
    debugcon_log = sys.argv[2] if len(sys.argv) > 2 else None
    
    if not Path(metrics_file).exists():
        print(f"ERROR: Metrics file not found: {metrics_file}")
        sys.exit(1)
    
    # Load metrics
    metrics = load_preempt_metrics(metrics_file)
    
    # Parse diagnostic markers if debugcon log provided
    diagnostic_markers = None
    if debugcon_log and Path(debugcon_log).exists():
        diagnostic_markers = parse_debugcon_log_for_diagnostic_markers(debugcon_log)
        print(f"Loaded diagnostic markers from: {debugcon_log}")
    
    # Calculate segment costs
    segment_costs = calculate_segment_costs(metrics)
    
    if segment_costs is None:
        print("ERROR: Failed to calculate segment costs")
        sys.exit(1)
    
    # Analyze regression
    inflated_segments = analyze_regression(segment_costs, BASELINE_METRICS, diagnostic_markers)
    
    # Exit with failure if regression detected (expected for unfixed code)
    if inflated_segments:
        print("TEST RESULT: FAIL (as expected - bug confirmed)")
        sys.exit(1)
    else:
        print("TEST RESULT: INCONCLUSIVE")
        sys.exit(2)

if __name__ == "__main__":
    main()

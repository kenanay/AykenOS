# Fail-Closed Proof Validation - Quick Start Guide

## TL;DR

```bash
# Generate QEMU trace
./scripts/qemu-fail-closed-proof-harness.sh

# Run validation gate
./scripts/ci-gate-fail-closed-proof.sh

# Or use Makefile
make ci-gate-fail-closed-proof
```

## What This Gate Does

Validates that kernel-level boundary violations result in deterministic termination with no execution continuation. Uses QEMU kernel trace as authoritative evidence.

## Required Markers

Your kernel trace must show:

```
BCIB_FORBIDDEN_BEFORE → [[AYKEN_SYSCALL_ENTER]] → [[AYKEN_BOUNDARY_KILL]]
```

With NO continuation markers after `[[AYKEN_BOUNDARY_KILL]]`.

## Common Failures

### Missing Markers
**Problem**: One or more required markers not found
**Fix**: Implement marker emission in kernel boundary enforcement code

### Process ID Mismatch
**Problem**: Markers belong to different processes
**Fix**: Check for race conditions or incorrect process tracking

### Continuation After Kill
**Problem**: BCIB_FORBIDDEN_AFTER or [[AYKEN_SYSCALL_EXIT]] found after kill
**Fix**: Verify fail-closed termination is complete, no return path

### Multiple Kills
**Problem**: More than one [[AYKEN_BOUNDARY_KILL]] marker
**Fix**: Investigate double execution or race condition

## Testing Your Implementation

### 1. Test with Valid Example
```bash
python3 scripts/validate_fail_closed_markers.py \
  evidence/fail-closed-proof/example_valid_trace.log
# Expected: PASS
```

### 2. Test with Invalid Example
```bash
python3 scripts/validate_fail_closed_markers.py \
  evidence/fail-closed-proof/example_invalid_trace.log
# Expected: FAIL with 3 violations
```

### 3. Test with Your Trace
```bash
# Generate your trace
./scripts/qemu-fail-closed-proof-harness.sh

# Validate it
python3 scripts/validate_fail_closed_markers.py \
  evidence/fail-closed-proof/qemu_kernel_trace.log
```

## Debugging Tips

### View Raw Trace
```bash
cat evidence/fail-closed-proof/qemu_kernel_trace.log
```

### Check Marker Counts
```bash
grep -c "BCIB_FORBIDDEN_BEFORE" evidence/fail-closed-proof/qemu_kernel_trace.log
grep -c "\[\[AYKEN_SYSCALL_ENTER\]\]" evidence/fail-closed-proof/qemu_kernel_trace.log
grep -c "\[\[AYKEN_BOUNDARY_KILL\]\]" evidence/fail-closed-proof/qemu_kernel_trace.log
```

### Find Process IDs
```bash
grep "pid=" evidence/fail-closed-proof/qemu_kernel_trace.log
```

### Check for Continuation Markers
```bash
# Find BOUNDARY_KILL line number
grep -n "\[\[AYKEN_BOUNDARY_KILL\]\]" evidence/fail-closed-proof/qemu_kernel_trace.log

# Check what comes after (replace N with line number + 1)
tail -n +N evidence/fail-closed-proof/qemu_kernel_trace.log
```

## Integration with CI

This gate is part of the CI freeze pipeline:

```
ci-gate-hygiene
    ↓
ci-gate-constitutional
    ↓
ci-gate-fail-closed-proof ← YOU ARE HERE
    ↓
MERGE ALLOWED
```

## Production Blocker

Tasks 3, 5, 6, and 10 CANNOT be marked complete without this gate passing.

## Need Help?

- Full documentation: `docs/ci-gates/fail-closed-proof-validation.md`
- Implementation details: `docs/ci-gates/IMPLEMENTATION_SUMMARY.md`
- Evidence directory: `evidence/fail-closed-proof/README.md`

## Gold Standard Example

A perfect trace looks like this:

```
[U] BCIB_FORBIDDEN_BEFORE: Process 42 attempting SYS_V2_SUBMIT_EXECUTION
[[AYKEN_SYSCALL_ENTER]] syscall=1001 pid=42
[[AYKEN_BOUNDARY_CHECK]] role=BCIB syscall=1001 allowed=false
[[AYKEN_BOUNDARY_KILL]] pid=42 reason=FORBIDDEN_SYSCALL

(LOG ENDS - no further output)
```

Key characteristics:
- Same pid (42) in all markers
- Exactly 1 BOUNDARY_KILL
- 2-line window between ENTER and KILL
- No logs after KILL

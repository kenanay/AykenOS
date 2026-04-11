# Fail-Closed Proof Validation

## Overview

The `ci-gate-fail-closed-proof` gate validates that fail-closed enforcement works at the kernel level using QEMU kernel trace evidence. This is a critical security gate that ensures boundary violations result in deterministic termination with no execution continuation.

**CRITICAL**: Host tests and emulated tests DO NOT satisfy this requirement. Only QEMU kernel trace is authoritative evidence for kernel-level claims.

## Requirements

This gate validates Requirements 16.1-16.15 from the Phase-16 specification:

- **16.1**: Kernel-level validation with QEMU trace evidence
- **16.2**: Canonical marker flow enforcement
- **16.3**: BOUNDARY_KILL emitted before scheduler removal
- **16.4-16.8**: Process identity and single kill guarantees
- **16.9-16.15**: Bounded execution window and hard stop validation

## Canonical Marker Flow

The gate validates the following deterministic marker sequence:

```
1. BCIB_FORBIDDEN_BEFORE
   ↓ (Userspace: BCIB-role process attempts forbidden syscall)
   
2. [[AYKEN_SYSCALL_ENTER]]
   ↓ (Kernel: Trap occurred, entered syscall dispatcher)
   
3. [[AYKEN_BOUNDARY_KILL]]
   ↓ (Kernel: Process terminated, fail-closed active)
   ↓ (CRITICAL: This marker emitted BEFORE scheduler removal)
   
4. (EXECUTION ENDS - no further markers allowed)
```

## Negative Guarantees

The following markers MUST NOT appear after `[[AYKEN_BOUNDARY_KILL]]`:

| Forbidden Marker | Meaning | Why It's Forbidden |
|-----------------|---------|-------------------|
| `BCIB_FORBIDDEN_AFTER` | Execution continued | Fail-closed didn't work |
| `[[AYKEN_SYSCALL_EXIT]]` | Syscall returned | Should terminate, not return |
| `[[AYKEN_SCHED_RESUME]]` | Process rescheduled | Kill was incomplete |
| Any logs from same process | Process still running | Hard stop failed |

## Validation Tests

### Test 1: Canonical Marker Flow
- Validates all three required markers are present
- Validates markers appear in correct order
- **Failure**: Missing markers or out-of-order sequence

### Test 2: Process Identity Consistency
- Extracts process_id from each marker
- Validates all markers belong to the SAME process
- **Prevents exploit**: Process A killed, Process B logs, gate incorrectly passes
- **Failure**: Markers from different process IDs

### Test 3: Single Kill Guarantee
- Counts `[[AYKEN_BOUNDARY_KILL]]` markers in entire trace
- Must be exactly 1 (not 0, not multiple)
- **Failure**: Zero kills = enforcement failed
- **Failure**: Multiple kills = unstable system / race condition

### Test 4: Bounded Execution Window
- Measures distance between `[[AYKEN_SYSCALL_ENTER]]` and `[[AYKEN_BOUNDARY_KILL]]`
- Window must be bounded (< 10 log lines) and deterministic
- **Failure**: Unbounded window indicates system hang or delayed enforcement
- **Failure**: Non-deterministic window indicates race condition

### Test 5: Negative Guarantees
- Scans all lines after `[[AYKEN_BOUNDARY_KILL]]`
- Validates no forbidden continuation markers present
- **Failure**: Any continuation marker found

### Test 6: Hard Stop Guarantee
- Scans for logs from same process after kill marker
- Validates process was removed from scheduler
- **Failure**: Process logs found after kill

### Test 7: Deterministic Error Code
- Searches for error codes: `BCIB_ERR_*`, `BOUNDARY_ERR_*`, `ABDF_ERR_*`
- Validates deterministic error reporting
- **Warning only**: Missing error code (not a failure)

## Usage

### Generate QEMU Trace

```bash
# Run QEMU test harness to generate kernel trace
./scripts/qemu-fail-closed-proof-harness.sh

# Output: evidence/fail-closed-proof/qemu_kernel_trace.log
```

### Run Validation Gate

```bash
# Run the CI gate
./scripts/ci-gate-fail-closed-proof.sh

# Output: evidence/fail-closed-proof/failclosed_proof_evidence.json
```

### Python Validator (Alternative)

```bash
# Run Python-based validator for detailed analysis
./scripts/validate_fail_closed_markers.py evidence/fail-closed-proof/qemu_kernel_trace.log

# Output: evidence/fail-closed-proof/qemu_kernel_trace_validation.json
```

## Evidence Artifacts

The gate produces the following evidence artifacts:

1. **qemu_kernel_trace.log**: Raw QEMU kernel trace (debugcon + serial)
2. **failclosed_proof_evidence.json**: Gate validation results
3. **gate_result.env**: Environment variables for CI integration
4. **qemu_debugcon.log**: Raw debugcon output
5. **qemu_serial.log**: Raw serial console output

## Integration with CI Pipeline

The fail-closed proof gate integrates into the CI pipeline as follows:

```
ci-gate-hygiene
    ↓
ci-gate-constitutional
    ↓
ci-gate-bcib-isolation
    ↓
ci-gate-boundary-enforcement
    ↓
ci-gate-fail-closed
    ↓
ci-gate-fail-closed-proof ← Kernel-level evidence validation
    ↓
MERGE ALLOWED
```

## Failure Modes

### Missing QEMU Trace
```
FAIL: QEMU_TRACE_MISSING
Reason: QEMU kernel trace file not found
Action: Run qemu-fail-closed-proof-harness.sh to generate trace
```

### Incomplete Marker Flow
```
FAIL: INCOMPLETE_MARKER_FLOW
Reason: One or more required markers missing
Action: Verify kernel marker emission is implemented
```

### Process Identity Mismatch
```
FAIL: PROCESS_IDENTITY_MISMATCH
Reason: Markers belong to different processes
Action: Check for race conditions or incorrect process tracking
```

### Multiple Kills Detected
```
FAIL: MULTIPLE_KILLS_DETECTED
Reason: More than one BOUNDARY_KILL marker found
Action: Investigate double execution or race condition
```

### Continuation After Kill
```
FAIL: CONTINUATION_AFTER_KILL
Reason: Forbidden markers found after BOUNDARY_KILL
Action: Verify fail-closed termination is complete
```

## Gold Standard Example

A successful fail-closed proof trace looks like this:

```
[U] BCIB_FORBIDDEN_BEFORE: Process 42 attempting SYS_V2_SUBMIT_EXECUTION
[[AYKEN_SYSCALL_ENTER]] syscall=1001 pid=42
[[AYKEN_BOUNDARY_CHECK]] role=BCIB syscall=1001 allowed=false
[[AYKEN_BOUNDARY_KILL]] pid=42 reason=FORBIDDEN_SYSCALL

(LOG ENDS HERE - no further output)
```

**Key characteristics**:
- All markers have same pid (42)
- Exactly 1 BOUNDARY_KILL marker
- ENTER to KILL window is 2 lines (bounded)
- No logs after KILL marker

## Common Mistakes

### Fake PASS
**Problem**: Only BEFORE + KILL present, but ENTER missing
**Cause**: Userspace simulation, no real kernel trap
**Fix**: Ensure test uses real syscall trap path

### Soft Fail
**Problem**: AFTER marker appears after KILL
**Cause**: Kill didn't work, execution continued
**Fix**: Verify fail-closed termination implementation

### Return Path Open
**Problem**: EXIT marker appears after KILL
**Cause**: Syscall returned instead of terminating
**Fix**: Ensure termination path doesn't return

### Scheduler Leak
**Problem**: Process logs appear after KILL
**Cause**: Process not removed from scheduler
**Fix**: Verify scheduler removal in termination path

## Constitutional Compliance

This gate enforces the following NON_OVERRIDABLE constitutional rules:

- **KERNEL.SAFETY.CRITICAL**: Kernel safety maintained through fail-closed enforcement
- **SECURITY.BOUNDARY.VIOLATION**: Boundary violations result in deterministic termination
- **DETERMINISM.GLOBAL**: Termination behavior is deterministic and reproducible

All violations result in ERROR level enforcement with immediate gate failure.

## Production Blocker Status

Tasks 3, 5, 6, and 10 CANNOT be marked complete without this gate passing:

- **Task 3**: BCIB execution entry enforcement
- **Task 5**: Runtime_Bridge syscall path
- **Task 6**: BCIB execution sandbox
- **Task 10**: Fail-closed enforcement

Missing QEMU evidence = task remains INCOMPLETE regardless of host test status.

## References

- Phase-16 Requirements: `.kiro/specs/phase16-bcib-abdf-isolation-contracts/requirements.md`
- Phase-16 Design: `.kiro/specs/phase16-bcib-abdf-isolation-contracts/design.md`
- Phase-16 Tasks: `.kiro/specs/phase16-bcib-abdf-isolation-contracts/tasks.md`
- Constitutional Rules: `_ayken/steering/NON_OVERRIDABLE.md`
- Phase Matrix: `_ayken/steering/PHASES.md`

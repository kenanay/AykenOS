# Marker Validation Enhancement

**Author**: Kenan AY — System Architect  
**Status**: Implemented  
**Task**: Task 1 - Dev loop marker guarantee enhancement

---

## Overview

This document describes the marker validation enhancements implemented in `scripts/dev_loop.sh` to provide robust boot validation with clear diagnostics and deterministic outcomes.

---

## Subtask 1.1: Marker Sequence Guarantee

### Purpose
Validate that boot markers appear in the correct order during kernel initialization.

### Required Sequence
```
[K][EARLY_BOOT_OK] → [K][LATE_INIT_END] → [[AYKEN_BOOT_OK]]
```

### Implementation
- Extract line numbers for each marker from boot log
- Compare line numbers to ensure strict ordering
- Report sequence violations with detailed diagnostics

### Validation Logic
```bash
early_line=$(grep -n "\[K\]\[EARLY_BOOT_OK\]" "$BOOT_LOG" | head -1 | cut -d: -f1)
late_line=$(grep -n "\[K\]\[LATE_INIT_END\]" "$BOOT_LOG" | head -1 | cut -d: -f1)
boot_line=$(grep -n "\[\[AYKEN_BOOT_OK\]\]" "$BOOT_LOG" | head -1 | cut -d: -f1)

# Validate: EARLY < LATE < BOOT_OK
if [ "$early_line" -gt "$late_line" ] || [ "$late_line" -gt "$boot_line" ]; then
    # Sequence violation detected
    exit 1
fi
```

### Success Output
```
✅ Smoke boot PASS
   Marker sequence validated: EARLY(line 42) → LATE(line 156) → BOOT_OK(line 289)
```

### Failure Output
```
❌ BOOT FAILED: Marker sequence violation

Expected order: [K][EARLY_BOOT_OK] → [K][LATE_INIT_END] → [[AYKEN_BOOT_OK]]
Actual order: [K][EARLY_BOOT_OK] appears AFTER [K][LATE_INIT_END]

Line numbers:
  [K][EARLY_BOOT_OK]: line 156
  [K][LATE_INIT_END]: line 42
  [[AYKEN_BOOT_OK]]: line 289

Last 50 lines of boot log:
...
```

---

## Subtask 1.2: Error Reporting Capability

### Purpose
Provide clear, actionable diagnostic output for all validation failure modes.

### Failure Modes

#### Missing Marker
```
❌ BOOT FAILED: [K][EARLY_BOOT_OK] marker not found

Expected marker: [K][EARLY_BOOT_OK]
This indicates early boot phase did not complete successfully.

Last 50 lines of boot log:
...
```

#### Sequence Violation
```
❌ BOOT FAILED: Marker sequence violation

Expected order: [K][EARLY_BOOT_OK] → [K][LATE_INIT_END] → [[AYKEN_BOOT_OK]]
Actual order: [K][LATE_INIT_END] appears AFTER [[AYKEN_BOOT_OK]]

Line numbers:
  [K][EARLY_BOOT_OK]: line 42
  [K][LATE_INIT_END]: line 289
  [[AYKEN_BOOT_OK]]: line 156
```

#### Log Directory Error
```
❌ BOOT FAILED: Cannot create log directory: out/logs
```

#### Log File Error
```
❌ BOOT FAILED: Cannot write to log file: out/logs/boot_watch.log
```

### Diagnostic Features
- Clear error messages explaining what went wrong
- Expected vs actual behavior
- Line numbers for sequence violations
- Last 50 lines of boot log for context
- Actionable information for debugging

---

## Subtask 1.3: Exit Status Contract Enforcement

### Purpose
Provide deterministic, scriptable validation outcomes through consistent exit codes.

### Exit Code Contract

| Exit Code | Meaning | Scenarios |
|-----------|---------|-----------|
| 0 | PASS | All validation checks succeeded |
| 1 | FAIL | Build failure, boot timeout, missing marker, sequence violation, test failure |
| 2 | Invalid Usage | Wrong command-line arguments |

### Implementation
```bash
# Success path
echo "✅ PASS: $MODE mode"
exit 0  # Implicit with set -e

# Failure paths
exit 1  # Validation failure

# Invalid usage
exit 2  # Wrong arguments
```

### Usage in CI/CD
```bash
# CI script example
if ./scripts/dev_loop.sh smoke; then
    echo "Validation passed"
else
    exit_code=$?
    if [ "$exit_code" -eq 1 ]; then
        echo "Validation failed"
    elif [ "$exit_code" -eq 2 ]; then
        echo "Invalid usage"
    fi
    exit "$exit_code"
fi
```

---

## Subtask 1.4: Log Directory Management

### Purpose
Ensure robust log directory and file lifecycle management with proper error handling.

### Features

#### Directory Creation
```bash
if [ ! -d "$LOG_DIR" ]; then
    mkdir -p "$LOG_DIR" || {
        echo "❌ BOOT FAILED: Cannot create log directory: $LOG_DIR"
        exit 1
    }
fi
```

#### File Initialization
```bash
: > "$BOOT_LOG" || {
    echo "❌ BOOT FAILED: Cannot write to log file: $BOOT_LOG"
    exit 1
}
```

#### Error Handling
- Check directory existence before use
- Validate write permissions
- Provide clear error messages on failure
- Fail fast with exit code 1

#### Lifecycle
1. Check if log directory exists
2. Create directory if missing (with error handling)
3. Clear previous log file (with error handling)
4. Capture boot output to log file
5. Validate markers from log file
6. Preserve log file for debugging

---

## Testing

### Unit Tests
Run the marker validation test suite:
```bash
./scripts/test_marker_validation.sh
```

### Exit Status Tests
Run the exit status contract test:
```bash
./scripts/test_exit_status_contract.sh
```

### Integration Tests
Test with actual kernel boot:
```bash
./scripts/dev_loop.sh smoke
```

---

## Constitutional Compliance

### DETERMINISM.GLOBAL
✅ **Compliant**: No global state mutations
- Validation logic is stateless
- Same input → same output
- Reproducible results

### KERNEL.RING0.POLICY
✅ **Compliant**: No policy decisions in Ring0
- Validation is userspace script
- Markers are pure output
- No kernel coupling

### SECURITY.BOUNDARY.VIOLATION
✅ **Compliant**: No Ring3 accessing Ring0 directly
- Dev loop reads serial output only
- No direct memory access
- Proper isolation maintained

---

## References

- **Spec**: `.kiro/specs/dev-loop-boot-monitoring/`
- **Requirements**: `requirements.md` (R1, R16, R17, R20)
- **Design**: `design.md` (Section 4.1, 4.3)
- **Implementation Guide**: `docs/dev-loop/IMPLEMENTATION_GUIDE.md`
- **Script**: `scripts/dev_loop.sh`

---

**Last Updated**: 2026-05-03  
**Maintainer**: Kenan AY — System Architect

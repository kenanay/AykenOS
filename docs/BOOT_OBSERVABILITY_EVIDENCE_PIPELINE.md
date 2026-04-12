# Boot Chain Observability Evidence Pipeline Requirements

**Author**: Kenan AY - Architectural Steward  
**Created**: 2026-04-12  
**Status**: Authoritative Reference

## Overview

This document defines the mandatory requirements for the boot chain observability evidence pipeline in AykenOS. These requirements ensure that boot trace evidence maintains integrity, temporal ordering, and channel-local truth for fail-closed proof validation.

## Core Principles

### 1. Raw Append-Order Trace (Mandatory)

**Requirement**: All output channels MUST preserve raw append-order trace without reordering, sorting, or temporal manipulation.

**Rationale**: Fail-closed systems require deterministic execution order proof. Any operation that reorders markers creates fake temporal ordering and invalidates security claims.

**Implementation**:
- Capture output to file in append mode
- NO post-processing that changes line order
- NO sorting, uniquing, or buffer reordering
- Preserve exact byte sequence as emitted by kernel/bootloader

**Violation Examples**:
```bash
# FORBIDDEN: Destroys temporal order
cat debugcon.log serial.log | sort > trace.log

# FORBIDDEN: Can reorder or drop lines
cat trace.log | uniq > deduped.log

# FORBIDDEN: Loses line context and order
grep -o "MARKER" trace.log > markers.txt
```

**Correct Examples**:
```bash
# CORRECT: Preserve raw append order per channel
cp debugcon.log debugcon.trace
cp serial.log serial.trace

# CORRECT: Channel-local analysis
grep "MARKER" debugcon.trace
grep "MARKER" serial.trace
```

### 2. Channel-Local Detection (Mandatory)

**Requirement**: Marker detection and temporal ordering MUST be performed within a single channel. Cross-channel merge creates fake temporal ordering.

**Rationale**: Debugcon (port 0xE9) and serial (port 0x3F8) are NOT on the same time axis. Concatenating them creates false ordering that cannot be proven deterministic.

**Implementation**:
- Keep separate trace files per channel: `debugcon.trace`, `serial.trace`
- Analyze markers within each channel independently
- Aggregate results: "marker found in debugcon OR serial"
- NEVER merge channels for temporal ordering claims

**Violation Example**:
```bash
# FORBIDDEN: Creates fake temporal ordering across channels
cat debugcon.log serial.log > merged.trace
grep "MARKER_A.*MARKER_B" merged.trace  # FALSE: Cannot prove A before B
```

**Correct Example**:
```bash
# CORRECT: Channel-local analysis
grep "MARKER_A" debugcon.trace && grep "MARKER_B" debugcon.trace
# OR
grep "MARKER_A" serial.trace && grep "MARKER_B" serial.trace
```

### 3. At Least One Channel Must Capture Markers (HARD FAIL)

**Requirement**: If ALL output channels (debugcon AND serial) are zero bytes, the system MUST HARD FAIL and stop all validation.

**Rationale**: Zero-byte logs indicate complete observability loss. Proceeding with validation would create false confidence in unverifiable claims.

**Implementation**:
```bash
DEBUGCON_SIZE=$(stat -c%s debugcon.log)
SERIAL_SIZE=$(stat -c%s serial.log)

if [[ $DEBUGCON_SIZE -eq 0 ]] && [[ $SERIAL_SIZE -eq 0 ]]; then
    echo "OUTPUT_CHANNEL_FAILURE: All channels empty"
    exit 1
fi
```

**CI Gate**: `stat -c%s "$DEBUGCON_LOG"` must be > 0 OR serial > 0

### 4. UEFI Print as Ground Truth Fallback (Bootloader Only)

**Requirement**: UEFI Print output serves as ground truth fallback for bootloader execution diagnosis.

**Rationale**: If debugcon=FAIL AND serial=FAIL BUT uefi=OK, the problem is QEMU capture path, NOT bootloader execution.

**Diagnostic Matrix**:
| Debugcon | Serial | UEFI Print | Diagnosis |
|----------|--------|------------|-----------|
| FAIL | FAIL | FAIL | Bootloader execution failure |
| FAIL | FAIL | OK | QEMU capture path broken |
| OK | * | * | Debugcon working (primary channel) |
| FAIL | OK | * | Serial working (fallback channel) |

**Implementation**:
- Bootloader emits markers to UEFI Print (firmware console)
- QEMU stdout/stderr captures UEFI Print output
- Use for diagnosis only, NOT authoritative evidence

## Forbidden Operations

The following operations are EXPLICITLY FORBIDDEN in evidence pipeline scripts:

### 1. `sort` - Destroys Temporal Order

**Why Forbidden**: Reorders lines alphabetically, breaking deterministic execution order proof.

**Example**:
```bash
# FORBIDDEN
cat trace.log | sort
```

**Impact**: Fail-closed proof requires marker order preservation. Sorting invalidates security claims.

### 2. `uniq` - Can Reorder or Drop Lines

**Why Forbidden**: Removes duplicate lines, potentially dropping legitimate repeated markers.

**Example**:
```bash
# FORBIDDEN
cat trace.log | uniq
```

**Impact**: Legitimate marker repetition (e.g., multiple syscalls) would be lost.

### 3. `grep -o` - Loses Line Context and Order

**Why Forbidden**: Extracts only matching text, losing line context and relative ordering.

**Example**:
```bash
# FORBIDDEN
grep -o "MARKER" trace.log
```

**Impact**: Cannot prove marker order or context without full line.

### 4. `awk` with Reorder Logic - Creates Fake Ordering

**Why Forbidden**: AWK can buffer and reorder lines, creating non-deterministic output.

**Example**:
```bash
# FORBIDDEN
awk '{lines[NR]=$0} END {for(i=NR;i>0;i--) print lines[i]}' trace.log
```

**Impact**: Reverses or reorders lines, breaking temporal proof.

### 5. Multiline Buffer Operations - Can Reorder or Drop Lines

**Why Forbidden**: Buffering multiple lines can cause reordering or line loss.

**Example**:
```bash
# FORBIDDEN
sed -n 'N;s/\n/ /;p' trace.log  # Joins lines, loses ordering
```

**Impact**: Line joining or buffering breaks append-order guarantee.

### 6. Cross-Channel Concatenation - Creates Fake Temporal Ordering

**Why Forbidden**: Merging debugcon and serial creates false temporal ordering across channels.

**Example**:
```bash
# FORBIDDEN (for temporal ordering claims)
cat debugcon.log serial.log > merged.trace
grep "MARKER_A.*MARKER_B" merged.trace  # FALSE CLAIM
```

**Allowed Exception**: Cross-channel concatenation is ONLY allowed for NON-AUTHORITATIVE human-readable summaries with explicit warnings:
```bash
# ALLOWED: Non-authoritative summary only
{
    echo "=== NON-AUTHORITATIVE SUMMARY ==="
    echo "=== DO NOT use for temporal ordering ==="
    cat debugcon.log serial.log
} > summary.txt
```

## CI Gates

### Gate 1: Channel Integrity (HARD FAIL)

**Rule**: At least one output channel (debugcon OR serial) MUST be non-empty.

**Check**:
```bash
if [[ $DEBUGCON_SIZE -eq 0 ]] && [[ $SERIAL_SIZE -eq 0 ]]; then
    echo "HARD FAIL: OUTPUT_CHANNEL_FAILURE"
    exit 1
fi
```

**Failure Action**: STOP ALL VALIDATION, exit with error code 1.

### Gate 2: Forbidden Operations Detection

**Rule**: Harness scripts MUST NOT contain forbidden operations.

**Check**:
```bash
grep -q "| sort" harness.sh && echo "FAIL: sort detected"
grep -q "| uniq" harness.sh && echo "FAIL: uniq detected"
grep -q "grep -o" harness.sh && echo "FAIL: grep -o detected"
```

**Failure Action**: CI FAIL, block deployment.

### Gate 3: Required Markers Present

**Rule**: Required boot markers MUST appear in at least one channel.

**Required Markers**:
- `[B][UEFI_BOOT_START]` - Bootloader start
- `[[AYKEN_BOOT_OK]]` - Kernel entry

**Check**:
```bash
grep -q "\[B\]\[UEFI_BOOT_START\]" debugcon.trace || \
grep -q "\[B\]\[UEFI_BOOT_START\]" serial.trace || \
echo "FAIL: UEFI_BOOT_START marker absent"
```

**Failure Action**: CI FAIL, marker absent violation.

### Gate 4: Marker Order Preservation

**Rule**: Markers MUST appear in correct order within each channel.

**Expected Order**: `[B][UEFI_BOOT_START]` before `[[AYKEN_BOOT_OK]]`

**Check**:
```bash
BOOT_START_LINE=$(grep -n "\[B\]\[UEFI_BOOT_START\]" debugcon.trace | cut -d: -f1)
BOOT_OK_LINE=$(grep -n "\[\[AYKEN_BOOT_OK\]\]" debugcon.trace | cut -d: -f1)

if [[ $BOOT_START_LINE -gt $BOOT_OK_LINE ]]; then
    echo "FAIL: Marker order broken"
fi
```

**Failure Action**: CI FAIL, marker order violation.

## Output Channels

### Primary Channel: Debugcon (Port 0xE9)

**Configuration**: `-debugcon file:$LOG -global isa-debugcon.iobase=0xe9`

**Characteristics**:
- Direct port write (no buffering)
- QEMU-specific debug console
- Primary evidence channel

**Usage**:
```c
// Bootloader/Kernel
static inline void debugcon_write_char(char c) {
    __asm__ volatile("outb %0, %1" : : "a"(c), "Nd"(0xE9));
}
```

### Fallback Channel: Serial (Port 0x3F8)

**Configuration**: `-serial file:$LOG`

**Characteristics**:
- COM1 serial port
- Requires deterministic initialization
- Fallback evidence channel

**Usage**:
```c
// Bootloader/Kernel
static inline void serial_write_char(char c) {
    // Poll TX empty bit before write
    while (!(inb(0x3FD) & 0x20));
    outb(0x3F8, c);
}
```

### Diagnostic Channel: UEFI Print (Bootloader Only)

**Configuration**: QEMU stdout/stderr

**Characteristics**:
- UEFI firmware console
- Bootloader-only (not available in kernel)
- Diagnostic/ground truth fallback

**Usage**:
```c
// Bootloader only
Print(L"[B][UEFI_BOOT_START]\n");
```

## Regression Lock Enforcement

### Automated CI Integration

**Script**: `scripts/ci-gate-boot-observability.sh`

**Invocation**:
```bash
# After QEMU boot test
./scripts/ci-gate-boot-observability.sh

# Exit code 0 = PASS, non-zero = FAIL
```

**CI Pipeline Integration**:
```yaml
# Example CI configuration
- name: Boot Observability Gate
  run: |
    make qemu-boot-test
    ./scripts/ci-gate-boot-observability.sh
  fail-fast: true
```

### Evidence Artifacts

**Generated Files**:
- `evidence/boot-observability/boot_observability_evidence.json` - Structured evidence
- `evidence/boot-observability/violations.log` - Violation details
- `evidence/boot-observability/debugcon.trace` - Authoritative debugcon trace
- `evidence/boot-observability/serial.trace` - Authoritative serial trace

**Evidence JSON Schema**:
```json
{
  "gate": "ci-gate-boot-observability",
  "timestamp": "2026-04-12T10:30:00Z",
  "result": "PASS|FAIL",
  "failure_code": "NONE|OUTPUT_CHANNEL_FAILURE|MARKER_ABSENT|...",
  "violations_detected": 0,
  "channel_integrity": {
    "debugcon_size": 1024,
    "serial_size": 512,
    "at_least_one_channel_working": true
  },
  "forbidden_operations": {
    "detected": 0
  },
  "required_markers": {
    "[B][UEFI_BOOT_START]": true,
    "[[AYKEN_BOOT_OK]]": true
  },
  "marker_order": {
    "preserved": true
  }
}
```

## Architectural Freeze Compliance

**Scope**: This evidence pipeline is a NON-ARCHITECTURAL BUGFIX.

**Allowed**:
- Evidence capture scripts
- Harness integrity fixes
- CI gate validation
- Documentation

**Forbidden**:
- New syscalls
- New execution layers
- New contracts
- Kernel policy changes
- Phase transition claims

**Verification**: All changes must remain within evidence pipeline repair boundaries.

## References

- Spec: `.kiro/specs/boot-chain-observability-restoration/`
- Bugfix Requirements: `bugfix.md`
- Design Document: `design.md`
- Implementation Tasks: `tasks.md`
- CI Gate Script: `scripts/ci-gate-boot-observability.sh`

---

**Document Status**: Authoritative Reference  
**Last Updated**: 2026-04-12  
**Author**: Kenan AY - Architectural Steward

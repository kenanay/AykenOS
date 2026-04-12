# Boot Observability Evidence Pipeline - Quick Reference

**Author**: Kenan AY - Architectural Steward  
**For**: Developers working with boot chain evidence

## TL;DR - Critical Rules

1. **NEVER sort, uniq, or reorder trace files** - Destroys temporal proof
2. **Keep channels separate** - NO cross-channel merge for ordering claims
3. **At least one channel must work** - Zero-byte logs = HARD FAIL
4. **Preserve raw append-order** - No post-processing that changes line order

## Quick Checks

### ✅ DO THIS

```bash
# Preserve raw append order per channel
cp debugcon.log debugcon.trace
cp serial.log serial.trace

# Channel-local marker detection
grep "MARKER" debugcon.trace
grep "MARKER" serial.trace

# Aggregate results (OR logic)
if grep -q "MARKER" debugcon.trace || grep -q "MARKER" serial.trace; then
    echo "Marker found"
fi

# Check channel integrity
DEBUGCON_SIZE=$(stat -c%s debugcon.log)
if [[ $DEBUGCON_SIZE -eq 0 ]]; then
    echo "WARNING: Debugcon empty"
fi
```

### ❌ DON'T DO THIS

```bash
# FORBIDDEN: Destroys temporal order
cat debugcon.log serial.log | sort > trace.log

# FORBIDDEN: Can drop lines
cat trace.log | uniq > deduped.log

# FORBIDDEN: Loses context
grep -o "MARKER" trace.log

# FORBIDDEN: Creates fake temporal ordering
cat debugcon.log serial.log > merged.log
grep "MARKER_A.*MARKER_B" merged.log  # FALSE CLAIM
```

## Required Boot Markers

| Marker | Location | Channel | Purpose | Status |
|--------|----------|---------|---------|--------|
| `[B][UEFI_BOOT_START]` | Bootloader | debugcon + serial | Bootloader entry | MANDATORY |
| `[B][KERNEL_ELF_LOADED]` | Bootloader | debugcon + serial | ELF load complete | OPTIONAL |
| `[B][JUMP_NOW]` | Bootloader | debugcon + serial | Before kernel jump | OPTIONAL |
| `[[AYKEN_BOOT_OK]]` | Kernel entry stub | debugcon + serial | Kernel entry | MANDATORY |
| `[K][EARLY_BOOT_OK]` | Kernel (kmain_real) | debugcon + serial | Early boot complete | MANDATORY |

**Canonical Order (Boot Flow)**:
1. `[B][UEFI_BOOT_START]` - Bootloader starts
2. `[B][KERNEL_ELF_LOADED]` - (optional) ELF loaded
3. `[B][JUMP_NOW]` - (optional) Before jump
4. `[[AYKEN_BOOT_OK]]` - Kernel entry stub executes (FIRST kernel marker)
5. `[K][EARLY_BOOT_OK]` - kmain_real executes (C function)

**CRITICAL**: `[[AYKEN_BOOT_OK]]` appears BEFORE `[K][EARLY_BOOT_OK]` because:
- `[[AYKEN_BOOT_OK]]` is emitted at entry stub (actual entry point, assembly)
- `[K][EARLY_BOOT_OK]` is emitted later in kmain_real (C function)
- Entry stub executes before C code, so this order is deterministic

## Output Channels

| Channel | Port | Config | Use Case |
|---------|------|--------|----------|
| Debugcon | 0xE9 | `-debugcon file:$LOG` | Primary evidence |
| Serial | 0x3F8 | `-serial file:$LOG` | Fallback evidence |
| UEFI Print | stdout | QEMU stdout | Diagnostic only |

## CI Gate Usage

```bash
# Run QEMU boot test
make qemu-boot-test

# Run CI gate
./scripts/ci-gate-boot-observability.sh

# Check result
if [[ $? -eq 0 ]]; then
    echo "PASS: Evidence pipeline integrity validated"
else
    echo "FAIL: Check evidence/boot-observability/violations.log"
fi
```

## Diagnostic Flow

```
1. Check channel sizes
   ├─ All zero? → HARD FAIL (OUTPUT_CHANNEL_FAILURE)
   └─ At least one non-zero? → Continue

2. Check for forbidden operations
   ├─ sort/uniq/grep -o detected? → FAIL
   └─ None detected? → Continue

3. Check required markers
   ├─ Marker missing? → FAIL (MARKER_ABSENT)
   └─ All present? → Continue

4. Check marker order
   ├─ Order broken? → FAIL (MARKER_ORDER_BROKEN)
   └─ Order preserved? → PASS
```

## Common Issues

### Issue: Debugcon log is 0 bytes

**Diagnosis**:
1. Check QEMU flags: `-debugcon file:$LOG -global isa-debugcon.iobase=0xe9`
2. Check if bootloader/kernel emit markers
3. Check UEFI Print output (diagnostic fallback)

**Fix**: Verify QEMU configuration and marker emission code

### Issue: Markers out of order

**Diagnosis**:
1. Check for `sort` in harness scripts
2. Check for cross-channel merge
3. Check for buffer reordering

**Fix**: Remove forbidden operations, preserve raw append-order

### Issue: CI gate fails with "FORBIDDEN_OPERATION"

**Diagnosis**: Harness script contains sort/uniq/grep -o

**Fix**: Remove forbidden operation, use channel-local analysis

## Evidence Files

After running CI gate:

```
evidence/boot-observability/
├── boot_observability_evidence.json  # Structured result
├── violations.log                     # Violation details
├── debugcon.trace                     # Authoritative debugcon
├── serial.trace                       # Authoritative serial
├── qemu_debugcon.log                  # Raw QEMU output
└── qemu_serial.log                    # Raw QEMU output
```

## Quick Reference: Marker Emission

### Bootloader (C)

```c
// After InitializeLib
debugcon_write("[B][UEFI_BOOT_START]\n");
serial_write("[B][UEFI_BOOT_START]\n");
Print(L"[B][UEFI_BOOT_START]\n");  // UEFI Print fallback
```

### Kernel Entry Stub (Assembly)

```asm
; At very first instruction
mov al, 'K'
out 0xE9, al        ; Debugcon
mov dx, 0x3F8
out dx, al          ; Serial (best-effort)
```

### Kernel (C)

```c
// After entry stub
dual_channel_write("[[AYKEN_BOOT_OK]]\n");
```

## For More Details

See: `docs/BOOT_OBSERVABILITY_EVIDENCE_PIPELINE.md`

---

**Last Updated**: 2026-04-12  
**Author**: Kenan AY - Architectural Steward

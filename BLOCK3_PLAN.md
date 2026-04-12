# Block 3: Deterministic Boot Marker Chain - Implementation Plan

**Status**: IN PROGRESS  
**Prerequisites**: ✅ Block 2 COMPLETE

## Block 2 Completion Evidence

**Proven Facts**:
- ✅ `efi_main()` executes successfully
- ✅ debugcon (port 0xE9) capture working
- ✅ Debugcon log size: 1476 bytes captured
- ✅ Bootloader markers visible in debugcon output

**Evidence Source**: `test_uefi_console_capture.sh` run
**Captured Markers**:
```
[B][UEFI_BOOT_START] efi_main entry
[B][INIT_LIB_OK]
[B][MEMMAP] status=0x0000000000000000
[B][GOP] status=0x0000000000000000
[B][ELF_MAGIC_OK]
[B][KERNEL_OPEN_OK]
```

**What Block 2 Proved**: Bootloader execution and debugcon capture mechanism
**What Block 2 Did NOT Prove**: Full boot chain to kernel (that's Block 3's job)

## Objective

Restore full boot marker chain from bootloader through kernel entry, with channel-local analysis and order verification.

## Current State (from Block 2 evidence)

**Bootloader markers already present**:
- `[B][UEFI_BOOT_START]` ✓
- `[B][INIT_LIB_OK]` ✓
- `[B][MEMMAP]` ✓
- `[B][GOP]` ✓
- `[B][ELF_MAGIC_OK]` ✓
- `[B][KERNEL_OPEN_OK]` ✓

**Missing markers**:
- `[B][KERNEL_ELF_LOADED]` - after elf_load_kernel completes
- `[B][JUMP_NOW]` - before ayken_jump_to_kernel
- `[[AYKEN_BOOT_OK]]` - kernel entry marker
- `[K][EARLY_BOOT_OK]` - kernel early boot marker

## Implementation Steps

### Step 1: Add Missing Bootloader Markers

**File**: `bootloader/efi/efi_main.c`

**Location 1**: After `elf_load_kernel()` success
```c
debugcon_write("[B][KERNEL_ELF_LOADED]\n");
```

**Location 2**: Before `ayken_jump_to_kernel()` call
```c
debugcon_write("[B][JUMP_NOW]\n");
```

**Rationale**: These markers bracket the critical kernel handoff moment.

### Step 2: Verify Kernel Entry Markers

**File**: `kernel/kernel.c` (or kernel entry point)

**Check for existing markers**:
- `[[AYKEN_BOOT_OK]]` - should already exist
- `[K][EARLY_BOOT_OK]` - should already exist

**If missing**: Add to kernel entry point (kmain or equivalent)

### Step 3: Create Channel-Local Test

**File**: `tests/boot_observability/test_block3_marker_chain.sh`

**Test logic**:
1. Run QEMU with debugcon capture
2. Extract markers from debugcon log (channel-local, no merge)
3. Verify marker order:
   - `[B][UEFI_BOOT_START]`
   - `[B][KERNEL_ELF_LOADED]`
   - `[B][JUMP_NOW]`
   - `[[AYKEN_BOOT_OK]]` or `[K][EARLY_BOOT_OK]`
4. FAIL if order broken
5. FAIL if any marker missing

**Critical rules**:
- NO cross-channel merge
- NO sort operation
- NO grep -o (loses context)
- Preserve append-order trace

### Step 4: Validate Marker Order

**Expected sequence**:
```
1. [B][UEFI_BOOT_START]
2. [B][INIT_LIB_OK]
3. [B][MEMMAP]
4. [B][GOP]
5. [B][ELF_MAGIC_OK]
6. [B][KERNEL_OPEN_OK]
7. [B][KERNEL_ELF_LOADED]  ← NEW
8. [B][JUMP_NOW]           ← NEW
9. [[AYKEN_BOOT_OK]]       ← KERNEL
10. [K][EARLY_BOOT_OK]     ← KERNEL
```

## Acceptance Criteria

- [ ] All bootloader markers present in debugcon log
- [ ] Kernel entry marker (`[[AYKEN_BOOT_OK]]` or `[K][EARLY_BOOT_OK]`) present
- [ ] Marker order preserved (no sort, no reorder)
- [ ] Test script validates order automatically
- [ ] Channel-local analysis (debugcon only, no merge)

## Files to Modify

1. `bootloader/efi/efi_main.c` - Add 2 markers
2. `kernel/kernel.c` - Verify/add kernel markers
3. `tests/boot_observability/test_block3_marker_chain.sh` - New test

## Expected Outcome

After Block 3:
- Full boot chain visible in debugcon log
- Bootloader → kernel handoff proven
- Marker order deterministic and verified
- Ready for Block 4 (regression lock)

## Next Action

Start with Step 1: Add missing bootloader markers to efi_main.c

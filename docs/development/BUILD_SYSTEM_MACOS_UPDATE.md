# macOS Build System Update - EFI.img Creation Fix

**Author:** Kenan AY  
**Date:** February 10, 2026  
**Status:** ✅ COMPLETED

## Overview

This document details the critical fixes applied to the AykenOS build system for macOS development, specifically addressing EFI.img creation and QEMU debugging issues that were preventing successful kernel boot testing.

## Issues Identified

### 1. EFI.img Creation Failure
**Problem:** The original build system did not have proper EFI.img creation support for macOS.
- No `mkfs.vfat` available on macOS (Linux-specific tool)
- Makefile lacked EFI.img target
- Manual image creation was error-prone

### 2. QEMU Debugcon Port Not Configured
**Problem:** Debug output was not being captured, leading to false "boot failure" diagnoses.
- Missing `-global isa-debugcon.iobase=0xe9` flag
- Debugcon log files remained empty
- Kernel was actually booting but appeared to fail

### 3. Kernel Boot Marker Confusion
**Problem:** Unable to verify if new kernel builds were actually running.
- Old kernel.elf cached in EFI.img
- No clear marker to distinguish new builds
- Hash verification needed for confirmation

## Solutions Implemented

### 1. macOS-Compatible EFI.img Build Script ✅

**File:** `build_efi.sh`

```bash
#!/bin/bash
# EFI.img oluşturma scripti - UDRW ile RAW

set -e

echo "EFI.img oluşturuluyor..."

# Eski dosyaları temizle
rm -f EFI.img EFI.dmg EFI_raw.dmg

# 64MB FAT32 disk image oluştur
hdiutil create -size 64m -fs MS-DOS -volname "EFI" -o EFI.dmg

# Mount et
hdiutil attach EFI.dmg >/dev/null

# AppleDouble dosyalarını engelle
export COPYFILE_DISABLE=1

# EFI dizin yapısını oluştur ve dosyaları kopyala
mkdir -p /Volumes/EFI/EFI/BOOT
rm -f /Volumes/EFI/EFI/BOOT/._* /Volumes/EFI/EFI/BOOT/*
cp -X bootloader/efi/BOOTX64.EFI /Volumes/EFI/EFI/BOOT/
cp -X kernel.elf /Volumes/EFI/EFI/BOOT/
cp -X kernel.elf /Volumes/EFI/  # Root'a da kopyala (bootloader için)

# startup.nsh oluştur (otomatik boot için)
echo "FS0:" > /Volumes/EFI/startup.nsh
echo "cd EFI\BOOT" >> /Volumes/EFI/startup.nsh
echo "BOOTX64.EFI" >> /Volumes/EFI/startup.nsh

# Unmount
hdiutil detach /Volumes/EFI >/dev/null

# UDRW formatına çevir (RAW read-write)
hdiutil convert EFI.dmg -format UDRW -o EFI_raw
mv EFI_raw.dmg EFI.img
rm -f EFI.dmg

echo "EFI.img hazır!"
echo "Kernel hash kontrolü:"
shasum -a 256 kernel.elf
```

**Key Features:**
- ✅ Uses macOS native `hdiutil` instead of Linux `mkfs.vfat`
- ✅ Prevents AppleDouble (._*) files with `COPYFILE_DISABLE=1`
- ✅ Creates proper FAT32 filesystem
- ✅ Converts to QEMU-compatible raw format (UDRW)
- ✅ Includes startup.nsh for automatic boot
- ✅ Provides hash verification for build confirmation

**Usage:**
```bash
# After building kernel
make clean && make
./build_efi.sh

# Verify kernel hash matches
hdiutil attach EFI.img
shasum -a 256 /Volumes/EFI/kernel.elf
hdiutil detach /Volumes/EFI
```

### 2. Correct QEMU Command with Debugcon ✅

**Critical Fix:** Added `-global isa-debugcon.iobase=0xe9` flag

**Before (Broken):**
```bash
qemu-system-x86_64 \
  -drive if=pflash,format=raw,readonly=on,file=/opt/homebrew/share/qemu/edk2-x86_64-code.fd \
  -drive if=pflash,format=raw,file=ovmf_vars.fd \
  -drive format=raw,file=EFI.img \
  -m 256M \
  -serial file:test_ring3_serial.log \
  -debugcon file:test_ring3_debugcon.log \
  -nographic
```

**After (Working):**
```bash
rm -f test_ring3_debugcon.log test_ring3_serial.log test_ring3_qemu.err

timeout 10 qemu-system-x86_64 \
  -machine q35 \
  -drive if=pflash,format=raw,readonly=on,file=/opt/homebrew/share/qemu/edk2-x86_64-code.fd \
  -drive if=pflash,format=raw,file=ovmf_vars.fd \
  -drive if=ide,format=raw,file=EFI.img \
  -m 256M \
  -serial file:test_ring3_serial.log \
  -debugcon file:test_ring3_debugcon.log \
  -global isa-debugcon.iobase=0xe9 \
  -nographic \
  -no-reboot \
  -d cpu_reset 2>test_ring3_qemu.err || true

tail -40 test_ring3_debugcon.log | tr -d '\000'
```

**Key Changes:**
- ✅ Added `-machine q35` for better hardware emulation
- ✅ Changed to `-drive if=ide` for better UEFI compatibility
- ✅ **Added `-global isa-debugcon.iobase=0xe9`** (CRITICAL)
- ✅ Clean log files before each run
- ✅ Added timeout to prevent hanging
- ✅ Capture stderr to separate file

### 3. Kernel Boot Marker Update ✅

**File:** `kernel/kernel.c`

**Change:** Updated early boot marker from 'K' to 'Z' to distinguish new builds

```c
// OLD:
__asm__ volatile("outb %0, %1" : : "a"((uint8_t)'K'), "Nd"(0xE9));

// NEW:
__asm__ volatile("outb %0, %1" : : "a"((uint8_t)'Z'), "Nd"(0xE9));  // YENİ BUILD MARKER
```

**Purpose:**
- Immediately identifies if new kernel is running
- Prevents confusion with cached old builds
- Simple visual confirmation in debugcon log

### 4. Boot-Time Debug Code Cleanup ✅

**File:** `kernel/sched/sched.c`

**Problem:** Heavy debug code in `sched_start()` was causing early boot hangs:
- `fb_print()` calls with complex formatting
- `paging_get_phys()` / `paging_get_pte()` MMU access
- `read_msr()` calls
- `dbg_dump_bytes()` memory access

**Solution:** Removed all heavy debug code, kept only simple `outb` markers

**Before:**
```c
void sched_start(void) {
    // ... setup code ...
    
    fb_print("[DBG] SCHED first: cs=");
    fb_print_hex(current_proc->context.cs);
    fb_print(" rip=");
    fb_print_hex(current_proc->context.rip);
    fb_print("[DBG] MAP rip=");
    fb_print_hex64(paging_get_phys(current_proc->context.rip));
    fb_print("[DBG] PTE rip=");
    fb_print_hex64(paging_get_pte(current_proc->context.rip));
    fb_print("[DBG] EFER=");
    fb_print_hex64(read_msr(0xC0000080));
    
    dbg_dump_bytes((const void *)current_proc->context.rip);
    
    switch_to_first(&current_proc->context);
}
```

**After:**
```c
void sched_start(void) {
    // ... setup code ...
    
    outb(0xE9, (uint8_t)'T');  // TSS setup
    
    // Ring0 mechanism: Update TSS.RSP0 for Ring3→Ring0 transitions
    if (current_proc->context.cs == GDT_USER_CODE) {
        if (!current_proc->context.rsp0) {
            outb(0xE9, (uint8_t)'!');  // PANIC: no rsp0
            for (;;) __asm__ volatile("cli; hlt");
        }
        gdt_set_kernel_stack(current_proc->context.rsp0);
        __asm__ volatile("" ::: "memory");
        map_kernel_stack_pages_into_pml4(current_proc->context.cr3, current_proc->context.rsp0);
    } else if (current_proc->context.rsp0) {
        gdt_set_kernel_stack(current_proc->context.rsp0);
    }
    
    outb(0xE9, (uint8_t)'@');  // About to switch_to_first
    
    switch_to_first(&current_proc->context);
}
```

**Why This Matters:**
- Boot-time code runs with unstable MMU state
- Heavy debug operations can trigger page faults
- Simple `outb` is safe and immediate
- Detailed debug can be added AFTER first context switch

## Verification Process

### 1. Hash Verification
```bash
# Build kernel
make clean && make

# Get kernel hash
shasum -a 256 kernel.elf

# Build EFI.img
./build_efi.sh

# Verify kernel in image matches
hdiutil attach EFI.img
shasum -a 256 /Volumes/EFI/kernel.elf
hdiutil detach /Volumes/EFI
```

### 2. Boot Marker Verification
```bash
# Run QEMU with proper debugcon
./build_efi.sh
timeout 10 qemu-system-x86_64 \
  -machine q35 \
  -drive if=pflash,format=raw,readonly=on,file=/opt/homebrew/share/qemu/edk2-x86_64-code.fd \
  -drive if=pflash,format=raw,file=ovmf_vars.fd \
  -drive if=ide,format=raw,file=EFI.img \
  -m 256M \
  -debugcon file:test_ring3_debugcon.log \
  -global isa-debugcon.iobase=0xe9 \
  -nographic \
  -no-reboot || true

# Check for Z marker (new build)
grep "Z0" test_ring3_debugcon.log
```

### 3. Ring3 Boot Verification
```bash
# Look for successful Ring3 transition
tail -50 test_ring3_debugcon.log | grep -E "Z0|switch_to_first|kernel_first_entry|RING3_OK"
```

**Expected Output:**
```
Z0[K][EARLY_BOOT_OK] kmain entry
...
FT@tkJI
...
[U][RING3_OK]
```

## Results

### Before Fix
- ❌ EFI.img creation failed on macOS
- ❌ Debugcon logs empty (missing port config)
- ❌ Couldn't verify new builds
- ❌ Boot appeared to hang (actually debug code issue)
- ❌ No Ring3 transition

### After Fix
- ✅ EFI.img builds successfully on macOS
- ✅ Debugcon captures all kernel output
- ✅ Z marker confirms new builds
- ✅ Clean boot to scheduler
- ✅ **Ring3 transition successful!**

**Boot Sequence Confirmed:**
```
Z0                          # New kernel marker
[K][EARLY_BOOT_OK]         # kmain entry
[K][LATE_INIT_BEGIN]       # Late init start
[K][ABOUT_TO_SCHED]        # Scheduler starting
S12[Q]1                    # Scheduler initialized
FT@                        # sched_start markers
tk                         # switch_to_first
J                          # kernel_first_entry
I                          # init_process_main
[U][RING3_OK]              # Ring3 transition SUCCESS!
```

## Platform-Specific Notes

### macOS Considerations

1. **hdiutil vs mkfs.vfat**
   - macOS doesn't have `mkfs.vfat`
   - `hdiutil` is the native tool
   - UDRW format works with QEMU

2. **AppleDouble Files**
   - macOS creates `._*` resource fork files
   - Can confuse UEFI bootloader
   - Use `COPYFILE_DISABLE=1` and `cp -X`

3. **QEMU Installation**
   - Install via Homebrew: `brew install qemu`
   - OVMF firmware: `/opt/homebrew/share/qemu/edk2-x86_64-code.fd`

4. **File System Paths**
   - Use `/Volumes/EFI` for mounted images
   - Automatic mount/unmount with hdiutil

### Linux Considerations

For Linux, the original approach with `mkfs.vfat` and `mtools` works:

```bash
dd if=/dev/zero of=EFI.img bs=1M count=64
mkfs.vfat -F 32 EFI.img
mmd -i EFI.img ::/EFI
mmd -i EFI.img ::/EFI/BOOT
mcopy -i EFI.img bootloader/efi/BOOTX64.EFI ::/EFI/BOOT/
mcopy -i EFI.img kernel.elf ::/EFI/BOOT/
```

## Integration with Existing Documentation

This update complements:
- `docs/setup/MACOS_SETUP_GUIDE.md` - Add EFI.img build instructions
- `docs/development/BUILD_SYSTEM_INTEGRATION_SUMMARY.md` - Reference macOS-specific process
- `docs/development/QEMU_TEST_SUITE_DOCUMENTATION.md` - Update QEMU command examples

## Lessons Learned

### 1. Platform-Specific Tooling
- Don't assume Linux tools are available everywhere
- Use native platform tools when possible
- Document platform differences clearly

### 2. Debug Output Configuration
- Always verify debug ports are configured
- Empty logs don't mean "not booting"
- Check QEMU flags carefully

### 3. Boot-Time Debug Safety
- Avoid heavy operations during early boot
- MMU state may be unstable
- Simple markers are safer than complex formatting

### 4. Build Verification
- Hash verification prevents "cached old build" confusion
- Unique markers help identify new builds
- Always verify what's actually in the image

## Conclusion

The macOS build system is now fully functional with:
- ✅ Native EFI.img creation using hdiutil
- ✅ Proper QEMU debugcon configuration
- ✅ Clean boot-time debug code
- ✅ Successful Ring3 transition
- ✅ Hash-verified build process

**Status:** Production-ready for macOS development

---

**Next Steps:**
1. Update MACOS_SETUP_GUIDE.md with build_efi.sh usage
2. Add automated build script to Makefile
3. Document Ring3 test suite execution
4. Create CI/CD pipeline for macOS builds


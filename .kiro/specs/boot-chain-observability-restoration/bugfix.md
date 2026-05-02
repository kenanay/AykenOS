# Bugfix Requirements Document

**Author**: Kenan AY - Architectural Steward  
**Created**: 2026-04-11  
**Status**: Implementation Ready

## Introduction

The boot chain observability evidence pipeline in AykenOS has regressed. The system cannot produce QEMU/kernel trace evidence for boot-trace dependent validations. Historical evidence shows boot markers were previously captured (`ApqiggggIBK0[K][EARLY_BOOT_OK]`), indicating this is a regression in the evidence capture path rather than a fundamental debugcon failure. This blocks specific boot-boundary validations that require QEMU kernel trace evidence, though it does not block Phase-16 as a whole (CURRENT_PHASE=15 OFFICIALLY CLOSED, Phase-16 pending).

## Bug Analysis

### Current Behavior (Defect)

1.1 WHEN QEMU boots with OVMF and the EFI image THEN the debugcon log file is 0 bytes with no bootloader markers

1.2 WHEN QEMU boots with OVMF and the EFI image THEN the debugcon log file contains no kernel entry markers ([[AYKEN_BOOT_OK]])

1.3 WHEN QEMU boots with OVMF and the EFI image THEN the serial log file is 0 bytes with no output

1.4 WHEN the bootloader executes THEN no observable markers are emitted to any output channel (debugcon, serial, or UEFI console)

1.5 WHEN the kernel entry point is reached THEN no observable markers are emitted to debugcon despite early outb instructions

1.6 WHEN boot-trace dependent validations run THEN they fail or become non-provable due to missing QEMU kernel trace evidence

### Expected Behavior (Correct)

2.1 WHEN QEMU boots with OVMF and the EFI image THEN the debugcon log SHALL contain bootloader start marker "[B][UEFI_BOOT_START]"

2.2 WHEN the bootloader loads the kernel ELF THEN the debugcon log SHALL contain "[B][KERNEL_ELF_LOADED]" marker

2.3 WHEN the bootloader jumps to kernel THEN the debugcon log SHALL contain "[B][JUMP_NOW]" marker

2.4 WHEN the kernel entry point executes THEN the debugcon log SHALL contain "[[AYKEN_BOOT_OK]]" marker

2.5 WHEN the kernel reaches kmain_real THEN the debugcon log SHALL contain "[K][EARLY_BOOT_OK]" marker

2.6 WHEN boot-trace dependent validations run THEN they SHALL successfully locate kernel trace markers in QEMU output

2.7 WHEN the boot chain completes THEN at least one output channel (debugcon, serial, or UEFI console) SHALL capture deterministic boot markers

### Unchanged Behavior (Regression Prevention)

3.1 WHEN the kernel boots successfully THEN the system SHALL CONTINUE TO initialize all subsystems correctly

3.2 WHEN QEMU runs with existing QEMU flags THEN the system SHALL CONTINUE TO boot without hanging

3.3 WHEN the bootloader loads the kernel ELF THEN the system SHALL CONTINUE TO map kernel segments to higher-half addresses

3.4 WHEN the kernel initializes THEN the system SHALL CONTINUE TO set up paging, heap, and memory management

3.5 WHEN existing validation tests run THEN they SHALL CONTINUE TO execute without breaking due to evidence pipeline changes

3.7 WHEN the fix is implemented THEN it SHALL remain within non-architectural bugfix boundaries (no new syscalls, execution layers, or contracts) per architectural freeze requirements

3.6 WHEN the build system compiles the kernel and bootloader THEN it SHALL CONTINUE TO produce valid ELF and EFI binaries

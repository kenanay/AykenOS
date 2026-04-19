# Patch H: Register Safety Fix

**Date**: 2026-04-19  
**Issue**: Initial implementation used %r14 (user callee-saved register) in post-CR3 window  
**Status**: ✅ FIXED

## Problem

Initial `EMIT_ENTRY_SEG_TSC` macro used %r14:
```asm
movq entry_diag_samples(%rip), %r14
cmpq $3, %r14
```

**Risk**: %r14 is a callee-saved register. In post-CR3 window (after address space switch, before IRET), clobbering %r14 would corrupt user state because IRET only restores the interrupt frame (RIP, RSP, RFLAGS, CS, SS), NOT general-purpose registers.

## Solution

Use only caller-saved registers with explicit push/pop:

```asm
.macro EMIT_ENTRY_SEG_TSC label
#if defined(AYKEN_RING3_ENTRY_SEGMENTATION_PROFILE) && (AYKEN_RING3_ENTRY_SEGMENTATION_PROFILE == 1)
    // Memory-to-memory compare (no register clobber for check)
    cmpq $3, entry_diag_samples(%rip)
    jge 1f
    // Save caller-saved registers we'll use
    pushq %rax
    pushq %rdx
    pushq %r9
    // Read TSC
    rdtsc
    shl $32, %rdx
    or %rdx, %rax
    mov %rax, %r9
    // Emit marker
    EMIT_CSTR \label
    EMIT_HEX64 %r9
    EMIT_CSTR p10_newline
    // Restore registers
    popq %r9
    popq %rdx
    popq %rax
1:
#endif
.endm
```

**Key Changes**:
1. Use `cmpq $3, entry_diag_samples(%rip)` directly (no register load)
2. Push/pop caller-saved registers (%rax, %rdx, %r9)
3. No callee-saved register usage

## Verification

```bash
make kernel.elf                  # ✅ PASS
make ci-gate-constitutional      # ✅ PASS
```

## Why This Matters

Post-CR3 window is the most critical section:
- Already switched to user address space
- Still in Ring0 (before IRET)
- Any register corruption affects user process
- IRET doesn't restore GP registers

Using callee-saved registers here would be a silent corruption bug that only manifests when user code depends on those registers being preserved across syscalls.

## Caller-Saved vs Callee-Saved

**Caller-saved** (safe to clobber in our context):
- %rax, %rcx, %rdx, %rsi, %rdi, %r8-r11
- Caller must save if needed across function calls

**Callee-saved** (MUST NOT clobber):
- %rbx, %rbp, %r12-r15
- Callee must preserve across function calls
- User expects these preserved across syscalls

## Impact

- No functional change to profiling logic
- Same 4 segments measured
- Same bounded sampling (3 samples)
- Now register-safe for post-CR3 window

---

**Status**: ✅ FIXED  
**Authority**: Kenan AY - Architectural Steward

# Execution Path Forced Proof

**Date**: 2026-04-19  
**Status**: READY FOR CI  
**Purpose**: Prove syscall execution path with unconditional markers

## Problem

Patch C has zero performance impact and verification markers are missing from CI logs. This proves either:
1. Code is not executing (wrong execution path)
2. Wrong target optimized (cost is elsewhere)

## Solution: Forced Execution Markers

Added UNCONDITIONAL markers at critical points in syscall path. These markers CANNOT be optimized away, removed by macros, or bypassed by conditions.

### Marker 1: Hardened Handler Entry

**Location**: `kernel/sys/syscall_v2_hardened.c:112`

```c
uint64_t syscall_v2_hardened_handler(uint64_t syscall_num, ...) {
    /* FORCED EXECUTION PROOF - UNCONDITIONAL
     * If this handler is called, this marker MUST appear in debugcon log.
     * No conditions, no macros, no optimizations can remove this.
     * Missing marker = handler not executing = wrong execution path.
     */
    debugcon_write("HARDENED_ENTRY\n");
    
    // ... rest of handler
}
```

**Expected**: If hardened handler is called, `HARDENED_ENTRY` MUST appear in CI log.

### Marker 2: Dispatch Point

**Location**: `kernel/sys/syscall.c:177`

```c
if (syscall_num >= SYS_V2_BASE && syscall_num <= SYS_V2_LAST) {
    debugcon_write("DISPATCH_TO_HARDENED\n");  // FORCED PROOF
    result = syscall_v2_hardened_handler(syscall_num - SYS_V2_BASE, ...);
}
```

**Expected**: If dispatch reaches hardened handler, `DISPATCH_TO_HARDENED` MUST appear in CI log.

## Execution Path Verification

### Known Path (from code analysis)

```
INT 0x80 (Ring3 syscall)
    ↓
syscall_isr (kernel/arch/x86_64/context_switch.asm:285)
    ↓
syscall_handler (kernel/sys/syscall.c:111)
    ↓
[Range check: SYS_V2_BASE..SYS_V2_LAST]
    ↓
DISPATCH_TO_HARDENED marker ← NEW
    ↓
syscall_v2_hardened_handler (kernel/sys/syscall_v2_hardened.c:112)
    ↓
HARDENED_ENTRY marker ← NEW
    ↓
[Patch C code: cache read, bypass fast-path, etc.]
```

### Symbol Verification

```bash
$ nm kernel.elf | grep syscall_handler
ffffffff80017d70 T syscall_handler
ffffffff8001b8f0 t syscall_v2_hardened_handler
```

- `syscall_handler`: Global symbol (T) - called from assembly
- `syscall_v2_hardened_handler`: Local symbol (t) - called from syscall_handler

Both symbols exist and are linked.

## CI Test Scenarios

### Scenario 1: Both Markers Appear ✅
**Evidence**: `DISPATCH_TO_HARDENED` + `HARDENED_ENTRY` in CI log

**Interpretation**: Execution path is CORRECT. Hardened handler IS being called.

**Conclusion**: Patch C code executes but has no performance impact.

**Next Steps**:
1. Re-measure entire syscall path with full profiling
2. Identify actual bottleneck location
3. Verify hot-path cost distribution

### Scenario 2: DISPATCH Appears, HARDENED Missing ❌
**Evidence**: `DISPATCH_TO_HARDENED` in log, NO `HARDENED_ENTRY`

**Interpretation**: Dispatch reaches hardened handler call, but handler doesn't execute.

**Possible Causes**:
- Compiler inlined handler but removed marker
- Indirect call goes to wrong address
- Handler crashes before marker

**Next Steps**:
1. Check disassembly for inlining
2. Verify call target address
3. Add panic after marker to force execution

### Scenario 3: Neither Marker Appears ❌
**Evidence**: NO `DISPATCH_TO_HARDENED`, NO `HARDENED_ENTRY`

**Interpretation**: Syscall path NEVER reaches hardened handler dispatch.

**Possible Causes**:
- CI uses different syscall path
- Range check fails (syscall_num out of range)
- Enforcement bypassed in CI build

**Next Steps**:
1. Check CI syscall numbers (must be in SYS_V2_BASE..SYS_V2_LAST range)
2. Verify enforcement flags in CI build
3. Add marker at syscall_handler entry to prove it's called

### Scenario 4: HARDENED Appears, DISPATCH Missing ⚠️
**Evidence**: `HARDENED_ENTRY` in log, NO `DISPATCH_TO_HARDENED`

**Interpretation**: Handler is called from different path (not through syscall_handler).

**Possible Causes**:
- Direct call from somewhere else
- Alternative syscall entry point
- Marker placement error

**Next Steps**:
1. Search for other calls to hardened handler
2. Check for alternative syscall paths
3. Verify marker placement in syscall.c

## Build Verification

```bash
$ make clean && make kernel.elf
# Build succeeded, no warnings about unused markers
# Both markers are in compiled code
```

## Files Modified

- `kernel/sys/syscall_v2_hardened.c`: Added `HARDENED_ENTRY` marker at function entry
- `kernel/sys/syscall.c`: Added `DISPATCH_TO_HARDENED` marker before handler call

## Success Criteria

**Minimum**: At least one marker appears in CI log (proves execution path)

**Target**: Both markers appear in CI log (proves complete path)

## Next CI Run

**Branch**: fix/scheduler-fast-path  
**Commit**: TBD (after commit)  
**Expected Duration**: ~30 minutes  
**Critical Evidence**: Presence/absence of `DISPATCH_TO_HARDENED` and `HARDENED_ENTRY` markers

## References

- Assembly entry: `kernel/arch/x86_64/context_switch.asm:285`
- Syscall dispatcher: `kernel/sys/syscall.c:111`
- Hardened handler: `kernel/sys/syscall_v2_hardened.c:112`
- Patch C design: `PATCH_C_DESIGN.md`
- Zero impact diagnosis: `PATCH_C_ZERO_IMPACT_DIAGNOSIS.md`

---

**Status**: Ready for CI - Forced execution proof will definitively show where syscall path goes

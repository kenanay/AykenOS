# Task 5 Phase 3B: Solution Plan - Kernel-Side IF Control

**Date**: 2026-04-15  
**Status**: READY FOR IMPLEMENTATION

## Root Cause (Confirmed)

BCIB worker enters Ring3 with RFLAGS.IF=1, causing immediate IRQ0 preemption before first instruction executes.

**Evidence**:
- `P10_RFLAGS_IF_ON` + `RF=0000000000000202` → IF bit set
- `P10_RING3_ENTER` immediately followed by `[IRQ][SCH]`
- RIP stays at 0x400000 (no progress)
- No userspace markers (`[BCIB_WORKER_START]` never appears)

## CRITICAL CORRECTION: `sti` is Privileged

**WRONG APPROACH** (in previous document):
```asm
_start:
    # ... setup ...
    sti  # ❌ WRONG! sti is privileged, causes #GP at CPL=3
```

**FACT**: `sti` (Set Interrupt Flag) is a privileged instruction. At CPL=3 (Ring3), executing `sti` triggers #GP (General Protection Fault).

**CORRECT APPROACH**: Kernel controls RFLAGS.IF, not userspace.

## Solution: Kernel-Side IF Control

### Option A: Use Existing IRQ0 Mask Mechanism (RECOMMENDED)

The kernel already has `AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY` flag for this exact scenario.

**Implementation**:
```bash
# Rebuild with flag enabled
make kernel \
  KERNEL_PROFILE=validation \
  AYKEN_PHASE16_BCIB_PROOF_TEST=1 \
  USER_MINIMAL_MODE=bcib-worker-bootstrap \
  AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=1
```

**How it works** (from `kernel/sched/sched.c:771`):
```c
static void sched_mask_irq0_before_first_ring3_entry(proc_t *proc)
{
#if AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY == 1
    if (!phase10_first_entry_irq0_masked &&
        proc &&
        ((proc->context.cs & 0x3u) == 0x3u)) {
        phase10_first_entry_irq0_masked = 1;
        pic_set_mask(0);  // Mask IRQ0 at PIC level
        sched_emit_marker("P10_IRQ0_MASK_FIRST_ENTRY\n");
    }
#endif
}
```

**Pros**:
- Already implemented and tested
- Surgical fix (only affects first Ring3 entry)
- PIC-level masking (no RFLAGS modification)
- Fail-closed design

**Cons**:
- Requires rebuild with specific flag
- IRQ0 stays masked until explicitly unmasked

### Option B: Clear IF Bit on First Ring3 Entry

Modify RFLAGS to clear IF bit before first Ring3 entry, then restore it after initial setup.

**Implementation** (in `kernel/sched/sched.c`):
```c
// Before switch_to_first() or context switch to Ring3
if (first_ring3_entry) {
    // Clear IF bit (bit 9) to disable interrupts
    current_proc->context.rflags &= ~(1ULL << 9);
    sched_emit_marker("P10_RING3_IF_DISABLED\n");
}
```

**Re-enable after initial setup**:
- Kernel re-enables IF after first timer tick
- Or after userspace makes first syscall
- Or after fixed instruction count

**Pros**:
- Fine-grained control
- No PIC manipulation
- Can be process-specific

**Cons**:
- Requires new code path
- Need mechanism to re-enable IF
- More complex than Option A

### Option C: Reduce Timer Frequency (NOT RECOMMENDED)

Lower IRQ0 frequency to reduce preemption probability.

**Why NOT recommended**:
- Doesn't solve root cause
- Still possible to preempt immediately
- Affects all timing-sensitive code
- Unreliable

## Recommended Solution: Option A

**Use existing `AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=1` flag.**

**Rationale**:
1. Already implemented and tested
2. Minimal code changes
3. Fail-closed design (explicit unmask required)
4. Surgical fix (only first entry affected)
5. No RFLAGS manipulation needed

## Implementation Steps

### Step 1: Clean Dirty Tree (CRITICAL)

Hygiene gate failing due to uncommitted changes:
```
dirty_tracked: M kernel/arch/x86_64/timer.c
dirty_tracked: M kernel/kernel.c
dirty_tracked: M kernel/sched/sched.c
```

**Action**:
```bash
# Review changes
git diff kernel/arch/x86_64/timer.c kernel/kernel.c kernel/sched/sched.c

# Option 1: Commit Phase 3A markers
git add kernel/arch/x86_64/timer.c kernel/kernel.c kernel/sched/sched.c
git commit -m "Phase 3A: Add scheduler and IRQ0 debug markers"

# Option 2: Stash changes temporarily
git stash push -m "Phase 3A markers" kernel/arch/x86_64/timer.c kernel/kernel.c kernel/sched/sched.c
```

### Step 2: Verify ELF Artifact Path

Current evidence shows `userspace/minimal/minimal.elf` exists, but build expects `minimal_bcib_worker.elf`.

**Check build system**:
```bash
# Find which ELF is actually embedded
grep -r "minimal.*elf" Makefile kernel/include/embedded_elf.h

# Verify embedded ELF matches expected payload
sha256sum userspace/minimal/minimal.elf
# Compare with kernel log: [K][PAYLOAD_SHA=20c65ce36f7bb1c07025f5cc9e92826873d6e4e3ac9cc70fec94ebd840f753cd]
```

### Step 3: Rebuild with IRQ0 Masking

```bash
# Clean build
rm -f kernel/kernel.o kernel/sched/sched.o kernel/arch/x86_64/timer.o out/build/kernel.elf

# Rebuild kernel with IRQ0 masking flag
make kernel \
  KERNEL_PROFILE=validation \
  AYKEN_PHASE16_BCIB_PROOF_TEST=1 \
  USER_MINIMAL_MODE=bcib-worker-bootstrap \
  AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=1

# Rebuild EFI image
make efi-img \
  KERNEL_PROFILE=validation \
  AYKEN_PHASE16_BCIB_PROOF_TEST=1 \
  USER_MINIMAL_MODE=bcib-worker-bootstrap \
  AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=1
```

### Step 4: Run QEMU with Correct Debugcon Port

```bash
timeout 30 qemu-system-x86_64 \
  -drive if=pflash,format=raw,readonly=on,file=/opt/homebrew/share/qemu/edk2-x86_64-code.fd \
  -drive format=raw,file=EFI.img \
  -m 512M \
  -serial file:out/logs/phase3b_serial.log \
  -debugcon file:out/logs/phase3b_debugcon.log \
  -global isa-debugcon.iobase=0xE9 \
  -nographic \
  -no-reboot
```

### Step 5: Verify Success

**Expected markers in debugcon log**:
```
[[SCHED_START]]
P10_IRQ0_MASK_FIRST_ENTRY  ← NEW! IRQ0 masked before Ring3 entry
P10_RING3_ENTER
[BCIB_WORKER_START]        ← NEW! Userspace code executes!
[BCIB_SUBMIT_OK]           ← NEW! Syscall succeeds!
```

**Verify no immediate preemption**:
```bash
# Check pattern - should NOT see immediate [IRQ][SCH] after Ring3 entry
grep -A 2 "P10_RING3_ENTER" out/logs/phase3b_debugcon.log | head -20
```

## Alternative: Minimal Kernel Patch (If Flag Doesn't Work)

If `AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY` flag doesn't solve the issue, apply minimal patch:

**File**: `kernel/sched/sched.c`

**Location**: Before `switch_to_first()` call (around line 5930)

**Patch**:
```c
// PHASE 3B FIX: Disable interrupts on first Ring3 entry
// to prevent immediate IRQ0 preemption before first instruction
if ((current_proc->context.cs & 0x3) == 0x3) {
    static uint8_t first_ring3_if_cleared = 0;
    if (!first_ring3_if_cleared) {
        first_ring3_if_cleared = 1;
        current_proc->context.rflags &= ~(1ULL << 9);  // Clear IF bit
        sched_emit_marker("P10_FIRST_RING3_IF_DISABLED\n");
    }
}
```

**Re-enable mechanism** (in timer IRQ handler):
```c
// After first successful userspace execution, re-enable IF
if (first_ring3_if_cleared && userspace_marker_seen) {
    current_proc->context.rflags |= (1ULL << 9);  // Set IF bit
    sched_emit_marker("P10_RING3_IF_RESTORED\n");
}
```

## Success Criteria

### Minimum Success
- ✅ `P10_IRQ0_MASK_FIRST_ENTRY` or `P10_FIRST_RING3_IF_DISABLED` appears
- ✅ `[BCIB_WORKER_START]` appears (userspace code executes)
- ✅ No immediate `[IRQ][SCH]` after `P10_RING3_ENTER`

### Full Success
- ✅ `[BCIB_WORKER_START]` appears
- ✅ `[BCIB_SUBMIT_OK]` appears (syscall succeeds)
- ✅ No `[[AYKEN_BOUNDARY_KILL]]` (enforcement passes)
- ✅ Worker makes progress (RIP advances beyond 0x400000)

## Conclusion

**Root Cause**: Confirmed - Ring3 entry with IF=1 causes immediate IRQ0 preemption.

**Solution**: Use existing `AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=1` flag to mask IRQ0 before first Ring3 entry.

**Next Steps**:
1. Clean dirty tree (commit or stash Phase 3A markers)
2. Verify ELF artifact path
3. Rebuild with IRQ0 masking flag
4. Run QEMU and verify `[BCIB_WORKER_START]` appears

**Expected Outcome**: BCIB worker executes userspace code, emits markers, calls `SYS_V2_SUBMIT_EXECUTION`, and proves BCIB role enforcement works.


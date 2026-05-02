# Phase 16 Performance Regression RCA - Next Steps

**Date**: 2026-04-17  
**Status**: Task 1 Complete - Expanding Investigation Scope

## Key Finding

Phase 16 features (BCIB worker, boundary enforcement, probe validation, diagnostic markers) are NOT the source of the +1,506ms boot time regression. Feature isolation measurements show zero effect from disabling these features.

## Recommended Investigation Path

### Option 1: Git Bisect (Recommended)

Use binary search to find the exact commit that introduced the regression:

```bash
# Start bisect
git bisect start

# Mark current commit as bad (has regression)
git bisect bad 6e883e64

# Mark baseline as good (no regression)
git bisect good 050332220d9  # April 9 baseline

# For each commit git bisect checks out:
make clean
make KERNEL_PROFILE=validation USER_MINIMAL_MODE=syscall-v2-runtime efi-img
# Run CI performance gate or boot measurement
# If boot_time > 11,752ms: git bisect bad
# If boot_time <= 11,752ms: git bisect good

# Bisect will identify the exact regression commit
```

**Advantages**:
- Finds exact commit that introduced regression
- Efficient (log N commits to check)
- Definitive answer

**Disadvantages**:
- Time-consuming (need to build and test each commit)
- May hit commits that don't build cleanly

### Option 2: Manual Commit Review

Review all commits between baseline and current:

```bash
git log --oneline 050332220d9..6e883e64
```

Focus on commits that could affect boot time:
- Compiler flag changes (Makefile, build scripts)
- Linker script changes (memory layout)
- Early boot code changes (kernel/kernel.c, boot/, loader/)
- Memory allocator changes (kernel/mem/)
- Interrupt handling changes (kernel/irq/)
- Timer/clock changes (kernel/time/)

**Advantages**:
- Can identify multiple contributing factors
- Provides context for changes

**Disadvantages**:
- Time-consuming manual review
- May miss non-obvious causes

### Option 3: CI Environment Verification

Run measurements in the authoritative CI environment (Linux x86_64 GitHub Actions):

```bash
# Push current branch to trigger CI
git push origin HEAD:perf-rca-test

# Check CI results for boot_time_ms
# Compare with local measurements
```

**Purpose**:
- Verify local Darwin arm64 measurements match CI Linux x86_64
- Confirm regression exists in CI (not just local artifact)
- Ensure we're measuring the same thing as the baseline

**Note**: Local measurements showed 12190ms wall-clock time, but CI measures boot_time_ms differently (from debugcon log timestamps, not wall-clock).

### Option 4: Profiling / Instrumentation

Add detailed timing instrumentation to identify slow code paths:

```c
// Add to kernel/kernel.c and other boot-critical files
uint64_t start_time = rdtsc();
// ... code section ...
uint64_t end_time = rdtsc();
dual_channel_write("[PERF] section_name: ");
print_u64(end_time - start_time);
dual_channel_write(" cycles\n");
```

**Advantages**:
- Identifies specific slow code paths
- Can pinpoint exact bottleneck

**Disadvantages**:
- Requires extensive instrumentation
- May alter timing (observer effect)
- Time-consuming to add and analyze

## Recommended Approach

**Phase 1: Quick Verification (1-2 hours)**
1. Run measurements in CI to verify regression exists in authoritative environment
2. Checkout baseline commit and verify it produces 10,684ms boot_time

**Phase 2: Binary Search (4-8 hours)**
3. Use git bisect to find exact regression commit
4. Analyze the identified commit for root cause

**Phase 3: Root Cause Analysis (2-4 hours)**
5. Once regression commit is found, analyze the specific changes
6. Determine if it's a code change, build flag change, or other factor
7. Design targeted fix

**Phase 4: Fix Implementation (2-4 hours)**
8. Implement fix based on root cause
9. Verify fix restores boot_time to ≤11,752ms
10. Ensure Phase 16 functionality preserved

## Alternative Hypotheses to Investigate

If git bisect doesn't find a single regression commit, consider:

1. **Cumulative effect**: Multiple small changes adding up to +1,506ms
2. **Build system drift**: Compiler version, flags, or linker settings changed
3. **CI infrastructure change**: GitHub Actions runner specs changed
4. **Measurement methodology change**: boot_time_ms calculation changed
5. **Non-deterministic behavior**: Regression only appears intermittently

## Files to Review

Based on typical boot time regressions, focus on:

- `Makefile` - Compiler flags, optimization levels
- `kernel/kernel.c` - Main boot sequence
- `kernel/mem/` - Memory allocator (heap allocation during boot)
- `kernel/irq/` - Interrupt setup (IRQ initialization overhead)
- `boot/` - Early boot code (before kernel_main)
- `loader/` - EFI loader (before kernel handoff)
- `scripts/ci/gate_performance.sh` - CI measurement methodology

## Success Criteria

Investigation is complete when:
1. Exact regression source identified (specific commit or change)
2. Root cause understood (why that change caused +1,506ms overhead)
3. Fix designed and validated (boot_time ≤ 11,752ms)
4. Phase 16 functionality preserved (all tests pass)

## Timeline Estimate

- **Quick path** (if git bisect finds obvious commit): 6-12 hours
- **Medium path** (if multiple commits contribute): 12-24 hours
- **Long path** (if root cause is subtle): 24-48 hours

## Contact

If investigation stalls or requires architectural decisions, consult:
- Kenan AY (Architectural Steward)
- Review constitutional performance requirements (10% threshold)
- Consider waiver process if regression is unavoidable

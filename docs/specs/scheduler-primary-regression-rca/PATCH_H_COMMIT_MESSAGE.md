# Patch H: Entry Segmentation Profiling

**Commit Message**:
```
perf(ring3): Add entry segmentation profiling for bottleneck identification

Add surgical tick measurements to Ring3 entry path to identify which
segment contains the remaining ~9.5% performance regression.

Context:
- Patch F confirmed boundary enforcement contributes ~8.5% overhead
- Patch G confirmed diagnostic markers contribute ~0% overhead
- Remaining ~9.5% regression is in Ring3 transition mechanics
- Entry window dominates at 22.6M ticks (81% of total latency)

Implementation:
- Add RDTSC helper (kernel/include/ayken_rdtsc.h)
- Add bounded sampling (3 samples max) to avoid marker spam
- Measure 4 key segments (coarse-grained profiling):
  1. ENTRY_START (before CR3 pivot)
  2. AFTER_CR3 (after CR3 pivot)
  3. AFTER_TEXT_PROOF (after post-CR3 text proof, if enabled)
  4. BEFORE_IRET (before final IRET)
- Add AYKEN_RING3_ENTRY_SEGMENTATION_PROFILE build flag
- Register-safe: Uses only caller-saved registers with push/pop
- No user callee-saved register clobbering

Expected Outcome:
- Identify dominant segment (likely POST_CR3_TEXT_PROOF or CR3_PIVOT)
- Guide Patch I optimization target
- Segment costs should sum to ~22.6M ticks

Measurement Strategy:
- Bounded sampling prevents marker spam (learned from Patch E)
- RDTSC overhead is ~20-30 cycles (negligible vs 22M tick window)
- Markers only in performance build, not production

Constitutional Compliance:
- Measurement only, no semantic changes
- No policy changes
- No security boundary changes
- Fail-closed semantics preserved
- Determinism preserved (RDTSC is deterministic)

Authority: Kenan AY - Architectural Steward
Spec: scheduler-primary-regression-rca
Task: 3.2 (Optimize enforcement hot-path - Patch H profiling)
```

**Files Changed**:
- `kernel/include/ayken_rdtsc.h` (NEW) - RDTSC helper
- `kernel/arch/x86_64/ring3_enter.S` - Entry segmentation markers
- `Makefile` - AYKEN_RING3_ENTRY_SEGMENTATION_PROFILE flag
- `docs/specs/scheduler-primary-regression-rca/PATCH_H_ENTRY_SEGMENTATION_PLAN.md` (NEW) - Plan document

**Build Verification**:
```bash
make kernel.elf  # PASS
```

**Next Steps**:
1. Commit and push to GitHub
2. Trigger CI run
3. Analyze qemu_debugcon.log for segment costs
4. Identify dominant segment
5. Design Patch I optimization based on findings

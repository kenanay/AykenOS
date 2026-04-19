# Patch C - Next Action Plan

> **2026-04-19 correction:** The "system does not use it" conclusion below was too strong.
> The reliable RCA is: production syscall diagnostics polluted the hot path and role-cache updates
> were not centralized. Patch D makes syscall diagnostics opt-in and fixes cache coherency.

**Date**: 2026-04-19  
**Status**: 🔴 CRITICAL - Zero Impact Confirmed  
**User Verdict**: "Implemented" ≠ "Effective"

## Situation (Sert Gerçek)

Patch C kodu yazıldı, derlendi, ama **sistem onu kullanmıyor**.

### CI Evidence (Run 24633589543 + Latest)
```
boot_time_ms:             12709  (Patch B: 12714, diff: -5ms = NOISE)
syscall_latency_ms:       208.16 (Patch B: 207.90, diff: +0.26ms = NOISE)
context_switch_latency:   208.16 (Patch B: 207.90, diff: +0.26ms = NOISE)
```

**Verdict**: ❌ ZERO IMPACT

### Build Evidence
```
AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE=1  ✅ (Makefile line 100)
Enforcement compiled in                       ✅
Patch C code exists                          ✅
```

### Execution Evidence
```
DIAG_HOT_* markers:        ❌ MISSING
PATCH_C_* markers:         ⏳ AWAITING (commit 44f0f7e1)
```

## User's Diagnosis (Doğru)

**"Kod yazılmış olabilir, ama sistem onu kullanmıyor olabilir."**

Bu şu anlama gelir:
1. Execution path Patch C'ye uğramıyor
2. CI performance harness başka yolu ölçüyor
3. Optimizasyon kalitesi değil, **entegrasyon doğruluğu** problemi

## Critical Question

**Patch C kodu gerçekten çalışıyor mu?**

### Kanıt 1: Marker Presence
- `PATCH_C_CACHE_HIT` veya `PATCH_C_CACHE_MISS` var mı?
- `PATCH_C2_FAST_PATH` veya `PATCH_C2_SLOW_PATH` var mı?

**If YES**: Kod çalışıyor ama etkisiz → optimization insufficient  
**If NO**: Kod çalışmıyor → integration failure

### Kanıt 2: Syscall Handler
- CI performance test `syscall_v2_hardened_handler()` çağırıyor mu?
- Yoksa legacy `syscall_v2_handler()` mi kullanıyor?

### Kanıt 3: Hot-Path Distribution
- Gerçek hot-path nerede?
- Patch C hedefi doğru mu?

## Next Action (Sıralı)

### Step 1: Marker Evidence (IMMEDIATE)
```bash
# Next CI run'da:
grep "PATCH_C_CACHE" debugcon.log
grep "PATCH_C2_" debugcon.log
```

**If markers present**:
→ Go to Step 3 (re-measure hot-path)

**If markers missing**:
→ Go to Step 2 (verify syscall handler)

### Step 2: Syscall Handler Verification (IF MARKERS MISSING)
```bash
# Check which handler is linked:
nm kernel.elf | grep "syscall.*handler"

# Check if hardened handler is called:
objdump -d kernel.elf | grep -A 20 "syscall_v2_hardened_handler"

# Add forced panic to prove execution:
# In syscall_v2_hardened.c:
panic("SYSCALL_HARDENED_HANDLER_EXECUTING");
```

**If hardened handler NOT called**:
→ CI uses legacy path, Patch C is dead code

**If hardened handler IS called**:
→ Markers should appear, investigate why they don't

### Step 3: Hot-Path Re-Measurement (IF MARKERS PRESENT)
```bash
# Run hot-path analyzer on CI log:
python scripts/ci/analyze_enforcement_hotpath.py debugcon.log

# Check if segment costs changed:
# Before Patch C:
#   ctx_type: 143k, bypass_check: 161k, validate: 195k
# After Patch C:
#   ctx_type: ???, bypass_check: ???, validate: ???
```

**If costs unchanged**:
→ Optimization has no effect, wrong target

**If costs changed but total same**:
→ Cost moved elsewhere, need full syscall profile

### Step 4: Full Syscall Profile (IF HOT-PATH INSUFFICIENT)
```bash
# Add markers for ENTIRE syscall path:
# - Entry to handler
# - Boundary init
# - Enforcement checks
# - Dispatch
# - Handler execution
# - Return

# Measure EVERYTHING, not just hot-path segments
```

## Hypotheses (Ranked by Likelihood)

### Hypothesis 1: CI Uses Different Syscall Path (HIGH)
**Evidence**:
- Markers missing
- Performance unchanged
- Enforcement enabled in build

**Test**:
- Check if CI performance harness uses hardened handler
- Verify syscall entry point in CI test

**If confirmed**:
- Patch C optimizes wrong path
- Need to optimize CI measurement path

### Hypothesis 2: Patch C Code Not Integrated (MEDIUM)
**Evidence**:
- Markers missing
- Code exists but not called

**Test**:
- Add panic in Patch C code
- Verify execution in CI

**If confirmed**:
- Integration bug
- Need to fix call site

### Hypothesis 3: Optimization Insufficient (LOW)
**Evidence**:
- Would show SOME impact, not zero

**Test**:
- Check if markers present
- Re-measure segment costs

**If confirmed**:
- Need deeper optimization
- Or different target

## Decision Tree

```
Next CI Run
    │
    ├─ Markers Present?
    │   ├─ YES → Re-measure hot-path
    │   │         ├─ Costs changed? → Investigate why no perf impact
    │   │         └─ Costs same? → Wrong target, need full profile
    │   │
    │   └─ NO → Verify syscall handler
    │             ├─ Hardened handler called? → Why no markers?
    │             └─ Legacy handler called? → Patch C is dead code
    │
    └─ Constitutional Fail?
        └─ Fix separately, doesn't affect Patch C diagnosis
```

## Success Criteria (Revised)

### Minimum (Diagnostic)
- ✅ Marker presence/absence confirmed
- ✅ Execution path identified
- ✅ Root cause documented

### Target (Performance)
- ❌ syscall_latency: <192ms (current: 208ms)
- ❌ boot_time: <11752ms (current: 12709ms)
- ❌ context_switch: <192ms (current: 208ms)

**Gap**: Still 18.7% over baseline

## User's Verdict (Final)

**"Patch C şu an 'implemented' olabilir, ama 'effective' değil."**

**Doğru**. Bu ikisini karıştırmamak gerekiyor.

## Next Immediate Action

1. **Wait for CI run** (commit 44f0f7e1)
2. **Check for markers** in CI log
3. **If missing**: Verify syscall handler
4. **If present**: Re-measure hot-path
5. **Document findings** with evidence

## Timeline

- **Now**: Awaiting CI run (commit 44f0f7e1)
- **+30 min**: CI completes
- **+45 min**: Download artifacts, check markers
- **+60 min**: Execute Step 1, 2, or 3 based on evidence
- **+90 min**: Document findings, propose next patch

## Critical Insight

**Problem is not optimization quality, it's integration correctness.**

Code can be perfect, but if system doesn't use it, impact is zero.

**User's diagnosis confirmed**: "Cost hesapladığın yerde değil" → We're optimizing the wrong path or the path isn't executing.

---

**Status**: 🔍 Awaiting marker evidence to determine if Patch C executes at all.

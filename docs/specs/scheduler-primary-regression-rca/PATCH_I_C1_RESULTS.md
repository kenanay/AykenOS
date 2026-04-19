# Patch I-C1 Results: PCID Test INVALID - Freeze Guard Rejection

**Date**: 2026-04-19  
**CI Run**: 24638819404  
**Commit**: d218a5c3  
**Verdict**: ❌ TEST INVALID - PCID BLOCKED BY FREEZE GUARD

## Critical Finding: PCID is Locked by Freeze Guard

**CI Status**: Only workspace gate ran, freeze gates (including performance) did NOT run.

**Root Cause**: `ci-freeze-guard` explicitly rejects `AYKEN_CR3_PCID=1`:

```makefile
@if [ "$(AYKEN_CR3_PCID)" != "0" ]; then \
    echo "ERROR: ci-freeze requires AYKEN_CR3_PCID=0 (current=$(AYKEN_CR3_PCID))"; \
    exit 2; \
fi
```

**Location**: `Makefile` lines 1603-1606

## What Happened

1. Patch I-C1 set `AYKEN_CR3_PCID=1` in Makefile
2. CI triggered and ran workspace gate (PASS)
3. Freeze guard checked PCID value
4. Freeze guard rejected PCID=1 (constitutional freeze discipline)
5. Freeze gates (performance, constitutional, etc.) did NOT run
6. No performance measurement occurred

## Evidence

**CI Jobs**:
```json
{"conclusion":"success","name":"WS 3.4 — Workspace Authority Boundary"}
```

Only workspace gate ran. No freeze gates, no performance gate.

**Artifacts**:
- Only `ci-gate-workspace-24638819404` artifact exists
- No `freeze-evidence-*` artifacts
- No performance gate report

**Constitutional Freeze Discipline** (`docs/governance/knobs.md`):
```
- `ci-freeze-guard` must enforce `AYKEN_CR3_PCID=0`.
- `ci-gate-ring3-execution-phase10a2` must pass `AYKEN_CR3_PCID=0` explicitly.
- `ci-gate-ring3-user-leaf-rule` must also pass `AYKEN_CR3_PCID=0` explicitly.
```

## Why PCID is Locked

**Constitutional Freeze Discipline** enforces PCID=0 for:
1. **Measurement reproducibility**: PCID affects TLB behavior, which affects timing
2. **Baseline consistency**: All baselines were measured with PCID=0
3. **Determinism contract**: PCID introduces micro-architectural variance
4. **Ring3 execution contract**: Ring3 gates require PCID=0 for validation

**PCID is NOT a "feature flag" - it's a measurement invariant.**

## Comparison with ENTRY_GUARD

| Flag | Patch I-B (ENTRY_GUARD) | Patch I-C1 (PCID) |
|------|------------------------|-------------------|
| **Lock Mechanism** | Measurement contract override | Freeze guard rejection |
| **CI Behavior** | Ran with override (ENTRY_GUARD=1) | Blocked before freeze gates |
| **Performance Data** | Generated (but invalid) | NOT generated |
| **Detection** | Check contract in report | Check CI jobs/artifacts |

**Key Difference**:
- ENTRY_GUARD: Measurement contract overrode flag, test ran but was invalid
- PCID: Freeze guard blocked test entirely, no measurement occurred

## Implications

### ❌ PCID Cannot Be A/B Tested

**PCID is locked at THREE levels**:
1. **Freeze guard**: Rejects PCID=1 before freeze gates run
2. **Ring3 gates**: Explicitly pass PCID=0 to execution gates
3. **Constitutional discipline**: Documented as mandatory PCID=0

**To test PCID, we would need to**:
1. Remove freeze guard check (violates constitutional discipline)
2. Update Ring3 gate contracts (requires baseline re-establishment)
3. Re-measure all baselines with PCID=1 (major change)
4. Update constitutional freeze discipline (governance change)

**This is NOT suitable for A/B testing.**

### ✅ Freeze Guard is Working Correctly

The freeze guard correctly prevented an invalid measurement:
- PCID=1 would invalidate baseline comparison
- PCID=1 would break measurement contract
- PCID=1 would violate constitutional discipline

**This is fail-closed CI working as designed.**

## Lessons Learned

### Measurement Lane Selection Error

**Patch I-C1 made a critical assumption error**:
- Assumed: "PCID not in measurement contract → can be toggled"
- Reality: "PCID locked by freeze guard → cannot be toggled"

**Correct check sequence**:
1. Check measurement contract (runtime override)
2. Check freeze guard (build-time rejection)
3. Check constitutional discipline (governance lock)

### Three Levels of Lock

**Flags can be locked at multiple levels**:

| Level | Mechanism | Detection | Example |
|-------|-----------|-----------|---------|
| **Runtime** | Measurement contract override | Check report contract fields | ENTRY_GUARD |
| **Build-time** | Freeze guard rejection | Check CI jobs/artifacts | PCID |
| **Governance** | Constitutional discipline | Check docs/governance | Both |

**All three must be checked before planning A/B test.**

## What This Proves

### ❌ PCID Test Invalid
- No performance measurement occurred
- Freeze guard blocked test correctly
- Cannot determine PCID impact

### ❌ PCID Cannot Be A/B Tested
- Locked by freeze guard (build-time)
- Locked by Ring3 gates (runtime)
- Locked by constitutional discipline (governance)

### ✅ Freeze Guard Working
- Correctly rejected invalid configuration
- Prevented baseline contamination
- Fail-closed CI functioning as designed

## Next Steps

### Option 1: Test Canonical Stub Variants (RECOMMENDED)

**Patch I-C2/I-C3**: Test canonical fetch stub configurations
- AYKEN_RING3_CANONICAL_FETCH_STUB
- AYKEN_RING3_SKIP_CR3_PIVOT
- AYKEN_RING3_SELF_RELOAD_CR3

**Check freeze guard first**: Verify these are NOT locked before testing.

### Option 2: Return to Profiling

**Patch H2**: Low-overhead profiling with memory buffer
- TEXT_PROOF ruled out (0.97%)
- ENTRY_GUARD untestable (contract lock)
- PCID untestable (freeze guard lock)
- Need profiling to identify actual bottleneck

### Option 3: Accept Current State

**Current regression**:
- Boundary enforcement: ~8.5% (Patch F)
- Remaining: ~9.5% (source unknown)
- Total: ~18%

**Focus on other optimization opportunities** instead of chasing this regression.

## Recommendation

**RECOMMENDED: Check freeze guard before next test**

**Before implementing any A/B test**:
1. Check `ci-freeze-guard` in Makefile for flag locks
2. Check `docs/governance/knobs.md` for constitutional locks
3. Check measurement contract for runtime locks
4. Only proceed if flag is NOT locked at any level

**Candidate flags to check**:
- AYKEN_RING3_CANONICAL_FETCH_STUB
- AYKEN_RING3_SKIP_CR3_PIVOT
- AYKEN_RING3_SELF_RELOAD_CR3

**If all CR3-related flags are locked**: Return to profiling (Patch H2).

## Key Insight

**Freeze guard is a HARD GATE, not a soft override.**

Unlike measurement contract (which overrides at runtime), freeze guard BLOCKS execution entirely. This is stronger enforcement for constitutional invariants.

**Hierarchy of enforcement**:
1. **Freeze guard**: Blocks invalid configurations (strongest)
2. **Measurement contract**: Overrides flags at runtime (medium)
3. **Documentation**: Describes expected behavior (weakest)

**Lesson**: Always check freeze guard BEFORE planning A/B tests.

## Artifact Locations

**CI Run**: 24638819404  
**Artifacts**: Only `ci-gate-workspace-24638819404` (no freeze evidence)  
**Freeze Guard**: `Makefile` lines 1603-1606  
**Constitutional Discipline**: `docs/governance/knobs.md`

---

**Status**: PCID test invalid (freeze guard rejection)  
**Next**: Check freeze guard for canonical stub flags, or return to profiling  
**Authority**: Kenan AY - Architectural Steward


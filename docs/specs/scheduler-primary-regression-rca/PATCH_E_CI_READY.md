# Patch E - CI Ready for Performance Verification

**Date**: 2026-04-19  
**Commit**: 12316741  
**Branch**: fix/scheduler-fast-path  
**Status**: 🚀 PUSHED TO CI

## 🎯 What Changed

### Patch E: Ring3 Transition Proof Throttle

**Target**: Entry window (22M ticks, 71% of total latency)

**Changes**:
1. Ring3 transition markers → one-shot emission
2. PIC mask dumps → one-shot emission  
3. Frame proof functions → one-shot emission
4. Register preservation → use memory flag instead of r13

**Preserved**:
- MARK:SW / MARK:IRET cadence (measurement contract)
- AYKEN_PERF_* markers (phase tracking)
- Mailbox markers (capability tracking)

## 📊 Expected Impact

### Before (CI Run 24634827102)

```
Total Latency: 30.95M ticks
├─ Entry Window: 22.0M ticks (71%) ← BOTTLENECK
│  ├─ Ring3 markers: 63x P10_RING3_ATTEMPT
│  ├─ Ring3 markers: 63x P10_RING3_COMMIT
│  ├─ PIC dumps: 126x PIC_MASK
│  └─ Frame proofs: multiple per transition
├─ Pure Syscall: 8.9M ticks (29%)
└─ Return: 0.5M ticks (2%)

Metrics:
- syscall_latency_ms_proxy: 207.61ms (baseline: 175.08ms, +18.6% FAIL)
- boot_time_ms: 12707ms (baseline: 10684ms, +18.9% FAIL)
```

### After (Expected)

```
Total Latency: ~20M ticks (estimated)
├─ Entry Window: ~12-15M ticks (reduced from 22M)
│  ├─ Ring3 markers: 1x P10_RING3_ATTEMPT (one-shot)
│  ├─ Ring3 markers: 1x P10_RING3_COMMIT (one-shot)
│  ├─ PIC dumps: 2x PIC_MASK (one-shot, before/after)
│  └─ Frame proofs: 1x per boot (one-shot)
├─ Pure Syscall: 8.9M ticks (unchanged)
└─ Return: 0.5M ticks (unchanged)

Metrics (estimated):
- syscall_latency_ms_proxy: ~170-180ms (baseline: 175.08ms, +0-3% PASS)
- boot_time_ms: ~11000-11500ms (baseline: 10684ms, +3-8% PASS)
```

## 🔍 How to Verify

### Step 1: Wait for CI Run

Branch pushed, CI will run automatically. Check:
https://github.com/kenanay/AykenOS/actions

### Step 2: Download Artifact

```bash
# Get the run ID from GitHub Actions
RUN_ID="<new_run_id>"

# Download artifact
gh run download ${RUN_ID} --dir /tmp/patch-e-artifacts

# Find performance evidence
PERF_DIR=$(find /tmp/patch-e-artifacts -type d -path "*/gates/performance" | head -1)
```

### Step 3: Check Metrics

```bash
# 1. Performance metrics
cat "${PERF_DIR}/report.json" | jq '.results | {
  syscall_latency_ms_proxy,
  boot_time_ms,
  preempt_iret_count,
  entry_latency_ticks,
  syscall_latency_ticks_pure
}'

# Expected:
# - syscall_latency_ms_proxy ≤ 183.836ms (baseline +5%)
# - boot_time_ms ≤ 11752ms (baseline +10%)
```

### Step 4: Verify Marker Reduction

```bash
# 2. Ring3 marker counts
DEBUGCON="${PERF_DIR}/../boot-audit/qemu_debugcon.log"

echo "P10_RING3_ATTEMPT: $(grep -c 'P10_RING3_ATTEMPT' ${DEBUGCON})"
echo "P10_RING3_COMMIT: $(grep -c 'P10_RING3_COMMIT' ${DEBUGCON})"
echo "PIC_MASK: $(grep -c 'PIC_MASK' ${DEBUGCON})"
echo "MARK:IRET: $(grep -c 'MARK:IRET' ${DEBUGCON})"

# Expected:
# - P10_RING3_ATTEMPT: 1 (was 63)
# - P10_RING3_COMMIT: 1 (was 63)
# - PIC_MASK: 2 (was 126)
# - MARK:IRET: ~61 (unchanged, measurement contract)
```

### Step 5: Check Patch C Markers

```bash
# 3. Verify Patch C still executes
grep -E "PATCH_C|DISPATCH|HARDENED" ${DEBUGCON}

# Expected:
# - DISPATCH_TO_HARDENED ✅
# - HARDENED_ENTRY ✅
# - PATCH_C_CACHE_HIT ✅
# - PATCH_C2_FAST_PATH ✅
```

## 📋 Success Criteria

### Primary (Performance Gate)

- [ ] syscall_latency_ms_proxy ≤ 183.836ms (baseline 175.08ms +5%)
- [ ] context_switch_latency_ms_proxy ≤ 183.836ms (baseline 175.08ms +5%)
- [ ] boot_time_ms ≤ 11752ms (baseline 10684ms +10%)
- [ ] violations_count = 0

### Secondary (Marker Verification)

- [ ] P10_RING3_ATTEMPT count = 1 (reduced from 63)
- [ ] P10_RING3_COMMIT count = 1 (reduced from 63)
- [ ] PIC_MASK count = 2 (reduced from 126)
- [ ] MARK:IRET count ~61 (preserved for measurement)
- [ ] Patch C markers present (execution confirmed)

### Tertiary (Functional)

- [ ] All constitutional gates pass
- [ ] Determinism replay consistency maintained
- [ ] No new violations introduced

## 🚨 If Patch E Insufficient

If metrics still exceed threshold, next steps:

### Option 1: More Aggressive Throttling

```c
// Completely disable in performance builds
#if !defined(AYKEN_PERF_BUILD) || (AYKEN_PERF_BUILD == 0)
    // Emit markers only in proof builds
    EMIT_CSTR p10_ring3_attempt
#endif
```

### Option 2: Separate Build Profiles

```makefile
# Performance build (minimal markers)
AYKEN_PERF_BUILD ?= 1
AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE ?= 0

# Proof build (full markers)
AYKEN_PERF_BUILD ?= 0
AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE ?= 1
```

### Option 3: Profile Entry Window Components

If still insufficient, profile:
1. Page table operations (CR3 pivot cost)
2. Scheduler mailbox operations
3. Remaining frame proof overhead
4. Text walk proof overhead

## 📚 References

- **Root Cause**: `CI_RUN_24634827102_FINAL_VERDICT.md`
- **Patch C Analysis**: `CI_RUN_24634827102_ANALYSIS.md`
- **Epistemology**: `EPISTEMOLOGY_CORRECTION.md`
- **Patch E Design**: `PATCH_E_TRANSITION_TRACE_THROTTLE.md`

## 🎓 Key Learnings

1. **Measurement model matters**: Optimized syscall body (29%) but metric includes entry window (71%)
2. **Debugcon I/O is expensive**: Ring3 markers (63x) + PIC dumps (126x) = significant overhead
3. **Artifact data > shell logs**: Markers were present in artifact, not in shell output
4. **Profile before optimizing**: Should have measured entry window first

---

**Status**: Waiting for CI performance gate results

**Next Action**: Monitor GitHub Actions, download artifact, verify metrics


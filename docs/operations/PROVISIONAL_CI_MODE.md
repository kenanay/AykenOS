# Provisional CI Mode

## Status: TEMPORARY

⚠️ **This document describes the current PROVISIONAL CI configuration.**

This is NOT the target architecture. This is a temporary operational mode while baremetal CI infrastructure is being prepared.

## Current State

### What We Have
- ✅ GitHub-hosted runners (`ubuntu-latest`)
- ✅ Functional CI gates (ABI, boundary, hygiene, etc.)
- ✅ Basic QEMU sanity checks
- ✅ Freeze enforcement
- ⚠️ **PROVISIONAL** performance baseline

### What We DON'T Have
- ❌ Deterministic hardware
- ❌ Fixed CPU frequency
- ❌ Stable performance authority
- ❌ True performance regression detection

## Why Provisional?

GitHub-hosted runners provide:
- **Shared CPU**: Non-deterministic performance
- **Variable frequency**: Turbo boost, power management
- **VM overhead**: Hypervisor noise
- **Image drift**: Runner images update without notice

**Result**: Performance measurements are NOT reproducible.

## What Works in Provisional Mode

### ✅ Functional Gates (Reliable)
- `ci-gate-abi`: ABI drift detection
- `ci-gate-boundary`: Symbol boundary enforcement
- `ci-gate-ring0-exports`: Export count validation
- `ci-gate-hygiene`: Git cleanliness
- `ci-gate-constitutional`: Constitutional compliance
- `ci-gate-workspace`: Workspace integrity
- `ci-gate-syscall-v2-runtime`: Syscall functional test

These gates are **deterministic** and work correctly on hosted runners.

### ⚠️ Soft Gates (Provisional)
- `ci-gate-performance`: **WARNING MODE ONLY**

Performance gate in provisional mode:
- Measures boot time, syscall latency
- Compares against baseline
- **Does NOT fail on regression** (warning only)
- Baseline is marked as `provisional`

## Authority Model

### Current Authority
```
PERF_BASELINE_AUTHORITY: "github-hosted-ubuntu-latest-x64"
```

This authority is:
- ✅ Consistent within GitHub infrastructure
- ❌ NOT deterministic across time
- ❌ NOT suitable for performance governance

### Target Authority (Future)
```
PERF_BASELINE_AUTHORITY: "baremetal-ubuntu22-x86_64-perf01"
```

This authority will be:
- ✅ Deterministic hardware
- ✅ Fixed frequency
- ✅ Stable baseline
- ✅ Constitutional performance authority

## Performance Gate Behavior

### Provisional Mode (Current)
```yaml
PERF_ENV_MISMATCH_POLICY: "warn"
PERF_REGRESSION_POLICY: "warn"
PERF_BASELINE_MODE: "provisional"
```

**Behavior**:
- Baseline exists but marked `provisional`
- Regressions logged but do NOT fail CI
- Environment mismatches logged but do NOT fail CI
- Evidence collected for future analysis

### Constitutional Mode (Target)
```yaml
PERF_ENV_MISMATCH_POLICY: "fail"
PERF_REGRESSION_POLICY: "fail"
PERF_BASELINE_MODE: "constitutional"
```

**Behavior**:
- Baseline is authoritative
- Regressions FAIL CI
- Environment mismatches FAIL CI
- Strict governance enforced

## Transition Plan

### Phase 1: Provisional CI (Current)
- [x] GitHub-hosted runners active
- [x] Functional gates enforced
- [x] Performance gate in warning mode
- [ ] Provisional baseline established

### Phase 2: Baremetal Preparation
- [ ] Dedicated x86_64 machine acquired
- [ ] BIOS configured (Turbo off, fixed frequency)
- [ ] CPU governor set to performance
- [ ] Toolchain locked
- [ ] Self-hosted runner installed
- [ ] Determinism validation passed

### Phase 3: Authority Switch
- [ ] Baremetal runner online
- [ ] New baseline initialized with baremetal authority
- [ ] Parallel CI runs (hosted + baremetal) for validation
- [ ] Performance gate switched to baremetal authority
- [ ] Hosted runner kept for functional gates

### Phase 4: Constitutional Mode
- [ ] Performance gate enforcement enabled
- [ ] Baseline marked `constitutional`
- [ ] Regression policy: `fail`
- [ ] Full deterministic CI active

## What to Trust

### Trust in Provisional Mode
✅ **Trust these verdicts**:
- ABI drift detection
- Symbol boundary violations
- Export count changes
- Git hygiene failures
- Constitutional violations
- Syscall functional failures

❌ **Do NOT trust these verdicts**:
- Performance regressions (noise)
- Boot time variance (VM overhead)
- Latency measurements (non-deterministic)

### Trust in Constitutional Mode (Future)
✅ **Trust everything**:
- All functional gates
- Performance regressions
- Baseline authority
- Deterministic measurements

## Developer Workflow

### During Provisional Mode

**For functional changes**:
```bash
# CI will validate functional correctness
git push origin feature-branch
# Wait for CI (functional gates enforced)
```

**For performance-sensitive changes**:
```bash
# CI will measure but NOT enforce
git push origin perf-optimization
# Check performance evidence manually
# Do NOT rely on CI verdict for performance
```

**For local validation**:
```bash
# Use local freeze (skips performance gate)
make ci-freeze-local
```

### After Constitutional Mode

**All changes**:
```bash
# CI will validate everything
git push origin any-branch
# Performance regressions will FAIL CI
# Baseline authority is deterministic
```

## Monitoring Provisional Mode

### What to Watch
- Performance variance trends (should be high)
- Baseline drift over time (expected)
- Runner image version changes (GitHub updates)

### Evidence Collection
Even in provisional mode, collect:
- Boot time measurements
- Syscall latency proxy
- Context switch latency proxy
- QEMU timeout events

This data will help validate baremetal setup later.

## Communication

### In PR Comments
When performance gate runs in provisional mode:

```
⚠️ Performance Gate: PROVISIONAL MODE

Boot time: 1234ms (baseline: 1200ms, +2.8%)
Status: WARNING (not enforced)

Note: Performance measurements are non-deterministic on GitHub-hosted runners.
This gate will be enforced after baremetal CI is operational.

See: docs/operations/PROVISIONAL_CI_MODE.md
```

### In Documentation
Always mark provisional baselines:

```json
{
  "baseline_authority": "github-hosted-ubuntu-latest-x64",
  "baseline_mode": "provisional",
  "deterministic": false,
  "enforcement": "warn"
}
```

## Exit Criteria

Provisional mode ends when:
1. ✅ Baremetal runner operational
2. ✅ Determinism validation passed (< 1% variance)
3. ✅ Constitutional baseline initialized
4. ✅ Parallel CI validation completed
5. ✅ Authority switch documented and committed

## Risks of Staying in Provisional Mode

### Short-term (Acceptable)
- Performance regressions undetected
- Baseline drift unnoticed
- Non-deterministic measurements

### Long-term (Unacceptable)
- Loss of performance governance
- "Provisional" becomes permanent
- Constitutional model abandoned
- Determinism never achieved

**Timeline**: Provisional mode should last < 3 months.

## References

- [Baremetal CI Setup Guide](./BAREMETAL_CI_SETUP.md)
- [Performance Baseline Gate Spec](../development/PERFORMANCE_BASELINE_GATE_SPEC.md)
- [Architecture Freeze Document](../../ARCHITECTURE_FREEZE.md)

## Accountability

**Current Mode**: PROVISIONAL  
**Target Mode**: CONSTITUTIONAL  
**Transition Owner**: TBD  
**Timeline**: TBD  
**Exit Criteria**: See above

---

**Last Updated**: 2026-02-16  
**Status**: Active (Provisional CI operational)  
**Next Review**: After baremetal runner setup

# Environment Drift Analysis - Root Cause Identified

## Critical Discovery

**GitHub CI runner image changed between baseline and current run:**

| Component | Baseline (050332220d9a) | Current (9b3358e6) | Status |
|-----------|-------------------------|---------------------|---------|
| ci_image_digest | `gha-ubuntu24-20260406.80.1-X64` | `gha-ubuntu24-20260413.86.1-X64` | ❌ CHANGED |
| env_hash | `edbe5bed0e83075ace80b5d9aa09588613afceedd8304e30891a20aae2dc4a67` | `edbe5bed0e83075ace80b5d9aa09588613afceedd8304e30891a20aae2dc4a67` | ✅ SAME |
| clang_version | `Ubuntu clang version 18.1.3 (1ubuntu1)` | `Ubuntu clang version 18.1.3 (1ubuntu1)` | ✅ SAME |
| qemu_version | `QEMU emulator version 8.2.2 (Debian 1:8.2.2+ds-0ubuntu1.16)` | `QEMU emulator version 8.2.2 (Debian 1:8.2.2+ds-0ubuntu1.16)` | ✅ SAME |

**Paradox:** env_hash is SAME but ci_image_digest is DIFFERENT!

## Performance Metrics from GitHub CI

### Actual Measurements (Run 24535837417)

```json
{
  "boot_time_ms": 12713,
  "context_switch_latency_ms_proxy": 208.213115,
  "syscall_latency_ms_proxy": 208.213115
}
```

### Baseline (050332220d9a)

```json
{
  "boot_time_ms": 10684,
  "context_switch_latency_ms_proxy": 175.081967,
  "syscall_latency_ms_proxy": 175.081967
}
```

### Regression

| Metric | Baseline | Actual | Delta | Percent |
|--------|----------|--------|-------|---------|
| boot_time_ms | 10684 | 12713 | +2029 | +19.0% |
| context_switch | 175.08 | 208.21 | +33.13 | +18.9% |
| syscall | 175.08 | 208.21 | +33.13 | +18.9% |

**Uniform ~19% regression across all metrics**

## Violations Reported

```
baseline_diff:metric_regression:syscall_latency_ms_proxy:baseline=175.081967:actual=208.213115:threshold=5%
baseline_diff:metric_regression:context_switch_latency_ms_proxy:baseline=175.081967:actual=208.213115:threshold=5%
baseline_diff:metric_regression:boot_time_ms:baseline=10684:actual=12713:threshold=10%
baseline_mismatch:scripts/ci/perf-baseline.lock.json
```

## Analysis

### Why env_hash is Same but ci_image_digest is Different?

**Hypothesis:** env_hash might not include ci_image_digest in its calculation.

Let me check what env_hash includes:

```bash
# From baseline
"env_hash": "edbe5bed0e83075ace80b5d9aa09588613afceedd8304e30891a20aae2dc4a67"

# Components that likely go into env_hash:
# - clang_version: Ubuntu clang version 18.1.3 (1ubuntu1)
# - ld_version: Ubuntu LLD 18.1.3 (compatible with GNU linkers)
# - qemu_version: QEMU emulator version 8.2.2 (Debian 1:8.2.2+ds-0ubuntu1.16)
# - nasm_version: NASM version 2.16.01
# - host_os: Linux
# - host_arch: x86_64
# - target_triple: x86_64-elf
# - kernel_profile: validation
# - marker_contract: {...}
```

**Conclusion:** env_hash does NOT include ci_image_digest, so it can be same even when runner image changes.

### What Changed in Runner Image?

GitHub Actions runner image updated from:
- `gha-ubuntu24-20260406.80.1-X64` (April 6, 2026 - version 80.1)
- `gha-ubuntu24-20260413.86.1-X64` (April 13, 2026 - version 86.1)

**Possible changes:**
- Kernel version (Linux kernel update)
- System libraries (glibc, etc.)
- CPU governor settings
- System daemons/services
- Kernel boot parameters
- Timer resolution/precision
- Scheduler settings
- Memory allocator
- File system cache behavior

### Why Uniform ~19% Regression?

**Pattern:** All metrics regressed by ~19% uniformly.

**This suggests:**
- NOT a specific code path regression
- NOT a mailbox/scheduler bug
- LIKELY a system-wide slowdown

**Possible causes:**
1. **CPU frequency scaling:** New runner image might have different CPU governor (powersave vs performance)
2. **Timer precision:** QEMU timer resolution might have changed
3. **Kernel scheduler:** Linux host scheduler settings might have changed
4. **System load:** Background services consuming more CPU
5. **Memory performance:** Different memory allocator or cache settings

## Bisect Results Reinterpretation

### What Bisect Actually Showed

All commits from baseline (050332220d9a) to current (9b3358e6) showed:
- Boot time: 11700-13900ms (all above threshold)
- Mailbox pattern: 61/61 fallback (all identical)

**This confirms:** The regression is NOT in the code, it's in the environment.

### Why Bisect Found "Baseline Update" as First Bad Commit?

Commit `71a2ef0a` (baseline update) was marked as "bad" because:
1. It updated the baseline to accept the current performance
2. But the baseline was measured in the OLD runner image (gha-ubuntu24-20260406.80.1-X64)
3. When bisect tested it in the NEW runner image (gha-ubuntu24-20260413.86.1-X64), it was slower
4. So bisect correctly identified that this commit's baseline is incompatible with new environment

## Decision Matrix

### Option 1: Update Baseline to New Environment (RECOMMENDED)

**Action:**
```bash
# Re-measure baseline commit in NEW environment
git checkout 050332220d9a
# Trigger GitHub CI run with label: baseline-update
# This will measure performance in gha-ubuntu24-20260413.86.1-X64
# Update scripts/ci/perf-baseline.lock.json with new measurements
```

**Pros:**
- Accepts that environment changed
- Establishes new baseline for new environment
- Future commits measured against correct baseline

**Cons:**
- Accepts ~19% performance loss
- Doesn't investigate WHY environment changed

**Verdict:** This is the CORRECT action if environment drift is unavoidable.

### Option 2: Pin CI Environment to Old Runner Image

**Action:**
```yaml
# .github/workflows/ci-freeze.yml
runs-on: ubuntu-24.04-20260406.80.1  # Pin to specific image
```

**Pros:**
- Maintains baseline performance
- No need to update baseline
- Consistent measurements

**Cons:**
- Old runner image might become unavailable
- Misses security updates in new images
- Not sustainable long-term

**Verdict:** Only viable if GitHub allows pinning to specific image versions.

### Option 3: Investigate Environment Difference

**Action:**
```bash
# Compare runner images
# - Kernel version
# - CPU governor
# - Timer settings
# - System services
# - Kernel boot parameters
```

**Pros:**
- Understands root cause
- Might find optimization opportunity
- Could report issue to GitHub

**Cons:**
- Time-consuming
- Might not be fixable
- GitHub controls runner images

**Verdict:** Useful for understanding, but might not lead to actionable fix.

### Option 4: Optimize Code to Compensate

**Action:**
```bash
# Profile hot paths
# Optimize scheduler/syscall/boot paths
# Aim to recover ~19% performance
```

**Pros:**
- Improves code quality
- Might exceed old baseline
- Addresses performance proactively

**Cons:**
- Significant effort
- Might not be possible to recover 19%
- Doesn't address root cause

**Verdict:** Good long-term strategy, but doesn't solve immediate CI failure.

## Recommended Action Plan

### Immediate (Unblock CI)

**Step 1:** Update baseline to new environment

```bash
# Create PR with baseline-update label
git checkout -b fix/baseline-update-new-runner-image
git push origin fix/baseline-update-new-runner-image

# In PR description:
# "Update baseline for GitHub Actions runner image gha-ubuntu24-20260413.86.1-X64
#  
#  Root cause: Runner image updated from 80.1 to 86.1 on April 13, 2026
#  Impact: ~19% uniform performance regression across all metrics
#  Decision: Accept environment drift, establish new baseline
#  
#  Evidence: Run 24535837417 shows env_hash unchanged but ci_image_digest changed"
```

**Step 2:** Add ci_image_digest to env_hash calculation

```bash
# Modify scripts/ci/gate_performance.sh to include ci_image_digest in env_hash
# This prevents future silent environment drifts
```

### Short-term (Understand)

**Step 3:** Document environment drift in ARCHITECTURE_FREEZE.md

```markdown
## Performance Baseline Authority

Baseline measurements are environment-specific:
- Runner image: gha-ubuntu24-20260413.86.1-X64
- When runner image changes, baseline MUST be updated
- env_hash alone is insufficient - check ci_image_digest
```

**Step 4:** Add CI check for runner image drift

```bash
# Add to CI workflow:
# - Detect when ci_image_digest changes
# - Automatically trigger baseline update workflow
# - Block merges until baseline is updated
```

### Long-term (Optimize)

**Step 5:** Profile and optimize hot paths

```bash
# Use perf/flamegraph to identify hot paths
# Optimize scheduler/syscall/boot paths
# Aim to recover performance loss
```

**Step 6:** Investigate runner image differences

```bash
# Compare kernel versions, CPU governor, timer settings
# Report findings to GitHub if issue is in runner image
# Consider self-hosted runners for stable environment
```

## Conclusion

**Root cause:** GitHub Actions runner image updated from 80.1 to 86.1, causing ~19% uniform performance regression.

**Evidence:**
- ✅ env_hash unchanged (toolchain same)
- ❌ ci_image_digest changed (runner image different)
- ✅ Uniform ~19% regression (system-wide, not code-specific)
- ✅ Bisect showed all commits slow (environment issue, not code regression)
- ✅ 61/61 mailbox pattern in baseline (not a new bug)

**Decision:** Update baseline to new environment, document drift, add ci_image_digest to env_hash.

**Action:** Create PR with baseline-update label, re-measure baseline in new environment.

**Timeline:** 30 minutes (trigger CI, update baseline, merge PR).


# Performance Baseline Renewal Procedure

**Effective date:** 2026-05-25
**Current authority:** `github-hosted-ubuntu-24.04-x64`
**Current phase:** Phase-17 officially closed; Phase-18 transition not activated
**Authority boundary:** A generated lock artifact is not acceptance or merge
authority. Import requires Kenan AY's recorded maintainer decision and
subsequent constitutional remote PASS; it is not Phase-17 closure.
**Duzenleyen / Gelistiren / Olusturan / Mimari Sorumlu:** Kenan AY
**Attribution boundary:** Documentation metadata only; not runtime, evidence,
baseline, merge, or closure authority.

## When Baseline Renewal is Required

The performance baseline must be renewed when any of the following changes occur:

### 1. Toolchain Changes
- **clang** version upgrade (major or minor)
- **lld** version upgrade
- **nasm** version upgrade
- **qemu** version upgrade

**Why**: These changes affect the `env_hash` in the baseline lock, causing CI to fail with `env_hash_mismatch`.

### 2. Runner Image Changes
- GitHub Actions runner image update (e.g., `ubuntu-24.04` → `ubuntu-26.04`)
- Runner image digest change (e.g., `20260201.15.1` → `20260301.1.0`)

**Why**: Runner image changes affect both `ci_image_digest` and potentially toolchain versions.

### 3. Intentional Performance Improvements
- Kernel optimization that improves boot time
- Scheduler optimization that improves context switch latency
- Syscall path optimization

**Why**: These improvements will cause the new metrics to fall outside the baseline thresholds, triggering a regression failure.

## Baseline Renewal Steps

### Step 1: Identify the Trigger
Determine which change requires baseline renewal:
- Toolchain upgrade PR
- Runner image update
- Performance optimization PR

### Step 2: Run Baseline Init Workflow

1. Go to: https://github.com/kenanay/AykenOS/actions/workflows/perf-baseline-init.yml
2. Click "Run workflow"
3. Select branch: `main` (or feature branch for testing)
4. Enter `ci_image_digest`:
   - Find current digest: Check recent CI run logs for `PERF_CI_IMAGE_DIGEST`
   - Format: `gha-ubuntu24-YYYYMMDD.X.Y-X64`
   - Example: `gha-ubuntu24-20260201.15.1-X64`
5. Click "Run workflow"

### Step 3: Verify Baseline Init Success

The workflow should:
- Exit with code `2` (expected: fail-closed with baseline write)
- Generate artifact: `perf-baseline-evidence`
- Create file: `scripts/ci/perf-baseline.lock.json`
- Validate the generated lock against the checked-out SHA, pinned digest,
  strict policy and non-zero runtime counters
- Never push the generated lock directly to `main` or any protected branch

### Step 4: Download and Commit Baseline Lock

1. Download artifact from workflow run
2. Extract `scripts/ci/perf-baseline.lock.json`
3. Verify critical fields:
   ```json
   {
     "env": {
       "ci_image_digest": "gha-ubuntu24-...",
       "env_hash": "...",
       "kernel_profile": "validation"
     },
     "raw_metrics": {
       "preempt_sw_count": >0,  // Must be > 0
       "preempt_iret_count": >0  // Must be > 0
     }
   }
   ```
4. Import the workflow-generated file into a reviewed baseline-renewal PR;
   do not hand-edit fields or copy it directly to a protected branch.
5. Commit to repository:
   ```bash
   git add scripts/ci/perf-baseline.lock.json
   git commit -m "ci(perf): renew baseline for [reason]
   
   - New env_hash: [hash]
   - New ci_image_digest: [digest]
   - Marker count: [count]
   - Reason: [toolchain upgrade / runner update / perf improvement]"
   ```

### Step 5: Create PR and Merge

1. Push to feature branch
2. Create PR with title: `ci(perf): renew baseline for [reason]`
3. Apply the governed `baseline-update` label so CI may evaluate the
   workflow-generated lock mutation; without it the lock remains immutable.
4. PR description should include:
   - Reason for renewal
   - Old vs new env_hash
   - Old vs new metrics (if performance improvement)
   - Verification that marker count > 0
4. Merge only after constitutional CI passes and Kenan AY's maintainer
   decision is recorded under the accepted single-maintainer authority model.

## Baseline Lock File Structure

```json
{
  "schema_version": 1,
  "created_at_utc": "ISO8601 timestamp",
  "git_sha": "commit hash at baseline creation",
  "env": {
    "baseline_authority": "github-hosted-ubuntu-24.04-x64",
    "ci_image_digest": "gha-ubuntu24-YYYYMMDD.X.Y-X64",
    "env_hash": "SHA256 of canonical env",
    "kernel_profile": "validation",
    "clang_version": "...",
    "ld_version": "...",
    "nasm_version": "...",
    "qemu_version": "...",
    "marker_contract": {
      "boot_ok_marker": "[K][BOOT_OK] Phase 4.4 minimal boot reached",
      "preempt_ring3_entry_guard": 1,
      "preempt_sw_count_pattern": "[SW|MARK:SW] count:",
      "preempt_iret_count_pattern": "[IRET markers] count:"
    }
  },
  "metrics": {
    "boot_time_ms": 10819,
    "context_switch_latency_ms_proxy": 0.759973,
    "syscall_latency_ms_proxy": 0.759973
  },
  "policy": {
    "env_mismatch_policy": "fail",
    "thresholds_percent": {
      "boot_time_ms": 10,
      "context_switch_latency_ms_proxy": 5,
      "syscall_latency_ms_proxy": 5
    }
  },
  "raw_metrics": {
    "preempt_sw_count": 39508,
    "preempt_iret_count": 39508,
    "entry_latency_ticks": {
      "ticks": 123456,
      "available": true
    },
    "syscall_latency_ticks_pure": {
      "ticks": 45678,
      "available": true
    }
  }
}
```

Notes:
- `preempt_ring3_entry_guard` is now part of the marker contract and must be present in renewed CI baselines.
- Proxy ms metrics remain the baseline-enforced surface for compatibility.
- Split tick metrics are carried in `raw_metrics` for measurement-model diagnostics and should not be hand-edited into `metrics`.

## Critical Validations

Before committing a new baseline:

1. **Marker Count > 0**
   ```bash
   jq '.raw_metrics.preempt_sw_count' scripts/ci/perf-baseline.lock.json
   # Must be > 0
   ```

2. **Env Hash Present**
   ```bash
   jq '.env.env_hash' scripts/ci/perf-baseline.lock.json
   # Must be 64-char hex string
   ```

3. **CI Image Digest Pinned**
   ```bash
   jq '.env.ci_image_digest' scripts/ci/perf-baseline.lock.json
   # Must match format: gha-ubuntu24-YYYYMMDD.X.Y-X64
   ```

4. **Policy is Strict**
   ```bash
   jq '.policy.env_mismatch_policy' scripts/ci/perf-baseline.lock.json
   # Must be "fail"
   ```

## Troubleshooting

### Baseline Init Fails with Exit Code 1
**Cause**: Build failure or test failure
**Solution**: Fix the underlying issue before renewing baseline

### Baseline Init Succeeds but Marker Count is 0
**Cause**: Debug flags not enabled in validation profile
**Solution**: Verify Makefile has `AYKEN_DEBUG_SCHED ?= 1` in validation block

### CI Fails with "env_hash_mismatch" After Merge
**Cause**: Baseline was created on different environment than CI
**Solution**: Re-run baseline init on CI environment (not local)

### CI Fails with "metric_regression" After Performance Improvement
**Cause**: New metrics exceed baseline thresholds (expected)
**Solution**: Renew baseline to capture new improved metrics

## Authority and Digest Pinning

Current configuration:
- **Authority**: `github-hosted-ubuntu-24.04-x64`
- **Runner**: `ubuntu-24.04` (pinned in workflow)
- **Digest Format**: `gha-ubuntu24-YYYYMMDD.X.Y-X64`
- **Imported Candidate Digest**: `gha-ubuntu24-20260518.149.1-X64`

The imported candidate digest was generated through governed workflow
evidence. Its constitutional acceptance and any eventual merge remain
separate review-controlled decisions.

**Important**: When GitHub updates `ubuntu-latest` to point to `ubuntu-26.04`, you must:
1. Update workflow `runs-on: ubuntu-26.04`
2. Update authority (if needed)
3. Renew baseline with new digest

## See Also

- [Constitutional CI Mode](CONSTITUTIONAL_CI_MODE.md)
- [Provisional CI Mode](PROVISIONAL_CI_MODE.md)
- [Performance Gate Documentation](../development/PERFORMANCE_GATE.md)
- [Baseline Init Workflow](../../.github/workflows/perf-baseline-init.yml)
- [Live Review Enforcement Blocker](https://github.com/kenanay/AykenOS/issues/145)

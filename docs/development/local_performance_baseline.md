# Local Performance Baseline

## Overview

For local development, you can create a LOCAL authority baseline that is separate from the CI baseline.

## Authority Separation

```
CI Authority:     github-hosted-ubuntu-latest-x64
Local Authority:  local-dev-Darwin-arm64 (or your platform)
```

These are SEPARATE authorities and do not conflict.

## Creating Local Baseline

```bash
./scripts/ci/local_perf_baseline_init.sh
```

This creates or refreshes: `scripts/ci/perf-baseline.local.lock.json`

## Using Local Baseline

```bash
# Run the local performance gate
make ci-gate-performance-local

# Evaluate sampled stability separately
make ci-gate-performance-stability

# Produce Phase-17 PR-4 local diagnostic readiness evidence
# (fails closed if the local stability report fails)
make ci-gate-phase17-performance-readiness-local

# Classify an existing failing readiness run against an optional stable
# reference; this reads evidence only and cannot grant acceptance authority
make ci-gate-phase17-performance-variance-diagnostic \
  RUN_ID=local-phase17-variance-diagnostic-20260524 \
  EVIDENCE_ROOT=evidence \
  PHASE17_VARIANCE_SOURCE_RUN_ID=local-phase17-performance-readiness-20260524-r2 \
  PHASE17_VARIANCE_REFERENCE_RUN_ID=local-phase17-performance-readiness-20260524

# Run bounded PR-4B reproduction measurements using the same deterministic
# runtime contract in image-reuse and rebuild-per-run conditions
make ci-gate-phase17-performance-variance-isolation \
  RUN_ID=local-phase17-variance-isolation-20260524-r3 \
  EVIDENCE_ROOT=evidence \
  PHASE17_VARIANCE_ISOLATION_RUNS=3 \
  PHASE17_VARIANCE_ISOLATION_WARMUP=1

# Or run the local freeze suite with local perf authority active
make ci-freeze-local
```

## Important Rules

1. **Never commit local baseline** - It's in `.gitignore`
2. **Local baseline is for development only** - Not for CI/production
3. **CI uses separate baseline** - `scripts/ci/perf-baseline.lock.json` (GitHub-hosted)
4. **Different authorities = different env_hash** - This is by design
5. **Phase-17 local readiness is fail-closed diagnostic only** - It requires
   median and local stability PASS, records `closure_eligible_component=false`,
   and still cannot replace remote locked-authority PASS
6. **Variance diagnosis is observation only** - A diagnostic PASS preserves
   the source stability verdict and cannot renew baseline, relax thresholds,
   establish root cause, or grant closure authority
7. **Bounded non-reproduction is not acceptance** - PR-4B runs the existing
   measurement contract in controlled image-reuse/rebuild-per-run groups;
   a PASS or a non-reproduced outlier does not erase a prior readiness FAIL
   or replace remote locked-baseline authority

## Workflow

### Initial Setup
```bash
# Create local baseline
./scripts/ci/local_perf_baseline_init.sh

# Verify it was created
ls -lh scripts/ci/perf-baseline.local.lock.json
```

### Development Cycle
```bash
# Make changes to kernel
vim kernel/kernel.c

# Test against local baseline
make ci-gate-performance-local

# Emit scoped Phase-17 local readiness evidence; stability is enforced inside
make ci-gate-phase17-performance-readiness-local

# If performance regressed, investigate
cat evidence/run-*/gates/performance/violations.txt

# If readiness failed on stability, fingerprint and classify existing evidence
make ci-gate-phase17-performance-variance-diagnostic \
  RUN_ID=local-phase17-variance-diagnostic-20260524 \
  EVIDENCE_ROOT=evidence \
  PHASE17_VARIANCE_SOURCE_RUN_ID=local-phase17-performance-readiness-20260524-r2 \
  PHASE17_VARIANCE_REFERENCE_RUN_ID=local-phase17-performance-readiness-20260524

# Attempt bounded reproduction without changing runtime, baseline or thresholds
make ci-gate-phase17-performance-variance-isolation \
  RUN_ID=local-phase17-variance-isolation-20260524-r3 \
  EVIDENCE_ROOT=evidence \
  PHASE17_VARIANCE_ISOLATION_RUNS=3 \
  PHASE17_VARIANCE_ISOLATION_WARMUP=1
```

### Refreshing Local Baseline
```bash
# Delete old baseline
rm scripts/ci/perf-baseline.local.lock.json

# Create new one
./scripts/ci/local_perf_baseline_init.sh
```

## CI Baseline (Separate)

CI baseline is managed through GitHub Actions:

```bash
# Trigger via GitHub Actions UI
# Workflow: ci-freeze
# Input: init_perf_baseline = true
```

This creates: `scripts/ci/perf-baseline.lock.json` (committed to repo)

Phase-17 PR-4 acceptance uses the committed CI baseline without changing its
authority or thresholds:

```bash
# Run by .github/workflows/ci-gate-phase17-performance-acceptance.yml
make ci-gate-phase17-performance-acceptance
```

That remote target covers the existing timer/preemption hot-path measurement
surface. It does not establish validation-only worker-completion or
timeout-race payload latency acceptance.

## Troubleshooting

### Metrics are null
This is normal if:
- EFI build failed
- QEMU test failed
- Preempt markers missing

Fix the underlying issue, then re-init baseline.

### env_hash mismatch
This means your environment changed:
- Toolchain version updated
- Kernel version changed
- QEMU version changed

Local gate now distinguishes:
- pure env drift -> baseline may auto-refresh
- env drift + metric regression -> fail-closed

## Architecture

```
Local Dev:
  Authority: local-dev-Darwin-arm64
  Baseline:  scripts/ci/perf-baseline.local.lock.json (gitignored)
  Policy:    PERF_ENV_MISMATCH_POLICY=waiver
  Entry:     make ci-gate-performance-local / make ci-gate-phase17-performance-readiness-local / make ci-freeze-local

CI/Provisional:
  Authority: github-hosted-ubuntu-latest-x64
  Baseline:  scripts/ci/perf-baseline.lock.json (committed)
  Policy:    PERF_ENV_MISMATCH_POLICY=waiver

CI/Constitutional:
  Authority: baremetal-<pinned-authority>
  Baseline:  scripts/ci/perf-baseline.lock.json (committed)
  Policy:    PERF_ENV_MISMATCH_POLICY=fail
```

Policy contract note:
- Accepted policy values are `fail` or `waiver`.
- `warn` is not a valid gate policy value.

This separation allows:
- Local development without CI dependency
- Strict CI enforcement
- No authority conflicts

Implementation note:
- Local comparison explicitly sets `PERF_ALLOW_UNTRACKED_BASELINE=1`.
- Local gate stores a separate threshold contract (`boot=20%`, `context=15%`, `syscall=15%`).
- CI/global enforcement still keys off the legacy proxy metrics (`boot_time_ms`, `context_switch_latency_ms_proxy`, `syscall_latency_ms_proxy`).
- Split latency diagnostics (`entry_latency_ticks`, `syscall_latency_ticks_pure`, `syscall_gate_return_latency_ticks`) are informational until the Linux authority baseline is renewed.
- Local gate stores a sampling contract (`sample_size=5`, `warmup_runs=1`, `aggregation=median`, `outlier_policy=none` by default).
- Local gate records jitter visibility in run evidence: `min`, `max`, `range`, `range_percent_of_median`, `median_abs_deviation`, and MAD-based outlier candidates.
- MAD-based outlier detection is currently diagnostic-only; samples are reported, not discarded.
- Local stability gate reads sampled performance evidence and applies a separate stability contract from `scripts/ci/perf-stability.contract.json`.
- Stability is evaluated independently from median performance:
  range and MAD breaches fail,
  outlier candidate count is currently warn-only in the default local profile.
- This exception is only for the gitignored local baseline path, not for committed CI lock files.
- Auto-refresh is limited to contract drift only:
  missing baseline, schema drift, measurement-contract drift, threshold drift, sampling drift, or legacy baseline metric holes.
- Pure env drift may auto-refresh only when current medians stay within baseline thresholds.
- Metric regression is not auto-refreshed; local gate stays fail-closed when current medians exceed the local baseline contract.
- Stability contract tuning lives outside the gate script so range/MAD/outlier policy can be adjusted without changing gate logic.
- `ci-gate-phase17-performance-readiness-local` combines local median
  performance evidence with `performance-stability` and fails closed on
  instability; even PASS remains diagnostic and cannot replace remote
  `ci-gate-phase17-performance-acceptance`.
- `ci-gate-phase17-performance-variance-diagnostic` reads existing readiness
  reports only. Its PASS verifies classification integrity while retaining an
  upstream stability FAIL as `blocked_by_source_stability_failure`; its
  fingerprint is not performance acceptance or root-cause proof.
- `ci-gate-phase17-performance-variance-isolation` executes bounded local
  measurements with PR-4 runtime-contract and terminal-counter parity
  enforced. The 2026-05-24 `r3` run did not reproduce the prior `sample-6`
  elapsed outlier (`image-reuse` peak `%1.300080`, `rebuild-per-run` peak
  `%0.743889`, diagnostic threshold `%3`); this leaves the existing
  readiness FAIL and remote locked-baseline acceptance requirement intact.

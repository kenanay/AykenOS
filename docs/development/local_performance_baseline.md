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

# Or run the local freeze suite with local perf authority active
make ci-freeze-local
```

## Important Rules

1. **Never commit local baseline** - It's in `.gitignore`
2. **Local baseline is for development only** - Not for CI/production
3. **CI uses separate baseline** - `scripts/ci/perf-baseline.lock.json` (GitHub-hosted)
4. **Different authorities = different env_hash** - This is by design

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

# If performance regressed, investigate
cat evidence/run-*/gates/performance/violations.txt
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
  Entry:     make ci-gate-performance-local / make ci-freeze-local

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
- Local gate stores a sampling contract (`sample_size=5`, `warmup_runs=1`, `aggregation=median`, `outlier_policy=none` by default).
- This exception is only for the gitignored local baseline path, not for committed CI lock files.
- Auto-refresh is limited to contract drift only:
  missing baseline, schema drift, measurement-contract drift, threshold drift, sampling drift, or legacy baseline metric holes.
- Pure env drift may auto-refresh only when current medians stay within baseline thresholds.
- Metric regression is not auto-refreshed; local gate stays fail-closed when current medians exceed the local baseline contract.

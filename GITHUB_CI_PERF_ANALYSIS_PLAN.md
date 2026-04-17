# GitHub CI Performance Analysis Plan

## Critical Understanding

**Local macOS tests are NOT authoritative** - they show different environment (Darwin/ARM64 vs Linux/x86_64).

**GitHub CI shows REAL regression:**
- Boot: 10684 → 12197ms (+14%)
- Context switch: 175.08 → 201.93ms (+15%)
- Syscall: 175.08 → 201.93ms (+15%)
- System is functional (marker chain healthy)
- Baseline from 2026-04-09 (commit 050332220d9a)

## Why Local Tests Were Misleading

| Metric | Baseline (GitHub) | Current (GitHub) | Local macOS |
|--------|-------------------|------------------|-------------|
| Environment | Linux x86_64 | Linux x86_64 | Darwin ARM64 |
| Clang | Ubuntu 18.1.3 | Ubuntu 18.1.3 | Apple 16.0.0 |
| QEMU | 8.2.2 | 8.2.2 | 10.2.0 |
| Boot (ms) | 10684 | 12197 | 13296+ |

Local tests showed "features OFF = slower" but that's environment-specific, not authoritative.

## Correct Next Steps

### Option 1: GitHub CI Experiment Matrix (Authoritative)

Create GitHub Actions workflow to run 4 configurations:

**Workflow: `.github/workflows/perf-analysis.yml`**

```yaml
name: Performance Analysis Matrix

on:
  workflow_dispatch:
    inputs:
      config:
        description: 'Configuration to test'
        required: true
        type: choice
        options:
          - baseline-all-features
          - observability-reduced
          - bcib-minimal
          - validation-minimal

jobs:
  perf-analysis:
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4
      
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y clang lld nasm qemu-system-x86-64 python3 jq
      
      - name: Build with configuration
        run: |
          case "${{ inputs.config }}" in
            baseline-all-features)
              make KERNEL_PROFILE=validation \
                   USER_MINIMAL_MODE=syscall-v2-runtime \
                   AYKEN_PERF_DIAG=1 \
                   efi-img
              ;;
            observability-reduced)
              make KERNEL_PROFILE=validation \
                   USER_MINIMAL_MODE=syscall-v2-runtime \
                   AYKEN_PERF_DIAG=1 \
                   AYKEN_DEBUG_SCHED=0 \
                   AYKEN_DEBUG_IRQ=0 \
                   AYKEN_RING3_FETCH_PROBE=0 \
                   AYKEN_RING3_POST_CR3_TEXT_PROBE=0 \
                   efi-img
              ;;
            bcib-minimal)
              make KERNEL_PROFILE=validation \
                   USER_MINIMAL_MODE=syscall-v2-runtime \
                   AYKEN_PERF_DIAG=1 \
                   AYKEN_SCHED_BOOTSTRAP_POLICY=0 \
                   efi-img
              ;;
            validation-minimal)
              make KERNEL_PROFILE=validation \
                   USER_MINIMAL_MODE=syscall-v2-runtime \
                   AYKEN_PERF_DIAG=1 \
                   AYKEN_RING3_ENTRY_GUARD=0 \
                   AYKEN_MB_SELFTEST=0 \
                   efi-img
              ;;
          esac
      
      - name: Run performance measurement
        run: |
          mkdir -p evidence/perf-analysis-${{ inputs.config }}
          scripts/ci/gate_performance.sh \
            --evidence-dir evidence/perf-analysis-${{ inputs.config }}
      
      - name: Upload evidence
        uses: actions/upload-artifact@v4
        with:
          name: perf-analysis-${{ inputs.config }}
          path: evidence/perf-analysis-${{ inputs.config }}/
```

Then run 4 times manually with different configs.

### Option 2: Binary Search in GitHub CI (Faster Initial Direction)

Since we have 95 commits between baseline and current, binary search in GitHub CI:

```bash
# On GitHub CI runner
git bisect start
git bisect bad HEAD  # 9b3358e6 (current, slow)
git bisect good 050332220d9a  # baseline (fast)

# GitHub CI will test ~7 commits
# Each test: build + run performance gate
# Result: commit where regression started
```

### Option 3: Hybrid Approach (Recommended)

1. **Quick bisect first** (2-3 hours in GitHub CI)
   - Narrows 95 commits → ~10 commits
   - Identifies when regression started
   
2. **Then perf diagnostic** (1 hour in GitHub CI)
   - Only measure features in identified commit range
   - Feature-level breakdown
   
3. **Decision** (conscious choice)
   - Optimize expensive feature OR accept with justification
   
4. **Baseline update** (authorized workflow)
   - With full documentation

## Why Not Continue Local Testing?

Local macOS results are diagnostic only because:
- Different CPU architecture (ARM64 vs x86_64)
- Different compiler (Apple clang vs Ubuntu clang)
- Different QEMU version (10.2.0 vs 8.2.2)
- Different host OS (Darwin vs Linux)

Environment hash mismatch means results are not comparable to baseline.

## Immediate Action Items

### For GitHub CI Testing:

1. **Create workflow file** (Option 1) OR
2. **Trigger manual bisect** (Option 2) OR
3. **Do both** (Option 3 - recommended)

### For Local Development:

1. Keep perf diagnostic infrastructure (already done)
2. Don't make decisions based on local macOS results
3. Wait for GitHub CI authoritative measurements

## Expected Timeline

**Option 1 (Matrix only):**
- 4 workflow runs × 15 min = 1 hour
- Analysis: 15 min
- Total: ~1.5 hours

**Option 2 (Bisect only):**
- ~7 bisect steps × 15 min = ~2 hours
- Still need feature breakdown after
- Total: ~3 hours

**Option 3 (Hybrid):**
- Bisect: ~2 hours → narrows to 10 commits
- Matrix on narrow range: 30 min
- Analysis: 15 min
- Total: ~2.5 hours

## Critical Rules

- ✅ Only trust GitHub CI measurements (authoritative environment)
- ✅ Measure before deciding
- ✅ Document findings
- ✅ Update baseline via authorized workflow
- ❌ Don't use local macOS results for decisions
- ❌ Don't update baseline without understanding root cause

## Current Status

- Perf diagnostic infrastructure: ✅ Ready
- Local testing: ⚠️ Misleading (environment mismatch)
- GitHub CI testing: ⏳ Needed (authoritative)
- Baseline update: ❌ Blocked until measurement complete

## Next Step

Choose approach and execute in GitHub CI environment.

Recommended: **Option 3 (Hybrid)** - bisect first to narrow range, then feature matrix.

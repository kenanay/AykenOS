# Deterministic Baremetal CI Setup Guide

## Overview

This guide describes how to set up a deterministic baremetal CI runner for AykenOS performance baseline authority. This setup ensures reproducible performance measurements across CI runs.

## Why Baremetal?

GitHub-hosted runners provide:
- ✅ Convenience
- ✅ Zero maintenance
- ❌ Shared CPU (non-deterministic)
- ❌ Variable frequency
- ❌ Different image versions
- ❌ VM overhead

Baremetal runners provide:
- ✅ Fixed hardware
- ✅ Deterministic performance
- ✅ Stable baseline authority
- ❌ Requires dedicated machine
- ❌ Manual setup

## Hardware Requirements

### Minimum Specs
- x86_64 CPU (Intel recommended for fixed frequency support)
- 8 GB RAM minimum
- SSD storage
- Stable network connection

### Critical: Dedicated Machine
⚠️ This machine must be:
- **NOT** used for daily development
- **NOT** used for personal computing
- **ONLY** used as CI authority

Mixing workloads destroys determinism.

## BIOS Configuration

Enter BIOS and configure:

```
❌ Turbo Boost          → DISABLED
❌ C-States             → DISABLED
❌ SpeedStep/P-State    → DISABLED
✅ Fixed Frequency Mode → ENABLED (if available)
❌ Hyper-Threading      → DISABLED (recommended)
```

**Goal**: CPU frequency must be constant.

## OS Installation

### Recommended: Ubuntu 22.04 LTS

```bash
# After fresh install
sudo apt update && sudo apt upgrade -y
```

## CPU Governor Configuration

```bash
# Install tools
sudo apt install linux-tools-common linux-tools-generic -y

# Set performance governor
sudo cpupower frequency-set -g performance

# Verify
cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor
# Output: performance

# Make persistent
echo 'GOVERNOR="performance"' | sudo tee /etc/default/cpupower
sudo systemctl enable cpupower
```

## Background Services Reduction

Minimize noise:

```bash
sudo systemctl disable snapd
sudo systemctl disable cups
sudo systemctl disable bluetooth
sudo systemctl disable avahi-daemon
```

## Toolchain Installation

Install fixed versions:

```bash
sudo apt install -y \
  clang-15 \
  lld-15 \
  nasm \
  qemu-system-x86 \
  jq \
  git
```

### Lock Toolchain Versions

```bash
sudo mkdir -p /etc/aykenos

# Record versions
{
  echo "clang: $(clang --version | head -n1)"
  echo "lld: $(ld.lld --version | head -n1)"
  echo "nasm: $(nasm -v)"
  echo "qemu: $(qemu-system-x86_64 --version | head -n1)"
} | sudo tee /etc/aykenos/toolchain.lock
```

## CI Authority Digest

Create authority identifier:

```bash
sudo mkdir -p /etc/aykenos
echo "baremetal-ubuntu22-x86_64-perf01" | sudo tee /etc/aykenos/ci_image_digest
```

This file is read by the performance gate to verify environment consistency.

## GitHub Self-Hosted Runner Setup

### 1. Navigate to GitHub

```
Repository → Settings → Actions → Runners → New self-hosted runner
```

Select: **Linux**

### 2. Install Runner

```bash
# Create runner directory
mkdir -p ~/actions-runner && cd ~/actions-runner

# Download latest runner
curl -o actions-runner-linux-x64.tar.gz -L \
  https://github.com/actions/runner/releases/latest/download/actions-runner-linux-x64.tar.gz

# Extract
tar xzf actions-runner-linux-x64.tar.gz

# Configure (use token from GitHub UI)
./config.sh \
  --url https://github.com/kenanay/AykenOS \
  --token <YOUR_TOKEN> \
  --labels self-hosted,linux,x64,aykenos-perf01

# Test run
./run.sh
```

### 3. Install as Service (Optional)

For 24/7 operation:

```bash
sudo ./svc.sh install
sudo ./svc.sh start
sudo ./svc.sh status
```

For on-demand operation:
- Start manually before CI runs: `./run.sh`
- Stop after CI completes: Ctrl+C

## Determinism Validation

### Test 1: Boot Time Variance

Run 5 consecutive measurements:

```bash
for i in {1..5}; do
  make ci-gate-performance
done
```

Check variance in `evidence/run-*/gates/performance/metrics.json`:
- Boot time variance should be < 1%
- If > 5%, determinism is insufficient

### Test 2: Noise Isolation

Run stress test while CI is running:

```bash
# Terminal 1
make ci-gate-performance

# Terminal 2 (during CI run)
stress-ng --cpu 4 --timeout 30s
```

Performance should NOT be affected. If affected, isolation is weak.

### Test 3: Frequency Stability

Monitor CPU frequency during CI:

```bash
watch -n 1 'cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_cur_freq'
```

All cores should show identical, constant frequency.

## Workflow Configuration

Update `.github/workflows/ci-freeze.yml`:

```yaml
init_perf_baseline:
  runs-on: [self-hosted, linux, x64, aykenos-perf01]
  env:
    PERF_BASELINE_AUTHORITY: "baremetal-ubuntu22-x86_64-perf01"
    PERF_CI_IMAGE_DIGEST_SOURCE: "file"  # Read from /etc/aykenos/ci_image_digest
```

## Maintenance

### Toolchain Updates

⚠️ **NEVER** update toolchain without re-baselining:

```bash
# If toolchain must be updated:
1. Update packages
2. Record new versions in /etc/aykenos/toolchain.lock
3. Update ci_image_digest (increment version)
4. Re-run baseline initialization
5. Commit new baseline to repo
```

### Runner Updates

GitHub runner auto-updates. Monitor for:
- Runner version changes
- Unexpected performance shifts

### Hardware Changes

Any hardware change requires:
- New authority identifier
- Complete re-baseline
- New baseline commit

## Troubleshooting

### Runner Not Picking Up Jobs

```bash
# Check runner status
cd ~/actions-runner
./run.sh

# Check labels match workflow
cat .runner
```

### Performance Variance Too High

1. Check CPU governor: `cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor`
2. Check background processes: `top`
3. Check BIOS settings (Turbo Boost off?)
4. Check thermal throttling: `sensors`

### Baseline Mismatch

```bash
# Verify digest file
cat /etc/aykenos/ci_image_digest

# Verify toolchain versions
cat /etc/aykenos/toolchain.lock

# Check for toolchain drift
clang --version
```

## Security Considerations

See: [Self-Hosted Runner Security](./SELF_HOSTED_RUNNER_HARDENING.md)

Key points:
- Do NOT use for public repos (security risk)
- Isolate network if possible
- Regular security updates
- Monitor for unauthorized access

## Cost-Benefit Analysis

### When to Use Baremetal

✅ Use when:
- Performance baseline is critical
- Determinism is required
- Academic/industrial grade validation needed
- Release builds

❌ Don't use when:
- Rapid prototyping
- Feature development
- Quick sanity checks

### Hybrid Approach (Recommended)

- **Daily development**: GitHub-hosted runners
- **Performance gates**: Baremetal runner
- **Release validation**: Baremetal runner

## On-Demand vs 24/7

### On-Demand (Recommended for Solo Developers)

**Workflow**:
1. Open PR
2. Start baremetal runner: `cd ~/actions-runner && ./run.sh`
3. Trigger workflow manually
4. Wait for completion
5. Stop runner: Ctrl+C

**Pros**:
- No electricity cost
- No 24/7 maintenance
- Determinism preserved

**Cons**:
- Manual intervention required
- No automatic PR checks

### 24/7 (Enterprise Model)

**Workflow**:
1. Install runner as service
2. PR automatically triggers CI
3. Fully automated

**Pros**:
- Fully automated
- Real CI experience

**Cons**:
- Electricity cost
- Machine always on
- Network dependency

## Authority Principle

The baremetal machine is the **performance authority**:
- All baseline measurements come from this machine
- All performance comparisons reference this machine
- Drift from this machine = performance regression

This is constitutional governance for performance.

## Next Steps

After setup:
1. Run determinism validation tests
2. Initialize performance baseline
3. Commit baseline to repo
4. Document machine specs in repo
5. Set up monitoring/alerts

## References

- [GitHub Self-Hosted Runner Docs](https://docs.github.com/en/actions/hosting-your-own-runners)
- [CPU Frequency Scaling](https://www.kernel.org/doc/html/latest/admin-guide/pm/cpufreq.html)
- [Performance Baseline Gate Spec](../development/PERFORMANCE_BASELINE_GATE_SPEC.md)

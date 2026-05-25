# AykenOS Development Loop

Fast iteration cycle for kernel development with automated boot verification and regression detection.

## Quick Start

```bash
# Quick boot check (5-10s)
./scripts/dev_loop.sh smoke

# Runtime contract tests
./scripts/dev_loop.sh contract

# Full evidence validation
./scripts/dev_loop.sh full
```

## Validation Modes

### Smoke (5-10 seconds)
**Purpose**: Quick boot validation for rapid iteration

**What it checks**:
- Build succeeds
- Kernel boots within timeout
- Critical boot marker `[[AYKEN_BOOT_OK]]` present
- Warning markers `[K][EARLY_BOOT_OK]` and `[K][LATE_INIT_END]` present

**When to use**: After every small code change

```bash
./scripts/dev_loop.sh smoke
```

### Contract (~1-2 minutes)
**Purpose**: Validate capability and VCP contracts

**What it checks**:
- All smoke validations
- VCP runtime hook behavior
- VCP trust verification
- VCP fail-closed behavior

**When to use**: After feature completion, before committing

```bash
./scripts/dev_loop.sh contract
```

### Full (~2-3 minutes)
**Purpose**: Comprehensive validation including evidence layer

**What it checks**:
- All contract validations
- VCP evidence layer behavior
- Verification layer contracts

**When to use**: Before closing task, before PR

```bash
./scripts/dev_loop.sh full
```

## Automated Regression Detection

When something breaks, use the regression finder to automatically identify which commit caused the problem.

### Usage

```bash
# Find regression between last-known-good commit and HEAD
./scripts/find_regression.sh <last-good-commit>

# Example
./scripts/find_regression.sh a1b2c3d
```

### How it works

1. Uses `git bisect` to binary search through commits
2. Tests each commit using `./scripts/oracle.sh` (smoke test)
3. Automatically identifies first bad commit
4. Saves individual test logs to `out/logs/bisect/<commit>.log`

### Example Workflow

```bash
# Yesterday everything worked (commit a1b2c3d)
# Today something is broken (HEAD)

# Find the breaking commit
./scripts/find_regression.sh a1b2c3d

# Output:
# First bad commit: e4f5g6h
# Individual logs: out/logs/bisect/

# Fix the identified commit
# Verify fix
./scripts/dev_loop.sh full
```

## Isolation Property Test

Validates that dev loop does not affect kernel behavior (constitutional requirement).

```bash
./scripts/test_devloop_isolation.sh
```

**What it validates**:
- Baseline run (no dev loop) vs dev loop run produce identical results
- Marker sets are identical
- Both runs have critical markers
- Marker sequences are correct
- Kernel unaffected by broken dev loop

## Daily Development Workflow

### Mental Model

| Stage | Question | Command | Time |
|-------|----------|---------|------|
| **smoke** | "Does it boot?" | `./scripts/dev_loop.sh smoke` | 5-10s |
| **contract** | "Does it work correctly?" | `./scripts/dev_loop.sh contract` | 1-2 min |
| **full** | "Is it provable?" | `./scripts/dev_loop.sh full` | 2-3 min |
| **regression** | "Which commit broke it?" | `./scripts/find_regression.sh <good>` | 2-5 min |

### Recommended Workflow

```bash
# 1. During coding (every small change)
./scripts/dev_loop.sh smoke

# 2. Feature complete (within task)
./scripts/dev_loop.sh contract

# 3. Before closing task
./scripts/dev_loop.sh full

# 4. When regression detected
./scripts/find_regression.sh <last-good-commit>
```

### Fail Handling

**smoke FAIL**:
- Stop immediately
- Bug exists
- Start debugging

**contract FAIL**:
- Runtime regression
- Check VCP / fail-closed behavior

**full FAIL**:
- Evidence / isolation problem
- Spec-level bug

## Configuration

### Timeout

Default boot timeout: 20 seconds

Override:
```bash
QEMU_TIMEOUT_SECONDS=30 ./scripts/dev_loop.sh smoke
```

### CPU Count

Automatically detected using `sysctl -n hw.ncpu` (macOS) or `nproc` (Linux).

Fallback: 4 CPUs

### Logs

All logs saved to `out/logs/`:
- `boot_watch.log` - Smoke boot log
- `bisect/<commit>.log` - Regression finder logs

## Exit Status Codes

- `0` - PASS (validation succeeded)
- `1` - FAIL (build, boot, or test failure)
- `2` - INVALID_ARGS (invalid mode specified)

## Boot Markers

The dev loop validates boot success through deterministic markers:

- `[K][EARLY_BOOT_OK]` - Early boot complete (after GDT, IDT, paging)
- `[K][LATE_INIT_END]` - Late initialization complete (after scheduler, processes)
- `[[AYKEN_BOOT_OK]]` - Full boot complete (CRITICAL)

**Marker sequence**: EARLY → LATE → BOOT_OK

## Constitutional Compliance

The dev loop complies with all constitutional rules:

- ✅ **DETERMINISM.GLOBAL**: No global state mutations
- ✅ **KERNEL.RING0.POLICY**: No policy decisions in Ring0
- ✅ **SECURITY.BOUNDARY.VIOLATION**: Userspace tooling only
- ✅ **KERNEL.CAPABILITY.BYPASS**: No capability bypasses

**Isolation guarantee**: Dev loop does NOT affect kernel behavior (validated by `test_devloop_isolation.sh`)

## Advanced Usage

### Continuous Validation Loop

```bash
# Run smoke test continuously until failure
while true; do
    ./scripts/dev_loop.sh smoke || break
    sleep 1
done
```

### Git Pre-Commit Hook

```bash
# .git/hooks/pre-commit
#!/bin/bash
./scripts/dev_loop.sh smoke || exit 1
```

### Log Inspection

```bash
# View last 50 lines of boot log
tail -n 50 out/logs/boot_watch.log

# Search for specific marker
grep "\[\[AYKEN_BOOT_OK\]\]" out/logs/boot_watch.log
```

## Troubleshooting

### Build Fails

Check compiler errors in output. Common issues:
- Missing dependencies
- Syntax errors
- Configuration errors

### Boot Timeout

Check if kernel is hanging:
```bash
tail -50 out/logs/boot_watch.log
```

If markers present but timeout occurred, increase timeout:
```bash
QEMU_TIMEOUT_SECONDS=30 ./scripts/dev_loop.sh smoke
```

### Missing Markers

Check kernel code for marker emission:
```c
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
    fb_print("[[AYKEN_BOOT_OK]]\n");
#endif
```

Ensure building with validation profile:
```bash
make KERNEL_PROFILE=validation AYKEN_VALIDATION=1
```

### Regression Finder Fails

Ensure oracle script is executable:
```bash
chmod +x scripts/oracle.sh
```

Verify commits exist:
```bash
git rev-parse <commit>
```

## Performance Regression Detection

The dev loop integrates with AykenOS's existing performance gate infrastructure to detect performance regressions.

### Quick Performance Check

```bash
# Quick check (boot time proxy, ~1s)
./scripts/check_perf_regression.sh quick
```

**What it checks**:
- Boot time proxy (marker line count)
- Compares against baseline from `scripts/ci/perf-baseline.lock.json`
- Uses threshold from baseline policy (default: ±10% for boot)

**Note**: This is a rough proxy based on log line count. For accurate measurement, use full mode.

### Full Performance Check

```bash
# Full check (TSC-based, delegates to ci-gate-performance)
./scripts/check_perf_regression.sh full
```

**What it checks**:
- Boot time (TSC-based, accurate)
- Syscall latency proxy
- Context switch latency proxy
- All metrics from `perf-baseline.lock.json`

**Delegates to**: `make ci-gate-performance`

### Integration with Dev Loop

Performance checks are **optional** in dev loop (not run by default):

```bash
# Run dev loop with performance check
./scripts/dev_loop.sh full && ./scripts/check_perf_regression.sh quick
```

**Rationale**: Performance checks add overhead and may be noisy on local machines. Use when needed.

### Baseline Management

The performance baseline is managed by the existing performance gate:

```bash
# Initialize or update baseline (CI authority required)
make ci-gate-performance
```

**Baseline location**: `scripts/ci/perf-baseline.lock.json`

**Baseline authority**: `github-hosted-ubuntu-24.04-x64` (CI environment)

**Local baselines**: Not recommended (environment differences cause false positives)

### CI Integration

Performance checks run automatically in CI:

```yaml
# In .github/workflows/devloop-ci.yml
- name: Performance check
  run: |
    ./scripts/check_perf_regression.sh full
```

**CI behavior**:
- Uses full mode (TSC-based, accurate)
- Compares against locked baseline
- Fails PR if regression detected
- Uploads performance logs as artifacts

### Thresholds

Default thresholds from baseline policy:

| Metric | Threshold |
|--------|-----------|
| Boot time | ±10% |
| Syscall latency | ±5% |
| Context switch | ±5% |

**Configurable**: Edit `scripts/ci/perf-baseline.lock.json` (requires CI authority)

### Troubleshooting

**Baseline missing**:
```bash
⚠️  WARNING: Performance baseline not found
   Run 'make ci-gate-performance' to initialize baseline
   Skipping performance check
```

**Solution**: Run `make ci-gate-performance` in CI environment to create baseline.

**False positives on local machine**:
```bash
❌ FAIL: Boot time regression detected
   Ratio: 1.15 exceeds threshold: 1.10
```

**Solution**: Local environments differ from CI. Use CI for authoritative performance checks.

**Markers missing**:
```bash
⚠️  WARNING: Boot markers not found, cannot estimate boot time
   Skipping performance check
```

**Solution**: Ensure kernel built with `AYKEN_VALIDATION=1` and markers emitted.

## CI Integration

The dev loop is integrated into GitHub Actions CI pipeline for automatic validation on PRs.

### CI Workflow

```
PR opened → smoke → contract → full → isolation → performance
                ↓
              FAIL
                ↓
          auto-bisect (finds first bad commit)
                ↓
          PR comment with result
```

### CI Jobs

1. **smoke** - Quick boot validation (10 min timeout)
2. **contract** - Runtime contract tests (15 min timeout)
3. **full** - Comprehensive validation (20 min timeout)
4. **isolation** - Isolation property test (10 min timeout)
5. **performance** - Performance regression check (10 min timeout)
6. **auto-bisect** - Automatic regression finder (30 min timeout, only on failure)

### Auto-Bisect Behavior

**When triggered**: Only when PR validation fails

**What it does**:
1. Identifies good commit (merge base with target branch)
2. Identifies bad commit (PR HEAD)
3. Uses git bisect to find first failing commit
4. Tests each commit with `./scripts/oracle.sh` (smoke test)
5. Uploads bisect logs as artifacts
6. Comments on PR with results

**Assumptions**:
- Base branch (main/develop) is known-good
- Oracle is deterministic (same commit → same result)
- No flaky tests

### Viewing CI Results

**Logs**: Download artifacts from GitHub Actions run
- `smoke-logs` - Smoke test logs
- `contract-logs` - Contract test logs
- `full-logs` - Full validation logs
- `isolation-logs` - Isolation property test logs
- `bisect-logs` - Bisect logs (only on failure)

**Bisect results**: Check PR comments for auto-bisect summary

### Branch Protection

Recommended GitHub branch protection settings:
- ✅ Require status checks to pass before merging
- ✅ Require `smoke`, `contract`, `full`, `isolation` jobs to pass
- ✅ Require branches to be up to date before merging

## Files

- `scripts/dev_loop.sh` - Main dev loop orchestrator
- `scripts/oracle.sh` - Deterministic validation check (for bisect)
- `scripts/find_regression.sh` - Automated regression finder (local)
- `scripts/test_devloop_isolation.sh` - Isolation property test
- `scripts/test_vcp_*.sh` - Contract test scripts
- `out/logs/` - Validation logs
- `.github/workflows/devloop-ci.yml` - CI workflow

## Spec Location

Full specification: `.kiro/specs/dev-loop-boot-monitoring/`
- `requirements.md` - 21 requirements
- `design.md` - Comprehensive design
- `tasks.md` - Implementation tasks

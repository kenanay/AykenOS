# Phase 16 Performance Regression RCA - Quick Start Guide

## Task 1: Feature Toggle Infrastructure (COMPLETE)

The feature toggle infrastructure has been implemented and is ready for measurement.

## Running Measurements

### Option 1: Automated Script (Recommended)

Run the automated measurement script to test all configurations:

```bash
cd .kiro/specs/phase16-performance-regression-rca
./measure_phase16_overhead.sh
```

This will:
1. Build 6 different configurations (all enabled, individual features disabled, all disabled)
2. Run 3 iterations per configuration
3. Measure boot_time for each iteration
4. Calculate mean and standard deviation
5. Generate a summary report with overhead analysis

**Output**: `measurements/run_<timestamp>/SUMMARY.md`

### Option 2: Manual Measurement

Test a specific configuration manually:

```bash
# Clean build
make clean

# Build with specific feature toggles
make KERNEL_PROFILE=validation \
     AYKEN_PHASE16_BCIB_WORKER_ENABLE=0 \
     AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE=1 \
     AYKEN_PHASE16_PROBE_VALIDATION_ENABLE=1 \
     AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE=1 \
     efi-img

# Measure boot time
tools/validation/phase_4_4_qemu_boot_audit.sh \
    --timeout 30 \
    --marker "[K][BOOT_OK] Phase 4.4 minimal boot reached" \
    --out-dir ./boot-audit-output
```

## Feature Toggle Flags

| Flag | Default | Controls |
|------|---------|----------|
| `AYKEN_PHASE16_BCIB_WORKER_ENABLE` | 1 | BCIB worker process creation |
| `AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE` | 1 | Syscall boundary validation |
| `AYKEN_PHASE16_PROBE_VALIDATION_ENABLE` | 1 | Ring3 probe frame validation (4KB memcmp) |
| `AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE` | 1 | Diagnostic marker emission (debugcon I/O) |

## Expected Results

### Baseline Measurements

- **All enabled** (current state): ~12,190ms (+1,506ms regression)
- **All disabled** (baseline): ~10,684ms (pre-Phase-16)

### Overhead Identification

The measurements will identify which feature(s) contribute to the regression:

```
Feature Overhead = (All Enabled) - (Feature Disabled)
```

Example:
- All enabled: 12,190ms
- Probe disabled: 11,400ms
- **Probe overhead**: 790ms

### Success Criteria

1. ✅ All disabled configuration returns to baseline (~10,684ms ±10%)
2. ✅ Individual feature overheads sum to total regression (~1,506ms)
3. ✅ Standard deviation < 5% of mean (deterministic measurements)
4. ✅ Multiple runs produce consistent results (±100ms)

## Interpreting Results

### Summary Report

The automated script generates a markdown table:

```markdown
| Configuration | BCIB Worker | Boundary | Probe | Markers | Mean Boot Time (ms) | StdDev (ms) |
|--------------|-------------|----------|-------|---------|---------------------|-------------|
| config1_all_enabled | ✓ | ✓ | ✓ | ✓ | 12190 | 45 |
| config2_bcib_worker_disabled | ✗ | ✓ | ✓ | ✓ | 11800 | 38 |
| config3_boundary_disabled | ✓ | ✗ | ✓ | ✓ | 11900 | 42 |
| config4_probe_disabled | ✓ | ✓ | ✗ | ✓ | 11400 | 35 |
| config5_markers_disabled | ✓ | ✓ | ✓ | ✗ | 11790 | 40 |
| config6_all_disabled | ✗ | ✗ | ✗ | ✗ | 10684 | 30 |
```

### Overhead Analysis

Calculate overhead for each feature:

- **BCIB Worker**: 12,190 - 11,800 = **390ms**
- **Boundary Enforcement**: 12,190 - 11,900 = **290ms**
- **Probe Validation**: 12,190 - 11,400 = **790ms** ⚠️ HIGH
- **Diagnostic Markers**: 12,190 - 11,790 = **400ms**

**Total**: 390 + 290 + 790 + 400 = **1,870ms** (close to observed +1,506ms)

### Next Steps

Based on overhead analysis:

1. **High overhead features** (>500ms): Prioritize for optimization
   - Example: Probe validation (790ms) → Replace 4KB memcmp with hash-based validation

2. **Medium overhead features** (200-500ms): Optimize if needed
   - Example: Diagnostic markers (400ms) → Buffer marker emissions

3. **Low overhead features** (<200ms): Acceptable, no optimization needed

## Troubleshooting

### Build Failures

**Error**: `Invalid AYKEN_PHASE16_*='X'. Use 0 or 1`

**Solution**: Ensure all feature toggle flags are set to 0 or 1:
```bash
make AYKEN_PHASE16_BCIB_WORKER_ENABLE=1  # ✓ Valid
make AYKEN_PHASE16_BCIB_WORKER_ENABLE=2  # ✗ Invalid
```

### Boot Failures

**Error**: Boot audit times out or fails

**Possible causes**:
1. Disabling boundary enforcement may cause security violations
2. Disabling BCIB worker may affect validation profile expectations

**Solution**: Check boot audit logs in `measurements/run_*/config*/iter_*/boot_audit.log`

### Inconsistent Measurements

**Issue**: High standard deviation (>5% of mean)

**Possible causes**:
1. Non-deterministic environment (local machine, not CI)
2. Background processes interfering with measurements
3. Thermal throttling or power management

**Solution**: Run measurements in CI environment (GitHub Actions) for deterministic results

## CI Integration

### Running in GitHub Actions

Add to `.github/workflows/phase16-rca.yml`:

```yaml
name: Phase 16 Performance RCA

on:
  workflow_dispatch:
    inputs:
      iterations:
        description: 'Number of iterations per configuration'
        required: false
        default: '5'

jobs:
  measure:
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v3
      
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y clang lld nasm qemu-system-x86 bc
      
      - name: Run Phase 16 RCA measurements
        run: |
          cd .kiro/specs/phase16-performance-regression-rca
          ./measure_phase16_overhead.sh ${{ github.event.inputs.iterations }}
      
      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: phase16-rca-measurements
          path: .kiro/specs/phase16-performance-regression-rca/measurements/
```

### Viewing Results

1. Go to Actions tab in GitHub
2. Select "Phase 16 Performance RCA" workflow
3. Click "Run workflow"
4. After completion, download "phase16-rca-measurements" artifact
5. Open `SUMMARY.md` to view results

## Documentation

- **Feature Toggles**: `FEATURE_TOGGLES.md` - Detailed documentation of all toggles
- **Bugfix Requirements**: `bugfix.md` - Bug condition and expected behavior
- **Design Document**: `design.md` - RCA methodology and fix strategy
- **Task List**: `tasks.md` - Implementation plan and progress

## Support

If you encounter issues:

1. Check build logs: `measurements/run_*/config*/build.log`
2. Check boot logs: `measurements/run_*/config*/iter_*/boot_audit.log`
3. Verify feature toggle flags are correctly set in compile command
4. Ensure clean build between configurations (`make clean`)

## Important Notes

⚠️ **DO NOT FIX CODE YET**: This is Task 1 (exploratory measurement). The goal is to identify the regression source, not fix it.

⚠️ **PRESERVE FUNCTIONALITY**: Feature toggles are for measurement only. All Phase 16 features must remain in production builds.

⚠️ **CI AUTHORITY**: Measurements must be performed in CI environment (GitHub Actions) for authoritative results.

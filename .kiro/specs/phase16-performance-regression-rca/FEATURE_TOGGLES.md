# Phase 16 Feature Toggle Infrastructure

## Overview

This document describes the feature toggle infrastructure created for Phase 16 performance regression root cause analysis (RCA). The infrastructure allows selective enabling/disabling of Phase 16 features to isolate the source of the +1,506ms boot time regression.

## Feature Toggle Flags

Four compile-time flags have been added to the build system:

### 1. AYKEN_PHASE16_BCIB_WORKER_ENABLE

**Default**: 1 (enabled in validation profile)

**Controls**: BCIB worker process creation during kernel boot

**Code Locations**:
- `kernel/kernel.c` lines 756-781: `bcib_worker_create()` and `user_worker_create()` calls

**When Disabled**:
- BCIB worker and USER worker processes are not created during boot
- Marker emitted: `[K][LATE]8.1 BCIB_WORKER_CREATE_DISABLED`
- Expected impact: Eliminates worker creation overhead (~allocation, initialization, context setup)

### 2. AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE

**Default**: 1 (enabled)

**Controls**: Syscall boundary enforcement validation

**Code Locations**:
- `kernel/sys/syscall_v2_hardened.c` lines 148-162: `boundary_validate_syscall()` and `boundary_detect_bridge_bypass()` calls

**When Disabled**:
- Boundary enforcement checks are skipped for all syscalls
- No fail-closed termination for boundary violations
- Expected impact: Eliminates per-syscall validation overhead

### 3. AYKEN_PHASE16_PROBE_VALIDATION_ENABLE

**Default**: 1 (enabled)

**Controls**: Ring3 probe frame validation (byte-by-byte comparison)

**Code Locations**:
- `kernel/sched/sched.c` line 4254: Cached probe frame validation
- `kernel/sched/sched.c` line 5137: Fresh probe frame validation

**When Disabled**:
- Probe frame byte-by-byte comparison (4KB memcmp) is skipped
- Frame matching uses only physical address comparison
- Expected impact: Eliminates 4KB memcmp overhead per probe validation

### 4. AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE

**Default**: 1 (enabled)

**Controls**: Diagnostic marker emission via debugcon (port 0xE9)

**Code Locations**:
- `kernel/kernel.c` line 124: `dual_channel_write()` function body
- All `dual_channel_write()` calls throughout kernel boot sequence

**When Disabled**:
- All diagnostic marker emissions are compiled out
- No debugcon I/O operations during boot
- Expected impact: Eliminates debugcon write overhead (multiple I/O operations per marker)

## Build System Integration

### Makefile Changes

**Flag Definitions** (Makefile lines 97-101):
```makefile
AYKEN_PHASE16_BCIB_WORKER_ENABLE ?= 1
AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE ?= 1
AYKEN_PHASE16_PROBE_VALIDATION_ENABLE ?= 1
AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE ?= 1
```

**Flag Validation** (Makefile lines 303-319):
```makefile
ifneq ($(filter $(AYKEN_PHASE16_BCIB_WORKER_ENABLE),0 1),$(AYKEN_PHASE16_BCIB_WORKER_ENABLE))
$(error Invalid AYKEN_PHASE16_BCIB_WORKER_ENABLE='$(AYKEN_PHASE16_BCIB_WORKER_ENABLE)'. Use 0 or 1)
endif
# ... (similar for other flags)
```

**CFLAGS Propagation** (Makefile lines 710-714):
```makefile
KERNEL_CFLAGS += -DAYKEN_PHASE16_BCIB_WORKER_ENABLE=$(AYKEN_PHASE16_BCIB_WORKER_ENABLE)
KERNEL_CFLAGS += -DAYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE=$(AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE)
KERNEL_CFLAGS += -DAYKEN_PHASE16_PROBE_VALIDATION_ENABLE=$(AYKEN_PHASE16_PROBE_VALIDATION_ENABLE)
KERNEL_CFLAGS += -DAYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE=$(AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE)
```

### Usage

**Build with specific feature toggles**:
```bash
make clean
make KERNEL_PROFILE=validation \
     AYKEN_PHASE16_BCIB_WORKER_ENABLE=0 \
     AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE=1 \
     AYKEN_PHASE16_PROBE_VALIDATION_ENABLE=1 \
     AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE=1 \
     efi-img
```

**Disable all Phase 16 features**:
```bash
make clean
make KERNEL_PROFILE=validation \
     AYKEN_PHASE16_BCIB_WORKER_ENABLE=0 \
     AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE=0 \
     AYKEN_PHASE16_PROBE_VALIDATION_ENABLE=0 \
     AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE=0 \
     efi-img
```

## Measurement Script

### measure_phase16_overhead.sh

**Location**: `.kiro/specs/phase16-performance-regression-rca/measure_phase16_overhead.sh`

**Purpose**: Automated measurement of boot_time across all feature toggle configurations

**Usage**:
```bash
cd .kiro/specs/phase16-performance-regression-rca
./measure_phase16_overhead.sh [iterations] [output_dir]
```

**Default**: 3 iterations per configuration, output to `./measurements/run_<timestamp>/`

### Measurement Configurations

The script measures 6 configurations:

1. **config1_all_enabled**: All Phase 16 features enabled (baseline regression)
   - Expected: ~12,190ms (current state with regression)

2. **config2_bcib_worker_disabled**: Only BCIB worker disabled
   - Tests: BCIB worker creation overhead

3. **config3_boundary_disabled**: Only boundary enforcement disabled
   - Tests: Syscall validation overhead

4. **config4_probe_disabled**: Only probe validation disabled
   - Tests: Frame matching (4KB memcmp) overhead

5. **config5_markers_disabled**: Only diagnostic markers disabled
   - Tests: Debugcon I/O overhead

6. **config6_all_disabled**: All Phase 16 features disabled
   - Expected: ~10,684ms (return to baseline)

### Output Format

**Directory Structure**:
```
measurements/run_<timestamp>/
├── SUMMARY.md                    # Markdown summary with table
├── config1_all_enabled/
│   ├── build.log                 # Build output
│   ├── boot_times.txt            # Raw boot times (one per line)
│   ├── summary.txt               # mean=X, stddev=Y, iterations=N
│   └── iter_1/
│       ├── boot_audit.log        # Boot audit output
│       └── ...                   # Boot audit artifacts
├── config2_bcib_worker_disabled/
│   └── ...
└── ...
```

**Summary Table Example**:
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

### Overhead Calculation

For each feature, calculate overhead as:
```
Overhead = (config1_all_enabled mean) - (feature_disabled mean)
```

Example:
- config1_all_enabled: 12,190ms
- config4_probe_disabled: 11,400ms
- **Probe validation overhead**: 12,190 - 11,400 = **790ms**

## CI Integration

### Running in CI

The measurement script can be integrated into CI workflows:

```yaml
- name: Phase 16 Performance RCA
  run: |
    cd .kiro/specs/phase16-performance-regression-rca
    ./measure_phase16_overhead.sh 5 ${{ github.workspace }}/measurements
    
- name: Upload Measurements
  uses: actions/upload-artifact@v3
  with:
    name: phase16-rca-measurements
    path: measurements/
```

### CI Environment Requirements

- **Deterministic environment**: GitHub Actions Linux x86_64 (ubuntu-24.04)
- **Pinned toolchain**: clang 14.0.0, ld.lld 14.0.0, QEMU 6.2.0
- **Consistent hardware**: Same runner type for all measurements
- **Multiple iterations**: Minimum 3 iterations per configuration for statistical validity

## Expected Outcomes

### Hypothesis Validation

The measurements will validate or refute the hypothesized overhead sources:

1. **BCIB Worker Creation**: ~200ms overhead (allocation, initialization, context setup)
2. **Boundary Enforcement**: ~300ms overhead (per-syscall validation during boot)
3. **Probe Validation**: ~800ms overhead (4KB memcmp per probe check)
4. **Diagnostic Markers**: ~400ms overhead (debugcon I/O operations)

**Total hypothesized**: ~1,700ms (close to observed +1,506ms regression)

### Success Criteria

1. **Regression Isolation**: config6_all_disabled returns to baseline (~10,684ms ±10%)
2. **Overhead Quantification**: Sum of individual feature overheads ≈ total regression
3. **Determinism**: StdDev < 5% of mean for each configuration
4. **Reproducibility**: Multiple runs produce consistent results (±100ms)

## Next Steps (Phase 2)

After measurements identify the regression source(s):

1. **Targeted Optimization**: Apply fixes to high-overhead features
   - Example: Replace 4KB memcmp with hash-based validation
   - Example: Cache boundary validation results
   - Example: Buffer diagnostic marker emissions

2. **Fix Validation**: Re-run measurements with fixes applied
   - Verify boot_time ≤ 11,752ms (baseline + 10%)
   - Confirm Phase 16 functionality unchanged

3. **CI Enforcement**: Update baseline and enable enforcement
   - Update `scripts/ci/perf-baseline.lock.json`
   - Set `enforcement_enabled: true`
   - Prevent future regressions

## Preservation Guarantees

**CRITICAL**: Feature toggles are for measurement ONLY. All Phase 16 functionality must be preserved in production:

- BCIB boundary enforcement: fail-closed semantics
- Runtime_Bridge syscall restrictions: no bypass
- Probe validation: Ring3 contract enforcement
- Diagnostic markers: observability for debugging

The toggles enable **optimization** (e.g., caching, batching), NOT **removal** of features.

## References

- **Bugfix Requirements**: `.kiro/specs/phase16-performance-regression-rca/bugfix.md`
- **Design Document**: `.kiro/specs/phase16-performance-regression-rca/design.md`
- **Task List**: `.kiro/specs/phase16-performance-regression-rca/tasks.md`
- **CI Performance Gate**: `scripts/ci/gate_performance.sh`
- **Baseline Lock**: `scripts/ci/perf-baseline.lock.json`

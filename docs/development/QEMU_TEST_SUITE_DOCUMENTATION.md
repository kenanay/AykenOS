# AykenOS QEMU Integration Test Suite Documentation
This document is subordinate to PHASE 0 – FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

**Author:** Kenan AY  
**Project:** AykenOS - Advanced AI-Integrated Operating System  
**Phase:** 1 Critical Fixes and Validation  
**Last Updated:** February 11, 2026

## Overview

This document describes the comprehensive QEMU-based testing framework developed for AykenOS Phase 1 validation. The test suite implements automated validation of critical kernel functionality through QEMU emulation, log analysis, and systematic verification of core system components.

## Recent Updates (February 11, 2026)

### Phase 4.5 Preempt Runner Stabilization ✅

Deterministic preempt validation now has two execution modes:

```bash
# Standard validation (marker + behavioral fallback)
make run-preempt

# Strict marker contract validation
make run-preempt-strict
```

What changed:
- `run-preempt` and `run-preempt-strict` now use a single runner: `run_preempt_test.sh`.
- Runner now merges `debugcon + serial` into one analysis stream before assertions.
- Strict mode requires canonical scheduler markers (`MARK:*`) and disables AB fallback.
- Strict target enforces fresh media (`FORCE_EFI_REBUILD=1`) to eliminate stale `EFI.img` risk.
- Build profile integration:
  - `KERNEL_PROFILE=validation`: `-O0 -g3`, debug markers enabled.
  - `KERNEL_PROFILE=release`: `-O2 -g1`, debug markers disabled.

Canonical strict signatures (merged log):
- `MARK:PID=2`
- `MARK:PID=3`
- `MARK:SW=K>U|U>K|U>U`
- `MARK:IRET`

Troubleshooting:
1. If strict mode fails with marker counts at 0 but AB looks healthy, verify `KERNEL_PROFILE=validation` and `AYKEN_DEBUG_SCHED=1` propagation.
2. If output is empty: keep `-global isa-debugcon.iobase=0xe9` and serial file capture enabled.
3. If shell noise dominates logs, verify startup path and force image rebuild (`FORCE_EFI_REBUILD=1`).
4. If early crash/reset occurs near first scheduler jump, validate stack ABI alignment in `kernel_first_entry` (`sub rsp, 8` before C call).

### macOS Build System Integration ✅

**New Script:** `build_efi.sh` - macOS-compatible EFI.img creation

The build system now supports macOS development with native tooling:
- Uses `hdiutil` instead of Linux `mkfs.vfat`
- Prevents AppleDouble (._*) files with `COPYFILE_DISABLE=1`
- Creates QEMU-compatible raw disk images (UDRW format)
- Includes automatic startup.nsh for UEFI boot
- Provides hash verification for build confirmation

**Usage:**
```bash
# Build kernel
make clean && make

# Create EFI.img
./build_efi.sh

# Verify kernel hash
hdiutil attach EFI.img
shasum -a 256 /Volumes/EFI/kernel.elf
hdiutil detach /Volumes/EFI
```

### Critical QEMU Configuration Fix ✅

**Problem:** Debug output was not being captured, leading to false "boot failure" diagnoses.

**Solution:** Added `-global isa-debugcon.iobase=0xe9` flag to QEMU command.

**Correct QEMU Command (macOS):**
```bash
rm -f test_ring3_debugcon.log test_ring3_serial.log test_ring3_qemu.err

timeout 10 qemu-system-x86_64 \
  -machine q35 \
  -drive if=pflash,format=raw,readonly=on,file=/opt/homebrew/share/qemu/edk2-x86_64-code.fd \
  -drive if=pflash,format=raw,file=ovmf_vars.fd \
  -drive if=ide,format=raw,file=EFI.img \
  -m 256M \
  -serial file:test_ring3_serial.log \
  -debugcon file:test_ring3_debugcon.log \
  -global isa-debugcon.iobase=0xe9 \
  -nographic \
  -no-reboot \
  -d cpu_reset 2>test_ring3_qemu.err || true

tail -40 test_ring3_debugcon.log | tr -d '\000'
```

**Key Changes:**
- ✅ Added `-machine q35` for better hardware emulation
- ✅ Changed to `-drive if=ide` for better UEFI compatibility
- ✅ **Added `-global isa-debugcon.iobase=0xe9`** (CRITICAL)
- ✅ Clean log files before each run
- ✅ Added timeout to prevent hanging
- ✅ Capture stderr to separate file

### Ring3 Transition Validation ✅

**Status:** Ring3 user process execution + timer preempt validation operational

**Verified Boot Sequence:**
```
Z0                          # New kernel marker
[K][EARLY_BOOT_OK]         # kmain entry
[K][LATE_INIT_BEGIN]       # Late init
[K][ABOUT_TO_SCHED]        # Scheduler starting
S12[Q]1                    # Scheduler initialized
FT@tk                      # switch_to_first
J                          # kernel_first_entry
I                          # init_process_main
QPID:2                     # Ring3 test process created
[SEL]PID=2                 # Scheduler selects Ring3 process
[SW]K>U                    # Context switch: Kernel → User
ABOUT_TO_IRETQ             # IRET frame built
[U][RING3_OK]              # ✅ RING3 TRANSITION SUCCESSFUL!
```

**Preempt strict sequence (validation profile):**
```
MARK:PID=2
MARK:SW=U>U
MARK:IRET
MARK:PID=3
MARK:SW=U>U
MARK:IRET
```

## Test Suite Architecture

### Core Components

1. **Master Test Runner** (`run_qemu_tests.sh` / `run_qemu_tests.ps1`)
   - Orchestrates all test execution
   - Provides unified reporting
   - Manages test prerequisites and environment

2. **Integrated Test Suite** (`qemu_integration_tests.sh` / `qemu_integration_tests.ps1`)
   - Comprehensive multi-component testing
   - Parallel validation of all critical systems
   - Unified test execution with detailed reporting

3. **Specialized Test Scripts**
   - `ring3_validation_test.sh` - Ring3 user process execution validation
   - `devfs_validation_test.sh` - DevFS device I/O operations validation
   - `syscall_roundtrip_test.sh` - Syscall interface roundtrip testing

### Test Framework Features

- **Automated QEMU Management**: Process lifecycle, timeout handling, cleanup
- **Pattern-Based Validation**: Regex pattern matching for success/failure detection
- **Comprehensive Logging**: Detailed execution logs with timestamp analysis
- **Cross-Platform Support**: Bash (Linux/WSL) and PowerShell (Windows) implementations
- **Configurable Execution**: Timeout, verbosity, log retention, interactive modes
- **Detailed Reporting**: Markdown reports with requirement traceability

## Test Categories

### 1. Boot Validation Tests

**Purpose:** Verify kernel boot sequence and initialization phases

**Test Patterns:**
- `AykenOS.*INIT` - AykenOS initialization messages
- `Kernel.*init.*done` - Kernel subsystem completion
- `EARLY INIT.*done` - Early initialization phase
- `LATE INIT.*done` - Late initialization phase
- `Scheduler.*ready` - Scheduler readiness

**Validation Criteria:**
- All initialization phases complete successfully
- No critical boot errors detected
- Boot sequence completes within timeout

**Requirements Validated:** 4.1

### 2. Ring3 User Process Execution Tests

**Purpose:** Validate Ring3 context switching and user process execution

**Test Patterns:**
- `GDT.*init` / `TSS.*init` - Privilege infrastructure
- `Ring3.*selector.*0x23` / `Ring3.*selector.*0x1b` - Correct GDT selectors
- `user.*process.*created` - User process creation
- `ai-service.*Ring3` - Ring3 process scheduling
- `context.*switch` - Context switching operations

**Validation Criteria:**
- GDT/IDT/TSS initialization successful (≥2 patterns)
- User process creation successful (≥1 pattern)
- Syscall interface setup successful (≥1 pattern)
- No critical Ring3 errors

**Requirements Validated:** 4.2

### 3. DevFS Device I/O Operations Tests

**Purpose:** Confirm DevFS device registration and VFS integration

**Test Patterns:**
- `devfs.*Initializing device filesystem` - DevFS initialization
- `devfs.*Registered.*null` - Standard device registration
- `devfs.*Registered.*zero` - Standard device registration
- `devfs.*Registered.*console` - Standard device registration
- `devfs.*Registered.*kbd` - Extended device registration
- `devfs.*Registered.*ttyS0` - Extended device registration
- `devfs.*Registered.*sda` - Extended device registration

**Device Coverage:**
- **Standard Devices:** `/dev/null`, `/dev/zero`, `/dev/console`
- **Extended Devices:** `/dev/kbd`, `/dev/ttyS0`, `/dev/sda`
- **Device Metadata:** Type classification, capability flags, descriptions

**Validation Criteria:**
- DevFS initialization successful (≥1 pattern)
- All standard devices registered (3/3)
- All extended devices registered (3/3)
- No device registration errors

**Requirements Validated:** 4.3

### 4. Syscall Roundtrip Tests

**Purpose:** Verify syscall interface and kernel-user transitions

**Test Patterns:**
- `syscall.*installing.*INT.*0x80` - Syscall gate installation
- `Syscall interface ready` - Interface readiness
- `SYS_write` / `SYS_read` / `SYS_open` - Syscall handlers
- `syscall.*handler` - Handler invocation
- `user.*AI.*service.*scheduled` - User process with syscalls

**Validation Criteria:**
- INT 0x80 gate installation successful (≥1 pattern)
- Syscall handler registration successful (≥1 pattern)
- No critical syscall errors
- User process syscall execution detected

**Requirements Validated:** 4.4

### 5. QEMU Debugging Interface Tests

**Purpose:** Validate QEMU monitor interface for advanced debugging

**Test Operations:**
- Monitor socket connection
- Register inspection commands (`info registers`)
- CPU state queries (`info cpus`)
- Memory information access (`info memory`)

**Validation Criteria:**
- Monitor interface accessible
- Commands execute successfully
- Valid responses received

## Usage Instructions

### Prerequisites

1. **QEMU Installation**
   ```bash
   # Ubuntu/Debian
   sudo apt install qemu-system-x86
   
   # Windows
   # Download from https://www.qemu.org/download/
   ```

2. **Build Environment**
   ```bash
   # Ensure kernel is built
   make all
   
   # Verify EFI image exists
   ls -la EFI.img kernel.elf
   ```

### Running Tests

#### Master Test Runner (Recommended)

```bash
# Run integrated test suite
./run_qemu_tests.sh

# Run individual test suites
./run_qemu_tests.sh --individual

# Verbose output with log retention
./run_qemu_tests.sh --verbose --save-logs

# Custom timeout
./run_qemu_tests.sh --timeout 120
```

```powershell
# PowerShell equivalent
.\run_qemu_tests.ps1
.\run_qemu_tests.ps1 -Individual
.\run_qemu_tests.ps1 -Verbose -SaveLogs
.\run_qemu_tests.ps1 -Timeout 120
```

#### Individual Test Scripts

```bash
# Comprehensive integration tests
./qemu_integration_tests.sh --suite all --verbose

# Specific test suites
./qemu_integration_tests.sh --suite boot
./qemu_integration_tests.sh --suite ring3
./qemu_integration_tests.sh --suite devfs
./qemu_integration_tests.sh --suite syscall

# Specialized validation scripts
./ring3_validation_test.sh --verbose --save-logs
./devfs_validation_test.sh --timeout 60
./syscall_roundtrip_test.sh --verbose
```

### Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `--timeout N` | Test timeout in seconds | 45-60s |
| `--verbose` | Enable detailed output | false |
| `--save-logs` | Preserve log files | false |
| `--interactive` | Show QEMU display | false |
| `--suite NAME` | Run specific test suite | all |

## Test Reports

### Report Types

1. **Individual Test Reports**
   - `*_output.log` - QEMU console output
   - `*_error.log` - QEMU error output
   - `*_analysis.log` - JSON test analysis

2. **Comprehensive Reports**
   - `qemu_integration_test_report.md` - Integrated suite report
   - `master_test_report.md` - Master test execution report

3. **Debug Information**
   - `qemu_debug.log` - QEMU debug output (when enabled)
   - `qemu_syscall_debug.log` - Syscall-specific debug output

### Report Contents

- **Test Results Summary**: Pass/fail status, execution times
- **Pattern Detection Analysis**: Success pattern matching results
- **Component Coverage**: Detailed validation of each system component
- **Requirements Traceability**: Mapping to Phase 1 requirements
- **Error Analysis**: Detailed failure information when applicable
- **Recommendations**: Next steps and remediation guidance

## Troubleshooting

### Common Issues

1. **QEMU Not Found**
   ```
   Solution: Install QEMU system emulation package
   ```

2. **Build Artifacts Missing**
   ```
   Solution: Run 'make all' to build kernel and EFI image
   ```

3. **Test Timeouts**
   ```
   Solution: Increase timeout with --timeout option
   Check system performance and QEMU configuration
   ```

4. **Pattern Detection Failures**
   ```
   Solution: Review kernel output logs for actual messages
   Verify kernel functionality is implemented correctly
   ```

### Debug Procedures

1. **Enable Verbose Output**
   ```bash
   ./run_qemu_tests.sh --verbose --save-logs
   ```

2. **Review Individual Logs**
   ```bash
   cat *_output.log | grep -E "ERROR|PANIC|FAIL"
   ```

3. **Manual QEMU Testing**
   ```bash
   qemu-system-x86_64 -drive format=raw,file=EFI.img -serial stdio -m 256M
   ```

4. **Check Prerequisites**
   ```bash
   ./validate_toolchain.sh --verbose
   ```

## Requirements Validation Matrix

| Requirement | Test Suite | Validation Method | Status |
|-------------|------------|-------------------|--------|
| 4.1 QEMU Boot Success Detection | Boot Validation | Log pattern analysis | ✅ |
| 4.2 Ring3 User Process Execution | Ring3 Validation | Context switch verification | ✅ |
| 4.3 DevFS Device I/O Operations | DevFS Validation | Device registration analysis | ✅ |
| 4.4 Syscall Roundtrip Testing | Syscall Validation | Interface verification | ✅ |
| 4.5 Comprehensive Test Reports | All Suites | Automated report generation | ✅ |

## Integration with Development Workflow

### Continuous Integration

The test suite is designed for integration with CI/CD pipelines:

```yaml
# Example CI configuration
test_phase1:
  script:
    - make all
    - ./run_qemu_tests.sh --timeout 120 --save-logs
  artifacts:
    reports:
      - master_test_report.md
      - qemu_integration_test_report.md
    logs:
      - "*.log"
```

### Development Testing

For active development, use individual test scripts to validate specific components:

```bash
# After modifying Ring3 code
./ring3_validation_test.sh --verbose

# After updating DevFS
./devfs_validation_test.sh --save-logs

# After syscall changes
./syscall_roundtrip_test.sh --timeout 90
```

## Future Enhancements

### Planned Improvements

1. **Extended Pattern Library**: Additional validation patterns for Phase 2 features
2. **Performance Metrics**: Timing analysis and performance regression detection
3. **Multi-Architecture Support**: ARM64 and RISC-V test variants
4. **Interactive Debugging**: Integration with GDB for detailed analysis
5. **Automated Bisection**: Automatic failure point identification

### Extensibility

The test framework is designed for easy extension:

- **New Test Patterns**: Add patterns to validation arrays
- **Additional Test Suites**: Create new specialized test scripts
- **Custom Validation**: Implement domain-specific validation logic
- **Enhanced Reporting**: Extend report generation with additional metrics

## Conclusion

The AykenOS QEMU Integration Test Suite provides comprehensive validation of Phase 1 critical functionality through automated testing, detailed analysis, and systematic verification. The framework ensures reliable detection of kernel boot success, Ring3 user process execution, DevFS device operations, and syscall interface functionality, providing confidence in the system's readiness for Phase 2 development.

---
*Documentation generated for AykenOS Phase 1 Critical Fixes and Validation*

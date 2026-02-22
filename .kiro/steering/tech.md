# AykenOS Constitutional Technical Stack

**Version:** 1.0 Constitutional Edition  
**Authority:** ARCHITECTURE_FREEZE.md  
**Enforcement:** CI Gates + Build Validation

This document defines mandatory build system, toolchain, and development practices for AykenOS.

## Build System (Constitutional)

Primary build tool: **GNU Make** (Makefile)

**Build Discipline:**
- Clean builds MUST be reproducible
- Build artifacts MUST NOT be tracked in git
- Profile changes MUST trigger full rebuild
- ABI changes MUST trigger `make generate-abi`

**Enforcement:**
- `make ci-gate-hygiene` (tracked artifacts)
- `make ci-gate-workspace` (reproducibility)
- `.build_profile.stamp` (profile tracking)

## Toolchain

### Kernel (C/Assembly)
- **Compiler**: `clang` (target: x86_64-elf)
- **Linker**: `ld.lld`
- **Assembler**: `nasm` (for .asm files)
- **Flags**: `-ffreestanding -m64 -mcmodel=large -fno-pic -mno-red-zone`

### UEFI Bootloader
- **Compiler**: `clang` (target: x86_64-pc-win32-coff)
- **Linker**: `lld-link`
- **Subsystem**: efi_application

### Userspace & Tools (Rust)
- **Compiler**: `rustc` / `cargo`
- **Target**: x86_64-unknown-none (for kernel components)

## Tech Stack

### Languages
- **C**: Kernel core (x86_64)
- **Assembly**: Low-level CPU operations (NASM, GNU AS)
- **Rust**: Userspace runtime, AI core, constitutional tools

### Key Libraries/Frameworks
- **gnu-efi**: UEFI development headers
- **QEMU**: Testing and emulation (qemu-system-x86_64)
- **OVMF**: UEFI firmware for QEMU

## Common Commands

### Build
```bash
# Clean build
make clean
make all                    # Build kernel + bootloader

# Profile-specific builds
make release                # Optimized build (default)
make validation             # Debug build with instrumentation
make validation-strict      # Validation + -Werror

# Individual components
make kernel                 # Kernel only
make bootloader             # Bootloader only
make userspace-runtime      # Rust userspace components
```

### Test & Run
```bash
# Create EFI disk image
make efi-img

# Run in QEMU
make run                    # Standard boot
make run-preempt            # Preemption validation
make run-preempt-strict     # Strict marker-mode validation

# Validation suite
make validate               # Full validation
make validate-toolchain     # Toolchain check
make validate-build         # Build system check
make validate-qemu          # QEMU integration test
```

### CI Gates (Freeze Enforcement - Constitutional)

**Mandatory Gates (Fail-Closed):**
```bash
# Individual gates (order matters)
make ci-gate-abi            # ABI stability check (MUST pass)
make ci-gate-boundary       # Ring0/Ring3 boundary enforcement (MUST pass)
make ci-gate-ring0-exports  # Ring0 export surface check (MUST pass)
make ci-gate-hygiene        # Repository cleanliness (MUST pass)
make ci-gate-constitutional # Constitutional compliance (MUST pass)
make ci-gate-workspace      # Workspace integrity (MUST pass)
make ci-gate-syscall-v2-runtime  # Syscall runtime validation (MUST pass)
make ci-gate-sched-bridge-runtime  # Scheduler bridge runtime validation (MUST pass)
make ci-gate-performance    # Performance regression check (MUST pass)

# Full CI suite
make ci                     # Standard CI (enforced gates)
make ci-freeze              # Strict freeze suite (all gates, fail-closed)
make ci-freeze-local        # Local freeze (skip perf/tooling)
```

**Gate Failure Policy:**
- Any gate failure → **PR BLOCKED**
- Evidence MUST be reviewed
- Manual intervention required
- No auto-fix allowed

**Evidence Location:**
- `evidence/run-<RUN_ID>/gates/` (per-gate reports)
- `evidence/run-<RUN_ID>/reports/summary.json` (verdict)

**Constitutional Requirements:**
- All gates MUST pass for merge
- Evidence MUST be committed
- Baseline changes require RFC
- Gate bypass is prohibited

### Development
```bash
# Setup environment
make setup                  # Auto-install dependencies
make install-deps           # Manual dependency installation
make check-deps             # Verify toolchain

# Quick dev cycle
make dev                    # Clean + build + test

# ABI management
make generate-abi           # Generate NASM includes from C headers
make guard-context-offsets  # Enforce context offset discipline
```

### Rust Components
```bash
# Ayken-core (AI/data systems)
cd ayken-core
cargo build                 # Build all crates
cargo test                  # Run tests
cargo build -p abdf         # Build specific crate

# Userspace runtime
cd userspace
cargo build                 # Build userspace components
cargo test                  # Run userspace tests

# Constitutional tool
cd ayken
cargo build                 # Build ayken CLI
cargo test                  # Run constitutional tests
./target/debug/ayken check  # Run constitutional check
```

### Ring3 Policy Library (C)
```bash
# libayken (Ring3 VFS/DevFS/Scheduler)
cd userspace/libayken
make all                    # Build all Ring3 policy components
make test                   # Build test binaries
make clean                  # Clean build artifacts
make check                  # Constitutional compliance check
```

## Build Profiles (Constitutional)

### Release (default)
- Optimization: `-O2`
- Debug info: `-g1`
- Flags: `AYKEN_DEBUG_IRQ=0`, `AYKEN_DEBUG_SCHED=0`
- **Use for:** Production builds, performance baselines

### Validation
- Optimization: `-O0`
- Debug info: `-g3`
- Flags: `AYKEN_DEBUG_IRQ=1`, `AYKEN_DEBUG_SCHED=1`, `AYKEN_VALIDATION=1`
- Optional: `VALIDATION_WERROR=1` for strict warnings
- **Use for:** CI gates, development, debugging

**Rules:**
- `AYKEN_SCHED_FALLBACK=1` ONLY allowed with `KERNEL_PROFILE=validation`
- `make ci-freeze` enforces `AYKEN_SCHED_FALLBACK=0`
- Profile mixing → undefined behavior

## Environment Variables (Constitutional)

**Mandatory Variables:**
- `KERNEL_PROFILE`: `release` or `validation` (default: `release`)
- `KERNEL_EXPORT_POLICY`: `0` or `1` (default: `1`, freeze requires `1`)

**Optional Variables:**
- `VALIDATION_WERROR`: `0` or `1` (treat warnings as errors)
- `AYKEN_SCHED_FALLBACK`: `0` or `1` (scheduler fallback, validation only)
- `PERF_BASELINE_MODE`: `constitutional` or `provisional` (default: `constitutional`)

**Freeze Mode Requirements:**
- `KERNEL_EXPORT_POLICY=1` (mandatory)
- `AYKEN_SCHED_FALLBACK=0` (mandatory)
- `PERF_BASELINE_MODE=constitutional` (mandatory)

## Dependencies

### Required
- clang (LLVM toolchain)
- ld.lld (LLVM linker)
- nasm (assembler)
- python3 (build scripts)
- nm (symbol inspection)

### Optional
- qemu-system-x86_64 (testing)
- cargo/rustc (Rust components)
- git (version control)

## Platform Support

- **Primary**: x86_64 (UEFI)
- **In Progress**: ARM64, RISC-V, Raspberry Pi, MCU
- **Host Development**: Linux, macOS, Windows (WSL2)

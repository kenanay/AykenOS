# AykenOS Constitutional Project Structure

**Version:** 1.0 Constitutional Edition  
**Authority:** ARCHITECTURE_FREEZE.md  
**Enforcement:** CI Gates + Symbol Scanning

This document defines the mandatory project structure and architectural boundaries for AykenOS.

## Top-Level Organization

```
AykenOS/
├── kernel/              # C-based kernel (Ring0)
├── bootloader/          # Multi-architecture bootloaders
├── userspace/           # Ring3 components (Rust)
├── ayken-core/          # AI/data systems (Rust)
├── ayken/               # Constitutional governance tool (Rust)
├── docs/                # Documentation
├── tools/               # Build and development tools
├── scripts/             # CI and automation scripts
├── evidence/            # CI gate evidence (auto-generated)
├── _ayken/              # Design specifications
└── firmware/            # OVMF and firmware files
```

## Kernel Structure (`kernel/`)

Ring0 mechanism-only code (no policy decisions).

```
kernel/
├── arch/x86_64/         # Architecture-specific code
│   ├── *.asm           # NASM assembly (context switch, interrupts)
│   ├── *.S             # GNU assembly
│   └── *.c             # x86_64-specific C code
├── sys/                 # System calls (v2 interface, 1000-1011)
├── mm/                  # Memory management (physical, virtual, heap)
├── sched/               # Scheduler mechanism (wake/block/switch)
├── proc/                # Process management
├── fs/                  # Filesystem stubs (minimal Ring0 interface)
├── drivers/             # Device drivers (console, timer, PIC)
└── include/             # Kernel headers
    ├── ayken_abi.h     # Single source of truth for ABI
    └── generated/       # Auto-generated headers
        ├── ayken_abi.inc    # NASM include (from ayken_abi.h)
        └── ring0.exports.map # Linker export map
```

### Key Kernel Files
- `ayken_abi.h`: ABI constants (context offsets, syscall IDs)
- `context_switch.asm`: Context switching (uses CTX_* constants only)
- `sys/syscall_v2.c`: Syscall dispatcher (1000-1011 range)

## Bootloader Structure (`bootloader/`)

Multi-architecture boot support.

```
bootloader/
├── efi/                 # UEFI x86_64 bootloader
│   ├── efi_main.c      # UEFI entry point
│   ├── ayken_boot.c    # Boot logic
│   ├── elf_loader.c    # ELF kernel loader
│   ├── paging.c        # Page table setup
│   └── boot.S          # Early assembly
├── arm64/               # ARM64 bootloader (in progress)
├── riscv/               # RISC-V bootloader (in progress)
├── rpi/                 # Raspberry Pi bootloader
└── mcu/                 # Microcontroller bootloader
```

## Userspace Structure (`userspace/`)

Ring3 policy components (all policy decisions happen here).

```
userspace/
├── libayken/            # Ring3 VFS/DevFS/Scheduler implementations (C)
│   ├── Makefile        # Build system for Ring3 policy library
│   ├── vfs.c/.h        # Virtual File System implementation
│   ├── devfs.c/.h      # Device File System implementation
│   ├── sched_hint.c/.h # Scheduler hint/policy implementation
│   └── *_test.c        # Test binaries for CI gate validation
├── ai-runtime/          # AI runtime services
├── bcib-runtime/        # BCIB execution engine
├── orchestration/       # Multi-agent orchestration
├── semantic-cli/        # Semantic command-line interface
└── dsl-parser/          # Domain-specific language parser
```

### libayken Build System

The `userspace/libayken/` directory contains a standalone Makefile for building Ring3 policy components:

**Build Targets:**
- `make all`: Build all Ring3 policy objects (vfs.o, devfs.o, sched_hint.o)
- `make test`: Build test binaries for CI gate validation
- `make clean`: Remove build artifacts
- `make check`: Constitutional compliance check

**Constitutional Design:**
- Ring3 policy implementations only (VFS, DevFS, Scheduler)
- No Ring0 dependencies (userspace-only)
- Syscall interface 1000-1011 only
- Fail-closed design (no Ring0 exports)

## Ayken-Core Structure (`ayken-core/`)

Rust-based AI and data systems.

```
ayken-core/
└── crates/
    ├── abdf/            # Ayken Binary Data Format (AI/ML data)
    ├── abdf-builder/    # ABDF builder tools
    ├── bcib/            # Binary CLI Instruction Buffer
    └── d4-constitutional/ # Constitutional policy engine
        └── bmode/       # B-MODE analysis framework (LOCKED)
            ├── register_invariants/  # Register analysis (LOCKED)
            └── integration/          # Integration pipeline (LOCKED)
```

## Constitutional Tool (`ayken/`)

Development governance system (NOT part of AykenOS runtime).

```
ayken/
├── ahs/                 # Architecture Health Score
├── ahts/                # Architecture Health Trend System
├── arre/                # Automated Refactoring Recommendations
├── arh/                 # Auto-Refactor Hints
├── mars/                # Module-level Architecture Risk Score
├── allow/               # Allow directive system
├── waiver/              # Waiver lifecycle management
├── rules/               # Constitutional rule definitions
├── cli/                 # Command-line interface
└── steering/            # Configuration files
    ├── AHS_CONFIG.toml
    ├── CLASSES.md
    ├── NON_OVERRIDABLE.md
    └── MODULE_BOUNDARIES.md
```

## Documentation Structure (`docs/`)

```
docs/
├── architecture-board/  # Architecture decision records
├── constitution/        # Constitutional framework docs
├── development/         # Development guides
├── operations/          # Operational procedures
├── phase1/              # Phase 1 reports
├── phase2/              # Phase 2 reports
├── rfc/                 # RFC templates and records
├── roadmap/             # Roadmap and freeze workflow
├── setup/               # Setup guides (Windows, Linux, macOS)
└── waivers/             # Waiver registry and templates
```

## Build Artifacts

```
build/                   # Compiled objects (gitignored)
evidence/                # CI gate evidence (run-based)
  └── run-<RUN_ID>/
      ├── meta/          # Run metadata (git, toolchain)
      ├── artifacts/     # Build artifacts (kernel.elf, maps)
      ├── gates/         # Individual gate reports
      └── reports/       # Summary reports
```

## Important Files

### Root Level
- `Makefile`: Main build system
- `linker.ld`: Kernel linker script (higher-half mapping)
- `README.md`: Project overview
- `ARCHITECTURE_FREEZE.md`: Freeze rules and enforcement
- `LICENSE`: Dual-license (ASAL + ACL)

### Configuration
- `.github/pull_request_template.md`: PR template
- `.github/workflows/ci-freeze.yml`: CI freeze workflow
- `.gitignore`: Build artifacts exclusion

## Naming Conventions

### Files
- Kernel C: `snake_case.c`
- Kernel headers: `snake_case.h`
- Assembly: `snake_case.asm` (NASM), `snake_case.S` (GNU)
- Rust: `snake_case.rs`

### Symbols
- Kernel functions: `snake_case` (e.g., `kmain`, `sys_v2_map_memory`)
- Macros/constants: `UPPER_SNAKE_CASE` (e.g., `CTX_RIP`, `SYS_V2_BASE`)
- Types: `snake_case_t` (e.g., `cpu_context_t`, `irq_timer_frame_t`)

### Modules
- Rust modules: `snake_case` (e.g., `register_invariants`, `bmode`)
- Rust crates: `kebab-case` (e.g., `ayken-core`, `bcib-runtime`)

## Architecture Boundaries (Constitutional)

### Ring0 (Kernel) - MECHANISM ONLY

**Allowed:**
- Memory primitives (map, unmap, protect)
- Context switch mechanism
- Interrupt handling (entry, dispatch, exit)
- Syscall dispatch (no policy decisions)

**Forbidden (PR Auto-Reject):**
- Policy decisions (scheduler logic, VFS access control)
- Direct userspace calls
- AI inference or ML operations
- File access decisions

**Enforcement:**
- Symbol-level scanning: `tools/ci/symbol-scan.sh`
- Deny list: `tools/ci/deny.symbols` (constitutional)
- Allow list: `tools/ci/allow.symbols` (constitutional)
- CI gate: `make ci-gate-boundary` (mandatory)

### Ring3 (Userspace) - POLICY ONLY

**Allowed:**
- All policy decisions (scheduler, VFS, DevFS, AI)
- BCIB execution engine
- AI runtime services
- Application logic

**Forbidden (PR Auto-Reject):**
- Direct hardware access (MUST use syscalls 1000-1011)
- Kernel function calls
- Memory management bypass

**Enforcement:**
- Syscall interface validation: `make ci-gate-syscall-v2-runtime`
- Boundary scan: `make ci-gate-boundary`

### Ring0 Export Surface (Constitutional)

Ring0 exports are constitutional surface. Changes require ADR.

**Current Ceiling:** 165 symbols (enforced)  
**Whitelist:** `scripts/ci/constitutional-ring0-symbol-whitelist.regex`  
**Enforcement:** `make ci-gate-ring0-exports`

**Rules:**
- New export → ADR required
- Export removal → version bump required
- Export ceiling breach → **FAIL**

## Module Organization Principles (Constitutional)

1. **Single Responsibility** (MUST): Each module has one clear purpose
2. **Mechanism/Policy Separation** (MUST): Ring0 = mechanism, Ring3 = policy
3. **Constitutional Compliance** (MUST): All code follows governance rules
4. **Evidence-Based** (MUST): Changes require CI gate evidence
5. **Immutability** (MUST): Locked modules (BMODE, register_invariants) are permanent

**Violation of principles 1-5 → PR AUTO-REJECT**

## ABI Change Protocol (Constitutional)

Changes to `kernel/include/ayken_abi.h` require:

1. **Version Bump**: `AYKEN_ABI_VERSION` MUST increment
2. **RFC Approval**: Architecture Board review required
3. **Evidence**: `make ci-gate-abi` MUST pass
4. **Regeneration**: `make generate-abi` MUST be run
5. **Documentation**: Update syscall transition guide

**Unauthorized ABI change → CI FAIL + PR REJECT**

## Test Organization

- Kernel tests: `*_test.c` (excluded from kernel.elf link)
- Rust tests: `tests/` subdirectories or `#[cfg(test)]` modules
- Integration tests: `tools/qemu/` scripts
- Property tests: `proptest-regressions/` directories

## Evidence Organization

All CI gate runs produce evidence in `evidence/run-<RUN_ID>/`:
- Deterministic run IDs: `YYYYMMDDTHHMMSSZ-<git-short-sha>`
- Immutable evidence: Never modified after creation
- Structured reports: JSON format for machine parsing
- Human-readable logs: Text format for debugging

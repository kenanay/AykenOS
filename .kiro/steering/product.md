# AykenOS Constitutional Product Definition

**Version:** 1.0 Constitutional Edition  
**Authority:** ARCHITECTURE_FREEZE.md  
**Enforcement:** CI Gates + Branch Protection

AykenOS is an AI-native, execution-centric operating system that reimagines traditional OS architecture with a data-driven, deterministic approach.

## Core Philosophy (Non-Negotiable)

- **Execution-Centric**: 11 mechanism syscalls (1000-1010) instead of traditional POSIX interface
- **Ring3 Empowerment**: All policy decisions (VFS, DevFS, scheduler, AI) MUST run in userspace
- **Ring0 Minimalism**: Kernel SHALL provide only mechanisms (memory, context, interrupts)
- **AI-Native Design**: AI is integrated at the core, not as an add-on
- **Capability-Based Security**: Token-based access control with granular permissions
- **Deterministic Execution**: Evidence-based, reproducible behavior enforced by CI

## Deterministic Execution Model

AykenOS enforces deterministic behavior at all levels:

- **No Busy-Loop Timing**: Timing hacks are prohibited
- **Tick-Based Regression**: Performance regression injection via controlled tick delays only
- **CI Reproducibility**: All builds MUST be reproducible on authority environment
- **Evidence Immutability**: `evidence/` directory is append-only, never modified
- **Baseline Lock**: Performance and ABI baselines are immutable without RFC approval

## Key Features

- Multi-architecture support (x86_64, ARM64, RISC-V, Raspberry Pi, MCU)
- BCIB (Binary Compressed Instruction Bundle) execution engine (deterministic)
- ABDF (Ayken Binary Data Format) for AI/ML data (immutable format)
- Constitutional governance system for development quality
- Preemptive multitasking with 100 Hz scheduler (deterministic tick)

## AI-Native Architecture

AykenOS is AI-ready, not AI-aware:

- **ABDF Format**: Immutable binary data format for AI/ML workloads
- **BCIB Engine**: Deterministic instruction bundles for AI execution
- **Ring3 AI Runtime**: AI services run strictly in userspace (Ring3)
- **Kernel AI-Agnostic**: Kernel provides mechanisms, AI provides policy
- **No Kernel Inference**: AI inference MUST NOT run in Ring0

## Non-Negotiable Rules

These rules are enforced by CI gates and MUST NOT be violated:

### 1. Ring0 Policy Prohibition
- Ring0 code MUST NOT contain policy decisions
- Scheduler logic, VFS access control, AI inference in Ring0 → **PR AUTO-REJECT**
- Violation detection: `make ci-gate-boundary`

### 2. ABI Stability
- Syscall range 1000-1010 is FROZEN
- ABI changes require version bump + RFC approval
- `ayken_abi.h` is single source of truth
- Violation detection: `make ci-gate-abi`

### 3. Ring0 Export Surface
- Ring0 exports are constitutional surface
- New exports require ADR (Architecture Decision Record)
- Export ceiling: 165 symbols (enforced)
- Violation detection: `make ci-gate-ring0-exports`

### 4. Evidence Integrity
- Evidence directory is immutable after creation
- Baseline locks require authorized workflow only
- Manual evidence modification → **VIOLATION**
- Enforcement: `make ci-gate-hygiene`

### 5. Determinism Requirement
- No timing-dependent behavior without tick injection
- CI reproducibility is mandatory
- Performance regression requires evidence
- Enforcement: `make ci-gate-performance`

## Current Status

- **Core OS**: Phase 4.5 COMPLETE (Gate-4 policy-accept proof operational)
- **Constitutional System**: Phases 1-12 COMPLETE (governance framework active)
- **Architecture Freeze**: ACTIVE (stabilization before AI integration)
- **CI Enforcement**: 11 gates active (ABI, Boundary, Ring0 Exports, Hygiene, Constitutional, Workspace, Syscall v2 Runtime, Sched Bridge Runtime, Policy Accept, Performance, Tooling Isolation)
- **Pre-CI Discipline**: Layered local advisory (fast: 4 gates, full: 9 gates, fail-closed)

## License

Dual-licensed:
- ASAL v1.0 (Source-Available) for educational/personal use
- ACL v1.0 (Commercial) for commercial applications

**Copyright © 2026 Kenan AY**

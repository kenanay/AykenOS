# AykenOS Constitutional Product Definition

**Version:** 1.0 Constitutional Edition  
**Authority:** ARCHITECTURE_FREEZE.md  
**Enforcement:** CI Gates + Branch Protection
**Current Execution Roadmap:** `docs/roadmap/CONSTITUTIONAL_STABILIZATION_ROADMAP_2026_05_23.md`
**Last Authority Sync:** 2026-05-23

AykenOS is an AI-native, execution-centric operating system that reimagines traditional OS architecture with a data-driven, deterministic approach.

## Core Philosophy (Non-Negotiable)

- **Execution-Centric**: 12 mechanism syscalls (1000-1011) instead of traditional POSIX interface
- **Ring3 Empowerment**: All policy decisions (VFS, DevFS, scheduler, AI) MUST run in userspace
- **Ring0 Minimalism**: Kernel SHALL provide only mechanisms (memory, context, interrupts)
- **AI-Native Design**: AI is a Ring3 policy/runtime concern; it is not kernel authority
- **Capability-Based Security**: Token-based access control with granular permissions
- **Deterministic Execution**: Evidence-based, reproducible behavior enforced by CI

## Deterministic Execution Model

AykenOS enforces deterministic behavior at all levels:

- **No Busy-Loop Timing**: Timing hacks are prohibited
- **Tick-Based Regression**: Performance regression injection via controlled tick delays only
- **CI Reproducibility**: All builds MUST be reproducible on authority environment
- **Evidence Immutability**: `evidence/` directory is append-only, never modified
- **Baseline Lock**: Performance and ABI baselines are immutable without RFC approval
- **Diagnostics Non-Authority**: Dashboards, AI output and observability cannot become runtime decision input

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
- Syscall range 1000-1011 is FROZEN
- ABI changes require version bump + RFC approval
- `shared/abi/ayken_abi.h` and `shared/abi/syscall_v2.h` are canonical sources
- Violation detection: `make ci-gate-abi`

### 3. Ring0 Export Surface
- Ring0 exports are constitutional surface
- New exports require ADR (Architecture Decision Record)
- Export ceiling: 193 symbols (enforced)
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
- **Phase 10 Runtime**: OFFICIALLY CLOSED (CPL3 entry + deterministic runtime, remote CI confirmed)
- **Phase 11 Verification**: OFFICIALLY CLOSED (ledger, ETI, replay, proof bundle, remote CI confirmed)
- **Phase 12 Trust Layer**: OFFICIALLY CLOSED (P12-01..P12-18 complete, remote CI run `23099070483` confirmed)
- **Phase 13 Kill-Switch**: GATES PASS (6/6 kill-switch gates PASS, tag `phase13-kill-switch-gates-pass`)
- **Phase 14 Observability**: OFFICIALLY CLOSED (distributed observability, 5 workstreams complete)
- **Phase 15 BCIB Engine**: OFFICIALLY CLOSED (BCIB Execution Engine v3, ci-freeze#24213727039, PR #104)
- **Phase 16 Verification Layer**: OFFICIALLY CLOSED (MVP complete, evidence chain verified, trust anchor established)
- **Phase 17 Execution Pipeline**: OFFICIALLY CLOSED (`phase17-official-closure` at `416a5392`)
- **Constitutional System**: Phases 1-17 COMPLETE (governance framework active)
- **Architecture Freeze**: ACTIVE (Phase-18 transition requires explicit pointer decision)
- **Worktree-Local Ring3 Rule**: executable user-leaf rule is live under `ci-gate-ring3-user-leaf-rule`; broader Phase10-A2 strict/global authority remains separate
- **CI Enforcement**: strict freeze chain includes dedicated `Ring3 User Leaf Rule` before broader `Ring3 Execution Phase10a2`, followed by low-half scaffold, mailbox/runtime gates, alias proof, Phase-13 kill-switch enforcement, and Phase-16 verification layer gates
- **Pre-CI Discipline**: Local advisory (5 core gates, ~60-90s, fail-closed) + verification layer integration
- **CURRENT_PHASE**: `17` (Phase-17 officially closed; Phase-18 transition not activated)
- **Current Execution Priority**: review `PHASE18_TRANSITION_DECISION.md` as a docs-only Platform Constitution transition package
- **Phase-18**: TRANSITION DECISION PACKAGE ONLY until explicit `CURRENT_PHASE` pointer transition
- **Scope Decision**: kernel expansion, new syscalls, Ring0 policy, AI Runtime authority and authority-surface growth remain deferred outside Phase-18 Platform Constitution scope

## License

Dual-licensed:
- ASAL v1.0 (Source-Available) for educational/personal use
- ACL v1.0 (Commercial) for commercial applications

**Copyright © 2026 Kenan AY**

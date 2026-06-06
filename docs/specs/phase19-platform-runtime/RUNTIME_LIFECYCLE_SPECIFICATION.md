# Phase-19 Runtime Lifecycle Specification

This document is subordinate to `PHASE19_RUNTIME_DECISION.md`,
`../../../PHASE19_POINTER_TRANSITION_DECISION.md`, and the Phase-18 Platform
Constitution reference set. In case of conflict, those documents prevail.

**Status:** ACTIVE RFC / RUNTIME IMPLEMENTATION NOT AUTHORIZED
**Contract id:** `ayken.phase19.runtime.lifecycle.v1`
**Authority boundary:** Documentation/specification only; not runtime source
code, scheduler policy, package installer, module loader, workspace runtime,
plugin host, capability issuer, trust issuer, Semantic CLI authority, AI
Runtime authority, syscall, kernel ABI expansion, or closure authority.

## Purpose

This RFC defines the deterministic lifecycle for the first possible
Phase-19 Platform Runtime MVP.

The lifecycle exists to bound future implementation. It does not implement a
runtime and does not authorize runtime implementation.

## Core Rule

```text
Lifecycle state != runtime authority
```

A lifecycle state may describe where a future MVP is in an admission/receipt
flow. It must never grant install, load, mount, execute, issue, trust, plugin,
Semantic CLI, AI, registry, or agent authority.

## Positive Scope

Version 1 lifecycle scope is limited to:

1. Static input bundle intake.
2. Input bundle binding.
3. Platform validation integration.
4. Workspace admission record preparation.
5. Runtime receipt emission.
6. Fail-closed denial.
7. Evidence output.

## Non-Goals

This lifecycle does not define:

1. A runtime binary.
2. A package manager.
3. A module loader.
4. A plugin loader.
5. A workspace mount engine.
6. A capability issuer.
7. A trust issuer.
8. Semantic CLI execution.
9. AI Runtime.
10. Agent behavior.
11. Kernel behavior.

## Lifecycle States

| State | Meaning | Authority boundary |
|---|---|---|
| `UNINITIALIZED` | No input bundle is bound | No work authorized |
| `INPUT_BOUND` | Static input bundle has been selected and digest-bound | Not validated or admitted |
| `VALIDATING` | Platform validation integration is evaluating input references | Not authority grant |
| `VALIDATION_REJECTED` | Validation failed or was blocked | Terminal denial |
| `VALIDATED_RECORDABLE` | Validation evidence is sufficient to prepare records | Not execution or trust |
| `ADMISSION_RECORDED` | Workspace admission record has been emitted | Not workspace creation or mount |
| `RECEIPT_EMITTED` | Runtime receipt has been emitted | Not token or handle |
| `ABORTED` | Flow stopped due to ambiguity, missing evidence, drift, or internal failure | Terminal denial |

Unknown states fail closed.

## Required Transition Order

The only valid positive transition order is:

```text
UNINITIALIZED
  -> INPUT_BOUND
  -> VALIDATING
  -> VALIDATED_RECORDABLE
  -> ADMISSION_RECORDED
  -> RECEIPT_EMITTED
```

The only valid denial transitions are:

```text
INPUT_BOUND -> ABORTED
VALIDATING -> VALIDATION_REJECTED
VALIDATING -> ABORTED
VALIDATED_RECORDABLE -> ABORTED
ADMISSION_RECORDED -> ABORTED
```

After `VALIDATION_REJECTED`, `RECEIPT_EMITTED`, or `ABORTED`, no further
authoritative transition is allowed.

## Determinism Requirements

A later implementation must prove:

1. Same input bundle digest produces the same lifecycle transcript.
2. Unknown fields produce the same denial state.
3. Missing references produce the same denial state.
4. Stale validation receipts produce the same denial state.
5. Authority-granting words produce the same denial state.
6. No wall-clock value is part of an authoritative lifecycle verdict.

## Evidence Requirements

Each lifecycle run must emit:

1. Input bundle digest.
2. Validation integration result digest.
3. Workspace admission record digest when emitted.
4. Runtime receipt digest when emitted.
5. Ordered lifecycle states.
6. Denial reason for rejected or aborted flows.

Evidence is output only. It must not become runtime control input.

## Acceptance Boundary

This RFC can support a future implementation only if that implementation
proves the positive path and negative denial paths through local and remote
evidence. Until then, this file is documentation only.

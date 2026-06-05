# Phase-19 Runtime Evidence Plan

This document is subordinate to `PHASE19_RUNTIME_DECISION.md` and the Phase-19
Runtime RFC set. In case of conflict, those documents prevail.

**Status:** PRE-IMPLEMENTATION RFC / PHASE-19 NOT ACTIVE / RUNTIME NOT AUTHORIZED
**Contract id:** `ayken.phase19.runtime.evidence_plan.v1`
**Authority boundary:** Documentation/specification only; not a CI gate
implementation, runtime implementation, loader, installer, workspace runtime,
issuer, trust assignment, Semantic CLI authority, AI Runtime authority,
syscall, kernel ABI expansion, or closure authority.

## Purpose

This plan defines the evidence that a later Phase-19 Runtime MVP
implementation must produce before activation or acceptance can be considered.

It does not implement any evidence gate.

## Core Rule

```text
Evidence plan != evidence PASS
```

Planning an evidence path does not prove runtime behavior. Only exact-SHA
local and remote evidence can support a later implementation review.

## Required Evidence Surfaces

A later implementation must provide evidence for:

1. Runtime lifecycle positive path.
2. Runtime lifecycle fail-closed negative paths.
3. Static input bundle canonical binding.
4. Platform validation integration.
5. Workspace admission record emission.
6. Runtime receipt emission.
7. Deterministic repeat.
8. Authority drift denial.
9. Kernel ABI freeze preservation.
10. Production default behavior.

## Positive Evidence

Positive evidence must prove only this bounded flow:

```text
static input bundle
  -> validation integration record
  -> workspace admission record
  -> runtime receipt
```

Positive evidence must explicitly state that it does not prove install, load,
mount, execute, issue, trust, plugin loading, Semantic CLI authority, AI
Runtime authority, registry publication, or agent behavior.

## Negative Evidence

Negative evidence must include at least:

1. Unknown input bundle field.
2. Duplicate input bundle key.
3. Missing manifest reference.
4. Stale manifest digest.
5. Mismatched package and manifest subject.
6. Missing Platform ABI validation receipt.
7. Validation FAIL.
8. Workspace declaration requesting real mount.
9. Receipt declaring token authority.
10. Trust classification treated as capability grant.
11. Plugin compatibility treated as loading.
12. Semantic CLI output treated as runtime authority.
13. AI output treated as runtime authority.
14. New syscall or kernel ABI expansion request.

Each negative case must fail closed before receipt success emission.

## Determinism Evidence

Determinism evidence must show:

1. Same input bundle produces same lifecycle transcript digest.
2. Same input bundle produces same admission record digest.
3. Same input bundle produces same receipt digest.
4. Denial cases produce stable denial reasons.
5. Wall-clock values do not affect authoritative verdicts.

## Remote Evidence

Before a future implementation can be considered, remote CI must prove:

1. Strict `ci-freeze` PASS on the candidate SHA.
2. Dev Loop PASS on the candidate SHA.
3. Any new runtime-specific gate PASS on the candidate SHA.
4. Kernel ABI gate PASS with `1000-1011`, count `12`, version `0x00010001`.
5. Phase-18 authority drift guard remains effective.

## Performance Evidence

If a future implementation touches runtime hot paths, it must define measured
surface, baseline authority, threshold policy, and local/remote acceptance
rules before claiming performance acceptance.

Docs-only RFC changes do not create runtime performance claims.

## Evidence Output Rule

Evidence is output only. It must not become control input for scheduling,
loading, admission, trust, capability, Semantic CLI, AI Runtime, registry, or
agent decisions.

## Acceptance Boundary

This plan is not a CI implementation. It is a required checklist for a later
implementation PR and pointer-transition review.

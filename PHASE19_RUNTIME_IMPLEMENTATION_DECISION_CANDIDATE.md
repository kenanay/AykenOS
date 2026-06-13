# Phase-19 Runtime Implementation Decision Candidate

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, `PHASE18_TRANSITION_DECISION.md`,
`PHASE18_ACTIVATION_DECISION.md`, the Phase-18 Platform Constitution
reference set, `AUTHORITY_DRIFT_GUARD.md`, `TERMINOLOGY_AUDIT.md`,
`PHASE19_RUNTIME_DECISION.md`, the Phase-19 Runtime RFC set,
`docs/specs/phase19-platform-runtime/CROSS_CONSISTENCY_REVIEW.md`,
`PHASE19_POINTER_TRANSITION_CANDIDATE.md`,
`PHASE19_ACTIVATION_PRECONDITIONS_REVIEW.md`, and
`PHASE19_POINTER_TRANSITION_DECISION.md`. In case of conflict, those
documents prevail.

**Status:** CANDIDATE / IMPLEMENTATION DECISION NOT ACCEPTED / RUNTIME SOURCE CODE NOT AUTHORIZED
**Candidate date:** 2026-06-06
**Candidate id:** `ayken.phase19.runtime_implementation_decision_candidate.v1`
**Authority boundary:** Candidate documentation only; not runtime source code,
not an implementation decision, not a manifest parser, not a package
installer, not a package executor, not a module loader, not workspace runtime,
not workspace creation, not real mount authority, not plugin host, not plugin
loading, not capability token minting, not capability issuance, not trust
assignment, not registry publication, not Semantic CLI authority, not AI
Runtime authority, not agent authority, not a syscall, not kernel ABI
expansion, not Ring0 policy, not merge authority, and not closure authority.

## Purpose

This candidate defines the narrowest acceptable shape for a later
Phase-19 Runtime MVP implementation decision.

It does not accept that implementation decision.

It does not authorize code.

It exists to prevent the next step from expanding from an admission/receipt
MVP into loader, installer, workspace runtime, issuer, Semantic CLI, AI, or
agent behavior.

`PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE_CANDIDATE.md` may narrow the
shape of a later implementation decision package, but it is not the
implementation decision package and does not authorize runtime source code.

`PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE_DRAFT.md` may narrow the
draft shape of a later implementation decision package, but it is not the
implementation decision package and does not authorize runtime source code.

`PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE.md` may accept the narrow
exact-SHA implementation decision package boundary, but it is not an
implementation PR, evidence package, acceptance review, or runtime source code
authority.

## Core Rule

```text
implementation decision candidate != implementation decision
implementation decision != runtime implementation
runtime MVP artifact != behavior source
```

The safe default remains no runtime source code.

## Candidate Decision Shape

A later implementation decision may be considered only if it is limited to a
minimal userspace admission harness for the accepted Phase-19 Runtime MVP
boundary.

The candidate decision may describe only this inert flow:

```text
static input bundle
  -> Phase-18 Platform ABI validation integration record
  -> workspace admission record
  -> deterministic runtime receipt
```

The flow must not install, load, mount, execute, issue, trust, publish,
schedule, grant, or bind anything.

## First Permitted Behavior Candidate

If a later exact-SHA implementation decision is accepted, the first behavior
that may be proposed is a bounded userspace harness that:

1. Reads a static, test-owned input bundle.
2. Checks only the bounded static test-owned bundle shape and referenced
   Phase-18 declarative contract metadata required for admission/receipt
   evidence.
3. Emits a validation-integration record.
4. Emits a workspace admission record.
5. Emits a deterministic runtime receipt.
6. Fails closed before receipt success on unknown, stale, contradictory, or
   authority-bearing input.

This behavior candidate is not a general parser. It is not a general manifest
parser. It is not a package installer. It is not a module loader. It is not a
workspace runtime. It is not a plugin host. It is not an issuer. It is not an
executor.

## Required Preconditions For A Later Decision

A later implementation decision PR must fail closed unless all of the
following are true:

| ID | Precondition | Required result |
|---|---|---|
| P19-I1 | Phase pointer stable | `CURRENT_PHASE=19` remains active only as planning/admission/receipt boundary |
| P19-I2 | RFC set accepted | all Phase-19 Runtime RFCs remain accepted and unchanged or are reviewed in the same decision chain |
| P19-I3 | Evidence plan and matrix mapped | `RUNTIME_EVIDENCE_PLAN.md` and `RUNTIME_EVIDENCE_MATRIX.md` positive, negative, determinism, remote, production-default, and performance requirements are mapped to concrete checks |
| P19-I4 | Inert artifact invariant preserved | input bundle, validation receipt, workspace admission record, and runtime receipt remain non-authority records |
| P19-I5 | Kernel ABI unchanged | syscall IDs `1000-1011`, syscall count `12`, and ABI version `0x00010001` remain frozen |
| P19-I6 | Runtime source separated | implementation code is reviewed in a later PR after this candidate; this candidate stays docs-only |
| P19-I7 | No authority drift | loader, installer, mount, execution, issuer, trust, Semantic CLI, AI, registry, and agent readings are denied |
| P19-I8 | Exact-SHA evidence required | strict `ci-freeze`, Dev Loop, and any new runtime-specific gates must PASS on the implementation decision subject |

Missing, ambiguous, or partially satisfied preconditions fail closed.

## Candidate Implementation Surface Limits

A later decision may propose a userspace-only source surface for a bounded
admission/receipt harness.

That later decision must not touch:

1. `kernel/` behavior.
2. `shared/abi/` syscall declarations or ABI layout.
3. Bootloader behavior.
4. Performance baselines.
5. CI workflow authority.
6. Package installer code.
7. Module loader code.
8. Workspace runtime or real mount code.
9. Plugin host or plugin loading code.
10. Capability issuer or token minting code.
11. Trust issuer or trust assignment code.
12. Semantic CLI authority paths.
13. AI Runtime authority paths.
14. Registry or agent code.

If any of these surfaces are required, the proposal must move to a separate
reviewed phase or authority decision.

## Required Evidence For A Later Implementation Decision

A later implementation decision must define exact evidence before runtime code
can be accepted:

1. Positive admission/receipt transcript for one static input bundle.
2. Deterministic repeat of lifecycle transcript digest.
3. Deterministic repeat of admission record digest.
4. Deterministic repeat of runtime receipt digest.
5. Negative unknown-field denial.
6. Negative stale-digest denial.
7. Negative missing Platform ABI validation denial.
8. Negative workspace real-mount denial.
9. Negative receipt-as-token denial.
10. Negative trust-as-capability denial.
11. Negative plugin-compatibility-as-loading denial.
12. Negative Semantic CLI output-as-authority denial.
13. Negative AI output-as-authority denial.
14. Kernel ABI freeze preservation.
15. Production default behavior.

Evidence is output only. It must not become runtime control input.

## Explicit Non-Goals

This candidate must not be read to authorize:

1. Runtime implementation.
2. General manifest parser implementation.
3. Package installation.
4. Package execution.
5. Module loading.
6. Plugin host, plugin loading, or plugin instantiation.
7. Workspace creation, workspace runtime, or real mounts.
8. Capability token minting.
9. Capability issuance.
10. Trust assignment or trust issuer behavior.
11. Registry publication or marketplace behavior.
12. Semantic CLI execution authority.
13. AI Runtime authority.
14. Agent behavior.
15. New syscalls.
16. Kernel ABI expansion.
17. Ring0 policy.
18. Observability-as-authority.

Unknown authority readings fail closed.

## Relationship To Later Phases

This candidate does not pull later phases into Phase-19:

1. Phase-20 registry/capability ecosystem remains later work.
2. Phase-21 Semantic CLI authority remains later work.
3. Phase-22 AI Runtime remains later work.
4. Phase-23+ agent systems remain later work.

Those phases require their own reviewed decision packages.

## Candidate Conclusion

This file records a candidate boundary for a later Phase-19 Runtime MVP
implementation decision.

The candidate allows discussion of the minimum admission/receipt harness
shape only.

Runtime implementation remains unauthorized.

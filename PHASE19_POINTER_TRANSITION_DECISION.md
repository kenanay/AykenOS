# Phase-19 Pointer Transition Decision

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, `PHASE18_TRANSITION_DECISION.md`,
`PHASE18_ACTIVATION_DECISION.md`, the Phase-18 Platform Constitution
reference set, `AUTHORITY_DRIFT_GUARD.md`, `TERMINOLOGY_AUDIT.md`,
`PHASE19_RUNTIME_DECISION.md`, the Phase-19 Runtime RFC set,
`docs/specs/phase19-platform-runtime/CROSS_CONSISTENCY_REVIEW.md`,
`PHASE19_POINTER_TRANSITION_CANDIDATE.md`, and
`PHASE19_ACTIVATION_PRECONDITIONS_REVIEW.md`. In case of conflict, those
documents prevail unless this decision is the narrower pointer authority for
`docs/roadmap/CURRENT_PHASE`.

**Status:** POINTER TRANSITION DECISION / CURRENT_PHASE=19 / RUNTIME IMPLEMENTATION NOT AUTHORIZED
**Decision date:** 2026-06-06
**Decision id:** `ayken.phase19.pointer_transition.decision.v1`
**Authority boundary:** Phase pointer decision only; not runtime
implementation, not a manifest parser, not a package installer, not a package
executor, not a module loader, not workspace runtime, not workspace creation,
not real mount authority, not plugin host, not plugin loading, not capability
token minting, not capability issuance, not trust assignment, not registry
publication, not Semantic CLI authority, not AI Runtime authority, not agent
authority, not a syscall, not kernel ABI expansion, not Ring0 policy, not
merge authority, and not closure authority.

## Decision

`docs/roadmap/CURRENT_PHASE` is transitioned from `CURRENT_PHASE=18` to
`CURRENT_PHASE=19`.

Phase-19 is active only as the documented **Platform Runtime MVP planning,
validation-integration, admission-record, and receipt-boundary phase**.

CURRENT_PHASE=19 does not authorize runtime implementation.

Implementation authority remains denied.

## Core Rule

```text
pointer transition != runtime implementation
CURRENT_PHASE=19 != runtime source code authority
runtime artifact != behavior source
```

The safe default remains no runtime behavior unless a later reviewed
implementation decision grants a specific bounded behavior and supplies its
own evidence package.

`PHASE19_RUNTIME_IMPLEMENTATION_DECISION_CANDIDATE.md` may narrow the shape of
that later implementation decision, but it is not the implementation decision
and does not authorize runtime source code.

## Transition Basis

The pointer transition is based on the accepted pre-transition chain:

1. Phase-17 official closure remains verified by `phase17-official-closure`
   at `416a5392afbe217e16d26a59e2e1716fdfa9c8f6`.
2. Phase-18 was active as Platform Constitution only before this transition
   and remains the accepted Platform Constitution reference set.
3. Phase-18 Authority Drift Guard remains the review guard for runtime,
   loader, issuer, workspace, plugin, trust, capability, Semantic CLI, AI, and
   agent drift.
4. Phase-18 Terminology Audit remains the vocabulary guard for high-risk
   terms.
5. `PHASE19_RUNTIME_DECISION.md` defines the Platform Runtime MVP planning
   boundary.
6. The Phase-19 Runtime RFC set under
   `docs/specs/phase19-platform-runtime/` defines lifecycle, static input
   bundle, validation integration, workspace admission record, runtime
   receipt, evidence, and denial boundaries.
7. `docs/specs/phase19-platform-runtime/CROSS_CONSISTENCY_REVIEW.md`
   records PASS for the RFC set without granting implementation authority.
8. `PHASE19_POINTER_TRANSITION_CANDIDATE.md` defines the pre-transition
   exact-SHA and inert artifact conditions.
9. `PHASE19_ACTIVATION_PRECONDITIONS_REVIEW.md` records PASS for the
   precondition documentation without activating Phase-19.

## Active Phase-19 Scope

Phase-19 active scope is limited to:

1. Maintaining the accepted Runtime MVP decision boundary.
2. Maintaining the accepted Runtime RFC set.
3. Maintaining cross-document consistency for the Runtime MVP boundary.
4. Planning validation integration with the Phase-18 Platform ABI Validation
   Gate.
5. Planning inert workspace admission records.
6. Planning deterministic runtime receipts.
7. Preparing a later implementation decision package and evidence plan.

The only allowed MVP shape remains:

```text
static input bundle
  -> Phase-18 Platform ABI validation integration
  -> workspace admission record
  -> deterministic runtime receipt
```

This flow must not install, load, mount, execute, issue, trust, publish,
schedule, or grant anything.

## Inert Artifact Rule

Every Phase-19 Runtime MVP artifact remains inert.

| Artifact | Safe meaning | Forbidden reading |
|---|---|---|
| Static input bundle | Digest-bound declarative input set | Parser, installer request, loader request, execution request, token request, workspace creation request |
| Validation receipt | Evidence that validation inputs were checked | Authorization, install permission, load permission, trust grant, capability grant, workspace grant |
| Workspace admission record | Deterministic record for later review | Workspace creation, filesystem mount, namespace creation, handle, access grant |
| Runtime receipt | Digest-bound evidence output | Bearer token, capability token, workspace handle, plugin binding, execution right |

If a later proposal makes one of these artifacts active, executable, loadable,
mountable, transferable, or authority-bearing, the proposal must fail closed
or move to a later reviewed phase.

## Implementation Remains Denied

This pointer transition must not authorize:

1. Runtime source code.
2. Manifest parser implementation.
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

## Kernel And ABI Boundary

The kernel ABI remains frozen:

1. Syscall IDs remain `1000-1011`.
2. Syscall count remains `12`.
3. ABI version remains `0x00010001`.
4. Ring0 remains mechanism only.
5. Ring3 runtime policy remains outside kernel authority.

This pointer transition does not change kernel code, userspace runtime code,
baseline data, CI workflows, syscall declarations, or ABI layout.

## Evidence Rule

This decision becomes valid only for the exact subject SHA that contains this
pointer transition after required remote checks pass.

Required evidence for the pointer transition subject:

1. Strict `ci-freeze` PASS.
2. Dev Loop PASS.
3. No runtime source code changes.
4. No kernel ABI change.
5. No CI gate or workflow authority widening.
6. Status, roadmap, README, and documentation index synchronization.

Historical PASS results may be cited as context only. They cannot be inherited
as authority for this pointer transition subject SHA.

If the subject SHA changes after evidence is recorded, the evidence must be
regenerated for the new subject SHA.

## Post-Merge Exact-SHA Evidence

PR #172 merged this pointer transition to `main` at subject SHA
`37d0bf46898d2c01b75863d72f68910524e596a7` on 2026-06-06.

Post-merge remote evidence for that subject:

1. Strict `ci-freeze` run `27062096603` - PASS.
2. Dev Loop CI run `27062096584` - PASS.
3. PR #172 changed documentation/status/pointer files only.
4. No runtime source code, kernel code, syscall declaration, ABI layout,
   baseline, CI workflow, loader, installer, workspace runtime, plugin host,
   capability issuer, trust issuer, Semantic CLI authority, AI Runtime
   authority or Ring0 policy change was included.

This evidence records the accepted pointer transition subject only. It does
not authorize runtime implementation. If a later evidence-sync commit changes
this file or any other authority document, that later commit requires its own
remote checks before it can be treated as an accepted documentation record.

## Relationship To Later Phases

Phase-19 must not pull later phases forward:

1. Phase-20 registry/capability ecosystem remains later work.
2. Phase-21 Semantic CLI authority remains later work.
3. Phase-22 AI Runtime remains later work.
4. Phase-23+ agent systems remain later work.

Those phases require their own reviewed decision packages.

## Decision Conclusion

This package transitions the formal phase pointer to `CURRENT_PHASE=19`.

The transition authorizes only the documented Runtime MVP planning,
validation-integration, admission-record, and receipt-boundary phase.

Runtime implementation remains unauthorized.

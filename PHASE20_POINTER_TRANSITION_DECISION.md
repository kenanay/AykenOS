# Phase-20 Pointer Transition Decision

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, the Phase-18 Platform Constitution reference set,
`docs/specs/phase18-platform-constitution/AUTHORITY_DRIFT_GUARD.md`,
`docs/specs/phase18-platform-constitution/TERMINOLOGY_AUDIT.md`,
`PHASE19_RUNTIME_DECISION.md`, the Phase-19 Runtime RFC set,
`PHASE19_POINTER_TRANSITION_DECISION.md`,
`PHASE19_RUNTIME_IMPLEMENTATION_MAIN_EXACT_SHA_EVIDENCE_SYNC.md`,
`PHASE19_RUNTIME_IMPLEMENTATION_POST_MERGE_CONSISTENCY_REVIEW.md`,
`PHASE19_REFERENCE_INTEGRITY_VALIDATION_MAIN_EXACT_SHA_EVIDENCE_SYNC.md`,
`PHASE19_CLOSURE_READINESS_EVIDENCE_MANIFEST.md`,
`PHASE19_CLOSURE_READINESS_EXACT_MAIN_REBIND.md`,
`PHASE19_CONSTITUTIONAL_CLOSURE_REVIEW.md`,
`PHASE19_CLOSURE_DECISION.md`, and
`PHASE20_POINTER_TRANSITION_CANDIDATE.md`. In case of conflict, those
documents prevail unless this decision is the narrower accepted pointer
authority for `docs/roadmap/CURRENT_PHASE`.

**Status:** PRE-PUBLICATION PHASE-20 POINTER TRANSITION DECISION /
CURRENT_PHASE=20 AUTHORIZED ONLY IF ACCEPTED WITH THE BOUNDED POINTER
UPDATE / PHASE-20 PLANNING-GOVERNANCE ONLY / NO RUNTIME ACTIVATION / NO
GENERAL RUNTIME AUTHORITY / NO IMPLEMENTATION AUTHORITY
**Decision date:** 2026-06-28
**Decision id:** `ayken.phase20.pointer_transition_decision.v1`
**Pre-transition base main SHA:** `3d5cdf818fd363e223e1602c8ead82ab57284147`
**Phase-20 pointer transition candidate publication SHA:**
`3d5cdf818fd363e223e1602c8ead82ab57284147`
**Accepted decision subject SHA:** Not assigned before publication. The
accepted subject is the exact main SHA that publishes this decision together
with the bounded `CURRENT_PHASE=20` pointer update after required exact-SHA
checks pass.
**Authority boundary:** Pointer transition decision only after accepted
publication; not runtime activation, not general runtime authority, not
Phase-20 implementation authority, not package execution, not module
loading, not workspace runtime, not plugin loading, not capability issuance,
not trust assignment, not Semantic CLI authority, not AI Runtime authority,
not agent authority, not source acceptance, not new evidence authority, not
constitutional amendment authority, not syscall or kernel ABI authority, not
workflow-threshold, baseline, dependency, or Ring0 authority.

## Purpose

This document records the Phase-20 pointer transition decision package.

If accepted with the bounded pointer update, it transitions
`docs/roadmap/CURRENT_PHASE` from:

```text
CURRENT_PHASE=19
```

to:

```text
CURRENT_PHASE=20
```

The transition authorizes only Phase-20 planning and governance for the
capability and registry ecosystem. It does not activate runtime behavior,
does not authorize implementation, and does not grant general runtime
authority.

## Acceptance Condition

This document is not accepted merely by being drafted.

This decision becomes accepted only if all of the following are true:

1. A bounded pointer-transition PR publishes this document.
2. That same PR updates `docs/roadmap/CURRENT_PHASE` from
   `CURRENT_PHASE=19` to `CURRENT_PHASE=20`.
3. The PR contains no runtime, kernel, package, module, workspace, plugin,
   capability, trust, Semantic CLI, AI Runtime, agent, syscall, ABI,
   workflow-threshold, baseline, dependency, or Ring0 implementation change.
4. The accepted publication subject receives required exact-SHA CI,
   governance, spec, and boundary PASS evidence.
5. The accepted publication subject is treated as the only exact subject for
   this pointer transition.

Until those conditions are satisfied, this document is a pre-publication
decision package and does not change the active phase pointer.

## Core Rule

```text
pointer transition decision != runtime activation
CURRENT_PHASE=20 != Phase-20 implementation authority
Phase-20 planning-governance != package/module/capability behavior
accepted publication SHA != inherited historical evidence
```

The safe default remains no runtime behavior, no implementation authority,
and no capability or registry authority unless a later reviewed Phase-20
implementation decision grants a specific bounded behavior with its own
exact-SHA evidence.

## Decision Inputs

| Input | Recorded result |
|---|---|
| Pre-transition base main subject | `3d5cdf818fd363e223e1602c8ead82ab57284147` |
| Current active phase before transition | `CURRENT_PHASE=19` |
| Phase-19 Closure Decision | `PHASE19_CLOSURE_DECISION.md` |
| Phase-19 closure decision subject | `17de2131e01f743d8ca3ac4e431e9362f08dff39` |
| Phase-19 closure decision publication subject | `b89d38d3c2a24e6b722a08ad8b61811e794cdd9b` |
| Phase-20 Pointer Transition Candidate | `PHASE20_POINTER_TRANSITION_CANDIDATE.md` |
| Phase-20 candidate publication subject | `3d5cdf818fd363e223e1602c8ead82ab57284147` |
| Required pointer update | `docs/roadmap/CURRENT_PHASE`: `CURRENT_PHASE=19` -> `CURRENT_PHASE=20` |

These inputs are decision inputs only. They do not grant runtime
activation, implementation authority, package authority, module authority,
capability authority, registry authority, or general runtime authority.

## Candidate Publication Evidence Input

The Phase-20 Pointer Transition Candidate was published on main at exact
subject:

```text
3d5cdf818fd363e223e1602c8ead82ab57284147
```

Observed post-merge evidence for that subject:

| Evidence | Run | Result |
|---|---|---|
| Strict `ci-freeze` | `28315492268` | PASS |
| AykenOS Dev Loop CI | `28315492273` | PASS |
| Dev Loop optimized | `28315492275` | PASS |
| Dev Loop validation | `28315492265` | PASS |
| Governance Summary | `28315492247` | PASS |
| Spec Purity | `28315492272` | PASS |
| Evidence Isolation | `28315492242`, `28315492271` | PASS |
| Observation Boundary | `28315492288` | PASS |
| Naming Compliance | `28315492290`, `28315492294` | PASS |
| Workspace boundary | `28315492270` | PASS |
| Semantic CLI contract boundary | `28315492269` | PASS |
| BCIB core boundary | `28315492243` | PASS |
| DSL BCIB contract boundary | `28315492287` | PASS |
| Data runtime BCIB boundary | `28315492263` | PASS |
| AI Runtime boundary | `28315492262` | PASS |
| Capability manager boundary | `28315492245` | PASS |
| Proofd observability boundary | `28315492267` | PASS |
| Toolchain opcode registry boundary | `28315492264` | PASS |

This evidence records candidate publication readiness only. It cannot be
inherited as accepted pointer-transition evidence for a later publication
subject.

## Pointer Decision

If accepted under the Acceptance Condition, this decision transitions the
formal phase pointer to:

```text
CURRENT_PHASE=20
```

Phase-20 is active only as a bounded planning and governance phase for the
capability and registry ecosystem that follows the Phase-18 Platform
Constitution and the closed Phase-19 Runtime MVP boundary.

The transition does not authorize Phase-20 implementation, runtime
activation, package execution, module loading, workspace runtime, plugin
loading, capability issuance, trust assignment, Semantic CLI authority, AI
Runtime authority, agent authority, syscall expansion, kernel ABI expansion,
or Ring0 policy.

## Active Phase-20 Planning Scope

The accepted Phase-20 planning-governance scope is limited to:

1. Capability registry shape and fail-closed registration rules.
2. Capability issuance preconditions and non-bypass constraints.
3. Package and module registry governance boundaries.
4. Distribution, publication, revocation, and quarantine policy inputs.
5. Cross-contract consistency between Phase-18 constitution records,
   Phase-19 runtime receipts, and later capability/registry records.
6. Evidence requirements for any later Phase-20 implementation slice.
7. Explicit separation from runtime activation, package execution, module
   loading, plugin loading, Semantic CLI authority, AI Runtime authority,
   and agent authority.

The only acceptable initial Phase-20 posture is planning and governance.
Any implementation, activation, execution, issuance, publication, loading,
mounting, trust assignment, or registry behavior requires a later reviewed
implementation decision and exact-SHA evidence.

## Implementation Remains Denied

This pointer transition must not authorize:

1. Runtime activation.
2. General runtime authority.
3. Package installation, loading, execution, scheduling, or publication.
4. Module loading.
5. Workspace creation, workspace runtime, or real mounts.
6. Plugin host, plugin loading, or plugin instantiation.
7. Capability token minting or capability issuance.
8. Trust assignment or trust issuer behavior.
9. Registry publication or marketplace behavior.
10. Semantic CLI execution or verdict authority.
11. AI Runtime authority.
12. Agent behavior.
13. New syscalls.
14. Kernel ABI expansion.
15. Workflow-threshold, baseline, dependency, or Ring0 policy changes.
16. Observability-as-authority.

Unknown authority readings fail closed.

## Kernel And ABI Boundary

The kernel ABI remains frozen:

1. Syscall IDs remain `1000-1011`.
2. Syscall count remains `12`.
3. ABI version remains `0x00010001`.
4. Ring0 remains mechanism only.
5. Ring3 runtime policy remains outside kernel authority.

This pointer transition must not change kernel code, userspace runtime
code, syscall declarations, ABI layout, baseline data, CI workflow
authority, loader behavior, installer behavior, workspace runtime, plugin
host behavior, capability issuer behavior, trust issuer behavior, Semantic
CLI authority, AI Runtime authority, agent behavior, or Ring0 policy.

## Required Exact-Subject Evidence

The accepted pointer-transition subject must receive exact-SHA evidence for:

1. Strict `ci-freeze` PASS.
2. AykenOS Dev Loop CI PASS.
3. Dev Loop optimized and validation PASS.
4. Governance Summary PASS.
5. Spec Purity PASS.
6. Evidence Isolation PASS.
7. Naming, observation, workspace, Semantic CLI, BCIB, data runtime, AI
   Runtime, capability manager, proofd observability, and toolchain opcode
   boundary gates PASS.
8. No runtime source code change.
9. No kernel source code or ABI change.
10. No workflow-threshold, baseline, dependency, or Ring0 policy change.
11. No package, module, workspace, plugin, capability, trust, Semantic CLI,
    AI Runtime, or agent implementation change.

Historical PASS results may be cited as context only. They cannot be
inherited as pointer-transition authority for another SHA.

If the accepted publication subject changes, evidence must be evaluated for
the new exact subject.

## Pointer PR Scope

The bounded pointer-transition PR may include only:

1. `PHASE20_POINTER_TRANSITION_DECISION.md`.
2. Updating `docs/roadmap/CURRENT_PHASE` from `CURRENT_PHASE=19` to
   `CURRENT_PHASE=20`.
3. Roadmap, status, and documentation index synchronization that describes
   Phase-20 as planning/governance only.
4. Exact-SHA remote PASS evidence for the accepted publication subject.
5. Explicit preservation of implementation, runtime activation, and
   Phase-20 behavior separation.

Any source implementation, runtime wiring, kernel or ABI change, baseline
change, workflow authority change, dependency change, or package, module,
workspace, plugin, capability, trust, Semantic CLI, AI Runtime, or agent
behavior change is out of scope and fails closed.

## Relationship To Later Phase-20 Work

This decision may open Phase-20 planning and governance only.

Later Phase-20 work requires separate reviewed authority before it may:

1. Define implementation subject scope.
2. Accept source changes.
3. Generate implementation evidence.
4. Merge implementation.
5. Activate behavior.
6. Issue capability, trust, registry, package, module, workspace, plugin,
   Semantic CLI, AI Runtime, or agent authority.

No later Phase-20 authority may be inferred from this pointer transition.

## Publication Boundary

If this decision is merged, the landing SHA publishes the pointer decision
and the bounded phase pointer update. The landing SHA must not be read as
runtime activation, general runtime authority, implementation acceptance,
package/module/capability behavior, registry behavior, Semantic CLI
authority, AI Runtime authority, agent authority, syscall expansion, kernel
ABI expansion, or Ring0 authority.

The accepted publication subject must be treated as the exact subject for
this pointer transition. Any later technical change, authority expansion,
implementation proposal, or Phase-20 activation requires a separate reviewed
decision path.

## Decision Conclusion

If accepted with the bounded pointer update and required exact-SHA PASS
evidence, this decision transitions the formal phase pointer to:

```text
CURRENT_PHASE=20
```

The transition authorizes only Phase-20 planning and governance for the
capability and registry ecosystem.

Runtime activation, general runtime authority, Phase-20 implementation
authority, and all package, module, workspace, plugin, capability, trust,
Semantic CLI, AI Runtime, agent, syscall, kernel ABI, workflow-threshold,
baseline, dependency, and Ring0 authority remain pending and unauthorized
until separately reviewed and decided.

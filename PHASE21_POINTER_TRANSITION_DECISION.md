# Phase-21 Pointer Transition Decision

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, the Phase-18 Platform Constitution reference set,
`docs/specs/phase18-platform-constitution/AUTHORITY_DRIFT_GUARD.md`,
`docs/specs/phase18-platform-constitution/TERMINOLOGY_AUDIT.md`,
`PHASE19_RUNTIME_DECISION.md`, the Phase-19 Runtime RFC set,
`docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_MATRIX.md`,
`PHASE19_CLOSURE_DECISION.md`,
`PHASE20_POINTER_TRANSITION_CANDIDATE.md`,
`PHASE20_POINTER_TRANSITION_DECISION.md`,
`PHASE20_GOVERNANCE_OVERVIEW.md`,
`PHASE20_CAPABILITY_MODEL.md`,
`PHASE20_CAPABILITY_IDENTITY.md`,
`PHASE20_CAPABILITY_MANIFEST_SCHEMA.md`,
`PHASE20_CAPABILITY_LIFECYCLE.md`,
`PHASE20_REGISTRY_MODEL.md`,
`PHASE20_REGISTRY_GOVERNANCE.md`,
`PHASE20_TRUST_MODEL.md`,
`PHASE20_DISTRIBUTION_POLICY.md`,
`PHASE20_CAPABILITY_EVIDENCE_MODEL.md`,
`PHASE20_CAPABILITY_ACCEPTANCE_WORKFLOW.md`,
`PHASE20_IMPLEMENTATION_DECISION.md`,
`PHASE20_IMPLEMENTATION_SLICE.md`,
`PHASE20_IMPLEMENTATION_REVIEW.md`,
`PHASE20_IMPLEMENTATION_ACCEPTANCE_DECISION.md`,
`PHASE20_RUNTIME_DECISION.md`,
`PHASE20_RUNTIME_DECISION_REVIEW.md`,
`PHASE20_RUNTIME_ACCEPTANCE_DECISION.md`,
`PHASE20_RUNTIME_ACTIVATION_DECISION.md`,
`PHASE20_RUNTIME_IMPLEMENTATION_DECISION.md`,
`PHASE20_RUNTIME_IMPLEMENTATION_REVIEW.md`,
`PHASE20_RUNTIME_IMPLEMENTATION_ACCEPTANCE_DECISION.md`,
`PHASE20_CLOSURE_DECISION.md`, and
`PHASE21_POINTER_TRANSITION_CANDIDATE.md`. In case of conflict, those
documents prevail unless this decision is the narrower accepted pointer
authority for `docs/roadmap/CURRENT_PHASE`.

**Status:** PRE-PUBLICATION PHASE-21 POINTER TRANSITION DECISION /
CURRENT_PHASE=21 AUTHORIZED ONLY IF ACCEPTED WITH THE BOUNDED POINTER
UPDATE / PHASE-21 POINTER GOVERNANCE ONLY / NO PHASE-21 IMPLEMENTATION
SCOPE / NO PHASE-21 ACTIVATION / NO IMPLEMENTATION PROCEDURE / NO SOURCE
MODIFICATION / NO CODE IMPLEMENTATION / NO CODE EXECUTION / NO PROCESS
START / NO RUNTIME STATE CREATION / NO PACKAGE AUTHORITY / NO PACKAGE
INSTALLATION / NO PACKAGE LOADING / NO PACKAGE EXECUTION / NO DEPLOYMENT /
NO CAPABILITY ISSUANCE / NO TRUST ASSIGNMENT / NO REGISTRY PUBLICATION /
NO DISTRIBUTION AUTHORITY / NO SOURCE MERGE AUTHORITY / NO SOURCE
ACCEPTANCE
**Decision date:** 2026-07-01
**Decision id:** `ayken.phase21.pointer_transition_decision.v1`
**Pre-transition base main SHA:** `923cf55ba6bbe39dd9ac8634cad58c10ff4d395e`
**Phase-21 pointer transition candidate publication SHA:**
`923cf55ba6bbe39dd9ac8634cad58c10ff4d395e`
**Phase-20 closure decision exact-main SHA:**
`ee1f1c7f43fe478c8cbdab3fbeb2844365c9c5bc`
**Accepted decision subject SHA:** Not assigned before publication. The
accepted subject is the exact main SHA that publishes this decision together
with the bounded `CURRENT_PHASE=21` pointer update after required exact-SHA
checks pass.
**Authority boundary:** Pointer transition decision only after accepted
publication; not Phase-21 implementation scope, not Phase-21 activation,
not runtime implementation procedure, not source modification, not code
implementation, not code execution, not process start, not runtime state
creation, not general runtime authority, not unbounded execution authority,
not package authority, not package installation, not package loading, not
package execution, not deployment, not source acceptance, not source merge
authority, not source repository authority, not module loading, not
workspace runtime, not plugin loading, not capability token minting, not
capability issuance, not trust assignment, not trust issuer authority, not
registry authority, not registry publication, not publication authority, not
distribution authority, not distribution execution, not Semantic CLI
authority, not AI Runtime authority, not agent authority, not syscall
expansion, not kernel ABI expansion, not workflow-threshold, baseline,
dependency, or Ring0 authority.

## Purpose

This document records the Phase-21 pointer transition decision package.

If accepted with the bounded pointer update, it transitions
`docs/roadmap/CURRENT_PHASE` from:

```text
CURRENT_PHASE=20
```

to:

```text
CURRENT_PHASE=21
```

The transition opens Phase-21 only as a bounded pointer-governance phase for
a possible First Bounded Implementation authority path.

It does not define Phase-21 implementation scope.

It does not activate Phase-21 runtime behavior.

It does not authorize runtime implementation procedure.

It does not authorize source modification, code implementation, code
execution, process start, runtime state creation, package loading,
capability issuance, registry publication, trust assignment, source merge,
or general runtime authority.

## Acceptance Condition

This document is not accepted merely by being drafted.

This decision becomes accepted only if all of the following are true:

1. A bounded pointer-transition PR publishes this document.
2. That same PR updates `docs/roadmap/CURRENT_PHASE` from
   `CURRENT_PHASE=20` to `CURRENT_PHASE=21`.
3. The PR contains no runtime source, kernel source, package source,
   module source, workspace runtime, plugin host, Semantic CLI, AI Runtime,
   agent, syscall, ABI, workflow-threshold, baseline, dependency, or Ring0
   implementation change.
4. The accepted publication subject receives required exact-SHA CI,
   governance, spec, and boundary PASS evidence.
5. The accepted publication subject is treated as the only exact subject for
   this pointer transition.

Until those conditions are satisfied, this document is a pre-publication
decision package and does not change the active phase pointer.

## Core Rule

```text
pointer transition decision != Phase-21 implementation scope
pointer transition decision != Phase-21 activation
CURRENT_PHASE=21 != runtime implementation procedure
CURRENT_PHASE=21 != source modification
CURRENT_PHASE=21 != code implementation
CURRENT_PHASE=21 != code execution
CURRENT_PHASE=21 != process start
CURRENT_PHASE=21 != runtime state creation
CURRENT_PHASE=21 != package loading
CURRENT_PHASE=21 != package execution
CURRENT_PHASE=21 != capability issuance
CURRENT_PHASE=21 != registry publication
CURRENT_PHASE=21 != trust assignment
CURRENT_PHASE=21 != source merge
Phase-21 opened != implementation authority
Phase-21 opened != execution authority
accepted publication SHA != inherited historical evidence
```

The safe default remains no runtime behavior, no implementation procedure,
no source modification, no code execution, no runtime state, and no package,
capability, registry, trust, distribution, or source merge authority unless
a later reviewed Phase-21 decision grants a specific bounded authority with
its own exact-SHA evidence.

Unknown authority readings fail closed.

## Decision Inputs

| Input | Recorded result |
|---|---|
| Pre-transition base main subject | `923cf55ba6bbe39dd9ac8634cad58c10ff4d395e` |
| Current active phase before transition | `CURRENT_PHASE=20` |
| Phase-20 Closure Decision | `PHASE20_CLOSURE_DECISION.md` |
| Phase-20 Closure Decision exact-main SHA | `ee1f1c7f43fe478c8cbdab3fbeb2844365c9c5bc` |
| Phase-21 Pointer Transition Candidate | `PHASE21_POINTER_TRANSITION_CANDIDATE.md` |
| Phase-21 candidate publication subject | `923cf55ba6bbe39dd9ac8634cad58c10ff4d395e` |
| Phase-21 candidate PR | PR #224 |
| Phase-21 candidate review | Approved by `kenanay2020-hub` |
| Phase-21 candidate merge method | Normal maintainer squash merge; no admin bypass recorded |
| Required pointer update | `docs/roadmap/CURRENT_PHASE`: `CURRENT_PHASE=20` -> `CURRENT_PHASE=21` |
| Candidate Phase-21 theme | First Bounded Implementation |

These inputs are decision inputs only. They do not grant Phase-21
implementation scope, Phase-21 activation, runtime implementation procedure,
source modification, code implementation, code execution, process start,
runtime state creation, package authority, module authority, capability
authority, registry authority, trust authority, source merge authority, or
general runtime authority.

## Candidate Publication Evidence Input

The Phase-21 Pointer Transition Candidate was published on main at exact
subject:

```text
923cf55ba6bbe39dd9ac8634cad58c10ff4d395e
```

Observed post-merge evidence for that subject:

| Evidence | Run | Result |
|---|---|---|
| Strict `ci-freeze` | `28527579602` | PASS |
| AykenOS Dev Loop CI | `28527579690` | PASS |
| Dev Loop optimized | `28527579605` | PASS |
| Dev Loop validation | `28527579568` | PASS |
| Governance Summary | `28527579488` | PASS |
| Spec Purity | `28527579601` | PASS |
| Evidence Isolation | `28527579561`, `28527579734` | PASS |
| Observation Boundary | `28527579578` | PASS |
| Naming Compliance | `28527579638`, `28527579639` | PASS |
| Workspace boundary | `28527579624` | PASS |
| Semantic CLI contract boundary | `28527579636` | PASS |
| BCIB core boundary | `28527579548` | PASS |
| DSL BCIB contract boundary | `28527579622` | PASS |
| Data runtime BCIB boundary | `28527579721` | PASS |
| AI Runtime boundary | `28527579644` | PASS |
| Capability manager boundary | `28527579618` | PASS |
| Proofd observability boundary | `28527579695` | PASS |
| Toolchain opcode registry boundary | `28527579583` | PASS |

This evidence records candidate publication readiness only. It cannot be
inherited as accepted pointer-transition evidence for a later publication
subject.

## Pointer Decision

If accepted under the Acceptance Condition, this decision transitions the
formal phase pointer to:

```text
CURRENT_PHASE=21
```

Phase-21 is active only as a bounded pointer-governance phase for a possible
First Bounded Implementation authority path.

The transition does not define Phase-21 implementation scope.

The transition does not authorize runtime implementation procedure.

The transition does not modify source.

The transition does not implement code.

The transition does not execute code.

The transition does not start a process.

The transition does not create runtime state.

The transition does not install, load, or execute packages.

The transition does not issue capabilities.

The transition does not publish registry entries.

The transition does not assign trust.

The transition does not authorize source merge.

## Active Phase-21 Pointer-Governance Scope

The accepted Phase-21 pointer-governance scope is limited to:

1. Establishing that Phase-21 may be the next active phase after closed
   Phase-20.
2. Preserving Phase-20 closure as an exact prerequisite.
3. Preserving Phase-19 runtime authority boundaries.
4. Preserving Phase-18 Platform Constitution authority.
5. Keeping the candidate Phase-21 theme as First Bounded Implementation.
6. Keeping any initial Phase-21 posture userspace-only.
7. Keeping any initial Phase-21 posture non-executing.
8. Keeping any initial Phase-21 posture validator, receipt, fixture, or CI
   gate oriented.
9. Requiring separate reviewed decisions for any Phase-21 implementation
   scope, procedure, source modification, code implementation, execution,
   process start, runtime state, package, capability, registry, trust,
   deployment, distribution, Semantic CLI, AI Runtime, agent, or source
   merge authority.

The accepted Phase-21 pointer-governance scope is not implementation scope.

The accepted Phase-21 pointer-governance scope is not runtime
implementation procedure.

The accepted Phase-21 pointer-governance scope is not source modification.

The accepted Phase-21 pointer-governance scope is not code implementation.

The accepted Phase-21 pointer-governance scope is not code execution.

The accepted Phase-21 pointer-governance scope is not process start.

The accepted Phase-21 pointer-governance scope is not runtime state
creation.

Any implementation, activation, execution, issuance, publication, loading,
mounting, trust assignment, registry behavior, or source merge requires a
later reviewed decision and exact-SHA evidence.

## Implementation Remains Denied

This pointer transition must not authorize:

1. Phase-21 implementation scope.
2. Phase-21 activation.
3. Runtime implementation procedure.
4. Source modification.
5. Code implementation.
6. Code execution.
7. Process start.
8. Runtime state creation.
9. General runtime authority.
10. Unbounded execution authority.
11. Package installation, loading, execution, scheduling, or publication.
12. Module loading.
13. Workspace creation, workspace runtime, or real mounts.
14. Plugin host, plugin loading, or plugin instantiation.
15. Capability token minting or capability issuance.
16. Trust assignment or trust issuer behavior.
17. Registry publication or marketplace behavior.
18. Distribution authority or distribution execution.
19. Source acceptance or source merge authority.
20. Semantic CLI execution or verdict authority.
21. AI Runtime authority.
22. Agent behavior.
23. New syscalls.
24. Kernel ABI expansion.
25. Workflow-threshold, baseline, dependency, or Ring0 policy changes.
26. Observability-as-authority.

Unknown authority readings fail closed.

## Phase-20 Closure Relationship

Phase-21 pointer transition consumes Phase-20 closure context only.

Phase-20 closure remains closed for exact subject:

```text
ee1f1c7f43fe478c8cbdab3fbeb2844365c9c5bc
```

This pointer transition does not reopen Phase-20.

This pointer transition does not reinterpret Phase-20 closure.

This pointer transition does not extend Phase-20 closure into runtime
implementation procedure authority.

This pointer transition does not convert Phase-20 Runtime Implementation
Acceptance Decision, Phase-20 Closure Decision, or any Phase-20 governance
record into source modification, code implementation, code execution,
process start, runtime state creation, package loading, capability issuance,
registry publication, trust assignment, or source merge authority.

Any reading that treats Phase-20 closure as Phase-21 implementation
procedure authority fails closed.

## Phase-19 Runtime Authority Relationship

Phase-21 pointer transition remains subordinate to Phase-19 runtime
authority records.

Phase-19 runtime records may be read as boundary context for:

1. Runtime MVP planning boundaries.
2. Runtime evidence expectations.
3. Runtime non-goals and denials.
4. Platform runtime constitutional constraints.
5. Userspace-only runtime constraints.
6. Frozen syscall and kernel ABI boundaries.
7. Denied package, module, workspace, plugin, trust, capability, AI
   Runtime, Semantic CLI, and agent authority readings.

This pointer transition must not broaden, replace, supersede, weaken, or
reinterpret Phase-19 runtime authority records.

This pointer transition must not use Phase-21 activation as Phase-19 runtime
authority.

This pointer transition must not use `CURRENT_PHASE=21` to infer runtime
authority.

Any Phase-21 pointer transition reading that conflicts with Phase-19 runtime
authority records fails closed.

## Kernel And ABI Boundary

The kernel ABI remains frozen.

This pointer transition does not authorize:

1. New syscalls.
2. Kernel ABI expansion.
3. Syscall ID changes.
4. Syscall count changes.
5. ABI version changes.
6. Ring0 policy movement.
7. Kernel source modification.
8. Runtime source modification.
9. Workflow-threshold changes.
10. Baseline changes.
11. Dependency changes.

Any kernel, ABI, workflow-threshold, baseline, dependency, or Ring0 policy
reading fails closed.

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
12. No package loading or package execution.
13. No capability issuance, registry publication, trust assignment, or
    source merge authority.

Historical PASS results may be cited as context only. They cannot be
inherited as pointer-transition authority for another SHA.

If the accepted publication subject changes, evidence must be evaluated for
the new exact subject.

## Pointer PR Scope

The bounded pointer-transition PR may include only:

1. `PHASE21_POINTER_TRANSITION_DECISION.md`.
2. Updating `docs/roadmap/CURRENT_PHASE` from `CURRENT_PHASE=20` to
   `CURRENT_PHASE=21`.
3. Roadmap, status, and documentation index synchronization that describes
   Phase-21 as pointer-governance and possible First Bounded Implementation
   path only.
4. Exact-SHA remote PASS evidence for the accepted publication subject.
5. Explicit preservation of implementation procedure, source modification,
   code execution, process start, runtime state, package, capability,
   registry, trust, source merge, and Phase-19 runtime authority separation.

Any source implementation, runtime wiring, kernel or ABI change, baseline
change, workflow authority change, dependency change, or package, module,
workspace, plugin, capability, trust, Semantic CLI, AI Runtime, or agent
behavior change is out of scope and fails closed.

## Relationship To Later Phase-21 Work

This decision may open Phase-21 pointer governance only.

Later Phase-21 work requires separate reviewed authority before it may:

1. Define implementation scope.
2. Define runtime implementation procedure.
3. Modify source.
4. Implement code.
5. Execute code.
6. Start a process.
7. Create runtime state.
8. Install, load, or execute packages.
9. Load modules.
10. Create workspace runtime or real mounts.
11. Load or instantiate plugins.
12. Issue capabilities.
13. Assign trust.
14. Publish registry entries.
15. Authorize distribution execution.
16. Accept or merge source.
17. Grant Semantic CLI, AI Runtime, or agent authority.

No later Phase-21 authority may be inferred from this pointer transition.

## Publication Boundary

If this decision is merged, the landing SHA publishes the pointer decision
and the bounded phase pointer update. The landing SHA must not be read as
Phase-21 implementation scope, Phase-21 activation, runtime implementation
procedure, source modification authority, code implementation authority,
code execution authority, process start authority, runtime state authority,
package loading authority, package execution authority, capability issuance
authority, registry publication authority, trust assignment authority,
source merge authority, implementation authority, or general runtime
authority.

The accepted publication subject must be treated as the exact subject for
this pointer transition. Any later technical change, authority expansion,
implementation proposal, procedure proposal, or Phase-21 activation requires
a separate reviewed decision path.

## Decision Invariants

Every later RFC must preserve these Phase-21 pointer transition decision
invariants:

1. Phase-21 pointer transition consumes exact Phase-20 Closure Decision and
   Phase-21 Pointer Transition Candidate context only.
2. Phase-21 pointer transition may update `CURRENT_PHASE=20` to
   `CURRENT_PHASE=21` only through the accepted pointer PR.
3. `CURRENT_PHASE=21` does not define Phase-21 implementation scope.
4. `CURRENT_PHASE=21` does not define runtime implementation procedure.
5. `CURRENT_PHASE=21` does not modify source.
6. `CURRENT_PHASE=21` does not implement code.
7. `CURRENT_PHASE=21` does not execute code.
8. `CURRENT_PHASE=21` does not start a process.
9. `CURRENT_PHASE=21` does not create runtime state.
10. `CURRENT_PHASE=21` does not load packages.
11. `CURRENT_PHASE=21` does not execute packages.
12. `CURRENT_PHASE=21` does not issue capabilities.
13. `CURRENT_PHASE=21` does not publish registry entries.
14. `CURRENT_PHASE=21` does not assign trust.
15. `CURRENT_PHASE=21` does not grant source merge authority.
16. Phase-21 pointer transition does not broaden Phase-19 runtime
    authority.
17. Phase-21 pointer transition does not reopen Phase-20.
18. Phase-21 pointer transition does not reinterpret Phase-20 closure.
19. Later Phase-21 implementation scope requires a separate reviewed
    decision path, if ever authorized.
20. Ambiguity fails closed.

Violation of any invariant fails closed.

## Architecture Signature

**Prepared by:** Kenan AY
**Role:** AykenOS Architecture Steward
**Document type:** Phase-21 RFC
**Architecture status:** Draft RFC / pending architectural review
**Authority notice:** This signature identifies the architectural authorship
of this RFC. It grants no Phase-21 implementation scope authority, runtime
implementation procedure authority, source modification authority, code
implementation authority, code execution authority, process start authority,
general runtime authority, unbounded execution authority, runtime state
authority, implementation authority, implementation approval authority,
source merge authority, trust authority, evidence authority, acceptance
authority, proof authority, constitutional authority, registry authority,
distribution authority, publication authority, capability issuance
authority, package authority, deployment authority, module authority, plugin
authority, Semantic CLI authority, AI Runtime authority, agent authority, or
Ring0 authority.

## Decision Conclusion

If accepted with the bounded pointer update and required exact-SHA PASS
evidence, this decision transitions the formal phase pointer to:

```text
CURRENT_PHASE=21
```

The transition authorizes only Phase-21 pointer governance for a possible
First Bounded Implementation authority path.

Phase-21 implementation scope, runtime implementation procedure, source
modification, code implementation, code execution, process start, runtime
state creation, package loading, package execution, capability issuance,
registry publication, trust assignment, source merge authority, and all
later-phase implementation authority remain pending and unauthorized until
separately reviewed and decided.

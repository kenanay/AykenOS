# Phase-20 Pointer Transition Candidate

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
`PHASE19_CONSTITUTIONAL_CLOSURE_REVIEW.md`, and
`PHASE19_CLOSURE_DECISION.md`. In case of conflict, those documents prevail
unless this candidate is the narrower pre-transition candidate for a later
Phase-20 pointer transition decision.

**Status:** PHASE-20 POINTER TRANSITION CANDIDATE / PHASE-19 CLOSURE
RECORDED / NO POINTER TRANSITION / NO PHASE-20 ACTIVATION / NO RUNTIME
ACTIVATION / NO GENERAL RUNTIME AUTHORITY
**Candidate date:** 2026-06-28
**Candidate id:** `ayken.phase20.pointer_transition_candidate.v1`
**Candidate base main SHA:** `b89d38d3c2a24e6b722a08ad8b61811e794cdd9b`
**Phase-19 closure decision subject SHA:** `17de2131e01f743d8ca3ac4e431e9362f08dff39`
**Phase-19 closure decision publication SHA:** `b89d38d3c2a24e6b722a08ad8b61811e794cdd9b`
**Authority boundary:** Candidate documentation only; not a pointer
transition decision, not `CURRENT_PHASE=20`, not Phase-20 activation, not
runtime activation, not general runtime authority, not a new implementation
decision, not source acceptance, not execution authorization, not package,
module, workspace, plugin, capability, trust, Semantic CLI, AI Runtime,
agent, syscall, kernel ABI, workflow-threshold, baseline, dependency, or
Ring0 authority.

## Purpose

This document records the candidate boundary for a later Phase-20 pointer
transition decision.

It records that the Phase-19 Closure Decision has been published at exact
main subject:

```text
b89d38d3c2a24e6b722a08ad8b61811e794cdd9b
```

It does not change `docs/roadmap/CURRENT_PHASE`, does not start Phase-20,
does not activate runtime behavior, and does not authorize implementation.

## Core Rule

```text
pointer transition candidate != pointer transition decision
Phase-19 closure decision != Phase-20 activation
CURRENT_PHASE=19 remains until a separate decision changes it
Phase-20 planning != runtime activation or general runtime authority
```

This candidate makes the later pointer transition reviewable. It does not
perform that transition.

## Candidate Entry Record

| Entry item | Recorded result |
|---|---|
| Current canonical main subject | `b89d38d3c2a24e6b722a08ad8b61811e794cdd9b` |
| Current phase pointer before this candidate | `CURRENT_PHASE=19` |
| Phase-19 Closure Decision | `PHASE19_CLOSURE_DECISION.md` |
| Phase-19 Closure Decision subject | `17de2131e01f743d8ca3ac4e431e9362f08dff39` |
| Phase-19 Closure Decision publication subject | `b89d38d3c2a24e6b722a08ad8b61811e794cdd9b` |
| Phase-19 Closure Decision PR | PR #196 |
| PR #196 merge method | Normal maintainer squash merge; no admin bypass |
| PR #196 changed file | `PHASE19_CLOSURE_DECISION.md` |

This entry record is historical context for the candidate only. It does not
grant Phase-20 pointer authority.

## Candidate Phase-20 Scope

A later Phase-20 pointer transition decision may consider starting only a
bounded Phase-20 planning phase for the capability and registry ecosystem
that follows the Phase-18 Platform Constitution and the Phase-19 Runtime MVP
closure.

The candidate Phase-20 scope may include planning and specification for:

1. Capability registry shape and fail-closed registration rules.
2. Capability issuance preconditions and non-bypass constraints.
3. Package/module registry governance boundaries.
4. Distribution, publication, revocation, and quarantine policy inputs.
5. Cross-contract consistency between Phase-18 constitution records,
   Phase-19 runtime receipts, and later capability/registry records.
6. Evidence requirements for any later implementation slice.
7. Explicit separation from runtime activation, package execution, module
   loading, plugin loading, Semantic CLI authority, AI Runtime authority, and
   agent authority.

The only acceptable early Phase-20 posture remains planning and governance.
Any implementation, activation, execution, issuance, or registry behavior
requires a later reviewed implementation decision and exact-SHA evidence.

## Candidate Non-Goals

This candidate must not be read to include:

1. Updating `docs/roadmap/CURRENT_PHASE` to `CURRENT_PHASE=20`.
2. Activating Phase-20.
3. Runtime activation.
4. General runtime authority.
5. Package installation, loading, execution, scheduling, or publication.
6. Module loading.
7. Workspace creation, workspace runtime, or real mounts.
8. Plugin host, plugin loading, or plugin instantiation.
9. Capability token minting or capability issuance.
10. Trust assignment or trust issuer behavior.
11. Semantic CLI execution or verdict authority.
12. AI Runtime authority.
13. Agent behavior.
14. New syscalls.
15. Kernel ABI expansion.
16. Workflow-threshold, baseline, dependency, or Ring0 policy changes.

Unknown authority readings fail closed.

## Required Preconditions For A Later Pointer Transition Decision

A later Phase-20 pointer transition decision must fail closed unless all of
the following are true for the exact decision subject:

| ID | Requirement | Required result |
|---|---|---|
| P20-P1 | Phase-19 Closure Decision published | `PHASE19_CLOSURE_DECISION.md` exists on main |
| P20-P2 | Phase-19 closure publication subject recorded | `b89d38d3c2a24e6b722a08ad8b61811e794cdd9b` or later reviewed subject |
| P20-P3 | Current phase remains 19 before transition | `docs/roadmap/CURRENT_PHASE` still contains `CURRENT_PHASE=19` before the pointer PR changes it |
| P20-P4 | Phase-18 Platform Constitution remains authoritative | Phase-18 authority drift and terminology guards remain active |
| P20-P5 | Phase-19 closure does not imply runtime activation | no document treats Phase-19 closure as runtime authority |
| P20-P6 | Candidate accepted or narrowed | this candidate or a narrower successor is reviewed before pointer transition |
| P20-P7 | Pointer PR scope remains bounded | only pointer/status/roadmap/index documentation changes are included |
| P20-P8 | No source implementation included | no runtime, kernel, package, module, workspace, plugin, Semantic CLI, AI Runtime, or agent source changes |
| P20-P9 | Kernel ABI remains frozen | syscall IDs, syscall count, and ABI version remain unchanged |
| P20-P10 | Exact-SHA evidence passes | strict `ci-freeze`, Dev Loop, governance, spec, and boundary checks pass on the pointer decision subject |
| P20-P11 | Phase-20 scope remains planning/governance | no capability issuance, registry publication, execution, loading, mounting, or trust assignment is authorized |
| P20-P12 | Separate implementation authority remains required | later Phase-20 implementation requires its own decision, evidence package, acceptance review, and merge decision |

Missing, stale, ambiguous, or differently scoped evidence fails closed.

## Candidate Pointer PR Scope

A later pointer transition PR may propose only:

1. A separate `PHASE20_POINTER_TRANSITION_DECISION.md`.
2. Updating `docs/roadmap/CURRENT_PHASE` from `CURRENT_PHASE=19` to
   `CURRENT_PHASE=20`.
3. Updating roadmap, status, and documentation index text to reflect
   Phase-20 as a planning/governance phase only.
4. Recording exact-SHA remote PASS for the pointer transition subject.
5. Preserving explicit implementation, runtime activation, and Phase-20
   behavior separation.

It must not include runtime source, kernel source, ABI changes, baseline
changes, workflow authority changes, dependency changes, or any package,
module, workspace, plugin, capability, trust, Semantic CLI, AI Runtime, or
agent implementation.

## Explicit Non-Authorization

This candidate does not authorize:

1. Phase-20 pointer transition.
2. `CURRENT_PHASE=20`.
3. Phase-20 activation.
4. Runtime activation.
5. General runtime authority.
6. Implementation acceptance or merge authority.
7. Package installation, loading, execution, scheduling, or publication.
8. Module, plugin, workspace, capability, trust, Semantic CLI, AI Runtime,
   or agent authority.
9. New syscalls, kernel ABI expansion, workflow-threshold changes, baseline
   changes, dependency changes, or Ring0 policy.

Unknown authority readings fail closed.

## Publication Boundary

If this candidate is merged, the landing SHA publishes only this candidate
record. The landing SHA must not be read as a Phase-20 pointer transition,
Phase-20 activation, runtime activation, implementation authority, or
general runtime authority.

A later pointer transition decision must bind its own exact subject SHA and
post-merge evidence. Historical PASS results may be cited as context only;
they cannot be inherited as pointer authority for another SHA.

## Candidate Conclusion

This candidate records that Phase-19 closure has been published and makes a
later Phase-20 pointer transition decision reviewable.

It does not transition `CURRENT_PHASE`, does not activate Phase-20, does not
activate runtime behavior, and does not authorize implementation or general
runtime authority.

# Phase-19 Pointer Transition Candidate

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, `PHASE18_TRANSITION_DECISION.md`,
`PHASE18_ACTIVATION_DECISION.md`, the Phase-18 Platform Constitution
reference set, `AUTHORITY_DRIFT_GUARD.md`, `TERMINOLOGY_AUDIT.md`,
`PHASE19_RUNTIME_DECISION.md`, the Phase-19 Runtime RFC set, and
`PHASE19_POINTER_TRANSITION_DECISION.md`. In case of conflict, those documents
prevail.

**Status:** ACCEPTED POINTER TRANSITION CANDIDATE / SUPERSEDED BY POINTER TRANSITION DECISION / IMPLEMENTATION NOT AUTHORIZED
**Candidate date:** 2026-06-05
**Candidate id:** `ayken.phase19.pointer_transition.candidate.v1`
**Authority boundary:** Documentation candidate only; not `CURRENT_PHASE=19`,
not the pointer transition decision, not runtime implementation, not a package
installer, not a module loader, not workspace runtime, not real mount
authority, not plugin loading, not capability issuance, not trust assignment,
not Semantic CLI authority, not AI Runtime authority, not a syscall, not
kernel ABI expansion, not merge authority, and not closure authority.

## Purpose

This candidate answers only one question:

```text
If Phase-19 is later transitioned, which authority conditions must be true?
```

It did not perform that transition. The later transition is recorded in
`PHASE19_POINTER_TRANSITION_DECISION.md`.

This document is intentionally not named `PHASE19_ACTIVATION_DECISION.md`.
It is a pre-transition candidate record. The later exact-SHA pointer
transition remains separate from runtime implementation authority.

## Core Rule

```text
pointer transition candidate != pointer transition
runtime decision != runtime implementation
runtime RFC set != runtime implementation
runtime artifact != behavior source
```

The safe default remains no Phase-19 activation.

## Inert Runtime Artifact Invariant

Every Phase-19 Runtime MVP artifact must be inert.

For Phase-19 planning, inert means the artifact is a deterministic record,
reference, digest binding, or evidence output. It does not create behavior,
permission, scheduling, execution, loading, mounting, trust, capability, or
policy authority.

The invariant applies to the full proposed MVP chain:

| Artifact | Permitted meaning | Forbidden reading |
|---|---|---|
| Static input bundle | Digest-bound declarative input set | Parser, installer request, loader request, execution request, token request, workspace creation request |
| Validation receipt | Evidence that Phase-18 validation inputs were checked | Authorization, install permission, load permission, trust grant, capability grant, workspace grant |
| Workspace admission record | Deterministic record that an input subject is admissible for later review | Workspace creation, filesystem mount, namespace creation, handle, access grant |
| Runtime receipt | Digest-bound evidence output for the inert pipeline | Bearer token, capability token, workspace handle, plugin binding, execution right |

If a future Phase-19 document makes one of these artifacts active, executable,
loadable, mountable, transferable, or authority-bearing, the proposal must
fail closed.

## Candidate Transition Chain

The only acceptable transition chain is:

```text
Phase-17 official closure verified
  -> Phase-18 active as Platform Constitution only
  -> Phase-19 Runtime Decision Package accepted
  -> Phase-19 Runtime RFC set accepted
  -> Phase-19 Runtime Cross-Consistency Review accepted
  -> Phase-19 Pointer Transition Candidate accepted
  -> Phase-19 Activation Preconditions Review accepted
  -> exact-SHA pointer transition PR
  -> CURRENT_PHASE=19 only after separate review and remote PASS
```

This candidate occupies only the candidate step. It did not update the
pointer.

## Required Preconditions For Pointer Transition

The `CURRENT_PHASE=19` pointer transition must fail closed unless all of the
following are true on the exact candidate SHA for that transition:

| ID | Requirement | Required result |
|---|---|---|
| P19-P1 | Phase-17 closure remains verified | `phase17-official-closure` resolves to `416a5392afbe217e16d26a59e2e1716fdfa9c8f6` |
| P19-P2 | Current phase remains 18 before transition | `docs/roadmap/CURRENT_PHASE` contained `CURRENT_PHASE=18` before the pointer PR changed it |
| P19-P3 | Phase-18 remains Platform Constitution only | no runtime interpretation of Phase-18 activation exists |
| P19-P4 | Authority Drift Guard remains active | runtime, loader, issuer, workspace, plugin, trust, capability, AI, and Semantic authority drift is rejected |
| P19-P5 | Terminology Audit remains active | high-risk vocabulary remains non-authoritative |
| P19-P6 | Phase-19 Runtime Decision Package is accepted | `PHASE19_RUNTIME_DECISION.md` remains the planning boundary |
| P19-P7 | Phase-19 Runtime RFC set is accepted | all required files under `docs/specs/phase19-platform-runtime/` remain present |
| P19-P8 | Phase-19 Runtime Cross-Consistency Review is accepted | `CROSS_CONSISTENCY_REVIEW.md` remains PASS and non-authoritative |
| P19-P9 | This pointer transition candidate is accepted | this file remains docs-only and does not update `CURRENT_PHASE` |
| P19-P10 | Kernel ABI remains frozen | syscall IDs remain `1000-1011`, count remains `12`, ABI version remains `0x00010001` |
| P19-P11 | Pointer PR is docs-only | no kernel, userspace runtime, parser, loader, installer, issuer, plugin, Semantic CLI, AI Runtime, or registry code is included |
| P19-P12 | Inert artifact invariant is preserved | input bundle, validation receipt, admission record, and runtime receipt remain records only |
| P19-P13 | Exact-SHA CI passes | strict `ci-freeze` and Dev Loop pass on the pointer transition SHA |
| P19-P14 | Implementation remains separated | runtime implementation still requires a later implementation decision and evidence package |
| P19-P15 | Activation Preconditions Review is accepted | `PHASE19_ACTIVATION_PRECONDITIONS_REVIEW.md` reviews preconditions without changing `CURRENT_PHASE` |

Missing, stale, ambiguous, or partially satisfied preconditions fail closed.

## Pointer PR Scope

A pointer transition PR may only propose:

1. Updating `docs/roadmap/CURRENT_PHASE` from `CURRENT_PHASE=18` to
   `CURRENT_PHASE=19`.
2. Updating roadmap/status/index text to say Phase-19 is active only as the
   Platform Runtime MVP planning or admission/receipt phase defined by the
   accepted boundary.
3. Recording exact-SHA remote `ci-freeze` and Dev Loop PASS for that pointer
   transition subject.
4. Preserving explicit implementation separation.

It must not include:

1. Runtime source code.
2. Manifest parser implementation.
3. Package installer or package executor.
4. Module loader.
5. Workspace runtime, workspace creation, or real mounts.
6. Plugin host, plugin loading, or plugin instantiation.
7. Capability token minting or capability issuance.
8. Trust assignment or trust issuer.
9. Registry publication.
10. Semantic CLI authority.
11. AI Runtime authority.
12. Agent behavior.
13. New syscalls.
14. Kernel ABI expansion.
15. Ring0 policy.

## Exact-SHA Rule

This candidate does not contain the final transition SHA because it did not
execute the transition.

The exact-SHA authority for the pointer transition can only be the reviewed
subject SHA of the PR that actually changes
`docs/roadmap/CURRENT_PHASE`. If that subject changes after evidence is
recorded, the pointer transition evidence must be regenerated on the new
subject SHA.

No historical PASS can be inherited across SHAs.

## Denial Conditions

Pointer transition planning must be denied if any of the following are true:

1. This candidate is used to update `CURRENT_PHASE` directly.
2. A future pointer PR includes runtime source code.
3. Runtime RFC text is treated as parser authority.
4. Validation PASS is treated as install, load, execute, trust, capability,
   workspace, plugin, Semantic CLI, or AI authority.
5. Workspace admission is treated as workspace creation or mount authority.
6. A runtime receipt is treated as a token, handle, credential, or capability.
7. A package, module, workspace, plugin, trust, or capability document grants
   behavior without a later implementation decision.
8. New syscall or kernel ABI expansion appears.
9. Exact-SHA remote `ci-freeze` or Dev Loop evidence is missing.
10. Later-phase registry, Semantic CLI, AI Runtime, or agent behavior is
    pulled into Phase-19 by terminology or examples.

## Candidate Conclusion

This candidate made Phase-19 pointer transition reviewable.

It did not authorize implementation. It preserves the Phase-18 rule that
Constitution is not Runtime and adds the Phase-19 rule that Runtime MVP
artifacts must remain inert until a separate implementation authority exists.

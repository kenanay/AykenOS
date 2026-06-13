# Phase-19 Platform Runtime Decision Package

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, `PHASE18_TRANSITION_DECISION.md`,
`PHASE18_ACTIVATION_DECISION.md`, the Phase-18 Platform Constitution
reference set, `AUTHORITY_DRIFT_GUARD.md`, `TERMINOLOGY_AUDIT.md`, and
`PHASE19_POINTER_TRANSITION_DECISION.md`. In case of conflict, those
documents prevail.

**Status:** DECISION PACKAGE / PHASE-19 ACTIVE AS PLANNING BOUNDARY / IMPLEMENTATION NOT AUTHORIZED
**Decision package date:** 2026-06-05
**Decision id:** `ayken.phase19.platform_runtime_decision.v1`
**Authority boundary:** Runtime MVP decision boundary only; not runtime
implementation, package installer, module loader, workspace runtime, real
mount authority, plugin host, capability issuer, trust issuer, Semantic CLI
authority, AI Runtime authority, syscall, kernel ABI expansion, merge
authority, or closure authority.

## Decision

Phase-19 may be proposed only as **Platform Runtime MVP**.

This package did not activate Phase-19 by itself. Phase-19 is now active only
through the separate exact-SHA `PHASE19_POINTER_TRANSITION_DECISION.md`
pointer decision.

This package does not authorize runtime source code. It authorizes only the
Runtime MVP planning boundary that preserves the Phase-18 Platform
Constitution boundaries.

## Core Rule

```text
Runtime decision != runtime implementation
```

The existence of this decision package must not be read as permission to
implement a manifest parser, package installer, module loader, workspace
runtime, plugin host, capability issuer, trust issuer, Semantic CLI execution
path, AI Runtime, registry service, or agent system.

The safe default remains no runtime.

## Phase-19 Runtime Definition

For Phase-19 planning, "Platform Runtime MVP" means a narrowly scoped
userspace control surface that may later evaluate Phase-18 constitutional
inputs and emit deterministic evidence records.

The planned runtime is:

1. Userspace only.
2. Built above the frozen syscall v2 ABI.
3. Bound to Phase-18 manifest, package, trust, capability, workspace, plugin,
   and Platform ABI validation contracts.
4. Evidence-producing and fail-closed.
5. Non-authoritative unless a later accepted implementation RFC grants a
   specific bounded behavior.

The planned runtime is not:

1. A kernel feature.
2. A new syscall surface.
3. A package manager.
4. A registry.
5. A module loader.
6. A plugin loader.
7. A real filesystem mount engine.
8. A capability token issuer.
9. A trust issuer.
10. Semantic CLI authority.
11. AI Runtime authority.
12. Agent authority.

## Candidate MVP Shape

The earliest safe Phase-19 MVP candidate is a deterministic admission and
receipt pipeline:

```text
static input bundle
  -> manifest/package shape validation
  -> Platform ABI validation decision
  -> workspace admission record
  -> runtime receipt
```

This candidate shape is not implementation authority. It is a planning
boundary for future RFCs.

The MVP candidate must not install, load, mount, execute, issue, trust,
publish, or schedule anything.

## First Running Thing

If a later Phase-19 implementation RFC is accepted, the first permitted
running artifact must be a minimal userspace admission harness.

That harness may only prove the following bounded behavior:

1. It consumes a static, test-owned input bundle.
2. It checks only the bounded static test-owned bundle shape and referenced
   Phase-18 declarative contract metadata required for admission/receipt
   evidence.
3. It emits a deterministic receipt.
4. It fails closed on unknown fields, missing links, stale hashes, invalid
   dependency declarations, authority drift, or ambiguous terms.

It must not execute the described package or module.

## Runtime RFC Set

Before any Phase-19 implementation PR, the following runtime RFC set must
remain accepted:

1. `RUNTIME_LIFECYCLE_SPECIFICATION.md`
2. `RUNTIME_INPUT_BUNDLE_SPECIFICATION.md`
3. `PLATFORM_VALIDATION_INTEGRATION_SPECIFICATION.md`
4. `WORKSPACE_ADMISSION_RUNTIME_SPECIFICATION.md`
5. `RUNTIME_RECEIPT_SPECIFICATION.md`
6. `RUNTIME_EVIDENCE_PLAN.md`
7. `RUNTIME_EVIDENCE_MATRIX.md`
8. `RUNTIME_NON_GOALS_AND_DENIALS.md`

The accepted RFC set lives under `docs/specs/phase19-platform-runtime/`. The
existence of that directory does not authorize implementation.

After the RFC set was accepted, a Phase-19 runtime cross-consistency review
was accepted before a pointer-transition discussion could use the RFC set as
planning input. That review is still not implementation authority.

After the cross-consistency review, `PHASE19_POINTER_TRANSITION_CANDIDATE.md`
defined the conditions for a later exact-SHA pointer transition. That
candidate did not update `CURRENT_PHASE` and did not authorize runtime
implementation.

After the pointer transition candidate,
`PHASE19_ACTIVATION_PRECONDITIONS_REVIEW.md` reviewed whether the documented
preconditions were complete enough for a later pointer transition discussion.
That review is still not implementation authority.

After that review, `PHASE19_POINTER_TRANSITION_DECISION.md` transitions
`docs/roadmap/CURRENT_PHASE` to `19` as planning/admission/receipt authority
only. That pointer transition is still not runtime implementation authority.

After the pointer transition,
`PHASE19_RUNTIME_IMPLEMENTATION_DECISION_CANDIDATE.md` records the candidate
boundary for a later implementation decision. That candidate does not
authorize runtime source code and does not replace a future exact-SHA
implementation decision.

`PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE_CANDIDATE.md` records the
candidate shape for that later decision package. It maps the minimum behavior,
evidence-row closure, exact-SHA acceptance preconditions, and fail-closed
conditions that a separate implementation decision package must satisfy. It
does not authorize runtime source code or implementation.

`PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE_DRAFT.md` records the draft
shape for that later decision package. It narrows the minimum behavior,
evidence binding, exact-SHA preconditions, and fail-closed denials further,
but it is still not the implementation decision package and does not authorize
runtime source code or implementation.

`PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE.md` records the exact-SHA
implementation decision package boundary. It is not an implementation PR,
evidence package, remote PASS result, acceptance review, or runtime source
code authority.

`PHASE19_RUNTIME_IMPLEMENTATION_EVIDENCE_PACKAGE.md` records evidence for the
draft PR #181 bounded admission/receipt implementation subject. It is not
acceptance review, merge authority, runtime activation, or general runtime
authority.

`PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW.md` records the first
acceptance review for that evidence package. Acceptance is not granted, PR
#181 remains draft, and additional transcript evidence is required before
acceptance can be reconsidered.

`MODULE_LOADING_MODEL.md`, `PLUGIN_INSTANTIATION_MODEL.md`, package
execution, real workspace mounts, capability issuance, trust assignment,
Semantic CLI authority, AI Runtime authority, and agent behavior are not part
of the initial Phase-19 MVP decision.

If a future document uses the word `loading`, it must first prove that it is a
non-loader admission model or move to a later phase decision.

## Phase-19 Does Not Authorize

This decision package must not authorize:

1. Runtime implementation.
2. Package installation.
3. Package execution.
4. Module loading.
5. Plugin loading or plugin instantiation.
6. Workspace creation.
7. Real filesystem mounts.
8. Capability token minting.
9. Capability issuance.
10. Trust assignment.
11. Registry publication.
12. Semantic CLI execution authority.
13. AI Runtime authority.
14. Agent systems.
15. New syscalls.
16. Kernel ABI expansion.
17. Ring0 policy.
18. Kernel loader behavior.
19. Observability-as-authority.

Any such work requires a separate reviewed decision, RFC set, evidence plan,
evidence matrix, and acceptance boundary.

## Preconditions For Phase-19 Activation Consideration

Phase-19 pointer transition cannot be considered unless all of the following
are true:

| ID | Precondition | Required result |
|---|---|---|
| P19-A1 | Phase-17 closure remains verified | `phase17-official-closure` resolves to `416a5392afbe217e16d26a59e2e1716fdfa9c8f6` |
| P19-A2 | Phase-18 remains accepted as Platform Constitution only | Phase-18 activation is not interpreted as runtime implementation |
| P19-A3 | Phase-18 Authority Drift Guard remains active | `AUTHORITY_DRIFT_GUARD.md` rejects runtime drift |
| P19-A4 | Phase-18 Terminology Audit remains active | high-risk words remain non-authoritative |
| P19-A5 | Runtime RFC set exists | all required Phase-19 runtime RFCs are accepted |
| P19-A6 | Runtime non-goals are explicit | installer, loader, execution, issuer, trust, AI, Semantic CLI, and agent authority are denied |
| P19-A7 | Kernel ABI remains frozen | syscall IDs remain `1000-1011`, count remains `12`, ABI version remains `0x00010001` |
| P19-A8 | Exact-SHA CI passes | strict `ci-freeze` and Dev Loop pass on the pointer transition candidate SHA |
| P19-A9 | Implementation is separated | pointer transition does not include runtime source code |
| P19-A10 | Evidence plan and matrix are accepted | runtime evidence paths, artifact evidence rows, receipts, negative cases, and fail-closed behavior are defined |
| P19-A11 | Runtime RFC cross-review is accepted | Phase-19 runtime RFC set has a reviewed cross-consistency record |
| P19-A12 | Pointer transition candidate is accepted | `PHASE19_POINTER_TRANSITION_CANDIDATE.md` defines exact-SHA transition conditions without changing `CURRENT_PHASE` |
| P19-A13 | Activation preconditions review is accepted | `PHASE19_ACTIVATION_PRECONDITIONS_REVIEW.md` reviews preconditions without activating Phase-19 |

Missing, stale, ambiguous, or partially satisfied preconditions fail closed.

## Runtime RFC Acceptance Criteria

Future Phase-19 runtime RFCs must define:

1. Exact input bundle boundaries.
2. Deterministic state transitions.
3. Receipt schema and digest binding.
4. Negative cases for unknown, stale, contradictory, or authority-granting
   inputs.
5. Explicit relationship to Phase-18 Platform ABI Validation Gate.
6. Local and remote evidence paths.
7. Performance measurement scope if runtime hot paths are touched.
8. Production default behavior.
9. Removal or closure conditions for validation-only paths.
10. Non-authority wording consistent with `TERMINOLOGY_AUDIT.md`.

An RFC that grants runtime behavior without these criteria must be rejected.

## Fail-Closed Denial Conditions

Phase-19 planning must be denied if any of the following are true:

1. This package is used to update `CURRENT_PHASE` directly.
2. Runtime source code is bundled with this decision package.
3. A manifest schema is treated as an active parser.
4. Package metadata is treated as install or execute authority.
5. Workspace admission is treated as a real mount.
6. Plugin compatibility is treated as loading.
7. Trust classification is treated as capability grant.
8. A validation receipt is treated as a bearer token.
9. Semantic CLI output is treated as runtime authority.
10. AI output is treated as runtime authority.
11. New syscall or kernel ABI expansion is present.
12. Required Phase-19 runtime RFCs are missing.
13. Required local or remote checks fail.
14. Exact-SHA evidence is missing.

The safe default is no runtime implementation.

## Relationship To Later Phases

Phase-19 is limited to Platform Runtime MVP planning, validation integration,
admission records, and deterministic receipts.

The following remain later-phase work:

1. Phase-20 Capability Ecosystem / Module Registry.
2. Phase-21 Semantic CLI Integration.
3. Phase-22 AI Runtime Foundation.
4. Phase-23+ Agent Systems.

Those later phases must not be pulled into Phase-19 by terminology, examples,
or convenience.

## Decision Package Conclusion

This package defines the safe Phase-19 Runtime MVP decision boundary.

It does not authorize implementation. It preserves the rule that Phase-18
remains the accepted Platform Constitution reference set and that
`CURRENT_PHASE=19` is only a planning/admission/receipt pointer.

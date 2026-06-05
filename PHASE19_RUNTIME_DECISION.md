# Phase-19 Platform Runtime Decision Package

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, `PHASE18_TRANSITION_DECISION.md`,
`PHASE18_ACTIVATION_DECISION.md`, the Phase-18 Platform Constitution
reference set, `AUTHORITY_DRIFT_GUARD.md`, and `TERMINOLOGY_AUDIT.md`. In
case of conflict, those documents prevail until a separate reviewed Phase-19
activation and closure authority exists.

**Status:** DECISION PACKAGE / PHASE-19 NOT ACTIVE / IMPLEMENTATION NOT AUTHORIZED
**Decision package date:** 2026-06-05
**Decision id:** `ayken.phase19.platform_runtime_decision.v1`
**Authority boundary:** Documentation decision only; not a phase pointer
transition, runtime implementation, package installer, module loader,
workspace runtime, real mount authority, plugin host, capability issuer,
trust issuer, Semantic CLI authority, AI Runtime authority, syscall, kernel
ABI expansion, merge authority, or closure authority.

## Decision

Phase-19 may be proposed only as **Platform Runtime MVP**.

This package does not activate Phase-19. `CURRENT_PHASE` remains `18` until a
separate exact-SHA pointer transition is reviewed and accepted.

This package does not authorize runtime source code. It authorizes only the
preparation of a future Phase-19 runtime RFC set that preserves the Phase-18
Platform Constitution boundaries.

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
2. It checks the bundle against accepted Phase-18 declarative contracts.
3. It emits a deterministic receipt.
4. It fails closed on unknown fields, missing links, stale hashes, invalid
   dependency declarations, authority drift, or ambiguous terms.

It must not execute the described package or module.

## Future Runtime RFC Set

Before any Phase-19 implementation PR, the following runtime RFC set must be
prepared and accepted:

1. `RUNTIME_LIFECYCLE_SPECIFICATION.md`
2. `RUNTIME_INPUT_BUNDLE_SPECIFICATION.md`
3. `PLATFORM_VALIDATION_INTEGRATION_SPECIFICATION.md`
4. `WORKSPACE_ADMISSION_RUNTIME_SPECIFICATION.md`
5. `RUNTIME_RECEIPT_SPECIFICATION.md`
6. `RUNTIME_EVIDENCE_PLAN.md`
7. `RUNTIME_NON_GOALS_AND_DENIALS.md`

`MODULE_LOADING_MODEL.md`, `PLUGIN_INSTANTIATION_MODEL.md`, package
execution, real workspace mounts, capability issuance, trust assignment,
Semantic CLI authority, AI Runtime authority, and agent behavior are not part
of the initial Phase-19 MVP decision.

If a future document uses the word `loading`, it must first prove that it is a
non-loader admission model or move to a later phase decision.

## Phase-19 Does Not Authorize

This decision package must not authorize:

1. `CURRENT_PHASE=19`.
2. Runtime implementation.
3. Package installation.
4. Package execution.
5. Module loading.
6. Plugin loading or plugin instantiation.
7. Workspace creation.
8. Real filesystem mounts.
9. Capability token minting.
10. Capability issuance.
11. Trust assignment.
12. Registry publication.
13. Semantic CLI execution authority.
14. AI Runtime authority.
15. Agent systems.
16. New syscalls.
17. Kernel ABI expansion.
18. Ring0 policy.
19. Kernel loader behavior.
20. Observability-as-authority.

Any such work requires a separate reviewed decision, RFC set, evidence plan,
and acceptance boundary.

## Preconditions For Phase-19 Activation Consideration

Phase-19 pointer transition cannot be considered unless all of the following
are true:

| ID | Precondition | Required result |
|---|---|---|
| P19-A1 | Phase-17 closure remains verified | `phase17-official-closure` resolves to `416a5392afbe217e16d26a59e2e1716fdfa9c8f6` |
| P19-A2 | Phase-18 remains active as Platform Constitution only | `CURRENT_PHASE=18` remains true before the separate pointer transition |
| P19-A3 | Phase-18 Authority Drift Guard remains active | `AUTHORITY_DRIFT_GUARD.md` rejects runtime drift |
| P19-A4 | Phase-18 Terminology Audit remains active | high-risk words remain non-authoritative |
| P19-A5 | Runtime RFC set exists | all required Phase-19 runtime RFCs are accepted |
| P19-A6 | Runtime non-goals are explicit | installer, loader, execution, issuer, trust, AI, Semantic CLI, and agent authority are denied |
| P19-A7 | Kernel ABI remains frozen | syscall IDs remain `1000-1011`, count remains `12`, ABI version remains `0x00010001` |
| P19-A8 | Exact-SHA CI passes | strict `ci-freeze` and Dev Loop pass on the candidate SHA |
| P19-A9 | Implementation is separated | pointer transition does not include runtime source code |
| P19-A10 | Evidence plan is accepted | runtime evidence paths, receipts, negative cases, and fail-closed behavior are defined |

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

The safe default is no Phase-19 activation.

## Relationship To Later Phases

Phase-19 is limited to Platform Runtime MVP planning and, after a separate
accepted activation, a bounded runtime MVP.

The following remain later-phase work:

1. Phase-20 Capability Ecosystem / Module Registry.
2. Phase-21 Semantic CLI Integration.
3. Phase-22 AI Runtime Foundation.
4. Phase-23+ Agent Systems.

Those later phases must not be pulled into Phase-19 by terminology, examples,
or convenience.

## Decision Package Conclusion

This package defines the safe Phase-19 Runtime MVP decision boundary.

It does not activate Phase-19. It does not authorize implementation. It
preserves the rule that Phase-18 remains the active Platform Constitution
until a separate reviewed pointer transition changes `CURRENT_PHASE`.

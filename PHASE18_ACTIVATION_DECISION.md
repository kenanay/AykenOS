# Phase-18 Activation Decision Package

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, `PHASE18_TRANSITION_DECISION.md`, and the Phase-18
Platform Constitution RFC set. In case of conflict, those documents prevail.

**Status:** ACTIVATION DECISION PACKAGE CANDIDATE / PHASE-18 NOT ACTIVATED
**Decision package date:** 2026-06-05
**Authority basis:** `phase17-official-closure` at
`416a5392afbe217e16d26a59e2e1716fdfa9c8f6`, the reviewed Phase-18 Platform
Constitution RFC set, and
`docs/specs/phase18-platform-constitution/CROSS_CONSISTENCY_REVIEW.md`.
**Attribution:** Kenan AY - informational documentation metadata only; not
runtime, merge, execution, install, trust, capability, workspace, plugin,
Semantic CLI, AI Runtime, syscall, kernel ABI, or phase pointer authority.

## Decision Candidate

Phase-18 may be activated only as **Platform Constitution**.

Activation, if later accepted through a separate exact-SHA pointer transition,
means the Phase-18 Platform Constitution contracts become the active roadmap
authority for module, package, capability, workspace, trust classification,
plugin boundary, and Platform ABI validation design.

This document does not activate Phase-18 and does not change
`docs/roadmap/CURRENT_PHASE`.

## Core Rule

```text
Constitution != Runtime
```

A manifest schema is not a manifest parser. A capability contract is not a
capability engine. A workspace lifecycle specification is not a workspace
runtime. A plugin boundary contract is not a plugin loader. A Platform ABI
Validation Gate specification is not a runtime validator.

The Platform Constitution may become active before any Platform Runtime exists.
That activation must not be interpreted as permission to implement, load,
install, enable, execute, mount, issue, trust, or run anything.

## Activation Scope

If Phase-18 is later activated, activation grants only:

1. Active roadmap ownership for Phase-18 as Platform Constitution.
2. Authority to maintain the accepted Platform Constitution reference set.
3. Authority to prepare non-runtime examples and review checklists that do not
   grant access, execute code, or create loader behavior.
4. Authority to prepare later implementation RFCs for Phase-19 or another
   explicitly reviewed phase.

Activation does not grant runtime implementation authority.

## Activation Does Not Authorize Implementation

Phase-18 activation must not authorize:

1. Runtime implementation.
2. Package installation.
3. Package execution.
4. Workspace creation.
5. Real filesystem mounts or logical mount runtime binding.
6. Plugin loading.
7. Plugin autoload.
8. Capability issuance.
9. Capability token minting.
10. Trust assignment.
11. Registry publication.
12. Runtime loader creation.
13. Semantic CLI execution authority.
14. AI Runtime authority.
15. New syscalls.
16. Kernel ABI expansion.
17. Ring0 policy.

Any such work requires a separate reviewed phase decision, implementation RFC,
evidence plan, and acceptance boundary.

## Activation Preconditions

The following preconditions are mandatory before any `CURRENT_PHASE=18`
pointer transition may be considered:

| ID | Precondition | Required result |
|---|---|---|
| A1 | Phase-17 official closure remains verified | `phase17-official-closure` resolves to `416a5392afbe217e16d26a59e2e1716fdfa9c8f6` |
| A2 | Phase-18 transition decision exists | `PHASE18_TRANSITION_DECISION.md` remains accepted as Platform Constitution direction |
| A3 | Platform Constitution RFC set exists | Module Manifest, Package Metadata, Trust Classification, Capability Contract, Workspace Lifecycle, Plugin Boundary, and Platform ABI Validation Gate are present |
| A4 | Cross-consistency review accepted | `CROSS_CONSISTENCY_REVIEW.md` records PASS or is superseded by a newer accepted review |
| A5 | Constitution/runtime separation explicit | This package preserves `Constitution != Runtime` |
| A6 | Kernel ABI frozen | Syscall IDs remain `1000-1011`, syscall count remains `12`, ABI version remains `0x00010001` |
| A7 | Kernel expansion absent | No new syscall, Ring0 policy, kernel loader, or kernel plugin system is bundled |
| A8 | Authority separation intact | Trust, validation, manifest, package, workspace, plugin, and capability records do not grant each other's authority |
| A9 | Local validation clean | Spec purity, naming, governance, drift activation, and diff checks pass for the candidate SHA |
| A10 | Remote validation clean | Required GitHub checks, including strict `ci-freeze` and Dev Loop, pass on the exact activation candidate SHA |
| A11 | Activation PR exact-SHA scoped | Activation evidence references the exact commit being considered |
| A12 | Pointer transition separate | `CURRENT_PHASE=18` is performed only in a separate pointer-transition change after this package is accepted |

Missing, stale, ambiguous, or partially satisfied preconditions fail closed.

## Required Pointer Transition Shape

The later pointer-transition change, if proposed, must be narrow:

1. Update `docs/roadmap/CURRENT_PHASE` from `17` to `18`.
2. Update status surfaces to say Phase-18 is active only as Platform
   Constitution.
3. Reference this activation decision package and the accepted exact-SHA
   validation results.
4. Avoid runtime implementation, source changes, package loader code,
   workspace runtime code, plugin host code, capability token issuer code,
   trust issuer code, Semantic CLI authority, AI Runtime authority, syscall
   changes, and kernel ABI changes.

If the pointer transition includes implementation work, it must be rejected.

## Fail-Closed Denial Conditions

Phase-18 activation must be denied if any of the following are true:

1. `CURRENT_PHASE` is changed in the same PR that first introduces this
   decision package.
2. Cross-consistency review is missing, stale, or contradicted.
3. Kernel ABI expansion is present or implied.
4. A new syscall is present or implied.
5. Ring0 policy is present or implied.
6. Any document treats trust as capability.
7. Any document treats validation PASS as authority grant.
8. Any document treats plugin compatibility as loading.
9. Any document treats workspace admission as runtime mount creation.
10. Any document treats capability decision records or receipts as bearer
    tokens.
11. Any document treats Semantic CLI or AI Runtime output as execution
    authority.
12. Runtime implementation is bundled with activation.
13. Required local or remote checks fail.
14. Activation evidence is not exact-SHA scoped.

The safe default is no activation.

## Activation Evidence Package

The accepted activation evidence package must include:

1. Exact activation candidate SHA.
2. Local validation summary.
3. Remote `ci-freeze` run id and conclusion.
4. Remote Dev Loop run id and conclusion.
5. Governance/spec/naming/evidence boundary run conclusions.
6. Confirmation that `CURRENT_PHASE` remained `17` until the later pointer
   transition change.
7. Confirmation that no runtime implementation or kernel ABI change was
   included.

CI PASS is not activation by itself. It is only an input to the reviewed
activation decision.

## Relationship To Phase-19

Phase-19 is the earliest planned phase for Platform Runtime MVP work.

Phase-18 activation must not pull Phase-19 work forward. Package installers,
runtime loaders, workspace runtime bindings, plugin host execution, capability
token issuance, trust issuer implementation, registry services, Semantic CLI
integration, and AI Runtime foundations remain outside Phase-18 unless a
separate reviewed decision changes the roadmap.

## Decision Package Conclusion

This package is ready for review as an activation decision candidate.

It does not activate Phase-18. The next safe action after accepting this
package is a separate, narrow `CURRENT_PHASE=18` pointer-transition proposal
that preserves Platform Constitution scope and excludes runtime
implementation.

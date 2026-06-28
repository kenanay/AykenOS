# Phase-20 Capability Lifecycle

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
`PHASE20_CAPABILITY_IDENTITY.md`, and
`PHASE20_CAPABILITY_MANIFEST_SCHEMA.md`. In case of conflict, those
documents prevail unless this lifecycle RFC is the narrower Phase-20
capability lifecycle record for the exact planning scope identified below.

**Status:** PHASE-20 CAPABILITY LIFECYCLE RFC / LIFECYCLE MODEL ONLY / NO
STATE CHANGE AUTHORITY FOR ANY SPECIFIC CAPABILITY / NO REGISTRY PUBLICATION
/ NO IMPLEMENTATION AUTHORITY / NO RUNTIME ACTIVATION / NO GENERAL RUNTIME
AUTHORITY / NO CAPABILITY ISSUANCE / NO TRUST ASSIGNMENT / NO DISTRIBUTION
AUTHORITY
**Lifecycle date:** 2026-06-28
**Lifecycle id:** `ayken.phase20.capability_lifecycle.v1`
**Lifecycle base main SHA:** `b4eaea2ab75e72afcb54a0e928240f7626eaf6ab`
**Current phase pointer:** `CURRENT_PHASE=20`
**Authority boundary:** Capability lifecycle model only; not lifecycle state
change authority for any specific capability, not registry authority, not
registry publication, not trust authority, not distribution authority, not
evidence acceptance, not implementation authority, not runtime activation,
not general runtime authority, not package installation, not package
execution, not module loading, not workspace runtime, not plugin loading,
not capability token minting, not capability issuance, not trust assignment,
not Semantic CLI authority, not AI Runtime authority, not agent authority,
not syscall expansion, not kernel ABI expansion, not workflow-threshold,
baseline, dependency, or Ring0 authority.

## Purpose

This document defines the Phase-20 lifecycle model for AykenOS capability
records.

It answers one question:

```text
How is the governance lifecycle of a capability modeled?
```

It does not answer:

```text
How is a lifecycle transition executed by tooling?
How is a capability admitted to a registry?
How is a capability trusted, distributed, implemented, activated, or run?
```

Those questions belong to later Phase-20 RFCs and implementation decisions.

## Core Rule

```text
capability lifecycle != runtime lifecycle
capability lifecycle state != capability issuance
accepted lifecycle state != implementation acceptance
accepted lifecycle state != registry publication
deprecated lifecycle state != revocation by itself
revoked lifecycle state != deletion from history
quarantine lifecycle state != proof of fault
lifecycle model != state change authority
```

Lifecycle makes capability governance state reviewable. It does not make a
capability active, trusted, published, accepted for source implementation,
executable, loadable, installable, runnable, or authority-bearing.

Unknown authority readings fail closed.

## Lifecycle Mission

The mission of capability lifecycle is to give each capability record a
stable, reviewable, fail-closed governance state.

Lifecycle exists so later RFCs can safely reason about whether a capability
record is a draft, under review, accepted as a governance record,
deprecated, quarantined, revoked, or retired.

The lifecycle model itself grants no state change authority for any specific
capability record.

It supports later:

1. Registry admission review.
2. Trust input association.
3. Distribution policy review.
4. Revocation and quarantine policy.
5. Evidence package construction.
6. Acceptance workflow.
7. Bounded implementation decisions.
8. Historical audit.

Each later use requires its own reviewed RFC or decision path.

## Lifecycle Definition

A capability lifecycle is the governance state model for an inert capability
record identified by `PHASE20_CAPABILITY_IDENTITY.md` and described by
`PHASE20_CAPABILITY_MANIFEST_SCHEMA.md`.

Lifecycle state is a governance label attached to a capability record for
review and audit purposes.

Lifecycle state is not:

1. A runtime state.
2. An execution state.
3. A package state.
4. A module state.
5. A plugin state.
6. A workspace state.
7. A registry publication state.
8. A trust assignment.
9. A capability token state.
10. Semantic CLI, AI Runtime, or agent authority.

## Lifecycle Principles

Every capability lifecycle rule must preserve these principles:

1. **Governance-only:** lifecycle state describes review posture; it does
   not activate behavior.
2. **Identity-bound:** lifecycle state belongs to an exact capability
   identity and must not silently transfer across identity changes.
3. **Manifest-aware:** lifecycle state must preserve manifest validation
   boundaries and subject/scope separation.
4. **Reviewable:** lifecycle transitions must be explicit enough for
   governance review.
5. **Fail-closed:** missing, ambiguous, stale, inherited, or differently
   scoped state fails closed.
6. **Historical:** lifecycle history must remain auditable even when a
   record is revoked or retired.
7. **No authority inheritance:** lifecycle state must not carry over
   evidence, trust, registry publication, implementation authority, or
   runtime authority.
8. **Transition-gated:** transition rules require later accepted workflow
   authority before they can be applied to a specific record.
9. **Quarantine-safe:** ambiguous, colliding, disputed, or unsafe readings
   must move toward quarantine or denial, not authority.

## Lifecycle State Vocabulary

The initial lifecycle vocabulary is conceptual:

| State | Meaning | Authority result |
|---|---|---|
| `concept` | Capability idea is being described before complete record form | No identity completion |
| `draft` | Capability record is being prepared for review | No acceptance |
| `review` | Capability record is submitted for governance review | No merge or publication |
| `accepted` | Capability record is accepted as a governance record | No implementation authority |
| `deprecated` | Capability record should not be used for new planning without review | No revocation by itself |
| `quarantined` | Capability record is held due to ambiguity, collision, dispute, or safety concern | No authority |
| `revoked` | Capability record is no longer accepted for future authority paths | No deletion from history |
| `retired` | Capability record is historical and closed for active planning | No resurrection by implication |

These states are governance concepts. They do not define a storage schema,
registry entry, parser behavior, command, runtime handle, package status,
module status, plugin status, workspace status, or trust status.

## State Semantics

### `concept`

`concept` is the earliest governance posture for a capability idea.

It may describe intent, subject, scope, or architectural motivation before
complete identity or manifest material exists.

`concept` does not imply identity completion, manifest validity, review
submission, acceptance, registry admission, trust, distribution,
implementation, or runtime activation.

### `draft`

`draft` means a capability record is being prepared for review.

A draft may have identity and manifest material, but that material remains
unaccepted unless later review accepts it.

`draft` does not imply registry admission, evidence acceptance, trust,
implementation authority, or runtime activation.

### `review`

`review` means a capability record has been submitted for governance review.

Review state is not approval. It only identifies that a record is being
evaluated under the applicable governance path.

`review` does not imply merge authority, acceptance, registry publication,
trust assignment, source acceptance, implementation authority, or runtime
activation.

### `accepted`

`accepted` means a capability record is accepted as a governance record.

Accepted governance state does not mean the capability is implemented,
issued, trusted, distributed, published, executable, loadable, installable,
or active.

An accepted capability record may become an input to later registry,
evidence, acceptance, or implementation RFCs. It does not authorize those
paths by itself.

### `deprecated`

`deprecated` means a capability record should not be used for new planning
without explicit review.

Deprecation is advisory governance state. It is not deletion, revocation,
quarantine, or runtime disablement by itself.

Deprecation must not be used to transfer authority to a replacement record.

### `quarantined`

`quarantined` means a capability record is held due to ambiguity, collision,
dispute, evidence concern, trust concern, scope concern, or safety concern.

Quarantine is fail-closed. It does not prove fault and does not grant
authority to any competing record.

A quarantined record must not be treated as accepted, published, trusted,
implemented, distributed, executable, or active.

### `revoked`

`revoked` means a capability record is no longer accepted for future
authority paths.

Revocation does not erase history. Historical evidence, review notes, and
decision records remain auditable.

Revocation must not silently revoke another identity, transfer authority,
or publish an alternative record.

### `retired`

`retired` means a capability record is historical and closed for active
planning.

Retirement does not imply deletion, runtime disablement, registry removal,
or replacement.

A retired record cannot become active again by implication. Any renewed use
requires a later reviewed path.

## Conceptual Transition Model

The initial lifecycle transition model is:

```text
concept
  -> draft
  -> review
  -> accepted
  -> deprecated
  -> retired
```

Additional fail-closed transitions may occur:

```text
draft -> quarantined
review -> draft
review -> quarantined
accepted -> quarantined
accepted -> revoked
deprecated -> quarantined
deprecated -> revoked
quarantined -> review
quarantined -> revoked
quarantined -> retired
revoked -> retired
```

This transition model is conceptual. It does not execute transitions for any
specific capability record.

Later acceptance workflow and evidence RFCs must define how a specific
transition is proposed, reviewed, recorded, accepted, rejected, merged, and
verified.

## Transition Classification

Lifecycle transitions are classified conceptually:

| Transition class | Examples | Authority result |
|---|---|---|
| Normal progression | `concept -> draft`, `draft -> review` | No acceptance |
| Review outcome | `review -> accepted`, `review -> draft` | No implementation authority |
| Safety or ambiguity | `draft -> quarantined`, `review -> quarantined`, `accepted -> quarantined` | No authority |
| Governance maintenance | `accepted -> deprecated`, `deprecated -> revoked` | No runtime action |
| Historical closure | `revoked -> retired`, `quarantined -> retired`, `deprecated -> retired` | No replacement authority |

This classification is descriptive only. It does not authorize transition
execution for any specific capability record.

## Terminal Lifecycle States

`revoked` and `retired` are terminal governance states by default.

No forward transition, reactivation, replacement, publication, trust,
distribution, implementation, or runtime authority is implied from either
state.

A terminal state may be cited as historical context. It must not be used to:

1. Resurrect a capability record.
2. Transfer authority to another capability record.
3. Establish alias or supersession.
4. Carry over evidence.
5. Carry over trust.
6. Carry over registry publication.
7. Carry over implementation authority.
8. Carry over runtime authority.

Any exception to terminal-state behavior requires a later reviewed RFC or
decision path with exact-subject evidence.

## Transition Requirements

A lifecycle transition for a specific capability record requires all of the
following:

1. Exact capability identity.
2. Exact manifest reference or explicit reason no manifest is applicable.
3. Current lifecycle state.
4. Proposed next lifecycle state.
5. Reason for transition.
6. Applicable governing RFCs.
7. Evidence requirements, if any.
8. Reviewer or maintainer action path, if any.
9. Exact subject SHA for the transition record.
10. Fail-closed handling for ambiguity.

This RFC defines requirements for later transition authority. It does not
grant transition authority itself.

## Forbidden Transition Readings

The following readings are denied:

1. `concept` to `accepted` without review.
2. `draft` to `registry publication`.
3. `review` to `implementation authority`.
4. `accepted` to `runtime activation`.
5. `accepted` to `trust assignment`.
6. `accepted` to `capability issuance`.
7. `deprecated` to `revoked` by implication.
8. `revoked` to `accepted` without a new reviewed path.
9. `retired` to `active` by implication.
10. `quarantined` to any authority-bearing state without explicit review.

Any transition not explicitly accepted by later reviewed records fails
closed.

## Identity Relationship

Lifecycle state is bound to capability identity.

`PHASE20_CAPABILITY_IDENTITY.md` defines capability identity as:

```text
(identity_domain, namespace, logical_name, version_identity, digest_binding)
```

Lifecycle does not redefine, normalize, alias, supersede, or rename that
identity.

Changing identity-defining fields creates a different identity unless later
reviewed RFCs define exact narrower behavior.

Lifecycle state must not transfer across identity changes by implication.

## Manifest Relationship

Lifecycle state is manifest-aware but not manifest-authoritative.

`PHASE20_CAPABILITY_MANIFEST_SCHEMA.md` defines a declarative, inert,
canonical-ready manifest model.

Lifecycle may refer to manifest validity, manifest ambiguity, manifest
subject, and manifest scope. It must not treat a manifest as executable,
loadable, installable, publishable, trusted, or active.

A lifecycle transition that relies on manifest content fails closed if that
manifest content is missing, ambiguous, stale, inherited from another
subject, or differently scoped.

## Subject And Scope Relationship

Lifecycle state must preserve capability subject and scope.

`PHASE20_CAPABILITY_MODEL.md` distinguishes:

```text
subject -> the future behavior being described
scope   -> the bounded domain and limits of that behavior
```

A lifecycle transition that changes subject or scope is not a silent state
transition. It is a new governance claim unless later reviewed RFCs define
exact equivalence rules.

## Quarantine Boundary

Quarantine is the safe state for unresolved lifecycle ambiguity.

A capability record may be quarantined when review identifies:

1. Identity collision.
2. Manifest ambiguity.
3. Subject/scope ambiguity.
4. Evidence conflict.
5. Trust-input conflict.
6. Registry-reference conflict.
7. Distribution concern.
8. Safety concern.
9. Authority ambiguity.
10. Incompatible interpretation across governing records.

Quarantine must not be used as proof of fault, acceptance, rejection,
publication, revocation, implementation, or runtime action.

## Revocation Boundary

Revocation is a governance denial for future authority paths.

Revocation does not:

1. Delete history.
2. Delete evidence.
3. Delete review records.
4. Remove registry records by itself.
5. Disable runtime behavior by itself.
6. Transfer authority to a replacement.
7. Establish alias or supersession.
8. Prove fault by itself.

Revocation procedures, proof requirements, notification rules, distribution
effects, registry effects, and rollback behavior belong to later RFCs.

## Retirement Boundary

Retirement closes a capability record for active planning.

Retirement is historical. It must preserve auditability and must not erase
prior decisions, evidence, or governance records.

Retirement does not create replacement authority, distribution authority,
trust authority, implementation authority, or runtime authority.

## Alias And Supersession Boundary

Alias and supersession remain denied by default.

Lifecycle state must not be used to claim that one capability identity is an
alias of another or that one capability identity supersedes another.

Any alias or supersession model requires later reviewed RFCs and exact
rules. It must not transfer evidence, acceptance, trust, registry
publication, distribution status, implementation authority, runtime
authority, or capability issuance authority by implication.

## Relationship To Later RFCs

The lifecycle model is a prerequisite for later Phase-20 RFCs.

| Later RFC | Lifecycle relationship |
|---|---|
| `PHASE20_REGISTRY_MODEL.md` | Uses lifecycle state as a registry input without granting publication. |
| `PHASE20_REGISTRY_GOVERNANCE.md` | Uses lifecycle state for admission and quarantine review without granting registry authority. |
| `PHASE20_TRUST_MODEL.md` | Uses lifecycle state as trust context without assigning trust. |
| `PHASE20_DISTRIBUTION_POLICY.md` | Uses lifecycle state for publication, revocation, quarantine, and retirement policy without granting distribution. |
| `PHASE20_CAPABILITY_EVIDENCE_MODEL.md` | Binds lifecycle transitions to evidence requirements without evidence inheritance. |
| `PHASE20_CAPABILITY_ACCEPTANCE_WORKFLOW.md` | Defines how lifecycle changes are reviewed, accepted, merged, and verified without granting implementation authority. |

Later RFCs may narrow lifecycle use. They must not broaden lifecycle state
into active authority without a separate reviewed decision.

## Lifecycle Invariants

Every later Phase-20 RFC must preserve these lifecycle invariants:

1. Lifecycle state is governance state, not runtime state.
2. Lifecycle state is identity-bound.
3. Lifecycle state is manifest-aware but not manifest-authoritative.
4. Lifecycle state preserves subject/scope separation.
5. Lifecycle state does not imply registry publication.
6. Lifecycle state does not imply trust assignment.
7. Lifecycle state does not imply distribution.
8. Lifecycle state does not imply evidence inheritance.
9. Lifecycle state does not imply source acceptance.
10. Lifecycle state does not imply implementation authority.
11. Lifecycle state does not imply runtime activation.
12. Accepted governance state is not implementation acceptance.
13. Quarantine grants no authority.
14. Revocation does not erase history.
15. Retirement does not create replacement authority.

Violation of any invariant fails closed.

## Non-Goals

This document does not define or authorize:

1. Lifecycle transition implementation.
2. Lifecycle storage implementation.
3. Lifecycle command syntax.
4. Registry key serialization.
5. Registry storage implementation.
6. Registry admission or publication.
7. Trust model.
8. Distribution policy.
9. Evidence package format.
10. Acceptance workflow.
11. Alias implementation.
12. Supersession implementation.
13. Version compatibility rules.
14. Dependency resolution.
15. Source implementation.
16. Runtime activation.
17. Package installation, loading, execution, scheduling, or publication.
18. Module loading.
19. Workspace creation, workspace runtime, or real mounts.
20. Plugin host, plugin loading, or plugin instantiation.
21. Capability token minting or capability issuance.
22. Trust assignment or trust issuer behavior.
23. Semantic CLI execution or verdict authority.
24. AI Runtime authority.
25. Agent behavior.
26. New syscalls.
27. Kernel ABI expansion.
28. Workflow-threshold, baseline, dependency, or Ring0 policy changes.

## Implementation Gating

No Phase-20 implementation slice may start from this lifecycle RFC alone.

Implementation remains denied unless a later reviewed implementation
decision:

1. Identifies the exact implementation subject.
2. References accepted prerequisite RFCs.
3. States the bounded behavior being authorized.
4. States the denied behaviors.
5. Produces exact-subject evidence.
6. Receives acceptance review.
7. Is merged under the applicable governance path.
8. Receives post-merge exact-SHA verification.

Lifecycle model is not implementation authority.

## Explicit Non-Authorization

This lifecycle RFC does not authorize:

1. Runtime activation.
2. General runtime authority.
3. Phase-20 implementation.
4. Source acceptance or merge authority.
5. Lifecycle state change authority for any specific capability.
6. Lifecycle transition implementation.
7. Registry admission or publication.
8. Trust assignment.
9. Distribution authority.
10. Evidence inheritance.
11. Package installation, loading, execution, scheduling, or publication.
12. Module loading.
13. Workspace creation, workspace runtime, or real mounts.
14. Plugin host, plugin loading, or plugin instantiation.
15. Capability token minting or capability issuance.
16. Semantic CLI execution or verdict authority.
17. AI Runtime authority.
18. Agent behavior.
19. New syscalls.
20. Kernel ABI expansion.
21. Workflow-threshold, baseline, dependency, or Ring0 policy changes.
22. Observability-as-authority.

Unknown authority readings fail closed.

## Lifecycle Conclusion

An AykenOS capability lifecycle is the governance state model that describes
how one inert capability record moves through concept, draft, review,
accepted, deprecated, quarantined, revoked, and retired states.

Capability lifecycle supports later registry, trust, distribution,
evidence, acceptance, and implementation RFCs. It does not authorize any of
them.

Runtime activation, general runtime authority, Phase-20 implementation
authority, capability issuance, registry publication, package execution,
module loading, workspace runtime, plugin loading, trust assignment,
Semantic CLI authority, AI Runtime authority, agent authority, syscall
expansion, kernel ABI expansion, workflow-threshold changes, baseline
changes, dependency changes, and Ring0 authority remain pending and
unauthorized until separately reviewed and decided.

## Architecture Signature

**Prepared by:** Kenan AY
**Role:** AykenOS Architecture Steward
**Document type:** Phase-20 RFC
**Architecture status:** Draft / pending review
**Authority notice:** This signature identifies the architectural authorship
of this RFC. It grants no runtime authority, implementation authority, trust
authority, execution authority, constitutional authority, registry authority,
capability issuance authority, package authority, module authority, plugin
authority, Semantic CLI authority, AI Runtime authority, agent authority, or
Ring0 authority.

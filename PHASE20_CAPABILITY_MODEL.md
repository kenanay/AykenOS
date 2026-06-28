# Phase-20 Capability Model

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, the Phase-18 Platform Constitution reference set,
`docs/specs/phase18-platform-constitution/AUTHORITY_DRIFT_GUARD.md`,
`docs/specs/phase18-platform-constitution/TERMINOLOGY_AUDIT.md`,
`PHASE19_RUNTIME_DECISION.md`, the Phase-19 Runtime RFC set,
`docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_MATRIX.md`,
`PHASE19_CLOSURE_DECISION.md`,
`PHASE20_POINTER_TRANSITION_CANDIDATE.md`,
`PHASE20_POINTER_TRANSITION_DECISION.md`, and
`PHASE20_GOVERNANCE_OVERVIEW.md`. In case of conflict, those documents
prevail unless this model is the narrower Phase-20 capability concept record
for the exact planning scope identified below.

**Status:** PHASE-20 CAPABILITY MODEL RFC / CONCEPTUAL MODEL ONLY / NO
RUNTIME ACTIVATION / NO GENERAL RUNTIME AUTHORITY / NO IMPLEMENTATION
AUTHORITY / NO CAPABILITY ISSUANCE / NO REGISTRY PUBLICATION / NO MANIFEST
SCHEMA AUTHORITY
**Model date:** 2026-06-28
**Model id:** `ayken.phase20.capability_model.v1`
**Model base main SHA:** `2901048942657e66076308cdb673dcfd5fa8d21a`
**Current phase pointer:** `CURRENT_PHASE=20`
**Authority boundary:** Capability model only; not a capability identity
decision, not a manifest schema, not lifecycle authority, not registry
authority, not trust authority, not distribution authority, not evidence
acceptance, not implementation authority, not runtime activation, not
general runtime authority, not package installation, not package execution,
not module loading, not workspace runtime, not plugin loading, not
capability token minting, not capability issuance, not registry publication,
not trust assignment, not Semantic CLI authority, not AI Runtime authority,
not agent authority, not syscall expansion, not kernel ABI expansion, not
workflow-threshold, baseline, dependency, or Ring0 authority.

## Purpose

This document defines the Phase-20 capability concept for AykenOS.

It answers one question:

```text
What is an AykenOS capability?
```

It does not answer:

```text
How is a capability identity encoded?
How is a capability manifest serialized?
How is a capability lifecycle state machine enforced?
How is a capability stored in a registry?
How is a capability trusted, distributed, accepted, implemented, or run?
```

Those questions belong to later Phase-20 RFCs.

## Core Rule

```text
capability != runtime permission
capability != bearer token
capability != package
capability != executable
capability != module or plugin
capability != registry publication
capability != trust assignment
capability != implementation authority
```

A capability is a constitutional planning object. It is not active behavior.

Unknown authority readings fail closed.

## Capability Definition

An AykenOS capability is an inert, identity-bound, lifecycle-aware
constitutional record used to describe a bounded future operating-system
behavior before that behavior is implemented or activated.

A capability record provides a stable architectural anchor for later:

1. Identity binding.
2. Manifest description.
3. Lifecycle review.
4. Registry indexing.
5. Trust input collection.
6. Distribution policy review.
7. Evidence attachment.
8. Acceptance workflow.
9. Bounded implementation decisions.

The capability record itself grants none of those authorities.

## Capability Mission

The mission of the capability model is to make future AykenOS behavior
reviewable before it becomes executable, loadable, issuable, publishable, or
authority-bearing.

Capabilities give AykenOS a common object around which later RFCs can
organize:

1. What is being described.
2. What identity binds it.
3. What metadata describes it.
4. What lifecycle state it occupies.
5. What evidence is required.
6. What registry relationships exist.
7. What remains explicitly unauthorized.

The model prefers architectural stability over implementation speed.

## Capability Design Principles

Every capability record and later capability RFC must preserve these
principles:

1. **Inert by default:** a capability record has no runtime effect.
2. **Identity-bound:** a capability must be capable of later binding to a
   stable identity model.
3. **Lifecycle-aware:** a capability must be capable of later participating
   in a reviewed lifecycle.
4. **Manifest-compatible:** a capability must be describable by a future
   declarative manifest without becoming a parser, loader, installer, or
   executor request.
5. **Registry-compatible:** a capability must be capable of later registry
   reference without becoming published or active.
6. **Evidence-ready:** a capability must be capable of later evidence
   attachment without treating evidence as authority.
7. **Fail-closed:** missing, ambiguous, stale, inherited, or differently
   scoped authority is denied.
8. **Decision-bound:** only a later reviewed decision may grant bounded
   behavior for an exact subject.

## Capability Record Model

This document defines conceptual fields only. It does not define a manifest
schema, file format, parser, Rust type, registry table, ABI, or serialized
representation.

| Conceptual field | Purpose | Authority result |
|---|---|---|
| Capability subject | Names what future behavior is being described | No execution |
| Capability scope | States the bounded domain of the future behavior | No implementation |
| Non-authority boundary | States what the capability does not authorize | No implicit grant |
| Identity placeholder | Reserves the need for stable identity binding | No identity encoding |
| Manifest placeholder | Reserves the need for declarative description | No schema authority |
| Lifecycle placeholder | Reserves the need for reviewed state handling | No state transition authority |
| Registry placeholder | Reserves the need for future indexing/reference | No publication |
| Trust placeholder | Reserves the need for trust inputs | No trust assignment |
| Evidence placeholder | Reserves the need for exact-subject evidence | No evidence inheritance |
| Acceptance placeholder | Reserves the need for review and merge workflow | No acceptance |

Later RFCs may refine these fields, but they must not reinterpret this
conceptual model as implementation authority.

Capability subject identifies the future behavior being described.
Capability scope defines the bounded domain and limits of that behavior.

## Capability Identity Boundary

This document requires that a capability be compatible with stable identity.

It does not define:

1. Digest algorithm.
2. Naming scheme.
3. Versioning scheme.
4. Collision rule.
5. Immutability rule.
6. Identity serialization.
7. Registry key format.

Those are owned by `PHASE20_CAPABILITY_IDENTITY.md`.

Until that RFC is reviewed and accepted, any identity-specific interpretation
fails closed.

## Capability Manifest Boundary

This document requires that a capability be describable by a future
declarative manifest.

It does not define:

1. Manifest filename.
2. Manifest syntax.
3. Manifest schema.
4. Manifest parser.
5. Manifest validation implementation.
6. Loader behavior.
7. Installer behavior.
8. Execution behavior.

Those are owned by `PHASE20_CAPABILITY_MANIFEST_SCHEMA.md` and later
reviewed implementation decisions.

Manifest compatibility does not grant parser, loader, installer, package,
module, plugin, workspace, Semantic CLI, AI Runtime, agent, or runtime
authority.

## Capability Boundary

A capability may describe future behavior, but it must not perform,
authorize, imply, inherit, or activate that behavior.

A capability must not be read as:

1. A runtime permission.
2. A bearer token.
3. A package or executable.
4. A module or plugin.
5. A workspace handle.
6. A registry publication.
7. A trust assignment.
8. A distribution grant.
9. A Semantic CLI verdict.
10. AI Runtime authority.
11. Agent authority.
12. Kernel or syscall authority.
13. Ring0 policy.

Any proposal that turns a capability record into active behavior requires a
separate reviewed decision path and exact-SHA evidence.

## Capability States Boundary

This document recognizes that capabilities will need lifecycle states.

At this model layer, states are conceptual only. A capability may later move
through reviewable states such as concept, draft, review, accepted,
registered, implementation-candidate, implemented, distributed, deprecated,
revoked, quarantined, retired, or rejected.

This document does not define the exact state machine.

`PHASE20_CAPABILITY_LIFECYCLE.md` must later define:

1. Valid states.
2. Invalid states.
3. Allowed transitions.
4. Forbidden transitions.
5. Required evidence for each transition.
6. Review requirements.
7. Registry effects.
8. Fail-closed handling for ambiguity.

No lifecycle state grants runtime activation, implementation authority,
capability issuance, registry publication, trust assignment, package
execution, module loading, plugin loading, workspace runtime, Semantic CLI
authority, AI Runtime authority, or agent authority by itself.

## Capability Relationships

A capability may later relate to other records, but relationship existence
is not authority.

Allowed relationship categories for later RFC definition include:

1. Capability-to-identity relationship.
2. Capability-to-manifest relationship.
3. Capability-to-lifecycle relationship.
4. Capability-to-registry relationship.
5. Capability-to-trust-input relationship.
6. Capability-to-distribution-policy relationship.
7. Capability-to-evidence relationship.
8. Capability-to-acceptance-review relationship.
9. Capability-to-implementation-candidate relationship.

This model does not define dependency resolution, loader order, execution
order, trust inheritance, registry publication, package installation,
module loading, plugin loading, workspace mounting, or capability issuance.

## Capability Invariants

Every later Phase-20 capability RFC must preserve these invariants:

1. A capability record is inert unless a later reviewed decision grants
   bounded behavior.
2. A capability record must have a bounded subject.
3. A capability record must have an explicit non-authority boundary.
4. A capability record must remain compatible with exact-subject evidence.
5. A capability record must not inherit authority from historical PASS
   results.
6. A capability record must not inherit authority from `CURRENT_PHASE=20`.
7. A capability record must not inherit authority from registry presence.
8. A capability record must not inherit authority from trust input presence.
9. A capability record must not inherit authority from manifest validity.
10. A capability record must not bypass later acceptance workflow.

Any violation fails closed.

## Positive Model

The positive Phase-20 capability model is:

```text
Capability
  -> describes a bounded future behavior
  -> binds later to identity
  -> is described later by an inert manifest
  -> moves later through a reviewed lifecycle
  -> may later be referenced by a registry
  -> may later collect trust inputs
  -> may later attach exact-subject evidence
  -> may later enter acceptance workflow
  -> may later point to a bounded implementation candidate
```

Every arrow is a governance relationship. No arrow means execution,
issuance, publication, loading, mounting, trust assignment, or runtime
activation.

## Non-Goals

This document does not define or authorize:

1. Capability identity format.
2. Capability manifest schema.
3. Capability lifecycle state machine.
4. Registry data model.
5. Registry governance.
6. Trust model.
7. Distribution policy.
8. Evidence package format.
9. Acceptance workflow.
10. Source implementation.
11. Runtime activation.
12. Package installation, loading, execution, scheduling, or publication.
13. Module loading.
14. Workspace creation, workspace runtime, or real mounts.
15. Plugin host, plugin loading, or plugin instantiation.
16. Capability token minting or capability issuance.
17. Registry publication or marketplace behavior.
18. Trust assignment or trust issuer behavior.
19. Semantic CLI execution or verdict authority.
20. AI Runtime authority.
21. Agent behavior.
22. New syscalls.
23. Kernel ABI expansion.
24. Workflow-threshold, baseline, dependency, or Ring0 policy changes.

## Future RFC Dependencies

This model is the prerequisite conceptual record for the following Phase-20
RFCs:

| Future RFC | Dependency on this model |
|---|---|
| `PHASE20_CAPABILITY_IDENTITY.md` | Defines how capability identity binds to the inert capability concept. |
| `PHASE20_CAPABILITY_MANIFEST_SCHEMA.md` | Defines how a declarative manifest describes a capability without activating it. |
| `PHASE20_CAPABILITY_LIFECYCLE.md` | Defines exact states and transitions for capability records. |
| `PHASE20_REGISTRY_MODEL.md` | Defines how registry records reference capability records. |
| `PHASE20_REGISTRY_GOVERNANCE.md` | Defines admission and publication separation for capability registry records. |
| `PHASE20_TRUST_MODEL.md` | Defines trust inputs without turning capabilities into trust assignments. |
| `PHASE20_DISTRIBUTION_POLICY.md` | Defines publication, revocation, quarantine, rollback, and mirror policy inputs. |
| `PHASE20_CAPABILITY_EVIDENCE_MODEL.md` | Defines exact-subject evidence requirements for capability-related decisions. |
| `PHASE20_CAPABILITY_ACCEPTANCE_WORKFLOW.md` | Defines review and acceptance workflow for later bounded Phase-20 slices. |

Later RFCs may narrow this model. They must not broaden it into active
authority without a separate reviewed decision.

## Implementation Gating

No Phase-20 implementation slice may start from this document alone.

Implementation remains denied unless a later reviewed implementation
decision:

1. Identifies the exact implementation subject.
2. States the bounded behavior being authorized.
3. States the denied behaviors.
4. References accepted prerequisite RFCs.
5. Produces exact-subject evidence.
6. Receives acceptance review.
7. Is merged under the applicable governance path.
8. Receives post-merge exact-SHA verification.

Planning documents are not implementation authority.

## Explicit Non-Authorization

This model does not authorize:

1. Runtime activation.
2. General runtime authority.
3. Phase-20 implementation.
4. Source acceptance or merge authority.
5. Capability identity acceptance.
6. Manifest schema acceptance.
7. Lifecycle state transition authority.
8. Registry admission or publication.
9. Trust assignment.
10. Distribution authority.
11. Evidence inheritance.
12. Package installation, loading, execution, scheduling, or publication.
13. Module loading.
14. Workspace creation, workspace runtime, or real mounts.
15. Plugin host, plugin loading, or plugin instantiation.
16. Capability token minting or capability issuance.
17. Semantic CLI execution or verdict authority.
18. AI Runtime authority.
19. Agent behavior.
20. New syscalls.
21. Kernel ABI expansion.
22. Workflow-threshold, baseline, dependency, or Ring0 policy changes.
23. Observability-as-authority.

Unknown authority readings fail closed.

## Model Conclusion

An AykenOS capability is an inert constitutional record for describing a
bounded future behavior before implementation or activation authority exists.

The capability model establishes the conceptual anchor for later identity,
manifest, lifecycle, registry, trust, distribution, evidence, acceptance, and
implementation RFCs.

Runtime activation, general runtime authority, Phase-20 implementation
authority, capability issuance, registry publication, package execution,
module loading, workspace runtime, plugin loading, trust assignment,
Semantic CLI authority, AI Runtime authority, agent authority, syscall
expansion, kernel ABI expansion, workflow-threshold changes, baseline
changes, dependency changes, and Ring0 authority remain pending and
unauthorized until separately reviewed and decided.

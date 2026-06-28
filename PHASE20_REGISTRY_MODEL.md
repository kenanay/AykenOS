# Phase-20 Registry Model

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
`PHASE20_CAPABILITY_MANIFEST_SCHEMA.md`, and
`PHASE20_CAPABILITY_LIFECYCLE.md`. In case of conflict, those documents
prevail unless this registry model RFC is the narrower Phase-20 registry
model record for the exact planning scope identified below.

**Status:** PHASE-20 REGISTRY MODEL RFC / REGISTRY RECORD MODEL ONLY / NO
REGISTRY ADMISSION AUTHORITY / NO REGISTRY PUBLICATION / NO REGISTRY
GOVERNANCE AUTHORITY / NO TRUST ASSIGNMENT / NO DISTRIBUTION AUTHORITY / NO
EVIDENCE ACCEPTANCE / NO IMPLEMENTATION AUTHORITY / NO RUNTIME ACTIVATION /
NO GENERAL RUNTIME AUTHORITY
**Registry model date:** 2026-06-29
**Registry model id:** `ayken.phase20.registry_model.v1`
**Registry model base main SHA:** `9b9861c72dd7c0e675caa96d69a581fc7f71c673`
**Current phase pointer:** `CURRENT_PHASE=20`
**Authority boundary:** Registry record model only; not registry admission
authority, not registry publication, not registry governance authority, not
trust authority, not distribution authority, not evidence acceptance, not
implementation authority, not runtime activation, not general runtime
authority, not package installation, not package execution, not module
loading, not workspace runtime, not plugin loading, not capability token
minting, not capability issuance, not trust assignment, not Semantic CLI
authority, not AI Runtime authority, not agent authority, not syscall
expansion, not kernel ABI expansion, not workflow-threshold, baseline,
dependency, or Ring0 authority.

## Purpose

This document defines the Phase-20 registry model for AykenOS capability
records.

It answers one question:

```text
How is a capability registry modeled?
```

It does not answer:

```text
How is a capability admitted to a registry?
How is a registry published?
How is a capability trusted, distributed, accepted, implemented, activated,
or run?
```

Those questions belong to later Phase-20 RFCs and implementation decisions.

## Core Rule

```text
registry model != registry governance
registry record != registry admission
registry entry != registry publication
registry index != package repository
registry view != API service
registry snapshot != distribution release
registry presence != trust assignment
registry presence != capability issuance
registry model != implementation authority
registry model != runtime activation
```

A registry makes capability records organizable and referenceable. It does
not make them active, trusted, published, accepted for source
implementation, executable, loadable, installable, distributable, runnable,
or authority-bearing.

Unknown authority readings fail closed.

## Registry Mission

The mission of the Phase-20 registry model is to define an inert,
reviewable, integrity-oriented record space for organizing capability
records.

A registry exists so later RFCs can refer to capability identities,
manifests, lifecycle states, metadata, indexes, views, and references
without turning those records into behavior.

The registry model itself grants no admission, publication, trust,
distribution, evidence, acceptance, implementation, or runtime authority.

It supports later:

1. Registry governance.
2. Registry admission review.
3. Trust input association.
4. Distribution policy review.
5. Revocation and quarantine policy.
6. Evidence package construction.
7. Acceptance workflow.
8. Bounded implementation decisions.

Each later use requires its own reviewed RFC or decision path.

## Registry Definition

A capability registry is a conceptual governance record set that organizes
capability records by identity, manifest reference, lifecycle state, and
reviewable metadata.

A registry is not:

1. A database implementation.
2. A package repository.
3. A module repository.
4. A plugin marketplace.
5. A runtime service.
6. An execution engine.
7. A loader.
8. An installer.
9. A trust issuer.
10. A capability issuer.
11. A distribution channel.
12. Semantic CLI, AI Runtime, or agent authority.

## Registry Principles

Every registry model rule must preserve these principles:

1. **Inert record space:** registry records organize references; they do
   not act.
2. **Identity-bound:** registry entries must reference exact capability
   identities and must not redefine identity.
3. **Manifest-aware:** registry entries may reference manifests but must not
   treat manifests as executable, loadable, publishable, trusted, or active.
4. **Lifecycle-aware:** registry entries may reference lifecycle state but
   must not execute lifecycle transitions.
5. **Index-denying:** indexes improve reviewability and lookup semantics;
   they do not grant authority.
6. **View-denying:** views present registry material; they do not publish,
   distribute, trust, execute, or accept records.
7. **Publication-denying:** registry presence is not registry publication.
8. **Trust-denying:** registry presence is not trust assignment.
9. **Evidence-denying:** registry presence does not accept evidence or
   inherit evidence.
10. **Fail-closed:** missing, ambiguous, stale, inherited, colliding, or
    differently scoped registry material fails closed.

## Registry Model Components

The conceptual registry model is composed of:

| Component | Purpose | Authority result |
|---|---|---|
| Registry record set | Organizes capability registry records | No database implementation |
| Registry entry | Represents one capability record in registry form | No admission or publication |
| Registry reference | Links to identity, manifest, lifecycle, or metadata records | No authority transfer |
| Registry namespace | Groups entries for reviewable organization | No ownership or trust |
| Registry index | Provides deterministic lookup dimensions | No authority |
| Registry view | Presents selected registry material | No API or distribution |
| Registry snapshot | Captures registry record state for review | No release or publication |
| Registry tombstone | Records historical absence, removal intent, or retired linkage | No deletion or revocation by itself |

These components are conceptual. They do not define a storage engine,
database schema, file format, API, command, wire format, package format,
loader input, runtime handle, or marketplace behavior.

## Registry Record Set

A registry record set is the conceptual collection of registry records for
Phase-20 capability governance.

It may organize:

1. Capability identity references.
2. Manifest references.
3. Lifecycle state references.
4. Metadata references.
5. Relationship references.
6. Index material.
7. View material.
8. Snapshot material.
9. Tombstone material.

A registry record set is not a database, repository, service, filesystem,
network protocol, package channel, module channel, plugin channel, runtime
registry, or trust root.

## Registry Entry

A registry entry is the conceptual registry representation of one capability
record.

A registry entry may include:

1. Exact capability identity reference.
2. Manifest reference.
3. Lifecycle state reference.
4. Registry metadata.
5. Relationship references.
6. Index keys.
7. Review notes.
8. Non-authorization notice.

A registry entry does not imply:

1. Admission.
2. Publication.
3. Trust.
4. Distribution.
5. Evidence acceptance.
6. Source acceptance.
7. Implementation authority.
8. Runtime activation.

## Registry Reference

A registry reference is an inert pointer from registry material to another
governance record.

Registry references may point to:

1. Capability identity records.
2. Manifest records.
3. Lifecycle state records.
4. Metadata records.
5. Later trust input records.
6. Later distribution policy records.
7. Later evidence records.
8. Later acceptance workflow records.

A registry reference must not carry authority from the referenced record.
It also must not grant authority to the referenced record.

Reference ambiguity, broken references, stale references, inherited
references, or references to differently scoped subjects fail closed.

## Registry Namespace

A registry namespace is a governed grouping context for registry entries.

It may group entries for reviewable organization, comparison, and lookup.

A registry namespace is not:

1. Ownership.
2. Delegation.
3. Reservation.
4. Trust.
5. Publication right.
6. Distribution right.
7. Capability issuance authority.
8. Runtime authority.

Namespace ownership, delegation, reservation, transfer, and conflict policy
belong to later registry governance RFCs.

## Registry Index

A registry index is a deterministic lookup dimension over registry records.

Indexes may be defined over:

1. Capability identity.
2. Namespace.
3. Logical name.
4. Version identity.
5. Manifest reference.
6. Lifecycle state.
7. Metadata keys.
8. Relationship references.

An index is not:

1. A search service.
2. A database index implementation.
3. A package resolver.
4. A dependency resolver.
5. A trust resolver.
6. A loader.
7. A distribution selector.
8. A runtime selector.

Index hits do not imply admission, publication, trust, distribution,
implementation authority, or runtime activation.

## Registry View

A registry view is a deterministic presentation of selected registry
material.

Views may support review by presenting registry entries, references,
indexes, lifecycle state, and metadata in a narrowed context.

A view is not:

1. An API endpoint.
2. A published registry.
3. A distribution feed.
4. A marketplace listing.
5. A package index.
6. A trust statement.
7. A runtime interface.

Unknown view readings fail closed.

## Registry Snapshot

A registry snapshot is a reviewable point-in-time capture of registry record
state.

Snapshots may support audit, evidence preparation, comparison, and later
acceptance workflows.

A snapshot is not:

1. A release.
2. A publication.
3. A distribution artifact.
4. A package repository state.
5. A trust proof.
6. An evidence package by itself.
7. A runtime state.

Snapshot integrity rules, canonicalization, signing, distribution, and
evidence binding belong to later RFCs.

## Registry Tombstone

A registry tombstone is a historical marker that records that a registry
entry is absent, removed from active registry planning, superseded by later
review, or retained only for audit context.

A tombstone does not delete history, revoke a capability by itself,
establish replacement authority, prove fault, assign trust, or publish an
alternative record.

Tombstone semantics, retention rules, revocation effects, supersession
rules, and registry governance procedures belong to later RFCs.

## Minimal Registry Entry Shape

The conceptual minimal registry entry shape is:

```text
registry_entry
  -> capability_identity_reference
  -> manifest_reference
  -> lifecycle_state_reference
  -> registry_metadata
  -> non_authorization_notice
```

This shape is conceptual. It does not define concrete serialization,
database columns, storage keys, file paths, command output, API response,
wire representation, or runtime object layout.

Missing, ambiguous, inherited, stale, or differently scoped required
registry entry material fails closed.

## Registry Relationship Model

Registry relationships are inert references between registry entries or
between registry entries and later governance records.

Relationship types may later include:

1. Dependency.
2. Compatibility.
3. Supersession.
4. Deprecation.
5. Quarantine.
6. Revocation.
7. Retirement.
8. Evidence association.

This RFC does not define those relationship semantics. It only reserves the
model space for later reviewed RFCs.

Relationship presence must not transfer evidence, acceptance, trust,
publication, distribution, implementation authority, runtime authority, or
capability issuance authority.

## Identity Relationship

Registry entries are identity-bound.

`PHASE20_CAPABILITY_IDENTITY.md` defines capability identity as:

```text
(identity_domain, namespace, logical_name, version_identity, digest_binding)
```

The registry model references this identity. It does not redefine,
normalize, alias, supersede, rename, publish, trust, or issue it.

Registry entries that reference missing, ambiguous, colliding, or
differently scoped identities fail closed.

## Manifest Relationship

Registry entries may reference capability manifests.

`PHASE20_CAPABILITY_MANIFEST_SCHEMA.md` defines a declarative, inert,
canonical-ready manifest model.

The registry model must not treat manifests as executable, loadable,
installable, publishable, trusted, distributed, accepted, or active.

Registry entries that rely on missing, ambiguous, stale, inherited, or
differently scoped manifest material fail closed.

## Lifecycle Relationship

Registry entries may reference capability lifecycle state.

`PHASE20_CAPABILITY_LIFECYCLE.md` defines lifecycle state as governance
state, not runtime state.

Registry presence must not execute lifecycle transitions or treat lifecycle
state as registry admission, publication, trust assignment, distribution,
implementation authority, or runtime activation.

Lifecycle ambiguity in registry material fails closed.

## Admission Boundary

Registry model does not define registry admission.

An entry shape, reference, index, view, or snapshot must not be interpreted
as admitted registry content.

Admission gates, review requirements, non-bypass constraints, reviewer
roles, rejection handling, quarantine handling, and merge procedures belong
to `PHASE20_REGISTRY_GOVERNANCE.md` or later reviewed records.

## Publication Boundary

Registry model does not define registry publication.

Registry record existence, registry entry presence, registry indexes,
registry views, or registry snapshots must not be interpreted as published
registry content.

Publication, distribution, mirror policy, revocation effects, quarantine
effects, rollback rules, and marketplace behavior belong to later RFCs.

## Trust Boundary

Registry model does not assign trust.

Registry presence, index presence, view presence, snapshot presence, or
relationship presence must not be interpreted as authenticity proof,
signature acceptance, trust assignment, trust issuer behavior, capability
issuance, or runtime authority.

Trust inputs and proof requirements belong to `PHASE20_TRUST_MODEL.md` or
later reviewed records.

## Evidence Boundary

Registry model does not accept evidence.

Registry entries may later reference evidence records, but such references
must not inherit evidence, accept evidence, validate evidence, or turn
evidence into authority.

Evidence package format, exact-subject binding, review requirements, and
post-merge verification belong to later evidence and acceptance RFCs.

## Registry Invariants

Every later Phase-20 RFC must preserve these registry invariants:

1. Registry records are inert governance records.
2. Registry presence is not registry admission.
3. Registry presence is not registry publication.
4. Registry entries are identity-bound.
5. Registry entries are manifest-aware but not manifest-authoritative.
6. Registry entries are lifecycle-aware but do not execute lifecycle
   transitions.
7. Registry indexes do not grant authority.
8. Registry views do not publish or distribute.
9. Registry snapshots are not releases.
10. Registry tombstones do not erase history.
11. Registry relationships do not transfer authority.
12. Registry records do not imply trust assignment.
13. Registry records do not imply evidence acceptance.
14. Registry records do not imply implementation authority.
15. Registry records do not imply runtime activation.

Violation of any invariant fails closed.

## Relationship To Later RFCs

The registry model is a prerequisite for later Phase-20 RFCs.

| Later RFC | Registry model relationship |
|---|---|
| `PHASE20_REGISTRY_GOVERNANCE.md` | Defines admission gates, non-bypass rules, reviewer paths, quarantine handling, and publication separation. |
| `PHASE20_TRUST_MODEL.md` | Uses registry references as trust context without assigning trust. |
| `PHASE20_DISTRIBUTION_POLICY.md` | Uses registry records, views, snapshots, and tombstones for publication and rollback policy without granting distribution. |
| `PHASE20_CAPABILITY_EVIDENCE_MODEL.md` | Uses registry references and snapshots for evidence binding without evidence inheritance. |
| `PHASE20_CAPABILITY_ACCEPTANCE_WORKFLOW.md` | Uses registry records for review tracking without implementation authority. |

Later RFCs may narrow registry use. They must not broaden registry model
into active authority without a separate reviewed decision.

## Non-Goals

This document does not define or authorize:

1. Registry admission.
2. Registry governance.
3. Registry publication.
4. Registry storage implementation.
5. Database schema.
6. API endpoint.
7. Search service.
8. Package repository.
9. Module repository.
10. Plugin marketplace.
11. Dependency resolver.
12. Trust model.
13. Distribution policy.
14. Evidence package format.
15. Acceptance workflow.
16. Alias implementation.
17. Supersession implementation.
18. Version compatibility rules.
19. Lifecycle transition implementation.
20. Source implementation.
21. Runtime activation.
22. Package installation, loading, execution, scheduling, or publication.
23. Module loading.
24. Workspace creation, workspace runtime, or real mounts.
25. Plugin host, plugin loading, or plugin instantiation.
26. Capability token minting or capability issuance.
27. Trust assignment or trust issuer behavior.
28. Semantic CLI execution or verdict authority.
29. AI Runtime authority.
30. Agent behavior.
31. New syscalls.
32. Kernel ABI expansion.
33. Workflow-threshold, baseline, dependency, or Ring0 policy changes.

## Implementation Gating

No Phase-20 implementation slice may start from this registry model RFC
alone.

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

Registry model is not implementation authority.

## Explicit Non-Authorization

This registry model RFC does not authorize:

1. Runtime activation.
2. General runtime authority.
3. Phase-20 implementation.
4. Source acceptance or merge authority.
5. Registry admission.
6. Registry governance.
7. Registry publication.
8. Registry storage implementation.
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

## Registry Model Conclusion

An AykenOS capability registry is an inert governance record set that
organizes capability identities, manifests, lifecycle states, metadata,
references, indexes, views, snapshots, and tombstones for later reviewed
governance.

Registry model supports later registry governance, trust, distribution,
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

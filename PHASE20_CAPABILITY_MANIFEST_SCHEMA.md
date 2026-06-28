# Phase-20 Capability Manifest Schema

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
`PHASE20_CAPABILITY_MODEL.md`, and
`PHASE20_CAPABILITY_IDENTITY.md`. In case of conflict, those documents
prevail unless this manifest schema RFC is the narrower Phase-20 capability
manifest schema record for the exact planning scope identified below.

**Status:** PHASE-20 CAPABILITY MANIFEST SCHEMA RFC / DECLARATIVE MANIFEST
MODEL ONLY / NO PARSER AUTHORITY / NO LOADER AUTHORITY / NO INSTALLER
AUTHORITY / NO RUNTIME ACTIVATION / NO GENERAL RUNTIME AUTHORITY / NO
IMPLEMENTATION AUTHORITY / NO CAPABILITY ISSUANCE / NO REGISTRY PUBLICATION
/ NO LIFECYCLE TRANSITION AUTHORITY
**Manifest date:** 2026-06-28
**Manifest id:** `ayken.phase20.capability_manifest_schema.v1`
**Manifest base main SHA:** `cb6dc1752573160ad5807d5b972ba7a3def28ce3`
**Current phase pointer:** `CURRENT_PHASE=20`
**Authority boundary:** Capability manifest schema model only; not parser
implementation, not manifest parser acceptance, not loader authority, not
installer authority, not lifecycle authority, not registry authority, not
trust authority, not distribution authority, not evidence acceptance, not
implementation authority, not runtime activation, not general runtime
authority, not package installation, not package execution, not module
loading, not workspace runtime, not plugin loading, not capability token
minting, not capability issuance, not registry publication, not trust
assignment, not Semantic CLI authority, not AI Runtime authority, not agent
authority, not syscall expansion, not kernel ABI expansion, not
workflow-threshold, baseline, dependency, or Ring0 authority.

## Purpose

This document defines the Phase-20 manifest schema model for AykenOS
capability records.

It answers one question:

```text
How is a capability described declaratively?
```

It does not answer:

```text
How is a manifest parsed by source code?
How is a capability loaded, installed, issued, published, trusted, accepted,
implemented, executed, or run?
```

Those questions belong to later Phase-20 RFCs and implementation decisions.

## Core Rule

```text
capability manifest != parser authority
capability manifest != loader authority
capability manifest != installer authority
capability manifest != package execution
capability manifest != lifecycle transition
capability manifest != registry publication
capability manifest != trust assignment
capability manifest != implementation authority
capability manifest != runtime activation
```

A manifest makes a capability describable. It does not make the capability
active, trusted, published, accepted, implemented, executable, loadable,
installable, runnable, or authority-bearing.

Unknown authority readings fail closed.

## Manifest Mission

The mission of a capability manifest is to provide a stable, reviewable,
declarative description of a capability record.

A manifest exists so later RFCs can refer to capability metadata, identity,
subject, scope, governance requirements, dependencies, evidence references,
and acceptance inputs without turning those records into behavior.

The manifest record itself grants no authority.

It supports later:

1. Identity serialization references.
2. Lifecycle state modeling.
3. Registry indexing.
4. Trust input attachment.
5. Distribution policy review.
6. Evidence package construction.
7. Acceptance workflow.
8. Bounded implementation decisions.

Each later use requires its own reviewed RFC or decision path.

## Manifest Definition

A capability manifest is a declarative constitutional record that describes
one inert capability record.

It may identify:

1. The capability identity reference.
2. The capability subject.
3. The capability scope.
4. The manifest record metadata.
5. The governance references that constrain interpretation.
6. Optional relationships to later lifecycle, registry, trust,
   distribution, evidence, and acceptance records.

A manifest is not:

1. Executable.
2. Loadable.
3. Installable.
4. Runnable.
5. Publishable.
6. Trusted.
7. Active.
8. Authority-bearing.

## Serialization Boundary

This RFC defines a conceptual schema model, not a concrete serialization
schema.

It does not define YAML, JSON, TOML, ABDF, binary, text, or any other
concrete serialization format.

No file extension, parser grammar, canonical byte layout, schema language,
wire representation, storage representation, or command representation is
accepted by this RFC.

Until a later reviewed RFC defines concrete serialization, all
serialization-specific readings fail closed.

Serialization must not be used to:

1. Create parser authority.
2. Create loader authority.
3. Create installer authority.
4. Define package format.
5. Define registry storage.
6. Carry over evidence.
7. Assign trust.
8. Activate runtime behavior.

## Manifest Principles

Every capability manifest schema rule must preserve these principles:

1. **Declarative and inert:** manifest content describes; it does not act.
2. **Identity-referencing:** a manifest references capability identity; it
   does not redefine identity.
3. **Subject/scope preserving:** manifest content must preserve the
   `PHASE20_CAPABILITY_MODEL.md` distinction between subject and scope.
4. **Canonical-ready:** manifest sections must be ordered and interpreted in
   a deterministic way for later digest and evidence RFCs.
5. **Validation-before-use:** malformed, incomplete, ambiguous, stale, or
   differently scoped manifests fail closed.
6. **Parser-denying:** schema definition does not authorize parser
   implementation or parser acceptance.
7. **Loader-denying:** manifest presence does not authorize loading,
   installation, execution, scheduling, mounting, or runtime behavior.
8. **Publication-denying:** manifest presence does not authorize registry
   publication, package publication, distribution, or marketplace behavior.
9. **Trust-denying:** manifest presence does not assign trust, prove
   authenticity, or accept signatures.
10. **Evidence-denying:** manifest presence does not inherit evidence or
    accept an evidence package.

## Manifest Sections

The capability manifest schema is conceptually organized into sections:

| Section | Purpose | Authority result |
|---|---|---|
| Manifest header | Identifies the record as a capability manifest | No parser authority |
| Identity reference | References the capability identity tuple | No identity redefinition |
| Subject and scope | Describes the future behavior and its bounded limits | No runtime behavior |
| Metadata | Records reviewable descriptive fields | No trust or publication |
| Governance references | Lists governing records and constraints | No constitutional amendment |
| Dependency references | Records inert references to other records, if any | No package or module loading |
| Lifecycle placeholder | References later lifecycle interpretation, if any | No lifecycle transition |
| Registry placeholder | References later registry interpretation, if any | No registry publication |
| Trust placeholder | References later trust inputs, if any | No trust assignment |
| Evidence placeholder | References later evidence inputs, if any | No evidence acceptance |
| Acceptance placeholder | References later acceptance workflow, if any | No source acceptance |

These sections are schema concepts. They do not define a concrete file
format, parser, loader, registry key, package format, wire format, command,
or runtime handle.

## Required Fields

A complete capability manifest record must include these conceptual required
fields:

| Field | Purpose | Authority result |
|---|---|---|
| Manifest domain | States that the record is a capability manifest | No execution domain |
| Manifest schema reference | Identifies the schema family being used | No parser implementation |
| Capability identity reference | References the accepted identity tuple | No capability issuance |
| Capability subject | States the future behavior being described | No behavior activation |
| Capability scope | States the bounded domain and limits of that behavior | No runtime boundary grant |
| Governance references | States the records that constrain interpretation | No authority expansion |
| Non-authorization notice | States that the manifest is inert | No implementation authority |

All required fields must be present before a manifest can be treated as
complete for governance review.

Missing, ambiguous, inherited, stale, or differently scoped required fields
fail closed.

## Optional Fields

A manifest may include optional conceptual fields only when they remain
declarative and non-authority-bearing.

Optional fields may include:

1. Human-readable summary.
2. Maintainer or steward metadata.
3. Dependency references.
4. Lifecycle intent references.
5. Registry relationship references.
6. Trust input references.
7. Distribution policy references.
8. Evidence references.
9. Acceptance workflow references.
10. Review notes.

Optional fields must not be used to bypass required fields, create identity,
carry over evidence, assign trust, publish to a registry, load a package,
instantiate a module or plugin, issue a capability, or activate runtime
behavior.

Unknown optional-field readings fail closed.

## Canonical Manifest

A canonical manifest is the deterministic governance representation of a
capability manifest after accepted ordering and validation rules are applied.

The canonical manifest exists to prepare later digest, evidence, registry,
and acceptance RFCs.

It is not:

1. A canonical byte format.
2. A parser output contract.
3. A wire format.
4. A registry storage format.
5. A package archive format.
6. A loader input.
7. An execution plan.
8. A runtime grant.

This RFC defines the concept of canonical manifest ordering. It does not
define a canonical byte representation, digest algorithm, signature format,
file extension, storage layout, or parser implementation.

## Canonical Ordering

The conceptual canonical order of manifest material is:

```text
manifest_header
  -> capability_identity_reference
  -> capability_subject
  -> capability_scope
  -> metadata
  -> governance_references
  -> dependency_references
  -> lifecycle_references
  -> registry_references
  -> trust_references
  -> distribution_references
  -> evidence_references
  -> acceptance_references
  -> non_authorization_notice
```

This ordering supports review consistency only.

It does not define parser order, source-code structure, on-disk layout,
serialization syntax, registry index order, package layout, or runtime
execution order.

Later RFCs may narrow ordering rules. They must not use ordering to create
authority.

## Validation Rules

Manifest validation is conceptual and fail-closed.

A manifest is invalid for governance review when:

1. A required field is missing.
2. A required field is ambiguous.
3. A required field is inherited from another subject without review.
4. A field claims identity behavior that conflicts with
   `PHASE20_CAPABILITY_IDENTITY.md`.
5. A field merges subject and scope into an ambiguous authority claim.
6. A field implies parser, loader, installer, package, module, plugin, or
   runtime behavior.
7. A field implies registry publication.
8. A field implies trust assignment.
9. A field implies evidence acceptance or evidence inheritance.
10. A field relies on an undefined canonicalization rule.
11. A field relies on an undefined lifecycle transition.
12. A field relies on an undefined alias, supersession, compatibility, or
    distribution rule.

Validation failure grants no authority. It requires correction or a later
reviewed decision path.

## Identity Relationship

`PHASE20_CAPABILITY_IDENTITY.md` defines capability identity as:

```text
(identity_domain, namespace, logical_name, version_identity, digest_binding)
```

This manifest schema does not redefine that tuple.

The manifest references identity. It does not own identity, rename identity,
normalize identity, alias identity, supersede identity, publish identity, or
grant authority to identity.

If manifest content conflicts with the accepted identity record, the
conflict fails closed.

## Subject And Scope Relationship

`PHASE20_CAPABILITY_MODEL.md` distinguishes capability subject from
capability scope:

```text
subject -> the future behavior being described
scope   -> the bounded domain and limits of that behavior
```

The manifest may describe both. It must not collapse them into a single
authority-bearing field.

A manifest that describes the same subject with a different scope is a
different governance claim unless later reviewed RFCs define exact
equivalence rules.

## Digest Relationship

The manifest schema is digest-ready but not digest-authoritative.

A later RFC may define how a canonical manifest is transformed into digest
material for evidence, registry, or acceptance purposes.

This RFC does not define:

1. Digest algorithm.
2. Canonical byte representation.
3. Hash input construction.
4. Signature format.
5. Trust proof format.
6. Evidence package format.
7. Registry storage format.

Digest presence or digest readiness does not prove trust, acceptance,
publication, implementation correctness, runtime safety, or execution
authority.

## Dependency Reference Boundary

Manifest dependency references are inert.

A dependency reference may identify another governance record for later
review, but it must not:

1. Install a package.
2. Load a module.
3. Instantiate a plugin.
4. Mount a workspace.
5. Execute code.
6. Publish a registry entry.
7. Transfer trust.
8. Inherit evidence.
9. Activate runtime behavior.

Dependency semantics, compatibility, ordering, resolution, and failure
handling belong to later RFCs.

## Manifest Invariants

Every later Phase-20 RFC must preserve these manifest invariants:

1. A capability manifest is declarative and inert.
2. A capability manifest references identity; it does not redefine identity.
3. A capability manifest preserves subject/scope separation.
4. A capability manifest does not imply manifest parser acceptance.
5. A capability manifest does not imply lifecycle acceptance.
6. A capability manifest does not imply registry publication.
7. A capability manifest does not imply trust assignment.
8. A capability manifest does not imply distribution.
9. A capability manifest does not imply evidence acceptance.
10. A capability manifest does not imply implementation authority.
11. A capability manifest does not imply runtime activation.
12. A canonical manifest is not a runtime object.
13. Manifest validation is not authority.

Violation of any invariant fails closed.

## Relationship To Later RFCs

The manifest schema is a prerequisite for later Phase-20 RFCs.

| Later RFC | Manifest relationship |
|---|---|
| `PHASE20_CAPABILITY_LIFECYCLE.md` | Uses manifest subject, scope, and identity reference to define lifecycle state without granting transition authority. |
| `PHASE20_REGISTRY_MODEL.md` | Uses canonical manifest references for registry records without granting publication. |
| `PHASE20_REGISTRY_GOVERNANCE.md` | Uses manifest validation boundaries for admission review without granting registry authority. |
| `PHASE20_TRUST_MODEL.md` | Uses manifest references for trust inputs without assigning trust. |
| `PHASE20_DISTRIBUTION_POLICY.md` | Uses manifest references for distribution policy without granting distribution. |
| `PHASE20_CAPABILITY_EVIDENCE_MODEL.md` | Uses canonical manifest material for evidence binding without evidence inheritance. |
| `PHASE20_CAPABILITY_ACCEPTANCE_WORKFLOW.md` | Uses manifest validation state for review tracking without implementation authority. |

Later RFCs may narrow manifest use. They must not broaden manifest schema
into active authority without a separate reviewed decision.

## Non-Goals

This document does not define or authorize:

1. Concrete manifest syntax.
2. File extension.
3. Parser behavior.
4. Parser implementation.
5. Loader behavior.
6. Installer behavior.
7. Package format.
8. Registry key serialization.
9. Registry storage implementation.
10. Lifecycle state machine.
11. Lifecycle transition authority.
12. Alias implementation.
13. Supersession implementation.
14. Version compatibility rules.
15. Dependency resolution.
16. Trust model.
17. Distribution policy.
18. Evidence package format.
19. Acceptance workflow.
20. Source implementation.
21. Runtime activation.
22. Package installation, loading, execution, scheduling, or publication.
23. Module loading.
24. Workspace creation, workspace runtime, or real mounts.
25. Plugin host, plugin loading, or plugin instantiation.
26. Capability token minting or capability issuance.
27. Registry publication or marketplace behavior.
28. Trust assignment or trust issuer behavior.
29. Semantic CLI execution or verdict authority.
30. AI Runtime authority.
31. Agent behavior.
32. New syscalls.
33. Kernel ABI expansion.
34. Workflow-threshold, baseline, dependency, or Ring0 policy changes.

## Implementation Gating

No Phase-20 implementation slice may start from this manifest schema RFC
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

Manifest schema is not implementation authority.

## Explicit Non-Authorization

This manifest schema RFC does not authorize:

1. Runtime activation.
2. General runtime authority.
3. Phase-20 implementation.
4. Source acceptance or merge authority.
5. Parser implementation or parser acceptance.
6. Loader implementation or loader acceptance.
7. Installer behavior.
8. Lifecycle state transition authority.
9. Registry admission or publication.
10. Trust assignment.
11. Distribution authority.
12. Evidence inheritance.
13. Package installation, loading, execution, scheduling, or publication.
14. Module loading.
15. Workspace creation, workspace runtime, or real mounts.
16. Plugin host, plugin loading, or plugin instantiation.
17. Capability token minting or capability issuance.
18. Semantic CLI execution or verdict authority.
19. AI Runtime authority.
20. Agent behavior.
21. New syscalls.
22. Kernel ABI expansion.
23. Workflow-threshold, baseline, dependency, or Ring0 policy changes.
24. Observability-as-authority.

Unknown authority readings fail closed.

## Manifest Conclusion

An AykenOS capability manifest is a declarative, inert, canonical-ready
governance record that describes one capability record by referencing its
identity, subject, scope, metadata, and review constraints.

Capability manifest schema supports later lifecycle, registry, trust,
distribution, evidence, acceptance, and implementation RFCs. It does not
authorize any of them.

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

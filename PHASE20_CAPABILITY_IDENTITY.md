# Phase-20 Capability Identity

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, the Phase-18 Platform Constitution reference set,
`docs/specs/phase18-platform-constitution/AUTHORITY_DRIFT_GUARD.md`,
`docs/specs/phase18-platform-constitution/TERMINOLOGY_AUDIT.md`,
`PHASE19_RUNTIME_DECISION.md`, the Phase-19 Runtime RFC set,
`docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_MATRIX.md`,
`PHASE19_CLOSURE_DECISION.md`,
`PHASE20_POINTER_TRANSITION_CANDIDATE.md`,
`PHASE20_POINTER_TRANSITION_DECISION.md`,
`PHASE20_GOVERNANCE_OVERVIEW.md`, and
`PHASE20_CAPABILITY_MODEL.md`. In case of conflict, those documents prevail
unless this identity RFC is the narrower Phase-20 capability identity record
for the exact planning scope identified below.

**Status:** PHASE-20 CAPABILITY IDENTITY RFC / IDENTITY MODEL ONLY / NO
RUNTIME ACTIVATION / NO GENERAL RUNTIME AUTHORITY / NO IMPLEMENTATION
AUTHORITY / NO CAPABILITY ISSUANCE / NO REGISTRY PUBLICATION / NO MANIFEST
SCHEMA AUTHORITY / NO LIFECYCLE TRANSITION AUTHORITY
**Identity date:** 2026-06-28
**Identity id:** `ayken.phase20.capability_identity.v1`
**Identity base main SHA:** `c816eb0d9f1c2669f9d4dd419c60f2cbdcbf916c`
**Current phase pointer:** `CURRENT_PHASE=20`
**Authority boundary:** Capability identity model only; not a manifest
schema, not lifecycle authority, not registry authority, not trust
authority, not distribution authority, not evidence acceptance, not
implementation authority, not runtime activation, not general runtime
authority, not package installation, not package execution, not module
loading, not workspace runtime, not plugin loading, not capability token
minting, not capability issuance, not registry publication, not trust
assignment, not Semantic CLI authority, not AI Runtime authority, not agent
authority, not syscall expansion, not kernel ABI expansion, not
workflow-threshold, baseline, dependency, or Ring0 authority.

## Purpose

This document defines the Phase-20 identity model for AykenOS capability
records.

It answers one question:

```text
How is a capability uniquely identified?
```

It does not answer:

```text
How is a capability manifest serialized?
How is a capability lifecycle state changed?
How is a capability stored in a registry?
How is a capability trusted, distributed, accepted, implemented, or run?
```

Those questions belong to later Phase-20 RFCs.

## Core Rule

```text
capability identity != capability issuance
capability identity != registry publication
capability identity != manifest schema
capability identity != lifecycle transition
capability identity != trust assignment
capability identity != implementation authority
capability identity != runtime activation
```

Identity makes a capability distinguishable. It does not make the capability
active, trusted, published, accepted, implemented, executable, loadable, or
authority-bearing.

Unknown authority readings fail closed.

## Identity Mission

The mission of capability identity is to give every capability record a
stable, deterministic, reviewable way to be distinguished from every other
capability record.

Capability identity exists so later RFCs can safely refer to a capability
without re-defining what object is being discussed.

Identity supports later:

1. Manifest references.
2. Lifecycle review.
3. Registry indexing.
4. Trust input association.
5. Distribution policy review.
6. Evidence attachment.
7. Acceptance workflow.
8. Bounded implementation decisions.

The identity record itself grants none of those authorities.

## Identity Definition

A capability identity is the canonical identity tuple that distinguishes one
inert capability record from another.

The canonical identity tuple is conceptually:

```text
(identity_domain, namespace, logical_name, version_identity, digest_binding)
```

This tuple is conceptual. It is not a manifest syntax, registry key format,
wire format, file path, package name, module name, plugin name, token, trust
claim, loader input, or runtime handle.

Later RFCs may define serialization rules. Until then, identity-specific
serialization readings fail closed.

## Identity Principles

Every capability identity rule must preserve these principles:

1. **Stable:** identity must not change silently after review.
2. **Deterministic:** the same reviewed identity inputs must resolve to the
   same identity result.
3. **Reviewable:** identity inputs must be explicit enough for governance
   review.
4. **Immutable by default:** changing identity-defining fields creates a
   different identity unless a later reviewed RFC defines a narrower
   exception.
5. **Collision-safe:** ambiguous or colliding identities fail closed.
6. **Alias-denying by default:** alternate names do not transfer identity or
   authority unless later reviewed records explicitly define alias behavior.
7. **Version-aware:** version identity is part of identity, not incidental
   metadata.
8. **Digest-ready:** identity must support later digest binding without
   choosing a digest algorithm in this RFC.
9. **Authority-denying:** identity does not imply acceptance, publication,
   trust, issuance, implementation, execution, or runtime authority.

## Identity Components

The identity tuple has five conceptual components:

| Component | Purpose | Authority result |
|---|---|---|
| Identity domain | Identifies the identity kind as a capability identity | No runtime authority |
| Namespace | Groups capability names under a governed naming context | No ownership or trust assignment |
| Logical name | Names the capability concept inside a namespace | No execution or publication |
| Version identity | Distinguishes capability versions | No lifecycle transition |
| Digest binding | Binds identity to later canonical material | No evidence acceptance |

All components must be present before a capability identity can be treated as
complete. Missing, ambiguous, stale, inherited, or differently scoped
components fail closed.

## Identity Domain

The identity domain distinguishes capability identity from other future
identity kinds.

For Phase-20 capability work, the identity domain is conceptually:

```text
capability
```

This domain label only states what kind of object is being identified. It
does not define a registry namespace, package type, loader domain, Semantic
CLI command, AI Runtime object, agent object, kernel object, or execution
domain.

## Namespace

A namespace is a governed naming context for capability logical names.

A namespace must be:

1. Explicit.
2. Stable after review.
3. Deterministic under comparison.
4. Non-empty.
5. Fail-closed when ambiguous.

A namespace does not grant:

1. Ownership.
2. Trust.
3. Publication rights.
4. Registry authority.
5. Package authority.
6. Runtime authority.
7. Capability issuance authority.

Namespace ownership, delegation, reservation, transfer, and conflict policy
are not defined here. They require later reviewed governance records.

## Logical Name

A logical name identifies the capability concept within a namespace.

The logical name must be stable enough for review and deterministic enough
for comparison.

A logical name is not:

1. A package name.
2. A module name.
3. A plugin name.
4. A registry publication.
5. A command name.
6. A syscall name.
7. A runtime handle.
8. A trust claim.
9. A distribution channel.

Changing a logical name changes the identity unless a later reviewed RFC
defines a narrower alias or supersession rule.

## Version Identity

Version identity distinguishes capability versions.

Version identity is part of capability identity. It is not incidental
metadata.

Two capability records with the same namespace and logical name but different
version identities are different capability identities.

This RFC does not define:

1. Version syntax.
2. Version ordering.
3. Compatibility ranges.
4. Upgrade behavior.
5. Downgrade behavior.
6. Supersession behavior.
7. Distribution channel selection.

Those topics belong to later lifecycle, registry, distribution, and
acceptance RFCs.

## Digest Binding

Digest binding reserves the identity model's link to later canonical
capability material.

The digest binding exists so a capability identity can later be tied to the
exact reviewed material that defines the capability record.

This RFC does not define:

1. Digest algorithm.
2. Canonical byte representation.
3. Manifest serialization.
4. Registry storage format.
5. Evidence package format.
6. Signature format.
7. Trust proof format.

Until later RFCs define those details, digest-specific interpretation fails
closed.

Digest presence does not prove trust, acceptance, publication,
implementation correctness, runtime safety, or execution authority.

## Canonical Capability Identifier

The canonical capability identifier is the deterministic identity result
derived from the identity tuple:

```text
(identity_domain, namespace, logical_name, version_identity, digest_binding)
```

The canonical identifier is a governance object, not an execution object.

It may later be serialized by `PHASE20_CAPABILITY_MANIFEST_SCHEMA.md` or
referenced by registry RFCs, but this document does not define that
serialization or registry representation.

No canonical identifier may be treated as:

1. A package locator.
2. A module locator.
3. A plugin locator.
4. A workspace path.
5. A URL.
6. A command.
7. A token.
8. A trust assertion.
9. A runtime grant.

## Identity Comparison

Capability identity comparison is conceptual and exact.

Two capability identities are the same only if all identity tuple components
match under the rules accepted by this RFC and later narrower RFCs.

Comparison must fail closed when:

1. A component is missing.
2. A component is ambiguous.
3. A component is inherited from a different subject without review.
4. A component uses an unknown normalization rule.
5. A component relies on an unaccepted digest interpretation.
6. A component relies on an unaccepted alias.
7. A component relies on an unaccepted version compatibility rule.

Approximate matching, fuzzy matching, implicit aliasing, trust-based
matching, registry-presence matching, or runtime-observed matching is denied.

## Identity Normalization Boundary

This RFC does not define case folding, Unicode normalization, whitespace
normalization, separator normalization, locale-specific comparison, or
human-readable display-name equivalence.

Until a later reviewed RFC defines canonical normalization rules, identity
comparison must treat unknown, mixed, inherited, or differently normalized
inputs as ambiguous and fail closed.

Normalization must not be used to:

1. Merge two identities.
2. Preserve identity across rename.
3. Create an alias.
4. Resolve a collision.
5. Carry over evidence.
6. Carry over trust inputs.
7. Carry over registry publication.
8. Carry over implementation or runtime authority.

## Immutability Boundary

Identity-defining fields are immutable by default after review.

The following changes create a different identity unless a later reviewed RFC
defines a narrower rule:

1. Identity domain change.
2. Namespace change.
3. Logical name change.
4. Version identity change.
5. Digest binding change.

Immutability does not mean the capability is accepted, published,
implemented, trusted, distributed, or executable. It only means the identity
is stable for governance reference.

## Rename Boundary

Rename is not identity preservation by default.

Changing a namespace or logical name creates a new identity unless a later
reviewed RFC defines explicit alias or supersession behavior.

A rename must not be used to:

1. Bypass review.
2. Carry over evidence.
3. Carry over trust inputs.
4. Carry over registry publication.
5. Carry over acceptance.
6. Carry over implementation authority.
7. Carry over runtime authority.

Unknown rename readings fail closed.

## Collision Boundary

An identity collision exists when two different capability records appear to
resolve to the same canonical capability identifier.

Collision handling is fail-closed:

1. Neither colliding record receives authority from the collision.
2. Registry publication remains denied.
3. Trust assignment remains denied.
4. Evidence inheritance remains denied.
5. Implementation authority remains denied.
6. Runtime activation remains denied.

Later RFCs may define collision reporting and quarantine procedures. They
must not turn collision into authority.

## Alias And Supersession Boundary

Alias and supersession are not defined by this RFC.

An alias is any alternate reference that claims to point to the same
capability identity.

Supersession is any claim that one capability identity replaces another.

Both are denied by default until later reviewed RFCs define exact rules.

An alias or supersession claim must not transfer:

1. Evidence.
2. Acceptance.
3. Trust.
4. Registry publication.
5. Distribution status.
6. Implementation authority.
7. Runtime authority.
8. Capability issuance authority.

## Relationship To Capability Subject And Scope

`PHASE20_CAPABILITY_MODEL.md` distinguishes capability subject from
capability scope:

```text
subject -> the future behavior being described
scope   -> the bounded domain and limits of that behavior
```

Capability identity identifies the capability record that describes that
subject and scope.

Identity must not erase the subject/scope distinction. Two records that
claim the same subject but different scope must not be treated as equivalent
unless later reviewed RFCs define exact equivalence rules.

## Relationship To Later RFCs

Capability identity is a prerequisite for later Phase-20 RFCs.

| Later RFC | Identity relationship |
|---|---|
| `PHASE20_CAPABILITY_MANIFEST_SCHEMA.md` | Serializes identity references without granting loader or execution authority. |
| `PHASE20_CAPABILITY_LIFECYCLE.md` | Uses identity to define state ownership and transition subject. |
| `PHASE20_REGISTRY_MODEL.md` | Uses identity as a registry reference target without granting publication. |
| `PHASE20_REGISTRY_GOVERNANCE.md` | Uses identity to separate admission from publication. |
| `PHASE20_TRUST_MODEL.md` | Attaches trust inputs to identity without assigning trust. |
| `PHASE20_DISTRIBUTION_POLICY.md` | References identity for publication and revocation policy without granting distribution. |
| `PHASE20_CAPABILITY_EVIDENCE_MODEL.md` | Binds evidence to exact identity and subject without evidence inheritance. |
| `PHASE20_CAPABILITY_ACCEPTANCE_WORKFLOW.md` | Uses identity for review and merge tracking without implementation authority. |

Later RFCs may narrow identity use. They must not broaden identity into
active authority without a separate reviewed decision.

## Identity Invariants

Every later Phase-20 RFC must preserve these identity invariants:

1. Capability identity is inert.
2. Capability identity is exact, not approximate.
3. Capability identity is stable after review.
4. Capability identity is version-aware.
5. Capability identity is digest-ready but not digest-authoritative until
   later canonicalization rules exist.
6. Capability identity does not imply manifest validity.
7. Capability identity does not imply lifecycle acceptance.
8. Capability identity does not imply registry publication.
9. Capability identity does not imply trust assignment.
10. Capability identity does not imply distribution.
11. Capability identity does not imply evidence inheritance.
12. Capability identity does not imply implementation authority.
13. Capability identity does not imply runtime activation.

Violation of any invariant fails closed.

## Non-Goals

This document does not define or authorize:

1. Manifest syntax.
2. Manifest parser behavior.
3. Registry key serialization.
4. Registry storage implementation.
5. Lifecycle state machine.
6. Alias implementation.
7. Supersession implementation.
8. Trust model.
9. Distribution policy.
10. Evidence package format.
11. Acceptance workflow.
12. Source implementation.
13. Runtime activation.
14. Package installation, loading, execution, scheduling, or publication.
15. Module loading.
16. Workspace creation, workspace runtime, or real mounts.
17. Plugin host, plugin loading, or plugin instantiation.
18. Capability token minting or capability issuance.
19. Registry publication or marketplace behavior.
20. Trust assignment or trust issuer behavior.
21. Semantic CLI execution or verdict authority.
22. AI Runtime authority.
23. Agent behavior.
24. New syscalls.
25. Kernel ABI expansion.
26. Workflow-threshold, baseline, dependency, or Ring0 policy changes.

## Implementation Gating

No Phase-20 implementation slice may start from this identity RFC alone.

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

Identity is not implementation authority.

## Explicit Non-Authorization

This identity RFC does not authorize:

1. Runtime activation.
2. General runtime authority.
3. Phase-20 implementation.
4. Source acceptance or merge authority.
5. Manifest schema acceptance.
6. Lifecycle state transition authority.
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

## Identity Conclusion

An AykenOS capability identity is the stable, deterministic, reviewable
identity tuple that distinguishes one inert capability record from another.

Capability identity supports later manifest, lifecycle, registry, trust,
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

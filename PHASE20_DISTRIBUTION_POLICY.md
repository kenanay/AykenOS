# Phase-20 Distribution Policy

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
`PHASE20_CAPABILITY_MANIFEST_SCHEMA.md`,
`PHASE20_CAPABILITY_LIFECYCLE.md`,
`PHASE20_REGISTRY_MODEL.md`,
`PHASE20_REGISTRY_GOVERNANCE.md`, and
`PHASE20_TRUST_MODEL.md`. In case of conflict, those documents prevail
unless this distribution policy RFC is the narrower Phase-20 distribution
policy record for the exact planning scope identified below.

**Status:** PHASE-20 DISTRIBUTION POLICY RFC / DISTRIBUTION POLICY MODEL
ONLY / NO REGISTRY PUBLICATION / NO PUBLICATION AUTHORITY / NO DISTRIBUTION
AUTHORITY / NO PACKAGE INSTALLATION / NO PACKAGE LOADING / NO PACKAGE
EXECUTION / NO IMPLEMENTATION AUTHORITY / NO RUNTIME ACTIVATION / NO
GENERAL RUNTIME AUTHORITY / NO TRUST ASSIGNMENT / NO EVIDENCE ACCEPTANCE /
NO CAPABILITY ISSUANCE
**Distribution policy date:** 2026-06-29
**Distribution policy id:** `ayken.phase20.distribution_policy.v1`
**Distribution policy base main SHA:** `f678c68d3abd7d6491f07a02a2a240464458527a`
**Current phase pointer:** `CURRENT_PHASE=20`
**Authority boundary:** Distribution policy model only; not registry
publication, not publication authority, not distribution authority, not
distribution execution, not transport protocol, not package repository, not
package installation, not package loading, not package execution, not
installer behavior, not deployment behavior, not trust assignment, not
evidence acceptance, not implementation authority, not runtime activation,
not general runtime authority, not module loading, not workspace runtime,
not plugin loading, not capability token minting, not capability issuance,
not Semantic CLI authority, not AI Runtime authority, not agent authority,
not syscall expansion, not kernel ABI expansion, not workflow-threshold,
baseline, dependency, or Ring0 authority.

## Purpose

This document defines the Phase-20 distribution policy model for AykenOS
capability and registry governance records.

It answers one question:

```text
Under what governance conditions may a capability or registry record become
eligible for distribution?
```

It does not answer:

```text
How is a record published?
How is a package built, installed, loaded, executed, or transported?
How is trust assigned?
How is evidence accepted?
How is a capability issued, implemented, activated, deployed, or run?
```

Those questions belong to later Phase-20 RFCs and implementation decisions.

## Core Rule

```text
distribution policy != registry publication
distribution eligibility != publication authority
distribution eligibility != distribution authority
distribution request != distribution
distribution decision != distribution execution
distribution policy != transport protocol
distribution policy != package repository
distribution != package installation
distribution != package loading
distribution != package execution
distribution != runtime activation
distribution != capability issuance
distribution != implementation authority
distribution != trust assignment
distribution != evidence acceptance
registry acceptance != publication
trust context != publication approval
evidence presence != publication approval
```

Distribution policy makes distribution eligibility conditions reviewable. It
does not publish, distribute, transport, install, load, execute, deploy,
activate, issue, trust, accept evidence for, or implement anything.

Unknown authority readings fail closed.

## Distribution Mission

The mission of the Phase-20 distribution policy model is to define inert,
reviewable conditions for future distribution eligibility.

Distribution policy exists so later RFCs can reason about:

1. Distribution subjects.
2. Distribution eligibility.
3. Distribution inputs.
4. Distribution constraints.
5. Publication boundaries.
6. Registry relationships.
7. Trust relationships.
8. Evidence relationships.
9. Quarantine inputs.
10. Revocation inputs.
11. Rollback inputs.
12. Later publication workflow and implementation prerequisites.

The distribution policy model itself grants no publication, distribution,
transport, package, installer, deployment, trust, evidence, implementation,
runtime, or capability issuance authority.

Each later use requires its own reviewed RFC or decision path.

## Distribution Definition

Distribution policy is the governance model for determining whether a
capability or registry record may become eligible for later distribution
review.

Distribution policy may describe:

1. The exact distribution subject.
2. Required registry governance context.
3. Required trust context.
4. Required evidence context.
5. Required lifecycle context.
6. Required quarantine, revocation, or rollback inputs.
7. Denied authority readings.
8. Later decision dependencies.

Distribution policy is not:

1. Registry publication.
2. Publication authority.
3. Distribution authority.
4. Distribution execution.
5. Transport protocol.
6. Package repository behavior.
7. Package installation.
8. Package loading.
9. Package execution.
10. Installer behavior.
11. Deployment behavior.
12. Trust assignment.
13. Evidence acceptance.
14. Capability issuance.
15. Source acceptance.
16. Implementation authority.
17. Runtime activation.
18. Semantic CLI, AI Runtime, or agent authority.

## Distribution Scope

This RFC defines only the distribution policy model.

It does not define:

1. Publication workflow.
2. Registry publication.
3. Packaging format.
4. Package repository.
5. Transport protocol.
6. Installer behavior.
7. Deployment mechanism.
8. Repository implementation.
9. Mirror implementation.
10. Marketplace behavior.
11. Runtime behavior.
12. Execution behavior.

Distribution policy is a governance policy layer. It is not a distribution
engine, package manager, installer, loader, mirror service, transport
service, deployment service, or runtime service.

Any publication-specific, packaging-specific, transport-specific,
installer-specific, runtime-specific, repository-specific, or
deployment-specific interpretation fails closed until later reviewed RFCs
define exact behavior.

## Distribution Subject

A distribution subject is the exact immutable governance subject being
evaluated for distribution eligibility.

A distribution subject may include:

1. Exact capability identity.
2. Manifest reference.
3. Registry reference.
4. Registry governance context.
5. Trust context, if required.
6. Evidence context, if required by later RFCs.
7. Lifecycle context.

A distribution subject is not:

1. A package.
2. A binary.
3. An executable artifact.
4. An installer.
5. A runtime object.
6. A repository object.
7. A transport object.
8. A deployment target.

Changing identity, manifest scope, registry subject, lifecycle context, or
other subject-defining material creates a different distribution subject
unless a later reviewed RFC defines exact narrower behavior.

## Distribution Subject Identity

Distribution policy is bound to exact capability identity and accepted
governance context.

The conceptual identity chain is:

```text
capability_identity
  -> manifest_reference
  -> registry_reference
  -> registry_governance_context
  -> trust_context
  -> distribution_policy_review
```

This chain is conceptual. It does not define serialization, package layout,
repository layout, transport layout, publication workflow, distribution
execution, or runtime behavior.

Any missing, ambiguous, stale, inherited, aliased, superseded, or
differently scoped identity link fails closed.

## Distribution Principles

Every distribution policy rule must preserve these principles:

1. **Policy-only:** distribution policy describes eligibility conditions; it
   does not execute distribution.
2. **Identity-bound:** distribution policy applies to exact capability or
   registry subjects and must not silently transfer across identity changes.
3. **Registry-aware:** registry governance may be policy context; it is not
   publication.
4. **Trust-aware:** trust context may be policy input; it is not
   distribution approval.
5. **Evidence-aware:** evidence references may be policy input; they are not
   evidence acceptance or distribution approval.
6. **Lifecycle-aware:** lifecycle state may constrain policy; it is not
   distribution authority.
7. **Implementation-independent:** distribution policy must not depend on a
   specific package manager, transport protocol, installer, loader, or
   runtime.
8. **Deterministic:** the same reviewed policy inputs must produce the same
   eligibility reading.
9. **Audit-preserving:** quarantine, revocation, rollback, rejection, and
   distribution decisions must remain reviewable.
10. **Fail-closed:** missing, ambiguous, stale, inherited, colliding, or
    differently scoped distribution material fails closed.

## Distribution Eligibility

Distribution eligibility is a reviewable policy posture for a distribution
subject.

A distribution subject may become eligible for later distribution review
only when all required policy inputs are present, exact, consistent, and
accepted by the governing records that define them.

Eligibility may require:

1. Exact capability identity.
2. Accepted manifest reference, if applicable.
3. Accepted lifecycle context.
4. Accepted registry governance context.
5. Trust context, if required.
6. Evidence context, if required.
7. Quarantine absence or quarantine resolution context.
8. Revocation absence or revocation resolution context.
9. Rollback input context, if applicable.
10. Non-authorization notice.

Distribution eligibility is not publication authority, distribution
authority, transport authority, package authority, installation authority,
execution authority, trust assignment, evidence acceptance, implementation
authority, capability issuance, or runtime activation.

## Distribution Inputs

Distribution inputs are policy-relevant governance records used to evaluate
distribution eligibility.

Distribution inputs may include:

1. Capability identity references.
2. Manifest references.
3. Lifecycle state references.
4. Registry record references.
5. Registry governance decision records.
6. Trust context records.
7. Evidence references.
8. Quarantine inputs.
9. Revocation inputs.
10. Rollback inputs.
11. Audit records.
12. Later publication workflow references.

Distribution input presence must not be interpreted as publication,
distribution, trust assignment, evidence acceptance, package installation,
package loading, package execution, implementation authority, capability
issuance, or runtime activation.

## Distribution Constraints

Distribution policy constraints are the rules that must hold before
distribution eligibility may be read.

Distribution policy must be:

1. Deterministic under accepted policy inputs.
2. Exact-subject bound.
3. Implementation-independent.
4. Transport-independent.
5. Package-format-independent.
6. Runtime-independent.
7. Registry-publication-denying.
8. Trust-assignment-denying.
9. Evidence-acceptance-denying.
10. Capability-issuance-denying.
11. Authority-denying by default.
12. Fail-closed on ambiguity.

Distribution policy must not:

1. Change capability identity.
2. Change manifest scope.
3. Execute lifecycle transitions.
4. Publish registry records.
5. Assign trust.
6. Accept evidence.
7. Select a package transport.
8. Install, load, execute, or deploy packages.
9. Change runtime state.
10. Authorize implementation.

Any constraint violation fails closed.

## Publication Boundary

Distribution policy does not define publication.

Publication is a later governance or implementation path that may define
how a specific record becomes published, mirrored, released, listed, or
made available.

This RFC does not define:

1. Publication workflow.
2. Publication approval.
3. Registry publication.
4. Mirror publication.
5. Release artifact creation.
6. Package repository publication.
7. Marketplace listing.
8. Publication revocation.
9. Publication rollback.

Distribution eligibility must not be interpreted as publication authority.

Registry acceptance, trust context, evidence presence, lifecycle state,
reviewer finding, maintainer decision, policy input, or distribution
decision must not publish a record by implication.

## Publication And Distribution Separation

Publication, distribution, installation, and execution are separate
concepts.

| Concept | Meaning in this RFC | Authority result |
|---|---|---|
| Publication | Later visibility or listing decision | Not defined here |
| Distribution eligibility | Policy posture for later distribution review | No publication authority |
| Distribution execution | Later mechanics of making material available | Not defined here |
| Installation | Local placement or deployment of material | No installer authority |
| Loading | Making material available to a loader or runtime | No loader authority |
| Execution | Runtime behavior | No runtime authority |

No concept in this table implies another by default.

Unknown publication, distribution, installation, loading, or execution
readings fail closed.

## Mirror Boundary

A mirror is not defined by this RFC.

Any later mirror model must preserve the following default readings:

```text
mirror != publication
mirror != distribution authority
mirror != trust assignment
mirror != evidence acceptance
mirror != runtime authority
```

Mirror eligibility, mirror synchronization, mirror retention, mirror
revocation, mirror rollback, and mirror trust behavior belong to later
reviewed RFCs or decisions.

## Registry Relationship

`PHASE20_REGISTRY_MODEL.md` defines registry records as inert governance
records.

`PHASE20_REGISTRY_GOVERNANCE.md` defines accepted registry governance state
as explicit admission governance, not publication.

Distribution policy may use registry records and accepted registry
governance state as policy inputs.

It must not treat registry presence, registry admission, registry
acceptance, registry index presence, registry view presence, registry
snapshot presence, or registry governance decision presence as publication,
distribution, trust assignment, package authority, implementation authority,
or runtime activation.

Registry ambiguity, collision, quarantine, rejection, stale reference, or
differently scoped registry material fails closed for distribution policy.

## Trust Relationship

`PHASE20_TRUST_MODEL.md` defines trust inputs, trust claims, proof
references, trust context, and assessment classes without assigning trust.

Distribution policy may use trust context and trust inputs as policy inputs.

It must not treat trust input presence, trust claim presence, proof
reference presence, signature presence, trust context, assessment class, or
trust dispute record as publication approval, distribution authority, trust
assignment, evidence acceptance, implementation authority, capability
issuance, or runtime activation.

Trust ambiguity, dispute, revocation input, stale trust input, unknown
issuer interpretation, unknown proof interpretation, or differently scoped
trust material fails closed for distribution policy.

## Evidence Relationship

Evidence may become a distribution policy input only after later evidence
RFCs define evidence format, exact-subject binding, review requirements,
and acceptance behavior.

This RFC does not accept evidence.

Evidence presence, evidence reference, evidence digest, evidence package,
test result, signature material, audit note, or post-merge result must not
be interpreted as publication approval, distribution authority, trust
assignment, proof acceptance, implementation authority, capability issuance,
or runtime activation by this RFC.

Evidence inheritance is denied.

## Quarantine / Revocation / Rollback Inputs

Quarantine, revocation, and rollback inputs are policy-relevant governance
material for later review.

A quarantine input may indicate unresolved ambiguity, collision, dispute,
scope concern, trust concern, evidence concern, registry concern,
publication concern, distribution concern, or safety concern.

A revocation input may indicate later review should consider whether a
distribution subject should stop being eligible for future distribution
paths.

A rollback input may indicate later review should consider whether a
publication, distribution, mirror, or release path should be reversed by a
later reviewed procedure.

Quarantine, revocation, or rollback input presence does not:

1. Prove fault.
2. Publish a record.
3. Distribute a record.
4. Revoke publication by itself.
5. Execute rollback by itself.
6. Delete history.
7. Transfer authority to another record.
8. Establish alias or supersession.
9. Accept evidence.
10. Assign trust.
11. Activate runtime behavior.

Quarantine handling, revocation proof requirements, rollback procedures,
publication effects, distribution effects, notification, mirror behavior,
and repository behavior belong to later RFCs.

## Rollback Boundary

Rollback input concerns governance review material.

Rollback does not imply:

1. Package removal.
2. Runtime rollback.
3. State rollback.
4. Execution rollback.
5. Registry deletion.
6. Evidence deletion.
7. Trust revocation by itself.
8. Publication revocation by itself.

Rollback procedures, rollback authority, rollback scope, rollback evidence,
and rollback verification belong to later reviewed RFCs or decisions.

## Distribution Decision Boundary

A distribution decision is not distribution execution.

A later reviewed decision may state that a subject is eligible or approved
for a later distribution path, but that decision must not be interpreted as
package installation, package loading, package execution, transport,
deployment, runtime activation, capability issuance, or implementation
authority unless a separate reviewed decision explicitly grants that exact
bounded behavior.

This RFC does not define distribution decision authority.

It only reserves the model distinction between:

```text
distribution request
  -> distribution policy review
  -> distribution eligibility decision
  -> later publication or distribution workflow
```

Every arrow is a governance dependency. No arrow implies execution,
publication, installation, loading, distribution, issuance, or runtime
activation.

Distribution decision records eligibility only.

It never records execution.

## Distribution Validation

Distribution policy validation is conceptual and fail-closed.

Distribution material is invalid for governance review when:

1. Distribution subject is missing or ambiguous.
2. Capability identity is missing or ambiguous.
3. Manifest scope is missing, stale, ambiguous, or differently scoped.
4. Lifecycle context is missing or ambiguous when required.
5. Registry governance context is missing or ambiguous when required.
6. Trust context relies on trust assignment not accepted by a later RFC.
7. Evidence context relies on unaccepted evidence semantics.
8. Publication context relies on undefined publication rules.
9. Transport context relies on undefined transport rules.
10. Package context relies on undefined package rules.
11. Runtime context relies on runtime-observed state.
12. Distribution material relies on alias or supersession without accepted
    rules.
13. Distribution material implies trust assignment.
14. Distribution material implies evidence acceptance.
15. Distribution material implies implementation or runtime authority.

Validation failure grants no authority. It requires correction, rejection,
quarantine, dispute recording, or a later reviewed decision path.

## Distribution Invariants

Every later Phase-20 RFC must preserve these distribution invariants:

1. Distribution policy is not registry publication.
2. Distribution eligibility is not publication authority.
3. Distribution eligibility is not distribution authority.
4. Distribution request is not distribution.
5. Distribution decision is not distribution execution.
6. Distribution policy is implementation-independent.
7. Distribution policy is transport-independent.
8. Distribution policy is runtime-independent.
9. Registry acceptance is not publication.
10. Trust context is not publication approval.
11. Evidence presence is not publication approval.
12. Distribution material does not imply trust assignment.
13. Distribution material does not imply evidence acceptance.
14. Distribution material does not imply capability issuance.
15. Distribution material does not imply package installation, loading, or
    execution.
16. Distribution material does not imply implementation authority.
17. Distribution material does not imply runtime activation.

Violation of any invariant fails closed.

## Later RFC Dependencies

The distribution policy model is a prerequisite for later Phase-20 RFCs and
decision paths.

| Later record | Distribution policy relationship |
|---|---|
| `PHASE20_CAPABILITY_EVIDENCE_MODEL.md` | Defines evidence binding and evidence package requirements without evidence inheritance. |
| `PHASE20_CAPABILITY_ACCEPTANCE_WORKFLOW.md` | Uses distribution policy inputs for review tracking without implementation authority. |
| Publication workflow | May define publication request, review, decision, and post-decision verification without implying runtime authority. |
| Distribution implementation | May define bounded distribution mechanics only after separate reviewed implementation authority. |
| Runtime interaction | May define runtime effects only after a separate reviewed runtime decision, if ever authorized. |

Later RFCs may narrow distribution policy use. They must not broaden this
policy model into registry publication, distribution execution, package
installation, trust assignment, evidence acceptance, implementation,
issuance, or runtime authority without a separate reviewed decision.

The current Phase-20 dependency chain is:

```text
Capability Model
  -> Capability Identity
  -> Capability Manifest Schema
  -> Capability Lifecycle
  -> Registry Model
  -> Registry Governance
  -> Trust Model
  -> Distribution Policy
  -> Evidence Model
  -> Acceptance Workflow
  -> Implementation Decision
```

Every arrow means a governance dependency. It does not imply publication,
distribution, installation, execution, issuance, or runtime activation.

## Explicit Non-Authorization

This distribution policy RFC does not authorize:

1. Runtime activation.
2. General runtime authority.
3. Phase-20 implementation.
4. Source acceptance or merge authority.
5. Registry publication.
6. Publication authority.
7. Distribution authority.
8. Distribution execution.
9. Transport protocol behavior.
10. Package repository behavior.
11. Package installation, loading, execution, scheduling, or publication.
12. Installer behavior.
13. Deployment behavior.
14. Trust assignment.
15. Trust issuer authority.
16. Signature acceptance.
17. Authenticity acceptance.
18. Evidence acceptance or evidence inheritance.
19. Capability issuance.
20. Module loading.
21. Workspace creation, workspace runtime, or real mounts.
22. Plugin host, plugin loading, or plugin instantiation.
23. Capability token minting.
24. Semantic CLI execution or verdict authority.
25. AI Runtime authority.
26. Agent behavior.
27. New syscalls.
28. Kernel ABI expansion.
29. Workflow-threshold, baseline, dependency, or Ring0 policy changes.
30. Observability-as-authority.

Unknown authority readings fail closed.

## Architecture Signature

**Prepared by:** Kenan AY
**Role:** AykenOS Architecture Steward
**Document type:** Phase-20 RFC
**Architecture status:** Draft RFC / pending architectural review
**Authority notice:** This signature identifies the architectural authorship
of this RFC. It grants no runtime authority, implementation authority, trust
authority, execution authority, constitutional authority, registry authority,
distribution authority, publication authority, evidence authority,
capability issuance authority, package authority, module authority, plugin
authority, Semantic CLI authority, AI Runtime authority, agent authority, or
Ring0 authority.

## Non-Goals

This document does not define or authorize:

1. Registry publication.
2. Publication workflow.
3. Publication approval.
4. Distribution authority.
5. Distribution execution.
6. Transport protocol.
7. Package format.
8. Package repository.
9. Package installation.
10. Package loading.
11. Package execution.
12. Installer behavior.
13. Deployment behavior.
14. Mirror implementation.
15. Marketplace behavior.
16. Trust assignment.
17. Trust issuer authority.
18. Signature verification.
19. Signature acceptance.
20. Evidence package format.
21. Evidence acceptance.
22. Evidence inheritance.
23. Acceptance workflow.
24. Source implementation.
25. Runtime activation.
26. Module loading.
27. Workspace creation, workspace runtime, or real mounts.
28. Plugin host, plugin loading, or plugin instantiation.
29. Capability token minting or capability issuance.
30. Semantic CLI execution or verdict authority.
31. AI Runtime authority.
32. Agent behavior.
33. New syscalls.
34. Kernel ABI expansion.
35. Workflow-threshold, baseline, dependency, or Ring0 policy changes.
36. Binary format.
37. Artifact storage.

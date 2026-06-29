# Phase-20 Implementation Decision

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
`PHASE20_REGISTRY_GOVERNANCE.md`,
`PHASE20_TRUST_MODEL.md`,
`PHASE20_DISTRIBUTION_POLICY.md`,
`PHASE20_CAPABILITY_EVIDENCE_MODEL.md`, and
`PHASE20_CAPABILITY_ACCEPTANCE_WORKFLOW.md`. In case of conflict, those
documents prevail unless this implementation decision RFC is the narrower
Phase-20 implementation decision record for the exact planning scope
identified below.

**Status:** PHASE-20 IMPLEMENTATION DECISION RFC / IMPLEMENTATION DECISION
RECORD MODEL ONLY / GOVERNANCE ELIGIBILITY ONLY / NO IMPLEMENTATION / NO
IMPLEMENTATION SLICE AUTHORITY / NO SOURCE MERGE AUTHORITY / NO SOURCE
ACCEPTANCE / NO RUNTIME ACTIVATION / NO GENERAL RUNTIME AUTHORITY / NO
PACKAGE INSTALLATION / NO PACKAGE LOADING / NO PACKAGE EXECUTION / NO
CAPABILITY ISSUANCE / NO TRUST ASSIGNMENT / NO REGISTRY PUBLICATION / NO
DISTRIBUTION AUTHORITY
**Implementation decision date:** 2026-06-30
**Implementation decision id:** `ayken.phase20.implementation_decision.v1`
**Implementation decision base main SHA:** `9e12308d358a8ed66e8af625d5994697b66a8a31`
**Current phase pointer:** `CURRENT_PHASE=20`
**Authority boundary:** Implementation decision record model only;
governance eligibility only; not implementation, not implementation slice
authority, not implementation execution, not source acceptance, not source
merge authority, not implementation authority, not runtime activation, not
general runtime authority, not package installation, not package loading,
not package execution, not module loading, not workspace runtime, not
plugin loading, not capability token minting, not capability issuance, not
trust assignment, not trust issuer authority, not registry authority, not
registry publication, not publication authority, not distribution
authority, not distribution execution, not Semantic CLI authority, not AI
Runtime authority, not agent authority, not syscall expansion, not kernel
ABI expansion, not workflow-threshold, baseline, dependency, or Ring0
authority.

## Purpose

This document defines the Phase-20 implementation decision record model for
accepted workflow subjects.

It answers one question:

```text
Under what governance conditions may an accepted workflow subject become
eligible for later bounded Implementation Slice review?
```

It does not answer:

```text
How is implementation performed?
How is source merged?
What source scope is implemented?
How is runtime activated?
How is a package installed, loaded, executed, or distributed?
How is a capability issued?
```

Those questions belong to later Phase-20 RFCs and implementation decisions.

## Core Rule

```text
implementation decision != implementation
implementation decision != implementation slice
implementation decision != runtime activation
implementation decision != source merge
implementation decision != package execution
implementation decision != capability issuance
accepted evidence set != implementation authority
accepted workflow subject != implementation authority
implementation decision record != implementation execution
eligible != implementation approved
eligible != implementation scheduled
eligibility != authority
decision subject != implementation subject
```

Decision records governance eligibility only.

A decision record is not an implementation artifact.

A decision record never authorizes source changes, runtime behavior,
capability execution, package execution, source merge, distribution,
capability issuance, or authority expansion.

Unknown authority readings fail closed.

## Implementation Decision Mission

The mission of the Phase-20 implementation decision model is to define an
explicit, auditable governance decision record for determining whether an
accepted workflow subject may proceed to later bounded Implementation Slice
review.

Implementation decision exists so later RFCs can reason about:

1. Decision subjects.
2. Accepted workflow subject prerequisites.
3. Accepted evidence set prerequisites.
4. Eligibility properties.
5. Implementation decision reviews.
6. Implementation decision records.
7. Decision outcomes.
8. Bounded implementation prerequisites.
9. Immutable decision history.
10. Later Implementation Slice dependencies.

The implementation decision model itself grants no implementation,
implementation slice authority, source merge authority, runtime,
distribution, package, trust, registry, or capability issuance authority.

Each later use requires its own reviewed RFC or decision path.

## Implementation Decision Definition

An implementation decision is a governance record that determines whether
an accepted workflow subject is eligible for later bounded Implementation
Slice review.

An implementation decision may describe:

1. The exact decision subject.
2. The accepted workflow subject.
3. The accepted evidence set.
4. The acceptance decision record.
5. The implementation decision review.
6. The decision outcome.
7. Eligibility constraints.
8. Later Implementation Slice dependency.
9. Denied authority readings.
10. Non-authorization notice.

An implementation decision is not:

1. Implementation.
2. Implementation slice.
3. Source acceptance.
4. Source merge authority.
5. Implementation approval.
6. Runtime activation.
7. Package installation, loading, or execution.
8. Registry publication.
9. Distribution authority.
10. Trust assignment.
11. Capability issuance.
12. Semantic CLI, AI Runtime, or agent authority.

## Implementation Decision Scope

This RFC defines only the implementation decision record model.

It does not define implementation mechanics, source scope, source merge
procedure, repository branch protection, package format, package
repository, artifact storage, binary format, runtime behavior, registry
publication, distribution execution, trust assignment, capability
issuance, module loading, plugin loading, or workspace runtime.

Implementation decision is a governance decision layer. It is not an
implementation gate, source merge engine, package manager, installer,
loader, runtime service, registry publisher, distribution engine, trust
issuer, or capability issuer.

Any implementation-specific, source-specific, merge-specific,
runtime-specific, package-specific, publication-specific,
distribution-specific, trust-specific, or capability-issuance-specific
interpretation fails closed until later reviewed RFCs define exact
behavior.

## Decision Subject

A decision subject is the exact accepted workflow subject being evaluated
for later bounded Implementation Slice review.

A decision subject must reference:

1. Exact accepted workflow subject.
2. Exact acceptance decision record.
3. Exact accepted evidence set.
4. Exact governance subject.
5. Exact reviewed subject SHA or identifier.
6. Governing RFCs.
7. Non-authorization notice.

Decision subject is not implementation subject.

Decision subject is not source scope, package scope, runtime object,
process, module, plugin, workspace state, registry publication, or
capability token.

Changing the accepted workflow subject, evidence set, acceptance decision
record, governance subject, reviewed subject SHA, or subject-defining
context creates a different decision subject unless a later reviewed RFC
defines exact narrower behavior.

## Accepted Workflow Subject Requirement

Implementation decision requires an accepted workflow subject.

The accepted workflow subject must come from
`PHASE20_CAPABILITY_ACCEPTANCE_WORKFLOW.md` and must be bound to:

1. One exact governance subject.
2. One exact accepted evidence set.
3. One maintainer decision result of `accepted`.
4. One acceptance decision record.
5. Exact-SHA binding.
6. Preserved acceptance audit history.

Accepted workflow subject presence is not implementation authority.

Accepted evidence set presence is not implementation authority.

Accepted workflow subject and accepted evidence set are prerequisites for
decision review only. They do not authorize implementation, source merge,
runtime activation, package execution, distribution, trust assignment,
registry publication, or capability issuance.

## Implementation Subject Boundary

Implementation subject is not defined by this RFC.

A future Implementation Slice RFC may define the exact bounded
implementation subject, source scope, implementation constraints, review
requirements, and denied behaviors.

This RFC reserves the boundary:

```text
decision subject != implementation subject
implementation decision record != implementation slice
implementation decision record != implementation artifact
```

An implementation decision record may make a decision subject eligible for
later Implementation Slice review. It does not create the implementation
subject, define source scope, authorize source changes, or permit
implementation execution.

Implementation Slice requires separate governance review.

## Eligibility Properties

Eligibility is a temporary governance state.

Eligibility means an exact decision subject may proceed to later bounded
Implementation Slice review under separately governed processes.

Eligibility:

1. Is exact-subject bound.
2. Is decision-record bound.
3. Is temporary governance state.
4. May expire.
5. May be withdrawn.
6. May be superseded by a later exact decision record.
7. Produces no authority.
8. Produces no implementation.
9. Produces no source merge.
10. Produces no runtime behavior.

Eligibility is not implementation approval.

Eligibility is not implementation authority.

Eligibility is not source merge authority, package authority, distribution
authority, runtime authority, trust assignment, registry publication, or
capability issuance.

## Immutable Decision Record

Implementation decision records are immutable by default after review.

Changing any of the following creates a different implementation decision
record unless a later reviewed RFC defines exact narrower behavior:

1. Decision subject.
2. Accepted workflow subject.
3. Accepted evidence set.
4. Acceptance decision record.
5. Exact reviewed subject SHA or identifier.
6. Decision review material.
7. Decision outcome.
8. Eligibility properties.
9. Non-authorization notice.

Replacement requires a new decision subject or a new implementation
decision record.

Decision immutability preserves auditability. It does not mean
implementation is approved, source is accepted, source may be merged, or
runtime may be activated.

Decision deletion, replacement, supersession, redaction, retention, and
quarantine procedures belong to later reviewed RFCs or decisions.

## Implementation Decision Review

Implementation decision review evaluates whether a decision subject is
eligible for later bounded Implementation Slice review.

Implementation decision review may evaluate:

1. Accepted workflow subject consistency.
2. Accepted evidence set consistency.
3. Acceptance decision record consistency.
4. Exact-SHA binding.
5. Registry context consistency.
6. Trust context consistency.
7. Distribution policy context consistency.
8. Quarantine, rejection, deferral, or supersession concerns.
9. Bounded implementation prerequisite readiness.
10. Non-authorization compliance.

Implementation decision review does not approve implementation.

Review output is advisory until an implementation decision record records a
decision outcome.

## Implementation Decision Record

An implementation decision record records the decision outcome for an
exact decision subject.

An implementation decision record must identify:

1. Exact decision subject.
2. Exact accepted workflow subject.
3. Exact accepted evidence set.
4. Exact acceptance decision record.
5. Exact reviewed subject SHA or identifier.
6. Decision outcome.
7. Decision reason.
8. Eligibility properties, if outcome is `eligible`.
9. Governing RFCs.
10. Non-authorization notice.
11. Fail-closed handling for later ambiguity.

Implementation decision records governance eligibility only.

Implementation decision records must never modify previous governance
records. They may read prior governance records as review context only.

An implementation decision record must not be interpreted as
implementation, implementation slice, source merge authority,
implementation approval, package execution, runtime activation,
distribution authority, registry publication, trust assignment, or
capability issuance.

## Implementation Inputs

Implementation decision inputs are governance records used to evaluate a
decision subject.

Implementation decision inputs may include:

1. Capability identity reference.
2. Manifest reference.
3. Lifecycle state reference.
4. Registry governance context.
5. Trust context.
6. Distribution policy context.
7. Evidence records.
8. Accepted evidence set.
9. Acceptance workflow record.
10. Acceptance decision record.
11. Quarantine, rejection, deferral, or supersession records.
12. Audit records.

Input presence is not implementation authority.

Input completeness is not implementation approval.

Input review is not source merge authority or runtime authority.

## Accepted Evidence Requirement

Implementation decision requires accepted evidence.

Accepted evidence means evidence accepted by the acceptance workflow for
the exact accepted workflow subject.

Accepted evidence must be:

1. Exact-subject bound.
2. Accepted by explicit acceptance workflow review.
3. Bound to a maintainer decision result of `accepted`.
4. Bound to an exact acceptance decision record.
5. Auditable.
6. Non-authoritative by itself.

Accepted evidence does not approve implementation.

Accepted evidence does not authorize source merge, runtime activation,
registry publication, distribution, package execution, trust assignment, or
capability issuance.

## Decision Outcomes

Implementation decision outcomes are governance outcomes only.

This RFC defines:

| Outcome | Meaning | Authority produced |
|---|---|---|
| `eligible` | Decision subject may proceed to later Implementation Slice review | None |
| `deferred` | Additional governance material is required | None |
| `denied` | Decision subject cannot continue under this decision record | None |

No other implementation decision outcome is defined by this RFC.

`eligible` is not implementation approved.

`deferred` is not rejection or quarantine by itself.

`denied` does not delete history, revoke another record, transfer
authority, or prove fault by itself.

Outcome presence must not be interpreted as implementation authority,
source merge authority, runtime activation, trust assignment, registry
publication, distribution authority, package execution, or capability
issuance.

## Bounded Implementation Principle

Implementation decision must preserve bounded implementation.

Implementation decision may make a decision subject eligible for later
bounded Implementation Slice review.

It must not make an entire capability, registry namespace, package family,
module family, plugin family, runtime surface, or workflow class eligible
by implication.

The conceptual boundary is:

```text
Accepted Workflow Subject
  -> Implementation Decision Review
  -> Implementation Decision Record
  -> eligible | deferred | denied
  -> later Implementation Slice RFC
```

Every arrow is a governance dependency. No arrow implies implementation,
source merge, package installation, package loading, package execution,
distribution, capability issuance, or runtime activation.

Implementation Slice requires separate governance review.

## Implementation Constraints

Implementation decision constraints are the rules that must hold before an
implementation decision outcome may be read.

Implementation decision must be:

1. Exact-subject bound.
2. Accepted-workflow-subject bound.
3. Accepted-evidence-set bound.
4. Exact-SHA bound.
5. Immutable by default after review.
6. Bounded-slice oriented.
7. Implementation-denying.
8. Source-merge-denying.
9. Runtime-denying.
10. Capability-issuance-denying.
11. Authority-denying by default.
12. Fail-closed on ambiguity.

Implementation decision must not:

1. Define implementation source scope.
2. Modify source.
3. Merge source.
4. Install, load, execute, or publish packages.
5. Change runtime state.
6. Issue capabilities or tokens.
7. Publish registry records.
8. Assign trust.
9. Execute distribution.
10. Authorize implementation.

Any constraint violation fails closed.

## Relationship Boundaries

Implementation decision may consume prior Phase-20 governance records as
review context only.

| Governance record | Accepted use | Denied reading |
|---|---|---|
| `PHASE20_CAPABILITY_ACCEPTANCE_WORKFLOW.md` | Use accepted workflow subject, accepted evidence set, and acceptance decision record as decision prerequisites | Accepted workflow subject is not implementation authority |
| `PHASE20_CAPABILITY_EVIDENCE_MODEL.md` | Use evidence records through the accepted evidence set | Evidence presence or validation is not implementation approval |
| `PHASE20_REGISTRY_MODEL.md` and `PHASE20_REGISTRY_GOVERNANCE.md` | Use registry records and registry governance decisions as context | Registry presence or acceptance is not source merge, publication, distribution, implementation, issuance, or runtime authority |
| `PHASE20_TRUST_MODEL.md` | Use trust context as review context | Trust input, claim, proof reference, signature presence, context, or assessment class is not implementation approval or trust assignment |
| `PHASE20_DISTRIBUTION_POLICY.md` | Use distribution policy context as review context | Distribution eligibility is not implementation authority, source merge, distribution execution, issuance, or runtime activation |
| Implementation decision records | Use eligibility context for later Phase-20 decision paths | Eligibility context is not implementation authority and never transfers authority |

Implementation decision does not modify acceptance state.

Implementation decision does not modify evidence records.

Implementation decision does not modify trust context.

Implementation decision does not execute distribution.

Ambiguous, stale, inherited, unaccepted, or differently scoped
relationship material fails closed for implementation decision.

## Implementation Validation Model

Implementation decision validation is conceptual and fail-closed.

Validation must never infer missing governance material.

Implementation decision material is invalid for governance review when:

1. Decision subject is missing or ambiguous.
2. Accepted workflow subject is missing, stale, ambiguous, inherited, or
   differently scoped.
3. Accepted evidence set is missing, stale, ambiguous, inherited, or
   differently scoped.
4. Acceptance decision record is missing or not `accepted`.
5. Exact-SHA binding is missing or ambiguous.
6. Validation infers missing governance material.
7. Decision subject is treated as implementation subject.
8. Eligibility is treated as authority.
9. Decision record is treated as implementation artifact.
10. Decision outcome is treated as source merge authority.
11. Decision material depends on unaccepted proof or signature semantics.
12. Decision material depends on trust assignment.
13. Decision material depends on publication or distribution authority.
14. Decision material depends on runtime-observed state.
15. Decision material relies on alias or supersession without accepted
    rules.
16. Decision material implies implementation authority.
17. Decision material implies source merge authority.
18. Decision material implies capability issuance.
19. Decision material implies runtime activation.

Validation failure grants no authority. It requires correction, denial,
deferral, quarantine, supersession, dispute recording, or a later reviewed
decision path.

Implementation decision validation is not implementation approval.

Validation produces only a validation result.

Validation never produces implementation authority.

## Implementation Invariants

Every later Phase-20 RFC must preserve these implementation decision
invariants:

1. Implementation decision records governance eligibility only.
2. Implementation decision is not implementation.
3. Implementation decision is not implementation slice.
4. Implementation decision record is not implementation execution.
5. Decision subject is not implementation subject.
6. Accepted workflow subject is not implementation authority.
7. Accepted evidence set is not implementation authority.
8. Eligibility is temporary governance state.
9. Eligibility may expire or be withdrawn.
10. Eligibility is not authority.
11. Eligible is not implementation approved.
12. Eligible is not implementation scheduled.
13. Implementation decision records are immutable by default after review.
14. Implementation decision records never modify previous governance
    records.
15. Replacement requires a new decision subject or new decision record.
16. Implementation Slice requires separate governance review.
17. Implementation decision does not imply source merge authority.
18. Implementation decision does not imply package execution.
19. Implementation decision does not imply registry publication.
20. Implementation decision does not imply distribution authority.
21. Implementation decision does not imply trust assignment.
22. Implementation decision does not imply capability issuance.
23. Implementation decision does not imply runtime activation.
24. Ambiguity fails closed.

Violation of any invariant fails closed.

## Later RFC Dependencies

The implementation decision model is a prerequisite for later Phase-20
decision paths.

| Later record | Implementation decision relationship |
|---|---|
| `PHASE20_IMPLEMENTATION_SLICE.md` | May define the exact bounded source scope only after an `eligible` implementation decision record. |
| `PHASE20_IMPLEMENTATION_REVIEW.md` | May review implementation slice evidence without inheriting decision authority. |
| `PHASE20_IMPLEMENTATION_ACCEPTANCE_DECISION.md` | May decide whether reviewed implementation work is accepted without implying runtime authority. |
| `PHASE20_RUNTIME_DECISION.md` | May define runtime effects only after separate reviewed runtime authority, if ever authorized. |

Later RFCs may narrow implementation decision use. They must not broaden
this decision model into implementation, source merge authority, package
execution, trust assignment, registry publication, distribution authority,
capability issuance, or runtime authority without a separate reviewed
decision.

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
  -> Capability Evidence Model
  -> Capability Acceptance Workflow
  -> Implementation Decision
  -> Implementation Slice
  -> Implementation Review
  -> Implementation Acceptance Decision
  -> Runtime Decision
```

Every arrow means a governance dependency. It does not imply
implementation approval, source merge authority, publication, distribution,
installation, execution, issuance, or runtime activation.

Every dependency is explicit.

No dependency is implied.

Each RFC defines only its own layer. No RFC produces the authority of the
next layer.

## Explicit Non-Authorization

This implementation decision RFC does not authorize:

1. Runtime activation.
2. General runtime authority.
3. Phase-20 implementation.
4. Implementation approval.
5. Implementation slice authority.
6. Source acceptance or source merge authority.
7. Package installation, loading, execution, scheduling, or publication.
8. Module loading.
9. Workspace creation, workspace runtime, or real mounts.
10. Plugin host, plugin loading, or plugin instantiation.
11. Capability token minting or capability issuance.
12. Trust assignment.
13. Trust issuer authority.
14. Registry authority.
15. Registry publication.
16. Publication authority.
17. Distribution authority.
18. Distribution execution.
19. Semantic CLI execution or verdict authority.
20. AI Runtime authority.
21. Agent behavior.
22. New syscalls.
23. Kernel ABI expansion.
24. Workflow-threshold, baseline, dependency, or Ring0 policy changes.
25. Observability-as-authority.

Unknown authority readings fail closed.

## Architecture Signature

**Prepared by:** Kenan AY
**Role:** AykenOS Architecture Steward
**Document type:** Phase-20 RFC
**Architecture status:** Draft RFC / pending architectural review
**Authority notice:** This signature identifies the architectural authorship
of this RFC. It grants no runtime authority, implementation authority,
implementation slice authority, trust authority, evidence authority,
acceptance authority, proof authority, execution authority, constitutional
authority, registry authority, distribution authority, publication
authority, capability issuance authority, package authority, module
authority, plugin authority, Semantic CLI authority, AI Runtime authority,
agent authority, or Ring0 authority.

## Non-Goals

This document does not define or authorize:

1. Implementation.
2. Implementation approval.
3. Implementation slice authority.
4. Implementation source scope.
5. Source acceptance or source merge authority.
6. Repository branch protection.
7. Runtime activation or general runtime authority.
8. Package format, repository, installation, loading, or execution.
9. Artifact storage or binary format.
10. Module loading.
11. Workspace creation, workspace runtime, or real mounts.
12. Plugin host, plugin loading, or plugin instantiation.
13. Capability token minting or capability issuance.
14. Trust assignment or trust issuer authority.
15. Registry authority or registry publication.
16. Publication workflow or publication approval.
17. Distribution authority or distribution execution.
18. Proof verification, signature verification, or signature acceptance.
19. Semantic CLI execution or verdict authority.
20. AI Runtime authority.
21. Agent behavior.
22. New syscalls.
23. Kernel ABI expansion.
24. Workflow-threshold, baseline, dependency, or Ring0 policy changes.

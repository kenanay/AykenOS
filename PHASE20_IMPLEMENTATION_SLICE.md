# Phase-20 Implementation Slice

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
`PHASE20_CAPABILITY_EVIDENCE_MODEL.md`,
`PHASE20_CAPABILITY_ACCEPTANCE_WORKFLOW.md`, and
`PHASE20_IMPLEMENTATION_DECISION.md`. In case of conflict, those documents
prevail unless this implementation slice RFC is the narrower Phase-20
implementation slice record for the exact planning scope identified below.

**Status:** PHASE-20 IMPLEMENTATION SLICE RFC / BOUNDED SOURCE SCOPE MODEL
ONLY / NO IMPLEMENTATION / NO IMPLEMENTATION APPROVAL / NO SOURCE MERGE
AUTHORITY / NO SOURCE ACCEPTANCE / NO PACKAGE AUTHORITY / NO PACKAGE
INSTALLATION / NO PACKAGE LOADING / NO PACKAGE EXECUTION / NO DEPLOYMENT /
NO RUNTIME ACTIVATION / NO GENERAL RUNTIME AUTHORITY / NO CAPABILITY
ISSUANCE / NO TRUST ASSIGNMENT / NO REGISTRY PUBLICATION / NO DISTRIBUTION
AUTHORITY
**Implementation slice date:** 2026-06-30
**Implementation slice id:** `ayken.phase20.implementation_slice.v1`
**Implementation slice base main SHA:** `0f74c3f572cb04fbcd4bc4868cedb3f22dc3ae6c`
**Current phase pointer:** `CURRENT_PHASE=20`
**Authority boundary:** Bounded source scope model only; not
implementation, not implementation approval, not source acceptance, not
source merge authority, not source repository authority, not package
authority, not package installation, not package loading, not package
execution, not deployment, not runtime activation, not general runtime
authority, not module loading, not workspace runtime, not plugin loading,
not capability token minting, not capability issuance, not trust
assignment, not trust issuer authority, not registry authority, not
registry publication, not publication authority, not distribution
authority, not distribution execution, not Semantic CLI authority, not AI
Runtime authority, not agent authority, not syscall expansion, not kernel
ABI expansion, not workflow-threshold, baseline, dependency, or Ring0
authority.

## Purpose

This document defines the Phase-20 implementation slice model for bounded
source scope records.

It answers one question:

```text
What is the exact bounded implementation scope for one implementation
slice subject?
```

It does not answer:

```text
How is implementation performed?
How is source merged?
How is implementation reviewed?
How is implementation accepted?
How is runtime activated?
How is a package installed, loaded, executed, deployed, or distributed?
How is a capability issued?
```

Those questions belong to later Phase-20 RFCs and implementation decisions.

## Core Rule

```text
implementation slice != implementation
implementation slice != implementation approval
implementation slice != source merge
implementation slice != package
implementation slice != runtime
implementation slice != deployment
implementation slice != capability issuance
implementation slice scope != source authority
allowed source scope != merge authority
slice subject != source repository
slice identity != source path ownership
slice validation != scope expansion
frozen boundary != optional boundary
```

Slice defines scope.

Slice never defines behavior.

An implementation slice records bounded source scope for later
implementation review. It does not implement, modify source, merge source,
approve implementation, execute packages, deploy artifacts, activate
runtime behavior, publish registries, distribute packages, assign trust, or
issue capabilities.

Unknown authority readings fail closed.

## Implementation Slice Mission

The mission of the Phase-20 implementation slice model is to define exact,
bounded, reviewable source scope for a later implementation review path.

Implementation slice exists so later RFCs can reason about:

1. Implementation slice subjects.
2. Implementation slice identity.
3. Eligible implementation decision prerequisites.
4. Bounded source scope.
5. Allowed source scope.
6. Excluded source scope.
7. Frozen boundaries.
8. Forbidden changes.
9. Slice integrity.
10. Slice review inputs.
11. Later implementation review prerequisites.

The implementation slice model itself grants no implementation,
implementation approval, source merge authority, package authority,
deployment, runtime, distribution, trust, registry, or capability issuance
authority.

Each later use requires its own reviewed RFC or decision path.

## Implementation Slice Definition

An implementation slice is a governance record that defines the exact
bounded source scope that may be considered by a later implementation
review.

An implementation slice may describe:

1. The exact implementation slice subject.
2. The implementation slice identity.
3. The eligible implementation decision record.
4. The bounded source scope.
5. Allowed source paths or areas.
6. Excluded source paths or areas.
7. Frozen boundaries.
8. Forbidden changes.
9. Slice integrity constraints.
10. Later implementation review dependency.
11. Non-authorization notice.

An implementation slice is not:

1. Implementation.
2. Implementation approval.
3. Source acceptance.
4. Source merge authority.
5. Source repository authority.
6. Package artifact.
7. Runtime unit.
8. Deployment unit.
9. Capability issuance.
10. Registry publication.
11. Distribution authority.
12. Semantic CLI, AI Runtime, or agent authority.

## Implementation Slice Scope

This RFC defines only the bounded source scope model.

It does not define implementation mechanics, source modification
procedure, source merge procedure, repository branch protection, package
format, artifact storage, binary format, deployment behavior, runtime
behavior, registry publication, distribution execution, trust assignment,
capability issuance, module loading, plugin loading, or workspace runtime.

Implementation slice is a governance scope layer. It is not an
implementation engine, source merge engine, package manager, installer,
loader, deployment service, runtime service, registry publisher,
distribution engine, trust issuer, or capability issuer.

Any implementation-specific, source-change-specific, merge-specific,
package-specific, deployment-specific, runtime-specific,
publication-specific, distribution-specific, trust-specific, or
capability-issuance-specific interpretation fails closed until later
reviewed RFCs define exact behavior.

## Implementation Slice Subject

An implementation slice subject is the exact governance subject whose
bounded source scope is being defined for later implementation review.

An implementation slice subject must reference:

1. Exact eligible implementation decision record.
2. Exact decision subject.
3. Exact accepted workflow subject.
4. Exact accepted evidence set.
5. Exact acceptance decision record.
6. Exact reviewed subject SHA or identifier.
7. Governing RFCs.
8. Non-authorization notice.

Implementation slice subject is not implementation.

Implementation slice subject is not source repository ownership, package
ownership, runtime ownership, module ownership, plugin ownership, registry
publication, deployment target, process, workspace state, or capability
token.

Changing the eligible implementation decision record, decision subject,
accepted workflow subject, accepted evidence set, acceptance decision
record, reviewed subject SHA, or subject-defining context creates a
different implementation slice subject unless a later reviewed RFC defines
exact narrower behavior.

## Implementation Slice Identity

Implementation slice identity distinguishes one implementation slice record
from another.

Implementation slice identity is conceptually composed of:

```text
(slice_domain, implementation_slice_subject, eligible_decision_record,
 bounded_source_scope, frozen_boundary, slice_binding)
```

This tuple is conceptual. It is not a source path syntax, source ownership
claim, package name, module name, crate name, repository branch, database
schema, command, token, runtime handle, or merge key.

Implementation slice identity remains stable for the lifetime of that
slice record.

Implementation slice identity remains stable until superseded by a later
reviewed slice record.

Changing identity-defining slice fields creates a different implementation
slice record unless a later reviewed RFC defines exact narrower behavior.

Implementation slice identity does not imply source authority, source merge
authority, implementation approval, package authority, deployment
authority, runtime authority, registry publication, distribution authority,
trust assignment, or capability issuance.

## Eligible Decision Requirement

Implementation slice requires an eligible implementation decision record.

The eligible implementation decision record must come from
`PHASE20_IMPLEMENTATION_DECISION.md` and must be bound to:

1. One exact decision subject.
2. One exact accepted workflow subject.
3. One exact accepted evidence set.
4. One exact acceptance decision record.
5. One decision outcome of `eligible`.
6. Exact-SHA binding.
7. Preserved decision audit history.

Eligible implementation decision record presence is not implementation
authority.

Eligible implementation decision record presence is not source merge
authority.

Eligibility may be consumed as prerequisite context only. It does not
authorize implementation, source changes, source merge, package execution,
deployment, runtime activation, distribution, trust assignment, registry
publication, or capability issuance.

## Bounded Source Scope

Bounded source scope is the exact technical source scope identified by an
implementation slice.

Bounded source scope may include:

1. Allowed files.
2. Allowed directories.
3. Allowed modules.
4. Allowed crates or packages as source organization only.
5. Allowed documentation records.
6. Allowed test records.
7. Allowed validation fixtures.
8. Review boundary.
9. Non-authorization notice.

Bounded source scope is not source authority.

Bounded source scope is not a package, runtime unit, deployment unit,
binary artifact, module instance, plugin instance, workspace mount, or
capability token.

Bounded source scope must be exact, auditable, and narrower than the whole
repository unless a later reviewed RFC defines exact narrower behavior.

Approximate, inherited, stale, implied, wildcard-only, unbounded, or
differently scoped source readings fail closed.

## Source Scope

Source scope is the implementation-slice view of source areas that may be
considered by later implementation review.

Source scope may identify:

1. File paths.
2. Directory paths.
3. Module paths.
4. Crate or package paths as source organization only.
5. Test fixture paths.
6. Validation fixture paths.
7. Documentation paths.
8. Generated-output paths only if explicitly bounded.

Source scope does not grant permission to edit, merge, build, package,
deploy, load, execute, or activate anything.

Source scope must not silently include adjacent files, generated artifacts,
dependency trees, build products, package outputs, runtime objects, or
workspace state.

## Allowed Scope

Allowed scope is the source material that may be considered by a later
implementation review.

Allowed scope must be:

1. Exact.
2. Bounded.
3. Reviewable.
4. Path- or record-specific.
5. Linked to one implementation slice identity.
6. Linked to one eligible implementation decision record.
7. Non-authoritative by itself.

Allowed scope is not merge authority.

Allowed scope is not implementation approval.

Allowed scope must not include excluded scope, frozen boundary material, or
forbidden change categories by implication.

## Excluded Scope

Excluded scope is source material outside the implementation slice.

Excluded scope may include:

1. Files not explicitly listed as allowed.
2. Directories not explicitly listed as allowed.
3. Adjacent modules.
4. Generated artifacts.
5. Package outputs.
6. Deployment assets.
7. Runtime objects.
8. Workspace state.
9. Registry publication material.
10. Capability issuance material.

Excluded scope remains outside the slice even when it is technically
nearby, imported, referenced, generated, or transitively related.

Excluded scope may not be changed by implication.

Any reading that pulls excluded scope into allowed scope fails closed.

## Frozen Boundary

Frozen boundary is governance-protected material that an implementation
slice must not enter.

Frozen boundary may include:

1. Phase-0 foundational authority.
2. Phase-18 constitutional authority.
3. Architecture freeze material.
4. Authority drift guard material.
5. Kernel ABI surfaces.
6. Syscall tables.
7. Ring0 policy surfaces.
8. Workflow-threshold policy.
9. Baseline policy.
10. Dependency policy.
11. Runtime activation surfaces.
12. Capability issuance surfaces.

Frozen Boundary must never be entered by an Implementation Slice.

Frozen boundary is not optional.

Frozen boundary may grow.

Frozen boundary must never shrink inside an existing slice record.

Frozen boundary material may not be changed, reinterpreted, narrowed,
expanded, bypassed, or converted into allowed scope by this RFC.

Any frozen-boundary ambiguity fails closed.

## Forbidden Changes

Forbidden changes are change categories that an implementation slice must
not permit.

This RFC forbids implementation slice readings that allow:

1. Runtime activation.
2. Source merge authority.
3. Package installation, loading, execution, scheduling, or publication.
4. Deployment behavior.
5. Module loading.
6. Plugin loading.
7. Workspace runtime or real mounts.
8. Capability token minting or capability issuance.
9. Registry publication.
10. Distribution execution.
11. Trust assignment.
12. Evidence acceptance.
13. Acceptance decision mutation.
14. Implementation decision mutation.
15. Kernel ABI expansion.
16. Syscall expansion.
17. Ring0 policy changes.
18. Workflow-threshold, baseline, or dependency changes.

Forbidden changes remain forbidden even if they appear inside an otherwise
allowed source path.

Forbidden change ambiguity fails closed.

## Slice Boundaries

Slice boundaries define the edge of the implementation slice.

The conceptual slice chain is:

```text
Accepted Workflow Subject
  -> Implementation Decision
  -> Eligible Decision Record
  -> Implementation Slice Subject
  -> Implementation Slice Identity
  -> Bounded Source Scope
  -> later Implementation Review
```

Every arrow is a governance dependency. No arrow implies implementation,
source modification, source merge, package execution, deployment,
distribution, capability issuance, or runtime activation.

Slice boundary must distinguish:

1. Decision subject from slice subject.
2. Slice subject from source repository.
3. Slice identity from source path ownership.
4. Allowed scope from excluded scope.
5. Allowed scope from frozen boundary.
6. Scope definition from behavior definition.
7. Later implementation review from implementation approval.

Unknown, implicit, inherited, stale, or differently scoped boundary
readings fail closed.

## Slice Integrity

Slice integrity is the reviewable consistency of implementation slice
subject, slice identity, eligible decision record, bounded source scope,
excluded scope, frozen boundary, and non-authorization notices.

Slice integrity may require:

1. Exact eligible decision binding.
2. Exact slice subject binding.
3. Stable slice identity.
4. Exact bounded source scope.
5. Exact excluded scope.
6. Frozen boundary preservation.
7. Forbidden change denial.
8. Non-authorization notice.
9. Audit preservation.

Slice integrity is not implementation approval.

Integrity checks do not grant source authority, merge authority, package
authority, deployment authority, runtime authority, distribution authority,
registry publication, trust assignment, or capability issuance.

## Slice Review Inputs

Slice review inputs are governance records used by later implementation
review.

Slice review inputs may include:

1. Implementation slice subject.
2. Implementation slice identity.
3. Eligible implementation decision record.
4. Accepted workflow subject.
5. Accepted evidence set.
6. Acceptance decision record.
7. Bounded source scope.
8. Allowed scope.
9. Excluded scope.
10. Frozen boundary.
11. Forbidden change list.
12. Audit records.

Slice review input presence is not implementation review.

Slice review input completeness is not implementation approval.

Slice review inputs do not authorize source modification, source merge,
package execution, deployment, runtime activation, distribution, registry
publication, trust assignment, or capability issuance.

## Scope Narrowing And Widening

Slice scope may narrow.

Slice scope must never widen without a new reviewed slice record.

Narrowing may remove allowed source scope, add excluded scope, add frozen
boundary constraints, or add forbidden change constraints.

Narrowing does not imply implementation approval, source merge authority,
runtime activation, or capability issuance.

Widening includes adding new source files, expanding directories, adding
adjacent modules, including generated artifacts, weakening frozen boundary
constraints, removing forbidden changes, or interpreting source scope more
broadly than the reviewed slice record.

Any attempted widening by implication fails closed.

## Relationship Boundaries

Implementation slice may consume prior Phase-20 governance records as
review context only.

| Previous record | Accepted reading | Denied reading |
|---|---|---|
| `PHASE20_IMPLEMENTATION_DECISION.md` | Eligible decision record as prerequisite context | Eligibility is not implementation, source merge authority, or runtime authority |
| `PHASE20_CAPABILITY_ACCEPTANCE_WORKFLOW.md` | Accepted workflow subject and acceptance decision record as context | Accepted workflow subject is not source authority |
| `PHASE20_CAPABILITY_EVIDENCE_MODEL.md` | Accepted evidence through acceptance workflow context | Accepted evidence is not implementation proof or implementation approval |
| `PHASE20_REGISTRY_MODEL.md` and `PHASE20_REGISTRY_GOVERNANCE.md` | Registry context for subject consistency | Registry context is not source scope, publication, implementation, issuance, or runtime authority |
| `PHASE20_TRUST_MODEL.md` | Trust context for review context | Trust context is not source authority, trust assignment, implementation approval, or runtime authority |
| `PHASE20_DISTRIBUTION_POLICY.md` | Distribution policy context for scope review | Distribution eligibility is not source scope, distribution execution, implementation, issuance, or runtime authority |
| `PHASE20_IMPLEMENTATION_REVIEW.md` | Later review input only | Implementation review reference is not implementation approval, source merge authority, or runtime authority |

Implementation slice does not modify prior governance records.

Implementation slice does not modify acceptance state.

Implementation slice does not modify evidence records.

Implementation slice does not modify implementation decision records.

Ambiguous, stale, inherited, unaccepted, or differently scoped
relationship material fails closed for implementation slice.

## Slice Validation Model

Implementation slice validation is conceptual and fail-closed.

Slice validation must never expand bounded source scope.

Slice validation must never reinterpret slice intent.

Implementation slice material is invalid for governance review when:

1. Implementation slice subject is missing or ambiguous.
2. Implementation slice identity is missing or ambiguous.
3. Eligible implementation decision record is missing, stale, ambiguous,
   inherited, or differently scoped.
4. Eligible implementation decision record does not have outcome
   `eligible`.
5. Bounded source scope is missing or ambiguous.
6. Allowed scope is approximate, wildcard-only, stale, inherited, or
   differently scoped.
7. Excluded scope is pulled into allowed scope by implication.
8. Frozen boundary is missing, ambiguous, weakened, or entered.
9. Forbidden changes are permitted by implication.
10. Slice validation expands bounded source scope.
11. Slice validation reinterprets slice intent.
12. Slice scope widens without a new reviewed slice record.
13. Source repository is treated as slice subject.
14. Slice identity is treated as source path ownership.
15. Allowed source scope is treated as merge authority.
16. Slice material depends on runtime-observed state.
17. Slice material relies on alias or supersession without accepted rules.
18. Slice material implies implementation approval.
19. Slice material implies source merge authority.
20. Slice material implies capability issuance.
21. Slice material implies runtime activation.

Validation failure grants no authority. It requires correction, denial,
deferral, quarantine, supersession, dispute recording, or a later reviewed
decision path.

Implementation slice validation is not implementation approval.

Validation produces only a validation result.

Validation never produces source authority, merge authority, implementation
authority, package authority, deployment authority, runtime authority, or
capability issuance.

## Slice Invariants

Every later Phase-20 RFC must preserve these implementation slice
invariants:

1. Slice defines scope.
2. Slice never defines behavior.
3. Implementation slice is not implementation.
4. Implementation slice is not implementation approval.
5. Implementation slice is not source merge.
6. Implementation slice is not package.
7. Implementation slice is not runtime.
8. Implementation slice is not deployment.
9. Implementation slice is not capability issuance.
10. Implementation slice requires an eligible implementation decision
    record.
11. Implementation slice subject is not source repository.
12. Implementation slice identity is not source path ownership.
13. Bounded source scope is not source authority.
14. Allowed source scope is not merge authority.
15. Excluded scope remains outside the slice.
16. Frozen Boundary must never be entered by an Implementation Slice.
17. Frozen boundary may grow.
18. Frozen boundary must never shrink inside an existing slice record.
19. Forbidden changes remain forbidden inside allowed paths.
20. Slice scope may narrow.
21. Slice scope must never widen without a new reviewed slice record.
22. Slice validation must never expand bounded source scope.
23. Slice validation must never reinterpret slice intent.
24. Implementation slice does not modify prior governance records.
25. Implementation Review requires separate governance review.
26. Implementation slice does not imply package execution.
27. Implementation slice does not imply registry publication.
28. Implementation slice does not imply distribution authority.
29. Implementation slice does not imply trust assignment.
30. Implementation slice does not imply runtime activation.
31. Ambiguity fails closed.

Violation of any invariant fails closed.

## Later RFC Dependencies

The implementation slice model is a prerequisite for later Phase-20
implementation review paths.

| Later record | Implementation slice relationship |
|---|---|
| `PHASE20_IMPLEMENTATION_REVIEW.md` | May review implementation work against the exact bounded source scope without inheriting merge or runtime authority. |
| `PHASE20_IMPLEMENTATION_ACCEPTANCE_DECISION.md` | May decide whether reviewed implementation work is accepted without implying runtime authority. |
| `PHASE20_RUNTIME_DECISION.md` | May define runtime effects only after separate reviewed runtime authority, if ever authorized. |

Later RFCs may narrow implementation slice use. They must not broaden this
slice model into implementation, source merge authority, package execution,
deployment, trust assignment, registry publication, distribution authority,
capability issuance, or runtime authority without a separate reviewed
decision.

Implementation Slice is the last Phase-20 RFC that defines only scope.

Later RFCs begin evaluating implementation work.

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
installation, execution, issuance, deployment, or runtime activation.

Every dependency is explicit.

No dependency is implied.

Each RFC defines only its own layer. No RFC produces the authority of the
next layer.

## Explicit Non-Authorization

This implementation slice RFC does not authorize:

1. Runtime activation.
2. General runtime authority.
3. Phase-20 implementation.
4. Implementation approval.
5. Implementation review approval.
6. Source acceptance or source merge authority.
7. Source repository authority.
8. Package installation, loading, execution, scheduling, or publication.
9. Deployment behavior.
10. Module loading.
11. Workspace creation, workspace runtime, or real mounts.
12. Plugin host, plugin loading, or plugin instantiation.
13. Capability token minting or capability issuance.
14. Trust assignment.
15. Trust issuer authority.
16. Registry authority.
17. Registry publication.
18. Publication authority.
19. Distribution authority.
20. Distribution execution.
21. Semantic CLI execution or verdict authority.
22. AI Runtime authority.
23. Agent behavior.
24. New syscalls.
25. Kernel ABI expansion.
26. Workflow-threshold, baseline, dependency, or Ring0 policy changes.
27. Observability-as-authority.

Unknown authority readings fail closed.

## Architecture Signature

**Prepared by:** Kenan AY
**Role:** AykenOS Architecture Steward
**Document type:** Phase-20 RFC
**Architecture status:** Draft RFC / pending architectural review
**Authority notice:** This signature identifies the architectural authorship
of this RFC. It grants no runtime authority, implementation authority,
implementation review authority, source merge authority, trust authority,
evidence authority, acceptance authority, proof authority, execution
authority, constitutional authority, registry authority, distribution
authority, publication authority, capability issuance authority, package
authority, deployment authority, module authority, plugin authority,
Semantic CLI authority, AI Runtime authority, agent authority, or Ring0
authority.

## Non-Goals

This document does not define or authorize:

1. Implementation.
2. Implementation approval.
3. Implementation review approval.
4. Implementation acceptance decision authority.
5. Source acceptance or source merge authority.
6. Source repository authority.
7. Repository branch protection.
8. Runtime activation or general runtime authority.
9. Package format, repository, installation, loading, or execution.
10. Deployment behavior.
11. Artifact storage or binary format.
12. Module loading.
13. Workspace creation, workspace runtime, or real mounts.
14. Plugin host, plugin loading, or plugin instantiation.
15. Capability token minting or capability issuance.
16. Trust assignment or trust issuer authority.
17. Registry authority or registry publication.
18. Publication workflow or publication approval.
19. Distribution authority or distribution execution.
20. Proof verification, signature verification, or signature acceptance.
21. Semantic CLI execution or verdict authority.
22. AI Runtime authority.
23. Agent behavior.
24. New syscalls.
25. Kernel ABI expansion.
26. Workflow-threshold, baseline, dependency, or Ring0 policy changes.

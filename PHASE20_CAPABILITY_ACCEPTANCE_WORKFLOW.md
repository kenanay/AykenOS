# Phase-20 Capability Acceptance Workflow

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
`PHASE20_DISTRIBUTION_POLICY.md`, and
`PHASE20_CAPABILITY_EVIDENCE_MODEL.md`. In case of conflict, those
documents prevail unless this acceptance workflow RFC is the narrower
Phase-20 capability acceptance workflow record for the exact planning scope
identified below.

**Status:** PHASE-20 CAPABILITY ACCEPTANCE WORKFLOW RFC / EVIDENCE
ACCEPTANCE WORKFLOW MODEL ONLY / NO IMPLEMENTATION APPROVAL / NO SOURCE
MERGE AUTHORITY / NO TRUST ASSIGNMENT / NO REGISTRY AUTHORITY / NO REGISTRY
PUBLICATION / NO PUBLICATION AUTHORITY / NO DISTRIBUTION AUTHORITY / NO
CAPABILITY ISSUANCE / NO RUNTIME ACTIVATION / NO GENERAL RUNTIME AUTHORITY
**Acceptance workflow date:** 2026-06-29
**Acceptance workflow id:** `ayken.phase20.capability_acceptance_workflow.v1`
**Acceptance workflow base main SHA:** `8c5a136ba96b4e9f73e69cad9d68a53e427386fc`
**Current phase pointer:** `CURRENT_PHASE=20`
**Authority boundary:** Evidence acceptance workflow model only; not
implementation approval, not source acceptance, not source merge authority,
not implementation authority, not runtime activation, not general runtime
authority, not trust assignment, not trust issuer authority, not registry
authority, not registry publication, not publication authority, not
distribution authority, not distribution execution, not capability
issuance, not package installation, not package loading, not package
execution, not module loading, not workspace runtime, not plugin loading,
not capability token minting, not Semantic CLI authority, not AI Runtime
authority, not agent authority, not syscall expansion, not kernel ABI
expansion, not workflow-threshold, baseline, dependency, or Ring0
authority.

## Purpose

This document defines the Phase-20 acceptance workflow model for evidence
records bound to exact capability governance subjects.

It answers one question:

```text
How are capability evidence records submitted, reviewed, accepted, rejected,
quarantined, and verified for an exact governance subject?
```

It does not answer:

```text
How is implementation approved?
How is source merged?
How is trust assigned?
How is a registry record published or distributed?
How is a capability issued, implemented, activated, loaded, or run?
```

Those questions belong to later Phase-20 RFCs and implementation decisions.

## Core Rule

```text
acceptance workflow != implementation approval
acceptance workflow != source merge authority
evidence accepted != implementation accepted
evidence reviewed != runtime enabled
evidence interpreted != capability issued
workflow completed != package executable
review finished != registry published
accepted evidence != runtime authority
acceptance decision != capability issuance
acceptance decision != authority expansion
post-merge verification != implementation authority
post-merge PASS != authority expansion
reviewer finding != maintainer decision
maintainer decision != implementation authority
acceptance record != repository state
```

Acceptance workflow defines a governance process. It does not define the
authority effects of that process.

Acceptance may interpret evidence for acceptance review. It does not
approve implementation, merge source, assign trust, publish registries,
authorize distribution, issue capabilities, execute packages, load modules,
activate runtime behavior, or grant authority.

Acceptance closes only the evidence review workflow for an exact subject.
It does not open implementation authority.

Unknown authority readings fail closed.

## Acceptance Mission

The mission of the Phase-20 acceptance workflow is to define an explicit,
auditable review path for evidence records bound to exact governance
subjects.

Acceptance workflow exists so later RFCs can reason about:

1. Acceptance subjects and evidence sets.
2. Evidence submission and review.
3. Evidence interpretation boundaries.
4. Reviewer findings and maintainer decisions.
5. Accepted, rejected, quarantined, deferred, or superseded workflow
   outcomes.
6. Exact-SHA decision binding.
7. Post-merge exact-SHA verification.
8. Acceptance audit history.
9. Later implementation decision prerequisites.

The acceptance workflow model itself grants no implementation approval,
source merge authority, trust assignment, registry publication,
distribution, runtime, or capability issuance authority.

Each later use requires its own reviewed RFC or decision path.

## Acceptance Definition

Capability acceptance workflow is the governance process that reviews one
evidence set bound to one exact governance subject and records a governance
outcome for that evidence set.

Acceptance workflow may describe:

1. The exact acceptance subject.
2. The exact evidence set under review.
3. Submission, review, and decision requirements.
4. Reviewer findings.
5. Maintainer decision result.
6. Acceptance state and outcome.
7. Exact-SHA binding.
8. Post-merge verification requirements.
9. Non-authorization notice.

Acceptance workflow is not implementation approval, source acceptance,
source merge authority, runtime activation, trust assignment, registry
authority, registry publication, publication authority, distribution
authority, capability issuance, package installation, package loading,
package execution, Semantic CLI authority, AI Runtime authority, or agent
authority.

## Acceptance Scope

This RFC defines only the evidence acceptance workflow model.

It does not define implementation decision authority, source merge
procedure, repository branch protection, package format, artifact storage,
runtime behavior, registry publication, distribution execution, trust
assignment, capability issuance, module loading, or plugin loading.

Acceptance workflow is a governance workflow layer. It is not a source
merge engine, implementation gate, package manager, installer, loader,
runtime service, registry publisher, trust issuer, distribution engine, or
capability issuer.

Any implementation-specific, merge-specific, runtime-specific,
publication-specific, distribution-specific, trust-specific, or
capability-issuance-specific interpretation fails closed until later
reviewed RFCs define exact behavior.

## Acceptance Subject

An acceptance workflow always binds to exactly one governance subject.

An acceptance subject is the exact governance subject whose evidence set is
being reviewed.

An acceptance subject may be associated with:

1. Capability identity.
2. Capability manifest reference.
3. Lifecycle state reference.
4. Registry governance context.
5. Trust context.
6. Distribution policy context.
7. Evidence records.
8. Exact reviewed subject SHA.

Acceptance targets only the evidence set bound to an exact governance
subject.

Acceptance never targets runtime, package, executable artifact, module,
plugin, registry publication, implementation, capability token, process, or
memory state.

Acceptance does not create a governance subject. It reviews evidence bound
to an existing governance subject.

Changing the governance subject, evidence set, subject SHA, identity,
manifest scope, lifecycle context, registry context, trust context, or
distribution context creates a different acceptance subject unless a later
reviewed RFC defines exact narrower behavior.

## Acceptance Evidence Set

An acceptance evidence set is the exact set of evidence records submitted
for one acceptance workflow.

An acceptance evidence set must be:

1. Exact-subject bound.
2. Evidence-record based.
3. Reviewable.
4. Referenced by exact identifiers.
5. Stable for the workflow decision being made.
6. Non-authoritative by itself.

One acceptance workflow reviews one evidence set for one acceptance
subject.

Evidence set presence is not evidence acceptance. Evidence set completeness
is not implementation approval. Evidence set review is not runtime
authority.

## Exact-SHA Binding

Acceptance workflow is exact-SHA bound.

The conceptual binding chain is:

```text
acceptance_record
  -> exact_governance_subject
  -> exact_main_sha_or_reviewed_subject_identifier
  -> evidence_set
  -> acceptance_decision_record
```

Acceptance binds to the reviewed governance subject and evidence set. It
does not bind to an implied implementation, runtime object, package,
process, memory state, loader state, plugin state, workspace state, or
capability token.

Acceptance is not repository state.

Exact-SHA binding may use:

1. Exact main SHA.
2. Exact reviewed subject SHA.
3. Exact decision subject SHA.
4. Exact evidence record identifiers.
5. Exact evidence set identifier.
6. Exact acceptance decision record identifier.

This RFC does not define canonical hash construction, digest algorithm,
artifact digest format, package digest format, source merge mechanics, or
signature format.

Missing, ambiguous, stale, inherited, aliased, superseded, or differently
scoped subject binding fails closed.

## Evidence Submission

Evidence submission requests acceptance workflow review for an exact
evidence set.

Evidence submission must identify:

1. Exact acceptance subject.
2. Exact evidence set.
3. Evidence record identifiers.
4. Evidence model reference.
5. Exact reviewed subject SHA or identifier.
6. Governing RFCs.
7. Review reason.
8. Non-authorization notice.

Evidence submission is not evidence acceptance.

Submission must not be interpreted as any authority listed in Explicit
Non-Authorization.

## Acceptance Review

Acceptance review interprets the submitted evidence set for workflow
purposes.

Acceptance review may evaluate:

1. Evidence subject consistency.
2. Evidence set completeness.
3. Exact-SHA binding.
4. Evidence identity consistency.
5. Evidence source consistency.
6. Evidence reference integrity.
7. Registry, trust, and distribution context consistency.
8. Quarantine, rejection, deferral, or supersession conditions.
9. Post-merge verification requirements.
10. Non-authorization compliance.

Acceptance review does not approve implementation by itself.

Review output is advisory until a maintainer decision records a workflow
result.

## Evidence Interpretation Boundary

Acceptance workflow is the first Phase-20 RFC that may interpret evidence
for acceptance workflow purposes.

Evidence interpretation means the workflow may read evidence records,
compare them to exact-subject requirements, and determine whether the
evidence set supports an acceptance workflow outcome.

Evidence interpretation does not prove implementation correctness, approve
implementation, merge source, assign trust, publish registry entries,
authorize distribution, issue capabilities, enable runtime behavior,
execute packages, or load modules or plugins.

Any interpretation that turns evidence into authority fails closed.

## Reviewer Authority

A reviewer records findings only.

A reviewer may confirm scope and exact-SHA binding, identify missing,
ambiguous, or conflicting evidence, identify quarantine conditions, and
recommend acceptance, rejection, quarantine, deferral, or supersession.

A reviewer must not approve implementation, grant authority, activate
runtime, publish registry entries, assign trust, authorize distribution,
issue capabilities, merge source, execute packages, or load modules or
plugins.

Reviewer findings are governance input, not authority.

## Reviewer Finding

A reviewer finding is a review record produced during acceptance workflow.

A reviewer finding may include:

1. Reviewed evidence set reference.
2. Exact acceptance subject reference.
3. Finding summary.
4. Evidence consistency notes.
5. Missing evidence notes.
6. Ambiguity or conflict notes.
7. Recommended workflow result.
8. Non-authorization notice.

A reviewer finding does not decide acceptance.

Reviewer finding presence must not be interpreted as any authority listed
in Explicit Non-Authorization.

## Maintainer Decision

A maintainer decision records the acceptance workflow result for a reviewed
evidence set.

Allowed maintainer acceptance decision results are:

1. `accepted`
2. `rejected`
3. `quarantined`

No other maintainer acceptance decision result is defined by this RFC.

A maintainer decision must identify:

1. Exact decision subject.
2. Exact acceptance subject.
3. Exact evidence set.
4. Reviewer findings considered.
5. Decision result.
6. Reason for decision.
7. Governing RFCs.
8. Exact-SHA binding.
9. Non-authorization notice.
10. Fail-closed handling for later ambiguity.

Maintainer decision records evidence workflow state only.

Maintainer decision never activates runtime, merges implementation,
approves implementation, issues capability authority, publishes registry
entries, assigns trust, authorizes distribution, executes packages, or
loads modules or plugins.

## Acceptance States

Acceptance states are governance workflow states.

The normal state path is:

```text
submitted
  -> reviewing
  -> accepted | rejected | quarantined
```

No state implies any authority listed in Explicit Non-Authorization.

| State | Meaning | Authority result |
|---|---|---|
| `submitted` | Evidence set has been submitted for review | No acceptance |
| `reviewing` | Evidence set is under acceptance review | No approval |
| `accepted` | Evidence set passed acceptance workflow for the exact subject | No implementation authority |
| `rejected` | Evidence set did not pass acceptance workflow | No deletion or revocation by itself |
| `quarantined` | Evidence set is held due to ambiguity, conflict, or safety concern | No authority |

State transitions must be explicit, auditable, and exact-subject bound.

Unknown, implicit, inherited, stale, or differently scoped states fail
closed.

## Acceptance Outcomes

Acceptance outcomes are governance outcomes only.

This RFC defines:

| Outcome | Meaning | Authority result |
|---|---|---|
| `accepted` | Evidence set is accepted for the exact workflow subject | No implementation approval |
| `rejected` | Evidence set is rejected for the exact workflow subject | No authority transfer |
| `quarantined` | Evidence set is held for unresolved ambiguity or concern | No authority |
| `deferred` | Workflow is delayed before maintainer acceptance decision | No acceptance |
| `superseded` | Workflow is replaced by a later exact reviewed workflow | No inheritance |

`deferred` and `superseded` are workflow dispositions. They are not
maintainer acceptance decision results.

Outcome presence must not be interpreted as implementation approval, source
merge authority, runtime activation, trust assignment, registry
publication, distribution authority, or capability issuance.

## Explicit Separation

Acceptance workflow concepts do not imply authority-bearing outcomes.

| Acceptance concept | Is not |
|---|---|
| Evidence accepted | Implementation accepted |
| Evidence reviewed | Runtime enabled |
| Evidence interpreted | Capability issued |
| Workflow completed | Package executable |
| Review finished | Registry published |
| Maintainer decision | Source merge authority |
| Post-merge PASS | Implementation authority |

No concept in this table implies another by default.

Unknown acceptance, implementation, runtime, issuance, publication, or
distribution readings fail closed.

## Rejection Handling

Rejection records that an evidence set did not pass acceptance workflow for
the exact acceptance subject.

Rejected evidence sets remain auditable.

Rejection does not delete history, delete evidence records, delete the
acceptance subject, revoke another record, transfer authority to a
replacement, establish alias or supersession by itself, prove fault by
itself, or block later resubmission by itself.

A rejected evidence set may be resubmitted only through a later reviewed
acceptance workflow. Resubmission must not inherit acceptance,
implementation approval, trust, publication, distribution, capability
issuance, or runtime authority.

## Quarantine Handling

Quarantine is the safe workflow result for unresolved evidence ambiguity.

An evidence set may be quarantined when review identifies subject
ambiguity, evidence set ambiguity, exact-SHA binding ambiguity, evidence
reference conflict, registry context conflict, trust context conflict,
distribution context conflict, missing review prerequisite, safety concern,
or incompatible interpretation across governing records.

Quarantine is not acceptance, rejection, implementation approval, source
merge authority, trust assignment, registry publication, distribution
authority, capability issuance, or runtime activation.

Quarantine does not prove fault and does not grant authority to competing
records.

## Deferral And Supersession Handling

Deferral and supersession are workflow dispositions, not acceptance
decisions.

Deferral may record that an evidence set requires later information before
a maintainer acceptance decision can be made.

Supersession may record that a later exact workflow replaces the current
workflow for review purposes.

Deferral or supersession does not accept evidence, reject evidence,
quarantine evidence, approve implementation, merge source, assign trust,
publish registry entries, authorize distribution, issue capabilities, or
activate runtime behavior.

Supersession inheritance is denied unless a later reviewed RFC defines
exact narrower behavior.

## Post-Merge Exact-SHA Verification

Post-merge exact-SHA verification is a governance verification step after a
later reviewed merge path has produced an exact main SHA.

The conceptual verification path is:

```text
acceptance
  -> later reviewed merge path
  -> exact_main_sha
  -> post_merge_verification
  -> closed
```

Every arrow is a governance dependency. No arrow implies implementation
approval, source merge authority, runtime activation, package execution,
module loading, plugin loading, capability issuance, publication, or
distribution.

Post-merge verification may confirm:

1. Exact merged SHA.
2. Exact accepted evidence set.
3. Exact acceptance decision record.
4. Expected file or record presence.
5. Expected non-authorization notices.
6. Expected CI or governance check results.
7. No unexpected scope expansion.

Post-merge PASS is not implementation authority.

Post-merge verification records exact-SHA verification only. It never
records execution authority.

## Relationship Boundaries

Acceptance workflow may consume prior Phase-20 governance records as review
context only.

| Prior record | Accepted use | Denied reading |
|---|---|---|
| `PHASE20_CAPABILITY_EVIDENCE_MODEL.md` | Interpret exact-subject evidence records for acceptance workflow purposes | Evidence presence, source success, CI success, test result, validation result, or integrity is not acceptance by implication |
| `PHASE20_REGISTRY_MODEL.md` and `PHASE20_REGISTRY_GOVERNANCE.md` | Use registry records and registry governance decisions as context | Registry presence or acceptance is not evidence acceptance, publication, distribution, implementation approval, issuance, or runtime activation |
| `PHASE20_TRUST_MODEL.md` | Use trust context as review context | Trust input, claim, proof reference, signature presence, context, or assessment class is not acceptance, implementation approval, trust assignment, distribution, issuance, or runtime activation |
| `PHASE20_DISTRIBUTION_POLICY.md` | Use distribution policy context as review context | Distribution eligibility or input presence is not acceptance, publication, distribution execution, implementation approval, issuance, or runtime activation |

Evidence acceptance requires explicit acceptance workflow review and a
maintainer decision result of `accepted`.

Registry records never inherit authority from acceptance workflow.

Acceptance workflow does not modify trust context.

Acceptance workflow does not execute distribution.

Ambiguous, stale, inherited, unaccepted, or differently scoped relationship
material fails closed for acceptance workflow.

## Implementation Decision Boundary

Acceptance workflow is a prerequisite for later implementation decision
records.

Acceptance workflow does not define implementation decision authority.

A later implementation decision, if accepted, must define:

1. Exact implementation subject.
2. Exact accepted evidence set.
3. Exact acceptance decision record.
4. Exact bounded behavior being authorized.
5. Exact denied behaviors.
6. Exact source scope.
7. Required review path.
8. Required post-merge verification.
9. Runtime boundary.
10. Non-authorization notice for anything outside scope.

Until such a reviewed implementation decision exists, implementation
authority remains denied.

## Acceptance Validation Model

Acceptance workflow validation is conceptual and fail-closed.

Acceptance material is invalid for governance review when:

1. Acceptance subject is missing or ambiguous.
2. Evidence set is missing or ambiguous.
3. Evidence records are broken, stale, ambiguous, inherited, or differently
   scoped.
4. Exact-SHA binding is missing or ambiguous.
5. Reviewer finding is treated as maintainer decision.
6. Maintainer decision is missing for accepted, rejected, or quarantined
   state.
7. Workflow disposition is treated as acceptance decision.
8. Acceptance depends on unaccepted proof or signature semantics.
9. Acceptance depends on trust assignment.
10. Acceptance depends on publication or distribution authority.
11. Acceptance depends on runtime-observed state.
12. Acceptance relies on alias or supersession without accepted rules.
13. Acceptance material implies implementation approval.
14. Acceptance material implies source merge authority.
15. Acceptance material implies capability issuance.
16. Acceptance material implies runtime activation.

Validation failure grants no authority. It requires correction, rejection,
quarantine, deferral, supersession, dispute recording, or a later reviewed
decision path.

Acceptance validation is not acceptance.

Validation produces only a validation result.

Validation never produces an acceptance result.

## Acceptance Invariants

Every later Phase-20 RFC must preserve these acceptance invariants:

1. Acceptance workflow binds to one exact governance subject.
2. Acceptance workflow reviews one exact evidence set.
3. One acceptance workflow has one final maintainer acceptance decision.
4. Acceptance decision requires exact-SHA binding.
5. Acceptance history is immutable by default after review.
6. Acceptance audit history is preserved.
7. Reviewer findings do not decide acceptance.
8. Maintainer decision is required for accepted, rejected, or quarantined
   state.
9. Deferred and superseded are workflow dispositions, not acceptance
   decisions.
10. Evidence accepted is not implementation accepted.
11. Evidence reviewed is not runtime enabled.
12. Evidence interpreted is not capability issued.
13. Workflow completed is not package executable.
14. Review finished is not registry published.
15. Post-merge PASS is not implementation authority.
16. Acceptance workflow does not imply source merge authority.
17. Acceptance workflow does not imply trust assignment.
18. Acceptance workflow does not imply registry publication.
19. Acceptance workflow does not imply distribution authority.
20. Acceptance workflow does not imply capability issuance.
21. Acceptance workflow does not imply runtime activation.
22. Ambiguity fails closed.

Violation of any invariant fails closed.

## Later RFC Dependencies

The acceptance workflow model is a prerequisite for later Phase-20 decision
paths.

| Later record | Acceptance workflow relationship |
|---|---|
| Implementation Decision | May use accepted evidence only after a separate reviewed implementation decision. |
| Implementation Slice | May define exact bounded source scope only after implementation decision authority. |
| Implementation Review | May review implementation evidence without inheriting acceptance authority. |
| Runtime Decision | May define runtime effects only after separate reviewed runtime authority, if ever authorized. |

Later RFCs may narrow acceptance workflow use. They must not broaden this
workflow model into implementation approval, source merge authority, trust
assignment, registry publication, distribution authority, capability
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
  -> Capability Evidence Model
  -> Capability Acceptance Workflow
  -> Implementation Decision
  -> Implementation Slice
  -> Implementation Review
  -> Runtime Decision
```

Every arrow means a governance dependency. It does not imply implementation
approval, source merge authority, publication, distribution, installation,
execution, issuance, or runtime activation.

## Explicit Non-Authorization

This acceptance workflow RFC does not authorize:

1. Runtime activation.
2. General runtime authority.
3. Phase-20 implementation.
4. Implementation approval.
5. Source acceptance or source merge authority.
6. Trust assignment.
7. Trust issuer authority.
8. Registry authority.
9. Registry publication.
10. Publication authority.
11. Distribution authority.
12. Distribution execution.
13. Capability issuance.
14. Package installation, loading, execution, scheduling, or publication.
15. Module loading.
16. Workspace creation, workspace runtime, or real mounts.
17. Plugin host, plugin loading, or plugin instantiation.
18. Capability token minting.
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
trust authority, evidence authority, acceptance authority, proof authority,
execution authority, constitutional authority, registry authority,
distribution authority, publication authority, capability issuance
authority, package authority, module authority, plugin authority, Semantic
CLI authority, AI Runtime authority, agent authority, or Ring0 authority.

## Non-Goals

This document does not define or authorize:

1. Implementation approval or implementation decision authority.
2. Implementation slice authority.
3. Source acceptance or source merge authority.
4. Repository branch protection.
5. Runtime activation or general runtime authority.
6. Trust assignment or trust issuer authority.
7. Registry authority or registry publication.
8. Publication workflow or publication approval.
9. Distribution authority or distribution execution.
10. Package format, repository, installation, loading, or execution.
11. Artifact storage or binary format.
12. Proof verification, signature verification, or signature acceptance.
13. Capability token minting or capability issuance.
14. Module loading.
15. Workspace creation, workspace runtime, or real mounts.
16. Plugin host, plugin loading, or plugin instantiation.
17. Semantic CLI execution or verdict authority.
18. AI Runtime authority.
19. Agent behavior.
20. New syscalls.
21. Kernel ABI expansion.
22. Workflow-threshold, baseline, dependency, or Ring0 policy changes.

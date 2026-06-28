# Phase-20 Registry Governance

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
`PHASE20_CAPABILITY_LIFECYCLE.md`, and
`PHASE20_REGISTRY_MODEL.md`. In case of conflict, those documents prevail
unless this registry governance RFC is the narrower Phase-20 registry
governance record for the exact planning scope identified below.

**Status:** PHASE-20 REGISTRY GOVERNANCE RFC / REGISTRY ADMISSION
GOVERNANCE MODEL ONLY / NO REGISTRY PUBLICATION / NO TRUST ASSIGNMENT / NO
DISTRIBUTION AUTHORITY / NO EVIDENCE ACCEPTANCE / NO IMPLEMENTATION
AUTHORITY / NO RUNTIME ACTIVATION / NO GENERAL RUNTIME AUTHORITY / NO
CAPABILITY ISSUANCE
**Registry governance date:** 2026-06-29
**Registry governance id:** `ayken.phase20.registry_governance.v1`
**Registry governance base main SHA:** `9a12cb7b7d9012006667324501a26fe680ae28de`
**Current phase pointer:** `CURRENT_PHASE=20`
**Authority boundary:** Registry admission governance model only; not
registry publication, not trust authority, not distribution authority, not
evidence acceptance, not implementation authority, not runtime activation,
not general runtime authority, not package installation, not package
execution, not module loading, not workspace runtime, not plugin loading,
not capability token minting, not capability issuance, not trust
assignment, not Semantic CLI authority, not AI Runtime authority, not agent
authority, not syscall expansion, not kernel ABI expansion, not
workflow-threshold, baseline, dependency, or Ring0 authority.

## Purpose

This document defines the Phase-20 governance model for admitting capability
registry records.

It answers one question:

```text
How does a capability registry record pass through registry governance?
```

It does not answer:

```text
How is a registry record published?
How is a registry record distributed?
How is trust assigned?
How is evidence accepted?
How is a capability implemented, activated, issued, loaded, or run?
```

Those questions belong to later Phase-20 RFCs and implementation decisions.

## Core Rule

```text
registry governance != registry publication
registry admission != trust assignment
registry accepted != runtime authority
registry governance != implementation authority
registry governance != distribution authority
registry governance != capability issuance
reviewer recommendation != maintainer decision
maintainer decision != authority expansion
```

Registry governance makes admission review explicit and auditable. It does
not make registry records published, trusted, distributed, implemented,
executable, loadable, installable, runnable, issued, or authority-bearing.

Unknown authority readings fail closed.

## Governance Mission

The mission of registry governance is to define the review path for deciding
whether an inert registry record may be accepted as a governance record.

Registry governance exists to make the following reviewable:

1. Admission requests.
2. Admission reviews.
3. Reviewer findings.
4. Maintainer decisions.
5. Quarantine records.
6. Rejection records.
7. Governance decision records.
8. Governance audit history.

The registry governance model itself grants no publication, trust,
distribution, evidence, implementation, runtime, or capability issuance
authority.

Each later use requires its own reviewed RFC or decision path.

## Governance Definition

Registry governance is the admission-review process for inert registry
records defined by `PHASE20_REGISTRY_MODEL.md`.

Registry governance may accept, reject, or quarantine a registry record as a
governance record.

Registry governance is not:

1. Registry publication.
2. Trust assignment.
3. Distribution.
4. Evidence acceptance.
5. Capability issuance.
6. Source acceptance.
7. Implementation authority.
8. Runtime activation.
9. Package repository behavior.
10. Module repository behavior.
11. Plugin marketplace behavior.
12. Semantic CLI, AI Runtime, or agent authority.

## Governance Principles

Every registry governance rule must preserve these principles:

1. **Explicit admission:** registry admission must be explicitly reviewed
   and recorded.
2. **No bypass:** no record may become admitted through implicit,
   inherited, trust-based, publication-based, runtime-observed, or
   tool-observed state.
3. **Reviewer separation:** reviewer findings support governance; they do
   not decide admission.
4. **Maintainer decision:** acceptance, rejection, or quarantine requires a
   maintainer decision record.
5. **Admission is not publication:** accepted registry governance state does
   not publish the record.
6. **Admission is not trust:** accepted registry governance state does not
   assign trust.
7. **Admission is not distribution:** accepted registry governance state
   does not authorize distribution.
8. **Admission is not implementation:** accepted registry governance state
   does not accept source or implementation.
9. **Audit preserved:** governance history must remain reviewable after
   rejection, quarantine, revocation, retirement, or later supersession.
10. **Fail-closed:** missing, ambiguous, stale, inherited, colliding, or
    differently scoped governance material fails closed.

## Governance Components

The conceptual registry governance model is composed of:

| Component | Purpose | Authority result |
|---|---|---|
| Admission request | Requests governance review for a registry record | No admission |
| Admission review | Evaluates the request against accepted prerequisites | No decision authority |
| Reviewer finding | Records review observations and recommendations | No admission authority |
| Maintainer decision | Accepts, rejects, or quarantines a registry record as governance state | No publication, trust, distribution, implementation, issuance, or runtime authority |
| Quarantine record | Holds a record due to ambiguity, collision, dispute, or safety concern | No authority |
| Rejection record | Records denial of admission | No deletion or revocation by itself |
| Governance decision record | Records the exact decision subject and result | No authority expansion |
| Governance audit record | Preserves historical review and decision context | No evidence acceptance |

These components are conceptual. They do not define a database schema,
storage engine, API endpoint, command, workflow implementation, package
format, wire format, runtime object, or marketplace behavior.

## Admission Request

An admission request asks for registry governance review of an inert registry
record.

An admission request must identify:

1. Exact registry entry or registry record subject.
2. Capability identity reference.
3. Manifest reference.
4. Lifecycle state reference.
5. Registry model reference.
6. Reason for admission review.
7. Applicable governing RFCs.
8. Non-authorization notice.

An admission request is not admission.

It must not be treated as publication, trust assignment, distribution,
evidence acceptance, source acceptance, implementation authority, capability
issuance, or runtime activation.

## Admission Review

An admission review evaluates whether an admission request is complete,
consistent, and bounded under accepted Phase-20 RFCs.

Admission review may evaluate:

1. Identity consistency.
2. Manifest consistency.
3. Lifecycle consistency.
4. Registry model consistency.
5. Subject/scope consistency.
6. Reference integrity.
7. Non-bypass compliance.
8. Quarantine or rejection conditions.
9. Governance audit completeness.

Admission review does not decide admission by itself.

Review output is advisory until a maintainer decision records a governance
result.

## Reviewer

A reviewer examines an admission request and records findings.

A reviewer may:

1. Confirm scope.
2. Identify ambiguity.
3. Identify missing prerequisites.
4. Identify reference conflicts.
5. Recommend acceptance.
6. Recommend rejection.
7. Recommend quarantine.
8. Request clarification.

A reviewer must not:

1. Admit a registry record by review alone.
2. Publish a registry record.
3. Assign trust.
4. Accept evidence.
5. Authorize distribution.
6. Accept source or implementation.
7. Issue capability authority.
8. Activate runtime behavior.

Reviewer findings are governance input, not authority.

## Maintainer Decision

A maintainer decision records the governance result for an admission
request.

Allowed maintainer decision results are:

1. `accepted`
2. `rejected`
3. `quarantined`

No other decision result is defined by this RFC.

The maintainer decision must identify:

1. Exact decision subject.
2. Exact registry record subject.
3. Reviewer findings considered.
4. Decision result.
5. Reason for decision.
6. Governing RFCs.
7. Non-authorization notice.
8. Fail-closed handling for later ambiguity.

A maintainer decision accepts only governance admission state. It does not
publish, trust, distribute, implement, issue, execute, load, install, or run
anything.

## Governance Decision Record

A governance decision record binds the admission result to an exact reviewed
subject.

It must record:

1. Decision identifier.
2. Decision date.
3. Exact subject SHA or reviewed subject identifier.
4. Registry record subject.
5. Result.
6. Denied authorities.
7. Required later RFCs or decisions.
8. Post-decision audit requirements, if any.

A governance decision record is not evidence acceptance, publication,
distribution, trust assignment, implementation authority, or runtime
authority.

## Governance Audit Record

A governance audit record preserves the review and decision context.

It may include:

1. Admission request references.
2. Reviewer findings.
3. Maintainer decision.
4. Quarantine or rejection reason.
5. Reference integrity notes.
6. Later follow-up requirements.

Audit records preserve history. They do not create authority, evidence
acceptance, trust, publication, distribution, implementation authority, or
runtime authority.

## Admission Pipeline

The registry governance admission pipeline is:

```text
submission
  -> admission_review
  -> validation
  -> reviewer_finding
  -> maintainer_decision
  -> accepted | rejected | quarantined
```

No stage may be skipped.

This pipeline is conceptual. It does not define a tool, command, workflow
implementation, storage implementation, or runtime process.

## Admission Validation

Admission validation is fail-closed.

An admission request is invalid when:

1. Capability identity is missing or ambiguous.
2. Manifest reference is missing or ambiguous.
3. Lifecycle state is missing or ambiguous.
4. Registry model reference is missing or ambiguous.
5. Subject/scope relationship is ambiguous.
6. Registry entry material conflicts with accepted RFCs.
7. Registry relationship material implies authority transfer.
8. Admission depends on undefined trust, distribution, evidence, or
   publication semantics.
9. Admission depends on runtime-observed state.
10. Admission depends on implementation existence.
11. Admission depends on a reviewer finding without maintainer decision.
12. Admission depends on inherited evidence or inherited trust.

Validation failure grants no authority. It requires correction, rejection,
or quarantine.

## Non-Bypass Rules

Registry governance must preserve the following non-bypass rules:

1. No direct admission.
2. No implicit admission.
3. No inherited admission.
4. No automatic admission.
5. No trust-based admission.
6. No publication-based admission.
7. No distribution-based admission.
8. No evidence-based admission without governance decision.
9. No runtime-based admission.
10. No implementation-based admission.
11. No reviewer-only admission.
12. No alias-based admission.
13. No supersession-based admission.
14. No registry-presence admission.
15. No tool-observed admission.

Any attempted bypass fails closed.

## Quarantine Handling

Quarantine is the safe governance result for unresolved registry admission
ambiguity.

A registry record may be quarantined when review identifies:

1. Identity collision.
2. Manifest ambiguity.
3. Lifecycle ambiguity.
4. Registry relationship conflict.
5. Subject/scope ambiguity.
6. Evidence conflict.
7. Trust-input conflict.
8. Publication ambiguity.
9. Distribution ambiguity.
10. Authority ambiguity.
11. Safety concern.
12. Incompatible interpretation across governing records.

Quarantine is not acceptance, rejection, publication, trust assignment,
distribution, evidence acceptance, implementation authority, capability
issuance, or runtime action.

Quarantine does not prove fault and does not grant authority to competing
records.

## Rejection Handling

Rejection records denial of registry governance admission.

Rejected registry records remain auditable.

Rejection does not:

1. Delete history.
2. Delete the admission request.
3. Delete reviewer findings.
4. Revoke another record.
5. Quarantine another record by itself.
6. Transfer authority to a replacement.
7. Establish alias or supersession.
8. Prove fault by itself.

A rejected record may be resubmitted only through a later reviewed admission
request. Resubmission must not inherit admission, evidence, trust,
publication, distribution, implementation authority, or runtime authority.

## Accepted Governance State

Accepted registry governance state means the registry record has passed
admission governance as an inert governance record.

Accepted governance state does not mean:

1. Published.
2. Trusted.
3. Distributed.
4. Evidence-accepted.
5. Source-accepted.
6. Implemented.
7. Executable.
8. Loadable.
9. Installable.
10. Runnable.
11. Capability-issued.
12. Runtime-active.

Accepted governance state may be used as input to later trust,
distribution, evidence, acceptance, or implementation RFCs. It does not
authorize those paths by itself.

## Admission Vs Publication

Admission and publication are separate governance concepts.

```text
accepted registry governance state != registry publication
registry publication != trust assignment
trust assignment != runtime authority
runtime authority != capability issuance
```

Registry governance may accept an inert registry record for governance
tracking. It does not publish that record.

Publication rules, distribution channels, mirrors, revocation effects,
quarantine effects, rollback behavior, and marketplace behavior belong to
later RFCs.

## Trust Separation

Registry governance does not assign trust.

Accepted governance state may become trust context for a later trust RFC.
It must not be interpreted as authenticity proof, signature acceptance,
issuer approval, capability issuance, package trust, distribution trust, or
runtime authority.

Trust inputs and proof requirements belong to `PHASE20_TRUST_MODEL.md` or
later reviewed records.

## Distribution Separation

Registry governance does not authorize distribution.

Accepted governance state must not be interpreted as publication, release,
mirror eligibility, package distribution, module distribution, plugin
distribution, marketplace listing, or rollback authority.

Distribution policy belongs to `PHASE20_DISTRIBUTION_POLICY.md` or later
reviewed records.

## Evidence Separation

Registry governance does not accept evidence.

Admission review may identify evidence requirements or evidence references,
but it must not accept evidence, inherit evidence, validate evidence, or
turn evidence into authority.

Evidence package format, exact-subject binding, review requirements, and
post-merge verification belong to later evidence and acceptance RFCs.

## Governance Invariants

Every later Phase-20 RFC must preserve these registry governance invariants:

1. Registry admission is explicit.
2. Registry admission requires review.
3. Reviewer findings do not decide admission.
4. Maintainer decision is required for accepted, rejected, or quarantined
   governance state.
5. Maintainer decision must not bypass governing records.
6. Accepted governance state is not publication.
7. Accepted governance state is not trust assignment.
8. Accepted governance state is not distribution.
9. Accepted governance state is not evidence acceptance.
10. Accepted governance state is not source acceptance.
11. Accepted governance state is not implementation authority.
12. Accepted governance state is not runtime activation.
13. Quarantine grants no authority.
14. Rejection preserves audit history.
15. Governance history is preserved.

Violation of any invariant fails closed.

## Relationship To Later RFCs

The registry governance model is a prerequisite for later Phase-20 RFCs.

| Later RFC | Registry governance relationship |
|---|---|
| `PHASE20_TRUST_MODEL.md` | Uses accepted governance state as trust context without assigning trust. |
| `PHASE20_DISTRIBUTION_POLICY.md` | Uses accepted, rejected, and quarantined governance states for publication and rollback policy without granting distribution. |
| `PHASE20_CAPABILITY_EVIDENCE_MODEL.md` | Uses admission review and governance decision records for evidence binding without evidence inheritance. |
| `PHASE20_CAPABILITY_ACCEPTANCE_WORKFLOW.md` | Uses registry governance decisions for review tracking without implementation authority. |

Later RFCs may narrow registry governance use. They must not broaden
registry governance into publication, trust, distribution, evidence,
implementation, issuance, or runtime authority without a separate reviewed
decision.

## Non-Goals

This document does not define or authorize:

1. Registry publication.
2. Registry storage implementation.
3. Registry API implementation.
4. Database schema.
5. Search service.
6. Package repository.
7. Module repository.
8. Plugin marketplace.
9. Dependency resolver.
10. Trust model.
11. Trust assignment.
12. Distribution policy.
13. Evidence package format.
14. Evidence acceptance.
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
27. Semantic CLI execution or verdict authority.
28. AI Runtime authority.
29. Agent behavior.
30. New syscalls.
31. Kernel ABI expansion.
32. Workflow-threshold, baseline, dependency, or Ring0 policy changes.

## Implementation Gating

No Phase-20 implementation slice may start from this registry governance RFC
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

Registry governance is not implementation authority.

## Explicit Non-Authorization

This registry governance RFC does not authorize:

1. Runtime activation.
2. General runtime authority.
3. Phase-20 implementation.
4. Source acceptance or merge authority.
5. Registry publication.
6. Registry storage implementation.
7. Registry API implementation.
8. Trust assignment.
9. Distribution authority.
10. Evidence acceptance or evidence inheritance.
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

## Registry Governance Conclusion

AykenOS registry governance is the explicit, auditable admission-review
model for inert registry records.

It defines admission requests, admission reviews, reviewer findings,
maintainer decisions, quarantine records, rejection records, governance
decision records, audit records, non-bypass rules, and the separation
between admission and publication.

Registry governance supports later trust, distribution, evidence,
acceptance, and implementation RFCs. It does not authorize any of them.

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

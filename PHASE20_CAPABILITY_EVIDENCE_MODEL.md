# Phase-20 Capability Evidence Model

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
`PHASE20_TRUST_MODEL.md`, and
`PHASE20_DISTRIBUTION_POLICY.md`. In case of conflict, those documents
prevail unless this evidence model RFC is the narrower Phase-20 capability
evidence model record for the exact planning scope identified below.

**Status:** PHASE-20 CAPABILITY EVIDENCE MODEL RFC / EVIDENCE RECORD MODEL
ONLY / NO EVIDENCE ACCEPTANCE / NO IMPLEMENTATION APPROVAL / NO TRUST
ASSIGNMENT / NO REGISTRY AUTHORITY / NO REGISTRY PUBLICATION / NO
PUBLICATION AUTHORITY / NO DISTRIBUTION AUTHORITY / NO CAPABILITY ISSUANCE
/ NO RUNTIME ACTIVATION / NO GENERAL RUNTIME AUTHORITY
**Evidence model date:** 2026-06-29
**Evidence model id:** `ayken.phase20.capability_evidence_model.v1`
**Evidence model base main SHA:** `b47f88bd0ffec1a40e1a948a680835117aec37d3`
**Current phase pointer:** `CURRENT_PHASE=20`
**Authority boundary:** Capability evidence record model only; not evidence
acceptance, not evidence approval, not proof acceptance, not implementation
approval, not source acceptance, not trust assignment, not trust issuer
authority, not registry authority, not registry publication, not
publication authority, not distribution authority, not capability issuance,
not package installation, not package loading, not package execution, not
module loading, not workspace runtime, not plugin loading, not runtime
activation, not general runtime authority, not Semantic CLI authority, not
AI Runtime authority, not agent authority, not syscall expansion, not
kernel ABI expansion, not workflow-threshold, baseline, dependency, or
Ring0 authority.

## Purpose

This document defines the Phase-20 evidence model for AykenOS capability
and registry governance records.

It answers one question:

```text
What is capability evidence, and how is it bound to an exact governance
subject?
```

It does not answer:

```text
How is evidence accepted?
How is implementation approved?
How is trust assigned?
How is a registry record published or distributed?
How is a capability issued, implemented, activated, loaded, or run?
```

Those questions belong to later Phase-20 RFCs and implementation decisions.

## Core Rule

```text
evidence != evidence acceptance
evidence != implementation approval
evidence != trust assignment
evidence != runtime authority
evidence != publication authority
evidence != registry authority
evidence != capability issuance
evidence presence != acceptance
evidence reference != proof acceptance
evidence validation != evidence acceptance
test result != implementation authority
evidence source != evidence record
```

Evidence records. Evidence never decides. Evidence never authorizes.

Evidence modeling makes evidence subjects, sources, references, integrity,
and exact-subject binding reviewable. It does not accept evidence, approve
implementation, assign trust, publish, distribute, issue capabilities,
execute, load, install, activate runtime behavior, or grant authority.

Unknown authority readings fail closed.

## Evidence Mission

The mission of the Phase-20 evidence model is to define inert, immutable,
exact-subject-bound evidence records for capability governance.

Evidence exists so later RFCs can reason about:

1. Evidence subjects.
2. Exact-SHA binding.
3. Evidence identity.
4. Evidence classes.
5. Evidence sources.
6. Evidence references.
7. Evidence integrity.
8. Evidence immutability.
9. Registry relationships.
10. Trust relationships.
11. Distribution relationships.
12. Acceptance workflow inputs.
13. Historical audit.

The evidence model itself grants no evidence acceptance, proof acceptance,
trust assignment, registry publication, distribution, implementation,
runtime, or capability issuance authority.

Each later use requires its own reviewed RFC or decision path.

## Evidence Definition

Capability evidence is an inert governance record that records evidence
material and binds it to an exact governance subject.

An evidence record may describe:

1. The exact evidence subject.
2. The exact subject SHA or reviewed subject identifier.
3. The evidence identity.
4. The evidence class.
5. The evidence source.
6. Evidence references.
7. Evidence integrity material.
8. Evidence scope.
9. Non-authorization notice.

An evidence record is not:

1. Evidence acceptance.
2. Proof acceptance.
3. Trust assignment.
4. Registry authority.
5. Registry publication.
6. Publication authority.
7. Distribution authority.
8. Capability issuance.
9. Source acceptance.
10. Implementation approval.
11. Runtime activation.
12. Semantic CLI, AI Runtime, or agent authority.

## Evidence Scope

This RFC defines only the evidence record model.

It does not define:

1. Evidence acceptance workflow.
2. Evidence package format.
3. Proof verification.
4. Signature verification.
5. Test framework behavior.
6. CI implementation.
7. Artifact storage.
8. Binary format.
9. Registry publication.
10. Distribution execution.
11. Source implementation.
12. Runtime behavior.

Evidence is a governance record layer. It is not an evidence engine, CI
system, test framework, artifact store, package repository, proof verifier,
signature verifier, implementation gate, or runtime service.

Any acceptance-specific, implementation-specific, CI-specific,
proof-specific, artifact-specific, package-specific, runtime-specific, or
storage-specific interpretation fails closed until later reviewed RFCs
define exact behavior.

## Evidence Subject

An evidence subject is the exact governance subject to which evidence is
bound.

An evidence subject may be:

1. Capability identity.
2. Capability manifest.
3. Lifecycle state record.
4. Registry entry.
5. Registry governance decision record.
6. Trust context record.
7. Distribution policy record.
8. Later acceptance workflow record.
9. Later bounded implementation decision record.

An evidence subject is not:

1. A runtime object.
2. A process.
3. A memory state.
4. A transient machine state.
5. A package execution.
6. A loader handle.
7. A plugin instance.
8. A capability token.

Evidence must be bound to exact governance subjects, not dynamic runtime
objects.

Evidence never creates a governance subject. It only references an existing
governance subject.

Changing identity, manifest scope, registry subject, lifecycle context,
trust context, distribution context, or other subject-defining material
creates a different evidence subject unless a later reviewed RFC defines
exact narrower behavior.

## Exact-SHA Binding

Evidence is exact-subject bound.

The conceptual binding chain is:

```text
capability_record
  -> capability_identity
  -> capability_manifest
  -> governance_subject
  -> exact_main_sha_or_reviewed_subject_identifier
  -> evidence_record
```

Evidence binds to the reviewed governance subject, not to an implied
implementation, runtime object, package, process, memory state, loader
state, plugin state, or workspace state.

Exact-SHA binding may use:

1. Exact main SHA.
2. Exact reviewed subject SHA.
3. Exact decision subject SHA.
4. Exact document identifier plus accepted subject SHA.
5. Exact registry governance subject identifier.

This RFC does not define canonical hash construction, digest algorithm,
artifact digest format, package digest format, evidence package format, or
signature format.

Missing, ambiguous, stale, inherited, aliased, superseded, or differently
scoped subject binding fails closed.

## Evidence Identity

Evidence identity distinguishes one evidence record from another.

Evidence identity is conceptually composed of:

```text
(evidence_domain, evidence_subject, evidence_class, evidence_source,
 evidence_reference, evidence_binding)
```

This tuple is conceptual. It is not a manifest syntax, registry key format,
database schema, file path, package name, URL, command, token, trust claim,
proof acceptance, or runtime handle.

Changing identity-defining evidence fields creates a different evidence
record unless a later reviewed RFC defines exact narrower behavior.

Evidence identity remains stable for the lifetime of that evidence record.

Evidence identity does not imply evidence acceptance, proof acceptance,
trust assignment, registry publication, distribution authority,
implementation approval, capability issuance, or runtime activation.

## Evidence Classes

Evidence classes are conceptual labels for evidence records.

The initial evidence class vocabulary is:

| Class | Meaning | Authority result |
|---|---|---|
| Specification evidence | Evidence tied to RFC/specification content | No acceptance |
| Governance evidence | Evidence tied to governance review or process state | No decision authority |
| Validation evidence | Evidence tied to validation output or checks | No acceptance |
| Review evidence | Evidence tied to reviewer observations | No approval |
| Decision evidence | Evidence tied to a decision record | No implementation authority |
| Publication evidence | Evidence tied to publication-related review material | No publication |
| Historical evidence | Evidence retained for audit context | No authority |

These classes are review labels only. They are not evidence acceptance
levels, trust levels, implementation approval levels, publication status,
distribution status, package status, runtime status, or authority grants.

Later RFCs may define narrower evidence class semantics. They must not turn
evidence class presence into authority.

## Evidence Sources

An evidence source is the origin or producer of evidence-relevant material.

Evidence sources may include:

1. Governance documents.
2. RFC records.
3. Review records.
4. Decision records.
5. Validation outputs.
6. CI runs.
7. Test outputs.
8. Logs.
9. Audit notes.
10. Registry records.
11. Trust context records.
12. Distribution policy records.

Evidence source is not evidence record.

For example:

```text
CI run
  -> evidence_source
  -> evidence_record
```

The source may produce or support evidence material. The evidence record is
the inert governance record that binds that material to an exact subject.

Source presence, source success, test success, CI success, review presence,
or decision presence must not be interpreted as evidence acceptance,
implementation approval, trust assignment, publication authority,
distribution authority, capability issuance, or runtime activation.

## Evidence References

An evidence reference is an inert reference from an evidence record to
evidence-relevant material.

Evidence references may point to:

1. Exact SHA.
2. Document id.
3. RFC id.
4. Review id.
5. Decision id.
6. Registry record id.
7. Trust context id.
8. Distribution policy id.
9. CI run id.
10. Validation output id.
11. Audit record id.

Evidence references must not point to authority-bearing dynamic runtime
subjects by implication, including:

1. Runtime object.
2. Process.
3. Memory state.
4. Thread state.
5. Loader state.
6. Plugin instance.
7. Workspace mount.
8. Capability token.

Evidence reference presence does not accept evidence, accept proof, assign
trust, publish, distribute, implement, issue, execute, load, install, or
activate runtime behavior.

Broken, stale, ambiguous, inherited, approximate, or differently scoped
evidence references fail closed.

## Evidence Integrity

Evidence integrity is the reviewable consistency of evidence material,
evidence subject, evidence identity, evidence source, and evidence
references.

Evidence integrity may require:

1. Exact subject binding.
2. Exact reference binding.
3. Stable evidence identity.
4. Source consistency.
5. Class consistency.
6. Scope consistency.
7. Non-authorization notice.
8. Audit preservation.

Evidence integrity is not evidence acceptance.

Integrity checks do not prove truth, assign trust, accept signatures,
accept proof, approve implementation, publish records, distribute records,
issue capabilities, or activate runtime behavior.

## Evidence Immutability

Evidence records are immutable by default after review.

Changing any of the following creates a different evidence record unless a
later reviewed RFC defines exact narrower behavior:

1. Evidence subject.
2. Subject SHA or reviewed subject identifier.
3. Evidence identity.
4. Evidence class.
5. Evidence source.
6. Evidence reference.
7. Evidence binding.
8. Evidence scope.

Evidence immutability preserves auditability. It does not mean the evidence
is accepted, trusted, published, distributed, implementation-approved, or
runtime-active.

Evidence deletion, replacement, supersession, redaction, retention, and
quarantine procedures belong to later reviewed RFCs or decisions.

## Registry Relationship

`PHASE20_REGISTRY_MODEL.md` defines registry records as inert governance
records.

`PHASE20_REGISTRY_GOVERNANCE.md` defines accepted registry governance state
as explicit admission governance, not publication, trust, evidence
acceptance, implementation approval, or runtime authority.

Evidence may reference registry records and registry governance decision
records.

Evidence may reference multiple registry records when each reference is
exact, bounded, and reviewable.

It must not treat registry presence, registry acceptance, registry index
presence, registry view presence, registry snapshot presence, or registry
governance decision presence as evidence acceptance, trust assignment,
publication authority, distribution authority, implementation approval,
capability issuance, or runtime activation.

Registry records never inherit authority from evidence.

Registry ambiguity, collision, quarantine, rejection, stale reference, or
differently scoped registry material fails closed for evidence modeling.

## Trust Relationship

`PHASE20_TRUST_MODEL.md` defines trust inputs, trust claims, proof
references, trust context, and assessment classes without assigning trust.

Evidence may reference trust context and trust inputs.

Evidence never modifies trust context.

It must not treat trust input presence, trust claim presence, proof
reference presence, signature presence, trust context, assessment class, or
trust dispute record as evidence acceptance, proof acceptance, trust
assignment, publication authority, distribution authority, implementation
approval, capability issuance, or runtime activation.

Trust ambiguity, dispute, revocation input, stale trust input, unknown
issuer interpretation, unknown proof interpretation, or differently scoped
trust material fails closed for evidence modeling.

## Distribution Relationship

`PHASE20_DISTRIBUTION_POLICY.md` defines distribution policy as an inert
governance model for future distribution eligibility.

Evidence may reference distribution policy records and distribution policy
inputs.

Distribution may consume evidence under later reviewed rules. Evidence never
consumes distribution.

It must not treat distribution eligibility, distribution input presence,
distribution decision presence, publication context, mirror context,
rollback input, quarantine input, or revocation input as evidence
acceptance, publication authority, distribution authority, implementation
approval, capability issuance, or runtime activation.

Distribution ambiguity, stale distribution input, unaccepted publication
semantics, unaccepted transport semantics, unaccepted package semantics, or
differently scoped distribution material fails closed for evidence
modeling.

## Acceptance Relationship

Evidence model is a prerequisite input for later acceptance workflow.

Acceptance workflow may later define how evidence is submitted, reviewed,
accepted, rejected, quarantined, merged, and verified.

Acceptance workflow is the first later RFC that may interpret evidence.
Evidence model never interprets evidence.

This RFC does not define evidence acceptance.

Evidence records may become inputs to acceptance workflow. They do not
authorize acceptance workflow, acceptance decisions, merge decisions,
implementation approval, or runtime activation by themselves.

## Evidence Validation Model

Evidence validation is conceptual and fail-closed.

Evidence material is invalid for governance review when:

1. Evidence subject is missing or ambiguous.
2. Exact-SHA binding is missing or ambiguous.
3. Evidence identity is missing or ambiguous.
4. Evidence class is missing or ambiguous.
5. Evidence source is missing or ambiguous.
6. Evidence reference is broken, stale, ambiguous, inherited, or
   differently scoped.
7. Evidence material relies on unaccepted proof semantics.
8. Evidence material relies on unaccepted signature semantics.
9. Evidence material relies on unaccepted trust assignment.
10. Evidence material relies on unaccepted publication or distribution
    semantics.
11. Evidence material relies on runtime-observed state.
12. Evidence material relies on alias or supersession without accepted
    rules.
13. Evidence material implies trust assignment.
14. Evidence material implies publication or distribution authority.
15. Evidence material implies capability issuance.
16. Evidence material implies implementation approval or runtime authority.

Validation failure grants no authority. It requires correction, rejection,
quarantine, dispute recording, or a later reviewed decision path.

Evidence validation is not evidence acceptance.

Validation produces only a validation result.

Validation never produces an acceptance result.

## Evidence Invariants

Every later Phase-20 RFC must preserve these evidence invariants:

1. Evidence records are inert governance records.
2. Evidence records are exact-subject bound.
3. Evidence records are immutable by default after review.
4. Evidence records preserve referential consistency.
5. Evidence records do not decide.
6. Evidence records do not authorize.
7. Evidence presence is not evidence acceptance.
8. Evidence reference is not proof acceptance.
9. Evidence validation is not evidence acceptance.
10. Evidence source is not evidence record.
11. Test result is not implementation authority.
12. CI success is not implementation authority.
13. Evidence records do not imply trust assignment.
14. Evidence records do not imply registry authority.
15. Evidence records do not imply registry publication.
16. Evidence records do not imply publication authority.
17. Evidence records do not imply distribution authority.
18. Evidence records do not imply capability issuance.
19. Evidence records do not imply implementation approval.
20. Evidence records do not imply runtime activation.

Violation of any invariant fails closed.

## Later RFC Dependencies

The evidence model is a prerequisite for later Phase-20 RFCs and decision
paths.

| Later record | Evidence model relationship |
|---|---|
| `PHASE20_CAPABILITY_ACCEPTANCE_WORKFLOW.md` | Defines how evidence records are submitted, reviewed, accepted, rejected, quarantined, merged, and verified. |
| First bounded implementation | May use accepted evidence only after a separate reviewed implementation decision. |

Later RFCs may narrow evidence use. They must not broaden this evidence
model into evidence acceptance, implementation approval, trust assignment,
registry authority, publication authority, distribution authority,
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
```

Every arrow means a governance dependency. It does not imply acceptance,
implementation approval, publication, distribution, installation,
execution, issuance, or runtime activation.

## Explicit Non-Authorization

This evidence model RFC does not authorize:

1. Runtime activation.
2. General runtime authority.
3. Phase-20 implementation.
4. Source acceptance or merge authority.
5. Evidence acceptance.
6. Evidence approval.
7. Proof acceptance.
8. Signature acceptance.
9. Authenticity acceptance.
10. Trust assignment.
11. Trust issuer authority.
12. Registry authority.
13. Registry publication.
14. Publication authority.
15. Distribution authority.
16. Distribution execution.
17. Capability issuance.
18. Package installation, loading, execution, scheduling, or publication.
19. Module loading.
20. Workspace creation, workspace runtime, or real mounts.
21. Plugin host, plugin loading, or plugin instantiation.
22. Capability token minting.
23. Semantic CLI execution or verdict authority.
24. AI Runtime authority.
25. Agent behavior.
26. New syscalls.
27. Kernel ABI expansion.
28. Workflow-threshold, baseline, dependency, or Ring0 policy changes.
29. Observability-as-authority.

Unknown authority readings fail closed.

## Architecture Signature

**Prepared by:** Kenan AY
**Role:** AykenOS Architecture Steward
**Document type:** Phase-20 RFC
**Architecture status:** Draft RFC / pending architectural review
**Authority notice:** This signature identifies the architectural authorship
of this RFC. It grants no runtime authority, implementation authority, trust
authority, evidence authority, proof authority, execution authority,
constitutional authority, registry authority, distribution authority,
publication authority, capability issuance authority, package authority,
module authority, plugin authority, Semantic CLI authority, AI Runtime
authority, agent authority, or Ring0 authority.

## Non-Goals

This document does not define or authorize:

1. Evidence acceptance.
2. Evidence approval.
3. Evidence package format.
4. Proof verification.
5. Proof acceptance.
6. Signature verification.
7. Signature acceptance.
8. Authenticity acceptance.
9. Test framework behavior.
10. CI implementation.
11. Artifact storage.
12. Binary format.
13. Trust assignment.
14. Trust issuer authority.
15. Registry authority.
16. Registry publication.
17. Publication workflow.
18. Publication approval.
19. Distribution authority.
20. Distribution execution.
21. Acceptance workflow.
22. Source implementation.
23. Implementation approval.
24. Runtime activation.
25. Package installation, loading, execution, scheduling, or publication.
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

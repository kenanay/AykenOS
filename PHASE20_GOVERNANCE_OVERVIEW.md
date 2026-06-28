# Phase-20 Governance Overview

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, the Phase-18 Platform Constitution reference set,
`docs/specs/phase18-platform-constitution/AUTHORITY_DRIFT_GUARD.md`,
`docs/specs/phase18-platform-constitution/TERMINOLOGY_AUDIT.md`,
`PHASE19_RUNTIME_DECISION.md`, the Phase-19 Runtime RFC set,
`docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_MATRIX.md`,
`PHASE19_CLOSURE_DECISION.md`,
`PHASE20_POINTER_TRANSITION_CANDIDATE.md`, and
`PHASE20_POINTER_TRANSITION_DECISION.md`. In case of conflict, those
documents prevail unless this overview is the narrower Phase-20 governance
entry record for the planning/governance scope identified below.

**Status:** PHASE-20 GOVERNANCE OVERVIEW / CAPABILITY AND REGISTRY
ECOSYSTEM PLANNING-GOVERNANCE ENTRY / NO RUNTIME ACTIVATION / NO GENERAL
RUNTIME AUTHORITY / NO IMPLEMENTATION AUTHORITY / NO CAPABILITY ISSUANCE /
NO REGISTRY PUBLICATION
**Overview date:** 2026-06-28
**Overview id:** `ayken.phase20.governance_overview.v1`
**Phase-20 pointer transition subject SHA:**
`29dfa940c0bd2c07d7a5403a75bb0ed5f0c2b2a9`
**Current phase pointer:** `CURRENT_PHASE=20`
**Authority boundary:** Governance overview only; not a Phase-20
implementation decision, not runtime activation, not general runtime
authority, not source acceptance, not package installation, not package
execution, not module loading, not workspace runtime, not plugin loading,
not capability issuance, not capability token minting, not trust assignment,
not registry publication, not Semantic CLI authority, not AI Runtime
authority, not agent authority, not syscall expansion, not kernel ABI
expansion, not workflow-threshold, baseline, dependency, or Ring0 authority.

## Purpose

This document is the Phase-20 constitutional entry overview.

It records the governance boundary for Phase-20 after the accepted pointer
transition to:

```text
CURRENT_PHASE=20
```

Phase-20 begins as the planning and governance phase for the AykenOS
capability and registry ecosystem.

This overview does not implement capability behavior, registry behavior,
distribution behavior, trust behavior, package behavior, module behavior,
Semantic CLI behavior, AI Runtime behavior, or agent behavior.

## Handoff From Phase-19

Phase-19 closed the bounded Platform Runtime MVP planning, admission,
receipt, reference-integrity, evidence, review, and closure chain.

Phase-20 inherits only the closed governance boundary from Phase-19. It does
not inherit runtime activation, general runtime authority, implementation
authority, capability authority, registry authority, trust authority,
Semantic CLI authority, AI Runtime authority, or agent authority.

The Phase-20 pointer transition published `CURRENT_PHASE=20` only as a
planning/governance phase. The pointer transition must not be interpreted as
authorization to install, execute, load, publish, issue, trust, mount,
schedule, or grant anything.

## Core Rule

```text
Phase-20 governance != Phase-20 implementation
capability model != capability issuance
registry model != registry publication
manifest schema != package loading or execution
trust model != trust assignment
CURRENT_PHASE=20 != runtime activation
```

Unknown authority readings fail closed.

## Phase-20 Design Principles

All Phase-20 RFCs, decisions, evidence packages, reviews, and later
implementation proposals must preserve the following principles:

1. **Fail closed:** missing, ambiguous, stale, or differently scoped
   authority is denied.
2. **Declarative first:** manifests, capability records, registry records,
   trust inputs, and evidence records are inert until a separate decision
   grants bounded authority.
3. **Exact-SHA binding:** evidence and acceptance are bound to the exact
   reviewed subject SHA and cannot be inherited by another subject.
4. **Review before authority:** no capability, registry, trust, package,
   module, workspace, plugin, Semantic CLI, AI Runtime, or agent authority is
   produced without a reviewed decision.
5. **Planning before implementation:** Phase-20 architecture and RFC records
   must precede implementation slices.
6. **Capability is not execution:** a capability record is not a package,
   executable, module, plugin, token, workspace handle, trust assignment, or
   runtime grant.
7. **Registry is not runtime:** a registry record organizes and governs
   references; it does not install, load, execute, publish, issue, or trust.
8. **Evidence is not authority:** evidence supports review, but only the
   applicable decision record can grant bounded authority.

## Capability Concept

A Phase-20 capability is a constitutional record type for describing a
bounded, identity-bearing, lifecycle-managed capability candidate.

A capability record may define metadata, identity, lifecycle state,
governance requirements, evidence requirements, and registry relationships.
It must remain fail-closed and non-authority-bearing unless a later reviewed
decision explicitly grants a bounded behavior for an exact subject.

A capability is not:

1. A runtime permission.
2. A bearer token.
3. A package or executable.
4. A module or plugin.
5. A workspace handle.
6. A trust assignment.
7. A registry publication.
8. Semantic CLI, AI Runtime, or agent authority.

## Capability Mission

The Phase-20 capability model exists to establish a stable, reviewable,
identity-bound representation of future operating-system behaviors.

Capabilities provide constitutional objects around which later
implementation, evidence, trust, distribution, lifecycle, and acceptance can
be organized.

Capabilities are architectural anchors, not executable runtime entities.
They let AykenOS describe, review, version, evidence, and govern future
behavior before any implementation or activation authority is considered.

## Manifest Concept

A Phase-20 capability manifest is declarative and inert.

It may describe capability metadata, identity inputs, dependency references,
evidence references, lifecycle state, and governance requirements. It must
not be read as a parser request, package installation request, module load
request, plugin load request, execution request, trust request, token
request, workspace request, publication request, or authority grant.

## Registry Concept

A Phase-20 registry exists to organize, reference, validate, and govern
capability records.

The registry may define indexes, integrity rules, metadata references,
revocation references, quarantine references, and evidence references. It is
not an execution service, package repository, module loader, plugin host,
trust issuer, capability issuer, workspace runtime, Semantic CLI authority,
AI Runtime authority, or agent authority.

Registry publication remains denied until a later reviewed decision grants a
bounded publication path for an exact subject.

## Trust Concept

A Phase-20 trust model records trust inputs and proof requirements. It does
not assign trust.

Trust records may describe issuer constraints, proof material, revocation
inputs, audit requirements, and non-bypass rules. They must not be
interpreted as trust assignment, authenticity proof, signature acceptance,
capability issuance, registry publication, package execution, or runtime
authority.

## Conceptual Architecture Flow

The Phase-20 planning flow is:

```text
Capability Concept
  -> Capability Identity
  -> Capability Manifest
  -> Capability Lifecycle
  -> Registry Model
  -> Registry Governance
  -> Trust Model
  -> Distribution Policy
  -> Evidence Model
  -> Acceptance Workflow
  -> First Bounded Implementation Candidate
```

Every arrow means a governance dependency. It does not imply execution,
issuance, publication, loading, mounting, trust assignment, or runtime
activation.

## Registry Placement

The registry sits after capability identity and manifest definition, and
before evidence-backed acceptance.

```text
Capability
  -> Identity
  -> Manifest
  -> Registry
  -> Evidence
  -> Acceptance
```

The registry gives AykenOS a governed place to organize and reference
capability records. It does not make those records active.

## Capability Lifecycle Vision

The long-form capability lifecycle should be modeled as:

```text
Concept
  -> Identity
  -> Manifest
  -> Review
  -> Acceptance
  -> Registry
  -> Implementation Candidate
  -> Implementation
  -> Distribution
  -> Retirement
```

This lifecycle is a planning model only. No lifecycle state grants runtime
activation, package execution, module loading, plugin instantiation,
capability issuance, registry publication, trust assignment, Semantic CLI
authority, AI Runtime authority, or agent authority by itself.

`PHASE20_CAPABILITY_LIFECYCLE.md` must later define the exact state machine,
allowed transitions, forbidden transitions, evidence requirements, and
fail-closed handling for lifecycle ambiguity.

## RFC Dependency Order

The initial RFC dependency order is:

```text
PHASE20_CAPABILITY_MODEL.md
  -> PHASE20_CAPABILITY_IDENTITY.md
  -> PHASE20_CAPABILITY_MANIFEST_SCHEMA.md
  -> PHASE20_CAPABILITY_LIFECYCLE.md
  -> PHASE20_REGISTRY_MODEL.md
  -> PHASE20_REGISTRY_GOVERNANCE.md
  -> PHASE20_TRUST_MODEL.md
  -> PHASE20_DISTRIBUTION_POLICY.md
  -> PHASE20_CAPABILITY_EVIDENCE_MODEL.md
  -> PHASE20_CAPABILITY_ACCEPTANCE_WORKFLOW.md
```

Later RFCs may narrow dependencies, but no later RFC may skip a prerequisite
when doing so would create authority ambiguity, registry ambiguity, evidence
ambiguity, or implementation ambiguity.

## Governance Levels

Phase-20 governance is layered:

| Level | Layer | Purpose | Authority result |
|---|---|---|---|
| 1 | Concept | Define what a capability is and is not | No execution |
| 2 | Identity | Define naming, digest binding, versioning, and immutability | No registry publication |
| 3 | Schema | Define inert manifest shape | No parser, installer, loader, or executor |
| 4 | Lifecycle | Define capability states and transitions | No token minting |
| 5 | Registry | Define registry record model and indexes | No publication |
| 6 | Trust | Define trust inputs and proof requirements | No trust assignment |
| 7 | Distribution | Define publication, revocation, quarantine, and rollback policy | No distribution authority |
| 8 | Evidence | Define exact-subject evidence requirements | No authority inheritance |
| 9 | Acceptance | Define review and merge workflow | No implementation without decision |
| 10 | Implementation | Later bounded source subject | No runtime activation by default |
| 11 | Runtime | Later activation subject, if ever authorized | Requires separate reviewed decision |

Moving between levels requires an explicit reviewed record. Lower levels do
not automatically grant higher-level authority.

## Constitutional Stability Goal

The purpose of Phase-20 is not rapid implementation.

The purpose is to establish a constitutional model that minimizes future
architectural drift, authority ambiguity, incompatible capability evolution,
registry ambiguity, trust ambiguity, and evidence ambiguity.

Implementation speed is secondary to architectural stability. Phase-20
should make later implementation easier by making the rules explicit before
source authority exists.

## Governance Scope

Phase-20 is responsible for defining the constitutional and architectural
model for the capability and registry ecosystem.

Phase-20 may define:

1. Capability concepts, identity, metadata, and lifecycle.
2. Capability manifest format and validation requirements.
3. Capability registry data model and governance constraints.
4. Distribution, publication, revocation, quarantine, and rollback policy.
5. Trust model inputs, issuer preconditions, and non-bypass constraints.
6. Evidence model for capability and registry admission.
7. Acceptance workflow for later bounded implementation slices.
8. Cross-document consistency between Phase-18 constitution records,
   Phase-19 runtime records, and Phase-20 capability/registry records.

Phase-20 must preserve fail-closed interpretation across all capability,
registry, trust, package, module, workspace, plugin, Semantic CLI, AI
Runtime, and agent terms.

## Out Of Scope

Phase-20 governance overview does not authorize:

1. Runtime activation.
2. General runtime authority.
3. Package installation, loading, execution, scheduling, or publication.
4. Module loading.
5. Workspace creation, workspace runtime, or real mounts.
6. Plugin host, plugin loading, or plugin instantiation.
7. Capability token minting or capability issuance.
8. Registry publication or marketplace behavior.
9. Trust assignment or trust issuer behavior.
10. Semantic CLI execution or verdict authority.
11. AI Runtime authority.
12. Agent behavior.
13. New syscalls.
14. Kernel ABI expansion.
15. Workflow-threshold, baseline, dependency, or Ring0 policy changes.
16. Observability-as-authority.

Any proposal that makes an inert Phase-20 record active, executable,
loadable, publishable, mountable, transferable, trust-bearing, or
authority-bearing requires a separate reviewed decision path and exact-SHA
evidence.

## Required RFC Set

The initial Phase-20 RFC set should be developed as explicit, reviewable
records before any Phase-20 implementation slice is accepted.

| RFC record | Purpose |
|---|---|
| `PHASE20_CAPABILITY_MODEL.md` | Defines what a capability is, what it is not, and how capability records remain fail-closed. |
| `PHASE20_CAPABILITY_IDENTITY.md` | Defines capability identity, digest binding, naming, versioning, immutability, and collision rules. |
| `PHASE20_CAPABILITY_MANIFEST_SCHEMA.md` | Defines the declarative manifest shape without granting parser, loader, installer, or execution authority. |
| `PHASE20_CAPABILITY_LIFECYCLE.md` | Defines draft, review, accepted, deprecated, revoked, quarantined, and retired states. |
| `PHASE20_REGISTRY_MODEL.md` | Defines registry data model, indexes, references, and integrity requirements. |
| `PHASE20_REGISTRY_GOVERNANCE.md` | Defines registry governance boundaries, admission gates, non-bypass constraints, and publication separation. |
| `PHASE20_TRUST_MODEL.md` | Defines trust inputs, issuer constraints, proof requirements, and fail-closed trust interpretation. |
| `PHASE20_DISTRIBUTION_POLICY.md` | Defines distribution, publication, revocation, quarantine, rollback, and mirror policy inputs. |
| `PHASE20_CAPABILITY_EVIDENCE_MODEL.md` | Defines evidence packages required before any capability or registry implementation can be accepted. |
| `PHASE20_CAPABILITY_ACCEPTANCE_WORKFLOW.md` | Defines the review, acceptance, merge, and post-merge exact-SHA workflow for Phase-20 slices. |

Missing, ambiguous, stale, or differently scoped RFC records fail closed.

## Work Packages

Phase-20 work is ordered as follows:

| Work package | Required output | Authority result |
|---|---|---|
| WP1 Capability Model | Capability concept and non-authority boundary | No issuance or execution |
| WP2 Capability Identity | Digest, naming, versioning, and immutability rules | No registry publication |
| WP3 Capability Manifest | Declarative manifest schema | No parser, loader, installer, or executor |
| WP4 Capability Lifecycle | State machine and fail-closed transitions | No token minting |
| WP5 Registry Model | Registry data model and integrity rules | No publication or marketplace |
| WP6 Registry Governance | Admission gates and non-bypass rules | No registry authority |
| WP7 Trust Model | Trust inputs and issuer constraints | No trust assignment |
| WP8 Distribution Policy | Publication, revocation, quarantine, and rollback policy | No package distribution authority |
| WP9 Evidence Model | Evidence package requirements | No evidence inheritance across subjects |
| WP10 Acceptance Workflow | Review and exact-SHA acceptance procedure | No implementation without later decision |
| WP11 First Bounded Implementation Candidate | Narrow implementation candidate after RFC completion | No merge without evidence/review |

The ordering may be narrowed by later governance records, but it must not be
broadened into implementation authority without a separate decision.

## Implementation Gating Rule

No Phase-20 implementation slice may begin, be accepted, or be merged unless
all of the following are true:

1. The relevant Phase-20 RFC records are reviewed and accepted.
2. The implementation subject is explicitly bounded.
3. The implementation decision states exactly what behavior is authorized.
4. The implementation decision states exactly what behavior remains denied.
5. An evidence package is generated for the exact implementation subject.
6. An acceptance review evaluates that evidence.
7. A merge decision authorizes only the reviewed bounded subject.
8. Post-merge exact-SHA evidence passes.

Implementation authority is never inherited from this overview, the
Phase-20 pointer transition, or any planning RFC.

## Evidence Chain

Phase-20 implementation work must use the following evidence chain:

```text
RFC set
  -> implementation decision candidate
  -> implementation decision
  -> bounded implementation subject
  -> evidence package
  -> acceptance review
  -> merge decision
  -> post-merge exact-SHA evidence
```

Evidence is not authority. Authority is produced only by the applicable
reviewed decision record for the bounded subject.

Historical PASS results may be cited as context only. They cannot be
inherited as authority for a new subject SHA.

## Positive Architecture Direction

Phase-20 must define the positive architecture of AykenOS capability and
registry governance, not merely list prohibitions.

The Phase-20 RFC set should define:

1. What a capability is.
2. What a capability is not.
3. How capability identity is formed and verified.
4. How capability metadata is represented.
5. How a capability manifest remains declarative and inert.
6. How registry records are indexed, referenced, and integrity-checked.
7. How trust inputs are recorded without becoming trust assignment.
8. How publication and revocation are reviewed before any active behavior.
9. How evidence is attached to capability and registry records.
10. How later implementation slices are kept narrow, reviewable, and
    exact-SHA-bound.

Positive architecture does not relax the fail-closed boundary. Any undefined
authority remains denied.

## Relationship To Later Phases

Phase-20 must not pull later phases forward.

1. Semantic CLI authority remains later work unless separately reviewed.
2. AI Runtime authority remains later work unless separately reviewed.
3. Agent behavior remains later work unless separately reviewed.
4. Package execution, module loading, plugin instantiation, capability
   issuance, registry publication, and trust assignment remain denied until
   separate Phase-20 or later-phase decisions authorize a bounded subject.

Later phases require their own reviewed decision packages and exact-SHA
evidence.

## Explicit Non-Authorization

This overview does not authorize:

1. Runtime activation.
2. General runtime authority.
3. Phase-20 implementation.
4. Source acceptance or merge authority.
5. Package installation, loading, execution, scheduling, or publication.
6. Module loading.
7. Workspace creation, workspace runtime, or real mounts.
8. Plugin host, plugin loading, or plugin instantiation.
9. Capability token minting or capability issuance.
10. Registry publication or marketplace behavior.
11. Trust assignment or trust issuer behavior.
12. Semantic CLI execution or verdict authority.
13. AI Runtime authority.
14. Agent behavior.
15. New syscalls.
16. Kernel ABI expansion.
17. Workflow-threshold, baseline, dependency, or Ring0 policy changes.
18. Observability-as-authority.

Unknown authority readings fail closed.

## Governance Conclusion

Phase-20 is active as the capability and registry ecosystem
planning/governance phase only.

The immediate Phase-20 objective is to produce and review the required RFC
set, beginning with the capability model and its non-authority boundary.

Phase-20 establishes the constitutional foundation upon which future
capability, registry, distribution, Semantic CLI, AI Runtime, and agent
ecosystems may later be developed through separately reviewed bounded
implementation slices.

Runtime activation, general runtime authority, Phase-20 implementation
authority, capability issuance, registry publication, package execution,
module loading, workspace runtime, plugin loading, trust assignment,
Semantic CLI authority, AI Runtime authority, agent authority, syscall
expansion, kernel ABI expansion, workflow-threshold changes, baseline
changes, dependency changes, and Ring0 authority remain pending and
unauthorized until separately reviewed and decided.

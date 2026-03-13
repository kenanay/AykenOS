# Phase-13 Architecture Map

**Version:** 1.0
**Status:** Draft (Phase-13 preparation)
**Date:** 2026-03-11
**Phase:** Phase-13 Distributed Verification Expansion
**Type:** Non-normative architecture map
**Related Spec:** `requirements.md`, `tasks.md`, `PHASE12_CLOSURE_ORDER.md`, `DISTRIBUTED_VERIFICATION_TOPOLOGY.md`, `VERIFICATION_OBSERVABILITY_MODEL.md`, `VERIFICATION_RELATIONSHIP_GRAPH.md`, `GLOBAL_VERIFICATION_GRAPH_MODEL.md`, `PARITY_LAYER_ARCHITECTURE.md`, `PROOFD_DIAGNOSTICS_SERVICE_SURFACE.md`, `AUTHORITY_TOPOLOGY_FORMAL_MODEL.md`, `VERIFICATION_CONTEXT_DISTRIBUTION_CONTRACT.md`, `VERIFIER_REGISTRY_LINEAGE_AND_DISTRIBUTION_MODEL.md`, `GENERIC_DETERMINISTIC_TRUTH_VERIFICATION_ARCHITECTURE.md`, `PHASE13_NEGATIVE_TEST_SPEC.md`, `PHASE13_KILL_SWITCH_GATES.md`, `PHASE13_COLLAPSE_SCENARIOS.md`, `VERIFICATION_DIVERSITY_LEDGER_SPEC.md`, `VERIFICATION_DIVERSITY_FLOOR_GATE.md`, `OBSERVABILITY_ROUTING_SEPARATION_GATE.md`, `AYKENOS_GATE_ARCHITECTURE.md`, `GATE_REGISTRY.md`

---

## 1. Purpose

This document maps the most likely Phase-13 architecture direction after Phase-12 closure.

It does not redefine Phase-12 acceptance criteria.

Its role is to preserve one rule:

`Phase-13 scales the existing truth surfaces`

not:

`Phase-13 replaces them`

The map exists to keep future work aligned around:

- replicated verification
- distributed replay boundary
- verifier federation
- verifier trust registry propagation
- service-backed diagnostics and observability

without collapsing verification into consensus.

---

## 2. Starting Point

Phase-11 delivered:

`portable proof`

Phase-12 delivers:

`trusted deterministic verification`

The Phase-13 bridge is:

`portable trusted verification across distributed nodes`

This means the Phase-13 architecture starts from already separated truth surfaces:

- subject surface
  - `bundle_id`
  - `trust_overlay_hash`
- context surface
  - `verification_context_id`
- authority surface
  - verifier trust registry lineage
  - `authority_chain_id`
- verdict surface
  - deterministic local verdict
- diagnostics surface
  - parity
  - convergence
  - drift attribution
  - determinism incidents

Phase-13 should scale these surfaces, not merge them.

---

## 3. Core Architectural Rule

The correct Phase-13 growth model is:

`verification -> distributed diagnostics -> distributed coordination boundary`

not:

`verification -> consensus -> hidden authority`

So the stable architectural boundary remains:

- `verification != authority`
- `authority != consensus`
- `parity = diagnostics`
- `proofd = service surface`

Phase-13 MUST preserve these distinctions.

---

## 4. Main Workstreams

### 4.1 Service-Backed Verification Expansion

`proofd` becomes the primary userspace service boundary for:

- verification execution
- signed receipt production
- diagnostics query
- run-scoped artifact discovery

But `proofd` still MUST NOT become:

- authority resolver of record
- consensus layer
- replay executor

The correct service sentence remains:

`proofd = verification and diagnostics service`

and:

`proofd != authority surface`

### 4.2 Verifier Federation

Phase-13 may introduce federation semantics between verifiers.

The purpose is:

- exchange verifier-trust artifacts
- compare authority lineages
- analyze distributed trust divergence

The purpose is not:

- elect a permanent federation truth
- create implicit trust transitivity

Verifier federation therefore grows from:

- verifier attestation
- verifier trust registry
- verifier registry lineage
- authority graph constraints
- authority resolution

### 4.3 Verification Context Propagation

Distributed verification reuse requires explicit propagation of:

- policy material
- registry material
- context-rules material
- declared `verification_context_id`

Phase-13 should therefore expand:

- content-addressed context packaging
- transport resolution rules
- context portability diagnostics

This stays distinct from:

- proof transport
- receipt transport
- verifier-trust transport

### 4.4 Trust Registry Propagation

Producer registry and verifier registry distribution become larger concerns in Phase-13.

The likely architecture direction is:

- signed registry snapshots
- explicit parent-hash and epoch lineage
- rollback and fork detection
- diagnostic propagation state

This is not yet:

- global trust synchronization
- consensus registry state

### 4.5 Replicated Verification Boundary

Phase-13 is the first phase where replicated verification can be explored without leaking into Phase-12 closure semantics.

The key boundary remains:

`verified proof != replay admission`

So replicated verification should begin as:

- diagnostics-rich verification reuse
- replay-boundary analysis
- distributed admission modeling

not:

- automatic replay execution
- kernel-side trust enforcement

### 4.6 Observability and Topology

Phase-13 should deepen derived observability artifacts:

- incident graph
- authority topology
- suppression reports
- convergence partitions
- historical authority islands
- insufficient evidence islands
- verification diversity ledger
- verification diversity ledger producer

These are observability structures.

They MUST remain:

- derived
- queryable
- non-authoritative

---

## 5. Likely Layered Stack

The expected Phase-13 stack is:

1. `proof-verifier`
   - deterministic local verification engine
2. `proofd`
   - verification execution and diagnostics query surface
3. distributed trust transport
   - context, receipt, attestation, registry, and run artifact exchange
4. federation diagnostics
   - parity, convergence, authority topology, incident graph, verification diversity ledger
5. replay boundary analysis
   - admission contracts and replicated verification boundary checks

This stack is intentionally not:

1. verifier
2. authority arbitration
3. consensus protocol
4. execution finality

---

## 6. Phase-13 Non-Goals

The following SHOULD remain outside initial Phase-13 scope unless separately ratified:

- distributed consensus
- global event ordering
- majority truth election
- cluster authority arbitration
- kernel-side trust execution
- automatic replay admission
- hidden policy substitution
- implicit verifier reputation systems

If a component starts doing those things, it has moved beyond the intended Phase-13 map.

---

## 7. Implementation Order

The most plausible implementation order is:

1. finish Phase-12 closure
2. stabilize `proofd` as closure-ready verification service
3. expand read-only diagnostics query surfaces
4. add verification diversity ledger as behavioral observability surface
5. add federated verifier-trust and registry propagation diagnostics
6. add replicated verification boundary artifacts
7. define controlled replay-admission interfaces

So Phase-13 starts with:

`service + transport + diagnostics scaling`

before:

`distributed execution semantics`

---

## 8. Architectural Risks

The most likely Phase-13 risks are:

### 8.1 Hidden Consensus Drift

Parity, graph, or `proofd` features could accidentally become truth-selection machinery.

This must be resisted.

### 8.2 Authority Inflation

Diagnostics artifacts such as dominant clusters or suppression outputs could be misread as authority decisions.

They are not.

### 8.3 Registry Distribution Complexity

Registry propagation can quietly become a control plane if lineage, rollback, and split-brain semantics are not kept explicit.

### 8.4 Replay Scope Creep

Replicated verification can easily slide into replay execution if the replay boundary is not held rigidly.

### 8.5 Service Semantic Drift

`proofd` must remain a service wrapper over canonical verifier and diagnostics artifacts, not a second semantic engine.

### 8.6 Hidden Reputation Drift

Graph and topology analytics could quietly become a verifier reputation system.

This must be resisted.

Historical divergence frequency, agreement ratios, dominant-cluster recurrence, or reliability-style scoring MUST NOT become implicit trust ranking.

### 8.7 Authority Topology Feedback Loop

Authority topology, convergence, or suppression observability could quietly bias verification routing or scheduling behavior.

This must be resisted.

Phase-13 diagnostics may explain topology and drift, but they MUST NOT become verifier ordering, preferred-node, or routing-priority input.

### 8.8 Verification Gravity Collapse

Verification activity could quietly concentrate around a small verifier subset even without explicit authority election.

This must be resisted.

Phase-13 should remain distributed in behavior, not only in nominal topology width.

### 8.9 Verifier Cartel Formation

Multiple verifiers could remain formally distinct while becoming operationally or linearly correlated enough to behave like one trust bloc.

This must be resisted.

Node multiplicity alone is not enough if verifier independence collapses.

### 8.10 Verification Basin Collapse

Verification reuse, replay review, or trust reuse could quietly fall into one practical authority basin through repeated convenience and reuse.

This must be resisted.

Phase-13 must not allow operational reuse to become authority absorption.

---

## 9. Governing Invariants

Phase-13 growth should preserve these invariants:

- canonical truth objects remain crate-owned and deterministic
- diagnostics remain derived artifacts
- service surfaces remain wrappers over canonical artifacts
- federation does not imply authority arbitration
- replicated verification does not imply replay admission
- verification history does not imply verifier reputation
- observability does not imply verification scheduling

The executable contract direction for these invariants is captured in:

- `PHASE13_NEGATIVE_TEST_SPEC.md`
- `VERIFICATION_INVARIANTS.md`
- `PHASE13_COLLAPSE_SCENARIOS.md`

The shortest operational reading remains:

- observability does not imply consensus
- observability does not imply scheduling

The shortest correct rule is:

`Phase-13 extends distributed verification observability and transport without redefining truth semantics`

---

## 10. Summary

Phase-13 should not be treated as a new theory phase.

It is the scaling phase for the architecture already established in Phase-12.

So the correct map is:

- stable verifier core
- stable `proofd` service
- explicit trust/context/authority transport
- federated diagnostics
- controlled replicated verification boundary

and not:

- hidden consensus
- authority arbitration
- replay execution by implication

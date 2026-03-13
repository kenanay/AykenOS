# Phase-13 Negative Test Specification

**Version:** 1.0
**Status:** Draft (Phase-13 preparation)
**Date:** 2026-03-13
**Phase:** Phase-12 / Phase-13 boundary
**Type:** Non-normative negative-test boundary specification
**Related Spec:** `PHASE13_ARCHITECTURE_MAP.md`, `VERIFICATION_INVARIANTS.md`, `PROOFD_DIAGNOSTICS_SERVICE_SURFACE.md`, `VERIFICATION_OBSERVABILITY_MODEL.md`, `VERIFICATION_RELATIONSHIP_GRAPH.md`, `GLOBAL_VERIFICATION_GRAPH_MODEL.md`, `AUTHORITY_TOPOLOGY_FORMAL_MODEL.md`, `N_NODE_CONVERGENCE_FORMAL_MODEL.md`, `VERIFIER_REPUTATION_PROHIBITION_GATE.md`

---

## 1. Purpose

This document defines the minimum negative-test boundary for Phase-13.

Its role is not to add new verification semantics.

Its role is to prevent semantic drift in:

- service APIs
- graph and topology artifacts
- aggregation semantics
- convergence reporting

The central enforcement rule is:

`every Phase-13 surface must be artifact-backed, read-only, and non-authoritative`

This document exists because Phase-13 is more likely to fail through contract drift than through missing theory.

---

## 2. Governing Risk Model

The main Phase-13 failure modes are:

1. observability drifting into authority
2. diagnostics drifting into control plane
3. graph drifting into consensus
4. parity drifting into truth election
5. graph analytics drifting into verifier reputation scoring
6. observability artifacts drifting into verification scheduling signals

So the shortest stable rule set is:

- `observability != authority`
- `diagnostics != control plane`
- `graph != consensus`
- `parity != truth`
- `verification history != verifier reputation`
- `observability != scheduling`

Negative tests must fail closed when any Phase-13 surface violates those rules.

---

## 3. Forbidden Service Semantics

The following semantics are forbidden for diagnostics, graph, topology, convergence, and authority-observability surfaces:

- elect truth
- resolve truth
- select winner
- commit cluster state
- promote dominant cluster
- accept authority
- override policy
- force accept
- retry verification from diagnostics namespace
- trigger replay admission
- mutate registry or context state
- compute verifier reputation
- expose historical correctness scores
- expose weighted verifier authority
- expose reliability-ranked trust

These semantics are forbidden even if they are exposed through harmless-looking query parameters or response metadata.

Examples of forbidden meanings:

- `majority_accept = canonical truth`
- `dominant_authority_chain_id = selected authority`
- `parity match = admission approved`
- `severity = policy action`
- `historical agreement ratio = verifier trust`
- `divergence rate = verifier correctness score`

If a service exposes these meanings, it has crossed the AykenOS Phase-13 boundary.

---

## 4. Namespace and Method Boundary

The diagnostics family remains read-only.

Current and future namespaces such as:

- `/diagnostics/*`
- `/graph/*`
- `/topology/*`
- `/convergence/*`
- `/authority/*`

MUST remain read-only if they expose observability artifacts.

So the default boundary is:

- `GET` allowed for artifact-backed queries
- `POST`, `PUT`, `PATCH`, `DELETE` forbidden for observability surfaces

This rule does not forbid execution endpoints outside observability namespaces.

It does forbid observability namespaces from silently becoming:

- replay controllers
- retry controllers
- policy mutation surfaces
- authority arbitration surfaces

---

## 5. Descriptive Field Contract

Some Phase-13 fields are especially dangerous because they can be misread as normative outputs.

Examples include:

- `dominant_authority_chain_id`
- `dominant_authority_cluster_key`
- `authority_cluster_count`
- `severity_counts`
- `largest_partition`
- `convergence_ratio`
- `historical_only_island_count`
- `suppressed_drift_count`

These fields are descriptive diagnostics metadata only.

They MUST NOT be used as inputs to:

- authority election
- truth selection
- replay admission
- policy mutation
- verifier trust promotion
- cluster-state commit

Required interpretation sentence:

`descriptive only; MUST NOT be used as authority, consensus, replay, or truth-election input`

If a schema or API contract introduces one of these fields without that semantic boundary, the contract is incomplete.

---

## 6. Verifier Reputation Prohibition

Phase-13 graph and observability surfaces MUST NOT compute or expose historical verifier reputation.

Forbidden metric classes include:

- verifier correctness score
- verifier reliability score
- node trust score
- historical correctness index
- weighted verifier authority
- authority alignment score
- dominant verifier frequency
- convergence leadership score

These metrics are forbidden because they convert:

`verification history`

into:

`implicit authority`

The allowed question is:

`what happened across verification results?`

The forbidden question is:

`which verifier should the system trust more next time?`

If graph analytics starts ranking verifiers by historical agreement or divergence behavior, the system has already drifted into hidden reputation semantics.

---

## 7. Graph and Topology Boundary

The graph rule is:

`graph explains verification state`

and:

`graph does not determine truth`

So graph and topology objects may describe:

- which nodes emitted which truth surfaces
- where parity diverged
- how authority lineages cluster
- how convergence partitions formed

They MUST NOT decide:

- which verdict becomes canonical
- which authority wins
- which cluster becomes trusted
- which replay becomes allowed

Allowed graph question:

`who verified what, and how do the results relate?`

Forbidden graph question:

`which result should the system accept?`

The same separation also applies to verification routing and scheduling.

Graph, topology, convergence, and authority-observability artifacts MAY explain verification shape.

They MUST NOT bias:

- verifier ordering
- preferred-node selection
- routing priority
- suppression-based scheduling
- dominant-cluster-first execution

---

## 8. Negative Test Matrix

### 8.1 Service Boundary Tests

`P13-NEG-01`

- Case: `POST /diagnostics/graph`
- Expected: fail closed
- Rule: observability namespace must not mutate or trigger execution

`P13-NEG-02`

- Case: `POST /diagnostics/authority-topology`
- Expected: fail closed
- Rule: authority observability must not become authority control

`P13-NEG-03`

- Case: `GET /diagnostics/graph?select_winner=true`
- Expected: fail closed
- Rule: query parameters must not smuggle truth election semantics

`P13-NEG-04`

- Case: `GET /diagnostics/convergence?commit=true`
- Expected: fail closed
- Rule: convergence query must not imply cluster-state commit

### 8.2 Majority and Dominance Tests

`P13-NEG-05`

- Case: `2/3` nodes emit the same verdict and one node diverges
- Expected: output remains divergence/convergence diagnostics only
- Forbidden outcome: majority verdict promoted to canonical truth

`P13-NEG-06`

- Case: `dominant_authority_chain_id` exists in topology artifact
- Expected: field remains descriptive only
- Forbidden outcome: dominant cluster reused as authority resolution result

`P13-NEG-07`

- Case: parity artifact shows strongest cluster or dominant surface
- Expected: cluster size may be reported
- Forbidden outcome: cluster size reused as trust or replay decision

### 8.3 Parity and Convergence Tests

`P13-NEG-08`

- Case: parity reports a full match partition
- Expected: observability reports convergence
- Forbidden outcome: convergence automatically implies admission, execution, or truth finality

`P13-NEG-09`

- Case: same `D_i`, different `K_i`
- Expected: determinism incident
- Forbidden outcome: service resolves one verdict as winner

`P13-NEG-10`

- Case: insufficient-evidence island appears in convergence artifact
- Expected: insufficient evidence remains explicit
- Forbidden outcome: island silently collapsed into a current cluster

### 8.4 Severity and Attribution Tests

`P13-NEG-11`

- Case: `severity = pure_determinism_failure`
- Expected: severity stays diagnostic metadata
- Forbidden outcome: severity changes policy or authority semantics

`P13-NEG-12`

- Case: authority suppression report exists
- Expected: suppression remains explanatory only
- Forbidden outcome: suppression rewrites canonical authority resolution

### 8.5 Schema and Payload Tests

`P13-NEG-13`

- Case: graph/topology payload includes fields such as:
  - `selected_truth`
  - `winning_verdict`
  - `committed_cluster`
  - `accepted_authority`
- Expected: fail closed
- Rule: payloads must not encode hidden consensus or arbitration outputs

`P13-NEG-14`

- Case: diagnostics response includes mutation hints such as:
  - `retry`
  - `override`
  - `promote`
  - `commit`
  - `recommended_action`
  - `mitigation`
  - `routing_hint`
  - `node_priority`
  - `verification_weight`
  - `execution_override`
- Expected: fail closed
- Rule: observability payloads must not embed control-plane affordances

### 8.6 Reputation and Scoring Tests

`P13-NEG-15`

- Case: graph or topology payload includes fields such as:
  - `verifier_score`
  - `trust_score`
  - `reliability_index`
  - `weighted_authority`
  - `correctness_rate`
- Expected: fail closed
- Rule: observability payloads must not expose verifier reputation or scoring outputs

`P13-NEG-16`

- Case: analytics layer derives:
  - historical agreement ratio by node
  - divergence leaderboard
  - node correctness ranking
- Expected: fail closed
- Rule: verification history must not be transformed into implicit authority ranking

### 8.7 Diagnostics Consumer Tests

`P13-CONS-01`

- Case: a non-observability runtime file reads fields such as:
  - `dominant_authority_chain_id`
  - `global_status`
  - `largest_outcome_cluster_size`
- Expected: fail closed
- Rule: descriptive diagnostics fields must not be consumed by execution-bearing runtime code

`P13-CONS-02`

- Case: a runtime file imports diagnostics artifacts such as:
  - `parity_convergence_report.json`
  - `parity_authority_drift_topology.json`
  - `parity_drift_attribution_report.json`
- Expected: fail closed
- Rule: diagnostics artifacts must not become policy, routing, or replay inputs

`P13-CONS-03`

- Case: `global_status` is reused as:
  - replay admission input
  - routing decision
  - policy or priority signal
- Expected: fail closed
- Rule: diagnostic convergence status must remain descriptive only

`P13-CONS-04`

- Case: historical-only or insufficient-evidence island summaries are reused for:
  - suppression
  - trust promotion
  - execution priority
- Expected: fail closed
- Rule: island diagnostics must remain explanatory only

### 8.8 Diagnostics Correlation Tests

`P13-CORR-01`

- Case: an approved diagnostics producer or passthrough function reads:
  - `global_status`
  - `dominant_authority_chain_id`
  - `parity_convergence_report.json`
- and passes that source directly into:
  - policy evaluation
  - verification execution
- Expected: fail closed
- Rule: descriptive diagnostics sources must not correlate directly with decision sinks

`P13-CORR-02`

- Case: a descriptive diagnostics source is renamed through local aliases and later reaches:
  - replay admission
  - routing hints
  - override signals
- Expected: fail closed
- Rule: aliasing must not hide diagnostics-to-decision flow

`P13-CORR-03`

- Case: an artifact name such as `parity_convergence_report.json` is assigned to a local variable and later reused in:
  - execution override
  - priority
  - promotion
- Expected: fail closed
- Rule: diagnostics artifact aliases must not become decision inputs

### 8.9 Routing and Scheduling Separation Tests

`P13-FEED-01`

- Case: authority-topology or convergence fields such as:
  - `dominant_authority_chain_id`
  - `largest_outcome_cluster_size`
  - `outcome_convergence_ratio`
- are reused as:
  - verifier ordering input
  - preferred-node selection
  - first-hop routing
- Expected: fail closed
- Rule: descriptive observability fields must not become verification scheduling signals

`P13-FEED-02`

- Case: a scheduling or routing layer prefers a cluster because diagnostics show:
  - dominant cluster recurrence
  - strongest current partition
  - lowest recent divergence
- Expected: fail closed
- Rule: topology or convergence observability must not bias verification diversity or routing order

`P13-FEED-03`

- Case: suppression or island diagnostics are reused for:
  - node quarantine
  - verifier exclusion
  - verification retry ordering
- Expected: fail closed
- Rule: explanatory observability artifacts must not become runtime scheduling or orchestration control

`P13-FEED-04`

- Case: scheduling logic optimizes for:
  - agreement likelihood
  - dominant-cluster recurrence
  - lowest divergence heuristics
- Expected: fail closed
- Rule: verification scheduling must preserve diversity rather than optimize for likely agreement

`P13-FEED-05`

- Case: a routing or verifier-selection file imports:
  - `authority_drift_topology`
  - `drift_attribution`
  - `determinism_incident`
  - `incident_graph`
- Expected: fail closed
- Rule: routing or scheduling code must not import observability modules directly

---

## 9. Minimum Contract Freezes

Phase-13 should freeze in this order:

1. `DeterminismIncidentSeverity` and incident taxonomy
2. `proofd` read-only query contract
3. graph and topology artifact field contract
4. `N`-node convergence negative matrix

This order is correct because it grows the safest derived surfaces first and leaves the most election-adjacent surface last.

---

## 10. Negative-Test Growth Direction

The negative matrix should not scale through manual case proliferation alone.

The preferred evolution path is:

- define forbidden field classes
- define forbidden action classes
- define forbidden query or method classes
- generate constrained case combinations from those classes

So the intended long-term model is:

`case matrix -> constrained generator`

Generation is correct only if the generated cases still reduce cleanly to:

- invariant summaries
- risk classes
- authoritative failure meanings

If generation obscures architectural meaning, it is the wrong generator.

---

## 11. Suggested Executable Gates

The following gate shapes are recommended:

- `ci-gate-proofd-observability-boundary`
- `ci-gate-graph-non-authoritative-contract`
- `ci-gate-convergence-non-election-boundary`
- `ci-gate-diagnostics-consumer-non-authoritative-contract`
- `ci-gate-diagnostics-callsite-correlation`
- `ci-gate-verifier-reputation-prohibition`
- `ci-gate-verification-determinism-contract`
- `ci-gate-observability-routing-separation`

Gate-specific reference surfaces:

- `PROOFD_OBSERVABILITY_BOUNDARY_GATE.md`
- `GRAPH_NON_AUTHORITATIVE_CONTRACT_GATE.md`
- `CONVERGENCE_NON_ELECTION_BOUNDARY_GATE.md`
- `DIAGNOSTICS_CONSUMER_NON_AUTHORITATIVE_CONTRACT_GATE.md`
- `DIAGNOSTICS_CALLSITE_CORRELATION_GATE.md`
- `OBSERVABILITY_ROUTING_SEPARATION_GATE.md`
- `VERIFIER_REPUTATION_PROHIBITION_GATE.md`
- `VERIFICATION_DETERMINISM_CONTRACT_GATE.md`

Each gate should fail closed.

Each gate should assert:

- artifact-backed behavior
- read-only behavior
- non-authoritative behavior
- no historical verifier scoring
- no observability-driven scheduling
- no routing-side observability imports

---

## 12. Summary

The shortest enforceable Phase-13 rule is:

`artifact-backed + read-only + non-authoritative`

If a Phase-13 surface becomes:

- mutating
- authority-bearing
- consensus-bearing
- truth-selecting
- reputation-bearing
- scheduling-bearing

then it is no longer an AykenOS observability surface.

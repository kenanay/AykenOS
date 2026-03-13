# Verification Invariants

**Version:** 1.0
**Status:** Informational architecture invariants
**Date:** 2026-03-13
**Phase:** Phase-12 / Phase-13 boundary
**Type:** Non-normative invariants note
**Related Spec:** `VERIFICATION_MODEL.md`, `AYKENOS_ARCHITECTURE_ONE_PAGE.md`, `AYKENOS_GLOBAL_ARCHITECTURE_DIAGRAM.md`, `AYKENOS_SYSTEM_POSITIONING_TABLE.md`, `PHASE13_ARCHITECTURE_MAP.md`, `PROOFD_DIAGNOSTICS_SERVICE_SURFACE.md`

---

## 1. Purpose

This document records the core invariants that keep AykenOS within its intended verification architecture.

Its role is to prevent architectural drift as Phase-13 grows.

The invariants here are not implementation details.

They are the main rules that preserve category identity.

---

## 2. Core Invariants

### 2.1 Deterministic Verification Invariant

`same subject + same context + same authority -> same verdict`

Verification semantics must remain deterministic for the same input surface.

### 2.2 Artifact Truth Invariant

`artifacts = canonical interface`

Receipts, manifests, verification reports, and derived evidence remain the durable truth surface.

Operational reading:

- verification acceptance must remain artifact-bound
- runtime cache, memory state, or network-majority state must not substitute for canonical evidence artifacts

### 2.3 Service Wrapper Invariant

`services wrap canonical artifacts`

Service APIs may execute verification and expose artifacts, but they do not replace the artifact-bound truth surface.

### 2.4 Authority Separation Invariant

`verification != authority`

Computing a verification result does not itself decide who may authoritatively reuse that result.

Operational reading:

- valid receipt != trusted verifier
- trusted proof != trusted verifier

### 2.5 Consensus Separation Invariant

`authority != consensus`

Authority semantics and distributed agreement remain distinct concerns.

### 2.6 Diagnostics Non-Authority Invariant

`diagnostics != authority`

Parity, convergence, topology, and incident surfaces remain observability outputs, not authority decisions.

### 2.7 Parity Non-Truth-Election Invariant

`parity != truth`

Parity explains cross-node result relationships; it does not elect one result as system truth.

### 2.8 Replay Boundary Invariant

`accepted proof != replay admission`

Successful verification does not automatically authorize replicated replay or execution reuse.

### 2.9 Topology Non-Consensus Invariant

`topology != consensus`

Distributed verifier topology may explain relationships between nodes, but it must not silently become a cluster-control or consensus surface.

### 2.10 Reputation Non-Authority Invariant

`verification history != verifier reputation`

Historical agreement, divergence, convergence frequency, or cluster membership frequency must not become implicit authority or trust-ranking inputs.

### 2.11 Graph Non-Truth-Inference Invariant

`graph != truth inference`

Graph, topology, and convergence analytics may describe verification structure, but they must not estimate, rank, recommend, or select truth.

### 2.12 Observability Non-Control Invariant

`observability != control`

Diagnostics outputs may explain drift, suppression, and incidents, but they must not emit actionable control signals that alter verification execution paths.

### 2.13 Verification Context Purity Invariant

`verification != environment dependent`

Verification semantics must not depend on time, randomness, ambient environment state, or network-visible context.

### 2.14 Diagnostics Consumer Non-Authority Invariant

`descriptive diagnostics != execution input`

Descriptive diagnostics artifacts may be produced and served, but they must not be consumed as policy, authority, replay, routing, suppression, priority, or execution input.

### 2.15 Diagnostics Correlation Non-Flow Invariant

`descriptive diagnostics != decision flow`

Even inside approved diagnostics producers or passthrough surfaces, descriptive diagnostics fields and artifact identities must not flow into policy, replay, routing, priority, override, or execution decision call sites.

### 2.16 Observability Scheduling Separation Invariant

`observability != scheduling`

Authority topology, convergence partitions, island summaries, suppression reports, and other descriptive observability artifacts must not influence verifier ordering, preferred-node selection, routing priority, or verification scheduling behavior.

Operational reading:

- verification routing must be observability blind
- routing code must not import observability modules directly
- verification scheduling may preserve diversity
- verification scheduling must not optimize for agreement likelihood

### 2.17 Nominal Diversity Non-Independence Invariant

`diversity != independence`

High verifier counts, acceptable entropy, or acceptable dominance ratios do not by themselves prove verifier independence.

Operational reading:

- nominal verifier multiplicity must not be mistaken for independent verifier behavior
- same-lineage or same-authority-chain correlation must remain observable
- execution-cluster concentration must not hide inside nominal diversity
- diversity floor is necessary but not sufficient for cartel resistance

### 2.18 Reuse Non-Basin-Collapse Invariant

`verification reuse != authority basin collapse`

Repeated verification reuse, replay review, or trust reuse must not become a practical sinkhole that absorbs future flow into one authority basin.

Operational reading:

- reuse convenience must not silently become authority absorption
- replay review paths must remain practically plural, not only theoretically available
- nominal topology width does not prove healthy authority-basin distribution
- diversity and independence gates are necessary but not sufficient for temporal basin health

---

## 3. Drift Signals

The following changes indicate architectural drift:

- a service API becoming the primary truth surface
- diagnostics outputs being consumed as authority decisions
- parity or topology being used to elect system truth
- graph or convergence analytics being used to infer or recommend truth
- replay admission being implied by verification success
- federation semantics drifting into hidden consensus
- graph analytics being converted into verifier scoring or reliability ranking
- diagnostics outputs being consumed as execution-routing or mitigation signals
- convergence partitions, cluster ratios, or island summaries being consumed as policy or election input
- descriptive diagnostics artifacts being imported into execution-bearing runtime consumers
- descriptive diagnostics aliases being forwarded into policy, replay, routing, or override call sites
- authority topology, convergence, or suppression observability influencing verification scheduling or routing order
- verification results drifting with ambient environment state
- nominal diversity metrics being used as proof that verifier independence still exists
- many verifier identities collapsing into one lineage, authority chain, or execution cluster while diversity floor still appears healthy
- practical verification or replay-boundary flow collapsing into one authority basin while topology still appears wide

If those changes occur, AykenOS has moved out of its intended category.

---

## 4. Summary

The shortest stable rule set is:

- `verification != authority`
- `authority != consensus`
- `parity = diagnostics`
- `artifacts = canonical interface`
- `services wrap canonical artifacts`
- `verification history != verifier reputation`
- `graph != truth inference`
- `observability != control`
- `convergence != election`
- `descriptive diagnostics != execution input`
- `descriptive diagnostics != decision flow`
- `observability != scheduling`
- `verification != environment dependent`
- `diversity != independence`
- `verification reuse != authority basin collapse`

These invariants are the main defense against Phase-13 scope drift.

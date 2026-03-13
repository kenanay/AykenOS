# Phase-13 Collapse Scenarios

**Version:** 1.0  
**Status:** Draft (Phase-13 threat horizon)  
**Date:** 2026-03-14  
**Phase:** Phase-13 distributed verification observability  
**Type:** Non-normative architectural threat model

---

## 1. Purpose

This document describes the main Phase-13 collapse scenarios that may still emerge even when:

- individual gates pass
- kill-switch reduction passes
- architectural coverage is complete

Its purpose is to identify the next threat horizon:

`system dynamics drift`

not:

`single-validator contract drift`

These scenarios matter because distributed verification systems can remain locally valid while becoming globally biased.

---

## 2. Reading Rule

Each collapse scenario is described using the same frame:

1. short definition
2. silent formation path
3. why current gates may still pass
4. early signals
5. missing invariant or stale invariant
6. required future gate or harness

This means the document is not a replacement for current gates.

It is the forward risk surface for the next layer of architectural defense.

---

## 3. Collapse Scenario 1: Verification Gravity Collapse

### 3.1 Short Definition

Verification activity slowly concentrates around a small subset of verifiers or verifier lineages, even though no explicit authority election occurs.

The system remains formally distributed.

Its behavior does not.

### 3.2 Silent Formation Path

The most common path is:

`verification reuse -> familiar verifier reuse -> lower-friction reuse -> repeated selection -> concentration`

This usually begins as operational convenience:

- the same verifier already has warm state
- the same verifier is easiest to reach
- the same verifier already holds relevant artifacts
- the same verifier is used first because it is already present in the flow

No explicit authority rule is added.

But routing, reuse, and convenience quietly create a center of gravity.

### 3.3 Why Current Gates May Still Pass

Current Phase-13 gates mostly protect:

- explicit authority drift
- explicit truth election
- observability-to-routing leakage
- diagnostics-to-control leakage

Verification Gravity Collapse can still pass those gates because:

- no forbidden field is consumed
- no majority or consensus semantic is emitted
- no reputational score is published
- no routing code may directly import observability modules

The system drifts through repeated operational preference, not through a forbidden schema field.

### 3.4 Early Signals

Typical early signals are:

- verifier diversity steadily decreases
- the same authority lineage repeatedly appears in successful verification paths
- a small verifier subset handles a disproportionate share of requests
- fallback paths become rare and eventually untested
- artifact portability remains valid, but operational reuse keeps collapsing toward the same verifier basin

### 3.5 Missing or Stale Invariant

The missing invariant is:

`verification scheduling must preserve diversity`

The stale invariant risk is:

`observability != scheduling`

This remains necessary, but it is not sufficient once concentration happens without direct observability input.

### 3.6 Required Gate or Harness

The correct next defense is a diversity-preservation harness.

Current shape:

- gate: `ci-gate-verification-diversity-floor`
- core check: no verification-facing flow may collapse below a declared verifier-diversity floor over a bounded evidence horizon
- primary artifact: `VERIFICATION_DIVERSITY_LEDGER_SPEC.md`
- gate contract: `VERIFICATION_DIVERSITY_FLOOR_GATE.md`

This should be a harness, not a token scan.

It must measure behavior across runs.

---

## 4. Collapse Scenario 2: Verifier Cartel Formation

### 4.1 Short Definition

Multiple verifiers remain formally distinct, but in practice move as a correlated bloc.

The network still appears distributed.

Its trust behavior is no longer independent.

### 4.2 Silent Formation Path

The common path is:

`shared lineage -> shared policy -> shared artifact exchange -> shared routing priority -> correlated verdict behavior`

This can emerge through:

- common operator practice
- shared registry lineage
- shared deployment cadence
- shared configuration surfaces
- repeated trust reuse within one verifier family

The system then behaves as if many verifiers are independent, while they are actually only one social or operational cluster.

### 4.3 Why Current Gates May Still Pass

Current gates may still pass because:

- each verifier can still emit valid receipts
- cross-node parity may remain internally consistent
- no explicit reputation score is emitted
- no explicit truth election occurs
- no direct authority arbitration field appears

The failure is not local invalidity.

It is loss of independence.

This is also where entropy illusion appears:

- unique verifier counts may still look healthy
- entropy may remain above floor
- dominance may remain below the configured maximum

But the same lineage, authority chain, or execution cluster may already be moving as one bloc.

### 4.4 Early Signals

Typical early signals are:

- the same verifier lineage repeatedly appears across nominally separate nodes
- authority-chain diversity drops even while node count stays high
- distinct nodes show unusually correlated policy and verdict behavior
- failure partitions shrink, but only because many nodes have become operationally identical
- registry propagation appears healthy, but practical authority mobility declines

### 4.5 Missing or Stale Invariant

The missing invariant is:

`distributed verification requires independent verifier diversity, not only node multiplicity`

The stale invariant risk is:

`valid receipt != trusted verifier`

That invariant still protects explicit authority confusion, but not cartel-style correlation across many nominally valid verifiers.

### 4.6 Required Future Gate or Harness

The correct next defense is a verifier-independence correlation harness.

Current shape:

- gate: `ci-gate-verifier-cartel-correlation`
- core check: independent-node evidence must remain distinguishable from same-lineage or same-cluster dominance over time
- primary artifact: `VERIFICATION_DIVERSITY_LEDGER_SPEC.md`
- gate contract: `VERIFIER_CARTEL_CORRELATION_GATE.md`

This should inspect:

- lineage distribution
- repeated authority-chain concentration
- correlated verdict patterns
- operator or registry clustering evidence where available

The shortest reading is:

`diversity floor may pass while cartel correlation still rises`

---

## 5. Collapse Scenario 3: Verification Basin Collapse

This scenario was previously described as:

`Authority Drift Sinkhole`

The newer phrase is preferred because it names the system-dynamics failure more directly.

### 5.1 Short Definition

Verification traffic, replay review, or trust reuse gradually falls into one authority basin because the system keeps choosing the path of least resistance.

This is not explicit election.

It is absorption.

### 5.2 Silent Formation Path

The common path is:

`mild routing preference -> more successful reuse -> lower operational friction -> more reuse -> one basin absorbs future traffic`

This often starts from benign-seeming heuristics:

- verifier already has artifacts
- verifier already has recent context
- verifier already handled nearby requests
- verifier appears easiest to reuse after previous success

Eventually the system develops an authority sinkhole:

all traffic is not forced to one verifier,
but behavior keeps falling into the same authority basin.

### 5.3 Why Current Gates May Still Pass

Current gates may still pass because:

- no explicit scheduling signal comes from observability
- routing logic may not directly consume forbidden diagnostics fields
- parity and topology artifacts remain descriptive
- no formal authority selection field exists
- diversity floor may still remain above threshold
- cartel correlation may still remain below suspicion threshold

The sinkhole forms through repeated operational absorption rather than through a forbidden explicit decision surface.

### 5.4 Early Signals

Typical early signals are:

- the same authority chain repeatedly becomes the terminal verification basin
- fallback or alternate authority paths decay from active to merely theoretical
- replay review or trust reuse increasingly lands on one practical authority island
- the system retains nominal topology width, but effective authority width collapses

### 5.5 Missing or Stale Invariant

The missing invariant is:

`verification traffic must not be absorbable into a single authority basin through operational reuse alone`

The stale invariant risk is:

`observability != control`

That invariant prevents one important causal path, but not gradual absorption produced by repeated convenience or reuse decisions.

The newer operational reading is:

`verification reuse != authority basin collapse`

### 5.6 Required Future Gate or Harness

The correct next defense is an authority-absorption harness.

Suggested shape:

- reserved gate: `ci-gate-authority-sinkhole-absorption`
- core check: repeated verification and replay-boundary flows must not converge toward one authority basin beyond declared tolerance
- primary artifact: `VERIFICATION_DIVERSITY_LEDGER_SPEC.md`
- gate contract: `AUTHORITY_SINKHOLE_ABSORPTION_GATE.md`

This should be modeled as a multi-run system-dynamics harness, not as a schema validator.

The shortest reading is:

`distribution may pass, independence may pass, but basin health may still collapse`

---

## 6. Shared Pattern

All three collapse scenarios share the same dangerous property:

`all local checks may pass while the global system drifts`

So the shortest reading is:

- current gates protect explicit semantic drift
- future harnesses must protect behavioral concentration drift

This is the difference between:

`contract correctness`

and:

`system-shape correctness`

---

## 7. Short Rule

The shortest Phase-13 threat-horizon rule is:

`distributed verification can fail through concentration long before it fails through explicit consensus`

So the next generation of defenses should measure:

- diversity
- independence
- basin absorption

not only local semantic correctness.

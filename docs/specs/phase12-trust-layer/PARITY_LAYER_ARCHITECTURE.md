# Parity Layer Architecture

**Version:** 1.0
**Status:** Draft (Phase-13 preparation)
**Date:** 2026-03-09
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Type:** Non-normative architecture boundary note
**Related Spec:** `requirements.md`, `tasks.md`, `PARITY_LAYER_FORMAL_MODEL.md`, `N_NODE_CONVERGENCE_FORMAL_MODEL.md`, `CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`, `PROOF_BUNDLE_ATTACK_SURFACE_SECURITY_MODEL.md`, `GENERIC_DETERMINISTIC_TRUTH_VERIFICATION_ARCHITECTURE.md`

---

## 1. Purpose

This document defines the architectural role and boundary invariants of the AykenOS parity layer.

The parity layer provides:

`distributed verification diagnostics`

It does not provide:

`consensus`

This note exists to prevent semantic drift as Phase-13 expands observability, `proofd` query surfaces, and graph-style diagnostics.

---

## 2. Core Definition

The parity layer compares node-level verification results and exposes divergence.

Parity answers:

- did nodes reach the same verification outcome
- if not, where did they diverge
- if not, why did they diverge

Parity does not answer:

- which node is correct
- which outcome becomes final
- which state should be committed

The correct architectural rule is:

`Parity Layer = Distributed Verification Diagnostics`

and:

`Parity Layer != consensus`

---

## 3. Architectural Position

The parity layer sits after verification and before any future service/query surface.

Architectural pipeline:

`portable proof -> verifier -> verdict -> receipt -> cross-node parity -> diagnostics / observability`

Current node-derived diagnostics pipeline:

`verification -> NodeParityOutcome -> drift_attribution -> DeterminismIncident -> convergence diagnostics`

The parity layer operates on derived verification artifacts.

It does not participate in primary trust evaluation.

---

## 4. Explicit Non-Goals

The parity layer MUST NOT:

- commit state
- produce event ordering
- select canonical truth
- elect majority outcome
- resolve cluster authority
- enforce consensus
- control replay admission

These are intentionally outside parity scope.

If any later component requires those behaviors, it is no longer parity.

---

## 5. Boundary Invariants

### 5.1 Truth Selection Invariant

Parity MUST surface disagreement without selecting truth.

### 5.2 State Mutation Invariant

Parity artifacts are derived diagnostics only.
Parity MUST NOT mutate verifier, receipt, or runtime state.

### 5.3 Ordering Invariant

Parity MUST NOT generate event ordering.

### 5.4 Majority Invariant

Cluster size, dominant surface, or majority outcome MAY be reported for diagnostics.
They MUST NOT imply authority or finality.

### 5.5 Derived Artifact Invariant

Parity artifacts MUST be derivable from canonical verification objects:

- `NodeParityOutcome`
- `DeterminismIncident`
- drift attribution
- verification context
- verdict subject

Parity MUST NOT introduce new canonical truth objects.

### 5.6 Canonical Object Invariant

Parity MUST NOT redefine canonical objects.

Parity MAY derive, aggregate, and visualize existing canonical verification objects.

Parity MUST NOT introduce alternative truth-bearing object definitions for:

- `NodeParityOutcome`
- `DeterminismIncident`
- verification context
- verdict subject
- drift attribution

### 5.7 Derived Severity Invariant

When Phase-13 introduces `DeterminismIncidentSeverity`, severity MUST be deterministically derived from existing diagnostics signals.

Severity MUST NOT be manually assigned.

Severity remains diagnostics metadata.

It MUST NOT become policy, authority, or consensus input.

---

## 6. Diagnostic Model

The parity layer explains distributed verification divergence.

Current diagnostic classes include:

- `PARITY_MATCH`
- `PARITY_SUBJECT_MISMATCH`
- `PARITY_CONTEXT_MISMATCH`
- `PARITY_VERIFIER_MISMATCH`
- `PARITY_HISTORICAL_ONLY`
- `PARITY_INSUFFICIENT_EVIDENCE`
- `PARITY_VERDICT_MISMATCH`

Current artifact surfaces include:

- `failure_matrix.json`
- `parity_report.json`
- `parity_consistency_report.json`
- `parity_determinism_report.json`
- `parity_determinism_incidents.json`
- `parity_drift_attribution_report.json`
- `parity_convergence_report.json`

These artifacts explain disagreement.
They do not resolve it.

---

## 7. Determinism Incidents

Parity elevates same-surface verdict divergence into explicit incident artifacts.

Formal condition:

`same D_i + different K_i -> DeterminismIncident`

These incidents are diagnostics events.
They are not consensus triggers.

Stable incident identifiers are required so the same semantic incident can be correlated across runs.

---

## 8. `proofd` Service Boundary

`proofd` is a verification service surface.

`proofd` MAY:

- execute verification
- apply trust policy
- emit receipts
- expose diagnostics
- provide read-only query APIs

`proofd` MUST NOT:

- commit cluster state
- elect cluster truth
- resolve majority outcome
- act as distributed authority
- become a policy-bearing distributed control plane

Formally:

`proofd = verification service`

and:

`proofd != authority surface`

---

## 9. `proofd` Query Surface

Phase-13 may introduce read-only diagnostic APIs such as:

- `GET /diagnostics/incidents`
- `GET /diagnostics/incidents/{incident_id}`
- `GET /diagnostics/incidents?severity=...`
- `GET /diagnostics/surfaces`

These APIs MUST expose existing diagnostics artifacts or canonical derived views.

They MUST NOT introduce new trust semantics.

---

## 10. Observability Graph

Phase-13 may introduce a derived graph representation of verification diagnostics.

Conceptual graph:

`G = (N, E, S, I)`

where:

- `N = nodes`
- `E = parity edges`
- `S = verification surfaces`
- `I = determinism incidents`

This graph MAY be used to analyze:

- verifier clusters
- authority drift topology
- determinism hotspots
- historical authority islands
- insufficient-evidence islands

However:

`Graph = observability topology`

and:

`Graph != consensus topology`

The graph is derived and non-canonical.

---

## 11. Relationship to Phase-12 and Phase-13

Phase-12 provides:

- trusted proof transport
- deterministic verification
- cross-node parity

Phase-13 expands:

- verification observability
- diagnostics tooling
- distributed divergence analysis

Phase-13 MUST NOT convert parity into consensus.

---

## 12. Governance Rule

Repository governance MUST preserve the distinction:

`COMPLETED_LOCAL != closure`

Parity diagnostics MAY become strong before phase closure.

Whole-phase closure still requires the normative CI gates defined in `requirements.md`.

---

## 13. Summary

The parity layer:

- reveals disagreement
- explains divergence
- supports observability

The parity layer does not:

- enforce agreement
- resolve authority
- implement consensus

The boundary is intentional and MUST remain stable as Phase-13 grows.

---

**Maintained by:** AykenOS Architecture Board
**Status:** Draft (Phase-13 preparation)

# AykenOS Architecture - One Page

**Version:** 1.0
**Status:** Informational architecture map
**Date:** 2026-03-13
**Phase:** Phase-12 / Phase-13 boundary
**Type:** Non-normative reference artifact
**Related Spec:** `README.md`, `docs/roadmap/overview.md`, `tasks.md`, `AYKENOS_GLOBAL_ARCHITECTURE_DIAGRAM.md`, `AYKENOS_TECHNICAL_DEFINITION_SET.md`, `VERIFICATION_MODEL.md`, `VERIFICATION_INVARIANTS.md`, `PARITY_LAYER_ARCHITECTURE.md`, `PROOFD_DIAGNOSTICS_SERVICE_SURFACE.md`, `PHASE13_ARCHITECTURE_MAP.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`

---

## 1. Purpose

This document gives a one-page architecture view of AykenOS at the current `Phase-12` local closure-ready boundary.

It exists to keep one distinction explicit:

`implementation layers != governance state`

Current repo truth is:

- `Phase-10 = official closed`
- `Phase-11 = official closed`
- `Phase-12 = local closure-ready`
- `CURRENT_PHASE = 10` until formal transition workflow executes

This document is descriptive.

It does not redefine acceptance criteria or phase governance.

### 1.1 Canonical AykenOS Technical Definition

AykenOS is a deterministic verification architecture that separates kernel execution, verification semantics, evidence artifacts, and distributed diagnostics into explicit layers. The kernel provides mechanism, userspace verification services produce artifact-bound verdicts and receipts, and parity/topology surfaces expose cross-node observability without elevating diagnostics into authority or consensus. In this model, artifacts are the canonical truth interface, services wrap canonical artifacts, and distributed verification scales through evidence-first observability rather than truth election or replicated-state consensus.

---

## 2. Core Model

AykenOS is organized around four primary layers:

`kernel / runtime`

`-> deterministic verification`

`-> evidence artifacts`

`-> diagnostics / observability`

This separation keeps execution, verification, artifact truth surfaces, and distributed diagnostics distinct.

---

## 3. Layered Architecture

### 3.1 Kernel / Runtime

The kernel is the execution substrate.

Primary responsibilities:

- process execution
- memory management
- syscall interface
- scheduler
- capability security

Architectural rule:

`kernel = mechanism`

and:

`policy = Ring3 userspace`

### 3.2 Verification Substrate

The verification substrate is the deterministic verification engine.

Primary components:

- `proof-verifier`
- trust-policy evaluation
- signer resolution
- quorum validation
- replay determinism checks

Core invariant:

`same subject + same context + same authority -> same verdict`

This layer produces the canonical verification semantics.

### 3.3 Evidence Layer

Verification results are exported as artifacts.

Primary artifact families:

- receipts
- run manifests
- verification reports
- trust-evaluation outputs

Architectural rule:

`services = temporary interface`

`artifacts = canonical truth surface`

### 3.4 Diagnostics Layer

The diagnostics layer exposes distributed observability artifacts.

Primary structures:

- parity diagnostics
- convergence reports
- determinism incidents
- authority topology
- incident graph

Its role is:

`observe truth divergence`

not:

`select truth`

---

## 4. Service Boundary

`proofd` is the primary userspace service surface.

Its responsibilities are:

- verification execution
- signed receipt production
- diagnostics query
- run-scoped artifact discovery

It MUST NOT become:

- authority arbitration
- consensus
- replay execution
- truth election

Correct service sentence:

`proofd = verification + diagnostics service`

and:

`proofd != authority surface`

---

## 5. Phase-13 Boundary

`Phase-13` is not a new truth theory.

Its role is:

`distributed verification expansion`

The most likely growth areas are:

- verifier federation diagnostics
- registry propagation
- verification context distribution
- replicated verification boundary analysis
- service-backed distributed observability artifacts

This growth must not redefine the existing truth surfaces.

---

## 6. Governing Invariants

The architecture is held together by a small set of stable rules:

- `verification != authority`
- `authority != consensus`
- `parity = diagnostics`
- `artifacts are canonical interfaces`
- `services wrap canonical artifacts`
- `diagnostics remain derived structures`

These invariants are the main defense against scope drift in `Phase-13`.

---

## 7. Explicit Non-Goals

The following remain outside initial `Phase-13` scope unless separately ratified:

- distributed consensus
- global event ordering
- majority truth election
- cluster authority arbitration
- implicit trust-reputation systems
- automatic replay execution

If components start doing those things, the architecture has moved into a different systems category.

---

## 8. System Summary

AykenOS can be summarized as:

`deterministic verification architecture`

`+ artifact-first truth model`

`+ distributed diagnostics observability`

Short form:

`verification -> evidence -> distributed diagnostics`

---

## 9. Why This Artifact Exists

This one-page map is intended to:

- help new readers understand the system quickly
- prevent phase-to-phase scope drift
- keep `Phase-13` expansion aligned with the current invariants
- provide a compact architecture reference for technical and research communication

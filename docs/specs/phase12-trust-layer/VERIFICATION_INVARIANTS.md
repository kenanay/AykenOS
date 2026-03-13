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

### 2.3 Service Wrapper Invariant

`services wrap canonical artifacts`

Service APIs may execute verification and expose artifacts, but they do not replace the artifact-bound truth surface.

### 2.4 Authority Separation Invariant

`verification != authority`

Computing a verification result does not itself decide who may authoritatively reuse that result.

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

---

## 3. Drift Signals

The following changes indicate architectural drift:

- a service API becoming the primary truth surface
- diagnostics outputs being consumed as authority decisions
- parity or topology being used to elect system truth
- replay admission being implied by verification success
- federation semantics drifting into hidden consensus

If those changes occur, AykenOS has moved out of its intended category.

---

## 4. Summary

The shortest stable rule set is:

- `verification != authority`
- `authority != consensus`
- `parity = diagnostics`
- `artifacts = canonical interface`
- `services wrap canonical artifacts`

These invariants are the main defense against Phase-13 scope drift.

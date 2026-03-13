# Artifact Schema

**Version:** 1.0
**Status:** Informational artifact model
**Date:** 2026-03-13
**Phase:** Phase-12 / Phase-13 boundary
**Type:** Non-normative artifact schema note
**Related Spec:** `VERIFICATION_MODEL.md`, `AYKENOS_GLOBAL_ARCHITECTURE_DIAGRAM.md`, `AYKENOS_TECHNICAL_DEFINITION_SET.md`, `PROOF_EXCHANGE_PROTOCOL_MESSAGE_FORMAT.md`, `PROOFD_DIAGNOSTICS_SERVICE_SURFACE.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`

---

## 1. Purpose

This document defines the compact artifact schema used by the AykenOS verification architecture.

Its role is to make one rule explicit:

`truth surfaces are carried by artifacts`

This note does not define every field of every artifact family.

It defines the architectural schema categories that the rest of the system depends on.

---

## 2. Core Artifact Surface

In the verification model:

`R = (Q, V, E)`

where:

- `Q = (S, C, A)`
- `V`
  - verdict
- `E`
  - artifact surface

The compact AykenOS artifact surface is:

`E = (receipt, manifest, verification_report, audit_artifact, diagnostics_artifact_set)`

This means artifacts are not an optional by-product.

They are part of the verification result object.

---

## 3. Artifact Families

### 3.1 Receipt

The receipt is the signed or unsigned verification result artifact that binds a verifier outcome to a concrete verdict subject.

Typical receipt responsibilities:

- verdict binding
- verifier identity binding
- authority-aware verification reuse boundary
- receipt signature verification

Architectural rule:

`receipt != portable identity`

### 3.2 Manifest

The manifest records run-scoped execution facts about verification.

Typical manifest responsibilities:

- request contract recording
- receipt mode recording
- emitted artifact references
- run-local reproducibility support

Architectural rule:

`manifest = execution trace artifact`

not:

`manifest = truth election surface`

### 3.3 Verification Report

The verification report is the structured explanation of the verification outcome.

Typical report responsibilities:

- status reporting
- violation reporting
- subject/context/authority binding visibility
- machine-readable diagnostics for local verification

### 3.4 Audit Artifact

Audit artifacts persist append-only or chain-linked verification history.

Typical audit responsibilities:

- event recording
- chain integrity
- replayable audit evidence

Architectural rule:

`audit artifact != global consensus ledger`

### 3.5 Diagnostics Artifact Set

Diagnostics artifacts expose derived observability over verification outputs.

Typical diagnostics artifacts:

- parity reports
- determinism incidents
- convergence reports
- authority topology reports
- incident graphs

Architectural rule:

`diagnostics artifacts = derived observability`

not:

`diagnostics artifacts = authority decision`

---

## 4. Schema Rule

The stable schema relation is:

`verification inputs -> verdict -> artifacts`

not:

`service response -> system truth`

This keeps the AykenOS artifact model aligned with its evidence-first architecture.

---

## 5. Canonical Artifact Rule

Artifacts are the canonical durable interface between:

- verification execution
- service surfaces
- transport
- diagnostics
- research and audit workflows

So the governing rule is:

`artifacts = canonical interface`

and:

`services wrap canonical artifacts`

---

## 6. Summary

The compact artifact schema is:

`E = (receipt, manifest, verification_report, audit_artifact, diagnostics_artifact_set)`

This is the artifact layer that makes AykenOS an evidence-first verification architecture.

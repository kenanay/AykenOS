# AykenOS System Positioning Table

**Version:** 1.0
**Status:** Informational research artifact
**Date:** 2026-03-13
**Phase:** Phase-12 / Phase-13 boundary
**Type:** Non-normative system-positioning reference
**Related Spec:** `AYKENOS_GLOBAL_ARCHITECTURE_DIAGRAM.md`, `AYKENOS_TECHNICAL_DEFINITION_SET.md`, `AYKENOS_RESEARCH_POSITIONING.md`, `AYKENOS_UNIQUE_ARCHITECTURAL_DECISIONS.md`, `AYKENOS_VS_BLOCKCHAIN_ARCHITECTURAL_DIFFERENCE.md`

---

## 1. Purpose

This document positions AykenOS relative to adjacent verification, trust, and supply-chain security systems.

The goal is not feature comparison.

The goal is architectural positioning across a small set of stable axes:

- primary problem addressed
- truth surface
- authority model
- truth election
- consensus requirement
- artifact role
- diagnostics role

This table is intended to clarify the architectural category AykenOS belongs to.

---

## 2. System Positioning Table

| System | Primary Problem | Truth Surface | Authority Model | Truth Election | Consensus Required | Artifact Role | Diagnostics Role |
|---|---|---|---|---|---|---|---|
| AykenOS | Deterministic verification architecture | Evidence artifacts (`receipts`, `manifests`, verification reports) | Explicit verifier authority model | None (`deterministic verification`) | No | Canonical interface | Distributed verification observability |
| Blockchain | Distributed state agreement | Ledger state | Validator consensus | Consensus protocol | Yes | Transaction history | Network health monitoring |
| TUF | Secure software-update distribution | Signed metadata | Root and delegated keys | Metadata authority | No | Package metadata verification | Minimal |
| Sigstore | Keyless artifact signing and transparency-backed authenticity | Transparency log plus signatures | Fulcio and Rekor infrastructure | Transparency log | Partial | Artifact signatures | Log transparency |
| `in-toto` | Supply-chain step verification | Layout plus link metadata | Layout owner keys | Layout policy | No | Step evidence | Limited |
| Reproducible Builds | Build determinism | Build outputs | Community verification | Community verification | No | Build outputs | Comparison tooling |

---

## 3. Architectural Interpretation

The systems above belong to different architectural classes.

### 3.1 Consensus Systems

Example:

- blockchain systems

These systems require network-wide agreement on a global state.

Architectural rule:

`truth = consensus state`

AykenOS does not belong to this category.

### 3.2 Metadata Verification Systems

Examples:

- TUF
- `in-toto`

These systems secure artifact distribution through signed metadata and policy-bearing metadata chains.

Architectural rule:

`truth = signed metadata chain`

AykenOS extends beyond this model by introducing deterministic verification semantics and distributed diagnostics over verifier outputs.

### 3.3 Signature Infrastructure Systems

Example:

- Sigstore

These systems optimize artifact signing, identity binding, and transparency.

Architectural rule:

`truth = transparency log + signatures`

AykenOS instead treats signatures as one input surface inside a larger verification architecture.

### 3.4 Deterministic Verification Systems

AykenOS introduces a different architectural model.

Core invariant:

`same subject + same context + same authority -> same verdict`

Truth is not defined by consensus or by metadata chains alone.

Instead:

`truth = artifact-bound verification results`

Diagnostics can then expose distributed observability across those results without becoming truth-election machinery.

---

## 4. AykenOS Architectural Category

AykenOS can be described as:

`deterministic verification architecture`

`+ artifact-first truth surfaces`

`+ distributed diagnostics observability`

The architecture explicitly separates:

- execution mechanism
- verification semantics
- artifact truth surfaces
- diagnostics observability

This separation allows distributed verification expansion without introducing consensus.

---

## 5. Governing Distinctions

The following rules distinguish AykenOS from the compared systems:

- `verification != authority`
- `authority != consensus`
- `parity = diagnostics`
- `artifacts = canonical interface`
- `services wrap canonical artifacts`

These rules prevent distributed verification diagnostics from drifting into authority or consensus layers.

---

## 6. Architectural Summary

Consensus systems elect truth.

Metadata systems authorize truth.

Transparency systems log truth.

AykenOS computes truth through deterministic verification.

---

## 7. Why This Table Exists

This artifact exists to stabilize how AykenOS is described across:

- research discussions
- architecture documentation
- paper drafts
- conference presentations

The intended outcome is:

`clear architectural positioning without terminology drift`

# Verifier Authority Model

**Version:** 1.0
**Status:** Informational authority model
**Date:** 2026-03-13
**Phase:** Phase-12 / Phase-13 boundary
**Type:** Non-normative authority model note
**Related Spec:** `VERIFICATION_MODEL.md`, `VERIFICATION_INVARIANTS.md`, `VERIFIER_AUTHORITY_SEMANTICS_AND_DELEGATION_CONTRACT.md`, `VERIFIER_AUTHORITY_RESOLUTION_ALGORITHM.md`, `VERIFIER_AUTHORITY_GRAPH_CONSTRAINTS.md`, `VERIFIER_REGISTRY_LINEAGE_AND_DISTRIBUTION_MODEL.md`, `AUTHORITY_TOPOLOGY_FORMAL_MODEL.md`

---

## 1. Purpose

This document defines the compact verifier authority model used by AykenOS.

Its role is to summarize the authority surface without replacing the underlying contracts and algorithms.

The central rule is:

`valid verifier signature != verifier authority`

---

## 2. Core Authority Surface

In the AykenOS verification model, the authority surface is:

`A = (result_class, verifier_registry_snapshot_hash, effective_authority_scope, authority_chain_id)`

This keeps authority explicit across four dimensions:

- result class
- registry lineage
- effective scope
- resolved authority chain

Authority is therefore not derived from a key alone.

It is derived from explicit trust and lineage inputs.

---

## 3. Authority Components

### 3.1 Verifier Identity

A verifier may have:

- node identity
- verifier key identity
- receipt-signing capability

This is necessary for receipt verification but not sufficient for distributed authority.

### 3.2 Registry Lineage

Verifier authority is anchored in an explicit verifier registry snapshot lineage.

Relevant fields include:

- snapshot hash
- parent hash
- epoch
- scope

Architectural rule:

`same receipt + different registry lineage => different authority interpretation`

### 3.3 Authority Scope

Authority must remain scope-bounded.

Typical scope classes include:

- distributed receipt issuer
- parity reporter
- context distributor
- historical audit only

Scope is least-privilege and fail-closed.

### 3.4 Authority Chain

Resolved distributed authority is carried by:

`authority_chain_id`

This binds delegated or rooted authority to a deterministic chain interpretation.

If resolution is ambiguous, authority fails closed.

---

## 4. Delegation Model

Verifier authority may be delegated only under explicit graph constraints.

The stable rules are:

- delegation is default-deny
- delegation graph must be acyclic
- delegated scope must not widen
- current authority must resolve uniquely

This means verifier authority is:

`explicitly modeled`

not:

`implicitly trusted`

---

## 5. Authority Separation Rules

The authority model depends on the following separations:

- `verification != authority`
- `authority != consensus`
- `authority visibility != authority arbitration`

These rules are the reason authority can be observed, compared, and diagnosed without silently becoming a control plane.

---

## 6. Summary

The compact verifier authority model is:

`A = (result_class, verifier_registry_snapshot_hash, effective_authority_scope, authority_chain_id)`

This is the authority surface that keeps AykenOS distributed verification explicit, fail-closed, and non-consensus.

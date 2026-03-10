# Verification Context Portability and Distribution Protocol

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-08
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Related Spec:** `requirements.md`, `VERIFICATION_CONTEXT_DISTRIBUTION_CONTRACT.md`, `VERIFICATION_CONTEXT_OBJECT_SPEC.md`, `VERIFIER_ATTESTATION_AND_TRUST_REGISTRY_CONTRACT.md`, `VERIFIER_AUTHORITY_SEMANTICS_AND_DELEGATION_CONTRACT.md`, `VERIFIER_REGISTRY_LINEAGE_AND_DISTRIBUTION_MODEL.md`, `CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`, `PROOF_BUNDLE_ATTACK_SURFACE_SECURITY_MODEL.md`, `tasks.md`

---

## 1. Purpose

This document defines how distributed verification context becomes portable across nodes without collapsing into bundle-local or receipt-local ambiguity.

Its job is to make the following transportable and reconstructable:
- the canonical verification context object
- the policy snapshot identity used to evaluate trust
- the registry snapshot identity used to evaluate trust
- the context-rules identity that governs distributed interpretation

This protocol is normative for:
- distributed receipt reuse
- cross-node parity exchange
- future `proofd` context transport

It does not define:
- consensus
- remote fetch authentication
- receipt DAG federation
- producer proof transport itself

---

## 2. Problem Statement

Phase-12 already defines:
- `verdict_subject`
- `verification_context_id`
- verifier authority semantics

That is enough to classify distributed agreement, but not enough to transport it safely.

Without an explicit portability protocol, nodes may share:
- a receipt without the context object
- a context identifier without resolvable context bytes
- a context object without the exact policy or registry material it refers to

This leads to:
- context drift
- false parity claims
- historical receipt reuse under current trust semantics
- local correctness being mistaken for distributed correctness

---

## 3. Core Separation

The following artifacts MUST remain distinct:

- proof artifact
  - what was evaluated
- context artifact
  - under which distributed rules it was evaluated
- verifier-trust artifact
  - why the verifying node may speak as distributed authority

Critical rule:

`portable proof != portable context != trusted verifier authority`

Receipt transport MUST NOT collapse these into one object.

---

## 4. Portable Context Package

### 4.1 Minimal Canonical Shape

Distributed transport MUST carry either a full inline context package or content-addressed references that resolve to the same canonical material.

Recommended minimal package:

```json
{
  "protocol_version": 1,
  "verification_context_id": "sha256:<hex>",
  "context_object_ref": "cas:sha256:<hex>",
  "context_rules_ref": "cas:sha256:<hex>",
  "policy_snapshot_ref": "cas:sha256:<hex>",
  "registry_snapshot_ref": "cas:sha256:<hex>"
}
```

### 4.2 Inline Form

The protocol MAY carry inline objects instead of refs:

```json
{
  "protocol_version": 1,
  "verification_context_id": "sha256:<hex>",
  "context_object": { "...": "..." },
  "context_rules_object": { "...": "..." },
  "policy_snapshot": { "...": "..." },
  "registry_snapshot": { "...": "..." }
}
```

### 4.3 Mixed Form

Inline and reference forms MAY be mixed, provided that every carried or resolved object is canonical and hash-bound.

### 4.4 Protocol Invariant

The package MUST be sufficient to reconstruct the exact local acceptance context used by the sender.

If it is not sufficient, distributed trust reuse MUST fail closed.

---

## 5. Resolution Rules

### 5.1 Context Object Resolution

`verification_context_id` MUST resolve to the exact canonical context object defined in:

`VERIFICATION_CONTEXT_OBJECT_SPEC.md`

### 5.2 Policy Snapshot Resolution

`policy_snapshot_ref` or inline `policy_snapshot` MUST resolve to the exact policy bytes whose canonical hash equals `policy_hash` in the context object.

### 5.3 Registry Snapshot Resolution

`registry_snapshot_ref` or inline `registry_snapshot` MUST resolve to the exact registry bytes whose canonical hash equals `registry_snapshot_hash` in the context object.

### 5.4 Context Rules Resolution

`context_rules_ref` or inline `context_rules_object` MUST resolve to the exact rules bytes whose canonical hash equals `context_rules_hash` in the context object.

### 5.5 No Silent Substitution Rule

The receiving node MUST NOT silently replace:
- policy material
- registry material
- context rules material

with local defaults when evaluating a distributed trust claim.

---

## 6. Portability Semantics

### 6.1 External Input Rule

The protocol may transport policy and registry material, but this does not change their status as trust inputs external to the proof bundle.

The proof bundle itself MUST NOT silently import distributed context.

### 6.2 Content-Addressed Rule

If a reference form is used, resolution MUST produce canonical bytes whose recomputed identity equals the declared reference identity.

### 6.3 Reconstructability Rule

A node may claim portable distributed context only if another node can reconstruct:
- the same `verification_context_id`
- the same `policy_hash`
- the same `registry_snapshot_hash`
- the same `context_rules_hash`

from the transported material.

### 6.4 Mutation Rule

Transport framing MUST NOT mutate:
- `verification_context_id`
- the canonical bytes used to compute it
- the canonical bytes of the referenced policy, registry, or context-rules objects

---

## 7. Fail-Closed Rules

The receiving node MUST reject shared distributed trust claims when:
- the context package is missing
- a required ref cannot be resolved
- a resolved object does not parse
- recomputed `verification_context_id` differs
- recomputed `policy_hash` differs
- recomputed `registry_snapshot_hash` differs
- recomputed `context_rules_hash` differs

Recommended classification:
- missing material => `PARITY_INSUFFICIENT_EVIDENCE`
- unequal context object => `PARITY_CONTEXT_MISMATCH`
- unequal verifier-trust interpretation after successful resolution => `PARITY_VERIFIER_MISMATCH`

---

## 8. Historical and Temporal Semantics

### 8.1 Historical Portability

An older context package MAY remain portable as historical evidence.

It MUST NOT automatically remain current acceptance context.

### 8.2 Epoch-Aware Interpretation

If context lineage or epoch fields are present, they MUST be preserved during transport.

### 8.3 No Silent Upgrade Rule

An older portable context package MUST NOT be silently reclassified as current distributed context after:
- policy evolution
- registry evolution
- verifier contract evolution
- context-rules evolution

---

## 9. Parity Implications

Cross-node parity claims require more than equal receipts.

A parity-capable transport MUST make it possible to compare:
- `verdict_subject`
- `verification_context_id`
- verifier-trust semantics

Therefore the portability protocol is a prerequisite for:
- `ci-gate-cross-node-parity` growth beyond local synthetic fixtures
- future A/B/C/D parity matrices
- `proofd` distributed trust responses

Without portable context resolution, parity claims remain verifier-local only.

---

## 10. Acceptance Criteria

10.1. THE System SHALL define a verification context portability protocol distinct from proof transport and receipt transport
10.2. THE protocol SHALL carry either inline canonical context material or content-addressed references sufficient to reconstruct the sender acceptance context
10.3. THE protocol SHALL preserve the distinction between proof artifact, context artifact, and verifier-trust artifact
10.4. THE receiving node SHALL recompute and verify `verification_context_id` from the transported context object
10.5. THE receiving node SHALL recompute and verify `policy_hash`, `registry_snapshot_hash`, and `context_rules_hash` from transported or resolved material
10.6. Missing or unresolvable context transport material SHALL fail closed for distributed trust reuse
10.7. THE protocol SHALL NOT permit silent substitution of local default policy, registry, or context-rules material for a claimed distributed context
10.8. Portable context transport SHALL NOT change the rule that policy and registry remain external trust inputs rather than bundle-authoritative inputs
10.9. Historical context packages MAY remain audit-valid artifacts but SHALL NOT automatically remain current distributed trust context
10.10. Cross-node parity claims SHALL rely on reconstructable context transport, not on receipt transport alone

---

## 11. Summary

Phase-12 already defines what distributed context means.

This protocol defines how that context becomes portable.

Without it, receipts remain portable but context does not, and distributed trust degrades into ambiguous local truth exchange.

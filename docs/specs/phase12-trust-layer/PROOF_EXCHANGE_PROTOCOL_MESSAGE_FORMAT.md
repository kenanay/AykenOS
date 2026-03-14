# Proof Exchange Protocol Message Format

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-08
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Type:** Normative message format note
**Related Spec:** `requirements.md`, `tasks.md`, `PROOF_BUNDLE_V2_SPEC.md`, `VERIFICATION_CONTEXT_OBJECT_SPEC.md`, `VERIFICATION_CONTEXT_PORTABILITY_AND_DISTRIBUTION_PROTOCOL.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`, `CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`

---

## 1. Purpose

This document defines the local Phase-12 proof-exchange message format used by `P12-13`.

Its purpose is narrow:

- transport a portable proof bundle without mutating payload identity
- transport trust overlay material without collapsing it into portable identity
- transport verification-context material without redefining local verifier-core semantics
- optionally transport receipt artifacts without making receipts part of portable identity

This message format defines a transport contract.

It does not define:

- service discovery
- remote fetch
- network encryption
- distributed consensus
- `proofd` request/response APIs

---

## 2. Core Invariant

The exchange protocol MUST preserve this separation:

`portable payload != trust overlay != verification context != receipt artifact != transport metadata`

Transport MUST NOT mutate:

- `bundle_id`
- `trust_overlay_hash`
- `verification_context_id`

Transport metadata MUST remain non-authoritative.

---

## 3. Top-Level Message Shape

The canonical top-level shape is:

```json
{
  "protocol_version": 1,
  "exchange_mode": "proof_bundle_transport_v1",
  "portable_payload": { "...": "..." },
  "trust_overlay": { "...": "..." },
  "verification_context": { "...": "..." },
  "receipt_artifact": { "...": "..." },
  "transport_metadata": { "...": "..." }
}
```

Required fields:

- `protocol_version`
- `exchange_mode`
- `portable_payload`
- `trust_overlay`
- `verification_context`
- `transport_metadata`

Optional fields:

- `receipt_artifact`

---

## 4. Portable Payload

`portable_payload` transports the Phase-11/Phase-12 portable proof identity surface.

Canonical inline form:

```json
{
  "payload_form": "proof_bundle_v2",
  "bundle_id": "<bundle_id>",
  "manifest": { "...": "..." },
  "checksums": { "...": "..." }
}
```

Rules:

- `bundle_id` MUST match the canonical recomputation from `manifest` and `checksums`
- transport MAY repackage bytes, but MUST NOT change portable identity
- receipt material MUST NOT be embedded into `portable_payload`

---

## 5. Trust Overlay

`trust_overlay` transports detached trust material for the same portable payload.

Canonical inline form:

```json
{
  "transport_form": "detached-inline",
  "bundle_id": "<bundle_id>",
  "producer": { "...": "..." },
  "signature_envelope": { "...": "..." },
  "trust_overlay_hash": "<trust_overlay_hash>"
}
```

Rules:

- `bundle_id` inside the overlay MUST match the portable payload `bundle_id`
- `trust_overlay_hash` MUST match canonical recomputation from `producer` and `signature_envelope`
- overlay transport MUST NOT mutate portable identity

---

## 6. Verification Context

`verification_context` transports the distributed interpretation surface needed to reconstruct trust evaluation.

Canonical inline form:

```json
{
  "protocol_version": 1,
  "verification_context_id": "<verification_context_id>",
  "context_object": { "...": "..." },
  "context_rules_object": { "...": "..." },
  "policy_snapshot": { "...": "..." },
  "registry_snapshot": { "...": "..." }
}
```

Rules:

- `verification_context_id` MUST match canonical recomputation from `context_object`
- `context_object.policy_hash` MUST match the canonical hash of `policy_snapshot`
- `context_object.registry_snapshot_hash` MUST match the canonical hash of `registry_snapshot`
- `context_object.context_rules_hash` MUST match the canonical hash of `context_rules_object`
- verification context transport MUST NOT redefine `bundle_id` or `trust_overlay_hash`

---

## 7. Receipt Artifact

`receipt_artifact` is optional transport for a derived verification artifact.

Canonical inline form:

```json
{
  "transport_form": "detached-inline",
  "receipt_type": "signed_verification_receipt",
  "receipt": { "...": "..." }
}
```

Rules:

- receipt transport is OPTIONAL for portable proof exchange
- receipt presence MUST NOT redefine payload identity or context identity
- receipt subject fields MUST continue to bind to:
  - `bundle_id`
  - `trust_overlay_hash`
  - `policy_hash`
  - `registry_snapshot_hash`
- missing receipt MUST NOT invalidate transport when the transport mode only requires portable proof + context

---

## 8. Transport Metadata

`transport_metadata` exists only for operational bookkeeping.

Canonical example:

```json
{
  "transport_id": "exchange-fixture-transport-1",
  "sender_node_id": "node-a",
  "sent_at_utc": "2026-03-08T12:15:00Z"
}
```

Rules:

- transport metadata MUST be non-authoritative
- changes in metadata MUST NOT alter:
  - `bundle_id`
  - `trust_overlay_hash`
  - `verification_context_id`
  - receipt binding semantics

---

## 9. Validation Contract

An implementation validating this message format MUST:

1. recompute `bundle_id` from transported manifest + checksums
2. recompute `trust_overlay_hash` from transported producer + signature envelope
3. recompute `policy_hash` from transported policy snapshot
4. recompute `registry_snapshot_hash` from transported registry snapshot
5. recompute `verification_context_id` from transported context object
6. reject any subject/context/overlay drift fail-closed
7. treat receipt transport as optional unless the surrounding transport mode explicitly requires it

Transport validation MUST fail closed when any identity-carrying surface drifts.

---

## 10. Mutation Semantics

The following mutations are REQUIRED to fail:

- portable payload `bundle_id` drift
- `trust_overlay_hash` drift
- `verification_context_id` drift
- receipt subject tuple drift when receipt transport is present and validated

The following mutation is ALLOWED without changing transport validity:

- metadata-only mutation under `transport_metadata`

This distinction is the minimum transport hardening rule for `P12-13`.

---

## 11. Gate Mapping

`ci-gate-proof-exchange` MUST validate this contract through a mutation matrix that includes at least:

- baseline separated inline transport
- metadata-only mutation
- receipt-absent portable transfer
- payload identity mutation
- overlay identity mutation
- context identity mutation
- receipt subject mutation

The gate MUST export:

- `exchange_contract_report.json`
- `transport_mutation_matrix.json`
- `report.json`
- `violations.txt`

---

## 12. Non-Goals

This message format does not define:

- remote node discovery
- request/response service APIs
- transport-level authentication
- multi-hop routing
- authority lookup federation
- quorum trust exchange

Those remain later `proofd` or Phase-13 concerns.

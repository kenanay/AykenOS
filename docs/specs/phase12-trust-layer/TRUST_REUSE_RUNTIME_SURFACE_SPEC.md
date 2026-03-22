# Trust Reuse Runtime Surface Specification

**Version:** 0.1  
**Status:** Draft (Phase-13 native runtime evidence contract)  
**Date:** 2026-03-14  
**Phase:** Phase-13 distributed verification observability  
**Type:** Normative runtime-evidence artifact specification

**Related Spec:** `AUTHORITY_SINKHOLE_COMPANION_FLOW_SPEC.md`, `AUTHORITY_SINKHOLE_ABSORPTION_GATE.md`, `CROSS_SURFACE_BASIN_ALIGNMENT_METRICS.md`, `VERIFICATION_CONTEXT_OBJECT_SPEC.md`, `VERIFIER_ATTESTATION_AND_TRUST_REGISTRY_CONTRACT.md`, `PROOFD_DIAGNOSTICS_SERVICE_SURFACE.md`

---

## 1. Purpose

This specification defines the minimum native runtime evidence surface for distributed trust reuse.

Its purpose is to:

- make trust-reuse activity observable without inventing authority semantics
- provide a native runtime source for Stage-2 sinkhole companion production
- bind trust reuse to explicit verification context and verifier-attestation material
- prevent receipt-only or cache-only trust reuse inference

The shortest rule is:

`trust reuse becomes native only when runtime evidence binds receipt, context, and verifier-trust semantics explicitly`

---

## 2. Boundary

This artifact is an observability surface only.

It is not:

- a routing surface
- a replay admission surface
- a verifier ranking surface
- an authority election surface

The artifact may describe trust-reuse outcomes.

It MUST NOT grant trust reuse by itself.

The shortest rule is:

`trust reuse runtime evidence != trust reuse authority`

---

## 3. Output Artifact

The minimum native runtime artifact is:

- `trust_reuse_runtime_surface.json`

This artifact should be emitted by the runtime component that actually evaluates distributed trust reuse semantics.

Current code reality:

- `proof-verifier` now exposes a `trust-reuse-runtime-evaluator` binary that materializes this surface from explicit receipt, verification-context, verifier-attestation, and verifier-registry artifacts
- rejected outcomes remain native runtime evidence when explicit trust-reuse evaluation happened and failed closed
- unresolved or malformed context / verifier-trust inputs still fail closed without native emission

It exists below Stage-2 companion production.

Correct layering:

`runtime trust reuse evidence -> trust_reuse_runtime_surface.json -> trust_reuse_flow_source.json -> trust_reuse_flow_report.json -> sinkhole Stage-2 metrics`

Incorrect layering:

`receipt -> guessed trust reuse -> sinkhole metrics`

---

## 4. Minimum Outer Shape

The minimum canonical outer shape is:

```json
{
  "surface_version": 1,
  "flow_surface": "trust_reuse_runtime",
  "status": "PASS",
  "run_id": "<run_id>",
  "source_kind": "local_runtime_evidence",
  "event_count": 1,
  "accepted_event_count": 1,
  "historical_only_event_count": 0,
  "rejected_event_count": 0,
  "events": [
    {
      "event_schema_version": 1,
      "event_id": "sha256:<digest>",
      "run_id": "<run_id>",
      "timestamp_unix_ns": 1710000000000000000,
      "subject_bundle_id": "<bundle_id>",
      "verification_context_id": "sha256:<policy_hash_projection>",
      "authority_chain_id": "sha256:<authority_chain>",
      "trust_reuse_outcome": "accepted",
      "terminal": true,
      "reused": true,
      "receipt_ref": "receipts/verification_receipt.json",
      "verification_context_ref": "cas:sha256:<context_object>",
      "verifier_attestation_ref": "cas:sha256:<attestation>",
      "verifier_registry_snapshot_hash": "<sha256-hex>"
    }
  ]
}
```

The `status` field is runtime materialization status only.

It does not mean the trust-reuse path is healthy.

---

## 5. Minimum Event Model

Every native trust-reuse event MUST contain at least:

```json
{
  "event_schema_version": 1,
  "event_id": "sha256:<digest>",
  "run_id": "<run_id>",
  "timestamp_unix_ns": "<unix-ns>",
  "subject_bundle_id": "<bundle_id>",
  "verification_context_id": "<policy_hash_projection>",
  "authority_chain_id": "<authority_chain_id>",
  "trust_reuse_outcome": "accepted | historical_only | rejected",
  "terminal": true,
  "reused": true,
  "receipt_ref": "<receipt-ref>",
  "verification_context_ref": "<context-ref>",
  "verifier_attestation_ref": "<attestation-ref>",
  "verifier_registry_snapshot_hash": "<registry-hash>"
}
```

This is the minimum native event set.

The event must prove that trust reuse was actually evaluated under explicit distributed trust semantics.

---

## 6. Canonical Field Roles

| Field | Meaning |
|---|---|
| `event_schema_version` | versioned event schema handle |
| `event_id` | content-addressed identity of the runtime trust-reuse event |
| `run_id` | local runtime run handle |
| `timestamp_unix_ns` | event ordering for windowed analysis |
| `subject_bundle_id` | verification subject whose trust reuse was evaluated |
| `verification_context_id` | cross-surface comparison key for the verification context; current code reality projects this to the policy-bound context identity used by VDL and replay companion flows |
| `authority_chain_id` | practical verifier authority chain that backed the reuse decision |
| `trust_reuse_outcome` | runtime result of the trust-reuse evaluation |
| `terminal` | whether this event records the terminal trust-reuse path |
| `reused` | whether this is operational reuse rather than first-pass evaluation |
| `receipt_ref` | content-addressed or stable local reference to the reused receipt |
| `verification_context_ref` | reference to the canonical verification context object |
| `verifier_attestation_ref` | reference to the verifier attestation object used for trust semantics |
| `verifier_registry_snapshot_hash` | verifier trust registry hash used in the trust-reuse evaluation |

These fields are the minimum native trust-reuse contract.

Anything weaker is still fallback, not native runtime evidence.

Current code reality:

- `verification_context_id` is the behavioral comparison key used by Stage-2 sinkhole alignment
- `verification_context_ref` preserves the full canonical verification-context artifact identity

---

## 7. Native Trust-Reuse Rule

An event MUST NOT be treated as native trust-reuse evidence unless all of the following are explicit:

- receipt reference
- verification-context binding
- verifier attestation reference
- verifier trust registry binding
- authority chain binding
- runtime outcome

So the forbidden shortcuts are:

- receipt-only reuse
- cache-hit-only reuse
- context-free reuse
- attestation-free reuse
- registry-free reuse

The shortest rule is:

`receipt alone is insufficient to materialize native trust reuse`

---

## 8. Outcome Semantics

The minimum outcome class is:

- `accepted`
- `historical_only`
- `rejected`

Interpretation:

- `accepted` = current distributed trust reuse succeeded
- `historical_only` = trust reuse was recognized only as historical evidence
- `rejected` = trust reuse failed closed

The runtime surface records what happened.

It MUST NOT reinterpret these outcomes into:

- trust scores
- preferred authorities
- routing hints
- verifier reliability rankings

Rejected native events remain valid runtime evidence.

They must not be reclassified as malformed surface data merely because no reusable trust path was admitted.

The shortest rule is:

`native rejected != invalid surface`

---

## 9. Event Identity

Every event MUST carry a content-addressed `event_id`.

Recommended rule:

1. canonicalize the event without `event_id`
2. hash the canonical bytes with SHA-256
3. encode the digest as `sha256:<digest>`

This provides:

- duplicate-event guard
- deterministic append order
- forensic reproducibility

---

## 10. Stable Ordering

The canonical sort order is:

1. `timestamp_unix_ns` ascending
2. `subject_bundle_id`
3. `verification_context_id`
4. `event_id`

This keeps sinkhole companion production reproducible.

---

## 11. Optional Extended Fields

Native runtime emitters may additionally include:

- `verification_node_id`
- `verifier_id`
- `lineage_id`
- `execution_cluster_id`
- `source_run_id`
- `reuse_group_id`
- `surface_local_path_id`
- `trust_reuse_source`

These remain descriptive only.

They must not become:

- authority preference
- verifier reputation
- replay admission policy
- scheduler input

---

## 12. Producer Binding

The Stage-2 companion producer should treat this artifact as the preferred native source for trust reuse.

Current precedence should be:

1. native runtime trust-reuse surface
2. explicit request-bound fallback
3. otherwise `NOT_EVALUATED`

The current evaluator-backed implementation occupies tier 1.

The fallback path exists only as a transition surface.

The native runtime surface is the long-term contract.

---

## 13. Fail-Closed Rules

The runtime emitter MUST fail closed if:

- `verification_context_ref` is absent or unresolved
- `verifier_attestation_ref` is absent or unresolved
- `verifier_registry_snapshot_hash` is absent
- `authority_chain_id` is absent
- `trust_reuse_outcome` cannot be determined exactly
- `terminal` or `reused` would need to be guessed

The shortest rule is:

`if trust reuse semantics are not explicit, no native trust-reuse runtime event may be emitted`

---

## 14. Relationship to Existing Contracts

This runtime surface depends on:

- `VERIFICATION_CONTEXT_OBJECT_SPEC.md`
- `VERIFIER_ATTESTATION_AND_TRUST_REGISTRY_CONTRACT.md`

It does not replace them.

Instead:

- the verification context object explains under which distributed context reuse is valid
- the verifier attestation and trust registry explain why the remote verifier may be trusted
- the runtime surface proves that a concrete trust-reuse decision actually happened

So the correct reading is:

`context object + verifier attestation + trust registry + runtime decision = native trust reuse evidence`

---

## 15. Non-Goals

This surface does not:

- elect a trusted authority
- recommend a preferred verifier
- explain replay routing
- summarize sinkhole health
- replace the Stage-2 companion producer

Those remain separate layers.

---

## 16. Short Rule

`native trust reuse requires explicit runtime evidence of receipt reuse under explicit context and verifier-trust semantics`

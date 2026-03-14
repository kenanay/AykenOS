# `proofd` Service Closure Plan

**Version:** 1.0
**Status:** Draft (executed locally; Phase-13 preparation)
**Date:** 2026-03-11
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Type:** Non-normative closure-planning note
**Related Spec:** `requirements.md`, `tasks.md`, `PROOFD_DIAGNOSTICS_SERVICE_SURFACE.md`, `PROOFD_SERVICE_FINAL_HARDENING_CHECKLIST.md`, `PARITY_LAYER_ARCHITECTURE.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`

---

## 1. Purpose

This document defines the smallest correct path from the early `proofd` read-only diagnostics skeleton to `P12-16` closure-ready service behavior.

It does not change the existing `proofd` boundary.

Its role is to answer four questions:

- what `proofd` must do before `P12-16` can be considered closure-ready
- which endpoint surface is required
- which evidence artifacts the gate must export
- how bootstrap service behavior decomposes into closure slices

The core planning rule is:

`bootstrap diagnostics PASS != proofd closure PASS`

That planning rule has now been executed locally in `run-local-phase12c-closure-2026-03-11`.

---

## 2. Current State

Current local reality:

- `userspace/proofd/` exists
- root and run-scoped diagnostics endpoints are active
- a local `POST /verify/bundle` execution slice is active
- `ci-gate-proofd-service` execution slice is active
- the current gate validates diagnostics passthrough, explicit policy/registry binding, signed receipt emission evidence, receipt signature verification, authority-aware receipt verification, and receipt-boundary preservation

Current local reality now also provides:

- final `P12-16` hardening semantics
- closure-level repeated execution evidence over the final service contract
- receipt verification and authority-aware receipt verification reports
- request-bound timestamp preservation under repeated identical execution

So the current state is:

`P12-16 = COMPLETED_LOCAL`

and:

`proofd = local closure-ready verification execution service`

---

## 3. Closure Target

`P12-16` becomes closure-ready when `proofd` satisfies Requirement 10 as a service and not merely as a diagnostics shell.

Minimum closure target:

- bundle intake
- verifier-core execution
- explicit policy input binding
- explicit registry input binding
- receipt emission
- diagnostics exposure of produced artifacts
- deterministic repeated service results for the same inputs

The closure target is therefore:

`proofd = verification execution service + read-only diagnostics surface`

while still preserving:

`proofd != authority surface`

and:

`proofd != consensus`

This target is now satisfied locally. The remaining work is remote / official confirmation, not new local service behavior.

---

## 4. Closure Invariants

### 4.1 Userspace Verification Invariant

`proofd` MUST execute verification in userspace.

It MUST NOT migrate trust evaluation into Ring0.

### 4.2 Verifier-Core Delegation Invariant

`proofd` MUST call verifier-core semantics.

It MUST NOT invent a second verification engine with divergent verdict rules.

### 4.3 Explicit Input Binding Invariant

`proofd` verification requests MUST bind:

- bundle input
- policy input
- registry input
- receipt mode

No implicit local default policy or registry substitution may occur in closure-ready mode.

### 4.4 Receipt-Derivation Invariant

Receipts emitted by `proofd` remain derived verification artifacts.

`proofd` MUST NOT treat receipts as portable identity or authority objects.

### 4.5 Diagnostics Purity Invariant

Existing diagnostics endpoints MUST remain read-only artifact surfaces.

Adding verification execution MUST NOT cause diagnostics endpoints to recompute or reinterpret parity artifacts.

### 4.6 Service Determinism Invariant

Same request inputs MUST yield:

- same final verdict
- same verdict subject
- same receipt payload semantics

except for explicitly non-identity timestamp fields where allowed by the receipt contract.

---

## 5. Closure Endpoint Shape

The minimum closure surface should add one execution family while preserving current diagnostics.

### 5.1 Verification Execute

`POST /verify/bundle`

Minimum request shape:

```json
{
  "bundle_path": "/abs/path/to/proof_bundle",
  "policy_path": "/abs/path/to/policy.json",
  "registry_path": "/abs/path/to/registry.json",
  "receipt_mode": "emit_signed",
  "run_id": "run-proofd-local-r1",
  "receipt_signer": {
    "verifier_node_id": "node-b",
    "verifier_key_id": "receipt-ed25519-key-2026-03-a",
    "signature_algorithm": "ed25519",
    "private_key": "base64:...",
    "verified_at_utc": "2026-03-08T12:00:00Z"
  },
  "diversity_binding": {
    "verifier_id": "verifier-node-b",
    "authority_chain_id": "sha256:proofd-authority-chain-node-b",
    "lineage_id": "lineage-receipt-node-b",
    "execution_cluster_id": "cluster-local-a"
  },
  "replay_boundary_binding": {
    "replay_contract_id": "replay-contract-proofd-local-a",
    "source_run_id": "fixture-run",
    "reuse_group_id": "reuse-group-proofd-a",
    "surface_local_path_id": "replay-path-proofd-a"
  },
  "trust_reuse_binding": {
    "trust_reuse_source": "trust-overlay-cache",
    "source_run_id": "source-run-proofd-bootstrap-a",
    "reuse_group_id": "reuse-group-proofd-a",
    "surface_local_path_id": "trust-path-proofd-a"
  }
}
```

`trust_reuse_binding` is now a fallback-only surface.

When bundle-native trust-reuse runtime evidence exists, `proofd` must prefer that native surface instead of the request-bound fallback.

Minimum response shape:

```json
{
  "status": "ok",
  "run_id": "run-proofd-local-r1",
  "verdict": "TRUSTED",
  "verdict_subject": {
    "bundle_id": "...",
    "trust_overlay_hash": "...",
    "policy_hash": "...",
    "registry_snapshot_hash": "..."
  },
  "receipt_emitted": true,
  "receipt_path": "receipts/verification_receipt.json",
  "behavioral_observability_emitted": true,
  "audit_ledger_path": "verification_audit_ledger.jsonl",
  "verification_diversity_ledger_binding_path": "verification_diversity_ledger_binding.json",
  "verification_diversity_ledger_path": "verification_diversity_ledger.json",
  "replay_boundary_flow_source_path": "replay_boundary_flow_source.json",
  "replay_boundary_flow_source_origin": "runtime_bundle_replay",
  "trust_reuse_flow_source_path": "trust_reuse_flow_source.json",
  "trust_reuse_flow_source_origin": "runtime_bundle_trust_reuse"
}
```

Run reuse rule:

- `proofd` MAY reuse an existing `run_id` only when the canonical request fingerprint matches the existing run manifest
- a different request under the same `run_id` MUST fail closed
- request bodies above the bounded local execution limit MUST fail closed before verification
- when `diversity_binding` is present, `proofd` MUST emit a deterministic `replay_boundary_flow_source.json` artifact from the bundle's native replay runtime surface and bind it to the same signed-receipt timestamp and diversity authority context
- when `reports/trust_reuse_runtime_surface.json` exists inside the bundle, `proofd` MUST prefer it and emit a deterministic `trust_reuse_flow_source.json` artifact with origin `runtime_bundle_trust_reuse`
- when the preferred native trust-reuse surface contains only rejected outcomes, `proofd` MUST still keep native precedence and emit `trust_reuse_flow_source.json` as `NO_REUSABLE_EVENTS` rather than treating the surface as malformed or silently falling back
- the current native trust-reuse surface may be materialized ahead of `proofd` by the `proof-verifier` `trust-reuse-runtime-evaluator` from explicit receipt, verification-context, verifier-attestation, and verifier-registry artifacts
- when bundle-native trust-reuse runtime evidence is absent but `trust_reuse_binding` is present, `proofd` MUST emit a deterministic `trust_reuse_flow_source.json` artifact as an explicit request-bound fallback surface
- if `replay_boundary_binding.source_run_id` is supplied alongside a native replay surface, it MUST match the bundle `meta/run.json` `run_id` or fail closed
- the future native replacement for the trust-reuse fallback must satisfy `TRUST_REUSE_RUNTIME_SURFACE_SPEC.md`

### 5.2 Diagnostics Remain Stable

The following existing families remain read-only:

- `GET /diagnostics/parity`
- `GET /diagnostics/incidents`
- `GET /diagnostics/drift`
- `GET /diagnostics/convergence`
- `GET /diagnostics/failure-matrix`
- `GET /diagnostics/graph`
- `GET /diagnostics/authority-topology`
- `GET /diagnostics/authority-suppression`
- `GET /diagnostics/runs`
- `GET /diagnostics/runs/{run_id}`
- run-scoped diagnostics variants

No closure slice should widen those endpoints into authority, policy, or consensus behavior.

---

## 6. Gate Evidence Layout

The normative `ci-gate-proofd-service` output must include at least:

- `proofd_service_report.json`
- `proofd_receipt_report.json`
- `report.json`
- `violations.txt`

For closure-ready execution, the gate exports:

- `proofd_endpoint_contract.json`
- `proofd_verify_request.json`
- `proofd_verify_response.json`
- `proofd_run_manifest.json`
- `proofd_receipt_verification_report.json`
- `proofd_repeated_execution_report.json`

Recommended layout:

```text
evidence/run-*/gates/proofd-service/
  report.json
  violations.txt
  proofd_service_report.json
  proofd_receipt_report.json
  proofd_endpoint_contract.json
  proofd_verify_request.json
  proofd_verify_response.json
  proofd_run_manifest.json
  proofd_receipt_verification_report.json
  proofd_repeated_execution_report.json
```

Recommended report semantics:

- `proofd_service_report.json`
  - service mode
  - verification execution active
  - deterministic repeated execution result
  - diagnostics passthrough preserved
- `proofd_receipt_report.json`
  - receipt boundary preserved
  - receipt emission active
  - receipt verification path exercised
- `proofd_endpoint_contract.json`
  - root diagnostics checks
  - run-scoped diagnostics checks
  - verify endpoint request/response checks

---

## 7. Gate Decomposition

### 7.1 Bootstrap Slice

Already active:

- read-only diagnostics root endpoints
- run discovery
- run summary
- run-scoped diagnostics passthrough
- receipt-boundary preservation

This proves:

`proofd diagnostics boundary is real`

It does not prove:

`proofd verification execution`

### 7.2 Execution Slice

Now active locally:

- `POST /verify/bundle`
- local verifier-core delegation
- explicit policy/registry binding
- signed receipt emission evidence
- signed receipt verification evidence
- authority-aware signed receipt verification evidence
- repeated execution determinism over the current execution request

This proves:

`proofd can execute verification`

### 7.3 Receipt Slice

Now active locally:

- signed receipt artifact
- machine-readable receipt report over the signed path
- deterministic receipt-boundary handling for the current service contract

This proves:

`proofd can emit derived verification receipts`

### 7.4 Closure Slice

Now active locally:

- diagnostics passthrough still stable
- verification execution active
- policy/registry input binding active
- receipt emission active
- deterministic repeated verification request stable

This proves:

`proofd = closure-ready local verification execution service`

---

## 8. Failure Classes

The closure gate should fail on at least:

- missing verify endpoint
- implicit policy substitution
- implicit registry substitution
- verifier-core mismatch
- receipt emission missing when required
- diagnostics endpoint contract drift
- run-scoped artifact merge or mutation
- repeated-request determinism drift

Recommended failure labels:

- `verify_endpoint_missing`
- `policy_binding_missing`
- `registry_binding_missing`
- `verifier_core_semantics_drift`
- `receipt_emission_missing`
- `diagnostics_passthrough_drift`
- `run_artifact_merge_detected`
- `repeated_request_determinism_failed`

---

## 9. Non-Goals

The `P12-16` closure plan does **not** imply:

- remote registry distribution
- network trust federation
- authority arbitration
- consensus
- replay admission
- replicated execution

If a `proofd` implementation begins doing those things, it has crossed into later-phase territory.

---

## 10. Summary

The current local gate proves:

`proofd = verification execution service + read-only diagnostics surface`

The remaining local implementation gap is:

`none`

So the correct next implementation order is:

1. preserve the signed receipt closure assertions against drift
2. confirm the same contract in remote / official evidence
3. fold that evidence into the formal Phase-12 closure decision

The boundary remains:

`proofd = userspace verification service`

and:

`proofd != authority surface`

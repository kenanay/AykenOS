# Proof Bundle v2 Specification

**Status:** Draft
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Authority:** Draft until ratified by the Architecture Board
**Compatibility Target:** Phase-11 `proof_bundle` portability contract

---

## 1. Purpose

Proof Bundle v2 extends the Phase-11 portable proof bundle with a trust layer.

Target progression:

`portable proof bundle -> trusted proof bundle -> deterministic distributed verification`

Phase-11 already guarantees:
- proof artifacts exist
- proof artifacts are portable
- offline verdict parity is reproducible

Phase-12 adds:
- producer attribution
- detached signatures
- trust policy evaluation
- cross-node verification parity
- verification receipts

This specification MUST preserve the existing Phase-11 portability contract.

---

## 2. Scope

This specification defines:
- the v2 bundle directory structure
- portable identity rules
- trust overlay rules
- canonical JSON and hash rules
- detached signature envelope rules
- verifier inputs and outputs
- receipt generation rules

This specification does not define:
- kernel runtime changes
- Ring0 trust enforcement
- replicated execution
- distributed consensus protocol
- networking transport implementation details

Trust evaluation remains userspace or offline. Ring0 remains mechanism-only.

---

## 3. Architectural Constraints

The following constraints are non-negotiable:

1. Phase-11 portability semantics MUST remain intact.
2. Bundle identity immutability MUST be preserved.
3. Trust metadata MUST NOT mutate portable bundle identity.
4. Verification policy MUST remain outside Ring0.
5. Same bundle + same policy + same registry snapshot MUST produce the same verdict.
6. Key rotation MUST be supported without invalidating older valid bundles.

---

## 4. Identity Model

### 4.1 Canonical Terms

`bundle_id`
- The canonical portable identity inherited from the Phase-11 bundle contract.
- On-disk schemas MUST use `bundle_id` for compatibility with existing `proof_bundle` artifacts.

`bundle_hash`
- Informal verifier/UI alias for `bundle_id`.
- New on-disk schemas SHOULD prefer `bundle_id`.
- `bundle_hash` MUST NOT appear as a distinct normative identity field in on-disk schemas.
- The term `portable_bundle_hash` is not used in this specification.

`trust_overlay_hash`
- Hash over trust overlay artifacts only.
- Used for audit, receipts, and deterministic trust evaluation.
- Not part of `bundle_id`.

`policy_hash`
- Hash of the verifier's external trust policy input.

`registry_snapshot_hash`
- Hash of the producer registry snapshot used during verification.

### 4.2 Portable Identity

Portable identity is inherited from the v1 bundle contract:

`bundle_id = H(canonical_manifest_without_bundle_id || canonical_checksums)`

Portable identity includes only the portable core:
- `manifest.json`
- `checksums.json`
- `evidence/`
- `traces/`
- `reports/`
- `meta/run.json`

Portable identity MUST NOT include:
- `producer/producer.json`
- `signatures/signature-envelope.json`
- `receipts/`
- local transport metadata
- verifier-local trust policy files
- verifier-local registry files

### 4.3 Trust Overlay Identity

The trust layer is modeled as a detached overlay:

`trust_overlay_hash = H(canonical_producer_json || canonical_signature_envelope_json)`

Trust overlay artifacts are:
- `producer/producer.json`
- `signatures/signature-envelope.json`

Trust overlay MAY evolve independently from the portable core identity.

### 4.4 Deterministic Verdict Subject

Verifier outputs are bound to:

`verdict_subject = (bundle_id, trust_overlay_hash, policy_hash, registry_snapshot_hash)`

Determinism invariant:

`same bundle_id + same trust_overlay_hash + same policy_hash + same registry_snapshot_hash => same verdict`

### 4.5 Distributed Verification Context

Distributed trust interpretation uses a separate identity:

`verification_context_id`

This identity does not replace `verdict_subject`.

Design rule:
- `verdict_subject` identifies what was judged
- `verification_context_id` identifies under which distributed context that judgment may be shared

`verification_context_id` MUST NOT mutate:
- `bundle_id`
- `trust_overlay_hash`
- `verdict_subject`

For distributed transport surfaces, receipts and exchanged audit artifacts MUST eventually be interpreted together with explicit verification context binding rather than as standalone shared trust evidence.

---

## 5. Directory Layout

Proof Bundle v2 extends the existing Phase-11 bundle layout instead of replacing it.

```text
proof_bundle_v2/
├── manifest.json
├── checksums.json
├── evidence/
│   ├── abdf_snapshot_hash.txt
│   ├── bcib_plan_hash.txt
│   ├── decision_ledger.jsonl
│   ├── eti_transcript.jsonl
│   ├── execution_trace_hash.txt
│   ├── kernel.elf
│   └── replay_trace_hash.txt
├── traces/
│   ├── execution_trace.jsonl
│   └── replay_trace.jsonl
├── reports/
│   ├── proof_manifest.json
│   ├── proof_verify.json
│   ├── replay_report.json
│   ├── report.json
│   └── summary.json
├── meta/
│   └── run.json
├── producer/
│   └── producer.json
├── signatures/
│   └── signature-envelope.json
└── receipts/
    └── *.json
```

Design rule:
- The portable core keeps the Phase-11 top-level structure.
- Trust artifacts are added as detached directories.
- `receipts/` contains derived verifier outputs and is optional.

---

## 6. Canonicalization and Hash Rules

### 6.1 Required Hash Algorithm

Initial mandatory algorithm:
- `sha256`

Future signature/hash agility MAY be added, but v2 portability identity remains SHA-256 based unless explicitly versioned.

### 6.2 Canonical JSON Rules

All JSON files hashed by the verifier MUST be canonicalized using:
- RFC 8785 (JCS, JSON Canonicalization Scheme)

Operational notes:
- verifiers hash canonicalized JSON bytes, not raw source file bytes
- UTF-8 encoding is required
- lexicographic key ordering, stable numeric encoding, and whitespace normalization follow RFC 8785
- transport-added formatting differences MUST NOT change the canonical hash outcome

### 6.3 Binary Artifact Hashing

Binary artifacts MUST be hashed over raw bytes.

Examples:
- `SHA256(kernel.elf bytes)`
- `SHA256(snapshot.abdf bytes)` if raw snapshot is later bundled
- `SHA256(plan.bcib bytes)` if raw plan is later bundled

### 6.4 Directory Tree Hashing

Directory hashes MUST NOT depend on filesystem iteration order.

Canonical tree hash:

`tree_hash = H(path_1 || file_hash_1 || path_2 || file_hash_2 || ... )`

where:
- paths are relative to bundle root
- paths are sorted lexicographically
- file hashes are the canonical per-file digests

---

## 7. Portable Core Schemas

### 7.1 `manifest.json`

`manifest.json` remains the Phase-11-compatible portable manifest.

Example:

```json
{
  "bundle_id": "9117ce71bded0099e95a87e70b5721cb96a6e41bb0106bea4540c90d8f41a52f",
  "bundle_version": 2,
  "checksums_file": "checksums.json",
  "compatibility_mode": "phase11-portable-core",
  "mode": "portable_proof_bundle_v2",
  "required_files": [
    "evidence/abdf_snapshot_hash.txt",
    "evidence/bcib_plan_hash.txt",
    "evidence/execution_trace_hash.txt",
    "evidence/replay_trace_hash.txt",
    "evidence/decision_ledger.jsonl",
    "evidence/eti_transcript.jsonl",
    "evidence/kernel.elf",
    "traces/execution_trace.jsonl",
    "traces/replay_trace.jsonl",
    "reports/proof_manifest.json",
    "reports/proof_verify.json",
    "reports/report.json",
    "reports/replay_report.json",
    "reports/summary.json",
    "meta/run.json"
  ],
  "source_final_state_hash": "106836c215d8bf9f97168ae0a93f1b76ea9ced887d04f71bfd1d3c86ac6cc14c",
  "source_proof_hash": "c8443fd190ed57d3ef3d1702cb6fac2b174198ff8ecf0ac19fe46da412d90b5d",
  "source_proof_verify_status": "PASS",
  "source_report_verdict": "PASS",
  "source_summary_verdict": "PASS"
}
```

Manifest invariants:
1. `manifest.bundle_version == 2`
2. `manifest.bundle_id == recomputed_bundle_id`
3. `required_files` MUST cover all portable core artifacts required by the verifier
4. trust overlay references MUST NOT be required to recompute `bundle_id`

### 7.2 `checksums.json`

`checksums.json` remains the portable checksum authority for the core payload.

Example:

```json
{
  "algorithm": "sha256",
  "bundle_version": 2,
  "files": {
    "evidence/abdf_snapshot_hash.txt": "708e979ccc1c47cdc1359987b49ae487a84522302f0dde219e1cbc686e307ad0",
    "evidence/bcib_plan_hash.txt": "c715344757afd8ebca9ea6c5eeaa04d8f0226dc24110f4b6b57bafcadb0de1a8",
    "evidence/decision_ledger.jsonl": "1168af21a022251fb5a90849942c493bc55f0177116a40a9f75ea85eee7cb5ff"
  }
}
```

Checksum invariants:
1. every portable file MUST have exactly one checksum entry
2. missing checksum entry is fail-closed
3. checksum mismatch is fail-closed

### 7.3 `reports/proof_manifest.json`

Phase-12 MUST preserve Phase-11 naming and field semantics for core proof material.

Required field names remain:
- `abdf_snapshot_hash`
- `bcib_plan_hash`
- `execution_trace_hash`
- `ledger_root_hash`
- `transcript_root_hash`
- `replay_result_hash`
- `final_state_hash`
- `event_count`
- `violation_count`
- `proof_hash`

Phase-12 MUST NOT rename these fields to new aliases in the portable core.

---

## 8. Trust Overlay Schemas

### 8.1 `producer/producer.json`

`producer/producer.json` declares producer identity for trust evaluation.

Example:

```json
{
  "metadata_version": 1,
  "producer_id": "ayken-ci",
  "producer_pubkey_id": "ed25519-key-2026-03-a",
  "producer_registry_ref": "trust://registry/ayken-ci",
  "producer_key_epoch": "2026-03",
  "build_id": "build-fe9031d7"
}
```

Producer invariants:
1. `producer_id` MUST remain stable across key rotation
2. `producer_pubkey_id` MUST identify one concrete public key
3. `producer_key_epoch` MUST advance monotonically when a producer rotates keys
4. `producer_registry_ref` MUST resolve to a registry authority namespace, not raw key bytes
5. producer metadata MUST be canonical and hash-stable

### 8.2 `signatures/signature-envelope.json`

The signature envelope is multi-signature ready from day one.

Example:

```json
{
  "envelope_version": 1,
  "bundle_id": "9117ce71bded0099e95a87e70b5721cb96a6e41bb0106bea4540c90d8f41a52f",
  "bundle_id_algorithm": "sha256",
  "signatures": [
    {
      "signer_id": "ayken-ci",
      "producer_pubkey_id": "ed25519-key-2026-03-a",
      "signature_algorithm": "ed25519",
      "signature": "base64:....",
      "signed_at_utc": "2026-03-07T10:33:00Z"
    }
  ]
}
```

Signature envelope invariants:
1. `signature-envelope.bundle_id == manifest.bundle_id`
2. every signature entry MUST include `signer_id` and `producer_pubkey_id`
3. signature verification input is `bundle_id` only
4. detached signature bytes MUST NOT mutate `bundle_id`
5. envelope MAY contain multiple signatures
6. multi-signature acceptance semantics remain external to the envelope and MUST be defined by trust policy
7. verifier MUST reject any signature entry whose `signature_algorithm` is not present in the verifier algorithm allowlist

Verification rule:

`verify(bundle_id, signature, pubkey) == PASS`

Normative algorithm baseline:

Ed25519 is the mandatory baseline signature algorithm for Phase-12.
Additional signature algorithms MAY be introduced only through explicit versioned algorithm agility.

### 8.3 Optional `receipts/`

`receipts/` is a derived output surface for verifier nodes.

Rules:
1. receipts MUST NOT be required for portable bundle verification
2. receipts MUST NOT mutate `bundle_id`
3. receipts MAY be added after bundle sealing

---

## 9. External Trust Inputs

Trust policy and producer registry remain external verifier inputs.

### 9.1 Trust Policy Input

Example:

```json
{
  "policy_version": 1,
  "policy_hash": "f0f1...aa",
  "quorum_policy_ref": "policy://quorum/at-least-1-of-n",
  "trusted_producers": [
    "ayken-ci",
    "ayken-core"
  ],
  "trusted_pubkey_ids": [
    "ed25519-key-2026-03-a"
  ],
  "required_signatures": {
    "type": "at_least",
    "count": 1
  },
  "revoked_pubkey_ids": []
}
```

Policy invariants:
1. policy MUST be canonical and hashable
2. policy MUST be external to the bundle
3. `policy_hash` MUST bind the final verdict
4. revoked key => deterministic reject
5. when multi-signature acceptance is enabled, quorum semantics MUST be explicit via `quorum_policy_ref` or an equivalent canonical in-policy structure

### 9.2 Producer Registry Snapshot

Verifier MUST resolve `producer_pubkey_id` through a concrete registry snapshot.

Minimum registry snapshot fields:
- `registry_format_version`
- `registry_version`
- `registry_snapshot_hash`
- mapping from `producer_id` to active and historical `producer_pubkey_id`
- concrete public key material for each resolvable `producer_pubkey_id`
- key status (`active`, `revoked`, `superseded`)

Registry invariants:
1. `registry_snapshot_hash` MUST be recorded in verification receipts
2. verifier MUST recompute canonical `registry_snapshot_hash` from registry snapshot content, excluding the declared `registry_snapshot_hash` field itself
3. recomputed registry hash MUST equal declared `registry_snapshot_hash` or verification MUST fail closed
4. the same registry snapshot MUST yield the same producer resolution results
5. unresolved or ambiguous key resolution is fail-closed

---

## 10. Verification Pipeline

Canonical verifier pipeline:

1. load portable bundle
2. validate `bundle_version` and schema versions
3. recompute per-file checksums for all portable files
4. recompute `bundle_id`
5. compare recomputed `bundle_id` with `manifest.bundle_id`
6. verify the Phase-11 proof chain from bundled evidence
7. load `producer/producer.json`
8. load `signatures/signature-envelope.json`
9. recompute `trust_overlay_hash`
10. recompute and validate canonical `registry_snapshot_hash`
11. resolve `producer_pubkey_id` through the selected registry snapshot to concrete public key material
12. verify detached signatures over `bundle_id`
13. evaluate trust policy
14. emit verdict and receipt

Design rule:
- signature verification and policy evaluation are separate stages
- valid signature does not imply acceptance

---

## 11. Verdicts

Minimum verdict set:
- `TRUSTED`
- `UNTRUSTED`
- `INVALID`
- `REJECTED_BY_POLICY`

Interpretation:
- `INVALID`: structural, checksum, proof-chain, or signature verification failure
- `UNTRUSTED`: proof valid but signer/producer not trusted
- `REJECTED_BY_POLICY`: proof valid and signer resolvable, but policy does not accept it
- `TRUSTED`: all required checks pass

---

## 12. Verification Receipt Schema

Example:

```json
{
  "receipt_version": 1,
  "bundle_id": "9117ce71bded0099e95a87e70b5721cb96a6e41bb0106bea4540c90d8f41a52f",
  "trust_overlay_hash": "a3d7...ff",
  "policy_hash": "f0f1...aa",
  "registry_snapshot_hash": "c1c2...99",
  "verifier_node_id": "node-b",
  "verifier_key_id": "receipt-ed25519-key-2026-03-a",
  "verdict": "TRUSTED",
  "verified_at_utc": "2026-03-07T10:36:00Z",
  "verifier_signature_algorithm": "ed25519",
  "verifier_signature": "base64:...."
}
```

Receipt invariants:
1. receipt MUST include `bundle_id`
2. receipt MUST include `policy_hash`
3. receipt MUST include `registry_snapshot_hash`
4. receipt MUST include `trust_overlay_hash`
5. signed receipt payload MUST bind verifier identity through `verifier_node_id` and `verifier_key_id`
6. signed receipt signature input MUST be the canonicalized receipt payload without detached signature fields
7. signed receipt verification MUST fail closed on payload subject mismatch or detached signature mismatch
8. receipt is a derived artifact and MUST NOT mutate `bundle_id`

This schema enables future receipt chains without contaminating the portable bundle identity.

---

## 13. Transport Rules

Transport layers MUST NOT mutate:
- portable payload bytes
- `manifest.json`
- `checksums.json`
- files under `evidence/`
- files under `traces/`
- files under `reports/`
- files under `meta/`
- `manifest.bundle_id`
- bundled Phase-11 proof artifacts

Transport layers MAY add:
- receipts
- cache metadata outside the portable core
- transport-local metadata outside the portable core

Invariant:

`transport MUST NOT mutate portable payload identity`

---

## 14. Fail-Closed Rules

The verifier MUST reject on any of the following:
- manifest checksum mismatch
- recomputed `bundle_id` mismatch
- proof-chain mismatch
- missing `producer/producer.json`
- missing `signatures/signature-envelope.json`
- `trust_overlay_hash` mismatch against any expected or receipt-bound overlay identity
- unresolved `producer_pubkey_id`
- revoked key
- invalid signature
- unsupported schema version
- policy mismatch
- ambiguous multi-signature quorum evaluation

Trust-critical failure rule:

`any trust-critical verification failure => fail closed`

---

## 15. Forward Compatibility

### 15.1 Phase-11 Compatibility Rule

Phase-12 trust metadata SHALL extend Phase-11 portability without changing Phase-11 bundle identity semantics.

### 15.2 Unknown Field Handling

Verifiers MAY ignore unknown non-identity metadata fields if:
- canonical JSON remains valid
- required fields remain present
- identity-affecting rules remain intact

### 15.3 Reserved Future Fields

Reserved future additions:
- `signature_agility`
- `quorum_policy_ref`
- `receipt_chain_ref`
- `trust_epoch`
- `producer_attestation_ref`

---

## 16. Closure Criteria

### 16.1 Phase-12A Closure

Required:
1. producer identity schema defined
2. detached signature envelope implemented
3. `bundle_id` unchanged by detached signatures
4. offline verification passes on another machine

### 16.2 Phase-12B Closure

Required:
1. policy is hash-bound
2. `same bundle + same policy + same registry snapshot => same verdict`
3. verifier crate and CLI are operational

### 16.3 Phase-12C Closure

Required:
1. cross-node parity suite passes
2. `proofd` verification service operates in userspace
3. receipts are generated and auditable
4. replay boundary remains explicit and controlled

---

## 17. Design Summary

This specification preserves the core AykenOS ladder:

- Phase-11: proof exists and travels
- Phase-12: proof is attributed, signed, and policy-verifiable
- Phase-13+: proof may be accepted and reused across nodes under stronger distributed protocols

The key separation remains:

`portable core identity != trust overlay artifacts`

If that separation is preserved:
- Phase-11 does not break
- Phase-12 can harden trust safely
- later distributed verification layers remain composable

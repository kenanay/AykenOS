# Requirements Document: Phase-12 Trust Layer

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-07
**Created by:** Kenan AY
**Maintained by:** Kenan AY
**Last Edited by:** Kenan AY
**Prerequisites:**
- Phase-11 `proof_bundle` portability contract (`P11-42`)
- `PROOF_BUNDLE_V2_SPEC.md`
- `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`
- `PROOF_BUNDLE_ATTACK_SURFACE_SECURITY_MODEL.md`
- `VERIFICATION_CONTEXT_DISTRIBUTION_CONTRACT.md`
- `VERIFICATION_CONTEXT_OBJECT_SPEC.md`
- `VERIFICATION_CONTEXT_PORTABILITY_AND_DISTRIBUTION_PROTOCOL.md`
- `VERIFIER_ATTESTATION_AND_TRUST_REGISTRY_CONTRACT.md`
- `CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`
- `VERIFIER_AUTHORITY_SEMANTICS_AND_DELEGATION_CONTRACT.md`
- `VERIFIER_REGISTRY_LINEAGE_AND_DISTRIBUTION_MODEL.md`
- `VERIFIER_AUTHORITY_GRAPH_CONSTRAINTS.md`
- `VERIFIER_AUTHORITY_RESOLUTION_ALGORITHM.md`

---

## Introduction

Phase-12 implements the **trust layer** for AykenOS proof portability.

It extends the Phase-11 portable proof bundle into a deterministic verification system with:
- producer attribution
- detached signatures
- registry-bound key resolution
- policy-bound acceptance
- verification receipts
- cross-node verification parity

Phase-12 transforms AykenOS from:

`portable proof`

into:

`trusted proof`

without changing the Phase-11 portable identity contract.

This requirements document defines:
- normative acceptance criteria
- normative verdict semantics
- normative CI gate expectations
- normative phase closure conditions

Individual milestones are tracked as P12-01 through P12-18.

---

## Glossary

### Core Concepts

- **Portable Core**: Phase-11-compatible proof bundle material that determines `bundle_id`
- **Trust Overlay**: Detached producer and signature artifacts that determine `trust_overlay_hash`
- **Verifier**: Userspace/offline engine that evaluates portable proof validity and trust acceptance
- **Registry Snapshot**: Explicit producer-key resolution input used during verification
- **Trust Policy**: External acceptance rules applied after proof and signature validity checks
- **Receipt**: Derived verification artifact emitted after verdict generation
- **Verification Context**: Distributed interpretation object that binds policy, registry, and verifier contract semantics for cross-node trust reuse
- **Verification Context Object**: Canonical hashable object that materializes distributed verification context for transport or parity comparison
- **Verifier Attestation**: Detached artifact that binds verifier identity, verifier key, and verifier contract semantics for distributed trust reuse
- **Verifier Trust Registry**: External trust registry that determines which verifier identities may act as distributed trust speakers
- **Parity Status**: Cross-node comparison classification distinct from local verifier verdicts
- **Verifier Authority Scope**: Explicit declaration of what a verifier is allowed to do as a distributed trust speaker
- **Verifier Registry Lineage**: Snapshot hash, parent hash, epoch, and scope semantics that determine verifier trust continuity across nodes
- **Verifier Authority Graph**: Directed delegation graph that constrains how verifier authority may propagate
- **Verifier Authority Resolution**: Deterministic procedure that resolves a verifier to one effective current or historical authority chain

### Identity Terms

- **bundle_id**: Canonical portable identity inherited from Phase-11
- **bundle_hash**: Informal UI alias for `bundle_id`
- **trust_overlay_hash**: Canonical hash of `producer/producer.json` and `signatures/signature-envelope.json`
- **policy_hash**: Canonical hash of verifier-local trust policy input
- **registry_snapshot_hash**: Canonical or declared hash of the verifier-local registry snapshot
- **verdict_subject**: `(bundle_id, trust_overlay_hash, policy_hash, registry_snapshot_hash)`
- **verification_context_id**: Canonical distributed context identity separate from `verdict_subject`

### Verdicts

- **INVALID**: Structural, integrity, proof, signature, or ambiguity failure
- **UNTRUSTED**: Proof valid, but producer or signer does not satisfy trust set
- **REJECTED_BY_POLICY**: Proof valid and signer resolvable, but explicit policy acceptance conditions are not met
- **TRUSTED**: Proof valid, signer valid, registry resolution valid, and policy accepts
- **Historical Only**: Non-verdict interpretation state for receipts that remain audit-valid but are not valid as current distributed acceptance evidence

---

## Requirements

### Requirement 1: Proof Bundle v2 Identity and Layout (P12-03)

**User Story:** As a verifier architect, I want a proof bundle layout that preserves Phase-11 identity semantics, so that trust metadata can evolve without breaking portability.

#### Acceptance Criteria

1.1. THE System SHALL preserve the Phase-11 portable core layout: `manifest.json`, `checksums.json`, `evidence/`, `traces/`, `reports/`, `meta/run.json`
1.2. THE System SHALL extend the bundle with detached trust overlay directories: `producer/`, `signatures/`, and optional derived `receipts/`
1.3. THE System SHALL define portable identity as `bundle_id = H(canonical_manifest_without_bundle_id || canonical_checksums)`
1.4. THE System SHALL define trust overlay identity as `trust_overlay_hash = H(JCS(producer/producer.json) || JCS(signatures/signature-envelope.json))`
1.5. THE System SHALL use `bundle_id` as the only normative portable identity term in on-disk schemas
1.6. THE portable identity SHALL NOT include `producer/producer.json`, `signatures/signature-envelope.json`, `receipts/`, verifier-local policy files, verifier-local registry files, or transport-local metadata
1.7. THE System SHALL canonicalize verifier-hashed JSON according to RFC 8785 (JCS) semantics
1.8. THE System SHALL implement `ci-gate-proof-bundle-v2-schema`
1.9. THE System SHALL implement `ci-gate-proof-bundle-v2-compat`
1.10. THE schema gate SHALL export `bundle_schema_report.json`, `report.json`, and `violations.txt` under `evidence/run-*/gates/proof-bundle-v2-schema/`
1.11. THE compatibility gate SHALL export `compatibility_report.json`, `report.json`, and `violations.txt` under `evidence/run-*/gates/proof-bundle-v2-compat/`
1.12. THE compatibility gate SHALL fail if a valid Phase-11 portable bundle cannot be interpreted as a trustless portable bundle in compatibility mode

---

### Requirement 2: Producer Identity Schema (P12-01)

**User Story:** As a verifier architect, I want canonical producer identity metadata, so that trust attribution remains stable across key rotations.

#### Acceptance Criteria

2.1. THE System SHALL define `producer/producer.json` as canonical producer declaration
2.2. THE producer declaration SHALL include at least: `metadata_version`, `producer_id`, `producer_pubkey_id`, `producer_registry_ref`, `producer_key_epoch`
2.3. THE `producer_id` SHALL remain stable across key rotations
2.4. THE `producer_pubkey_id` SHALL identify one concrete public key
2.5. THE `producer_registry_ref` SHALL reference a registry or trust-root namespace, not raw key bytes
2.6. THE `producer_key_epoch` SHALL support deterministic key rotation tracking
2.7. THE producer declaration SHALL be canonical and hash-stable
2.8. THE producer declaration SHALL NOT mutate `bundle_id`
2.9. THE System SHALL implement `ci-gate-proof-producer-schema`
2.10. THE producer schema gate SHALL export `producer_schema_report.json`, `producer_identity_examples.json`, `report.json`, and `violations.txt` under `evidence/run-*/gates/proof-producer-schema/`

---

### Requirement 3: Detached Signature Envelope and Signature Verification (P12-02, P12-04)

**User Story:** As a verifier architect, I want detached signatures bound to portable proof identity, so that trust can be added without mutating portability.

#### Acceptance Criteria

3.1. THE System SHALL define `signatures/signature-envelope.json` as detached signature container
3.2. THE signature envelope SHALL include at least: `envelope_version`, `bundle_id`, `bundle_id_algorithm`, and `signatures[]`
3.3. EACH signature entry SHALL include at least: `signer_id`, `producer_pubkey_id`, `signature_algorithm`, `signature`, `signed_at_utc`
3.4. THE signature verification input SHALL be `bundle_id` only
3.5. THE detached signature envelope SHALL be multi-signature ready from initial release
3.6. Multi-signature storage SHALL remain in the envelope, but acceptance semantics SHALL remain external to trust policy
3.7. Missing, malformed, or inconsistent signature metadata SHALL fail closed
3.8. Structurally present but cryptographically unverified signature data SHALL NOT be sufficient for `TRUSTED`
3.9. THE System SHALL implement `ci-gate-proof-signature-envelope`
3.10. THE System SHALL implement `ci-gate-proof-signature-verify`
3.11. THE signature envelope gate SHALL export `signature_envelope_report.json`, `identity_stability_report.json`, `report.json`, and `violations.txt` under `evidence/run-*/gates/proof-signature-envelope/`
3.12. THE signature verification gate SHALL export `signature_verify.json`, `registry_resolution_report.json`, `report.json`, and `violations.txt` under `evidence/run-*/gates/proof-signature-verify/`
3.13. WHEN detached signature verification is active, THE System SHALL verify signatures using an allowlisted algorithm set
3.14. Initial mandatory detached signature algorithm SHALL be Ed25519 unless explicitly versioned otherwise
3.15. Ed25519 SHALL remain the mandatory baseline signature algorithm for Phase-12, and additional algorithms SHALL require explicit versioned algorithm agility before acceptance
3.16. UNTIL detached signature cryptography is active for the selected allowlisted algorithm, THE verifier MAY operate in fail-closed bootstrap mode where bundles requiring trust semantics yield `INVALID` rather than `TRUSTED`

---

### Requirement 4: Registry Snapshot and Key Lifecycle (P12-05, P12-06)

**User Story:** As a verifier architect, I want deterministic registry resolution and auditable key lifecycle handling, so that signer trust cannot be confused or silently downgraded.

#### Acceptance Criteria

4.1. THE System SHALL define a registry snapshot format with at least: `registry_format_version`, `registry_version`, `registry_snapshot_hash`, producer-to-key mappings, and concrete public key material for resolvable keys
4.2. THE registry snapshot SHALL represent key state using at least: `active`, `revoked`, `superseded`
4.3. THE verifier SHALL resolve `producer_pubkey_id` through an explicit registry snapshot to a concrete public key and key state
4.4. Unresolved key resolution SHALL fail closed
4.5. Ambiguous key ownership SHALL fail closed
4.6. Revoked key resolution SHALL produce `INVALID`
4.7. Key rotation SHALL preserve auditability of previously valid bundles when verified against the applicable registry snapshot
4.8. THE System SHALL implement `ci-gate-proof-registry-resolution`
4.9. THE System SHALL implement `ci-gate-proof-key-rotation`
4.10. THE registry resolution gate SHALL export `registry_snapshot.json`, `registry_resolution_matrix.json`, `report.json`, and `violations.txt` under `evidence/run-*/gates/proof-registry-resolution/`
4.11. THE key rotation gate SHALL export `rotation_matrix.json`, `revocation_matrix.json`, `report.json`, and `violations.txt` under `evidence/run-*/gates/proof-key-rotation/`
4.12. THE verifier SHALL recompute the canonical `registry_snapshot_hash` from the verifier-local registry snapshot using deterministic JSON canonicalization and SHA-256
4.13. THE recomputed hash SHALL equal the declared `registry_snapshot_hash` or verification SHALL fail closed
4.14. THE verifier SHALL bind verdict subject and receipt output to the canonical recomputed `registry_snapshot_hash`

---

### Requirement 5: `proof-verifier` Core Pipeline (P12-07)

**User Story:** As a verifier architect, I want a deterministic userspace verifier engine, so that trusted proof evaluation is reproducible and independent from kernel runtime.

#### Acceptance Criteria

5.1. THE System SHALL implement a Rust verifier crate at `ayken-core/crates/proof-verifier/`
5.2. THE verifier SHALL be library-first and userspace/offline
5.3. THE verifier SHALL expose a core API that consumes a bundle path, trust policy input, registry snapshot input, and receipt mode input
5.4. THE verifier pipeline SHALL execute in the following logical order: bundle load, layout validation, portable checksum validation, portable proof validation, `bundle_id` recomputation, overlay validation, signer resolution, detached signature verification, policy evaluation, verdict derivation, receipt emission
5.5. Signature validity SHALL remain logically separate from policy acceptance
5.6. Proof validity SHALL be evaluated before trust acceptance
5.7. THE verifier SHALL remain outside Ring0
5.8. THE verifier SHALL implement deterministic verdict behavior: same inputs SHALL yield the same verdict
5.9. THE System SHALL implement `ci-gate-proof-verifier-core`
5.10. THE verifier core gate SHALL export `verifier_core_report.json`, `determinism_matrix.json`, `report.json`, and `violations.txt` under `evidence/run-*/gates/proof-verifier-core/`
5.11. Malformed bundle structure, checksum drift, proof inconsistency, or trust-critical ambiguity SHALL fail closed

---

### Requirement 6: Trust Policy Schema and Quorum Acceptance (P12-08, P12-15)

**User Story:** As a verifier architect, I want explicit and hashable trust policy semantics, so that acceptance decisions are deterministic and reviewable.

#### Acceptance Criteria

6.1. THE trust policy SHALL remain external to the bundle
6.2. THE trust policy SHALL be canonical and hashable
6.3. THE trust policy SHALL include enough structure to express: trusted producers, trusted key IDs, required signatures, revoked key IDs, and explicit quorum policy
6.4. THE trust policy SHALL produce `policy_hash` that binds the final verdict
6.5. IF proof validity succeeds but producer or signer is outside the trust set, THE verdict SHALL be `UNTRUSTED`
6.6. IF proof validity succeeds and signer resolution succeeds, but explicit policy acceptance conditions are not met, THE verdict SHALL be `REJECTED_BY_POLICY`
6.7. IF proof validity, signature validity, registry resolution, and policy acceptance all succeed, THE verdict SHALL be `TRUSTED`
6.8. Ambiguous quorum evaluation SHALL fail closed and SHALL NOT produce `TRUSTED`
6.9. THE System SHALL implement `ci-gate-proof-trust-policy`
6.10. THE System SHALL implement `ci-gate-proof-multisig-quorum`
6.11. THE trust policy gate SHALL export `policy_schema_report.json`, `policy_hash_report.json`, `report.json`, and `violations.txt` under `evidence/run-*/gates/proof-trust-policy/`
6.12. THE multi-signature gate SHALL export `quorum_matrix.json`, `quorum_evaluator_report.json`, `report.json`, and `violations.txt` under `evidence/run-*/gates/proof-multisig-quorum/`

---

### Requirement 7: Verdict Binding and Output Contract (P12-09)

**User Story:** As a verifier architect, I want every trust verdict bound to explicit input identities, so that cross-node claims are deterministic and auditable.

#### Acceptance Criteria

7.1. THE final verdict subject SHALL be `verdict_subject = (bundle_id, trust_overlay_hash, policy_hash, registry_snapshot_hash)`
7.2. No weaker tuple SHALL be accepted for distributed verification claims
7.3. THE verifier SHALL include `bundle_id`, `trust_overlay_hash`, `policy_hash`, and `registry_snapshot_hash` in machine-readable verdict output
7.4. THE verifier SHALL include the same binding fields in emitted receipts
7.5. THE System SHALL implement `ci-gate-proof-verdict-binding`
7.6. THE verdict binding gate SHALL export `verdict_binding_report.json`, `verdict_subject_examples.json`, `report.json`, and `violations.txt` under `evidence/run-*/gates/proof-verdict-binding/`
7.7. SAME `bundle_id` + SAME `trust_overlay_hash` + SAME `policy_hash` + SAME `registry_snapshot_hash` SHALL yield the SAME final verdict

---

### Requirement 8: Offline CLI Surface (P12-10)

**User Story:** As an operator, I want an offline CLI verification surface, so that trusted proof can be verified without service infrastructure.

#### Acceptance Criteria

8.1. THE System SHALL expose an offline CLI for proof bundle verification
8.2. THE CLI SHALL accept a bundle input plus external policy and registry inputs
8.3. THE CLI SHALL produce human-readable verdict output
8.4. THE CLI SHALL produce machine-readable JSON output
8.5. THE CLI SHALL report the final verdict and verdict subject binding fields
8.6. THE CLI SHALL remain a thin wrapper over verifier core semantics
8.7. THE System SHALL implement `ci-gate-proof-verifier-cli`
8.8. THE CLI gate SHALL export `cli_smoke_report.json`, `cli_output_contract.json`, `report.json`, and `violations.txt` under `evidence/run-*/gates/proof-verifier-cli/`

---

### Requirement 9: Verification Receipts and Audit Artifacts (P12-11, P12-12)

**User Story:** As a verifier architect, I want derived verification artifacts, so that acceptance can be audited without contaminating portable identity.

#### Acceptance Criteria

9.1. THE System SHALL define a verification receipt schema that includes at least: `bundle_id`, `trust_overlay_hash`, `policy_hash`, `registry_snapshot_hash`, `verifier_node_id`, `verdict`, `verified_at_utc`
9.2. Signed receipt payloads SHALL additionally bind verifier key identity via `verifier_key_id` or an equivalent canonical field
9.3. Receipt data SHALL be derived artifact data and SHALL NOT mutate `bundle_id`
9.4. THE System MAY emit unsigned receipts during bootstrap implementation stages
9.5. Unsigned receipts SHALL NOT be represented as trust-complete cross-node trust anchors
9.6. THE System SHALL canonicalize signed receipt payloads deterministically before signature generation and verification
9.7. THE initial mandatory signed receipt algorithm SHALL be Ed25519 unless explicitly versioned otherwise
9.8. THE verifier SHALL reject signed receipts whose detached signature does not verify against the canonical receipt payload and verifier public key
9.9. THE verifier SHALL reject signed receipts whose payload subject does not match the recomputed `verdict_subject`
9.10. THE System SHALL define an append-only audit event format for verification actions
9.11. THE audit event format SHALL include at least: `event_version`, `event_type`, `event_id`, `event_time_utc`, `verifier_node_id`, `bundle_id`, `trust_overlay_hash`, `policy_hash`, `registry_snapshot_hash`, `verdict`, `receipt_hash`, `previous_event_hash`
9.12. THE audit event format SHALL record who verified what, under which policy and registry snapshot, with which verdict
9.13. THE System SHALL compute `event_id` as a canonical hash over the audit event excluding the detached `event_id` field itself
9.14. THE System SHALL compute `receipt_hash` from canonical receipt bytes and bind it into the audit event
9.15. THE verifier SHALL reject audit ledgers with `previous_event_hash` chain drift or recomputed `event_id` mismatch
9.16. THE verifier SHALL reject audit event bindings whose associated signed receipt does not pass canonical receipt signature verification
9.17. Audit append operations SHALL be serialized so concurrent appends cannot fork the `previous_event_hash` chain
9.18. THE System SHALL implement `ci-gate-proof-receipt`
9.19. THE System SHALL implement `ci-gate-proof-audit-ledger`
9.20. THE receipt gate SHALL export `receipt_schema_report.json`, `receipt_emit_report.json`, `report.json`, and `violations.txt` under `evidence/run-*/gates/proof-receipt/`
9.21. THE audit ledger gate SHALL export `verification_audit_ledger.jsonl`, `audit_integrity_report.json`, `report.json`, and `violations.txt` under `evidence/run-*/gates/proof-audit-ledger/`

---

### Requirement 10: Bundle Exchange, Cross-Node Parity, and `proofd` Service (P12-13, P12-14, P12-16)

**User Story:** As a distributed verifier architect, I want portable trust evaluation across nodes, so that proof can be accepted consistently beyond the producer machine.

#### Acceptance Criteria

10.1. THE transport layer SHALL NOT mutate portable payload identity
10.2. THE transport layer MAY carry overlay and receipt artifacts separately from portable core payload
10.3. SAME bundle input + SAME policy input + SAME registry snapshot SHALL yield SAME final verdict across nodes
10.4. THE System SHALL implement cross-node verification parity testing
10.5. THE System SHALL implement a userspace verification service at `userspace/proofd/`
10.6. `proofd` SHALL perform verification, policy application, and receipt emission in userspace
10.7. `proofd` SHALL NOT move trust evaluation into Ring0
10.8. THE System SHALL implement `ci-gate-proof-exchange`
10.9. THE System SHALL implement `ci-gate-cross-node-parity`
10.10. THE System SHALL implement `ci-gate-proofd-service`
10.11. THE bundle exchange gate SHALL export `exchange_contract_report.json`, `transport_mutation_matrix.json`, `report.json`, and `violations.txt` under `evidence/run-*/gates/proof-exchange/`
10.12. THE parity gate SHALL export `parity_report.json`, `failure_matrix.json`, `report.json`, and `violations.txt` under `evidence/run-*/gates/cross-node-parity/`
10.13. THE `proofd` gate SHALL export `proofd_service_report.json`, `proofd_receipt_report.json`, `report.json`, and `violations.txt` under `evidence/run-*/gates/proofd-service/`
10.14. THE System SHALL define `verification_context_id` as a distributed context identity distinct from `verdict_subject`
10.15. THE System SHALL compute `verification_context_id` from at least: `policy_hash`, `registry_snapshot_hash`, `verifier_contract_version`, and `context_rules_hash`
10.16. A receipt SHALL NOT be treated as shared distributed trust evidence unless its verification context is explicitly present, hash-bound, and equal to the verifier-local acceptance context
10.17. Context mismatch SHALL fail closed for distributed acceptance and SHALL NOT degrade to warning-only behavior
10.18. Context mismatch SHALL NOT be re-labeled as `UNTRUSTED`
10.19. Historical receipts MAY be retained as audit artifacts but SHALL NOT be treated as current distributed acceptance proof
10.20. Cross-node parity claims SHALL require equal `verification_context_id` in addition to equal `verdict_subject`
10.21. Bundle exchange and `proofd` transport surfaces SHALL carry explicit verification context binding or a content-addressed equivalent
10.22. THE System SHALL define a canonical verification context object schema for distributed transport and parity use
10.23. THE verifier SHALL reject context objects whose declared and recomputed `verification_context_id` differ
10.24. THE System SHALL define `context_rules_hash` over an explicit canonical context-rules object
10.24a. THE System SHALL define a verification context portability protocol distinct from proof transport and receipt transport
10.24b. Distributed context transport SHALL carry either inline canonical context material or content-addressed references sufficient to reconstruct the sender acceptance context
10.24c. THE verifier SHALL reject distributed trust claims whose transported policy, registry, or context-rules material cannot be resolved and recomputed to the declared context identities
10.24d. THE verifier SHALL NOT silently substitute local default policy, registry, or context-rules material for a claimed distributed context package
10.24e. Portable context transport SHALL preserve the distinction between proof artifact, context artifact, and verifier-trust artifact
10.25. THE System SHALL define separate verifier-trust semantics for distributed receipt reuse
10.26. A signed receipt SHALL NOT be treated as shared distributed trust evidence unless its signer verifier is trusted under an explicit verifier trust registry
10.27. THE System SHALL preserve the distinction: `trusted proof != trusted verifier`
10.28. Cross-node distributed acceptance claims SHALL require equal trusted verifier semantics in addition to equal `verdict_subject` and equal `verification_context_id`
10.29. THE System SHALL define cross-node parity failure semantics distinct from local verifier verdicts
10.30. Context mismatch SHALL classify as a parity failure state, not as `UNTRUSTED`
10.31. Historical-only distributed interpretation SHALL classify as parity historical-only state, not as current distributed acceptance
10.32. Missing parity artifacts SHALL fail closed as insufficient parity evidence
10.33. THE System SHALL define verifier authority semantics distinct from receipt signature validity
10.34. Shared distributed receipt acceptance SHALL require trusted verifier authority scope in addition to trusted verifier identity
10.35. Delegation of verifier authority SHALL default to deny unless explicitly declared
10.36. Ambiguous verifier identity or authority mapping SHALL fail closed
10.37. THE System SHALL define verifier registry lineage and distribution semantics for distributed verifier trust interpretation
10.38. THE System SHALL define `verifier_registry_snapshot_hash`, `verifier_registry_parent_hash`, and `verifier_registry_epoch` as a coherent verifier registry lineage surface
10.39. Same-scope same-epoch different-hash verifier registry snapshots SHALL be treated as lineage fork, not benign variation
10.40. The verifier SHALL NOT silently downgrade current distributed verifier authority to an older conflicting lineage snapshot
10.41. THE System SHALL constrain verifier delegation as an acyclic authority graph
10.42. Self-delegation and indirect authority cycles SHALL fail closed
10.43. Delegated authority scope SHALL only narrow, never widen
10.44. Delegation depth SHALL be bounded and overflow SHALL fail closed
10.45. THE System SHALL define a deterministic verifier authority resolution algorithm
10.46. A delegated verifier SHALL resolve to exactly one effective parent chain after filtering, or authority resolution SHALL fail closed
10.47. Silent parent-chain tie-breakers SHALL NOT be used unless explicitly versioned in a future contract
10.48. THE verifier trust registry SHALL declare current root verifier authorities explicitly
10.49. Nodes with no delegated parent SHALL NOT be treated as current root authority unless explicitly listed in the verifier trust registry root set
10.50. Current delegated authority SHALL have at most one surviving effective parent edge after normalization and historical filtering, or resolution SHALL fail closed
10.51. Successful delegated authority resolution SHALL expose canonical `authority_chain_id` for parity and audit comparison
10.52. Cross-node parity for delegated verifier authority SHALL require equal `authority_chain_id` when verifier trust semantics claim current delegated authority

---

### Requirement 11: Replay Admission Boundary and Research Scope (P12-17, P12-18)

**User Story:** As a kernel architect, I want trusted proof acceptance separated from replay authorization, so that Phase-12 does not silently become a replay-execution policy layer.

#### Acceptance Criteria

11.1. Accepted proof SHALL NOT imply automatic replay admission
11.2. Replay admission SHALL require an explicit, separate contract
11.3. Replicated verification or replay research SHALL remain outside Phase-12 closure criteria unless separately ratified
11.4. THE System SHALL implement `ci-gate-proof-replay-admission-boundary`
11.5. THE System SHALL implement `ci-gate-proof-replicated-verification-boundary`
11.6. THE replay boundary gate SHALL export `replay_admission_report.json`, `boundary_contract.json`, `report.json`, and `violations.txt` under `evidence/run-*/gates/proof-replay-admission-boundary/`
11.7. THE research boundary gate SHALL export `research_boundary_note.md`, `phase13_bridge_report.json`, `report.json`, and `violations.txt` under `evidence/run-*/gates/proof-replicated-verification-boundary/`

---

### Requirement 12: CI Gate Integration

**User Story:** As a release architect, I want explicit normative gate sets for each closure level, so that Phase-12 completion is objective and reproducible.

#### Acceptance Criteria

12.1. THE normative Phase-12A gate set SHALL include: `ci-gate-proof-producer-schema`, `ci-gate-proof-signature-envelope`, `ci-gate-proof-bundle-v2-schema`, `ci-gate-proof-bundle-v2-compat`, `ci-gate-proof-signature-verify`, `ci-gate-proof-registry-resolution`, `ci-gate-proof-key-rotation`
12.2. THE normative Phase-12B gate set SHALL include: `ci-gate-proof-verifier-core`, `ci-gate-proof-trust-policy`, `ci-gate-proof-verdict-binding`, `ci-gate-proof-verifier-cli`, `ci-gate-proof-receipt`, `ci-gate-proof-audit-ledger`
12.3. THE normative Phase-12C gate set SHALL include: `ci-gate-proof-exchange`, `ci-gate-cross-node-parity`, `ci-gate-proof-multisig-quorum`, `ci-gate-proofd-service`, `ci-gate-proof-replay-admission-boundary`, `ci-gate-proof-replicated-verification-boundary`
12.4. WHEN an invariant mapped to a normative gate is violated, THE corresponding gate SHALL fail
12.5. WHEN a required normative gate is missing, THE associated closure level SHALL NOT be considered complete

---

### Requirement 13: Security and Performance Verification

**User Story:** As a release architect, I want every Phase-12 change to carry explicit security and performance checks, so that trust hardening does not regress safety or determinism.

#### Acceptance Criteria

13.1. WHEN a Phase-12 PR is prepared, THE PR SHALL include a security check summary
13.2. WHEN a Phase-12 PR is prepared, THE PR SHALL include a performance check summary
13.3. Malformed or tampered bundle inputs SHALL fail closed
13.4. No Phase-12 change SHALL leak trust policy logic into Ring0
13.5. Heavy verification operations SHALL remain userspace/offline unless separately ratified
13.6. Threat-model-impacting changes SHALL update `PROOF_BUNDLE_ATTACK_SURFACE_SECURITY_MODEL.md` in the same PR
13.7. Core API, module-boundary, or verifier pipeline changes SHALL update `PROOF_VERIFIER_CRATE_ARCHITECTURE.md` in the same PR

---

### Requirement 14: Constitutional and Architectural Compliance

**User Story:** As an architecture board reviewer, I want Phase-12 to preserve AykenOS constitutional boundaries, so that trust verification does not collapse mechanism and policy separation.

#### Acceptance Criteria

14.1. THE Phase-12 layer SHALL NOT move trust verification into Ring0
14.2. THE Phase-12 layer SHALL NOT redefine Phase-11 portable identity semantics
14.3. THE Phase-12 layer SHALL NOT treat receipts as portable identity
14.4. THE Phase-12 layer SHALL NOT silently import trust policy from inside the bundle
14.5. THE Phase-12 layer SHALL preserve deterministic verdict behavior
14.6. THE Phase-12 layer SHALL preserve the distinction: `valid proof != trusted proof`
14.7. THE Phase-12 layer SHALL preserve the distinction: `trusted proof != replay admission`

---

### Requirement 15: Backward Compatibility

**User Story:** As a verifier architect, I want backward compatibility with Phase-11 portability, so that older portable proofs remain useful when trust semantics are introduced.

#### Acceptance Criteria

15.1. A valid Phase-11 portable bundle SHALL remain interpretable as a trustless portable bundle
15.2. Incompatible bundle schema changes SHALL require version increment
15.3. Incompatible signature envelope schema changes SHALL require version increment
15.4. Unknown non-identity metadata fields MAY be ignored if required fields remain present and identity rules remain intact
15.5. Forward extension of trust metadata SHALL NOT mutate `bundle_id`

---

### Requirement 15A: Documentation Synchronization

**User Story:** As a release architect, I want all Phase-12 documentation updated with implementation changes, so that acceptance, architecture, and security never drift.

#### Acceptance Criteria

15A.1. WHEN task status changes, THE PR SHALL update `tasks.md`
15A.2. WHEN architecture or verifier boundary behavior changes, THE PR SHALL update `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`
15A.3. WHEN identity or schema semantics change, THE PR SHALL update `PROOF_BUNDLE_V2_SPEC.md`
15A.4. WHEN acceptance criteria or gate norms change, THE PR SHALL update `requirements.md`
15A.5. WHEN security posture or attack model changes, THE PR SHALL update `PROOF_BUNDLE_ATTACK_SURFACE_SECURITY_MODEL.md`
15A.6. THE PR description SHALL include a `Documentation Delta` section

---

## Out of Scope (Phase 13+)

The following are explicitly OUT OF SCOPE for Phase-12:

- remote registry distribution
- transport encryption
- distributed consensus
- replay execution admission
- kernel-side trust enforcement
- replicated execution
- proof consensus across nodes
- hardware root of trust

---

## Success Criteria

Phase-12A is considered closure-ready when:
- producer identity schema is complete
- detached signature envelope is complete
- bundle v2 identity and compatibility gates pass
- signature verification gate passes
- registry resolution and key rotation gates pass

Phase-12B is considered closure-ready when:
- verifier core gate passes
- trust policy gate passes
- verdict binding gate passes
- verifier CLI gate passes
- receipt gate passes
- audit ledger gate passes

Phase-12C is considered closure-ready when:
- exchange gate passes
- cross-node parity gate passes
- multi-signature gate passes
- `proofd` gate passes
- replay admission boundary gate passes
- replicated verification remains outside Phase-12 core closure

Phase-12 as a whole is considered closure-ready when:
- Phase-12A normative gates are green
- Phase-12B normative gates are green
- required Phase-12C boundaries are documented and green
- documentation and security model remain aligned with implementation

---

## References

- `docs/specs/phase12-trust-layer/PROOF_BUNDLE_V2_SPEC.md`
- `docs/specs/phase12-trust-layer/PROOF_EXCHANGE_PROTOCOL_MESSAGE_FORMAT.md`
- `docs/specs/phase12-trust-layer/PROOF_VERIFIER_CRATE_ARCHITECTURE.md`
- `docs/specs/phase12-trust-layer/PROOF_BUNDLE_ATTACK_SURFACE_SECURITY_MODEL.md`
- `docs/specs/phase12-trust-layer/VERIFICATION_CONTEXT_DISTRIBUTION_CONTRACT.md`
- `docs/specs/phase12-trust-layer/VERIFICATION_CONTEXT_OBJECT_SPEC.md`
- `docs/specs/phase12-trust-layer/VERIFIER_ATTESTATION_AND_TRUST_REGISTRY_CONTRACT.md`
- `docs/specs/phase12-trust-layer/CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`
- `docs/specs/phase12-trust-layer/VERIFIER_AUTHORITY_SEMANTICS_AND_DELEGATION_CONTRACT.md`
- `docs/specs/phase12-trust-layer/VERIFIER_REGISTRY_LINEAGE_AND_DISTRIBUTION_MODEL.md`
- `docs/specs/phase12-trust-layer/VERIFIER_AUTHORITY_GRAPH_CONSTRAINTS.md`
- `docs/specs/phase12-trust-layer/VERIFIER_AUTHORITY_RESOLUTION_ALGORITHM.md`
- `docs/specs/phase12-trust-layer/tasks.md`
- Phase-11 `P11-42` Proof Bundle Portability

---

**Maintained by:** AykenOS Architecture Board
**Last Updated:** 2026-03-07
**Status:** Draft

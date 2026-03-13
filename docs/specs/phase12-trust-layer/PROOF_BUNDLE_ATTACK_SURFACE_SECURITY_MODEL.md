# Security Model: Proof Bundle Attack Surface

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-07
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Related Spec:** `requirements.md`, `PROOF_BUNDLE_V2_SPEC.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`, `VERIFICATION_CONTEXT_DISTRIBUTION_CONTRACT.md`, `VERIFICATION_CONTEXT_OBJECT_SPEC.md`, `VERIFICATION_CONTEXT_PORTABILITY_AND_DISTRIBUTION_PROTOCOL.md`, `VERIFIER_ATTESTATION_AND_TRUST_REGISTRY_CONTRACT.md`, `VERIFIER_AUTHORITY_SEMANTICS_AND_DELEGATION_CONTRACT.md`, `VERIFIER_REGISTRY_LINEAGE_AND_DISTRIBUTION_MODEL.md`, `VERIFIER_AUTHORITY_GRAPH_CONSTRAINTS.md`, `VERIFIER_AUTHORITY_RESOLUTION_ALGORITHM.md`, `AUTHORITY_TOPOLOGY_FORMAL_MODEL.md`, `CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`, `tasks.md`

---

## 1. Purpose

This document defines the attack surface and security model for the Phase-12 proof bundle verifier.

Its job is to answer four concrete questions:
- what the verifier is defending
- where trust boundaries exist
- which attack classes must fail closed
- which protections are already implemented vs deferred

This document is normative for Phase-12 security direction.

It is not a cryptography textbook and it is not a transport protocol document.

---

## 2. Security Goals

Phase-12 security goals are:
- preserve Phase-11 portable identity semantics
- prevent trust metadata from mutating portable proof identity
- reject tampered, downgraded, or structurally ambiguous proof bundles
- bind final acceptance to explicit policy and registry inputs
- prevent distributed trust reuse under mismatched verification context
- keep trust evaluation outside Ring0
- preserve deterministic verdict behavior across nodes

Target property:

`same bundle_id + same trust_overlay_hash + same policy_hash + same registry_snapshot_hash => same verdict`

---

## 3. Non-Goals

This document does not define:
- transport encryption
- distributed consensus
- remote registry distribution
- remote attestation
- replay execution admission
- kernel-side trust enforcement

Critical boundary:

`accepted proof != admitted replay`

Phase-12 verifies trust.
It does not authorize execution replay by itself.

---

## 4. Assets Under Protection

Security-relevant assets:
- `bundle_id`
- portable core artifacts under `manifest.json`, `checksums.json`, `evidence/`, `traces/`, `reports/`, `meta/run.json`
- `trust_overlay_hash`
- `producer/producer.json`
- `signatures/signature-envelope.json`
- `policy_hash`
- `registry_snapshot_hash`
- verification receipt contents
- deterministic verdict outcome

Security consequence:
- corruption of any portable core asset threatens proof validity
- corruption of any overlay asset threatens trust attribution
- corruption of policy or registry input threatens acceptance semantics

---

## 5. Trust Boundaries

### 5.1 Portable Core Boundary

Portable core is Phase-11 trust-neutral proof material.

Required rule:
- portable core determines `bundle_id`
- portable core must remain valid even when trust overlay changes

### 5.2 Trust Overlay Boundary

Trust overlay is detached and includes:
- producer declaration
- signature envelope

Required rule:
- overlay must not change `bundle_id`
- overlay must still be hashable and audit-visible

### 5.3 Policy Boundary

Policy is external verifier input.

Required rule:
- policy decides acceptance, not proof validity

### 5.4 Registry Boundary

Registry snapshot is external verifier input.

Required rule:
- key resolution must be explicit, deterministic, and snapshot-bound

### 5.5 Receipt Boundary

Receipts are derived artifacts.

Required rule:
- receipts must never mutate portable proof identity

---

## 6. Adversary Model

The verifier must assume an attacker may:
- modify bundle files in transit or at rest
- reorder or replace files inside a bundle
- forge producer metadata
- substitute or confuse signature envelopes
- provide stale or poisoned registry snapshots
- provide downgraded or substituted policies
- replay old receipts as if they were current trust evidence
- exploit parser ambiguity or non-canonical JSON behavior
- attempt algorithm confusion via mislabeled signature metadata
- attempt trust escalation by mixing valid proof with invalid trust data

The verifier is not required to assume:
- Ring0 compromise
- hardware trust anchor compromise
- cryptographic primitive break

Those may exist in the real world, but they are outside Phase-12 core scope.

---

## 7. Pipeline Attack Surface

### 7.1 Bundle Load and Layout Validation

Attack class:
- missing required files
- extra unexpected paths used to confuse tooling
- path confusion
- malformed directory tree

Required mitigation:
- required portable and overlay paths must be explicit
- layout mismatch must fail closed
- verifier must resolve files relative to bundle root only

### 7.2 Portable Core Integrity Verification

Attack class:
- tampered `checksums.json`
- tampered evidence or report payload
- manifest/checksum disagreement
- stale or substituted proof material

Required mitigation:
- recompute checksums
- recompute `bundle_id`
- reject checksum or identity mismatch

### 7.3 Proof-Chain Validation

Attack class:
- fake `proof_verify.json`
- partial proof-chain replacement
- replay report substitution
- ledger/transcript root drift

Required mitigation:
- proof-chain validation must remain independent from trust policy
- core proof inconsistency must produce `INVALID`

### 7.4 Overlay Parsing and Identity

Attack class:
- producer metadata mutation
- signature envelope mutation
- signer identity confusion
- detached signature omission

Required mitigation:
- recompute `trust_overlay_hash`
- reject missing or inconsistent overlay state
- reject ambiguous or empty signer metadata

### 7.5 Registry Resolution

Attack class:
- registry poisoning
- stale snapshot reuse
- ambiguous key ownership
- revoked-key substitution

Required mitigation:
- verification must bind to `registry_snapshot_hash`
- ambiguous resolution must fail closed
- revoked keys must reject deterministically

### 7.6 Policy Evaluation

Attack class:
- policy downgrade
- silent quorum weakening
- trusted producer expansion
- trusted key expansion

Required mitigation:
- verdict must bind to `policy_hash`
- policy must be canonical and hashable
- quorum rules must be explicit and deterministic

### 7.7 Verification Context Distribution

Attack class:
- receipt transported without explicit context
- policy/registry drift hidden behind valid local receipt
- shared trust claim made under mismatched verifier contract semantics

Required mitigation:
- distributed trust reuse must bind explicit `verification_context_id`
- distributed trust reuse must carry reconstructable context transport material, not receipt-only identifiers
- context mismatch must fail closed
- local receipts must not be treated as standalone distributed trust evidence

### 7.8 Receipt Emission

Attack class:
- forged receipt
- stale receipt replay
- receipt used as proof replacement

Required mitigation:
- receipt must be derived from verdict subject
- receipt must never participate in `bundle_id`
- later signed receipts must bind the same subject tuple

---

## 8. Primary Attack Classes

### 8.1 Portable Core Tampering

Scenario:
- attacker mutates `manifest.json`, `checksums.json`, `evidence/`, `traces/`, or `reports/`

Expected defense:
- checksum mismatch or `bundle_id` mismatch
- verdict = `INVALID`

### 8.2 Trust Overlay Tampering

Scenario:
- attacker swaps `producer.json` or `signature-envelope.json`

Expected defense:
- `trust_overlay_hash` changes
- overlay invariants break or crypto verification fails
- verdict = `INVALID`

### 8.3 Signature Confusion

Scenario:
- envelope claims a signer identity that does not match resolved key ownership
- algorithm label is manipulated
- signature is structurally present but cryptographically invalid

Expected defense:
- signature validity and policy acceptance remain separate
- invalid signature never becomes `TRUSTED`
- verdict = `INVALID`

Architecture note:
- crypto verification should live in a future `crypto/` module, not in overlay schema parsing

### 8.4 Registry Poisoning

Scenario:
- malicious verifier input supplies a corrupted or downgraded registry snapshot

Expected defense:
- final verdict must expose `registry_snapshot_hash`
- resolution ambiguity or revoked-key resolution must fail closed
- cross-node parity only claims validity when the same registry snapshot is used

### 8.5 Policy Downgrade

Scenario:
- attacker supplies a weaker trust policy than intended

Expected defense:
- final verdict must expose `policy_hash`
- receipts must carry the exact `policy_hash`
- acceptance under one policy must not be misrepresented as acceptance under another

### 8.6 Receipt Replay

Scenario:
- old receipt is replayed after key revocation, policy change, or overlay change

Expected defense:
- receipts are advisory derived artifacts
- verifier must recompute subject inputs and not trust receipts as primary truth
- stale receipt without matching subject inputs must be irrelevant

### 8.7 Replay Admission Confusion

Scenario:
- a valid trusted proof is treated as automatic authorization for runtime replay

Expected defense:
- replay admission remains a separate contract
- proof verification success alone must not imply execution authorization

### 8.8 Canonicalization Ambiguity

Scenario:
- attacker exploits JSON formatting differences or parser behavior to produce inconsistent hashes

Expected defense:
- canonical JSON must follow RFC 8785 JCS semantics
- logical content, not source formatting, must drive hash identity

### 8.9 Distributed Context Drift

Scenario:
- two nodes validate the same bundle under different policy or registry context and incorrectly treat the results as the same shared trust fact

Expected defense:
- distributed acceptance claims must bind `verification_context_id`
- unequal context must reject as distributed trust evidence
- old but valid receipts may remain historical artifacts, not current acceptance proof

### 8.10 Untrusted Verifier Receipt Amplification

Scenario:
- a valid signed receipt is emitted by a verifier that is not trusted as a distributed trust speaker
- downstream nodes mistake receipt signature validity for verifier trust authority

Expected defense:
- signed receipt validity and trusted verifier status remain separate
- verifier trust registry resolution must be explicit and fail closed
- untrusted verifier receipts must not become shared distributed trust facts

### 8.11 Cross-Node Parity Misclassification

Scenario:
- nodes disagree on subject, context, verifier trust, or verdict
- downstream systems collapse the mismatch into a misleading generic success or local-verdict label

Expected defense:
- parity status must be classified separately from local verifier verdict
- `historical_only` must not be reported as current parity success
- context mismatch must not be re-labeled as `UNTRUSTED`

### 8.12 Verifier Authority Capture

Scenario:
- a verifier remains cryptographically valid
- but verifier trust registry or authority semantics are manipulated so that an untrusted or over-scoped verifier is treated as a trusted distributed speaker

Expected defense:
- verifier authority semantics must remain separate from mere receipt signature validity
- ambiguous verifier identity or authority mapping must fail closed
- delegation must default to deny unless explicitly bounded

### 8.13 Verifier Registry Split-Brain and Rollback

Scenario:
- nodes use conflicting verifier trust registry snapshots in the same registry scope
- one node silently downgrades to an older or forked lineage snapshot

Expected defense:
- verifier registry lineage must be explicit via snapshot hash, parent hash, and epoch
- same-scope same-epoch different-hash snapshots must be treated as fork
- rollback or forked lineage must not be silently treated as current verifier authority

### 8.14 Verifier Authority Loop Attack

Scenario:
- verifier delegation edges form a cycle or self-sustaining loop
- authority appears valid at each hop but becomes self-authorizing as a graph

Expected defense:
- verifier delegation graph must remain acyclic
- self-delegation must fail closed
- delegated scope must only narrow and depth must remain bounded

### 8.15 Delegation Fork and Resolution Drift

Scenario:
- a delegate has more than one valid-looking parent chain
- different nodes select different chains through implicit or implementation-defined tie-breaking

Expected defense:
- authority resolution must be deterministic and explicit
- multiple surviving parent chains must fail closed as ambiguity
- silent parent selection heuristics must be forbidden unless explicitly versioned
- current root authority must come from explicit registry-declared roots
- successful delegated authority resolution should expose canonical `authority_chain_id` for parity comparison

---

## 9. Fail-Closed Rules

The verifier MUST reject on:
- missing required portable path
- missing required overlay path
- unsupported schema version
- checksum mismatch
- `bundle_id` mismatch
- proof-chain mismatch
- `trust_overlay_hash` mismatch
- unresolved `producer_pubkey_id`
- revoked key
- ambiguous registry ownership
- invalid detached signature
- ambiguous quorum evaluation
- policy mismatch

Trust-critical rule:

`any trust-critical verification failure => deterministic reject`

---

## 10. Deterministic Security Invariants

### 10.1 Identity Separation Invariant

`bundle_id != trust_overlay_hash`

Portable proof identity and trust overlay identity must remain separate.

### 10.2 Acceptance Binding Invariant

`verdict_subject = (bundle_id, trust_overlay_hash, policy_hash, registry_snapshot_hash)`

No weaker tuple is acceptable for distributed verification claims.

### 10.3 Fail-Closed Invariant

Missing trust-critical mechanism must never degrade to warning-only acceptance.

### 10.4 External Input Invariant

Policy and registry are verifier-local external inputs.
They must never be silently imported from inside the bundle.

---

## 11. Current Implementation Status

Current `proof-verifier` skeleton status:

Implemented:
- bundle load and layout validation
- checksum validation
- `bundle_id` recomputation
- strict `proof_manifest` validation and proof-hash recomputation
- ledger root and transcript root recomputation from bundled evidence
- replay/report cross-consistency validation
- producer and signature envelope parsing
- `trust_overlay_hash` recomputation
- registry snapshot resolution
- canonical `registry_snapshot_hash` recomputation and declared-vs-recomputed binding
- Ed25519 detached signature verification over `bundle_id`
- detached signature algorithm allowlist enforcement
- policy evaluation
- verdict subject construction
- signed and unsigned receipt emission
- signed receipt payload/signature verification
- append-only audit event generation and ledger append path
- audit ledger hash-chain, receipt-hash, and signed receipt verification
- verifier-trust registry canonical hash validation with explicit root verifier set handling
- deterministic verifier authority resolution with fail-closed ambiguity and canonical `authority_chain_id` emission

Intentionally not yet implemented:
- full proof manifest field validation

Current security posture:
- verifier remains fail-closed on malformed, non-allowlisted, unresolved, or cryptographically invalid detached signatures
- detached signature verification is now active for the initial mandatory Ed25519 path
- additional signature algorithms remain out of baseline scope unless introduced through explicit versioned algorithm agility
- portable-core proof validity is now artifact-driven instead of report-driven for proof manifest bindings and replay/report consistency
- canonical registry snapshot binding is now artifact-driven instead of trusting only declared registry metadata
- signed receipt verification is now active for canonical receipt payloads and stale subject mismatch rejection
- shared distributed receipt acceptance is now bound to current verifier authority through verifier-trust registry validation, authority-scope checks, and canonical `authority_chain_id`
- audit transparency is now active through append-only event chaining, signed receipt verification, and receipt-hash binding
- audit receipt verification can now reuse verifier authority binding when verifier-trust registry material is supplied
- audit append path is now serialized to prevent concurrent chain forks on verifier-local ledgers
- local Phase-12A gate evidence is now active for producer schema, detached signature envelope, bundle-v2 schema/compatibility, detached signature verification, registry resolution, and key rotation/revocation lifecycle checks
- local `ci-gate-proof-verifier-core` evidence is now active and proves deterministic verifier-core outcomes across trusted, policy-rejected, untrusted, and invalid core-path scenarios
- local `ci-gate-proof-trust-policy` evidence is now active and proves canonical policy hash stability plus fail-closed handling for unsupported quorum semantics
- local `ci-gate-proof-verdict-binding` evidence is now active and proves stable four-field verdict subject binding plus receipt payload reuse of the same tuple
- local `ci-gate-proof-verifier-cli` evidence is now active and proves the thin offline CLI remains a wrapper over verifier-core semantics while exporting stable human-readable and JSON verdict binding output
- dedicated receipt/audit gate evidence is now active through local `ci-gate-proof-receipt` and `ci-gate-proof-audit-ledger` execution
- local `ci-gate-verifier-authority-resolution` evidence now covers signed receipt authority binding in addition to authority graph resolution
- verifier authority resolution is now artifact-driven through canonical verifier-trust registry binding, explicit roots, and deterministic `authority_chain_id`
- delegation depth overflow is now surfaced as a distinct fail-closed authority result rather than collapsing into generic no-valid-chain output
- effective authority scope is now derived from surviving chain semantics instead of mirroring only requested scope
- authority negative coverage now includes historical-only, revoked, orphan, scope-mismatch, algorithm-drift, key-material-drift, missing-`authority_chain_id`, and depth-overflow cases
- authority gate evidence now computes `authority_chain_id_equal` from real resolver-vs-receipt authority comparison
- portable-core proof validation now enforces proof-manifest mode/signature contract fields, digest-shape checks, and replay-trace hash bindings in addition to prior manifest hash recomputation
- local `ci-gate-proof-exchange` evidence now validates that transport preserves payload / overlay / verification-context identity while treating transport metadata as non-authoritative
- local cross-node parity gate evidence now classifies baseline parity, subject drift, context drift (including verifier-contract-version drift), delegated authority-chain drift, authority-scope drift, historical-only authority, insufficient-evidence, explicit verdict-drift guard, and receipt-absent parity-artifact conditions into `failure_matrix.json` with real `authority_chain_id_equal` and `effective_authority_scope_equal` comparison
- local parity reporting is now split into `parity_consistency_report.json` for distributed drift classes and `parity_determinism_report.json` for same-surface verdict divergence alarms
- local parity evidence now also exports `parity_determinism_incidents.json`, making same-`D_i` / different-`K_i` determinism failures explicit incident artifacts with stable hash-based `incident_id` values instead of only aggregate counts
- determinism incidents now also carry derived severity labels so pure model failures can be distinguished from drift-shaped incidents without turning severity into policy or consensus semantics
- the local parity pipeline now suppresses historical-only, insufficient-evidence, or hidden-drift same-surface verdict splits as false determinism candidates instead of escalating them as true determinism incidents
- local parity evidence now also exports `parity_convergence_report.json`, giving a first node-derived `N`-node aggregate surface over stable `NodeParityOutcome` objects and explicit `D_i` / `K_i` partitions
- local parity evidence now also exports `parity_drift_attribution_report.json`, attributing each surface partition to subject/context/authority/verdict/evidence causes rather than reporting only aggregate split counts
- local parity drift evidence now also summarizes `historical_authority_islands` and `insufficient_evidence_islands`, so authority-epoch lag and evidence-gap clusters are visible as explicit diagnostics artifacts instead of being buried inside generic partition counts
- local parity evidence may now also export `parity_authority_drift_topology.json`, making dominant current authority clusters and drifted authority islands visible without turning topology into authority selection or consensus semantics
- local parity evidence may now also export `parity_authority_suppression_report.json`, making false authority drift suppression explicit when scope aliases, registry skew, or historical shadowing would otherwise inflate drift diagnostics
- parity node-object generation is now centralized in `authority/parity.rs`, making the crate parity layer the single hash authority for `surface_key` / `outcome_key` derivation
- portable-core negative coverage now includes proof-manifest count and digest drift for `event_count`, `violation_count`, `proof_hash`, `replay_result_hash`, `config_hash`, and `kernel_image_hash`
- the current verifier / transport stack is still not closure-complete because full proof-manifest field coverage, broader audit tamper corpus, multisignature/quorum transport, and service-backed distributed verification context transport remain pending

This is the correct posture for active P12-07 hardening.

---

## 12. Required Hardening Roadmap

### Milestone 1: Portable Core Hardening (baseline active)

Continue:
- extend negative corpus around `proof_manifest` drift and corrupted report bindings
- strengthen replay trace / final-state cross-consistency coverage
- keep proof validity artifact-driven rather than report-driven

Primary attacks reduced:
- proof substitution
- report drift
- partial bundle tampering

### Milestone 2: Crypto Separation and Signature Verification

Add:
- broader algorithm agility beyond initial Ed25519 allowlist if explicitly versioned
- expanded negative corpus for signature confusion and malformed key material
- cross-node crypto parity coverage

Primary attacks reduced:
- signature confusion
- fake signer attribution
- structural-only overlay acceptance

### Milestone 3: Registry Snapshot Integrity (baseline active)

Continue:
- extend negative corpus for registry hash drift, ambiguous ownership, and stale snapshot confusion
- harden snapshot format evolution rules
- expand cross-node registry parity coverage

Primary attacks reduced:
- registry poisoning
- stale snapshot confusion
- cross-node registry drift

### Milestone 4: Receipt Hardening (baseline active)

Continue:
- expand receipt tamper/staleness negative corpus
- keep signed receipt gate evidence aligned with verifier-core output contract
- add append-only audit linkage for signed receipt events

Primary attacks reduced:
- forged receipt replay
- unsigned receipt misuse

### Milestone 5: Audit Ledger Hardening (baseline active)

Continue:
- expand audit tamper corpus for event-id drift, chain drift, and receipt-hash mismatch
- keep dedicated gate evidence aligned with appended ledger outputs
- harden service-level persistence and retention semantics without moving trust into Ring0

Primary attacks reduced:
- verification repudiation
- silent receipt replay without audit trace
- append-only chain tampering

### Milestone 6: Verification Context Distribution

Add:
- explicit `verification_context_id` contract separate from `verdict_subject`
- context binding for distributed receipt and audit exchange
- mismatch corpus for policy/registry/verifier-contract drift across nodes

Primary attacks reduced:
- false shared trust claims
- context drift confusion
- historical receipt misinterpretation as current acceptance

---

## 13. Evidence and Gate Expectations

Security hardening should eventually be bound to explicit gates.

Recommended future gates:
- `ci-gate-p12-portable-core-hardening`
- `ci-gate-p12-signature-crypto-verify`
- `ci-gate-p12-registry-hash-binding`
- `ci-gate-p12-receipt-signing`
- `ci-gate-p12-policy-downgrade-negative`
- `ci-gate-p12-registry-poisoning-negative`

Recommended evidence:
- tamper matrix report
- policy downgrade matrix
- registry poisoning matrix
- signature confusion negative cases
- receipt replay negative cases

---

## 14. Security Summary

Phase-12 turns proof portability into trust-aware verification, but only if the verifier rejects ambiguity.

The core rule is simple:

`valid proof != trusted proof`

And the distributed rule is equally simple:

`trusted proof != replay admission`

If identity separation, explicit external trust inputs, and fail-closed behavior remain intact, the Phase-12 verifier can harden safely without breaking Phase-11 portability.

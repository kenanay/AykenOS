# Design Note: `proof-verifier` Crate Architecture

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-07
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Related Spec:** `requirements.md`, `PROOF_BUNDLE_V2_SPEC.md`, `PROOF_BUNDLE_ATTACK_SURFACE_SECURITY_MODEL.md`, `PROOF_EXCHANGE_PROTOCOL_MESSAGE_FORMAT.md`, `VERIFICATION_CONTEXT_DISTRIBUTION_CONTRACT.md`, `VERIFICATION_CONTEXT_OBJECT_SPEC.md`, `VERIFICATION_CONTEXT_PORTABILITY_AND_DISTRIBUTION_PROTOCOL.md`, `VERIFIER_ATTESTATION_AND_TRUST_REGISTRY_CONTRACT.md`, `VERIFIER_AUTHORITY_SEMANTICS_AND_DELEGATION_CONTRACT.md`, `VERIFIER_REGISTRY_LINEAGE_AND_DISTRIBUTION_MODEL.md`, `VERIFIER_AUTHORITY_GRAPH_CONSTRAINTS.md`, `VERIFIER_AUTHORITY_RESOLUTION_ALGORITHM.md`, `CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`, `PARITY_LAYER_ARCHITECTURE.md`, `AUTHORITY_TOPOLOGY_FORMAL_MODEL.md`, `PROOFD_DIAGNOSTICS_SERVICE_SURFACE.md`, `tasks.md`
**Target Crate:** `ayken-core/crates/proof-verifier/`

---

## 1. Purpose

This document defines the implementation architecture for the Phase-12 `proof-verifier` Rust crate.

The crate exists to verify:
- Phase-11 portable proof identity (`bundle_id`)
- portable core integrity and proof-chain validity
- detached trust overlay integrity
- producer registry resolution
- trust policy evaluation
- deterministic verdict emission

This crate is the core of P12-07.

It is intentionally:
- userspace/offline
- library-first
- fail-closed
- deterministic
- separate from transport, orchestration, and kernel runtime

---

## 2. Architectural Position

The `proof-verifier` crate sits between the portable proof bundle and higher-level acceptance surfaces.

Architectural ladder:

`proof_bundle -> proof-verifier -> verdict -> receipt -> distributed acceptance`

Boundary rules:
- Ring0 does not import this crate.
- Network transport does not live in this crate.
- Long-running service behavior does not live in this crate.
- CLI is a thin wrapper and is not part of the core verification engine.

The crate consumes immutable inputs and emits a deterministic result.

---

## 3. Core Invariants

### 3.1 Portable Identity Invariant

`bundle_id = H(canonical_manifest_without_bundle_id || canonical_checksums)`

The crate MUST treat `bundle_id` as the only normative portable identity term.

### 3.2 Trust Overlay Invariant

`trust_overlay_hash = H(JCS(producer/producer.json) || JCS(signatures/signature-envelope.json))`

The crate MUST verify trust overlay integrity without mutating `bundle_id`.

### 3.3 Deterministic Verdict Invariant

`same bundle_id + same trust_overlay_hash + same policy_hash + same registry_snapshot_hash => same verdict`

### 3.4 Runtime Boundary Invariant

Verification policy and trust evaluation MUST remain outside Ring0.

### 3.5 Fail-Closed Invariant

Any trust-critical verification failure MUST produce deterministic reject behavior.

---

## 4. Crate Boundary

### 4.1 In Scope

- bundle loading from a filesystem path or in-memory representation
- portable core schema validation
- checksum and bundle identity recomputation
- proof-chain validation
- producer declaration parsing
- signature envelope parsing
- trust overlay hash recomputation
- registry snapshot parsing and key resolution
- detached signature cryptography over resolved public keys
- trust policy parsing and evaluation
- verdict subject construction
- receipt object generation

### 4.2 Out of Scope

- network fetch of bundles
- network fetch of registries
- consensus or quorum across machines
- replay execution
- kernel integration
- service supervision
- append-only audit ledger persistence
- distributed verification context distribution

Design rule:
- the crate may define receipt and audit event data structures
- persistent logging and service orchestration belong to `proofd` or other wrappers

---

## 5. Public API Shape

The core API should be library-first and deterministic.

Recommended surface:

```rust
pub struct VerifyRequest<'a> {
    pub bundle_path: &'a std::path::Path,
    pub policy: &'a TrustPolicy,
    pub registry_snapshot: &'a RegistrySnapshot,
    pub receipt_mode: ReceiptMode,
}

pub struct VerificationOutcome {
    pub verdict: VerificationVerdict,
    pub subject: VerdictSubject,
    pub findings: Vec<VerificationFinding>,
    pub receipt: Option<VerificationReceipt>,
}

pub enum VerificationVerdict {
    Trusted,
    Untrusted,
    Invalid,
    RejectedByPolicy,
}

pub fn verify_bundle(request: &VerifyRequest) -> Result<VerificationOutcome, VerifierRuntimeError>;
```

Error separation rule:
- deterministic verification failures become `VerificationVerdict` results plus findings
- host/runtime failures remain `VerifierRuntimeError`

Examples of runtime errors:
- bundle path unreadable
- receipt output path unwritable
- registry snapshot file cannot be opened

Examples of deterministic invalid results:
- schema mismatch
- checksum mismatch
- `bundle_id` mismatch
- proof-chain mismatch
- invalid detached signature
- revoked key
- ambiguous quorum evaluation

This separation keeps machine verdicts stable while avoiding process-level ambiguity.

---

## 6. Recommended Source Layout

```text
ayken-core/crates/proof-verifier/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    ├── errors.rs
    ├── types.rs
    ├── canonical/
    │   ├── mod.rs
    │   ├── digest.rs
    │   ├── jcs.rs
    │   └── tree_hash.rs
    ├── bundle/
    │   ├── mod.rs
    │   ├── loader.rs
    │   ├── layout.rs
    │   ├── manifest.rs
    │   └── checksums.rs
    ├── portable_core/
    │   ├── mod.rs
    │   ├── checksum_validator.rs
    │   ├── identity.rs
    │   └── proof_chain_validator.rs
    ├── overlay/
    │   ├── mod.rs
    │   ├── producer.rs
    │   ├── signature_envelope.rs
    │   └── overlay_validator.rs
    ├── crypto/
    │   ├── mod.rs
    │   └── ed25519.rs
    ├── registry/
    │   ├── mod.rs
    │   ├── snapshot.rs
    │   └── resolver.rs
    ├── authority/
    │   ├── mod.rs
    │   ├── determinism_incident.rs
    │   ├── drift_attribution.rs
    │   ├── parity.rs
    │   ├── snapshot.rs
    │   └── resolution.rs
    ├── policy/
    │   ├── mod.rs
    │   ├── schema.rs
    │   ├── quorum.rs
    │   └── policy_engine.rs
    ├── verdict/
    │   ├── mod.rs
    │   ├── subject.rs
    │   └── verdict_engine.rs
    ├── receipt/
    │   ├── mod.rs
    │   ├── schema.rs
    │   ├── receipt_emitter.rs
    │   └── verify.rs
    ├── audit/
    │   ├── mod.rs
    │   ├── schema.rs
    │   ├── ledger.rs
    │   └── verify.rs
    └── testing/
        ├── mod.rs
        ├── fixtures.rs
        └── golden.rs
```

The crate SHOULD begin as a library crate.

P12-10 MAY later add:
- a binary target under `src/bin/`
- or a thin sibling tool crate

The preferred default is:
- keep the verification engine in the library
- keep command-line UX in a wrapper layer

---

## 7. Module Responsibilities

### 7.1 `canonical/`

Responsibilities:
- RFC 8785 JCS canonicalization
- SHA-256 digest helpers
- canonical tree hashing

This module is the root determinism dependency.

No module may implement ad-hoc hashing outside this boundary.

### 7.2 `bundle/`

Responsibilities:
- filesystem layout validation
- bundle root loading
- required file discovery
- parsing of `manifest.json` and `checksums.json`

This module does not decide verdicts.

It only materializes validated inputs for later stages.

### 7.3 `portable_core/`

Responsibilities:
- recompute file checksums for portable payload
- recompute `bundle_id`
- verify manifest/checksum consistency
- validate Phase-11 proof-chain artifacts

This module owns:
- portable identity verification
- portable-core proof validity

It must not inspect trust policy semantics.

### 7.4 `overlay/`

Responsibilities:
- parse `producer/producer.json`
- parse `signatures/signature-envelope.json`
- recompute `trust_overlay_hash`
- validate overlay structural invariants

This module proves:
- the trust overlay is well-formed
- the overlay hash is stable

It does not decide whether a signer is trusted.

### 7.5 `registry/`

Responsibilities:
- parse immutable registry snapshots
- recompute canonical `registry_snapshot_hash`
- enforce declared-vs-recomputed registry hash binding
- resolve `producer_pubkey_id`
- enforce resolution determinism
- surface concrete public key material plus active, revoked, and superseded key state

This module does not fetch remote registry state.

Registry acquisition belongs outside the crate.

### 7.6 `authority/`

Responsibilities:
- parse immutable verifier trust registry snapshots
- recompute canonical `verifier_registry_snapshot_hash`
- validate explicit root verifier set semantics
- validate authority graph constraints and fail-closed ambiguity
- resolve delegated verifier authority deterministically
- classify delegation depth overflow as a distinct fail-closed result
- compute effective authority scope from canonical chain semantics
- emit canonical `authority_chain_id` for parity and audit comparison
- compare cross-node delegated authority outcomes into deterministic parity/failure-matrix surfaces
- build canonical `NodeParityOutcome` objects as the single hash authority for `D_i` / `K_i`
- attribute node-derived drift across subject/context/authority/verdict/evidence surfaces
- emit explicit `DeterminismIncident` artifacts with stable hash-based `incident_id` values and deterministically derived severity metadata when nodes share `D_i` but diverge on `K_i`, while suppressing drift-shaped or non-current-evidence false determinism candidates
- normalize authority-chain / scope identity for diagnostics and emit suppression reports when semantic-equivalent authority surfaces would otherwise appear as false drift
- derive `parity_authority_drift_topology.json` from canonical authority-chain and scope partitions without turning authority clustering into truth selection or consensus semantics
- derive `parity_incident_graph.json` from `NodeParityOutcome` plus true determinism incidents without introducing new truth-bearing objects or consensus semantics

Phase-12 depth semantics are counted as explicit delegation hops from an explicit root.

This module is parallel to local proof verification.

It does not mutate local `verify_bundle()` verdict semantics.
It evaluates verifier-trust authority for distributed parity and later `proofd` surfaces.

### 7.7 `crypto/`

Responsibilities:
- enforce the detached signature algorithm allowlist
- decode resolved public key material
- verify detached signatures over `bundle_id`
- emit deterministic invalid findings on malformed or cryptographically invalid signatures

This module owns signature validity, not policy acceptance.

### 7.8 `policy/`

Responsibilities:
- parse and validate trust policy schema
- compute `policy_hash`
- evaluate signature quorum rules
- evaluate trusted producer and trusted key policy

This module owns acceptance semantics.

Signature validity and signature acceptance remain separate concerns.

### 7.9 `verdict/`

Responsibilities:
- build `VerdictSubject`
- map verification findings to final verdict
- ensure deterministic verdict synthesis

Recommended rule:
- `INVALID` covers structural, integrity, proof, or signature validity failure
- `UNTRUSTED` covers valid proof with non-trusted producer/key
- `REJECTED_BY_POLICY` covers policy-explicit non-acceptance
- `TRUSTED` covers full acceptance

### 7.10 `receipt/`

Responsibilities:
- build verification receipt objects
- sign or serialize receipt payloads
- verify signed receipt payload/signature binding
- bind shared distributed receipt acceptance to verifier-trust registry authority resolution and scope checks
- keep receipt bytes out of `bundle_id`

Internal verifier hardening may inject forged resolved-authority fixtures to exercise post-resolution fail-closed paths such as missing `authority_chain_id`.

Receipt persistence is not a core engine concern.

The module should return receipt objects or bytes to the caller.

### 7.11 `audit/`

Responsibilities:
- build deterministic verification audit events
- append hash-chained audit events to append-only ledgers through serialized append operations
- verify audit ledger integrity, receipt-hash binding, and signed receipt validity when receipt material is available
- verify authority-bound receipt reuse when verifier-trust registry material is available

Audit events remain derived artifacts and MUST NOT affect `bundle_id` or trust acceptance semantics.

### 7.12 `testing/`

Responsibilities:
- golden bundle fixtures
- deterministic matrix fixtures
- tamper cases
- rotation and revocation test data

This module keeps proof-verifier determinism testable without service scaffolding.

---

## 8. Dependency Direction

One-way dependency graph:

```text
canonical
    ^
    |
bundle ----> portable_core
    \           ^
     \          |
      -> overlay ----\
                      \
registry -------------> crypto \
registry -------------> authority
policy ------------------------> verdict -> receipt -> audit
```

Rules:
- `canonical` is a foundational utility layer.
- `portable_core` depends on `bundle` and `canonical`.
- `overlay` depends on `canonical`.
- `crypto` depends on overlay outputs plus resolved registry material, but does not own acceptance semantics.
- `authority` depends on canonicalized verifier-trust registry inputs and remains outside local verdict synthesis.
- `registry` and `policy` remain independent input-evaluation layers.
- `verdict` is the first layer allowed to see portable, overlay, crypto, policy, and registry results together.
- `receipt` depends on verdict outputs, never the other way around.
- `audit` depends on verdict and receipt outputs, never the other way around.

Forbidden dependency patterns:
- `policy -> bundle`
- `policy -> portable_core`
- `overlay -> verdict`
- `crypto -> policy`
- `portable_core -> policy`
- `receipt -> policy`
- `audit -> policy`

This preserves mechanism/policy separation inside the crate itself.

---

## 9. Verification Pipeline Mapping

The crate pipeline should map directly to spec order:

1. `bundle::loader`
   - load bundle root
   - validate required layout
2. `portable_core::checksum_validator`
   - recompute portable file checksums
3. `portable_core::identity`
   - recompute `bundle_id`
4. `portable_core::proof_chain_validator`
   - verify Phase-11 proof chain
5. `overlay::producer`
   - parse producer declaration
6. `overlay::signature_envelope`
   - parse detached signatures
7. `overlay::overlay_validator`
   - recompute `trust_overlay_hash`
8. `registry::resolver`
   - recompute and validate canonical `registry_snapshot_hash`
   - resolve `producer_pubkey_id`
9. `crypto::ed25519`
   - verify detached signatures over `bundle_id`
10. `policy::policy_engine`
   - evaluate acceptance rules and quorum
11. `verdict::verdict_engine`
   - emit deterministic verdict
12. `receipt::receipt_emitter`
   - produce derived receipt if requested
13. `audit::ledger`
   - append and verify hash-chained audit events if requested

Critical ordering rule:
- proof validity is decided before trust acceptance

This preserves:
- `valid proof != trusted proof`
- `trusted proof != replay admission`

---

## 10. Data Model

Recommended core types:

```text
LoadedBundle
PortableCoreState
ProducerDeclaration
SignatureEnvelope
RegistrySnapshot
ResolvedSignerSet
TrustPolicy
VerdictSubject
VerificationFinding
VerificationOutcome
VerificationReceipt
```

Recommended `VerdictSubject` fields:
- `bundle_id`
- `trust_overlay_hash`
- `policy_hash`
- `registry_snapshot_hash`

Recommended `VerificationFinding` fields:
- `code`
- `message`
- `location`
- `severity`
- `deterministic`

Design rule:
- findings should explain why a verdict occurred
- findings should not redefine the verdict contract

---

## 11. Candidate Dependency Policy

Preferred minimal dependency set:
- `serde`
- `serde_json`
- `sha2`
- `thiserror`
- `time`

Cryptography and signature policy:
- detached signature verification should use a narrowly scoped, audited dependency
- signature algorithm expansion should be feature-gated
- default milestone targets Ed25519 only

Canonicalization policy:
- prefer a JCS implementation with deterministic test coverage
- if an external crate is insufficient, implement a small local adapter around RFC 8785 behavior

Do not introduce:
- async runtime dependencies
- network clients
- database clients
- service frameworks

The crate should remain small, deterministic, and offline-first.

---

## 12. Testing Strategy

### 12.1 Unit Tests

Required unit coverage:
- JCS canonicalization stability
- checksum mismatch detection
- `bundle_id` recomputation
- `trust_overlay_hash` recomputation
- revoked key detection
- quorum evaluation
- verdict classification

### 12.2 Golden Bundle Tests

Use fixed fixtures for:
- valid trusted bundle
- valid but untrusted bundle
- invalid signature bundle
- tampered portable core bundle
- rotated-key bundle
- revoked-key bundle

### 12.3 Determinism Matrix

Required matrix dimensions:
- different file ordering on disk
- formatting changes in JSON source files
- same bundle across repeated runs
- same inputs on different machines

Expected invariant:
- same logical inputs yield byte-identical verdict subject hashes and stable verdicts

### 12.4 Negative Tests

Minimum negative cases:
- missing required file
- manifest/checksums disagreement
- unsupported schema version
- bad detached signature
- ambiguous registry resolution
- quorum underflow

All must fail closed.

---

## 13. Milestone Mapping

### P12-07 Core Crate

Must establish:
- `canonical/`
- `bundle/`
- `portable_core/`
- `overlay/`
- `registry/`
- `authority/`
- `policy/`
- `verdict/`

Dedicated local `ci-gate-proof-verifier-core` evidence SHOULD execute this exact library path and export deterministic outcome matrices rather than a parallel mock pipeline.

### P12-10 CLI

Should remain thin:
- parse arguments
- load policy and registry snapshot
- call `verify_bundle`
- print verdict or JSON output

Semantic surface expansion SHOULD follow `PROOF_VERIFIER_SEMANTIC_CLI_ROADMAP.md` so `P12-10` closure minimum stays offline-first and does not absorb `proofd` or exchange-protocol behavior early.

The current local Stage-1 CLI surface is implemented under:
- `ayken-core/crates/proof-verifier/src/bin/proof-verifier.rs`

Dedicated local `ci-gate-proof-verifier-cli` evidence SHOULD execute the real binary and validate:
- offline `verify bundle`
- external policy and registry loading
- human-readable verdict output
- machine-readable JSON verdict binding output

CLI-specific formatting must not leak into the library core.

### P12-11 Receipt

May activate:
- `receipt/`
- receipt serialization
- receipt signing integration
- dedicated receipt gate evidence via verifier-core-aligned harness execution

### P12-12 Audit Ledger

Should reuse:
- `VerificationOutcome`
- `VerificationReceipt`
- deterministic findings

Core crate MAY append deterministic audit events to verifier-local ledgers.
Service-level retention, shipping, and federation remain outside the core crate.
Dedicated audit-ledger gate evidence SHOULD be produced from the same verifier-core path rather than a parallel reimplementation.
Distributed verification context transport MUST remain outside the local `verdict_subject` model and SHOULD be layered above the core crate.
The canonical verification context object schema likewise belongs to the distributed layer, not the local proof-verification core.

### P12-13 Bundle Exchange Protocol

Bundle exchange remains above the crate boundary.

The local `P12-13` implementation slice SHOULD:
- reuse real verifier-core artifacts
- serialize transport-ready payload / overlay / context / receipt surfaces
- validate fail-closed transport mutation behavior outside the library core

The local `ci-gate-proof-exchange` path therefore belongs in harness / script / evidence layers, not inside `verify_bundle()`.

This preserves the architectural rule:
- verifier core = deterministic evaluation engine
- exchange protocol = transport contract
- `proofd` = service/orchestration layer

### P12-16 `proofd`

Should treat the crate as a pure engine:
- service loads inputs
- service invokes verifier
- service persists receipts and audit events

This keeps the crate composable for offline and service modes.

---

## 14. Workspace Integration Plan

When P12-07 implementation begins:

1. add `crates/proof-verifier` to `ayken-core/Cargo.toml`
2. create library crate skeleton
3. land `canonical`, `bundle`, and `portable_core` first
4. add `overlay`, `registry`, and `policy`
5. add `verdict`
6. gate with deterministic golden fixtures before CLI work

Recommended branch sequence:
- `feat/p12-proof-verifier-core`
- `feat/p12-proof-verifier-cli`
- `feat/p12-verification-receipt`

---

## 15. Summary

The `proof-verifier` crate should be treated as the deterministic trust engine of Phase-12.

Its internal separation must preserve:
- portable identity vs trust overlay
- signature validity vs policy acceptance
- library engine vs CLI/service wrapper
- verification vs replay admission

If these separations hold:
- P12-07 remains implementation-safe
- P12-10 stays thin
- P12-11 and P12-16 can compose on top without architectural drift

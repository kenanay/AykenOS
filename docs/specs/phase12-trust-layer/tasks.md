# Tasks Document: Phase-12 Trust Layer

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-07
**Related Spec:** `PROOF_BUNDLE_V2_SPEC.md`, `requirements.md`
**Created by:** Kenan AY
**Maintained by:** Kenan AY
**Last Edited by:** Kenan AY

---

## 1. Scope

Phase-12 extends Phase-11 proof portability into a trusted proof transport and deterministic distributed verification layer.

This phase is explicitly split into:
- **P12A** Trusted Proof Bundle
- **P12B** Verifier Layer
- **P12C** Distributed Verification

Out of scope:
- kernel runtime changes
- replicated execution
- consensus protocol
- Ring0 trust enforcement
- mutation of Phase-11 portable bundle identity

---

## 2. Execution Policy

- 1 PR = 1 invariant
- Fail-closed verification only
- No direct merge without gate PASS
- Evidence artifacts mandatory for each implemented gate
- `bundle_id` is the only normative portable identity term for on-disk artifacts
- Default task owner: Kenan AY (unless explicitly reassigned)

---

## 3. Core Invariants

### 3.1 Portable Identity Invariant

`bundle_id = H(canonical_manifest_without_bundle_id || canonical_checksums)`

Phase-11 portability identity MUST remain unchanged.

### 3.2 Trust Overlay Invariant

`trust_overlay_hash = H(JCS(producer/producer.json) || JCS(signatures/signature-envelope.json))`

Producer metadata and detached signatures MUST remain outside `bundle_id`.

### 3.3 Deterministic Verdict Invariant

`same bundle_id + same trust_overlay_hash + same policy_hash + same registry_snapshot_hash => same verdict`

### 3.4 Fail-Closed Invariant

Any trust-critical verification failure MUST produce deterministic reject.

### 3.5 Mechanism / Policy Invariant

Trust verification remains userspace/offline and MUST NOT migrate into Ring0.

---

## 4. Task Status Ledger

| Issue | Task | Status | Last Update | Notes |
|------|------|--------|-------------|-------|
| P12-01 | Producer Identity Schema | COMPLETED_LOCAL | 2026-03-08 | local `ci-gate-proof-producer-schema` exports canonical schema and rotation-stability evidence |
| P12-02 | Detached Signature Envelope | COMPLETED_LOCAL | 2026-03-08 | local `ci-gate-proof-signature-envelope` proves detached envelope identity stability |
| P12-03 | Proof Bundle v2 Layout | COMPLETED_LOCAL | 2026-03-08 | local schema + compatibility gates validate v2 layout without portable identity mutation |
| P12-04 | Signature Verification Gate | COMPLETED_LOCAL | 2026-03-08 | local detached signature verify gate exercises allowlisted Ed25519 verification |
| P12-05 | Producer Registry and Trust Root Inputs | COMPLETED_LOCAL | 2026-03-08 | local registry resolution gate covers explicit producer-to-key mapping failure modes |
| P12-06 | Key Rotation and Revocation Contract | COMPLETED_LOCAL | 2026-03-08 | local lifecycle gate covers active/superseded/revoked signer states |
| P12-07 | Rust `proof-verifier` Core Crate | COMPLETED_LOCAL | 2026-03-08 | local `ci-gate-proof-verifier-core` now proves deterministic core outcomes across trusted, policy-rejected, untrusted, and invalid cases |
| P12-08 | Canonical Trust Policy Schema | COMPLETED_LOCAL | 2026-03-08 | local `ci-gate-proof-trust-policy` proves canonical policy hash stability and deterministic verdict binding across trust scenarios |
| P12-09 | Verdict Binding (`policy_hash`, `registry_snapshot_hash`) | COMPLETED_LOCAL | 2026-03-08 | local `ci-gate-proof-verdict-binding` proves four-field verdict subject stability and receipt binding |
| P12-10 | `proof-verifier` CLI | COMPLETED_LOCAL | 2026-03-08 | thin offline `verify bundle` CLI plus local `ci-gate-proof-verifier-cli` now active; richer semantic surfaces remain deferred and Phase-12 whole closure is still pending `P12-13+` |
| P12-11 | Verification Receipt / Acceptance Certificate | COMPLETED_LOCAL | 2026-03-08 | signed receipt payload/sign/verify path active; `ci-gate-proof-receipt` local PASS |
| P12-12 | Verification Audit Ledger | COMPLETED_LOCAL | 2026-03-08 | append-only hash-chained audit events active; `ci-gate-proof-audit-ledger` local PASS |
| P12-13 | Bundle Exchange Protocol | COMPLETED_LOCAL | 2026-03-08 | local `ci-gate-proof-exchange` validates portable identity-preserving inline transport and mutation semantics |
| P12-14 | Cross-Node Verification Parity Suite | IN_PROGRESS | 2026-03-09 | local theorem-driven parity matrix now exercises match, subject, context, verifier-root, verifier-scope, historical, insufficient-evidence, verdict-guard, and receipt-absent cases |
| P12-15 | Multi-Signature / N-of-M Acceptance Policy | PLANNED | 2026-03-07 | quorum trust evaluation |
| P12-16 | `proofd` Userspace Verification Service | PLANNED | 2026-03-07 | long-running verification and receipt service |
| P12-17 | Replay Admission Boundary Contract | PLANNED | 2026-03-07 | accepted proof != automatic replay |
| P12-18 | Replicated Verification Research Track | PLANNED | 2026-03-07 | explicit bridge to Phase-13 without scope leak |

---

## 5. Documentation Sync Policy (Mandatory)

For every completed task, documentation MUST be updated in the same PR.

Minimum required updates:
- `docs/specs/phase12-trust-layer/tasks.md`
  - task status/progress
  - gate result summary
- `docs/specs/phase12-trust-layer/PROOF_BUNDLE_V2_SPEC.md`
  - schema or identity rule changes
- `docs/specs/phase12-trust-layer/PROOF_EXCHANGE_PROTOCOL_MESSAGE_FORMAT.md`
  - exchange message shape or transport identity rules
- `docs/specs/phase12-trust-layer/PROOF_VERIFIER_CRATE_ARCHITECTURE.md`
  - module, boundary, or core API changes
- `docs/specs/phase12-trust-layer/PROOF_BUNDLE_ATTACK_SURFACE_SECURITY_MODEL.md`
  - threat model, fail-closed rules, or hardening roadmap changes

Update when impacted:
- `docs/specs/phase12-trust-layer/AYKENOS_DISTRIBUTED_TRUTH_MODEL_FORMAL_SECURITY_PROPERTIES.md`
- `docs/specs/phase12-trust-layer/GENERIC_DETERMINISTIC_TRUTH_VERIFICATION_ARCHITECTURE.md`
- `docs/specs/phase12-trust-layer/PHASE12_SECURITY_MODEL_COMPARATIVE_ANALYSIS.md`
- `docs/specs/phase12-trust-layer/CROSS_NODE_PARITY_HARDENING_CHECKLIST.md`
- `docs/specs/phase12-trust-layer/N_NODE_CONVERGENCE_FORMAL_MODEL.md`
- `docs/specs/phase12-trust-layer/PARITY_LAYER_FORMAL_MODEL.md`
- `docs/specs/phase12-trust-layer/PARITY_LAYER_ARCHITECTURE.md`
- `docs/specs/phase12-trust-layer/PROOF_EXCHANGE_PROTOCOL_MESSAGE_FORMAT.md`
- `docs/specs/phase12-trust-layer/PROOF_VERIFIER_SEMANTIC_CLI_ROADMAP.md`
- `docs/specs/phase12-trust-layer/TRUTH_STABILITY_THEOREM.md`
- `docs/specs/phase12-trust-layer/VERIFICATION_CONVERGENCE_THEOREM.md`
- `docs/specs/phase12-trust-layer/design.md`
- `docs/specs/phase12-trust-layer/requirements.md`
- `docs/specs/phase12-trust-layer/VERIFICATION_CONTEXT_DISTRIBUTION_CONTRACT.md`
- `docs/specs/phase12-trust-layer/VERIFICATION_CONTEXT_OBJECT_SPEC.md`
- `docs/specs/phase12-trust-layer/VERIFICATION_CONTEXT_PORTABILITY_AND_DISTRIBUTION_PROTOCOL.md`
- `docs/specs/phase12-trust-layer/VERIFIER_ATTESTATION_AND_TRUST_REGISTRY_CONTRACT.md`
- `docs/specs/phase12-trust-layer/CROSS_NODE_PARITY_FAILURE_SEMANTICS_SPEC.md`
- `docs/specs/phase12-trust-layer/N_NODE_CONVERGENCE_FORMAL_MODEL.md`
- `docs/specs/phase12-trust-layer/PARITY_LAYER_FORMAL_MODEL.md`
- `docs/specs/phase12-trust-layer/VERIFIER_AUTHORITY_SEMANTICS_AND_DELEGATION_CONTRACT.md`
- `docs/specs/phase12-trust-layer/VERIFIER_REGISTRY_LINEAGE_AND_DISTRIBUTION_MODEL.md`
- `docs/specs/phase12-trust-layer/VERIFIER_AUTHORITY_GRAPH_CONSTRAINTS.md`
- `docs/specs/phase12-trust-layer/VERIFIER_AUTHORITY_RESOLUTION_ALGORITHM.md`
- `docs/security/PROOF_TRUST_POLICY.md`
- root-level operational files (e.g. `README.md`, `.github/workflows/ci-freeze.yml`, `Makefile`)

PR documentation rule:
- Every Phase-12 PR MUST include a `Documentation Delta` section in the PR body.
- If no doc changed, the PR MUST state explicit reason.

---

## 6. Language Selection Policy

Use the most suitable language per layer:
- **Rust**: verifier core, schema validation, canonicalization, signature verification, policy evaluation, receipt emission
- **Bash/Python**: CI gate orchestration, evidence generation, parity harnesses, report formatting
- **C**: none by default in Phase-12 core; Ring0 changes are out of scope unless separately ratified

Rules:
- Prefer Rust where parser/verifier correctness and deterministic behavior matter.
- Keep trust logic out of Ring0.
- Do not introduce kernel-side trust or signature logic in Phase-12A/P12B/P12C core milestones.

---

## 7. Security and Performance Control Plan

Each task PR MUST include both:
- **Security Check**
  - fail-closed behavior on malformed/tampered bundle or overlay input
  - no new privilege escalation path
  - no trust-policy leakage into Ring0
- **Performance Check**
  - verification runtime measured
  - receipt or registry lookup overhead measured where relevant
  - no regression to existing Phase-11 proof portability flow

Minimum commands before PR update:
- `make pre-ci`
- `make ci-gate-performance`
- task-specific Phase-12 gate(s) once implemented

---

## 8. Workstreams

### WS-A: Trusted Proof Bundle

#### T1 - P12-01 Producer Identity Schema
- Branch: `feat/p12-producer-identity-schema`
- Owner: Kenan AY
- Invariant: producer declaration is trust overlay, not portable identity
- Status: COMPLETED_LOCAL
- Deliverables:
  - `producer/producer.json` schema
  - producer versioning rules
  - `producer_key_epoch`
  - registry reference model
- Gate: `ci-gate-proof-producer-schema`
- Evidence:
  - `producer_schema_report.json`
  - `producer_identity_examples.json`
  - `report.json`
  - `violations.txt`

Scope note (normative for this milestone):
- Producer identity is declared inside the detached trust overlay.
- Producer identity MUST NOT change `bundle_id`.
- Local `ci-gate-proof-producer-schema` now exports canonical schema evidence and rotation-stability examples and passes locally.

#### T2 - P12-02 Detached Signature Envelope
- Branch: `feat/p12-detached-signature-envelope`
- Owner: Kenan AY
- Invariant: detached signature bytes MUST NOT mutate `bundle_id`
- Status: COMPLETED_LOCAL
- Deliverables:
  - `signatures/signature-envelope.json`
  - multi-signature-capable schema
  - signature algorithm field
  - signing time field
- Gate: `ci-gate-proof-signature-envelope`
- Evidence:
  - `signature_envelope_report.json`
  - `identity_stability_report.json`
  - `report.json`
  - `violations.txt`

Scope note (normative for this milestone):
- Signature format is detached and overlay-only.
- Multi-signature storage is in-envelope; acceptance semantics remain policy-defined.
- Local `ci-gate-proof-signature-envelope` now exports detached envelope structure and identity-stability evidence and passes locally.

#### T3 - P12-03 Proof Bundle v2 Layout
- Branch: `feat/p12-proof-bundle-v2-layout`
- Owner: Kenan AY
- Invariant: Phase-11 core payload naming and identity semantics remain stable
- Status: COMPLETED_LOCAL
- Deliverables:
  - `PROOF_BUNDLE_V2_SPEC.md`
  - canonical v2 directory tree
  - v1 -> v2 compatibility mapping
  - portable-core vs overlay boundary notes
- Gates:
  - `ci-gate-proof-bundle-v2-schema`
  - `ci-gate-proof-bundle-v2-compat`
- Evidence:
  - `bundle_schema_report.json`
  - `compatibility_report.json`
  - `report.json`
  - `violations.txt`

Scope note (normative for this milestone):
- Portable core remains `manifest.json`, `checksums.json`, `evidence/`, `traces/`, `reports/`, `meta/run.json`.
- Overlay directories extend the bundle; they do not redefine it.
- Local `ci-gate-proof-bundle-v2-schema` and `ci-gate-proof-bundle-v2-compat` now validate v2 layout and preserved Phase-11 portable-core boundaries.

#### T4 - P12-04 Signature Verification Gate
- Branch: `feat/p12-signature-verification-gate`
- Owner: Kenan AY
- Invariant: `verify(bundle_id, sig, pubkey) == PASS`
- Status: COMPLETED_LOCAL
- Deliverables:
  - `ci-gate-proof-signature-verify`
  - signature verifier harness
  - report and violation outputs
- Gate: `ci-gate-proof-signature-verify`
- Evidence:
  - `signature_verify.json`
  - `registry_resolution_report.json`
  - `report.json`
  - `violations.txt`

Scope note (normative for this milestone):
- Gate validates detached signatures after portable proof verification, not before it.
- Invalid signatures, revoked keys, and unresolved key IDs fail closed.
- Local `ci-gate-proof-signature-verify` now exports detached signature verification and registry-resolution evidence and passes locally.

#### T5 - P12-05 Producer Registry and Trust Root Inputs
- Branch: `feat/p12-producer-registry`
- Owner: Kenan AY
- Invariant: accepted producer MUST resolve through explicit trust registry
- Status: COMPLETED_LOCAL
- Deliverables:
  - registry snapshot schema
  - `registry_format_version`
  - `registry_snapshot_hash`
  - trust root input contract
- Gate: `ci-gate-proof-registry-resolution`
- Evidence:
  - `registry_snapshot.json`
  - `registry_resolution_matrix.json`
  - `report.json`
  - `violations.txt`
- Local `ci-gate-proof-registry-resolution` now covers active, ambiguous, unknown, and missing-material producer key resolution states and passes locally.

#### T6 - P12-06 Key Rotation and Revocation Contract
- Branch: `feat/p12-key-rotation-revocation`
- Owner: Kenan AY
- Invariant: key rotation MUST NOT break auditability of old proof bundles
- Status: COMPLETED_LOCAL
- Deliverables:
  - key epoch semantics
  - revocation format
  - deterministic lookup rules
- Gate: `ci-gate-proof-key-rotation`
- Evidence:
  - `rotation_matrix.json`
  - `revocation_matrix.json`
  - `report.json`
  - `violations.txt`
- Local `ci-gate-proof-key-rotation` now covers active, superseded, and revoked key lifecycle states and passes locally.

---

### WS-B: Verifier Layer

#### T7 - P12-07 Rust `proof-verifier` Core Crate
- Branch: `feat/p12-proof-verifier-core`
- Owner: Kenan AY
- Invariant: verifier remains outside kernel
- Status: COMPLETED_LOCAL
- Deliverables:
- `ayken-core/crates/proof-verifier/`
- `docs/specs/phase12-trust-layer/PROOF_VERIFIER_CRATE_ARCHITECTURE.md`
- `bundle/` loader + manifest/checksum parsing
- `portable_core/` checksum + proof-chain validation
- `overlay/` producer + signature envelope + overlay validation
- `crypto/` detached signature verification boundary + Ed25519 verifier
- `authority/` verifier-trust registry validation + authority graph constraints + deterministic authority resolution
- `policy/` policy schema + quorum + evaluation
- `verdict/` verdict subject + verdict engine
- `receipt/` receipt schema + emitter hooks
- Gate: `ci-gate-proof-verifier-core`
- Evidence:
  - `verifier_core_report.json`
  - `determinism_matrix.json`
  - `report.json`
  - `violations.txt`

Progress note:
- Library-first verifier crate is bootstrapped and `cargo test -p proof-verifier` passes.
- Detached signature verification now executes through `crypto/ed25519.rs` over `bundle_id`.
- Portable core hardening is active: strict `proof_manifest` binding checks, ledger/transcript root recomputation, and replay/report cross-consistency validation are in verifier core.
- Canonical `registry_snapshot_hash` recomputation and declared-vs-recomputed binding are now active in verifier core.
- Signed receipt payload canonicalization, Ed25519 signing, and signed receipt verification are now active in verifier core.
- Append-only audit event generation and ledger append path are now active in verifier core.
- Verifier-trust registry validation, explicit root-set handling, deterministic authority resolution, and canonical `authority_chain_id` emission are now active in verifier core.
- Signed receipt acceptance can now be bound to current verifier authority through verifier-trust registry resolution, authority-scope checks, and canonical `authority_chain_id` comparison.
- Local `ci-gate-verifier-authority-resolution` evidence now exercises signed receipt authority binding in addition to bare authority graph resolution.
- Delegation depth overflow now classifies deterministically as `AuthorityGraphDepthExceeded` instead of collapsing into generic no-valid-chain failure.
- Authority resolution now computes `effective_authority_scope` from the surviving chain semantics rather than copying requested scope verbatim.
- Authority tamper corpus now covers historical-only, revoked, orphan, scope-mismatch, algorithm-drift, key-material-drift, missing-`authority_chain_id`, and depth-overflow cases.
- Authority gate evidence now computes `authority_chain_id_equal` from real resolver-vs-receipt authority comparison instead of placeholder reporting.
- Portable-core proof validation now enforces proof-manifest mode/signature contract fields, digest-shape checks, and replay-trace hash bindings in addition to existing manifest hash recomputation.
- Local `ci-gate-cross-node-parity` evidence now emits a real `failure_matrix.json` and classifies delegated authority-chain drift as `PARITY_VERIFIER_MISMATCH` through `authority_chain_id_equal`.
- Portable-core negative coverage now includes proof-manifest `event_count`, `violation_count`, `proof_hash`, `replay_result_hash`, and `config_hash` / `kernel_image_hash` drift cases.
- Local `ci-gate-proof-verifier-core` evidence now exercises deterministic verifier-core behavior across trusted, rejected-by-policy, untrusted, detached-signature-invalid, and missing-manifest-invalid scenarios.
- Verification context portability and distribution protocol is now defined as a separate truth surface so future parity expansion and `proofd` transport can bind reconstructable context material instead of receipt-only exchange.
- Remaining verifier hardening work stays in full proof-manifest field coverage and broader negative corpus.

#### T8 - P12-08 Canonical Trust Policy Schema
- Branch: `feat/p12-trust-policy-schema`
- Owner: Kenan AY
- Invariant: policy MUST be hash-stable and deterministic
- Status: COMPLETED_LOCAL
- Deliverables:
  - trust policy schema
  - trusted producer list
  - trusted key list
  - required signature count/quorum surface
- Gate: `ci-gate-proof-trust-policy`
- Evidence:
  - `policy_schema_report.json`
  - `policy_hash_report.json`
  - `report.json`
  - `violations.txt`

Progress note:
- Local `ci-gate-proof-trust-policy` evidence now proves policy externality, canonical `policy_hash` stability, and deterministic verdict binding across trusted, untrusted, rejected-by-policy, and invalid-unsupported-quorum scenarios.
- Unsupported quorum semantics now fail closed through schema validation instead of remaining implicit policy ambiguity.

#### T9 - P12-09 Verdict Binding (`policy_hash`, `registry_snapshot_hash`)
- Branch: `feat/p12-verdict-binding`
- Owner: Kenan AY
- Invariant: verdict subject MUST include `bundle_id`, `trust_overlay_hash`, `policy_hash`, `registry_snapshot_hash`
- Status: COMPLETED_LOCAL
- Deliverables:
  - verdict subject definition
  - output contract for policy and registry hash binding
  - audit replay basis notes
- Gate: `ci-gate-proof-verdict-binding`
- Evidence:
  - `verdict_binding_report.json`
  - `verdict_subject_examples.json`
  - `report.json`
  - `violations.txt`

Progress note:
- Local `ci-gate-proof-verdict-binding` evidence now proves that `verdict_subject = (bundle_id, trust_overlay_hash, policy_hash, registry_snapshot_hash)` remains stable across repeated verification for the same inputs.
- Signed receipt emission now reuses the same four-field verdict binding tuple and the gate exports weaker tuple examples as explicitly disallowed for distributed claims.

#### T10 - P12-10 `proof-verifier` CLI
- Branch: `feat/p12-proof-verifier-cli`
- Owner: Kenan AY
- Invariant: CLI is a thin shell over deterministic verifier core
- Status: COMPLETED_LOCAL
- Deliverables:
  - CLI command surface
  - human-readable verdict output
  - machine-readable JSON output
- Gate: `ci-gate-proof-verifier-cli`
- Evidence:
  - `cli_smoke_report.json`
  - `cli_output_contract.json`
  - `report.json`
  - `violations.txt`

Progress note:
- Semantic CLI direction is now evaluated and staged in `PROOF_VERIFIER_SEMANTIC_CLI_ROADMAP.md`.
- Local `proof-verifier verify bundle <bundle> --policy <policy.json> --registry <registry.json> [--json]` is now active as a thin wrapper over `verify_bundle()`.
- Local `ci-gate-proof-verifier-cli` evidence proves offline CLI execution, human-readable verdict output, and JSON verdict binding output without implicit persistence.
- `COMPLETED_LOCAL` here is task-local only; it does not imply full Phase-12 closure while `P12-13+` distributed workstreams remain open.
- Receipt verification, parity comparison, rich inspection, and service-adjacent surfaces remain deferred into post-closure expansion so `P12-10` does not absorb `proofd` behavior.

#### T11 - P12-11 Verification Receipt / Acceptance Certificate
- Branch: `feat/p12-verification-receipt`
- Owner: Kenan AY
- Invariant: receipt is derived artifact, not bundle payload identity
- Status: COMPLETED_LOCAL
- Deliverables:
  - receipt schema
  - verifier signature format
  - receipt output path convention
- Gate: `ci-gate-proof-receipt`
- Evidence:
  - `receipt_schema_report.json`
  - `receipt_emit_report.json`
  - `report.json`
  - `violations.txt`

Progress note:
- Signed receipt payload canonicalization, Ed25519 receipt signing, and signed receipt verification are active in verifier core.
- Negative coverage includes tampered signature and stale subject mismatch rejection.
- `ci-gate-proof-receipt` now exports receipt schema/emission evidence and passes locally.
- Remaining work stays in expanded receipt tamper corpus and future service-level persistence.

#### T12 - P12-12 Verification Audit Ledger
- Branch: `feat/p12-verification-audit-ledger`
- Owner: Kenan AY
- Invariant: verification audit trail MUST remain immutable and attributable
- Status: COMPLETED_LOCAL
- Deliverables:
  - audit ledger schema
  - verification event record format
  - append-only log contract
- Gate: `ci-gate-proof-audit-ledger`
- Evidence:
  - `verification_audit_ledger.jsonl`
  - `audit_integrity_report.json`
  - `report.json`
  - `violations.txt`

Progress note:
- Audit events now hash-bind receipt output and subject tuple inside an append-only `previous_event_hash` chain.
- Verifier core can append audit events through serialized append operations and verify ledger integrity/tamper conditions.
- Audit verification now includes signed receipt verification when receipt binding material is available.
- `ci-gate-proof-audit-ledger` now exports ledger/integrity evidence and passes locally.
- Remaining work stays in expanded tamper corpus and future multi-node audit federation.

---

### WS-C: Distributed Verification

#### T13 - P12-13 Bundle Exchange Protocol
- Branch: `feat/p12-bundle-exchange-protocol`
- Owner: Kenan AY
- Invariant: transport MUST NOT mutate payload identity
- Status: COMPLETED_LOCAL
- Deliverables:
  - exchange message format
  - verification context portability protocol
  - payload/overlay/receipt separation
  - transport contract notes
- Gate: `ci-gate-proof-exchange`
- Evidence:
  - `exchange_contract_report.json`
  - `transport_mutation_matrix.json`
  - `report.json`
  - `violations.txt`

Progress note:
- Local `ci-gate-proof-exchange` evidence is now active and validates a real inline exchange package with explicit payload / overlay / verification-context / receipt separation.
- The transport mutation matrix now proves metadata-only mutation is non-authoritative while payload, overlay, context, and receipt-subject drift fail closed.
- `PROOF_EXCHANGE_PROTOCOL_MESSAGE_FORMAT.md` now defines the local message shape used by the gate.
- `COMPLETED_LOCAL` here is task-local only; full `Phase-12` closure remains blocked on `P12-14+` distributed workstreams.

#### T14 - P12-14 Cross-Node Verification Parity Suite
- Branch: `feat/p12-cross-node-parity`
- Owner: Kenan AY
- Invariant: distributed verification parity MUST be deterministic
- Status: IN_PROGRESS
- Deliverables:
  - node A/B/C verification parity tests
  - parity report
  - failure matrix
- Gate: `ci-gate-cross-node-parity`
- Evidence:
  - `parity_report.json`
  - `parity_consistency_report.json`
  - `parity_determinism_report.json`
  - `parity_determinism_incidents.json`
  - `parity_convergence_report.json`
  - `parity_drift_attribution_report.json`
  - `failure_matrix.json`
  - `report.json`
  - `violations.txt`

Progress note:
- The local parity gate now exercises a ten-scenario hardening slice: baseline `PARITY_MATCH`, `PARITY_SUBJECT_MISMATCH`, two `PARITY_CONTEXT_MISMATCH` variants (`verification_context_id` drift and verifier-contract-version drift), two `PARITY_VERIFIER_MISMATCH` variants (trusted-root drift and authority-scope drift), `PARITY_HISTORICAL_ONLY`, `PARITY_INSUFFICIENT_EVIDENCE`, `PARITY_VERDICT_MISMATCH`, and an explicit receipt-absent parity-artifact path.
- Scenario-specific evidence is now exported under `scenario_reports/` alongside the matrix-level artifacts.
- The local gate now exports `parity_consistency_report.json` and `parity_determinism_report.json` so ordinary distributed drift and deterministic model-alarm surfaces are reported separately.
- The local gate now also exports `parity_determinism_incidents.json`, lifting same-`D_i` / different-`K_i` conditions into first-class `DeterminismIncident` objects with stable hash-based `incident_id` values instead of leaving them implicit inside pairwise rows.
- The local gate now also exports `parity_convergence_report.json` as a node-derived aggregate built from stable `NodeParityOutcome` objects plus `D_i` / `K_i` partitions, while preserving the underlying pairwise classifier and raw `failure_matrix.json`.
- `NodeParityOutcome` generation is now crate-owned through `authority/parity.rs`; `surface_key` and `outcome_key` are no longer treated as ad hoc harness-computed fields.
- The local gate now also exports `parity_drift_attribution_report.json`, explaining each node-derived surface partition in terms of subject/context/authority/verdict/evidence drift relative to the dominant surface.
- The local drift-attribution artifact now also reports cluster-level `historical_authority_islands` and `insufficient_evidence_islands`, so Phase-12 diagnostics can distinguish isolated epoch/evidence lag from ordinary partition counts.
- The current matrix now makes the receipt-absent artifact contract explicit through `local_verification_outcome` rather than silently depending on receipt transport.
- `CROSS_NODE_PARITY_HARDENING_CHECKLIST.md` now defines the broader hardening matrix, including remaining subject/context/authority drift and full matrix aggregation scenarios beyond the active local slice.
- `P12-14` remains open until the parity suite moves beyond the current minimal failure matrix into the broader theorem-driven scenario set.

#### T15 - P12-15 Multi-Signature / N-of-M Acceptance Policy
- Branch: `feat/p12-multisig-quorum`
- Owner: Kenan AY
- Invariant: quorum policy evaluation MUST be deterministic
- Status: PLANNED
- Deliverables:
  - quorum policy schema
  - quorum evaluator
  - multi-signature test matrix
- Gate: `ci-gate-proof-multisig-quorum`
- Evidence:
  - `quorum_matrix.json`
  - `quorum_evaluator_report.json`
  - `report.json`
  - `violations.txt`

#### T16 - P12-16 `proofd` Userspace Verification Service
- Branch: `feat/p12-proofd-service`
- Owner: Kenan AY
- Invariant: distributed acceptance remains userspace/policy layer
- Status: PLANNED
- Deliverables:
  - `userspace/proofd/`
  - bundle intake
  - verification execution
  - receipt emission
  - policy application
- Gate: `ci-gate-proofd-service`
- Evidence:
  - `proofd_service_report.json`
  - `proofd_receipt_report.json`
  - `report.json`
  - `violations.txt`

#### T17 - P12-17 Replay Admission Boundary Contract
- Branch: `feat/p12-replay-admission-boundary`
- Owner: Kenan AY
- Invariant: accepted proof and replicated replay are distinct concerns
- Status: PLANNED
- Deliverables:
  - replay admission rules
  - verifier/replay interface contract
  - boundary statement
- Gate: `ci-gate-proof-replay-admission-boundary`
- Evidence:
  - `replay_admission_report.json`
  - `boundary_contract.json`
  - `report.json`
  - `violations.txt`

#### T18 - P12-18 Replicated Verification Research Track
- Branch: `research/p12-replicated-verification-boundary`
- Owner: Kenan AY
- Invariant: replicated replay MUST NOT leak into P12A/P12B/P12C core closure criteria
- Status: PLANNED
- Deliverables:
  - research-track note
  - explicit non-goals
  - Phase-13 bridge note
- Gate: `ci-gate-proof-replicated-verification-boundary`
- Evidence:
  - `research_boundary_note.md`
  - `phase13_bridge_report.json`
  - `report.json`
  - `violations.txt`

---

## 9. Repository Mapping

Docs:
- `docs/specs/phase12-trust-layer/design.md`
- `docs/specs/phase12-trust-layer/requirements.md`
- `docs/specs/phase12-trust-layer/tasks.md`
- `docs/specs/phase12-trust-layer/PROOF_BUNDLE_V2_SPEC.md`
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

Security:
- `docs/security/PROOF_TRUST_POLICY.md`

Rust:
- `ayken-core/crates/proof-verifier/`

Userspace:
- `userspace/proofd/`

---

## 10. Dependency Order

Core trusted proof path:
1. P12-01
2. P12-02
3. P12-03
4. P12-04
5. P12-05
6. P12-06
7. P12-07
8. P12-08
9. P12-09
10. P12-10
11. P12-11
12. P12-12

Distributed verification path:
1. P12-13
2. P12-14
3. P12-15
4. P12-16
5. P12-17

Research path:
1. P12-18

---

## 11. Validation Checklist (Per PR)

- [ ] Invariant clearly stated in PR body
- [ ] One CI gate mapped to invariant
- [ ] Evidence artifacts present and complete
- [ ] Negative tests included
- [ ] Fail-closed behavior verified
- [ ] No policy leakage into Ring0
- [ ] `bundle_id` semantics preserved
- [ ] Documentation Delta section added and complete
- [ ] Security check completed and summarized
- [ ] Performance check completed and summarized
- [ ] Language choice justified (Rust/Bash/Python)

---

## 12. Planned Local Pre-merge Commands

Run before pushing once the relevant task gates exist:

```bash
make pre-ci
make ci-gate-performance
make ci-gate-proof-producer-schema
make ci-gate-proof-signature-envelope
make ci-gate-proof-bundle-v2-schema
make ci-gate-proof-bundle-v2-compat
make ci-gate-proof-signature-verify
make ci-gate-proof-registry-resolution
make ci-gate-proof-key-rotation
make ci-gate-proof-verifier-core
make ci-gate-proof-trust-policy
make ci-gate-proof-verdict-binding
make ci-gate-proof-verifier-cli
make ci-gate-proof-receipt
make ci-gate-proof-audit-ledger
make ci-gate-verifier-authority-resolution
make ci-gate-proof-exchange
make ci-gate-cross-node-parity
make ci-gate-proof-multisig-quorum
make ci-gate-proofd-service
make ci-gate-proof-replay-admission-boundary
make ci-gate-proof-replicated-verification-boundary
```

Add component-specific gate(s) from the issue under implementation.

---

## 13. Closure Criteria

### Phase-12A Closure

Satisfied when:
- producer schema defined
- detached signature envelope defined
- proof bundle v2 layout documented
- signature verification gate passes
- trust registry resolution works
- key rotation/revocation contract passes

### Phase-12B Closure

Satisfied when:
- `proof-verifier` crate works
- trust policy schema defined
- `policy_hash` + `registry_snapshot_hash` bind verdict
- verifier CLI works
- receipt generation works
- verification audit ledger works

### Phase-12C Closure

Satisfied when:
- bundle exchange protocol defined
- cross-node parity suite passes
- multi-signature policy works
- `proofd` service works
- replay admission boundary documented
- replicated verification remains outside Phase-12 core

---

## 14. Non-Goals

Phase-12 does NOT:
- modify Ring0 runtime behavior
- move verifier into kernel
- redefine Phase-11 portable identity
- introduce consensus
- implement replicated execution

---

## 15. Summary

Phase-12 advances AykenOS from:

`portable proof`

to:

`trusted proof`

without mutating Phase-11 portable bundle identity.

Architectural ladder:
- Phase-11 -> proof portability
- Phase-12 -> trust transport + deterministic verification
- Phase-13+ -> replicated verification / distributed replay boundary

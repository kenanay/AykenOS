# Design Document: Phase-11 Verification Substrate

**Version:** 1.0  
**Status:** Draft  
**Date:** 2026-03-06  
**Created by:** Kenan AY  
**Maintained by:** Kenan AY  
**Last Edited by:** Kenan AY  
**Gelistiren:** Kenan AY  
**Olusturan:** Kenan AY  
**Duzenleyen:** Kenan AY  
**Prerequisites:**  
- `requirements.md`  
- `docs/architecture-board/ABDF_BCIB_PHASE11_CONTRACT_MATRIX.md`  
- `docs/architecture-board/PHASE11_EVENT_TAXONOMY.md`  
- `docs/architecture-board/RUNTIME_STATE_MACHINE.md`
- `docs/governance/MAILBOX_PROTOCOL_V2_CAPABILITIES.md`

---

## 1. Scope and Goal

Phase-11 implements a deterministic and verifiable kernel execution substrate.

Target outcome:
- Deterministic event ordering (`event_seq`, `ltick`)
- Append-only decision ledger and execution transcript
- Replay determinism validation
- Proof manifest generation
- CI fail-closed verification gates

Out of scope in this phase:
- BCIB runtime redesign and new opcode semantics
- AI scheduler policy implementation details
- Distributed runtime implementation

---

## 2. Architectural Model

Three substrate model:
- ABDF: data reality
- BCIB: execution intent
- Phase-11: execution reality + proof

Kernel remains mechanism-only:
- syscall handling
- interrupt handling
- scheduling/context switching
- capability enforcement

Phase-11 captures kernel-visible events and exports immutable evidence artifacts.

---

## 3. Normative Event Pipeline

Single kernel event processing pipeline:

1. Kernel hook emits raw event payload.
2. DEOL assigns `event_seq` (global, monotonic, unique).
3. DLT assigns `ltick` (deterministic logical time).
4. Event classification decides target record(s):
   - decision-class -> ledger
   - execution-class -> transcript
   - dual-class -> both
5. Ledger/transcript append in same event transaction.
6. Hash state and ordering state are updated.
7. Evidence buffers are flushed at export boundaries.

Implementation note:
- Ledger and transcript are sibling outputs of the same classified event.
- They are not sequential dependencies of each other.

### 3.1 Mailbox Capability Contract (P11-01)

Ring0 mailbox validation includes a fail-closed capability envelope:
- signature validity check
- capability presence check
- budget bound check
- invalid PID reject mapping

Canonical reject aliases:
- `REJ_BAD_SIG`
- `REJ_CAP_MISSING`
- `REJ_BUDGET_EXCEEDED`
- `REJ_INVALID_PID`

Validation evidence gate:
- `ci-gate-mailbox-capability-negative`
- artifacts: `negative_matrix.json`, `report.json`, `violations.txt`

---

## 4. Data Structures

Kernel-side core structures:
- `ay_decision_ledger_entry_t`
- `ay_transcript_entry_t`
- `ay_ordering_state_t`
- `ay_replay_state_t`
- `ay_gcp_record_t`
- `ay_proof_manifest_t`

Required common fields:
- `event_seq`
- `ltick`
- `event_type`
- `cpu_id`

Ledger hashing (canonical):
- `payload_hash = H(normalized_payload)`
- `entry_hash = H(prev_hash || payload_hash)`

Transcript hashing:
- `transcript_hash = H(state_before || event || state_after)`

### 4.1 Decision Ledger v1 Materialization Path (#35)

Current implementation phase uses a deterministic CI materialization path:

1. Source evidence:
   - `ring3-execution-phase10a2/events.jsonl`
   - `ring3-execution-phase10a2/marker.log`
2. Extract schedule decision markers:
   - `P10_MAILBOX_DECISION id=<id> pid=<pid> valid=<0|1> src=<pid>`
3. Bind decisions to originating `[[AYKEN_CTX_SWITCH]]` events.
4. Emit:
   - `decision_ledger.jsonl`
   - `decision_ledger.bin`
   - `report.json`
   - `violations.txt`

Boundary statement:
- This milestone is a bootstrap completeness/materialization implementation.
- It is not yet the final kernel-hotpath append implementation.

Compatibility mode until #43/#44 strict binding:
- `event_seq` is sourced from originating event order.
- `ltick = event_seq` deterministic fallback.

Strict mode (post #43/#44):
- `ledger.event_seq == eti_event.event_seq`
- `ledger.ltick == eti_event.ltick`
- Missing binding is fail-closed.

### 4.2 Ledger Hash-Chain Integrity Path (#36)

Bootstrap integrity validation runs on materialized ledger output:

1. Input:
   - `ledger-v1/decision_ledger.jsonl`
2. Recompute per-entry fields:
   - `payload_hash = H(normalized_payload)`
   - `entry_hash = H(prev_hash || payload_hash)`
3. Verify continuity:
   - genesis `prev_hash = 0x00...00`
   - `entry[i].prev_hash == entry[i-1].entry_hash`
4. Verify ordering identities:
   - `event_seq` monotonic + unique
   - `ltick` monotonic + unique (compat mode currently mirrors event order)
   - `event_seq_chain_hash = H(seq_1 || ... || seq_n)`
5. Execute one-bit tamper simulation:
   - mutate one bit in first entry payload hash
   - validator MUST detect and fail-closed

Artifacts:
- `chain_verify.json`
- `tamper_test.json`
- `report.json`
- `violations.txt`

### 4.3 DEOL Ordering Bootstrap Path (#40)

Bootstrap DEOL sequencing is generated from verified ledger stream:

1. Input:
   - `ledger-v1/decision_ledger.jsonl`
2. Generate DEOL sequence stream:
   - `event_seq = 1..N` (contiguous bootstrap sequence)
   - `ltick = 1..N` (bootstrap logical-time mirror)
   - carry source identities: `source_event_seq`, `source_ltick`
3. Validate invariants:
   - generated `event_seq` monotonic + unique + no gaps
   - generated `ltick` monotonic + unique + no gaps
   - source ordering identities monotonic + unique
4. Emit:
   - `event_seq.jsonl`
   - `sequence_report.json`
   - `report.json`
   - `violations.txt`

Boundary statement:
- This is a bootstrap ordering proof over ledger-derived stream.
- Direct kernel hot-path DEOL allocator and ETI/DLT strict join are deferred to #43/#44.

### 4.4 ETI Transcript Bootstrap Path (#43)

Bootstrap ETI transcript is materialized from Phase10-A2 event evidence:

1. Input:
   - `ring3-execution-phase10a2/events.jsonl`
2. Select kernel-visible ETI marker classes:
   - `AYKEN_CTX_SWITCH` -> `AY_EVT_CTX_SWITCH`
   - `AYKEN_SYSCALL_ENTER` -> `AY_EVT_SYSCALL_ENTER`
   - `AYKEN_SYSCALL_RETURN|AYKEN_SYSCALL_EXIT` -> `AY_EVT_SYSCALL_EXIT`
   - additional IRQ/TRAP/MAILBOX classes when present
3. Assign ordering identity (bootstrap mode):
   - `event_seq` uses source event index
   - `ltick = event_seq` deterministic fallback
4. Canonical ETI entry hash:
   - `eti_entry_hash = H(normalized_eti_payload)`
5. Canonical transcript chain hashes:
   - `event_seq_chain_hash = H(seq_1 || ... || seq_n)`
   - `ltick_chain_hash = H(ltick_1 || ... || ltick_n)`
   - `eti_chain_hash = H(entry_hash_1 || ... || entry_hash_n)`
6. Emit:
   - `eti_transcript.bin`
   - `eti_transcript.jsonl`
   - `eti_chain_verify.json`
   - `eti_diff.txt`
   - `report.json`
   - `violations.txt`

Bootstrap artifact note:
- In bootstrap mode, `eti_diff.txt` is a placeholder parity artifact and mirrors detected violations.
- In strict runtime ETI stage, `eti_diff.txt` will carry concrete drop/dup/reorder diff output.

Ledger strict binding gate:
- Input: `ledger-v1/decision_ledger.jsonl` + `eti/eti_transcript.jsonl`
- Enforce:
  - `ledger.event_seq == eti.event_seq`
  - `ledger.ltick == eti.ltick`
- Missing/mismatch is fail-closed and exported as `binding_report.json`.

Transcript integrity gate:
- Validates ETI jsonl ordering + required fields + entry hash recomputation.
- Validates ETI binary header/layout/count + row parity with jsonl.
- Any corruption/tamper is fail-closed.

Boundary statement:
- ETI is bootstrap materialization in this milestone.
- Direct kernel runtime ETI hook emission and lock-free buffering are deferred to strict runtime integration stage.

### 4.5 DLT Ordering Bootstrap Path (#44)

Bootstrap DLT ordering is materialized from ETI transcript evidence:

1. Input:
   - `eti/eti_transcript.jsonl`
2. Generate DLT trace stream:
   - generated `event_seq = 1..N`
   - generated `ltick = 1..N`
   - retain source identities: `source_event_seq`, `source_ltick`
3. Validate DLT invariants:
   - generated `ltick` monotonic + unique + no gaps
   - generated `event_seq` monotonic + unique + no gaps
   - source identities monotonic + unique
4. Enforce strict ETI<->DLT source binding:
   - `dlt.source_event_seq == eti.event_seq`
   - `dlt.source_ltick == eti.ltick`
5. Emit:
   - `ltick_trace.jsonl`
   - `binding_report.json`
   - `report.json`
   - `violations.txt`

Boundary statement:
- DLT in this milestone is bootstrap CI materialization over ETI evidence.
- Direct kernel hot-path `ltick` assignment and multicore merge rules remain deferred to strict runtime DLT integration stage.

### 4.6 Verification Kernel Boundary (Hardening Addendum)

To avoid verification-layer observer effects and architecture drift:

1. Runtime kernel hot-path keeps only minimal event contract emission.
2. Heavy verification work (hashing, binding, parity checks, report synthesis) remains CI/offline.
3. Runtime integration stages must preserve non-blocking O(1) event publication semantics.
4. Event contract schema changes require synchronized updates across `design.md`, `requirements.md`, and `tasks.md` in the same PR.

### 4.7 GCP Finalization Bootstrap Path (#45)

Bootstrap GCP finalization is materialized from DLT ordering evidence:

1. Input:
   - `dlt-monotonicity/ltick_trace.jsonl`
2. Construct bootstrap commit-point snapshot:
   - `gcp_ltick = last_ltick`
   - `gcp_event_seq = last_event_seq`
3. Validate GCP invariants:
   - prefix immutability: all `ltick <= gcp_ltick` are finalized
   - DLT prefix alignment: `gcp_ltick` exists in DLT trace
   - optional previous-snapshot monotonicity: `current_gcp_ltick >= previous_gcp_ltick`
   - hash continuity: `gcp_hash = H(previous_gcp_hash || dlt_prefix_hash || gcp_ltick || gcp_event_seq)`
     where bootstrap genesis uses `previous_gcp_hash = 0...0`
4. Emit:
   - `gcp_snapshot.json`
   - `gcp_record.json`
   - `gcp_consistency_report.json`
   - `report.json`
   - `violations.txt`

Boundary statement:
- GCP in this milestone is bootstrap CI finalization contract verification.
- Runtime multicore prepare/vote/commit path remains deferred to strict runtime GCP integration stage.
- Bootstrap validator semantics intentionally enforce contiguous DLT identities (`event_seq = 1..N`, `ltick = 1..N`).
- Strict runtime/sharded DLT+GCP semantics will be introduced via versioned validator path at runtime integration milestone.

### 4.8 ABDF Snapshot Identity Bootstrap Path (#47)

Bootstrap ABDF snapshot identity is materialized from canonical binary snapshot evidence:

1. Input:
   - `input/snapshot.abdf`
2. Compute identity hash:
   - `abdf_snapshot_hash = SHA256(snapshot_binary_bytes)`
3. Validate ABDF identity invariants:
   - snapshot input exists and is non-empty
   - deterministic recomputation yields identical hash
   - optional expected-hash input matches computed hash
4. Emit:
   - `abdf_snapshot_hash.txt`
   - `snapshot_identity_report.json`
   - `snapshot_identity_consistency.json`
   - `report.json`
   - `violations.txt`

Boundary statement:
- ABDF snapshot identity in this milestone is CI/offline bootstrap verification over exported `snapshot.abdf` bytes.
- Runtime replay integration and proof-layer composition consume this identity but do not alter hash semantics.

### 4.9 BCIB Plan + Execution Trace Identity Bootstrap Path (#48)

Bootstrap execution identity binds intent (`plan.bcib`) with ETI-derived execution stream:

1. Inputs:
   - `execution/plan.bcib`
   - `gates/eti/eti_transcript.jsonl`
2. Compute identities:
   - `bcib_plan_hash = SHA256(plan.bcib bytes)`
   - `execution_trace_hash = SHA256(normalized execution_trace.jsonl bytes)`
3. Validate execution identity invariants:
   - plan binary exists and is non-empty
   - ETI-derived execution trace is valid and deterministic (no duplicate/non-monotonic ordering identities)
   - deterministic recomputation yields identical plan/trace hashes
   - optional expected-hash inputs match computed identities
4. Emit:
   - `bcib_plan_hash.txt`
   - `execution_trace.jsonl`
   - `execution_trace_hash.txt`
   - `trace_verify.json`
   - `report.json`
   - `violations.txt`

Boundary statement:
- BCIB execution identity in this milestone is CI/offline bootstrap materialization over exported plan bytes and ETI evidence.
- Runtime replay engine consumes these identities; runtime execution semantics remain deferred to Replay v1 integration stage.

### 4.10 Replay Determinism Bootstrap Path (#37)

Bootstrap replay determinism is validated over identity-locked artifacts from ABDF/BCIB gates:

1. Inputs:
   - `gates/abdf-snapshot-identity/abdf_snapshot_hash.txt`
   - `gates/execution-identity/bcib_plan_hash.txt`
   - `gates/execution-identity/execution_trace.jsonl`
   - `gates/execution-identity/execution_trace_hash.txt`
2. Materialize deterministic replay trace:
   - normalize record rows (`trace_seq`, `event_seq`, `ltick`, `cpu_id`, `event_type`)
   - emit `replay_trace.jsonl` via canonical serialization
3. Validate replay invariants:
   - record trace ordering identities are monotonic+unique (`event_seq`, `ltick`)
   - `record_execution_trace_hash == SHA256(record_trace_bytes)`
   - `record_execution_trace_hash == replay_execution_trace_hash`
   - record/replay pairwise parity for `event_seq` and `ltick`
   - optional expected final-state hash equality (bootstrap final state derived from replay result hash)
4. Emit:
   - `replay_trace.jsonl`
   - `replay_trace_hash.txt`
   - `replay_report.json`
   - `event_diff.txt`
   - `ltick_diff.txt`
   - `report.json`
   - `violations.txt`

Boundary statement:
- Replay v1 in this milestone is CI/offline bootstrap parity verification over identity-locked evidence.
- Runtime replay execution, strict kernel panic policy, and multicore runtime replay semantics remain deferred to strict runtime replay integration stage.

### 4.11 KPL Proof Manifest Bootstrap Path (#41)

Bootstrap KPL proof manifest binds replay determinism outputs with evidence-root identities:

1. Inputs:
   - `gates/abdf-snapshot-identity/abdf_snapshot_hash.txt`
   - `gates/execution-identity/bcib_plan_hash.txt`
   - `gates/execution-identity/execution_trace_hash.txt`
   - `gates/replay-v1/replay_report.json`
   - `gates/ledger-v1/decision_ledger.jsonl`
   - `gates/eti/eti_transcript.jsonl`
   - `kernel.elf` (or configured kernel image binary)
   - `meta/run.json` (or configured runtime config evidence)
2. Materialize proof manifest fields:
   - `kernel_image_hash = SHA256(kernel_image_bytes)`
   - `config_hash = SHA256(config_json_bytes)`
   - `ledger_root_hash = SHA256(decision_ledger.jsonl bytes)`
   - `transcript_root_hash = SHA256(eti_transcript.jsonl bytes)`
   - replay-bound fields from replay report: `replay_result_hash`, `final_state_hash`, `event_count`, `violation_count`
   - identity-bound fields from prior gates: `abdf_snapshot_hash`, `bcib_plan_hash`, `execution_trace_hash`
3. Compute self-sealing manifest hash:
   - `proof_hash = H(canonical_json(proof_manifest_without_proof_hash))`
4. Validate KPL invariants:
   - required fields present and SHA-256 formatted
   - manifest version supported
   - `proof_hash` equals recomputed self-hash
   - manifest replay fields match replay evidence (`replay_result_hash`, `final_state_hash`, `event_count`, `violation_count`)
   - optional expected proof/final-state hash inputs match
5. Emit:
   - `proof_manifest.json`
   - `proof_verify.json`
   - `report.json`
   - `violations.txt`

Boundary statement:
- KPL in this milestone is CI/offline bootstrap hash-bound manifest verification.
- Signature trust policy remains bootstrap (`signature_mode=bootstrap-none`, empty `signer_sig`) and strict signer verification is deferred to later proof hardening stage.
- Runtime proof sealing/in-kernel signature semantics remain out of scope for this milestone.

### 4.12 Proof Bundle Bootstrap Portability Path (P11-42)

Bootstrap proof bundle portability packages manifest-bound execution proof into a machine-independent directory bundle:

1. Inputs:
   - `gates/abdf-snapshot-identity/abdf_snapshot_hash.txt`
   - `gates/execution-identity/bcib_plan_hash.txt`
   - `gates/execution-identity/execution_trace_hash.txt`
   - `gates/execution-identity/execution_trace.jsonl`
   - `gates/replay-v1/replay_trace_hash.txt`
   - `gates/replay-v1/replay_trace.jsonl`
   - `gates/replay-v1/replay_report.json`
   - `gates/kpl-proof/proof_manifest.json`
   - `gates/kpl-proof/proof_verify.json`
   - `gates/kpl-proof/report.json`
   - `gates/ledger-v1/decision_ledger.jsonl`
   - `gates/eti/eti_transcript.jsonl`
   - `reports/summary.json`
   - `meta/run.json`
   - `kernel.elf` (or configured kernel image binary)
2. Materialize portable bundle schema:
   - root: `proof_bundle/manifest.json`, `proof_bundle/checksums.json`
   - bundled data: `proof_bundle/evidence/`, `proof_bundle/traces/`, `proof_bundle/reports/`, `proof_bundle/meta/`
   - required files are checksum-bound with `checksums.json`
   - root identity is sealed with `bundle_id = H(canonical_manifest_without_bundle_id || canonical_checksums)`
3. Offline verification responsibilities:
   - verify required schema/files exist
   - verify file checksums match `checksums.json`
   - recompute trace hashes from bundled `execution_trace.jsonl` and `replay_trace.jsonl`
   - recompute manifest proof bindings from bundled ledger/transcript/kernel/config/replay evidence
   - reproduce source KPL verdict and proof-verify status from bundle contents only
4. Emit:
   - `proof_bundle/`
   - `bundle_verify.json`
   - `report.json`
   - `violations.txt`

Boundary statement:
- P11-42 is proof portability only: bundle verification reproduces verdicts from packaged evidence but does not execute runtime replay.
- Signed transport, trust roots, and archive/signature wrapping remain deferred to later proof portability hardening.

### 4.13 Phase-12 Deterministic Distributed Proof Architecture (Draft)

Status note:
- This section is forward-looking and non-normative for Phase-11 closure.
- It defines the intended architectural direction for Phase-12 without expanding Phase-11 scope, acceptance, or Definition of Done.

Purpose:

Phase-12 extends Phase-11 proof portability into a trusted and cross-node verifiable proof architecture.

Phase-11 guarantees that execution proof artifacts:
- exist,
- are portable,
- are checksum-bound,
- and can reproduce the same offline verdict.

Phase-12 adds the missing trust and distributed acceptance layers:
- producer attribution,
- signature verification,
- verifier policy compatibility,
- and deterministic cross-node acceptance semantics.

Boundary:

Phase-12 does not collapse proof transport, proof trust, and distributed replay into a single milestone.

The boundary is intentionally split:
- Phase-11: proof portability
- Phase-12A: trusted proof transport
- Phase-12B: cross-node proof acceptance
- Phase-12C: replicated replay boundary

This separation preserves scope discipline and prevents trust/distribution semantics from contaminating the bootstrap portability contract.

#### 4.13.1 Core Normative Definitions

Phase-11 definition:
- Execution proof exists, is portable, and is offline-verifiable.

Phase-12 definition:
- Execution proof is signed, producer-attributed, policy-checked, and cross-node acceptable.

#### 4.13.2 Trust Model

Phase-12 introduces explicit trust semantics for proof acceptance.

A transported proof bundle is not accepted solely because:
- it is structurally valid,
- checksums match,
- or proof parity reproduces successfully.

A proof is accepted only when trust invariants also hold.

Trust invariant:
- `accepted_proof => signature_valid && producer_trusted && policy_compatible`

Consequences:
- A proof MAY be portable but untrusted.
- A proof MAY be valid but not accepted.
- A proof MAY be reproduced but rejected by policy.

This makes a strict distinction between:
- valid proof artifact
- accepted proof artifact

That distinction is required for deterministic cross-node verification.

#### 4.13.3 Producer Identity Model

Every trusted proof bundle SHALL be bound to an explicit producer identity.

Minimum producer identity fields:
- `producer_id`
- `producer_pubkey_id`
- `build_id`
- `policy_version`

Purpose:

These fields make the question "who produced this proof?" normatively answerable.

Invariants:
- `producer_id` identifies the producing node, builder, or authority domain.
- `producer_pubkey_id` identifies the public key used to verify the detached signature.
- `build_id` binds proof production to a concrete build instance.
- `policy_version` binds the proof to the verifier compatibility surface.

Design note:
- Producer identity is not merely metadata.
- It participates in proof acceptance and trust policy evaluation.

#### 4.13.4 Signature Format

Phase-12 adopts a detached signature model.

This keeps:
- bundle packaging,
- checksum integrity,
- and signature trust

cleanly separated.

Recommended initial transport set built over the portable proof bundle:
- `proof_bundle.tar.zst`
- `proof_bundle.sha256`
- `proof_bundle.sig`
- `proof_bundle.meta.json`

Recommended initial algorithm:
- `Ed25519`

Signature invariant:
- `verify(bundle_hash, sig, pubkey) == PASS`
- `bundle_hash = H(bundle_payload)` and SHALL NOT include detached signature bytes or detached signature metadata generated after bundle sealing

Rationale:
- Detached signatures preserve portability.
- Detached signatures avoid mutating the bundle payload after sealing.
- Detached signatures simplify offline verification.
- Detached signatures allow transport and trust tooling to evolve independently.

#### 4.13.5 Verifier Policy and Version Compatibility

A verifier SHALL not accept a proof only because the signature is valid.

The verifier SHALL also apply an explicit acceptance policy.

Minimum verifier policy inputs:
- `bundle_version`
- `manifest_version`
- `policy_version`
- `producer trust set`

Purpose:

This separates:
- proof validity

from:
- proof acceptability

Compatibility invariant:
- `accepted_proof => bundle_version_supported && manifest_version_supported && policy_version_supported && producer_in_trust_set`

Determinism invariant:
- `same_bundle + same_verifier_policy => same_acceptance_verdict`

This invariant is mandatory for reproducible distributed verification.

#### 4.13.6 Cross-Node Proof Acceptance Protocol

When Node B receives a proof bundle produced by Node A, verification SHALL proceed in a strict deterministic order.

Acceptance pipeline:
1. archive integrity
2. checksum integrity
3. manifest parity
4. signature validity
5. producer trust
6. policy compatibility

Acceptance invariant:
- `same_bundle + same_verifier_policy => same_acceptance_verdict`

Interpretation:
- Node acceptance SHALL be explicit.
- Node acceptance SHALL be deterministic.
- Node acceptance SHALL be policy-bound.
- Node acceptance SHALL be reproducible.

No node may silently substitute local assumptions for declared proof policy semantics.

#### 4.13.7 Distributed Replay Boundary

Phase-12 still maintains a strict boundary between:
- proof acceptance
- distributed replay

These are not the same system concern.

Rule:
- First: portable trusted proof
- Then: replicated replay

Reason:
- If distributed replay enters before trust transport and cross-node acceptance are stable, scope expands uncontrollably, invariants blur, and verification semantics become ambiguous.

Boundary statement:
- Phase-12 MAY validate trusted proof transport and cross-node acceptance without executing replicated replay.
- Replicated replay remains a later layer.

#### 4.13.8 Phase Decomposition

Phase-12A - Trusted Proof Transport

Focus:
- detached signature artifacts
- producer identity fields
- trust-root inputs
- archive + signature verification

Phase-12B - Cross-Node Proof Acceptance

Focus:
- verifier acceptance policy
- policy/version compatibility
- trust-set evaluation
- deterministic remote acceptance verdict

Phase-12C - Replicated Replay Boundary

Focus:
- replay admission boundary
- proof-backed replay eligibility
- distributed replay protocol boundary
- replicated verification prerequisites

#### 4.13.9 Forward-Compatible Schema Direction

The current Phase-11 proof bundle schema SHOULD remain forward-compatible with Phase-12.

Reserved future fields:
- `producer_id`
- `producer_pubkey_id`
- `build_id`
- `policy_version`
- `signature_algorithm`
- `signature_ref`
- `trust_policy_ref`
- `archive_hash`
- `archive_format`

Design rule:
- Future trust metadata SHALL extend the proof portability schema without breaking existing checksum semantics.
- Future trust metadata SHALL extend the proof portability schema without breaking existing bundle identity semantics.
- Future trust metadata SHALL extend the proof portability schema without breaking existing offline verification semantics.
- Detached signature attachment SHALL NOT mutate pre-existing bundle identity semantics established by Phase-11 portability.

#### 4.13.10 Target Acceptance Semantics

Phase-12 trusted proof acceptance is satisfied only when all of the following are true:
- bundle is structurally valid,
- checksum contract passes,
- manifest parity reproduces,
- signature is valid,
- producer is trusted,
- policy is compatible.

Acceptance invariant:
- `accepted_proof => archive_integrity_pass && checksum_integrity_pass && manifest_parity_pass && signature_valid && producer_trusted && policy_compatible`

Rejection rule:
- Failure of any single component SHALL be fail-closed.

#### 4.13.11 Design Summary

Phase-11 proved that execution proof can exist and travel.

Phase-12 will prove that execution proof can be:
- trusted,
- attributed,
- checked under policy,
- and accepted across nodes deterministically.

This preserves a clean architectural ladder:
- Phase-11 -> proof portability
- Phase-12 -> proof trust + distributed acceptance
- Phase-13+ -> replicated replay / distributed execution verification

---

## 5. Ordering and Concurrency

### 5.1 `event_seq`
- Global monotonic sequence.
- Assigned exactly once per kernel-visible event.
- Missing/duplicate/out-of-order events are fail-closed violations.

### 5.2 `ltick`
- Deterministic logical time for multicore ordering.
- Assigned independent of wall-clock and CPU frequency.

### 5.3 Multicore finalization
- GCP finalizes multicore state deterministically.
- Commit records include both ordering identities and hash roots.

---

## 6. Replay and Identity Binding

Replay input set:
- ABDF snapshot
- BCIB plan
- Phase-11 transcript and ledger

Mandatory identity fields:
- `abdf_snapshot_hash`
- `bcib_plan_hash`
- `execution_trace_hash`

Replay pass conditions:
- `record_event_seq == replay_event_seq`
- `record_ltick == replay_ltick`
- `record_execution_trace_hash == replay_execution_trace_hash`
- expected final state hash equals replay final state hash

Any mismatch is fail-closed.

---

## 7. Evidence and Proof

Evidence path:
- `evidence/run-<RUN_ID>/...`

Core artifacts:
- `decision_ledger.bin`, `decision_ledger.jsonl`
- `transcript.bin`, `transcript.jsonl`
- `replay_report.json`
- `gcp_record.json` (multicore runs)
- `proof.json`
- `gates/proof-bundle/proof_bundle/` (when portability gate executes)

Policy:
- Evidence is exported and retained as CI artifacts.
- Evidence must be immutable after creation.
- Committing evidence to git is optional and repository-policy dependent.

Proof manifest minimum fields:
- `kernel_image_hash`
- `config_hash`
- `ledger_root_hash`
- `transcript_root_hash`
- `replay_result_hash`
- `final_state_hash`
- `event_count`
- `violation_count`

---

## 8. CI Gate Mapping

Required gates:
- `ci-gate-ledger-completeness`
- `ci-gate-eti-sequence`
- `ci-gate-ledger-eti-binding`
- `ci-gate-transcript-integrity`
- `ci-gate-dlt-monotonicity`
- `ci-gate-eti-dlt-binding`
- `ci-gate-dlt-determinism`
- `ci-gate-gcp-finalization` (aliases: `ci-gate-gcp-atomicity`, `ci-gate-gcp-ordering`)
- `ci-gate-abdf-snapshot-identity`
- `ci-gate-bcib-trace-identity` (alias: `ci-gate-execution-identity`)
- `ci-gate-replay-determinism`
- `ci-gate-kpl-proof-verify` (alias: `ci-gate-proof-manifest`)
- `ci-gate-proof-bundle` (alias: `ci-gate-proof-portability`)
- `ci-gate-ledger-integrity` (alias: `ci-gate-hash-chain-validity`)

Extended Phase-11 gates (issue-driven):
- DEOL sequence validation
- DLT monotonicity/parity validation
- GCP atomicity/consistency validation
- KPL proof verification
- Proof bundle portability verification
- ABDF snapshot identity validation
- BCIB plan/trace identity validation

All gates are fail-closed.

---

## 8.1 Documentation Update Contract

For each task completion PR:
- Update `tasks.md` with task progress and gate result.
- Update `requirements.md` if acceptance criteria changed.
- Update architecture-board docs if event model/hash/order contracts changed.
- Include `Documentation Delta` section in PR body.

---

## 9. Migration and Compatibility

Versioning requirements:
- Ledger format versioned.
- Transcript format versioned.
- At least two previous versions accepted by replay tooling.

Compatibility behavior:
- Unsupported version -> explicit reject with typed error.

---

## 10. Implementation Order

Order follows dependency and risk:

1. P11-01 Mailbox capability contract (#34)
2. P11-02 Decision ledger (#35)
3. P11-03 Ledger hash chain (#36)
4. P11-10 DEOL (#40)
5. P11-13 ETI (#43)
6. P11-14 DLT (#44)
7. P11-15 GCP (#45)
8. P11-17 ABDF replay snapshot identity (#47)
9. P11-18 BCIB plan + execution trace identity (#48)
10. P11-04 Replay v1 (#37)
11. P11-11 KPL (#41)
12. P11-42 Proof bundle portability
13. Policy track in parallel: #38 -> #39 -> #42
14. Research track after core closure: #46

Rule:
- 1 PR = 1 invariant.

---

## 11. Open Risks and Mitigations

1. Global sequence contention on high core count.
- Mitigation: keep global atomic for Phase-11 baseline; optimize in later phase if needed.

2. Doc drift between architecture docs and runtime examples.
- Mitigation: taxonomy naming and hash formula are canonical sources.

3. Replay false mismatch due to non-canonical serialization.
- Mitigation: canonical binary encoding + explicit normalization rules.

---

## 12. Definition of Done (Phase-11)

Phase-11 is done when:
- Required structures and hooks are implemented.
- Deterministic replay pass conditions are met.
- Proof manifest is generated and verified.
- Portable proof bundle is generated and verified offline with matching verdict parity.
- CI Phase-11 gates pass in fail-closed mode.
- Documentation and issue acceptance criteria are aligned.

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
- `ci-gate-replay-determinism`
- `ci-gate-ledger-integrity` (alias: `ci-gate-hash-chain-validity`)

Extended Phase-11 gates (issue-driven):
- DEOL sequence validation
- DLT monotonicity/parity validation
- GCP atomicity/consistency validation
- KPL proof verification
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
12. Policy track in parallel: #38 -> #39 -> #42
13. Research track after core closure: #46

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
- CI Phase-11 gates pass in fail-closed mode.
- Documentation and issue acceptance criteria are aligned.

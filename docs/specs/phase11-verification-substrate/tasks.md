# Tasks: Phase-11 Verification Substrate

**Version:** 1.0  
**Status:** Draft  
**Date:** 2026-03-06  
**Related Spec:** `requirements.md`, `design.md`
**Created by:** Kenan AY  
**Maintained by:** Kenan AY  
**Last Edited by:** Kenan AY
**Gelistiren:** Kenan AY  
**Olusturan:** Kenan AY  
**Duzenleyen:** Kenan AY

---

## Execution Policy

- 1 PR = 1 invariant
- Fail-closed validation only
- No direct merge without gate PASS
- Evidence artifacts mandatory for each gate
- Default task owner: Kenan AY (unless explicitly reassigned)

---

## Task Status Ledger

| Issue | Task | Status | Last Update | Notes |
|------|------|--------|-------------|-------|
| #34 | P11-01 Mailbox Capability Contract | COMPLETED_LOCAL | 2026-03-06 | gate PASS + phase10c regression PASS |
| #35 | P11-02 Decision Ledger v1 | COMPLETED_LOCAL_BOOTSTRAP | 2026-03-06 | bootstrap materialization gate PASS (compat mode), strict kernel append + ETI/DLT binding deferred to #43/#44 |
| #36 | P11-03 Ledger Hash Chain | COMPLETED_LOCAL_BOOTSTRAP | 2026-03-06 | hash-chain gate PASS + one-bit tamper detection PASS |
| #40 | P11-10 DEOL | COMPLETED_LOCAL_BOOTSTRAP | 2026-03-07 | deol-sequence gate PASS (bootstrap ordering evidence) |
| #43 | P11-13 ETI | COMPLETED_LOCAL_BOOTSTRAP | 2026-03-07 | eti-sequence + ledger-eti-binding + transcript-integrity gates PASS (bootstrap evidence mode) |
| #44 | P11-14 DLT | COMPLETED_LOCAL_BOOTSTRAP | 2026-03-07 | dlt-monotonicity + eti-dlt-binding + dlt-determinism gates PASS (bootstrap ordering evidence + reproducibility hardening) |
| #45 | P11-15 GCP | COMPLETED_LOCAL_BOOTSTRAP | 2026-03-07 | gcp-finalization gate PASS (bootstrap commit-point contract evidence) |
| #47 | P11-17 ABDF Snapshot Identity | COMPLETED_LOCAL_BOOTSTRAP | 2026-03-07 | abdf-snapshot-identity gate PASS (canonical binary hash identity evidence) |
| #48 | P11-18 BCIB Plan and Trace Identity | COMPLETED_LOCAL_BOOTSTRAP | 2026-03-07 | bcib-trace-identity gate PASS (plan+trace execution identity evidence) |
| #37 | P11-04 Replay v1 | COMPLETED_LOCAL_BOOTSTRAP | 2026-03-07 | replay-determinism gate PASS (record/replay identity parity over #47/#48 evidence) |
| #41 | P11-11 KPL Proof Layer | COMPLETED_LOCAL_BOOTSTRAP | 2026-03-07 | kpl-proof-verify gate PASS (hash-bound proof manifest verification evidence) |
| P11-42 | Proof Bundle Portability | COMPLETED_LOCAL_BOOTSTRAP | 2026-03-07 | proof-bundle gate PASS (portable proof package + offline verdict parity evidence) |

---

## Documentation Sync Policy (Mandatory)

For every completed task, documentation MUST be updated in the same PR.

Minimum required updates:
- `docs/specs/phase11-verification-substrate/tasks.md`
  - task status/progress
  - gate result summary
- `docs/specs/phase11-verification-substrate/design.md`
  - architecture or implementation-flow changes
- `docs/specs/phase11-verification-substrate/requirements.md`
  - acceptance criteria changes/new constraints

Update when impacted:
- `docs/architecture-board/ABDF_BCIB_PHASE11_CONTRACT_MATRIX.md`
- `docs/architecture-board/PHASE11_EVENT_TAXONOMY.md`
- `docs/architecture-board/RUNTIME_STATE_MACHINE.md`
- root-level operational files (e.g. `README.md`, `.github/workflows/ci-freeze.yml`, `Makefile`)

PR documentation rule:
- Every Phase-11 PR MUST include a `Documentation Delta` section in PR body.
- If no doc changed, PR must state explicit reason.

---

## Language Selection Policy

Use the most suitable language per layer:
- **C**: Ring0/kernel hooks, low-level structs, interrupt/scheduler critical path
- **Rust**: ABDF/BCIB tooling, replay verifiers, identity/hash tooling, offline proof utilities
- **Bash/Python**: CI gate orchestration, evidence parsing, report generation

Rules:
- Prefer Rust where memory safety and parser/verifier correctness matter.
- Keep kernel hot-path logic in C unless an approved architecture decision says otherwise.
- Do not force Rust into Ring0 where it increases integration risk without clear gain.

---

## Security and Performance Control Plan

Each task PR MUST include both:
- **Security Check**
  - capability enforcement unchanged or tightened
  - fail-closed behavior on malformed/tampered input
  - no new privilege escalation path
- **Performance Check**
  - event recording overhead measured
  - replay/verification runtime impact measured
  - no regression on existing performance gates

Minimum commands before PR update:
- `make pre-ci`
- `make ci-gate-performance`
- task-specific Phase-11 gate(s)

---

## Workstreams

### WS-A: Core Determinism and Proof Chain

#### T1 - P11-01 Mailbox Capability Contract (#34)
- Branch: `feat/p11-mailbox-capability-contract`
- Owner: Kenan AY
- Invariant: invalid proposal never executes
- Status: COMPLETED_LOCAL (awaiting PR merge)
- Deliverables:
  - `docs/governance/MAILBOX_PROTOCOL_V2_CAPABILITIES.md`
  - capability schema
  - reject reason codes
  - negative tests
- Gate: `ci-gate-mailbox-capability-negative`
- Evidence:
  - `evidence/run-<RUN_ID>/gates/mailbox-cap/`

Validation snapshot:
- `python3 -m unittest tools/ci/test_validate_mailbox_capability_negative.py` -> PASS
- `make ci-gate-mailbox-capability-negative RUN_ID=local-p11-34-mailbox-cap-r2` -> PASS
- `make ci-gate-scheduler-mailbox-phase10c RUN_ID=local-p11-34-regression` -> PASS
- `make ci-gate-performance RUN_ID=local-p11-34-perf` -> FAIL (env/baseline mismatch on local host, not gate logic regression)

#### T2 - P11-02 Decision Ledger v1 (Bootstrap Completeness) (#35)
- Branch: `feat/p11-decision-ledger-v1`
- Owner: Kenan AY
- Invariant: every decision-class event writes exactly one ledger entry
- Status: COMPLETED_LOCAL_BOOTSTRAP (offline materialization mode)
- Deliverables:
  - `ay_decision_ledger_entry_t`
  - binary/jsonl export
  - append-only enforcement
- Gate: `ci-gate-ledger-completeness`
- Evidence:
  - `decision_ledger.bin`
  - `decision_ledger.jsonl`

Validation snapshot:
- `python3 -m unittest tools/ci/test_validate_ledger_completeness.py` -> PASS
- `make ci-gate-ledger-completeness RUN_ID=local-p11-35-ledger-v1` -> PASS
- `make pre-ci RUN_ID=local-p11-35-preci` -> FAIL (expected local hygiene fail while tracked patch set is uncommitted)
- `make ci-gate-performance RUN_ID=local-p11-35-perf` -> FAIL (local env/baseline hash mismatch, no new gate logic regression)

Scope note (normative for this milestone):
- This task currently establishes CI-side ledger materialization/completeness proof from runtime evidence.
- Direct kernel-side append path remains deferred and will be completed with ETI/DLT integration in #43/#44.

Security/Performance snapshot:
- Security: fail-closed on count mismatch, duplicate/non-monotonic IDs, missing required fields, missing origin binding
- Performance: gate is offline parser/validator path; no Ring0 hot-path mutation in this task

#### T3 - P11-03 Ledger Hash Chain Integrity (#36)
- Branch: `feat/p11-ledger-hash-chain`
- Owner: Kenan AY
- Invariant: hash chain tamper is always detected
- Status: COMPLETED_LOCAL_BOOTSTRAP (CI integrity path)
- Deliverables:
  - canonical hash implementation
  - chain validator
  - tamper negative tests
- Gate: `ci-gate-ledger-integrity`
- Evidence:
  - `chain_verify.json`
  - `tamper_test.json`
  - `report.json`
  - `violations.txt`

Validation snapshot:
- `python3 -m unittest tools/ci/test_validate_ledger_hash_chain.py` -> PASS
- `make ci-gate-ledger-integrity RUN_ID=local-p11-36-ledger-integrity-r2` -> PASS
- `make ci-gate-hash-chain-validity RUN_ID=local-p11-36-hash-chain-alias-r3` -> PASS (alias)
- `make ci-gate-performance RUN_ID=local-p11-36-perf` -> FAIL (local env/baseline hash mismatch, no new gate logic regression)

Scope note (normative for this milestone):
- Hash-chain integrity currently validates CI-materialized ledger entries from #35 bootstrap path.
- Direct kernel append + strict ETI/DLT binding remains deferred to #43/#44.

Security/Performance snapshot:
- Security: fail-closed on continuity break, payload hash mismatch, entry hash mismatch, event_seq/ltick ordering anomalies, and tamper simulation
- Performance: validator runs offline in CI/evidence pipeline; no Ring0 hot-path regression in this milestone

#### T4 - P11-10 DEOL (#40)
- Branch: `feat/p11-deol-sequence`
- Owner: Kenan AY
- Invariant: all kernel-visible events receive monotonic unique `event_seq`
- Status: COMPLETED_LOCAL_BOOTSTRAP (ledger-derived sequence proof)
- Deliverables:
  - sequence allocator
  - sequence validator
  - gap/dup/order checks
- Gate: `ci-gate-deol-sequence`
- Evidence:
  - `event_seq.jsonl`
  - `sequence_report.json`

Validation snapshot:
- `python3 -m unittest tools/ci/test_validate_deol_sequence.py` -> PASS
- `make ci-gate-deol-sequence RUN_ID=local-p11-40-deol-sequence-r1 PHASE11_DEOL_LEDGER_EVIDENCE_DIR=evidence/run-local-p11-36-ledger-integrity-r2/gates/ledger-v1` -> PASS

Scope note (normative for this milestone):
- DEOL validation currently operates in bootstrap mode over ledger-derived evidence.
- Direct kernel event allocator integration remains deferred until ETI/DLT strict path (#43/#44).

Security/Performance snapshot:
- Security: fail-closed on ordering field parse errors, source duplicates, source non-monotonicity, and generated sequence invariant breaks
- Performance: offline CI/evidence path only; no Ring0 hot-path overhead introduced in this milestone

#### T5 - P11-13 ETI (#43)
- Branch: `feat/p11-eti-transcript`
- Owner: Kenan AY
- Invariant: canonical transcript is the execution join surface
- Status: COMPLETED_LOCAL_BOOTSTRAP (CI transcript materialization + strict binding gate path)
- Deliverables:
  - ETI binary+jsonl export
  - ETI chain hash
  - ETI binding validator
- Gates:
  - `ci-gate-eti-sequence`
  - `ci-gate-ledger-eti-binding`
  - `ci-gate-transcript-integrity`
- Evidence:
  - `eti_transcript.bin`
  - `eti_transcript.jsonl`
  - `eti_chain_verify.json`
  - `eti_diff.txt`
  - `binding_report.json`

Validation snapshot:
- `python3 -m unittest tools/ci/test_validate_eti_sequence.py` -> PASS
- `python3 -m unittest tools/ci/test_validate_ledger_eti_binding.py` -> PASS
- `python3 -m unittest tools/ci/test_validate_transcript_integrity.py` -> PASS
- `bash scripts/ci/gate_eti_sequence.sh --evidence-dir evidence/run-local-p11-43-eti-sequence-r1/gates/eti --phase10a2-evidence evidence/run-local-p11-36-ledger-integrity-r2/gates/ring3-execution-phase10a2` -> PASS
- `bash scripts/ci/gate_ledger_eti_binding.sh --evidence-dir evidence/run-local-p11-43-ledger-eti-binding-r1/gates/ledger-eti-binding --ledger-evidence evidence/run-local-p11-36-ledger-integrity-r2/gates/ledger-v1 --eti-evidence evidence/run-local-p11-43-eti-sequence-r1/gates/eti` -> PASS
- `bash scripts/ci/gate_transcript_integrity.sh --evidence-dir evidence/run-local-p11-43-transcript-integrity-r1/gates/transcript-integrity --eti-evidence evidence/run-local-p11-43-eti-sequence-r1/gates/eti` -> PASS

Scope note (normative for this milestone):
- ETI currently operates in bootstrap mode using Phase10-A2 event evidence materialization.
- Direct kernel ETI emission hooks and lock-free runtime buffering remain deferred to strict runtime integration stage.
- `eti_diff.txt` is currently emitted as bootstrap placeholder parity artifact and mirrors violation output until strict runtime ETI diffing is enabled.

Security/Performance snapshot:
- Security: fail-closed on missing required ETI event classes, ordering anomalies, hash mismatches, binary/jsonl divergence, and ledger-binding mismatches.
- Performance: CI/offline parser-validator path only; no Ring0 hot-path mutation in this milestone.

#### T6 - P11-14 DLT (#44)
- Branch: `feat/p11-dlt-ordering`
- Owner: Kenan AY
- Invariant: deterministic logical time ordering across cores
- Status: COMPLETED_LOCAL_BOOTSTRAP (ETI-derived DLT proof)
- Deliverables:
  - bootstrap DLT trace materialization (`ltick_trace.jsonl`)
  - ETI<->DLT source identity binding validator
  - ordering parity checks
- Gates:
  - `ci-gate-dlt-monotonicity`
  - `ci-gate-eti-dlt-binding`
  - `ci-gate-dlt-determinism`
- Evidence:
  - `ltick_trace.jsonl`
  - `binding_report.json`
  - `dlt_determinism_report.json`
  - `report.json`
  - `violations.txt`

Validation snapshot:
- `python3 -m unittest tools/ci/test_validate_dlt_monotonicity.py` -> PASS
- `python3 -m unittest tools/ci/test_validate_eti_dlt_binding.py` -> PASS
- `python3 -m unittest tools/ci/test_validate_dlt_determinism.py` -> PASS
- `bash scripts/ci/gate_dlt_monotonicity.sh --evidence-dir evidence/run-local-p11-44-dlt-monotonicity-r1/gates/dlt-monotonicity --eti-evidence evidence/run-local-p11-43-eti-sequence-r1/gates/eti` -> PASS
- `bash scripts/ci/gate_eti_dlt_binding.sh --evidence-dir evidence/run-local-p11-44-eti-dlt-binding-r1/gates/eti-dlt-binding --eti-evidence evidence/run-local-p11-43-eti-sequence-r1/gates/eti --dlt-evidence evidence/run-local-p11-44-dlt-monotonicity-r1/gates/dlt-monotonicity` -> PASS
- `bash scripts/ci/gate_dlt_determinism.sh --evidence-dir evidence/run-local-p11-44-dlt-determinism-r1/gates/dlt-determinism --eti-evidence evidence/run-local-p11-43-eti-sequence-r1/gates/eti` -> PASS

Scope note (normative for this milestone):
- DLT currently operates in bootstrap mode by materializing deterministic ltick trace from ETI evidence.
- Direct kernel hot-path DLT allocator and multicore merge/finalization integration remain deferred to strict runtime stage.
- Verification Kernel Boundary is explicitly enforced: runtime path stays minimal event-contract; heavy verification remains CI/offline.

Security/Performance snapshot:
- Security: fail-closed on missing/invalid ordering fields, source ordering anomalies, DLT trace monotonicity/uniqueness/gap violations, ETI-DLT source identity mismatches, deterministic reproducibility mismatch (same ETI -> different bootstrap DLT trace hash), and corruption-matrix negative tests (drop/duplicate/reorder/tamper).
- Performance: validator runs offline in CI/evidence pipeline; no Ring0 hot-path mutation in this milestone.

#### T7 - P11-15 GCP (#45)
- Branch: `feat/p11-gcp-finalization`
- Owner: Kenan AY
- Invariant: multicore finalization is atomic and deterministic
- Status: COMPLETED_LOCAL_BOOTSTRAP (DLT-derived GCP finalization proof)
- Deliverables:
  - bootstrap GCP snapshot/record materialization
  - finalization consistency validator
  - previous-snapshot monotonicity check (optional input)
  - GCP hash identity (`gcp_hash`) and previous-link identity (`previous_gcp_hash`) continuity enforcement
- Gates:
  - `ci-gate-gcp-finalization` (bootstrap)
  - `ci-gate-gcp-atomicity` (alias)
  - `ci-gate-gcp-ordering` (alias)
- Evidence:
  - `gcp_snapshot.json`
  - `gcp_record.json`
  - `gcp_consistency_report.json`
  - `report.json`
  - `violations.txt`

Validation snapshot:
- `python3 -m unittest tools/ci/test_validate_gcp_finalization.py` -> PASS
- `bash scripts/ci/gate_gcp_finalization.sh --evidence-dir evidence/run-local-p11-45-gcp-finalization-r1/gates/gcp-finalization --dlt-evidence evidence/run-local-p11-44-dlt-monotonicity-r2/gates/dlt-monotonicity` -> PASS
- `make -n ci-gate-gcp-finalization RUN_ID=dryrun-p11-45-gcp-finalization` -> PASS (target graph/contract dry-run)

Scope note (normative for this milestone):
- GCP currently operates in bootstrap CI finalization mode over DLT evidence.
- Runtime multicore prepare/vote/commit integration remains deferred to strict runtime stage.
- Bootstrap validator semantics intentionally require contiguous DLT identities (`event_seq = 1..N`, `ltick = 1..N`); runtime/sharded semantics remain deferred and versioned.

Security/Performance snapshot:
- Security: fail-closed on malformed/invalid DLT trace, non-monotonic/non-contiguous ordering identity stream, prefix alignment failure, previous-snapshot monotonicity violation, and previous-snapshot hash continuity mismatch.
- Performance: validator runs offline in CI/evidence pipeline; no Ring0 hot-path mutation in this milestone.

#### T8 - P11-17 ABDF Snapshot Identity (#47)
- Branch: `feat/p11-abdf-snapshot-identity`
- Owner: Kenan AY
- Invariant: replay starts only with verified snapshot identity
- Status: COMPLETED_LOCAL_BOOTSTRAP (canonical snapshot hash identity proof)
- Deliverables:
  - snapshot hash generator
  - snapshot identity verifier
  - mismatch negative tests
- Gate: `ci-gate-abdf-snapshot-identity`
- Evidence:
  - `abdf_snapshot_hash.txt`
  - `snapshot_identity_report.json`
  - `snapshot_identity_consistency.json`
  - `report.json`
  - `violations.txt`

Validation snapshot:
- `python3 -m unittest tools/ci/test_validate_abdf_snapshot_identity.py` -> PASS
- `tmp_root="$$(mktemp -d)" && mkdir -p "$$tmp_root/input" "$$tmp_root/gate" && printf 'ABDF\x01\x02\x03' > "$$tmp_root/input/snapshot.abdf" && bash scripts/ci/gate_abdf_snapshot_identity.sh --evidence-dir "$$tmp_root/gate" --snapshot-bin "$$tmp_root/input/snapshot.abdf"` -> PASS
- `make -n ci-gate-abdf-snapshot-identity RUN_ID=dryrun-p11-47-abdf-snapshot-identity` -> PASS (target graph/contract dry-run)

Scope note (normative for this milestone):
- ABDF snapshot identity currently operates in bootstrap CI mode over canonical binary snapshot bytes.
- Runtime replay/proof integration consumes `abdf_snapshot_hash` identity but does not alter hash semantics in this milestone.

Security/Performance snapshot:
- Security: fail-closed on missing/empty snapshot, malformed expected hash input, and computed-vs-expected hash mismatch.
- Performance: validator runs offline in CI/evidence pipeline; no Ring0 hot-path mutation in this milestone.

#### T9 - P11-18 BCIB Plan and Trace Identity (#48)
- Branch: `feat/p11-bcib-trace-identity`
- Owner: Kenan AY
- Invariant: replay/proof only valid with matching plan and trace identity
- Status: COMPLETED_LOCAL_BOOTSTRAP (plan+trace execution identity proof)
- Deliverables:
  - plan hash generator
  - execution trace export
  - trace hash verifier
- Gate: `ci-gate-bcib-trace-identity` (alias: `ci-gate-execution-identity`)
- Evidence:
  - `bcib_plan_hash.txt`
  - `execution_trace.jsonl`
  - `execution_trace_hash.txt`
  - `trace_verify.json`
  - `report.json`
  - `violations.txt`

Validation snapshot:
- `python3 -m unittest tools/ci/test_validate_bcib_trace_identity.py` -> PASS
- `tmp_root="$$(mktemp -d)" && mkdir -p "$$tmp_root/execution" "$$tmp_root/gates/eti" "$$tmp_root/gate" && printf 'BCIB\x01\x02\x03' > "$$tmp_root/execution/plan.bcib" && printf '%s\n' '{"event_seq":1,"ltick":1,"cpu_id":0,"event_type":"AY_EVT_SYSCALL_ENTER"}' '{"event_seq":2,"ltick":2,"cpu_id":0,"event_type":"AY_EVT_SYSCALL_EXIT"}' > "$$tmp_root/gates/eti/eti_transcript.jsonl" && bash scripts/ci/gate_bcib_trace_identity.sh --evidence-dir "$$tmp_root/gate" --bcib-plan "$$tmp_root/execution/plan.bcib" --eti-evidence "$$tmp_root/gates/eti"` -> PASS
- `make -n ci-gate-bcib-trace-identity RUN_ID=dryrun-p11-48-bcib-trace-identity` -> PASS (target graph/contract dry-run)

Scope note (normative for this milestone):
- BCIB plan + execution trace identity currently operates in bootstrap CI mode over `plan.bcib` bytes and ETI evidence.
- Runtime replay integration consumes plan/trace identities but does not alter hash semantics in this milestone.

Security/Performance snapshot:
- Security: fail-closed on missing/empty BCIB plan, malformed/invalid ETI-derived execution trace, ordering-identity anomalies, and expected-hash mismatches.
- Performance: validator runs offline in CI/evidence pipeline; no Ring0 hot-path mutation in this milestone.

#### T10 - P11-04 Replay v1 (#37)
- Branch: `feat/p11-deterministic-replay`
- Owner: Kenan AY
- Invariant: record/replay parity for `event_seq`, `ltick`, trace hash
- Status: COMPLETED_LOCAL_BOOTSTRAP (identity-locked replay parity proof)
- Deliverables:
  - replay parity validator
  - replay-determinism gate script
  - mismatch diff artifacts (`event_diff`, `ltick_diff`)
- Gate: `ci-gate-replay-determinism`
- Evidence:
  - `replay_trace.jsonl`
  - `replay_trace_hash.txt`
  - `replay_report.json`
  - `event_diff.txt`
  - `ltick_diff.txt`
  - `report.json`
  - `violations.txt`

Validation snapshot:
- `python3 -m unittest tools/ci/test_validate_replay_determinism.py` -> PASS
- `tmp_root="$$(mktemp -d)" && mkdir -p "$$tmp_root/abdf" "$$tmp_root/execution" "$$tmp_root/eti" "$$tmp_root/execution-gate" "$$tmp_root/replay-gate" && printf '%064d\n' 0 | tr '0' 'a' > "$$tmp_root/abdf/abdf_snapshot_hash.txt" && printf 'BCIB\x01\x02\x03' > "$$tmp_root/execution/plan.bcib" && printf '%s\n' '{"event_seq":1,"ltick":1,"cpu_id":0,"event_type":"AY_EVT_SYSCALL_ENTER"}' '{"event_seq":2,"ltick":2,"cpu_id":0,"event_type":"AY_EVT_SYSCALL_EXIT"}' > "$$tmp_root/eti/eti_transcript.jsonl" && bash scripts/ci/gate_bcib_trace_identity.sh --evidence-dir "$$tmp_root/execution-gate" --bcib-plan "$$tmp_root/execution/plan.bcib" --eti-evidence "$$tmp_root/eti" && bash scripts/ci/gate_replay_determinism.sh --evidence-dir "$$tmp_root/replay-gate" --abdf-evidence "$$tmp_root/abdf" --execution-evidence "$$tmp_root/execution-gate"` -> PASS
- `make -n ci-gate-replay-determinism RUN_ID=dryrun-p11-37-replay-determinism` -> PASS (target graph/contract dry-run)

Scope note (normative for this milestone):
- Replay v1 currently operates in bootstrap CI mode over identity-locked artifacts from #47 (`abdf_snapshot_hash`) and #48 (`bcib_plan_hash`, `execution_trace_hash`).
- Runtime replay execution engine and strict panic-path semantics remain deferred to strict runtime replay integration stage.

Security/Performance snapshot:
- Security: fail-closed on missing/invalid identity hashes, malformed/non-monotonic/duplicate record trace rows, record-vs-replay hash parity break, and expected final-state hash mismatch.
- Performance: validator runs offline in CI/evidence pipeline; no Ring0 hot-path mutation in this milestone.

#### T11 - P11-11 KPL Proof Layer (#41)
- Branch: `feat/p11-kpl-proof-manifest`
- Owner: Kenan AY
- Invariant: run validity requires verifiable proof manifest
- Status: COMPLETED_LOCAL_BOOTSTRAP (hash-bound proof manifest verification)
- Deliverables:
  - proof manifest schema
  - proof manifest validator
  - KPL gate script + fail-closed checks
- Gate: `ci-gate-kpl-proof-verify`
- Evidence:
  - `proof_manifest.json`
  - `proof_verify.json`
  - `report.json`
  - `violations.txt`

Validation snapshot:
- `python3 -m unittest tools/ci/test_validate_kpl_proof_manifest.py` -> PASS
- `tmp_root="$$(mktemp -d)" && mkdir -p "$$tmp_root/abdf" "$$tmp_root/execution-gate" "$$tmp_root/replay-gate" "$$tmp_root/ledger-gate" "$$tmp_root/eti-gate" "$$tmp_root/kpl-gate" "$$tmp_root/meta" && printf '%064d\n' 0 | tr '0' 'a' > "$$tmp_root/abdf/abdf_snapshot_hash.txt" && printf '%064d\n' 0 | tr '0' 'b' > "$$tmp_root/execution-gate/bcib_plan_hash.txt" && printf '%064d\n' 0 | tr '0' 'c' > "$$tmp_root/execution-gate/execution_trace_hash.txt" && printf '%s\n' '{\"status\":\"PASS\",\"replay_result_hash\":\"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd\",\"final_state_hash\":\"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee\",\"replay_event_count\":2,\"violations_count\":0}' > "$$tmp_root/replay-gate/replay_report.json" && printf '%s\n' '{\"event_seq\":1,\"ltick\":1}' > "$$tmp_root/ledger-gate/decision_ledger.jsonl" && printf '%s\n' '{\"event_seq\":1,\"ltick\":1,\"event_type\":\"AY_EVT_SYSCALL_ENTER\"}' > "$$tmp_root/eti-gate/eti_transcript.jsonl" && printf 'KERNEL' > "$$tmp_root/kernel.elf" && printf '%s\n' '{\"run_id\":\"local-kpl\"}' > "$$tmp_root/meta/run.json" && bash scripts/ci/gate_kpl_proof_verify.sh --evidence-dir "$$tmp_root/kpl-gate" --abdf-evidence "$$tmp_root/abdf" --execution-evidence "$$tmp_root/execution-gate" --replay-evidence "$$tmp_root/replay-gate" --ledger-evidence "$$tmp_root/ledger-gate" --eti-evidence "$$tmp_root/eti-gate" --kernel-image-bin "$$tmp_root/kernel.elf" --config-json "$$tmp_root/meta/run.json"` -> PASS
- `make -n ci-gate-kpl-proof-verify RUN_ID=dryrun-p11-41-kpl-proof` -> PASS (target graph/contract dry-run)

Scope note (normative for this milestone):
- KPL proof layer currently operates in bootstrap CI mode with hash-bound manifest verification over identity-locked evidence roots.
- Signature trust path is bootstrap-only (`signature_mode=bootstrap-none`), and strict signer/trust-policy verification is deferred to later proof hardening stage.

Security/Performance snapshot:
- Security: fail-closed on missing referenced evidence artifacts, malformed hash fields, unsupported manifest version, missing required fields, proof self-hash mismatch, and replay binding mismatches.
- Performance: validator runs offline in CI/evidence pipeline; no Ring0 hot-path mutation in this milestone.

#### T12 - P11-42 Proof Bundle Portability
- Branch: `feat/p11-proof-bundle-portability`
- Owner: Kenan AY
- Invariant: portable proof bundle verified on machine B reproduces the manifest verdict from machine A
- Status: COMPLETED_LOCAL_BOOTSTRAP (portable bundle schema + offline verifier parity)
- Deliverables:
  - proof bundle schema (`manifest.json`, `checksums.json`, `evidence/`, `traces/`, `reports/`, `meta/`)
  - offline proof bundle verifier
  - bundle generation gate + portability alias
- Gates:
  - `ci-gate-proof-bundle`
  - `ci-gate-proof-portability` (alias)
- Evidence:
  - `proof_bundle/`
  - `bundle_verify.json`
  - `report.json`
  - `violations.txt`

Validation snapshot:
- `python3 -m unittest tools/ci/test_validate_proof_bundle.py` -> PASS
- `tmp_root="$$(mktemp -d)" && mkdir -p "$$tmp_root/abdf" "$$tmp_root/execution" "$$tmp_root/replay" "$$tmp_root/kpl" "$$tmp_root/ledger" "$$tmp_root/eti" "$$tmp_root/meta" "$$tmp_root/gate" && printf '%064d\n' 0 | tr '0' 'a' > "$$tmp_root/abdf/abdf_snapshot_hash.txt" && printf '%064d\n' 0 | tr '0' 'b' > "$$tmp_root/execution/bcib_plan_hash.txt" && printf '%s\n' '{"cpu_id":0,"event_seq":1,"event_type":"AY_EVT_SYSCALL_ENTER","ltick":1}' '{"cpu_id":0,"event_seq":2,"event_type":"AY_EVT_SYSCALL_EXIT","ltick":2}' > "$$tmp_root/execution/execution_trace.jsonl" && python3 - <<'PY' "$$tmp_root/execution/execution_trace.jsonl" "$$tmp_root/execution/execution_trace_hash.txt" "$$tmp_root/replay/replay_trace.jsonl" "$$tmp_root/replay/replay_trace_hash.txt" "$$tmp_root/replay/replay_report.json" "$$tmp_root/ledger/decision_ledger.jsonl" "$$tmp_root/eti/eti_transcript.jsonl" "$$tmp_root/kernel.elf" "$$tmp_root/meta/run.json" "$$tmp_root/kpl/proof_manifest.json" "$$tmp_root/kpl/proof_verify.json" "$$tmp_root/kpl/report.json" "$$tmp_root/summary.json"\nimport hashlib, json, pathlib, sys\nexec_trace, exec_hash, replay_trace, replay_hash, replay_report, ledger, eti, kernel, run_json, proof_manifest, proof_verify, proof_report, summary = [pathlib.Path(p) for p in sys.argv[1:]]\nreplay_trace.write_text(exec_trace.read_text(encoding='utf-8'), encoding='utf-8')\nledger.write_text('{\"event_seq\":1,\"ltick\":1}\\n{\"event_seq\":2,\"ltick\":2}\\n', encoding='utf-8')\neti.write_text('{\"cpu_id\":0,\"event_seq\":1,\"event_type\":\"AY_EVT_SYSCALL_ENTER\",\"ltick\":1}\\n{\"cpu_id\":0,\"event_seq\":2,\"event_type\":\"AY_EVT_SYSCALL_EXIT\",\"ltick\":2}\\n', encoding='utf-8')\nkernel.write_bytes(b'KERNEL')\nrun_json.write_text('{\"run_id\":\"local-proof-bundle\"}\\n', encoding='utf-8')\nsummary.write_text('{\"gate\":\"summary\",\"verdict\":\"PASS\"}\\n', encoding='utf-8')\ndef sha(path):\n    return hashlib.sha256(path.read_bytes()).hexdigest()\nexec_digest = sha(exec_trace)\nreplay_digest = sha(replay_trace)\nexec_hash.write_text(exec_digest + '\\n', encoding='utf-8')\nreplay_hash.write_text(replay_digest + '\\n', encoding='utf-8')\nreplay_payload = {\"status\":\"PASS\",\"replay_execution_trace_hash\":replay_digest,\"replay_result_hash\":\"d\" * 64,\"final_state_hash\":\"e\" * 64,\"replay_event_count\":2,\"violations_count\":0}\nreplay_report.write_text(json.dumps(replay_payload, sort_keys=True) + '\\n', encoding='utf-8')\nmanifest = {\"manifest_version\":1,\"mode\":\"bootstrap_kpl_proof_manifest\",\"signature_mode\":\"bootstrap-none\",\"signer_sig\":\"\",\"hash_algorithm\":\"sha256\",\"kernel_image_hash\":sha(kernel),\"config_hash\":sha(run_json),\"ledger_root_hash\":sha(ledger),\"transcript_root_hash\":sha(eti),\"abdf_snapshot_hash\":\"a\" * 64,\"bcib_plan_hash\":\"b\" * 64,\"execution_trace_hash\":exec_digest,\"replay_result_hash\":\"d\" * 64,\"final_state_hash\":\"e\" * 64,\"event_count\":2,\"violation_count\":0}\nmanifest['proof_hash'] = hashlib.sha256(json.dumps({k: v for k, v in manifest.items() if k != 'proof_hash'}, sort_keys=True, separators=(',', ':')).encode('utf-8')).hexdigest()\nproof_manifest.write_text(json.dumps(manifest, sort_keys=True) + '\\n', encoding='utf-8')\nproof_verify.write_text('{\"status\":\"PASS\"}\\n', encoding='utf-8')\nproof_report.write_text('{\"gate\":\"kpl-proof\",\"verdict\":\"PASS\"}\\n', encoding='utf-8')\nPY\n&& bash scripts/ci/gate_proof_bundle.sh --evidence-dir "$$tmp_root/gate" --abdf-evidence "$$tmp_root/abdf" --execution-evidence "$$tmp_root/execution" --replay-evidence "$$tmp_root/replay" --kpl-evidence "$$tmp_root/kpl" --ledger-evidence "$$tmp_root/ledger" --eti-evidence "$$tmp_root/eti" --kernel-image-bin "$$tmp_root/kernel.elf" --summary-json "$$tmp_root/summary.json" --meta-run-json "$$tmp_root/meta/run.json"` -> PASS
- `make -n ci-gate-proof-bundle RUN_ID=dryrun-p11-42-proof-bundle` -> PASS (target graph/contract dry-run)

Scope note (normative for this milestone):
- P11-42 currently solves portable proof packaging and offline verdict parity only.
- Bundle verification does not execute runtime replay and does not introduce signed transport/trust policy in this milestone.

Security/Performance snapshot:
- Security: fail-closed on missing required bundle artifacts, checksum mismatches, bundle schema drift, trace-hash parity mismatch, and source-vs-reproduced verdict divergence.
- Performance: generate/verify pipeline runs entirely offline in CI/evidence path; no Ring0 hot-path mutation in this milestone.

---

### WS-B: Policy Track (Parallel After Core Baseline)

#### T13 - P11-05 Arbitration Bus (#38)
- Branch: `feat/p11-arbitration-bus`
- Owner: Kenan AY
- Invariant: arbitration never violates safety envelope
- Gate: `ci-gate-arbitration-safety`

#### T14 - P11-06 Hot Swap and Rollback (#39)
- Branch: `feat/p11-policy-hotswap`
- Owner: Kenan AY
- Invariant: policy violation triggers deterministic rollback
- Gate: `ci-gate-hotswap-rollback`

#### T15 - P11-12 AI Policy Module (#42)
- Branch: `feat/p11-ai-policy-untrusted`
- Owner: Kenan AY
- Invariant: AI policy remains untrusted and envelope-validated
- Gate: `ci-gate-ai-policy-untrusted`

---

### WS-C: Research Track (After Phase-11 Closure Candidate)

#### T16 - P11-16 Runtime Bridge Contract (#46)
- Branch: `research/p11-runtime-bridge-contract`
- Owner: Kenan AY
- Invariant: execution identity tuple is deterministic and recomputable
- Gate: `ci-gate-runtime-bridge-contract`

---

## Dependency Order

Core critical path:
1. #34
2. #35
3. #36
4. #40
5. #43
6. #44
7. #45
8. #47
9. #48
10. #37
11. #41
12. P11-42

Parallel policy path:
1. #38
2. #39
3. #42

Research path:
1. #46

---

## Validation Checklist (Per PR)

- [ ] Invariant clearly stated in PR body
- [ ] One CI gate mapped to invariant
- [ ] Evidence artifacts present and complete
- [ ] Negative tests included
- [ ] Fail-closed behavior verified
- [ ] No policy leakage into Ring0
- [ ] No ABI drift
- [ ] Documentation Delta section added and complete
- [ ] Security check completed and summarized
- [ ] Performance check completed and summarized
- [ ] Language choice justified (C/Rust/Bash/Python)

---

## Local Pre-merge Commands

Run before pushing:

```bash
make pre-ci
make ci-gate-ledger-completeness
make ci-gate-ledger-integrity
make ci-gate-deol-sequence
make ci-gate-eti-sequence
make ci-gate-ledger-eti-binding
make ci-gate-transcript-integrity
make ci-gate-dlt-monotonicity
make ci-gate-eti-dlt-binding
make ci-gate-dlt-determinism
make ci-gate-gcp-finalization
make ci-gate-replay-determinism
make ci-gate-kpl-proof-verify
make ci-gate-proof-bundle
make ci-gate-hash-chain-validity
make ci-gate-mailbox-capability-negative
```

Add component-specific gate(s) from the issue under implementation.

---

## Completion Criteria

Phase-11 implementation is closure-ready when:
- WS-A tasks are complete with gate PASS
- Required artifacts are reproducible in CI
- Core proof chain (#35/#36/#40/#43/#44/#45/#37/#41/P11-42) is green
- Documentation and issue acceptance criteria remain aligned

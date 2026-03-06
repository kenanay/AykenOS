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
| #40 | P11-10 DEOL | PENDING | 2026-03-06 | waits #35/#36 |
| #43 | P11-13 ETI | PENDING | 2026-03-06 | waits #40 |
| #44 | P11-14 DLT | PENDING | 2026-03-06 | waits #43 |
| #45 | P11-15 GCP | PENDING | 2026-03-06 | waits #44 |
| #47 | P11-17 ABDF Snapshot Identity | PENDING | 2026-03-06 | waits #43/#44 |
| #48 | P11-18 BCIB Plan and Trace Identity | PENDING | 2026-03-06 | waits #43/#44 |
| #37 | P11-04 Replay v1 | PENDING | 2026-03-06 | waits #47/#48 |
| #41 | P11-11 KPL Proof Layer | PENDING | 2026-03-06 | waits #37 |

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
- Deliverables:
  - sequence allocator
  - sequence validator
  - gap/dup/order checks
- Gate: `ci-gate-deol-sequence`
- Evidence:
  - `event_seq.jsonl`
  - `sequence_report.json`

#### T5 - P11-13 ETI (#43)
- Branch: `feat/p11-eti-transcript`
- Owner: Kenan AY
- Invariant: canonical transcript is the execution join surface
- Deliverables:
  - ETI binary+jsonl export
  - ETI chain hash
  - ETI binding validator
- Gates:
  - `ci-gate-eti-sequence`
  - `ci-gate-ledger-eti-binding`
- Evidence:
  - `eti_transcript.bin`
  - `eti_transcript.jsonl`

#### T6 - P11-14 DLT (#44)
- Branch: `feat/p11-dlt-ordering`
- Owner: Kenan AY
- Invariant: deterministic logical time ordering across cores
- Deliverables:
  - `ltick` assignment
  - cross-core merge rules
  - ordering parity checks
- Gates:
  - `ci-gate-dlt-monotonicity`
  - `ci-gate-eti-dlt-binding`
- Evidence:
  - `ltick_trace.jsonl`
  - `binding_report.json`

#### T7 - P11-15 GCP (#45)
- Branch: `feat/p11-gcp-finalization`
- Owner: Kenan AY
- Invariant: multicore finalization is atomic and deterministic
- Deliverables:
  - prepare/vote/commit flow
  - commit record model
  - abort path handling
- Gates:
  - `ci-gate-gcp-atomicity`
  - `ci-gate-gcp-ordering`
- Evidence:
  - `gcp_record.json`
  - `gcp_consistency_report.json`

#### T8 - P11-17 ABDF Snapshot Identity (#47)
- Branch: `feat/p11-abdf-snapshot-identity`
- Owner: Kenan AY
- Invariant: replay starts only with verified snapshot identity
- Deliverables:
  - snapshot hash generator
  - snapshot identity verifier
  - mismatch negative tests
- Gate: `ci-gate-abdf-snapshot-identity`
- Evidence:
  - `abdf_snapshot_hash.txt`
  - `snapshot_identity_report.json`

#### T9 - P11-18 BCIB Plan and Trace Identity (#48)
- Branch: `feat/p11-bcib-trace-identity`
- Owner: Kenan AY
- Invariant: replay/proof only valid with matching plan and trace identity
- Deliverables:
  - plan hash generator
  - execution trace export
  - trace hash verifier
- Gate: `ci-gate-bcib-trace-identity`
- Evidence:
  - `bcib_plan_hash.txt`
  - `execution_trace_hash.txt`
  - `execution_trace.jsonl`

#### T10 - P11-04 Replay v1 (#37)
- Branch: `feat/p11-deterministic-replay`
- Owner: Kenan AY
- Invariant: record/replay parity for `event_seq`, `ltick`, trace hash
- Deliverables:
  - replay runtime
  - strict mismatch policy
  - parity validator
- Gate: `ci-gate-replay-determinism`
- Evidence:
  - `replay_report.json`
  - `event_diff.txt`
  - `ltick_diff.txt`

#### T11 - P11-11 KPL Proof Layer (#41)
- Branch: `feat/p11-kpl-proof-manifest`
- Owner: Kenan AY
- Invariant: run validity requires verifiable proof manifest
- Deliverables:
  - proof manifest schema
  - signing + verification
  - manifest join checks
- Gate: `ci-gate-kpl-proof-verify`
- Evidence:
  - `proof_manifest.json`
  - `proof_verify.json`

---

### WS-B: Policy Track (Parallel After Core Baseline)

#### T12 - P11-05 Arbitration Bus (#38)
- Branch: `feat/p11-arbitration-bus`
- Owner: Kenan AY
- Invariant: arbitration never violates safety envelope
- Gate: `ci-gate-arbitration-safety`

#### T13 - P11-06 Hot Swap and Rollback (#39)
- Branch: `feat/p11-policy-hotswap`
- Owner: Kenan AY
- Invariant: policy violation triggers deterministic rollback
- Gate: `ci-gate-hotswap-rollback`

#### T14 - P11-12 AI Policy Module (#42)
- Branch: `feat/p11-ai-policy-untrusted`
- Owner: Kenan AY
- Invariant: AI policy remains untrusted and envelope-validated
- Gate: `ci-gate-ai-policy-untrusted`

---

### WS-C: Research Track (After Phase-11 Closure Candidate)

#### T15 - P11-16 Runtime Bridge Contract (#46)
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
make ci-gate-transcript-integrity
make ci-gate-replay-determinism
make ci-gate-hash-chain-validity
make ci-gate-mailbox-capability-negative
```

Add component-specific gate(s) from the issue under implementation.

---

## Completion Criteria

Phase-11 implementation is closure-ready when:
- WS-A tasks are complete with gate PASS
- Required artifacts are reproducible in CI
- Core proof chain (#35/#36/#40/#43/#44/#45/#37/#41) is green
- Documentation and issue acceptance criteria remain aligned

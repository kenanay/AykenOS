# Tasks: Phase-11 Verification Substrate

**Version:** 1.0  
**Status:** Draft  
**Date:** 2026-03-06  
**Related Spec:** `requirements.md`, `design.md`

---

## Execution Policy

- 1 PR = 1 invariant
- Fail-closed validation only
- No direct merge without gate PASS
- Evidence artifacts mandatory for each gate

---

## Workstreams

### WS-A: Core Determinism and Proof Chain

#### T1 - P11-01 Mailbox Capability Contract (#34)
- Branch: `feat/p11-mailbox-capability-contract`
- Invariant: invalid proposal never executes
- Deliverables:
  - capability schema
  - reject reason codes
  - negative tests
- Gate: `ci-gate-mailbox-capability-negative`
- Evidence:
  - `evidence/run-<RUN_ID>/gates/mailbox-capability/`

#### T2 - P11-02 Decision Ledger v1 (#35)
- Branch: `feat/p11-decision-ledger-v1`
- Invariant: every decision-class event writes exactly one ledger entry
- Deliverables:
  - `ay_decision_ledger_entry_t`
  - binary/jsonl export
  - append-only enforcement
- Gate: `ci-gate-ledger-completeness`
- Evidence:
  - `decision_ledger.bin`
  - `decision_ledger.jsonl`

#### T3 - P11-03 Ledger Hash Chain Integrity (#36)
- Branch: `feat/p11-ledger-hash-chain`
- Invariant: hash chain tamper is always detected
- Deliverables:
  - canonical hash implementation
  - chain validator
  - tamper negative tests
- Gate: `ci-gate-ledger-integrity`
- Evidence:
  - `ledger_integrity_report.json`
  - `violations.txt`

#### T4 - P11-10 DEOL (#40)
- Branch: `feat/p11-deol-sequence`
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
- Invariant: arbitration never violates safety envelope
- Gate: `ci-gate-arbitration-safety`

#### T13 - P11-06 Hot Swap and Rollback (#39)
- Branch: `feat/p11-policy-hotswap`
- Invariant: policy violation triggers deterministic rollback
- Gate: `ci-gate-hotswap-rollback`

#### T14 - P11-12 AI Policy Module (#42)
- Branch: `feat/p11-ai-policy-untrusted`
- Invariant: AI policy remains untrusted and envelope-validated
- Gate: `ci-gate-ai-policy-untrusted`

---

### WS-C: Research Track (After Phase-11 Closure Candidate)

#### T15 - P11-16 Runtime Bridge Contract (#46)
- Branch: `research/p11-runtime-bridge-contract`
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

---

## Local Pre-merge Commands

Run before pushing:

```bash
make pre-ci
make ci-gate-ledger-completeness
make ci-gate-transcript-integrity
make ci-gate-replay-determinism
make ci-gate-hash-chain-validity
```

Add component-specific gate(s) from the issue under implementation.

---

## Completion Criteria

Phase-11 implementation is closure-ready when:
- WS-A tasks are complete with gate PASS
- Required artifacts are reproducible in CI
- Core proof chain (#35/#36/#40/#43/#44/#45/#37/#41) is green
- Documentation and issue acceptance criteria remain aligned


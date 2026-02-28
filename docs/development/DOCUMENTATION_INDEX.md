# AykenOS Documentation Index
This document is subordinate to PHASE 0 – FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

**Last Updated:** 2026-02-28  
**Snapshot Basis:** `WORKTREE (post-mailbox-v1-freeze docs)`

## Current Phase
- **Core OS:** Phase 4.5 In Progress (stabilization)
- **Constitutional System:** Phases 1-12 complete
- **Phase 4.4:** complete baseline (boot/ring3/int80)

## Primary Truth Sources
Kod gercekligi icin once su dosyalari referans alin:

1. `README.md`
2. `ARCHITECTURE_FREEZE.md`
3. `docs/roadmap/overview.md`
4. `docs/development/PROJECT_STATUS_REPORT.md`
5. `docs/development/PHASE_4_5_PROGRESS_REPORT.md`
6. `.github/workflows/ci-freeze.yml`
7. `Makefile`

## CI / Freeze Documentation
1. `docs/operations/CONSTITUTIONAL_CI_MODE.md`
2. `docs/operations/PROVISIONAL_CI_MODE.md`
3. `docs/operations/PERF_BASELINE_POLICY.md`
4. `docs/roadmap/freeze-enforcement-workflow.md`

## Gate References
`make ci-freeze` zincirinde aktif dokumanlanan gate'ler:
1. abi
2. boundary
3. ring0-exports
4. hygiene
5. tooling-isolation
6. constitutional
7. workspace
8. syscall-v2-runtime
9. performance

Ayrica: `ci-summarize`

## Core Code References
1. `kernel/sys/syscall_v2.h`
2. `kernel/sys/syscall_v2.c`
3. `kernel/sys/syscall.c`
4. `kernel/sched/sched.c`
5. `kernel/sched/sched.h`
6. `kernel/fs/vfs.c`
7. `kernel/fs/devfs.c`

## Technical Specifications

### Core System Specifications
1. `docs/development/SCHEDULER_ARBITRATION_CONTRACT.md` - Legacy/historical arbitration design note (superseded by mailbox v1 freeze for C1)
2. `docs/development/CAPABILITY_SYSTEM_REFERENCE.md` - Capability-based security system reference
3. `docs/development/BCIB_SUBMISSION_PROTOCOL.md` - BCIB graph submission and execution protocol
4. `docs/development/RING3_IMPLEMENTATION.md` - Ring3 policy layer implementation
5. `docs/development/SYSCALL_TRANSITION_GUIDE.md` - Syscall v2 migration guide
6. `docs/development/DEVFS_IMPLEMENTATION.md` - DevFS architecture
7. `docs/governance/MAILBOX_PROTOCOL_V1_FREEZE.md` - Mailbox protocol v1 freeze (C1 authority + Gate-4/4.5 proof contract)
8. `docs/development/SCHEDULER_MAILBOX_DEVELOPER_GUIDE.md` - Scheduler mailbox practical developer guide (publish/validate/consume flow)
9. `docs/development/PROOF_GATE_DEBUG_HANDBOOK.md` - Gate-4/Gate-4.5 debug playbook and invariant triage
10. `docs/governance/MAILBOX_ABI_HARDENING_NOTES.md` - ABI layout/marker drift hardening checklist
11. `docs/governance/MAILBOX_PROTOCOL_V2_C2_REVIEW_FREEZE_CANDIDATE.md` - C2 multi-owner review-freeze candidate (`non-normative`)
12. `docs/governance/PHASE10C_C2_STRICT_INVARIANTS.md` - C2 strict formal invariant set and validator mapping

### CI and Operations
1. `docs/operations/CONSTITUTIONAL_CI_MODE.md` - Constitutional CI mode specification
2. `docs/operations/PROVISIONAL_CI_MODE.md` - Provisional CI mode specification
3. `docs/operations/PERF_BASELINE_POLICY.md` - Performance baseline policy
4. `docs/operations/CI_GATE_TROUBLESHOOTING.md` - CI gate troubleshooting guide

### Architecture Documentation
1. `ARCHITECTURE_FREEZE.md` - Architecture freeze specification
2. `docs/roadmap/freeze-enforcement-workflow.md` - Freeze enforcement workflow
3. `docs/development/PROJECT_STRUCTURE.md` - Project structure documentation

## Note
Eski raporlarda gecen bazi "tamamlandi" iddialari kod snapshot'i ile birebir ortusmeyebilir. Bu dosya, merkezi giris noktasi olarak kod-temelli guncel referans setini listeler.

# AykenOS Documentation Index
This document is subordinate to PHASE 0 – FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

**Last Updated:** 2026-02-21  
**Snapshot Basis:** `HEAD 464cd009f4d0`

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
1. `docs/development/SCHEDULER_ARBITRATION_CONTRACT.md` - Scheduler Ring0/Ring3 arbitration protocol
2. `docs/development/CAPABILITY_SYSTEM_REFERENCE.md` - Capability-based security system reference
3. `docs/development/BCIB_SUBMISSION_PROTOCOL.md` - BCIB graph submission and execution protocol
4. `docs/development/RING3_IMPLEMENTATION.md` - Ring3 policy layer implementation
5. `docs/development/SYSCALL_TRANSITION_GUIDE.md` - Syscall v2 migration guide
6. `docs/development/DEVFS_IMPLEMENTATION.md` - DevFS architecture

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

# 20260305 - Two-Level Authority Scheduler (Distributed Policy Authority)

## Context
Phase10-A2 kaniti kapanirken kritik risk alani scheduler <-> mailbox <-> userspace karar yoludur.
Kernel convenience fallback'lari zamanla policy authority'nin Ring0'a kaymasina (policy authority collapse)
neden olabilir.

## Decision
AykenOS scheduler modeli iki seviyeli authority olarak freeze edilir:

1. Level-1 (Ring0): execution authority
2. Level-2 (Ring3): policy authority

Ring0 sadece su sorumluluklara sahiptir:
1. interrupt handling
2. context switch / CR3-TSS mekanizmasi
3. mailbox kararini dogrulama + uygulama
4. fail-closed enforcement

Ring0 asla su sorumluluklari ustlenmez:
1. process selection policy
2. fairness/priority policy
3. scheduler policy synthesis

Ring3 scheduler authority modeli:
1. scheduling policy uretimi Ring3'te kalir
2. mailbox ile karar publish edilir
3. kernel sadece execution uygular

## Enforcement
Bu karar CI'da su kanallarla enforce edilir:

1. `ci-gate-scheduler-mailbox-phase10c`
2. `P10_SCHED_EVENT_NOTIFY` activation marker contract
3. source authority checks (`kernel/sched/sched.c`, `kernel/arch/x86_64/timer.c`)
4. forbidden fallback markers (`P10_SCHED_FALLBACK`, `P10_READY_HEAD_FALLBACK`)

## Consequences
1. Bootstrap bypass sadece pre-user-proof penceresinde izinlidir.
2. Runtime policy owner Ring3 olarak sabitlenir.
3. Phase10-B/C'de scheduler activation authority ayrimi kademeli sertlestirilir.

## Informative Technical Flow (2026-03-05 Snapshot)
Bu bolum bilgilendirici amaclidir; karar metninin kendisi degistirilmez.

1. Ring3 entry canonical path:
   - `kernel/arch/x86_64/ring3_enter.S` icinde `ring3_enter_iretq`
   - IRETQ frame: `SS -> RSP -> RFLAGS -> CS -> RIP`
   - Marker zinciri: `P10_RING3_ATTEMPT -> P10_RFLAGS_IF_ON -> P10_CR3_SWITCH -> P10_RING3_ENTER`
2. User proof path:
   - `userspace/minimal/minimal.S` mailbox epoch publish + `int 0x80` + `int3`
   - `kernel/arch/x86_64/interrupts.c` Ring3 #BP yolunda `P10_RING3_USER_CODE` emit eder.
3. Scheduler activation path:
   - `kernel/arch/x86_64/timer.c` IRQ path `P10_SCHED_EVENT_NOTIFY` marker'i emit eder
   - Ardindan `sched_request_resched_irq()` ile scheduler mekanizmasi tetiklenir.

## C1 -> C2 Clarification
1. C1 (aktif runtime gerceklik):
   - single-owner mailbox authority (`AYKEN_SCHED_OWNER_PID=2`)
   - mailbox payload'i `epoch/candidate_pid` cekirdeginde uygulanir
2. C2 strict (governance + evidence seviyesi):
   - `[[AYKEN_SCHED_MB_ACCEPT]]`, `[[AYKEN_SCHED_ARBITER_DECISION]]`, `[[AYKEN_CTX_SWITCH]]`, `[[AYKEN_SCHED_CURSOR_ADVANCE]]`
   - invariant validator owner-set/fairness/shape kontrolleri yapar
3. Not:
   - Multi-owner scheduler runtime davranisi tam aktif kabul edilmez.
   - Multi-owner semantics su an once CI/governance kontrati olarak enforce edilir; runtime genislemesi sonraki faz isidir.

## Phase10 Evolution Map
1. Phase10-A2: Ring3 execution proof closure (`P10_RING3_USER_CODE`)
2. Phase10-B: ELF loader/runtime semantik bosluklarini kapatma
3. Phase10-C: scheduler+mailbox+preemption entegrasyonunu strict gate ile stabilize etme

## Status
`approved`

## Reviewers
1. Architecture Board
2. Constitutional CI Governance

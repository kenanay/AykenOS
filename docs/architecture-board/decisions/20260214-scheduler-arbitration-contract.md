# Scheduler Arbitration Contract Decision

## Metadata

- Decision ID: ABD-2026-02-14-02
- Title: Scheduler arbitration contract (Yol A: mailbox hint + Ring0 final arbiter)
- Date: 2026-02-14
- Related RFC: N/A (freeze enforcement hardening)
- Related Waiver: N/A
- Related Decision: `docs/architecture-board/decisions/20260214-scheduler-fallback-isolation.md`

## Context

Scheduler fallback isolation sonrası strict path'te Ring3, mailbox üzerinden `next` adayını stage etmektedir. Bu adımın mimari anlamı netleştirilmezse Ring3 tarafı "kesin seçim" yapar gibi yorumlanabilir ve Ring0 enforcement rolü zayıflar.

## Decision

`approved`

## Contract (Yol A)

1. Ring3 `stage_next` çağrısı bir **hint** üretir; kesin seçim değildir.
2. Ring0 staged adayı doğrular, kabul eder veya veto eder.
3. Ring0 doğrulama en az şu kontrolleri içerir:
   - registered process pointer
   - state (`PROC_READY` veya `PROC_RUNNING`)
   - context sanity (`rip/rsp` non-zero, segment/rsp0 kuralları)
4. Strict modda (`AYKEN_SCHED_FALLBACK=0`) Ring0 kendi seçim policy'sini çalıştırmaz.
5. Scheduler armed olduktan sonra kabul edilebilir aday yoksa davranış fail-closed olur.
6. Syscall v2 freeze kontratı korunur; scheduler bridge çağrıları ayrı pencerede tutulur (`0x90..0x9F`).

## Rationale

1. Ring3 policy özerkliğini korurken kernel enforcement kaybını önler.
2. Ring0 son hakem rolü, yanlış/stale hint'lerin context switch'e taşınmasını engeller.
3. Bu model execution-centric mimariyi "policy outside, enforcement inside" olarak netleştirir.

## Non-Goals (Phase 2.6 Scope)

1. Fairness/starvation kontrolünün tam enforcement'u bu kararla kapanmaz.
2. Pointer tabanlı köprüden PID/handle tabanlı köprüye geçiş bu fazda tamamlanmaz.

## Follow-ups (Phase 2.7)

1. `stage_next` girişi pointer yerine `pid/handle` tabanına taşınacak.
2. Ring0 tarafında minimal fairness guard eklenecek (aynı süreç tekrar seçimi için koşullu veto).
3. Gerekirse ownership/capability tabanlı ek doğrulama eklenecek.

## Evidence

- `kernel/include/sched_mailbox_abi.h`
- `kernel/sys/syscall.c`
- `kernel/sched/sched.c`
- `kernel/include/proc.h`
- `kernel/proc/proc.c`

## Sign-off

- Reviewer 1: Kenan AY
- Reviewer 2: Pending
- Reviewer 3: Pending

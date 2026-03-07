# Runtime Integration Guardrails (Phase 10+)
This document is subordinate to PHASE 0 - FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

**Status:** ACTIVE (Fail-Closed)  
**Scope:** `kernel/`, `userspace/`, `ayken-core/`, `ayken/`, `userspace/semantic-cli`  
**Last Updated:** 2026-03-07

## 1) Purpose
Bu belge, gelistirme sirasinda entegrasyon sirasinin gozden kacmasini engellemek icin zorunlu mimari guardrail setini tanimlar.

## 2) Constitutional Layering (Non-Negotiable)
Katmanlar:
1. Donanim
2. Ring0 Kernel (mekanizma)
3. Ring3 Runtime (policy)
4. Veri/Meta katmani (`ABDF` / `BCIB`)
5. AI agent + orchestration
6. Kullanici arayuzu (`semantic-cli`)

Kural:
1. Ring0 sadece mekanizma saglar.
2. Policy (scheduler, VFS/DevFS, AI runtime) Ring3'te kalir.

## 3) Canonical Integration Path
Tek dogru runtime akis:

`semantic-cli` -> `bcib-runtime` -> `ayken-core/bcib` -> `kernel syscalls` -> `kernel`

## 4) Forbidden Direct Couplings
Asagidaki baglantilar fail-closed ihlal kabul edilir:
1. `semantic-cli` -> `kernel` (dogrudan syscall baglama)
2. `ayken` (governance tool) -> runtime execution path
3. `ayken-core` -> Ring0 policy dependency
4. Kernel icinden `userspace/` header/API dogrudan cagri

## 5) Phase-Gated Integration Rules
### Phase 10-A2 (Closure Baseline)
1. Local closure evidence mevcuttur: `local-freeze-p10p11`.
2. `ayken-core` ve `semantic-cli` gelistirilebilir, ancak kernel runtime yoluna baglanmaz.
3. Runtime claim icin zorunlu kanit hala `ci-gate-ring3-execution-phase10a2` strict PASS'tir.
4. Bu kontratin tekrar kirilmasi halinde `missing_marker:P10_RING3_USER_CODE` yeniden blocker kabul edilir.

### Phase 10-B
1. `bcib-runtime` <-> `ayken-core/bcib` entegrasyonu acilabilir.
2. Syscall semantik bosluklari (`syscall_v2.c`) azaltilir.
3. E2E semantic-cli runtime baglantisi hala opsiyonel/advisory seviyede tutulur.

### Phase 10-C
1. Process lifecycle + scheduler stabilization tamamlanir.
2. `semantic-cli` -> `bcib-runtime` -> kernel syscall zinciri aktive edilebilir.
3. Marker/scheduler kontratlari strict gate ile korunur.

### Phase 11+
1. AI runtime ve agent entegrasyonu acilir.
2. `ABDF`/`BCIB` execution path production-hardening fazina girer.

## 6) PR Checklist (Required)
Her runtime/CLI/core PR'inda asagidaki maddeler acik olmalidir:
1. Degisiklik hangi katmanda yapildi?
2. Canonical integration path'e dokundu mu?
3. Phase-gated kurala gore bu baglama bu fazda izinli mi?
4. Kanit/run-id nedir? (`evidence/run-<id>/...`)
5. Docs drift guncellemesi yapildi mi?
6. `ci-gate-feature-phase` sonucu PASS mi?
7. `architecture.features.yaml` degisti ise full-scan evidence eklendi mi?

## 7) Automatic Enforcement (CI + Local)
1. CI fail-closed kontrolu: `ci-gate-runtime-layering`
2. Lokal calistirma: `make ci-gate-runtime-layering`
3. CI fail-closed faz uygunluk kontrolu: `ci-gate-feature-phase`
4. Lokal calistirma: `make ci-gate-feature-phase`
5. Feature registry (tek kaynak): `architecture.features.yaml`
6. CI workflow: `.github/workflows/ci-freeze.yml` icinde `Runtime layering guardrail` + `Feature phase guardrail` adimlari
7. `architecture.features.yaml` degisirse feature gate otomatik full-scan moduna gecmelidir.
8. Evidence:
   - `evidence/run-<id>/gates/runtime-layering/`
   - `evidence/run-<id>/gates/feature-phase/`
9. Feature-phase fail mantigi:
   - phase-uygunsuz **degisen dosya**: `FAIL` (severity=ERROR)
   - full-scan'de phase-uygunsuz mevcut/legacy dosya: `WARN` (drift gorunurlugu, merge blocker degil)
   - izlenen mimari koklerde (`kernel/`, `userspace/`, `runtime/`, `ayken-core/`, `ayken/`) feature-eslesmesi olmayan yeni path: `FAIL`

## 8) Authority References
1. `README.md` (snapshot truth)
2. `docs/roadmap/overview.md` (guncel teknik gerceklik)
3. `docs/development/PROJECT_STATUS_REPORT.md` (code + evidence status)
4. `Makefile` + `scripts/ci/*` + `.github/workflows/ci-freeze.yml` (tek gercek otorite)
5. `architecture.features.yaml` (feature-phase policy authority)

## 9) Phase10-A2 Mini Trace Contract (Scheduler <-> Mailbox <-> User)
Phase10-A2 triage icin gate raporu su 6 state'i birlikte yayinlar:
1. `S1_RING3_ENTER` -> `P10_RING3_ENTER`
2. `S2_SYSCALL_ENTER` -> `P10_SYSCALL_ENTER`
3. `S3_SYSCALL_RETURN` -> `P10_SYSCALL_RETURN`
4. `S4_CAP_ENFORCED` -> `P10_CAP_ENFORCED`
5. `S5_MAILBOX_DECISION` -> `P10_MAILBOX_DECISION` (diagnostic-only, optional)
6. `S6_RING3_USER_CODE` -> `P10_RING3_USER_CODE`

Fail-closed yorum kurali:
1. `missing_marker:P10_RING3_USER_CODE` tek basina yeterli blocker'dir.
2. `trace_cut_before_user:<MAILBOX_FATAL>` varsa sorun scheduler/mailbox karar yolunda user code'dan once kesilmistir.
3. `mini_trace_summary.risk_signals` alaninda:
   - `mailbox_liveness_risk`: mailbox fatal marker user marker'dan once.
   - `scheduler_preemption_before_user`: ilk `P10_IRQ_SCHED_DECISION`, `P10_RING3_USER_CODE` oncesinde.
   - `scheduler_priority_inversion_signal`: user marker yok + yuksek IRQ scheduler karari.
   - `user_path_visibility_gap`: user marker yok ama fatal precursor da yok (instrumentation/path gorunurlugu acigi).

Kanit alanlari:
1. `report.json` -> `mini_trace_sequence`, `mini_trace_observed`, `mini_trace_summary`
2. `violations.txt` -> `trace_cut_before_user:*`

## 10) Policy Authority Collapse Guard
Iki seviyeli authority kurali (distributed scheduler):
1. Ring0: execution authority (interrupt, context-switch, enforcement)
2. Ring3: policy authority (process secimi, fairness, priority)

Fail-closed guard:
1. Ring0 scheduler policy uretemez.
2. Ring0 fallback secimi (`ready_head` vb.) kalici policy yoluna donusemez.
3. `P10_SCHED_EVENT_NOTIFY` marker'i scheduler activation authority'nin IRQ notify katmaninda kaldigini kanitlar.
4. Source guard enforcement `ci-gate-scheduler-mailbox-phase10c` icinde calisir.

## 11) Informative Execution Chain (Phase10)
Bu bolum bilgilendirici amaclidir; fail/pass kararinin resmi otoritesi gate raporudur.

1. Ring3 entry canonical zinciri:
   - `P10_RING3_ATTEMPT`
   - `P10_RFLAGS_IF_ON`
   - `P10_CR3_SWITCH`
   - `P10_RING3_ENTER`
2. Ring3 proof yolu:
   - Ring3 minimal stub (`userspace/minimal/minimal.S`) `int 0x80` ve `int3` tetikler
   - Ring3 #BP path `P10_RING3_USER_CODE` marker'ini uretir
3. Scheduler/mailbox zinciri:
   - `P10_SCHED_EVENT_NOTIFY` (IRQ notify)
   - `P10_IRQ_SCHED_DECISION`
   - `P10_MAILBOX_DECISION`
   - `P10_DECISION_APPLIED`
4. Diagnostic sinyaller:
   - `scheduler_preemption_before_user` tek basina blocker degildir
   - `missing_marker:P10_RING3_USER_CODE` blocker'dir
   - `trace_cut_before_user:*` blocker'dir
5. Bootstrap bariyeri:
   - Pre-user-proof penceresinde `P10_MAILBOX_MISS_PRE_USER_BYPASS` gecici korunma marker'i gorulebilir
   - Bu marker kalici policy fallback anlamina gelmez; A2 closure guvenligi icindir

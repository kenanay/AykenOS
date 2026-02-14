# Scheduler Fallback Isolation Decision

## Metadata

- Decision ID: ABD-2026-02-14-01
- Title: Scheduler fallback isolation (default-off, validation-only)
- Date: 2026-02-14
- Related RFC: N/A (freeze enforcement hardening)
- Related Waiver: N/A

## Context

Freeze döneminde Ring0 policy sızıntısını engellemek için scheduler fallback davranışı kesin kurala bağlanmalıdır. Ring0 tarafındaki geçici round-robin fallback yalnızca geçiş/validation amacıyla tutulabilir; default/release veya strict freeze zincirinde aktif kalamaz.

## Decision

`approved`

## Rationale

1. Ring0 yalnız mekanizma olmalıdır; policy kararları Ring3 tarafında kalmalıdır.
2. Fallback default açık kalırsa constitutional drift ve davranış belirsizliği oluşur.
3. Validation-only izolasyon, geçiş sırasında test kabiliyetini korurken mainline mimariyi kirletmez.

## Evidence

- run_id: N/A (policy decision record)
- evidence_path: `docs/roadmap/freeze-enforcement-workflow.md`, `ARCHITECTURE_FREEZE.md`, `Makefile`, `scripts/ci/gate_constitutional.sh`
- summary_verdict: N/A (karar kaydı)

## Conditions

1. `AYKEN_SCHED_FALLBACK ?= 0` repo defaultu zorunludur.
2. `AYKEN_SCHED_FALLBACK=1` yalnız `KERNEL_PROFILE=validation` ile kullanılabilir.
3. `make ci-freeze` fallback açıkken hard-fail etmelidir.
4. Fallback kapalıyken Ring0 seçim policy çalıştırmaz; Ring3 yalnız mailbox ile `next` stage eder (bootstrap öncesi tek-seferlik ready list tüketimi hariç).
5. Constitutional strict-mode fallback kontratını evidence ile doğrulamalıdır.

## Follow-ups

1. Ring3 scheduler policy path tamamlandığında fallback kodu tamamen kaldırılacak.
2. Geçiş süresince fallback açımı sadece validation amaçlı ve kanıtlı run'larla sınırlı tutulacak.

## Sign-off

- Reviewer 1: Kenan AY
- Reviewer 2: Pending
- Reviewer 3: Pending

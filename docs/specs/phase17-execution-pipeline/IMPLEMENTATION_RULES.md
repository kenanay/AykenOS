# Phase-17 Implementation Rules (NON-OVERRIDABLE)

**Authority:** Kenan AY - Architectural Steward  
**Status:** BINDING  
**Phase:** 17  
**Effective Date:** 2026-05-01  
**Purpose:** Determinism, fail-closed ve mimari sınırların korunması

---

## 1. Determinism Kuralları

**Temel Garanti:**
```
Aynı BCIB + aynı execution_context_snapshot 
→ aynı raw_output ve fingerprint
```

**Kurallar:**
- `execution_context_snapshot` kapsamı **GENİŞLETİLEMEZ** (değişiklik = spec güncellemesi gerektirir)
- Global kernel state (heap, process table, IRQ state) snapshot'a dahil **EDİLMEZ**
- Nondeterministic source (system time, random) **YASAKTIR**

**İhlal Sonucu:**
> Determinism guarantee geçersiz olur

---

## 2. Production Code Immutability (NON-OVERRIDABLE)

**Temel Kural:**
> `kernel/sys/execution_slot.c` ve ilgili production dosyalar OVERWRITE EDİLEMEZ

**Yasaklar:**
- ❌ Production dosya overwrite (prototype ile değiştirme)
- ❌ Büyük refactor (toplu silme/değiştirme)
- ❌ Satır sayısı düşürme (1500+ → 500)
- ❌ Kritik sembollerin silinmesi

**İzin Verilenler:**
- ✅ Yeni dosya ekleme (`execution_marker_validation.c`)
- ✅ Guarded include ekleme (`#ifdef AYKEN_FEATURE`)
- ✅ Küçük additive değişiklikler

**Enforcement:**
```bash
# CI gate zorunlu
ci-gate-execution-slot-integrity

# Kontroller:
- Line count >= 1500 (execution_slot.c)
- Critical markers present (g_execution_slots, prepare_result, etc.)
- No prototype indicators (malloc, printf, etc.)
```

**İhlal Sonucu:**
> CI FAIL → merge BLOCKED

**Rationale:**
- Deterministic execution pipeline korunur
- Evidence chain bozulmaz
- CI authority geçerliliği sürer

**Incident Reference:**
> Commit b3e2aee7: 1910 satır production code yanlışlıkla prototype ile overwrite edildi.
> Recovery: Revert + integrity gate eklendi.

---

## 3. Execution Slot Kuralları

**Temel Kural:**
> Tüm execution state değişimleri **yalnızca** `execution_slot` içinde yapılır

**Yasaklar:**
- ❌ execution_slot dışından result buffer'a yazım
- ❌ VERIFIED olmadan COMMITTED state'ine geçiş
- ❌ State machine bypass

**State Machine (Zorunlu):**
```
EXECUTING → VERIFYING → VERIFIED → COMMITTED
```

**Her state geçişi:**
- Atomic olmalı
- Logged olmalı
- Reversible olmalı (FAILED state'e)

---

## 3. Verification Kuralları (Fail-Closed)

**Temel Kural:**
> Verification PASS olmadan result publish YOKTUR

**Terminal State Koşulları:**
```c
if (result_size == 0)           → TERMINAL
if (buffer_overflow)            → TERMINAL
if (hash_computation_failed)    → TERMINAL
if (verification_timeout)       → TERMINAL
```

**Terminal State Davranışı:**
- Scheduler tarafından tekrar çalıştırılmaz
- Result buffer publish edilmez
- Evidence kaydedilir

**Kural:**
> TERMINAL state = permanent failure (no retry)

---

## 4. Result Buffer Kuralları

**Format (Sabit):**
```c
struct result_buffer {
    uint32_t magic;           // 0x52455355 ('RESU')
    uint32_t version;         // 0x00000001
    uint32_t payload_size;
    uint8_t  payload[];
} __attribute__((packed));
```

**Kurallar:**
- Format değişimi **YASAKTIR** (ABI break)
- VERIFY sonrası buffer **IMMUTABLE**
- Payload değişimi = determinism ihlali

**Enforcement:**
```c
// VERIFY sonrası
buffer->flags |= BUFFER_IMMUTABLE;

// Sonraki write attempt
if (buffer->flags & BUFFER_IMMUTABLE) {
    panic("result buffer write after verify");
}
```

---

## 5. Hash & Fingerprint Kuralları

**raw_output_hash:**
```c
raw_output_hash = SHA256(raw_bytes)
```

**execution_fingerprint:**
```c
execution_fingerprint = SHA256(
    bcib_hash ||
    execution_context_snapshot_hash ||
    raw_output_hash
)
```

**Kurallar:**
- Hash input sırası **DEĞİŞTİRİLEMEZ**
- Hash algoritması **SABİT** (SHA256)
- Padding/encoding **SABİT**

**İhlal Sonucu:**
> Fingerprint parity bozulur → CI FAIL

---

## 6. AI Runtime Kuralları (Phase-17 Sınırı)

**Zorunlu Kısıtlar:**
```bash
THREADS=1       # Zorunlu
SEED=FIXED      # Zorunlu
MODEL_HASH=FIXED # Zorunlu
```

**Kapsam Dışı (Phase-18):**
- AI runtime internal state
- Model weights determinism
- Semantic output determinism

**Kural:**
> AI boot ≠ AI determinism

---

## 7. Yasaklar

**Phase-17'de YASAKTIR:**

❌ Semantic output validation  
❌ Embedding / vector store  
❌ Distributed execution  
❌ Adaptive scheduler  
❌ Performance optimizasyonu (kanıt öncesi)  
❌ Multi-threading (AI runtime)  
❌ Dynamic memory behavior  
❌ System time kullanımı (business logic)

**Neden Yasak?**
> Bu özellikler determinism'i bozar veya Phase-17 kapsamı dışındadır

---

## 8. CI ve Gate Kuralları

**Temel Kural:**
> Gate PASS = yalnızca ölçtüğü şeyi garanti eder

**Yasaklar:**
- ❌ Yanlış isimlendirme (determinism demek için gerçek determinism ölçülmeli)
- ❌ Build gate → execution kanıtı
- ❌ Marker presence → determinism kanıtı

**Doğru Model:**
```
Gate ismi = ölçülen şey
Gate scope = açık tanımlı
Gate guarantee_level = explicit
```

**Örnek:**
```json
{
  "gate": "bcib-stub-build-integrity",
  "guarantee_level": "build_only",
  "does_not_prove": ["execution", "determinism"]
}
```

---

## 9. Commit Tanımı (Kritik)

**Doğru Tanım:**
```c
// commit = result buffer'ın userspace'e publish edilmesi
publish_result_to_userspace(slot);
```

**Yanlış Tanım:**
```c
// ❌ YANLIŞ: Internal state change ≠ commit
slot->state = SLOT_STATE_COMMITTED;
```

**Neden Kritik?**
> Eğer commit = internal state change:
> - Userspace partial data görebilir
> - Determinism bozulur
> - Verification anlamsızlaşır

**Kural:**
> VERIFY öncesi publish YASAKTIR

---

## 10. Kural İhlali

**Bu kurallar NON_OVERRIDABLE'dır:**
- Allow mekanizması **UYGULANAMAZ**
- Waiver mekanizması **UYGULANAMAZ**
- Exception mekanizması **UYGULANAMAZ**

**İhlal Sonucu:**
> Phase-17 invalid sayılır

**Enforcement:**
- CI gates
- Code review
- Constitutional compliance

---

## 11. Snapshot Scope (Dar Kapsam)

**İçinde:**
```c
execution_context_snapshot = {
    execution_slot_id,
    execution_slot_state,
    bcib_input_buffer_identity,
    mapped_io_buffer_bounds,
    context_id,
    scheduler_eligibility,
    ABI_version,
    syscall_version
}
```

**Dışında:**
- Kernel heap state
- Process table
- IRQ state
- Timer state
- AI model state

**Kural:**
> Snapshot genişletme = spec change + determinism re-evaluation

---

## 12. Verification Mode

**STRICT Mode (CI/Debug):**
```c
- Full SHA256 hash computation
- Fingerprint generation
- Contract validation
- Evidence generation
```

**RELAXED Mode (Runtime):**
```c
- Sanity checks only
- No crypto overhead
- Lightweight validation
```

**Kural:**
```c
#ifdef AYKEN_VALIDATION
    mode = VERIFICATION_MODE_STRICT;
#else
    mode = VERIFICATION_MODE_RELAXED;
#endif
```

---

## 13. Implementation Order

**Zorunlu Sıra:**
```
1. Inline verification skeleton
2. ci-gate-bcib-determinism v2 (kernel output)
3. Python-generated → KERNEL-GENERATED transition
```

**Kural:**
> Implementation → gate → claim

**Yasak:**
> Gate → implementation (gate neyi test ediyor?)

---

## 14. Evidence Requirements

**Her execution için:**
```
evidence/run-<RUN_ID>/
├── raw_output.bin          (kernel-produced)
├── raw_output.sha256
├── execution_fingerprint.bin
├── execution_context_snapshot.bin
└── receipt.json
```

**Kural:**
- Evidence immutable
- Evidence CI-safe (ephemeral → persistent)
- Evidence reproducible

---

## 15. Final Rule

**Phase-17 Success Condition:**
```
Aynı BCIB + aynı context → aynı output (kanıtlanmış)
```

**Phase-17 Failure Condition:**
```
Herhangi bir kural ihlali
```

---

**Hazırlayan:** Kenan AY - Architectural Steward  
**Tarih:** 01 Mayıs 2026  
**Versiyon:** 1.0  
**Durum:** BINDING (NON-OVERRIDABLE)

**© 2026 Kenan AY - AykenOS Project**

# Phase-17 — Kanıtlanabilir Execution & Deterministic AI Bootstrap

**Authority:** Kenan AY - Architectural Steward  
**Status:** PLANNING  
**Phase:** 17  
**Effective Date:** 2026-05-01  
**Prerequisites:** Phase-16 Official Closure

---

## Executive Summary

Phase-17, AykenOS'un execution-centric mimarisinde **gerçek BCIB execution determinism** ve **inline verification** katmanını hayata geçirir. Phase-16'da verification layer MVP'si tamamlandı; Phase-17'de bu katman **kernel içinde** çalışır hale gelir ve **real BCIB execution** aktif olur.

**Kritik Ayrım:**
- Phase-16: Verification layer **external** (stub determinism)
- Phase-17: Verification layer **inline** (real execution determinism)
- Phase-18: AI runtime **semantic** determinism (kapsam dışı)

---

## 1. Önkoşul (Başlamadan)

Phase-17 **başlamadan önce** aşağıdaki koşullar sağlanmalı:

### 1.1 Phase-16 Official Closure

```bash
# Tag kontrolü
git tag | grep phase16-official-closure

# CI freeze PASS referansı
# Evidence: Phase-16 closure commit SHA
```

**Gerekli Artifactlar:**
- `phase16-official-closure` tag mevcut
- CI freeze PASS (remote run ID kaydedilmiş)
- Evidence snapshot frozen (immutable)

### 1.2 Closure Manifest

**Dosya:** `reports/phase16_official_closure/closure_manifest.json`

**İçerik:**
```json
{
  "phase": 16,
  "closure_type": "official_closure",
  "closure_state": "CONFIRMED",
  "commit_sha": "<HEAD_SHA>",
  "ci_freeze_run_id": "<RUN_ID>",
  "ci_result": "PASS",
  "determinism": {
    "stub": true,
    "real_execution": false
  },
  "verification_layer": {
    "status": "MVP_COMPLETE",
    "mode": "external"
  },
  "limitations": [
    "real BCIB execution not implemented",
    "inline verification not active",
    "AI runtime not present"
  ],
  "next_phase": 17
}
```

**Kural:** Bu dosya olmadan Phase-17 açılmaz.

---

## 2. Verification Contract v2 (Otorite)

### 2.1 Kapsam

**Given:**
- BCIB bytes (immutable)
- `execution_context_snapshot` (dar kapsam)
- ABDF input artifacts
- Syscall ABI/version

**Then:**
- Execution trace identity stabil
- `raw_output` bytes stabil
- `execution_fingerprint` stabil
- Result **yalnızca** verify PASS sonrası commit edilir

### 2.2 execution_context_snapshot Tanımı (Dar ve Kanıtlanabilir)

```c
execution_context_snapshot = {
  execution_slot_id,
  execution_slot_state,
  bcib_input_buffer_identity,
  mapped_io_buffer_bounds,
  context_id,
  scheduler_eligibility (ilgili context),
  ABI_version,
  syscall_version
}
```

**Kapsam Dışı (Özellikle):**
- Tüm kernel heap
- Process table
- IRQ state
- AI model internal state

**Neden Dar Kapsam?**
- Kanıtlanabilir determinism
- Reproducible execution
- Fail-closed semantics

**Enforcement (Zorunlu):**
```c
// kernel/sys/execution_slot.c
#define EXECUTION_CONTEXT_SNAPSHOT_SIZE 64

_Static_assert(
    sizeof(execution_context_snapshot_t) == EXECUTION_CONTEXT_SNAPSHOT_SIZE,
    "execution_context_snapshot size must not change"
);

// Runtime check
if (snapshot_contains_global_state(snapshot)) {
    panic("execution_context_snapshot: global state leak detected");
}
```

**Kural:**
> Snapshot genişletme = spec change + determinism re-evaluation + CI gate update

### 2.3 Fail-Closed Kuralları (Zorunlu)

| Durum | Davranış |
|-------|----------|
| `bcib_hash` mismatch | REJECT (commit yok) |
| `raw_output_hash` mismatch | INVALIDATE + re-exec |
| `fingerprint` missing | GATE FAIL (deployment blok) |
| Verification timeout | FAIL (publish yok) |
| Receipt malformed | REJECT (evidence geçersiz) |

**Kural:** Hiçbir exception mekanizması bu kuralları override edemez.

---

## 3. Execution Receipt v2 (Kanıt Nesnesi)

### 3.1 Yapı

**Kernel (Canonical Binary Layout):**
```c
// kernel/include/execution_receipt.h
struct execution_receipt_v2 {
    uint8_t bcib_hash[32];
    uint8_t execution_context_snapshot_hash[32];
    uint8_t raw_output_hash[32];           // ← bağlayıcı
    uint8_t execution_fingerprint[32];
    uint32_t verification_version;
    uint8_t semantic_output_hash[32];      // ← advisory (Phase-18)
    uint8_t semantic_output_hash_present;  // 0 = absent, 1 = present
} __attribute__((packed));
```

**Userspace (Parser):**
```rust
// userspace/ayken-core/src/receipt.rs
pub struct ExecutionReceipt {
    pub bcib_hash: [u8; 32],
    pub execution_context_snapshot_hash: [u8; 32],
    pub raw_output_hash: [u8; 32],           // ← bağlayıcı
    pub execution_fingerprint: [u8; 32],
    pub verification_version: u32,
    pub semantic_output_hash: Option<[u8; 32]>, // ← advisory (Phase-18)
}
```

### 3.2 Bağlayıcı vs Advisory

**Bağlayıcı (Phase-17):**
- `bcib_hash`
- `execution_context_snapshot_hash`
- `raw_output_hash`
- `execution_fingerprint`

**Advisory (Phase-17'de bağlayıcı değil):**
- `semantic_output_hash` → Phase-18'de bağlayıcı olur

---

## 4. Inline Verification (Çekirdek İçinde)

### 4.1 Yer (Kesin)

**Dosya:** `kernel/sys/execution_slot.c`

**Fonksiyon:** `execution_slot_verify()`

### 4.2 State Machine

```
EXECUTING → VERIFYING → VERIFIED → COMMITTED
```

**Kurallar:**
1. Verify PASS olmadan commit YOK
2. **Commit = result buffer publish to userspace** (not internal state change)
3. `SYS_V2_WAIT_RESULT` sadece doğrulanmış sonucu döndürür
4. Userspace doğrulaması advisory (otorite değil)

**Kritik Ayrım:**
```c
// ❌ YANLIŞ: Internal state change = commit
slot->state = SLOT_STATE_COMMITTED;  // Bu commit DEĞİL

// ✅ DOĞRU: Result buffer publish = commit
publish_result_to_userspace(slot);   // Bu commit
```

**Neden Kritik?**
> Eğer commit = internal state change olursa:
> - Userspace partial data görebilir
> - Determinism bozulur
> - Verification anlamsızlaşır

### 4.3 Implementasyon

```c
// kernel/sys/execution_slot.c

typedef enum {
    SLOT_STATE_IDLE,
    SLOT_STATE_EXECUTING,
    SLOT_STATE_VERIFYING,    // ← yeni
    SLOT_STATE_VERIFIED,     // ← yeni
    SLOT_STATE_COMMITTED,
    SLOT_STATE_FAILED
} execution_slot_state_t;

typedef enum {
    VERIFICATION_MODE_STRICT,   // CI / debug (full verification)
    VERIFICATION_MODE_RELAXED   // Runtime (lightweight checks)
} verification_mode_t;

int execution_slot_verify(execution_slot_t *slot, verification_mode_t mode) {
    // Performance-critical path: conditional verification
    if (unlikely(mode == VERIFICATION_MODE_STRICT)) {
        // 1. Compute execution_context_snapshot_hash
        uint8_t context_hash[32];
        compute_context_snapshot_hash(slot, context_hash);
        
        // 2. Compute raw_output_hash
        uint8_t output_hash[32];
        compute_raw_output_hash(slot->result_buffer, slot->result_size, output_hash);
        
        // 3. Generate execution_fingerprint
        uint8_t fingerprint[32];
        generate_execution_fingerprint(slot, fingerprint);
        
        // 4. Verify contract
        if (!verify_execution_contract(slot, context_hash, output_hash, fingerprint)) {
            slot->state = SLOT_STATE_FAILED;
            return -1;
        }
    } else {
        // Relaxed mode: lightweight sanity checks only
        if (!verify_slot_sanity(slot)) {
            slot->state = SLOT_STATE_FAILED;
            return -1;
        }
    }
    
    // 5. Transition to VERIFIED
    slot->state = SLOT_STATE_VERIFIED;
    return 0;
}
```

**Performance Contract:**
- **STRICT mode:** CI, debug, validation builds (full hash + fingerprint)
- **RELAXED mode:** Runtime (sanity checks only, no crypto overhead)
- Heavy verification **CI'da** yapılır, runtime'da değil

---

## 5. Real BCIB Execution Path (Stub'dan Çıkış)

### 5.1 Stub Devre Dışı

**Makefile:**
```makefile
AYKEN_BCIB_STUB_RESULT_ENABLE ?= 0  # ← Phase-17'de 0
```

### 5.2 Gerçek Hat

```
SUBMIT → scheduler → execution_slot → result buffer
```

**Marker'lar:**
```
[BCIB_SUBMIT]
[EXECUTION_START]
[RESULT_OK]
[WAIT_OK]
```

### 5.3 Üretilen Çıktı

- **kernel-produced** `raw_output` (byte-level)
- **deterministic** (aynı BCIB + aynı context → aynı output)

---

## 6. Determinism Gates (CI Otoritesi)

### 6.1 Gate A — Real BCIB Determinism

**Hedef:** `ci-gate-bcib-determinism`

**Kontrol:**
```bash
make ci-gate-bcib-determinism
```

**Validasyon:**
- İki bağımsız run
- `raw_output_hash(run1) == raw_output_hash(run2)`

**Evidence:**
```
evidence/run-<RUN_ID>/gates/bcib-determinism/
├── run1/
│   ├── raw_output.bin
│   ├── raw_output.sha256
│   └── execution_fingerprint.bin
├── run2/
│   ├── raw_output.bin
│   ├── raw_output.sha256
│   └── execution_fingerprint.bin
├── report.json
└── violations.txt
```

**Failure → Merge REJECT**

### 6.2 Gate B — Fingerprint Consistency

**Hedef:** `ci-gate-execution-fingerprint-consistency`

**Kontrol:**
```bash
make ci-gate-execution-fingerprint-consistency
```

**Validasyon:**
- `execution_fingerprint(run1) == execution_fingerprint(run2)`

**Evidence:**
```
evidence/run-<RUN_ID>/gates/fingerprint-consistency/
├── fingerprint_run1.bin
├── fingerprint_run2.bin
├── report.json
└── violations.txt
```

**Failure → Merge REJECT**

---

### 6.3 Gate C — Marker Order Enforcement

**Hedef:** `ci-gate-marker-order`

**Kontrol:**
```bash
make ci-gate-marker-order
```

**Validasyon:**
- Marker sequence doğru sırada mı?
- Zorunlu marker'lar mevcut mu?

**Expected Sequence:**
```
[EXEC_START]
[EXEC_OUTPUT_WRITTEN]
[EXEC_COMPLETE_OK]
[VERIFY_START]
[VERIFY_PASS]
[RESULT_OK]
[WAIT_OK]
```

**Evidence:**
```
evidence/run-<RUN_ID>/gates/marker-order/
├── markers.log
├── sequence_check.json
├── report.json
└── violations.txt
```

**Failure → Merge REJECT**

**Neden Kritik?**
> Marker var ama sıra yanlış → determinism kırılır (sessizce)

---

### 6.4 Phase-16 Stub Gate Korunur

**Hedef:** `ci-gate-bcib-stub-build-integrity`

**Durum:** Korunur (regression detection için)

**Kapsam:** Build validation ve trace window drift detection

**Guarantee Level:** `build_only`

**Evidence Report Contract:**
```json
{
  "gate": "bcib-stub-build-integrity",
  "result": "PASS",
  "guarantee_level": "build_only",
  "does_prove": [
    "compile_success",
    "marker_strings_present",
    "trace_window_stable"
  ],
  "does_not_prove": [
    "execution",
    "determinism",
    "pipeline_integrity",
    "real_bcib_processing"
  ],
  "note": "This gate validates build artifacts only. Real execution determinism requires Phase-17 gates."
}
```

**Kural:** Phase-17 gate'leri **ek olarak** çalışır, stub gate'i replace etmez.

**Kritik Uyarı:**
> `build-integrity PASS` ≠ "BCIB hazır"  
> Sadece compile + marker check  
> Real execution Phase-17'de kanıtlanır

---

## 7. Deterministic AI Runtime (Ring3 Bootstrap)

### 7.1 Amaç

AI runtime çalışır, ama **deterministik kısıtlar** altında.

**NEYİ Kanıtlar:**
- Ring3 AI runtime boot
- Boundary bypass yok

**NEYİ Kanıtlamaz:**
- AI çıktısının semantik determinismi (Phase-18)

### 7.2 Zorunlu Kısıtlar

```bash
THREADS=1
SEED=FIXED
MODEL_HASH=FIXED
```

**Kural:** Bu kısıtlar olmadan AI runtime boot edilmez.

### 7.3 Implementasyon

**Dosya:** `userspace/ai-runtime/bootstrap.rs`

```rust
pub fn bootstrap_deterministic_ai_runtime() -> Result<(), Error> {
    // 1. Validate constraints
    if !is_single_threaded() {
        return Err(Error::NonDeterministicConfig);
    }
    
    if !is_seed_fixed() {
        return Err(Error::NonDeterministicConfig);
    }
    
    if !is_model_hash_fixed() {
        return Err(Error::NonDeterministicConfig);
    }
    
    // 2. Boot runtime
    boot_ai_runtime()?;
    
    Ok(())
}
```

---

## 8. AI Boot Gate

### 8.1 Gate

**Hedef:** `ci-gate-ai-runtime-boot`

**Kontrol:**
```bash
make ci-gate-ai-runtime-boot
```

**Validasyon:**
- Boot ediyor mu?
- Deterministic config aktif mi?

**Evidence:**
```
evidence/run-<RUN_ID>/gates/ai-runtime-boot/
├── boot.log
├── config.json
├── report.json
└── violations.txt
```

**Failure → Merge REJECT**

### 8.2 Kapsam Dışı

**Bu gate NEYİ test etmez:**
- Çıktı doğruluğu
- Semantik determinism
- Model-level doğruluk

---

## 9. Closure Evidence (Kanıt Seti)

### 9.1 Gerekli Artifactlar

```
evidence/phase17-final/
├── determinism/
│   ├── run1/
│   │   ├── raw_output.bin
│   │   ├── raw_output.sha256
│   │   └── execution_fingerprint.bin
│   └── run2/
│       ├── raw_output.bin
│       ├── raw_output.sha256
│       └── execution_fingerprint.bin
├── receipts/
│   ├── receipt_run1.json
│   └── receipt_run2.json
├── ci/
│   ├── ci-freeze.log
│   ├── gate_reports/
│   └── summary.json
└── closure_manifest.json
```

### 9.2 Evidence Üretimi (CI-Safe)

**Kural:** Evidence üretimi CI içinde repo'ya yazılmaz.

**Doğru Model:**
```bash
# CI içinde
EVIDENCE_DIR="${TMPDIR:-/tmp}/ayken-evidence/$RUN_ID"
mkdir -p "$EVIDENCE_DIR"
# ... evidence üret ...

# Artifact upload (GitHub Actions)
- name: Upload evidence
  uses: actions/upload-artifact@v4
  with:
    name: phase17-evidence
    path: /tmp/ayken-evidence/
```

**Closure sırasında:**
```bash
# Artifact download
mkdir -p evidence/phase17-final/
cp -r /downloaded-artifacts/* evidence/phase17-final/
git add evidence/phase17-final/
git commit -m "Phase-17 evidence snapshot"
```

---

## 10. Phase-17 Closure Kriterleri

Phase-17 **kapatılabilmesi** için aşağıdaki koşullar sağlanmalı:

### 10.1 Zorunlu Koşullar

- [ ] Phase-16 closure referansı mevcut
- [ ] Verification Contract v2 yürürlükte
- [ ] Execution Receipt v2 aktif
- [ ] Inline verification çekirdek içinde çalışıyor
- [ ] Real BCIB execution aktif
- [ ] Determinism gate PASS
- [ ] Fingerprint gate PASS
- [ ] AI boot gate PASS
- [ ] Fail-closed kurallar uygulanıyor
- [ ] Evidence snapshot frozen
- [ ] CI freeze PASS under declared authority run

### 10.2 Closure Manifest

**Dosya:** `reports/phase17_official_closure/closure_manifest.json`

```json
{
  "phase": 17,
  "closure_type": "official_closure",
  "closure_state": "CONFIRMED",
  "commit_sha": "<HEAD_SHA>",
  "ci_freeze_run_id": "<RUN_ID>",
  "ci_result": "PASS",
  "determinism": {
    "stub": false,
    "real_execution": true,
    "semantic": false
  },
  "verification_layer": {
    "status": "INLINE_ACTIVE",
    "mode": "kernel"
  },
  "ai_runtime": {
    "boot": true,
    "deterministic_config": true,
    "semantic_determinism": false
  },
  "evidence_snapshot": "evidence/phase17-final/",
  "limitations": [
    "AI semantic determinism not implemented (Phase-18)",
    "model-level verification not active"
  ],
  "next_phase": 18
}
```

---

## 11. Sınırlar (Felsefe ile Uyum)

### 11.1 Verification = Acceptance Condition

Verification **opsiyonel değil**, **zorunlu** acceptance condition.

### 11.2 Ağır Doğrulama CI'da

Runtime'da değil, **CI'da** ağır doğrulama yapılır.

### 11.3 Raw Output Bağlayıcı

`raw_output` bağlayıcı, `semantic_output` değil (Phase-18'de bağlayıcı olur).

### 11.4 AI State Kapsam Dışı

AI model internal state Phase-17 kapsamında değil (Phase-18).

### 11.5 İsim = Ölçtüğün Şey

"Determinism" iddiası **ölçülen** determinism (yanlış güven yok).

---

## 12. Workstream Breakdown

### Workstream 0 — Phase-16 Closure (Önkoşul)

**Görevler:**
1. `phase16-official-closure` tag oluştur
2. Evidence snapshot freeze
3. CI freeze PASS referansı kaydet

**Çıktı:** Closure manifest

---

### Workstream 1 — Verification Contract v2

**Görevler:**
1. `execution_context_snapshot` tanımı
2. Hash input sırası
3. Fail-closed kuralları
4. Failure semantics

**Dosyalar:**
- `docs/specs/phase17-execution-pipeline/VERIFICATION_CONTRACT_V2.md`

---

### Workstream 2 — Execution Receipt v2

**Görevler:**
1. Receipt struct tanımı
2. Bağlayıcı vs advisory ayrımı
3. Serialization format

**Dosyalar:**
- `kernel/include/execution_receipt.h`
- `userspace/ayken-core/src/receipt.rs`

---

### Workstream 3 — Inline Verification (Kernel)

**Görevler:**
1. State machine implementasyonu
2. `execution_slot_verify()` fonksiyonu
3. Syscall integration

**Dosyalar:**
- `kernel/sys/execution_slot.c`
- `kernel/include/execution_slot.h`

---

### Workstream 4 — Real BCIB Execution Path

**Görevler:**
1. Stub devre dışı
2. Gerçek execution hattı aktif
3. Marker'lar

**Dosyalar:**
- `Makefile` (AYKEN_BCIB_STUB_RESULT_ENABLE=0)
- `kernel/sys/bcib_execution.c`

---

### Workstream 5 — Determinism Gates (CI)

**Görevler:**
1. `ci-gate-bcib-determinism` implementasyonu
2. `ci-gate-execution-fingerprint-consistency` implementasyonu
3. Evidence generation

**Dosyalar:**
- `Makefile`
- `scripts/ci/gate-bcib-determinism.sh`
- `scripts/ci/gate-fingerprint-consistency.sh`

---

### Workstream 6 — Deterministic AI Runtime (Ring3 Bootstrap)

**Görevler:**
1. Constraint validation
2. Bootstrap logic
3. Config enforcement

**Dosyalar:**
- `userspace/ai-runtime/bootstrap.rs`
- `userspace/ai-runtime/config.toml`

---

### Workstream 7 — AI Boot Gate

**Görevler:**
1. `ci-gate-ai-runtime-boot` implementasyonu
2. Evidence generation

**Dosyalar:**
- `Makefile`
- `scripts/ci/gate-ai-runtime-boot.sh`

---

### Workstream 8 — Closure Evidence

**Görevler:**
1. Evidence snapshot structure
2. CI-safe evidence generation
3. Closure manifest

**Dosyalar:**
- `reports/phase17_official_closure/closure_manifest.json`
- `.github/workflows/ci-freeze.yml` (artifact upload)

---

## 13. Net Çerçeve

**Tek faz:** Phase-17  
**Alt yapı:** Workstream / görev (faz içinde faz yok)  
**Süre:** Belirtilmez (kanıt odaklı)

---

## 14. Referanslar

1. `ARCHITECTURE_FREEZE.md`
2. `_ayken/steering/PHASES.md`
3. `_ayken/steering/NON_OVERRIDABLE.md`
4. `docs/roadmap/CURRENT_PHASE`
5. `AYKENOS_SON_DURUM_RAPORU_2026_05_01.md`

---

## 15. İlk İmplementasyon Sırası (Kritik)

### 15.1 Phase-17 Başlangıç Şartı

**Önce:**
1. Phase-16 official closure tamamlanmalı
2. `phase16-official-closure` tag mevcut olmalı

**Sonra:**
Phase-17 başlayabilir

---

### 15.2 İlk İmplementasyon: Inline Verification Skeleton

**Hedef:** Kernel-generated result'e geçiş

**Sıra:**
```
1. Inline verification skeleton (kernel/sys/execution_slot.c)
   - State machine (EXECUTING → VERIFYING → VERIFIED → COMMITTED)
   - Verification mode (STRICT / RELAXED)
   - Commit = result buffer publish

2. ci-gate-bcib-determinism v2 (kernel output based)
   - Python-generated result → KERNEL-GENERATED result
   - İki run parity (kernel output)
   - Evidence: raw_output.bin (kernel-produced)

3. Real BCIB execution path aktif
   - Stub devre dışı (AYKEN_BCIB_STUB_RESULT_ENABLE=0)
   - Gerçek execution hattı
```

**Kritik Kural:**
> Python-generated result → KERNEL-GENERATED result geçişi yapılmadan  
> Phase-17 başlamış sayılmaz

---

### 15.3 Neden Bu Sıra?

**Eğer sıra değişirse:**
- Gate önce → implementasyon sonra ❌
  - Gate neyi test ediyor? (belirsiz)
  
- Implementasyon önce → gate sonra ✔
  - Gate gerçek davranışı test ediyor

**AykenOS Kuralı:**
> İmplementasyon → sonra gate  
> Gate → sonra iddia

---

## 16. Performans Regression Riski

### 16.1 Risk

**Inline verification her path'te çalışırsa:**
```
scheduler latency ↑
syscall latency ↑
throughput ↓
```

**Sonuç:** Phase-17 performans regression üretir (kaçınılmaz)

---

### 16.2 Çözüm

**Verification Mode:**
```c
typedef enum {
    VERIFICATION_MODE_STRICT,   // CI / debug (full verification)
    VERIFICATION_MODE_RELAXED   // Runtime (lightweight checks)
} verification_mode_t;
```

**Kullanım:**
```c
#ifdef AYKEN_VALIDATION
    verification_mode_t mode = VERIFICATION_MODE_STRICT;
#else
    verification_mode_t mode = VERIFICATION_MODE_RELAXED;
#endif

execution_slot_verify(slot, mode);
```

**Kural:**
- CI / validation builds: STRICT (full hash + fingerprint)
- Runtime / release builds: RELAXED (sanity checks only)

---

## 17. Son Uyarılar

### 17.1 Yapılmazsa Phase-17 Çöker

❌ Inline verification her path'te full crypto  
❌ Commit = internal state change  
❌ Build-integrity PASS = "BCIB hazır" algısı  
❌ Python-generated result devam eder

---

### 17.2 Yapılırsa Phase-17 Başarılı

✅ Verification mode (STRICT / RELAXED)  
✅ Commit = result buffer publish  
✅ Build-integrity guarantee_level açık  
✅ Kernel-generated result geçişi

---

**Hazırlayan:** Kenan AY - Architectural Steward  
**Tarih:** 01 Mayıs 2026  
**Versiyon:** 1.1 (Critical Risks Addressed)  
**Durum:** PLANNING

**© 2026 Kenan AY - AykenOS Project**

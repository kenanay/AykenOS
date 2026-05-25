# AykenOS Hook Konfigürasyonu

**Versiyon:** 2.0  
**Tarih:** 2026-03-14  
**Durum:** AKTİF  
**Konum:** `docs/hooks/` (`.kiro/hooks/` kaldırıldı)

## Hook Felsefesi

AykenOS hook'ları **pre-CI disiplin katmanlarıdır**, CI'ın yerini almaz. Şunları zorunlu kılar:

1. **Fail-Closed**: İhlalde dur, mimari sorunları otomatik düzeltme
2. **Path-Based**: Geniş wildcard değil, belirli dosya desenleri hedefle
3. **Evidence-Based**: Raporlar üret, manuel müdahale iste
4. **Constitutional**: ARCHITECTURE_FREEZE.md kurallarıyla uyumlu

## Aktif Hook'lar

### 1. Pre-CI Discipline ⭐ (Birincil)
**Dosya:** `docs/hooks/pre-ci-discipline.kiro.hook`  
**Olay:** `agentStop`  
**Tip:** `runCommand`  
**Komut:** `bash scripts/ci/pre_ci_discipline.sh`  
**Uygulama:** Fail-closed, ilk başarısızlıkta dur

4 temel kapıyı sırayla çalıştırır:
1. `make ci-gate-abi` — ABI kararlılığı
2. `make ci-gate-boundary` — Ring0/Ring3 sınır zorunluluğu
3. `make ci-gate-hygiene` — Depo temizliği
4. `make ci-gate-constitutional` — Anayasal uyumluluk

**Test:** `bash scripts/ci/test_pre_ci_discipline.sh` (22 özellik testi)  
**Spec:** `docs/specs/pre-ci-discipline/`

### 2. Documentation Sync Mandatory
**Dosya:** `docs/hooks/doc-sync-mandatory.kiro.hook`  
**Olay:** `fileEdited`  
**Desenler:** `shared/abi/ayken_abi.h`, `shared/abi/syscall_v2.h`, `kernel/include/ayken_abi.h`, `kernel/sys/syscall_v2.c`, `kernel/arch/x86_64/context_switch.asm`, `bootloader/efi/efi_main.c`, `ARCHITECTURE_FREEZE.md`, `Makefile`
**Eylem:** Mimari değişiklikler için dokümantasyon güncellemelerini zorunlu kıl  
**Uygulama:** Fail-closed, dokümanlar senkronize edilene kadar blokla

### 3. Ring0 Build Guard
**Dosya:** `docs/hooks/ring0-build-guard.kiro.hook`  
**Olay:** `fileEdited`  
**Desenler:** `kernel/**/*.{c,h,asm,S}`, `bootloader/**/*.{c,h,S}`, `linker.ld`  
**Eylem:** `-Werror` ile katı build doğrulaması  
**Uygulama:** Fail-closed, build hatasında dur

### 4. Rust Constitutional Check
**Dosya:** `docs/hooks/rust-constitutional-check.kiro.hook`  
**Olay:** `fileEdited`  
**Desenler:** `ayken/**/*.rs`, `ayken-core/**/*.rs`, `userspace/**/*.rs`  
**Eylem:** `cargo test` + clippy çalıştır, anayasal uyumluluğu doğrula  
**Uygulama:** Fail-closed, kilitli modül değişikliklerini reddet

### 5. ABI Drift Guard
**Dosya:** `docs/hooks/abi-drift-guard.kiro.hook`  
**Olay:** `fileEdited`  
**Desenler:** `shared/abi/ayken_abi.h`, `shared/abi/syscall_v2.h`, `kernel/include/ayken_abi.h`, `kernel/arch/x86_64/context_switch.asm`, `kernel/sys/syscall_v2.c`
**Eylem:** ABI yüzey değişikliklerini tespit et, yeniden üretim disiplinini zorunlu kıl  
**Uygulama:** Fail-closed, ABI değişiklikleri için RFC gerektirir

### 6. Ring3 Boundary Guard
**Dosya:** `docs/hooks/ring3-boundary-guard.kiro.hook`  
**Olay:** `fileEdited`  
**Desenler:** `kernel/**/*.{c,h}`, `userspace/**/*.rs`  
**Eylem:** Ring0/Ring3 ayrımını doğrula, politika sızıntısını tespit et  
**Uygulama:** Fail-closed, sınır değişiklikleri için RFC gerektirir

### 7. CI Gate Simulation (Eski — Devre Dışı)
**Dosya:** `docs/hooks/ci-gate-simulation.kiro.hook`  
**Durum:** `enabled: false` — Pre-CI Discipline hook'u tarafından değiştirildi  
**Geçiş:** Pre-CI Discipline (`runCommand` tipi) bu hook'un yerini alır

## Hook Yürütme Akışı

```
Dosya Kaydet → Hook Tetikle → Doğrulama → PASS/FAIL
                                               ↓
                                             FAIL → DUR
                                               ↓
                                        İhlali Raporla
                                               ↓
                                    Manuel Düzeltme İste
```

## Hook'ların Olmadığı Şeyler

- ❌ CI yedeği (gerçek CI kapıları merge için zorunludur)
- ❌ Branch koruması (GitHub branch kurallarını kullan)
- ❌ CODEOWNERS zorunluluğu (.github/CODEOWNERS kullan)
- ❌ Otomatik düzeltme araçları (ihlaller manuel müdahale gerektirir)

## Hook'ların Olduğu Şeyler

- ✅ Pre-CI disiplin katmanı
- ✅ Erken ihlal tespiti
- ✅ Geliştirici geri bildirim döngüsü
- ✅ Anayasal zorunluluk hatırlatıcısı

## Anayasal Uyum

Tüm hook'lar ARCHITECTURE_FREEZE.md ile uyumludur:

| Bölüm | Kural | Hook |
|-------|-------|------|
| 3.1 | Syscall Sözleşme Değişmezleri | ABI Drift Guard |
| 3.2 | Ring0/Ring3 Sınır Değişmezleri | Ring3 Boundary Guard |
| 4.1 | ABI Kapısı | Pre-CI Discipline |
| 4.2 | Sınır Kapısı | Ring0 Build Guard + Ring3 Boundary Guard |
| 4.6 | Anayasal Kapı | Rust Constitutional Check |

## Hook Bakımı

### Yeni Hook Ekleme
1. Belirli dosya desenleri tanımla (geniş wildcard yok)
2. Doğru olay tipini seç (`fileEdited`, `agentStop`)
3. Fail-closed prompt yaz (danışma dili yok)
4. Gerçek dosya değişiklikleriyle test et
5. Bu dosyada belgele

### Hook Değiştirme
1. Fail-closed semantiğini koru
2. Yol desenlerini belirli tut
3. Versiyon numarasını güncelle
4. Değişiklikleri git commit mesajında belgele

### Hook Devre Dışı Bırakma
Hook JSON'unda `"enabled": false` ayarla. Sebebi commit mesajında belgele.

## Kanıt Konumu

Hook yürütmesi kanıt üretmez. Kanıt için:
- `make ci-gate-*` komutlarını manuel çalıştır
- `evidence/run-<RUN_ID>/` dizinlerini kontrol et
- `reports/summary.json`'ı kapı kararları için incele

---

**Bakımı:** AykenOS Core Team  
**Son Güncelleme:** 2026-03-14  
**Önceki Konum:** `.kiro/hooks/HOOK_CONFIGURATION.md` (kaldırıldı)

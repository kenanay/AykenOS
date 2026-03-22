# Uygulama Planı: Alias-Aware Address Space Leak Proof

## Genel Bakış

Bu plan, AykenOS Phase 11 (Memory Model Verification) kapsamında alias-aware adres uzayı
sızıntısızlık kanıtı altyapısını adım adım inşa eder. Dil: C (kernel/mm, kernel/proc,
kernel/sys). Tüm bileşenler Ring0 mekanizma katmanında yaşar; heap tahsisi yoktur.

Uygulama sırası: veri yapıları → AliasRegistry → proc_t entegrasyonu →
sys_v2_map_memory entegrasyonu → teardown + verifier → CI gate.

## Görevler

- [x] 1. Temel veri yapıları ve başlık dosyası (`kernel/include/alias_registry.h`)
  - `alias_entry_t` ve `alias_registry_t` struct tanımlarını yaz
  - `alias_proof_result_t` struct tanımını yaz
  - `AYKEN_MAX_ALIAS_ENTRIES=32`, `AYKEN_MAX_ALIASES_PER_FRAME=8` makrolarını tanımla
  - `alias_registry_record()`, `alias_registry_remove()`, `alias_registry_find()`,
    `alias_registry_count_for_frame()` fonksiyon prototiplerini ekle
  - `alias_verifier_run()`, `alias_verifier_emit_proof()` prototiplerini ekle
  - `exit_teardown_alias_phase()` ve `proc_run_alias_proof_selftest()` prototiplerini ekle
  - `AYKEN_VALIDATION` / `AYKEN_ALIAS_PROOF_SELFTEST` makro korumalarını ekle
  - FOOTPRINT CHECKPOINT: `alias_registry_t` struct'ı tamamlandığında `sizeof(alias_registry_t)`
    ve `sizeof(proc_t)` delta'sını ölç; sonucu `alias_registry.h` başına yorum olarak ekle
    (örn: `/* alias_registry_t: ~2KB, proc_t delta: +2KB */`); sessiz şişme önlenir
  - _Gereksinimler: 1.1, 1.2, 1.3, 10.1, 10.2, 10.3_

- [x] 2. AliasRegistry çekirdek implementasyonu (`kernel/mm/alias_registry.c`)
  - [x] 2.1 `alias_registry_record()` fonksiyonunu yaz
    - NULL / sıfır / hizasız `phys_frame` kontrolü → `-EINVAL`
    - `alias_registry_find()` ile mevcut entry arama
    - Yeni entry oluşturma: `entry_count >= AYKEN_MAX_ALIAS_ENTRIES` → `-ENOMEM`
    - Duplicate tarama döngüsü → idempotent dönüş (0)
    - `alias_count >= AYKEN_MAX_ALIASES_PER_FRAME` → `-ENOMEM`
    - Başarılı kayıt: `alias_vas[alias_count++] = alias_va`
    - _Gereksinimler: 1.4, 1.5, 1.6, 1.7, 2.1, 2.2, 2.4, 10.1, 10.4, 10.5_

  - [ ]* 2.2 Property testi: `alias_registry_record()` idempotens (Property 2)
    - **Property 2: Idempotens**
    - Aynı `(phys_frame, alias_va)` çifti N kez kaydedildiğinde `alias_count` değişmez
    - **Validates: Requirements 1.5, 10.4**

  - [ ]* 2.3 Property testi: Entry kapasite sınırı (Property 3)
    - **Property 3: Entry Kapasite Sınırı**
    - 32 farklı frame kaydedildikten sonra yeni frame → `-ENOMEM`, registry değişmez
    - **Validates: Requirements 1.2, 2.1**

  - [ ]* 2.4 Property testi: Per-frame kapasite sınırı (Property 4)
    - **Property 4: Per-Frame Kapasite Sınırı**
    - Aynı frame için 8 alias kaydedildikten sonra yeni VA → `-ENOMEM`, registry değişmez
    - **Validates: Requirements 1.3, 2.2, 10.2**

  - [ ]* 2.5 Property testi: Hizasız frame reddi (Property 5)
    - **Property 5: Hizasız Frame Reddi**
    - `phys_frame & 0xFFF != 0` olan her değer için → `-EINVAL`, registry değişmez
    - **Validates: Requirements 1.6, 10.1**

  - [x] 2.6 `alias_registry_remove()` fonksiyonunu yaz
    - `alias_registry_find()` ile entry bul; bulunamazsa `-EINVAL`
    - `alias_vas` dizisinde VA'yı bul ve sil (son elemanla yer değiştir)
    - `alias_count == 0` ise `in_use = 0` yap
    - _Gereksinimler: 1.8_

  - [x] 2.7 `alias_registry_find()` ve `alias_registry_count_for_frame()` fonksiyonlarını yaz
    - `find`: `entry_count` üzerinde lineer tarama, `in_use && phys_frame == target` eşleşmesi
    - `count_for_frame`: `find` sonucundan `alias_count` döner, bulunamazsa 0
    - _Gereksinimler: 1.9, 1.10, 1.11_

  - [ ]* 2.8 Property testi: Kayıt sonrası erişilebilirlik (Property 1)
    - **Property 1: Kayıt Sonrası Erişilebilirlik**
    - Başarılı `record()` sonrası `find()` != NULL ve `count_for_frame() >= 1`
    - **Validates: Requirements 1.4, 1.9, 1.11**

  - [ ]* 2.9 Property testi: Veri bütünlüğü invariant'ları (Property 15)
    - **Property 15: Veri Bütünlüğü Invariant'ları**
    - Herhangi bir kayıt dizisi sonrası: `in_use==1` olan tüm entry'lerde
      `phys_frame & 0xFFF == 0` ve `alias_count >= 1`
    - **Validates: Requirements 10.1, 10.3**

- [x] 3. Checkpoint — AliasRegistry birim testleri
  - `kernel/tests/validation/alias_proof_test.c` içinde şu senaryoları yaz:
    `test_alias_registry_single_frame_two_aliases()`,
    `test_alias_registry_idempotent_record()`,
    `test_alias_registry_capacity_limit()`
  - Tüm testlerin geçtiğini doğrula; sorular varsa kullanıcıya sor.
  - _Gereksinimler: 1.1–1.11, 2.1–2.5_

- [x] 4. `proc_t` genişletmesi (`kernel/include/proc.h`)
  - `proc_t` struct'ına `alias_registry_t alias_reg;` alanını ekle
  - `teardown_started` bayrağının mevcut olduğunu doğrula; yoksa ekle
  - `alias_registry.h` include'unu ekle
  - _Gereksinimler: 1.1, 4.1_

- [x] 5. `sys_v2_map_memory()` entegrasyonu (`kernel/sys/syscall_v2.c`)
  - [x] 5.1 `alias_registry_record()` çağrısını PTE kurulumundan sonra ekle
    - PTE kurulumu başarılıysa `alias_registry_record(&proc->alias_reg, phys_frame, va)` çağır
    - `alias_registry_record()` `-ENOMEM` dönerse: PTE'yi geri al (unmap), `ESYS_V2_RESOURCE_BUSY` döndür
    - `alias_registry_record()` `-EINVAL` dönerse: hata kodunu yansıt
    - TRANSACTIONAL CONTRACT: mapping "committed" sayılmadan önce registry kaydı da committed olmalı;
      record fail ederse PTE rollback zorunludur — yarım commit yok
    - ROLLBACK DOĞRULAMA ZORUNLU: rollback'in gerçekten yapıldığını doğrulamak için
      `paging_get_pte_in_pml4(proc->pml4_phys, va) == 0` assert'i rollback sonrası
      eklenmeli; kısmi rollback (PTE silinmiş ama hata kodu yanlış) tam rollback
      yapmamaktan daha tehlikelidir — sistemi "temiz" sanmaya iter
    - _Gereksinimler: 3.1, 3.2, 2.3, 2.5_

  - [x] 5.2 Freeze Invariant kontrolünü ve memory barrier'ı ekle
    - `sys_v2_map_memory()` başında `proc->teardown_started == 1` kontrolü
    - Teardown aktifse `-EINVAL` döndür, PTE kurma, registry'ye yazma
    - `alias_registry_record()` içinde `teardown_started` kontrolünden önce `smp_rmb()` ekle
      (read barrier — teardown_started'ı taze oku, CPU reorder'a karşı)
    - `sys_v2_exit()` içinde `teardown_started = 1` set edilmeden önce `smp_wmb()`,
      set edildikten sonra `smp_mb()` ekle (tüm registry yazmaları görünür olsun)
    - NOT: barrier olmadan Core 1 teardown başlatır, Core 2 hâlâ registry'ye yazıyor olabilir
      → verifier yanlış snapshot alır → false negative
    - HAPPENS-BEFORE CONTRACT: barrier yerleşimi kod yorumu olarak belgelenmeli —
      `/* smp_wmb(): alias_registry_record() writes happen-before teardown_started=1 */`
      ve `/* smp_rmb(): read teardown_started after all prior writes are visible */`
      şeklinde; "barrier koydum" yeterli değil, happens-before ilişkisi reviewable olmalı
    - BARRIER SEMBOLİK KOYMA YASAĞI: barrier çağrısının yanında happens-before yorumu
      yoksa review'da reddedilmeli; yorum olmayan barrier, barrier yokmuş gibi değerlendirilir
      — 3 ay sonra "bu neden burada?" sorusuna cevap verilemeyen barrier yanlış yere taşınır
    - FREEZE INVARIANT TEST ZORUNLU (Görev 6 checkpoint'inde): `teardown_started=1`
      set edildikten sonra `sys_v2_map_memory()` çağrısı yapılmalı; `-EINVAL` döndüğü
      ve `alias_reg`'in değişmediği doğrulanmalı — bu test geçmeden 5.2 tamamlanmış sayılmaz
    - _Gereksinimler: 3.4, 4.2, 4.4, 4.5_

  - [x] 5.3 `AYKEN_VALIDATION` makro korumasını ekle
    - `alias_registry_record()` çağrısını `#if defined(AYKEN_VALIDATION)` bloğuna al
    - _Gereksinimler: 3.3_

  - [ ]* 5.4 Property testi: map_memory → Registry senkronizasyonu (Property 7)
    - **Property 7: map_memory → Registry Senkronizasyonu**
    - Başarılı `sys_v2_map_memory()` sonrası `proc->alias_reg` içinde kayıt mevcut
    - **Validates: Requirements 3.1**

  - [ ]* 5.5 Property testi: Fail-closed kapasite politikası (Property 6)
    - **Property 6: Fail-Closed Kapasite Politikası**
    - Dolu registry koşulunda `sys_v2_map_memory()` PTE kurmaz, hata döner,
      registry ile page table arasında divergence oluşmaz
    - **Validates: Requirements 2.3, 2.4, 2.5, 3.2**

  - [ ]* 5.6 Property testi: Teardown Freeze Invariantı (Property 8)
    - **Property 8: Teardown Freeze Invariantı**
    - `teardown_started == 1` iken `sys_v2_map_memory()` → `-EINVAL`,
      PTE kurulmaz, `alias_reg` değişmez
    - **Validates: Requirements 3.4, 4.1, 4.2, 4.3, 4.4**

- [x] 6. Checkpoint — sys_v2_map_memory entegrasyon testleri
  - Kapasite aşımı → PTE kurulmadığını doğrula
  - Freeze invariant → teardown sırasında mapping reddini doğrula
  - Tüm testlerin geçtiğini doğrula; sorular varsa kullanıcıya sor.
  - _Gereksinimler: 2.3–2.5, 3.1–3.4, 4.1–4.4_

- [x] 7. AliasVerifier implementasyonu (`kernel/mm/alias_verifier.c`)
  - [x] 7.1 `alias_verifier_run()` fonksiyonunu yaz
    - `proc == NULL || proc->state != PROC_ZOMBIE` → `-EINVAL`
    - `out_result` sıfırla
    - İç içe döngü: `entry_count` × `alias_count` — her VA için `paging_get_pte_in_pml4()` çağır
    - PTE == 0 → `verified_clean++`; PTE != 0 → `leaked_count++`, ilk sızan VA/phys kaydet
    - `leaked_count > 0` → `-1` döndür; aksi halde `0`
    - `proc->alias_reg` değiştirilmez (yan etki yok)
    - VERİFİER YAN ETKİ YASAĞI: verifier yalnızca ölçer, müdahale etmez.
      `alias_reg` içindeki hiçbir alan (in_use, alias_count, alias_vas, phys_frame)
      verifier çalışması sırasında yazılmamalı. Registry'yi "düzeltmeye",
      "normalize etmeye" veya "temizlemeye" çalışan verifier, proof motoru değil
      örtbas motorudur — bu `KERNEL.SAFETY.CRITICAL` ihlalidir
    - _Gereksinimler: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8_

  - [ ]* 7.2 Property testi: Verifier sayaç tutarlılığı (Property 10)
    - **Property 10: Verifier Sayaç Tutarlılığı**
    - `verified_clean + leaked_count == total_alias_entries` her zaman geçerli;
      `leaked_count == 0` → dönüş 0, `leaked_count > 0` → dönüş -1
    - **Validates: Requirements 5.4, 5.5, 5.6**

  - [ ]* 7.3 Property testi: Verifier yan etki yok (Property 11)
    - **Property 11: Verifier Yan Etki Yok**
    - `alias_verifier_run()` öncesi ve sonrası `proc->alias_reg` içeriği bit-for-bit aynı
    - **Validates: Requirements 5.7**

  - [x] 7.4 `alias_verifier_emit_proof()` fonksiyonunu yaz
    - `leaked_count == 0` → debugcon'a `[[AYKEN_ALIAS_PROOF_OK]] pid=<N> total=<M> verified=<M> leaked=0 tlb_scope=local` yaz
    - `leaked_count > 0` → debugcon'a `[[AYKEN_ALIAS_LEAK_DETECTED]] pid=<N> total=<M> verified=<V> leaked=<L> first_va=0x<VA> first_phys=0x<PA> tlb_scope=local` yaz
    - `tlb_scope=local` alanı her çıktıda zorunlu — v1 kapsam sınırını CI evidence yüzeyine taşır
    - Çıktı formatı deterministik; aynı `alias_proof_result_t` için çıktı değişmez
    - _Gereksinimler: 6.1, 6.2, 6.3, 6.4_

  - [ ]* 7.5 Property testi: Emit proof determinizmi (Property 12)
    - **Property 12: Emit Proof Determinizmi**
    - Aynı `alias_proof_result_t` ile iki kez çağrıldığında debugcon çıktısı aynı
    - **Validates: Requirements 6.3**

  - [ ]* 7.6 Property testi: Emit proof format tutarlılığı (Property 13)
    - **Property 13: Emit Proof Format Tutarlılığı**
    - `leaked_count == 0` → `[[AYKEN_ALIAS_PROOF_OK]]` token mevcut;
      `leaked_count > 0` → `[[AYKEN_ALIAS_LEAK_DETECTED]]` token mevcut
    - **Validates: Requirements 6.1, 6.2**

- [x] 8. `exit_teardown_alias_phase()` implementasyonu (`kernel/proc/proc.c`)
  - [x] 8.1 Teardown alias temizleme döngüsünü yaz (TLB flush garantili)
    - `proc->alias_reg` üzerinde iç içe döngü: her alias VA için
      `paging_unmap_in_pml4(proc->pml4_phys, va)` çağır
    - Her VA için `invlpg(va)` çağır — TLB entry'yi geçersiz kıl (ZORUNLU)
    - `sys_v2_invalidate_local_page_if_active()` gerçekten `invlpg` yaptığını doğrula;
      yapmıyorsa doğrudan `invlpg(va)` çağır
    - NOT: `pte == 0` tek başına yeterli değil — TLB'de eski mapping kalabilir;
      bu olmadan tasarım "page-table-proof" olur, "leak-proof" olmaz
    - KAYNAK KOD DOĞRULAMA ZORUNLU: `sys_v2_invalidate_local_page_if_active()`
      implementasyonunu kaynak koddan oku ve gerçekten `invlpg` instruction'ı
      ürettiğini doğrula; "muhtemelen yapıyor" varsayımı kabul edilmez —
      yapmıyorsa doğrudan `invlpg(va)` çağır, wrapper'a güvenme
    - _Gereksinimler: 5.1, 6.6, 7.3_

  - [x] 8.2 Verifier çağrısı ve fail-closed enforcement'ı ekle
    - `alias_verifier_run(proc, &result)` çağır
    - `alias_verifier_emit_proof(&result, proc->pid)` çağır
    - `verdict != 0` → `debugcon_write("[[AYKEN_ALIAS_LEAK_DETECTED]]\n")` + `halt_forever()`
    - _Gereksinimler: 6.4, 6.5, 6.6_

  - [x] 8.3 `sys_v2_exit()` içinde `exit_teardown_alias_phase()` çağrısını ekle
    - Canonical teardown'dan sonra, `PROC_ZOMBIE` state set edildikten sonra çağır
    - `teardown_started = 1` set edildiğini doğrula (Freeze Invariant)
    - LLD NOTU: Aynı phys frame'i paylaşan canonical VA ile alias VA ayrıştırması
      veri-model düzeyinde mekanik olarak tanımlanmalı — "yalnız alias VA'ları unmap et"
      ilkesi doğru, ama hangi kaydın canonical (mapping_ledger), hangisinin alias
      (alias_reg) olduğu karar yüzeyi LLD'de somut kod sözleşmesiyle yazılmalı;
      aksi halde canonical lineage korunumu (Gereksinim 7) yanlış uygulanabilir
    - CANONICAL/ALIAS MEKANİK SINIR ZORUNLU: `exit_teardown_alias_phase()` içinde
      yalnızca `proc->alias_reg` üzerinde döngü kurulmalı; `proc->mapping_ledger`'a
      hiçbir koşulda dokunulmamalı. Bu ayrım kod seviyesinde mekanik olmalı:
      `alias_reg` döngüsü ve `mapping_ledger` döngüsü aynı fonksiyonda birleştirilmemeli,
      ayrı scope'larda tutulmalı. Canonical VA yanlışlıkla silinirse test geçer
      ama veri modeli bozulur — bu sessiz veri kaybıdır
    - _Gereksinimler: 4.1, 6.6_

  - [ ]* 8.4 Property testi: Evrensel teardown temizliği (Property 9)
    - **Property 9: Evrensel Teardown Temizliği**
    - `exit_teardown_alias_phase()` sonrası `alias_reg`'deki her VA için
      `paging_get_pte_in_pml4(proc->pml4_phys, va) == 0`
    - **Validates: Requirements 5.1**

  - [ ]* 8.5 Property testi: Canonical lineage korunumu (Property 14)
    - **Property 14: Canonical Lineage Korunumu**
    - `exit_teardown_alias_phase()` sonrası `proc->mapping_ledger` içindeki
      tüm canonical kayıtlar değişmemiş
    - **Validates: Requirements 7.1, 7.2, 7.3**

  - [x] 8.6 Hard cap ABI-visible behavior contract'ını belgele ve doğrula
    - `AYKEN_MAX_ALIAS_ENTRIES=32` ve `AYKEN_MAX_ALIASES_PER_FRAME=8` sınırlarının
      validation profile'da ABI-visible behavior oluşturduğunu `alias_registry.h`'a yorum olarak ekle
    - Kapasite aşımında `sys_v2_map_memory()` `-ENOMEM` / `ESYS_V2_RESOURCE_BUSY` döner;
      bu davranış validation profile'da user-space tarafından gözlemlenebilir
    - Selftest içinde kapasite sınırına ulaşıldığında mapping'in reddedildiğini doğrulayan
      senaryo ekle (`test_alias_registry_capacity_limit()` kapsamında)
    - HARD CAP "INTERNAL DETAIL" YASAĞI: bu limitler implementation detail değil,
      ABI-visible contract'tır. `alias_registry.h` başına şu format zorunlu:
      `/* ABI-VISIBLE CONTRACT: AYKEN_MAX_ALIAS_ENTRIES=32, AYKEN_MAX_ALIASES_PER_FRAME=8
          validation profile'da sys_v2_map_memory() bu limitleri aşınca
          ESYS_V2_RESOURCE_BUSY döner; userspace bu davranışı gözlemleyebilir.
          Limit değişikliği ABI değişikliğidir — RFC gerektirir. */`
      Bu yorum olmadan biri limiti "internal" sanıp değiştirir, proof yüzeyi kayar
    - _Gereksinimler: 2.1, 2.2, 2.3, 11.3_

- [x] 9. Checkpoint — Verifier ve teardown birim testleri
  - `test_alias_verifier_clean_pass()`: teardown sonrası tüm PTE'ler sıfır → `leaked_count == 0`
  - `test_alias_verifier_leak_detection()`: kasıtlı sızdırılmış PTE → `leaked_count > 0`,
    `first_leaked_va` doğru
  - Tüm testlerin geçtiğini doğrula; sorular varsa kullanıcıya sor.
  - _Gereksinimler: 5.1–5.8, 6.1–6.6, 7.1–7.3_

- [x] 10. Validation selftest (`kernel/mm/alias_verifier.c` — makro korumalı blok)
  - `#if defined(AYKEN_VALIDATION) && (AYKEN_ALIAS_PROOF_SELFTEST == 1)` bloğu içinde
    `proc_run_alias_proof_selftest()` fonksiyonunu yaz
  - Selftest senaryoları: tek frame'e iki alias + temizleme, idempotent kayıt,
    kapasite sınırı, temiz teardown (`leaked_count == 0`), kasıtlı sızıntı tespiti
  - Selftest sonunda `[[AYKEN_ALIAS_PROOF_OK]]` witness'ını debugcon'a yaz
  - `AYKEN_ALIAS_PROOF_SELFTEST` tanımlı değilse selftest kodu derleme dışı kalır
  - SELFTEST İZOLASYON NOTU: Her senaryo bağımsız witness üretmeli; monolitik akış
    yasak. Yani her test case kendi `[[AYKEN_ALIAS_SELFTEST_PASS: <senaryo_adı>]]`
    veya `[[AYKEN_ALIAS_SELFTEST_FAIL: <senaryo_adı>]]` satırını debugcon'a yazmalı.
    Tek bir `[[AYKEN_ALIAS_PROOF_OK]]` witness'ı tüm senaryoların geçtiğini değil,
    yalnızca son adımın geçtiğini kanıtlar — bu yeterli değil. Hangi witness neden
    düştü ayırt edilebilmeli; aksi halde CI log'da "selftest geçti" görünür ama
    hangi senaryo başarısız oldu bilinemez. Nihai `[[AYKEN_ALIAS_PROOF_OK]]` yalnızca
    tüm senaryolar ayrı ayrı geçtikten sonra yazılır.
  - _Gereksinimler: 9.1, 9.2, 9.3, 9.4_

- [x] 11. CI Gate: audit script ve Makefile hedefi
  - [x] 11.1 `tools/validation/alias_proof_audit.sh` scriptini yaz
    - `boot.log` argümanını al
    - `[[AYKEN_ALIAS_PROOF_OK]]` witness'ının tam olarak 1 kez geçtiğini doğrula
    - `[[AYKEN_ALIAS_LEAK_DETECTED]]` witness'ının 0 kez geçtiğini doğrula
    - `leaked=0` alanını ve `total == verified` koşulunu doğrula
    - Başarısızlıkta `violations.txt`'e yaz ve non-zero exit kodu döndür
    - `report.json` çıktısını oluştur; `proof_scope=admitted_surface` alanını ekle
    - AUDIT SCRIPT GRANÜLERİTE ZORUNLU: her kontrol ayrı exit code üretmeli;
      aşağıdaki kontrollerin her biri bağımsız olarak doğrulanmalı ve başarısızlığı
      ayrı satır olarak `violations.txt`'e yazılmalı:
      (1) `[[AYKEN_ALIAS_PROOF_OK]]` tam olarak 1 kez mevcut
      (2) `[[AYKEN_ALIAS_LEAK_DETECTED]]` tam olarak 0 kez mevcut
      (3) `leaked=0` alanı mevcut ve değeri sayısal 0
      (4) `total` ve `verified` sayısal olarak eşit
      (5) `report.json`'da `proof_scope=admitted_surface` alanı mevcut
      Toplu "bir şeyler yanlış" mesajı yeterli değil — hangi kontrol neden
      başarısız oldu `violations.txt`'te ayrı satırda görünmeli
    - _Gereksinimler: 8.2, 8.3, 8.4, 8.6, 8.7_

  - [x] 11.2 `Makefile`'a `ci-gate-alias-proof` hedefini ekle
    - `AYKEN_VALIDATION=1 AYKEN_ALIAS_PROOF_SELFTEST=1 KERNEL_PROFILE=validation` ile çalışır
    - `run-validation-boot` çıktısını `evidence/run-$(RUN_ID)/gates/alias-proof/boot.log`'a yönlendir
    - `tools/validation/alias_proof_audit.sh` ile analiz et
    - Evidence dizinlerini oluştur: `boot.log`, `report.json`, `violations.txt`
    - _Gereksinimler: 8.1, 8.6_

  - [x] 11.3 `ci-freeze` zincirinde `ci-gate-alias-proof`'u 24. gate olarak ekle
    - `ci-kill-switch-phase13`'ten önce, mevcut 23. gate'ten sonra yerleştir
    - _Gereksinimler: 8.8_

  - [x] 11.4 `gate_alias_proof.sh`'yi mevcut boot/runtime gate pattern'ine göre yeniden düzenle
    - Çalışan gate'lerden (low-half-kheap, ring3-execution) runner profilini devral
    - Boot witness kontrolü ekle: `[[AYKEN_BOOT_OK]]` önkoşulu sağlanmalı
    - Runner profili genişlet: AYKEN_CR3_PCID=0, AYKEN_MB_SELFTEST=1, AYKEN_GATE4_POLICY_TEST=0, AYKEN_SCHED_BOOTSTRAP_POLICY=0
    - Boot audit script kullan: `phase_4_4_qemu_boot_audit.sh` ile boot witness doğrula
    - Marker log kontrolü ekle: boot witness yoksa alias audit'e geçme
    - Sıra: boot witness → runtime canlılık (opsiyonel) → alias selftest witness → audit/report
    - Makefile target'ını güncelle: runner profilini yeni değişkenlerle çağır
    - _Gereksinimler: 8.1, 8.2, 8.6, 11.1_

- [x] 12. Final checkpoint — Tüm testler ve CI gate
  - `AYKEN_VALIDATION=1 AYKEN_ALIAS_PROOF_SELFTEST=1 KERNEL_PROFILE=validation make kernel` ile derleme doğrula
  - `ci-gate-alias-proof` hedefini çalıştır; `[[AYKEN_ALIAS_PROOF_OK]]` witness'ını doğrula
  - Tüm testlerin geçtiğini doğrula; sorular varsa kullanıcıya sor.
  - _Gereksinimler: 8.1–8.8, 9.1–9.4, 11.1–11.3_

  **Teknik Borç Kaydı (Task 12 öncesi kilitlenmiş):**

  - [ ] 12.T1 Mock selftest → gerçek proc-context selftest yükseltmesi
    - Şu an `proc_run_alias_proof_selftest()` kernel late-init'te sıfırlanmış mock
      `proc_t` üzerinde çalışıyor (pid=1, PROC_ZOMBIE, sıfır pml4_phys)
    - Bu gate'i geçirir ama gerçek exit/teardown entegrasyon proof'u değildir
    - Closure için: gerçek bir user proc'un exit akışında `exit_teardown_alias_phase()`
      çağrısı üzerinden witness üretilmeli; mock selftest bu yolu temsil etmez
    - _Seviye: closure eşiği_

  - [ ] 12.T2 emit_proof format/determinizm integration coverage
    - `alias_verifier_emit_proof()` çağrıları unit testlerden kaldırıldı (gate uyumluluğu)
    - Şu an emit_proof format ve determinizm hiçbir yerde egzersiz edilmiyor
    - Closure için: Task 7.5 (emit proof determinizmi) ve 7.6 (emit proof format)
      integration test yüzeyinde tamamlanmalı
    - _Seviye: closure eşiği_

  - [ ] 12.T3 Unit test / selftest / gate witness kaynak ayrımı belgelenmeli
    - Mevcut durum: execute_alias_proof_tests() = unit, proc_run_alias_proof_selftest() = gate
    - Bu ayrım kod yorumlarında var ama resmi contract olarak belgelenmemiş
    - Closure için: alias_proof_test.h veya alias_verifier.h'a witness kaynak sözleşmesi
      açıkça yazılmalı; ileride biri yanlışlıkla unit test'e gate marker eklemesini önler
    - _Seviye: closure eşiği_

## Notlar

### Görev Öncelik Seviyeleri

**Seviye 1 — Zorunlu (minimum merge eşiği):**
Bunlar olmadan "Phase 11 v1 uygulanmış" denemez. Yıldızsız tüm ana görevler bu seviyededir:
Görev 1, 2.1, 2.6, 2.7, 3, 4, 5.1, 5.2, 5.3, 6, 7.1, 7.4, 8.1, 8.2, 8.3, 8.6, 9, 10, 11.1, 11.2, 11.3, 12

**Seviye 2 — Faz kapanışı için güçlü gerekli (closure eşiği):**
Belge dili "opsiyonel" diyor; AykenOS standardı açısından Phase 11 v1 kapandı demek için büyük kısmı tamamlanmalı:
- 2.2 (idempotens), 2.3 (entry kapasite), 2.4 (per-frame kapasite), 2.5 (hizasız frame)
- 2.8 (kayıt sonrası erişilebilirlik), 2.9 (veri bütünlüğü invariant'ları)
- 5.4 (map_memory senkronizasyonu), 5.5 (fail-closed kapasite), 5.6 (freeze invariant)
- 7.2 (verifier sayaç tutarlılığı), 7.3 (verifier yan etki yok)
- 7.5 (emit proof determinizmi), 7.6 (emit proof format)
- 8.4 (evrensel teardown temizliği), 8.5 (canonical lineage korunumu)

**Seviye 3 — Sonraya bırakılabilir:**
Kapsam genişleten veya doğruluk güvenini artıran; sistemin çekirdek mekanizmasını var etmeyen testler.
İlk merge için ertelenebilir; ancak faz kapanışından önce tamamlanması beklenir.

---

- Her görev, traceability için ilgili gereksinimlere referans verir
- Property testleri, validation selftest altyapısı üzerine inşa edilir (`AYKEN_ALIAS_PROOF_SELFTEST=1`)
- Tüm C kodu `clang -ffreestanding -m64 -mcmodel=large -fno-pic -mno-red-zone` ile derlenir
- Heap tahsisi yoktur; tüm veri yapıları statik boyutludur (`proc_t` içine gömülü)
- `halt_forever()` çağrısı `MEMORY.LEAK.INTENTIONAL` NON_OVERRIDABLE kuralının doğrudan uygulamasıdır
- **Memory ordering (Görev 5.2)**: `smp_wmb/mb/rmb` barrier'ları zorunludur — belge değil, mekanizma
- **TLB flush (Görev 8.1)**: `invlpg(va)` her alias VA için zorunludur — olmadan "page-table-proof" olur, "leak-proof" olmaz
- **Hard cap contract (Görev 8.6)**: Kapasite sınırı validation profile'da ABI-visible behavior; belgelenmeli
- **proc_t footprint (Görev 1)**: `alias_registry_t` proc_t'ye gömülünce struct boyutu ve cache-line baskısı
  artar. LLD aşamasında `sizeof(proc_t)` delta'sı ölçülmeli ve reviewable olarak belgelenmeli;
  sessiz şişme önlenmelidir. (`AYKEN_MAX_ALIAS_ENTRIES=32 × alias_entry_t` ≈ sabit ama izlenmeli)
- **CI evidence proof_scope alanı (Görev 11.1)**: `report.json` çıktısına `proof_scope=admitted_surface`
  alanı eklenmeli; kapsam sınırı yalnızca belgede değil, evidence yüzeyinde de birinci sınıf alan
  olarak taşınmalıdır. Bu AykenOS evidence disiplinine uygundur.

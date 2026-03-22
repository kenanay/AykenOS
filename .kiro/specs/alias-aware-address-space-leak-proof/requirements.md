# Gereksinim Belgesi: Alias-Aware Address Space Leak Proof

## Giriş

Bu belge, AykenOS Phase 11 (Memory Model Verification) kapsamında geliştirilen
**alias-aware adres uzayı sızıntısızlık kanıtı** özelliğinin gereksinimlerini tanımlar.

Bir süreç çıkışından (exit) sonra adres uzayındaki tüm alias eşlemelerinin —
yani birden fazla sanal adresin aynı fiziksel frame'e işaret ettiği N:1 durumların —
eksiksiz temizlendiğini doğrulayan bir altyapı kurulacaktır.

**v1 Kapsam Sınırı**: Bu gereksinimler, `sys_v2_map_memory()` üzerinden geçen
kabul edilmiş (admitted) mapping yüzeyini kapsar. Kernel-internal mapping'ler,
identity mapping'ler, shared memory ve fork/remap/COW lifecycle olayları v1
kapsamı dışındadır.

**Mevcut kanıt hattı bağlamı**: single-exit proof ✅, parametric N-exit proof ✅,
adversarial interleaving proof ✅. Bu özellik, alias-aware tam adres uzayı
sızıntısızlık kanıtını ekler.

---

## Sözlük

- **AliasRegistry**: Bir sürecin adres uzayındaki tüm alias eşlemelerini
  (phys_frame → [va_0, va_1, ..., va_N]) takip eden kernel-tarafı veri yapısı.
  `proc_t` içine gömülüdür; heap tahsisi yoktur.
- **AliasVerifier**: Süreç çıkışı sonrasında `AliasRegistry`'deki tüm kayıtların
  PTE düzeyinde temizlendiğini doğrulayan bileşen.
- **alias_entry_t**: Tek bir fiziksel frame'e eşlenen tüm sanal adresleri tutan
  kayıt yapısı.
- **alias_proof_result_t**: Doğrulama sonucunu taşıyan yapı; `total_alias_entries`,
  `verified_clean`, `leaked_count`, `first_leaked_va`, `first_leaked_phys` alanlarını içerir.
- **Admitted Surface**: `sys_v2_map_memory()` syscall'ı üzerinden geçen ve
  `AliasRegistry`'ye kaydedilen mapping yüzeyi.
- **Canonical Lineage**: Tek sanal adres → fiziksel frame eşlemesi; `mapping_ledger`
  tarafından takip edilir.
- **CI_Gate**: QEMU boot log'unu analiz ederek kanıt witness'larını doğrulayan
  otomatik CI kapısı.
- **Debugcon**: Kernel doğrulama çıktısı için kullanılan seri debug konsolu.
- **Fail-Closed**: Hata durumunda sistemi durduran (halt_forever) politika;
  sessiz başarısızlığa izin vermez.
- **Freeze_Invariant**: Teardown başladıktan sonra yeni mapping kabul edilmemesini
  garanti eden değişmez.
- **PML4**: x86_64 dört seviyeli sayfa tablosunun kök yapısı.
- **PTE**: Page Table Entry; sanal adres → fiziksel frame eşlemesini tutan sayfa
  tablosu girişi.
- **proc_t**: AykenOS süreç kontrol bloğu.
- **Ring0**: Kernel ayrıcalık seviyesi; yalnızca mekanizma kodu içerir.
- **sys_v2_map_memory**: AykenOS syscall v2 arayüzündeki bellek eşleme sistem çağrısı.
- **sys_v2_exit**: AykenOS syscall v2 arayüzündeki süreç çıkış sistem çağrısı.
- **Teardown**: Süreç çıkışı sırasında adres uzayının temizlenmesi süreci.
- **PROC_ZOMBIE**: Teardown tamamlanmış süreç durumu.

---

## Gereksinimler

### Gereksinim 1: AliasRegistry Veri Yapısı

**Kullanıcı Hikayesi:** Bir kernel geliştiricisi olarak, bir sürecin adres uzayındaki
tüm alias eşlemelerini takip eden statik boyutlu bir kayıt defteri istiyorum;
böylece teardown sırasında hangi sanal adreslerin temizlenmesi gerektiğini
belirleyebileyim.

#### Kabul Kriterleri

1. THE AliasRegistry SHALL `proc_t` yapısı içine gömülü olarak yer alır ve
   heap tahsisi gerektirmez.

2. THE AliasRegistry SHALL en fazla `AYKEN_MAX_ALIAS_ENTRIES` (32) adet fiziksel
   frame kaydını destekler. Bu limit ABI-visible contract'tır: validation profile'da
   `sys_v2_map_memory()` bu limiti aşınca `ESYS_V2_RESOURCE_BUSY` döner ve userspace
   bu davranışı gözlemleyebilir. Limit değişikliği ABI değişikliğidir — RFC gerektirir.
   `alias_registry.h` başına şu format zorunludur:
   `/* ABI-VISIBLE CONTRACT: AYKEN_MAX_ALIAS_ENTRIES=32, AYKEN_MAX_ALIASES_PER_FRAME=8
       validation profile'da sys_v2_map_memory() bu limitleri aşınca
       ESYS_V2_RESOURCE_BUSY döner; userspace bu davranışı gözlemleyebilir.
       Limit değişikliği ABI değişikliğidir — RFC gerektirir. */`

3. THE AliasRegistry SHALL her fiziksel frame için en fazla `AYKEN_MAX_ALIASES_PER_FRAME`
   (8) adet sanal adres kaydını destekler. Bu limit de ABI-visible contract kapsamındadır
   (bkz. Gereksinim 1.2).

4. WHEN `alias_registry_record()` geçerli bir `(phys_frame, alias_va)` çiftiyle
   çağrıldığında, THE AliasRegistry SHALL bu çifti kayıt altına alır ve 0 döner.

5. WHEN `alias_registry_record()` aynı `(phys_frame, alias_va)` çiftiyle ikinci
   kez çağrıldığında, THE AliasRegistry SHALL kayıt sayısını artırmaz ve 0 döner
   (idempotent davranış).

6. WHEN `alias_registry_record()` çağrısında `phys_frame` 4KB hizalı değilse
   (`phys_frame & 0xFFF != 0`), THE AliasRegistry SHALL `-EINVAL` döner ve
   kayıt yapmaz.

7. WHEN `alias_registry_record()` çağrısında `reg` veya `phys_frame` NULL/sıfır
   ise, THE AliasRegistry SHALL `-EINVAL` döner.

8. WHEN `alias_registry_remove()` geçerli bir `(phys_frame, alias_va)` çiftiyle
   çağrıldığında, THE AliasRegistry SHALL bu kaydı siler.

9. WHEN `alias_registry_find()` kayıtlı bir `phys_frame` ile çağrıldığında,
   THE AliasRegistry SHALL ilgili `alias_entry_t` pointer'ını döner.

10. WHEN `alias_registry_find()` kayıtlı olmayan bir `phys_frame` ile çağrıldığında,
    THE AliasRegistry SHALL NULL döner.

11. THE AliasRegistry SHALL `alias_registry_count_for_frame()` aracılığıyla
    belirli bir fiziksel frame için kayıtlı alias sayısını döner.

---

### Gereksinim 2: Fail-Closed Kapasite Politikası

**Kullanıcı Hikayesi:** Bir güvenlik mühendisi olarak, registry kapasitesi
aşıldığında eşlemenin sessizce devam etmesini değil, açıkça reddedilmesini
istiyorum; böylece registry ile page table arasında divergence oluşmasını
engelleyebileyim.

#### Kabul Kriterleri

1. WHEN `alias_registry_record()` çağrısında `entry_count >= AYKEN_MAX_ALIAS_ENTRIES`
   ise, THE AliasRegistry SHALL `-ENOMEM` döner ve kayıt yapmaz.

2. WHEN `alias_registry_record()` çağrısında belirli bir frame için
   `alias_count >= AYKEN_MAX_ALIASES_PER_FRAME` ise, THE AliasRegistry SHALL
   `-ENOMEM` döner ve kayıt yapmaz.

3. WHEN `sys_v2_map_memory()` çağrısında `alias_registry_record()` `-ENOMEM`
   dönerse, THE sys_v2_map_memory SHALL eşlemeyi reddeder, PTE kurmaz ve
   `ESYS_V2_RESOURCE_BUSY` döner.

4. IF `alias_registry_record()` `-ENOMEM` döndüyse, THEN THE AliasRegistry SHALL
   kayıt durumunu değiştirmez (atomik red).

5. WHILE kapasite aşımı koşulu geçerliyken, THE sys_v2_map_memory SHALL hiçbir
   koşulda PTE kurmadan önce registry kaydını atlamaz.

---

### Gereksinim 3: sys_v2_map_memory Entegrasyonu

**Kullanıcı Hikayesi:** Bir kernel geliştiricisi olarak, kullanıcı alanından
gelen her bellek eşleme isteğinin otomatik olarak alias registry'ye kaydedilmesini
istiyorum; böylece admitted surface içindeki tüm alias'ların takip edildiğini
garanti edebileyim.

#### Kabul Kriterleri

1. WHEN `sys_v2_map_memory()` bir mapping'i commit ettiğinde, THE sys_v2_map_memory
   SHALL bu eşlemeyi `proc_t.alias_reg`'e kaydetmiş sayılır. Mapping ancak PTE
   kurulumu VE `alias_registry_record()` kaydı birlikte başarılı olduğunda
   commit edilmiş sayılır; `alias_registry_record()` başarısız olursa PTE rollback
   zorunludur — kısmi commit yoktur. Rollback'in gerçekten yapıldığı
   `paging_get_pte_in_pml4(proc->pml4_phys, va) == 0` assert'i ile doğrulanmalıdır;
   kısmi rollback (PTE silinmiş ama hata kodu yanlış yansıtılmış) tam rollback
   yapmamaktan daha tehlikelidir çünkü sistemi "temiz" sanmaya iter.

2. WHEN `alias_registry_record()` başarısız olursa, THE sys_v2_map_memory SHALL
   PTE kurmaz ve hata kodu döner.

3. THE sys_v2_map_memory SHALL yalnızca `KERNEL_PROFILE=validation` veya
   `AYKEN_VALIDATION=1` ile derlenen kernel'lerde alias kaydını aktif eder.

4. WHILE `proc.teardown_started == 1` iken, THE sys_v2_map_memory SHALL bu
   süreç için `-EINVAL` döner ve yeni eşleme yapmaz (Freeze Invariant).

---

### Gereksinim 4: Teardown Freeze Invariantı

**Kullanıcı Hikayesi:** Bir doğrulama mühendisi olarak, teardown başladıktan
sonra yeni alias eşlemelerinin kabul edilmemesini istiyorum; böylece verifier'ın
çalıştığı pencerede registry'nin sabit kalmasını garanti edebileyim.

#### Kabul Kriterleri

1. WHEN `sys_v2_exit()` teardown sürecini başlattığında, THE sys_v2_exit SHALL
   `proc.teardown_started` bayrağını 1 olarak işaretler.

2. WHILE `proc.teardown_started == 1` iken, THE sys_v2_map_memory SHALL bu
   süreç için gelen tüm eşleme isteklerini `-EINVAL` ile reddeder.

3. THE Freeze_Invariant SHALL teardown başladıktan sonra `AliasRegistry`'nin
   yeni kayıt almadığını garanti eder; böylece verifier penceresi temizdir.

4. IF teardown sırasında `sys_v2_map_memory()` çağrılırsa, THEN THE sys_v2_map_memory
   SHALL PTE kurmaz ve registry'yi değiştirmez.

5. WHEN `teardown_started` yayınlandığında, THE system SHALL memory ordering'i
   şu şekilde enforce eder: önceki tüm `alias_registry_record()` yazmaları,
   verifier gözlemi başlamadan önce globally visible olmalıdır.
   (`smp_wmb()` → `teardown_started=1` → `smp_mb()` sırası zorunludur;
   `alias_registry_record()` içinde `smp_rmb()` ile taze okuma yapılmalıdır.)
   Her barrier çağrısının yanında happens-before ilişkisi kod yorumu olarak
   belgelenmek zorundadır: `/* smp_wmb(): alias_registry_record() writes
   happen-before teardown_started=1 */` formatında. Yorum olmayan barrier,
   barrier yokmuş gibi değerlendirilir ve review'da reddedilir.

---

### Gereksinim 5: AliasVerifier — Teardown Sonrası Doğrulama

**Kullanıcı Hikayesi:** Bir doğrulama mühendisi olarak, süreç çıkışı sonrasında
tüm alias eşlemelerinin PTE düzeyinde temizlendiğini otomatik olarak doğrulayan
bir mekanizma istiyorum; böylece adres uzayı sızıntısızlık kanıtını üretebiliyeyim.

#### Kabul Kriterleri

1. WHEN `alias_verifier_run()` `PROC_ZOMBIE` durumundaki bir süreçle çağrıldığında,
   THE AliasVerifier SHALL `alias_registry`'deki her alias VA için
   `paging_get_pte_in_pml4()` çağırarak PTE değerini kontrol eder.

2. WHEN bir alias VA için `paging_get_pte_in_pml4()` 0 dönerse, THE AliasVerifier
   SHALL `verified_clean` sayacını 1 artırır.

3. WHEN bir alias VA için `paging_get_pte_in_pml4()` 0'dan farklı dönerse,
   THE AliasVerifier SHALL `leaked_count` sayacını 1 artırır ve ilk sızan VA ile
   frame bilgisini kaydeder.

4. THE AliasVerifier SHALL doğrulama sonunda
   `verified_clean + leaked_count == total_alias_entries` koşulunu sağlar
   (sayaç tutarlılığı).

5. WHEN `alias_verifier_run()` tamamlandığında ve `leaked_count == 0` ise,
   THE AliasVerifier SHALL 0 döner.

6. WHEN `alias_verifier_run()` tamamlandığında ve `leaked_count > 0` ise,
   THE AliasVerifier SHALL -1 döner.

7. THE AliasVerifier SHALL `alias_verifier_run()` çalışması sırasında
   `proc.alias_reg` yapısını değiştirmez (yan etki yok). `alias_reg` içindeki
   hiçbir alan (in_use, alias_count, alias_vas, phys_frame) verifier çalışması
   sırasında yazılmamalıdır. Registry'yi "düzeltmeye", "normalize etmeye" veya
   "temizlemeye" çalışan verifier, proof motoru değil örtbas motorudur — bu
   `KERNEL.SAFETY.CRITICAL` NON_OVERRIDABLE ihlalidir.

8. WHEN `alias_verifier_run()` çağrısında `proc` NULL ise veya
   `proc->state != PROC_ZOMBIE` ise, THE AliasVerifier SHALL `-EINVAL` döner.

9. THE AliasVerifier, local-core TLB correctness'in teardown fazı tarafından
   sağlandığı ortamda çalışır. Her alias VA için `invlpg(va)` çağrısı
   `exit_teardown_alias_phase()` tarafından yapılmış olmalıdır; TLB flush
   olmadan `pte == 0` kontrolü yeterli değildir ve bu tasarım "leak-proof"
   sayılamaz. (TLB flush sorumluluğu verifier'a değil, teardown fazına aittir
   — bkz. Gereksinim 6.6) `sys_v2_invalidate_local_page_if_active()` bu görevi
   üstlenebilir; ancak çağrının gerçekten `invlpg` instruction'ı ürettiği kaynak
   koddan doğrulanmalıdır — "muhtemelen yapıyor" varsayımı kabul edilmez.
   Doğrulanmıyorsa doğrudan `invlpg(va)` çağrılmalıdır.
   v1 kapsamı: local-core TLB flush garantilidir; SMP remote-core TLB shootdown
   bu versiyonda kapsam dışıdır. (v1 assumes local-core teardown verification;
   remote-core TLB shootdown is out of scope unless explicitly enabled.)

---

### Gereksinim 6: Kanıt Yayını ve Fail-Closed Enforcement

**Kullanıcı Hikayesi:** Bir CI mühendisi olarak, teardown sonrası doğrulama
sonucunun debugcon'a deterministik bir formatta yazılmasını ve sızıntı tespit
edildiğinde sistemin durmasını istiyorum; böylece CI gate'i bu witness'ı
güvenilir biçimde doğrulayabilsin.

#### Kabul Kriterleri

1. WHEN `alias_verifier_emit_proof()` `leaked_count == 0` olan bir sonuçla
   çağrıldığında, THE AliasVerifier SHALL debugcon'a tam olarak şu formatta
   yazar:
   `[[AYKEN_ALIAS_PROOF_OK]] pid=<N> total=<M> verified=<M> leaked=0 tlb_scope=local`

2. WHEN `alias_verifier_emit_proof()` `leaked_count > 0` olan bir sonuçla
   çağrıldığında, THE AliasVerifier SHALL debugcon'a tam olarak şu formatta
   yazar:
   `[[AYKEN_ALIAS_LEAK_DETECTED]] pid=<N> total=<M> verified=<V> leaked=<L> first_va=0x<VA> first_phys=0x<PA> tlb_scope=local`

3. THE AliasVerifier SHALL `alias_verifier_emit_proof()` çıktısını deterministik
   üretir; aynı `alias_proof_result_t` girişi için çıktı her zaman aynıdır.

4. THE AliasVerifier SHALL `tlb_scope=local` alanını her proof çıktısına ekler;
   bu alan v1'in yalnızca local-core TLB flush garantilediğini, remote-core
   TLB shootdown'ın kapsam dışı olduğunu CI evidence yüzeyinde açıkça taşır.

5. WHEN `exit_teardown_alias_phase()` tamamlandığında ve `leaked_count > 0` ise,
   THE exit_teardown_alias_phase SHALL `halt_forever()` çağırır ve sistem devam
   etmez (MEMORY.LEAK.INTENTIONAL NON_OVERRIDABLE kuralı).

6. IF `alias_verifier_run()` -1 dönerse, THEN THE exit_teardown_alias_phase SHALL
   `[[AYKEN_ALIAS_LEAK_DETECTED]]` debugcon'a yazdıktan sonra `halt_forever()`
   çağırır.

7. THE exit_teardown_alias_phase SHALL teardown sırasında şu sırayı izler:
   (1) tüm alias VA'ları PML4'ten temizle ve her VA için `invlpg(va)` çağır,
   (2) `alias_verifier_run()` çağır,
   (3) `alias_verifier_emit_proof()` çağır, (4) fail-closed enforcement uygula.
   NOT: `invlpg(va)` sorumluluğu teardown fazına aittir; verifier yalnızca
   PTE durumunu gözlemler.

---

### Gereksinim 7: Canonical Lineage Korunumu

**Kullanıcı Hikayesi:** Bir kernel geliştiricisi olarak, alias teardown sürecinin
mevcut canonical mapping_ledger kayıtlarını etkilememesini istiyorum; böylece
mevcut kanıt hattının (single-exit, N-exit, adversarial) bütünlüğü korunmuş olsun.

#### Kabul Kriterleri

1. WHEN `exit_teardown_alias_phase()` tamamlandığında, THE exit_teardown_alias_phase
   SHALL `proc.mapping_ledger` içindeki canonical kayıtları değiştirmez.

2. THE AliasVerifier SHALL yalnızca `proc.alias_reg` içindeki alias kayıtlarını
   doğrular; `proc.mapping_ledger` canonical kayıtlarını okumaz veya değiştirmez.

3. WHEN `exit_teardown_alias_phase()` tamamlandığında, THE exit_teardown_alias_phase
   SHALL yalnızca `alias_registry`'de kayıtlı VA'ları `paging_unmap_in_pml4()` ile
   temizler; canonical lineage VA'larına dokunmaz. Bu ayrım kod seviyesinde mekanik
   olmalıdır: `alias_reg` döngüsü ve `mapping_ledger` döngüsü aynı fonksiyonda
   birleştirilmemeli, ayrı scope'larda tutulmalıdır. Canonical VA yanlışlıkla
   silinirse test geçer ama veri modeli sessizce bozulur.

---

### Gereksinim 8: CI Gate Entegrasyonu

**Kullanıcı Hikayesi:** Bir CI mühendisi olarak, alias-aware adres uzayı
sızıntısızlık kanıtının otomatik olarak doğrulandığı bir CI kapısı istiyorum;
böylece her merge öncesinde kanıtın geçerli olduğu garanti edilsin.

#### Kabul Kriterleri

1. THE CI_Gate SHALL `ci-gate-alias-proof` adlı bir Makefile hedefi olarak
   tanımlanır ve `AYKEN_VALIDATION=1 AYKEN_ALIAS_PROOF_SELFTEST=1
   KERNEL_PROFILE=validation` ortam değişkenleriyle çalışır.

2. WHEN `ci-gate-alias-proof` çalıştığında, THE CI_Gate SHALL QEMU boot log'unda
   `[[AYKEN_ALIAS_PROOF_OK]]` witness'ının tam olarak 1 kez geçtiğini doğrular.

3. WHEN `ci-gate-alias-proof` çalıştığında, THE CI_Gate SHALL QEMU boot log'unda
   `[[AYKEN_ALIAS_LEAK_DETECTED]]` witness'ının 0 kez geçtiğini doğrular.

4. WHEN `ci-gate-alias-proof` çalıştığında, THE CI_Gate SHALL `leaked=0` alanını
   ve `total == verified` koşulunu doğrular.

5. IF `ci-gate-alias-proof` başarısız olursa, THEN THE CI_Gate SHALL merge'i
   engeller (PR BLOCKED).

6. THE CI_Gate SHALL kanıt çıktılarını şu konumlara yazar:
   `evidence/run-<RUN_ID>/gates/alias-proof/boot.log`,
   `evidence/run-<RUN_ID>/gates/alias-proof/report.json`,
   `evidence/run-<RUN_ID>/gates/alias-proof/violations.txt`.

7. THE CI_Gate SHALL `report.json` çıktısına `proof_scope=admitted_surface` alanını
   ekler; kapsam sınırı yalnızca belgede değil, evidence yüzeyinde de birinci sınıf
   alan olarak taşınır ve otomatik araçlar tarafından parse edilebilir.

8. THE CI_Gate SHALL `ci-freeze` zincirinin 24. kapısı olarak eklenir ve
   `ci-kill-switch-phase13`'ten önce çalışır.

9. THE CI_Gate audit script'i aşağıdaki beş kontrolü bağımsız olarak doğrular ve
   her başarısızlığı `violations.txt`'e ayrı satır olarak yazar:
   (1) `[[AYKEN_ALIAS_PROOF_OK]]` tam olarak 1 kez mevcut,
   (2) `[[AYKEN_ALIAS_LEAK_DETECTED]]` tam olarak 0 kez mevcut,
   (3) `leaked=0` alanı mevcut ve değeri sayısal 0,
   (4) `total` ve `verified` sayısal olarak eşit,
   (5) `report.json`'da `proof_scope=admitted_surface` alanı mevcut.
   Toplu "bir şeyler yanlış" mesajı yeterli değildir; hangi kontrol neden
   başarısız oldu `violations.txt`'te ayrı satırda görünmelidir.

---

### Gereksinim 9: Validation Selftest

**Kullanıcı Hikayesi:** Bir doğrulama mühendisi olarak, alias proof mekanizmasının
kendi kendini test eden bir selftest moduna sahip olmasını istiyorum; böylece
CI ortamında izole ve tekrarlanabilir biçimde doğrulanabilsin.

#### Kabul Kriterleri

1. WHERE `AYKEN_ALIAS_PROOF_SELFTEST=1` ve `KERNEL_PROFILE=validation` ise,
   THE AliasVerifier SHALL `proc_run_alias_proof_selftest()` fonksiyonunu
   çalıştırır.

2. WHERE `AYKEN_ALIAS_PROOF_SELFTEST=1` değilse veya `KERNEL_PROFILE=validation`
   değilse, THE AliasVerifier SHALL selftest kodunu derleme zamanında dışarıda
   bırakır (makro koruması).

3. WHEN selftest çalıştığında, THE AliasVerifier SHALL en az şu senaryoları
   kapsar: tek frame'e iki alias kaydı ve temizlenmesi, idempotent kayıt,
   kapasite sınırı testi, temiz teardown (leaked_count == 0), kasıtlı sızıntı
   tespiti (leaked_count > 0). Her senaryo bağımsız witness üretmelidir:
   `[[AYKEN_ALIAS_SELFTEST_PASS: <senaryo_adı>]]` veya
   `[[AYKEN_ALIAS_SELFTEST_FAIL: <senaryo_adı>]]` formatında debugcon'a yazılmalıdır.
   Monolitik akış yasaktır — tek bir `[[AYKEN_ALIAS_PROOF_OK]]` tüm senaryoların
   geçtiğini kanıtlamaz; hangi senaryo düştü ayırt edilebilmelidir.

4. WHEN selftest tamamlandığında, THE AliasVerifier SHALL debugcon'a
   `[[AYKEN_ALIAS_PROOF_OK]]` witness'ını yazar; bu witness yalnızca tüm
   senaryolar ayrı ayrı geçtikten sonra yazılır.

---

### Gereksinim 10: alias_entry_t Veri Bütünlüğü

**Kullanıcı Hikayesi:** Bir kernel geliştiricisi olarak, alias kayıt yapısının
her zaman tutarlı bir durumda olmasını istiyorum; böylece verifier'ın güvenilir
veri üzerinde çalıştığını garanti edebileyim.

#### Kabul Kriterleri

1. THE AliasRegistry SHALL `alias_entry_t.phys_frame` değerinin her zaman
   4KB hizalı olduğunu garanti eder (`phys_frame & 0xFFF == 0`).

2. THE AliasRegistry SHALL `alias_entry_t.alias_count` değerinin her zaman
   `AYKEN_MAX_ALIASES_PER_FRAME` değerini aşmadığını garanti eder.

3. THE AliasRegistry SHALL `alias_entry_t.in_use == 1` iken
   `alias_entry_t.alias_count >= 1` koşulunu sağlar.

4. THE AliasRegistry SHALL aynı `alias_va` değerinin aynı entry içinde iki kez
   kaydedilmesini önler (duplicate koruması).

5. THE AliasRegistry SHALL `alias_registry_record()` başarısız olduğunda
   (`-EINVAL` veya `-ENOMEM`) kayıt yapısını değiştirmez.

---

### Gereksinim 11: v1 Kapsam Sınırı Belgelenmesi

**Kullanıcı Hikayesi:** Bir güvenlik denetçisi olarak, v1 kanıtının hangi
mapping yüzeyini kapsadığını ve hangi yüzeyi kapsamadığını açıkça bilmek
istiyorum; böylece yanlış kapsam algısından kaynaklanan güvenlik değerlendirme
hatalarını önleyebileyim.

#### Kabul Kriterleri

1. THE AliasRegistry SHALL yalnızca `sys_v2_map_memory()` üzerinden geçen
   user-space mapping'leri kaydeder; kernel-internal mapping'ler, identity
   mapping'ler ve shared memory bu registry'ye yazılmaz.

2. THE AliasRegistry SHALL `fork`, `remap` ve copy-on-write lifecycle olaylarını
   v1'de desteklemez; bu olaylar sırasında oluşan alias'lar registry'ye
   kaydedilmez.

3. THE CI_Gate SHALL `ci-gate-alias-proof` gate'inin yalnızca admitted surface
   (sys_v2_map_memory kapsamı) için kanıt ürettiğini, global authoritativeness
   iddiasında bulunmadığını belgelenmiş biçimde raporlar.

4. THE v1 implementation SHALL belgelenmiş iki açık riski taşır; bunlar bilinçli
   kapsam kararlarıdır ve Phase 11 final closure öncesinde kapatılmalıdır:

   **Risk 1 — Remote-Core TLB Correctness**: `invlpg(va)` yalnızca local-core
   TLB'yi geçersiz kılar. Multi-core sistemde Core 1, teardown sonrası hâlâ eski
   TLB entry üzerinden erişim sağlayabilir; verifier PASS verir ama sistem leak
   içerebilir. `tlb_scope=local` alanı bu sınırı CI evidence yüzeyine taşır.
   Kapatma yolu: v1.5'te remote-core TLB shootdown eklenmesi.

   **Risk 2 — Registry Linear Scan**: `alias_registry_find()` O(N) tarama
   kullanır (N ≤ 32). v1'de bounded ve kabul edilebilir; v1.5/v2'de registry
   büyüdükçe bottleneck olabilir. Kapatma yolu: v2'de hash veya sorted lookup;
   `alias_registry_find()` arayüzü kırılmaz.


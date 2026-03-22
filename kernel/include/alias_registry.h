// kernel/include/alias_registry.h
#ifndef AYKEN_ALIAS_REGISTRY_H
#define AYKEN_ALIAS_REGISTRY_H

#include <stdint.h>

/* ABI-VISIBLE CONTRACT: AYKEN_MAX_ALIAS_ENTRIES=32, AYKEN_MAX_ALIASES_PER_FRAME=8
 * validation profile'da sys_v2_map_memory() bu limitleri aşınca
 * ESYS_V2_RESOURCE_BUSY döner; userspace bu davranışı gözlemleyebilir.
 * Limit değişikliği ABI değişikliğidir — RFC gerektirir.
 */
#define AYKEN_MAX_ALIAS_ENTRIES     32
#define AYKEN_MAX_ALIASES_PER_FRAME 8

/* Forward declaration */
typedef struct proc proc_t;

/* alias_entry_t: Tek bir fiziksel frame'e eşlenen tüm sanal adresleri tutan kayıt yapısı
 * 
 * Doğrulama Kuralları:
 * - phys_frame 4KB hizalı olmalı (phys_frame & 0xFFF == 0)
 * - alias_count <= AYKEN_MAX_ALIASES_PER_FRAME
 * - in_use == 1 ise alias_count >= 1
 * - Aynı alias_va aynı entry'de iki kez kaydedilemez
 */
typedef struct {
    uint8_t  in_use;                                      /* 0=boş, 1=aktif */
    uint8_t  reserved_padding[7];                         /* alignment padding */
    uint64_t phys_frame;                                  /* izlenen fiziksel frame PA */
    uint64_t alias_vas[AYKEN_MAX_ALIASES_PER_FRAME];      /* bu frame'e eşlenen VA'lar */
    uint32_t alias_count;                                 /* geçerli alias sayısı */
    uint32_t reserved;
} alias_entry_t;

/* alias_registry_t: Bir sürecin adres uzayındaki tüm alias eşlemelerini takip eden
 * kernel-tarafı veri yapısı. proc_t içine gömülüdür; heap tahsisi yoktur.
 * 
 * Ring0 mekanizma katmanında yaşar; politika kararı içermez.
 */
typedef struct {
    alias_entry_t entries[AYKEN_MAX_ALIAS_ENTRIES];
    uint32_t      entry_count;
    uint32_t      reserved;
} alias_registry_t;

/* alias_proof_result_t: Doğrulama sonucunu taşıyan yapı
 * 
 * Doğrulama Kuralları:
 * - Kanıt geçerli ise: verified_clean == total_alias_entries && leaked_count == 0
 * - leaked_count > 0 → MEMORY.LEAK ihlali → halt_forever()
 */
typedef struct {
    uint32_t total_alias_entries;    /* toplam alias kaydı sayısı */
    uint32_t verified_clean;         /* PTE=0 doğrulanan alias VA sayısı */
    uint32_t leaked_count;           /* hâlâ present=1 olan alias VA sayısı */
    uint32_t reserved;
    uint64_t first_leaked_va;        /* ilk sızan VA (debug için) */
    uint64_t first_leaked_phys;      /* ilk sızan frame (debug için) */
} alias_proof_result_t;

/* ============================================================================
 * AliasRegistry API
 * ============================================================================ */

/* alias_registry_record: Bir (phys_frame, alias_va) çiftini kayıt altına alır
 * 
 * Önkoşullar:
 * - reg != NULL
 * - phys_frame != 0 && (phys_frame & 0xFFF) == 0 (4KB hizalı)
 * - alias_va != 0
 * - reg->entry_count <= AYKEN_MAX_ALIAS_ENTRIES
 * 
 * Sonkoşullar:
 * - Başarı (0): alias_registry_find(reg, phys_frame) != NULL
 * - Başarı (0): alias_registry_count_for_frame(reg, phys_frame) >= 1
 * - Hata (-ENOMEM): kayıt değişmez
 * - Idempotent: aynı (phys_frame, alias_va) çifti iki kez kaydedilirse ikinci çağrı 0 döner
 * 
 * Dönüş:
 * - 0: başarı
 * - -EINVAL: geçersiz girdi (NULL pointer, sıfır frame, hizasız frame)
 * - -ENOMEM: kapasite aşımı (entry_count >= AYKEN_MAX_ALIAS_ENTRIES veya
 *            alias_count >= AYKEN_MAX_ALIASES_PER_FRAME)
 */
int alias_registry_record(alias_registry_t *reg,
                          uint64_t phys_frame,
                          uint64_t alias_va);

/* alias_registry_remove: Bir (phys_frame, alias_va) çiftini kaydından siler
 * 
 * Önkoşullar:
 * - reg != NULL
 * - phys_frame != 0
 * - alias_va != 0
 * 
 * Dönüş:
 * - 0: başarı
 * - -EINVAL: entry bulunamadı veya geçersiz girdi
 */
int alias_registry_remove(alias_registry_t *reg,
                          uint64_t phys_frame,
                          uint64_t alias_va);

/* alias_registry_find: Belirli bir fiziksel frame için entry'yi bulur
 * 
 * Önkoşullar:
 * - reg != NULL
 * - phys_frame != 0
 * 
 * Dönüş:
 * - NULL olmayan pointer: entry bulundu
 * - NULL: entry bulunamadı
 */
alias_entry_t *alias_registry_find(alias_registry_t *reg,
                                   uint64_t phys_frame);

/* alias_registry_count_for_frame: Belirli bir frame için kayıtlı alias sayısını döner
 * 
 * Önkoşullar:
 * - reg != NULL
 * - phys_frame != 0
 * 
 * Dönüş:
 * - >= 0: kayıtlı alias sayısı (entry bulunamazsa 0)
 */
uint32_t alias_registry_count_for_frame(alias_registry_t *reg,
                                        uint64_t phys_frame);

/* ============================================================================
 * AliasVerifier API
 * ============================================================================ */

/* alias_verifier_run: Süreç çıkışı sonrasında alias_registry'deki tüm kayıtların
 * PTE düzeyinde temizlendiğini doğrular
 * 
 * Önkoşullar:
 * - proc != NULL && proc->state == PROC_ZOMBIE
 * - out_result != NULL
 * - proc->pml4_phys != 0
 * 
 * Sonkoşullar:
 * - out_result->verified_clean + out_result->leaked_count == out_result->total_alias_entries
 * - Dönüş 0 ⟹ out_result->leaked_count == 0
 * - Dönüş -1 ⟹ out_result->leaked_count > 0
 * - Yan etki yok: proc->alias_reg değişmez
 * 
 * VERİFİER YAN ETKİ YASAĞI: verifier yalnızca ölçer, müdahale etmez.
 * alias_reg içindeki hiçbir alan (in_use, alias_count, alias_vas, phys_frame)
 * verifier çalışması sırasında yazılmamalı. Registry'yi "düzeltmeye",
 * "normalize etmeye" veya "temizlemeye" çalışan verifier, proof motoru değil
 * örtbas motorudur — bu KERNEL.SAFETY.CRITICAL ihlalidir.
 * 
 * Dönüş:
 * - 0: kanıt geçerli (sızıntı yok)
 * - -1: sızıntı tespit edildi
 * - -EINVAL: geçersiz girdi (NULL pointer veya proc->state != PROC_ZOMBIE)
 */
int alias_verifier_run(proc_t *proc,
                       alias_proof_result_t *out_result);

/* alias_verifier_emit_proof: Doğrulama sonucunu debugcon'a yazar
 * 
 * Önkoşullar:
 * - result != NULL
 * - pid > 0
 * 
 * Sonkoşullar:
 * - leaked_count == 0 ise debugcon'a [[AYKEN_ALIAS_PROOF_OK]] yazılır
 * - leaked_count > 0 ise debugcon'a [[AYKEN_ALIAS_LEAK_DETECTED]] yazılır
 * - Çıktı formatı deterministik ve CI gate tarafından parse edilebilir
 * 
 * Çıktı Formatı:
 * [[AYKEN_ALIAS_PROOF_OK]] pid=<N> total=<M> verified=<M> leaked=0 tlb_scope=local
 * veya sızıntı durumunda:
 * [[AYKEN_ALIAS_LEAK_DETECTED]] pid=<N> total=<M> verified=<V> leaked=<L> first_va=0x<VA> first_phys=0x<PA> tlb_scope=local
 * 
 * tlb_scope=local: v1'in yalnızca local-core TLB flush garantilediğini,
 * remote-core TLB shootdown'ın kapsam dışı olduğunu proof report yüzeyinde açıkça taşır.
 */
void alias_verifier_emit_proof(const alias_proof_result_t *result,
                               int pid);

/* ============================================================================
 * Teardown API
 * ============================================================================ */

/* exit_teardown_alias_phase: Süreç çıkışı sırasında alias eşlemelerini temizler
 * ve doğrulama yapar
 * 
 * Önkoşullar:
 * - proc != NULL
 * - proc->state == PROC_ZOMBIE
 * - proc->teardown_started == 1
 * 
 * FREEZE INVARIANT: teardown_started=1 iken sys_v2_map_memory() bu proc için
 * -EINVAL döner. Yani teardown başladıktan sonra yeni alias kaydı gelmez;
 * verifier penceresi temizdir.
 * 
 * Sonkoşullar:
 * - Tüm alias VA'lar için paging_get_pte_in_pml4() == 0
 * - debugcon'da [[AYKEN_ALIAS_PROOF_OK]] witness mevcut
 * 
 * Fail-closed: leaked_count > 0 ise halt_forever() çağrılır
 * (MEMORY.LEAK.INTENTIONAL NON_OVERRIDABLE kuralı)
 */
void exit_teardown_alias_phase(proc_t *proc);

/* ============================================================================
 * Validation Selftest API
 * ============================================================================ */

#if defined(AYKEN_VALIDATION) && (AYKEN_ALIAS_PROOF_SELFTEST == 1)

/* proc_run_alias_proof_selftest: Alias proof mekanizmasının kendi kendini test
 * eden selftest modunu çalıştırır
 * 
 * Selftest senaryoları:
 * - Tek frame'e iki alias kaydı ve temizlenmesi
 * - Idempotent kayıt
 * - Kapasite sınırı testi
 * - Temiz teardown (leaked_count == 0)
 * - Kasıtlı sızıntı tespiti (leaked_count > 0)
 * 
 * SELFTEST İZOLASYON NOTU: Her senaryo bağımsız witness üretmeli; monolitik akış yasak.
 * Her test case kendi [[AYKEN_ALIAS_SELFTEST_PASS: <senaryo_adı>]] veya
 * [[AYKEN_ALIAS_SELFTEST_FAIL: <senaryo_adı>]] satırını debugcon'a yazmalı.
 * 
 * Nihai [[AYKEN_ALIAS_PROOF_OK]] yalnızca tüm senaryolar ayrı ayrı geçtikten sonra yazılır.
 * 
 * Önkoşullar:
 * - owner_proc != NULL
 */
void proc_run_alias_proof_selftest(proc_t *owner_proc);

#endif /* AYKEN_VALIDATION && AYKEN_ALIAS_PROOF_SELFTEST */

/* ============================================================================
 * FOOTPRINT CHECKPOINT
 * ============================================================================
 * 
 * alias_registry_t struct'ı tamamlandığında sizeof(alias_registry_t) ve
 * sizeof(proc_t) delta'sı ölçülmeli; sonuç bu başlığa yorum olarak eklenmeli.
 * 
 * Örnek: alias_registry_t: ~2KB, proc_t delta: +2KB
 * 
 * Bu ölçüm sessiz şişmeyi önler ve reviewable footprint tracking sağlar.
 * 
 * Hesaplama:
 * - alias_entry_t: 8 (in_use+padding) + 8 (phys_frame) + 64 (alias_vas) + 8 (counts) = 88 bytes
 * - alias_registry_t: 88 * 32 (entries) + 8 (counters) = 2824 bytes (~2.76 KB)
 * - proc_t delta: +2824 bytes (+2.76 KB)
 * 
 * MEASURED FOOTPRINT:
 * - alias_registry_t: 2824 bytes (2.76 KB)
 * - proc_t delta: +2824 bytes (+2.76 KB)
 * 
 * Cache-line impact: ~44 cache lines (64-byte lines)
 * 
 * ============================================================================ */

#endif /* AYKEN_ALIAS_REGISTRY_H */

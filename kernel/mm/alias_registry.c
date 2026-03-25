// kernel/mm/alias_registry.c
// AliasRegistry çekirdek implementasyonu
// Phase 11: Memory Model Verification — Alias-Aware Address Space Leak Proof

#include <alias_registry.h>
#include <errno.h>
#include <stddef.h>

/* ============================================================================
 * Helper Functions (Task 2.7)
 * ============================================================================ */

/* alias_registry_find: Belirli bir fiziksel frame için entry'yi bulur
 * 
 * Algoritma: entry_count üzerinde lineer tarama, in_use && phys_frame == target eşleşmesi
 * 
 * Önkoşullar:
 * - reg != NULL
 * - phys_frame != 0
 * 
 * Dönüş:
 * - NULL olmayan pointer: entry bulundu
 * - NULL: entry bulunamadı
 * 
 * Validates: Requirements 1.9, 1.10, 1.11
 */
alias_entry_t *alias_registry_find(alias_registry_t *reg, uint64_t phys_frame)
{
    if (reg == NULL || phys_frame == 0) {
        return NULL;
    }

    // Lineer tarama: in_use && phys_frame == target eşleşmesi
    for (uint32_t i = 0; i < reg->entry_count; i++) {
        alias_entry_t *entry = &reg->entries[i];
        if (entry->in_use && entry->phys_frame == phys_frame) {
            return entry;
        }
    }

    return NULL;
}

/* alias_registry_count_for_frame: Belirli bir frame için kayıtlı alias sayısını döner
 * 
 * Algoritma: find sonucundan alias_count döner, bulunamazsa 0
 * 
 * Önkoşullar:
 * - reg != NULL
 * - phys_frame != 0
 * 
 * Dönüş:
 * - >= 0: kayıtlı alias sayısı (entry bulunamazsa 0)
 * 
 * Validates: Requirements 1.9, 1.10, 1.11
 */
uint32_t alias_registry_count_for_frame(alias_registry_t *reg, uint64_t phys_frame)
{
    alias_entry_t *entry = alias_registry_find(reg, phys_frame);
    if (entry == NULL) {
        return 0;
    }
    return entry->alias_count;
}

/* ============================================================================
 * Core Registry Functions (Task 2.1)
 * ============================================================================ */

/* alias_registry_record: Bir (phys_frame, alias_va) çiftini kayıt altına alır
 * 
 * Algoritma (Pseudocode'dan):
 * 1. NULL / sıfır / hizasız phys_frame kontrolü → -EINVAL
 * 2. alias_registry_find() ile mevcut entry arama
 * 3. Entry yoksa yeni entry oluştur:
 *    - entry_count >= AYKEN_MAX_ALIAS_ENTRIES → -ENOMEM
 * 4. Duplicate tarama döngüsü → idempotent dönüş (0)
 * 5. alias_count >= AYKEN_MAX_ALIASES_PER_FRAME → -ENOMEM
 * 6. Başarılı kayıt: alias_vas[alias_count++] = alias_va
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
 * Döngü Değişmezi: Duplicate tarama döngüsünde, i < k olan tüm alias_vas[i] != alias_va
 * 
 * Validates: Requirements 1.4, 1.5, 1.6, 1.7, 2.1, 2.2, 2.4, 10.1, 10.4, 10.5
 */
int alias_registry_record(alias_registry_t *reg, uint64_t phys_frame, uint64_t alias_va)
{
    // Adım 1: NULL / sıfır / hizasız phys_frame kontrolü → -EINVAL
    if (reg == NULL || phys_frame == 0 || alias_va == 0) {
        return -EINVAL;
    }

    // Hizalama kontrolü: phys_frame 4KB hizalı olmalı (Requirement 1.6, 10.1)
    if ((phys_frame & 0xFFF) != 0) {
        return -EINVAL;
    }

    // Adım 2: Mevcut entry'yi ara
    alias_entry_t *entry = alias_registry_find(reg, phys_frame);

    // Adım 3: Entry yoksa yeni entry oluştur
    if (entry == NULL) {
        // Kapasite kontrolü: entry_count >= AYKEN_MAX_ALIAS_ENTRIES → -ENOMEM
        if (reg->entry_count >= AYKEN_MAX_ALIAS_ENTRIES) {
            return -ENOMEM;  // Requirement 2.1
        }

        // Yeni entry oluştur
        entry = &reg->entries[reg->entry_count];
        entry->in_use = 1;
        entry->phys_frame = phys_frame;
        entry->alias_count = 0;
        reg->entry_count++;
    }

    // Adım 4: Duplicate tarama döngüsü → idempotent dönüş (0)
    // LOOP INVARIANT: i < k olan tüm alias_vas[i] != alias_va
    for (uint32_t i = 0; i < entry->alias_count; i++) {
        if (entry->alias_vas[i] == alias_va) {
            return 0;  // Zaten kayıtlı, idempotent (Requirement 1.5, 10.4)
        }
    }

    // Adım 5: Per-frame kapasite kontrolü
    if (entry->alias_count >= AYKEN_MAX_ALIASES_PER_FRAME) {
        return -ENOMEM;  // Requirement 2.2
    }

    // Adım 6: Başarılı kayıt
    entry->alias_vas[entry->alias_count] = alias_va;
    entry->alias_count++;

    // POSTCONDITION: alias_registry_find(reg, phys_frame) != NULL
    // POSTCONDITION: alias_registry_count_for_frame(reg, phys_frame) >= 1
    return 0;
}

/* ============================================================================
 * Registry Removal (Task 2.6)
 * ============================================================================ */

/* alias_registry_remove: Bir (phys_frame, alias_va) çiftini kaydından siler
 * 
 * Algoritma:
 * 1. alias_registry_find() ile entry bul; bulunamazsa -EINVAL
 * 2. alias_vas dizisinde VA'yı bul ve sil (son elemanla yer değiştir)
 * 3. alias_count == 0 ise in_use = 0 yap
 * 
 * Önkoşullar:
 * - reg != NULL
 * - phys_frame != 0
 * - alias_va != 0
 * 
 * Dönüş:
 * - 0: başarı
 * - -EINVAL: entry bulunamadı veya geçersiz girdi
 * 
 * Validates: Requirements 1.8
 */
int alias_registry_remove(alias_registry_t *reg, uint64_t phys_frame, uint64_t alias_va)
{
    // Adım 1: Geçerlilik kontrolü
    if (reg == NULL || phys_frame == 0 || alias_va == 0) {
        return -EINVAL;
    }

    // Entry'yi bul
    alias_entry_t *entry = alias_registry_find(reg, phys_frame);
    if (entry == NULL) {
        return -EINVAL;  // Entry bulunamadı
    }

    // Adım 2: alias_vas dizisinde VA'yı bul ve sil (son elemanla yer değiştir)
    int found = 0;
    for (uint32_t i = 0; i < entry->alias_count; i++) {
        if (entry->alias_vas[i] == alias_va) {
            // Son elemanla yer değiştir (swap-and-pop pattern)
            entry->alias_vas[i] = entry->alias_vas[entry->alias_count - 1];
            entry->alias_count--;
            found = 1;
            break;
        }
    }

    if (!found) {
        return -EINVAL;  // VA bulunamadı
    }

    // Adım 3: alias_count == 0 ise in_use = 0 yap
    if (entry->alias_count == 0) {
        entry->in_use = 0;
    }

    return 0;
}

# AykenOS Architectural Transformation - Requirements Specification

**Oluşturan:** Kenan AY  
**Proje:** AykenOS - Veri-Merkezli AI-Native İşletim Sistemi  
**Tarih:** 9 Ocak 2026  
**Sürüm:** 2.0 - AykenOS Felsefesi Entegrasyonu

## 1. Giriş ve Vizyon

### 1.1 Proje Vizyonu
AykenOS, **geleneksel işletim sistemi paradigmalarını tamamen yeniden tanımlayan** özgün bir yaklaşımdır. Bu gereksinim spesifikasyonu, mevcut POSIX-benzeri implementasyondan **veri-merkezli, AI-native** hedef mimariye dönüşümü tanımlar.

### 1.2 Temel Paradigma Değişimi
```
Geleneksel OS:  Dosya → Komut → Çıktı
AykenOS:       Veri Nesnesi → Niyet → AI Destekli Sonuç
```

**Ayırt Edici İlkeler:**
- **Veri Birincildir**: Dosya kavramı ikincil, veri nesneleri birincil
- **AI-Native**: Yapay zeka sistem çekirdeğinde, eklenti değil
- **Bağlam Odaklı**: Komut değil, çalışma bağlamı ile etkileşim
- **Güvenli AI**: AI asla doğrudan kontrol etmez, öneri üretir

### 1.3 Kapsam
Bu dönüşüm, çekirdek mimarisi, sistem çağrısı arayüzü, veri yönetimi paradigması ve AI entegrasyonunu kapsarken sistem kararlılığı ve performansını korur.

## 2. Fonksiyonel Gereksinimler

### FR-1: Faz 1.5 Stabilizasyon Gereksinimleri ✅ TAMAMLANDI

#### FR-1.0: Faz 1.5 Kapsam Sınırları (KRİTİK) ✅
- **FR-1.0.1** ✅ **v2 syscall kodu yazılmadı** Faz 1.5 süresince
- **FR-1.0.2** ✅ **Execution-centric syscall interface geliştirmesi yapılmadı** Faz 1.5'te
- **FR-1.0.3** ✅ **Sadece mevcut POSIX-benzeri (v1) syscall seti** test edildi ve doğrulandı
- **FR-1.0.4** ✅ **Sadece round-trip işlevsellik** mevcut read/write/open/close/exit syscall'ları
- **FR-1.0.5** ✅ **ENGELLEME DOĞASI:** Faz 2, Faz 1.5 %100 tamamlanana kadar başlamadı

#### FR-1.1: Toolchain Doğrulaması ✅
- **FR-1.1.1** ✅ Tüm gerekli build araçları otomatik tespit edilebilir ve kurulabilir
- **FR-1.1.2** ✅ Windows, macOS ve Linux geliştirme ortamları için çapraz platform desteği
- **FR-1.1.3** ✅ QEMU entegrasyonu timeout yönetimi ile güvenilir boot doğrulaması sağlar
- **FR-1.1.4** ✅ Build otomasyonu başarı/başarısızlık durumlarını doğru tespit eder

#### FR-1.2: Ring3 Çalıştırma Kararlılığı ✅
- **FR-1.2.1** ✅ Ring3 kullanıcı süreçleri QEMU ortamında güvenilir çalışır
- **FR-1.2.2** ✅ INT 0x80 syscall mekanizması %100 güvenilir Ring3↔Ring0 geçişleri sağlar
- **FR-1.2.3** ✅ Tüm mevcut syscall'lar (read/write/open/close/exit) Ring3'ten doğru çalışır
- **FR-1.2.4** ✅ Ring3 süreçleri arası context switching kararlı ve tutarlıdır

#### FR-1.3: Kod Kalitesi ve Tutarlılık ✅
- **FR-1.3.1** ✅ Tüm build uyarıları giderildi
- **FR-1.3.2** ✅ GDT selector sabitleri assembly ve C kodu genelinde tutarlı
- **FR-1.3.3** ✅ Kullanılmayan kod (switch_to_user_mode) kaldırıldı veya düzgün entegre edildi
- **FR-1.3.4** ✅ Dokümantasyon mevcut implementasyon durumunu doğru yansıtır

### FR-2: Faz 2 Veri-Merkezli Sistem Gereksinimleri

#### FR-2.1: Execution-Centric Syscall Interface
- **FR-2.1.1** Faz 2 dokümantasyonuna göre tam olarak 10 syscall implement edilmeli:
  - sys_v2_map_memory (0) - Bellek eşleme
  - sys_v2_unmap_memory (1) - Bellek eşleme kaldırma
  - sys_v2_switch_context (2) - Context değiştirme
  - sys_v2_submit_execution (3) - Execution gönderme
  - sys_v2_wait_result (4) - Sonuç bekleme
  - sys_v2_interrupt_return (5) - Interrupt dönüşü
  - sys_v2_time_query (6) - Zaman sorgulama
  - sys_v2_capability_bind (7) - Capability bağlama
  - sys_v2_capability_revoke (8) - Capability iptal etme
  - sys_v2_exit (9) - Çıkış

#### FR-2.2: Meta-Veri Deposu Sistemi
- **FR-2.2.1** JSON tabanlı meta-veri deposu implement edilmeli
- **FR-2.2.2** Veri konteyneri CRUD operasyonları çalışmalı
- **FR-2.2.3** Schema validation sistemi aktif olmalı
- **FR-2.2.4** Container meta-data yönetimi functional olmalı

#### FR-2.3: Veri Türü Sistemi
- **FR-2.3.1** Tabular veri türü tam functional olmalı
- **FR-2.3.2** Text veri türü tam functional olmalı
- **FR-2.3.3** ABDF serialization/deserialization çalışmalı
- **FR-2.3.4** Veri türü extensibility desteği olmalı

#### FR-2.4: Shell-VFS Köprüsü
- **FR-2.4.1** DSL parser hiyerarşik komutları çözebilmeli (>, >>, >[])
- **FR-2.4.2** Shell context yönetimi çalışmalı
- **FR-2.4.3** Veri konteyner binding functional olmalı
- **FR-2.4.4** End-to-end veri işleme senaryosu çalışmalı

#### FR-2.5: POSIX-Veri Çift Görünümü
- **FR-2.5.1** POSIX araçları düz dosya görünümü almalı
- **FR-2.5.2** AykenOS shell veri nesnesi görünümü almalı
- **FR-2.5.3** İki yönlü senkronizasyon çalışmalı
- **FR-2.5.4** Veri tutarlılığı korunmalı

### FR-3: Faz 3 AI-Native Sistem Gereksinimleri

#### FR-3.1: AI Çekirdek Altyapısı
- **FR-3.1.1** TinyLLM runtime AykenOS'ta çalışmalı
- **FR-3.1.2** AI inference < 1 saniye sürmeli (basit sorgular)
- **FR-3.1.3** AI güvenlik sınırları aktif olmalı
- **FR-3.1.4** AI servisleri izole çalışmalı

#### FR-3.2: Shell AI Entegrasyonu
- **FR-3.2.1** Doğal dil sorguları sistem komutlarına çevrilmeli
- **FR-3.2.2** AI komut önerileri güvenli sınırlar içinde olmalı
- **FR-3.2.3** Tehlikeli komutlar için kullanıcı onayı istenmeli
- **FR-3.2.4** Bağlamsal AI yardım sistemi çalışmalı

#### FR-3.3: Veri AI Entegrasyonu
- **FR-3.3.1** AI destekli veri analizi çalışmalı
- **FR-3.3.2** Otomatik veri özetleme functional olmalı
- **FR-3.3.3** Pattern detection ve anomaly detection çalışmalı
- **FR-3.3.4** Doğal dil veri sorguları desteklenmeli

#### FR-3.4: AI Güvenlik Çerçevesi
- **FR-3.4.1** AI asla doğrudan sistem kontrolü yapmamalı
- **FR-3.4.2** Tüm AI önerileri güvenlik doğrulamasından geçmeli
- **FR-3.4.3** Kullanıcı onay mekanizması çalışmalı
- **FR-3.4.4** AI audit logging aktif olmalı

## Non-Functional Requirements

### NFR-1: Performance Requirements
- **NFR-1.1** Ring3↔Ring0 transitions must not exceed 10μs latency
- **NFR-1.2** Syscall overhead must not increase by more than 20% during transition
- **NFR-1.3** Memory usage must not increase by more than 50MB during dual-interface period
- **NFR-1.4** Boot time must not increase by more than 2 seconds

### NFR-2: Reliability Requirements  
- **NFR-2.1** System must maintain 99.9% uptime during transition period
- **NFR-2.2** Rollback to Phase 1 implementation must be possible within 5 minutes
- **NFR-2.3** No data corruption must occur during architectural transition
- **NFR-2.4** All existing functionality must remain available during transition

### NFR-3: Security Requirements
- **NFR-3.1** Capability system must prevent privilege escalation
- **NFR-3.2** Ring0 attack surface must be minimized to 10 syscalls maximum
- **NFR-3.3** Resource access must be mediated through capability tokens
- **NFR-3.4** No Ring3 code must be able to access Ring0 resources directly

### NFR-4: Maintainability Requirements
- **NFR-4.1** Code must follow existing AykenOS coding standards
- **NFR-4.2** All new interfaces must be thoroughly documented
- **NFR-4.3** Migration path must be clearly documented with examples
- **NFR-4.4** Test coverage must be maintained above 80% for new code

### NFR-5: Compatibility Requirements
- **NFR-5.1** Existing ABDF/BCIB Rust infrastructure must remain functional
## 3. Fonksiyonel Olmayan Gereksinimler

### NFR-1: Performans Gereksinimleri
- **NFR-1.1** Boot zamanı < 5 saniye (QEMU'da)
- **NFR-1.2** Syscall gecikmesi < 1 mikrosaniye
- **NFR-1.3** AI inference < 1 saniye (basit sorgular)
- **NFR-1.4** Context switch < 10 mikrosaniye
- **NFR-1.5** Veri sorgusu < 100 milisaniye (orta boyut veri)

### NFR-2: Güvenlik Gereksinimleri
- **NFR-2.1** AI asla doğrudan sistem kontrolü yapmamalı
- **NFR-2.2** Capability-based security tüm kaynak erişimlerini kontrol etmeli
- **NFR-2.3** Ring0'da sadece mekanizma, Ring3'te politika olmalı
- **NFR-2.4** Veri seviyesi erişim kontrolü çalışmalı

### NFR-3: Kullanılabilirlik Gereksinimleri
- **NFR-3.1** Doğal dil shell etkileşimi sezgisel olmalı
- **NFR-3.2** Veri-odaklı komutlar öğrenilebilir olmalı
- **NFR-3.3** AI önerileri anlaşılır açıklamalarla gelmeli
- **NFR-3.4** Hata mesajları actionable olmalı

### NFR-4: Ölçeklenebilirlik Gereksinimleri
- **NFR-4.1** Çoklu veri konteyneri desteği
- **NFR-4.2** Concurrent AI servisleri
- **NFR-4.3** Büyük veri setleri için optimizasyon
- **NFR-4.4** Çoklu platform desteği (x86_64, ARM64, RISC-V)

### NFR-5: Geriye Uyumluluk Gereksinimleri
- **NFR-5.1** Faz 2 geçiş süresince sistem kararlılığı korunmalı
- **NFR-5.2** Faz 1 uygulamaları geçiş süresince çalışmaya devam etmeli
- **NFR-5.3** Build sistemi hem eski hem yeni mimarileri desteklemeli
- **NFR-5.4** Dokümantasyon mimari değişiklikleri yansıtacak şekilde güncellenmeli

## 4. Kısıtlamalar

### C-1: Faz Sıralama Kısıtlamaları
- **C-1.1** Faz 1.5 %100 tamamlanmadan Faz 2.1 başlayamaz
- **C-1.2** Her Faz 2 alt-fazı tamamlanmadan sonraki başlayamaz
- **C-1.3** Faz 1.5 süresince mimari değişiklik yapılamaz
- **C-1.4** Legacy temizlik (Faz 2.5) son adım olmalı

### C-2: AykenOS Felsefe Uyum Kısıtlamaları
- **C-2.1** Veri-merkezli paradigma tutarlı uygulanmalı
- **C-2.2** AI-native entegrasyon güvenlik sınırları içinde kalmalı
- **C-2.3** Bağlam odaklı etkileşim korunmalı
- **C-2.4** Geleneksel dosya sistemi paradigmasından sapma olmalı

### C-3: Dokümantasyon Uyum Kısıtlamaları
- **C-3.1** Implementasyon mevcut Faz 1 dokümantasyonunu sıkı takip etmeli
- **C-3.2** Implementasyon mevcut Faz 2 dokümantasyonunu sıkı takip etmeli
- **C-3.3** Dokümante faz hedeflerinden sapma izin verilmez
- **C-3.4** Tüm değişiklikler spesifik dokümantasyon gereksinimlerine izlenebilir olmalı

### C-4: Kaynak Kısıtlamaları
- **C-4.1** Implementasyon mevcut bellek kısıtlamaları içinde çalışmalı
- **C-4.2** Ek donanım gereksinimleri getirilemez
- **C-4.3** Geliştirme mevcut toolchain ve build ortamını kullanmalı
- **C-4.4** Test mevcut QEMU doğrulama altyapısını kullanmalı

## 5. Kabul Kriterleri

### AC-1: Faz 1.5 Kabul Kriterleri ✅ TAMAMLANDI
- [x] Tüm toolchain doğrulama scriptleri Windows, macOS ve Linux'ta geçiyor
- [x] Ring3 kullanıcı süreçleri QEMU'da 1000+ iterasyon güvenilir çalışıyor
- [x] Tüm syscall'lar otomatik test suite'inde başarıyla tamamlanıyor
- [x] Temiz build'de sıfır build uyarısı
- [x] Tüm dokümantasyon implementasyon durumunu doğru yansıtıyor

### AC-2: Faz 2.1 Kabul Kriterleri (Execution-Centric Syscalls)
- [ ] Tüm 10 execution-centric syscall implement edildi ve test edildi
- [ ] Capability sistemi güvenlik testlerinde yetkisiz erişimi engelliyor
- [ ] Dual syscall interface hem v1 hem v2 çağrıları destekliyor
- [ ] Migrasyon dokümantasyonu çalışan örnekler içeriyor
- [ ] Performans regresyonu %20'den az

### AC-3: Faz 2.2 Kabul Kriterleri (Veri-Merkezli Sistem)
- [ ] Meta-veri deposu functional (JSON tabanlı)
- [ ] Tabular ve text veri türleri tam çalışıyor
- [ ] Shell DSL komutları veri nesnelerine bağlanıyor
- [ ] POSIX-veri çift görünümü çalışıyor
- [ ] `data.create`, `data.add`, `data.query` komutları functional

### AC-4: Faz 2.3 Kabul Kriterleri (Ring3 Runtime)
- [ ] VFS operasyonları tamamen Ring3'te Ring0 mekanizma ile çalışıyor
- [ ] Scheduler policy Ring3'te yapılandırılabilir algoritmalarla çalışıyor
- [ ] Cihaz erişimi özel olarak capability token'ları kullanıyor
- [ ] Ring0 bileşenlerinde policy kodu kalmıyor

### AC-5: Faz 3 Kabul Kriterleri (AI-Native Sistem)
- [ ] TinyLLM modeli AykenOS'ta çalışıyor
- [ ] Doğal dil sorguları sistem komutlarına çevriliyor
- [ ] AI önerileri güvenli sınırlar içinde uygulanıyor
- [ ] AI servisleri izole çalışıyor
- [ ] AI güvenlik çerçevesi aktif

### AC-6: Faz 2.5 Kabul Kriterleri (Legacy Temizlik)
- [ ] Ring0'da sadece 10 syscall kalıyor
- [ ] Ring0'da POSIX syscall kalmıyor
- [ ] Ring0'da policy kodu kalmıyor
- [ ] Tüm işlevsellik Ring3 runtime üzerinden çalışıyor
- [ ] Sistem tam doğrulama suite'ini geçiyor

## 6. İzlenebilirlik Matrisi

| Gereksinim | Faz 1 Dok | Faz 2 Dok | AykenOS Felsefe | İmplementasyon Görevi |
|------------|-----------|-----------|-----------------|---------------------|
| FR-1.1.1 | ✓ | - | Toolchain | Task 1.5.1.1 ✅ |
| FR-1.2.1 | ✓ | - | Ring3 Stability | Task 1.5.2.1 ✅ |
| FR-2.1.1 | - | ✓ | Execution-Centric | Task 2.1.1.1 |
| FR-2.2.1 | - | ✓ | Veri-Merkezli | Task 2.2.1.1 |
| FR-3.1.1 | - | ✓ | AI-Native | Task 2.3.1.1 |
| FR-3.2.1 | - | ✓ | Doğal Dil | Task 2.4.1.1 |

## 7. Risk Değerlendirmesi

### Yüksek Risk Öğeleri
- **R-1** **Yetersiz Test Altyapısı → Faz 1.5 Engelleyici** ✅ ÇÖZÜLDİ
- **R-2** **Dual Interface Regresyon Riski** - Geçiş süresince iki syscall interface'i birlikte var olduğunda, birindeki değişiklikler diğerini beklenmedik şekilde etkileyebilir
- **R-3** **AI Güvenlik Sınırları** - AI'nın sistem kontrolü ele geçirme riski
- **R-4** **Veri-Merkezli Paradigma Karmaşıklığı** - Geleneksel dosya sisteminden veri nesnelerine geçiş
- **R-5** **Performans Degradasyonu** - Ring3 migrasyonu performans kaybına neden olabilir

### Orta Risk Öğeleri
- **R-6** **Dokümantasyon Tutarsızlığı** - Azaltma: Düzenli dokümantasyon incelemeleri
- **R-7** **Entegrasyon Karmaşıklığı** - Azaltma: Kademeli entegrasyon yaklaşımı (A-B-C adımları)
- **R-8** **Test Kapsamı Boşlukları** - Azaltma: Otomatik test suite genişletmesi
- **R-9** **AI Model Performansı** - Azaltma: Lightweight model kullanımı ve fallback mekanizmaları

### Düşük Risk Öğeleri
- **R-10** **Build Sistemi Değişiklikleri** - Azaltma: Kademeli build sistemi güncellemeleri
- **R-11** **Kod Kalitesi Sorunları** - Azaltma: Kod inceleme süreci

## 8. Sonuç

Bu gereksinim spesifikasyonu, AykenOS'un **veri-merkezli, AI-native** vizyonunu teknik olarak gerçekleştiren kapsamlı bir dönüşüm planı sunar. Her gereksinim, projenin **özgün değer önerisini** koruyarak **teknik mükemmelliği** hedefler.

**Kritik Başarı Faktörleri:**
- Faz bazlı %100 tamamlanma zorunluluğu
- AykenOS felsefesinin tutarlı uygulanması
- AI güvenlik sınırlarının korunması
- Veri-odaklı paradigmanın doğru implementasyonu

---

**Oluşturan:** Kenan AY  
**AykenOS Architectural Transformation - Requirements Specification v2.0**  
**© 2026 AykenOS Project**
- **R-13** Timeline delays - Mitigation: Phase-based approach with clear gates
# AykenOS Dokümantasyon Planı ve Roadmap

**AykenOS - The Constitutional AI Operating System**  
*Anayasal Yapay Zeka İşletim Sistemi*

**Son Güncelleme:** 26 Nisan 2026  
**Proje Durumu:** Phase-16 Faz B (Ring3 Infrastructure PROVEN)  
**Dokümantasyon Versiyonu:** v2.1

## 📋 Genel Strateji

### Hedef Kitle
1. **Yeni Başlayanlar** - AykenOS'u ilk kez keşfedenler
2. **Sistem Yöneticileri** - Kurulum ve yapılandırma yapacaklar
3. **Geliştiriciler** - AykenOS üzerinde geliştirme yapacaklar
4. **Mimari Uzmanları** - Anayasal sistem ve felsefe ile ilgilenenler
5. **Katkıda Bulunanlar** - Projeye katkı yapmak isteyenler

### Dokümantasyon Felsefesi
- **Türkçe Öncelikli** - Ana dil Türkçe, İngilizce çeviri ikincil
- **Pratik Odaklı** - Her kavram somut örneklerle açıklanmalı
- **Felsefe Entegreli** - Teknik detaylar Türk felsefesi ile harmanlanmalı
- **Aşamalı Öğrenme** - Basit'ten karmaşığa doğru yapılandırılmalı
- **Canlı Dokümantasyon** - Kod ile senkronize, güncel tutulmalı

## 🗂️ Dokümantasyon Yapısı

### Seviye 1: Temel Kategoriler
```
docs/
├── 01-baslangic/          # Başlangıç ve Kurulum
├── 02-mimari/             # Sistem Mimarisi
├── 03-anayasal-sistem/    # Constitutional System
├── 04-gelistirme/         # Development Guide
├── 05-api-referans/       # API Reference
├── 06-felsefe/            # Philosophy & Principles
├── 07-topluluk/           # Community & Contributing
├── 08-ornekler/           # Examples & Tutorials
├── 09-sorun-giderme/      # Troubleshooting
└── 10-referans/           # Reference Materials
```

### Seviye 2: Alt Kategoriler

#### 01-baslangic/
- `hizli-baslangic.html` - Quick Start Guide ✅ (Mevcut)
- `sistem-gereksinimleri.html` - System Requirements
- `kurulum-rehberi.html` - Installation Guide ✅ (Mevcut)
- `ilk-adimlar.html` - First Steps
- `vs-code-kurulumu.html` - VS Code Setup
- `kod-yapisini-kesfetme.html` - Code Structure Exploration ✅ (Mevcut)
- `mimari-felsefe.html` - Architectural Philosophy ✅ (Mevcut)

#### 02-mimari/
- `genel-bakis.html` - Architecture Overview ✅ (Mevcut)
- `cekirdek-mimari.html` - Kernel Architecture
- `kullanici-alani.html` - Userspace Architecture
- `guvenlik-modeli.html` - Security Model
- `bellek-yonetimi.html` - Memory Management
- `surecler-ve-threadler.html` - Processes & Threads
- `ring0-ring3-ayirimi.html` - Ring0/Ring3 Separation
- `syscall-arayuzu.html` - Syscall Interface (1000-1010)
- `bcib-execution-engine.html` - BCIB Execution Engine

#### 03-anayasal-sistem/
- `anayasal-yonetisim.html` - Constitutional Governance ✅ (Mevcut)
- `kural-sistemi.html` - Rule System
- `allow-sistemi.html` - Allow System
- `waiver-sistemi.html` - Waiver System
- `ahs-sistemi.html` - Architecture Health Score
- `ahts-sistemi.html` - Architecture Health Time-Series
- `mars-sistemi.html` - Module Architecture Risk Score
- `arre-sistemi.html` - Allow → Refactor Recommendation Engine
- `arh-sistemi.html` - Auto-Refactor Hints
- `ci-gates.html` - CI Gates System (21+ Gates)
- `evidence-system.html` - Evidence System
- `phase-matrix.html` - Phase Matrix Authority

#### 04-gelistirme/
- `gelistirici-rehberi.html` - Developer Guide
- `kod-standartlari.html` - Coding Standards
- `test-yazma.html` - Writing Tests
- `debugging.html` - Debugging Guide
- `performans-optimizasyonu.html` - Performance Optimization
- `ci-cd-entegrasyonu.html` - CI/CD Integration
- `build-system.html` - Build System (Makefile)
- `toolchain-setup.html` - Toolchain Setup
- `qemu-testing.html` - QEMU Testing
- `ring3-development.html` - Ring3 Development

#### 05-api-referans/
- `abdf-api.html` - ABDF Format API
- `bcib-api.html` - BCIB Format API
- `kernel-api.html` - Kernel API
- `userspace-api.html` - Userspace API
- `cli-komutlari.html` - CLI Commands (ayken-cli v0.1)
- `rust-crates.html` - Rust Crates Reference
- `syscall-reference.html` - Syscall Reference (1000-1010)
- `abi-specification.html` - ABI Specification
- `capability-api.html` - Capability System API

#### 06-felsefe/
- `tasarim-felsefesi.html` - Design Philosophy
- `turk-yaklasimi.html` - Turkish Approach
- `anayasal-ilkeler.html` - Constitutional Principles
- `determinizm.html` - Determinism
- `mimari-estetiği.html` - Architectural Aesthetics
- `execution-centric-paradigm.html` - Execution-Centric Paradigm
- `ai-native-design.html` - AI-Native Design Philosophy
- `mechanism-policy-separation.html` - Mechanism-Policy Separation

#### 07-topluluk/
- `katkida-bulunma.html` - Contributing Guide
- `davranis-kurallari.html` - Code of Conduct
- `topluluk-rehberi.html` - Community Guide
- `iletisim.html` - Communication Channels
- `etkinlikler.html` - Events & Meetups
- `governance-participation.html` - Governance Participation
- `constitutional-compliance.html` - Constitutional Compliance Guide

#### 08-ornekler/
- `basit-uygulama.html` - Simple Application
- `kernel-modulu.html` - Kernel Module
- `userspace-servisi.html` - Userspace Service
- `anayasal-entegrasyon.html` - Constitutional Integration
- `performans-olcumu.html` - Performance Measurement
- `bcib-worker-example.html` - BCIB Worker Example
- `ring3-policy-example.html` - Ring3 Policy Example
- `capability-usage.html` - Capability Usage Examples

#### 09-sorun-giderme/
- `sik-sorunlar.html` - Common Issues
- `hata-kodlari.html` - Error Codes
- `debug-araclari.html` - Debug Tools
- `performans-sorunlari.html` - Performance Issues
- `anayasal-ihlaller.html` - Constitutional Violations
- `build-problems.html` - Build Problems
- `qemu-issues.html` - QEMU Issues
- `ring3-debugging.html` - Ring3 Debugging
- `ci-gate-failures.html` - CI Gate Failures

#### 10-referans/
- `terimler-sozlugu.html` - Glossary
- `komut-referansi.html` - Command Reference
- `yapilandirma-referansi.html` - Configuration Reference
- `mimari-kararlari.html` - Architecture Decision Records
- `surum-notlari.html` - Release Notes
- `phase-history.html` - Phase History
- `constitutional-rules.html` - Constitutional Rules Reference
- `performance-baselines.html` - Performance Baselines

## 🎯 Öncelik Sıralaması

### Faz 1: Temel Dokümantasyon (Hafta 1-2)
**Hedef**: Yeni kullanıcıların sistemi anlayıp kullanmaya başlaması
**Durum**: Phase-16 Faz B breakthrough sonrası güncelleme

1. **Kritik Öncelik**:
   - `01-baslangic/hizli-baslangic.html` ✅ (Mevcut)
   - `01-baslangic/kurulum-rehberi.html` ✅ (Mevcut - güncelleme gerekli)
   - `02-mimari/genel-bakis.html` ✅ (Mevcut - Phase-16 güncellemesi gerekli)
   - `03-anayasal-sistem/anayasal-yonetisim.html` ✅ (Mevcut)

2. **Yüksek Öncelik**:
   - `01-baslangic/sistem-gereksinimleri.html` (Yeni - Phase-16 gereksinimleri)
   - `01-baslangic/vs-code-kurulumu.html` (Yeni)
   - `04-gelistirme/gelistirici-rehberi.html` (Yeni - Ring3 development odaklı)
   - `06-felsefe/execution-centric-paradigm.html` (Yeni - 11 syscall paradigması)

### Faz 2: Teknik Derinlik (Hafta 3-4)
**Hedef**: Geliştiricilerin sistem üzerinde çalışmaya başlaması
**Odak**: Ring3 infrastructure ve BCIB worker development

1. **Kritik Öncelik**:
   - `03-anayasal-sistem/ci-gates.html` (Yeni - 21+ gates sistemi)
   - `03-anayasal-sistem/phase-matrix.html` (Yeni - Phase Matrix Authority)
   - `05-api-referans/syscall-reference.html` (Yeni - 1000-1010 syscalls)
   - `05-api-referans/cli-komutlari.html` (Güncelleme - ayken-cli v0.1)

2. **Yüksek Öncelik**:
   - `02-mimari/ring0-ring3-ayirimi.html` (Yeni - Constitutional boundary)
   - `02-mimari/bcib-execution-engine.html` (Yeni - Phase-15 BCIB v3)
   - `04-gelistirme/build-system.html` (Yeni - Makefile sistemi)
   - `04-gelistirme/ring3-development.html` (Yeni - Ring3 policy development)

### Faz 3: İleri Seviye (Hafta 5-6)
**Hedef**: Uzman kullanıcılar ve katkıda bulunanlar için kaynak
**Odak**: Constitutional system ve performance optimization

1. **Kritik Öncelik**:
   - `03-anayasal-sistem/ahs-sistemi.html` (Güncelleme - ≥95 threshold)
   - `03-anayasal-sistem/evidence-system.html` (Yeni - Immutable evidence)
   - `07-topluluk/constitutional-compliance.html` (Yeni - Compliance guide)
   - `08-ornekler/bcib-worker-example.html` (Yeni - Phase-16 Faz B)

2. **Yüksek Öncelik**:
   - `03-anayasal-sistem/ahts-sistemi.html` (Güncelleme)
   - `03-anayasal-sistem/mars-sistemi.html` (Güncelleme)
   - `09-sorun-giderme/ring3-debugging.html` (Yeni - Ring3 troubleshooting)
   - `09-sorun-giderme/ci-gate-failures.html` (Yeni - CI gate debugging)

### Faz 4: Tamamlama (Hafta 7-8)
**Hedef**: Kapsamlı referans ve topluluk kaynakları
**Odak**: Performance baselines ve multi-architecture support

1. **Orta Öncelik**:
   - `10-referans/phase-history.html` (Yeni - Phase 1-16 history)
   - `10-referans/performance-baselines.html` (Yeni - Baseline locks)
   - `08-ornekler/capability-usage.html` (Yeni - Capability examples)
   - `09-sorun-giderme/build-problems.html` (Yeni - Build troubleshooting)

## 📊 Proje Durumu Güncellemesi (2026-04-26)

### Tamamlanan Fazlar
- ✅ **Phase-15**: BCIB Execution Engine v3 (Official Closure)
- ✅ **Phase-16 Faz A**: ayken-cli v0.1 (92% Complete)
- 🔄 **Phase-16 Faz B**: Ring3 Infrastructure (30% - Ring3 breakthrough achieved)

### Kritik Başarılar
- ✅ Ring3 First-Retirement Starvation SOLVED (2026-04-24)
- ✅ Syscall Infrastructure PROVEN
- ✅ Instruction Retirement VALIDATED
- ✅ 293 unit/integration tests + 12 property tests PASS

### Güncel Teknik Metrikler
```
Kod Tabanı:              ~55,000 LOC (kernel + userspace + tools)
Test Kapsamı:            ~75-80%
Constitutional Tests:    350+ passing
CI Gates:                21+ active
Architecture Health:     ≥95 (AHS threshold)
```

### Dokümantasyon Güncellemesi Gereken Alanlar

1. **Ring3 Infrastructure** (Yüksek Öncelik)
   - Ring3 first-retirement breakthrough
   - Syscall path validation
   - Instruction retirement proof
   - BCIB worker payload development

2. **Constitutional System** (Yüksek Öncelik)
   - 21+ CI gates sistemi
   - Phase Matrix Authority
   - Evidence system (immutable)
   - Performance baseline locks

3. **Build System** (Orta Öncelik)
   - Profile-based builds (release/validation)
   - Feature flag system
   - Multi-platform support
   - Deterministic builds

4. **API References** (Orta Öncelik)
   - ayken-cli v0.1 commands
   - BCIB v3 API
   - Syscall 1000-1010 reference
   - Capability system API

## 📝 İçerik Standartları

### Her Sayfa İçin Zorunlu Bölümler
1. **Başlık ve Özet** - Sayfanın amacı ve kapsamı
2. **Ön Koşullar** - Gerekli bilgi ve kurulumlar
3. **Ana İçerik** - Konunun detaylı açıklaması
4. **Pratik Örnekler** - Kod örnekleri ve kullanım senaryoları
5. **Sonraki Adımlar** - İlgili sayfalar ve kaynaklar
6. **Referanslar** - Dış bağlantılar ve kaynaklar

### İçerik Kalite Kriterleri
- **Türkçe Dil Kalitesi** - Doğru gramer ve terminoloji
- **Teknik Doğruluk** - Güncel ve doğru bilgi
- **Kod Örnekleri** - Çalışan ve test edilmiş örnekler
- **Görsel Destekler** - Diagramlar ve ekran görüntüleri
- **Erişilebilirlik** - Screen reader ve klavye uyumlu

### Stil Rehberi
- **Başlıklar**: H1 (Sayfa başlığı), H2 (Ana bölümler), H3 (Alt bölümler)
- **Kod Blokları**: Syntax highlighting ile
- **Uyarılar**: Alert boxları ile (info, warning, success, danger)
- **Bağlantılar**: İç ve dış bağlantılar açık şekilde belirtilmeli
- **Terimler**: İlk kullanımda açıklanmalı, glossary'ye bağlanmalı

## 🔄 Güncelleme Stratejisi

### Otomatik Güncelleme
- **CI/CD Entegrasyonu** - Kod değişikliklerinde otomatik güncelleme
- **Version Tracking** - Her release ile dokümantasyon versiyonlama
- **Link Kontrolü** - Kırık bağlantıların otomatik tespiti

### Manuel Güncelleme
- **Haftalık Review** - İçerik kalitesi ve güncellik kontrolü
- **Topluluk Geri Bildirimi** - Kullanıcı önerilerinin değerlendirilmesi
- **Uzman İncelemesi** - Teknik doğruluk kontrolü

## 📊 Başarı Metrikleri

### Kullanım Metrikleri
- Sayfa görüntüleme sayıları
- Ortalama sayfa kalış süresi
- Bounce rate (hemen çıkma oranı)
- En çok ziyaret edilen sayfalar

### Kalite Metrikleri
- Kullanıcı geri bildirimleri
- GitHub issue'ları ve PR'lar
- Topluluk forumlarındaki sorular
- Dokümantasyon ile ilgili bug raporları

### Topluluk Metrikleri
- Yeni katkıda bulunan sayısı
- Dokümantasyon PR'larının sayısı
- Çeviri katkıları
- Topluluk etkinliklerine katılım

## 🌐 Çok Dilli Destek

### Birincil Dil: Türkçe
- Tüm içerik önce Türkçe yazılacak
- Türk felsefesi ve kültürel referanslar korunacak
- Teknik terimler Türkçe karşılıkları ile açıklanacak

### İkincil Dil: İngilizce
- Kritik sayfalar İngilizce'ye çevrilecek
- Uluslararası topluluk için erişim sağlanacak
- Türk felsefesi bağlamı korunarak çevrilecek

### Çeviri Süreci
1. **Türkçe İçerik Tamamlama** - Önce Türkçe versiyonu bitir
2. **Çeviri Önceliklendirme** - En önemli sayfalardan başla
3. **Topluluk Katkısı** - Çeviri için gönüllü katkıda bulunanlar
4. **Kalite Kontrolü** - Native speaker kontrolü

## 🛠️ Teknik Altyapı

### Statik Site Generator
- **Mevcut**: Manuel HTML/CSS/JS
- **Gelecek**: Jekyll, Hugo veya Docusaurus entegrasyonu
- **Avantajlar**: Markdown desteği, otomatik navigation, search

### Hosting ve Deployment
- **GitHub Pages** - Ücretsiz ve kolay
- **Netlify** - Daha gelişmiş özellikler
- **Vercel** - Hızlı deployment
- **Custom Domain** - aykenos.org/docs

### Arama ve Navigasyon
- **Client-side Search** - Lunr.js veya Fuse.js
- **Sidebar Navigation** - Otomatik kategori ağacı
- **Breadcrumbs** - Sayfa konumu gösterimi
- **Related Pages** - İlgili içerik önerileri

Bu plan, AykenOS'un kapsamlı ve kullanıcı dostu bir dokümantasyon ekosistemi oluşturmak için gerekli tüm adımları içermektedir. Öncelik sıralaması ile aşamalı bir yaklaşım benimsenmiş, hem yeni başlayanlar hem de uzman kullanıcılar için değerli kaynaklar sağlanması hedeflenmiştir.
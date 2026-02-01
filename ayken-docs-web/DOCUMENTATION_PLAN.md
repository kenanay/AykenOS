# AykenOS Dokümantasyon Planı ve Roadmap

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
- `hizli-baslangic.html` - Quick Start Guide
- `sistem-gereksinimleri.html` - System Requirements
- `kurulum-rehberi.html` - Installation Guide
- `ilk-adimlar.html` - First Steps
- `vs-code-kurulumu.html` - VS Code Setup

#### 02-mimari/
- `genel-bakis.html` - Architecture Overview
- `cekirdek-mimari.html` - Kernel Architecture
- `kullanici-alani.html` - Userspace Architecture
- `guvenlik-modeli.html` - Security Model
- `bellek-yonetimi.html` - Memory Management
- `surecler-ve-threadler.html` - Processes & Threads

#### 03-anayasal-sistem/
- `anayasal-yonetisim.html` - Constitutional Governance
- `kural-sistemi.html` - Rule System
- `allow-sistemi.html` - Allow System
- `waiver-sistemi.html` - Waiver System
- `ahs-sistemi.html` - Architecture Health Score
- `ahts-sistemi.html` - Architecture Health Time-Series
- `mars-sistemi.html` - Module Architecture Risk Score
- `arre-sistemi.html` - Allow → Refactor Recommendation Engine
- `arh-sistemi.html` - Auto-Refactor Hints

#### 04-gelistirme/
- `gelistirici-rehberi.html` - Developer Guide
- `kod-standartlari.html` - Coding Standards
- `test-yazma.html` - Writing Tests
- `debugging.html` - Debugging Guide
- `performans-optimizasyonu.html` - Performance Optimization
- `ci-cd-entegrasyonu.html` - CI/CD Integration

#### 05-api-referans/
- `abdf-api.html` - ABDF Format API
- `bcib-api.html` - BCIB Format API
- `kernel-api.html` - Kernel API
- `userspace-api.html` - Userspace API
- `cli-komutlari.html` - CLI Commands
- `rust-crates.html` - Rust Crates Reference

#### 06-felsefe/
- `tasarim-felsefesi.html` - Design Philosophy
- `turk-yaklasimi.html` - Turkish Approach
- `anayasal-ilkeler.html` - Constitutional Principles
- `determinizm.html` - Determinism
- `mimari-estetiği.html` - Architectural Aesthetics

#### 07-topluluk/
- `katkida-bulunma.html` - Contributing Guide
- `davranis-kurallari.html` - Code of Conduct
- `topluluk-rehberi.html` - Community Guide
- `iletisim.html` - Communication Channels
- `etkinlikler.html` - Events & Meetups

#### 08-ornekler/
- `basit-uygulama.html` - Simple Application
- `kernel-modulu.html` - Kernel Module
- `userspace-servisi.html` - Userspace Service
- `anayasal-entegrasyon.html` - Constitutional Integration
- `performans-olcumu.html` - Performance Measurement

#### 09-sorun-giderme/
- `sik-sorunlar.html` - Common Issues
- `hata-kodlari.html` - Error Codes
- `debug-araclari.html` - Debug Tools
- `performans-sorunlari.html` - Performance Issues
- `anayasal-ihlaller.html` - Constitutional Violations

#### 10-referans/
- `terimler-sozlugu.html` - Glossary
- `komut-referansi.html` - Command Reference
- `yapilandirma-referansi.html` - Configuration Reference
- `mimari-kararlari.html` - Architecture Decision Records
- `surum-notlari.html` - Release Notes

## 🎯 Öncelik Sıralaması

### Faz 1: Temel Dokümantasyon (Hafta 1-2)
**Hedef**: Yeni kullanıcıların sistemi anlayıp kullanmaya başlaması

1. **Kritik Öncelik**:
   - `01-baslangic/hizli-baslangic.html` ✅ (Mevcut)
   - `01-baslangic/kurulum-rehberi.html`
   - `02-mimari/genel-bakis.html`
   - `03-anayasal-sistem/anayasal-yonetisim.html`

2. **Yüksek Öncelik**:
   - `01-baslangic/sistem-gereksinimleri.html`
   - `01-baslangic/vs-code-kurulumu.html`
   - `04-gelistirme/gelistirici-rehberi.html`
   - `06-felsefe/tasarim-felsefesi.html`

### Faz 2: Teknik Derinlik (Hafta 3-4)
**Hedef**: Geliştiricilerin sistem üzerinde çalışmaya başlaması

1. **Kritik Öncelik**:
   - `03-anayasal-sistem/kural-sistemi.html`
   - `03-anayasal-sistem/allow-sistemi.html`
   - `03-anayasal-sistem/waiver-sistemi.html`
   - `05-api-referans/cli-komutlari.html`

2. **Yüksek Öncelik**:
   - `02-mimari/cekirdek-mimari.html`
   - `02-mimari/kullanici-alani.html`
   - `04-gelistirme/kod-standartlari.html`
   - `05-api-referans/abdf-api.html`
   - `05-api-referans/bcib-api.html`

### Faz 3: İleri Seviye (Hafta 5-6)
**Hedef**: Uzman kullanıcılar ve katkıda bulunanlar için kaynak

1. **Kritik Öncelik**:
   - `03-anayasal-sistem/ahs-sistemi.html`
   - `03-anayasal-sistem/ahts-sistemi.html`
   - `03-anayasal-sistem/mars-sistemi.html`
   - `07-topluluk/katkida-bulunma.html`

2. **Yüksek Öncelik**:
   - `03-anayasal-sistem/arre-sistemi.html`
   - `03-anayasal-sistem/arh-sistemi.html`
   - `08-ornekler/anayasal-entegrasyon.html`
   - `09-sorun-giderme/sik-sorunlar.html`

### Faz 4: Tamamlama (Hafta 7-8)
**Hedef**: Kapsamlı referans ve topluluk kaynakları

1. **Orta Öncelik**:
   - Kalan API referans sayfaları
   - Detaylı örnekler ve tutorial'lar
   - Sorun giderme rehberleri
   - Topluluk ve katkı rehberleri

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
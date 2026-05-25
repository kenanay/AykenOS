# AykenOS Dokümantasyon - Sonraki Adımlar

**Tarih:** 2026-03-03  
**Durum:** İlk sayfa tamamlandı, ikinci sayfa içerik hazır

## Güncel Öncelik Notu - 2026-05-24

Bu belgedeki sayfa üretim backlog'u tarihsel docs-web iş planıdır. Güncel
işletim sistemi ilerleme otoritesi
[`PROJECT_STATUS_2026_05_24.md`](PROJECT_STATUS_2026_05_24.md) ve ana repo
roadmap belgesidir.

Bugünkü teknik öncelik yeni web sayfası veya yeni OS feature'ı değildir:
PR-4A ile ortak `sample-6` olarak sınıflandırılan local performance
oynaklığının bounded izolasyon ölçümleriyle ayrılması, ardından clean-tree
remote locked-baseline kabul sonucunun alınmasıdır. Phase-17 kapanmış
sayılmaz; Phase-18 aktive edilemez.

**Düzenleyen / Geliştiren / Oluşturan / Mimari Sorumlu:** Kenan AY
**Yetki sınırı:** Dokümantasyon metadata'sı; runtime veya closure otoritesi değildir.

## Tamamlanan İşler

### ✅ Faz 1: Altyapı (TAMAMLANDI)
1. Branding ve isimlendirme
   - Resmi isim: "AykenOS - The Constitutional AI Operating System"
   - BRANDING.md rehberi
   - README.md ve web sitesi güncellendi

2. Dokümantasyon yapısı
   - 10 kategori dizin yapısı
   - Şablon sistemi (_template.html)
   - Yönetim aracı (manage-docs.sh)
   - Yapı dokümantasyonu

3. Eğitimsel yaklaşım
   - 7-seviye öğrenme yolu
   - İlk sayfa: Mimari Felsefe (TAMAMLANDI)
   - İkinci sayfa: Kod Yapısını Keşfetme (İÇERİK HAZIR)

## Devam Eden İşler

### 🚧 Faz 2: İçerik Oluşturma (DEVAM EDİYOR)

#### Seviye 1: Projeyi Tanıma

1. ✅ **Mimari Felsefe** - TAMAMLANDI
   - Geleneksel vs AykenOS
   - 11 syscall felsefesi
   - Ring0/Ring3 ayrımı
   - Pratik örnekler

2. 📝 **Kod Yapısını Keşfetme** - İÇERİK HAZIR
   - Dizin yapısı analizi
   - ABI tanımı incelemesi
   - Syscall dispatcher
   - Assembly entry point
   - Hands-on alıştırmalar
   - **Durum:** Markdown içerik hazır, HTML'e entegre edilecek

3. 📋 **Boot Sürecini Anlama** - PLANLI
   - UEFI bootloader
   - ELF loading
   - Kernel initialization
   - GDT/IDT kurulumu

## Hemen Yapılacaklar

### Öncelik 1: Kod Yapısını Keşfetme Sayfasını Tamamla

```bash
# İçerik hazır: kod-yapisini-kesfetme-content.md
# Yapılacak: HTML sayfasına entegre et
```

**Adımlar:**
1. `kod-yapisini-kesfetme-content.md` içeriğini HTML formatına çevir
2. Kod blokları ekle (syntax highlighting)
3. Bilgi kutuları ekle (info-box, warning-box)
4. Alıştırma bölümleri ekle
5. Referans kaynaklar ekle

### Öncelik 2: Boot Sürecini Anlama Sayfasını Oluştur

```bash
./scripts/manage-docs.sh create 01-baslangic boot-surecini-anlama "Boot Sürecini Anlama"
```

**İçerik Planı:**
- UEFI firmware nedir?
- Bootloader görevleri
- ELF format analizi
- Kernel initialization aşamaları
- GDT/IDT kurulumu detayları
- Hands-on: Boot log analizi

### Öncelik 3: Diğer Kategoriler için Index Sayfaları

```bash
# 02-mimari
./scripts/manage-docs.sh create 02-mimari index "Sistem Mimarisi"

# 03-anayasal-sistem
./scripts/manage-docs.sh create 03-anayasal-sistem index "Anayasal Sistem"

# ... diğer kategoriler
```

## Uzun Vadeli Plan

### Seviye 2: Bellek Yönetimi (2-3 hafta)

1. **Paging ve Virtual Memory**
   - 4-level page tables
   - TLB management
   - Page fault handling

2. **Physical Memory Allocator**
   - Bitmap allocator
   - Buddy system
   - Performance analysis

3. **Kernel Heap**
   - kmalloc/kfree implementation
   - Slab allocator
   - Memory leak detection

### Seviye 3: Context Switching (2 hafta)

1. **CPU Context Yapısı**
   - Register layout
   - Context size optimization
   - Cache line alignment

2. **Context Switch Mekanizması**
   - Assembly implementation
   - Performance measurement
   - Optimization techniques

3. **Scheduler Basics**
   - Ready queue
   - Blocked queue
   - Preemption

### Seviye 4: Syscall Detayları (2 hafta)

1. **Syscall Entry/Exit**
   - INT 0x80 vs SYSCALL instruction
   - Parameter passing
   - Return value handling

2. **Error Handling**
   - errno convention
   - Error propagation
   - Debugging techniques

3. **Performance Optimization**
   - Fast path
   - Syscall batching
   - Benchmark results

### Seviye 5: Ring3 Implementation (3 hafta)

1. **VFS Implementation**
   - Policy decisions
   - Filesystem selection
   - Access control

2. **DevFS Implementation**
   - Device management
   - Driver interface
   - Hot-plug support

3. **Scheduler Policy**
   - Priority calculation
   - Time slice allocation
   - Load balancing

### Seviye 6: BCIB Engine (3 hafta)

1. **BCIB Format**
   - Binary structure
   - Instruction encoding
   - Compression

2. **Execution Engine**
   - Interpreter
   - JIT compilation
   - Optimization

3. **Performance**
   - Benchmark suite
   - Profiling
   - Tuning

### Seviye 7: AI Integration (4 hafta)

1. **ABDF Format**
   - Data layout
   - Metadata
   - Versioning

2. **AI Runtime**
   - Model loading
   - Inference engine
   - Resource management

3. **Multi-Agent Orchestration**
   - Agent communication
   - Task scheduling
   - Coordination

## Araçlar ve Kaynaklar

### Geliştirme Araçları

```bash
# Sayfa oluşturma
./scripts/manage-docs.sh create <kategori> <dosya> <başlık>

# Listeleme
./scripts/manage-docs.sh list
./scripts/manage-docs.sh list 01-baslangic

# Doğrulama
./scripts/manage-docs.sh validate

# İstatistikler
./scripts/manage-docs.sh stats
```

### İçerik Şablonları

- `docs/_template.html` - Sayfa şablonu
- `docs/README.md` - Yapı rehberi
- `EDUCATIONAL_DOCUMENTATION_GUIDE.md` - Eğitim rehberi

### Stil Rehberleri

- `BRANDING.md` - Branding kuralları
- `TYPOGRAPHY.md` - Tipografi
- `BRAND_COLORS.md` - Renk paleti

## Katkı Süreci

### Yeni Sayfa Ekleme

1. İçerik taslağı oluştur (Markdown)
2. Kod örnekleri hazırla
3. Alıştırmalar tasarla
4. Referans kaynaklar derle
5. HTML'e entegre et
6. Review ve test
7. Commit ve push

### Kalite Kontrol

- [ ] Kod örnekleri çalışıyor mu?
- [ ] Alıştırmalar anlaşılır mı?
- [ ] Referans kaynaklar güncel mi?
- [ ] Görseller eksiksiz mi?
- [ ] Linkler çalışıyor mu?
- [ ] Mobil uyumlu mu?
- [ ] Erişilebilir mi?

## Metrikler ve Hedefler

### Kısa Vadeli (1 ay)

- [ ] Seviye 1 tamamlansın (3 sayfa)
- [ ] Her kategori için index sayfası
- [ ] 10+ kod örneği
- [ ] 20+ alıştırma

### Orta Vadeli (3 ay)

- [ ] Seviye 1-3 tamamlansın
- [ ] 50+ sayfa
- [ ] 100+ kod örneği
- [ ] Video içerik başlasın

### Uzun Vadeli (6 ay)

- [ ] Tüm seviyeler tamamlansın
- [ ] 100+ sayfa
- [ ] 500+ kod örneği
- [ ] İnteraktif örnekler
- [ ] Video serisi

## Commit Stratejisi

### Küçük, Atomik Commit'ler

```bash
# Her sayfa için ayrı commit
git add docs/01-baslangic/kod-yapisini-kesfetme.html
git commit -m "docs: Add 'Kod Yapısını Keşfetme' educational page"

# Her kategori index'i için ayrı commit
git add docs/02-mimari/index.html
git commit -m "docs: Add 'Sistem Mimarisi' category index"
```

### Commit Mesajı Formatı

```
docs: <kısa açıklama>

<detaylı açıklama>

Features:
- <özellik 1>
- <özellik 2>

Related: <ilgili dosyalar>
```

## Sonraki Toplantı Gündem

1. Kod Yapısını Keşfetme sayfası review
2. Boot Sürecini Anlama içerik planı
3. Video içerik stratejisi
4. Topluluk katkı süreci

---

**Son Güncelleme:** 2026-03-03  
**Sonraki Review:** 2026-03-10

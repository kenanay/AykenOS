# AykenOS Dokümantasyon Yapısı

Bu dizin, AykenOS'un tüm dokümantasyon içeriğini barındırır.

## Durum Otoritesi

Kategori içerikleri eğitim ve referans amaçlıdır. Güncel Phase-17
stabilizasyon durumu için
[`../PROJECT_STATUS_2026_05_24.md`](../PROJECT_STATUS_2026_05_24.md)
okunmalıdır. Phase-17 formal closure pending durumundadır; PR-4A diagnostic
PASS, performance acceptance veya closure değildir.

## Dizin Yapısı

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

## Dosya Adlandırma Kuralları

- Tüm dosyalar küçük harf ve tire ile ayrılmış olmalı: `hizli-baslangic.html`
- Her kategori dizininde bir `index.html` bulunmalı
- Türkçe karakterler kullanılabilir: `kurulum-rehberi.html`

## İçerik Standartları

Her HTML sayfası şu yapıyı takip etmelidir:

1. **Başlık ve Özet** - Sayfanın amacı
2. **Ön Koşullar** - Gerekli bilgi
3. **Ana İçerik** - Detaylı açıklama
4. **Pratik Örnekler** - Kod örnekleri
5. **Sonraki Adımlar** - İlgili sayfalar
6. **Referanslar** - Dış kaynaklar

## Güncelleme

Dokümantasyon güncellemeleri için:
- Ana plan: `../DOCUMENTATION_PLAN.md`
- Stil rehberi: `../TYPOGRAPHY.md`
- Renk paleti: `../BRAND_COLORS.md`

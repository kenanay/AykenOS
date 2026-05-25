# Referans Materyalleri

Bu bölüm AykenOS ile ilgili referans materyalleri ve kaynak dokümanları içerir.

## Güncel Durum Referansı - 2026-05-24

- Docs-web durum senkronizasyonu: [`../../PROJECT_STATUS_2026_05_24.md`](../../PROJECT_STATUS_2026_05_24.md)
- Ana execution roadmap: `docs/roadmap/CONSTITUTIONAL_STABILIZATION_ROADMAP_2026_05_23.md`
- Faz otoritesi: Phase-16 son resmi kapanış; Phase-17 aktif ve kapanış bekliyor.
- Performans otoritesi: PR-4 local readiness `FAIL`; PR-4A diagnostic
  `PASS` yalnız ortak `sample-6` variance fingerprint kaydıdır; PR-4B
  bounded diagnostic `PASS` bu outlier'ı yeniden üretmemiş, fakat acceptance
  veya closure kurmamıştır.

## İçerik

- **Terimler Sözlüğü** - Glossary of terms
- **Komut Referansı** - Complete command reference
- **Yapılandırma Referansı** - Configuration file reference
- **Mimari Kararları** - Architecture Decision Records (ADR)
- **Sürüm Notları** - Release notes and changelogs

## Referans Kategorileri

### Terminoloji
- AykenOS özel terimleri
- Anayasal sistem kavramları
- Teknik terimler sözlüğü
- Türkçe-İngilizce karşılıklar

### Komut Referansı
- CLI komutlarının tam listesi
- Parameter açıklamaları
- Kullanım örnekleri
- Exit code'lar

### Yapılandırma
- Configuration file formatları
- Environment variables
- Build-time options
- Runtime parameters

### Mimari Kararları
- Design decision records
- Trade-off analyses
- Alternative evaluations
- Implementation rationales

## Hızlı Referans

### Temel Komutlar
```bash
ayken check                 # Constitutional compliance check
ayken ahs check            # Architecture health score
ayken waiver list          # List active waivers
ayken fix --safe          # Apply safe fixes
```

### Yapılandırma Dosyaları
- `.ayken/config.toml` - Main configuration
- `.ayken/waivers.toml` - Waiver definitions
- `.ayken/steering/` - Steering files
- `.vscode/settings.json` - VS Code integration

### Önemli Dizinler
- `kernel/` - Kernel source code
- `userspace/` - Userspace applications
- `ayken-core/` - Core libraries
- `docs/` - Documentation

## Hedef Kitle

- Hızlı referans arayanlar
- API dokümantasyonu kullananlar
- Configuration yapanlar
- Sistem yöneticileri

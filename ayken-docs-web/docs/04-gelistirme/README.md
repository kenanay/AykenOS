# Geliştirme Dokümantasyonu

Bu bölüm AykenOS üzerinde geliştirme yapmak isteyen geliştiriciler için rehberler içerir.

## Aktif Geliştirme Önceliği - 2026-05-24

Phase-17 için yeni özellik geliştirmeden önce PR-4 local stability riski
otorite zincirinde kapatılacaktır. PR-4A evidence, üç ölçüm proxy'sinde ortak
`sample-6` sapmasını kaydetmiştir; PR-4B bounded aynı-kontrat kampanya bu
sapmayı yeniden üretmemiştir. Bu non-reproduction kabul sayılmaz. Sonraki
değişiklikler remote locked-baseline acceptance ve gerektiğinde CI
ortamındaki stage-localization sırasını korumalıdır.

## İçerik

- **Geliştirici Rehberi** - Geliştirme süreçleri ve best practices
- **Kod Standartları** - Coding conventions ve style guide
- **Test Yazma** - Unit test, integration test stratejileri
- **Debugging** - Debug araçları ve teknikleri
- **Performans Optimizasyonu** - Performance tuning rehberi
- **CI/CD Entegrasyonu** - Continuous integration setup

## Geliştirme Felsefesi

- **Anayasal Uyumluluk** - Her kod anayasal kurallara uymalı
- **Test-Driven Development** - Testler önce yazılmalı
- **Deterministic Behavior** - Öngörülebilir sistem davranışı
- **Documentation First** - Kod yazmadan önce dokümantasyon

## Hedef Kitle

- AykenOS geliştiricileri
- Kernel contributors
- Userspace developers
- DevOps engineers

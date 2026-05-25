# Anayasal Sistem Dokümantasyonu

Bu bölüm AykenOS'un en özgün özelliği olan Anayasal Yönetişim Sistemini açıklar.

## Güncel Authority Sınırı - 2026-05-24

Performans doğrulaması artık fail-closed kabul sınırındadır: median PASS,
stability FAIL sonucunu geçersiz kılamaz. PR-4A diagnostic PASS yalnız
mevcut kanıtın ortak `sample-6` sapması olarak sınıflandırıldığını gösterir;
remote locked-baseline veya Phase-17 closure yetkisi oluşturmaz.

## İçerik

- **Anayasal Yönetişim** - Temel felsefe ve yaklaşım
- **Kural Sistemi** - Constitutional rule engine
- **Allow Sistemi** - Geçici istisna mekanizması
- **Waiver Sistemi** - Toplu istisna yönetimi
- **AHS Sistemi** - Architecture Health Score
- **AHTS Sistemi** - Architecture Health Time-Series
- **MARS Sistemi** - Module Architecture Risk Score
- **ARRE Sistemi** - Allow → Refactor Recommendation Engine
- **ARH Sistemi** - Auto-Refactor Hints

## Temel Felsefe

> "İstisna = bilinçli karar"
> "İyi mimari → istisnasız mimaridir"
> "Tek snapshot yalan söyler. Trend asla yalan söylemez."

## Hedef Kitle

- Mimari uzmanları
- Kalite mühendisleri
- Proje yöneticileri
- Anayasal yönetişim ile ilgilenen araştırmacılar

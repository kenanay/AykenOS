# Sistem Mimarisi Dokümantasyonu

Bu bölüm AykenOS'un teknik mimarisini detaylı olarak açıklar.

## Güncel Mimari Sınır - 2026-05-24

- Ring0 mekanizma, Ring3 policy ayrımı korunur.
- Canonical syscall yüzeyi `1000-1011` / 12 syscall'dır.
- Phase-17 stabilization devam eder; PR-4A variance diagnosis yalnız gözlem
  üretir, kernel policy veya closure yetkisi üretmez.
- Güncel durum kaydı: [`../../PROJECT_STATUS_2026_05_24.md`](../../PROJECT_STATUS_2026_05_24.md).

## İçerik

- **Genel Bakış** - Sistem mimarisinin genel görünümü
- **Çekirdek Mimarisi** - Kernel yapısı ve bileşenleri
- **Kullanıcı Alanı** - Userspace mimarisi ve servisleri
- **Güvenlik Modeli** - Ring0/Ring3 güvenlik yaklaşımı
- **Bellek Yönetimi** - Memory management stratejileri
- **Süreçler ve Thread'ler** - Process ve thread yönetimi

## Hedef Kitle

- Sistem mimarları
- Kernel geliştiricileri
- İleri seviye geliştiriciler
- Akademik araştırmacılar

# AykenOS Documentation Website

Bu dizin AykenOS projesi için bağımsız dokümantasyon web sitesini içerir.

## Güncel Durum Kaydı

Projenin 2026-05-24 tarihli stabilization-first durum özeti
[`PROJECT_STATUS_2026_05_24.md`](PROJECT_STATUS_2026_05_24.md) dosyasındadır.
Phase-16 son resmi kapanıştır; Phase-17 aktiftir ve resmi kapanış bekler.
PR-4 local readiness sonucu `FAIL`, PR-4A variance analizi ise yalnız
diagnostic `PASS` durumundadır. PR-4B bounded aynı-kontrat ölçümünde önceki
outlier yeniden üretilmemiştir; bu sonuç acceptance değildir ve remote
locked-baseline otoritesi kurulmamıştır.

## Yapı
- Proje ile tamamen bağımsız
- Statik HTML/CSS/JS yapısı
- Responsive tasarım
- Türkçe ve İngilizce dil desteği

## Geliştirme
Bu dizin proje ana dizininden tamamen ayrıdır ve kendi bağımsız yapısına sahiptir.

Bu senkronizasyonda uygulama dosyaları değiştirilmemiş, yalnız Markdown
dokümantasyon metadata'sı güncellenmiştir. Statik HTML içindeki tarihsel
durum ifadeleri resmi faz/closure otoritesi değildir.

## Kurulum
Statik dosyalar olarak herhangi bir web sunucusunda çalışabilir.

---

**Düzenleyen / Geliştiren / Oluşturan / Mimari Sorumlu:** Kenan AY
**Yetki sınırı:** Dokümantasyon metadata'sı; runtime veya closure otoritesi değildir.

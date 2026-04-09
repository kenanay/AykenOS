# Phase-14 Kapanış Kriterleri Belgesi

**Belge Türü**: Normatif  
**Faz**: Phase-14 — Distributed Observability Hardening  
**Durum**: KAPANIŞ ADAYI — Uzak ci-freeze onayı bekleniyor  
**Tek Doğru Kaynak**: `reports/phase14_official_closure_candidate/closure_index.json`

---

## 1. Kapanış Kriterleri Kontrol Listesi

Aşağıdaki 11 kriter Phase-14'ün resmi olarak kapalı sayılması için **tamamının** karşılanması zorunludur. Kısmi kapanış yoktur.

| # | Kriter | Durum |
|---|--------|-------|
| 1 | Truth surface senkronizasyonu: tracker, README, DOCUMENTATION_INDEX tüm WS 3.1–3.5 MERGED gösteriyor | TAMAMLANDI |
| 2 | İş Akışı Doğrulama Matrisi: `PHASE14_WORKSTREAM_VALIDATION_MATRIX.md` oluşturuldu | TAMAMLANDI |
| 3 | Gözlemlenebilirlik Sınırı Kanıtı: `PHASE14_OBSERVABILITY_BOUNDARY_PROOF.md` oluşturuldu | TAMAMLANDI |
| 4 | pre-ci disiplin: ABI + Boundary + Hygiene + Constitutional + Determinism — tümü PASS | TAMAMLANDI |
| 5 | Kapanış adayı paketi: `reports/phase14_official_closure_candidate/` oluşturuldu | BEKLEMEDE |
| 6 | Özellik tabanlı testler: Property 1 (yasak alan reddi) PASS | BEKLEMEDE |
| 7 | `ARCHITECTURE_FREEZE.md` Phase-14 kapanış girişi eklendi | BEKLEMEDE |
| 8 | Uzak `ci-freeze` PASS onayı alındı (GitHub Actions) | BEKLEMEDE |
| 9 | `closure_decision_record.json` gerçek değerlerle dolduruldu | BEKLEMEDE |
| 10 | `phase14-official-closure-confirmed` etiketi uygulandı | BEKLEMEDE |
| 11 | `CURRENT_PHASE=15` güncellendi | BEKLEMEDE |

> **Not**: Kriter 8 (uzak ci-freeze PASS) kapanış otoritesinin tek kaynağıdır.
> Yerel `make ci-freeze-local` PASS sonucu bu kriteri karşılamaz.

---

## 2. Biçimsel Kapanış Tanımı

```
C = kapanış kriterleri kümesi (yukarıdaki 11 kriter)
E = ci-freeze kanıtı (uzak GitHub Actions PASS)
S = truth surface durumu (closure_index.json ile tutarlı)

Phase-14 KAPALI iff:
  ∀c ∈ C: c = TAMAMLANDI
  ∧ E = PASS (uzak GitHub Actions, HEAD SHA bağlı)
  ∧ S = tutarlı (closure_index.json kesin kaynak)
```

Bu tanım normatiftir. Yukarıdaki koşulların tamamı sağlanmadan Phase-14 kapalı sayılamaz.

---

## 3. Faz Geçiş Kuralı

```
CI PASS → `phase14-official-closure-confirmed` etiketi → CURRENT_PHASE=15
```

Kurallar:

- `CURRENT_PHASE=15` güncellemesi yalnızca `phase14-official-closure-confirmed` etiketi uygulandıktan sonra yapılabilir.
- `phase14-official-closure-confirmed` etiketi yalnızca uzak GitHub Actions `ci-freeze` PASS sonucu alındıktan sonra uygulanabilir.
- Yerel `ci-freeze-local` PASS sonucu etiket uygulaması için yeterli değildir.
- Etiket uygulanmadan `CURRENT_PHASE` güncellenemez.
- CI PASS olmadan etiket uygulanamaz.

Bu kural ihlal edilirse kapanış geçersiz sayılır ve yeni bir kapanış döngüsü başlatılması gerekir.

---

## 4. Yürütme Sorumluluk Hiyerarşisi

Kapanış süreci üç sorumluluk katmanına bölünmüştür. Hiçbir aktör kapanışı tek başına tamamlayamaz.

### Geliştirici (Kriter 1–7)

- Truth surface senkronizasyonunu sağlar (tracker, README, DOCUMENTATION_INDEX)
- İş Akışı Doğrulama Matrisini oluşturur
- Gözlemlenebilirlik Sınırı Kanıtı belgesini oluşturur
- pre-ci disiplinini (ABI + Boundary + Hygiene + Constitutional + Determinism) doğrular
- Kapanış adayı paketini (`reports/phase14_official_closure_candidate/`) hazırlar
- Özellik tabanlı testlerin (Property 1–7) geçtiğini doğrular
- `ARCHITECTURE_FREEZE.md` dosyasına Phase-14 kapanış girişini ekler

### CI Sistemi — GitHub Actions (Kriter 8)

- Uzak `ci-freeze` iş akışını çalıştırır
- Tüm Phase-14 kapılarının PASS durumunu doğrular
- Yetkili PASS/FAIL kararını verir
- HEAD SHA'yı PASS sonucuna bağlar
- Kapanış otoritesinin tek kaynağıdır

### Depo Otoritesi — Geliştirici + CI Onayı (Kriter 9–11)

- `closure_decision_record.json` dosyasını gerçek değerlerle doldurur (HEAD SHA, CI run ID, UTC zaman damgası)
- `phase14-official-closure-confirmed` etiketini kapanış HEAD SHA'sına uygular
- `docs/roadmap/CURRENT_PHASE` dosyasını `CURRENT_PHASE=15` olarak günceller

---

## 5. Kapanış Kanıtı Yer Tutucuları

Aşağıdaki alanlar uzak `ci-freeze` PASS sonrası doldurulacaktır:

```
head_sha: <kapanış HEAD SHA — uzak ci-freeze PASS sonrası doldurulacak>
ci_freeze_run_id: <GitHub Actions çalışma kimliği — uzak ci-freeze PASS sonrası doldurulacak>
closure_timestamp_utc: <ISO 8601 UTC — uzak ci-freeze PASS sonrası doldurulacak>
```

Bu alanlar `reports/phase14_official_closure_candidate/closure_decision_record.json` dosyasında saklanır.

---

## 6. Tek Doğru Kaynak İşaretçisi

```
reports/phase14_official_closure_candidate/closure_index.json
```

Bu dosya Phase-14 kapanış gerçeğinin kesin kaynağıdır. Tracker, README, raporlar ve bu belge dahil tüm türetilmiş belgeler `closure_index.json` ile çeliştiğinde `closure_index.json` geçerli kabul edilir.

---

## 7. Başarısızlık Modları

Aşağıdaki koşulların herhangi biri kapanışı engeller. Kısmi kapanış yoktur.

| Koşul | Sonuç |
|-------|-------|
| Herhangi bir `ci-freeze` kapısı başarısız | Kapanış reddedilir |
| Diagnostics yanıtında yasak alan tespit edildi | Kapanış reddedilir |
| Truth yüzeyleri birbiriyle çelişiyor | Kapanış reddedilir |
| Doğrulama matrisinde PASS kanıtı eksik | Kapanış reddedilir |
| Endpoint/şema uyumsuzluğu tespit edildi | Kapanış reddedilir |
| Uzak `ci-freeze` onayı alınmadan kapanış iddiası | Kapanış geçersiz |
| Hygiene kapısı başarısız (dirty tracked dosyalar) | Kapanış reddedilir |
| SHA-256 özet uyumsuzluğu | Kapanış reddedilir |

---

## 8. Kanıt Öncelik Sırası

Kapanış doğrulaması aşağıdaki katı öncelik sırasını izler:

1. CI kanıtı (`ci-freeze` uzak çalışma kimliği ve PASS sonucu)
2. Çalışma zamanı test sonuçları (proofd + obs-cli test sayıları)
3. Sözleşme uyumluluğu (endpoint şema doğrulaması, yasak alan taraması)
4. Dokümantasyon (tracker, README, mimari harita)

Dokümantasyon tek başına kapanış için yeterli değildir. CI kanıtı olmadan kapanış iddiası geçersizdir.

---

## 9. Kapanış Sonrası Değişmezler

Phase-14 resmi olarak kapandıktan sonra aşağıdaki değişmezler kalıcı olarak geçerlidir:

- Observability sözleşmeleri değiştirilemez; herhangi bir değişiklik yeni bir faz gerektirir.
- Diagnostics yüzeyleri `non_authoritative` olarak kalır; bu sınıflandırma geriye dönük değiştirilemez.
- `service != authority`, `diagnostics != decision`, `parity != consensus` değişmezleri Phase-15 ve sonrasında da geçerlidir.
- `phase14-official-closure-confirmed` etiketi değiştirilemez; yeni bir kapanış iddiası yeni bir etiket gerektirir.
- Geriye dönük sıralama/puanlama semantiği eklenemez.

---

## 10. İzlenebilirlik

| Gereksinim | Bu Belgedeki Karşılık |
|------------|----------------------|
| 5.1 | Bölüm 1 — Kapanış kriterleri kontrol listesi |
| 5.2 | Bölüm 2 — Biçimsel kapanış tanımı |
| 5.3 | Bölüm 1 — Her kriterin TAMAMLANDI/BEKLEMEDE durumu |
| 5.4 | Bölüm 5 — HEAD SHA ve CI run ID yer tutucuları |
| 5.5 | Bölüm 7 — Başarısızlık modları |
| 14.1 | Bölüm 4 — Yürütme sorumluluk hiyerarşisi |
| 14.2 | Bölüm 3 — CI PASS olmadan etiket uygulanamaz |
| 14.3 | Bölüm 3 — Etiket uygulanmadan CURRENT_PHASE güncellenemez |

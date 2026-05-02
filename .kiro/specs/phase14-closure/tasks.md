# Uygulama Planı: Phase-14 Resmi Kapanışı

## Genel Bakış

Bu plan, AykenOS Phase-14 (Distributed Observability Hardening) fazının resmi
kapanış sürecini yürütmek için gereken tüm kodlama görevlerini kapsar. Görevler
sırayla uygulanmalıdır; her adım bir öncekinin üzerine inşa edilir. Kapanış
otoritesi yalnızca uzak GitHub Actions `ci-freeze` PASS sonucudur.

## Yürütme Modu

Bu plan sırayla yürütülür:

- Adımlar yeniden sıralanamaz
- Paralel yürütme yasaktır
- Her adım bir sonrakine geçmeden önce doğrulama gerektirir

## Durdurma Koşulları

Aşağıdaki koşulların herhangi birinde yürütme **derhal durur**:

- Herhangi bir pre-ci kapısı başarısız olursa
- Herhangi bir ci-freeze kapısı başarısız olursa
- Herhangi bir zorunlu artefakt üretilemezse
- Herhangi bir SHA-256 uyumsuzluğu tespit edilirse

Başarısızlık sonrası hiçbir adım çalıştırılmaz.

## Görevler

- [x] 1. İş Akışı Doğrulama Matrisi belgesi oluştur
  - `docs/specs/phase14-distributed-observability/PHASE14_WORKSTREAM_VALIDATION_MATRIX.md` dosyasını oluştur
  - WS 3.1–3.5 için tam tablo: sözleşme belgesi, endpoint/yüzey, şema kapsamı, negatif test, PASS kanıtı, birleştirme durumu, bağımlılık sütunları
  - WS 3.3 → WS 3.4 → WS 3.5 bağımlılık zincirini belgele
  - Her iş akışı için CI çalışma kimliklerini (ci-freeze#23989067554, ci-freeze#23999026616) kaydet
  - _Gereksinimler: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.9_

- [x] 2. Gözlemlenebilirlik Sınırı Kanıtı belgesi oluştur
  - `docs/specs/phase14-distributed-observability/PHASE14_OBSERVABILITY_BOUNDARY_PROOF.md` dosyasını oluştur
  - Beş temel değişmezi belgele: `service != authority`, `diagnostics != decision`, `parity != consensus`, `trust does not affect verdict`, `observability does not imply scheduling`
  - Her değişmez için kanıt ve uygulama noktasını kaydet
  - `SummaryEpistemicBoundary` yapısını ve `produces_truth/decision/ranking = false` beyanını belgele
  - `FORBIDDEN_OBSERVABILITY_FIELDS` listesini (34 giriş) ve çalışma zamanı uygulama mekanizmasını (`500 forbidden_observability_field_exposed`) belgele
  - `ci-gate-proofd-observability-boundary` ve `ci-gate-observability-routing-separation` PASS kanıtlarını kaydet
  - _Gereksinimler: 3.1, 3.2, 3.3, 3.4, 3.5_

- [x] 3. Kapanış Kriterleri belgesi oluştur
  - `docs/specs/phase14-distributed-observability/PHASE14_CLOSURE_CRITERIA.md` dosyasını oluştur
  - Her kriterin durumunu (TAMAMLANDI / BEKLEMEDE) içeren açık kontrol listesi oluştur
  - Biçimsel kapanış tanımını ekle: `Phase-14 KAPALI iff tüm kriterler karşılandı + uzak ci-freeze onayı alındı`
  - Faz geçiş kuralını belgele: CI PASS → etiket → `CURRENT_PHASE=15`
  - Kapanış otoritesi hiyerarşisini (geliştirici / CI sistemi / depo otoritesi) kaydet
  - HEAD SHA ve CI çalışma kimliği için yer tutucu alanları ekle
  - _Gereksinimler: 5.1, 5.2, 5.3, 5.4, 14.1, 14.2, 14.3_

- [ ] 4. Kapanış adayı paketi oluştur
  - [x] 4.1 `reports/phase14_official_closure_candidate/README.md` dosyasını oluştur
    - Kapanış iddiasını, kalan yönetim adımlarını ve `closure_index.json` tek doğru kaynak işaretçisini belgele
    - _Gereksinimler: 6.1, 16.1, 16.2_

  - [x] 4.2 `reports/phase14_official_closure_candidate/closure_manifest.json` dosyasını oluştur
    - İş akışı matrisi anlık görüntüsünü JSON formatında kaydet (WS 3.1–3.5, her biri için sözleşme belgesi, endpoint, şema kapsamı, negatif test, PASS kanıtı, birleştirme durumu)
    - _Gereksinimler: 6.1, 6.2, 10.5_

  - [x] 4.3 `reports/phase14_official_closure_candidate/closure_manifest.sha256` dosyasını oluştur
    - `closure_manifest.json` dosyasının SHA-256 özetini kaydet
    - _Gereksinimler: 13.3_

  - [x] 4.4 `reports/phase14_official_closure_candidate/evidence_index.json` dosyasını oluştur
    - CI çalışma referanslarını kaydet: `ci-freeze#23989067554` ve `ci-freeze#23999026616`
    - Her çalışma için: çalışma kimliği, sonuç (PASS), kapsanan kapılar, tarih
    - _Gereksinimler: 4.2, 4.3, 6.1_

  - [x] 4.5 `reports/phase14_official_closure_candidate/evidence_index.sha256` dosyasını oluştur
    - `evidence_index.json` dosyasının SHA-256 özetini kaydet
    - _Gereksinimler: 13.3_

  - [x] 4.6 `reports/phase14_official_closure_candidate/closure_decision_record.json` dosyasını oluştur
    - Zorunlu alanları içeren şablon oluştur: `phase`, `closure_state`, `head_sha` (yer tutucu), `ci_freeze_run_id` (yer tutucu), `closure_timestamp_utc` (yer tutucu), `closure_verdict` (yer tutucu), `reproducibility_verified: true`, `closure_candidate_package`, `workstreams_closed`, `next_phase`
    - _Gereksinimler: 10.1, 10.2, 10.3, 11.5, 15.1_

  - [x] 4.7 `reports/phase14_official_closure_candidate/closure_index.json` dosyasını oluştur
    - Tüm artefakt referanslarını ve SHA-256 özetlerini içeren ana indeks dosyasını oluştur
    - `schema`, `closure_type`, `phase`, `tag`, `authority`, `closure_state`, `artifacts` (her artefakt için `path` + `sha256`), `workstreams_closed`, `current_phase_file`, `current_phase_value` alanlarını ekle
    - `remote_ci_confirmation` bloğunu yer tutucu değerlerle ekle
    - _Gereksinimler: 6.2, 6.3, 13.2, 13.3, 16.1, 16.4_

  - [x] 4.8 Kapanış paketi bütünlüğünü doğrula
    - `closure_index.json` içindeki tüm `path` değerlerinin dosya sisteminde mevcut olduğunu doğrula
    - Her artefakt için SHA-256 özetini yeniden hesapla ve `closure_index.json` içindeki değerle karşılaştır
    - Eksik veya uyumsuz artefakt varsa süreci durdur ve hangi artefaktın başarısız olduğunu raporla
    - Doğrulama başarılıysa `closure_index.json` içine `"integrity_verified": true` alanını ekle
    - _Gereksinimler: 6.3, 13.3, 13.4_

- [x] 5. Özellik tabanlı testleri `userspace/proofd/src/lib.rs` dosyasına ekle
  - [x] 5.1 Özellik 1: Yasak alan reddi özellik testini yaz
    - `FORBIDDEN_OBSERVABILITY_FIELDS` listesindeki herhangi bir alan içeren yanıtın `500 forbidden_observability_field_exposed` ile reddedildiğini doğrula
    - `// Feature: phase14-closure, Property 1: Forbidden field rejection` etiketiyle işaretle
    - `forbidden_observability_field_tokens()` ile tüm yasak alanlar üzerinde test
    - _Gereksinimler: 3.4, 3.6_

  - [x] 5.2 Özellik 2: Epistemic sınır değişmezi özellik testini yaz
    - Herhangi bir summary yanıtında `produces_truth`, `produces_decision`, `produces_ranking` alanlarının tamamının `false` olduğunu doğrula
    - `// Feature: phase14-closure, Property 2: Epistemic boundary invariant` etiketiyle işaretle
    - _Gereksinimler: 3.2_

  - [x] 5.3 Özellik 3: Kapanış paketi SHA-256 bütünlüğü özellik testini yaz
    - `closure_index.json` içindeki SHA-256 özetlerinin artefakt dosyalarıyla eşleştiğini doğrula
    - `// Feature: phase14-closure, Property 3: Closure package SHA-256 integrity` etiketiyle işaretle
    - _Gereksinimler: 6.2, 6.3, 13.3, 13.4_

  - [x] 5.4 Özellik 4: ci-freeze deterministik yeniden üretilebilirlik özellik testini yaz
    - Aynı HEAD SHA ve sabit ortam koşullarıyla iki `ci-freeze` simülasyonunun özdeş sonuç ürettiğini doğrula
    - `// Feature: phase14-closure, Property 4: ci-freeze deterministic reproducibility` etiketiyle işaretle
    - _Gereksinimler: 11.1, 11.3, 11.4_

  - [x] 5.5 Özellik 5: Kapanış karar kaydı zorunlu alanlar özellik testini yaz
    - Geçerli bir kapanış karar kaydında zorunlu alanların tamamının mevcut olduğunu doğrula
    - `// Feature: phase14-closure, Property 5: Closure decision record required fields` etiketiyle işaretle
    - _Gereksinimler: 10.1, 10.4_

  - [x] 5.6 Özellik 6: Truth surface tutarsızlık engeli özellik testini yaz
    - Birbiriyle çelişen truth yüzeyleri için tutarsızlığın tespit edildiğini doğrula
    - `// Feature: phase14-closure, Property 6: Truth surface conflict blocks closure` etiketiyle işaretle
    - _Gereksinimler: 1.4, 2.8_

  - [x] 5.7 Özellik 7: Kapanış paketi eksik kanıt engeli özellik testini yaz
    - Herhangi bir iş akışı için PASS kanıtı eksik olduğunda kapanışın engellendiğini doğrula
    - `// Feature: phase14-closure, Property 7: Missing evidence blocks closure` etiketiyle işaretle
    - _Gereksinimler: 2.8, 6.5_

- [x] 6. Kontrol noktası — Tüm testlerin geçtiğini doğrula
  - 7/7 phase14_closure_property_tests PASS (2026-04-08)

- [x] 7. `ARCHITECTURE_FREEZE.md` dosyasına Phase-14 kapanış girişi ekle
  - Bölüm 8 (`Freeze Entry Criteria`) altındaki durum listesine Phase-14 kapanış satırını ekle (Phase-13 formatıyla tutarlı)
  - Örnek format: `✅ Phase-14 distributed observability hardening: OFFICIALLY CLOSED (CI run <id>, PR #<n>, tag: phase14-official-closure-confirmed)`
  - Bölüm 16 (`Document Control`) revizyon geçmişine yeni satır ekle (versiyon 2.0, tarih, değişiklik özeti)
  - _Gereksinimler: 9.4, 12.5_

- [x] 8. Faz geçiş yürütme (uzak ci-freeze PASS sonrası)
  - [x] 8.1 `docs/roadmap/CURRENT_PHASE` dosyasını `CURRENT_PHASE=15` olarak güncelle
    - Yalnızca uzak GitHub Actions `ci-freeze` PASS onayı alındıktan sonra çalıştır
    - _Gereksinimler: 7.1, 7.2_

  - [x] 8.2 `reports/phase14_official_closure_candidate/closure_decision_record.json` dosyasını gerçek değerlerle doldur
    - `head_sha` alanını kapanış HEAD SHA değeriyle güncelle
    - `ci_freeze_run_id` alanını GitHub Actions çalışma kimliğiyle güncelle
    - `closure_timestamp_utc` alanını ISO 8601 UTC zaman damgasıyla güncelle
    - `closure_verdict` alanını `PASS` olarak güncelle
    - `closure_index.json` içindeki `closure_decision_record` SHA-256 özetini yeniden hesapla ve güncelle
    - _Gereksinimler: 10.1, 10.2, 15.1, 15.2_

  - [x] 8.3 `PHASE14_DEVELOPMENT_TRACKER.md` dosyasına kapanış kaydı ekle
    - Kapanış tarihini, HEAD SHA'yı, CI çalışma kimliğini ve `phase14-official-closure-confirmed` etiketini kaydet
    - _Gereksinimler: 7.5_

  - [x] 8.4 CI çalışma doğrulaması yap
    - `ci_freeze_run_id` değerinin gerçekten PASS sonucu ürettiğini GitHub Actions loglarından doğrula
    - CI çalışmasının HEAD SHA ile eşleştiğini doğrula
    - Uyumsuzluk varsa `closure_decision_record.json` güncellemesini durdur ve uyumsuzluğu raporla
    - _Gereksinimler: 4.2, 4.3, 14.2_

  - [x] 8.5 Kapanış sonrası immutability lock uygula
    - `ARCHITECTURE_FREEZE.md` dosyasına observability sözleşme dosyalarının değiştirilemez olduğunu belgele
    - `phase14-official-closure-confirmed` etiketinin korumalı olduğunu repo policy'de kaydet (force-push yasağı)
    - `PHASE14_DEVELOPMENT_TRACKER.md` dosyasına `closure_state: IMMUTABLE` kaydını ekle
    - _Gereksinimler: 8.1, 8.4, 14.3_

- [x] 9. Son kontrol noktası — Tüm testlerin geçtiğini doğrula
  - Tüm testlerin geçtiğini doğrula, sorular varsa kullanıcıya sor.

## Notlar

- `*` ile işaretli görevler isteğe bağlıdır; daha hızlı MVP için atlanabilir
- Her görev izlenebilirlik için belirli gereksinimlere referans verir
- Kontrol noktaları artımlı doğrulama sağlar
- Özellik testleri evrensel doğruluk özelliklerini doğrular
- Görev 8 yalnızca uzak `ci-freeze` PASS onayı alındıktan sonra yürütülür; yerel çalışmalar kapanış otoritesi vermez
- `closure_index.json` tüm türetilmiş belgeler üzerinde öncelikli kapanış gerçeği kaynağıdır
- Phase-13 referans paketi: `reports/phase13_official_closure_candidate/`
- **Durdurma koşulu**: Herhangi bir adımda başarısızlık → sonraki adımlar çalıştırılmaz
- **Task 4.8 kapanışın gerçek gate'idir**: SHA-256 doğrulaması geçmeden Task 5'e geçilmez
- **Task 8.4 CI doğrulaması zorunludur**: run ID + HEAD SHA eşleşmesi olmadan 8.5 çalıştırılmaz

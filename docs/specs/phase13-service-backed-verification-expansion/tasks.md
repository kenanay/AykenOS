# Uygulama Görevleri

## Görevler

- [ ] 1. `run_id` Opsiyonel Hale Getirme
  - [ ] 1.1 `VerifyBundleRequestBody.run_id` alanını `String`'den `Option<String>`'e değiştir
  - [ ] 1.2 `validate_verify_bundle_request()` içinde `run_id: None` durumunda UUID üret; format
        kontrolünü yalnızca `Some` durumunda uygula
  - [ ] 1.3 `verify_bundle_request()` içinde `run_id` `None` ise `uuid::Uuid::new_v4().to_string()`
        ile üret ve `request.run_id`'ye ata
  - [ ] 1.4 Mevcut `verify_bundle_endpoint_rejects_run_id_reuse_with_different_request_fingerprint`
        testinin hâlâ geçtiğini doğrula
  - Referans: Gereksinim 1.3

- [ ] 2. Verify Response — Normatif Alan Uyumu
  - [ ] 2.1 `VerifyBundleResponseBody`'ye `request_fingerprint: String` alanı ekle
  - [ ] 2.2 `verify_bundle_request()` içinde `request_fingerprint` alanını yanıta yaz
  - [ ] 2.3 Yanıt gövdesinin Phase13_Forbidden_Fields kümesindeki alanları içermediğini doğrulayan
        bir test ekle
  - Referans: Gereksinim 1.1, 1.11

- [ ] 3. `NO_REUSABLE_EVENTS` Structured JSON
  - [ ] 3.1 `build_runtime_trust_reuse_flow_source_document()` içinde `NO_REUSABLE_EVENTS` durumunu
        tespit eden kodu bul
  - [ ] 3.2 String sentinel yerine `{"status": "NO_REUSABLE_EVENTS"}` JSON nesnesi üretecek biçimde
        güncelle
  - [ ] 3.3 `verify_bundle_endpoint_keeps_native_trust_reuse_when_all_events_are_rejected` testini
        yeni format için güncelle
  - Referans: Gereksinim 1.8, 6.4

- [ ] 4. Artifact Fetch — 403 / 404 Ayrımı ve Path Normalization
  - [ ] 4.1 `resolve_run_artifact_path()` fonksiyonunu iki aşamalı kontrol yapacak biçimde
        güncelle:
        1. Yol Allowed_Artifact_Set içinde mi? → değilse `ServiceError::Forbidden("artifact_path_not_allowed")`
        2. Dosya diskte var mı? → yoksa `ServiceError::NotFound("artifact_not_found")`
  - [ ] 4.2 `ServiceError` enum'una `Forbidden(&'static str)` varyantı ekle; `error_response()`
        içinde HTTP 403 olarak map'le
  - [ ] 4.3 `parse_run_artifact_path()` içinde `is_safe_path_segment()` kontrolünün `..` ve `.`
        segmentlerini reddettiğini doğrula; reddetmiyorsa güncelle. Bu path normalization'ın
        ilk katmanıdır: segment güvenliği sağlandıktan sonra `resolve_run_artifact_path()`
        Allowed_Artifact_Set kontrolü yapar.
  - [ ] 4.4 `run_artifact_endpoint_rejects_invalid_relative_path` testini 403 beklentisiyle
        güncelle
  - [ ] 4.5 Allowed_Artifact_Set içinde olan ama diskte bulunmayan bir yol için 404 döndüğünü
        doğrulayan test ekle
  - Referans: Gereksinim 3.6, 3.7, 3.11

- [ ] 5. Artifact Discovery — `run_dir_not_found` Propagation
  - [ ] 5.1 `build_run_artifact_index()` içinde `list_run_artifact_descriptors()` çağrısından önce
        `run_dir.is_dir()` kontrolü ekle; yoksa `ServiceError::NotFound("run_dir_not_found")`
        döndür
  - [ ] 5.2 Run dizini yokken `GET /diagnostics/runs/{run_id}/artifacts` çağrısının 404
        `run_dir_not_found` döndürdüğünü doğrulayan test ekle
  - Referans: Gereksinim 3.2

- [ ] 6. Federation Diagnostics — Spec Uyumu ve Projeksiyon Katmanı
  - [ ] 6.1 `build_run_federation_diagnostics()` içinde run dizini yoksa önce
        `run_dir_not_found` döndür; ardından ledger yoksa `artifact_not_found` döndür
  - [ ] 6.2 `FederationDiagnosticsProjection` struct'ını tanımla:
        `run_id`, `verifier_count`, `observed_verifiers`, `authority_chain_distribution`,
        `execution_cluster_distribution`, `missing_execution_cluster_entry_count`
  - [ ] 6.3 `SpecFederationVerifierEntry` struct'ını tanımla:
        `verifier_id: String`, `lineage_id: Option<String>`
  - [ ] 6.4 `SpecAuthorityChainEntry` struct'ını tanımla:
        `authority_chain_id: String`, `entry_count: usize`
  - [ ] 6.5 `SpecExecutionClusterEntry` struct'ını tanımla:
        `cluster_id: String`, `entry_count: usize`
  - [ ] 6.6 `build_run_federation_diagnostics()` içinde iç
        `FederationDiagnosticsResponseBody`'yi hesapla, ardından
        `FederationDiagnosticsProjection`'a dönüştür; projeksiyon serialize edilsin
  - [ ] 6.7 `observed_verifiers` dizisini `verifier_id`'ye göre leksikografik sırala
  - [ ] 6.8 Yanıt gövdesinin Phase13_Forbidden_Fields kümesindeki alanları içermediğini
        doğrulayan serialize guard testi ekle (bkz. Görev 10)
  - [ ] 6.9 `run_scoped_federation_endpoint_summarizes_diversity_ledger` testini yeni alan
        adları için güncelle
  - Referans: Gereksinim 4, 5

- [ ] 7. Property-Based Testler
  - [ ] 7.1 P1 — Run_Id Fingerprint Çakışma Koruması: aynı `run_id`, farklı fingerprint → 409
  - [ ] 7.2 P4 — Artifact Discovery Salt Okunur Değişmezi: GET öncesi/sonrası run dizini özdeş
  - [ ] 7.3 P5 — Federation Forbidden Field Değişmezi: yanıt Phase13_Forbidden_Fields ∩ ∅
  - [ ] 7.4 P6 — Federation Sıralama Değişmezi: tüm dağılım dizileri leksikografik sırada
  - [ ] 7.5 P7 — Artifact Fetch Passthrough: yanıt gövdesi = diskteki baytlar
  - [ ] 7.6 P8 — Method Not Allowed: tüm diagnostics path'lerine POST → 405
  - [ ] 7.7 P9 — Artifact Path Normalization: `..` veya `.` segment içeren yollar → HTTP 403;
        Allowed_Artifact_Set dışındaki normalize edilmiş yollar → HTTP 403
  - [ ] 7.8 `cargo test --manifest-path userspace/proofd/Cargo.toml` ile tüm testlerin geçtiğini
        doğrula
  - Referans: Gereksinim 9.2, 9.3, 9.4

- [ ] 8. Atomic Manifest Creation
  - [ ] 8.1 `verify_bundle_request()` içinde `proofd_run_manifest.json` yazımını
        `OpenOptions::new().write(true).create_new(true)` ile atomik hale getir
        (`O_CREAT | O_EXCL` semantiği)
  - [ ] 8.2 `AlreadyExists` hatası durumunda mevcut manifest'i oku, fingerprint karşılaştır;
        çakışma varsa HTTP 409 `run_id_fingerprint_conflict` döndür
  - [ ] 8.3 Eşzamanlı iki isteğin aynı `run_id` için yarıştığı senaryoyu simüle eden test ekle:
        yalnızca birinin manifest yazmasına izin verildiğini doğrula
  - Referans: Gereksinim 1.12

- [ ] 9. Spec Projection Layer — Federation Diagnostics
  - [ ] 9.1 `FederationDiagnosticsProjection` struct'ının iç
        `FederationDiagnosticsResponseBody`'den bağımsız olduğunu doğrula: iç struct'a yeni
        alan eklendiğinde projeksiyon yanıtı değişmemeli
  - [ ] 9.2 `build_run_federation_diagnostics()` dönüş tipini `FederationDiagnosticsProjection`
        olarak güncelle; serialize edilen yanıtın yalnızca projeksiyon alanlarını içerdiğini
        doğrulayan test ekle
  - Referans: Gereksinim 4, 5; Design §4 Spec Projection Layer

- [ ] 10. Forbidden Fields Serialize-Level Guard
  - [ ] 10.1 `PHASE13_FORBIDDEN_FIELDS` sabit dizisini `lib.rs` içinde tanımla
  - [ ] 10.2 `FederationDiagnosticsProjection` için serialize guard testi ekle:
        `serde_json::to_value(&projection)` çıktısında forbidden field olmadığını doğrula
  - [ ] 10.3 `VerifyBundleResponseBody` için serialize guard testi ekle:
        yanıt gövdesinde forbidden field olmadığını doğrula
  - [ ] 10.4 Tüm serialize guard testlerinin `PHASE13_FORBIDDEN_FIELDS` sabitine referans
        verdiğini doğrula (liste kopyalanmamalı)
  - Referans: Gereksinim 8; Design §6 Forbidden Fields Compile-Time Guard

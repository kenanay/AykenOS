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

- [ ] 4. Artifact Fetch — 403 / 404 Ayrımı
  - [ ] 4.1 `resolve_run_artifact_path()` fonksiyonunu iki aşamalı kontrol yapacak biçimde
        güncelle:
        1. Yol Allowed_Artifact_Set içinde mi? → değilse `ServiceError::Forbidden("artifact_path_not_allowed")`
        2. Dosya diskte var mı? → yoksa `ServiceError::NotFound("artifact_not_found")`
  - [ ] 4.2 `ServiceError` enum'una `Forbidden(&'static str)` varyantı ekle; `error_response()`
        içinde HTTP 403 olarak map'le
  - [ ] 4.3 `run_artifact_endpoint_rejects_invalid_relative_path` testini 403 beklentisiyle
        güncelle
  - [ ] 4.4 Allowed_Artifact_Set içinde olan ama diskte bulunmayan bir yol için 404 döndüğünü
        doğrulayan test ekle
  - Referans: Gereksinim 3.6, 3.7

- [ ] 5. Artifact Discovery — `run_dir_not_found` Propagation
  - [ ] 5.1 `build_run_artifact_index()` içinde `list_run_artifact_descriptors()` çağrısından önce
        `run_dir.is_dir()` kontrolü ekle; yoksa `ServiceError::NotFound("run_dir_not_found")`
        döndür
  - [ ] 5.2 Run dizini yokken `GET /diagnostics/runs/{run_id}/artifacts` çağrısının 404
        `run_dir_not_found` döndürdüğünü doğrulayan test ekle
  - Referans: Gereksinim 3.2

- [ ] 6. Federation Diagnostics — Spec Uyumu
  - [ ] 6.1 `build_run_federation_diagnostics()` içinde run dizini yoksa önce
        `run_dir_not_found` döndür; ardından ledger yoksa `artifact_not_found` döndür
  - [ ] 6.2 `FederationDiagnosticsResponseBody`'ye spec uyumlu alanlar ekle:
        - `verifier_count: usize` (= `unique_verifier_count`)
        - `observed_verifiers: Vec<SpecFederationVerifierEntry>` (`verifier_id` + opsiyonel
          `lineage_id`)
  - [ ] 6.3 `SpecFederationVerifierEntry` struct'ını tanımla:
        `verifier_id: String`, `lineage_id: Option<String>`
  - [ ] 6.4 `observed_verifiers` dizisini `verifier_id`'ye göre leksikografik sırala
  - [ ] 6.5 `authority_chain_distribution` ve `execution_cluster_distribution` içindeki `id`
        alanını sırasıyla `authority_chain_id` ve `cluster_id` olarak serialize et
        (`#[serde(rename)]` veya ayrı projeksiyon struct'ı)
  - [ ] 6.6 Yanıt gövdesinin Phase13_Forbidden_Fields kümesindeki alanları içermediğini doğrulayan
        test ekle
  - [ ] 6.7 `run_scoped_federation_endpoint_summarizes_diversity_ledger` testini yeni alan adları
        için güncelle
  - Referans: Gereksinim 4, 5

- [ ] 7. Property-Based Testler
  - [ ] 7.1 P1 — Run_Id Fingerprint Çakışma Koruması: aynı `run_id`, farklı fingerprint → 409
  - [ ] 7.2 P4 — Artifact Discovery Salt Okunur Değişmezi: GET öncesi/sonrası run dizini özdeş
  - [ ] 7.3 P5 — Federation Forbidden Field Değişmezi: yanıt Phase13_Forbidden_Fields ∩ ∅
  - [ ] 7.4 P6 — Federation Sıralama Değişmezi: tüm dağılım dizileri leksikografik sırada
  - [ ] 7.5 P7 — Artifact Fetch Passthrough: yanıt gövdesi = diskteki baytlar
  - [ ] 7.6 P8 — Method Not Allowed: tüm diagnostics path'lerine POST → 405
  - [ ] 7.7 `cargo test --manifest-path userspace/proofd/Cargo.toml` ile tüm testlerin geçtiğini
        doğrula
  - Referans: Gereksinim 9.2, 9.3, 9.4

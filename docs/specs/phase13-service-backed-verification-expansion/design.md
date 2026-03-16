# Tasarım Belgesi

## Genel Bakış

Bu belge, `userspace/proofd` crate'i içindeki Phase-13 §4.1 Service-Backed Verification Expansion
genişlemesinin teknik tasarımını tanımlar.

Temel mimari kural değişmez:

```
proofd = verification execution service + diagnostics service surface
proofd != authority surface
```

Tüm değişiklikler mevcut `proofd` crate'i içinde kalır; yeni crate bağımlılığı eklenmez.
Crate içinde modül dosyası eklenebilir.

---

## Mimari

### Bileşen Sınırları

```
POST /verify/bundle
    │
    ├── parse_verify_bundle_request()         — alan ayrıştırma
    ├── validate_verify_bundle_request()      — zorunlu alan + bağımlılık doğrulama
    ├── verify_existing_run_fingerprint()     — run_id çakışma koruması (409)
    ├── verify_bundle_request()               — doğrulama yürütme + artifact yazma
    │       ├── write_verification_context_package()  — 4 context artifact
    │       ├── diversity binding akışı               — replay_boundary_flow_source.json
    │       └── trust reuse çözümleme                 — bundle-native öncelik, fallback
    │
GET /diagnostics/runs/{run_id}/artifacts
    │
    └── build_run_artifact_index()            — kanonik yol listesi + content-type

GET /diagnostics/runs/{run_id}/artifacts/{artifact_path...}
    │
    └── resolve_run_artifact_path()           — allowed set kontrolü + passthrough servis

GET /diagnostics/runs/{run_id}/federation
    │
    └── build_run_federation_diagnostics()    — VDL projeksiyon, tanımlayıcı, salt okunur
```

### Değişmez Sınırlar

Tüm yeni endpoint'ler şu değişmezleri korur:

- `proofd != authority surface`
- `parity != consensus`
- `verification != authority`
- `observability != scheduling`
- `verification history != verifier reputation`

Yalnızca `POST /verify/bundle` artifact materialize eder. Tüm `GET /diagnostics/...`
endpoint'leri kesinlikle salt okunurdur.

---

## Bileşen Tasarımı

### 1. `POST /verify/bundle` — Mevcut Durum ve Tamamlanacaklar

`VerifyBundleRequestBody` zaten şu alanları içeriyor ve `lib.rs` içinde işleniyor:

```rust
struct VerifyBundleRequestBody {
    bundle_path: String,           // zorunlu, mutlak yol
    policy_path: String,           // zorunlu, mutlak yol
    registry_path: String,         // zorunlu, mutlak yol
    receipt_mode: Option<VerifyBundleReceiptMode>,
    run_id: String,                // zorunlu (sağlanmazsa üretilir)
    receipt_signer: Option<VerifyBundleReceiptSigner>,
    diversity_binding: Option<VerifyBundleDiversityBinding>,
    replay_boundary_binding: Option<VerifyBundleReplayBoundaryBinding>,
    trust_reuse_binding: Option<VerifyBundleTrustReuseBinding>,
}
```

Mevcut `validate_verify_bundle_request()` şu bağımlılıkları zaten zorluyor:
- `diversity_binding` → `receipt_mode: emit_signed` gerektirir
- `replay_boundary_binding` → `diversity_binding` gerektirir
- `trust_reuse_binding` → `diversity_binding` gerektirir

#### `run_id` Yönetimi

Mevcut implementasyon `run_id`'yi zorunlu tutuyor. Spec'e göre opsiyonel olmalı:

```
run_id sağlanmışsa:
    evidence/{run_id}/ dizinini kullan
    proofd_run_manifest.json içindeki fingerprint'i kontrol et
    fingerprint çakışması → HTTP 409 {"error": "run_id_fingerprint_conflict"}

run_id sağlanmamışsa:
    yeni UUID üret (format: ASCII alfanümerik + tire, max 128 karakter)
    evidence/{yeni_run_id}/ dizinini oluştur
```

`VerifyBundleRequestBody.run_id` alanı `Option<String>` olarak değiştirilmeli; `verify_bundle_request()` içinde `None` durumunda UUID üretilmeli.

#### Fingerprint Hesaplama

`compute_verify_bundle_request_fingerprint()` zaten mevcuttur. `VerifyBundleRequestBody`'nin
Canonical_JSON kodlaması SHA-256 ile hashlenir. `verify_existing_run_fingerprint()` de mevcuttur.

#### Verify Response — Normatif Alanlar

```json
{
  "status": "ok",
  "run_id": "<run_id>",
  "verdict": "<PASS|FAIL>",
  "verdict_subject": {},
  "receipt_emitted": false,
  "receipt_path": null,
  "request_fingerprint": "<sha256-hex>",
  "behavioral_observability_emitted": false,
  "findings_count": 0
}
```

Yanıt gövdesi Phase13_Forbidden_Fields kümesindeki alanları içermez.

#### Trust Reuse Çözümleme

```
bundle-native reports/trust_reuse_runtime_surface.json mevcutsa:
    → birincil kaynak olarak kullan
    → reusable event yoksa: {"status": "NO_REUSABLE_EVENTS"} yaz
    → trust_reuse_binding parametresini yok say

bundle-native surface yoksa ve trust_reuse_binding sağlanmışsa:
    → fallback kaynak olarak kullan

ikisi de yoksa:
    → trust_reuse_flow_source.json üretimini atla
```

`build_runtime_trust_reuse_flow_source_document()` zaten mevcuttur. `NO_REUSABLE_EVENTS` durumu
için mevcut string sentinel'ı `{"status": "NO_REUSABLE_EVENTS"}` structured JSON'a dönüştürülmeli.

---

### 2. Context Package Materialization — Mevcut Durum

`write_verification_context_package()` zaten `lib.rs` içinde mevcuttur ve şu dört artifact'ı
yazar:

```
evidence/{run_id}/
    context/
        policy_snapshot.json          ← Canonical_JSON
        registry_snapshot.json        ← Canonical_JSON
        context_rules.json            ← Canonical_JSON
        verification_context_object.json ← Canonical_JSON
```

`write_canonical_json_file_if_absent_or_same()` ve `copy_file_if_absent_or_same()` fonksiyonları
mevcuttur. Bu bileşen için yeni implementasyon gerekmez; mevcut davranışın spec'e uygunluğu
doğrulanır.

---

### 3. Run-Scoped Artifact Discovery — Mevcut Durum ve Tamamlanacaklar

#### `GET /diagnostics/runs/{run_id}/artifacts`

`build_run_artifact_index()` zaten mevcuttur ve şu yanıtı üretir:

```json
{
  "run_id": "<run_id>",
  "artifact_count": 3,
  "artifacts": [
    {"path": "proofd_run_manifest.json", "content_type": "application/json"},
    {"path": "context/policy_snapshot.json", "content_type": "application/json"}
  ]
}
```

`list_run_artifact_paths()` `RUN_LEVEL_ARTIFACTS` ve `NESTED_RUN_LEVEL_ARTIFACTS` sabitlerini
kullanır. Bu sabitler Allowed_Artifact_Set'i tanımlar.

Tamamlanacak: `build_run_artifact_index()` run dizini yoksa `run_dir_not_found` döndürmeli.
Mevcut `list_run_artifact_paths()` bunu zaten yapıyor; `build_run_artifact_index()` bu hatayı
propagate etmeli.

#### `GET /diagnostics/runs/{run_id}/artifacts/{artifact_path...}`

`resolve_run_artifact_path()` zaten mevcuttur. Mevcut implementasyon `list_run_artifact_paths()`
çıktısına göre 404 döndürüyor. Spec'e göre iki ayrı hata kodu gerekli:

- Yol Allowed_Artifact_Set dışındaysa → HTTP 403 `artifact_path_not_allowed`
- Yol Allowed_Artifact_Set içinde ama diskte yoksa → HTTP 404 `artifact_not_found`

`resolve_run_artifact_path()` bu iki durumu ayırt edecek biçimde güncellenmeli.

---

### 4. Run-Scoped Federation Diagnostics — Mevcut Durum ve Uyumsuzluklar

`build_run_federation_diagnostics()` zaten mevcuttur. Ancak mevcut `FederationDiagnosticsResponseBody`
spec'teki yanıt şemasıyla uyumsuz:

**Mevcut struct alanları:**
```
run_id, source_artifact_path, entry_count,
unique_verification_node_count, unique_verifier_count,
unique_authority_chain_count, unique_lineage_count,
unique_execution_cluster_count, missing_execution_cluster_entry_count,
verification_node_distribution, verifier_distribution,
authority_chain_distribution, lineage_distribution,
execution_cluster_distribution, observed_entries
```

**Spec'in gerektirdiği alanlar (Gereksinim 4):**
```
run_id, verifier_count, observed_verifiers,
authority_chain_distribution, execution_cluster_distribution,
missing_execution_cluster_entry_count
```

**Uyumsuzluk analizi:**
- `verifier_count` → mevcut `unique_verifier_count` ile karşılanabilir
- `observed_verifiers[].verifier_id` → mevcut `observed_entries[].verifier_id` ile karşılanabilir
- `observed_verifiers[].lineage_id` → mevcut `observed_entries[].lineage_id` ile karşılanabilir
- `authority_chain_distribution[].authority_chain_id` → mevcut `authority_chain_distribution[].id`
- `execution_cluster_distribution[].cluster_id` → mevcut `execution_cluster_distribution[].id`

Mevcut struct daha zengin veri içeriyor. Spec'in gerektirdiği alanlar mevcut veriden türetilebilir.
Seçenek: mevcut zengin yanıtı koruyup spec alanlarını da eklemek (additive, non-breaking).

**Sıralama:** `build_federation_distribution()` `BTreeMap` kullanıyor, dolayısıyla leksikografik
sıra zaten sağlanıyor. `observed_entries` için `verifier_id`'ye göre sıralama eklenmeli.

**run dizini yoksa:** Mevcut implementasyon `artifact_not_found` döndürüyor. Spec'e göre önce
`run_dir_not_found` kontrolü yapılmalı.

---

### 5. Diagnostics Yüzeyi Salt Okunur Değişmezi

Mevcut `is_observability_path()` ve `route_request_with_body()` içindeki POST → 405 mantığı
`/diagnostics/` prefix'i için zaten çalışıyor. `/diagnostics/runs/{run_id}/artifacts` ve
`/diagnostics/runs/{run_id}/federation` bu prefix altında olduğundan kapsanıyor.

---

## Veri Modelleri

### VerifyBundleRequestBody — Değişiklik

`run_id: String` → `run_id: Option<String>` olarak değiştirilmeli.

### FederationDiagnosticsResponseBody — Ek Alanlar

Spec uyumluluğu için mevcut struct'a şu alanlar eklenmeli:

```rust
verifier_count: usize,                          // = unique_verifier_count
observed_verifiers: Vec<SpecFederationVerifierEntry>,
```

```rust
struct SpecFederationVerifierEntry {
    verifier_id: String,
    lineage_id: Option<String>,
}
```

`authority_chain_distribution` ve `execution_cluster_distribution` içindeki `id` alanı
sırasıyla `authority_chain_id` ve `cluster_id` olarak yeniden adlandırılmalı ya da spec uyumlu
ayrı bir projeksiyon üretilmeli.

---

## Hata Kodları

| HTTP Kodu | Hata Kodu | Durum |
|-----------|-----------|-------|
| 400 | `missing_required_field` | Zorunlu alan eksik |
| 400 | `unsupported_query_parameter` | Desteklenmeyen sorgu parametresi |
| 403 | `artifact_path_not_allowed` | Artifact yolu Allowed_Artifact_Set dışında |
| 404 | `run_dir_not_found` | Run dizini mevcut değil |
| 404 | `artifact_not_found` | Artifact Allowed_Artifact_Set içinde ama diskte yok |
| 405 | `method_not_allowed` | POST diagnostics endpoint'ine |
| 409 | `run_id_fingerprint_conflict` | Aynı run_id, farklı fingerprint |
| 500 | `invalid_federation_artifact` | VDL geçersiz JSON |

---

## Property-Based Test Stratejisi

Tüm testler `proptest` kütüphanesi ile yazılır ve
`cargo test --manifest-path userspace/proofd/Cargo.toml` ile çalıştırılır. Testler CI ortamında
sınırlı ve deterministik çalışacak biçimde yapılandırılır.

### Doğruluk Özellikleri

**P1 — Run_Id Fingerprint Çakışma Koruması:**
```
∀ run_id, req1, req2:
  fingerprint(req1) ≠ fingerprint(req2) ∧ run_id aynı
  → ikinci istek HTTP 409 döndürür
```

**P2 — Context Package Determinizmi:**
```
∀ bundle, policy, registry:
  run1(bundle, policy, registry).context_package
  = run2(bundle, policy, registry).context_package
```

**P3 — Trust Reuse Öncelik Değişmezi:**
```
∀ bundle_with_native_surface, trust_reuse_binding:
  native_surface_present(bundle)
  → trust_reuse_flow_source kaynak = bundle-native
```

**P4 — Artifact Discovery Salt Okunur Değişmezi:**
```
∀ run_id, run_dir:
  files_before = list(run_dir)
  GET /diagnostics/runs/{run_id}/artifacts
  files_after = list(run_dir)
  → files_before = files_after
```

**P5 — Federation Diagnostics Forbidden Field Değişmezi:**
```
∀ run_id, ledger:
  response = GET /diagnostics/runs/{run_id}/federation
  → response ∩ Phase13_Forbidden_Fields = ∅
```

**P6 — Federation Sıralama Değişmezi:**
```
∀ run_id, ledger:
  response = GET /diagnostics/runs/{run_id}/federation
  → is_sorted(response.observed_verifiers, by: verifier_id)
  ∧ is_sorted(response.authority_chain_distribution, by: authority_chain_id)
  ∧ is_sorted(response.execution_cluster_distribution, by: cluster_id)
```

**P7 — Artifact Fetch Passthrough Değişmezi:**
```
∀ run_id, artifact_path ∈ Allowed_Artifact_Set:
  artifact_exists(run_id, artifact_path)
  → response_body = read_bytes(artifact_path)
```

**P8 — Method Not Allowed Değişmezi:**
```
∀ diagnostics_path:
  POST diagnostics_path → HTTP 405
```

---

## Kill-Switch Değişmezleri

| Değişmez | Korunma Biçimi |
|----------|----------------|
| `proofd != authority surface` | Yeni endpoint'ler artifact servis eder; karar vermez |
| `parity != consensus` | Federation diagnostics tanımlayıcıdır; seçici değil |
| `verification != authority` | Doğrulama yürütme authority resolution içermez |
| `observability != scheduling` | Diagnostics endpoint'leri routing veya scheduling input üretmez |
| `verification history != verifier reputation` | Phase13_Forbidden_Fields yanıt gövdesinden dışlanır |

---

## Uygulama Kısıtlamaları

- Tüm değişiklikler yalnızca `userspace/proofd` crate'i içinde kalır
- Yeni crate bağımlılığı eklenmez; crate içinde modül dosyası eklenebilir
- Mevcut `proptest` dev-dependency kullanılır
- `cargo test --manifest-path userspace/proofd/Cargo.toml` ile çalıştırılabilir

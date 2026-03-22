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

#### Atomic Manifest Creation

Aynı `run_id` için eşzamanlı iki `POST /verify/bundle` isteği geldiğinde race condition oluşabilir: her iki istek de manifest henüz yazılmamış görür ve birbirinin manifest'ini ezebilir.

Güvenli implementasyon `O_CREAT | O_EXCL` semantiğini kullanır:

```rust
// Rust'ta OpenOptions ile atomic create:
use std::fs::OpenOptions;

let manifest_path = run_dir.join(PROOFD_RUN_MANIFEST_FILE);
let result = OpenOptions::new()
    .write(true)
    .create_new(true)   // O_CREAT | O_EXCL — atomik, race-free
    .open(&manifest_path);

match result {
    Ok(file) => { /* manifest yaz */ }
    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
        // Başka bir istek manifest'i zaten yazdı → fingerprint kontrol et
        // Çakışma varsa → HTTP 409
    }
    Err(e) => { /* I/O hatası */ }
}
```

Bu yaklaşım şunu garanti eder: iki eşzamanlı istek aynı `run_id` için yarışırsa, yalnızca biri manifest'i yazar; diğeri `AlreadyExists` hatası alır ve fingerprint kontrolüne yönlendirilir.

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

#### `resolve_run_artifact_path()` — 403 / 404 Ayrımı ve Path Normalization

Mevcut `resolve_run_artifact_path()` yalnızca `list_run_artifact_paths()` çıktısına göre 404 döndürüyor. Spec iki ayrı hata kodu ve path normalization gerektiriyor:

```
artifact_path geldiğinde:
    1. normalize(artifact_path):
       - ".." veya "." segment içeriyorsa → HTTP 403 artifact_path_not_allowed
       - path traversal karakteri içeriyorsa → HTTP 403 artifact_path_not_allowed
    2. normalized_path ∈ Allowed_Artifact_Set?
       - hayır → HTTP 403 artifact_path_not_allowed
    3. disk'te mevcut mu?
       - hayır → HTTP 404 artifact_not_found
    4. evet → dosya içeriğini döndür
```

`parse_run_artifact_path()` zaten `is_safe_path_segment()` ile segment güvenliğini kontrol ediyor. Bu kontrol path normalization'ın ilk katmanını oluşturuyor. `resolve_run_artifact_path()` ise Allowed_Artifact_Set kontrolünü `list_run_artifact_paths()` yerine sabit küme üzerinden yapacak biçimde güncellenmeli:

```rust
fn resolve_run_artifact_path(run_dir: &Path, artifact_path: &str) -> Result<PathBuf, ServiceError> {
    // Allowed_Artifact_Set kontrolü (403)
    let allowed: std::collections::HashSet<&str> = RUN_LEVEL_ARTIFACTS
        .iter()
        .chain(NESTED_RUN_LEVEL_ARTIFACTS.iter())
        .copied()
        .collect();
    if !allowed.contains(artifact_path) {
        return Err(ServiceError::Forbidden("artifact_path_not_allowed"));
    }
    // Disk varlık kontrolü (404)
    let full_path = run_dir.join(artifact_path);
    if !full_path.is_file() {
        return Err(ServiceError::NotFound("artifact_not_found"));
    }
    Ok(full_path)
}
```

`ServiceError` enum'una `Forbidden(&'static str)` varyantı eklenmeli; `error_response()` içinde HTTP 403 olarak map'lenmeli.

---

### 4. Run-Scoped Federation Diagnostics — Mevcut Durum, Uyumsuzluklar ve Projeksiyon Katmanı

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

#### Spec Projection Layer

İç `FederationDiagnosticsResponseBody` struct'ı doğrudan API yanıtı olarak kullanılmamalıdır. İç veri modeli değişikliklerini API yüzeyinden izole etmek için bir projeksiyon katmanı kullanılır:

```rust
// İç zengin struct — değişmez
struct FederationDiagnosticsResponseBody { ... }

// Spec uyumlu projeksiyon — API yanıtı olarak serialize edilir
#[derive(Serialize)]
struct FederationDiagnosticsProjection {
    run_id: String,
    verifier_count: usize,
    observed_verifiers: Vec<SpecFederationVerifierEntry>,
    authority_chain_distribution: Vec<SpecDistributionEntry>,
    execution_cluster_distribution: Vec<SpecDistributionEntry>,
    missing_execution_cluster_entry_count: usize,
}

#[derive(Serialize)]
struct SpecFederationVerifierEntry {
    verifier_id: String,
    lineage_id: Option<String>,
}

#[derive(Serialize)]
struct SpecDistributionEntry {
    // authority_chain_distribution için: authority_chain_id
    // execution_cluster_distribution için: cluster_id
    // Ayrı struct veya #[serde(rename)] ile çözülür
}
```

`build_run_federation_diagnostics()` iç `FederationDiagnosticsResponseBody`'yi hesaplar, ardından `FederationDiagnosticsProjection`'a dönüştürür ve projeksiyon serialize edilir. Bu yaklaşım:
- İç modeli API sözleşmesinden ayırır
- Forbidden fields'ın yanlışlıkla eklenmesini engeller
- Gelecekteki iç model değişikliklerini API'yi kırmadan yapmayı mümkün kılar

**Uyumsuzluk analizi:**
- `verifier_count` → mevcut `unique_verifier_count` ile karşılanabilir
- `observed_verifiers[].verifier_id` → mevcut `observed_entries[].verifier_id` ile karşılanabilir
- `observed_verifiers[].lineage_id` → mevcut `observed_entries[].lineage_id` ile karşılanabilir
- `authority_chain_distribution[].authority_chain_id` → mevcut `authority_chain_distribution[].id`
- `execution_cluster_distribution[].cluster_id` → mevcut `execution_cluster_distribution[].id`

**Sıralama:** `build_federation_distribution()` `BTreeMap` kullanıyor, dolayısıyla leksikografik
sıra zaten sağlanıyor. `observed_verifiers` için `verifier_id`'ye göre sıralama eklenmeli.

**run dizini yoksa:** Mevcut implementasyon `artifact_not_found` döndürüyor. Spec'e göre önce
`run_dir_not_found` kontrolü yapılmalı.

---

### 6. Forbidden Fields Compile-Time Guard

Phase13_Forbidden_Fields kümesindeki alanların yanlışlıkla response struct'larına eklenmesini engellemek için yalnızca runtime test yeterli değildir. Projeksiyon katmanı bu riski azaltır, ancak ek bir güvence katmanı gereklidir.

**Yaklaşım: Test-Level Serialize Guard**

Her projeksiyon struct için bir test, struct'ın serialize çıktısını forbidden field listesiyle karşılaştırır:

```rust
#[test]
fn federation_projection_contains_no_forbidden_fields() {
    let projection = FederationDiagnosticsProjection { /* minimal örnek */ };
    let serialized = serde_json::to_value(&projection).unwrap();
    let obj = serialized.as_object().unwrap();
    for forbidden in PHASE13_FORBIDDEN_FIELDS {
        assert!(
            !obj.contains_key(*forbidden),
            "Forbidden field '{}' found in FederationDiagnosticsProjection",
            forbidden
        );
    }
}
```

`PHASE13_FORBIDDEN_FIELDS` sabit dizisi `lib.rs` içinde tanımlanır ve tüm testlerde referans alınır. Bu yaklaşım:
- Yeni bir alan struct'a eklendiğinde test anında başarısız olur
- CI'da otomatik olarak yakalanır
- Serde deny list gerektirmez (daha az invasive)

---

### 7. Diagnostics Yüzeyi Salt Okunur Değişmezi

Mevcut `is_observability_path()` ve `route_request_with_body()` içindeki POST → 405 mantığı
`/diagnostics/` prefix'i için zaten çalışıyor. `/diagnostics/runs/{run_id}/artifacts` ve
`/diagnostics/runs/{run_id}/federation` bu prefix altında olduğundan kapsanıyor.

---

## Veri Modelleri

### VerifyBundleRequestBody — Değişiklik

`run_id: String` → `run_id: Option<String>` olarak değiştirilmeli.

### FederationDiagnosticsProjection — Yeni Projeksiyon Struct'ı

İç `FederationDiagnosticsResponseBody` doğrudan serialize edilmez. Bunun yerine spec uyumlu
projeksiyon struct'ı API yanıtı olarak kullanılır:

```rust
#[derive(Serialize)]
struct FederationDiagnosticsProjection {
    run_id: String,
    verifier_count: usize,
    observed_verifiers: Vec<SpecFederationVerifierEntry>,
    authority_chain_distribution: Vec<SpecAuthorityChainEntry>,
    execution_cluster_distribution: Vec<SpecExecutionClusterEntry>,
    missing_execution_cluster_entry_count: usize,
}

#[derive(Serialize)]
struct SpecFederationVerifierEntry {
    verifier_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    lineage_id: Option<String>,
}

#[derive(Serialize)]
struct SpecAuthorityChainEntry {
    authority_chain_id: String,
    entry_count: usize,
}

#[derive(Serialize)]
struct SpecExecutionClusterEntry {
    cluster_id: String,
    entry_count: usize,
}
```

`build_run_federation_diagnostics()` iç `FederationDiagnosticsResponseBody`'yi hesaplar,
ardından `FederationDiagnosticsProjection`'a dönüştürür. Projeksiyon serialize edilir.

### PHASE13_FORBIDDEN_FIELDS — Sabit Dizi

```rust
const PHASE13_FORBIDDEN_FIELDS: &[&str] = &[
    "preferred_verifier", "winning_verifier", "trust_rank",
    "verifier_score", "trust_score", "reliability_index",
    "weighted_authority", "correctness_rate", "agreement_ratio",
    "node_success_ratio", "verifier_reputation",
    "recommended_action", "routing_hint", "execution_override",
    "retry", "override", "promote", "commit", "mitigation",
    "node_priority", "verification_weight",
];
```

Bu sabit `lib.rs` içinde tanımlanır; serialize guard testleri bu sabite referans verir.

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

**P9 — Artifact Path Normalization Değişmezi:**
```
∀ artifact_path:
  contains_traversal_segment(artifact_path)   -- ".." veya "." segment
  → HTTP 403 artifact_path_not_allowed

∀ artifact_path ∉ Allowed_Artifact_Set:
  normalize(artifact_path) ∉ Allowed_Artifact_Set
  → HTTP 403 artifact_path_not_allowed
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
| `diagnostics never influence verification result` | Diagnostics code path, verification decision path'ten tamamen ayrıdır; `GET /diagnostics/...` endpoint'leri doğrulama sonucunu etkileyecek hiçbir yan etki üretmez |

---

## Uygulama Kısıtlamaları

- Tüm değişiklikler yalnızca `userspace/proofd` crate'i içinde kalır
- Yeni crate bağımlılığı eklenmez; crate içinde modül dosyası eklenebilir
- Mevcut `proptest` dev-dependency kullanılır
- `cargo test --manifest-path userspace/proofd/Cargo.toml` ile çalıştırılabilir

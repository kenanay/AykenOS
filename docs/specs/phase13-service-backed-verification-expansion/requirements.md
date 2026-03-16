# Gereksinimler Belgesi

## Giriş

Phase-13 §4.1 Service-Backed Verification Expansion, `proofd` userspace servisinin doğrulama
yürütme, imzalı makbuz üretimi ve diagnostics sorgu yüzeyini genişletir. Bu spec,
`PHASE13_ARCHITECTURE_MAP.md §4.1` workstream'ini kapsar.

Temel mimari kural değişmez:

```
proofd = verification execution service + diagnostics service surface
proofd != authority surface
```

Bu spec kapsamındaki genişleme iki gruba ayrılır:

**Grup A — Verify + Context + Trust Reuse:**
- `POST /verify/bundle` endpoint'inin `diversity_binding`, `replay_boundary_binding`,
  `trust_reuse_binding` ve `run_id` parametreleriyle genişletilmesi
- Context package materialization: dört kanonik artifact run dizinine yazılır
- Trust reuse runtime surface: bundle-native `reports/trust_reuse_runtime_surface.json`
  entegrasyonu

**Grup B — Diagnostics Surface Expansion:**
- Run-scoped artifact discovery: `GET /diagnostics/runs/{run_id}/artifacts` ve
  `GET /diagnostics/runs/{run_id}/artifacts/{artifact_path...}`
- Run-scoped federation diagnostics: `GET /diagnostics/runs/{run_id}/federation`

Tüm değişiklikler `userspace/proofd` crate'i içinde kalır; yeni crate bağımlılığı eklenmez.
Crate içinde modül dosyası eklenebilir.

## Sözlük

- **Proofd**: `userspace/proofd` Rust servisi; bundle doğrulaması yürütür ve salt okunur
  diagnostics sunar.
- **Run**: `POST /verify/bundle` çağrısının tek bir yürütmesi; `run_id` ile tanımlanır ve
  `evidence/{run_id}/` altında artifact üretir.
- **Run_Id**: Doğrulama çalıştırmasını benzersiz biçimde tanımlayan string; `POST /verify/bundle`
  isteğinde sağlanır. Sağlanmazsa servis tarafından üretilir. Format: yalnızca ASCII alfanümerik,
  tire ve alt çizgi; boş olamaz; path traversal karakteri içeremez; maksimum 128 karakter.
- **Request_Fingerprint**: Kanonik `VerifyBundleRequestBody`'nin SHA-256 hex hash'i;
  `proofd_run_manifest.json` içinde saklanır. Aynı fingerprint'e sahip iki run, özdeş istek
  parametreleriyle çağrılmış demektir.
- **Canonical_JSON**: `proof_verifier::canonical::jcs::canonicalize_json` ile üretilen
  deterministik JSON kodlaması. Bu fonksiyon crate içinde mevcuttur.
- **Verify_Bundle_Endpoint**: `POST /verify/bundle` endpoint'i; doğrulama yürütür ve run
  artifact'larını üretir.
- **Diversity_Binding**: `POST /verify/bundle` isteğinde opsiyonel parametre; doğrulama
  çeşitlilik bağlamasını etkinleştirir. `receipt_mode: emit_signed` gerektirir.
- **Replay_Boundary_Binding**: `POST /verify/bundle` isteğinde opsiyonel Stage-2 parametre;
  replay sınır bağlamasını etkinleştirir. `diversity_binding` gerektirir.
- **Trust_Reuse_Binding**: `POST /verify/bundle` isteğinde opsiyonel fallback parametre;
  bundle-native trust reuse runtime surface yoksa kullanılır. `diversity_binding` gerektirir.
- **Context_Package**: Doğrulama sırasında run dizinine yazılan dört artifact kümesi:
  `context/policy_snapshot.json`, `context/registry_snapshot.json`,
  `context/context_rules.json`, `context/verification_context_object.json`.
- **Trust_Reuse_Runtime_Surface**: Bundle-native `reports/trust_reuse_runtime_surface.json`
  artifact'ı; trust reuse kanıtı için tercih edilen kaynak.
- **Artifact_Discovery_Endpoint**: `GET /diagnostics/runs/{run_id}/artifacts` endpoint'i;
  run-local kanonik artifact yollarını listeler.
- **Artifact_Fetch_Endpoint**: `GET /diagnostics/runs/{run_id}/artifacts/{artifact_path...}`
  endpoint'i; tek bir run artifact'ını salt okunur olarak döndürür.
- **Federation_Diagnostics_Endpoint**: `GET /diagnostics/runs/{run_id}/federation` endpoint'i;
  `verification_diversity_ledger.json` üzerinden tanımlayıcı projeksiyon sunar.
- **Verification_Diversity_Ledger**: `verification_diversity_ledger.json` artifact'ı;
  doğrulayıcı federasyon dağılımını kaydeder.
- **Allowed_Artifact_Set**: `GET /diagnostics/runs/{run_id}/artifacts/{artifact_path...}`
  üzerinden erişilebilen kanonik run artifact yolları kümesi. Normatif liste Gereksinim 3'te
  tanımlanmıştır.
- **Evidence_Root**: `proofd`'a evidence tabanı olarak geçirilen dizin; her run için bir alt
  dizin içerir.
- **Phase13_Forbidden_Fields**: Tüm yeni endpoint'lerde yanıt gövdesinde bulunması yasak olan
  alan adları kümesi. Normatif liste Gereksinim 8'de tanımlanmıştır.

## Gereksinimler

### Gereksinim 1: `POST /verify/bundle` Parametre Genişletmesi

**Kullanıcı Hikayesi:** Bir doğrulama operatörü olarak, `POST /verify/bundle` endpoint'ine
`diversity_binding`, `replay_boundary_binding`, `trust_reuse_binding` ve `run_id` parametrelerini
geçirebilmek istiyorum; böylece doğrulama çalıştırması bağlam bağlaması ve run-scoped artifact
üretimiyle zenginleştirilir.

#### Kabul Kriterleri

1. WHEN `POST /verify/bundle` isteği `bundle_path`, `policy_path`, `registry_path` alanlarıyla
   gönderildiğinde, THE Verify_Bundle_Endpoint SHALL doğrulamayı yürütür ve HTTP 200 ile aşağıdaki
   normatif alanları içeren yanıt döndürür: `run_id` (string), `status` (string),
   `verdict` (string), `verdict_subject` (object), `receipt_emitted` (boolean),
   `request_fingerprint` (string), `findings_count` (non-negative integer).
2. WHEN `POST /verify/bundle` isteği `run_id` alanı içerdiğinde, THE Verify_Bundle_Endpoint SHALL
   tüm run artifact'larını `evidence/{run_id}/` dizinine yazar.
3. WHEN `POST /verify/bundle` isteği `run_id` alanı içermediğinde, THE Verify_Bundle_Endpoint
   SHALL yeni bir `run_id` üretir (format: ASCII alfanümerik, tire, alt çizgi; max 128 karakter)
   ve artifact'ları ilgili dizine yazar.
4. WHEN `POST /verify/bundle` isteği `diversity_binding` alanı içerdiğinde, THE
   Verify_Bundle_Endpoint SHALL `replay_boundary_flow_source.json`'ı bundle'ın kendi replay
   runtime surface'inden üretir.
5. WHEN `POST /verify/bundle` isteği `diversity_binding` içermediğinde, THE
   Verify_Bundle_Endpoint SHALL `replay_boundary_flow_source.json` üretimini atlar.
6. WHEN `POST /verify/bundle` isteği `trust_reuse_binding` alanı içerdiğinde ve bundle-native
   `reports/trust_reuse_runtime_surface.json` mevcut değilse, THE Verify_Bundle_Endpoint SHALL
   `trust_reuse_binding` değerini fallback olarak kullanır.
7. WHEN bundle-native `reports/trust_reuse_runtime_surface.json` mevcutsa, THE
   Verify_Bundle_Endpoint SHALL bu artifact'ı `trust_reuse_flow_source.json` üretimi için tercih
   eder ve `trust_reuse_binding` parametresini yok sayar.
8. WHEN bundle-native `reports/trust_reuse_runtime_surface.json` mevcutsa ancak değerlendirme
   sonucu yeniden kullanılabilir yol üretmemişse, THE Verify_Bundle_Endpoint SHALL
   `trust_reuse_flow_source.json` içinde `{"status": "NO_REUSABLE_EVENTS"}` yazar.
9. WHEN aynı `run_id` altında farklı kanonik istek fingerprint'iyle ikinci bir
   `POST /verify/bundle` isteği geldiğinde, THE Verify_Bundle_Endpoint SHALL fail-closed davranır
   ve HTTP 409 ile `{"error": "run_id_fingerprint_conflict"}` döndürür.
10. WHEN `POST /verify/bundle` isteği zorunlu alanlardan herhangi birini (`bundle_path`,
    `policy_path`, `registry_path`) içermediğinde, THE Verify_Bundle_Endpoint SHALL HTTP 400 ile
    `{"error": "missing_required_field"}` döndürür.
11. THE Verify_Bundle_Endpoint yanıt gövdesi Phase13_Forbidden_Fields kümesindeki alanların
    hiçbirini içermez.

### Gereksinim 2: Context Package Materialization

**Kullanıcı Hikayesi:** Bir doğrulama operatörü olarak, her doğrulama çalıştırmasının run dizinine
kanonik bir context package yazmasını istiyorum; böylece doğrulama bağlamı izlenebilir ve
diagnostics yüzeyinden sorgulanabilir olur.

#### Kabul Kriterleri

1. WHEN `POST /verify/bundle` başarıyla tamamlandığında, THE Verify_Bundle_Endpoint SHALL
   `context/policy_snapshot.json`'ı run dizinine Canonical_JSON kodlamasıyla yazar.
2. WHEN `POST /verify/bundle` başarıyla tamamlandığında, THE Verify_Bundle_Endpoint SHALL
   `context/registry_snapshot.json`'ı run dizinine Canonical_JSON kodlamasıyla yazar.
3. WHEN `POST /verify/bundle` başarıyla tamamlandığında, THE Verify_Bundle_Endpoint SHALL
   `context/context_rules.json`'ı run dizinine Canonical_JSON kodlamasıyla yazar.
4. WHEN `POST /verify/bundle` başarıyla tamamlandığında, THE Verify_Bundle_Endpoint SHALL
   `context/verification_context_object.json`'ı run dizinine Canonical_JSON kodlamasıyla yazar.
5. WHEN bundle-native `reports/trust_reuse_runtime_surface.json` trust reuse kanıtı olarak
   kullanıldığında, THE Verify_Bundle_Endpoint SHALL bu artifact'ı run dizinine kopyalar.
6. WHEN `context/policy_snapshot.json` run dizininde zaten mevcutsa, THE Verify_Bundle_Endpoint
   SHALL mevcut dosyanın baytlarının yeni Canonical_JSON kodlamasıyla eşleştiğini doğrular;
   çakışma varsa hata döndürür.
7. WHEN `context/registry_snapshot.json` run dizininde zaten mevcutsa, THE Verify_Bundle_Endpoint
   SHALL mevcut dosyanın baytlarının yeni Canonical_JSON kodlamasıyla eşleştiğini doğrular;
   çakışma varsa hata döndürür.
8. THE Verify_Bundle_Endpoint SHALL context package artifact'larını authority resolution, trust
   election veya consensus semantiği olmadan yazar.
9. Yalnızca `POST /verify/bundle` artifact materialize eder. Tüm `GET /diagnostics/...`
   endpoint'leri kesinlikle salt okunurdur ve hiçbir artifact yazmaz.

### Gereksinim 3: Run-Scoped Artifact Discovery

**Kullanıcı Hikayesi:** Bir doğrulama operatörü olarak, belirli bir run'ın ürettiği artifact'ları
listeleyebilmek ve tek tek okuyabilmek istiyorum; böylece run çıktılarını authority resolution
olmadan inceleyebilirim.

#### Normatif Allowed_Artifact_Set

Aşağıdaki kanonik yollar `GET /diagnostics/runs/{run_id}/artifacts/{artifact_path...}` üzerinden
erişilebilir. Bu küme dışındaki her yol HTTP 403 döndürür.

```
proofd_run_manifest.json
receipts/verification_receipt.json
receipts/signed_verification_receipt.json
context/policy_snapshot.json
context/registry_snapshot.json
context/context_rules.json
context/verification_context_object.json
reports/trust_reuse_runtime_surface.json
replay_boundary_flow_source.json
trust_reuse_flow_source.json
verification_diversity_ledger_binding.json
verification_diversity_ledger.json
verification_diversity_ledger_append_report.json
verification_audit_ledger.jsonl
report.json
parity_report.json
parity_authority_suppression_report.json
parity_authority_drift_topology.json
parity_incident_graph.json
parity_determinism_incidents.json
parity_drift_attribution_report.json
parity_convergence_report.json
failure_matrix.json
```

#### Kabul Kriterleri

1. WHEN `GET /diagnostics/runs/{run_id}/artifacts` çağrıldığında ve run dizini mevcutsa, THE
   Artifact_Discovery_Endpoint SHALL HTTP 200 ile `run_id` (string), `artifact_count`
   (non-negative integer) ve `artifacts` (array) alanlarını içeren yanıt döndürür; her artifact
   elemanı `path` (string) ve `content_type` (string) alanlarına sahiptir.
2. WHEN `GET /diagnostics/runs/{run_id}/artifacts` çağrıldığında ve run dizini mevcut değilse,
   THE Artifact_Discovery_Endpoint SHALL HTTP 404 ile `{"error": "run_dir_not_found"}` döndürür.
3. WHEN `GET /diagnostics/runs/{run_id}/artifacts` çağrıldığında sorgu parametresi içeriyorsa,
   THE Artifact_Discovery_Endpoint SHALL HTTP 400 ile
   `{"error": "unsupported_query_parameter"}` döndürür.
4. WHEN `POST /diagnostics/runs/{run_id}/artifacts` çağrıldığında, THE Proofd SHALL HTTP 405 ile
   `{"error": "method_not_allowed"}` döndürür.
5. WHEN `GET /diagnostics/runs/{run_id}/artifacts/{artifact_path...}` çağrıldığında ve artifact
   Allowed_Artifact_Set içinde mevcutsa, THE Artifact_Fetch_Endpoint SHALL HTTP 200 ile artifact
   içeriğini değiştirilmeden döndürür.
6. WHEN `GET /diagnostics/runs/{run_id}/artifacts/{artifact_path...}` çağrıldığında ve artifact
   Allowed_Artifact_Set içinde ama diskte mevcut değilse, THE Artifact_Fetch_Endpoint SHALL
   HTTP 404 ile `{"error": "artifact_not_found"}` döndürür.
7. WHEN `GET /diagnostics/runs/{run_id}/artifacts/{artifact_path...}` çağrıldığında ve artifact
   yolu Allowed_Artifact_Set dışındaysa, THE Artifact_Fetch_Endpoint SHALL HTTP 403 ile
   `{"error": "artifact_path_not_allowed"}` döndürür.
8. THE Artifact_Fetch_Endpoint SHALL artifact içeriğini değiştirmez, yorumlamaz veya
   dönüştürmez.
9. THE Artifact_Discovery_Endpoint SHALL herhangi bir artifact'ı diske yazmaz.
10. THE Artifact_Fetch_Endpoint SHALL herhangi bir artifact'ı diske yazmaz.

### Gereksinim 4: Run-Scoped Federation Diagnostics

**Kullanıcı Hikayesi:** Bir doğrulama operatörü olarak, bir run'ın
`verification_diversity_ledger.json` artifact'ı üzerinden tanımlayıcı bir federasyon projeksiyonu
görmek istiyorum; böylece doğrulayıcı dağılımını authority resolution olmadan inceleyebilirim.

#### Kabul Kriterleri

1. WHEN `GET /diagnostics/runs/{run_id}/federation` çağrıldığında ve
   `verification_diversity_ledger.json` mevcutsa, THE Federation_Diagnostics_Endpoint SHALL
   HTTP 200 ile tanımlayıcı federasyon projeksiyonunu döndürür.
2. WHEN `GET /diagnostics/runs/{run_id}/federation` çağrıldığında ve
   `verification_diversity_ledger.json` mevcut değilse, THE Federation_Diagnostics_Endpoint SHALL
   HTTP 404 ile `{"error": "artifact_not_found"}` döndürür.
3. WHEN `GET /diagnostics/runs/{run_id}/federation` çağrıldığında ve run dizini mevcut değilse,
   THE Federation_Diagnostics_Endpoint SHALL HTTP 404 ile
   `{"error": "run_dir_not_found"}` döndürür.
4. WHEN `GET /diagnostics/runs/{run_id}/federation` çağrıldığında sorgu parametresi içeriyorsa,
   THE Federation_Diagnostics_Endpoint SHALL HTTP 400 ile
   `{"error": "unsupported_query_parameter"}` döndürür.
5. WHEN `POST /diagnostics/runs/{run_id}/federation` çağrıldığında, THE Proofd SHALL HTTP 405 ile
   `{"error": "method_not_allowed"}` döndürür.
6. THE Federation_Diagnostics_Endpoint SHALL yanıt gövdesinde `run_id` (string) alanını içerir.
7. THE Federation_Diagnostics_Endpoint SHALL yanıt gövdesinde `verifier_count` (non-negative
   integer) alanını içerir.
8. THE Federation_Diagnostics_Endpoint SHALL yanıt gövdesinde `observed_verifiers` dizisini
   içerir; her eleman `verifier_id` (string) ve `lineage_id` (opsiyonel string) alanlarına
   sahiptir.
9. THE Federation_Diagnostics_Endpoint SHALL yanıt gövdesinde `authority_chain_distribution`
   dizisini içerir; her eleman `authority_chain_id` (string) ve `entry_count` (non-negative
   integer) alanlarına sahiptir.
10. THE Federation_Diagnostics_Endpoint SHALL yanıt gövdesinde `execution_cluster_distribution`
    dizisini içerir; her eleman `cluster_id` (string) ve `entry_count` (non-negative integer)
    alanlarına sahiptir. `execution_cluster_id` alanı null olan ledger girişleri bu dağılıma dahil
    edilmez; bunun yerine yanıtta `missing_execution_cluster_entry_count` (non-negative integer)
    alanı olarak raporlanır.
11. THE Federation_Diagnostics_Endpoint SHALL herhangi bir artifact'ı diske yazmaz.
12. THE Federation_Diagnostics_Endpoint SHALL tercih edilen doğrulayıcı seçmez, doğrulayıcı
    güvenini sıralamaz, authority resolution yazmaz veya federasyon gerçek seçimi ima etmez.

### Gereksinim 5: Federation Diagnostics Yanıt Semantiği ve Fail-Closed Kısıtlamalar

**Kullanıcı Hikayesi:** Bir sistem mimarı olarak, federation diagnostics projeksiyonunun tamamen
tanımlayıcı ve kesinlikle fail-closed olmasını istiyorum; böylece endpoint doğrulama sonuçlarını
etkilemek veya policy kararları almak için kullanılamaz.

#### Kabul Kriterleri

1. THE Federation_Diagnostics_Endpoint SHALL yanıt gövdesinde Phase13_Forbidden_Fields kümesindeki
   alanların hiçbirini içermez.
2. THE Federation_Diagnostics_Endpoint SHALL `observed_verifiers` dizisini `verifier_id` değerine
   göre leksikografik sırayla döndürür.
3. THE Federation_Diagnostics_Endpoint SHALL `authority_chain_distribution` dizisini
   `authority_chain_id` değerine göre leksikografik sırayla döndürür.
4. THE Federation_Diagnostics_Endpoint SHALL `execution_cluster_distribution` dizisini `cluster_id`
   değerine göre leksikografik sırayla döndürür.
5. WHEN `verification_diversity_ledger.json` geçerli JSON olarak ayrıştırılamıyorsa, THE
   Federation_Diagnostics_Endpoint SHALL HTTP 500 ile
   `{"error": "invalid_federation_artifact"}` döndürür.
6. THE Proofd SHALL bu endpoint aracılığıyla authority resolution, trust election veya consensus
   semantiği eklemez.

### Gereksinim 6: Trust Reuse Runtime Surface Entegrasyonu

**Kullanıcı Hikayesi:** Bir doğrulama operatörü olarak, `proofd`'un bundle-native trust reuse
kanıtını tercih etmesini ve yalnızca bu kanıt yoksa explicit `trust_reuse_binding` parametresine
düşmesini istiyorum; böylece trust reuse akışı doğal bundle kanıtına dayanır.

#### Kabul Kriterleri

1. WHEN bundle-native `reports/trust_reuse_runtime_surface.json` mevcutsa, THE
   Verify_Bundle_Endpoint SHALL bu artifact'ı `trust_reuse_flow_source.json` üretimi için birincil
   kaynak olarak kullanır.
2. WHEN bundle-native `reports/trust_reuse_runtime_surface.json` mevcut değilse ve
   `trust_reuse_binding` parametresi sağlanmışsa, THE Verify_Bundle_Endpoint SHALL
   `trust_reuse_binding` değerini fallback kaynak olarak kullanır.
3. WHEN bundle-native `reports/trust_reuse_runtime_surface.json` mevcut değilse ve
   `trust_reuse_binding` parametresi de sağlanmamışsa, THE Verify_Bundle_Endpoint SHALL
   `trust_reuse_flow_source.json` üretimini atlar.
4. WHEN bundle-native trust reuse değerlendirmesi yeniden kullanılabilir yol üretmemişse, THE
   Verify_Bundle_Endpoint SHALL `trust_reuse_flow_source.json` içine
   `{"status": "NO_REUSABLE_EVENTS"}` yazar. "Yeniden kullanılabilir yol üretmeme" kararı
   bundle-native evaluator output contract'ına göre belirlenir.
5. THE Verify_Bundle_Endpoint SHALL trust reuse akışını authority resolution, trust election veya
   consensus semantiği olmadan yürütür.
6. FOR ALL geçerli bundle'lar, `trust_reuse_flow_source.json` üretimi deterministik olmalıdır:
   aynı bundle ve aynı parametrelerle çağrılan iki run özdeş `trust_reuse_flow_source.json`
   üretir.

### Gereksinim 7: Diagnostics Yüzeyi Salt Okunur Değişmezi

**Kullanıcı Hikayesi:** Bir sistem mimarı olarak, tüm `GET /diagnostics/runs/{run_id}/...`
endpoint'lerinin kesinlikle salt okunur kalmasını istiyorum; böylece diagnostics yüzeyi hiçbir
zaman kontrol düzlemine veya mutasyon semantiğine dönüşmez.

#### Kabul Kriterleri

1. THE Artifact_Discovery_Endpoint SHALL diske hiçbir artifact yazmaz.
2. THE Artifact_Fetch_Endpoint SHALL diske hiçbir artifact yazmaz.
3. THE Federation_Diagnostics_Endpoint SHALL diske hiçbir artifact yazmaz.
4. WHEN herhangi bir `GET /diagnostics/runs/{run_id}/...` endpoint'ine `POST` isteği
   gönderildiğinde, THE Proofd SHALL HTTP 405 ile `{"error": "method_not_allowed"}` döndürür.
5. WHEN herhangi bir `GET /diagnostics/runs/{run_id}/...` endpoint'ine desteklenmeyen sorgu
   parametresi içeren `GET` isteği gönderildiğinde, THE Proofd SHALL HTTP 400 ile
   `{"error": "unsupported_query_parameter"}` döndürür.
6. THE Proofd SHALL diagnostics endpoint'lerinden döndürülen artifact içeriğine hiçbir alan
   eklemez, hiçbir alanı kaldırmaz ve hiçbir değeri dönüştürmez.

### Gereksinim 8: Phase-13 Forbidden Fields Sözleşmesi

**Kullanıcı Hikayesi:** Bir sistem mimarı olarak, bu spec kapsamındaki tüm yeni endpoint'lerin
merkezi bir forbidden fields sözleşmesine uymasını istiyorum; böylece `proofd` authority, majority
veya control-plane semantiğine kaymaz.

#### Normatif Phase13_Forbidden_Fields Kümesi

Aşağıdaki alan adları bu spec kapsamındaki tüm yeni endpoint yanıtlarında bulunması yasaktır:

```
preferred_verifier, winning_verifier, trust_rank,
verifier_score, trust_score, reliability_index,
weighted_authority, correctness_rate, agreement_ratio,
node_success_ratio, verifier_reputation,
recommended_action, routing_hint, execution_override,
retry, override, promote, commit, mitigation,
node_priority, verification_weight
```

#### Kabul Kriterleri

1. THE Proofd SHALL `proofd != authority surface` değişmezini bu spec kapsamındaki tüm yeni
   endpoint'lerde korur.
2. THE Proofd SHALL `parity != consensus` değişmezini bu spec kapsamındaki tüm yeni
   endpoint'lerde korur.
3. THE Proofd SHALL `verification != authority` değişmezini bu spec kapsamındaki tüm yeni
   endpoint'lerde korur.
4. THE Proofd SHALL `observability != scheduling` değişmezini bu spec kapsamındaki tüm yeni
   endpoint'lerde korur.
5. THE Proofd SHALL `verification history != verifier reputation` değişmezini bu spec kapsamındaki
   tüm yeni endpoint'lerde korur.
6. THE Proofd SHALL yeni endpoint'lerin hiçbirinde Phase13_Forbidden_Fields kümesindeki alanları
   içermez.

### Gereksinim 9: Uygulama Kısıtlamaları

**Kullanıcı Hikayesi:** Bir geliştirici olarak, tüm değişikliklerin mevcut `proofd` crate'i
içinde kalmasını istiyorum; böylece yeni crate bağımlılıkları eklenmez ve mimari sınırlar temiz
kalır.

#### Kabul Kriterleri

1. THE Proofd SHALL tüm değişiklikleri yalnızca `userspace/proofd` crate'i içinde uygular; yeni
   crate bağımlılığı eklenmez. Crate içinde yeni modül dosyası eklenebilir.
2. THE Proofd SHALL tüm testleri `proptest` kütüphanesini kullanarak yazar (zaten dev-dependency
   olarak mevcuttur).
3. THE Proofd SHALL tüm testleri `cargo test --manifest-path userspace/proofd/Cargo.toml`
   komutuyla çalıştırılabilir biçimde yazar.
4. THE Proofd SHALL property testlerini CI ortamında sınırlı ve deterministik çalışacak biçimde
   yazar.

# Gereksinimler Belgesi

## Giriş

Phase-13 kill-switch CI kapıları, `proofd` servisinin ve onun diagnostics yüzeylerinin mimari sınırlarını zorunlu kılan dört Rust property-based test kapısıdır. Bu kapılar, `userspace/proofd/src/lib.rs` içinde uygulanır ve şu temel mimari değişmezleri doğrular:

- `observability != scheduling` (gözlemlenebilirlik, zamanlama kararlarını yönlendirmez)
- `convergence != truth election` (yakınsama, gerçek seçimi değildir)
- `verification history != verifier reputation` (doğrulama geçmişi, doğrulayıcı itibarı değildir)
- `proofd diagnostics = read-only artifact passthrough` (proofd diagnostics yalnızca salt okunur artifact geçişidir)

Bu kapılar, AykenOS'un deterministik dağıtık doğrulama sistemi olarak kalmasını sağlar ve dağıtık konsensüs davranışına doğru kaymasını engeller.

## Sözlük

- **Proofd**: `userspace/proofd` Rust servisi; bundle doğrulaması yürütür ve salt okunur diagnostics sunar.
- **Kill_Switch_Gate**: Mimari kategori kimliğinin bozulmaya başlaması durumunda build'i anında öldüren CI kapısı.
- **Observability_Boundary**: `proofd` diagnostics ad alanının salt okunur, artifact destekli ve yetkisiz olma zorunluluğu.
- **Routing_Separation**: Doğrulama yönlendirme/zamanlama kodunun tanımlayıcı gözlemlenebilirlik alanlarını içe aktarmaması veya tüketmemesi zorunluluğu.
- **Convergence_Non_Election**: Yakınsama artifact'larının gerçek seçimi veya seçim alanlarını açığa çıkarmaması zorunluluğu.
- **Reputation_Prohibition**: Diagnostics artifact'larının doğrulayıcı itibarı veya puanlama alanlarını açığa çıkarmaması zorunluluğu.
- **Forbidden_Field**: Bir yanıt gövdesinde veya kaynak kodunda bulunması yasak olan alan adı.
- **Parity_Artifact**: `parity_convergence_report.json` veya `parity_drift_attribution_report.json` gibi parity diagnostics dosyaları.
- **Route_Request**: `proofd` içindeki `route_request` ve `route_request_with_body` fonksiyonları.
- **Source_Scan**: Kaynak kod baytlarını test zamanında okuyarak yasak alan adlarının varlığını doğrulayan deterministik birim testi.

## Gereksinimler

### Gereksinim 1: Observability Sınır Kapısı (`ci-gate-proofd-observability-boundary`)

**Kullanıcı Hikayesi:** Bir sistem mimarı olarak, `proofd` diagnostics ad alanının salt okunur, artifact destekli ve yetkisiz kalmasını istiyorum; böylece gözlemlenebilirlik yüzeyleri hiçbir zaman kontrol düzlemine, gerçek seçimine veya mutasyon semantiğine dönüşmez.

#### Kabul Kriterleri

1. WHEN herhangi bir observability yoluna (`/diagnostics/` ile başlayan) `POST` isteği gönderildiğinde, THE Proofd SHALL HTTP 405 ve `{"error": "method_not_allowed"}` döndürür.
2. WHEN herhangi bir observability yoluna desteklenmeyen sorgu parametresi içeren `GET` isteği gönderildiğinde, THE Proofd SHALL HTTP 400 ve `{"error": "unsupported_query_parameter"}` döndürür.
3. WHEN herhangi bir observability `GET` uç noktasından yanıt alındığında, THE Proofd SHALL yanıt gövdesinde şu yasak alanların hiçbirini içermez: `dominant_authority_chain_id`, `largest_outcome_cluster_size`, `outcome_convergence_ratio`, `global_status`, `historical_authority_islands`, `insufficient_evidence_islands`.
4. WHEN herhangi bir observability `GET` uç noktasından yanıt alındığında, THE Proofd SHALL yanıt gövdesinde şu kontrol düzlemi ipuçlarının hiçbirini içermez: `retry`, `override`, `promote`, `commit`, `recommended_action`, `mitigation`, `routing_hint`, `node_priority`, `verification_weight`, `execution_override`.
5. THE Proofd SHALL `/diagnostics/incidents` dışındaki tüm `GET` uç noktalarında sorgu parametrelerini reddetir.
6. WHERE `/diagnostics/incidents` uç noktası kullanıldığında, THE Proofd SHALL yalnızca `severity`, `surface_key` ve `node_id` sorgu parametrelerine izin verir.

### Gereksinim 2: Observability Yönlendirme Ayrımı Kapısı (`ci-gate-observability-routing-separation`)

**Kullanıcı Hikayesi:** Bir sistem mimarı olarak, doğrulama yönlendirme ve zamanlama kodunun tanımlayıcı gözlemlenebilirlik alanlarını içe aktarmamasını veya tüketmemesini istiyorum; böylece gözlemlenebilirlik artifact'ları hiçbir zaman doğrulama çeşitliliğini veya yönlendirme sırasını etkilemez.

#### Kabul Kriterleri

1. THE Source_Scan SHALL `handle_run_endpoint`, `route_request` ve `route_request_with_body` fonksiyon gövdelerinde şu yasak alan adlarının hiçbirinin bulunmadığını doğrular: `dominant_authority_chain_id`, `largest_outcome_cluster_size`, `outcome_convergence_ratio`, `global_status`, `historical_authority_islands`, `insufficient_evidence_islands`.
2. THE Source_Scan SHALL bu üç fonksiyonun doğrudan çağırdığı yardımcı fonksiyon gövdelerini de kapsar; yasak alan adının yalnızca bir alt yardımcıya taşınması kapıyı devre dışı bırakmaz.
3. THE Source_Scan SHALL yönlendirme ve zamanlama bağlamlarında bu yasak alanların string literal olarak da bulunmadığını doğrular.
4. THE Source_Scan SHALL kaynak dosyayı test zamanında okur ve deterministik olarak çalışır; harici dosya sistemi durumuna bağımlı değildir.
5. IF yasak alan adlarından herhangi biri yönlendirme fonksiyon bağlamlarında tespit edilirse, THEN THE Gate SHALL başarısız olur ve hangi alanın ihlali tetiklediğini raporlar.

### Gereksinim 3: Yakınsama Seçim Dışı Sınır Kapısı (`ci-gate-convergence-non-election-boundary`)

**Kullanıcı Hikayesi:** Bir sistem mimarı olarak, yakınsama artifact'larının gerçek seçimi veya seçim semantiği alanlarını açığa çıkarmamasını istiyorum; böylece parity ve yakınsama raporları hiçbir zaman hangi kümenin veya bölümün kazandığını ilan etmez.

#### Kabul Kriterleri

1. WHEN `parity_convergence_report.json` artifact'ı `route_request` aracılığıyla sunulduğunda, THE Proofd SHALL yanıt gövdesinde şu yasak seçim alanlarının hiçbirini içermez: `winning_cluster`, `selected_partition`, `preferred_cluster`, `cluster_policy_input`, `partition_replay_admission`, `verification_weight`, `execution_route`, `committed_cluster`.
2. WHEN `parity_drift_attribution_report.json` artifact'ı `route_request` aracılığıyla sunulduğunda, THE Proofd SHALL yanıt gövdesinde aynı yasak seçim alanlarının hiçbirini içermez.
3. THE Proofd SHALL sentetik parity artifact'larını geçici dizine yazıp `route_request` aracılığıyla sunarak bu kısıtlamayı doğrular; parity artifact'ları raw passthrough olarak sunulsa bile, diagnostics surface contract gereği yasak seçim alanları içeren artifact'lar diagnostics yanıt yüzeyinde kabul edilmez.
4. IF herhangi bir yasak seçim alanı yakınsama veya drift attribution yanıtında tespit edilirse, THEN THE Gate SHALL başarısız olur.

### Gereksinim 4: Doğrulayıcı İtibar Yasağı Kapısı (`ci-gate-verifier-reputation-prohibition`)

**Kullanıcı Hikayesi:** Bir sistem mimarı olarak, diagnostics artifact'larının doğrulayıcı itibarı veya puanlama alanlarını açığa çıkarmamasını istiyorum; böylece doğrulama geçmişi hiçbir zaman örtük otorite sıralamasına dönüşmez.

#### Kabul Kriterleri

1. WHEN herhangi bir parity artifact'ı `route_request` aracılığıyla sunulduğunda, THE Proofd SHALL yanıt gövdesinde şu yasak itibar alanlarının hiçbirini içermez: `verifier_score`, `trust_score`, `reliability_index`, `weighted_authority`, `correctness_rate`, `agreement_ratio`, `node_success_ratio`, `verifier_reputation`.
2. THE Proofd SHALL sentetik parity artifact'larını geçici dizine yazıp `route_request` aracılığıyla sunarak bu kısıtlamayı doğrular.
3. IF herhangi bir yasak itibar alanı herhangi bir parity artifact yanıtında tespit edilirse, THEN THE Gate SHALL başarısız olur.
4. THE Proofd SHALL bu yasağı tüm parity artifact türleri için uygular: `parity_report.json`, `parity_convergence_report.json`, `parity_drift_attribution_report.json`, `parity_authority_suppression_report.json`, `parity_authority_drift_topology.json`, `parity_incident_graph.json`, `parity_consistency_report.json`, `parity_determinism_report.json`.

### Gereksinim 5: Negatif Matris Kapsamı

**Kullanıcı Hikayesi:** Bir doğrulama operatörü olarak, kill-switch kapılarının `PHASE13_NEGATIVE_TEST_SPEC.md` içinde tanımlanan negatif test matrisini tam olarak kapsamasını istiyorum; böylece tüm bilinen drift vektörleri otomatik olarak tespit edilir.

#### Kabul Kriterleri

1. THE Kill_Switch_Gate kapıları SHALL `P13-NEG-01`, `P13-NEG-02`, `P13-NEG-03`, `P13-NEG-04` vakalarını kapsar (Gereksinim 1 aracılığıyla): observability yollarına POST → 405; desteklenmeyen sorgu parametreleri → 400.
2. THE Kill_Switch_Gate kapıları SHALL `P13-NEG-13`, `P13-NEG-14` vakalarını kapsar (Gereksinim 1 aracılığıyla): yanıt gövdelerinde gizli konsensüs çıktıları veya kontrol düzlemi ipuçları bulunmaz.
3. THE Kill_Switch_Gate kapıları SHALL `P13-FEED-01`, `P13-FEED-02`, `P13-FEED-03`, `P13-FEED-04`, `P13-FEED-05` vakalarını kapsar (Gereksinim 2 aracılığıyla): yönlendirme kodu gözlemlenebilirlik alanlarını tüketmez.
4. THE Kill_Switch_Gate kapıları SHALL `P13-NEG-07`, `P13-NEG-08`, `P13-NEG-09`, `P13-NEG-10` vakalarını kapsar (Gereksinim 3 aracılığıyla): yakınsama artifact'larında seçim alanları bulunmaz.
5. THE Kill_Switch_Gate kapıları SHALL `P13-NEG-15`, `P13-NEG-16` vakalarını kapsar (Gereksinim 4 aracılığıyla): parity artifact'larında itibar veya puanlama alanları bulunmaz.

### Gereksinim 5b: Artifact Passthrough Bütünlüğü

**Kullanıcı Hikayesi:** Bir sistem mimarı olarak, `proofd` diagnostics servisinin artifact içeriğini hiçbir şekilde değiştirmemesini, yorumlamamasını, toparlamamasını, oy vermemesini veya sıralamamasını istiyorum; böylece diagnostics yüzeyi gerçek anlamda salt okunur bir artifact geçişi olarak kalır.

#### Kabul Kriterleri

1. WHEN herhangi bir güvenli JSON artifact `GET /diagnostics/convergence` aracılığıyla sunulduğunda, THE Proofd SHALL yanıt gövdesini yazılan artifact ile birebir aynı JSON değeri olarak döndürür.
2. THE Proofd SHALL artifact içeriğine hiçbir alan eklemez, hiçbir alanı kaldırmaz ve hiçbir değeri dönüştürmez.
3. THE Proofd SHALL artifact içeriğini yorumlamaz, toparlamamaz, oy vermez veya sıralamaz.
4. Bu özellik `prop6_artifact_passthrough_integrity` property testi ile doğrulanır.

### Gereksinim 6: Uygulama Kısıtlamaları

**Kullanıcı Hikayesi:** Bir geliştirici olarak, tüm kill-switch kapı testlerinin mevcut `proofd` crate'i içinde uygulanmasını istiyorum; böylece yeni bağımlılıklar veya dosyalar eklenmez ve mimari sınırlar temiz kalır.

#### Kabul Kriterleri

1. THE Kill_Switch_Gate testleri SHALL yalnızca `userspace/proofd/src/lib.rs` içinde uygulanır; yeni dosya veya crate bağımlılığı eklenmez.
2. THE Kill_Switch_Gate testleri SHALL `proptest` kütüphanesini kullanır (zaten dev-dependency olarak mevcuttur).
3. THE Kill_Switch_Gate property testleri SHALL `ProptestConfig::with_cases(100)` veya daha az iterasyon ile çalışır.
4. THE Kill_Switch_Gate birim testleri SHALL `mod tests_kill_switch_gates` modülü içinde yer alır.
5. THE Kill_Switch_Gate property testleri SHALL `mod proptest_kill_switch_gates` modülü içinde yer alır.
6. THE Kill_Switch_Gate testleri SHALL `cargo test --manifest-path userspace/proofd/Cargo.toml` komutuyla çalıştırılır.
7. THE Source_Scan testi (Kapı 2) SHALL deterministik bir birim testi olarak uygulanır; property testi değildir.

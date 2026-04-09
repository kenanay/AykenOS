# Phase-14 Gözlemlenebilirlik Sınırı Kanıtı

**Belge Türü:** Biçimsel Sınır Kanıtı  
**Faz:** Phase-14 — Distributed Observability Hardening  
**Durum:** KAPALI — ci-freeze#23989067554, ci-freeze#23999026616  
**Otorite:** `ARCHITECTURE_FREEZE.md`

---

## 1. Beş Temel Değişmez

Aşağıdaki tablo, Phase-14 gözlemlenebilirlik sınırının biçimsel değişmezlerini,
her değişmez için somut kanıtı ve sistemdeki uygulama noktasını göstermektedir.

| Değişmez | Kanıt | Uygulama Noktası |
|----------|-------|-----------------|
| `service != authority` | `authority_classification = "non_authoritative"` tüm summary yanıtlarında; `SummaryEpistemicBoundary.produces_truth = false` | `api_schema.rs`: `validate_observability_summary_contract_v1`, `validate_machine_structured_summary_contract_v1` |
| `diagnostics != decision` | `produces_decision = false` her summary yanıtında; diagnostics endpoint'leri hiçbir execution kararı üretmez | `SummaryEpistemicBoundary` struct in `lib.rs`; `summary_epistemic_boundary()` function |
| `parity != consensus` | Graf yüzeyi `aggregation_mode = "overlay_only"`; `majority_cluster`, `winning_cluster`, `selected_partition` yasak alanlar | `CROSS_NODE_OBSERVABILITY_GRAPH_CONTRACT_v1.md`; `ci-gate-convergence-non-election-boundary` |
| `trust does not affect verdict` | Doğrulama sonucu güven puanından bağımsız; `trust_rank`, `verifier_score`, `reliability_index` yasak alanlar | `FORBIDDEN_OBSERVABILITY_FIELDS` in `api_contract.rs`; `ci-gate-verifier-reputation-prohibition` |
| `observability does not imply scheduling` | Diagnostics endpoint'leri `routing_hint`, `execution_override`, `recommended_action`, `node_priority` üretmez | `ci-gate-observability-routing-separation`; `ci-gate-diagnostics-consumer-non-authoritative-contract` |

### 1.1 `service != authority`

proofd bir diagnostics servisidir; otorite kaynağı değildir. Her public
endpoint yanıtı `authority_classification = "non_authoritative"` alanını
taşır. `SummaryEpistemicBoundary.produces_truth = false` beyanı bu değişmezi
makine tarafından doğrulanabilir biçimde kodlar.

`api_schema.rs` içindeki `validate_observability_summary_contract_v1` ve
`validate_machine_structured_summary_contract_v1` fonksiyonları, bu alanın
her yanıtta mevcut ve doğru değerde olduğunu şema düzeyinde zorunlu kılar.

### 1.2 `diagnostics != decision`

Diagnostics endpoint'leri gözlem verisi sunar; execution kararı üretmez.
`produces_decision = false` beyanı `SummaryEpistemicBoundary` struct'ında
sabit olarak tanımlanmıştır. `lib.rs` içindeki `summary_epistemic_boundary()`
fonksiyonu bu struct'ı her summary yanıtına enjekte eder.

### 1.3 `parity != consensus`

Çapraz düğüm parity gözlemi, konsensüs mekanizması değildir. Graf yüzeyi
yalnızca `aggregation_mode = "overlay_only"` modunda çalışır. Çoğunluk
semantiği içeren alanlar (`majority_cluster`, `winning_cluster`,
`selected_partition`) `FORBIDDEN_OBSERVABILITY_FIELDS` listesinde yer alır
ve `ci-gate-convergence-non-election-boundary` kapısı tarafından derleme
zamanında engellenir.

### 1.4 `trust does not affect verdict`

Doğrulama sonucu, doğrulayıcının güven puanından bağımsız olarak hesaplanır.
`trust_rank`, `verifier_score`, `reliability_index` alanları yasak alan
listesinde bulunur. Bu değişmez `ci-gate-verifier-reputation-prohibition`
kapısı tarafından doğrulanır.

### 1.5 `observability does not imply scheduling`

Gözlemlenebilirlik verisi zamanlama kararlarını etkilemez. Diagnostics
endpoint'leri `routing_hint`, `execution_override`, `recommended_action`,
`node_priority` alanlarını hiçbir koşulda üretemez. Bu ayrım
`ci-gate-observability-routing-separation` ve
`ci-gate-diagnostics-consumer-non-authoritative-contract` kapıları tarafından
zorunlu kılınır.

---

## 2. Epistemic Sınır Beyanı

Her summary yanıtı aşağıdaki yapıyı içerir:

```rust
SummaryEpistemicBoundary {
    produces_truth: false,
    produces_decision: false,
    produces_ranking: false,
}
```

Bu yapı `lib.rs` içinde tanımlanmıştır ve `api_schema.rs` içindeki
`validate_summary_epistemic_boundary()` fonksiyonu tarafından her yanıtta
zorunlu kılınır. Üç alanın tamamı `false` olmak zorundadır; herhangi birinin
`true` olması şema doğrulamasını başarısız kılar ve yanıt istemciye iletilmez.

Bu beyan aşağıdaki endpoint'lerin her yanıtında bulunur:

- `GET /diagnostics/summary`
- `GET /diagnostics/runs/{run_id}/summary`

---

## 3. Yasak Alan Listesi

`userspace/proofd/src/api_contract.rs` içindeki `FORBIDDEN_OBSERVABILITY_FIELDS`
sabiti 34 giriş içerir. Alanlar iki kategoride gruplandırılmıştır:

### Kategori 1 — Truth/Election Semantiği (P13-NEG-13)

Bu alanlar seçim, konsensüs veya kesin doğru belirleme semantiği taşır;
gözlemlenebilirlik yüzeyinde hiçbir koşulda yer alamaz:

`winner`, `winningpartition`, `resolvedtruth`, `selectedtruth`, `elect`,
`committedcluster`, `acceptedauthority`, `resolvetruth`, `selectwinner`,
`winningverdict`, `acceptauthority`

### Kategori 2 — Execution/Scheduling Semantiği (P13-NEG-14)

Bu alanlar zamanlama, yönlendirme veya execution kararı semantiği taşır;
diagnostics endpoint'lerinde bulunmaları `observability does not imply scheduling`
değişmezini ihlal eder:

`score`, `routinghint`, `executionoverride`, `recommendedaction`,
`recommendedactions`, `preferrednode`, `preferredverifier`, `trustranking`,
`priority`, `nodepriority`, `verificationweight`, `retry`, `override`,
`promote`, `commit`, `mitigation`, `forceaccept`, `quarantine`,
`autoquarantine`, `autorecovery`, `suppressnode`, `triggerreplayadmission`,
`commitclusterstate`

---

## 4. Çalışma Zamanı Uygulama Mekanizması

Yasak alan listesi yalnızca statik bir referans değildir; çalışma zamanında
her yanıt üzerinde aktif olarak uygulanır.

`lib.rs` içindeki `observability_json_response()` fonksiyonu, her yanıtı
istemciye iletmeden önce `scan_forbidden_observability_fields()` fonksiyonunu
çağırır. Bu tarama JSON yanıtının tüm anahtar adlarını
`FORBIDDEN_OBSERVABILITY_FIELDS` listesiyle karşılaştırır.

**Yasak alan tespit edilirse:**

- HTTP yanıt kodu: `500`
- Hata kodu: `forbidden_observability_field_exposed`
- Yanıt istemciye iletilmez (fail-closed davranışı)

Bu mekanizma fail-closed tasarım ilkesini uygular: şüpheli durumda yanıt
engellenir, geçirilmez. `ci-gate-proofd-observability-boundary` kapısı bu
davranışı entegrasyon testleriyle doğrular.

---

## 5. Gate PASS Referansları

Aşağıdaki tablo, Phase-14 gözlemlenebilirlik sınırını doğrulayan CI kapılarını
ve bunların PASS durumunu göstermektedir:

| Kapı | Amaç | Durum |
|------|------|-------|
| `ci-gate-proofd-observability-boundary` | GET endpoint read-only, POST 405, forbidden field scan | PASS (`ci-freeze#23989067554`) |
| `ci-gate-observability-routing-separation` | No routing/scheduling semantics in observability artifacts | PASS (`ci-freeze#23989067554`) |
| `ci-gate-graph-non-authoritative-contract` | Graph surface non-authoritative | PASS (`ci-freeze#23999026616`) |
| `ci-gate-diagnostics-consumer-non-authoritative-contract` | Consumer non-authoritative | PASS (`ci-freeze#23999026616`) |
| `ci-gate-convergence-non-election-boundary` | No election semantics in convergence | PASS |
| `ci-gate-verifier-reputation-prohibition` | No reputation/scoring fields | PASS |

Tüm kapılar PASS durumundadır. Herhangi bir kapının FAIL durumuna geçmesi
bu kanıtı geçersiz kılar ve Phase-14 kapanışını engeller.

---

## 6. Kanıt Özeti

Bu belge, Phase-14 gözlemlenebilirlik sınırının beş temel değişmezinin
aşağıdaki mekanizmalar aracılığıyla uygulandığını kanıtlar:

1. **Şema düzeyinde zorunluluk** — `api_schema.rs` doğrulama fonksiyonları
2. **Struct düzeyinde sabit beyan** — `SummaryEpistemicBoundary` in `lib.rs`
3. **Çalışma zamanı tarama** — `scan_forbidden_observability_fields()` her yanıtta
4. **CI kapı doğrulaması** — altı kapı, iki ayrı ci-freeze çalışmasında PASS
5. **Sözleşme belgesi** — `CROSS_NODE_OBSERVABILITY_GRAPH_CONTRACT_v1.md`

Bu kanıt `closure_index.json` tarafından referans alınır ve Phase-14 resmi
kapanışının zorunlu bir bileşenidir.

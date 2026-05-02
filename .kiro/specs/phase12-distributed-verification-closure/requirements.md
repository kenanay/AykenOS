# Gereksinimler Belgesi

## Giriş

Phase 12 (Distributed Verification) kapsamında P12-01 ile P12-13 arası görevler local olarak
tamamlanmıştır. Ancak `ci-kill-switch-phase13` hedefinin çalıştırılabilmesi için gerekli olan
12 CI gate henüz implement edilmemiş; `ci-gate-hygiene` çıktısında `coverage: INCOMPLETE` hatası
üretilmektedir.

Bu spec, eksik 12 gate'in `scripts/ci/` ve `tools/ci/` altındaki shell/Python implementasyonlarını
tamamlayarak `ci-kill-switch-phase13` hedefinin `PASS` vermesini ve Phase 12'nin resmi closure
sürecine girebilmesini sağlar.

Temel mimari kural değişmez:

```
proofd = verification execution service + diagnostics service surface
proofd != authority surface
parity semantics = "distributed verification diagnostics" (consensus değil)
```

Tüm değişiklikler `userspace/proofd`, `scripts/ci/` ve `tools/ci/` kapsamında kalır.
Yeni crate bağımlılığı eklenmez.

## Sözlük

- **Proofd**: `userspace/proofd` Rust servisi; bundle doğrulaması yürütür ve salt okunur
  diagnostics sunar.
- **Kill_Switch_Gate**: `ci-kill-switch-phase13` Makefile hedefi altında tanımlı 12 CI gate'in
  tamamı; Phase 12 closure için zorunludur.
- **Coverage_Status**: `kill_switch_summary.json` içindeki `coverage.coverage_status` alanı;
  tüm kill-switch gate'leri PASS verdiğinde `"COMPLETE"` olur.
- **Gate_Script**: `scripts/ci/gate_<name>.sh` formatındaki shell script; ilgili Python
  validator'ı çağırır ve evidence artifact'larını üretir.
- **Validator**: `tools/ci/validate_<name>.py` formatındaki Python scripti; artifact'ları
  doğrular ve `report.json` + detay raporu üretir.
- **Artifact_Root**: Gate'lerin doğrulama için kullandığı artifact dizini; genellikle
  `cross-node-parity` harness çıktısı veya `proofd-service` gate çıktısıdır.
- **Phase12_Gate_Harness**: `ayken-core/crates/proof-verifier/examples/phase12_gate_harness.rs`
  içindeki Rust örneği; `cross-node-parity` modunda test artifact'ları üretir.
- **Proofd_Gate_Harness**: `userspace/proofd/examples/proofd_gate_harness.rs` içindeki Rust
  örneği; `service-contract` modunda proofd endpoint'lerini test eder.
- **Non_Authoritative_Contract**: Diagnostics endpoint'lerinin authority, consensus veya
  scheduling semantiği içermemesi kuralı; `proofd != authority surface` değişmezinin CI
  enforcement katmanı.
- **Parity_Semantics**: Distributed verification diagnostics; consensus veya election semantiği
  değil, tanımlayıcı (descriptive) parity analizi.
- **Forbidden_Fields**: Gate'lerin artifact'larda bulunmasını yasakladığı alan adları kümesi;
  reputation, scoring, routing, election semantiği taşıyan alanlar.
- **Evidence_Dir**: Her gate'in çıktılarını yazdığı dizin; `evidence/run-<id>/gates/<gate-name>/`.
- **Proof_Receipt**: `POST /verify/bundle` çağrısının ürettiği imzalı doğrulama makbuzu;
  `receipts/signed_verification_receipt.json` artifact'ı.
- **Verdict_Binding**: Proof receipt içindeki verdict alanının doğrulama sonucuyla bağlı olması
  kuralı; receipt verdict'inin bağımsız olarak doğrulanabilmesi.

## Gereksinimler

### Gereksinim 1: ci-gate-proof-receipt

**Kullanıcı Hikayesi:** Bir CI operatörü olarak, `ci-gate-proof-receipt` gate'inin
`scripts/ci/gate_proof_receipt.sh` üzerinden çalışmasını ve imzalı doğrulama makbuzunun
şema uyumluluğunu doğrulamasını istiyorum; böylece Phase 12 kill-switch coverage'ı tamamlanır.

#### Kabul Kriterleri

1. WHEN `ci-gate-proof-receipt` hedefi çalıştırıldığında, THE Gate_Script SHALL
   `scripts/ci/gate_proof_receipt.sh --evidence-dir <dir>` komutunu çalıştırır ve
   `report.json` ile `receipt_emit_report.json` artifact'larını üretir.
2. WHEN `gate_proof_receipt.sh` çalıştırıldığında, THE Gate_Script SHALL
   `Phase12_Gate_Harness`'ı `cross-node-parity` modunda çağırarak test artifact'larını
   bootstrap eder.
3. WHEN `receipts/signed_verification_receipt.json` artifact'ı mevcutsa, THE Validator SHALL
   receipt şemasının zorunlu alanlarını (`run_id`, `verdict`, `bundle_hash`,
   `verified_at_utc`, `verifier_node_id`) doğrular.
4. WHEN `receipts/signed_verification_receipt.json` artifact'ı mevcut değilse, THE Validator
   SHALL `receipt_not_emitted` durumunu `SKIP` olarak raporlar; gate FAIL vermez.
5. IF receipt artifact'ı geçersiz JSON içeriyorsa, THEN THE Validator SHALL
   `invalid_json:signed_verification_receipt.json` ihlali üretir ve gate FAIL verir.
6. THE Gate_Script SHALL `report.json` içinde `"gate": "proof-receipt"` ve
   `"verdict": "PASS"|"FAIL"` alanlarını üretir.
7. THE Gate_Script SHALL NON_OVERRIDABLE kurallarından hiçbirini ihlal etmez;
   özellikle `CONSTITUTIONAL.ENFORCEMENT.BYPASS` yasağına uyar.

### Gereksinim 2: ci-gate-proof-verdict-binding

**Kullanıcı Hikayesi:** Bir CI operatörü olarak, `ci-gate-proof-verdict-binding` gate'inin
proof receipt içindeki verdict alanının doğrulama sonucuyla bağlı olduğunu doğrulamasını
istiyorum; böylece receipt'in bağımsız olarak doğrulanabilirliği garanti altına alınır.

#### Kabul Kriterleri

1. WHEN `ci-gate-proof-verdict-binding` hedefi çalıştırıldığında, THE Gate_Script SHALL
   `scripts/ci/gate_proof_verdict_binding.sh --evidence-dir <dir>` komutunu çalıştırır ve
   `report.json` ile `verdict_binding_report.json` artifact'larını üretir.
2. WHEN `proofd_run_manifest.json` ve `receipts/signed_verification_receipt.json` her ikisi
   de mevcutsa, THE Validator SHALL receipt içindeki `verdict` alanının manifest içindeki
   `verdict` alanıyla eşleştiğini doğrular.
3. WHEN verdict alanları eşleşmiyorsa, THE Validator SHALL
   `verdict_binding_mismatch:manifest_verdict:<v1>:receipt_verdict:<v2>` ihlali üretir.
4. WHEN receipt artifact'ı mevcut değilse, THE Validator SHALL `receipt_absent` durumunu
   `SKIP` olarak raporlar; gate FAIL vermez.
5. THE Validator SHALL receipt içinde `preferred_verifier`, `winning_verifier`,
   `trust_rank`, `verifier_score` gibi Forbidden_Fields bulunması durumunda
   `forbidden_field:<field>` ihlali üretir.
6. THE Gate_Script SHALL `report.json` içinde `"gate": "proof-verdict-binding"` ve
   `"verdict": "PASS"|"FAIL"` alanlarını üretir.

### Gereksinim 3: ci-gate-verifier-authority-resolution

**Kullanıcı Hikayesi:** Bir CI operatörü olarak, `ci-gate-verifier-authority-resolution`
gate'inin `proofd` servisinin authority resolution yapmadığını doğrulamasını istiyorum;
böylece `verification != authority` değişmezi CI katmanında enforce edilir.

#### Kabul Kriterleri

1. WHEN `ci-gate-verifier-authority-resolution` hedefi çalıştırıldığında, THE Gate_Script
   SHALL `scripts/ci/gate_verifier_authority_resolution.sh --evidence-dir <dir>` komutunu
   çalıştırır ve `report.json` ile `authority_resolution_report.json` artifact'larını üretir.
2. WHEN `Artifact_Root` içindeki parity ve federation artifact'ları taranırken, THE Validator
   SHALL `preferred_verifier`, `winning_verifier`, `trust_rank`, `weighted_authority`,
   `recommended_action`, `routing_hint`, `execution_override` alanlarının bulunmadığını
   doğrular.
3. WHEN herhangi bir artifact'ta authority resolution semantiği taşıyan bir alan bulunursa,
   THE Validator SHALL `forbidden_authority_field:<artifact>:<path>:<field>` ihlali üretir.
4. WHEN `Artifact_Root` içinde gerekli artifact'lardan herhangi biri eksikse, THE Validator
   SHALL `missing_required_artifact:<name>` ihlali üretir.
5. THE Gate_Script SHALL `report.json` içinde `"gate": "verifier-authority-resolution"` ve
   `"verdict": "PASS"|"FAIL"` alanlarını üretir.
6. THE Validator SHALL `verification != authority` değişmezini; `proofd` servisinin
   authority election, trust election veya consensus semantiği içermediğini doğrular.

### Gereksinim 4: ci-gate-cross-node-parity

**Kullanıcı Hikayesi:** Bir CI operatörü olarak, `ci-gate-cross-node-parity` gate'inin
birden fazla bağımsız verifier node'unun parity artifact'larını doğrulamasını istiyorum;
böylece distributed verification diagnostics'in tutarlılığı garanti altına alınır.

#### Kabul Kriterleri

1. WHEN `ci-gate-cross-node-parity` hedefi çalıştırıldığında, THE Gate_Script SHALL
   `scripts/ci/gate_cross_node_parity.sh --evidence-dir <dir>` komutunu çalıştırır ve
   `report.json`, `parity_report.json`, `parity_closure_audit_report.json` artifact'larını
   üretir.
2. WHEN `Phase12_Gate_Harness` `cross-node-parity` modunda çalıştırıldığında, THE Harness
   SHALL en az iki farklı verifier node'u simüle eden parity artifact'larını üretir.
3. WHEN parity artifact'ları taranırken, THE Validator SHALL `parity != consensus`
   değişmezini doğrular; `winning_cluster`, `selected_partition`, `majority_cluster`,
   `cluster_commit` gibi election semantiği taşıyan alanların bulunmadığını kontrol eder.
4. WHEN `parity_convergence_report.json` içindeki `global_status` alanı
   `ALLOWED_GLOBAL_STATUSES` kümesi dışında bir değer içeriyorsa, THE Validator SHALL
   `invalid_global_status:<value>` ihlali üretir.
5. WHEN `parity_closure_audit_report.json` üretilirken, THE Gate_Script SHALL parity
   closure audit'in salt okunur ve tanımlayıcı olduğunu; herhangi bir execution kararı
   içermediğini doğrular.
6. THE Gate_Script SHALL `report.json` içinde `"gate": "cross-node-parity"` ve
   `"verdict": "PASS"|"FAIL"` alanlarını üretir.

### Gereksinim 5: ci-gate-proofd-service

**Kullanıcı Hikayesi:** Bir CI operatörü olarak, `ci-gate-proofd-service` gate'inin
`proofd` servisinin endpoint sözleşmesini, receipt üretimini ve tekrarlı çalıştırma
determinizmini doğrulamasını istiyorum.

#### Kabul Kriterleri

1. WHEN `ci-gate-proofd-service` hedefi çalıştırıldığında, THE Gate_Script SHALL
   `scripts/ci/gate_proofd_service.sh --evidence-dir <dir>` komutunu çalıştırır ve
   `proofd_service_report.json`, `proofd_receipt_report.json`,
   `proofd_endpoint_contract.json`, `proofd_verify_request.json`,
   `proofd_verify_response.json`, `proofd_run_manifest.json`,
   `proofd_receipt_verification_report.json`, `proofd_repeated_execution_report.json`
   artifact'larını üretir.
2. WHEN `Proofd_Gate_Harness` `service-contract` modunda çalıştırıldığında, THE Harness
   SHALL `POST /verify/bundle` endpoint'ini çağırır ve yanıt şemasını doğrular.
3. WHEN `POST /verify/bundle` yanıtı incelenirken, THE Validator SHALL yanıt gövdesinde
   `run_id`, `status`, `verdict`, `verdict_subject`, `receipt_emitted`,
   `request_fingerprint`, `findings_count` alanlarının bulunduğunu doğrular.
4. WHEN `POST /verify/bundle` yanıtı incelenirken, THE Validator SHALL Forbidden_Fields
   kümesindeki (`preferred_verifier`, `winning_verifier`, `trust_rank`, `verifier_score`,
   `trust_score`, `reliability_index`, `weighted_authority`, `correctness_rate`,
   `agreement_ratio`, `node_success_ratio`, `verifier_reputation`, `recommended_action`,
   `routing_hint`, `execution_override`, `retry`, `override`, `promote`, `commit`,
   `mitigation`, `node_priority`, `verification_weight`) alanların bulunmadığını doğrular.
5. WHEN aynı istek parametreleriyle iki kez `POST /verify/bundle` çağrıldığında, THE
   Validator SHALL her iki çalıştırmanın özdeş `request_fingerprint` ürettiğini doğrular.
6. THE Gate_Script SHALL `report.json` içinde `"gate": "proofd-service"` ve
   `"verdict": "PASS"|"FAIL"` alanlarını üretir.

### Gereksinim 6: ci-gate-proofd-observability-boundary

**Kullanıcı Hikayesi:** Bir CI operatörü olarak, `ci-gate-proofd-observability-boundary`
gate'inin `proofd` diagnostics endpoint'lerinin kesinlikle salt okunur kaldığını ve
`observability != scheduling` değişmezini doğrulamasını istiyorum.

#### Kabul Kriterleri

1. WHEN `ci-gate-proofd-observability-boundary` hedefi çalıştırıldığında, THE Gate_Script
   SHALL `scripts/ci/gate_proofd_observability_boundary.sh --evidence-dir <dir>` komutunu
   çalıştırır ve `proofd_observability_boundary_report.json`,
   `proofd_observability_negative_matrix.json` artifact'larını üretir.
2. WHEN `GET /diagnostics/runs/{run_id}/artifacts` endpoint'i çağrıldığında, THE Validator
   SHALL yanıtın herhangi bir artifact yazmadığını doğrular; dosya sistemi değişmezliğini
   kontrol eder.
3. WHEN `POST /diagnostics/runs/{run_id}/artifacts` isteği gönderildiğinde, THE Validator
   SHALL HTTP 405 `method_not_allowed` yanıtı alındığını doğrular.
4. WHEN `GET /diagnostics/runs/{run_id}/federation` endpoint'i çağrıldığında, THE Validator
   SHALL yanıt gövdesinde `routing_hint`, `execution_override`, `recommended_action`,
   `node_priority` gibi scheduling semantiği taşıyan alanların bulunmadığını doğrular.
5. WHEN `proofd_observability_negative_matrix.json` üretilirken, THE Gate_Script SHALL
   tüm `POST /diagnostics/...` endpoint'lerine yapılan isteklerin 405 döndürdüğünü
   matris formatında raporlar.
6. THE Gate_Script SHALL `report.json` içinde `"gate": "proofd-observability-boundary"` ve
   `"verdict": "PASS"|"FAIL"` alanlarını üretir.

### Gereksinim 7: ci-gate-graph-non-authoritative-contract

**Kullanıcı Hikayesi:** Bir CI operatörü olarak, `ci-gate-graph-non-authoritative-contract`
gate'inin `parity_incident_graph.json` artifact'ının authority veya election semantiği
içermediğini doğrulamasını istiyorum.

#### Kabul Kriterleri

1. WHEN `ci-gate-graph-non-authoritative-contract` hedefi çalıştırıldığında, THE Gate_Script
   SHALL `scripts/ci/gate_graph_non_authoritative_contract.sh --evidence-dir <dir>` komutunu
   çalıştırır ve `report.json` ile `graph_non_authoritative_report.json` artifact'larını
   üretir.
2. WHEN `parity_incident_graph.json` taranırken, THE Validator SHALL graph node ve edge
   alanlarında `preferred_verifier`, `winning_verifier`, `trust_rank`, `routing_hint`,
   `execution_override`, `node_priority`, `verification_weight` gibi Forbidden_Fields
   bulunmadığını doğrular.
3. WHEN `parity_incident_graph.json` taranırken, THE Validator SHALL graph yapısının
   tanımlayıcı (descriptive) olduğunu; herhangi bir execution kararı veya authority
   resolution içermediğini doğrular.
4. WHEN `parity_incident_graph.json` artifact'ı mevcut değilse, THE Validator SHALL
   `missing_required_artifact:parity_incident_graph.json` ihlali üretir.
5. IF `parity_incident_graph.json` geçersiz JSON içeriyorsa, THEN THE Validator SHALL
   `invalid_json:parity_incident_graph.json` ihlali üretir.
6. THE Gate_Script SHALL `report.json` içinde `"gate": "graph-non-authoritative-contract"` ve
   `"verdict": "PASS"|"FAIL"` alanlarını üretir.

### Gereksinim 8: ci-gate-convergence-non-election-boundary

**Kullanıcı Hikayesi:** Bir CI operatörü olarak, `ci-gate-convergence-non-election-boundary`
gate'inin `parity_convergence_report.json` ve `parity_drift_attribution_report.json`
artifact'larının election veya selection semantiği içermediğini doğrulamasını istiyorum.

#### Kabul Kriterleri

1. WHEN `ci-gate-convergence-non-election-boundary` hedefi çalıştırıldığında, THE Gate_Script
   SHALL `scripts/ci/gate_convergence_non_election_boundary.sh --evidence-dir <dir>` komutunu
   çalıştırır ve `report.json` ile `convergence_non_election_report.json` artifact'larını
   üretir.
2. WHEN `parity_convergence_report.json` taranırken, THE Validator SHALL `global_status`
   alanının `N_PARITY_CONSISTENCY_SPLIT`, `N_PARITY_CONVERGED`, `N_PARITY_DETERMINISM_VIOLATION`,
   `N_PARITY_HISTORICAL_ISLAND`, `N_PARITY_INSUFFICIENT_EVIDENCE`, `N_PARITY_MIXED`
   değerlerinden biri olduğunu doğrular.
3. WHEN `parity_convergence_report.json` taranırken, THE Validator SHALL
   `winning_cluster`, `selected_partition`, `majority_cluster`, `cluster_commit`,
   `cluster_truth`, `canonical_cluster`, `preferred_cluster`, `accepted_cluster`,
   `execution_route`, `routing_hint`, `cluster_policy_input` gibi election semantiği
   taşıyan Exact_Forbidden_Fields'ın bulunmadığını doğrular.
4. WHEN `cluster_derivation` alanı mevcutsa, THE Validator SHALL değerinin
   `node_parity_outcome_dk_partitions` olduğunu doğrular.
5. WHEN `edge_match_cluster_derivation` alanı mevcutsa, THE Validator SHALL değerinin
   `pairwise_match_graph_connected_components` olduğunu doğrular.
6. THE Gate_Script SHALL `report.json` içinde `"gate": "convergence-non-election-boundary"` ve
   `"verdict": "PASS"|"FAIL"` alanlarını üretir.
7. THE Validator SHALL mevcut `validate_convergence_non_election_boundary.py` implementasyonunu
   kullanır; bu script zaten `tools/ci/` altında mevcuttur ve gate script tarafından çağrılır.

### Gereksinim 9: ci-gate-diagnostics-consumer-non-authoritative-contract

**Kullanıcı Hikayesi:** Bir CI operatörü olarak,
`ci-gate-diagnostics-consumer-non-authoritative-contract` gate'inin diagnostics
tüketicilerinin (consumer) authority veya scheduling kararı almadığını doğrulamasını
istiyorum; böylece `observability != scheduling` değişmezi consumer katmanında da enforce
edilir.

#### Kabul Kriterleri

1. WHEN `ci-gate-diagnostics-consumer-non-authoritative-contract` hedefi çalıştırıldığında,
   THE Gate_Script SHALL
   `scripts/ci/gate_diagnostics_consumer_non_authoritative_contract.sh --evidence-dir <dir>`
   komutunu çalıştırır ve `report.json` ile
   `diagnostics_consumer_contract_report.json` artifact'larını üretir.
2. WHEN diagnostics consumer artifact'ları taranırken, THE Validator SHALL consumer
   çıktılarında `recommended_action`, `routing_hint`, `execution_override`, `retry`,
   `override`, `promote`, `commit`, `mitigation`, `node_priority` alanlarının
   bulunmadığını doğrular.
3. WHEN `GET /diagnostics/runs/{run_id}/federation` yanıtı incelenirken, THE Validator
   SHALL yanıtın yalnızca tanımlayıcı alanlar içerdiğini; herhangi bir execution kararı
   veya authority resolution içermediğini doğrular.
4. WHEN `GET /diagnostics/runs/{run_id}/artifacts` yanıtı incelenirken, THE Validator
   SHALL artifact listesinin salt okunur olduğunu ve herhangi bir yazma işlemi
   tetiklemediğini doğrular.
5. THE Gate_Script SHALL `report.json` içinde
   `"gate": "diagnostics-consumer-non-authoritative-contract"` ve
   `"verdict": "PASS"|"FAIL"` alanlarını üretir.
6. THE Validator SHALL mevcut
   `validate_diagnostics_consumer_non_authoritative_contract.py` implementasyonunu kullanır.

### Gereksinim 10: ci-gate-diagnostics-callsite-correlation

**Kullanıcı Hikayesi:** Bir CI operatörü olarak,
`ci-gate-diagnostics-callsite-correlation` gate'inin diagnostics endpoint çağrılarının
doğrulama sonuçlarıyla korelasyon içermediğini doğrulamasını istiyorum; böylece
`diagnostics never influence verification result` değişmezi enforce edilir.

#### Kabul Kriterleri

1. WHEN `ci-gate-diagnostics-callsite-correlation` hedefi çalıştırıldığında, THE Gate_Script
   SHALL `scripts/ci/gate_diagnostics_callsite_correlation.sh --evidence-dir <dir>` komutunu
   çalıştırır ve `report.json` ile `diagnostics_callsite_correlation_report.json`
   artifact'larını üretir.
2. WHEN diagnostics callsite'ları analiz edilirken, THE Validator SHALL
   `GET /diagnostics/...` endpoint çağrılarının `POST /verify/bundle` sonucunu
   etkileyecek herhangi bir yan etki üretmediğini doğrular.
3. WHEN diagnostics callsite artifact'ları taranırken, THE Validator SHALL callsite
   çıktılarında `execution_override`, `routing_hint`, `recommended_action`,
   `verification_weight`, `node_priority` alanlarının bulunmadığını doğrular.
4. WHEN `diagnostics_callsite_correlation_report.json` üretilirken, THE Gate_Script SHALL
   her diagnostics endpoint'i için callsite → verification path izolasyonunu raporlar.
5. THE Gate_Script SHALL `report.json` içinde
   `"gate": "diagnostics-callsite-correlation"` ve `"verdict": "PASS"|"FAIL"` alanlarını
   üretir.
6. THE Validator SHALL mevcut `validate_diagnostics_callsite_correlation.py`
   implementasyonunu kullanır.

### Gereksinim 11: ci-gate-observability-routing-separation

**Kullanıcı Hikayesi:** Bir CI operatörü olarak,
`ci-gate-observability-routing-separation` gate'inin observability yüzeyinin routing veya
scheduling kararlarından tamamen ayrı kaldığını doğrulamasını istiyorum; böylece
`observability != scheduling` değişmezi artifact düzeyinde kanıtlanır.

#### Kabul Kriterleri

1. WHEN `ci-gate-observability-routing-separation` hedefi çalıştırıldığında, THE Gate_Script
   SHALL `scripts/ci/gate_observability_routing_separation.sh --evidence-dir <dir>` komutunu
   çalıştırır ve `report.json`, `observability_routing_separation_report.json`,
   `observability_routing_negative_matrix.json` artifact'larını üretir.
2. WHEN observability artifact'ları taranırken, THE Validator SHALL `routing_hint`,
   `execution_override`, `recommended_action`, `node_priority`, `verification_weight`,
   `retry`, `override`, `promote`, `commit`, `mitigation` alanlarının bulunmadığını
   doğrular.
3. WHEN `observability_routing_negative_matrix.json` üretilirken, THE Gate_Script SHALL
   her observability endpoint'i için routing/scheduling semantiği taşıyan alan yokluğunu
   matris formatında raporlar.
4. WHEN herhangi bir observability artifact'ında routing semantiği taşıyan bir alan
   bulunursa, THE Validator SHALL
   `forbidden_routing_field:<artifact>:<path>:<field>` ihlali üretir.
5. THE Gate_Script SHALL `report.json` içinde
   `"gate": "observability-routing-separation"` ve `"verdict": "PASS"|"FAIL"` alanlarını
   üretir.
6. THE Validator SHALL mevcut `validate_observability_routing_separation.py`
   implementasyonunu kullanır.

### Gereksinim 12: ci-gate-verifier-reputation-prohibition

**Kullanıcı Hikayesi:** Bir CI operatörü olarak,
`ci-gate-verifier-reputation-prohibition` gate'inin tüm parity ve diagnostics
artifact'larında verifier reputation veya scoring semantiği bulunmadığını doğrulamasını
istiyorum; böylece `verification history != verifier reputation` değişmezi enforce edilir.

#### Kabul Kriterleri

1. WHEN `ci-gate-verifier-reputation-prohibition` hedefi çalıştırıldığında, THE Gate_Script
   SHALL `scripts/ci/gate_verifier_reputation_prohibition.sh --evidence-dir <dir>` komutunu
   çalıştırır ve `report.json` ile `reputation_prohibition_report.json` artifact'larını
   üretir.
2. WHEN `parity_report.json`, `parity_determinism_incidents.json`,
   `parity_drift_attribution_report.json`, `parity_convergence_report.json`,
   `parity_authority_drift_topology.json`, `parity_authority_suppression_report.json`,
   `parity_incident_graph.json` artifact'ları taranırken, THE Validator SHALL
   `agreement_ratio`, `authority_alignment_score`, `convergence_leadership_score`,
   `correctness_rate`, `dominant_verifier_frequency`, `historical_correctness_index`,
   `node_success_ratio`, `node_trust_score`, `reliability_index`, `trust_score`,
   `verifier_reputation`, `verifier_score`, `weighted_authority` alanlarının
   bulunmadığını doğrular.
3. WHEN herhangi bir artifact'ta reputation pattern'ı (`reputation`, `reliability`,
   `verifier.*correctness`, `weighted.*authority`, `verifier.*score`) eşleşen bir alan
   bulunursa, THE Validator SHALL
   `forbidden_reputation_field:<artifact>:<path>:<field>:<rule>` ihlali üretir.
4. WHEN `Artifact_Root` içinde gerekli artifact'lardan herhangi biri eksikse, THE Validator
   SHALL `missing_required_artifact:<name>` ihlali üretir.
5. THE Gate_Script SHALL `report.json` içinde
   `"gate": "verifier-reputation-prohibition"` ve `"verdict": "PASS"|"FAIL"` alanlarını
   üretir.
6. THE Validator SHALL mevcut `validate_verifier_reputation_prohibition.py`
   implementasyonunu kullanır; bu script zaten `tools/ci/` altında mevcuttur.

### Gereksinim 13: Kill-Switch Coverage Tamamlanması

**Kullanıcı Hikayesi:** Bir CI operatörü olarak, `ci-kill-switch-phase13` hedefinin tüm
12 gate'i PASS verdikten sonra `kill_switch_summary.json` içindeki
`coverage.coverage_status` alanının `"COMPLETE"` olmasını istiyorum; böylece Phase 12
resmi closure sürecine girebilir.

#### Kabul Kriterleri

1. WHEN `ci-kill-switch-phase13` hedefi çalıştırıldığında ve tüm 12 gate PASS verdiğinde,
   THE CI_System SHALL `ci-kill-switch-summary` hedefini çalıştırır ve
   `kill_switch_summary.json` içindeki `coverage.coverage_status` alanının `"COMPLETE"`
   olduğunu doğrular.
2. WHEN `ci-kill-switch-summary` hedefi çalıştırıldığında, THE CI_System SHALL
   `tools/ci/summarize.sh --run-dir <dir> --require-kill-switch-completeness` komutunu
   çalıştırır.
3. WHEN herhangi bir kill-switch gate FAIL verirse, THE CI_System SHALL
   `coverage.coverage_status` alanını `"INCOMPLETE"` olarak raporlar ve
   `ci-kill-switch-summary` hedefi exit code 2 ile sonlanır.
4. THE CI_System SHALL `ci-kill-switch-phase13` hedefinin başarıyla tamamlanmasını
   `phase12-official-closure-prep` hedefinin ön koşulu olarak değerlendirir.
5. THE CI_System SHALL NON_OVERRIDABLE kurallarından hiçbirini ihlal etmez; özellikle
   `CONSTITUTIONAL.ENFORCEMENT.BYPASS` yasağına uyar — kill-switch gate'lerinin
   devre dışı bırakılması veya atlanması kesinlikle yasaktır.
6. FOR ALL 12 kill-switch gate'i, THE Gate_Script SHALL `report.json` içinde
   `"verdict": "PASS"` ürettiğinde ve yalnızca o zaman `coverage_status` `"COMPLETE"`
   olur; kısmi PASS kabul edilmez.

### Gereksinim 14: Uygulama Kısıtlamaları ve Constitutional Uyum

**Kullanıcı Hikayesi:** Bir geliştirici olarak, tüm gate implementasyonlarının mevcut
`scripts/ci/` ve `tools/ci/` yapısına uymasını ve NON_OVERRIDABLE constitutional
kurallarını ihlal etmemesini istiyorum.

#### Kabul Kriterleri

1. THE Gate_Script SHALL tüm implementasyonları yalnızca `scripts/ci/` ve `tools/ci/`
   dizinleri içinde tutar; yeni dizin yapısı oluşturulmaz.
2. THE Gate_Script SHALL mevcut `Phase12_Gate_Harness` ve `Proofd_Gate_Harness` Rust
   örneklerini kullanır; yeni crate bağımlılığı eklenmez.
3. THE Gate_Script SHALL `set -euo pipefail` ile başlar ve hata durumunda fail-closed
   davranır; `CONSTITUTIONAL.ENFORCEMENT.BYPASS` yasağına uyar.
4. THE Validator SHALL `DETERMINISM.GLOBAL` kuralına uyar; global state mutasyonu
   içermez ve deterministik çıktı üretir.
5. THE Validator SHALL `MEMORY.CONTRACT.VIOLATION` kuralına uyar; Python implementasyonu
   bellek güvenliği ihlali içermez.
6. WHEN herhangi bir gate script `exit 3` ile sonlanırsa, THE CI_System SHALL bunu
   tooling/usage hatası olarak değerlendirir; gate FAIL sayılmaz ancak CI pipeline
   durdurulur.
7. THE Gate_Script SHALL `cargo test --manifest-path userspace/proofd/Cargo.toml`
   komutuyla çalıştırılabilir proofd testlerini bozmaz.
8. THE Gate_Script SHALL `SECURITY.INFORMATION.LEAK` kuralına uyar; evidence artifact'ları
   private key, credential veya PII içermez.

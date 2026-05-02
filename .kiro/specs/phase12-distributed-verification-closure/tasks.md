# Uygulama Planı: Phase 12 Distributed Verification Closure

## Genel Bakış

12 CI gate'in `scripts/ci/` ve `tools/ci/` altındaki implementasyonlarını tamamlayarak
`ci-kill-switch-phase13` hedefinin `PASS` vermesini ve `coverage_status: COMPLETE` üretmesini
sağlar. Tüm gate script'leri ve Makefile hedefleri mevcuttur; eksik olan validator
implementasyonları ve harness entegrasyonlarıdır.

Constitutional değişmezler tüm görevlerde geçerlidir:
- `set -euo pipefail` — CONSTITUTIONAL.ENFORCEMENT.BYPASS yasağı
- Global state mutasyonu yok — DETERMINISM.GLOBAL
- Artifact'larda credential/PII yok — SECURITY.INFORMATION.LEAK

## Görevler

- [x] 1. Tip A Gate'leri: Harness-Driven Validator'ları Implement Et
  - [x] 1.1 `validate_proof_receipt_gate.py` validator'ını implement et
    - `tools/ci/validate_proof_receipt_gate.py` dosyasını oluştur
    - `signed_verification_receipt.json` artifact'ının zorunlu alanlarını doğrula:
      `run_id`, `verdict`, `bundle_hash`, `verified_at_utc`, `verifier_node_id`
    - Artifact yoksa `receipt_not_emitted` → `SKIP` döndür (gate FAIL vermez)
    - Geçersiz JSON → `invalid_json:signed_verification_receipt.json` ihlali üret
    - Forbidden fields (`preferred_verifier`, `winning_verifier`, `trust_rank`, `verifier_score`) kontrolü ekle
    - `--artifact-root`, `--out-report`, `--out-detail-report`, `--violations-out` argümanlarını destekle
    - `report.json` içinde `"gate": "proof-receipt"` ve `"verdict": "PASS"|"FAIL"` üret
    - _Gereksinimler: 1.3, 1.4, 1.5, 1.6_

  - [ ]* 1.2 `validate_proof_receipt_gate.py` için özellik testi yaz
    - **Özellik 1: Receipt Şema Doğrulaması** — zorunlu alan eksikse ihlal üretilmeli
    - **Özellik 2: Geçersiz JSON İhlali** — geçersiz JSON → `invalid_json:...` ihlali
    - `test_validate_proof_receipt_gate.py` içine `@given` + `@settings(max_examples=100)` ile ekle
    - _Gereksinimler: 1.3, 1.5_

  - [x] 1.3 `validate_proof_verdict_binding_gate.py` validator'ını implement et
    - `tools/ci/validate_proof_verdict_binding_gate.py` dosyasını oluştur
    - `proofd_run_manifest.json` ve `receipts/signed_verification_receipt.json` her ikisi mevcutsa
      `verdict` alanlarının eşleştiğini doğrula
    - Eşleşmeme → `verdict_binding_mismatch:manifest_verdict:<v1>:receipt_verdict:<v2>` ihlali
    - Receipt yoksa → `receipt_absent` → `SKIP` (gate FAIL vermez)
    - Forbidden fields kontrolü: `preferred_verifier`, `winning_verifier`, `trust_rank`, `verifier_score`
    - `--artifact-root`, `--out-report`, `--out-detail-report`, `--violations-out` argümanlarını destekle
    - `report.json` içinde `"gate": "proof-verdict-binding"` ve `"verdict": "PASS"|"FAIL"` üret
    - _Gereksinimler: 2.2, 2.3, 2.4, 2.5, 2.6_

  - [ ]* 1.4 `validate_proof_verdict_binding_gate.py` için özellik testi yaz
    - **Özellik 3: Verdict Binding Tutarlılığı** — eşleşmeyen verdict → ihlal
    - **Özellik 4: Forbidden Fields Yasağı (Receipt)** — yasak alan → ihlal
    - `test_validate_proof_verdict_binding_gate.py` içine ekle
    - _Gereksinimler: 2.2, 2.3, 2.5_

  - [x] 1.5 `gate_proof_receipt.sh` ile `validate_proof_verdict_binding_gate.py` entegrasyonunu doğrula
    - `gate_proof_receipt.sh` harness çıktısından `receipts/signed_verification_receipt.json` üretildiğini doğrula
    - `gate_proof_verdict_binding.sh` harness çıktısından `proofd_run_manifest.json` üretildiğini doğrula
    - Her iki gate script'inin `report.json` içinde doğru `"gate"` alanını ürettiğini kontrol et
    - _Gereksinimler: 1.1, 1.2, 2.1_

  - [x] 1.6 `validate_verifier_authority_resolution_gate.py` validator'ını implement et
    - `tools/ci/validate_verifier_authority_resolution_gate.py` dosyasını oluştur
    - Parity ve federation artifact'larında authority resolution semantiği taşıyan alanları tara:
      `preferred_verifier`, `winning_verifier`, `trust_rank`, `weighted_authority`,
      `recommended_action`, `routing_hint`, `execution_override`
    - İhlal formatı: `forbidden_authority_field:<artifact>:<path>:<field>`
    - Eksik artifact → `missing_required_artifact:<name>` ihlali
    - `--artifact-root`, `--out-report`, `--out-detail-report`, `--violations-out` argümanlarını destekle
    - `report.json` içinde `"gate": "verifier-authority-resolution"` ve `"verdict": "PASS"|"FAIL"` üret
    - _Gereksinimler: 3.2, 3.3, 3.4, 3.5, 3.6_

  - [ ]* 1.7 `validate_verifier_authority_resolution_gate.py` için özellik testi yaz
    - **Özellik 5: Authority Resolution Forbidden Fields** — yasak alan → ihlal
    - `test_validate_verifier_authority_resolution_gate.py` içine ekle
    - _Gereksinimler: 3.2, 3.6_

  - [x] 1.8 `validate_cross_node_parity_gate.py` validator'ını implement et
    - `tools/ci/validate_cross_node_parity_gate.py` dosyasını oluştur
    - `parity_convergence_report.json` içindeki `global_status` alanının `ALLOWED_GLOBAL_STATUSES`
      kümesinde olduğunu doğrula; değilse `invalid_global_status:<value>` ihlali üret
    - Election semantiği taşıyan alanları tara: `winning_cluster`, `selected_partition`,
      `majority_cluster`, `cluster_commit`; ihlal → `forbidden_election_field:<artifact>:<path>:<field>`
    - `parity_closure_audit_report.json` içinde execution kararı olmadığını doğrula
    - `--artifact-root`, `--out-report`, `--out-detail-report`, `--violations-out` argümanlarını destekle
    - `report.json` içinde `"gate": "cross-node-parity"` ve `"verdict": "PASS"|"FAIL"` üret
    - _Gereksinimler: 4.3, 4.4, 4.5, 4.6_

  - [ ]* 1.9 `validate_cross_node_parity_gate.py` için özellik testi yaz
    - **Özellik 6: Parity Non-Election Değişmezi** — election alanı → ihlal
    - **Özellik 7: Global Status Kısıtlaması** — geçersiz status → `invalid_global_status` ihlali
    - `test_validate_cross_node_parity_gate.py` içine ekle
    - _Gereksinimler: 4.3, 4.4_

- [x] 2. Checkpoint — Tip A gate validator'larını doğrula
  - Tüm testlerin geçtiğini doğrula, sorular varsa kullanıcıya sor.

- [x] 3. Tip B Gate'leri: Proofd-Driven Validator'ları Implement Et
  - [x] 3.1 `validate_proofd_service_gate.py` validator'ını implement et
    - `tools/ci/validate_proofd_service_gate.py` dosyasını oluştur
    - `proofd_verify_response.json` içinde zorunlu alanları doğrula:
      `run_id`, `status`, `verdict`, `verdict_subject`, `receipt_emitted`,
      `request_fingerprint`, `findings_count`
    - Forbidden fields kümesini tara: `preferred_verifier`, `winning_verifier`, `trust_rank`,
      `verifier_score`, `trust_score`, `reliability_index`, `weighted_authority`,
      `recommended_action`, `routing_hint`, `execution_override`, `retry`, `override`,
      `promote`, `commit`, `mitigation`, `node_priority`, `verification_weight`
    - `proofd_repeated_execution_report.json` içinde iki çalıştırmanın özdeş
      `request_fingerprint` ürettiğini doğrula (DETERMINISM.GLOBAL uyumu)
    - `--artifact-root`, `--out-report`, `--out-detail-report`, `--violations-out` argümanlarını destekle
    - `report.json` içinde `"gate": "proofd-service"` ve `"verdict": "PASS"|"FAIL"` üret
    - _Gereksinimler: 5.3, 5.4, 5.5, 5.6_

  - [ ]* 3.2 `validate_proofd_service_gate.py` için özellik testi yaz
    - **Özellik 8: Verify Response Zorunlu Alanları** — eksik alan → ihlal
    - **Özellik 9: Verify Response Forbidden Fields** — yasak alan → ihlal
    - **Özellik 10: Verify Determinizmi** — aynı istek → özdeş `request_fingerprint`
    - `test_validate_proofd_service_gate.py` içine ekle
    - _Gereksinimler: 5.3, 5.4, 5.5_

  - [x] 3.3 `validate_proofd_observability_boundary_gate.py` validator'ını implement et
    - `tools/ci/validate_proofd_observability_boundary_gate.py` dosyasını oluştur
    - `proofd_observability_boundary_report.json` içinde GET endpoint'lerinin dosya sistemi
      değişmezliğini koruduğunu doğrula (read-only değişmezi)
    - `proofd_observability_negative_matrix.json` içinde tüm `POST /diagnostics/...`
      endpoint'lerinin 405 döndürdüğünü doğrula; ihlal → `post_method_not_rejected:<endpoint>`
    - Federation endpoint yanıtında scheduling semantiği taşıyan alanları tara:
      `routing_hint`, `execution_override`, `recommended_action`, `node_priority`
    - `--artifact-root`, `--out-report`, `--out-detail-report`, `--violations-out` argümanlarını destekle
    - `report.json` içinde `"gate": "proofd-observability-boundary"` ve `"verdict": "PASS"|"FAIL"` üret
    - _Gereksinimler: 6.2, 6.3, 6.4, 6.5, 6.6_

  - [ ]* 3.4 `validate_proofd_observability_boundary_gate.py` için özellik testi yaz
    - **Özellik 11: GET Endpoint Read-Only Değişmezi** — GET çağrısı artifact yazmamalı
    - **Özellik 12: POST Diagnostics 405 Zorunluluğu** — POST → 405 yanıtı
    - `test_validate_proofd_observability_boundary_gate.py` içine ekle
    - _Gereksinimler: 6.2, 6.3_

- [x] 4. Checkpoint — Tip B gate validator'larını doğrula
  - Tüm testlerin geçtiğini doğrula, sorular varsa kullanıcıya sor.

- [x] 5. Tip C Gate'leri: Mevcut Artifact-Scan Validator'larını Doğrula ve Tamamla
  - [x] 5.1 `validate_graph_non_authoritative_contract.py` implementasyonunu doğrula
    - `parity_incident_graph.json` artifact'ının varlığını kontrol et; yoksa
      `missing_required_artifact:parity_incident_graph.json` ihlali üretildiğini doğrula
    - Geçersiz JSON → `invalid_json:parity_incident_graph.json` ihlali üretildiğini doğrula
    - Forbidden fields taramasının graph node ve edge alanlarını kapsadığını doğrula:
      `preferred_verifier`, `winning_verifier`, `trust_rank`, `routing_hint`,
      `execution_override`, `node_priority`, `verification_weight`
    - `report.json` içinde `"gate": "graph-non-authoritative-contract"` alanının üretildiğini doğrula
    - _Gereksinimler: 7.2, 7.3, 7.4, 7.5, 7.6_

  - [ ]* 5.2 `validate_graph_non_authoritative_contract.py` için özellik testi yaz
    - **Özellik 13: Graph Non-Authoritative Değişmezi** — yasak alan → ihlal
    - `test_validate_graph_non_authoritative_contract_gate.py` içine ekle
    - _Gereksinimler: 7.2, 7.3_

  - [x] 5.3 `validate_convergence_non_election_boundary.py` implementasyonunu doğrula
    - `global_status` alanının `ALLOWED_GLOBAL_STATUSES` kümesinde olduğunu doğrula
    - `cluster_derivation` → `node_parity_outcome_dk_partitions` değer kontrolünü doğrula
    - `edge_match_cluster_derivation` → `pairwise_match_graph_connected_components` değer kontrolünü doğrula
    - Exact forbidden fields listesinin eksiksiz olduğunu doğrula:
      `winning_cluster`, `selected_partition`, `majority_cluster`, `cluster_commit`,
      `cluster_truth`, `canonical_cluster`, `preferred_cluster`, `accepted_cluster`,
      `execution_route`, `routing_hint`, `cluster_policy_input`
    - `report.json` içinde `"gate": "convergence-non-election-boundary"` alanının üretildiğini doğrula
    - _Gereksinimler: 8.2, 8.3, 8.4, 8.5, 8.6_

  - [ ]* 5.4 `validate_convergence_non_election_boundary.py` için özellik testi yaz
    - **Özellik 6: Parity Non-Election Değişmezi** — election alanı → ihlal
    - **Özellik 7: Global Status Kısıtlaması** — geçersiz status → ihlal
    - `test_validate_convergence_non_election_boundary_gate.py` içine ekle
    - _Gereksinimler: 8.2, 8.3_

  - [x] 5.5 `validate_verifier_reputation_prohibition.py` implementasyonunu doğrula
    - Tüm 7 zorunlu artifact'ın tarandığını doğrula:
      `parity_report.json`, `parity_determinism_incidents.json`,
      `parity_drift_attribution_report.json`, `parity_convergence_report.json`,
      `parity_authority_drift_topology.json`, `parity_authority_suppression_report.json`,
      `parity_incident_graph.json`
    - Exact forbidden fields listesinin eksiksiz olduğunu doğrula (13 alan)
    - Pattern rules'un `reputation`, `reliability`, `verifier.*correctness`,
      `weighted.*authority`, `verifier.*score` pattern'larını kapsadığını doğrula
    - İhlal formatının `forbidden_reputation_field:<artifact>:<path>:<field>:<rule>` olduğunu doğrula
    - `report.json` içinde `"gate": "verifier-reputation-prohibition"` alanının üretildiğini doğrula
    - _Gereksinimler: 12.2, 12.3, 12.4, 12.5, 12.6_

  - [ ]* 5.6 `validate_verifier_reputation_prohibition.py` için özellik testi yaz
    - **Özellik 16: Reputation Prohibition** — reputation/scoring alanı → ihlal
    - `test_validate_verifier_reputation_prohibition_gate.py` içine ekle
    - _Gereksinimler: 12.2, 12.3_

- [x] 6. Tip D Gate'leri: Mevcut Source-Scan Validator'larını Doğrula ve Tamamla
  - [x] 6.1 `validate_diagnostics_consumer_non_authoritative_contract.py` implementasyonunu doğrula
    - Consumer artifact ve kaynak dosyalarında scheduling semantiği taşıyan alanları taradığını doğrula:
      `recommended_action`, `routing_hint`, `execution_override`, `retry`, `override`,
      `promote`, `commit`, `mitigation`, `node_priority`
    - `GET /diagnostics/runs/{run_id}/federation` yanıtının yalnızca tanımlayıcı alanlar içerdiğini
      doğrulayan kontrolün mevcut olduğunu doğrula
    - `GET /diagnostics/runs/{run_id}/artifacts` yanıtının salt okunur olduğunu doğrulayan
      kontrolün mevcut olduğunu doğrula
    - `report.json` içinde `"gate": "diagnostics-consumer-non-authoritative-contract"` alanının
      üretildiğini doğrula
    - _Gereksinimler: 9.2, 9.3, 9.4, 9.5_

  - [ ]* 6.2 `validate_diagnostics_consumer_non_authoritative_contract.py` için özellik testi yaz
    - **Özellik 14: Consumer Non-Authoritative Değişmezi** — scheduling alanı → ihlal
    - `test_validate_diagnostics_consumer_non_authoritative_contract_gate.py` içine ekle
    - _Gereksinimler: 9.2, 9.3_

  - [x] 6.3 `validate_diagnostics_callsite_correlation.py` implementasyonunu doğrula
    - `GET /diagnostics/...` çağrılarından türetilen değişkenlerin `POST /verify/bundle`
      call site'larına akmadığını doğrulayan taint analizi kontrolünün mevcut olduğunu doğrula
    - Callsite çıktılarında yasak alanların tarandığını doğrula:
      `execution_override`, `routing_hint`, `recommended_action`,
      `verification_weight`, `node_priority`
    - Her diagnostics endpoint'i için callsite → verification path izolasyon raporunun
      üretildiğini doğrula
    - `report.json` içinde `"gate": "diagnostics-callsite-correlation"` alanının üretildiğini doğrula
    - _Gereksinimler: 10.2, 10.3, 10.4, 10.5_

  - [ ]* 6.4 `validate_diagnostics_callsite_correlation.py` için özellik testi yaz
    - **Özellik 15: Diagnostics Callsite İzolasyonu** — diagnostics → verification path akışı → ihlal
    - `test_validate_diagnostics_callsite_correlation_gate.py` içine ekle
    - _Gereksinimler: 10.2, 10.3_

  - [x] 6.5 `validate_observability_routing_separation.py` implementasyonunu doğrula
    - Observability artifact'larında routing/scheduling semantiği taşıyan alanları taradığını doğrula:
      `routing_hint`, `execution_override`, `recommended_action`, `node_priority`,
      `verification_weight`, `retry`, `override`, `promote`, `commit`, `mitigation`
    - `observability_routing_negative_matrix.json` üretiminin her endpoint için
      routing semantiği yokluğunu matris formatında raporladığını doğrula
    - İhlal formatının `forbidden_routing_field:<artifact>:<path>:<field>` olduğunu doğrula
    - `report.json` içinde `"gate": "observability-routing-separation"` alanının üretildiğini doğrula
    - _Gereksinimler: 11.2, 11.3, 11.4, 11.5_

  - [ ]* 6.6 `validate_observability_routing_separation.py` için özellik testi yaz
    - **Özellik 14: Consumer Non-Authoritative Değişmezi** (routing varyantı) — routing alanı → ihlal
    - `test_validate_observability_routing_separation_gate.py` içine ekle
    - _Gereksinimler: 11.2, 11.4_

- [x] 7. Checkpoint — Tip C/D gate validator'larını doğrula
  - Tüm testlerin geçtiğini doğrula, sorular varsa kullanıcıya sor.

- [x] 8. Kill-Switch Coverage ve Entegrasyon
  - [x] 8.1 `summarize.sh` içinde `--require-kill-switch-completeness` davranışını doğrula
    - `tools/ci/summarize.sh` içinde 12 kill-switch gate'inin tamamının `PASS` vermesi
      durumunda `coverage_status: "COMPLETE"` üretildiğini doğrula
    - Herhangi bir gate FAIL verirse `coverage_status: "INCOMPLETE"` ve exit code 2
      üretildiğini doğrula
    - Kısmi PASS'ın kabul edilmediğini doğrula (Req 13.6)
    - _Gereksinimler: 13.1, 13.2, 13.3, 13.6_

  - [ ]* 8.2 `summarize.sh` için özellik testi yaz
    - **Özellik 17: Kill-Switch Coverage Tamamlanması** — tüm PASS → COMPLETE, herhangi FAIL → INCOMPLETE
    - `test_summarize_ci_run.py` içine `@given(st.lists(..., min_size=12, max_size=12))` ile ekle
    - _Gereksinimler: 13.1, 13.3, 13.6_

  - [x] 8.3 Gate script'lerinin fail-closed davranışını doğrula
    - Tüm 12 gate script'inin `set -euo pipefail` ile başladığını doğrula
    - Exit code semantiğinin doğru olduğunu doğrula: 0=PASS, 2=FAIL, 3=tooling hatası
    - Bootstrap başarısızlığı → `artifact_bootstrap_failed:cross_node_parity_harness` ihlali
      ve exit 2 üretildiğini doğrula
    - _Gereksinimler: 14.3, 14.6_

  - [ ]* 8.4 Gate script fail-closed davranışı için özellik testi yaz
    - **Özellik 18: Gate Script Fail-Closed Davranışı** — `set -euo pipefail` varlığını doğrula
    - Her gate script dosyasının ilk satırlarında `set -euo pipefail` bulunduğunu kontrol et
    - _Gereksinimler: 14.3_

  - [x] 8.5 `ci-kill-switch-phase13` Makefile hedefini uçtan uca doğrula
    - Tüm 12 gate'in `ci-kill-switch-phase13` bağımlılık zincirinde tanımlı olduğunu doğrula
    - Her gate'in `report.json` dosyasını `$(EVIDENCE_RUN_DIR)/reports/` altına kopyaladığını doğrula
    - `ci-kill-switch-summary` hedefinin `kill_switch_summary.json` içinde
      `coverage.coverage_status: "COMPLETE"` ürettiğini doğrula
    - `phase12-official-closure-prep` hedefinin `ci-kill-switch-phase13` ön koşuluna bağlı
      olduğunu doğrula
    - _Gereksinimler: 13.1, 13.4, 13.5_

- [x] 9. Final Checkpoint — Tüm testlerin geçtiğini doğrula
  - `ci-kill-switch-phase13` hedefinin `PASS` verdiğini ve `coverage_status: "COMPLETE"`
    üretildiğini doğrula. Sorular varsa kullanıcıya sor.

## Notlar

- `*` ile işaretli görevler opsiyoneldir; hızlı MVP için atlanabilir
- Her görev ilgili gereksinimlere referans verir
- Tip C/D gate'lerde validator'lar zaten mevcuttur; doğrulama ve eksik kontrollerin tamamlanması gerekir
- Tip A/B gate'lerde yeni validator dosyaları oluşturulacaktır
- Constitutional kurallar (NON_OVERRIDABLE.md) tüm implementasyonlarda geçerlidir
- Özellik testleri `hypothesis` kütüphanesi ile yazılır, minimum 100 iterasyon

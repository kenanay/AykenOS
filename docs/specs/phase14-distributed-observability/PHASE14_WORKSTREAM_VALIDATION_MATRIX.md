# Phase-14 İş Akışı Doğrulama Matrisi

**Faz:** 14 — Distributed Observability Hardening  
**Durum:** TÜM İŞ AKIŞLARI BİRLEŞTİRİLDİ  
**Son Güncelleme:** 2026-04-07  
**Otorite:** `ARCHITECTURE_FREEZE.md`  
**Kapanış Kanıtı:** `ci-freeze#23989067554`, `ci-freeze#23999026616`

---

## 1. İş Akışı Doğrulama Matrisi

| WS ID | Sözleşme Belgesi | Endpoint / Yüzey | Şema Kapsamı | Negatif Test | PASS Kanıtı | Birleştirme Durumu | Bağımlılık |
|-------|-----------------|-----------------|--------------|--------------|-------------|-------------------|------------|
| 3.1 | `docs/specs/phase14-distributed-observability/PROOFD_EXTERNAL_DIAGNOSTICS_CONTRACT_v1.md` | `GET /diagnostics/version`, `X-Ayken-API-Version` response header | Full (`VERSION_REQUIRED_FIELDS` in `api_schema.rs`) | `diagnostics_version_schema_violation_fails_closed` | PR #87, `ci-freeze#23989067554` | MERGED | none |
| 3.2 | `userspace/proofd/src/determinism/contract.rs` + `verification_determinism_contract.json` | `POST /internal/replay`, `ci-gate-determinism-replay-consistency` | Full (`verification_determinism_contract.json` schema) | `internal_replay_endpoint_emits_determinism_incident_on_hash_mismatch` | `ci-freeze#23989067554` | MERGED | none |
| 3.3 | `userspace/proofd/src/api_contract.rs` + `userspace/proofd/src/api_schema.rs` | Tüm public diagnostics endpoint'leri (`ROOT_DIAGNOSTICS_ENDPOINTS` + `RUN_SCOPED_DIAGNOSTICS_ENDPOINTS`) | Full (34 `FORBIDDEN_OBSERVABILITY_FIELDS`, runtime scan in `observability_json_response()`) | `root_summary_rejects_forbidden_field_score`, `parity_endpoint_fail_closes_when_artifact_exposes_forbidden_field` | PR #94, `ci-freeze#23989067554` | MERGED | none |
| 3.4 | `docs/specs/phase14-distributed-observability/CROSS_NODE_OBSERVABILITY_GRAPH_CONTRACT_v1.md` | `GET /diagnostics/graph`, `GET /diagnostics/graph/overlay`, `GET /diagnostics/runs/{run_id}/graph` | Full (`ROOT_GRAPH_REQUIRED_FIELDS`, `GRAPH_OVERLAY_REQUIRED_FIELDS`, `RUN_GRAPH_REQUIRED_FIELDS`) | `graph_endpoint_rejects_truth_selection_query`, `run_scoped_graph_endpoint_fail_closes_on_forbidden_field` | PR #96, `ci-freeze#23999026616` | MERGED | WS 3.3 (api_contract.rs registry) |
| 3.5 | `docs/specs/phase14-distributed-observability/OBSERVABILITY_UX_CONTRACT_v1.md` | `GET /diagnostics/summary`, `GET /diagnostics/runs/{run_id}/summary` | Full (`SUMMARY_REQUIRED_FIELDS`, `MACHINE_SUMMARY_REQUIRED_FIELDS`, `RUN_SCOPED_SUMMARY_REQUIRED_FIELDS`) | `root_summary_is_queryless`, `root_summary_rejects_forbidden_field_score`, `root_summary_schema_fails_closed_when_overlay_missing` | 210 lib + 6 main proofd tests PASS, 63 obs-cli tests PASS (2026-04-07) | MERGED | WS 3.3 (schema enforcement), WS 3.4 (graph surface for overlay data) |

---

## 2. Bağımlılık Zinciri

Aşağıdaki diyagram WS 3.3, 3.4 ve 3.5 arasındaki zorunlu bağımlılık sırasını göstermektedir:

```
WS 3.3 (proofd Sorgu/Hizmet Sınırı Sertleştirme)
  │
  │  api_contract.rs kayıt defteri + api_schema.rs altyapısı
  │
  ▼
WS 3.4 (Çapraz Düğüm Gözlemlenebilirlik Grafiği)
  │
  │  graph surface for overlay data
  │  (build_partitioned_root_graph_diagnostics + build_root_graph_overlay_diagnostics)
  │
  ▼
WS 3.5 (Gözlemlenebilirlik UX — İnsan Tarafından Okunabilir Katman)
```

**Bağımlılık Açıklamaları:**

- **WS 3.4 → WS 3.3:** Graf endpoint'leri `api_contract.rs` kayıt defterinden çözümlenir; şema kapsamı beyanları `api_schema.rs` altyapısını kullanır.
- **WS 3.5 → WS 3.3:** `GET /diagnostics/summary` şema doğrulaması ve yasak alan uygulaması `api_schema.rs` + `api_contract.rs` altyapısını kullanır.
- **WS 3.5 → WS 3.4:** `build_root_summary_diagnostics` → `build_partitioned_root_graph_diagnostics` + `build_root_graph_overlay_diagnostics` çağrı zinciri.

WS 3.1 ve WS 3.2 bağımsızdır; herhangi bir iş akışına bağımlılıkları yoktur.

---

## 3. CI Çalışma Kimliği Referansları

| CI Çalışma Kimliği | Sonuç | Kapsanan İş Akışları | İlgili PR |
|--------------------|-------|---------------------|-----------|
| `ci-freeze#23989067554` | PASS | WS 3.1, WS 3.2, WS 3.3 | PR #87 (WS 3.1), PR #94 (WS 3.3) |
| `ci-freeze#23999026616` | PASS | WS 3.4 | PR #96 |

**Not:** WS 3.5 yerel doğrulama kanıtıyla doğrulanmıştır (210 lib + 6 main proofd testleri + 63 obs-cli testi, 2026-04-07). Uzak `ci-freeze` onayı kapanış prosedürünün bir parçasıdır.

---

## 4. Özet — Tüm İş Akışları Birleştirildi

Tüm Phase-14 iş akışları (WS 3.1 – WS 3.5) `main` üzerinde birleştirilmiştir:

| WS ID | İş Akışı Adı | Durum |
|-------|-------------|-------|
| 3.1 | Read-Only External API Stabilization | ✅ MERGED |
| 3.2 | Replay Determinism Stability Hardening | ✅ MERGED |
| 3.3 | `proofd` Query/Service Boundary Hardening | ✅ MERGED |
| 3.4 | Cross-Node Observability Graph | ✅ MERGED |
| 3.5 | Observability UX (Human-Readable Layer) | ✅ MERGED |

**Phase-14 kapanış ön koşulu karşılanmıştır.** Resmi kapanış için uzak `ci-freeze` PASS onayı ve `closure_index.json` bütünlük doğrulaması gereklidir.

---

## 5. Referanslar

- Phase-14 geliştirme takipçisi: `docs/specs/phase14-distributed-observability/PHASE14_DEVELOPMENT_TRACKER.md`
- Harici diagnostics sözleşmesi (WS 3.1): `docs/specs/phase14-distributed-observability/PROOFD_EXTERNAL_DIAGNOSTICS_CONTRACT_v1.md`
- Graf sözleşmesi (WS 3.4): `docs/specs/phase14-distributed-observability/CROSS_NODE_OBSERVABILITY_GRAPH_CONTRACT_v1.md`
- UX sözleşmesi (WS 3.5): `docs/specs/phase14-distributed-observability/OBSERVABILITY_UX_CONTRACT_v1.md`
- Mimari harita: `docs/specs/phase14-distributed-observability/PHASE14_ARCHITECTURE_MAP.md`
- Kapanış adayı paketi: `reports/phase14_official_closure_candidate/`
- CI çalışma `ci-freeze#23989067554`: WS 3.1–3.3 uzak PASS kanıtı
- CI çalışma `ci-freeze#23999026616`: WS 3.4 uzak PASS kanıtı

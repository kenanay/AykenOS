# Freeze Enforcement Workflow (No-Timeline)
This document is subordinate to PHASE 0 - FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

**Status:** ACTIVE (Freeze-linked)  
**Scope:** Mainline enforcement during architecture freeze  
**Purpose:** Enforcement'i kalıcılaştırmak, drift'i sıfıra sabitlemek, exit kriterlerini kanıtla kapatmak

---

## 0) Governance Rule

1. Mainline'a feature merge yok.
2. Her iş kalemi `gate + evidence` ile kapanır.
3. "Bitti" iddiası ancak aşağıdakilerle geçerlidir:
   - `reports/summary.json` verdict = `PASS`
   - Evidence dizini path'i PR açıklamasında referanslı
   - İlgili doküman güncellemesi yapılmış

---

## 1) Freeze Entry Blocker Closure Pack

Bu kalemler kapanmadan freeze "aktif niyet"tir; "tam enforcement" değildir.

### 1.1 Syscall ABI Single Source Lock

**Workflow**
1. `kernel/include/ayken_abi.h` tek kaynak olarak korunur.
2. `make generate-abi` deterministik çıktı üretir.
3. Kernel dispatcher ve userspace wrapper aynı kaynaktan türetilir.

**Done Criteria**
1. ABI gate `PASS`
2. Header/generation parity raporu üretilmiş
3. Evidence: `evidence/run-<RUN_ID>/gates/abi/`

### 1.2 Userspace Syscall Register Mapping Fix

**Workflow**
1. Tek mapping: `RDI/RSI/RDX/R10`
2. Ring3 giriş/çıkış register yolu test ile doğrulanır.

**Done Criteria**
1. Syscall roundtrip `PASS`
2. Register contract testi `PASS`
3. Evidence: `evidence/run-<RUN_ID>/gates/abi/` + roundtrip kanıtı

### 1.3 Scheduler Fallback Isolation/Removal

**Workflow**
1. Kernel policy fallback tamamen kaldırılır veya feature flag ile izole edilir.
2. Default durum `fallback disabled` olur.
3. Kaldırma planı repo içinde izlenir.

**Done Criteria**
1. Boundary gate `PASS`
2. Runtime davranış testi `PASS`
3. Evidence: `evidence/run-<RUN_ID>/gates/boundary/`

### 1.4 Tracked Build Artifact Cleanup

**Workflow**
1. `target/`, `obj/`, `*.o`, `*.elf` tracked olmayacak.
2. Hygiene gate merge-blocking olacak.

**Done Criteria**
1. Hygiene gate `PASS`
2. Merge context temiz (`git diff --exit-code HEAD` policy)
3. Evidence: `evidence/run-<RUN_ID>/gates/hygiene/`

---

## 2) CI Gates Non-Bypassable Suite

### 2.1 Mandatory Gate Targets

1. `make ci-gate-abi`
2. `make ci-gate-boundary`
3. `make ci-gate-workspace`
4. `make ci-gate-hygiene`
5. `make ci-gate-performance`
6. `make ci-summarize`

### 2.2 Gate Implementation Status (Repo Truth)

1. **Implemented:** `ci-gate-boundary`, `ci-gate-hygiene`, `ci-summarize`
2. **Planned (hard-fail stubs):** `ci-gate-abi`, `ci-gate-workspace`, `ci-gate-performance`
3. Stub hedefler bilinçli olarak `exit 2` döner; bu, "varmış gibi" geçmeyi engeller.

### 2.3 CI Entry Point Contract

1. `make ci` = mevcut minimum zorunlu zincir (`ci-gate-boundary` + `ci-gate-hygiene` + `validate-full`)
2. `make ci-freeze` = strict freeze suite (planlı gate stub'ları dahil)
3. `summary.json` verdict `PASS` değilse ilgili make hedefi fail eder.

### 2.4 Evidence Standard (Canonical Layout)

**Goal:** Auditable, comparable, deterministic evidence.

```text
evidence/
└── run-<run_id>/
    ├── meta/
    │   ├── run.json
    │   ├── git.txt
    │   └── toolchain.txt
    ├── artifacts/
    │   ├── <artifact>
    │   └── <artifact>.sha256
    ├── gates/
    │   └── <gate>/
    │       ├── report.json
    │       ├── meta.txt
    │       └── *.raw/*.filtered/*.violations (opsiyonel)
    ├── logs/
    └── reports/
        └── summary.json
```

**Done When**
1. Tüm gate'ler bu şemaya yazar.
2. `summary.json` auto-discovery ile keşfedilen tüm gate raporlarını listeler.

---

## 3) Boundary Enforcement Hardening

### 3.1 Denylist Alignment

Kernel artifact içinde istemediğimiz semboller:
1. libc/stdio/alloc sembolleri
2. unwind/c++ runtime sembolleri
3. POSIX syscall sembolleri
4. Policy-leak niyeti taşıyan project-specific semboller

### 3.2 Allowlist Discipline

1. Her allow satırında gerekçe yorumu bulunur.
2. Mümkünse allow satırı `file_regex` ile scope daraltır.

### 3.3 Optional Source Heuristics

1. Kaynak seviyesinde hedefli pattern taraması eklenebilir.
2. Nihai otorite binary symbol scan'dir.

---

## 4) Performance Baseline Freeze

### 4.1 Baseline Manifest

1. `evidence/baselines/x86_64/<baseline-id>/env.json`
2. `evidence/baselines/x86_64/<baseline-id>/results.json`
3. `evidence/baselines/x86_64/<baseline-id>/raw.log`
4. Referans: `BASELINE_SHA + BASELINE_ENV_HASH`

### 4.2 Gate Behavior

1. Env hash mismatch davranışı tek doğru kuralla sabitlenir:
   - `FAIL` veya `WAIVER required`
2. Aynı repo içinde çelişkili davranış olamaz.

---

## 5) Constitutional Gate Linkage

Her merge için constitutional kanıt zorunlu:
1. AHS raporu
2. `NON_OVERRIDABLE = 0`
3. Waiver varsa expiry + issue link + teknik gerekçe
4. Evidence: `evidence/run-<RUN_ID>/gates/constitutional/report.json`

---

## 6) Claim Freeze Operating Procedure

### 6.1 PR Template Required Fields

1. Gate run id
2. Evidence path
3. Changed contracts (`yes/no`)
4. RFC link (gerekiyorsa)
5. Waiver link (gerekiyorsa)

Template path:
- `docs/development/PR_FREEZE_TEMPLATE.md`
- `.github/pull_request_template.md`

### 6.2 "Completed/Production-ready" Claim Rule

1. `summary.json` = `PASS`
2. Test/benchmark kanıtı var
3. Doküman güncellemesi var
4. Architecture review notu var

---

## 7) RFC + Waiver Operationalization

Repo içinde görünür işlem akışı zorunludur:
1. `docs/rfc/0001-template.md`
2. `docs/waivers/README.md`
3. `docs/waivers/WAIVER_TEMPLATE.md`
4. `docs/architecture-board/decisions/README.md`
5. `docs/architecture-board/decisions/0001-template.md`

Kural:
1. RFC/Waiver yoksa breaking change yok.

---

## 8) Claim Freeze (No Evidence = No Claim)

### 8.1 Claim Requirements

A "Completed/Production-ready" claim requires:
1. `summary.json` PASS
2. Tests committed and referenced
3. Benchmark results committed and referenced
4. Documentation updated
5. Architecture review note (decision record)

**Evidence**
1. `evidence/run-<id>/reports/summary.json`
2. Links to test/bench outputs in repo
3. Decision record or review approval note

**Done When**
1. Claim is verifiable by reading evidence and repo refs.

---

## 9) Freeze Exit Criteria Closure Checklist

Freeze lift is blocked until **all** of these are closed with evidence:

1. Ring3 policy fully hardened (no kernel fallback in default build)
2. Scheduler fallback removed or isolated (default off) + removal path documented
3. Syscall drift = 0 (ABI gate passes consistently with evidence)
4. CI gates stable (suite consistently PASS with evidence)
5. AHS trend not declining (≥95 maintained; `non_overridable=0`)
6. Performance regression = 0 (baseline comparison passes)
7. All freeze-blocking issues closed (tracked issues list is empty)
8. Architecture Board approval recorded (decision record present)

**Evidence**
1. A freeze-exit bundle run (`evidence/run-<id>` full suite)
2. Decision record reference

**Done When**
1. All exit criteria have evidence and approval record exists.

---

## 10) Work Queue (Live Checklist)

- [ ] Close ABI single-source + generator determinism
- [ ] Close syscall register mapping invariant test
- [ ] Remove/isolate scheduler fallback (default off)
- [ ] Repo hygiene: remove tracked artifacts and enforce
- [ ] Finalize boundary deny/allow lists and document rationale
- [ ] Add perf baseline manifest + perf gate
- [ ] Wire constitutional gate + waiver docs
- [ ] RFC + waiver directories and templates
- [ ] Freeze exit bundle run + board approval record

---

## 11) Daily Operating Rhythm

Her iterasyonda minimum rutin:
1. `make ci-gate-boundary`
2. `make ci-gate-hygiene`
3. `make ci` (mevcut minimum suite)
4. Evidence path'i PR'a yaz

Fail olursa:
1. Fail kaynağını sınıflandır (`deny/allow` mi, gerçek leak mi)
2. Düzelt
3. Yeni run id ile tekrar çalıştır

---

## Definition of Done (Freeze Context)

Bir kalem sadece aşağıdakiler birlikte sağlandığında kapanır:
1. İlgili gate `PASS`
2. `summary.json` `PASS`
3. Evidence path PR'da referanslı
4. İlgili doküman güncel

---

## Notes

1. Freeze yavaşlamak için değil, mimari rastlantısallığı kaldırmak içindir.
2. Her merge evidence ile denetlenebilir olmalıdır.

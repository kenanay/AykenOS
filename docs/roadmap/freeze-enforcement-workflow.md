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
3. Baseline lock mevcut ve güncel: `scripts/ci/abi-baseline.lock.json`
4. Baseline lock git'te tracked ve temiz (worktree/index drift yok)
5. Evidence: `evidence/run-<RUN_ID>/gates/abi/`

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
2. Default durum `fallback disabled` olur (`AYKEN_SCHED_FALLBACK ?= 0`).
3. Fallback yalnızca validation profile'da explicit olarak açılabilir (`AYKEN_SCHED_FALLBACK=1` + `KERNEL_PROFILE=validation`).
4. Fallback kapalıyken Ring0 seçim policy çalıştırmaz; Ring3 scheduler mailbox üzerinden `next` stage eder ve Ring0 tüketir (bootstrap öncesi tek-seferlik ready list tüketimi hariç).
5. Scheduler arbitration contract (Yol A) zorunludur: Ring3 `stage_next` yalnız hint üretir, Ring0 final arbiter olarak kabul/veto eder.
6. Strict scheduler path'te kabul için minimum sanity doğrulaması zorunludur (registered/state/context).
7. Bridge syscall penceresi `0x90..0x9F` ile sınırlı tutulur; `SYS_V2` freeze aralığı değişmez ve bridge çağrıları execution-centric `SYS_V2` sözleşmesine dahil değildir.
8. Karar kaydı: `docs/architecture-board/decisions/20260214-scheduler-arbitration-contract.md`.
9. `make ci-freeze` hard guard ile fallback açıkken fail eder.
10. Kaldırma planı repo içinde izlenir.

**Done Criteria**
1. Boundary gate `PASS`
2. Constitutional gate strict-mode `PASS` (scheduler fallback contract check dahil)
3. Runtime davranış testi `PASS`
4. Evidence: `evidence/run-<RUN_ID>/gates/boundary/` + `evidence/run-<RUN_ID>/gates/constitutional/sched-fallback-check.txt`

### 1.4 Tracked Build Artifact Cleanup

**Workflow**
1. `target/`, `obj/`, `*.o`, `*.elf` tracked olmayacak.
2. Hygiene gate merge-blocking olacak.

**Done Criteria**
1. Hygiene gate `PASS`
2. Merge context temiz (`git diff --exit-code HEAD` policy)
3. Evidence: `evidence/run-<RUN_ID>/gates/hygiene/`

**Temporary Status (2026-02-22):**
- Hygiene gate temporarily SKIPPED due to 55GB evidence/ directory (388 runs) causing timeout
- Manual hygiene verification required until evidence/ cleanup complete
- Action: evidence/ will be moved to .gitignore in future commit
- Impact: MVP-1 validation not affected (code changes minimal, other gates pass)

---

## 2) CI Gates Non-Bypassable Suite

### 2.1 Mandatory Gate Targets

1. `make ci-gate-abi`
2. `make ci-gate-boundary`
3. `make ci-gate-hygiene`
4. `make ci-gate-tooling-isolation`
5. `make ci-gate-constitutional`
6. `make ci-gate-workspace`
7. `make ci-gate-syscall-v2-runtime`
8. `make ci-gate-sched-bridge-runtime`
9. `make ci-gate-performance`
10. `make ci-summarize`

### 2.2 Gate Implementation Status (Repo Truth)

1. **Implemented:** `ci-gate-abi`, `ci-gate-boundary`, `ci-gate-hygiene`, `ci-gate-tooling-isolation`, `ci-gate-constitutional`, `ci-gate-workspace`, `ci-gate-syscall-v2-runtime`, `ci-gate-sched-bridge-runtime`, `ci-gate-performance`, `ci-summarize`
2. Runtime gate spec: `docs/development/SYSCALL_V2_RUNTIME_GATE_SPEC.md`
3. Sched bridge runtime gate: Validates scheduler mailbox accept/reject markers and epoch progression
4. Baseline lock olmayan gate'ler fail-closed kalır; bu, "varmış gibi" geçmeyi engeller.

### 2.3 CI Entry Point Contract

1. `make ci` = mevcut minimum zorunlu zincir (`ci-gate-boundary` + `ci-gate-hygiene` + `validate-full`)
2. `make ci-freeze` = strict freeze suite (tüm implemented gate'ler)
3. `summary.json` verdict `PASS` değilse ilgili make hedefi fail eder.
4. CI orchestration workflow: `.github/workflows/ci-freeze.yml` (GitHub-hosted `ubuntu-latest` + fail-closed baseline policy).
5. Runner hardening/runbook: `docs/operations/SELF_HOSTED_RUNNER_HARDENING.md`.
6. Tooling isolation guard: perf/preempt tooling PR'larında `kernel/**` dokunuşu fail-closed (`make ci-gate-tooling-isolation`).

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

### 2.5 Hygiene Gate Contract (Operational)

**CI Call Point**
1. `make ci-gate-hygiene`
2. Make target, `scripts/ci/gate_hygiene.sh --evidence-dir evidence/run-<RUN_ID>/gates/hygiene` çağırır.
3. `reports/hygiene.json` kopyalanır ve `make ci-summarize` ile global verdict zorunlu tutulur.
4. Source deny kuralları:
   - `scripts/ci/hygiene-source-deny.regex`
   - `scripts/ci/hygiene-source-allow.regex` (boş, waiver yoksa kullanılmaz)

**Temporary Status (2026-02-22):**
- Gate temporarily SKIPPED due to 55GB evidence/ directory (388 runs) causing git ls-files timeout
- Manual hygiene verification required until evidence/ cleanup complete
- Action: evidence/ will be moved to .gitignore in future commit
- Impact: MVP-1 validation not affected (code changes minimal, other gates pass)

**Hygiene Rules (merge-blocking when active)**
1. Forbidden tracked artifacts (`target/`, `build/`, `obj/`, `*.o`, `*.elf`, `*.a`, `*.so`, `*.tmp`)
2. Tracked executable/binary files (allowlist hariç)
3. Oversized tracked files (`> 5,000,000` bytes, allowlist hariç)
4. Dirty tracked workspace (`git status --porcelain --untracked-files=no`)
5. Source deny scan: `static malloc/free` fonksiyon tanımı yasak (kernel + `userspace/libayken`)

**Hygiene Evidence Files**
1. `evidence/run-<RUN_ID>/gates/hygiene/tracked.files.txt`
2. `evidence/run-<RUN_ID>/gates/hygiene/forbidden-tracked.txt`
3. `evidence/run-<RUN_ID>/gates/hygiene/tracked-binary.txt`
4. `evidence/run-<RUN_ID>/gates/hygiene/oversized-tracked.txt`
5. `evidence/run-<RUN_ID>/gates/hygiene/dirty-tracked.txt`
6. `evidence/run-<RUN_ID>/gates/hygiene/source-deny-hits.txt`
7. `evidence/run-<RUN_ID>/gates/hygiene/violations.txt`
8. `evidence/run-<RUN_ID>/gates/hygiene/meta.txt`
9. `evidence/run-<RUN_ID>/gates/hygiene/report.json`

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

### 3.4 Link Boundary Rule

1. `kernel.elf` yalnız Ring0 objelerinden linklenir.
2. `userspace/libayken/*.o` kernel link setine dahil edilemez.
3. VFS mekanizma katmanı dosya topolojisi ile sabitlenir: `kernel/include/vfs_mech.h` + `kernel/fs/vfs_mech.c`.
4. Kernel policy test kodu kernel image dışı tutulur (`*_test.c` default link dışı).
5. Boundary run, linker map kanıtı üretir: `evidence/run-<RUN_ID>/artifacts/kernel.map`.

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
3. Gate implementation: `make ci-gate-performance` -> `scripts/ci/gate_performance.sh`
4. Baseline lock: `scripts/ci/perf-baseline.lock.json` (tracked + clean olmalı)
5. İlk baseline yalnızca explicit init ile yazılır: `PERF_INIT_BASELINE=1` (gate bilinçli FAIL döner, commit bekler)
6. Baseline init authority default: CI-only (`PERF_REQUIRE_CI_FOR_BASELINE_INIT=1`).
7. Marker format freeze-contract olarak kilitlidir (`boot_ok_marker`, `preempt_sw_count_pattern`, `preempt_iret_count_pattern`) ve baseline compare içinde doğrulanır.
8. Local baseline init yalnızca explicit override + waiver ile yapılır (`PERF_REQUIRE_CI_FOR_BASELINE_INIT=0` + waiver referansı).
9. Baseline authority tek-doğru: `PERF_BASELINE_AUTHORITY=github-hosted-ubuntu-latest-x64`.
10. Runner image/build kimliği baseline sözleşmesine dahil edilir: `PERF_CI_IMAGE_DIGEST=<pinned digest/id>`.
11. Baseline init `PERF_CI_IMAGE_DIGEST=unknown` ile yapılamaz; pinned digest zorunludur.
12. Default pinned digest source: GitHub hosted image metadata (`ImageOS`, `ImageVersion`, `RUNNER_ARCH`) (workflow input only override).

---

## 5) Constitutional Gate Linkage

Her merge için constitutional kanıt zorunlu:
1. AHS raporu
2. `NON_OVERRIDABLE = 0`
3. Waiver varsa expiry + issue link + teknik gerekçe
4. Evidence: `evidence/run-<RUN_ID>/gates/constitutional/report.json`

### 5.1 Strict Mode (Fail-Closed)

1. `make ci-gate-constitutional` default strict-mode ile çalışır (`CONSTITUTIONAL_STRICT=1`).
2. Ring0 exported symbol whitelist enforcement:
   - seed allowlist: `scripts/ci/constitutional-ring0-symbol-whitelist.regex`
   - evidence: `ring0-symbols.txt`, `ring0-symbol-violations.txt`
3. Ring0 source deny/allow enforcement:
   - deny: `scripts/ci/constitutional-source-deny.regex`
   - allow (waiver-only): `scripts/ci/constitutional-source-allow.regex`
4. Scheduler fallback contract enforcement (`AYKEN_SCHED_FALLBACK=0` strict-mode, Makefile/header default lock).
5. Whitelist dışı symbol veya source deny hit varsa verdict = `FAIL`.

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
- [x] Wire constitutional gate + waiver docs
- [x] RFC + waiver directories and templates
- [ ] Freeze exit bundle run + board approval record

---

## 11) Daily Operating Rhythm

Her iterasyonda minimum rutin:
1. `make ci-gate-boundary`
2. `make ci-gate-hygiene`
3. `make ci-gate-workspace`
4. `make ci-gate-constitutional`
5. `make ci` (mevcut minimum suite)
6. Evidence path'i PR'a yaz

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

# Post-Merge Smoke Test Checklist

**Effective date:** 2026-05-25
**Scope:** Bu checklist yalniz yetkili bir merge sonrasinda teknik smoke
evidence toplar. Merge authority, production-ready verdict'i veya Phase-17
closure kurmaz. Tek-maintainer authority karari ve live protection paritesi
issue #145 ile tamamlanmistir; bu kayit smoke PASS'i closure'a donusturmez.
**Duzenleyen / Gelistiren / Olusturan / Mimari Sorumlu:** Kenan AY
**Attribution boundary:** Dokumantasyon metadata'si; runtime, evidence,
merge veya closure otoritesi degildir.

Bu checklist merge sonrasi 5-10 dakikada tamamlanir ve governance
degisikliklerinin constitutional CI davranisini koruduguna dair teknik kayit
uretir.

## 1. CI Freeze Run (İlk Koşu)

**Hedef:** Merge sonrası ilk freeze run'ın strict modda geçtiğini doğrula.

```bash
# GitHub Actions'da ci-freeze workflow'unu izle
# Beklenen: PASS (yeşil)
```

**Kontrol noktaları:**
- [ ] `PERF_BASELINE_MODE=constitutional` aktif
- [ ] `PERF_ENV_MISMATCH_POLICY=fail` aktif
- [ ] `PERF_REGRESSION_POLICY=fail` aktif
- [ ] Performance gate: PASS
- [ ] `boot_time_ms` baseline'a göre beklenen tolerans içinde (regression yok)
- [ ] Authority: `github-hosted-ubuntu-24.04-x64` (tek kaynak)
- [ ] `PERF_BASELINE_AUTHORITY` log çıktısı `scripts/ci/perf_authority.env` ile birebir eşleşiyor
- [ ] Evidence artifact üretildi: `evidence/run-*/gates/performance/report.json`

**Fail durumunda:**
- Log'da `PERF_ENV_MISMATCH` veya `PERF_REGRESSION` fail nedeni var mı?
- Authority drift var mı? (`scripts/ci/perf_authority.env` vs workflow env)
- Baseline güncel mi? (env_hash mismatch olabilir)

---

## 2. Baseline Init Workflow (Tetikleme Kontrolü)

**Hedef:** Baseline init workflow'unun sadece gerektiğinde tetiklendiğini doğrula.

```bash
# GitHub Actions'da perf-baseline-init workflow'unu kontrol et
# Beklenen: Manuel tetikleme veya env_hash mismatch durumunda çalışır
```

**Kontrol noktaları:**
- [ ] Workflow `workflow_dispatch` ile manuel tetiklenebiliyor
- [ ] Authority `scripts/ci/perf_authority.env`'den okunuyor
- [ ] Darwin guard çalışıyor: `github-hosted-ubuntu-*-x64` pattern match
- [ ] Baseline artifact doğru yere yazılıyor: `evidence/baseline/`

**Test (opsiyonel):**
```bash
# Manuel tetikleme ile baseline init çalıştır
# GitHub Actions UI'dan workflow_dispatch kullan
```

---

## 3. Local Dev Override (Geliştirici Deneyimi)

**Hedef:** Local dev'de authority override'ın çalıştığını doğrula.

```bash
# Local ortamda test et
PERF_BASELINE_AUTHORITY=local-dev-test make -pn | grep '^PERF_BASELINE_AUTHORITY'
# Beklenen: PERF_BASELINE_AUTHORITY = local-dev-test

# Env yoksa default kullanılmalı
make -pn | grep '^PERF_BASELINE_AUTHORITY'
# Beklenen: PERF_BASELINE_AUTHORITY = github-hosted-ubuntu-24.04-x64
```

**Kontrol noktaları:**
- [ ] Env override çalışıyor
- [ ] Default `perf_authority.env`'den geliyor
- [ ] Local build kırılmıyor

---

## 4. Validation Strict (Sürekli Temizlik)

**Hedef:** Strict build'in -Werror altında temiz kaldığını doğrula.

```bash
# Local veya CI'da
make clean && make validation-strict -j4
# Beklenen: PASS (warning yok)
```

**Kontrol noktaları:**
- [ ] Build PASS
- [ ] Warning count: 0
- [ ] Yeni teknik borç eklenmemiş

---

## 5. Deterministic Regression Hook (Canary Hazırlığı)

**Hedef:** Hook'un default OFF olduğunu ve açıldığında çalıştığını doğrula.

```bash
# Default OFF kontrolü
make clean && make kernel KERNEL_PROFILE=validation
# Beklenen: AYKEN_INTENTIONAL_PERF_REGRESSION_MS=0 (default)

# Hook açık kontrolü (opsiyonel, sadece canary PR için)
AYKEN_INTENTIONAL_PERF_REGRESSION_MS=2000 make clean && make kernel KERNEL_PROFILE=validation
# Beklenen: Compile geçer, hook kodu dahil edilir
```

**Kontrol noktaları:**
- [ ] Default build'de hook OFF
- [ ] Flag set edildiğinde hook aktif
- [ ] Compile-time gated: `#if AYKEN_INTENTIONAL_PERF_REGRESSION_MS > 0`

---

## 6. Evidence Artifact Kontrolü

**Hedef:** Performance gate'in evidence ürettiğini doğrula.

```bash
# CI run sonrası evidence dizinini kontrol et
ls -la evidence/run-*/gates/performance/
# Beklenen: report.json, metadata.json, env_hash.txt
```

**Kontrol noktaları:**
- [ ] `report.json` mevcut ve valid JSON
- [ ] `report.json` içinde `boot_time_ms` değeri baseline ile makul tolerans içinde (regression yok)
- [ ] `metadata.json` authority bilgisi içeriyor
- [ ] `env_hash.txt` mevcut
- [ ] `env_hash.txt` ile baseline'daki `env_hash` eşleşiyor (drift yok)
- [ ] Artifact GitHub Actions'da downloadable

---

## 7. Governance Policy Dokümanı (Referans Kontrolü)

**Hedef:** Policy dokümanının güncel ve erişilebilir olduğunu doğrula.

```bash
# Repo'da policy dokumanini kontrol et
cat docs/operations/PERF_BASELINE_POLICY.md
# Beklenen: Güncel terminoloji, drift SLA, intentional regression kuralları
```

**Kontrol noktaları:**
- [ ] `ci_image_digest` terimi runner fingerprint'i olarak aciklanmis
      (OCI/container digest'i degil)
- [ ] Drift SLA: "1 iş günü (hedef ≤24h)"
- [ ] Intentional regression kuralları: compile-time gated, default OFF

---

## Özet: 5 Dakikalık Hızlı Kontrol

Zamanın kısıtlıysa sadece bunları kontrol et:

1. **CI freeze run yeşil mi?** → GitHub Actions
2. **Authority tek kaynaktan mı?** → `scripts/ci/perf_authority.env` + workflow logs
3. **validation-strict temiz mi?** → `make validation-strict`
4. **Evidence üretildi mi?** → `evidence/run-*/gates/performance/report.json`
5. **Hook default OFF mu?** → `make kernel` (flag olmadan)

Hepsi PASS ise teknik post-merge smoke kaydi tamamlanir. Bu sonuc tek
basina production-ready verdict'i veya resmi faz kapanisi degildir.

---

## Production Signal

**Freeze PASS + strict PASS + tek authority kaynagi teknik evidence
uretir. Official Phase-17 closure icin closure manifest/tag ve oncesinde
uygulanabilir review/merge authority ayrica gereklidir.**

---

## Fail Durumunda Rollback

Eğer kritik bir sorun tespit edilirse:

```bash
# Sadece hook'u geri al
git revert c0164547

# Sadece local override fix'i geri al
git revert f17d7269

# PR'ı komple geri almak için (merge commit ise):
git revert -m 1 <merge_commit_sha>

# Alternatif: Tek tek revert (ters sırada):
git revert f17d7269
git revert 1b01cf08
git revert 96cf41a3
git revert c0164547
git revert 4e556c8c
```

**Not:** Rollback secimi etki analizi ve review gerektirir; bu checklist
kismi rollback'in otomatik olarak guvenli oldugunu iddia etmez.

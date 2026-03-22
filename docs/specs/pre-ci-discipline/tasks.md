# Uygulama Planı: Pre-CI Discipline

## Genel Bakış

Pre-CI Discipline altyapısını resmileştiren görev listesi. Mevcut betik (`scripts/ci/pre_ci_discipline.sh`) ve Makefile hedefi (`make pre-ci`) zaten doğru çalışmaktadır. Bu görevler: hook dosyasını `docs/hooks/` altına taşımayı, test betiğini yazmayı ve dokümantasyonu tamamlamayı kapsar.

## Görevler

- [x] 1. `.kiro` dizinini kaldır — tüm içeriği `docs/` altına taşı
  - `docs/hooks/` dizini oluştur; `.kiro/hooks/*.kiro.hook` dosyalarını buraya kopyala
  - `docs/steering/` dizini oluştur; `.kiro/steering/*.md` dosyalarını buraya kopyala
  - `docs/hooks/HOOK_CONFIGURATION.md` dosyasını yeni konum bilgisiyle güncelle
  - `ci-gate-simulation.kiro.hook` içindeki inline shell kodunu `scripts/ci/pre_ci_discipline.sh`'e delege edecek şekilde güncelle
  - _Gereksinimler: 3.1, 3.4_

  - [x] 1.1 `docs/hooks/pre-ci-discipline.kiro.hook` dosyasını oluştur
    - `runCommand` tipiyle `bash scripts/ci/pre_ci_discipline.sh` komutunu çalıştır
    - `agentStop` olayına bağla
    - `shortName: "pre-ci-discipline"`, `workspaceFolderName: "AykenOS"` ayarla
    - _Gereksinimler: 3.1, 3.2, 3.3_

  - [x] 1.2 Hook konfigürasyon doğrulama testi yaz
    - `jq` ile hook JSON yapısını doğrula: `enabled`, `when.type`, `then.type`, `shortName`
    - `ci-gate-simulation` ve `pre-ci-discipline` hook'larının farklı `shortName`'e sahip olduğunu doğrula
    - _Gereksinimler: 3.4_

- [x] 2. `scripts/ci/pre_ci_discipline.sh` betiğini RUN_ID desteğiyle güncelle
  - `RUN_ID` değişkenini `YYYYMMDDTHHMMSSZ-<git-short-sha>` formatında üret
  - Başarısızlık mesajındaki kanıt yolunu `${EVIDENCE_ROOT:-out/evidence}/run-${RUN_ID}/reports/` olarak güncelle
  - `set -euo pipefail` başlığının mevcut olduğunu doğrula (zaten var, değiştirme)
  - _Gereksinimler: 2.3, 6.1, 6.4_

  - [x] 2.1 RUN_ID format özellik testi yaz
    - `# Feature: pre-ci-discipline, Property 6: Deterministik yeniden üretilebilirlik`
    - Aynı mock kapı sonuçlarıyla iki çalıştırmanın aynı çıkış kodunu ürettiğini doğrula
    - _Gereksinimler: 6.3_

- [x] 3. Test betiği `scripts/ci/test_pre_ci_discipline.sh` oluştur
  - Mock `make` fonksiyonu: kapı adına göre yapılandırılabilir çıkış kodu döndürür
  - Her kapı pozisyonu (1-4) için başarısızlık senaryosu testi
  - Tüm kapılar geçer senaryosu testi
  - _Gereksinimler: 1.1, 1.2, 1.3, 1.4_

  - [x] 3.1 Kapı sırası özellik testi yaz
  - [x] 3.2 Fail-closed özellik testi yaz
  - [x] 3.3 Başarısızlık çıkış kodu özellik testi yaz
  - [x] 3.4 Başarısızlık çıktısı bütünlüğü özellik testi yaz
  - [x] 3.5 Başarı çıktısı bütünlüğü özellik testi yaz
  - [x] 3.6 Workspace mutasyon yasağı özellik testi yaz

- [x] 4. Kontrol noktası — Tüm testler geçmeli
  - `bash scripts/ci/test_pre_ci_discipline.sh` çalıştır
  - Tüm testlerin geçtiğini doğrula; sorun varsa kullanıcıya sor.

- [x] 5. `docs/hooks/HOOK_CONFIGURATION.md` güncelle
  - `pre-ci-discipline` hook'unu aktif hook listesine ekle
  - Hook dosyalarının artık `docs/hooks/` altında olduğunu belgele
  - `ci-gate-simulation` hook'unun geçiş planını ekle (devre dışı bırakma koşulları)
  - _Gereksinimler: 3.4, 5.2_

- [x] 6. Son kontrol noktası — Tüm testler geçmeli
  - `bash scripts/ci/test_pre_ci_discipline.sh` çalıştırıldı: **30/30 PASS**
  - Hook JSON dosyaları `jq` ile doğrulandı (Property 1.2)
  - Tüm özellik testleri geçti

## Notlar

- `*` ile işaretli görevler isteğe bağlıdır; MVP için atlanabilir
- Görev 1 önceliklidir: hook dosyalarının yeni konumu diğer görevleri etkiler
- `scripts/ci/pre_ci_discipline.sh` mevcut haliyle büyük ölçüde doğrudur; yalnızca RUN_ID formatı eklenir
- Tüm özellik testleri shell tabanlıdır (Rust proptest değil); betik `bash` ile yazılmıştır

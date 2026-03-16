# Gereksinimler Belgesi

## Giriş

Pre-CI Discipline, gerçek CI çalıştırılmadan önce geliştirici iş istasyonunda çalışan yerel, fail-closed bir disiplin katmanıdır. Dört temel CI kapısını (`ci-gate-abi`, `ci-gate-boundary`, `ci-gate-hygiene`, `ci-gate-constitutional`) sıkı bir sırayla çalıştırır ve ilk başarısızlıkta durur. Bu sistem, CI'ın yerini almaz; CI merge için zorunlu olmaya devam eder. Pre-CI Discipline, geliştiriciye erken geri bildirim sağlayan bir danışma katmanıdır.

Mevcut uygulama `scripts/ci/pre_ci_discipline.sh` ve `pre-ci-discipline.sh` wrapper'ı ile `Makefile`'daki `pre-ci` hedefi üzerinden çalışmaktadır. Bu spec, söz konusu altyapıyı Kiro hook sistemi ile entegre ederek, davranışı resmileştirir ve test edilebilir hale getirir.

## Sözlük

- **Pre_CI_Discipline**: Gerçek CI öncesinde çalışan yerel fail-closed disiplin katmanı.
- **Discipline_Gate**: `make ci-gate-*` komutlarından biri; ABI, Boundary, Hygiene veya Constitutional.
- **Gate_Runner**: `run_gate()` fonksiyonu; tek bir kapıyı çalıştırır ve başarısızlıkta durur.
- **Fail_Closed**: Herhangi bir kapı başarısız olduğunda yürütmenin derhal durması ve otomatik düzeltme yapılmaması politikası.
- **Hook**: `.kiro/hooks/` dizinindeki Kiro otomasyon tanımı; belirli IDE olaylarına tepki verir.
- **Evidence_Root**: Kapı kanıtlarının yazıldığı dizin; varsayılan `out/evidence/run-<RUN_ID>/`.
- **Gate_Order**: ABI → Boundary → Hygiene → Constitutional sırası; bu sıra anayasal olarak sabittir.
- **Advisory_Layer**: Pre-CI Discipline'in CI'ın yerini almadığını, yalnızca erken uyarı sağladığını belirten statü.

## Gereksinimler

### Gereksinim 1: Fail-Closed Kapı Yürütme Sırası

**Kullanıcı Hikayesi:** Bir geliştirici olarak, pre-ci disiplin kapılarının sabit bir sırayla çalışmasını ve ilk başarısızlıkta durmasını istiyorum; böylece ihlaller erken tespit edilir ve sonraki kapılar yanlış bir güven vermez.

#### Kabul Kriterleri

1. WHEN `make pre-ci` veya `pre-ci-discipline.sh` çalıştırıldığında, THE Pre_CI_Discipline SHALL kapıları şu sırayla çalıştırır: ABI Gate → Boundary Gate → Hygiene Gate → Constitutional Gate.
2. WHEN herhangi bir Discipline_Gate başarısız olduğunda, THE Gate_Runner SHALL yürütmeyi derhal durdurur ve kalan kapıları çalıştırmaz.
3. WHEN herhangi bir Discipline_Gate başarısız olduğunda, THE Gate_Runner SHALL çıkış kodu olarak `2` döndürür.
4. WHEN tüm dört Discipline_Gate başarıyla tamamlandığında, THE Pre_CI_Discipline SHALL "ALL GATES PASS" mesajını çıktıya yazar ve çıkış kodu `0` döndürür.
5. THE Pre_CI_Discipline SHALL hiçbir koşulda kapı başarısızlığını otomatik olarak düzeltmeye çalışmaz.
6. THE Pre_CI_Discipline SHALL hiçbir koşulda kapı yürütme sırasını değiştirmeye izin vermez.

### Gereksinim 2: Kapı Başarısızlık Raporlaması

**Kullanıcı Hikayesi:** Bir geliştirici olarak, hangi kapının başarısız olduğunu ve kanıtları nerede bulacağımı açıkça görmek istiyorum; böylece manuel müdahale için doğru yere yönlendirilirim.

#### Kabul Kriterleri

1. WHEN bir Discipline_Gate başarısız olduğunda, THE Gate_Runner SHALL başarısız olan kapının adını çıktıya yazar.
2. WHEN bir Discipline_Gate başarısız olduğunda, THE Gate_Runner SHALL "Stopping execution (fail-closed)." mesajını çıktıya yazar.
3. WHEN bir Discipline_Gate başarısız olduğunda, THE Gate_Runner SHALL kanıt dizininin yolunu (`evidence/run-<RUN_ID>/reports/`) çıktıya yazar.
4. WHEN bir Discipline_Gate başarıyla tamamlandığında, THE Gate_Runner SHALL o kapı için "PASS" onayını çıktıya yazar.
5. THE Pre_CI_Discipline SHALL başarısızlık durumunda otomatik düzeltme önerisi sunmaz; yalnızca kanıt konumunu gösterir.

### Gereksinim 3: Kiro Hook Entegrasyonu

**Kullanıcı Hikayesi:** Bir geliştirici olarak, pre-ci disiplin katmanının Kiro agent durduğunda otomatik olarak tetiklenmesini istiyorum; böylece her agent çalışmasından sonra disiplin kontrolü yapılır.

#### Kabul Kriterleri

1. WHEN Kiro agent yürütmesi tamamlandığında (`agentStop` olayı), THE Hook SHALL `scripts/ci/pre_ci_discipline.sh` betiğini çalıştırır.
2. THE Hook SHALL `fail-closed` modda çalışır; kapı başarısızlığı hook yürütmesini durdurur.
3. THE Hook SHALL otomatik kod düzeltmesi yapmaz; yalnızca ihlali raporlar ve geliştiriciden manuel müdahale ister.
4. THE Hook SHALL mevcut `ci-gate-simulation.kiro.hook` ile çakışmaz; ikisi birbirini tamamlar.
5. WHERE hook devre dışı bırakılmak istendiğinde, THE Hook SHALL `"enabled": false` ayarıyla devre dışı bırakılabilir; bu değişiklik git commit mesajında belgelenmelidir.

### Gereksinim 4: Makefile Entegrasyonu

**Kullanıcı Hikayesi:** Bir geliştirici olarak, `make pre-ci` komutunu çalıştırarak pre-ci disiplin katmanını başlatabilmek istiyorum; böylece build sistemiyle tutarlı bir arayüz kullanırım.

#### Kabul Kriterleri

1. THE Makefile SHALL `pre-ci` hedefini `scripts/ci/pre_ci_discipline.sh` betiğine delege eder.
2. WHEN `make pre-ci` çalıştırıldığında, THE Pre_CI_Discipline SHALL dört kapıyı sırayla çalıştırır (Gereksinim 1 ile aynı sıra).
3. THE Makefile `pre-ci` hedefi SHALL CI'ın yerini almaz; `make ci` ve `make ci-freeze` hedefleri bağımsız olarak çalışmaya devam eder.
4. IF `KERNEL_PROFILE` ortam değişkeni ayarlanmamışsa, THEN THE Pre_CI_Discipline SHALL varsayılan olarak `release` profilini kullanır.

### Gereksinim 5: CI Bağımsızlığı ve Danışma Statüsü

**Kullanıcı Hikayesi:** Bir sistem mimarı olarak, pre-ci disiplin katmanının gerçek CI'ın yerini almadığının açıkça belgelenmesini istiyorum; böylece geliştiriciler pre-ci geçişini merge için yeterli saymaz.

#### Kabul Kriterleri

1. THE Pre_CI_Discipline SHALL başarılı tamamlanma mesajında "Real CI remains mandatory for merge." ifadesini içerir.
2. THE Pre_CI_Discipline SHALL yalnızca dört temel kapıyı çalıştırır: ABI, Boundary, Hygiene, Constitutional; runtime kapıları (Ring0 Exports, Workspace, Syscall v2, Sched Bridge, Policy Accept, Performance) çalıştırmaz.
3. THE Pre_CI_Discipline SHALL `make ci-freeze` veya `make ci` hedeflerini tetiklemez.
4. WHEN pre-ci tüm kapıları geçtiğinde, THE Pre_CI_Discipline SHALL "Local discipline satisfied." mesajını çıktıya yazar; bu mesaj CI geçişi anlamına gelmez.

### Gereksinim 6: Betik Kararlılığı ve Yeniden Üretilebilirlik

**Kullanıcı Hikayesi:** Bir geliştirici olarak, pre-ci disiplin betiğinin deterministik ve yeniden üretilebilir şekilde çalışmasını istiyorum; böylece aynı kaynak durumunda her zaman aynı sonucu alırım.

#### Kabul Kriterleri

1. THE Pre_CI_Discipline SHALL `set -euo pipefail` ile çalışır; beklenmedik hatalarda sessizce devam etmez.
2. THE Pre_CI_Discipline SHALL harici durum veya zamanlama bağımlılığı içermez; yalnızca `make ci-gate-*` komutlarının çıkış kodlarına dayanır.
3. WHEN aynı kaynak durumunda iki kez çalıştırıldığında, THE Pre_CI_Discipline SHALL aynı kapı sonuçlarını üretir.
4. THE Pre_CI_Discipline SHALL `EVIDENCE_ROOT` ortam değişkenini destekler; ayarlanmamışsa `out/evidence` varsayılanını kullanır.

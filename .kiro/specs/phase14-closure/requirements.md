# Gereksinimler Belgesi — Phase-14 Resmi Kapanışı

## Giriş

Bu belge, AykenOS Phase-14 (Distributed Observability Hardening) fazının resmi
kapanış sürecini tanımlar. Tüm Phase-14 iş akışları (3.1–3.5) `main` üzerinde
birleştirilmiştir. Kapanış, belirlenmiş kriterlerin tamamının karşılandığını
kanıtlayan bir süreçtir; bu süreç sonunda `CURRENT_PHASE=14` durumu
`CLOSED (official closure confirmed)` olarak işaretlenir ve Phase-15 geçişi
etkinleştirilir.

Kapanış sürecinin kısa doğru cümlesi:

`Phase-14 KAPALI iff tüm kriterler karşılandı + uzak ci-freeze onayı alındı`

---

## Sözlük

- **Phase-14**: AykenOS Distributed Observability Hardening fazı; iş akışları
  3.1–3.5'i kapsar.
- **ci-freeze**: `make ci-freeze` ile tetiklenen katı CI kapı zinciri; tüm
  uygulanan kapıları çalıştırır ve PASS/FAIL sonucu üretir.
- **Kapanış Adayı Paketi**: Tracker anlık görüntüsü, test özetleri, endpoint
  listesi, yasak alan negatif test özeti, CI çalışma kimliği ve HEAD SHA'yı
  içeren tek dizin.
- **Gözlemlenebilirlik Sınırı Kanıtı**: Diagnostics yüzeyinin otorite/karar/
  sıralama semantiği üretmediğini belgeleyen kanıt.
- **İş Akışı Doğrulama Matrisi**: Her iş akışı için sözleşme belgesi, endpoint/
  yüzey, şema kapsamı, negatif test, PASS kanıtı ve birleştirme durumunu
  içeren tablo.
- **Truth Surface Sync**: Tracker, README, mimari harita ve yol haritası
  dosyalarının tutarlı durumu yansıtması.
- **proofd**: Phase-14 doğrulama ve diagnostics hizmeti;
  `userspace/proofd/src/lib.rs` ana uygulama kaynağı.
- **obs-cli**: `userspace/obs-cli/` — `machine_structured` projeksiyonunu
  tüketen salt okunur CLI tüketici kasası.
- **Yasak Alan**: `score`, `winner`, `routing_hint`, `resolved_truth` gibi
  otorite/karar semantiği taşıyan ve diagnostics yanıtlarında bulunmaması
  gereken alanlar.
- **Kapanış Kriterleri Belgesi**: Phase-14 kapanışının hangi koşullar altında
  gerçekleştiğini açıklayan normatif belge.
- **Faz Geçiş Kuralı**: `CURRENT_PHASE` değerinin güncellenmesini yöneten kural.
- **Kapanış Otoritesi**: Kapanış kararını veren mekanizma; yalnızca uzak `ci-freeze` PASS sonucu ve ilişkili HEAD SHA.
- **Kanıt Öncelik Sırası**: CI kanıtı > çalışma zamanı testleri > sözleşme uyumluluğu > dokümantasyon.
- **Başarısızlık Modu**: Kapanışı engelleyen koşul; kısmi kapanış yoktur.
- **Kapsam Dondurma**: Kapanış sonrası yeni özellik ekleme yasağı.
- **Kapanış Sonrası Değişmez**: Kapanış sonrası değiştirilemez mimari kural.
- **Kapanış Karar Kaydı**: Kapanış onaylandığında oluşturulan değiştirilemez kayıt; HEAD SHA, CI çalışma kimliği, zaman damgası içerir.
- **Yeniden Üretilebilirlik Garantisi**: Aynı HEAD SHA + ortam → özdeş ci-freeze sonucu.
- **Kapanış Geçersizleştirme**: Kapanış sonrası regresyon tespitinde kapanışı ACTIVE'e döndüren kural.
- **Yürütme Sorumluluk Sınırı**: Geliştirici / CI sistemi / depo otoritesi arasındaki sorumluluk bölünmesi; tek aktör kapanışı tamamlayamaz.
- **Zaman Tutarlılığı Kısıtı**: Tüm kapanış artefaktlarının UTC zaman damgası kullanması ve tutarlı olması zorunluluğu.
- **Tek Doğru Kaynak**: `closure_index.json` — tüm türetilmiş belgeler üzerinde öncelikli kapanış gerçeği kaynağı.

---

## Kapanış Otoritesi

Phase-14 kapanış otoritesi yalnızca şu iki koşulun birlikte sağlanmasıyla belirlenir:

- Uzak `ci-freeze` (GitHub Actions) — PASS sonucu
- PASS sonucuyla ilişkili HEAD SHA

Yerel çalışmalar yalnızca danışma niteliğindedir ve kapanış otoritesi vermez.
`make ci-freeze-local` PASS olsa bile bu, resmi kapanış için yeterli değildir.

---

## Kanıt Öncelik Sırası

Kapanış doğrulaması aşağıdaki katı öncelik sırasını izler:

1. CI kanıtı (`ci-freeze` uzak çalışma kimliği ve PASS sonucu)
2. Çalışma zamanı test sonuçları (proofd + obs-cli test sayıları)
3. Sözleşme uyumluluğu (endpoint şema doğrulaması, yasak alan taraması)
4. Dokümantasyon (tracker, README, mimari harita)

Dokümantasyon tek başına kapanış için yeterli değildir. CI kanıtı olmadan
kapanış iddiası geçersizdir.

---

## İş Akışı Bağımlılık İlişkisi

Phase-14 iş akışları arasında aşağıdaki bağımlılık ilişkisi mevcuttur:

- WS 3.5 (Observability UX), WS 3.3'e (şema uygulaması) bağımlıdır:
  `GET /diagnostics/summary` şema doğrulaması `api_schema.rs` altyapısını kullanır.
- WS 3.5 (Observability UX), WS 3.4'e (graph yüzeyi) bağımlıdır:
  `build_root_summary_diagnostics` `build_partitioned_root_graph_diagnostics` ve
  `build_root_graph_overlay_diagnostics` fonksiyonlarını çağırır.
- WS 3.4 (Cross-Node Graph), WS 3.3'e (sözleşme kayıt defteri) bağımlıdır:
  graph endpoint'leri `api_contract.rs` kayıt defterinden çözümlenir.

Kapanış, bu bağımlılık bütünlüğünü doğrulamalıdır: WS 3.3 olmadan WS 3.5
şema uygulaması çalışmaz; WS 3.4 olmadan WS 3.5 overlay verileri üretemez.

---

## Başarısızlık Modları

Kapanış aşağıdaki koşulların herhangi birinde reddedilir:

- Herhangi bir `ci-freeze` kapısı başarısız olursa
- Diagnostics yanıtında herhangi bir yasak alan tespit edilirse
- Truth yüzeyleri (tracker, README, mimari harita) birbiriyle çelişiyorsa
- Doğrulama matrisinde herhangi bir iş akışı için PASS kanıtı eksikse
- Endpoint/şema uyumsuzluğu tespit edilirse
- Uzak `ci-freeze` onayı alınmadan kapanış iddiasında bulunulursa
- Hygiene kapısı başarısız olursa (dirty tracked dosyalar mevcutsa)

Bu koşulların herhangi biri kapanışı engeller; kısmi kapanış yoktur.

---

## Kapanış Sonrası Değişmezler

Phase-14 resmi olarak kapandıktan sonra aşağıdaki değişmezler geçerlidir:

- Observability sözleşmeleri değiştirilemez hale gelir; herhangi bir değişiklik
  yeni bir faz gerektirir.
- Diagnostics yüzeyleri otorite dışı (`non_authoritative`) olarak kalır;
  bu sınıflandırma geriye dönük olarak değiştirilemez.
- Geriye dönük olarak sıralama/puanlama semantiği eklenemez.
- `service != authority`, `diagnostics != decision`, `parity != consensus`
  değişmezleri Phase-15 ve sonrasında da geçerliliğini korur.
- `phase14-official-closure-confirmed` etiketi değiştirilemez; yeni bir
  kapanış iddiası yeni bir etiket gerektirir.

---

## Kapsam Dondurma

Phase-14 resmi olarak kapandıktan sonra bu faz kapsamına yeni gözlemlenebilirlik
özellikleri eklenemez. Kapanış sonrası izin verilen değişiklikler yalnızca
şunlardır:

- Hata düzeltmeleri (sözleşmeyi etkilemeyen)
- Dokümantasyon güncellemeleri
- Sözleşmeyi etkilemeyen performans iyileştirmeleri

Yeni endpoint, yeni şema alanı veya yeni sözleşme semantiği eklemek Phase-15
veya sonraki bir faz gerektirir.

---

## Gereksinimler

### Gereksinim 1: Truth Surface Senkronizasyonu

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, tüm Phase-14 truth
yüzeylerinin tutarlı ve güncel olmasını istiyorum; böylece kapanış kararı
doğru bir temele dayanır.

#### Kabul Kriterleri

1. THE Tracker SHALL `PHASE14_DEVELOPMENT_TRACKER.md` dosyasında tüm iş
   akışlarını (3.1–3.5) `MERGED` durumunda kaydetmiş olmalıdır.
2. THE Tracker SHALL `docs/roadmap/CURRENT_PHASE` dosyasının
   `CURRENT_PHASE=14` değerini içerdiğini doğrulamalıdır.
3. WHEN truth surface sync tamamlandığında, THE Tracker SHALL son güncelleme
   tarihini ve HEAD SHA'yı kaydetmelidir.
4. IF herhangi bir truth yüzeyi (README, mimari harita, tracker) birbiriyle
   çelişiyorsa, THEN THE Sistem SHALL kapanış sürecini engellemeli ve
   tutarsızlığı raporlamalıdır.

---

### Gereksinim 2: İş Akışı Doğrulama Matrisi

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, her Phase-14 iş akışının
sözleşme belgesi, endpoint yüzeyi, şema kapsamı, negatif test ve PASS kanıtı
ile birlikte belgelenmiş olmasını istiyorum; böylece kapanış iddiası
doğrulanabilir kanıta dayanır.

#### Kabul Kriterleri

1. THE Matris SHALL her iş akışı (3.1, 3.2, 3.3, 3.4, 3.5) için aşağıdaki
   sütunları içermelidir: iş akışı kimliği, sözleşme belgesi yolu, endpoint/
   yüzey listesi, şema kapsamı beyanı, negatif test referansı, PASS kanıtı
   (CI çalışma kimliği veya yerel kanıt referansı), birleştirme durumu.
2. WHEN matris oluşturulduğunda, THE Matris SHALL her iş akışı için birleştirme
   durumunun `MERGED` olduğunu doğrulamalıdır.
3. THE Matris SHALL WS 3.1 için `GET /diagnostics/version` ve
   `X-Ayken-API-Version` başlığını kapsayan sözleşme kanıtını içermelidir.
4. THE Matris SHALL WS 3.2 için `verification_determinism_contract.json`
   üretimini ve `ci-gate-determinism-replay-consistency` PASS kanıtını
   içermelidir.
5. THE Matris SHALL WS 3.3 için `api_contract.rs` ve `api_schema.rs` kaynak
   referanslarını, yasak alan çalışma zamanı uygulamasını ve
   `ci-gate-proofd-schema-coverage` PASS kanıtını içermelidir.
6. THE Matris SHALL WS 3.4 için `CROSS_NODE_OBSERVABILITY_GRAPH_CONTRACT_v1.md`
   referansını, `GET /diagnostics/graph` ve `GET /diagnostics/graph/overlay`
   endpoint kanıtını ve `ci-gate-graph-non-authoritative-contract` PASS
   kanıtını içermelidir.
7. THE Matris SHALL WS 3.5 için `OBSERVABILITY_UX_CONTRACT_v1.md` referansını,
   `GET /diagnostics/summary` ve `GET /diagnostics/runs/{run_id}/summary`
   endpoint kanıtını, `obs-cli` tüketici kasası doğrulamasını (63 test PASS)
   ve `210 lib + 6 main` test PASS kanıtını içermelidir.
8. IF herhangi bir iş akışı için PASS kanıtı eksikse, THEN THE Sistem SHALL
   matrisi eksik olarak işaretlemeli ve kapanış sürecini engellemeli.
9. THE Matris SHALL WS 3.3 → WS 3.4 → WS 3.5 bağımlılık zincirinin bütünlüğünü
   doğrulamalıdır: WS 3.5 şema doğrulaması WS 3.3 altyapısına, overlay verileri
   WS 3.4 yüzeyine bağımlıdır.

---

### Gereksinim 3: Gözlemlenebilirlik Sınırı Kanıtı

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, diagnostics yüzeyinin
otorite, karar veya sıralama semantiği üretmediğini kanıtlamak istiyorum;
böylece `diagnostics != decision` ve `service != authority` değişmezleri
kapanışta belgelenmiş olur.

#### Kabul Kriterleri

1. THE Kanıt Belgesi SHALL aşağıdaki değişmezlerin her birinin neden geçerli
   olduğunu açıklamalıdır: `service != authority`, `diagnostics != decision`,
   `parity != consensus`, `trust does not affect verdict`,
   `observability does not imply scheduling`.
2. THE Kanıt Belgesi SHALL her public diagnostics endpoint için
   `produces_truth=false`, `produces_decision=false`,
   `produces_ranking=false` epistemic sınır beyanını kaydetmelidir.
3. THE Kanıt Belgesi SHALL yasak alan listesini (`score`, `winner`,
   `routing_hint`, `resolved_truth`, `recommended_action` vb.) ve bu alanların
   çalışma zamanında nasıl reddedildiğini (`500 forbidden_observability_field_exposed`)
   belgelemelidir.
4. WHEN bir negatif test çalıştırıldığında, THE Sistem SHALL yasak alan içeren
   bir yanıtın `500 forbidden_observability_field_exposed` hatasıyla
   reddedildiğini doğrulamalıdır.
5. THE Kanıt Belgesi SHALL `ci-gate-proofd-observability-boundary` ve
   `ci-gate-observability-routing-separation` kapılarının PASS durumunu
   kaydetmelidir.
6. IF diagnostics yanıtı herhangi bir yasak alan içeriyorsa, THEN THE proofd
   SHALL yanıtı `500 forbidden_observability_field_exposed` ile reddetmelidir.

---

### Gereksinim 4: CI Freeze Otoritesi

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, `make ci-freeze` komutunun
PASS sonucu üretmesini istiyorum; böylece kapanış resmi hale gelir.

#### Kabul Kriterleri

1. WHEN `make ci-freeze` çalıştırıldığında, THE CI Sistemi SHALL tüm uygulanan
   kapıları sırayla çalıştırmalıdır.
2. THE CI Sistemi SHALL `ci-freeze` çalışmasının PASS sonucu ürettiğini
   doğrulamalıdır; bu sonuç uzak GitHub Actions iş akışı çalışma kimliği ile
   kanıtlanmalıdır.
3. THE CI Sistemi SHALL `ci-freeze` çalışmasının HEAD SHA'sını kaydetmelidir.
4. IF `ci-freeze` herhangi bir kapıda başarısız olursa, THEN THE Sistem SHALL
   başarısız kapıyı ve hata mesajını raporlamalı ve kapanış sürecini
   engellemeli.
5. THE CI Sistemi SHALL Phase-14'e özgü kapıların (`ci-gate-proofd-service`,
   `ci-gate-proofd-schema-coverage`, `ci-gate-proofd-observability-boundary`,
   `ci-gate-observability-routing-separation`,
   `ci-gate-graph-non-authoritative-contract`,
   `ci-gate-diagnostics-consumer-non-authoritative-contract`,
   `ci-gate-determinism-replay-consistency`) tamamının PASS durumunda
   olduğunu doğrulamalıdır.
6. WHILE `ci-freeze` çalışırken, THE CI Sistemi SHALL `AYKEN_SCHED_FALLBACK=0`
   ve `AYKEN_CR3_PCID=0` kısıtlamalarını uygulamalıdır.
7. THE CI Sistemi SHALL yerel `ci-freeze-local` çalışmasının kapanış otoritesi
   vermediğini doğrulamalıdır; yalnızca uzak GitHub Actions çalışması kapanış
   için geçerlidir.

---

### Gereksinim 5: Kapanış Kriterleri Belgesi

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, Phase-14 kapanışının hangi
koşullar altında gerçekleştiğini açıklayan normatif bir belge istiyorum; böylece
kapanış kararı belirsizlik içermez.

#### Kabul Kriterleri

1. THE Kapanış Kriterleri Belgesi SHALL Phase-14'ün resmi olarak kapalı
   sayılması için gereken tüm koşulları açıkça listelemelidir.
2. THE Kapanış Kriterleri Belgesi SHALL faz geçiş kuralını içermelidir:
   `Phase-14 KAPALI iff tüm kriterler karşılandı + uzak ci-freeze onayı alındı`.
3. THE Kapanış Kriterleri Belgesi SHALL her kriterin durumunu (TAMAMLANDI /
   BEKLEMEDE) kaydetmelidir.
4. THE Kapanış Kriterleri Belgesi SHALL kapanış iddiasını, HEAD SHA'yı ve CI
   çalışma kimliğini içermelidir.
5. IF herhangi bir kriter karşılanmamışsa, THEN THE Belge SHALL kapanışın
   gerçekleşmediğini açıkça belirtmeli ve eksik kriterleri listelemelidir.

---

### Gereksinim 6: Kapanış Adayı Paketi

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, Phase-14 kapanışını
kanıtlayan tüm artefaktları tek bir dizinde toplamak istiyorum; böylece
kapanış iddiası bağımsız olarak doğrulanabilir.

#### Kabul Kriterleri

1. THE Paket SHALL tek bir dizin altında aşağıdaki artefaktları içermelidir:
   tracker anlık görüntüsü, test özetleri (proofd + obs-cli), endpoint listesi,
   yasak alan negatif test özeti, CI çalışma kimliği, HEAD SHA, kapanış iddiası.
2. THE Paket SHALL `closure_index.json` dosyasını içermelidir; bu dosya tüm
   artefaktlara referans vermelidir.
3. WHEN paket oluşturulduğunda, THE Sistem SHALL `closure_index.json`
   dosyasının tüm referans edilen artefaktların mevcut olduğunu doğrulamalıdır.
4. THE Paket SHALL Phase-13 kapanış paketi yapısıyla (`reports/phase13_official_closure_candidate/`)
   tutarlı bir dizin yapısı kullanmalıdır.
5. IF herhangi bir zorunlu artefakt eksikse, THEN THE Sistem SHALL paketi
   eksik olarak işaretlemeli ve eksik artefaktları listelemelidir.
6. THE Paket SHALL `phase14-official-closure-confirmed` etiketinin hangi
   commit SHA'sına uygulanacağını kaydetmelidir.

---

### Gereksinim 7: Faz Geçiş Kuralı

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, Phase-14'ten Phase-15'e
geçişin yalnızca tüm kapanış kriterleri karşılandıktan sonra gerçekleşmesini
istiyorum; böylece erken geçiş riski ortadan kalkar.

#### Kabul Kriterleri

1. THE Sistem SHALL `CURRENT_PHASE` değerini yalnızca tüm kapanış kriterleri
   karşılandıktan ve uzak `ci-freeze` onayı alındıktan sonra güncellemeli.
2. WHEN faz geçişi gerçekleştiğinde, THE Sistem SHALL `docs/roadmap/CURRENT_PHASE`
   dosyasını `CURRENT_PHASE=15` olarak güncellemelidir.
3. THE Sistem SHALL `phase14-official-closure-confirmed` etiketini kapanış
   HEAD SHA'sına uygulamalıdır.
4. IF kapanış kriterleri karşılanmadan faz geçişi denenirse, THEN THE Sistem
   SHALL geçişi engellemeli ve eksik kriterleri raporlamalıdır.
5. THE Sistem SHALL faz geçiş kaydını `PHASE14_DEVELOPMENT_TRACKER.md`
   dosyasına eklemelidir; bu kayıt kapanış tarihini, HEAD SHA'yı ve CI
   çalışma kimliğini içermelidir.

---

### Gereksinim 8: Kapanış Sonrası Değişmezler

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, Phase-14 kapandıktan sonra
observability sözleşmelerinin ve sınır kurallarının değiştirilemez hale gelmesini
istiyorum; böylece kapanış sonrası mimari bütünlük korunur.

#### Kabul Kriterleri

1. WHEN Phase-14 resmi olarak kapandığında, THE Sistem SHALL observability
   sözleşmelerini (`OBSERVABILITY_UX_CONTRACT_v1.md`,
   `CROSS_NODE_OBSERVABILITY_GRAPH_CONTRACT_v1.md`,
   `PROOFD_EXTERNAL_DIAGNOSTICS_CONTRACT_v1.md`) değiştirilemez olarak
   işaretlemelidir; herhangi bir değişiklik yeni bir faz gerektirir.
2. THE Sistem SHALL `non_authoritative` sınıflandırmasının geriye dönük olarak
   değiştirilemeyeceğini kaydetmelidir.
3. THE Sistem SHALL `service != authority`, `diagnostics != decision`,
   `parity != consensus` değişmezlerinin Phase-15 ve sonrasında da geçerli
   olduğunu belgelemelidir.
4. THE Sistem SHALL `phase14-official-closure-confirmed` etiketinin
   değiştirilemez olduğunu kaydetmelidir; yeni bir kapanış iddiası yeni bir
   etiket gerektirir.
5. IF kapanış sonrası herhangi bir sözleşme değişikliği denenirse, THEN THE
   Sistem SHALL değişikliği reddedip yeni faz gereksinimini raporlamalıdır.

---

### Gereksinim 9: Kapsam Dondurma

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, Phase-14 kapandıktan sonra
bu faz kapsamına yeni gözlemlenebilirlik özelliklerinin eklenmesini engellemek
istiyorum; böylece kapanış bütünlüğü korunur.

#### Kabul Kriterleri

1. WHEN Phase-14 resmi olarak kapandığında, THE Sistem SHALL bu faz kapsamına
   yeni endpoint, yeni şema alanı veya yeni sözleşme semantiği eklenmesini
   engellemeli.
2. THE Sistem SHALL kapanış sonrası izin verilen değişiklikleri açıkça
   tanımlamalıdır: hata düzeltmeleri (sözleşmeyi etkilemeyen), dokümantasyon
   güncellemeleri, sözleşmeyi etkilemeyen performans iyileştirmeleri.
3. IF kapanış sonrası yeni bir observability özelliği eklenmek istenirse,
   THEN THE Sistem SHALL bu değişikliğin Phase-15 veya sonraki bir faz
   gerektirdiğini raporlamalıdır.
4. THE Sistem SHALL kapsam dondurma kuralını `ARCHITECTURE_FREEZE.md`
   dosyasına kaydetmelidir.

---

### Gereksinim 10: Kapanış Karar Kaydı

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, Phase-14 kapanışının
değiştirilemez bir karar kaydıyla belgelenmesini istiyorum; böylece kapanış
kararı bağımsız olarak denetlenebilir.

#### Kabul Kriterleri

1. WHEN Phase-14 kapanışı onaylandığında, THE Sistem SHALL aşağıdaki alanları
   içeren bir kapanış karar kaydı oluşturmalıdır: HEAD SHA, `ci-freeze` çalışma
   kimliği, kapanış zaman damgası (UTC), kapanış kararı (`PASS`), kapanış adayı
   paketine referans.
2. THE Kapanış Karar Kaydı SHALL değiştirilemez olmalı ve kapanış artefaktlarıyla
   birlikte saklanmalıdır.
3. THE Kapanış Karar Kaydı SHALL `closure_decision_record.json` dosyası olarak
   `reports/phase14_official_closure_candidate/` dizininde yer almalıdır.
4. IF kapanış karar kaydı eksikse veya bozuksa, THEN THE Sistem SHALL kapanışı
   geçersiz saymalı ve kaydın yeniden oluşturulmasını talep etmelidir.
5. THE Kapanış Karar Kaydı SHALL Phase-13 kapanış kaydıyla (`reports/phase13_official_closure_candidate/closure_manifest.json`)
   tutarlı bir format kullanmalıdır.

---

### Gereksinim 11: Yeniden Üretilebilirlik Garantisi

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, Phase-14 kapanışının
deterministik ve yeniden üretilebilir olmasını istiyorum; böylece aynı HEAD
SHA ve ortam koşulları her zaman aynı `ci-freeze` sonucunu üretir.

#### Kabul Kriterleri

1. THE Sistem SHALL aynı HEAD SHA ve aynı ortam koşullarının özdeş `ci-freeze`
   sonuçları ürettiğini doğrulamalıdır.
2. THE Sistem SHALL `ci-gate-determinism-replay-consistency` kapısının
   yeniden üretilebilirlik garantisini doğruladığını kaydetmelidir.
3. IF aynı HEAD SHA ile farklı `ci-freeze` sonuçları elde edilirse, THEN THE
   Sistem SHALL bu sapmayı ihlal olarak raporlamalı ve kapanışı engellemeli.
4. THE Sistem SHALL kapanış kanıtının `DETERMINISM.GLOBAL` NON_OVERRIDABLE
   kuralına uygun olduğunu doğrulamalıdır.
5. THE Sistem SHALL yeniden üretilebilirlik garantisini kapanış karar kaydına
   (`closure_decision_record.json`) eklenecek `reproducibility_verified: true`
   alanıyla belgelemelidir.

---

### Gereksinim 12: Kapanış Geçersizleştirme Kuralı

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, kapanış sonrası tespit
edilen regresyonların Phase-14 kapanışını geçersiz kılmasını istiyorum; böylece
hatalı bir kapanış kalıcı hale gelmez.

#### Kabul Kriterleri

1. THE Sistem SHALL Phase-14 kapanışını aşağıdaki koşulların herhangi birinde
   geçersiz saymalıdır: kapanış sonrası `ci-freeze` observability sözleşmelerinde
   regresyon tespit ederse; diagnostics yanıtında yasak alan ortaya çıkarsa;
   truth yüzeylerinde sürüklenme tespit edilirse.
2. WHEN kapanış geçersizleştirildiğinde, THE Sistem SHALL `phase14-official-closure-confirmed`
   etiketini geçersiz olarak işaretlemeli ve Phase-14'ü `ACTIVE` durumuna
   döndürmelidir.
3. THE Sistem SHALL geçersizleştirme nedenini ve tespit zamanını
   `PHASE14_DEVELOPMENT_TRACKER.md` dosyasına kaydetmelidir.
4. IF kapanış geçersizleştirilirse, THEN THE Sistem SHALL yeni bir kapanış
   döngüsünün başlatılması gerektiğini raporlamalıdır; mevcut kapanış adayı
   paketi geçersiz sayılır.
5. THE Sistem SHALL geçersizleştirme kuralının `ARCHITECTURE_FREEZE.md`
   dosyasındaki mimari yönetim ilkeleriyle tutarlı olduğunu doğrulamalıdır.

---

### Gereksinim 13: Denetlenebilirlik

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, tüm kapanışla ilgili
artefaktların iç sistem durumuna bağımlı olmadan bağımsız olarak
doğrulanabilmesini istiyorum.

#### Kabul Kriterleri

1. THE Sistem SHALL tüm kapanış artefaktlarının (kapanış karar kaydı, test
   özetleri, CI kanıtı, endpoint listesi) iç sistem durumuna erişim
   gerektirmeden doğrulanabilir olmasını sağlamalıdır.
2. THE Kapanış Adayı Paketi SHALL `closure_index.json` aracılığıyla tüm
   artefaktlara bağımsız referans sağlamalıdır.
3. THE Sistem SHALL kapanış kanıtının SHA-256 özet değerlerini
   `closure_index.json` dosyasına kaydetmelidir.
4. IF herhangi bir artefaktın özet değeri eşleşmiyorsa, THEN THE Sistem SHALL
   denetim bütünlüğü ihlali raporlamalı ve kapanışı geçersiz saymalıdır.

---

### Gereksinim 14: Yürütme Sorumluluk Sınırı

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, kapanış sürecinin hiçbir
aktör tarafından tek taraflı olarak tamamlanamamasını istiyorum; böylece insan
hatasıyla kapanış riski sıfırlanır.

#### Kabul Kriterleri

1. THE Sistem SHALL kapanış sürecini üç sorumluluk katmanına bölmelidir:
   geliştirici (paket hazırlama + truth sync + yerel pre-ci), CI sistemi
   (uzak ci-freeze + yetkili karar + HEAD SHA bağlama), depo otoritesi
   (etiket uygulama + CURRENT_PHASE güncelleme + karar kaydı).
2. THE Sistem SHALL CI PASS olmadan kapanış etiketinin uygulanamayacağını
   zorunlu kılmalıdır.
3. THE Sistem SHALL kapanış etiketinin uygulanmadan `CURRENT_PHASE`'in
   güncellenemeyeceğini zorunlu kılmalıdır.
4. IF herhangi bir aktör kapanışı tek başına tamamlamaya çalışırsa, THEN THE
   Sistem SHALL bu girişimi engellemeli ve eksik sorumluluk katmanını
   raporlamalıdır.

---

### Gereksinim 15: Zaman Tutarlılığı Kısıtı

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, tüm kapanış artefaktlarının
tutarlı UTC zaman damgaları kullanmasını istiyorum; böylece dağıtık sistemde
zaman tutarsızlığından kaynaklanan geçersizleştirme riski ortadan kalkar.

#### Kabul Kriterleri

1. THE Sistem SHALL tüm kapanış artefaktlarında UTC zaman damgası kullanmalıdır;
   format ISO 8601: `2026-04-07T23:09:55Z`.
2. THE `closure_decision_record.json` içindeki `closure_timestamp_utc` alanı
   SHALL `ci-freeze` tamamlanma zamanına eşit veya daha sonra olmalıdır.
3. THE Sistem SHALL tüm artefaktlar arasındaki zaman damgası tutarlılığını
   doğrulamalıdır (±60 saniye tolerans).
4. IF zaman damgaları birbiriyle tutarsızsa, THEN THE Sistem SHALL kapanışı
   geçersiz saymalı ve tutarsızlığı raporlamalıdır.

---

### Gereksinim 16: Tek Doğru Kaynak İşaretçisi

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, Phase-14 kapanış gerçeğinin
kesin kaynağının açıkça tanımlanmasını istiyorum; böylece çelişki durumunda
hangi belgenin geçerli olduğu belirsizlik içermez.

#### Kabul Kriterleri

1. THE Sistem SHALL `reports/phase14_official_closure_candidate/closure_index.json`
   dosyasını Phase-14 kapanış gerçeğinin kesin kaynağı olarak tanımlamalıdır.
2. THE Sistem SHALL diğer tüm belgelerin (tracker, README, raporlar) türetilmiş
   görünümler olduğunu kaydetmelidir.
3. IF herhangi bir türetilmiş belge `closure_index.json` ile çelişiyorsa, THEN
   THE Sistem SHALL `closure_index.json`'ı geçerli kaynak olarak kabul etmeli
   ve çelişkiyi raporlamalıdır.
4. THE Sistem SHALL `closure_index.json` dosyasının SHA-256 özetini
   bağımsız doğrulama için kaydetmelidir.

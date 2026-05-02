# Gereksinimler Belgesi — Phase-15: BCIB Execution Engine v3 & Workstream Roadmap

**Belge Türü:** Normatif
**Faz:** Phase-15
**Durum:** DRAFT
**Hazırlayan / Oluşturan:** Kenan AY
**Geliştiren / Düzenleyen:** Kenan AY
**Dijital İmza:** Kenan AY
**Oluşturma Tarihi:** 2026-04-08
**Son Güncelleme:** 2026-04-08

---

## Giriş

Bu belge, AykenOS Phase-15 roadmap'ini tanımlar. Phase-15'in merkezi iş akışı
**WS 3.1: BCIB Execution Engine v3**'tür; diğer tüm workstream'ler bu çekirdeğe
bağımlı sırayla yürütülür.

Phase-14 kapanış değişmezleri (`service != authority`, `diagnostics != decision`,
`parity != consensus`) Phase-15 boyunca geçerliliğini korur ve BCIB v3'ün
observability yüzeyi bu değişmezlere uymalıdır.

**BCIB v3 evolution ilkesi:** BCIB v3, v0.2 executor'ın sıfırdan yeniden yazımı
değildir; mevcut `userspace/bcib-runtime/` altyapısı üzerine inşa edilen
evolution'dır. v0.2 semantiğiyle backward-compatible veya fail-closed.

**Mevcut altyapı:**
- `userspace/bcib-runtime/` — BCIB v0.2 executor (Phase 2.3, TAMAMLANDI)
- `userspace/dsl-parser/` — DSL parser (mevcut)
- `userspace/semantic-cli/` — Semantic CLI (mevcut)
- `userspace/ai-runtime/` — AI runtime (mevcut)
- `userspace/orchestration/` — Orchestration katmanı (mevcut)
- `userspace/proofd/` + `userspace/obs-cli/` — Phase-14 observability (IMMUTABLE)

**Phase matrix (mevcut P4.4 Dev):**

| Kural | P4.4 (Dev) | P4.5 (Stab) | P5 (Prod) |
|-------|-----------|------------|---------|
| `DETERMINISM.GLOBAL` | ERROR | ERROR | ERROR |
| `MEMORY.CONTRACT.VIOLATION` | ERROR | ERROR | ERROR |
| `KERNEL.SAFETY.CRITICAL` | ERROR | ERROR | ERROR |
| `KERNEL.CAPABILITY.BYPASS` | ERROR | ERROR | ERROR |
| `ALLOC.HEAP_DIRECT` | ALLOW | ALLOW | WARN |
| `ERROR.PANIC` | ERROR | ERROR | ERROR |
| `MEMORY.LEAK` | WARN | ERROR | ERROR |
| `ALLOC.GLOBAL` | ALLOW | WARN | ERROR |

---

## Sözlük

- **BCIB**: Binary Compressed Instruction Buffer — AykenOS'un execution-centric yürütme birimi formatı.
- **BCIB_Executor**: `userspace/bcib-runtime/` içindeki Ring3 yürütme motoru; üç ayrı sorumluluk taşır ve bunlar birbirinden ayrı tutulmalıdır: (1) **BCIB_Verifier/Planner** — BCIB grafiğini doğrular, control-flow analizi yapar ve yürütme planı üretir; (2) **BCIB_Execution_Runtime** — planlanmış yürütmeyi lifecycle state machine üzerinden yönetir; (3) **Scheduler_Submit_Bridge** — yürütmeyi `SYS_V2_SUBMIT_EXECUTION (1003)` üzerinden kernel'e iletir ve result lifecycle'ı yönetir. Ring3 policy/runtime bileşenidir; kernel-resident policy semantiği içermez.
- **BCIB_v3**: Phase-15'te geliştirilen BCIB Execution Engine'in üçüncü büyük sürümü; v0.2 semantiğiyle backward-compatible veya fail-closed. Sıfırdan rewrite değil, evolution.
- **SYS_V2_SUBMIT_EXECUTION**: Syscall ID 1003 — ABI freeze kapsamında değiştirilemez. BCIB grafiğini Ring0'a iletir; yalnızca execution submission ve result lifecycle bridging için kullanılır.
- **ABI_Freeze**: `kernel/include/ayken_abi.h` kaynaklı, 1000-1010 aralığındaki syscall sözleşmesi; Phase-15 boyunca değiştirilemez.
- **Capability_Manager**: Ring3'te token tabanlı yetki yönetimi; kernel bypass NON_OVERRIDABLE ihlalidir.
- **Execution_Trace**: Bir BCIB grafiğinin yürütülmesi sırasında üretilen deterministik iz kaydı; replay ve doğrulama için kullanılır.
- **Execution_Lifecycle**: Bir BCIB yürütmesinin tam yaşam döngüsü: submit → bounded_slice → yield/wait → resume → complete/cancel.
- **Fail_Closed**: Geçersiz veya desteklenmeyen girdi karşısında sessizce devam etmek yerine açık hata döndürme semantiği.
- **NON_OVERRIDABLE**: `_ayken/steering/NON_OVERRIDABLE.md` kaynaklı mutlak yasaklar; hiçbir Allow/Waiver mekanizması geçersiz kılamaz.
- **Phase_Matrix**: `_ayken/steering/PHASES.md` kaynaklı faz-kural matrisi; her kuralın mevcut fazda ERROR/WARN/ALLOW durumunu belirler.
- **CI_Gate**: `make ci-freeze` zincirindeki otomatik doğrulama kapısı; PASS olmadan merge reddedilir.
- **Governance_Gate**: WS 3.10 kapsamındaki faz geçiş kapısı; tüm workstream tamamlanma kanıtlarını doğrular.
- **Integration_Contract**: İki bileşen arasındaki input/output/fail-mode/ownership sınırını tanımlayan sözleşme.
- **DSL_Parser**: `userspace/dsl-parser/` — AykenOS komut dilini BCIB grafiğine dönüştüren parser.
- **Semantic_CLI**: `userspace/semantic-cli/` — Kullanıcı komutlarını DSL'e çeviren semantik komut satırı arayüzü.
- **AI_Runtime**: `userspace/ai-runtime/` — Ring3'te izole çalışan TinyLLM çıkarım motoru; öneri üretir, otorite değildir.
- **Observability_Surface**: Phase-14 değişmezlerine uygun diagnostics yüzeyi; otorite/karar/sıralama semantiği üretmez.
- **Approved_Runtime_Service_Boundary**: AI runtime ve diğer downstream bileşenlerin kernel mekanizmalarına erişebildiği onaylı servis sınırı; BCIB executor bu sınırın birincil implementasyonudur.
- **Workspace**: `userspace/` altındaki çalışma alanı yönetim katmanı; WS 3.1 çekirdeği tamamlanmadan production-ready sayılamaz.
- **Data_Runtime**: Veri odaklı işlem katmanı; BCIB üzerinden veri sorgularını yürütür.
- **Toolchain**: Derleme, test ve CI araç zinciri; opcode registry, encoder/decoder version lock, golden fixture'ları kapsar.
- **WS**: Workstream — Phase-15 iş akışı birimi.
- **ABDF**: AykenOS Binary Data Format — AykenOS'un yetkili veri substratı; tüm veri nesnelerinin canonical depolama ve erişim sözleşmesini tanımlar. BCIB execution engine'in veri katmanı değildir; BCIB, ABDF-managed nesnelere handle/referans üzerinden erişir.
- **ABDF_Handle**: ABDF tarafından yönetilen bir veri nesnesine tip-güvenli referans; raw pointer değildir, ABDF capability enforcement'ına tabidir.

---

## Workstream Öncelik Hiyerarşisi ve Bağımlılık Grafiği

### Öncelik Sırası (Kritiklik Sırasıyla)

```
Seviye 1 — Çekirdek (blocking):
  WS 3.1: BCIB Execution Engine v3

Seviye 2 — Güvenlik ve Gözlemlenebilirlik (WS 3.1 tamamlanmadan başlanamaz):
  WS 3.7: Capability/Security
  WS 3.8: Observability Integration

Seviye 3 — Kullanıcı Arayüzü Katmanı (WS 3.1 + 3.7 tamamlanmadan production-ready sayılamaz):
  WS 3.2: System DSL
  WS 3.3: Semantic CLI  (WS 3.2'ye bağımlı)

Seviye 4 — Runtime Katmanı (WS 3.1 tamamlanmadan production-ready sayılamaz):
  WS 3.5: Data Runtime
  WS 3.6: AI Runtime

Seviye 5 — Downstream (WS 3.1 çekirdeği tamamlanmadan stabilize edilemez):
  WS 3.4: Workspace

Seviye 6 — Araç Zinciri ve Kapanış:
  WS 3.9: Toolchain/Extensibility
  WS 3.10: Governance Gates  ← TÜM WS'LERİN KAPANIŞ KAPISI
```

### Bağımlılık Grafiği

```
WS 3.1: BCIB Execution Engine v3  ← ÇEKIRDEK
    ├── WS 3.7: Capability/Security       (Seviye 2)
    ├── WS 3.8: Observability Integration (Seviye 2)
    ├── WS 3.2: System DSL                (Seviye 3)
    │       └── WS 3.3: Semantic CLI      (Seviye 3)
    ├── WS 3.5: Data Runtime              (Seviye 4)
    ├── WS 3.6: AI Runtime                (Seviye 4)
    ├── WS 3.4: Workspace                 (Seviye 5, non-blocking for core)
    ├── WS 3.9: Toolchain/Extensibility   (Seviye 6)
    └── WS 3.10: Governance Gates         (Seviye 6, kapanış kapısı)
```

**Bağımlılık kuralı:** WS 3.1 tamamlanmadan hiçbir downstream workstream
production-ready sayılamaz. WS 3.7 ve WS 3.8, WS 3.1 ile eş zamanlı
geliştirilebilir ancak WS 3.1 PASS olmadan CI gate'leri geçemez.

---

## Yapılmaması Gerekenler (SHALL NOT)

Bu bölüm teknik borç önleyici mutlak yasakları tanımlar.

1. **BCIB_Executor SHALL NOT** Ring0'a policy kararı taşımalıdır; syscall etkileşimi yalnızca execution submission ve result lifecycle bridging ile sınırlıdır.
2. **BCIB_Executor SHALL NOT** kernel-resident policy semantiği içermelidir; tüm karar mantığı Ring3'te kalır.
3. **BCIB_Executor SHALL NOT** syscall v2 numaralandırmasını değiştirmelidir; 1000-1010 aralığı ABI freeze kapsamındadır.
4. **BCIB_Executor SHALL NOT** Phase-14 immutable observability sözleşmelerini mutasyona uğratmalıdır.
5. **Downstream workstream'ler SHALL NOT** WS 3.1 çekirdek doğrulamasını bypass etmelidir.
6. **Workspace SHALL NOT** otorite yüzeyi haline gelmelidir; yalnızca execution katmanı üzerinden işlem yapabilir.
7. **AI_Runtime SHALL NOT** karar otoritesi olmalıdır; yalnızca öneri üretir, scheduling veya routing kararı veremez.
8. **AI_Runtime SHALL NOT** kernel mekanizmalarına Approved_Runtime_Service_Boundary dışından erişmelidir.
9. **Toolchain SHALL NOT** opcode ID'lerini yeniden kullanmalıdır; v0.2 opcode ID'leri v3'te rezervedir.
10. **Hiçbir bileşen SHALL NOT** NON_OVERRIDABLE kuralı için Allow veya Waiver mekanizması kullanmalıdır.

---

## Integration Contract Özeti

Her bileşen çifti için input/output/fail-mode/ownership sınırı:

| Bileşen Çifti | Input Contract | Output Contract | Fail Mode | Ownership |
|---------------|---------------|-----------------|-----------|-----------|
| BCIB ↔ DSL_Parser | DSL AST / canonical DSL command stream | Validated BCIB graph (BCIB IR) | fail-closed, `BCIB_ERR_INVALID_GRAPH` | DSL_Parser üretir, BCIB_Executor tüketir |
| BCIB ↔ Semantic_CLI | DSL komutu (string) | Kullanıcı mesajı | hata kodu → açıklayıcı mesaj | CLI kullanıcıya raporlar |
| BCIB ↔ Capability_Manager | Capability token seti | allow/deny kararı | `BCIB_ERR_CAPABILITY_DENIED` | Capability_Manager karar verir |
| BCIB ↔ Scheduler_Bridge | Execution slice talebi | yield/resume sinyali | fail-closed, scheduler'a müdahale yok | Ring0 final arbiter |
| BCIB ↔ proofd/obs-cli | Diagnostics sorgusu | non-authoritative yanıt | `500 forbidden_observability_field_exposed` | Phase-14 IMMUTABLE |
| BCIB ↔ AI_Runtime | Çıkarım isteği + capability | Öneri (non-authoritative) | `AI_ERR_CAPABILITY_DENIED` | AI öneri üretir, BCIB yürütür |
| BCIB ↔ bcib-runtime v0.2 | v0.2 BCIB grafiği | Uyumlu yürütme veya fail-closed | `BCIB_ERR_UNSUPPORTED_VERSION` | v3 executor backward-compat sağlar |
| BCIB ↔ ABDF | ABDF_Handle (veri referansı) | Veri okuma/yazma sonucu (ABDF sözleşmesiyle) | `BCIB_ERR_ABDF_ACCESS_DENIED` | ABDF veri sahibidir; BCIB tüketicidir |

---

---

## Gereksinimler — Seviye 1: Çekirdek Yürütme (Core Execution)

> Bu bölümdeki gereksinimler blocking'dir. WS 3.1 tamamlanmadan hiçbir
> downstream workstream production-ready sayılamaz.

### Gereksinim 1: BCIB_Executor Ring3 Sınırı

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, BCIB_Executor'ın Ring3
policy/runtime bileşeni olarak kalmasını ve syscall etkileşiminin yalnızca
execution submission ve result lifecycle bridging ile sınırlı tutulmasını
istiyorum; böylece kernel'e policy semantiği sızmaz.

#### Kabul Kriterleri

1. THE BCIB_Executor SHALL Ring3 policy/runtime bileşeni olarak kalmalıdır;
   kernel-resident policy semantiği içermemelidir.
2. THE BCIB_Executor SHALL syscall etkileşimini yalnızca `SYS_V2_SUBMIT_EXECUTION (1003)`
   üzerinden execution submission ve result lifecycle bridging ile sınırlamalıdır.
3. THE BCIB_Executor SHALL Ring0'a karar, planlama veya instruction semantics
   taşımamalıdır; tüm bu mantık Ring3'te kalır.
4. IF BCIB_Executor Ring0'a policy kararı taşımaya çalışırsa, THEN THE CI
   SHALL `ci-gate-boundary` kapısında FAIL üretmelidir.
5. THE BCIB_Executor SHALL v0.2 semantiğiyle backward-compatible veya
   fail-closed olmalıdır; sıfırdan rewrite semantiği yasaktır.
6. THE BCIB_Executor SHALL üç ayrı sorumluluk katmanına ayrılmalıdır:
   `BCIB_Verifier/Planner`, `BCIB_Execution_Runtime` ve
   `Scheduler_Submit_Bridge`; bu katmanlar birbirinin implementation
   detaylarına doğrudan bağımlı olmamalıdır.
7. THE `Scheduler_Submit_Bridge` SHALL yalnızca `SYS_V2_SUBMIT_EXECUTION (1003)`
   üzerinden kernel ile iletişim kurmalıdır; execution kararı bu katmanda
   alınamaz.

---

### Gereksinim 2: Execution Lifecycle Sözleşmesi

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, BCIB yürütmesinin tam
yaşam döngüsünün (submit → bounded_slice → yield/wait → resume → complete/cancel)
requirement seviyesinde tanımlanmasını istiyorum; böylece scheduler borcu
oluşmaz.

#### Kabul Kriterleri

1. THE BCIB_Executor SHALL bounded execution slice semantiğini uygulamalıdır;
   sınırsız yürütme yasaktır.
2. THE BCIB_Executor SHALL yield semantiğini desteklemelidir: bir BCIB yürütmesi
   gönüllü olarak CPU'yu bırakabilmeli ve scheduler bridge üzerinden resume
   edilebilmelidir.
3. THE BCIB_Executor SHALL wait semantiğini desteklemelidir: bir BCIB yürütmesi
   dış olay bekleyebilmeli ve olay geldiğinde resume edilebilmelidir.
4. THE BCIB_Executor SHALL resume semantiğini desteklemelidir: yield veya wait
   durumundaki bir yürütme, önceki durumunu koruyarak devam edebilmelidir.
5. THE BCIB_Executor SHALL completion ownership'i tanımlamalıdır: bir yürütme
   tamamlandığında sonuç sahipliği açıkça belirlenmiş olmalıdır.
6. THE BCIB_Executor SHALL cancellation semantiğini desteklemelidir: bir
   yürütme iptal edildiğinde tüm kaynaklar deterministik olarak serbest
   bırakılmalıdır.
7. IF scheduler bridge yield/resume sinyali üretemezse, THEN THE BCIB_Executor
   SHALL fail-closed semantiğiyle yürütmeyi sonlandırmalıdır.
8. THE BCIB_Executor SHALL execution starvation'ı önlemelidir; bir context
   diğer context'lerin yürütme fırsatını süresiz engelleyemez.
9. THE BCIB_Executor SHALL scheduler ile fairness constraint'lerine uygun
   etkileşim kurmalıdır; priority, preemption ve fairness politikası
   scheduler'ın yetkisindedir — BCIB bu politikayı override edemez.

---

### Gereksinim 3: Memory Model

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, BCIB v3'ün kendi memory
modelinin requirement seviyesinde tanımlanmasını istiyorum; böylece memory
safety yalnızca iyi niyet değil, doğrulanabilir sözleşme olur.

#### Kabul Kriterleri

1. THE BCIB_Executor SHALL slot-based transient state kullanmalıdır; yürütme
   süresi boyunca geçici durum slot'lara bağlıdır ve yürütme bitiminde
   deterministik olarak temizlenir.
2. THE BCIB_Executor SHALL handle-based long-lived state kullanmalıdır;
   uzun ömürlü durum ham pointer yerine handle üzerinden erişilir.
3. THE BCIB_Executor SHALL DSL/CLI katmanına raw pointer expose etmemelidir;
   tüm erişim handle veya bounded slice üzerinden gerçekleşir.
4. THE BCIB_Executor SHALL bounded pool kullanmalıdır; sınırsız heap büyümesi
   `MEMORY.LEAK` ihlali olarak raporlanır.
5. THE BCIB_Executor SHALL verifier-enforced index bounds uygulamalıdır;
   sınır dışı erişim `MEMORY.CONTRACT.VIOLATION` olarak raporlanır.
6. IF herhangi bir `Box::leak` veya `mem::forget` kullanımı tespit edilirse,
   THEN THE CI SHALL `MEMORY.LEAK.INTENTIONAL` NON_OVERRIDABLE ihlali olarak
   FAIL üretmelidir.
7. THE BCIB_Executor SHALL handle ownership transfer kurallarını tanımlamalıdır:
   bir handle yalnızca tek sahibi olabilir; transfer açık devir semantiğiyle
   gerçekleşir, kopyalama yasaktır.
8. THE BCIB_Executor SHALL borrow/read-only vs mutable erişim ayrımını
   uygulamalıdır; aynı anda birden fazla mutable erişim `MEMORY.CONTRACT.VIOLATION`
   olarak raporlanır.
9. THE BCIB_Executor SHALL cleanup order'ı tanımlamalıdır: yürütme sonlandığında
   (complete veya cancel) kaynaklar ters bağımlılık sırasıyla serbest bırakılır.
10. THE BCIB_Executor SHALL execution cancel sırasında teardown contract'ı
    uygulamalıdır: cancel sinyali alındığında tüm slot'lar temizlenir, tüm
    handle'lar serbest bırakılır ve bounded pool'a geri döndürülür; teardown
    deterministik ve tekrarlanabilir olmalıdır.

---

### Gereksinim 3b: Execution Lifecycle State Machine

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, BCIB yürütmesinin resmi
state machine'inin requirement seviyesinde tanımlanmasını istiyorum; böylece
illegal state transition'lar fail-closed olarak reddedilir ve test üretimi
netleşir.

#### Kabul Kriterleri

1. THE BCIB_Executor SHALL aşağıdaki resmi state setini uygulamalıdır:
   `Created → Ready → Running → (Yielded | Waiting) → Running → (Completed | Failed | Cancelled)`.
2. THE BCIB_Executor SHALL her state için geçerli geçişleri tanımlamalıdır:
   - `Created` → `Ready` (doğrulama başarılı)
   - `Ready` → `Running` (execution slice başladı)
   - `Running` → `Yielded` (gönüllü yield)
   - `Running` → `Waiting` (dış olay bekleniyor)
   - `Running` → `Completed` (başarılı tamamlanma)
   - `Running` → `Failed` (hata)
   - `Running` → `Cancelled` (iptal sinyali)
   - `Yielded` → `Running` (resume)
   - `Waiting` → `Running` (olay geldi)
3. THE BCIB_Executor SHALL illegal state transition'ı fail-closed olarak
   reddetmelidir; illegal transition `BCIB_ERR_ILLEGAL_STATE_TRANSITION`
   hatasıyla sonuçlanır.
4. IF bir yürütme `Failed` veya `Cancelled` state'e geçerse, THEN THE
   BCIB_Executor SHALL teardown contract'ı (Gereksinim 3, madde 10)
   deterministik olarak uygulamalıdır.

---

### Gereksinim 4: Determinizm ve Fail-Closed

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, BCIB v3 yürütmesinin
deterministik ve fail-closed olmasını istiyorum; böylece aynı girdi her zaman
aynı sonucu üretir ve geçersiz girdi sessizce geçmez.

#### Kabul Kriterleri

1. THE BCIB_Executor SHALL aynı BCIB grafiği ve ortam koşulları için özdeş
   yürütme sonucu üretmelidir; `DETERMINISM.GLOBAL` NON_OVERRIDABLE kuralı
   geçerlidir.
2. THE BCIB_Executor SHALL geçersiz veya desteklenmeyen BCIB grafiği için
   fail-closed semantiğiyle açık hata döndürmelidir; sessiz devam yasaktır.
3. THE BCIB_Executor SHALL `DETERMINISM.RNG.UNSEEDED` ihlali üretmemelidir;
   rastgele sayı üretimi gerekiyorsa seeded RNG kullanılmalıdır.
4. IF aynı HEAD SHA ile farklı yürütme sonuçları elde edilirse, THEN THE
   Sistem SHALL `DETERMINISM.GLOBAL` ihlali olarak raporlamalıdır.
5. THE `ExecutionPlan` SHALL deterministik canonical hash üretmelidir;
   `canonical_hash()` metodu aynı plan içeriği için her zaman aynı hash'i
   döndürmelidir; bu hash `ProgramCache` key'inin `PlanHash` bileşeni olarak
   kullanılır ve distributed verification ile replay senaryolarında plan
   kimliğini doğrular.

---

## Gereksinimler — Seviye 2: Güvenlik ve Gözlemlenebilirlik

> Bu bölümdeki gereksinimler WS 3.1 ile eş zamanlı geliştirilebilir ancak
> WS 3.1 PASS olmadan CI gate'leri geçemez.

### Gereksinim 5: Capability/Security (WS 3.7)

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, BCIB v3'ün capability
modeliyle entegrasyonunun token tabanlı ve kernel bypass içermeden
gerçekleşmesini istiyorum.

#### Kabul Kriterleri

1. THE Capability_Manager SHALL token tabanlı yetki yönetimi uygulamalıdır;
   kernel bypass `KERNEL.CAPABILITY.BYPASS` NON_OVERRIDABLE ihlalidir.
2. THE BCIB_Executor SHALL her yürütme öncesinde capability token setini
   doğrulamalıdır; doğrulama başarısız olursa `BCIB_ERR_CAPABILITY_DENIED`
   döndürülmelidir.
3. THE Capability_Manager SHALL Ring3'te çalışmalıdır; Ring0'a capability
   kararı taşınamaz.
4. IF capability doğrulaması atlanırsa, THEN THE CI SHALL `ci-gate-boundary`
   kapısında FAIL üretmelidir.

---

### Gereksinim 6: Observability Integration (WS 3.8)

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, BCIB v3'ün diagnostics
yüzeyinin Phase-14 immutable sözleşmelerine uygun olmasını istiyorum; yeni
yüzey Phase-14 sınırlarını genişletemez.

#### Kabul Kriterleri

1. THE BCIB_Executor SHALL Phase-14 observability sözleşmelerini
   (`OBSERVABILITY_UX_CONTRACT_v1.md`, `CROSS_NODE_OBSERVABILITY_GRAPH_CONTRACT_v1.md`,
   `PROOFD_EXTERNAL_DIAGNOSTICS_CONTRACT_v1.md`) mutasyona uğratmamalıdır.
2. THE BCIB diagnostics yüzeyi SHALL `produces_truth=false`,
   `produces_decision=false`, `produces_ranking=false` epistemic sınır
   beyanını korumalıdır.
3. THE BCIB diagnostics yüzeyi SHALL `FORBIDDEN_OBSERVABILITY_FIELDS`
   listesindeki hiçbir alanı expose etmemelidir.
4. IF BCIB diagnostics yanıtı yasak alan içeriyorsa, THEN THE proofd SHALL
   `500 forbidden_observability_field_exposed` ile reddetmelidir.
5. THE WS 3.8 SHALL yalnızca mevcut Phase-14 sınırlarıyla uyumlu ek yüzey
   ekleyebilir; contract mutation yasaktır.

---

## Gereksinimler — Seviye 3: Kullanıcı Arayüzü Katmanı

> WS 3.1 + WS 3.7 tamamlanmadan production-ready sayılamaz.

### Gereksinim 7: System DSL (WS 3.2)

**Kullanıcı Hikayesi:** Bir geliştirici olarak, AykenOS komut dilinin BCIB
grafiğine deterministik dönüşümünü sağlayan DSL parser'ın BCIB v3 sözleşmesiyle
uyumlu olmasını istiyorum.

#### Kabul Kriterleri

1. THE DSL_Parser SHALL AykenOS komutlarını geçerli BCIB v3 grafiğine
   dönüştürmelidir; geçersiz komut fail-closed hata üretmelidir.
2. THE DSL_Parser SHALL BCIB_Executor'a yalnızca validated BCIB grafiği
   iletmelidir; ham DSL string'i executor'a geçilemez.
3. THE DSL_Parser SHALL v0.2 DSL semantiğiyle backward-compatible olmalıdır
   veya açık migration path sunmalıdır.

---

### Gereksinim 8: Semantic CLI (WS 3.3)

**Kullanıcı Hikayesi:** Bir geliştirici olarak, kullanıcı komutlarının DSL'e
deterministik çevrildiği Semantic CLI'ın WS 3.2'ye bağımlı sırayla
geliştirilmesini istiyorum.

#### Kabul Kriterleri

1. THE Semantic_CLI SHALL kullanıcı komutlarını DSL_Parser'a iletmeden önce
   semantik doğrulama uygulamalıdır.
2. THE Semantic_CLI SHALL WS 3.2 DSL_Parser tamamlanmadan production-ready
   sayılmamalıdır.
3. THE Semantic_CLI SHALL hata durumunda kullanıcıya açıklayıcı mesaj
   döndürmelidir; ham hata kodu expose edilmemelidir.

---

## Gereksinimler — Seviye 4: Runtime Katmanı

> WS 3.1 tamamlanmadan production-ready sayılamaz.

### Gereksinim 9: Data Runtime (WS 3.5)

**Kullanıcı Hikayesi:** Bir geliştirici olarak, veri odaklı işlemlerin BCIB
üzerinden yürütüldüğü Data Runtime'ın BCIB v3 memory modeline uygun olmasını
istiyorum.

#### Kabul Kriterleri

1. THE Data_Runtime SHALL veri sorgularını BCIB_Executor üzerinden yürütmelidir;
   doğrudan syscall yasaktır.
2. THE Data_Runtime SHALL BCIB v3 memory modelini (slot-based transient,
   handle-based long-lived) uygulamalıdır.
3. THE Data_Runtime SHALL WS 3.1 tamamlanmadan production-ready sayılmamalıdır.
4. THE Data_Runtime SHALL ABDF erişiminin sıfır maliyetli olmadığını varsaymamalıdır;
   ABDF access latency, batching ve caching stratejisi tanımlanmalıdır.
5. THE Data_Runtime SHALL ABDF erişimini performance-bounded tutmalıdır;
   ABDF latency spike'ı BCIB execution pipeline'ını bloke etmemelidir —
   blocking ABDF erişimi scheduler'a yield edilmelidir.

---

### Gereksinim 10: AI Runtime (WS 3.6)

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, AI runtime'ın Ring3'te
izole çalışmasını, yalnızca öneri üretmesini ve kernel mekanizmalarına
Approved_Runtime_Service_Boundary dışından erişmemesini istiyorum.

#### Kabul Kriterleri

1. THE AI_Runtime SHALL Ring3'te izole çalışmalıdır; Ring0 AI logic
   `KERNEL.RING0.POLICY` NON_OVERRIDABLE ihlalidir.
2. THE AI_Runtime SHALL yalnızca öneri üretmelidir; scheduling, routing veya
   execution kararı veremez.
3. THE AI_Runtime SHALL kernel mekanizmalarına yalnızca Approved_Runtime_Service_Boundary
   üzerinden erişmelidir; bu boundary BCIB executor olabilir ancak tek yol
   olarak kilitlenmez — ileride ek onaylı boundary tanımlanabilir.
4. THE AI_Runtime SHALL capability token olmadan BCIB yürütmesi başlatamamalıdır.
5. IF AI_Runtime scheduling veya routing kararı üretmeye çalışırsa, THEN THE
   Sistem SHALL bu girişimi `KERNEL.RING0.POLICY` ihlali olarak raporlamalıdır.
6. THE AI_Runtime output'u SHALL doğrudan execution'ı etkileyemez; AI önerisi
   açık kullanıcı veya policy onayı olmadan execution path'e dönüştürülemez.
7. THE AI_Runtime SHALL execution bias üretmemelidir; aynı girdi için AI
   önerisi deterministik veya açıkça non-deterministic olarak işaretlenmiş
   olmalıdır.

---

## Gereksinimler — Seviye 5: Downstream

> WS 3.1 çekirdeği tamamlanmadan stabilize edilemez; non-blocking for core
> engine completion.

### Gereksinim 11: Workspace (WS 3.4)

**Kullanıcı Hikayesi:** Bir geliştirici olarak, Workspace katmanının BCIB
çekirdeği tamamlanmadan production-ready sayılmamasını ve otorite yüzeyi
haline gelmemesini istiyorum.

#### Kabul Kriterleri

1. THE Workspace SHALL yalnızca execution katmanı üzerinden işlem yapmalıdır;
   doğrudan syscall veya kernel erişimi yasaktır.
2. THE Workspace SHALL WS 3.1 çekirdeği tamamlanmadan production-ready
   sayılmamalıdır; bu durum `closure_state: downstream, non-blocking for
   core engine completion` olarak işaretlenmelidir.
3. THE Workspace SHALL otorite yüzeyi haline gelmemelidir; karar semantiği
   içeremez.
4. IF Workspace BCIB çekirdeği tamamlanmadan stabilize edilmeye çalışılırsa,
   THEN THE Sistem SHALL teknik borç riski olarak raporlamalıdır.

---

## Gereksinimler — Seviye 6: Araç Zinciri ve Kapanış

### Gereksinim 12: Toolchain/Extensibility (WS 3.9)

**Kullanıcı Hikayesi:** Bir geliştirici olarak, toolchain'in opcode registry,
encoder/decoder version lock ve golden fixture'larla somutlaştırılmasını
istiyorum; soyut "geliştirilebilir toolchain" hedefi teknik borç üretir.

#### Kabul Kriterleri

1. THE Toolchain SHALL opcode registry için tek doğru kaynak tanımlamalıdır;
   opcode ID'leri yeniden kullanılamaz, v0.2 ID'leri v3'te rezervedir.
2. THE Toolchain SHALL encoder/decoder version lock mekanizması içermelidir;
   version uyumsuzluğu fail-closed hata üretmelidir.
3. THE Toolchain SHALL golden fixture'ları tanımlamalı ve bunları CI'da
   doğrulamalıdır; fixture uyumsuzluğu CI FAIL üretmelidir.
4. THE Toolchain SHALL compatibility corpus tanımlamalıdır; v0.2 → v3
   migration senaryoları corpus'ta yer almalıdır.
5. IF opcode ID çakışması tespit edilirse, THEN THE CI SHALL FAIL üretmeli
   ve çakışan ID'leri raporlamalıdır.
6. THE Toolchain SHALL opcode breaking change policy tanımlamalıdır; mevcut
   opcode semantiğini değiştiren her değişiklik version bump ve compatibility
   validation gerektirmelidir.
7. THE Toolchain SHALL opcode version bump'ı CI'da zorunlu kılmalıdır;
   version bump olmadan breaking change merge edilemez.

---

### Gereksinim 13: Governance Gates ve Phase-15 Kapanış Kriterleri (WS 3.10)

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, Phase-15 kapanışının
hangi koşullar altında gerçekleşeceğinin şimdiden seed edilmesini istiyorum;
böylece kapanış kriterleri belirsizlik içermez.

#### Kabul Kriterleri

1. THE Governance_Gate SHALL tüm workstream'lerin (WS 3.1–3.9) tamamlanma
   kanıtlarını doğrulamalıdır; eksik kanıt kapanışı engeller.
2. THE Phase-15 kapanış adayı paketi SHALL Phase-14 modeliyle tutarlı yapı
   kullanmalıdır: `closure_index.json`, `closure_manifest.json`,
   `evidence_index.json`, `closure_decision_record.json`.
3. THE her workstream SHALL per-workstream CI proof tanımlamalıdır; CI gate
   PASS kanıtı olmadan workstream tamamlanmış sayılamaz.
4. THE Phase-15 kapanış otoritesi SHALL yalnızca uzak GitHub Actions
   `ci-freeze` PASS sonucu ve ilişkili HEAD SHA olmalıdır; yerel çalışmalar
   kapanış otoritesi vermez.
5. IF herhangi bir workstream için CI proof eksikse, THEN THE Governance_Gate
   SHALL kapanışı engellemeli ve eksik kanıtı raporlamalıdır.

---

## Failure Modes

Aşağıdaki koşulların herhangi biri Phase-15 kapanışını engeller:

| Koşul | Eylem |
|-------|-------|
| WS 3.1 tamamlanmadan downstream workstream production-ready iddiası | Teknik borç riski raporla; iddiayı reddet |
| BCIB_Executor Ring0'a policy kararı taşıması | `ci-gate-boundary` FAIL; merge reddet |
| Memory model ihlali (`Box::leak`, sınır dışı erişim) | NON_OVERRIDABLE ihlali; CI FAIL |
| Capability bypass | `KERNEL.CAPABILITY.BYPASS` NON_OVERRIDABLE; CI FAIL |
| Phase-14 observability sözleşmesi mutasyonu | Değişikliği reddet; yeni faz gerektirir |
| AI runtime karar otoritesi iddiası | `KERNEL.RING0.POLICY` ihlali raporla |
| Opcode ID çakışması | CI FAIL; çakışan ID'leri raporla |
| Workspace otorite yüzeyi haline gelmesi | Teknik borç riski raporla |
| Herhangi bir NON_OVERRIDABLE ihlali | CI FAIL; deployment block; immutable audit |
| BCIB veri depolama semantiği tanımlamaya çalışması | ABDF boundary ihlali; merge reddet |
| ABDF capability enforcement bypass girişimi | `KERNEL.CAPABILITY.BYPASS` NON_OVERRIDABLE; CI FAIL |
| Resource limit aşımı (instruction flood, memory exhaustion) | fail-closed termination; context temizle |
| Malformed/malicious BCIB program yürütme girişimi | structural/control-flow validation FAIL; yürütme başlatılmaz |

---

## Property-Based Test Gereksinimleri

Aşağıdaki özellikler `proptest` ile doğrulanmalıdır (minimum 100 iterasyon):

1. **Özellik 1 — Execution Determinizm:** Aynı BCIB grafiği ve ortam için
   iki yürütme özdeş sonuç üretmelidir.
2. **Özellik 2 — Fail-Closed:** Geçersiz BCIB grafiği için yürütme sessizce
   devam etmemeli, açık hata döndürmelidir.
3. **Özellik 3 — Memory Bound:** Herhangi bir yürütme için bellek kullanımı
   bounded pool sınırını aşmamalıdır.
4. **Özellik 4 — Capability Enforcement:** Capability token olmadan başlatılan
   yürütme `BCIB_ERR_CAPABILITY_DENIED` ile reddedilmelidir.
5. **Özellik 5 — Observability Boundary:** Herhangi bir BCIB diagnostics
   yanıtında `FORBIDDEN_OBSERVABILITY_FIELDS` listesindeki alan bulunmamalıdır.
6. **Özellik 6 — Lifecycle Completeness:** Herhangi bir yürütme için
   submit → complete/cancel döngüsü tüm kaynakları deterministik olarak
   serbest bırakmalıdır.
7. **Özellik 7 — Version Compatibility:** v0.2 BCIB grafiği için yürütme
   ya backward-compatible sonuç üretmeli ya da deterministik fail-closed
   (`BCIB_ERR_UNSUPPORTED_VERSION`) döndürmelidir; sessiz kısmi uyum yasaktır.
8. **Özellik 8 — Lifecycle State Transition:** Herhangi bir illegal state
   transition (örn. `Completed → Running`, `Cancelled → Yielded`) fail-closed
   olarak `BCIB_ERR_ILLEGAL_STATE_TRANSITION` ile reddedilmelidir; hiçbir
   illegal transition kabul edilmemelidir.
9. **Özellik 9 — Execution Isolation:** İki farklı ExecutionContext arasında
   capability olmadan cross-context slot/handle erişimi
   `BCIB_ERR_ISOLATION_VIOLATION` ile reddedilmelidir.
10. **Özellik 10 — ABDF Boundary:** ABDF-defined interface bypass edilerek
    yapılan erişim `BCIB_ERR_ABDF_ACCESS_DENIED`; revoked handle ile erişim
    `BCIB_ERR_ABDF_HANDLE_REVOKED` döndürmelidir.
11. **Özellik 11 — Bounded Slice Yield:** Cost budget tükendiğinde yürütme
    `Yielded` state'e geçmeli; budget aşımı gerçekleşmemelidir.
12. **Özellik 12 — Plan/Runtime Consistency:** Runtime'da yürütülen instruction
    seti `ExecutionPlan`'daki instruction setiyle birebir aynı olmalıdır;
    plan dışı instruction yürütülmemeli, dynamic instruction mutation
    gerçekleşmemelidir.

---

## Compatibility Validation

Bu bölüm, Phase-15 bileşenlerinin mevcut altyapıyla entegrasyon doğrulamasını
tanımlar. "Hatasız entegrasyon" hedefi bu sözleşmelerle doğrulanır.

| Bileşen | Mevcut Altyapı | Doğrulama Yöntemi | Non-Regression Kriteri |
|---------|---------------|-------------------|----------------------|
| BCIB v3 | `userspace/bcib-runtime/` (v0.2) | v0.2 corpus üzerinde regression test | v0.2 semantiği backward-compatible veya fail-closed |
| DSL_Parser | `userspace/dsl-parser/` | mevcut DSL komutları üzerinde golden fixture testi | mevcut komutlar aynı BCIB IR üretmeli |
| Semantic_CLI | `userspace/semantic-cli/` | mevcut CLI senaryoları üzerinde regression test | mevcut kullanıcı komutları aynı DSL çıktısı üretmeli |
| AI_Runtime | `userspace/ai-runtime/` | öneri üretimi non-regression testi | öneri semantiği değişmemeli; otorite iddiası yasak |
| proofd/obs-cli | `userspace/proofd/` + `userspace/obs-cli/` | Phase-14 contract non-regression | `FORBIDDEN_OBSERVABILITY_FIELDS` ve epistemic boundary değişmez |

**Kural:** Herhangi bir mevcut bileşenin non-regression testi başarısız olursa,
Phase-15 değişikliği merge edilemez. Bu tablo per-workstream CI proof
gereksiniminin (Gereksinim 13, madde 3) tamamlayıcısıdır.

---

## Per-Workstream CI Gate Tablosu

| WS | Workstream | Zorunlu CI Gate | Tamamlanma Kanıtı |
|----|-----------|-----------------|-------------------|
| 3.1 | BCIB Execution Engine v3 | `ci-gate-bcib-v3-core` | determinizm + fail-closed + memory model PASS |
| 3.2 | System DSL | `ci-gate-dsl-bcib-contract` | DSL → BCIB IR golden fixture PASS |
| 3.3 | Semantic CLI | `ci-gate-semantic-cli-contract` | CLI → DSL regression PASS |
| 3.4 | Workspace | `ci-gate-workspace` (mevcut) | non-blocking; WS 3.1 sonrası |
| 3.5 | Data Runtime | `ci-gate-data-runtime-bcib` | BCIB üzerinden veri sorgusu PASS |
| 3.6 | AI Runtime | `ci-gate-ai-runtime-boundary` | öneri-only, capability-gated PASS |
| 3.7 | Capability/Security | `ci-gate-capability-manager` | token-based, no bypass PASS |
| 3.8 | Observability Integration | `ci-gate-proofd-observability-boundary` (mevcut) | Phase-14 non-regression PASS |
| 3.9 | Toolchain/Extensibility | `ci-gate-toolchain-opcode-registry` | opcode ID lock + golden fixture PASS |
| 3.10 | Governance Gates | `ci-freeze` (uzak GitHub Actions) | tüm WS kanıtları + HEAD SHA PASS |

---

## Gereksinimler — Güvenlik (Security)

> Bu bölümdeki gereksinimler "niyet" değil, enforce edilebilir sözleşmedir.
> Her madde CI gate veya property test ile doğrulanabilir olmalıdır.

### Gereksinim 14: Capability Modeli Sertleştirme

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, capability modelinin
granüler, devredilemeyen ve iptal edilebilir olmasını istiyorum; böylece
privilege escalation ve capability forgery mümkün olmaz.

#### Kabul Kriterleri

1. THE Capability_Manager SHALL capability'leri non-forgeable olarak
   uygulamalıdır; capability token dışarıdan üretilemez veya taklit edilemez.
2. THE Capability_Manager SHALL capability'leri non-escalatable olarak
   uygulamalıdır; bir capability kendi kapsamı dışında yetki veremez.
3. THE Capability_Manager SHALL capability'leri execution context'e açıkça
   bağlamalıdır; context dışında kullanım `BCIB_ERR_CAPABILITY_DENIED`
   üretmelidir.
4. THE Capability_Manager SHALL capability propagation'ı doğrulamalıdır;
   bir BCIB instruction zinciri boyunca capability geçişi her adımda
   yeniden doğrulanmalıdır.
5. THE Capability_Manager SHALL capability revocation'ı desteklemelidir;
   iptal edilen capability anında geçersiz olmalı ve bağımlı execution
   path'leri fail-closed sonlandırılmalıdır.
6. THE Capability_Manager SHALL capability inheritance kurallarını
   tanımlamalıdır; alt execution context üst context'in capability'lerini
   otomatik olarak miras alamaz — açık devir gereklidir.
7. IF capability check constant-time'da tamamlanamazsa, THEN THE Sistem
   SHALL timing side-channel riskini raporlamalıdır.

---

### Gereksinim 15: Execution Isolation

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, her BCIB execution
context'inin izole olmasını istiyorum; cross-context erişim açık capability
olmadan mümkün olmamalıdır.

#### Kabul Kriterleri

1. THE BCIB_Executor SHALL her ExecutionContext için izole slot space
   uygulamalıdır; bir context'in slot'larına başka bir context erişemez.
2. THE BCIB_Executor SHALL her ExecutionContext için izole handle space
   uygulamalıdır; handle'lar context sınırını geçemez.
3. THE BCIB_Executor SHALL cross-context erişimi yalnızca açık capability
   token ile izin vermelidir; implicit erişim yasaktır.
4. IF cross-context erişim capability olmadan denenirse, THEN THE
   BCIB_Executor SHALL `BCIB_ERR_ISOLATION_VIOLATION` ile fail-closed
   sonlandırmalıdır.

---

### Gereksinim 16: Input Validation ve DoS Koruması

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, tüm BCIB programlarının
yürütülmeden önce doğrulanmasını ve resource abuse'a karşı korunmasını
istiyorum; böylece malformed input ve kaynak tükenmesi sistemi çökertmez.

#### Kabul Kriterleri

1. THE BCIB_Executor SHALL her BCIB programını yürütmeden önce aşağıdaki
   doğrulama aşamalarından geçirmelidir:
   - structural validation (format bütünlüğü)
   - control-flow validation (döngü/sonsuz yürütme tespiti)
   - capability validation (gerekli token'lar mevcut mu)
   - bounds validation (index ve bellek sınırları)
2. THE BCIB_Executor SHALL doğrulama başarısız olan programı yürütmemelidir;
   fail-closed hata döndürülmelidir.
3. THE BCIB_Executor SHALL resource limit'leri uygulamalıdır:
   - max instruction count per execution (toplam)
   - max instructions per slice (per-slice cheap-op spam guard; cost budget
     tükenmese bile bir slice'ta yürütülebilecek instruction sayısını sınırlar;
     aşım → yield, fail-closed değil)
   - max memory allocation per context
   - max concurrent handles per context
   - max AI request quota per execution
4. THE BCIB_Executor SHALL instruction side-effect sınıfını tanımlamalıdır:
   - `pure` — yan etkisiz
   - `data-mutating` — ABDF veri mutasyonu
   - `external` — AI/UI çağrısı
5. THE BCIB_Executor SHALL `external` ve `data-mutating` sınıfı instruction'lar
   için capability doğrulaması zorunlu kılmalıdır.
6. IF herhangi bir resource limit aşılırsa, THEN THE BCIB_Executor SHALL
   fail-closed termination uygulamalı ve context'i deterministik olarak
   temizlemelidir.

---

## Gereksinimler — Performans (Performance)

> Bu bölümdeki gereksinimler "hedef" değil, doğrulanabilir sözleşmedir.
> Her madde benchmark veya CI gate ile ölçülebilir olmalıdır.

### Gereksinim 17: Execution Cost Modeli

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, her BCIB instruction'ının
tanımlı execution cost'a sahip olmasını istiyorum; böylece scheduler cost-based
budgeting uygulayabilir ve instruction count'a dayalı naive slice'ın ürettiği
borç oluşmaz.

#### Kabul Kriterleri

1. THE Toolchain SHALL her BCIB instruction için tanımlı execution cost
   (cost unit) tanımlamalıdır; `pure` < `data-mutating` < `external`.
2. THE BCIB_Executor SHALL scheduler'a instruction count değil, cost-based
   budget sunmalıdır; scheduler cost budget'ı tüketince yield tetiklenmelidir.
3. THE BCIB_Executor SHALL `external` sınıfı instruction'lar için ayrı
   cost accounting uygulamalıdır; AI/UI çağrısı cost'u normal instruction
   cost'undan ayrı izlenmelidir.
4. IF cost model tanımsız bir instruction tespit edilirse, THEN THE CI SHALL
   FAIL üretmeli ve tanımsız instruction'ı raporlamalıdır.

---

### Gereksinim 18: Memory Performans Modeli

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, slot ve handle
allocation'ının bounded pool üzerinden çalışmasını ve reuse stratejisinin
tanımlı olmasını istiyorum; böylece unbounded heap büyümesi ve allocation
overhead oluşmaz.

#### Kabul Kriterleri

1. THE BCIB_Executor SHALL slot allocation için bounded pool kullanmalıdır;
   pool tükenirse fail-closed hata üretilmelidir, unbounded büyüme yasaktır.
2. THE BCIB_Executor SHALL handle allocation için bounded pool kullanmalıdır;
   aynı kural geçerlidir.
3. THE BCIB_Executor SHALL güvenli olduğu durumlarda slot ve handle reuse
   uygulamalıdır; teardown sonrası temizlenmiş slot/handle pool'a geri
   döndürülmelidir.
4. THE BCIB_Executor SHALL allocation stratejisini CI benchmark'ta
   ölçülebilir kılmalıdır; allocation overhead baseline'ı aşarsa CI WARN
   üretilmelidir.

---

### Gereksinim 19: Decode ve Dispatch Performansı

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, BCIB decode ve opcode
dispatch'inin düşük overhead'li olmasını istiyorum; böylece execution
pipeline'ında decode bottleneck oluşmaz.

#### Kabul Kriterleri

1. THE BCIB_Executor SHALL BCIB decode overhead'ini minimize etmelidir;
   decode cost tanımlı ve ölçülebilir olmalıdır.
2. THE BCIB_Executor SHALL constant-time opcode dispatch desteklemelidir;
   opcode lookup O(1) olmalıdır.
3. THE Toolchain SHALL validated BCIB programlarını cache'leyebilmelidir;
   aynı program tekrar parse/validate edilmeden yeniden kullanılabilmelidir.
4. THE DSL_Parser SHALL compiled DSL output'unu cache'leyebilmelidir;
   aynı DSL komutu tekrar parse edilmeden BCIB IR olarak yeniden
   kullanılabilmelidir.
5. THE Toolchain SHALL cache invalidation stratejisi tanımlamalıdır;
   opcode version bump veya DSL semantik değişikliği cache'i geçersiz
   kılmalıdır — stale cache kullanımı `BCIB_ERR_CACHE_STALE` ile
   reddedilmelidir.
6. THE `ProgramCache` SHALL LRU (Least Recently Used) eviction policy
   uygulamalıdır; kapasite dolunca en eski erişilen entry atılmalıdır;
   non-deterministic eviction yasaktır — eviction sırası her zaman erişim
   zamanına göre belirlenir.
7. THE `ProgramCache` key'i `(PlanHash, CapabilitySetHash, ResourceLimitsHash)`
   üçlüsünden oluşmalıdır; `PlanHash`, `ExecutionPlan::canonical_hash()`
   çıktısıdır; aynı program farklı capability set veya resource limit ile
   farklı cache entry'dir — yanlış cache hit silent privilege escalation
   riskini önler.

---

### Gereksinim 20: Async ve Blocking Operasyon Yönetimi

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, AI ve data operasyonlarının
execution thread'ini bloke etmemesini istiyorum; böylece blocking operasyon
scheduler'ı dondurmuyor.

#### Kabul Kriterleri

1. THE BCIB_Executor SHALL blocking operasyonları (AI çağrısı, data sorgusu)
   execution thread'ini bloke etmeden yürütmelidir; blocking operasyon
   başlamadan önce scheduler'a yield edilmelidir.
2. THE BCIB_Executor SHALL backpressure mekanizması uygulamalıdır; downstream
   bileşen (AI/data) hazır değilse yürütme wait state'e geçmelidir.
3. THE BCIB_Executor SHALL concurrency limit uygulamalıdır; aynı anda
   yürütülebilecek `external` instruction sayısı bounded olmalıdır.
4. IF concurrency limit aşılırsa, THEN THE BCIB_Executor SHALL yeni
   `external` instruction'ı wait state'e almalı veya fail-closed
   reddetmelidir.

---

### Gereksinim 21: Security-Performance Kesişim Sözleşmesi

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, güvenlik kısıtlamalarının
tanımlı performance budget içinde uygulanmasını istiyorum; böylece security
enforcement performance'ı öngörülemez şekilde etkilemez.

#### Kabul Kriterleri

1. THE Capability_Manager SHALL capability check'i constant-time'da
   tamamlamalıdır; timing side-channel riski oluşturmamalıdır.
2. THE BCIB_Executor SHALL input validation'ı bounded-time'da tamamlamalıdır;
   validation cost execution cost modeline dahil edilmelidir.
3. THE BCIB_Executor SHALL security constraint'leri performance budget
   içinde uygulamalıdır; security check'in cost'u instruction cost modeline
   yansıtılmalıdır.
4. IF security enforcement tanımlı time budget'ı aşarsa, THEN THE Sistem
   SHALL bu sapmayı CI benchmark'ta raporlamalıdır.

---

## Gereksinimler — ABDF Boundary

> ABDF, AykenOS'un yetkili veri substratıdır. BCIB execution engine veri
> sahibi değildir; ABDF-managed nesnelere handle üzerinden erişir.
> Bu bölüm BCIB ↔ ABDF sınırını tanımlar.
>
> **Temel prensip:** `Execution != Data` / `BCIB != ABDF`

### Gereksinim 22: ABDF Veri Substrat Sınırı

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, BCIB execution engine'in
veri depolama semantiği tanımlamamasını ve tüm veri operasyonlarının ABDF
sözleşmesi üzerinden gerçekleşmesini istiyorum; böylece execution ve data
katmanları birbirine yapışmaz.

#### Kabul Kriterleri

1. THE BCIB_Executor SHALL ABDF'yi AykenOS'un yetkili veri substratı olarak
   kabul etmelidir; BCIB veri depolama semantiği tanımlayamaz ve
   uygulayamaz.
2. THE BCIB_Executor SHALL ABDF-managed veri nesnelerine yalnızca
   `ABDF_Handle` veya referans üzerinden erişmelidir; raw pointer ile
   doğrudan ABDF belleğine erişim yasaktır.
3. THE BCIB_Executor SHALL tüm veri operasyonlarını ABDF-defined interface
   üzerinden yürütmelidir; ABDF'yi bypass eden veri erişimi
   `BCIB_ERR_ABDF_ACCESS_DENIED` ile reddedilmelidir.
4. THE BCIB_Executor SHALL ABDF capability enforcement'ına tabi olmalıdır;
   ABDF'nin reddettiği veri erişimi BCIB tarafından da reddedilmelidir.
5. THE BCIB_Executor SHALL ABDF veri modelini yeniden tanımlamamalıdır;
   BCIB opcode'ları ABDF storage semantiğini değiştiremez.
6. IF BCIB bir instruction aracılığıyla ABDF dışında veri depolamaya
   çalışırsa, THEN THE Sistem SHALL bu girişimi `ABDF_BOUNDARY_VIOLATION`
   olarak raporlamalı ve yürütmeyi fail-closed sonlandırmalıdır.

---

### Gereksinim 23: ABDF Handle Lifecycle

**Kullanıcı Hikayesi:** Bir mimari yönetici olarak, ABDF handle'larının
BCIB execution lifecycle'ına bağlı olarak deterministik şekilde yönetilmesini
istiyorum; böylece dangling handle ve use-after-free riski oluşmaz.

#### Kabul Kriterleri

1. THE BCIB_Executor SHALL ABDF handle'larını execution context'e bağlamalıdır;
   context sonlandığında (complete/cancel/fail) tüm ABDF handle'ları
   deterministik olarak serbest bırakılmalıdır.
2. THE BCIB_Executor SHALL ABDF handle'larını execution context dışına
   expose etmemelidir; handle context sınırını geçemez.
3. THE BCIB_Executor SHALL ABDF handle revocation'ını desteklemelidir;
   ABDF tarafından iptal edilen handle anında geçersiz olmalı ve
   `BCIB_ERR_ABDF_HANDLE_REVOKED` üretilmelidir.
4. IF execution cancel sırasında ABDF handle serbest bırakılamazsa, THEN
   THE Sistem SHALL bu durumu `MEMORY.LEAK` ihlali olarak raporlamalıdır.

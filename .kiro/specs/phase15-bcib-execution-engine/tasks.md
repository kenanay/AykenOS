# Implementation Plan: Phase-15 BCIB Execution Engine v3

## Overview

Bu plan, `userspace/bcib-runtime/` içindeki v0.2 executor'ı üç katmanlı v3
mimarisine evrimleştirir. Sıralama bağımlılık grafiğine göre belirlendi:
Group 1 (Core Refactor) tüm downstream grupların blocking bağımlısıdır.
Her task tek sorumluluk taşır ve bir CI gate ile kapanır.

**Her task yürütülmeden önce aşağıdaki iki belge kontrol edilmelidir:**
- `.kiro/specs/phase15-bcib-execution-engine/design.md` — teknik tasarım, interface tanımları, data modelleri
- `.kiro/specs/phase15-bcib-execution-engine/requirements.md` — kabul kriterleri, SHALL/SHALL NOT kuralları, property tanımları

Dil: **Rust** (mevcut `userspace/bcib-runtime/` altyapısıyla uyumlu)

---

## Tasks


---

## Group 1 — BCIB Core Refactor

> Bağımlılık: Yok (başlangıç noktası)
> Gate: `ci-gate-bcib-v3-core`
>
> **Not:** `CapabilityManager` trait'i bu grupta stub olarak tanımlanır (Task 1.4).
> Gerçek implementasyon Group 7'dedir. Group 3 (Task 9) bu trait üzerinden ilerler;
> döngüsel bağımlılık yoktur.

- [ ] 1. BcibExecutor sorumluluklarını üç katmana ayır
  - `userspace/bcib-runtime/src/executor.rs` içindeki `BcibExecutor` struct'ını
    `verifier_planner.rs`, `execution_runtime.rs`, `scheduler_bridge.rs` olarak
    üç ayrı modüle böl
  - Her modül yalnızca kendi tanımlı integration contract'ı üzerinden iletişim kurar;
    implementation detaylarına çapraz bağımlılık yasaktır
  - `mod.rs` içinde public API'yi yeniden ihraç et; mevcut çağrı noktaları kırılmamalı
  - _Requirements: 1.6, 1.7_

  - [x] 1.1 `BCIB_Verifier/Planner` iskeletini oluştur
    - `userspace/bcib-runtime/src/verifier_planner.rs` dosyasını oluştur
    - `BcibVerifierPlanner` struct'ını ve `verify_and_plan()` imzasını tanımla
    - Dört doğrulama aşaması için stub implementasyon yaz (her aşama `todo!()` ile işaretli)
    - `ExecutionPlan` ve `BcibError` tiplerini `types.rs`'e taşı
    - _Requirements: 1.6_

  - [x] 1.2 `BCIB_Execution_Runtime` iskeletini oluştur
    - `userspace/bcib-runtime/src/execution_runtime.rs` dosyasını oluştur
    - `BcibExecutionRuntime` struct'ını, `ExecutionContext`, `ExecutionState` enum'ını tanımla
    - `create_context()`, `run_slice()`, `resume()`, `cancel()`, `state_of()` imzalarını yaz
    - State machine geçiş tablosunu `VALID_TRANSITIONS` const olarak kodla
    - _Requirements: 1.6, 3b.1, 3b.2_

  - [x] 1.3 `Scheduler_Submit_Bridge` iskeletini oluştur
    - `userspace/bcib-runtime/src/scheduler_bridge.rs` dosyasını oluştur
    - `SchedulerSubmitBridge` struct'ını ve `submit()`, `wait_result()`,
      `yield_slice()`, `await_resume()` imzalarını tanımla
    - Syscall 1003 (`SYS_V2_SUBMIT_EXECUTION`) bağlantı noktasını stub olarak işaretle
    - _Requirements: 1.2, 1.7_

  - [x] 1.4 `CapabilityManager` trait stub'ını oluştur
    - `userspace/bcib-runtime/src/capability_manager.rs` dosyasını oluştur
    - `CapabilityCheck` trait'ini tanımla: `check(token_id, resource, ctx_id) -> Result<(), BcibError>`
    - `NoopCapabilityManager` stub implementasyonunu yaz (her check'i `Ok(())` döndürür)
    - Group 3 (Task 9) bu trait üzerinden ilerler; gerçek implementasyon Group 7'de (Task 27) yapılır
    - _Requirements: 5.2, 1.6_

  - [x] 1.5 `AbdfHandle` type stub'ını oluştur
    - `userspace/bcib-runtime/src/abdf_boundary.rs` dosyasını oluştur
    - `AbdfHandle` struct'ını stub olarak tanımla: `context_id: ExecutionContextId`, `handle_id: u64`
    - `AbdfHandle::stub(context_id, handle_id)` constructor'ı yaz
    - Group 4 (Task 12, 14) bu stub üzerinden `Vec<AbdfHandle>` alanını tutar;
      gerçek ABDF erişim semantiği Group 6'da (Task 22) implement edilir
    - _Requirements: 22.1, 1.6_

- [x] 2. Checkpoint — Group 1 derleme doğrulaması
  - Tüm yeni modüller `cargo check` hatasız geçmeli
  - Mevcut v0.2 test suite'i kırılmamış olmalı (`cargo test --lib`)
  - `CapabilityCheck` trait stub'ı mevcut olmalı (Task 1.4)
  - `AbdfHandle` type stub'ı mevcut olmalı (Task 1.5)
  - Ensure all tests pass, ask the user if questions arise.


---

## Group 2 — Binary Format + Opcode Registry

> Bağımlılık: Group 1 (types.rs hazır olmalı)
> Gate: `ci-gate-toolchain-opcode-registry`

- [x] 3. v3 binary header/section layout'u implement et
  - `userspace/bcib-runtime/src/binary_format.rs` dosyasını oluştur
  - `BcibHeader` (16 byte: magic, version, flags, section_count, reserved) struct'ını tanımla
  - `SectionEntry` (section_id, offset, length) ve `SectionId` enum'ını (Instructions=0x01,
    Capabilities=0x02, CostHints=0x03) tanımla
  - `parse_header()` ve `parse_section_table()` fonksiyonlarını yaz; magic/version
    uyumsuzluğunda `BCIB_ERR_INVALID_GRAPH` veya `BCIB_ERR_UNSUPPORTED_VERSION` döndür
  - _Requirements: 12.1, 12.2, 16.1_

  - [ ]* 3.1 Write unit tests for binary format parsing
    - Geçerli v3 header parse testi
    - Geçersiz magic byte → `BCIB_ERR_INVALID_GRAPH` testi
    - Desteklenmeyen version → `BCIB_ERR_UNSUPPORTED_VERSION` testi
    - _Requirements: 12.2, 16.1_

- [x] 4. Opcode registry'yi tek doğru kaynak olarak tanımla
  - `userspace/bcib-runtime/src/opcode_registry.rs` dosyasını oluştur
  - Altı opcode sınıfını (control 0x00–0x0F, memory 0x10–0x1F, data 0x20–0x2F,
    ai 0x30–0x3F, ui 0x40–0x4F, diagnostics 0x50–0x5F) `const` tablosu olarak tanımla
  - Her opcode için `SideEffectClass` (Pure/DataMutating/External) ve `CostUnit` ata
  - v0.2 reserved opcode'larını (`0x00` Nop, `0x01` End, `0x10` DataCreate, `0x11` DataAdd,
    `0x12` DataQuery, `0x20` UiRender, `0x30` AiAsk) `RESERVED_V02` listesinde kilitle
  - `lookup_opcode()` → O(1) dispatch; bilinmeyen opcode `BCIB_ERR_INVALID_GRAPH` döndürür
  - _Requirements: 12.1, 12.5, 12.6, 17.1_

  - [ ]* 4.1 Write unit tests for opcode registry
    - v0.2 reserved ID'lerin yeniden kullanım girişimi → hata testi
    - Bilinmeyen opcode lookup → `BCIB_ERR_INVALID_GRAPH` testi
    - Her sınıf için cost unit doğrulama testi (`pure < data-mutating < external`)
    - _Requirements: 12.1, 12.5, 17.1_

- [x] 5. Version/compatibility check mekanizmasını implement et
  - `userspace/bcib-runtime/src/compat.rs` dosyasını oluştur
  - `check_version_compatibility()`: v3 (0x0003) → geçer; v0.2 (0x0002) → backward-compat
    path veya `BCIB_ERR_UNSUPPORTED_VERSION`; diğer → fail-closed
  - `validate_opcode_no_conflict()`: registry'deki ID çakışmasını CI'da tespit eden
    build-time assertion yaz
  - _Requirements: 1.5, 12.4, 12.5_

  - [ ]* 5.1 Write property test for version compatibility (Property 7)
    - `// Feature: phase15-bcib-execution-engine, Property 7: Version Compatibility`
    - Generator: rastgele v0.2 BCIB byte dizisi
    - Assertion: ya backward-compatible sonuç ya da deterministik `BCIB_ERR_UNSUPPORTED_VERSION`;
      sessiz kısmi uyum yasaktır
    - **Property 7: Version Compatibility**
    - **Validates: Requirements 1.5, 12.4**

- [x] 6. Checkpoint — Group 2 opcode registry doğrulaması
  - `ci-gate-toolchain-opcode-registry` gate'i için opcode ID lock testi PASS
  - v0.2 golden fixture'ları parse edilebilmeli
  - Ensure all tests pass, ask the user if questions arise.


---

## Group 3 — Verification Pipeline

> Bağımlılık: Group 1 (BcibVerifierPlanner iskeleti + CapabilityCheck trait stub), Group 2 (opcode registry)
> Gate: `ci-gate-bcib-v3-core`

- [x] 7. Structural validation implement et
  - `BcibVerifierPlanner::verify_structural()` metodunu yaz
  - BCIB header magic + version kontrolü (`parse_header()` kullan)
  - Section layout bütünlüğü: section_count × 8 byte offset hesabı, overlap kontrolü
  - Her instruction için `lookup_opcode()` çağrısı; bilinmeyen opcode → fail-closed
  - Her instruction'a `SideEffectClass` ata (`opcode_registry` üzerinden)
  - Başarısız → `BCIB_ERR_INVALID_GRAPH`
  - _Requirements: 16.1, 16.4_

  - [ ]* 7.1 Write unit tests for structural validation
    - Geçerli BCIB grafiği → `Ok(())` testi
    - Bozuk magic → `BCIB_ERR_INVALID_GRAPH` testi
    - Bilinmeyen opcode → `BCIB_ERR_INVALID_GRAPH` testi
    - _Requirements: 16.1, 16.2_

- [x] 8. Control-flow validation implement et
  - `BcibVerifierPlanner::verify_control_flow()` metodunu yaz
  - Döngü tespiti: DFS ile cycle detection; sonsuz döngü → `BCIB_ERR_CONTROL_FLOW_VIOLATION`
  - Erişilemeyen instruction tespiti: reachability analizi; dead code → `BCIB_ERR_CONTROL_FLOW_VIOLATION`
  - Jump target geçerliliği: hedef instruction index'i bounds içinde mi?
  - Structural validation PASS olmadan bu aşama çalışmaz (fail-fast sıralama)
  - _Requirements: 16.1, 16.2_

  - [ ]* 8.1 Write unit tests for control-flow validation
    - Sonsuz döngü içeren BCIB → `BCIB_ERR_CONTROL_FLOW_VIOLATION` testi
    - Erişilemeyen instruction → `BCIB_ERR_CONTROL_FLOW_VIOLATION` testi
    - Geçersiz jump target → `BCIB_ERR_CONTROL_FLOW_VIOLATION` testi
    - _Requirements: 16.1, 16.2_

- [x] 9. Capability validation implement et
  - `BcibVerifierPlanner::verify_capabilities()` metodunu yaz
  - `data-mutating` ve `external` sınıfı her instruction için `CapabilityCheck::check()` çağır
    (Group 1 Task 1.4'te tanımlanan trait; Group 7'de gerçek implementasyonla değiştirilir)
  - Eksik token → `BCIB_ERR_CAPABILITY_DENIED`; fail-fast, ilk eksik token'da dur
  - `CapabilitySet` parametresi `verify_and_plan()` imzasından geçer; kernel bypass yok
  - _Requirements: 5.2, 16.5_

  - [ ]* 9.1 Write unit tests for capability validation
    - Boş capability set + data-mutating instruction → `BCIB_ERR_CAPABILITY_DENIED` testi
    - Geçerli token seti → `Ok(())` testi
    - _Requirements: 5.2, 16.5_

- [x] 10. Bounds validation implement et
  - `BcibVerifierPlanner::verify_bounds()` metodunu yaz
  - Index sınır kontrolü: operand index'leri instruction array boyutunu aşamaz
  - `ResourceLimits` parametresinden: max_instruction_count, max_memory_per_context,
    max_concurrent_handles, max_ai_quota kontrollerini uygula
  - Aşım → `BCIB_ERR_BOUNDS_VIOLATION`
  - _Requirements: 16.3, 3.5_

  - [ ]* 10.1 Write unit tests for bounds validation
    - max_instruction_count aşımı → `BCIB_ERR_BOUNDS_VIOLATION` testi
    - Geçersiz operand index → `BCIB_ERR_BOUNDS_VIOLATION` testi
    - _Requirements: 16.3, 3.5_

  - [ ]* 10.2 Write property test for fail-closed (Property 2)
    - `// Feature: phase15-bcib-execution-engine, Property 2: Fail-Closed`
    - Generator: rastgele malformed BCIB byte dizisi (proptest arbitrary)
    - Assertion: her malformed input açık `BCIB_ERR_*` döndürmeli; `Ok` yasak
    - **Property 2: Fail-Closed**
    - **Validates: Requirements 4.2, 16.1, 16.2, 3.5**

- [x] 11b. ExecutionPlan canonicalization ve immutability enforcement implement et
  - `ExecutionPlan` struct'ını `types.rs`'te immutable olarak tanımla: tüm alanlar
    `pub(crate)` veya `readonly`; plan oluşturulduktan sonra mutasyon yasaktır
  - Tüm jump hedefleri `verify_and_plan()` içinde çözümlenmeli ve plan'a absolute
    index olarak yazılmalı; runtime'da yeniden yorumlama yasaktır
  - Tüm capability check'leri plan üretimi sırasında pre-bound edilmeli:
    her instruction için gerekli `CapabilityTokenId` listesi plan'a eklenmeli
  - Her instruction'ın `side_effect_class` alanı plan'da sealed olarak işaretlenmeli;
    runtime bu değeri değiştiremez
  - `ExecutionPlan` runtime'a geçtikten sonra verifier/planner katmanı tarafından
    erişilemez; tek yönlü transfer
  - `ExecutionPlan::canonical_hash()` metodunu implement et: plan içeriğinin
    deterministik canonical binary encoding'ini üret ve stabil hash döndür;
    aynı plan her zaman aynı hash'i üretmeli (`DETERMINISM.GLOBAL` kuralı);
    bu hash `ProgramCacheKey` içinde `PlanHash` olarak kullanılır
  - Kabul kriteri: runtime'da plan'ı mutate etmeye çalışan kod `cargo check`'te
    derleme hatası üretmeli
  - _Requirements: 1.6, 4.1, 19.3_

  - [ ]* 11b.1 Write property test for plan/runtime consistency (Property 12)
    - `// Feature: phase15-bcib-execution-engine, Property 12: Plan/Runtime Consistency`
    - Generator: rastgele geçerli BCIB grafiği
    - Assertion: runtime'da yürütülen instruction seti `ExecutionPlan`'daki instruction
      setiyle birebir aynı olmalı; plan dışı instruction yürütülmemeli,
      dynamic instruction mutation gerçekleşmemeli
    - **Property 12: Plan/Runtime Consistency**
    - **Validates: Requirements 4.1, 1.6**

- [x] 11c. Checkpoint — Group 3 verification pipeline doğrulaması
  - Dört aşamalı pipeline sıralı çalışmalı (fail-fast)
  - `verify_and_plan()` tüm aşamaları birleştirmeli ve `ExecutionPlan` üretmeli
  - `ExecutionPlan` immutability contract'ı `cargo check`'te derleme hatası üretmeli
  - Ensure all tests pass, ask the user if questions arise.


---

## Group 4 — Runtime State + Memory Model

> Bağımlılık: Group 1 (BcibExecutionRuntime iskeleti), Group 3 (ExecutionPlan üretimi)
> Gate: `ci-gate-bcib-v3-core`
>
> **Not:** `AbdfHandle` tipi bu grupta Task 1.5'te oluşturulan `abdf_boundary.rs` stub'ından alınır.
> Gerçek ABDF erişim semantiği Group 6'dadır (Task 22). Group 4, yalnızca
> `Vec<AbdfHandle>` alanını tutar ve teardown sırasında handle'ları ABDF'ye bildirir.

- [x] 12. ExecutionContext ve state machine'i implement et
  - `BcibExecutionRuntime::create_context()` metodunu yaz
  - `ExecutionContext` struct'ını doldur: id, state (Created), plan, capability_set,
    slot_space (IsolatedSlotSpace), handle_space (IsolatedHandleSpace), cost_tracker, abdf_handles
  - `state_of()` metodunu implement et
  - `VALID_TRANSITIONS` tablosunu kullanarak `transition_state()` private metodunu yaz;
    tabloda olmayan geçiş → `BCIB_ERR_ILLEGAL_STATE_TRANSITION`
  - _Requirements: 3b.1, 3b.2, 3b.3_

  - [x] 12.1 Write property test for illegal state transition rejection (Property 8)
    - `// Feature: phase15-bcib-execution-engine, Property 8: Illegal State Transition Rejection`
    - Generator: rastgele (ExecutionState, ExecutionState) çifti; geçerli geçişler filtrelenir
    - Assertion: illegal her geçiş `BCIB_ERR_ILLEGAL_STATE_TRANSITION` döndürmeli
    - **Property 8: Illegal State Transition Rejection**
    - **Validates: Requirements 3b.3**

- [x] 13. Bounded slot pool ve handle pool implement et
  - `userspace/bcib-runtime/src/pools.rs` dosyasını oluştur
  - `BoundedPool<T>` generic struct'ını yaz: sabit kapasiteli, `acquire()` / `release()` API
  - Pool tükenince `BCIB_ERR_RESOURCE_EXHAUSTED` döndür; unbounded büyüme yasak
  - `IsolatedSlotSpace` ve `IsolatedHandleSpace` wrapper'larını yaz;
    context ID ile etiketlenmiş erişim — yanlış context ID → `BCIB_ERR_ISOLATION_VIOLATION`
  - _Requirements: 3.1, 3.2, 3.4, 15.1, 15.2, 18.1, 18.2_

  - [ ]* 13.1 Write property test for memory bound (Property 3)
    - `// Feature: phase15-bcib-execution-engine, Property 3: Memory Bound`
    - Generator: rastgele geçerli BCIB grafiği; pool boyutu sabit
    - Assertion: bellek kullanımı pool sınırını aşmamalı; aşım → `BCIB_ERR_RESOURCE_EXHAUSTED`
    - **Property 3: Memory Bound**
    - **Validates: Requirements 3.4, 16.3, 18.1, 18.2**

  - [ ]* 13.2 Write property test for execution isolation (Property 9)
    - `// Feature: phase15-bcib-execution-engine, Property 9: Execution Isolation`
    - Generator: iki farklı ExecutionContextId + rastgele slot/handle erişim girişimi
    - Assertion: capability olmadan cross-context erişim → `BCIB_ERR_ISOLATION_VIOLATION`
    - **Property 9: Execution Isolation**
    - **Validates: Requirements 15.1, 15.2, 15.3, 15.4**

- [x] 14. Teardown contract implement et
  - `BcibExecutionRuntime::teardown()` private metodunu yaz
  - Ters bağımlılık sırasıyla: (1) external instruction'ları iptal et,
    (2) ABDF handle'ları serbest bırak, (3) slot'ları temizle → pool'a döndür,
    (4) handle'ları temizle → pool'a döndür, (5) ExecutionContext → pool'a döndür,
    (6) capability token'ları revoke et
  - `cancel()` ve `Failed` state geçişinde otomatik çağrılır
  - Teardown başarısız → `MEMORY.LEAK` NON_OVERRIDABLE ihlali logla
  - _Requirements: 3.9, 3.10, 3b.4_

  - [ ]* 14.1 Write unit tests for teardown contract
    - Cancel sonrası tüm slot'lar pool'a döndü mü? testi
    - Failed state sonrası ABDF handle'lar serbest bırakıldı mı? testi
    - Teardown sırası doğrulaması (ters bağımlılık) testi
    - _Requirements: 3.9, 3.10_

  - [ ]* 14.2 Write property test for lifecycle completeness (Property 6)
    - `// Feature: phase15-bcib-execution-engine, Property 6: Lifecycle Completeness`
    - Generator: rastgele geçerli BCIB grafiği; complete veya cancel ile sonlandır
    - Assertion: tüm slot, handle, ABDF handle, capability token serbest bırakılmalı
    - **Property 6: Lifecycle Completeness**
    - **Validates: Requirements 2.6, 3.1, 3.9, 3.10, 3b.4, 23.1**

- [x] 15. Illegal state transition rejection'ı `run_slice()` ve `cancel()` içinde uygula
  - `run_slice()`: context `Ready` veya `Yielded` değilse → `BCIB_ERR_ILLEGAL_STATE_TRANSITION`
  - `cancel()`: terminal state'deyse (Completed/Failed/Cancelled) → `BCIB_ERR_ILLEGAL_STATE_TRANSITION`
  - `resume()`: context `Yielded` veya `Waiting` değilse → `BCIB_ERR_ILLEGAL_STATE_TRANSITION`
  - _Requirements: 3b.3_

- [x] 16. Checkpoint — Group 4 memory model doğrulaması
  - Bounded pool tükenme senaryosu test edilmeli
  - Teardown contract deterministik çalışmalı
  - Ensure all tests pass, ask the user if questions arise.


---

## Group 5 — Scheduler Integration

> Bağımlılık: Group 1 (SchedulerSubmitBridge iskeleti), Group 4 (ExecutionContext hazır)
> Gate: `ci-gate-bcib-v3-core`

- [x] 17. Cost-based slice execution implement et
  - `BcibExecutionRuntime::run_slice()` metodunu doldur
  - `CostBudget` (total, remaining, external_budget) struct'ını `types.rs`'e ekle
  - Her instruction yürütülürken `cost_tracker`'dan `remaining` düş
  - `remaining == 0` → `Running → Yielded` geçişi; `yield_slice()` çağır
  - `external` instruction için `external_budget`'tan ayrı düş
  - `max_instructions_per_slice` fallback guard'ını uygula: `ResourceLimits`'ten alınan
    bu değer, cost budget tükenmese bile bir slice'ta yürütülebilecek instruction sayısını
    sınırlar; cheap op spam ile scheduler fairness'ının bozulmasını önler;
    aşım → `Running → Yielded` geçişi (fail-closed değil, yield — execution devam eder)
  - _Requirements: 2.1, 2.2, 2.8, 17.2, 17.3_

  - [ ]* 17.1 Write property test for bounded slice yield (Property 11)
    - `// Feature: phase15-bcib-execution-engine, Property 11: Bounded Slice Yield`
    - Generator: rastgele geçerli BCIB grafiği + tanımlı cost budget
    - Assertion: budget tükenince `Yielded` state'e geçmeli; budget aşımı yasak
    - **Property 11: Bounded Slice Yield**
    - **Validates: Requirements 2.1, 2.2, 17.2**

- [x] 18. Yield/resume sinyallerini `SchedulerSubmitBridge` üzerinden implement et
  - `SchedulerSubmitBridge::yield_slice()` ve `await_resume()` metodlarını doldur
  - Syscall 1003 (`SYS_V2_SUBMIT_EXECUTION`) bağlantısını kur
  - `SYS_V2_WAIT_RESULT (1004)` ile sonuç bekleme döngüsünü yaz
  - Bridge başarısız → `BCIB_ERR_SCHEDULER_BRIDGE_FAIL`; execution fail-closed sonlandır
  - _Requirements: 1.2, 1.7, 2.2, 2.4_

  - [ ]* 18.1 Write unit tests for yield/resume signaling
    - Yield sonrası context `Yielded` state'de mi? testi
    - Resume sonrası context `Running` state'e geçti mi? testi
    - Bridge fail → `BCIB_ERR_SCHEDULER_BRIDGE_FAIL` testi
    - _Requirements: 2.2, 2.4_

- [x] 19. Wait handling implement et
  - `BcibExecutionRuntime::run_slice()` içinde `external` instruction başlamadan
    önce `Running → Waiting` geçişini uygula
  - `EventDescriptor` struct'ını tanımla (AI/data event tipi + handle)
  - Olay gelince `Waiting → Running` geçişini tetikle
  - Blocking operasyon scheduler'a yield edilmeden başlatılamaz
  - ABDF blocking erişimi de bu kurala tabidir: `abdf_call.is_blocking()` ise
    `Running → Waiting` geçişi yapılır ve `yield_slice()` çağrılır; ABDF latency
    spike'ı execution thread'ini bloke edemez
  - _Requirements: 2.3, 20.1, 20.2, 9.4, 9.5_

- [x] 20. Starvation/fairness hook'larını implement et
  - `SchedulerSubmitBridge` içinde starvation sayacı tut: N slice boyunca
    resume edilemeyen context için diagnostic signal emit et (log kaydı);
    escalation = karar değil, event — scheduling kararı scheduler'ın yetkisindedir
  - Bir context diğer context'lerin fırsatını engelleyemez; fairness constraint
    scheduler'ın yetkisinde — BCIB yalnızca yield/resume sinyali üretir
  - Concurrency limit enforcement: `external` instruction sayısı `ResourceLimits`'ten
    alınır; limit kontrolü `BCIB_Execution_Runtime::run_slice()` içinde yapılır,
    `Scheduler_Submit_Bridge`'de değil — bridge karar vermez, yalnızca sinyal iletir
  - Bridge SHALL NOT: scheduling kararı vermek, execution sırasını değiştirmek,
    execution request'i düşürmek
  - _Requirements: 2.8, 2.9, 20.3, 20.4_

- [x] 21. Checkpoint — Group 5 scheduler entegrasyon doğrulaması
  - Cost budget tükenince yield tetiklenmeli
  - Wait/resume döngüsü çalışmalı
  - Ensure all tests pass, ask the user if questions arise.


---

## Group 6 — ABDF Boundary

> Bağımlılık: Group 4 (ExecutionContext, handle pool), Group 3 (capability validation)
> Gate: `ci-gate-bcib-v3-core`

- [x] 22. ABDF_Handle entegrasyonunu implement et
  - `userspace/bcib-runtime/src/abdf_boundary.rs` dosyasını oluştur
  - `AbdfHandle` wrapper struct'ını tanımla: context ID ile etiketlenmiş,
    raw pointer expose etmez
  - `BcibExecutionRuntime` içinde `abdf_handles: Vec<AbdfHandle>` alanını doldur
  - ABDF-defined interface üzerinden veri erişimi; bypass → `BCIB_ERR_ABDF_ACCESS_DENIED`
  - BCIB opcode'ları ABDF storage semantiğini değiştiremez
  - _Requirements: 22.1, 22.2, 22.3_

  - [ ]* 22.1 Write property test for ABDF boundary (Property 10)
    - `// Feature: phase15-bcib-execution-engine, Property 10: ABDF Boundary`
    - Generator: rastgele veri erişim girişimi (ABDF bypass veya revoked handle)
    - Assertion: bypass → `BCIB_ERR_ABDF_ACCESS_DENIED`; revoked → `BCIB_ERR_ABDF_HANDLE_REVOKED`
    - **Property 10: ABDF Boundary**
    - **Validates: Requirements 22.2, 22.3, 22.4, 23.3**

- [x] 23. ABDF access contract ve capability enforcement'ı implement et
  - `data-mutating` ve `external` instruction'lar için ABDF erişiminde
    `CapabilityManager::check()` çağrısını zorunlu kıl
  - ABDF'nin reddettiği erişim BCIB tarafından da reddedilir:
    `BCIB_ERR_ABDF_ACCESS_DENIED`
  - BCIB instruction aracılığıyla ABDF dışında veri depolama girişimi →
    `ABDF_BOUNDARY_VIOLATION`; fail-closed
  - _Requirements: 22.3, 22.4, 22.6_

- [x] 24. ABDF handle revocation implement et
  - `AbdfHandle::revoke()` metodunu yaz; revoke sonrası handle geçersiz
  - Revoked handle ile erişim → `BCIB_ERR_ABDF_HANDLE_REVOKED`
  - Revocation anında bağımlı execution path fail-closed sonlandırılır
  - _Requirements: 23.3_

  - [ ]* 24.1 Write unit tests for ABDF handle revocation
    - Revoke sonrası aynı handle ile erişim → `BCIB_ERR_ABDF_HANDLE_REVOKED` testi
    - Revocation sonrası execution fail-closed sonlandı mı? testi
    - _Requirements: 23.3_

- [x] 25. ABDF handle lifecycle cleanup implement et
  - Teardown contract'ına (Task 14) ABDF handle serbest bırakma adımını entegre et
  - Context sonlandığında (complete/cancel/fail) tüm `abdf_handles` ABDF'ye bildirilir
  - Cancel sırasında ABDF handle serbest bırakılamazsa → `MEMORY.LEAK` ihlali logla
  - _Requirements: 23.1, 23.2, 23.4_

- [x] 26. Checkpoint — Group 6 ABDF boundary doğrulaması
  - ABDF bypass girişimi fail-closed reddedilmeli
  - Handle lifecycle context sonlanınca temizlenmeli
  - Ensure all tests pass, ask the user if questions arise.


---

## Group 7 — Security

> Bağımlılık: Group 3 (capability validation pipeline), Group 4 (IsolatedSlotSpace/HandleSpace)
> Gate: `ci-gate-capability-manager`

- [x] 27. CapabilityManager'ı implement et
  - `userspace/bcib-runtime/src/capability_manager.rs` dosyasını oluştur
  - `CapabilityManager` struct'ını yaz: `BoundedPool<CapabilityToken>` üzerinde
  - `bind()`, `revoke()`, `check()`, `transfer()` metodlarını implement et
  - Token non-forgeable: dışarıdan üretim yasak; yalnızca `bind()` ile eklenir
  - Token non-escalatable: kendi kapsamı dışında yetki veremez
  - Token context-bound: `check()` içinde `ctx_id` doğrulaması zorunlu
  - `check()` constant-time'da tamamlanmalı (timing side-channel riski yok)
  - _Requirements: 5.1, 5.3, 14.1, 14.2, 14.3, 21.1_

  - [ ]* 27.1 Write property test for capability enforcement (Property 4)
    - `// Feature: phase15-bcib-execution-engine, Property 4: Capability Enforcement`
    - Generator: rastgele BCIB grafiği + boş veya eksik capability set
    - Assertion: eksik token → `BCIB_ERR_CAPABILITY_DENIED`; revoked token → `BCIB_ERR_CAPABILITY_DENIED`
    - **Property 4: Capability Enforcement**
    - **Validates: Requirements 5.1, 5.2, 14.1, 14.2, 14.3, 14.5**

- [x] 28. Context isolation enforcement'ı implement et
  - `IsolatedSlotSpace` ve `IsolatedHandleSpace` içinde cross-context erişim kontrolünü doldur
  - Capability olmadan cross-context erişim → `BCIB_ERR_ISOLATION_VIOLATION`; fail-closed
  - Explicit capability token ile cross-context transfer: `CapabilityManager::transfer()` kullan
  - Alt context üst context capability'lerini otomatik miras alamaz; açık devir zorunlu
  - _Requirements: 14.6, 15.1, 15.2, 15.3, 15.4_

  - [ ]* 28.1 Write unit tests for context isolation
    - Capability olmadan cross-context slot erişimi → `BCIB_ERR_ISOLATION_VIOLATION` testi
    - Geçerli capability ile transfer → başarılı testi
    - Otomatik miras girişimi → reddedildi testi
    - _Requirements: 15.3, 15.4_

- [x] 29. Resource limit enforcement'ı implement et
  - `ResourceLimits` struct'ını `types.rs`'e ekle:
    max_instruction_count, max_memory_per_context, max_concurrent_handles, max_ai_quota,
    max_instructions_per_slice (per-slice cheap-op spam guard; Task 17'de kullanılır)
  - `run_slice()` içinde her instruction sonrası limit kontrolü yap
  - Aşım → `BCIB_ERR_RESOURCE_EXHAUSTED`; fail-closed termination; context deterministik temizle
  - _Requirements: 16.3, 16.6, 2.8_

- [x] 30. Side-effect class enforcement'ı implement et
  - `run_slice()` içinde her instruction'ın `SideEffectClass`'ını kontrol et
  - `DataMutating` veya `External` → `CapabilityManager::check()` zorunlu
  - `Pure` → capability check atlanır (cost: COST_PURE = 1)
  - _Requirements: 16.4, 16.5_

- [x] 31. Checkpoint — Group 7 security doğrulaması
  - `ci-gate-capability-manager` gate'i için token-based, no bypass PASS
  - Capability bypass girişimi CI'da `KERNEL.CAPABILITY.BYPASS` ihlali üretmeli
  - Ensure all tests pass, ask the user if questions arise.


---

## Group 8 — Performance

> Bağımlılık: Group 5 (cost accounting), Group 2 (opcode registry + cache), Group 4 (pool reuse)
> Gate: `ci-gate-bcib-v3-core`

- [x] 32. Cost accounting'i doldur ve `CostTracker` implement et
  - `userspace/bcib-runtime/src/cost_tracker.rs` dosyasını oluştur
  - `CostTracker` struct'ı: `total`, `remaining`, `external_used` alanları
  - `charge(cost: CostUnit)` → remaining'den düş; sıfırın altına inemez
  - `charge_external(cost: CostUnit)` → `external_used`'a ekle; `external_budget` aşımı → fail
  - `COST_PURE = 1`, `COST_DATA_MUTATING = 10`, `COST_EXTERNAL = 100` sabitlerini kullan
  - _Requirements: 17.1, 17.2, 17.3_

- [x] 33. O(1) opcode dispatch path'ini doğrula ve optimize et
  - `opcode_registry.rs` içindeki `lookup_opcode()` fonksiyonunun array index veya
    hash map ile O(1) çalıştığını doğrula
  - Decode overhead'ini minimize et: `parse_header()` + `parse_section_table()` tek geçişte
  - `BCIB_ERR_INVALID_GRAPH` döndüren path'lerde erken çıkış (fail-fast) uygula
  - _Requirements: 19.1, 19.2_

- [x] 34. Validated BCIB program cache'ini implement et
  - `userspace/bcib-runtime/src/program_cache.rs` dosyasını oluştur
  - `ProgramCache` struct'ı: `LinkedHashMap<ProgramCacheKey, ExecutionPlan>` (bounded kapasiteli)
  - Eviction policy: LRU (Least Recently Used) — kapasite dolunca en eski entry atılır;
    non-deterministic eviction yasaktır; eviction sırası her zaman erişim zamanına göre belirlenir
  - `ProgramCacheKey = (ProgramHash, CapabilitySetHash, ResourceLimitsHash)` — aynı
    program farklı capability set veya resource limit ile farklı cache entry'dir;
    yanlış cache hit → silent privilege escalation riski önlenir
  - `ProgramHash`, `ExecutionPlan::canonical_hash()` çıktısıdır (Task 11b)
  - `get_or_validate()`: cache hit → plan döndür (LRU sırasını güncelle); miss → `verify_and_plan()` çağır, cache'e ekle
  - Cache invalidation: opcode version bump veya DSL semantik değişikliği → cache temizle
  - Stale cache kullanımı → `BCIB_ERR_CACHE_STALE`
  - _Requirements: 19.3, 19.5_

  - [ ]* 34.1 Write unit tests for cache invalidation
    - Version bump sonrası cache hit → `BCIB_ERR_CACHE_STALE` testi
    - Cache miss → `verify_and_plan()` çağrıldı mı? testi
    - _Requirements: 19.5_

- [x] 35. Async/backpressure mekanizmasını implement et
  - `external` instruction başlamadan önce concurrency sayacını kontrol et
  - Limit aşımı → `Waiting` state'e al (backpressure); fail-closed seçeneği de mevcut
  - `external_budget` tükenince yeni `external` instruction → fail-closed
  - _Requirements: 20.2, 20.3, 20.4_

- [x] 36. Checkpoint — Group 8 performance doğrulaması
  - Cost accounting doğru çalışmalı (budget aşımı yok)
  - Cache invalidation senaryosu test edilmeli
  - Ensure all tests pass, ask the user if questions arise.


---

## Group 9 — Observability

> Bağımlılık: Group 4 (ExecutionContext state), Group 7 (capability enforcement)
> Gate: `ci-gate-proofd-observability-boundary`

- [x] 37. BCIB diagnostics endpoint'lerini implement et
  - `userspace/bcib-runtime/src/diagnostics.rs` dosyasını oluştur
  - `GET /diagnostics/bcib/execution/{ctx_id}` → execution state (non-authoritative)
  - `GET /diagnostics/bcib/lifecycle/{ctx_id}` → lifecycle geçiş geçmişi
  - `GET /diagnostics/bcib/cost/{ctx_id}` → cost budget kullanımı
  - Epistemic sınır beyanını her yanıta ekle:
    `produces_truth: false, produces_decision: false, produces_ranking: false`
  - _Requirements: 6.1, 6.2_

  - [ ]* 37.1 Write property test for observability boundary (Property 5)
    - `// Feature: phase15-bcib-execution-engine, Property 5: Observability Boundary`
    - Generator: rastgele execution state + diagnostics sorgusu
    - Assertion: yanıtta `FORBIDDEN_OBSERVABILITY_FIELDS` listesindeki hiçbir alan yok;
      `produces_truth/decision/ranking` her zaman `false`
    - **Property 5: Observability Boundary**
    - **Validates: Requirements 6.2, 6.3**

- [x] 38. Forbidden field kontrolünü implement et
  - `FORBIDDEN_OBSERVABILITY_FIELDS` listesini `diagnostics.rs`'e sabit olarak ekle
  - Her diagnostics yanıtı serialize edilmeden önce bu listeye karşı kontrol et
  - Yasak alan tespit edilirse → `500 forbidden_observability_field_exposed` döndür;
    yanıt gönderilmez
  - _Requirements: 6.3, 6.4_

  - [ ]* 38.1 Write unit tests for forbidden field enforcement
    - Yasak alan içeren yanıt → `500 forbidden_observability_field_exposed` testi
    - Temiz yanıt → `200 OK` testi
    - _Requirements: 6.3, 6.4_

- [x] 39. Phase-14 non-regression testlerini çalıştır ve doğrula
  - `userspace/proofd/` ve `userspace/obs-cli/` mevcut test suite'lerini çalıştır
  - Phase-14 immutable sözleşmeleri (`OBSERVABILITY_UX_CONTRACT_v1.md`,
    `CROSS_NODE_OBSERVABILITY_GRAPH_CONTRACT_v1.md`,
    `PROOFD_EXTERNAL_DIAGNOSTICS_CONTRACT_v1.md`) mutasyona uğramamış olmalı
  - Herhangi bir non-regression başarısız → Phase-15 değişikliği merge edilemez
  - _Requirements: 6.1, 6.5_

- [x] 40. Checkpoint — Group 9 observability doğrulaması
  - `ci-gate-proofd-observability-boundary` PASS
  - Phase-14 non-regression PASS
  - Ensure all tests pass, ask the user if questions arise.


---

## Gate Wiring (Group 9 sonrası, Group 10 öncesi)

> Bu görev Group 10'un bağımlısıdır; Group 10 başlamadan önce tamamlanmalıdır.

- [x] 43. Per-workstream CI gate'lerini wire et
  - `Makefile` veya `.github/workflows/` içinde aşağıdaki gate'leri tanımla:
    - `ci-gate-bcib-v3-core`: determinizm + fail-closed + memory model testleri
    - `ci-gate-toolchain-opcode-registry`: opcode ID lock + golden fixture
    - `ci-gate-capability-manager`: token-based, no bypass testleri
    - `ci-gate-proofd-observability-boundary`: Phase-14 non-regression
    - `ci-gate-dsl-bcib-contract`: DSL → BCIB IR golden fixture (WS 3.2)
    - `ci-gate-semantic-cli-contract`: CLI → DSL regression (WS 3.3)
    - `ci-gate-data-runtime-bcib`: BCIB üzerinden veri sorgusu (WS 3.5)
    - `ci-gate-ai-runtime-boundary`: öneri-only, capability-gated (WS 3.6)
    - `ci-gate-workspace`: workspace otorite sınırı (WS 3.4, mevcut)
  - Her gate bağımsız çalışabilmeli; PASS olmadan merge reddedilmeli
  - _Requirements: 13.3_


---

## Group 10 — Downstream Workstream Stubs (WS 3.2–3.6)

> Bağımlılık: Group 1 (WS 3.1 çekirdeği tamamlanmış olmalı), Task 43 (gate'ler wire edilmiş olmalı)
> Gate: Per-workstream gate'ler (WS 3.1 PASS olmadan geçilemez)
>
> Bu grup, requirements.md §Seviye 3–5'teki workstream'lerin Phase-15 kapanışı için
> minimum kanıt görevlerini tanımlar. Her workstream kendi CI gate'ini geçmeden
> Task 44 (kanıt paketi) tamamlanamaz. Gate tanımları Task 43'te yapılır.

- [x] 40b. WS 3.2 System DSL — CI gate kanıtı
  - `userspace/dsl-parser/` üzerinde `ci-gate-dsl-bcib-contract` gate'ini çalıştır
  - Mevcut DSL komutları BCIB v3 IR üretmeli (golden fixture regression)
  - Gate PASS kanıtını `evidence_index.json`'a ekle
  - _Requirements: 7.1, 7.2, 7.3_

- [x] 40c. WS 3.3 Semantic CLI — CI gate kanıtı
  - `userspace/semantic-cli/` üzerinde `ci-gate-semantic-cli-contract` gate'ini çalıştır
  - Mevcut CLI senaryoları aynı DSL çıktısını üretmeli (regression)
  - Gate PASS kanıtını `evidence_index.json`'a ekle
  - _Requirements: 8.1, 8.2, 8.3_

- [x] 40d. WS 3.5 Data Runtime — CI gate kanıtı
  - `userspace/` içindeki data runtime bileşeni üzerinde `ci-gate-data-runtime-bcib` gate'ini çalıştır
  - BCIB üzerinden veri sorgusu PASS; doğrudan syscall kullanımı yok
  - Gate PASS kanıtını `evidence_index.json`'a ekle
  - _Requirements: 9.1, 9.2, 9.3_

- [x] 40e. WS 3.6 AI Runtime — CI gate kanıtı
  - `userspace/ai-runtime/` üzerinde `ci-gate-ai-runtime-boundary` gate'ini çalıştır
  - Öneri-only, capability-gated; scheduling/routing kararı üretmediği doğrulanmalı
  - Gate PASS kanıtını `evidence_index.json`'a ekle
  - _Requirements: 10.1, 10.2, 10.3, 10.4_

- [x] 40f. WS 3.4 Workspace — CI gate kanıtı
  - `ci-gate-workspace` gate'ini çalıştır (mevcut, non-blocking for core)
  - Workspace'in otorite yüzeyi haline gelmediğini doğrula
  - Gate PASS kanıtını `evidence_index.json`'a ekle
  - _Requirements: 11.1, 11.2, 11.3_


---

## Group 11 — Tests and Gates

> Bağımlılık: Group 1–10 ve Task 43 tamamlanmış olmalı
> Gate: `ci-freeze` (uzak GitHub Actions)

- [x] 41. Kalan property testlerini implement et

  - [ ]* 41.1 Write property test for execution determinism (Property 1)
    - `// Feature: phase15-bcib-execution-engine, Property 1: Execution Determinism`
    - Generator: rastgele geçerli BCIB grafiği + sabit ortam koşulları
    - Assertion: aynı grafik iki kez yürütülünce özdeş sonuç; farklı sonuç → `DETERMINISM.GLOBAL` ihlali
    - **Property 1: Execution Determinism**
    - **Validates: Requirements 4.1, 4.4**

  - [ ]* 41.2 Write integration tests for end-to-end execution lifecycle
    - `verify_and_plan()` → `create_context()` → `run_slice()` → `cancel()` tam döngüsü
    - v0.2 BCIB grafiği ile backward-compat veya fail-closed doğrulaması
    - Teardown contract sonrası pool'ların temiz olduğunu doğrula
    - _Requirements: 1.5, 2.6, 3.9_

- [x] 42. v0.2 golden fixture testlerini implement et
  - `userspace/bcib-runtime/tests/fixtures/` dizinini oluştur
  - v0.2 corpus'tan en az 5 golden fixture BCIB binary dosyası ekle
  - Her fixture için: parse → verify → plan → execute döngüsü; beklenen sonuçla karşılaştır
  - Fixture uyumsuzluğu → CI FAIL
  - _Requirements: 1.5, 12.3, 12.4_

  - [ ]* 42.1 Write property test for version compatibility (golden corpus)
    - v0.2 corpus'taki her fixture için Property 7 assertion'ını çalıştır
    - Sessiz kısmi uyum tespit edilirse → CI FAIL
    - **Property 7: Version Compatibility (corpus)**
    - **Validates: Requirements 1.5, 12.4**

- [x] 44. Governance gate kanıt paketini hazırla
  - `closure_index.json`, `closure_manifest.json`, `evidence_index.json`
    dosyalarını oluştur (Phase-14 modeliyle tutarlı yapı)
  - Her workstream için CI gate PASS kanıtını `evidence_index.json`'a ekle
  - Kapanış otoritesi: yalnızca uzak GitHub Actions `ci-freeze` PASS + HEAD SHA
  - _Requirements: 13.1, 13.2, 13.3, 13.4_

- [x] 46. Repo-geneli derleme ve test doğrulaması
  - `userspace/` altındaki tüm crate'ler için `cargo check` hatasız geçmeli
  - Library target'ı olan crate'ler için `cargo test --lib` hatasız geçmeli:
    `userspace/bcib-runtime/`, `userspace/dsl-parser/`, `userspace/semantic-cli/`,
    `userspace/ai-runtime/`, `userspace/proofd/`
  - Binary-only crate'ler (`userspace/obs-cli/`) için `cargo test` (integration/doc testleri) hatasız geçmeli
  - Herhangi bir crate'de derleme hatası → Phase-15 merge reddedilir
  - (Phase-15 kapsamı `userspace/` ile sınırlıdır; `ayken-core/` ve `ayken/` ayrı gate'e tabidir)
  - _Requirements: 13.3_

- [x] 45. Final checkpoint — Tüm gate'ler ve testler
  - Tüm 12 property testi PASS (min 100 iterasyon) — starred property testler zorunludur
  - Tüm per-workstream CI gate'leri PASS (Task 43'te tanımlanan 9 gate)
  - v0.2 golden fixture'ları PASS
  - Phase-14 non-regression PASS
  - Repo-geneli `cargo check` + `cargo test` hatasız (Task 46 PASS)
  - `ci-freeze` için kanıt paketi hazır (Task 44 PASS)
  - Ensure all tests pass, ask the user if questions arise.

---

## Notes

- `*` ile işaretli sub-task'ler erken iterasyonlarda ertelenebilir; ancak Task 45
  (final checkpoint) için tüm starred property testleri zorunludur — "opsiyonel"
  yalnızca uygulama sırası için geçerlidir, kapanış kriteri için değil
- Her task ilgili requirements'a referans verir (traceability)
- Fail-closed notu: `BCIB_ERR_*` döndürmeyen hiçbir hata yolu kabul edilmez
- NON_OVERRIDABLE ihlalleri (`KERNEL.CAPABILITY.BYPASS`, `MEMORY.LEAK.INTENTIONAL`,
  `DETERMINISM.GLOBAL`, `ERROR.PANIC`) için Allow/Waiver mekanizması yoktur
- Tüm Rust kodu `cargo check` + `cargo test` hatasız geçmelidir — kapsam: `userspace/`;
  library target'ı olan crate'ler `--lib`, binary-only crate'ler (`obs-cli`) `cargo test` ile test edilir
  (Phase-15 kapsamı `userspace/` ile sınırlıdır; `ayken-core/` ve `ayken/` ayrı gate'e tabidir, Task 46)
- Property testler `proptest` kütüphanesi ile yazılır (min 100 iterasyon)
- Kapanış otoritesi: yalnızca uzak GitHub Actions `ci-freeze` PASS + HEAD SHA

### Property Test Özeti

| Property | Task | Validates |
|----------|------|-----------|
| Property 1: Execution Determinism | 41.1 | Req 4.1, 4.4 |
| Property 2: Fail-Closed | 10.2 | Req 4.2, 16.1, 16.2, 3.5 |
| Property 3: Memory Bound | 13.1 | Req 3.4, 16.3, 18.1, 18.2 |
| Property 4: Capability Enforcement | 27.1 | Req 5.1, 5.2, 14.1–14.3, 14.5 |
| Property 5: Observability Boundary | 37.1 | Req 6.2, 6.3 |
| Property 6: Lifecycle Completeness | 14.2 | Req 2.6, 3.1, 3.9, 3.10, 3b.4, 23.1 |
| Property 7: Version Compatibility | 5.1, 42.1 | Req 1.5, 12.4 |
| Property 8: Illegal State Transition | 12.1 | Req 3b.3 |
| Property 9: Execution Isolation | 13.2 | Req 15.1–15.4 |
| Property 10: ABDF Boundary | 22.1 | Req 22.2–22.4, 23.3 |
| Property 11: Bounded Slice Yield | 17.1 | Req 2.1, 2.2, 17.2 |
| Property 12: Plan/Runtime Consistency | 11b.1 | Req 4.1, 1.6 |

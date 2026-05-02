# Design Belgesi — Phase-15: BCIB Execution Engine v3

**Belge Türü:** Normatif Design
**Faz:** Phase-15
**Durum:** DRAFT
**Hazırlayan:** Kenan AY
**Oluşturma Tarihi:** 2026-04-08

---

## Overview

BCIB Execution Engine v3, AykenOS'un execution-centric mimarisinin Ring3 yürütme
çekirdeğidir. `userspace/bcib-runtime/` içindeki v0.2 executor üzerine inşa edilen
bir evolution'dır — sıfırdan rewrite değil.

### Temel Tasarım Kararları

**Karar 1 — Üç Katman Ayrımı (Zorunlu)**
v0.2'de tek bir `BcibExecutor` struct'ı tüm sorumlulukları taşıyordu. v3'te bu
sorumluluklar üç ayrı katmana bölünür:
- `BCIB_Verifier/Planner` — doğrulama ve planlama
- `BCIB_Execution_Runtime` — lifecycle state machine yönetimi
- `Scheduler_Submit_Bridge` — kernel iletişimi

Bu ayrım, her katmanın bağımsız test edilmesini ve Ring0 sınırının korunmasını
sağlar. Katmanlar birbirinin implementation detaylarına doğrudan bağımlı olamaz;
yalnızca tanımlı integration contract'lar üzerinden iletişim kurar.

**Karar 2 — Fail-Closed Semantiği (Mutlak)**
Geçersiz girdi, desteklenmeyen versiyon veya illegal state transition karşısında
sessiz devam yasaktır. Her hata durumu açık `BCIB_ERR_*` kodu döndürür.

**Karar 3 — v0.2 Backward Compatibility**
v0.2 BCIB grafiği ya uyumlu şekilde yürütülür ya da deterministik
`BCIB_ERR_UNSUPPORTED_VERSION` döndürür. Sessiz kısmi uyum yasaktır.

**Karar 4 — BCIB != ABDF / Execution != Data**
BCIB execution engine veri sahibi değildir. Tüm veri operasyonları ABDF
sözleşmesi üzerinden `ABDF_Handle` ile gerçekleşir. BCIB, ABDF storage
semantiğini tanımlayamaz.

**Karar 5 — Phase-14 Observability Değişmezleri IMMUTABLE**
`service != authority`, `diagnostics != decision`, `parity != consensus`
değişmezleri Phase-15 boyunca geçerliliğini korur. BCIB diagnostics yüzeyi
bu sınırları genişletemez.

### Mimari Kısıtlar (Freeze Kapsamı)

- Syscall v2 ABI (1000-1010): değiştirilemez
- `SYS_V2_SUBMIT_EXECUTION (1003)`: tek kernel iletişim noktası
- Ring0 = mechanism only; Ring3 = policy/runtime
- NON_OVERRIDABLE kuralları: hiçbir Allow/Waiver mekanizması geçersiz kılamaz
- AI output doğrudan execution'a dönüştürülemez

---

## Architecture

### Katman Modeli

```
┌─────────────────────────────────────────────────────────────────┐
│                        Ring3 Userspace                          │
│                                                                 │
│  ┌──────────────┐    ┌──────────────────────────────────────┐  │
│  │  DSL_Parser  │───▶│         BCIB_Verifier/Planner        │  │
│  │  Semantic_CLI│    │  structural | control-flow |         │  │
│  └──────────────┘    │  capability | bounds validation      │  │
│                      │  + execution plan üretimi            │  │
│                      └──────────────┬───────────────────────┘  │
│                                     │ ExecutionPlan             │
│                      ┌──────────────▼───────────────────────┐  │
│  ┌──────────────┐    │      BCIB_Execution_Runtime           │  │
│  │Capability_Mgr│◀──▶│  Lifecycle State Machine              │  │
│  └──────────────┘    │  Created→Ready→Running→…→Terminal    │  │
│  ┌──────────────┐    │  slot/handle pool yönetimi           │  │
│  │  ABDF Layer  │◀──▶│  cost-based budget tracking          │  │
│  └──────────────┘    └──────────────┬───────────────────────┘  │
│  ┌──────────────┐                   │ SubmitRequest             │
│  │  AI_Runtime  │◀──▶│      Scheduler_Submit_Bridge          │  │
│  └──────────────┘    │  SYS_V2_SUBMIT_EXECUTION (1003)       │  │
│                      │  yield/resume signaling               │  │
│                      │  fairness + starvation önleme         │  │
│                      └──────────────┬───────────────────────┘  │
└─────────────────────────────────────┼───────────────────────────┘
                                      │ syscall 1003
┌─────────────────────────────────────▼───────────────────────────┐
│                        Ring0 Kernel                             │
│              SYS_V2_SUBMIT_EXECUTION (1003)                     │
│              SYS_V2_WAIT_RESULT (1004)                          │
│              [mechanism only — no policy]                       │
└─────────────────────────────────────────────────────────────────┘
```

### Bileşen Bağımlılık Grafiği

```
DSL_Parser ──────────────────────────────────────────────────────┐
Semantic_CLI ────────────────────────────────────────────────────┤
                                                                  ▼
                                                    BCIB_Verifier/Planner
                                                          │
                                                          │ ExecutionPlan
                                                          ▼
Capability_Manager ──────────────────────────▶ BCIB_Execution_Runtime
ABDF Layer ──────────────────────────────────▶ BCIB_Execution_Runtime
AI_Runtime ──────────────────────────────────▶ BCIB_Execution_Runtime
                                                          │
                                                          │ SubmitRequest
                                                          ▼
                                               Scheduler_Submit_Bridge
                                                          │
                                                          │ syscall 1003
                                                          ▼
                                                    Ring0 Kernel
```

### Workstream Bağımlılık Sırası

```
WS 3.1 (BCIB Core) ──blocking──▶ WS 3.7 (Capability)
                   ──blocking──▶ WS 3.8 (Observability)
                   ──blocking──▶ WS 3.2 (DSL)
                                      └──▶ WS 3.3 (Semantic CLI)
                   ──blocking──▶ WS 3.5 (Data Runtime)
                   ──blocking──▶ WS 3.6 (AI Runtime)
                   ──non-blocking▶ WS 3.4 (Workspace)
                   ──blocking──▶ WS 3.9 (Toolchain)
                   ──blocking──▶ WS 3.10 (Governance)
```

---

## Components and Interfaces

### 1. BCIB_Verifier/Planner

**Sorumluluk:** BCIB grafiğini dört aşamalı doğrulama pipeline'ından geçirir
ve yürütme planı üretir. Yürütme kararı vermez — yalnızca doğrular ve planlar.

**Doğrulama Pipeline'ı (sıralı, fail-fast):**

```
1. Structural Validation
   - BCIB header magic + version kontrolü
   - Section layout bütünlüğü
   - Opcode registry'ye karşı opcode doğrulaması
   - Instruction side-effect sınıfı belirleme (pure/data-mutating/external)

2. Control-Flow Validation
   - Döngü tespiti (sonsuz yürütme yasak)
   - Erişilemeyen instruction tespiti
   - Jump target geçerliliği

3. Capability Validation
   - data-mutating ve external instruction'lar için capability token kontrolü
   - Capability_Manager'a sorgu (Ring3, kernel bypass yok)

4. Bounds Validation
   - Index sınır kontrolü
   - Max instruction count per slice kontrolü
   - Max memory allocation per context kontrolü
   - Max concurrent handles per context kontrolü
   - Max AI request quota per execution kontrolü
```

**Interface:**

```rust
pub struct BcibVerifierPlanner;

impl BcibVerifierPlanner {
    /// Dört aşamalı doğrulama + plan üretimi.
    /// Herhangi bir aşama başarısız olursa fail-closed hata döner.
    pub fn verify_and_plan(
        &self,
        graph: &BcibGraph,
        capability_set: &CapabilitySet,
        resource_limits: &ResourceLimits,
    ) -> Result<ExecutionPlan, BcibError>;
}
```

**Hata Kodları:**
- `BCIB_ERR_INVALID_GRAPH` — structural validation başarısız
- `BCIB_ERR_CONTROL_FLOW_VIOLATION` — sonsuz döngü veya geçersiz jump
- `BCIB_ERR_CAPABILITY_DENIED` — gerekli capability token eksik
- `BCIB_ERR_BOUNDS_VIOLATION` — index veya resource limit aşımı
- `BCIB_ERR_UNSUPPORTED_VERSION` — v0.2 dışı versiyon, backward-compat yok

---

### 2. BCIB_Execution_Runtime

**Sorumluluk:** Lifecycle state machine'i yönetir. Planlanmış yürütmeyi
cost-based budget ile bounded slice'lara böler. Slot/handle pool'u yönetir.
Capability ve ABDF erişimini koordine eder.

**State Machine:**

```
Created ──(verify OK)──▶ Ready ──(slice start)──▶ Running
                                                      │
                              ┌───────────────────────┤
                              │                       │
                         (yield)                 (wait event)
                              │                       │
                              ▼                       ▼
                           Yielded               Waiting
                              │                       │
                         (resume)              (event arrived)
                              │                       │
                              └───────────────────────┘
                                          │
                                          ▼
                                       Running
                                          │
                    ┌─────────────────────┼─────────────────────┐
                    │                     │                     │
               (success)              (error)              (cancel)
                    │                     │                     │
                    ▼                     ▼                     ▼
                Completed             Failed               Cancelled
```

**Geçerli Geçişler:**

| From | To | Tetikleyici |
|------|----|-------------|
| Created | Ready | verify_and_plan() başarılı |
| Ready | Running | execution slice başladı |
| Running | Yielded | gönüllü yield (cost budget tükendi) |
| Running | Waiting | dış olay bekleniyor (AI/data) |
| Running | Completed | başarılı tamamlanma |
| Running | Failed | hata |
| Running | Cancelled | iptal sinyali |
| Yielded | Running | resume sinyali |
| Waiting | Running | olay geldi |

**Illegal Geçişler (fail-closed):** Yukarıdaki tabloda olmayan her geçiş
`BCIB_ERR_ILLEGAL_STATE_TRANSITION` ile reddedilir.

**Interface:**

```rust
pub struct BcibExecutionRuntime {
    contexts: BoundedPool<ExecutionContext>,
    slot_pool: BoundedPool<ExecutionSlot>,
    handle_pool: BoundedPool<AbdfHandle>,
}

impl BcibExecutionRuntime {
    pub fn create_context(
        &mut self,
        plan: ExecutionPlan,
        capability_set: CapabilitySet,
    ) -> Result<ExecutionContextId, BcibError>;

    pub fn run_slice(
        &mut self,
        ctx_id: ExecutionContextId,
        budget: CostBudget,
    ) -> Result<SliceResult, BcibError>;

    pub fn resume(
        &mut self,
        ctx_id: ExecutionContextId,
    ) -> Result<(), BcibError>;

    pub fn cancel(
        &mut self,
        ctx_id: ExecutionContextId,
    ) -> Result<(), BcibError>;

    pub fn state_of(
        &self,
        ctx_id: ExecutionContextId,
    ) -> Result<ExecutionState, BcibError>;
}
```

**Teardown Contract (deterministik, ters bağımlılık sırası):**
1. Tüm `external` instruction'lar iptal edilir
2. Tüm ABDF handle'ları serbest bırakılır
3. Tüm slot'lar temizlenir ve pool'a döndürülür
4. ExecutionContext pool'a döndürülür
5. Capability token'lar revoke edilir

---

### 3. Scheduler_Submit_Bridge

**Sorumluluk:** `SYS_V2_SUBMIT_EXECUTION (1003)` üzerinden kernel ile
iletişim kurar. Yield/resume sinyallerini yönetir. Fairness constraint'lerini
uygular. Execution kararı bu katmanda alınamaz.

**Interface:**

```rust
pub struct SchedulerSubmitBridge;

impl SchedulerSubmitBridge {
    /// BCIB grafiğini Ring0'a iletir.
    /// Execution kararı vermez — yalnızca submission ve result lifecycle.
    pub fn submit(
        &self,
        graph: &BcibGraph,
        context_id: u64,
    ) -> Result<ExecutionId, BcibError>;

    /// Execution sonucunu bekler.
    pub fn wait_result(
        &self,
        execution_id: ExecutionId,
        timeout_ms: u64,
    ) -> Result<ExecutionResult, BcibError>;

    /// Yield sinyali üretir — scheduler'a CPU bırakılır.
    pub fn yield_slice(&self, ctx_id: ExecutionContextId) -> Result<(), BcibError>;

    /// Resume sinyali bekler.
    pub fn await_resume(&self, ctx_id: ExecutionContextId) -> Result<(), BcibError>;
}
```

**Fairness Kuralları:**
- Bir context diğer context'lerin yürütme fırsatını süresiz engelleyemez
- Cost budget tükenince yield zorunludur
- Starvation tespiti: N slice boyunca resume edilemeyen context escalate edilir
- Scheduler policy override yasaktır — BCIB yalnızca yield/resume sinyali üretir

---

### 4. Capability_Manager (WS 3.7)

**Sorumluluk:** Token tabanlı yetki yönetimi. Ring3'te çalışır. Kernel bypass
`KERNEL.CAPABILITY.BYPASS` NON_OVERRIDABLE ihlalidir.

**Capability Özellikleri:**
- **Non-forgeable:** Token dışarıdan üretilemez veya taklit edilemez
- **Non-escalatable:** Capability kendi kapsamı dışında yetki veremez
- **Revocable:** İptal anında geçerlilik yitirir; bağımlı path'ler fail-closed
- **Context-bound:** Context dışında kullanım `BCIB_ERR_CAPABILITY_DENIED`
- **Explicit inheritance:** Alt context üst context capability'lerini otomatik miras alamaz

**Interface:**

```rust
pub struct CapabilityManager {
    tokens: BoundedPool<CapabilityToken>,
}

impl CapabilityManager {
    pub fn bind(&mut self, token: CapabilityToken) -> Result<(), BcibError>;
    pub fn revoke(&mut self, token_id: CapabilityTokenId);
    pub fn check(
        &self,
        token_id: CapabilityTokenId,
        resource: CapabilityResource,
        ctx_id: ExecutionContextId,
    ) -> Result<(), BcibError>;
    pub fn transfer(
        &mut self,
        token_id: CapabilityTokenId,
        from_ctx: ExecutionContextId,
        to_ctx: ExecutionContextId,
    ) -> Result<(), BcibError>;
}
```

**Timing Constraint:** Capability check constant-time'da tamamlanmalıdır;
timing side-channel riski oluşturmamalıdır (Gereksinim 21.1).

---

### 5. Observability Surface (WS 3.8)

**Sorumluluk:** Phase-14 immutable sözleşmelerine uygun diagnostics yüzeyi.
Otorite, karar veya sıralama semantiği üretmez.

**Epistemic Sınır Beyanı (değiştirilemez):**
```
produces_truth=false
produces_decision=false
produces_ranking=false
```

**BCIB Diagnostics Endpoint'leri (Phase-14 sınırlarıyla uyumlu):**
- `GET /diagnostics/bcib/execution/{ctx_id}` — execution state (non-authoritative)
- `GET /diagnostics/bcib/lifecycle/{ctx_id}` — lifecycle geçiş geçmişi
- `GET /diagnostics/bcib/cost/{ctx_id}` — cost budget kullanımı

**Yasak Alanlar:** `FORBIDDEN_OBSERVABILITY_FIELDS` listesindeki hiçbir alan
expose edilemez. Yasak alan içeren yanıt `500 forbidden_observability_field_exposed`
ile reddedilir.

---

## Data Models

### BcibInstruction

```rust
/// BCIB v3 instruction — opcode + operands + side-effect sınıfı
pub struct BcibInstruction {
    pub opcode: OpcodeId,
    pub operands: Vec<Operand>,
    pub side_effect_class: SideEffectClass,
    pub cost: CostUnit,
}

/// Instruction side-effect sınıfı (Gereksinim 16.4)
pub enum SideEffectClass {
    Pure,           // yan etkisiz; cost: düşük
    DataMutating,   // ABDF veri mutasyonu; capability gerekli
    External,       // AI/UI çağrısı; capability gerekli; ayrı cost accounting
}
```

### Opcode Registry ve Sınıflandırma

Opcode'lar altı sınıfa ayrılır. v0.2 opcode ID'leri v3'te rezervedir —
yeniden kullanım yasaktır.

| Sınıf | Opcode Aralığı | Örnekler | Side-Effect |
|-------|---------------|----------|-------------|
| control | 0x00–0x0F | Nop, End, Jump, JumpIf | Pure |
| memory | 0x10–0x1F | SlotAlloc, SlotFree, HandleBorrow | Pure/DataMutating |
| data | 0x20–0x2F | DataCreate, DataAdd, DataQuery | DataMutating |
| ai | 0x30–0x3F | AiAsk, AiStream | External |
| ui | 0x40–0x4F | UiRender, UiEvent | External |
| diagnostics | 0x50–0x5F | TraceEmit, CostReport | Pure |

**v0.2 Reserved Opcodes (değiştirilemez):**
- `0x00` Nop, `0x01` End, `0x10` DataCreate, `0x11` DataAdd,
  `0x12` DataQuery, `0x20` UiRender, `0x30` AiAsk

### BCIB Binary Format (Section Layout)

```
┌─────────────────────────────────────────────────────┐
│  Header (16 bytes)                                  │
│    magic:   [u8; 4]  = b"BCIB"                      │
│    version: u16      = 0x0003 (v3) | 0x0002 (v0.2) │
│    flags:   u16                                     │
│    section_count: u16                               │
│    reserved: [u8; 4]                                │
├─────────────────────────────────────────────────────┤
│  Section Table (section_count × 8 bytes)            │
│    section_id: u16                                  │
│    offset:     u32                                  │
│    length:     u16                                  │
├─────────────────────────────────────────────────────┤
│  Instruction Section (section_id = 0x01)            │
│    instructions: [BcibInstruction]                  │
├─────────────────────────────────────────────────────┤
│  Capability Section (section_id = 0x02)             │
│    required_capabilities: [CapabilityDescriptor]    │
├─────────────────────────────────────────────────────┤
│  Cost Hint Section (section_id = 0x03, optional)    │
│    cost_hints: [CostHint]                           │
└─────────────────────────────────────────────────────┘
```

### ExecutionPlan (Immutable, Canonical)

```rust
/// Immutable execution plan produced by BcibVerifierPlanner.
/// All fields are pub(crate) — mutation after creation is a compile error.
pub struct ExecutionPlan {
    pub(crate) instructions: Vec<BcibInstruction>,
    pub(crate) version: u16,
}

impl ExecutionPlan {
    /// Deterministic canonical hash of this plan's content.
    ///
    /// - Same plan content → same hash (DETERMINISM.GLOBAL).
    /// - Used as PlanHash in ProgramCacheKey.
    /// - Enables distributed verification and replay identity checks.
    pub fn canonical_hash(&self) -> u64 { /* stable hash of canonical encoding */ }

    pub fn instructions(&self) -> &[BcibInstruction] { &self.instructions }
    pub fn version(&self) -> u16 { self.version }
}
```

### ExecutionContext

```rust
pub struct ExecutionContext {
    pub id: ExecutionContextId,
    pub state: ExecutionState,
    pub plan: ExecutionPlan,
    pub capability_set: CapabilitySet,
    pub slot_space: IsolatedSlotSpace,   // context-izole slot alanı
    pub handle_space: IsolatedHandleSpace, // context-izole handle alanı
    pub cost_tracker: CostTracker,
    pub abdf_handles: Vec<AbdfHandle>,
}
```

```rust
pub enum ExecutionState {
    Created,
    Ready,
    Running { slice_start: Instant },
    Yielded { resume_token: ResumeToken },
    Waiting { event_descriptor: EventDescriptor },
    Completed { result: ExecutionResult },
    Failed { error: BcibError },
    Cancelled,
}
```

### CostModel ve ResourceLimits

```rust
/// Cost unit — instruction cost accounting için temel birim
pub type CostUnit = u32;

/// Cost budget — bir execution slice için toplam budget
pub struct CostBudget {
    pub total: CostUnit,
    pub remaining: CostUnit,
    pub external_budget: CostUnit,  // AI/UI için ayrı accounting
}

/// Instruction cost sabitleri (pure < data-mutating < external)
pub const COST_PURE: CostUnit = 1;
pub const COST_DATA_MUTATING: CostUnit = 10;
pub const COST_EXTERNAL: CostUnit = 100;

/// Per-context resource limits (Requirement 16.3, 2.8).
pub struct ResourceLimits {
    /// Toplam instruction sayısı sınırı (tüm execution boyunca).
    pub max_instruction_count: usize,
    /// Per-slice instruction sayısı sınırı — cheap-op spam guard.
    /// Cost budget tükenmese bile bir slice'ta bu kadar instruction yürütülür;
    /// aşım → Running → Yielded (fail-closed değil, yield).
    pub max_instructions_per_slice: usize,
    pub max_memory_per_context: usize,
    pub max_concurrent_handles: usize,
    pub max_ai_quota: usize,
}
```

### Error Taxonomy (BCIB_ERR_* Kodları)

| Kod | Kategori | Açıklama |
|-----|----------|----------|
| `BCIB_ERR_INVALID_GRAPH` | Structural | BCIB header/format geçersiz |
| `BCIB_ERR_CONTROL_FLOW_VIOLATION` | Structural | Sonsuz döngü veya geçersiz jump |
| `BCIB_ERR_CAPABILITY_DENIED` | Security | Gerekli capability token eksik veya geçersiz |
| `BCIB_ERR_BOUNDS_VIOLATION` | Memory | Index veya resource limit aşımı |
| `BCIB_ERR_UNSUPPORTED_VERSION` | Compatibility | v0.2 dışı versiyon, backward-compat yok |
| `BCIB_ERR_ILLEGAL_STATE_TRANSITION` | Lifecycle | Geçersiz state machine geçişi |
| `BCIB_ERR_ABDF_ACCESS_DENIED` | ABDF Boundary | ABDF capability enforcement reddi |
| `BCIB_ERR_ABDF_HANDLE_REVOKED` | ABDF Boundary | ABDF tarafından iptal edilen handle |
| `BCIB_ERR_ISOLATION_VIOLATION` | Security | Cross-context erişim capability olmadan |
| `BCIB_ERR_RESOURCE_EXHAUSTED` | Resource | Bounded pool tükendi |
| `BCIB_ERR_CACHE_STALE` | Toolchain | Stale cache — version bump sonrası geçersiz |
| `BCIB_ERR_SCHEDULER_BRIDGE_FAIL` | Scheduler | Yield/resume sinyali üretilemedi |
| `ABDF_BOUNDARY_VIOLATION` | ABDF Boundary | BCIB ABDF dışında veri depolamaya çalıştı |

### ProgramCache

```rust
/// Validated BCIB program cache — LRU eviction, bounded capacity.
///
/// Cache key includes CapabilitySetHash and ResourceLimitsHash to prevent
/// silent privilege escalation from incorrect cache hits (Requirement 19.3).
pub struct ProgramCache {
    inner: LinkedHashMap<ProgramCacheKey, ExecutionPlan>,
    capacity: usize,
}

/// Cache key — three-part composite (Requirements 19.3, 4.5).
pub struct ProgramCacheKey {
    /// Deterministic hash of ExecutionPlan content (ExecutionPlan::canonical_hash()).
    pub plan_hash: u64,
    /// Hash of the CapabilitySet granted to this execution.
    pub capability_set_hash: u64,
    /// Hash of the ResourceLimits applied to this execution.
    pub resource_limits_hash: u64,
}
```

**Eviction Policy:** LRU (Least Recently Used). Kapasite dolunca en eski
erişilen entry atılır. Non-deterministic eviction yasaktır — eviction sırası
her zaman erişim zamanına göre belirlenir (Requirement 19.6).

**Cache Invalidation:** Opcode version bump veya DSL semantik değişikliği
tüm cache'i temizler. Stale entry → `BCIB_ERR_CACHE_STALE` (Requirement 19.5).

### Compatibility Validation Tablosu
|---------|---------------|-------------------|----------------------|
| BCIB v3 | `userspace/bcib-runtime/` (v0.2) | v0.2 corpus regression test | backward-compatible veya fail-closed |
| DSL_Parser | `userspace/dsl-parser/` | golden fixture testi | aynı BCIB IR üretmeli |
| Semantic_CLI | `userspace/semantic-cli/` | regression test | aynı DSL çıktısı üretmeli |
| AI_Runtime | `userspace/ai-runtime/` | öneri non-regression | öneri semantiği değişmemeli |
| proofd/obs-cli | `userspace/proofd/` + `userspace/obs-cli/` | Phase-14 non-regression | IMMUTABLE sözleşme ihlali yok |
| orchestration | `userspace/orchestration/` | integration test | BCIB v3 API uyumlu |

---

## Error Handling

### Fail-Closed Semantiği (Mutlak Kural)

Her hata durumu açık `BCIB_ERR_*` kodu döndürür. Sessiz devam yasaktır.
Bu kural NON_OVERRIDABLE kapsamındadır.

### Hata Yayılım Modeli

```
Verifier/Planner hatası
    → ExecutionPlan üretilmez
    → ExecutionContext Created state'e geçemez
    → Caller'a BCIB_ERR_* döner

Runtime hatası (Running state)
    → ExecutionState::Failed { error }
    → Teardown contract deterministik olarak çalışır
    → Tüm kaynaklar serbest bırakılır
    → Caller'a hata kodu döner

Scheduler bridge hatası
    → BCIB_ERR_SCHEDULER_BRIDGE_FAIL
    → Execution fail-closed sonlandırılır
    → Teardown contract çalışır

ABDF erişim hatası
    → BCIB_ERR_ABDF_ACCESS_DENIED veya BCIB_ERR_ABDF_HANDLE_REVOKED
    → İlgili instruction fail-closed
    → Execution Failed state'e geçer

Capability hatası
    → BCIB_ERR_CAPABILITY_DENIED
    → Instruction yürütülmez
    → Execution Failed state'e geçer

Resource exhaustion
    → BCIB_ERR_RESOURCE_EXHAUSTED
    → Fail-closed termination
    → Context deterministik temizlenir
```

### Teardown Contract (Deterministik)

Cancel veya Failed state geçişinde ters bağımlılık sırasıyla:

1. `external` instruction'lar iptal edilir (AI/UI çağrıları)
2. ABDF handle'lar serbest bırakılır (ABDF'ye bildirim)
3. Slot'lar temizlenir → bounded pool'a döndürülür
4. Handle'lar temizlenir → bounded pool'a döndürülür
5. ExecutionContext → pool'a döndürülür
6. Capability token'lar revoke edilir

Teardown başarısız olursa `MEMORY.LEAK` NON_OVERRIDABLE ihlali raporlanır.

### Phase Matrix Uyumu

| Kural | P4.4 Davranışı | Tetikleyici |
|-------|---------------|-------------|
| `DETERMINISM.GLOBAL` | ERROR | Aynı girdi farklı sonuç |
| `MEMORY.CONTRACT.VIOLATION` | ERROR | Sınır dışı erişim |
| `MEMORY.LEAK.INTENTIONAL` | ERROR | `Box::leak` / `mem::forget` |
| `KERNEL.RING0.POLICY` | ERROR | Ring0'a policy kararı |
| `KERNEL.CAPABILITY.BYPASS` | ERROR | Capability bypass |
| `ERROR.PANIC` | ERROR | Herhangi bir `panic!` |
| `MEMORY.LEAK` | WARN (P4.4) | Teardown sonrası serbest bırakılmayan kaynak |

---

## Testing Strategy

### Dual Testing Yaklaşımı

BCIB v3 hem unit/integration testleri hem de property-based testleri gerektirir.
Property-based testler için **`proptest`** kütüphanesi kullanılır (minimum 100 iterasyon).

### Unit / Integration Testleri

**Odak alanları:**
- Spesifik hata kodlarının doğru döndürüldüğü örnekler
- State machine geçiş örnekleri (her geçiş için en az 1 test)
- v0.2 backward-compatibility golden fixture'ları
- Teardown contract doğrulaması (cancel + fail senaryoları)
- Capability revocation sonrası erişim reddi
- ABDF handle lifecycle (context sonlanınca handle serbest bırakılır)
- Opcode registry lock (v0.2 ID'leri rezerve)
- CI gate entegrasyon testleri

**Kaçınılacaklar:**
- Property testlerin kapsadığı geniş input alanlarını unit test olarak yazmak
- Fazla sayıda benzer örnek — property testler bunu daha iyi karşılar

### Property-Based Testler (proptest, min 100 iterasyon)

Her property testi design belgesindeki ilgili property'ye referans içerir.
Tag formatı: `// Feature: phase15-bcib-execution-engine, Property N: <property_text>`

**Property 1 — Execution Determinizm**
Aynı BCIB grafiği ve ortam için iki yürütme özdeş sonuç üretmelidir.
Generator: rastgele geçerli BCIB grafiği + ortam koşulları.

**Property 2 — Fail-Closed**
Geçersiz BCIB grafiği için yürütme sessizce devam etmemeli, açık hata döndürmelidir.
Generator: rastgele malformed BCIB byte dizisi.

**Property 3 — Memory Bound**
Herhangi bir yürütme için bellek kullanımı bounded pool sınırını aşmamalıdır.
Generator: rastgele geçerli BCIB grafiği; pool boyutu sabit.

**Property 4 — Capability Enforcement**
Capability token olmadan başlatılan yürütme `BCIB_ERR_CAPABILITY_DENIED` ile reddedilmelidir.
Generator: rastgele BCIB grafiği + boş capability set.

**Property 5 — Observability Boundary**
Herhangi bir BCIB diagnostics yanıtında `FORBIDDEN_OBSERVABILITY_FIELDS` listesindeki
alan bulunmamalıdır.
Generator: rastgele execution state + diagnostics sorgusu.

**Property 6 — Lifecycle Completeness**
Herhangi bir yürütme için submit → complete/cancel döngüsü tüm kaynakları
deterministik olarak serbest bırakmalıdır.
Generator: rastgele geçerli BCIB grafiği; cancel veya complete ile sonlandır.

**Property 7 — Version Compatibility**
v0.2 BCIB grafiği için yürütme ya backward-compatible sonuç üretmeli ya da
deterministik `BCIB_ERR_UNSUPPORTED_VERSION` döndürmelidir; sessiz kısmi uyum yasaktır.
Generator: rastgele v0.2 BCIB grafiği.

**Property 8 — Lifecycle State Transition**
Herhangi bir illegal state transition fail-closed olarak
`BCIB_ERR_ILLEGAL_STATE_TRANSITION` ile reddedilmelidir.
Generator: rastgele (state, transition) çifti; illegal olanlar filtrelenir.

**Property 9 — Execution Isolation**
İki farklı ExecutionContext arasında capability olmadan cross-context erişim
`BCIB_ERR_ISOLATION_VIOLATION` ile reddedilmelidir.
Generator: iki farklı ExecutionContextId + rastgele slot/handle erişim girişimi.

**Property 10 — ABDF Boundary**
ABDF bypass → `BCIB_ERR_ABDF_ACCESS_DENIED`; revoked handle → `BCIB_ERR_ABDF_HANDLE_REVOKED`.
Generator: rastgele veri erişim girişimi (bypass veya revoked handle).

**Property 11 — Bounded Slice Yield**
Cost budget tükenince `Yielded` state'e geçmeli; budget aşımı yasak.
Generator: rastgele geçerli BCIB grafiği + tanımlı cost budget.

**Property 12 — Plan/Runtime Consistency**
Runtime'da yürütülen instruction seti `ExecutionPlan`'daki ile birebir aynı olmalı;
plan dışı instruction yürütülmemeli, dynamic mutation gerçekleşmemeli.
Generator: rastgele geçerli BCIB grafiği.

### CI Gate Tablosu

| WS | Gate | Tamamlanma Kanıtı |
|----|------|-------------------|
| 3.1 | `ci-gate-bcib-v3-core` | determinizm + fail-closed + memory model PASS |
| 3.2 | `ci-gate-dsl-bcib-contract` | DSL → BCIB IR golden fixture PASS |
| 3.3 | `ci-gate-semantic-cli-contract` | CLI → DSL regression PASS |
| 3.4 | `ci-gate-workspace` (mevcut) | non-blocking |
| 3.5 | `ci-gate-data-runtime-bcib` | BCIB üzerinden veri sorgusu PASS |
| 3.6 | `ci-gate-ai-runtime-boundary` | öneri-only, capability-gated PASS |
| 3.7 | `ci-gate-capability-manager` | token-based, no bypass PASS |
| 3.8 | `ci-gate-proofd-observability-boundary` (mevcut) | Phase-14 non-regression PASS |
| 3.9 | `ci-gate-toolchain-opcode-registry` | opcode ID lock + golden fixture PASS |
| 3.10 | `ci-freeze` (uzak GitHub Actions) | tüm WS kanıtları + HEAD SHA PASS |

**Kapanış Otoritesi:** Yalnızca uzak GitHub Actions `ci-freeze` PASS sonucu
ve ilişkili HEAD SHA. Yerel çalışmalar kapanış otoritesi vermez.

---

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system — essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

Bu bölüm, requirements'taki kabul kriterlerini evrensel olarak nitelendirilebilir
property'lere dönüştürür. Her property `proptest` ile minimum 100 iterasyon
çalıştırılır. Property-based testing bu feature için uygundur çünkü:
- BCIB_Executor saf fonksiyon benzeri input/output davranışı sergiler
- Geniş input uzayı (rastgele BCIB grafiği, capability set, state) edge case'leri ortaya çıkarır
- Determinizm, fail-closed, memory bound gibi evrensel özellikler tüm inputlar için geçerlidir

---

### Property 1: Execution Determinizm

*For any* geçerli BCIB grafiği ve sabit ortam koşulları için, aynı grafik iki
kez yürütüldüğünde özdeş sonuç üretmelidir; farklı sonuç `DETERMINISM.GLOBAL`
ihlalidir.

**Validates: Requirements 4.1, 4.4**

---

### Property 2: Fail-Closed

*For any* malformed, geçersiz veya desteklenmeyen BCIB grafiği için, yürütme
sessizce devam etmemeli ve açık `BCIB_ERR_*` hata kodu döndürmelidir; hata
kodu boş veya belirsiz olmamalıdır.

**Validates: Requirements 4.2, 16.1, 16.2, 3.5**

---

### Property 3: Memory Bound

*For any* geçerli BCIB grafiği ve sabit bounded pool boyutu için, yürütme
sırasında bellek kullanımı pool sınırını aşmamalıdır; pool tükenirse
`BCIB_ERR_RESOURCE_EXHAUSTED` döndürülmeli ve unbounded büyüme gerçekleşmemelidir.

**Validates: Requirements 3.4, 16.3, 18.1, 18.2**

---

### Property 4: Capability Enforcement

*For any* BCIB grafiği ve boş veya eksik capability set için, yürütme
`BCIB_ERR_CAPABILITY_DENIED` ile reddedilmelidir; ayrıca *for any* geçerli
capability token için, token revoke edildikten sonra aynı token ile yapılan
erişim girişimi `BCIB_ERR_CAPABILITY_DENIED` döndürmelidir; token dışarıdan
üretilerek forge edilemez ve kendi kapsamı dışında yetki veremez.

**Validates: Requirements 5.1, 5.2, 14.1, 14.2, 14.3, 14.5**

---

### Property 5: Observability Boundary

*For any* execution state ve diagnostics sorgusu için, BCIB diagnostics
yanıtında `FORBIDDEN_OBSERVABILITY_FIELDS` listesindeki hiçbir alan
bulunmamalıdır; `produces_truth`, `produces_decision`, `produces_ranking`
alanları her zaman `false` olmalıdır.

**Validates: Requirements 6.2, 6.3**

---

### Property 6: Lifecycle Completeness

*For any* geçerli BCIB grafiği için, submit → complete veya submit → cancel
döngüsü tamamlandığında tüm slot'lar, handle'lar, ABDF handle'ları ve
capability token'lar deterministik olarak serbest bırakılmalıdır; kaynak
sızıntısı olmamalıdır.

**Validates: Requirements 2.6, 3.1, 3.9, 3.10, 3b.4, 23.1**

---

### Property 7: Version Compatibility

*For any* v0.2 BCIB grafiği için, yürütme ya backward-compatible sonuç
üretmeli ya da deterministik `BCIB_ERR_UNSUPPORTED_VERSION` döndürmelidir;
sessiz kısmi uyum veya belirsiz hata yasaktır.

**Validates: Requirements 1.5, 12.4**

---

### Property 8: Illegal State Transition Rejection

*For any* (mevcut state, hedef state) çifti için, geçerli geçiş tablosunda
yer almayan her geçiş girişimi `BCIB_ERR_ILLEGAL_STATE_TRANSITION` ile
fail-closed reddedilmelidir; hiçbir illegal transition kabul edilmemelidir.

**Validates: Requirements 3b.3**

---

### Property 9: Execution Isolation

*For any* iki farklı ExecutionContext için, bir context'in slot veya handle
alanına diğer context capability olmadan erişim girişimi
`BCIB_ERR_ISOLATION_VIOLATION` ile reddedilmelidir; implicit cross-context
erişim hiçbir zaman kabul edilmemelidir.

**Validates: Requirements 15.1, 15.2, 15.3, 15.4**

---

### Property 10: ABDF Boundary

*For any* veri erişim girişimi için, ABDF-defined interface bypass edilerek
yapılan erişim `BCIB_ERR_ABDF_ACCESS_DENIED` ile reddedilmelidir; ayrıca
*for any* ABDF handle için, ABDF tarafından revoke edilen handle ile yapılan
erişim girişimi `BCIB_ERR_ABDF_HANDLE_REVOKED` döndürmelidir.

**Validates: Requirements 22.2, 22.3, 22.4, 23.3**

---

### Property 11: Bounded Slice Yield

*For any* geçerli BCIB grafiği ve tanımlı cost budget için, cost budget
tükendiğinde yürütme scheduler'a yield etmeli ve `Yielded` state'e geçmelidir;
budget aşımı gerçekleşmemelidir.

**Validates: Requirements 2.1, 2.2, 17.2**

---

### Property 12: Plan/Runtime Consistency

*For any* geçerli BCIB grafiği için, runtime'da yürütülen instruction seti
`ExecutionPlan`'daki instruction setiyle birebir aynı olmalıdır; plan dışı
instruction yürütülmemeli ve dynamic instruction mutation gerçekleşmemelidir.
`ExecutionPlan::canonical_hash()` değeri runtime boyunca değişmemelidir.

**Validates: Requirements 4.1, 1.6, 4.5**

---

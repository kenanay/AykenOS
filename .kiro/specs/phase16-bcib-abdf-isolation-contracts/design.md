# Design Belgesi — Phase-16: BCIB/ABDF Isolation & Boundary Enforcement

**Belge Türü:** Normatif Design
**Faz:** Phase-16
**Durum:** DRAFT
**Hazırlayan:** Kenan AY
**Oluşturma Tarihi:** 2026-04-11

---

## Overview

Bu belge, BCIB (Bytecode Instruction Block) execution engine ile ABDF (Append-Based Data Format) veri katmanı arasındaki izolasyon ve sınır denetimi mekanizmalarının tasarımını tanımlar. Temel kural **"Execution ≠ Data"** prensibidir.

BCIB, Ring3'te izole ve deterministik bir şekilde çalışırken, ABDF immutable (değiştirilemez) ve snapshot-tutarlı veri depolama sağlar. İki sistem arasındaki tek iletişim noktası `Runtime_Bridge` bileşenidir.

### Temel Tasarım Kararları

**Karar 1 — Tam İzolasyon (Execution ≠ Data)**
BCIB doğrudan hafızaya, cihaz sürücülerine veya ABDF'ye erişemez. Tüm işlemler capability tabanlı bir arayüz olan `Runtime_Bridge` üzerinden gerçekleştirilir.

**Karar 2 — Fail-Closed Semantiği**
İzolasyon, yetki (capability) veya boundary (sınır) ihlallerinde sistem `fail-closed` (güvenli kapanma) prensibiyle deterministik bir hata kodu (`BCIB_ERR_*` veya `ABDF_ERR_*`) döndürerek çalışmayı sonlandırır. Güvenlik ihlallerinde kısmi kurtarma yapılamaz.

**Karar 3 — Runtime_Bridge Kernel Boundary Preservation**
Runtime_Bridge, kernel syscall boundary'yi değiştirmez veya bypass etmez. Kernel interaction yalnızca tanımlı syscall yüzeyleri (`SYS_V2_SUBMIT_EXECUTION`) üzerinden olur. Bridge, syscall surface'ın yerini almaz.

**Karar 4 — BCIB Execution Entry Point Control**
BCIB execution yalnızca approved submission path ile başlatılabilir. Direct invocation, test helpers, debug hooks veya internal calls aracılığıyla BCIB runtime'ı çağırmak yasaktır.

**Karar 5 — Opaque Handles (Görünmez Referanslar)**
ABDF nesneleri BCIB'ye doğrudan bellek adresleri (raw pointers) olarak verilmez. Verilere yalnızca context'e bağlı, iptal edilebilir `ABDF_Handle` referansları aracılığıyla erişilir.

**Karar 5 — Opaque Handles (Görünmez Referanslar)**
ABDF nesneleri BCIB'ye doğrudan bellek adresleri (raw pointers) olarak verilmez. Verilere yalnızca context'e bağlı, iptal edilebilir `ABDF_Handle` referansları aracılığıyla erişilir.

**Karar 6 — Immutable Data (Değiştirilemez Veri)**
ABDF'deki veri yansımaları (snapshots) execution sırasında sabittir (immutable). Değişiklikler yalnızca `Runtime_Bridge` üzerinden yeni nesneler oluşturarak veya "append-only" (yalnızca ekleme) uzantılarıyla yapılır.

**Karar 7 — Phase-15 Uyumluluğu**
Phase-16 değişiklikleri, Phase-15'te dondurulan BCIB temel yürütme semantiğini değiştirmez. BCIB opcodeları sadece *niyet (intent)* bildirir, icra `Runtime_Bridge` tarafından gerçekleştirilir.

### Mimari Kısıtlar (Constitutional Compliance)

Bu tasarım aşağıdaki NON_OVERRIDABLE (Vazgeçilemez) anayasal kuralları zorunlu kılar:
- `SECURITY.BOUNDARY.VIOLATION`: Ring3'ten Ring0'a doğrudan erişim engellenir.
- `KERNEL.SAFETY.CRITICAL`: Kernel bütünlüğü ve güvenliği korunur.
- `DETERMINISM.GLOBAL`: Side-effect (yan etki) sıralaması ve determinizmi zorunlu tutulur.
- `MEMORY.CONTRACT.VIOLATION`: Sınırlandırılmış hafıza erişimi (bounded memory) ve raw pointer yasağı.

---

## Architecture

### Katman Modeli ve Sınırlar

```text
┌─────────────────────────────────────────────────────────────┐
│                       Ring3 Userspace                       │
│                                                             │
│   ┌─────────────────┐                                       │
│   │ BCIB_Executor   │───(Intent Only)──┐                    │
│   │ (Sandboxed)     │                  │                    │
│   └─────────────────┘                  ▼                    │
│                        ┌──────────────────────────────┐     │
│                        │       Runtime_Bridge         │     │
│                        │ - Capability Validation      │     │
│                        │ - Handle Translation         │     │
│                        │ - Side-Effect Ordering       │     │
│                        └──────────────────────────────┘     │
│   ┌─────────────────┐                  │                    │
│   │ ABDF Substrate  │◄──(Handles)──────┤                    │
│   │ (Immutable)     │                  │                    │
│   └─────────────────┘                  │                    │
│                                        ▼                    │
└─────────────────────────────────────────────────────────────┘
                                         │
┌────────────────────────────────────────▼────────────────────┐
│                       Ring0 Kernel                          │
│                (SYS_V2_SUBMIT_EXECUTION)                    │
└─────────────────────────────────────────────────────────────┘
```

### Bileşenler ve Arayüzler

#### 1. Runtime_Bridge

**Sorumluluk:** BCIB ve dış sistemler (ABDF, Cihazlar, Kernel) arasındaki tek izinli iletişim arabirimi. Execution_Context başına çalışır. Capability doğrulaması ve `ABDF_Handle` çevirisi yapar.

```rust
pub struct RuntimeBridge;

impl RuntimeBridge {
    /// BCIB niyetlerini (intent) dış dünyada capability ile gerçekleştirir
    pub fn execute_side_effect(
        &self,
        intent: SideEffectIntent,
        capability: &CapabilityToken,
        ctx: &ExecutionContext,
    ) -> Result<SideEffectResult, BcibError>;
    
    /// BCIB'ye ABDF'den veya cihazlardan segment okuma
    pub fn read_segment(
        &self,
        handle: AbdfHandle,
        capability: &CapabilityToken,
    ) -> Result<AbdfSegment, BcibError>;
    
    /// ABDF üzerinde yeni veri veya append-only mutasyon
    pub fn mutate_abdf(
        &self,
        mutation: AbdfMutation,
        capability: &CapabilityToken,
    ) -> Result<AbdfHandle, BcibError>;
}
```

#### 2. ABDF_Handle Management

**Sorumluluk:** Raw bellek pointer'larına olan ihtiyacı ortadan kaldırarak memory safety sağlamak. Opaque handle referanslarını yönetir, execution context'lere bağlar ve yaşam döngülerini (lifecycle) izler.

#### 3. Execution_Sandbox

**Sorumluluk:** BCIB yürütme motorunu kapalı bir kutuya (sandbox) hapsetmek. Bellek limitleri (bounds), kernel çağrısı yasakları ve cross-context izolasyon ihlallerini denetler. Sınır aşıldığı an execution fail-closed prensibiyle sonlandırılır.

---

## Data Models

### Segment Tipleri (ABDF Types)

ABDF verileri tip güvenli segmentler halinde temsil edilir:
- `Input`: BCIB'ye sağlanan salt okunur (read-only) girdi verisi.
- `Event`: Dış dünyadan veya cihazlardan gelen olaylar.
- `DeviceStatus`: Donanım durumu yansımaları.
- `ReadResult`: Okuma işlemleri sonucu dönen veri.
- `ExecutionResult`: Çalıştırma sonucu çıktı verisi.
- `ExecutionTrace`: Deterministik yürütme adımlarının izi (side-effect ordering log).
- `Ref`: Başka bir ABDF nesnesine referans.

### Handle Model

```rust
pub struct AbdfHandle {
    pub id: HandleId,
    pub segment_type: SegmentType,
    pub ctx_id: ExecutionContextId, // Handle hangi context'e ait
    pub status: HandleStatus,       // Valid, Revoked, Expired
}
```

### Error Taxonomy (Hata Kodları)

| Kod | Açıklama |
|-----|----------|
| `BCIB_ERR_ISOLATION_VIOLATION` | Ring0'a erişim, direkt device call, bypass denemesi. |
| `BCIB_ERR_BRIDGE_BYPASS` | Runtime_Bridge atlanarak işlem yapılmaya çalışılması. |
| `BCIB_ERR_CAPABILITY_SCOPE_VIOLATION`| Capability'nin kapsamı dışında kullanılması. |
| `BCIB_ERR_UNDECLARED_SIDE_EFFECT` | Önceden bildirilmemiş yan etki kullanımı. |
| `BCIB_ERR_OPCODE_VIOLATION` | Phase-15 opcode ihlali (doğrudan icra denemesi). |
| `ABDF_ERR_DIRECT_MUTATION` | ABDF'ye doğrudan mutasyon isteği. |
| `BCIB_ERR_ABDF_HANDLE_REVOKED` | İptal edilmiş handle üzerinden erişim. |
| `ABDF_ERR_TYPE_VIOLATION` | Segment tip kısıtlamalarının ihlali. |
| `BCIB_ERR_DEVICE_ACCESS_VIOLATION` | Donanıma doğrudan erişim (MMIO vb.) isteği. |
| `ABDF_BOUNDARY_VIOLATION` | BCIB ile ABDF sınırının delinmesi. |
| `BCIB_ERR_CONTEXT_ISOLATION_VIOLATION`| İki farklı Execution_Context arası yetkisiz erişim. |
| `BCIB_ERR_SANDBOX_ESCAPE` | Sandbox kaçış denemesi. |

---

## Testing Strategy & Correctness Properties

Tasarım, **proptest** kütüphanesi (min 100 iterasyon) ile doğrulanacak Property-Based testlere dayanır.

**Property 1: Execution Isolation Invariant**
*Açıklama:* BCIB yürütmesi hiçbir koşulda kernel memory, device registers veya yasaklı syscall'lara erişemez.
*Test:* Rastgele BCIB talimat dizileri ile test; Ring0/Device erişim tespiti fail ile sonuçlanmalıdır.

**Property 2: Handle Opacity Invariant**
*Açıklama:* ABDF handle'ları raw memory adresi içermez.
*Test:* Handle representation test edilerek dereference edilemeyeceği kanıtlanır.

**Property 3: Capability Scope Invariant**
*Açıklama:* Yetkiler (Capabilities) tanımlandığı context, instruction type ve segment ile sınırlandırılmıştır.
*Test:* Kapsam dışı yetki kullanımı `BCIB_ERR_CAPABILITY_SCOPE_VIOLATION` döndürür.

**Property 4: Immutability Preservation**
*Açıklama:* BCIB yürütülmesi sırasında ABDF verileri sabittir (immutable).
*Test:* Eşzamanlı read operasyonlarında veri değişmemeli, in-place mutasyon oluşmamalıdır.

**Property 5: Side-Effect Determinism**
*Açıklama:* Aynı girdi ile başlatılan BCIB her zaman aynı sıralamada yan etki dizisi üretir.
*Test:* Eşit state ve girdi ile tekrarlı yürütmeler aynı side-effect ordering'i göstermelidir.

**Property 6: Boundary Enforcement**
*Açıklama:* Sınır ihlalleri her zaman fail-closed davranışla sistemi sonlandırır.
*Test:* Doğrudan syscall veya memory bypass girişimleri deterministik hata kodlarıyla sonlanmalıdır.

**Property 7: Handle Revocation**
*Açıklama:* İptal edilmiş (revoked) bir handle ile işlem yapılamaz.
*Test:* Revoke edilmiş handle erişimi `BCIB_ERR_ABDF_HANDLE_REVOKED` döndürür.

**Property 8: Context Isolation**
*Açıklama:* Execution context'leri arası yetkisiz erişim başarısız olur.
*Test:* Cross-context handle veya capability erişimleri `BCIB_ERR_CONTEXT_ISOLATION_VIOLATION` döndürür.

**Property 9: Mutation Path Enforcement**
*Açıklama:* Yalnızca Runtime_Bridge üzerinden ABDF mutasyonuna izin verilir.
*Test:* Doğrudan mutasyon girişimleri `ABDF_ERR_DIRECT_MUTATION` ile sonlanır.

**Property 10: Device Access Isolation**
*Açıklama:* BCIB'den cihaza doğrudan erişim engellenir.
*Test:* Doğrudan I/O ve MMIO girişimleri `BCIB_ERR_DEVICE_ACCESS_VIOLATION` döndürür.

**Property 11: Sandbox Escape Prevention**
*Açıklama:* Sandbox'tan çıkış girişimleri başarısız olur.
*Test:* Kaçış denemeleri fail-closed termination ve `BCIB_ERR_SANDBOX_ESCAPE` döndürür.

**Property 12: Capability Requirement Enforcement**
*Açıklama:* Capability gerektiren operasyonlar, eksik yetki durumunda başarısız olur.
*Test:* `data-mutating` ve `external` tip opcode'lar için yetki yokluğu fail üretir.

---

## CI Gates

Phase-16 entegrasyonu aşağıdaki CI geçitlerinden başarıyla geçmelidir:
1. `ci-gate-bcib-isolation`
2. `ci-gate-abdf-immutability`
3. `ci-gate-boundary-enforcement`
4. `ci-gate-determinism`
5. `ci-gate-capability-enforcement`
6. `ci-gate-fail-closed`

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

---

## Kernel-Level Validation Architecture

### Evidence-Based Security Model

AykenOS güvenlik iddialarını **kanıt tabanlı (evidence-based)** bir modelle doğrular. Userspace testleri veya emüle edilmiş ortamlar, kernel seviyesindeki güvenlik sınırlarını kanıtlamak için yeterli değildir.

**Otorite Hiyerarşisi:**
```
QEMU Kernel Trace (En Yüksek Otorite)
    ↓
Kernel Syscall Dispatcher
    ↓
Boundary Enforcement Layer
    ↓
Userspace Tests (Sadece API Doğrulama)
```

### Canonical Marker Flow (Zorunlu Sıralama)

Fail-closed enforcement'ın kernel seviyesinde çalıştığını kanıtlamak için şu marker akışı **deterministik ve sıralı** olarak görülmelidir:

```
1. BCIB_FORBIDDEN_BEFORE
   ↓ (Userspace: BCIB-role process forbidden syscall denemesi)
   
2. [[AYKEN_SYSCALL_ENTER]]
   ↓ (Kernel: Trap gerçekleşti, syscall dispatcher'a düştü)
   
3. [[AYKEN_BOUNDARY_CHECK]] (opsiyonel ama güçlü)
   ↓ (Kernel: Boundary validation path'ine girdi)
   
4. [[AYKEN_BOUNDARY_KILL]] 🔥 KRİTİK NOKTA
   ↓ (Kernel: Process terminate edildi, fail-closed aktif)
   ↓ (ÖNEMLI: Bu marker scheduler removal'dan ÖNCE emit edilir)
   
5. (BURADA BİTER - başka marker OLMAMALI)
```

### Negative Guarantees (Yasaklı Marker'lar)

`[[AYKEN_BOUNDARY_KILL]]` sonrasında şu marker'lar **ASLA** görülmemelidir:

| Marker | Anlamı | Neden Yasak |
|--------|--------|-------------|
| `BCIB_FORBIDDEN_AFTER` | Execution devam etti | Fail-closed çalışmadı |
| `[[AYKEN_SYSCALL_EXIT]]` | Syscall return oldu | Terminate yerine dönüş yapıldı |
| `[[AYKEN_SCHED_RESUME]]` | Process tekrar schedule edildi | Kill incomplete |
| Aynı process'ten herhangi bir log | Process hala çalışıyor | Hard stop yok |

### Fail-Closed Tanımı (Teknik)

Fail-closed enforcement şu garantileri sağlar:

**Irreversible Termination (Geri Dönüşsüz Sonlandırma):**
- Process scheduler'dan çıkarılır
- Execution slot temizlenir
- Resume path yoktur

**No Continuation (Devam Yok):**
- Kill marker sonrası hiçbir kod çalışmaz
- Syscall return olmaz
- Partial state commit olmaz

**No Recovery (Kurtarma Yok):**
- Sistem violation'ı düzeltmeye çalışmaz
- Retry mekanizması yoktur
- Degraded mode yoktur

**Deterministic Outcome (Deterministik Sonuç):**
- Aynı violation her zaman aynı error code üretir
- Aynı termination sequence izlenir
- Audit trail immutable'dır

### Host vs Kernel Evidence Ayrımı

#### Host-Level Tests (Userspace)
**Kapsam:**
- API contract validation
- Error return code checks
- Data structure logic
- Harness behavior

**YAPAMAZ:**
- Kernel trap path'ini kanıtlayamaz
- Syscall dispatcher behavior'ını doğrulayamaz
- Scheduler termination'ı ispatlayamaz
- Boundary enforcement'ı kernel seviyesinde gösteremez

**Kullanım:**
- Development-time validation
- Unit test coverage
- Regression detection
- API stability checks

#### Kernel-Level Evidence (QEMU Trace)
**Kapsam:**
- Gerçek kernel trap execution
- Syscall dispatcher behavior
- Scheduler state changes
- Boundary enforcement activation
- Process termination proof

**YAPAR:**
- Kernel boundary claims'i kanıtlar
- Fail-closed behavior'ı gösterir
- Security guarantees'i ispat eder
- Production-ready evidence sağlar

**Kullanım:**
- Security audit
- Constitutional compliance
- Production gate validation
- Formal verification input

### CI Gate: ci-gate-fail-closed-proof

**Amaç:** Fail-closed enforcement'ın kernel seviyesinde çalıştığını QEMU trace ile kanıtlamak.

**Input:**
- QEMU kernel trace log (debugcon + serial output)
- Test scenario: BCIB-role process → forbidden syscall attempt

**Validation Logic:**

```bash
# 1. Marker Sequence Check (ZORUNLU SIRALAMA)
grep "BCIB_FORBIDDEN_BEFORE" trace.log
grep "\\[\\[AYKEN_SYSCALL_ENTER\\]\\]" trace.log
grep "\\[\\[AYKEN_BOUNDARY_KILL\\]\\]" trace.log

# 2. Process Identity Validation (KRİTİK)
# Tüm marker'lar aynı process_id'ye ait olmalı
PROCESS_ID=$(grep "BCIB_FORBIDDEN_BEFORE" trace.log | extract_pid)
grep "\\[\\[AYKEN_SYSCALL_ENTER\\]\\].*pid=$PROCESS_ID" trace.log
grep "\\[\\[AYKEN_BOUNDARY_KILL\\]\\].*pid=$PROCESS_ID" trace.log

# 3. Single Kill Validation (KRİTİK)
# Tam olarak 1 tane BOUNDARY_KILL olmalı
KILL_COUNT=$(grep -c "\\[\\[AYKEN_BOUNDARY_KILL\\]\\]" trace.log)
[ "$KILL_COUNT" -eq 1 ] || exit 1

# 4. Bounded Execution Window (KRİTİK)
# ENTER ile KILL arası sınırlı olmalı
ENTER_LINE=$(grep -n "\\[\\[AYKEN_SYSCALL_ENTER\\]\\]" trace.log | cut -d: -f1)
KILL_LINE=$(grep -n "\\[\\[AYKEN_BOUNDARY_KILL\\]\\]" trace.log | cut -d: -f1)
WINDOW=$((KILL_LINE - ENTER_LINE))
[ "$WINDOW" -lt 10 ] || exit 1  # < 10 log lines

# 5. Negative Assertion (KILL SONRASI SCAN)
# KILL marker'dan sonra şunlar OLMAMALI:
grep -A 9999 "\\[\\[AYKEN_BOUNDARY_KILL\\]\\]" trace.log | \
  grep -E "BCIB_FORBIDDEN_AFTER|AYKEN_SYSCALL_EXIT|AYKEN_SCHED_RESUME"
# → EMPTY olmalı (hiçbir match bulmamalı)

# 6. Hard Stop Guarantee
# KILL sonrası aynı process'ten log olmamalı
grep -A 9999 "\\[\\[AYKEN_BOUNDARY_KILL\\]\\]" trace.log | \
  grep "pid=$PROCESS_ID"
# → EMPTY olmalı
```

**Pass Criteria:**
- ✅ Tüm required marker'lar sıralı ve mevcut
- ✅ Tüm marker'lar aynı process_id'ye ait
- ✅ Tam olarak 1 tane `[[AYKEN_BOUNDARY_KILL]]` var (0 veya >1 = FAIL)
- ✅ ENTER ile KILL arası execution window bounded ve deterministik
- ✅ `[[AYKEN_BOUNDARY_KILL]]` scheduler removal'dan ÖNCE emit edilmiş
- ✅ KILL sonrası continuation marker yok
- ✅ Process scheduler'dan düşmüş
- ✅ Execution slot temizlenmiş

**Fail Criteria:**
- ❌ Required marker eksik
- ❌ Marker sırası yanlış
- ❌ Marker'lar farklı process_id'lere ait
- ❌ 0 veya birden fazla `[[AYKEN_BOUNDARY_KILL]]` marker
- ❌ Execution window unbounded veya non-deterministic
- ❌ KILL sonrası continuation marker var
- ❌ Process hala çalışıyor

**Output:**
- `failclosed_proof_evidence.json` - marker flow ve validation sonuçları
- Failure durumunda: `FAIL_CLOSED_PROOF_INVALID` error code

### Audit Script Mantığı

Script şu adımları izler:

```python
def validate_fail_closed_proof(trace_log):
    # 1. Marker extraction
    markers = extract_markers_in_order(trace_log)
    
    # 2. Required sequence check
    assert markers[0] == "BCIB_FORBIDDEN_BEFORE"
    assert markers[1] == "[[AYKEN_SYSCALL_ENTER]]"
    assert markers[2] == "[[AYKEN_BOUNDARY_KILL]]"
    
    # 3. Process identity validation (CRITICAL)
    process_id = extract_process_id(markers[0])
    assert extract_process_id(markers[1]) == process_id
    assert extract_process_id(markers[2]) == process_id
    
    # 4. Single kill validation (CRITICAL)
    kill_count = count_markers(trace_log, "[[AYKEN_BOUNDARY_KILL]]")
    assert kill_count == 1  # exactly one, not zero, not multiple
    
    # 5. Bounded execution window (CRITICAL)
    enter_position = find_marker_position(trace_log, "[[AYKEN_SYSCALL_ENTER]]")
    kill_position = find_marker_position(trace_log, "[[AYKEN_BOUNDARY_KILL]]")
    window_size = kill_position - enter_position
    assert window_size < 10  # bounded to < 10 log lines
    assert is_deterministic(window_size)  # same violation = same window
    
    # 6. Scan after kill (CRITICAL)
    after_kill = trace_log[kill_position:]
    
    # 7. Negative assertions
    assert "BCIB_FORBIDDEN_AFTER" not in after_kill
    assert "[[AYKEN_SYSCALL_EXIT]]" not in after_kill
    assert "[[AYKEN_SCHED_RESUME]]" not in after_kill
    assert no_process_logs_after_kill(after_kill, process_id)
    
    # 8. Deterministic error code check
    assert extract_error_code(trace_log) == "BCIB_ERR_ISOLATION_VIOLATION"
    
    return PROOF_VALID
```

### Gold Standard (Hedef Log Formatı)

Başarılı bir fail-closed proof şöyle görünmelidir:

```
[U] BCIB_FORBIDDEN_BEFORE: Process 42 attempting SYS_V2_SUBMIT_EXECUTION
[[AYKEN_SYSCALL_ENTER]] syscall=1001 pid=42
[[AYKEN_BOUNDARY_CHECK]] role=BCIB syscall=1001 allowed=false
[[AYKEN_BOUNDARY_KILL]] pid=42 reason=FORBIDDEN_SYSCALL

(LOG BURADA BİTER - başka satır yok)
```

**Kritik Noktalar:**
- Tüm marker'lar aynı pid (42)
- Tam olarak 1 tane BOUNDARY_KILL
- ENTER ile KILL arası 2 satır (bounded window)
- KILL sonrası hiçbir log yok

**Yanlış Örnek (Fail):**

```
[U] BCIB_FORBIDDEN_BEFORE: Process 42 attempting SYS_V2_SUBMIT_EXECUTION
[[AYKEN_SYSCALL_ENTER]] syscall=1001 pid=42
[[AYKEN_BOUNDARY_KILL]] pid=42 reason=FORBIDDEN_SYSCALL
[[AYKEN_SYSCALL_EXIT]] syscall=1001 result=0  ← ❌ FAIL: syscall returned
[U] BCIB_FORBIDDEN_AFTER: Execution continued  ← ❌ FAIL: execution continued
```

### En Sık Yapılan Hatalar

| Hata | Açıklama | Tespit |
|------|----------|--------|
| **Fake PASS** | Sadece BEFORE + KILL var, ama ENTER yok | Userspace simulate, kernel trap yok |
| **Soft Fail** | AFTER log geliyor | Kill çalışmamış, execution devam etmiş |
| **Return Path Açık** | EXIT marker var | Syscall dönmüş, terminate olmamış |
| **Scheduler Kaçırıyor** | Process tekrar çalışıyor | Kill incomplete, resume olmuş |
| **Process ID Mismatch** | Marker'lar farklı pid'lere ait | Process A killed, Process B logged |
| **Multiple Kill** | Birden fazla BOUNDARY_KILL | Unstable system, race condition |
| **Unbounded Window** | ENTER ile KILL arası çok uzun | System hang, delayed enforcement |

### Integration with Existing Gates

`ci-gate-fail-closed-proof` diğer gate'lerle şu şekilde entegre olur:

```
ci-gate-hygiene (code quality)
    ↓
ci-gate-constitutional (NON_OVERRIDABLE rules)
    ↓
ci-gate-bcib-isolation (execution isolation)
    ↓
ci-gate-boundary-enforcement (boundary controls)
    ↓
ci-gate-fail-closed-proof (kernel-level evidence) ← YENİ
    ↓
MERGE ALLOWED
```

**Blocker Davranışı:**
- `ci-gate-fail-closed-proof` FAIL ederse merge BLOCKED
- Kernel trace eksikse gate FAIL
- Continuation marker varsa gate FAIL
- Marker sequence yanlışsa gate FAIL

---

## CI Gates

Phase-16 entegrasyonu aşağıdaki CI geçitlerinden başarıyla geçmelidir:
1. `ci-gate-bcib-isolation`
2. `ci-gate-abdf-immutability`
3. `ci-gate-boundary-enforcement`
4. `ci-gate-determinism`
5. `ci-gate-capability-enforcement`
6. `ci-gate-fail-closed`
7. `ci-gate-fail-closed-proof` ← **YENİ: Kernel-level evidence validation**

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


---

## Runtime_Bridge QEMU Proof Infrastructure (Task 5)

### Overview

Task 5 requires QEMU kernel trace evidence to prove Runtime_Bridge syscalls (1012/1013/1014) reach the kernel dispatcher and return correctly. This section documents the proof infrastructure created to validate Runtime_Bridge execution.

### Boot Path Architecture

**Critical Decision:** AykenOS uses OVMF + EFI.img boot model, NOT `-kernel`/`-initrd`.

The Runtime_Bridge proof harness MUST use the same boot path as the working syscall validation infrastructure:

```bash
qemu-system-x86_64 \
    -machine q35 \
    -drive if=pflash,format=raw,readonly=on,file=$OVMF_CODE \
    -drive if=pflash,format=raw,file=$OVMF_VARS_COPY \
    -drive format=raw,file=$EFI_IMG \
    -serial file:$SERIAL_LOG \
    -chardev file,id=dbgcon,path=$DEBUGCON_LOG \
    -device isa-debugcon,iobase=0xe9,chardev=dbgcon \
    -m 256M \
    -no-reboot \
    -no-shutdown \
    -display none
```

**Key Components:**
- **OVMF Firmware**: UEFI firmware for x86_64 (CODE + VARS)
- **EFI.img**: Bootable disk image containing kernel + userspace payload
- **Deterministic Boot**: Blank OVMF VARS copy prevents stale NVRAM interference
- **Dual Channels**: Both debugcon (0xE9) and serial output captured

### Runtime_Bridge Test Payload

**Location:** `userspace/minimal/minimal_runtime_bridge_test.S`

**Purpose:** Exercises Runtime_Bridge syscalls and emits validation markers

**Syscalls Tested:**
- `SYS_V2_DEVICE_OPERATION` (1012) - Device interaction
- `SYS_V2_EXTERNAL_CALL` (1013) - External system calls
- `SYS_V2_ABDF_OPERATION` (1014) - ABDF data operations

**Marker Emission:**
- Uses `SYS_V2_DEBUG_PUTCHAR` (1010) to emit markers
- Markers appear in debugcon/serial logs
- Deterministic ordering for validation

### Marker Contract

Runtime_Bridge test emits these markers in order:

```
[U][RUNTIME_BRIDGE_TEST_START]
[U][RUNTIME_BRIDGE_DEVICE_OP_BEFORE]
  → INT 0x80 (syscall 1012)
  → [[AYKEN_SYSCALL_ENTER]]
  → [[AYKEN_SYSCALL_EXIT]]
[U][RUNTIME_BRIDGE_DEVICE_OP_AFTER]
[U][RUNTIME_BRIDGE_EXTERNAL_CALL_BEFORE]
  → INT 0x80 (syscall 1013)
  → [[AYKEN_SYSCALL_ENTER]]
  → [[AYKEN_SYSCALL_EXIT]]
[U][RUNTIME_BRIDGE_EXTERNAL_CALL_AFTER]
[U][RUNTIME_BRIDGE_ABDF_OP_BEFORE]
  → INT 0x80 (syscall 1014)
  → [[AYKEN_SYSCALL_ENTER]]
  → [[AYKEN_SYSCALL_EXIT]]
[U][RUNTIME_BRIDGE_ABDF_OP_AFTER]
[U][RUNTIME_BRIDGE_TEST_COMPLETE]
```

**Validation Logic:**
- All `[U]` markers must be present (userspace execution)
- At least 3 `[[AYKEN_SYSCALL_ENTER]]` markers (kernel entry)
- At least 3 `[[AYKEN_SYSCALL_EXIT]]` markers (kernel return)
- Completion marker proves test finished

### QEMU Proof Harness

**Location:** `scripts/qemu-runtime-bridge-proof-harness.sh`

**Responsibilities:**
1. Resolve OVMF firmware (supports Linux/macOS)
2. Create temporary OVMF VARS copy
3. Launch QEMU with correct boot path
4. Capture debugcon and serial logs
5. Validate channel integrity (fail if both empty)
6. Run Runtime_Bridge audit script

**OVMF Firmware Resolution:**
Searches standard locations:
- `/usr/share/OVMF/OVMF_CODE_4M.fd` (Linux, 4MB)
- `/usr/share/OVMF/OVMF_CODE.fd` (Linux, standard)
- `/usr/share/edk2/ovmf/OVMF_CODE.fd` (Alternative Linux)
- `/usr/share/qemu/OVMF_CODE.fd` (QEMU-specific)
- `/opt/homebrew/share/qemu/edk2-x86_64-code.fd` (macOS Homebrew)

**Channel Integrity:**
- HARD FAIL if both debugcon and serial are empty (0 bytes)
- Prevents false positives from boot failures
- Ensures observable evidence exists

### Runtime_Bridge Audit Script

**Location:** `tools/validation/runtime_bridge_audit.sh`

**Purpose:** Validate Runtime_Bridge marker flow in QEMU traces

**Validation Steps:**
1. Count all Runtime_Bridge markers
2. Verify TEST_START marker present
3. Verify DEVICE_OP_BEFORE/AFTER pair
4. Verify EXTERNAL_CALL_BEFORE/AFTER pair
5. Verify ABDF_OP_BEFORE/AFTER pair
6. Verify TEST_COMPLETE marker present
7. Count SYSCALL_ENTER markers (expect ≥3)
8. Count SYSCALL_EXIT markers (expect ≥3)

**Output:**
- Clear PASS/FAIL verdict
- Marker counts for debugging
- Actionable warnings for missing markers
- Exit code 0 on PASS, non-zero on FAIL

### Integration with Build System

**Minimal Mode:** `runtime-bridge-test`

**Build Command:**
```bash
USER_MINIMAL_MODE=runtime-bridge-test make efi-img
```

**Makefile Integration:**
```makefile
else ifeq ($(MINIMAL_MODE),runtime-bridge-test)
MINIMAL_SRC := minimal_runtime_bridge_test.S
```

**Effect:**
- Embeds Runtime_Bridge test into EFI.img
- Kernel boots and launches Runtime_Bridge test
- Test executes syscalls 1012/1013/1014
- Markers appear in QEMU logs

### Evidence Generation Workflow

```
1. Build EFI.img with runtime-bridge-test mode
   └─> USER_MINIMAL_MODE=runtime-bridge-test make efi-img
   └─> ⏳ PENDING

2. Run QEMU proof harness
   └─> ./scripts/qemu-runtime-bridge-proof-harness.sh
   └─> ⏳ PENDING

3. Harness launches QEMU with OVMF + EFI.img
   └─> Captures debugcon and serial logs
   └─> ⏳ PENDING

4. Harness runs audit script on logs
   └─> tools/validation/runtime_bridge_audit.sh
   └─> ⏳ PENDING

5. Audit script validates marker flow
   └─> ⏳ PENDING: Marker presence not confirmed
   └─> ⏳ PENDING: Runtime_Bridge syscalls 1012/1013/1014 not yet validated

6. Evidence stored in evidence/runtime-bridge-proof/
   └─> ⏳ PENDING: No verified trace evidence yet
```

### Current Status (2026-04-12)

**STATUS: PROOF NOT YET ESTABLISHED**

**Infrastructure Ready:**
- ✅ QEMU harness uses correct OVMF + EFI.img boot path
- ✅ Runtime_Bridge marker contract defined
- ✅ Runtime_Bridge audit script created
- ✅ Harness supports multiple OVMF locations
- ✅ Deterministic boot with blank OVMF VARS
- ✅ Channel integrity validation
- ✅ Runtime_Bridge test payload created and integrated

**NOT VERIFIED:**
- ❌ QEMU harness execution - no verified kernel trace evidence
- ❌ Runtime_Bridge marker presence not confirmed in actual trace
- ❌ Execution path not proven with observable evidence
- ❌ Hygiene gate FAILING (3 dirty tracked files)

**Pending:**
- ⏳ Run QEMU harness and capture actual trace
- ⏳ Verify markers appear in trace output
- ⏳ Resolve hygiene violations (dirty tracked files)
- ⏳ Integrate real DevFS handlers (replace 0xDEADBEEF stub)
- ⏳ Integrate real ABDF handlers (replace fake ABDF stub)
- ⏳ Create forbidden test for fail-closed validation
- ⏳ Pass `ci-gate-fail-closed-proof`

### Comparison: General Syscall vs Runtime_Bridge Tests

| Aspect | General Syscall Test | Runtime_Bridge Test |
|--------|---------------------|---------------------|
| **Purpose** | Validate syscall roundtrip (any syscall) | Validate Runtime_Bridge syscalls (1012/1013/1014) |
| **Markers** | `[U][SYSCALL_OK]` or `[[AYKEN_SYSCALL_V2_OK]]` | Runtime_Bridge-specific markers |
| **Audit Script** | `phase_4_4_syscall_roundtrip_audit.sh` | `runtime_bridge_audit.sh` |
| **Scope** | Phase 4.4 closure (general syscall path) | Phase-16 Task 5 (Runtime_Bridge path) |
| **Boot Path** | OVMF + EFI.img | OVMF + EFI.img (same) |
| **Evidence** | Proves syscall infrastructure works | Proves Runtime_Bridge syscalls work |

**Key Insight:** These are two different tests with different markers and different audit scripts. Using the wrong audit script produces false negatives.

### References

- Working OVMF pattern: `tools/validation/syscall_roundtrip_test.sh`
- Runtime_Bridge test payload: `userspace/minimal/minimal_runtime_bridge_test.S`
- Phase 4.4 audit (for comparison): `tools/validation/phase_4_4_syscall_roundtrip_audit.sh`
- Task 5 progress: `.kiro/specs/phase16-bcib-abdf-isolation-contracts/TASK_5_PROGRESS_2026_04_12.md`


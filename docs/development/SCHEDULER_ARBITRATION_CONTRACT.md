# Scheduler Arbitration Contract (Yol A)

> Legacy notice (2026-02-28):
> This document is historical/contextual and is no longer the normative
> scheduler protocol reference for C1.
>
> Current normative references:
> - `docs/governance/MAILBOX_PROTOCOL_V1_FREEZE.md`
> - `kernel/include/sched_mailbox_abi.h`
> - `constitution/abi_mailbox.json`

**Project:** AykenOS  
**Version:** 1.0  
**Status:** LEGACY (SUPERSEDED FOR C1)  
**Effective Date:** 2026-02-21  
**Owner:** AykenOS Core Architecture Team

---

## 1. Genel Bakış

Bu belge, AykenOS'un Ring0/Ring3 scheduler arbitration sözleşmesini (Yol A) tanımlar. Bu sözleşme, scheduler policy kararlarının Ring3'te alınmasını ve Ring0'ın sadece mekanizma sağlamasını garanti eder.

### Temel Prensipler

- **Ring3 Policy:** Tüm scheduling kararları Ring3'te alınır
- **Ring0 Mechanism:** Ring0 sadece context switch mekanizması sağlar
- **Hint-Based:** Ring3, Ring0'a öneri (hint) gönderir, emir vermez
- **Fail-Closed:** Ring0, güvenli olmayan durumda sistem durdurur

---

## 2. Mimari Sözleşme

### 2.1 Ring3 Sorumlulukları

Ring3 scheduler policy şunlardan sorumludur:

1. **Process Selection:** Hangi process'in çalışacağına karar verme
2. **Priority Management:** Process önceliklerini yönetme
3. **Time Slice Allocation:** Her process'e ne kadar CPU zamanı verileceğine karar verme
4. **Load Balancing:** CPU'lar arası yük dengeleme (multi-core)
5. **Scheduling Hints:** Ring0'a `stage_next()` ile öneri gönderme

### 2.2 Ring0 Sorumlulukları

Ring0 scheduler mechanism şunlardan sorumludur:

1. **Context Switch Execution:** Process context'lerini değiştirme
2. **Hint Validation:** Ring3'ten gelen önerileri doğrulama
3. **Safety Enforcement:** Güvenli olmayan geçişleri veto etme
4. **Fail-Closed Behavior:** Kabul edilebilir aday yoksa sistem durdurma
5. **IRQ-Tail Preemption:** Timer interrupt sonrası preemption mekanizması

---

## 3. Scheduler Mailbox Protokolü

### 3.1 Mailbox Yapısı

```c
typedef struct scheduler_mailbox {
    uint64_t staged_pid;           // Ring3'ün önerdiği process ID
    uint64_t staged_timestamp;     // Öneri zamanı
    uint32_t staged_valid;         // Öneri geçerli mi?
    uint32_t veto_reason;          // Ring0 veto nedeni (varsa)
} scheduler_mailbox_t;
```

### 3.2 Ring3 → Ring0 Hint Flow

```
1. Ring3 scheduler policy çalışır
2. Ring3, next process'i seçer
3. Ring3, scheduler_stage_next(pid) çağırır
4. Mailbox güncellenir: staged_pid = pid, staged_valid = 1
5. Ring0, timer interrupt'ta mailbox'ı okur
6. Ring0, staged process'i validate eder
7. Ring0, kabul veya veto kararı verir
```

### 3.3 Validation Kriterleri

Ring0, aşağıdaki kriterlere göre hint'i validate eder:

```c
bool validate_staged_process(uint64_t pid) {
    proc_t *proc = find_process_by_pid(pid);
    
    // Kritik validasyonlar
    if (!proc) return false;                    // Process mevcut değil
    if (proc->state != PROC_STATE_READY) return false;  // Ready state'de değil
    if (!proc->context) return false;           // Context yok
    if (!proc->page_table) return false;        // Page table yok
    
    // Context sanity checks
    if (proc->context->rip == 0) return false;  // Invalid RIP
    if (proc->context->rsp == 0) return false;  // Invalid RSP
    
    return true;
}
```

### 3.4 Veto Nedenleri

Ring0, şu durumlarda hint'i veto eder:

| Veto Kodu | Neden | Açıklama |
|-----------|-------|----------|
| `VETO_INVALID_PID` | Process bulunamadı | PID geçersiz veya process yok |
| `VETO_NOT_READY` | Process ready değil | Process blocked veya terminated |
| `VETO_NO_CONTEXT` | Context yok | Process context'i initialize edilmemiş |
| `VETO_NO_PAGE_TABLE` | Page table yok | Virtual memory yapısı eksik |
| `VETO_INVALID_RIP` | RIP geçersiz | Instruction pointer 0 veya invalid |
| `VETO_INVALID_RSP` | RSP geçersiz | Stack pointer 0 veya invalid |

---

## 4. Fail-Closed Semantiği

### 4.1 Fail-Closed Prensibi

Ring0, güvenli olmayan durumda **asla** rastgele bir process'e geçiş yapmaz. Bunun yerine:

```c
void sched_yield_irq(void) {
    // Mailbox'tan staged process'i al
    uint64_t staged_pid = scheduler_mailbox.staged_pid;
    
    // Validate et
    if (!validate_staged_process(staged_pid)) {
        // FAIL-CLOSED: Güvenli olmayan durum
        fb_print("[SCHED] No valid candidate, halting\n");
        cli();  // Interrupt'ları kapat
        hlt();  // CPU'yu durdur
        // Sistem burada kalır, reboot gerekir
    }
    
    // Geçerli ise context switch yap
    proc_t *next = find_process_by_pid(staged_pid);
    context_switch(current_proc, next);
}
```

### 4.2 Fail-Closed Senaryoları

Fail-closed davranışı şu durumlarda tetiklenir:

1. **No Staged Process:** Mailbox'ta geçerli öneri yok
2. **Invalid Staged Process:** Önerilen process validate edilemiyor
3. **Veto After Arm:** Scheduler armed olduktan sonra veto
4. **Context Corruption:** Process context'i corrupt olmuş

### 4.3 Recovery Mekanizması

Fail-closed durumundan kurtulma:

- **Watchdog Timer:** Sistem watchdog timer ile reboot edilir
- **Manual Reboot:** Kullanıcı manuel reboot yapar
- **Debug Mode:** Debug build'lerde panic handler devreye girer

---

## 5. IRQ-Tail Preemption

### 5.1 Preemption Flow

```
1. Timer interrupt (100 Hz) tetiklenir
2. IRQ handler çalışır
3. IRQ handler, sched_request_resched_irq() çağırır
4. Deferred preemption flag set edilir
5. IRQ handler return eder
6. Syscall exit path, sched_take_resched() kontrol eder
7. Flag set ise, sched_yield() çağrılır
8. Context switch gerçekleşir
```

### 5.2 Deferred Preemption API

```c
// IRQ context'te preemption request et
void sched_request_resched_irq(void);

// Syscall exit'te preemption flag kontrol et
uint32_t sched_take_resched(void);

// Preemption gerçekleştir
void sched_yield(void);
```

### 5.3 Preemption Marker Contract

Preemption doğrulaması için marker sistemi:

```c
// Ring3 user process marker gönderir
"[U][SYSCALL_OK]"

// Ring0 kernel marker yanıtlar
"[[AYKEN_SYSCALL_V2_OK]]"
```

CI gate, bu marker'ları parse ederek preemption'ın çalıştığını doğrular.

---

## 6. Bridge Syscall Window

### 6.1 Reserved Range

Scheduler arbitration için reserved syscall range:

```c
#define SCHED_BRIDGE_BASE  0x90
#define SCHED_BRIDGE_LAST  0x9F
// Total: 16 syscalls reserved
```

Bu range, execution-centric `SYS_V2` (1000-1011) aralığından ayrıdır.

### 6.2 Bridge Syscalls

| Syscall | ID | Açıklama |
|---------|-----|----------|
| `sched_stage_next` | 0x90 | Ring3'ten Ring0'a process hint gönder |
| `sched_get_veto` | 0x91 | Son veto nedenini al |
| `sched_mailbox_status` | 0x92 | Mailbox durumunu sorgula |
| (reserved) | 0x93-0x9F | Gelecek kullanım için reserved |

### 6.3 Stage Next Syscall

```c
// Ring3 kullanımı
uint64_t scheduler_stage_next(uint64_t pid) {
    return syscall(0x90, pid, 0, 0, 0);
}

// Ring0 implementasyonu
uint64_t sys_sched_stage_next(uint64_t pid) {
    scheduler_mailbox.staged_pid = pid;
    scheduler_mailbox.staged_timestamp = get_system_time();
    scheduler_mailbox.staged_valid = 1;
    scheduler_mailbox.veto_reason = 0;
    return 0;
}
```

---

## 7. Fallback Policy Isolation

### 7.1 Fallback Durumu

AykenOS, Ring3 scheduler tam operasyonel olana kadar geçici bir fallback policy içerir:

```c
// Makefile
AYKEN_SCHED_FALLBACK ?= 0  // Default: OFF

// kernel/sched/sched.h
#define AYKEN_SCHED_FALLBACK 0  // Constitutional requirement
```

### 7.2 Fallback Removal Plan

1. **Phase 4.5:** Ring3 scheduler tam implementasyonu
2. **Phase 4.6:** Fallback feature flag ile izole edilir
3. **Phase 5.0:** Fallback tamamen kaldırılır

### 7.3 CI Enforcement

```bash
# Constitutional gate, fallback'in kapalı olduğunu doğrular
make ci-gate-constitutional

# Fallback açıksa CI fail eder
```

---

## 8. Test ve Validation

### 8.1 Unit Tests

```c
// Test: Ring3 hint kabul edilir
void test_valid_hint_accepted(void) {
    proc_t *proc = create_test_process();
    scheduler_stage_next(proc->pid);
    assert(scheduler_mailbox.staged_pid == proc->pid);
    assert(scheduler_mailbox.staged_valid == 1);
}

// Test: Invalid hint veto edilir
void test_invalid_hint_vetoed(void) {
    scheduler_stage_next(9999);  // Invalid PID
    sched_yield_irq();
    assert(scheduler_mailbox.veto_reason == VETO_INVALID_PID);
}

// Test: Fail-closed davranışı
void test_fail_closed_behavior(void) {
    scheduler_mailbox.staged_valid = 0;
    // sched_yield_irq() burada halt etmeli
    // Test framework, halt'ı catch eder
}
```

### 8.2 Integration Tests

```bash
# Preemption marker testi
make run-preempt

# Strict marker mode (fallback kapalı)
make run-preempt-strict

# CI gate
make ci-gate-syscall-v2-runtime
```

### 8.3 Performance Tests

```bash
# Context switch latency
make ci-gate-performance

# Baseline: ±5% threshold
# Metric: context_switch_latency_ms_proxy
```

---

## 9. Güvenlik Garantileri

### 9.1 Privilege Escalation Prevention

- Ring3, Ring0'ı **asla** doğrudan kontrol edemez
- Ring0, Ring3 hint'lerini **her zaman** validate eder
- Invalid hint, **asla** execute edilmez

### 9.2 Denial of Service Prevention

- Ring3, sürekli invalid hint göndererek DoS yapamaz
- Ring0, fail-closed ile sistemi korur
- Watchdog timer, deadlock'ları önler

### 9.3 Information Disclosure Prevention

- Ring3, Ring0 internal state'e erişemez
- Mailbox, sadece minimal bilgi içerir
- Veto nedenleri, debug bilgisi sızdırmaz

---

## 10. Performans Özellikleri

### 10.1 Latency

- **Hint Latency:** < 1 μs (mailbox write)
- **Validation Latency:** < 500 ns (sanity checks)
- **Context Switch Latency:** 1-2 μs (measured)

### 10.2 Throughput

- **Hints per Second:** > 100,000 (100 Hz timer)
- **Context Switches per Second:** > 10,000

### 10.3 Overhead

- **Mailbox Memory:** 32 bytes
- **Validation Code:** < 100 instructions
- **Total Overhead:** < 1% CPU time

---

## 11. Gelecek Geliştirmeler

### 11.1 Multi-Core Support

- Per-CPU mailbox'lar
- CPU affinity hints
- Load balancing hints

### 11.2 Real-Time Support

- Priority inheritance hints
- Deadline scheduling hints
- Latency guarantees

### 11.3 AI Integration

- ML-based scheduling hints
- Predictive preemption
- Workload classification

---

## 12. Referanslar

- `kernel/sched/sched.h` - Scheduler mechanism API
- `kernel/sched/sched.c` - Scheduler implementation
- `userspace/libayken/scheduler/` - Ring3 scheduler policy
- `ARCHITECTURE_FREEZE.md` - Freeze sözleşmesi
- `docs/architecture-board/decisions/20260214-scheduler-arbitration-contract.md` - Karar kaydı

---

**© 2026 Kenan AY - AykenOS Project**

# AykenOS Driver + BCIB + ABDF Detaylı Teknik İmplementasyon Planı

**Tarih:** 11 Nisan 2026  
**Yazar:** Kenan AY - Architectural Steward  
**Kapsam:** Execution closure, hardware discovery, driver omurgası, ABDF device segments, BCIB device bridge  
**Mimari Eksen:** Ring0 = mekanizma, Ring3 = politika  
**Durum:** Mevcut kod tabanı analizi tamamlandı

---

## 0. Yönetici Özeti

Bu plan, AykenOS'un mevcut execution-centric mimarisini bozmadan gerçek donanım hattını sisteme bağlamak için gerekli teknik adımları detaylandırır. Plan, mevcut kod tabanının kapsamlı analizine dayanır ve her fazda yapılması gereken spesifik implementasyonları içerir.

### Kritik Bulgular

**Güçlü Çekirdek:**
- BCIB v0.2 frozen format mevcut (`ayken-core/crates/bcib/`)
- ABDF segment modeli tanımlı (`ayken-core/crates/abdf/`)
- BCIB Execution Engine v3 üç katmanlı mimari tamamlanmış
- Execution slot mekanizması operasyonel (`kernel/sys/execution_slot.c`)
- DevFS Ring3 policy modeli kurulmuş (`kernel/fs/devfs.c`)
- Keyboard driver stub mevcut (`kernel/drivers/console/keyboard.c`)
- Capability manager altyapısı hazır

**Kritik Eksikler:**
- Phase 16 Faz B kapanmamış: Ring3 BCIB execution worker eksik
- Gerçek `submit_execution/wait_result` kernel path'leri eksik
- Kernel determinism proof kapanmamış
- PCI enumeration yok
- Device registry/auto-bind sistemi yok
- Gerçek driver'lar eksik (keyboard stub seviyesinde)
- ABDF device segment tipleri tanımsız (InputEvent, DeviceStatus, ReadResult)
- BCIB device opcode'ları eksik
- DevFS → BCIB runtime bridge eksik

### Öncelik Sırası

1. **Phase 0: Execution Closure** (BLOCKER - Priority 1)
2. **Phase 1: Hardware Discovery** (PCI enumeration)
3. **Phase 2: Driver Registry ve Auto-Bind**
4. **Phase 3: İlk Gerçek Driver** (PS/2 Keyboard)
5. **Phase 4: ABDF Device Segments**
6. **Phase 5: BCIB Device Bridge**
7. **Phase 6: Device → ABDF → BCIB Pipeline**
8. **Phase 7: Semantic Layer**
9. **Phase 8: Security Hardening**

---

## 1. Mimari Sınırlar ve Hükümler

### 1.1 Ring0 Sınırı (Mekanizma)

Ring0'da YALNIZCA şunlar kalmalı:
- PCI scan / hardware discovery
- Port I/O / MMIO / IRQ bağlama
- Driver probe/init/read/write/poll mekanizması
- DevFS publish ve node dispatch mekanizması
- Syscall v2 execution submission/wait mekanizması

Ring0 YAPAMAZ:
- AI inference
- Policy kararı
- Semantic yorum
- Scheduler policy logic
- Kullanıcı niyeti çözümleme

### 1.2 Ring3 Sınırı (Politika)

Ring3'te şunlar kalmalı:
- BCIB yorumlama
- Runtime bridge
- Semantic CLI
- AI runtime
- Device verisinin anlamlandırılması
- Policy ve capability kararı

### 1.3 BCIB İzolasyon Sınırı

BCIB şu sınırlar içinde kalmalı:
- Syscall çağırmaz
- Driver çağırmaz
- Raw IRQ, MMIO, I/O port bilmez
- Kernel pointer veya device pointer taşımaz
- Yalnızca ABDF segmentleri ve runtime bridge üzerinden dış dünyayla temas eder

### 1.4 ABDF Rolü

ABDF:
- Driver iç veri yapısı DEĞİL
- Typed runtime state/result surface
- Replay ve determinism için authoritative typed representation

---

## PHASE 0: Execution Closure (PRODUCTION BLOCKER)

### Amaç

Host runtime'da kanıtlanmış execution zincirini gerçek QEMU/kernel sonucu ile kapatmak.

### Neden Önce Bu?

Phase 16 Faz B kapanmadan şu iddia yapılamaz:
> "Aynı BCIB → aynı kernel sonucu"

Şu an kanıtlanan yüzey sadece host runtime determinism'dir; kernel determinism closure henüz tamamlanmamıştır.

### Mevcut Durum Analizi

**Mevcut:**
- `kernel/sys/execution_slot.c`: Execution slot state machine mevcut
- State transitions: CREATED → READY → RUNNING → COMPLETED → RESULT_MAPPED
- Trace mechanism: Her transition kaydediliyor
- Hash mechanism: SHA256 result fingerprinting mevcut
- `userspace/bcib-runtime/src/lib.rs`: Runtime yapısı mevcut ama execution worker eksik

**Eksik:**
- Ring3 BCIB execution worker payload
- Gerçek `SYS_V2_SUBMIT_EXECUTION` implementation
- Gerçek `SYS_V2_WAIT_RESULT` implementation
- Kernel result fingerprint üretimi
- Host runtime sonucu ile kernel sonucu karşılaştırma yüzeyi

### İmplementasyon Detayları

#### 0.1 Ring3 BCIB Execution Worker

**Dosya:** `userspace/bcib-runtime/src/execution_worker.rs`

```rust
//! BCIB Execution Worker - Ring3 execution payload
//! 
//! Bu modül, kernel execution slot'tan BCIB graph'ı alır,
//! yorumlar ve sonucu geri yazar.

use crate::types::{BcibError, ExecutionResult};
use crate::execution_runtime::BcibExecutionRuntime;
use crate::abdf_boundary::AbdfHandle;

/// Execution worker configuration
pub struct ExecutionWorkerConfig {
    pub max_instructions: u64,
    pub timeout_ms: u64,
    pub enable_tracing: bool,
}

/// Execution worker - kernel execution slot ile çalışır
pub struct ExecutionWorker {
    runtime: BcibExecutionRuntime,
    config: ExecutionWorkerConfig,
}

impl ExecutionWorker {
    pub fn new(config: ExecutionWorkerConfig) -> Self {
        Self {
            runtime: BcibExecutionRuntime::new(),
            config,
        }
    }

    /// Kernel execution slot'tan BCIB graph'ı al ve çalıştır
    pub fn execute_from_slot(&mut self, execution_id: u64) -> Result<ExecutionResult, BcibError> {
        // 1. Kernel'dan BCIB graph'ı map et (EXECUTION_PAYLOAD_VA)
        let bcib_data = self.map_bcib_payload(execution_id)?;
        
        // 2. BCIB graph'ı parse et
        let graph = self.parse_bcib_graph(&bcib_data)?;
        
        // 3. Runtime'da çalıştır
        let result = self.runtime.execute_graph(&graph)?;
        
        // 4. Sonucu kernel execution slot'a yaz
        self.write_result_to_slot(execution_id, &result)?;
        
        // 5. Result fingerprint üret
        let fingerprint = self.compute_result_fingerprint(&result)?;
        
        Ok(ExecutionResult {
            execution_id,
            fingerprint,
            output_size: result.output_size,
        })
    }

    fn map_bcib_payload(&self, execution_id: u64) -> Result<Vec<u8>, BcibError> {
        // EXECUTION_PAYLOAD_VA adresinden BCIB graph'ı oku
        // Bu syscall v2 map_memory ile yapılacak
        todo!("Implement syscall v2 map_memory integration")
    }

    fn parse_bcib_graph(&self, data: &[u8]) -> Result<BcibGraph, BcibError> {
        // BCIB v0.2 format parse
        todo!("Implement BCIB graph parsing")
    }

    fn write_result_to_slot(&self, execution_id: u64, result: &ExecutionResult) -> Result<(), BcibError> {
        // Execution output window'a sonucu yaz
        todo!("Implement result writing to execution slot")
    }

    fn compute_result_fingerprint(&self, result: &ExecutionResult) -> Result<[u8; 32], BcibError> {
        // SHA256 fingerprint hesapla
        todo!("Implement SHA256 fingerprinting")
    }
}
```

**UYARI:** Bu worker pointer-free kalmalı, syscall dışında kernel'a dokunmamalı.

#### 0.2 Kernel Execution Slot BCIB Dispatch

**Dosya:** `kernel/sys/execution_slot.c` (ekleme)

```c
/**
 * @brief Execute BCIB graph in Ring3 worker
 * 
 * Bu fonksiyon execution slot'taki BCIB graph'ı Ring3 worker'a gönderir.
 * Worker sonucu execution output window'a yazar.
 */
int execution_slot_dispatch_bcib_locked(exec_slot_t *slot)
{
    proc_t *target_proc;
    int result;

    if (!slot || !slot->in_use) {
        return -1;
    }

    // State: READY → RUNNING
    if (execution_slot_transition_locked(slot, EXEC_SLOT_READY, EXEC_SLOT_RUNNING) != 0) {
        return -1;
    }

    // Target process'i bul
    target_proc = proc_find_by_pid((int)slot->target_context_id);
    if (!target_proc) {
        execution_slot_require_finish_locked(slot, EXEC_SLOT_FAILED, 
                                            "execution_slot_dispatch_bcib_locked.no_proc");
        return -1;
    }

    // BCIB payload'ı target process'e map et
    if (proc_map_execution_payload(target_proc, slot) != 0) {
        execution_slot_require_finish_locked(slot, EXEC_SLOT_FAILED,
                                            "execution_slot_dispatch_bcib_locked.map_failed");
        return -1;
    }

    // Output window'u hazırla
    if (execution_slot_prepare_output_locked(slot) != 0) {
        execution_slot_require_finish_locked(slot, EXEC_SLOT_FAILED,
                                            "execution_slot_dispatch_bcib_locked.output_failed");
        return -1;
    }

    // Output window'u target process'e map et
    if (proc_map_execution_output(target_proc, slot) != 0) {
        execution_slot_require_finish_locked(slot, EXEC_SLOT_FAILED,
                                            "execution_slot_dispatch_bcib_locked.output_map_failed");
        return -1;
    }

    // Ring3 worker'ı uyandır (process scheduling)
    target_proc->active_execution_id = slot->execution_id;
    proc_wake(target_proc);

    return 0;
}
```

**UYARI:** Bu fonksiyon Ring0 mekanizma olarak kalmalı, BCIB yorumlama yapmamalı.


#### 0.3 SYS_V2_SUBMIT_EXECUTION Implementation

**Dosya:** `kernel/sys/syscall_v2.c` (ekleme)

```c
/**
 * @brief Submit BCIB execution to kernel
 * 
 * Syscall: SYS_V2_SUBMIT_EXECUTION
 * Args:
 *   - bcib_graph: BCIB graph buffer (userspace)
 *   - graph_size: BCIB graph size
 *   - target_context_id: Target execution context
 *   - timeout_ms: Execution timeout
 * Returns: execution_id or 0 on error
 */
uint64_t sys_v2_submit_execution(const void *bcib_graph, 
                                  uint64_t graph_size,
                                  uint64_t target_context_id,
                                  uint64_t timeout_ms)
{
    execution_slot_guard_t guard = {0};
    execution_slot_trace_scope_t trace_scope = {0};
    exec_slot_t *slot = NULL;
    proc_t *current_proc = proc_current();
    uint64_t execution_id = 0;

    // Validate parameters
    if (!bcib_graph || graph_size == 0 || graph_size > AYKEN_EXECUTION_PAYLOAD_WINDOW_SIZE) {
        return 0;
    }

    if (target_context_id == 0) {
        target_context_id = current_proc->pid;
    }

    // Enter critical section
    execution_slot_enter_critical(&guard);
    execution_slot_trace_scope_enter(&trace_scope, EXEC_TRACE_ACTOR_SYSCALL);

    // Allocate execution slot
    slot = execution_slot_alloc_locked(current_proc->pid, target_context_id);
    if (!slot) {
        goto cleanup;
    }

    execution_id = slot->execution_id;

    // Store BCIB graph
    if (execution_slot_store_bcib_locked(slot, bcib_graph, graph_size) != 0) {
        execution_slot_release_locked(slot);
        execution_id = 0;
        goto cleanup;
    }

    // Set deadline
    if (timeout_ms > 0) {
        slot->deadline_tick = timer_ticks() + (timeout_ms * TIMER_HZ / 1000);
    }

    // Transition: CREATED → READY
    if (execution_slot_transition_locked(slot, EXEC_SLOT_CREATED, EXEC_SLOT_READY) != 0) {
        execution_slot_release_locked(slot);
        execution_id = 0;
        goto cleanup;
    }

    // Enqueue for execution
    if (execution_slot_enqueue_locked(slot) != 0) {
        execution_slot_require_finish_locked(slot, EXEC_SLOT_FAILED,
                                            "sys_v2_submit_execution.enqueue_failed");
        execution_id = 0;
        goto cleanup;
    }

cleanup:
    execution_slot_trace_scope_exit(&trace_scope);
    execution_slot_exit_critical(&guard);
    return execution_id;
}
```

**UYARI:** Bu syscall Ring0 mekanizma olarak kalmalı, BCIB içeriğini yorumlamamalı.

#### 0.4 SYS_V2_WAIT_RESULT Implementation

**Dosya:** `kernel/sys/syscall_v2.c` (ekleme)

```c
/**
 * @brief Wait for execution result
 * 
 * Syscall: SYS_V2_WAIT_RESULT
 * Args:
 *   - execution_id: Execution ID to wait for
 *   - timeout_ms: Wait timeout (0 = infinite)
 * Returns: 0 on success, -1 on error
 */
int sys_v2_wait_result(uint64_t execution_id, uint64_t timeout_ms)
{
    execution_slot_guard_t guard = {0};
    execution_slot_trace_scope_t trace_scope = {0};
    exec_slot_t *slot = NULL;
    proc_t *current_proc = proc_current();
    uint64_t deadline_tick = 0;
    int result = -1;

    if (execution_id == 0) {
        return -1;
    }

    if (timeout_ms > 0) {
        deadline_tick = timer_ticks() + (timeout_ms * TIMER_HZ / 1000);
    }

    execution_slot_enter_critical(&guard);
    execution_slot_trace_scope_enter(&trace_scope, EXEC_TRACE_ACTOR_SYSCALL);

    // Find execution slot
    slot = execution_slot_find_locked(execution_id);
    if (!slot || slot->owner_pid != current_proc->pid) {
        goto cleanup;
    }

    // Check if already terminal
    if (execution_slot_state_is_terminal(slot->state)) {
        result = 0;
        goto cleanup;
    }

    // Wait for completion
    execution_slot_exit_critical(&guard);
    
    // Block until terminal state or timeout
    result = proc_wait_on(&slot->wait_key, deadline_tick);
    
    execution_slot_enter_critical(&guard);

    // Verify slot still valid
    slot = execution_slot_find_locked(execution_id);
    if (!slot || slot->owner_pid != current_proc->pid) {
        result = -1;
        goto cleanup;
    }

    // Check final state
    if (!execution_slot_state_is_terminal(slot->state)) {
        result = -1;
        goto cleanup;
    }

    result = 0;

cleanup:
    execution_slot_trace_scope_exit(&trace_scope);
    execution_slot_exit_critical(&guard);
    return result;
}
```

**UYARI:** Bu syscall blocking olabilir, timeout mekanizması gerekli.

#### 0.5 Kernel Result Fingerprint Üretimi

**Mevcut:** `kernel/sys/execution_slot.c` içinde `execution_slot_hash_result_frames_locked()` mevcut.

**Gerekli Ekleme:** Result hash'i execution output'a ekle.

```c
/**
 * @brief Finalize execution result with fingerprint
 * 
 * Bu fonksiyon execution result'ı finalize eder ve SHA256 fingerprint üretir.
 */
int execution_slot_finalize_result_locked(exec_slot_t *slot)
{
    uint8_t digest[AYKEN_SHA256_DIGEST_SIZE];
    ayken_execution_result_hash_v1_t *hash_header;

    if (!slot || !slot->in_use) {
        return -1;
    }

    // Validate output
    if (execution_slot_validate_output_locked(slot, NULL) != 0) {
        return -1;
    }

    // Prepare result frames
    if (execution_slot_prepare_result_locked(slot) != 0) {
        return -1;
    }

    // Hash result frames
    if (execution_slot_hash_result_frames_locked(slot, digest) != 0) {
        return -1;
    }

    // Prepare hash frame (already done in prepare_result_locked)
    // Hash frame contains ayken_execution_result_hash_v1_t

    // Transition: RUNNING → COMPLETED
    if (execution_slot_transition_locked(slot, EXEC_SLOT_RUNNING, EXEC_SLOT_COMPLETED) != 0) {
        return -1;
    }

    return 0;
}
```

**UYARI:** Hash hesaplama deterministik olmalı, aynı result → aynı hash.

#### 0.6 Host Runtime vs Kernel Result Karşılaştırma

**Dosya:** `userspace/bcib-runtime/src/determinism_proof.rs`

```rust
//! Determinism Proof - Host runtime vs Kernel result comparison
//!
//! Bu modül, host runtime'da hesaplanan sonuç ile kernel'dan gelen
//! sonucu karşılaştırır ve determinism proof üretir.

use sha2::{Sha256, Digest};

/// Determinism proof result
#[derive(Debug, Clone)]
pub struct DeterminismProof {
    pub execution_id: u64,
    pub host_fingerprint: [u8; 32],
    pub kernel_fingerprint: [u8; 32],
    pub match_result: bool,
    pub timestamp: u64,
}

/// Determinism verifier
pub struct DeterminismVerifier {
    proofs: Vec<DeterminismProof>,
}

impl DeterminismVerifier {
    pub fn new() -> Self {
        Self {
            proofs: Vec::new(),
        }
    }

    /// Verify determinism: host runtime vs kernel
    pub fn verify(&mut self, 
                  execution_id: u64,
                  bcib_graph: &[u8],
                  kernel_result: &[u8]) -> DeterminismProof {
        // 1. Host runtime'da BCIB'i çalıştır
        let host_result = self.execute_in_host_runtime(bcib_graph);
        
        // 2. Host result fingerprint hesapla
        let host_fingerprint = self.compute_fingerprint(&host_result);
        
        // 3. Kernel result fingerprint al
        let kernel_fingerprint = self.extract_kernel_fingerprint(kernel_result);
        
        // 4. Karşılaştır
        let match_result = host_fingerprint == kernel_fingerprint;
        
        let proof = DeterminismProof {
            execution_id,
            host_fingerprint,
            kernel_fingerprint,
            match_result,
            timestamp: self.get_timestamp(),
        };
        
        self.proofs.push(proof.clone());
        proof
    }

    fn execute_in_host_runtime(&self, bcib_graph: &[u8]) -> Vec<u8> {
        // Host runtime'da BCIB'i çalıştır (mevcut BcibExecutionRuntime kullan)
        todo!("Implement host runtime execution")
    }

    fn compute_fingerprint(&self, result: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(result);
        hasher.finalize().into()
    }

    fn extract_kernel_fingerprint(&self, kernel_result: &[u8]) -> [u8; 32] {
        // Kernel result'tan ayken_execution_result_hash_v1_t parse et
        todo!("Implement kernel fingerprint extraction")
    }

    fn get_timestamp(&self) -> u64 {
        // Timestamp al (deterministic değil, sadece logging için)
        0
    }

    /// Get all proofs
    pub fn get_proofs(&self) -> &[DeterminismProof] {
        &self.proofs
    }

    /// Get mismatch count
    pub fn mismatch_count(&self) -> usize {
        self.proofs.iter().filter(|p| !p.match_result).count()
    }
}
```

**UYARI:** Host runtime execution deterministik olmalı, kernel ile aynı sonucu üretmeli.

### Kabul Kriterleri (Phase 0)

- [ ] Ring3 BCIB execution worker implementasyonu tamamlandı
- [ ] `SYS_V2_SUBMIT_EXECUTION` syscall çalışıyor
- [ ] `SYS_V2_WAIT_RESULT` syscall çalışıyor
- [ ] Kernel result fingerprint üretimi çalışıyor
- [ ] Host runtime vs kernel result karşılaştırma çalışıyor
- [ ] Aynı BCIB graph → aynı kernel fingerprint (QEMU altında)
- [ ] Execution closure truth surfaces güncellendi
- [ ] Phase 16 Faz B resmi olarak kapandı

### Test Stratejisi (Phase 0)

1. **Unit Test:** Mock BCIB graph ile execution worker test
2. **Integration Test:** Syscall v2 submit/wait test
3. **QEMU Test:** Gerçek kernel altında determinism test
4. **Proof Test:** 100 farklı BCIB graph ile determinism proof

---

## PHASE 1: Hardware Discovery Omurgası

### Amaç

OS'un donanımı sistematik olarak tanımasını sağlamak.

### Mevcut Durum Analizi

**Mevcut:**
- `kernel/drivers/console/keyboard.c`: Port I/O için `inb()` inline fonksiyonu mevcut
- Keyboard driver IRQ1 kullanıyor (PIC tabanlı)

**Eksik:**
- PCI enumeration yok
- Device registry yok
- Unified device modeli yok
- BAR/IRQ metadata yok

### İmplementasyon Detayları

#### 1.1 PCI Enumeration

**Dosya:** `kernel/bus/pci.h`

```c
/**
 * @file kernel/bus/pci.h
 * @brief PCI Bus Enumeration and Device Discovery
 * 
 * Ring0 mechanism only - no policy decisions.
 */

#ifndef AYKEN_PCI_H
#define AYKEN_PCI_H

#include <stdint.h>

// PCI Configuration Space Ports
#define PCI_CONFIG_ADDRESS  0xCF8
#define PCI_CONFIG_DATA     0xCFC

// PCI Header Type
#define PCI_HEADER_TYPE_DEVICE      0x00
#define PCI_HEADER_TYPE_BRIDGE      0x01
#define PCI_HEADER_TYPE_CARDBUS     0x02

// PCI Class Codes
#define PCI_CLASS_STORAGE           0x01
#define PCI_CLASS_NETWORK           0x02
#define PCI_CLASS_DISPLAY           0x03
#define PCI_CLASS_MULTIMEDIA        0x04
#define PCI_CLASS_BRIDGE            0x06
#define PCI_CLASS_SERIAL            0x0C

// PCI Device Structure
typedef struct pci_device {
    uint8_t bus;
    uint8_t dev;
    uint8_t func;
    uint16_t vendor_id;
    uint16_t device_id;
    uint8_t class_code;
    uint8_t subclass;
    uint8_t prog_if;
    uint8_t header_type;
    uint32_t bar[6];
    uint8_t irq_line;
    uint8_t irq_pin;
    uint32_t capability_token;
    char devfs_name[32];
    void *driver_data;
    struct pci_device *next;
} pci_device_t;

// PCI Functions
void pci_init(void);
uint32_t pci_config_read_dword(uint8_t bus, uint8_t dev, uint8_t func, uint8_t offset);
uint16_t pci_config_read_word(uint8_t bus, uint8_t dev, uint8_t func, uint8_t offset);
uint8_t pci_config_read_byte(uint8_t bus, uint8_t dev, uint8_t func, uint8_t offset);
void pci_config_write_dword(uint8_t bus, uint8_t dev, uint8_t func, uint8_t offset, uint32_t value);
pci_device_t *pci_enumerate_devices(void);
pci_device_t *pci_find_device(uint16_t vendor_id, uint16_t device_id);
pci_device_t *pci_find_class(uint8_t class_code, uint8_t subclass);

#endif // AYKEN_PCI_H
```

**Dosya:** `kernel/bus/pci.c`

```c
/**
 * @file kernel/bus/pci.c
 * @brief PCI Bus Enumeration Implementation
 */

#include "pci.h"
#include "../arch/x86_64/port_io.h"
#include "../drivers/console/fb_console.h"
#include <stddef.h>

static pci_device_t *g_pci_devices = NULL;
static uint32_t g_pci_device_count = 0;

/**
 * @brief Read 32-bit value from PCI configuration space
 */
uint32_t pci_config_read_dword(uint8_t bus, uint8_t dev, uint8_t func, uint8_t offset)
{
    uint32_t address = (1U << 31) |
                       ((uint32_t)bus << 16) |
                       ((uint32_t)dev << 11) |
                       ((uint32_t)func << 8) |
                       (offset & 0xFC);
    
    outl(PCI_CONFIG_ADDRESS, address);
    return inl(PCI_CONFIG_DATA);
}

/**
 * @brief Read 16-bit value from PCI configuration space
 */
uint16_t pci_config_read_word(uint8_t bus, uint8_t dev, uint8_t func, uint8_t offset)
{
    uint32_t dword = pci_config_read_dword(bus, dev, func, offset & 0xFC);
    return (uint16_t)((dword >> ((offset & 2) * 8)) & 0xFFFF);
}

/**
 * @brief Read 8-bit value from PCI configuration space
 */
uint8_t pci_config_read_byte(uint8_t bus, uint8_t dev, uint8_t func, uint8_t offset)
{
    uint32_t dword = pci_config_read_dword(bus, dev, func, offset & 0xFC);
    return (uint8_t)((dword >> ((offset & 3) * 8)) & 0xFF);
}

/**
 * @brief Write 32-bit value to PCI configuration space
 */
void pci_config_write_dword(uint8_t bus, uint8_t dev, uint8_t func, uint8_t offset, uint32_t value)
{
    uint32_t address = (1U << 31) |
                       ((uint32_t)bus << 16) |
                       ((uint32_t)dev << 11) |
                       ((uint32_t)func << 8) |
                       (offset & 0xFC);
    
    outl(PCI_CONFIG_ADDRESS, address);
    outl(PCI_CONFIG_DATA, value);
}

/**
 * @brief Check if PCI device exists
 */
static int pci_device_exists(uint8_t bus, uint8_t dev, uint8_t func)
{
    uint16_t vendor_id = pci_config_read_word(bus, dev, func, 0x00);
    return vendor_id != 0xFFFF;
}

/**
 * @brief Allocate PCI device structure
 */
static pci_device_t *pci_alloc_device(void)
{
    // TODO: Use proper memory allocator
    // For now, use static allocation
    static pci_device_t devices[256];
    static uint32_t device_index = 0;
    
    if (device_index >= 256) {
        return NULL;
    }
    
    return &devices[device_index++];
}

/**
 * @brief Probe PCI device and create device structure
 */
static pci_device_t *pci_probe_device(uint8_t bus, uint8_t dev, uint8_t func)
{
    pci_device_t *device;
    uint32_t i;
    
    if (!pci_device_exists(bus, dev, func)) {
        return NULL;
    }
    
    device = pci_alloc_device();
    if (!device) {
        return NULL;
    }
    
    // Read device info
    device->bus = bus;
    device->dev = dev;
    device->func = func;
    device->vendor_id = pci_config_read_word(bus, dev, func, 0x00);
    device->device_id = pci_config_read_word(bus, dev, func, 0x02);
    device->class_code = pci_config_read_byte(bus, dev, func, 0x0B);
    device->subclass = pci_config_read_byte(bus, dev, func, 0x0A);
    device->prog_if = pci_config_read_byte(bus, dev, func, 0x09);
    device->header_type = pci_config_read_byte(bus, dev, func, 0x0E);
    device->irq_line = pci_config_read_byte(bus, dev, func, 0x3C);
    device->irq_pin = pci_config_read_byte(bus, dev, func, 0x3D);
    
    // Read BARs
    for (i = 0; i < 6; i++) {
        device->bar[i] = pci_config_read_dword(bus, dev, func, 0x10 + (i * 4));
    }
    
    device->capability_token = 0;
    device->driver_data = NULL;
    device->next = NULL;
    
    // Generate devfs name
    // Format: pci_VVVV_DDDD (vendor_device)
    // TODO: Implement proper name generation
    device->devfs_name[0] = '\0';
    
    return device;
}

/**
 * @brief Enumerate all PCI devices
 */
pci_device_t *pci_enumerate_devices(void)
{
    uint8_t bus, dev, func;
    pci_device_t *head = NULL;
    pci_device_t *tail = NULL;
    
    fb_print("[kernel/pci] Enumerating PCI devices...\n");
    
    for (bus = 0; bus < 256; bus++) {
        for (dev = 0; dev < 32; dev++) {
            for (func = 0; func < 8; func++) {
                pci_device_t *device = pci_probe_device(bus, dev, func);
                
                if (!device) {
                    continue;
                }
                
                // Add to linked list
                if (!head) {
                    head = device;
                    tail = device;
                } else {
                    tail->next = device;
                    tail = device;
                }
                
                g_pci_device_count++;
                
                fb_print("[kernel/pci] Found device: ");
                fb_print_int(device->vendor_id);
                fb_print(":");
                fb_print_int(device->device_id);
                fb_print(" class=");
                fb_print_int(device->class_code);
                fb_print("\n");
                
                // If not multifunction, skip other functions
                if (func == 0 && !(device->header_type & 0x80)) {
                    break;
                }
            }
        }
    }
    
    fb_print("[kernel/pci] Found ");
    fb_print_int(g_pci_device_count);
    fb_print(" PCI devices\n");
    
    g_pci_devices = head;
    return head;
}

/**
 * @brief Find PCI device by vendor/device ID
 */
pci_device_t *pci_find_device(uint16_t vendor_id, uint16_t device_id)
{
    pci_device_t *device = g_pci_devices;
    
    while (device) {
        if (device->vendor_id == vendor_id && device->device_id == device_id) {
            return device;
        }
        device = device->next;
    }
    
    return NULL;
}

/**
 * @brief Find PCI device by class code
 */
pci_device_t *pci_find_class(uint8_t class_code, uint8_t subclass)
{
    pci_device_t *device = g_pci_devices;
    
    while (device) {
        if (device->class_code == class_code && device->subclass == subclass) {
            return device;
        }
        device = device->next;
    }
    
    return NULL;
}

/**
 * @brief Initialize PCI subsystem
 */
void pci_init(void)
{
    fb_print("[kernel/pci] Initializing PCI subsystem\n");
    pci_enumerate_devices();
}
```

**UYARI:** PCI enumeration Ring0 mekanizma olarak kalmalı, device policy yapmamalı.


#### 1.2 Port I/O Assembly Support

**Dosya:** `kernel/arch/x86_64/port_io.h`

```c
/**
 * @file kernel/arch/x86_64/port_io.h
 * @brief x86_64 Port I/O Operations
 */

#ifndef AYKEN_PORT_IO_H
#define AYKEN_PORT_IO_H

#include <stdint.h>

// 8-bit I/O
static inline uint8_t inb(uint16_t port) {
    uint8_t ret;
    __asm__ volatile("inb %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

static inline void outb(uint16_t port, uint8_t value) {
    __asm__ volatile("outb %0, %1" : : "a"(value), "Nd"(port));
}

// 16-bit I/O
static inline uint16_t inw(uint16_t port) {
    uint16_t ret;
    __asm__ volatile("inw %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

static inline void outw(uint16_t port, uint16_t value) {
    __asm__ volatile("outw %0, %1" : : "a"(value), "Nd"(port));
}

// 32-bit I/O
static inline uint32_t inl(uint16_t port) {
    uint32_t ret;
    __asm__ volatile("inl %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

static inline void outl(uint16_t port, uint32_t value) {
    __asm__ volatile("outl %0, %1" : : "a"(value), "Nd"(port));
}

#endif // AYKEN_PORT_IO_H
```

**UYARI:** Port I/O inline fonksiyonlar olmalı, performans kritik.

#### 1.3 Unified Device Registry (Rust)

**Dosya:** `userspace/device-runtime/Cargo.toml`

```toml
[package]
name = "device-runtime"
version = "0.1.0"
edition = "2021"

[dependencies]
```

**Dosya:** `userspace/device-runtime/src/lib.rs`

```rust
//! Device Runtime - Ring3 device management
//!
//! Bu modül, Ring0'dan gelen device bilgilerini Ring3'te yönetir.

pub mod device;
pub mod registry;
pub mod driver;
pub mod binder;
pub mod capability;
pub mod class;
pub mod ffi;

pub use device::Device;
pub use registry::DeviceRegistry;
pub use driver::{Driver, DriverOps};
pub use binder::DeviceBinder;
pub use capability::DeviceCapability;
pub use class::DeviceClass;
```

**Dosya:** `userspace/device-runtime/src/device.rs`

```rust
//! Device Model - Unified device representation

use std::fmt;

/// Device identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceId {
    pub bus: u8,
    pub dev: u8,
    pub func: u8,
}

/// Device class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceClass {
    Storage,
    Network,
    Display,
    Multimedia,
    Bridge,
    Serial,
    Input,
    Unknown(u8),
}

impl DeviceClass {
    pub fn from_pci_class(class_code: u8) -> Self {
        match class_code {
            0x01 => DeviceClass::Storage,
            0x02 => DeviceClass::Network,
            0x03 => DeviceClass::Display,
            0x04 => DeviceClass::Multimedia,
            0x06 => DeviceClass::Bridge,
            0x0C => DeviceClass::Serial,
            _ => DeviceClass::Unknown(class_code),
        }
    }
}

/// Device structure
#[derive(Debug, Clone)]
pub struct Device {
    pub id: DeviceId,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class: DeviceClass,
    pub subclass: u8,
    pub prog_if: u8,
    pub bars: [u32; 6],
    pub irq_line: u8,
    pub irq_pin: u8,
    pub devfs_name: String,
    pub capability_token: u32,
}

impl Device {
    /// Create new device
    pub fn new(id: DeviceId, vendor_id: u16, device_id: u16) -> Self {
        Self {
            id,
            vendor_id,
            device_id,
            class: DeviceClass::Unknown(0),
            subclass: 0,
            prog_if: 0,
            bars: [0; 6],
            irq_line: 0,
            irq_pin: 0,
            devfs_name: String::new(),
            capability_token: 0,
        }
    }

    /// Get device name
    pub fn name(&self) -> String {
        if !self.devfs_name.is_empty() {
            self.devfs_name.clone()
        } else {
            format!("pci_{:04x}_{:04x}", self.vendor_id, self.device_id)
        }
    }

    /// Check if device has BAR
    pub fn has_bar(&self, index: usize) -> bool {
        index < 6 && self.bars[index] != 0
    }

    /// Get BAR address
    pub fn bar_address(&self, index: usize) -> Option<u64> {
        if !self.has_bar(index) {
            return None;
        }

        let bar = self.bars[index];
        if bar & 1 == 1 {
            // I/O space
            Some((bar & !0x3) as u64)
        } else {
            // Memory space
            Some((bar & !0xF) as u64)
        }
    }

    /// Check if BAR is MMIO
    pub fn bar_is_mmio(&self, index: usize) -> bool {
        if !self.has_bar(index) {
            return false;
        }
        self.bars[index] & 1 == 0
    }
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Device({:02x}:{:02x}.{} {:04x}:{:04x} {:?})",
            self.id.bus, self.id.dev, self.id.func,
            self.vendor_id, self.device_id,
            self.class
        )
    }
}
```

**Dosya:** `userspace/device-runtime/src/registry.rs`

```rust
//! Device Registry - Device management

use crate::device::{Device, DeviceId, DeviceClass};
use std::collections::HashMap;

/// Device registry
pub struct DeviceRegistry {
    devices: HashMap<DeviceId, Device>,
}

impl DeviceRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }

    /// Register device
    pub fn register(&mut self, device: Device) -> Result<(), String> {
        if self.devices.contains_key(&device.id) {
            return Err(format!("Device {:?} already registered", device.id));
        }

        self.devices.insert(device.id, device);
        Ok(())
    }

    /// Unregister device
    pub fn unregister(&mut self, id: &DeviceId) -> Option<Device> {
        self.devices.remove(id)
    }

    /// Find device by ID
    pub fn find_by_id(&self, id: &DeviceId) -> Option<&Device> {
        self.devices.get(id)
    }

    /// Find device by vendor/device ID
    pub fn find_by_vendor_device(&self, vendor_id: u16, device_id: u16) -> Option<&Device> {
        self.devices.values().find(|d| {
            d.vendor_id == vendor_id && d.device_id == device_id
        })
    }

    /// Find devices by class
    pub fn find_by_class(&self, class: DeviceClass) -> Vec<&Device> {
        self.devices.values().filter(|d| d.class == class).collect()
    }

    /// Get all devices
    pub fn devices(&self) -> Vec<&Device> {
        self.devices.values().collect()
    }

    /// Get device count
    pub fn count(&self) -> usize {
        self.devices.len()
    }

    /// Iterate devices
    pub fn iter(&self) -> impl Iterator<Item = &Device> {
        self.devices.values()
    }
}

impl Default for DeviceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

**UYARI:** Device registry Ring3'te kalmalı, Ring0'dan FFI ile beslenecek.

### Kabul Kriterleri (Phase 1)

- [ ] PCI enumeration çalışıyor (QEMU altında)
- [ ] PCI device'lar listelenebiliyor
- [ ] BAR ve IRQ bilgileri okunabiliyor
- [ ] Unified device modeli oluşturuldu
- [ ] Device registry Ring3'te çalışıyor
- [ ] Device metadata görünür

### Test Stratejisi (Phase 1)

1. **QEMU Test:** PCI device enumeration
2. **Mock Test:** Device registry unit test
3. **Integration Test:** Ring0 → Ring3 device info transfer

---

## PHASE 2: Driver Registry ve Auto-Bind

### Amaç

Bulunan device'ları uygun driver'larla kontrollü biçimde eşleştirmek.

### İmplementasyon Detayları

#### 2.1 Driver Trait System

**Dosya:** `userspace/device-runtime/src/driver.rs`

```rust
//! Driver Trait System

use crate::device::Device;
use std::fmt;

/// Driver operations
pub trait DriverOps: Send + Sync {
    /// Check if driver matches device
    fn matches(&self, device: &Device) -> bool;

    /// Probe device
    fn probe(&self, device: &Device) -> Result<(), String>;

    /// Initialize device
    fn init(&mut self, device: &Device) -> Result<(), String>;

    /// Read from device
    fn read(&self, device: &Device, buffer: &mut [u8]) -> Result<usize, String>;

    /// Write to device
    fn write(&self, device: &Device, buffer: &[u8]) -> Result<usize, String>;

    /// Poll device (optional)
    fn poll(&self, device: &Device) -> Result<bool, String> {
        let _ = device;
        Ok(false)
    }

    /// Handle IRQ (optional)
    fn irq(&self, device: &Device) -> Result<(), String> {
        let _ = device;
        Ok(())
    }

    /// Read event (optional)
    fn read_event(&self, device: &Device) -> Result<Vec<u8>, String> {
        let _ = device;
        Err("read_event not implemented".to_string())
    }

    /// Get driver name
    fn name(&self) -> &str;
}

/// Driver wrapper
pub struct Driver {
    ops: Box<dyn DriverOps>,
}

impl Driver {
    /// Create new driver
    pub fn new(ops: Box<dyn DriverOps>) -> Self {
        Self { ops }
    }

    /// Get driver operations
    pub fn ops(&self) -> &dyn DriverOps {
        &*self.ops
    }

    /// Get mutable driver operations
    pub fn ops_mut(&mut self) -> &mut dyn DriverOps {
        &mut *self.ops
    }
}

impl fmt::Debug for Driver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Driver({})", self.ops.name())
    }
}
```

**UYARI:** Driver trait Ring3'te kalmalı, Ring0 driver'ları FFI ile wrap edilecek.

#### 2.2 Device Binder

**Dosya:** `userspace/device-runtime/src/binder.rs`

```rust
//! Device Binder - Auto-bind devices to drivers

use crate::device::{Device, DeviceId};
use crate::driver::Driver;
use crate::registry::DeviceRegistry;
use std::collections::HashMap;

/// Device binding
#[derive(Debug)]
pub struct DeviceBinding {
    pub device_id: DeviceId,
    pub driver_name: String,
}

/// Device binder
pub struct DeviceBinder {
    drivers: Vec<Driver>,
    bindings: HashMap<DeviceId, usize>, // device_id -> driver_index
}

impl DeviceBinder {
    /// Create new binder
    pub fn new() -> Self {
        Self {
            drivers: Vec::new(),
            bindings: HashMap::new(),
        }
    }

    /// Register driver
    pub fn register_driver(&mut self, driver: Driver) {
        self.drivers.push(driver);
    }

    /// Bind device to driver
    pub fn bind(&mut self, device: &Device) -> Result<DeviceBinding, String> {
        // Check if already bound
        if self.bindings.contains_key(&device.id) {
            return Err(format!("Device {:?} already bound", device.id));
        }

        // Find matching driver
        for (index, driver) in self.drivers.iter().enumerate() {
            if !driver.ops().matches(device) {
                continue;
            }

            // Probe device
            if let Err(e) = driver.ops().probe(device) {
                eprintln!("Driver {} probe failed: {}", driver.ops().name(), e);
                continue;
            }

            // Bind
            self.bindings.insert(device.id, index);

            return Ok(DeviceBinding {
                device_id: device.id,
                driver_name: driver.ops().name().to_string(),
            });
        }

        Err(format!("No driver found for device {:?}", device.id))
    }

    /// Unbind device
    pub fn unbind(&mut self, device_id: &DeviceId) -> Result<(), String> {
        if self.bindings.remove(device_id).is_none() {
            return Err(format!("Device {:?} not bound", device_id));
        }
        Ok(())
    }

    /// Get driver for device
    pub fn get_driver(&self, device_id: &DeviceId) -> Option<&Driver> {
        let index = self.bindings.get(device_id)?;
        self.drivers.get(*index)
    }

    /// Get mutable driver for device
    pub fn get_driver_mut(&mut self, device_id: &DeviceId) -> Option<&mut Driver> {
        let index = *self.bindings.get(device_id)?;
        self.drivers.get_mut(index)
    }

    /// Auto-bind all devices
    pub fn auto_bind_all(&mut self, registry: &DeviceRegistry) -> Vec<DeviceBinding> {
        let mut bindings = Vec::new();

        for device in registry.iter() {
            match self.bind(device) {
                Ok(binding) => {
                    println!("Bound device {:?} to driver {}", device.id, binding.driver_name);
                    bindings.push(binding);
                }
                Err(e) => {
                    eprintln!("Failed to bind device {:?}: {}", device.id, e);
                }
            }
        }

        bindings
    }

    /// Get all bindings
    pub fn bindings(&self) -> Vec<DeviceBinding> {
        self.bindings.iter().map(|(device_id, index)| {
            DeviceBinding {
                device_id: *device_id,
                driver_name: self.drivers[*index].ops().name().to_string(),
            }
        }).collect()
    }
}

impl Default for DeviceBinder {
    fn default() -> Self {
        Self::new()
    }
}
```

**UYARI:** Auto-bind Ring3'te çalışmalı, capability check yapmalı.

#### 2.3 Capability Mapping

**Dosya:** `userspace/device-runtime/src/capability.rs`

```rust
//! Device Capability Management

use crate::device::{Device, DeviceId};
use std::collections::HashMap;

/// Device capability scope
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityScope {
    Read,
    Write,
    Control,
    Admin,
}

/// Device capability
#[derive(Debug, Clone)]
pub struct DeviceCapability {
    pub device_id: DeviceId,
    pub scopes: Vec<CapabilityScope>,
    pub token: u32,
}

/// Capability manager
pub struct CapabilityManager {
    capabilities: HashMap<DeviceId, DeviceCapability>,
    next_token: u32,
}

impl CapabilityManager {
    /// Create new capability manager
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
            next_token: 1,
        }
    }

    /// Grant capability
    pub fn grant(&mut self, device: &Device, scopes: Vec<CapabilityScope>) -> DeviceCapability {
        let token = self.next_token;
        self.next_token += 1;

        let capability = DeviceCapability {
            device_id: device.id,
            scopes,
            token,
        };

        self.capabilities.insert(device.id, capability.clone());
        capability
    }

    /// Revoke capability
    pub fn revoke(&mut self, device_id: &DeviceId) -> Option<DeviceCapability> {
        self.capabilities.remove(device_id)
    }

    /// Check capability
    pub fn check(&self, device_id: &DeviceId, scope: CapabilityScope) -> bool {
        if let Some(capability) = self.capabilities.get(device_id) {
            capability.scopes.contains(&scope)
        } else {
            false
        }
    }

    /// Get capability
    pub fn get(&self, device_id: &DeviceId) -> Option<&DeviceCapability> {
        self.capabilities.get(device_id)
    }
}

impl Default for CapabilityManager {
    fn default() -> Self {
        Self::new()
    }
}
```

**UYARI:** Capability enforcement Ring3'te yapılmalı, Ring0'a güvenilmemeli.

### Kabul Kriterleri (Phase 2)

- [ ] Driver trait system çalışıyor
- [ ] Device binder auto-bind yapabiliyor
- [ ] Capability mapping çalışıyor
- [ ] Driver'lar device'lara otomatik bağlanıyor
- [ ] Çakışmalı bind engelleniyor
- [ ] Probe başarısızlığı sistem stabilitesini bozmuyor

### Test Stratejisi (Phase 2)

1. **Unit Test:** Driver trait mock test
2. **Integration Test:** Auto-bind test
3. **Capability Test:** Capability enforcement test

---

## PHASE 3: İlk Gerçek Driver (PS/2 Keyboard)

### Amaç

IRQ tabanlı, test edilebilir, ABDF InputEvent için doğal kaynak olan ilk gerçek driver'ı implement etmek.

### Mevcut Durum Analizi

**Mevcut:**
- `kernel/drivers/console/keyboard.c`: Stub seviyesinde keyboard driver
- Scancode → ASCII translation mevcut
- Ring buffer mevcut
- IRQ1 handler stub mevcut

**Eksik:**
- Mock hw_ops yok
- Real hw_ops yok
- DevFS publish yok
- Rust wrapper yok
- ABDF entegrasyonu yok

### İmplementasyon Detayları

#### 3.1 Keyboard Driver Rust Wrapper

**Dosya:** `userspace/device-runtime/src/drivers/keyboard.rs`

```rust
//! PS/2 Keyboard Driver (Rust Wrapper)

use crate::device::Device;
use crate::driver::DriverOps;

/// Keyboard event
#[derive(Debug, Clone, Copy)]
pub struct KeyboardEvent {
    pub scancode: u8,
    pub ascii: u8,
    pub timestamp: u64,
    pub flags: u8,
}

/// Keyboard driver
pub struct KeyboardDriver {
    name: String,
    buffer: Vec<KeyboardEvent>,
    max_buffer_size: usize,
}

impl KeyboardDriver {
    /// Create new keyboard driver
    pub fn new() -> Self {
        Self {
            name: "ps2_keyboard".to_string(),
            buffer: Vec::new(),
            max_buffer_size: 256,
        }
    }

    /// Read event from buffer
    pub fn read_event_internal(&mut self) -> Option<KeyboardEvent> {
        if self.buffer.is_empty() {
            None
        } else {
            Some(self.buffer.remove(0))
        }
    }

    /// Add event to buffer
    pub fn add_event(&mut self, event: KeyboardEvent) {
        if self.buffer.len() >= self.max_buffer_size {
            // Drop oldest event
            self.buffer.remove(0);
        }
        self.buffer.push(event);
    }
}

impl DriverOps for KeyboardDriver {
    fn matches(&self, device: &Device) -> bool {
        // Match PS/2 keyboard (vendor 0x0000, device 0x0000 for legacy)
        // Or match by class (Input)
        device.vendor_id == 0x0000 && device.device_id == 0x0000
    }

    fn probe(&self, device: &Device) -> Result<(), String> {
        println!("Probing PS/2 keyboard: {}", device);
        // TODO: Actual probe logic
        Ok(())
    }

    fn init(&mut self, device: &Device) -> Result<(), String> {
        println!("Initializing PS/2 keyboard: {}", device);
        // TODO: Call kernel keyboard_init() via FFI
        Ok(())
    }

    fn read(&self, _device: &Device, buffer: &mut [u8]) -> Result<usize, String> {
        // TODO: Read from kernel keyboard buffer via FFI
        let _ = buffer;
        Ok(0)
    }

    fn write(&self, _device: &Device, _buffer: &[u8]) -> Result<usize, String> {
        Err("Keyboard does not support write".to_string())
    }

    fn read_event(&self, _device: &Device) -> Result<Vec<u8>, String> {
        // TODO: Read keyboard event
        Ok(Vec::new())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl Default for KeyboardDriver {
    fn default() -> Self {
        Self::new()
    }
}
```

**UYARI:** Rust wrapper Ring3'te kalmalı, kernel FFI ile iletişim kurmalı.


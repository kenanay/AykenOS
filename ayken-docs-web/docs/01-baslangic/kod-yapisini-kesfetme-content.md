# Kod Yapısını Keşfetme - İçerik Taslağı

## Özet
- AykenOS dizin yapısı ve organizasyonu
- Kernel kaynak kodunu sistematik inceleme
- ABI tanımları ve syscall interface
- Assembly entry point analizi
- Hands-on kod okuma alıştırmaları

## Ön Koşullar
- Mimari Felsefe sayfasını tamamlamış olmak
- C programlama dili (struct, pointer)
- Temel Assembly bilgisi (önerilen)
- Git ve komut satırı kullanımı

## 1. Dizin Yapısı

```
AykenOS/
├── kernel/              # Ring0 kodu (C + Assembly)
│   ├── arch/x86_64/    # x86_64 mimari-spesifik
│   │   ├── boot.S      # Boot assembly
│   │   ├── context_switch.asm  # Context switching
│   │   ├── gdt_idt.c   # GDT/IDT kurulumu
│   │   └── syscall_entry.asm   # Syscall entry point
│   ├── mm/             # Bellek yönetimi
│   │   ├── phys_mem.c  # Fiziksel bellek allocator
│   │   ├── paging.c    # Virtual memory/paging
│   │   ├── kheap.c     # Kernel heap
│   │   └── user_as.c   # User address space
│   ├── sched/          # Scheduler mekanizması
│   │   ├── sched.c     # Scheduler core
│   │   └── sched_mailbox.c  # Ring3 communication
│   ├── sys/            # Syscall dispatcher
│   │   ├── syscall_v2.c  # Syscall v2 implementation
│   │   └── capability_manager.c  # Capability system
│   ├── proc/           # Process management
│   │   └── proc.c      # Process structures
│   ├── fs/             # Filesystem stubs (minimal)
│   │   ├── vfs.c       # VFS mechanism only
│   │   └── devfs.c     # DevFS mechanism only
│   ├── drivers/        # Device drivers
│   │   ├── console/    # Console driver
│   │   └── serial/     # Serial port driver
│   ├── elf/            # ELF parser
│   │   └── parser.c    # ELF64 loader
│   ├── lib/            # Kernel library
│   │   └── string.c    # String functions
│   ├── include/        # Kernel headers
│   │   ├── ayken_abi.h # ABI single source of truth
│   │   ├── syscall.h   # Syscall definitions
│   │   ├── mm.h        # Memory management
│   │   └── generated/  # Auto-generated headers
│   │       └── ayken_abi.inc  # NASM include
│   ├── kernel.c        # Kernel main (kmain)
│   └── ring3_jump.c    # Ring3 transition
│
├── bootloader/         # UEFI bootloader
│   └── efi/
│       ├── efi_main.c  # UEFI entry point
│       ├── elf_loader.c  # ELF kernel loader
│       └── paging.c    # Initial paging setup
│
├── userspace/          # Ring3 kodu
│   └── libayken/       # Ring3 policy library (C)
│       ├── vfs.c       # VFS policy implementation
│       ├── devfs.c     # DevFS policy implementation
│       └── sched_hint.c  # Scheduler policy hints
│
├── ayken-core/         # AI/data systems (Rust)
│   └── crates/
│       ├── abdf/       # ABDF format
│       └── bcib/       # BCIB execution engine
│
└── ayken/              # Constitutional tool (Rust)
    ├── cli/            # CLI interface
    └── rules/          # Constitutional rules
```

## 2. İlk İnceleme: ABI Tanımı

### Adım 1: ayken_abi.h Dosyasını Açın

**Dosya:** `kernel/include/ayken_abi.h`

Bu dosya, AykenOS'un "single source of truth" ABI tanımıdır.

```c
// kernel/include/ayken_abi.h
#ifndef AYKEN_ABI_H
#define AYKEN_ABI_H

#include <stdint.h>

/*
 * AykenOS ABI v1.0 - Single Source of Truth
 *
 * Ring0/Ring3 execution contract constants shared by C code.
 * NASM include is auto-generated from this header:
 *   kernel/include/generated/ayken_abi.inc
 */
#define AYKEN_ABI_VERSION 0x00010000u

/* cpu_context_t layout offsets (bytes) */
#define CTX_R15      0u
#define CTX_R14      8u
#define CTX_R13      16u
#define CTX_R12      24u
#define CTX_RBX      32u
#define CTX_RBP      40u
#define CTX_RIP      48u
#define CTX_RSP      56u
#define CTX_RFLAGS   64u
#define CTX_CR3      72u
#define CTX_CS       80u
#define CTX_SS       82u
#define CTX_RSP0     88u
#define CTX_SIZE     96u
```

**Püf Noktası 1: Neden Offset'ler?**

Assembly kodunda struct field'lara erişmek için offset'ler kullanılır:

```nasm
; context_switch.asm
; RIP'i context'ten yükle
mov rax, [rdi + CTX_RIP]  ; rdi = context pointer, CTX_RIP = 48
```

C'de aynı işlem:
```c
uint64_t rip = context->rip;  // Compiler offset'i hesaplar
```

**Püf Noktası 2: Context Boyutu**

```c
#define CTX_SIZE     96u  // 96 byte = 12 register × 8 byte
```

96 byte, tam olarak 1.5 cache line (64 byte × 1.5). Bu, context switch'te 2 cache miss demektir.

### Adım 2: Syscall Tanımlarını İnceleyin

**Dosya:** `kernel/sys/syscall_v2.h`

```c
// Syscall numbering
#define SYS_V2_BASE        1000
#define SYS_V2_MAX_INDEX   10
#define SYS_V2_NR          (SYS_V2_MAX_INDEX + 1)  // 11 syscall

// Syscall IDs (internal, 0-10)
#define SYS_V2_MAP_MEMORY        0  // User görür: 1000
#define SYS_V2_UNMAP_MEMORY      1  // User görür: 1001
#define SYS_V2_SWITCH_CONTEXT    2  // User görür: 1002
#define SYS_V2_SUBMIT_EXECUTION  3  // User görür: 1003
#define SYS_V2_WAIT_RESULT       4  // User görür: 1004
#define SYS_V2_INTERRUPT_RETURN  5  // User görür: 1005
#define SYS_V2_TIME_QUERY        6  // User görür: 1006
#define SYS_V2_CAPABILITY_BIND   7  // User görür: 1007
#define SYS_V2_CAPABILITY_REVOKE 8  // User görür: 1008
#define SYS_V2_EXIT              9  // User görür: 1009
#define SYS_V2_DEBUG_PUTCHAR    10  // User görür: 1010
```

**Püf Noktası 3: Dual Numbering**

- **Internal (kernel):** 0-10 (array index için)
- **External (user):** 1000-1010 (ABI stability için)

Kernel'de:
```c
syscall_handler_t handlers[SYS_V2_NR] = {
    [SYS_V2_MAP_MEMORY] = sys_v2_map_memory,  // handlers[0]
    [SYS_V2_UNMAP_MEMORY] = sys_v2_unmap_memory,  // handlers[1]
    // ...
};
```

User'da:
```c
// User syscall wrapper
void* map_memory(void* addr, size_t size, int flags) {
    return (void*)syscall(1000, addr, size, flags);  // 1000 = SYS_V2_BASE + 0
}
```

## 3. Syscall Dispatcher İncelemesi

### Adım 3: syscall_v2.c Dosyasını Açın

**Dosya:** `kernel/sys/syscall_v2.c`

```c
// Syscall dispatcher
uint64_t syscall_v2_dispatch(uint64_t syscall_id, 
                              uint64_t arg1, uint64_t arg2, 
                              uint64_t arg3, uint64_t arg4) {
    // 1. Syscall ID'yi normalize et (1000-1010 → 0-10)
    if (syscall_id < SYS_V2_BASE || syscall_id > SYS_V2_LAST) {
        return -ENOSYS;  // Invalid syscall
    }
    
    uint64_t index = syscall_id - SYS_V2_BASE;
    
    // 2. Handler'ı çağır
    switch (index) {
        case SYS_V2_MAP_MEMORY:
            return (uint64_t)sys_v2_map_memory(
                (void*)arg1, (size_t)arg2, (int)arg3
            );
        
        case SYS_V2_SUBMIT_EXECUTION:
            return sys_v2_submit_execution(
                (void*)arg1, (size_t)arg2
            );
        
        case SYS_V2_TIME_QUERY:
            return sys_v2_time_query();
        
        // ... diğer syscall'lar
        
        default:
            return -ENOSYS;
    }
}
```

**Püf Noktası 4: Switch vs Function Pointer Array**

Mevcut kod switch-case kullanıyor. Alternatif:

```c
// Function pointer array (daha hızlı)
typedef uint64_t (*syscall_handler_t)(uint64_t, uint64_t, uint64_t, uint64_t);

syscall_handler_t handlers[SYS_V2_NR] = {
    [SYS_V2_MAP_MEMORY] = (syscall_handler_t)sys_v2_map_memory,
    [SYS_V2_UNMAP_MEMORY] = (syscall_handler_t)sys_v2_unmap_memory,
    // ...
};

uint64_t syscall_v2_dispatch(uint64_t syscall_id, ...) {
    uint64_t index = syscall_id - SYS_V2_BASE;
    if (index >= SYS_V2_NR) return -ENOSYS;
    
    return handlers[index](arg1, arg2, arg3, arg4);
}
```

**Performance:** Function pointer array ~5-10% daha hızlı (branch prediction yok).

## 4. Assembly Entry Point

### Adım 4: syscall_entry.asm Dosyasını İnceleyin

**Dosya:** `kernel/arch/x86_64/syscall_entry.asm`

```nasm
; Syscall entry point (INT 0x80)
global syscall_entry
extern syscall_v2_dispatch

syscall_entry:
    ; 1. Kernel stack'e geç
    swapgs                  ; GS register'ı kernel GS ile değiştir
    mov [gs:0], rsp         ; User RSP'yi sakla
    mov rsp, [gs:8]         ; Kernel RSP'yi yükle
    
    ; 2. User context'i sakla (callee-saved registers)
    push r15
    push r14
    push r13
    push r12
    push rbx
    push rbp
    
    ; 3. Syscall parametrelerini hazırla
    ; System V AMD64 ABI:
    ; rdi = arg1, rsi = arg2, rdx = arg3, rcx = arg4, r8 = arg5, r9 = arg6
    ; Syscall convention:
    ; rax = syscall_id, rdi = arg1, rsi = arg2, rdx = arg3, r10 = arg4
    
    mov rdi, rax            ; arg1: syscall_id
    mov rsi, rdi            ; arg2: arg1 (user rdi)
    mov rdx, rsi            ; arg3: arg2 (user rsi)
    mov rcx, rdx            ; arg4: arg3 (user rdx)
    mov r8, r10             ; arg5: arg4 (user r10, not rcx!)
    
    ; 4. C dispatcher'ı çağır
    call syscall_v2_dispatch
    
    ; 5. User context'i geri yükle
    pop rbp
    pop rbx
    pop r12
    pop r13
    pop r14
    pop r15
    
    ; 6. User stack'e dön
    mov rsp, [gs:0]         ; User RSP'yi geri yükle
    swapgs                  ; GS'i user GS ile değiştir
    
    ; 7. User mode'a dön
    iretq                   ; Interrupt return (RIP, CS, RFLAGS, RSP, SS pop eder)
```

**Püf Noktası 5: swapgs Instruction**

`swapgs` x86_64'e özgü bir instruction'dır:
- GS register'ını `IA32_KERNEL_GS_BASE` MSR ile değiştirir
- Per-CPU data'ya hızlı erişim sağlar
- Ring0'da kernel GS, Ring3'te user GS

```c
// Per-CPU data structure
struct per_cpu_data {
    uint64_t user_rsp;      // [gs:0]
    uint64_t kernel_rsp;    // [gs:8]
    uint64_t current_task;  // [gs:16]
    // ...
};
```

**Püf Noktası 6: Syscall Convention**

Linux syscall convention:
- `rax` = syscall number
- `rdi`, `rsi`, `rdx`, `r10`, `r8`, `r9` = arguments (NOT rcx!)

Neden `rcx` değil? Çünkü `syscall` instruction `rcx`'i RIP için kullanır.

## 5. Hands-On Alıştırmalar

### Alıştırma 1: Syscall Sayısını Doğrulama

```bash
# Terminal'de
cd kernel/include
grep "SYS_V2_" ayken_abi.h | grep "#define" | wc -l

# Beklenen çıktı: 11
```

### Alıştırma 2: Context Yapısı Boyutu

```c
// test_context_size.c
#include <stdio.h>
#include "kernel/include/ayken_abi.h"

int main() {
    printf("Context size: %u bytes\n", CTX_SIZE);
    printf("Cache lines: %.2f\n", (float)CTX_SIZE / 64.0);
    
    // Register count
    int reg_count = CTX_SIZE / 8;
    printf("Registers: %d\n", reg_count);
    
    return 0;
}
```

**Beklenen Çıktı:**
```
Context size: 96 bytes
Cache lines: 1.50
Registers: 12
```

**Analiz:** 1.5 cache line = 2 cache miss per context switch.

### Alıştırma 3: Syscall Overhead Benchmark

```c
// benchmark_syscall.c
#include <time.h>
#include <stdio.h>

// Minimal syscall (sadece overhead)
static inline uint64_t sys_v2_time_query(void) {
    uint64_t result;
    asm volatile(
        "mov $1006, %%rax\n"  // SYS_V2_TIME_QUERY
        "int $0x80\n"
        "mov %%rax, %0\n"
        : "=r"(result)
        :
        : "rax"
    );
    return result;
}

int main() {
    struct timespec start, end;
    const int iterations = 1000000;
    
    clock_gettime(CLOCK_MONOTONIC, &start);
    
    for (int i = 0; i < iterations; i++) {
        sys_v2_time_query();
    }
    
    clock_gettime(CLOCK_MONOTONIC, &end);
    
    long ns = (end.tv_sec - start.tv_sec) * 1000000000L + 
              (end.tv_nsec - start.tv_nsec);
    
    printf("Total time: %ld ns\n", ns);
    printf("Per syscall: %ld ns\n", ns / iterations);
    printf("Syscalls/sec: %.2f M\n", (float)iterations / (ns / 1000.0));
    
    return 0;
}
```

**Beklenen Sonuç (modern CPU):**
```
Total time: 150000000 ns
Per syscall: 150 ns
Syscalls/sec: 6.67 M
```

### Alıştırma 4: ABI Version Check

```bash
# ABI version'ı kontrol et
grep "AYKEN_ABI_VERSION" kernel/include/ayken_abi.h

# Çıktı: #define AYKEN_ABI_VERSION 0x00010000u
# 0x00010000 = Version 1.0.0 (major.minor.patch)
```

### Alıştırma 5: Syscall Handler Mapping

Her syscall'ın hangi dosyada implement edildiğini bulun:

```bash
# MAP_MEMORY handler'ını bul
grep -r "sys_v2_map_memory" kernel/

# Beklenen: kernel/sys/syscall_v2.c
```

**Görev:** 11 syscall'ın hepsini bulun ve bir tablo oluşturun:

| Syscall ID | Name | File | Line |
|------------|------|------|------|
| 1000 | MAP_MEMORY | syscall_v2.c | 123 |
| 1001 | UNMAP_MEMORY | syscall_v2.c | 145 |
| ... | ... | ... | ... |

## 6. Kod Okuma Stratejisi

### Top-Down Yaklaşım

1. **Başlangıç:** `kernel/kernel.c` → `kmain()`
2. **Initialization:** GDT, IDT, memory, scheduler
3. **Syscall Setup:** `syscall_v2_init()`
4. **Ring3 Jump:** `ring3_jump.c`

### Bottom-Up Yaklaşım

1. **ABI:** `ayken_abi.h` → Temel tanımlar
2. **Syscall:** `syscall_v2.h` → Interface
3. **Implementation:** `syscall_v2.c` → Logic
4. **Assembly:** `syscall_entry.asm` → Entry point

### Feature-Based Yaklaşım

Bir özelliği takip edin:

**Örnek: Bellek Haritalama**
1. User call: `map_memory()` wrapper
2. Syscall entry: `syscall_entry.asm`
3. Dispatcher: `syscall_v2_dispatch()`
4. Handler: `sys_v2_map_memory()`
5. Implementation: `paging.c` → `map_page()`

## 7. Referans Kaynaklar

### Kitaplar

1. **"Intel 64 and IA-32 Architectures Software Developer's Manual"**
   - Volume 3: System Programming Guide
   - Bölüm 5: Interrupt and Exception Handling
   - Bölüm 6: Task Management

2. **"Operating Systems: Three Easy Pieces"** - Remzi Arpaci-Dusseau
   - Bölüm 4: The Abstraction: The Process
   - Bölüm 6: Mechanism: Limited Direct Execution
   - Bölüm 13: The Abstraction: Address Spaces

3. **"Linux Kernel Development"** - Robert Love
   - Bölüm 5: System Calls
   - Bölüm 7: Interrupts and Interrupt Handlers
   - Bölüm 10: Kernel Synchronization Methods

### Online Kaynaklar

1. **OSDev Wiki**
   - [System Calls](https://wiki.osdev.org/System_Calls)
   - [Context Switching](https://wiki.osdev.org/Context_Switching)
   - [GDT Tutorial](https://wiki.osdev.org/GDT_Tutorial)

2. **Intel Documentation**
   - [Intel SDM](https://software.intel.com/content/www/us/en/develop/articles/intel-sdm.html)
   - [System V AMD64 ABI](https://refspecs.linuxbase.org/elf/x86_64-abi-0.99.pdf)

3. **AykenOS Repository**
   - [GitHub: AykenOS](https://github.com/kenanay/AykenOS)
   - [Documentation](https://github.com/kenanay/AykenOS/tree/main/docs)

### Akademik Makaleler

1. **"The UNIX Time-Sharing System"** - Dennis Ritchie & Ken Thompson (1974)
   - Klasik UNIX sistem çağrısı tasarımı

2. **"Exokernel: An Operating System Architecture for Application-Level Resource Management"** - MIT (1995)
   - Minimal kernel yaklaşımı (AykenOS'a benzer)

3. **"Capability-Based Computer Systems"** - Henry M. Levy (1984)
   - Capability-based security modeli

### Video Kaynakları

1. **"Writing an OS in Rust"** - Philipp Oppermann
   - Blog serisi ve video içerik
   - [os.phil-opp.com](https://os.phil-opp.com)

2. **"Operating Systems"** - MIT OpenCourseWare
   - 6.828: Operating System Engineering

### Pratik Araçlar

1. **QEMU Documentation**
   - [QEMU System Emulation](https://www.qemu.org/docs/master/system/index.html)

2. **GDB Debugging**
   - [Debugging with GDB](https://sourceware.org/gdb/current/onlinedocs/gdb/)

3. **objdump ve nm**
   - Binary analiz araçları
   - `man objdump`, `man nm`

## 8. İleri Okuma Önerileri

### Başlangıç Seviyesi
1. Mimari Felsefe sayfasını tekrar okuyun
2. OSDev Wiki'den "Getting Started" bölümünü inceleyin
3. "Operating Systems: Three Easy Pieces" kitabının ilk 6 bölümünü okuyun

### Orta Seviyesi
1. Intel SDM Volume 3'ün ilgili bölümlerini okuyun
2. Linux kernel kaynak kodunu inceleyin (`arch/x86/entry/`)
3. System V AMD64 ABI dokümanını okuyun

### İleri Seviyesi
1. Exokernel makalesini okuyun ve AykenOS ile karşılaştırın
2. Capability-based systems literatürünü araştırın
3. Kendi syscall'ınızı implement edin

## 9. Pratik Projeler

### Proje 1: Syscall Tracer
Kendi `strace` benzeri aracınızı yazın:
- Syscall ID'lerini yakalayın
- Parametreleri decode edin
- Execution time'ı ölçün

### Proje 2: Context Visualizer
Context yapısını görselleştirin:
- Register değerlerini gösterin
- Memory layout'u çizin
- Cache line alignment'ı analiz edin

### Proje 3: Custom Syscall
Yeni bir syscall ekleyin:
- `sys_v2_hello_world()` implement edin
- ABI'yi güncelleyin
- Test programı yazın

## 10. Sonraki Adımlar

- **Boot Sürecini Anlama:** UEFI'den kernel'e geçiş
- **Bellek Yönetimi:** Paging ve virtual memory
- **Context Switching:** CPU state management

---

**Tahmini Süre:** 45-60 dakika  
**Zorluk:** Orta  
**Gerekli Araçlar:** Text editor, terminal, grep

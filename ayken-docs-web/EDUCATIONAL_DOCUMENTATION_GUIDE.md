# AykenOS Eğitimsel Dokümantasyon Rehberi

**Version:** 1.0  
**Date:** 2026-03-03  
**Purpose:** Akademik ve teknik öğrenme yolu

## Felsefe

AykenOS dokümantasyonu sadece "ne" ve "nasıl" değil, "neden" ve "nasıl öğrenilir" sorularına da cevap vermelidir. Her konu:

1. **Teorik Temel** - Akademik arka plan
2. **Pratik Uygulama** - Kod örnekleri ve püf noktaları
3. **Karşılaştırmalı Analiz** - Geleneksel yaklaşımlarla kıyaslama
4. **Derinlemesine İnceleme** - Assembly, bellek düzeni, CPU davranışı
5. **Alıştırmalar** - Hands-on öğrenme

## Öğrenme Yolu: İlk Adım

### Seviye 0: Ön Hazırlık (Gerekli Bilgi)

Projeyi anlamak için gereken temel bilgiler:

#### 0.1 İşletim Sistemi Temelleri
- **Gerekli:** Ring0/Ring3 ayrımı, privilege levels
- **Gerekli:** Virtual memory, paging
- **Gerekli:** Context switching, interrupts
- **Önerilen:** POSIX syscall interface
- **Önerilen:** ELF binary format

#### 0.2 x86_64 Mimarisi
- **Gerekli:** CPU registers (RAX, RBX, RCX, RDX, RSI, RDI, RSP, RBP, RIP)
- **Gerekli:** Segment registers (CS, DS, SS)
- **Gerekli:** Control registers (CR0, CR3, CR4)
- **Gerekli:** GDT (Global Descriptor Table)
- **Gerekli:** IDT (Interrupt Descriptor Table)
- **Önerilen:** TSS (Task State Segment)
- **Önerilen:** SYSCALL/SYSRET vs INT/IRET

#### 0.3 C ve Assembly
- **Gerekli:** C pointer arithmetic
- **Gerekli:** Struct memory layout
- **Gerekli:** NASM syntax
- **Önerilen:** Inline assembly
- **Önerilen:** Calling conventions (System V AMD64 ABI)

### Seviye 1: Projeyi Tanıma (İlk Adım - BURADAN BAŞLA)

#### 1.1 Mimari Felsefe (30 dakika okuma)

**Dosya:** `docs/01-baslangic/mimari-felsefe.html`

**İçerik:**
```markdown
# AykenOS Mimari Felsefesi

## Geleneksel vs AykenOS

### Geleneksel İşletim Sistemleri
- 300+ syscall (Linux: ~400, Windows: ~2000)
- Dosya odaklı (file-centric)
- Kernel'da politika kararları
- Monolitik veya mikrokernel

### AykenOS Yaklaşımı
- 11 syscall (1000-1010)
- Yürütme odaklı (execution-centric)
- Ring3'te politika kararları
- Minimal kernel + empowered userspace

## Neden 11 Syscall?

Geleneksel sistemlerde:
```c
// Linux'ta dosya açma
int fd = open("/path/to/file", O_RDONLY);
read(fd, buffer, size);
close(fd);
```

AykenOS'ta:
```c
// Veri konteynerine erişim
execution_plan_t plan = {
    .operation = OP_DATA_QUERY,
    .target = "users",
    .filter = "age > 18"
};
sys_v2_submit_execution(&plan, sizeof(plan));
result_t result = sys_v2_wait_result();
```

**Fark:** Dosya sistemi operasyonları yerine, veri operasyonları.

## Ring0 vs Ring3 Ayrımı

### Ring0 (Kernel) - Sadece Mekanizma
```c
// Bellek haritalama mekanizması
void* sys_v2_map_memory(void* addr, size_t size, int flags) {
    // Sadece sayfa tablosunu güncelle
    // POLİTİKA KARARI YOK!
    return map_pages(addr, size, flags);
}
```

### Ring3 (Userspace) - Tüm Politika
```c
// VFS politika kararı (Ring3'te)
int vfs_open(const char* path, int flags) {
    // Erişim kontrolü
    if (!check_permission(path, flags)) {
        return -EACCES;
    }
    
    // Dosya sistemi seçimi
    filesystem_t* fs = select_filesystem(path);
    
    // Bellek haritalama (Ring0'a syscall)
    void* mapped = sys_v2_map_memory(...);
    
    return create_file_descriptor(mapped);
}
```

**Önemli:** Kernel sadece "nasıl" yapar, userspace "ne" yapılacağına karar verir.
```

**Alıştırma:**
1. Linux'ta `strace ls` komutunu çalıştırın, kaç syscall kullanıldığını sayın
2. AykenOS'un 11 syscall'ı ile aynı işlevi nasıl yapabileceğinizi düşünün

---

#### 1.2 Kod Yapısını Keşfetme (45 dakika hands-on)

**Dosya:** `docs/01-baslangic/kod-yapisini-kesfetme.html`

**İçerik:**
```markdown
# Kod Yapısını Keşfetme

## Dizin Yapısı

```
AykenOS/
├── kernel/              # Ring0 kodu
│   ├── arch/x86_64/    # Mimari-spesifik kod
│   ├── mm/             # Bellek yönetimi
│   ├── sched/          # Scheduler mekanizması
│   ├── sys/            # Syscall dispatcher
│   └── include/        # Kernel header'ları
├── bootloader/         # UEFI bootloader
├── userspace/          # Ring3 kodu
│   └── libayken/       # VFS, DevFS, Scheduler policy
└── ayken-core/         # AI/data systems (Rust)
```

## İlk İnceleme: Syscall Interface

### Adım 1: ABI Tanımını İnceleyin

**Dosya:** `kernel/include/ayken_abi.h`

```c
// Syscall ID'leri
#define SYS_V2_BASE 1000
#define SYS_V2_MAP_MEMORY       (SYS_V2_BASE + 0)  // 1000
#define SYS_V2_UNMAP_MEMORY     (SYS_V2_BASE + 1)  // 1001
#define SYS_V2_SWITCH_CONTEXT   (SYS_V2_BASE + 2)  // 1002
// ... toplam 11 syscall

// Context yapısı (CPU state)
typedef struct {
    uint64_t rax, rbx, rcx, rdx;
    uint64_t rsi, rdi, rbp, rsp;
    uint64_t r8, r9, r10, r11, r12, r13, r14, r15;
    uint64_t rip, rflags;
    uint64_t cs, ss;
} cpu_context_t;
```

**Püf Noktası:** Context yapısı CPU register'larının 1:1 kopyasıdır. Bu, context switch'i çok hızlı yapar.

### Adım 2: Syscall Dispatcher'ı İnceleyin

**Dosya:** `kernel/sys/syscall_v2.c`

```c
uint64_t syscall_v2_dispatch(uint64_t syscall_id, 
                              uint64_t arg1, uint64_t arg2, 
                              uint64_t arg3, uint64_t arg4) {
    switch (syscall_id) {
        case SYS_V2_MAP_MEMORY:
            return (uint64_t)sys_v2_map_memory(
                (void*)arg1, (size_t)arg2, (int)arg3
            );
        
        case SYS_V2_SUBMIT_EXECUTION:
            return sys_v2_submit_execution(
                (void*)arg1, (size_t)arg2
            );
        
        // ... diğer syscall'lar
        
        default:
            return -ENOSYS;  // Geçersiz syscall
    }
}
```

**Püf Noktası:** Switch-case yerine function pointer array kullanılabilir (daha hızlı).

### Adım 3: Assembly Entry Point'i İnceleyin

**Dosya:** `kernel/arch/x86_64/syscall_entry.asm`

```nasm
; Syscall entry point (INT 0x80)
syscall_entry:
    ; 1. Kernel stack'e geç
    swapgs                  ; GS register'ı kernel GS ile değiştir
    mov [gs:0], rsp         ; User RSP'yi sakla
    mov rsp, [gs:8]         ; Kernel RSP'yi yükle
    
    ; 2. User context'i sakla
    push rax                ; Syscall ID
    push rbx
    push rcx
    push rdx
    push rsi
    push rdi
    ; ... diğer register'lar
    
    ; 3. C dispatcher'ı çağır
    mov rdi, rax            ; arg1: syscall_id
    mov rsi, rbx            ; arg2: arg1
    mov rdx, rcx            ; arg3: arg2
    mov rcx, rdx            ; arg4: arg3
    mov r8, rsi             ; arg5: arg4
    call syscall_v2_dispatch
    
    ; 4. User context'i geri yükle
    pop r15
    pop r14
    ; ... diğer register'lar
    
    ; 5. User mode'a dön
    swapgs
    iretq                   ; Interrupt return
```

**Püf Noktası:** `swapgs` instruction'ı x86_64'e özgüdür ve kernel/user GS register'ını değiştirir. Bu, per-CPU data'ya hızlı erişim sağlar.

## Hands-On Alıştırma

### Alıştırma 1: Syscall Sayısını Doğrulama

```bash
# AykenOS dizininde
cd kernel/include
grep "SYS_V2_" ayken_abi.h | grep "#define" | wc -l
# Çıktı: 11 olmalı
```

### Alıştırma 2: Context Yapısı Boyutu

```c
// test.c
#include <stdio.h>
#include "kernel/include/ayken_abi.h"

int main() {
    printf("Context size: %zu bytes\n", sizeof(cpu_context_t));
    printf("Register count: %zu\n", sizeof(cpu_context_t) / 8);
    return 0;
}
```

**Beklenen Çıktı:**
```
Context size: 160 bytes
Register count: 20
```

**Analiz:** 20 register × 8 byte = 160 byte. Bu, cache line'a (64 byte) sığmaz, bu yüzden context switch 3 cache miss yapar.

### Alıştırma 3: Syscall Overhead Ölçümü

```c
// benchmark.c
#include <time.h>
#include <stdio.h>

// Boş syscall (sadece overhead)
void benchmark_syscall() {
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);
    
    for (int i = 0; i < 1000000; i++) {
        sys_v2_time_query();  // En basit syscall
    }
    
    clock_gettime(CLOCK_MONOTONIC, &end);
    
    long ns = (end.tv_sec - start.tv_sec) * 1000000000L + 
              (end.tv_nsec - start.tv_nsec);
    
    printf("Syscall overhead: %ld ns\n", ns / 1000000);
}
```

**Beklenen Sonuç:** ~100-200 ns per syscall (modern CPU'larda)
```

**Alıştırma:**
1. `ayken_abi.h` dosyasını açın ve tüm syscall'ları listeleyin
2. Her syscall'ın ne yaptığını kendi kelimelerinizle açıklayın
3. Hangi syscall'ların Ring3'te policy kararı gerektirdiğini belirleyin

---

#### 1.3 Boot Sürecini Anlama (60 dakika deep-dive)

**Dosya:** `docs/01-baslangic/boot-surecini-anlama.html`

**İçerik:**
```markdown
# Boot Sürecini Anlama

## Boot Aşamaları

```
UEFI Firmware
    ↓
BOOTX64.EFI (AykenOS Bootloader)
    ↓
kernel.elf (AykenOS Kernel)
    ↓
Ring3 Init Process
```

## Aşama 1: UEFI Bootloader

**Dosya:** `bootloader/efi/efi_main.c`

### UEFI Nedir?

UEFI (Unified Extensible Firmware Interface), modern bilgisayarlarda BIOS'un yerini alan firmware interface'idir.

**Avantajları:**
- 64-bit mode'da çalışır (BIOS 16-bit)
- Dosya sistemi desteği (FAT32)
- Grafik desteği
- Güvenli boot

### Bootloader Görevleri

```c
EFI_STATUS efi_main(EFI_HANDLE ImageHandle, 
                    EFI_SYSTEM_TABLE *SystemTable) {
    // 1. UEFI servislerini başlat
    InitializeLib(ImageHandle, SystemTable);
    
    // 2. Kernel dosyasını yükle
    EFI_FILE_PROTOCOL *kernel_file;
    root->Open(root, &kernel_file, L"\\kernel.elf", 
               EFI_FILE_MODE_READ, 0);
    
    // 3. Kernel'i belleğe oku
    UINTN kernel_size;
    kernel_file->GetInfo(kernel_file, &gEfiFileInfoGuid, 
                         &kernel_size, NULL);
    void *kernel_buffer = AllocatePool(kernel_size);
    kernel_file->Read(kernel_file, &kernel_size, kernel_buffer);
    
    // 4. ELF header'ı parse et
    Elf64_Ehdr *elf_header = (Elf64_Ehdr*)kernel_buffer;
    if (elf_header->e_ident[EI_MAG0] != ELFMAG0) {
        Print(L"Invalid ELF magic!\n");
        return EFI_LOAD_ERROR;
    }
    
    // 5. Program header'ları yükle
    Elf64_Phdr *program_headers = 
        (Elf64_Phdr*)(kernel_buffer + elf_header->e_phoff);
    
    for (int i = 0; i < elf_header->e_phnum; i++) {
        if (program_headers[i].p_type == PT_LOAD) {
            // Segment'i belleğe kopyala
            CopyMem((void*)program_headers[i].p_vaddr,
                    kernel_buffer + program_headers[i].p_offset,
                    program_headers[i].p_filesz);
        }
    }
    
    // 6. Paging'i kur (4-level page tables)
    setup_paging();
    
    // 7. UEFI boot services'i kapat
    SystemTable->BootServices->ExitBootServices(
        ImageHandle, MapKey
    );
    
    // 8. Kernel'e atla
    void (*kernel_entry)(boot_info_t*) = 
        (void(*)(boot_info_t*))elf_header->e_entry;
    
    kernel_entry(&boot_info);
    
    // Buraya asla ulaşılmamalı
    return EFI_SUCCESS;
}
```

### Püf Noktaları

**1. Higher-Half Kernel**

AykenOS kernel'i `0xFFFFFFFF80000000` adresinde çalışır (higher-half):

```
0x0000000000000000 - 0x00007FFFFFFFFFFF: User space (128 TB)
0xFFFF800000000000 - 0xFFFFFFFFFFFFFFFF: Kernel space (128 TB)
```

**Neden?**
- User space ve kernel space ayrımı
- User process'ler kernel'i göremez
- Güvenlik

**2. Identity Mapping**

Bootloader, fiziksel belleği hem düşük hem yüksek adreslere map eder:

```c
// Identity mapping (0x0 → 0x0)
map_page(0x0, 0x0, PAGE_PRESENT | PAGE_WRITE);

// Higher-half mapping (0xFFFFFFFF80000000 → 0x0)
map_page(0xFFFFFFFF80000000, 0x0, PAGE_PRESENT | PAGE_WRITE);
```

**Neden?**
- Bootloader düşük adreslerde çalışır
- Kernel yüksek adreslerde çalışır
- Geçiş sırasında her iki mapping de gerekli

**3. Boot Info Yapısı**

```c
typedef struct {
    uint64_t memory_map_addr;
    uint64_t memory_map_size;
    uint64_t kernel_physical_base;
    uint64_t kernel_virtual_base;
    uint64_t framebuffer_addr;
    uint64_t framebuffer_width;
    uint64_t framebuffer_height;
} boot_info_t;
```

Bu yapı, bootloader'dan kernel'e bilgi aktarır.

## Aşama 2: Kernel Initialization

**Dosya:** `kernel/kernel.c`

```c
void kmain(boot_info_t *boot_info) {
    // 1. Serial port'u başlat (debug için)
    serial_init();
    serial_print("AykenOS booting...\n");
    
    // 2. GDT'yi kur (Global Descriptor Table)
    gdt_init();
    
    // 3. IDT'yi kur (Interrupt Descriptor Table)
    idt_init();
    
    // 4. Fiziksel bellek yöneticisini başlat
    phys_mem_init(boot_info->memory_map_addr, 
                  boot_info->memory_map_size);
    
    // 5. Virtual memory'yi başlat
    paging_init();
    
    // 6. Kernel heap'i başlat
    kheap_init();
    
    // 7. Scheduler'ı başlat
    sched_init();
    
    // 8. Syscall interface'i kur
    syscall_v2_init();
    
    // 9. Timer'ı başlat (100 Hz)
    timer_init(100);
    
    // 10. Interrupt'ları aç
    asm volatile("sti");
    
    // 11. Ring3 init process'i yükle
    load_init_process();
    
    // 12. Idle loop
    while (1) {
        asm volatile("hlt");  // CPU'yu uyut
    }
}
```

### Derinlemesine: GDT Kurulumu

**GDT Nedir?**

Global Descriptor Table, x86_64'te segment descriptor'ları tutar.

```c
typedef struct {
    uint16_t limit_low;
    uint16_t base_low;
    uint8_t  base_mid;
    uint8_t  access;
    uint8_t  granularity;
    uint8_t  base_high;
} __attribute__((packed)) gdt_entry_t;

gdt_entry_t gdt[5] = {
    {0, 0, 0, 0, 0, 0},              // Null descriptor
    {0xFFFF, 0, 0, 0x9A, 0xAF, 0},   // Kernel code (Ring0)
    {0xFFFF, 0, 0, 0x92, 0xAF, 0},   // Kernel data (Ring0)
    {0xFFFF, 0, 0, 0xFA, 0xAF, 0},   // User code (Ring3)
    {0xFFFF, 0, 0, 0xF2, 0xAF, 0},   // User data (Ring3)
};
```

**Access Byte Analizi:**

```
Kernel Code (0x9A):
  1001 1010
  │││└ ┴┴┴┴─ Type: Code, Execute/Read
  ││└─────── Descriptor type: Code/Data
  │└──────── DPL: 00 (Ring 0)
  └───────── Present: 1

User Code (0xFA):
  1111 1010
  │││└ ┴┴┴┴─ Type: Code, Execute/Read
  ││└─────── Descriptor type: Code/Data
  │└──────── DPL: 11 (Ring 3)
  └───────── Present: 1
```

**Püf Noktası:** DPL (Descriptor Privilege Level) 2 bit ile Ring0-Ring3 ayrımını yapar.

## Hands-On Alıştırma

### Alıştırma 1: Boot Log Analizi

```bash
# QEMU ile boot edin ve serial output'u kaydedin
make run > boot.log 2>&1

# Boot aşamalarını analiz edin
grep "AykenOS" boot.log
grep "GDT" boot.log
grep "IDT" boot.log
```

### Alıştırma 2: Memory Map İncelemesi

```c
// boot_info'dan memory map'i yazdır
void print_memory_map(boot_info_t *boot_info) {
    EFI_MEMORY_DESCRIPTOR *desc = 
        (EFI_MEMORY_DESCRIPTOR*)boot_info->memory_map_addr;
    
    for (uint64_t i = 0; i < boot_info->memory_map_size; i++) {
        serial_printf("Region %d: 0x%016lx - 0x%016lx (%s)\n",
                      i,
                      desc->PhysicalStart,
                      desc->PhysicalStart + desc->NumberOfPages * 4096,
                      memory_type_to_string(desc->Type));
        desc = (EFI_MEMORY_DESCRIPTOR*)((uint8_t*)desc + 
                boot_info->memory_map_descriptor_size);
    }
}
```

### Alıştırma 3: GDT Entry Oluşturma

```c
// Kendi GDT entry'nizi oluşturun
gdt_entry_t create_gdt_entry(uint32_t base, uint32_t limit, 
                              uint8_t access, uint8_t flags) {
    gdt_entry_t entry;
    entry.base_low = base & 0xFFFF;
    entry.base_mid = (base >> 16) & 0xFF;
    entry.base_high = (base >> 24) & 0xFF;
    entry.limit_low = limit & 0xFFFF;
    entry.granularity = ((limit >> 16) & 0x0F) | (flags & 0xF0);
    entry.access = access;
    return entry;
}

// Test: Ring2 code segment oluşturun
gdt_entry_t ring2_code = create_gdt_entry(0, 0xFFFFF, 0xBA, 0xAF);
```

**Soru:** Ring2 neden kullanılmıyor? (İpucu: Modern OS'ler sadece Ring0 ve Ring3 kullanır)
```

---

## Sonraki Adımlar

Bu üç bölümü tamamladıktan sonra:

1. **Seviye 2:** Bellek Yönetimi Deep-Dive
2. **Seviye 3:** Context Switching ve Scheduler
3. **Seviye 4:** Syscall Interface Detayları
4. **Seviye 5:** Ring3 VFS/DevFS Implementation
5. **Seviye 6:** BCIB Execution Engine
6. **Seviye 7:** AI Integration

Her seviye:
- 2-3 saat öğrenme süresi
- Teorik + pratik dengesi
- Hands-on alıştırmalar
- Gerçek kod analizi
- Performance ölçümleri

## Öğrenme Kaynakları

### Kitaplar
- "Operating Systems: Three Easy Pieces" (Remzi Arpaci-Dusseau)
- "Intel 64 and IA-32 Architectures Software Developer's Manual"
- "Linux Kernel Development" (Robert Love)

### Online Kaynaklar
- OSDev Wiki: https://wiki.osdev.org
- Intel Manual: https://software.intel.com/content/www/us/en/develop/articles/intel-sdm.html
- AykenOS GitHub: https://github.com/kenanay/AykenOS

### Video Serisi (Önerilecek)
- "Writing an OS in Rust" (Philipp Oppermann)
- "Operating Systems" (MIT OpenCourseWare)

## Değerlendirme

Her seviyenin sonunda:
- Kod analizi görevi
- Pratik uygulama projesi
- Referans kaynaklar listesi

**Örnek Mini Proje (Seviye 1):**
"Kendi syscall'ınızı ekleyin: `sys_v2_hello_world()` - Kernel'den 'Hello from Ring0!' yazdırır."

---

**Sonraki Sayfa:** [Seviye 2: Bellek Yönetimi Deep-Dive](bellek-yonetimi-deep-dive.html)
```

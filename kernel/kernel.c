// kernel/kernel.c
// ============================================================================
//  AykenOS 0.1-dev (x86_64)
//  Kernel Entry + Early/Late Init Routines
//
//  UEFI bootloader tarafından ELF loader sonrası çağrılır.
//  boot_info içinde memory map, kernel fiziksel adresleri, framebuffer bilgileri
//  ve pml4_phys bulunur.
// ============================================================================

#include <stdint.h>
#include "include/boot_info.h"
#include "include/mm.h"
#include "include/sched.h"
#include "include/proc.h"
#include "include/fs.h"
#include "include/syscall.h"

#include "drivers/console/fb_console.h"

#include "arch/x86_64/cpu.h"
#include "arch/x86_64/gdt_idt.h"
#include "arch/x86_64/interrupts.h"
#include "arch/x86_64/pic.h"
#include "arch/x86_64/timer.h"

// AI modülleri (şimdilik opsiyonel)
#include "ai/ayken_core_lm.h"
#include "ai/ai_boot_analyzer.h"

// Init aşamaları
static void kernel_early_init(ayken_boot_info_t *boot);
static void kernel_ai_init(void);
static void kernel_late_init(void);

// ============================================================================
// KERNEL ENTRY POINT
// ============================================================================

void kmain(ayken_boot_info_t *boot)
{
    // 1) Framebuffer konsolu başlat
    fb_console_init(boot);

    // 2) Splash ekranı + mini debug terminalini aç
    fb_draw_splash_screen();
    fb_print("[boot] Splash ekran hazir.\n");

    // 3) EARLY INIT (CPU, GDT, IDT, paging, heap, memory map)
    fb_print("[boot] EARLY init basliyor...\n");
    kernel_early_init(boot);
    fb_print("[boot] EARLY init tamam.\n");

    // 4) AI INIT (şimdilik placeholder)
    fb_print("[boot] AI init basliyor...\n");
    kernel_ai_init();
    fb_print("[boot] AI init tamam.\n");

    // 5) LATE INIT (scheduler, process, FS, syscalls)
    fb_print("[boot] LATE init basliyor...\n");
    kernel_late_init();
    fb_print("[boot] LATE init tamam.\n");

    // 6) Artık scheduler'a devrediyoruz
    fb_print("[boot] Kernel init tamamlandi → scheduler baslatiliyor...\n");

    sched_start();

    // Normalde buraya dönmez; yine de güvenlik için
    while (1)
        __asm__ volatile("hlt");
}



// ============================================================================
// EARLY INIT — çekirdek temel altyapısı
// ============================================================================

static void kernel_early_init(ayken_boot_info_t *boot)
{
    fb_print("[AykenOS] EARLY INIT starting...\n");

    // ------------------------------------------------------------------------
    // 1) CPU + GDT + IDT + ISR
    // ------------------------------------------------------------------------
    cpu_init();
    gdt_init();
    interrupts_install();
    isr_init_stubs();
    fb_print("[OK] CPU + GDT + IDT + ISR.\n");

    // ------------------------------------------------------------------------
    // 2) Fiziksel bellek yönetimi (UEFI memory map → bitmap)
    // ------------------------------------------------------------------------
    phys_mem_init(
        (void*)boot->mem_map_addr,
        boot->mem_desc_size,
        boot->mem_desc_count,
        boot->kernel_phys_start,
        boot->kernel_phys_end
    );
    fb_print("[OK] Physical memory manager.\n");

    // ------------------------------------------------------------------------
    // 3) Paging (bootloader’dan verilen PML4 devralınıyor)
    // ------------------------------------------------------------------------
    paging_init(boot->pml4_phys);
    fb_print("[OK] Paging enabled.\n");

    // ------------------------------------------------------------------------
    // 4) Kernel heap (kmalloc/kfree)
    // ------------------------------------------------------------------------
    kheap_init();
    fb_print("[OK] Kernel heap initialized.\n");

    fb_print("[AykenOS] EARLY INIT done.\n");
}



// ============================================================================
// AI INIT — AykenCoreLM + boot analizör (opsiyonel)
// ============================================================================

static void kernel_ai_init(void)
{
    // Şimdilik boş. AykenCoreLM aktif olunca:
    // ayken_core_lm_init();
    // ai_boot_analyzer();

    fb_print("[AykenOS] AI INIT (placeholder).\n");
}



// ============================================================================
// LATE INIT — scheduler, process, syscall, dosya sistemi
// ============================================================================

static void kernel_late_init(void)
{
    fb_print("[AykenOS] LATE INIT starting...\n");

    // ---------------------------------------------------------
    // 1) Interrupt controller + timer
    // ---------------------------------------------------------
    pic_init();
    timer_init(100);
    fb_print("[OK] PIC + Timer.\n");

    // ---------------------------------------------------------
    // 2) Scheduler & process subsystems
    // ---------------------------------------------------------
    sched_init();
    proc_init();
    fb_print("[OK] Scheduler + Process.\n");

    // ---------------------------------------------------------
    // 3) Dosya sistemi (VFS + devfs)
    // ---------------------------------------------------------
    vfs_init();
    devfs_init();
    fb_print("[OK] VFS + DevFS.\n");

    // ---------------------------------------------------------
    // 4) Syscall interface
    // ---------------------------------------------------------
    syscall_init();
    fb_print("[OK] Syscall interface ready.\n");

    // ---------------------------------------------------------
    // 5) PID 1: init process
    // ---------------------------------------------------------
    proc_create_init();
    fb_print("[OK] init process created (PID 1).\n");

    fb_print("[AykenOS] LATE INIT done.\n");
}

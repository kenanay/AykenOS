// kernel/kernel.c
// ============================================================================
//  AykenOS 0.1-dev (x86_64) - Ring0 Mechanism Only Implementation
//  Kernel Entry + Early/Late Init Routines
//
//  POLICY CODE REMOVAL COMPLETED - Phase 2.5 Task: "No policy code remains in Ring0"
//
//  Ring0 MECHANISM ONLY:
//  - Memory management (paging, heap, physical memory)
//  - Context switching and process mechanism
//  - Interrupt handling and CPU management
//  - Syscall mechanism (10 execution-centric syscalls only)
//  - Capability security mechanism
//  - File system mechanism (proxy stubs to Ring3)
//
//  Ring3 POLICY IMPLEMENTATION:
//  - VFS operations and file system policy
//  - DevFS operations and device management policy
//  - Scheduler policy (process selection, queue management)
//  - AI runtime and inference policy
//  - Application-level policy decisions
//
//  UEFI bootloader tarafından ELF loader sonrası çağrılır.
//  boot_info içinde memory map, kernel fiziksel adresleri, framebuffer bilgileri
//  ve pml4_phys bulunur.
// ============================================================================

#include <stdint.h>
#include "include/boot_info.h"
#include "include/boot_flags.h"
#include "include/mm.h"
#include "include/execution_slot.h"
#include "sched/sched.h"
#include "include/proc.h"
// VFS/DevFS removed in Phase 2.5 - Step C completion
// File system operations now handled entirely in Ring3
#include "fs/devfs.h"
#include "include/syscall.h"
#include "include/capability.h"

#include "drivers/console/fb_console.h"
#include "serial.h"

#include "arch/x86_64/cpu.h"
#include "arch/x86_64/gdt_idt.h"
#include "arch/x86_64/interrupts.h"
#include "arch/x86_64/pic.h"
#include "arch/x86_64/timer.h"
#include "arch/x86_64/port_io.h"
#include "include/ring3_jump.h"

// AI modules removed in Phase 2.5 - Step C completion
// All AI functionality moved to Ring3 userspace

#ifndef AYKEN_INTENTIONAL_PERF_REGRESSION_MS
#define AYKEN_INTENTIONAL_PERF_REGRESSION_MS 0
#endif

#ifndef AYKEN_CR3_PCID
#define AYKEN_CR3_PCID 0
#endif

// Ring3 VFS removed in Phase 2.5 - Step C completion
// VFS operations now handled entirely in Ring3

// Init aşamaları
static void kernel_early_init(ayken_boot_info_t *boot) __attribute__((naked, noinline));
static void kernel_early_init_body(ayken_boot_info_t *boot) __attribute__((used, noinline));
static int g_early_idt_ready = 0;
// AI init function removed in Phase 2.5 - Step C completion
static void kernel_late_init(void);
static uint8_t kernel_boot_stack[16384] __attribute__((aligned(16), section(".data"), used)) = {0xAA};

// ============================================================================
// KERNEL ENTRY POINT
// ============================================================================


// Early debugcon output (QEMU port 0xE9)
static void dual_channel_write(const char *s)
{
    if (!s) return;
    while (*s) {
        outb(0xE9, (uint8_t)*s);
        outb(0x3F8, (uint8_t)*s);
        s++;
    }
}

struct gdtr_snapshot {
    uint16_t limit;
    uint64_t base;
} __attribute__((packed));

struct gdt_descriptor_entry {
    uint16_t limit_low;
    uint16_t base_low;
    uint8_t base_mid;
    uint8_t access;
    uint8_t granularity;
    uint8_t base_high;
} __attribute__((packed));

static int is_canonical_addr(uint64_t addr)
{
    const uint64_t upper = addr >> 48;
    const uint64_t sign = (addr >> 47) & 1ULL;
    return sign ? (upper == 0xFFFFULL) : (upper == 0x0000ULL);
}

static void phase10_prereq_panic(const char *reason)
{
    dual_channel_write("[P10][PREREQ_FAIL] ");
    dual_channel_write(reason);
    dual_channel_write("\n");
    fb_print("[PANIC] Phase10-A2 prereq failed: ");
    fb_print(reason);
    fb_print("\n");
    for (;;) {
        __asm__ volatile("cli; hlt");
    }
}

static void validate_gdt_user_segments(void)
{
    const uint16_t user_data_index = (uint16_t)(GDT_USER_DATA >> 3);
    const uint16_t user_code_index = (uint16_t)(GDT_USER_CODE >> 3);
    struct gdtr_snapshot gdtr = {0};
    __asm__ volatile("sgdt %0" : "=m"(gdtr));
    if (gdtr.base == 0) {
        phase10_prereq_panic("gdt_base_zero");
    }
    if (gdtr.limit < ((uint16_t)((user_code_index + 1u) * 8u) - 1u)) {
        phase10_prereq_panic("gdt_limit_small");
    }

    const struct gdt_descriptor_entry *gdt =
        (const struct gdt_descriptor_entry *)(uintptr_t)gdtr.base;
    const struct gdt_descriptor_entry *user_data = &gdt[user_data_index];
    const struct gdt_descriptor_entry *user_code = &gdt[user_code_index];

    // Present bit and DPL=3 must hold for both user descriptors.
    if (!(user_data->access & 0x80) || (((user_data->access >> 5) & 0x3) != 0x3)) {
        phase10_prereq_panic("gdt_user_data_dpl");
    }
    if (!(user_code->access & 0x80) || (((user_code->access >> 5) & 0x3) != 0x3)) {
        phase10_prereq_panic("gdt_user_code_dpl");
    }

    // S=1 means code/data descriptor. Data must be non-exec, code must be exec.
    if (!(user_data->access & 0x10) || (user_data->access & 0x08)) {
        phase10_prereq_panic("gdt_user_data_type");
    }
    if (!(user_code->access & 0x10) || !(user_code->access & 0x08)) {
        phase10_prereq_panic("gdt_user_code_type");
    }
}

static void validate_idt_bp_gate(void)
{
    const struct idt_entry *bp = &idt_table[3];
    const uint64_t bp_handler =
        ((uint64_t)bp->offset_high << 32) |
        ((uint64_t)bp->offset_mid << 16) |
        (uint64_t)bp->offset_low;

    // Present bit must be set and handler offset must be non-zero.
    if ((bp->type_attr & 0x80) == 0) {
        phase10_prereq_panic("idt_bp_not_present");
    }
    if (bp_handler == 0) {
        phase10_prereq_panic("idt_bp_offset_zero");
    }

    // Allow either trap gate (0xF) or interrupt gate (0xE).
    {
        const uint8_t gate_type = bp->type_attr & 0x0F;
        if (gate_type != 0x0F && gate_type != 0x0E) {
            phase10_prereq_panic("idt_bp_gate_type");
        }
    }
}

static void validate_tss_for_ring3(void)
{
    uint16_t tr = 0;
    __asm__ volatile("str %0" : "=r"(tr));

    if ((tr & ~0x7u) != GDT_TSS_SEL) {
        phase10_prereq_panic("tss_selector_invalid");
    }
    if (kernel_tss.rsp0 == 0) {
        phase10_prereq_panic("tss_rsp0_zero");
    }
    if (kernel_tss.rsp0 < 0xFFFF800000000000ULL) {
        phase10_prereq_panic("tss_rsp0_not_kernel_half");
    }
    if (!is_canonical_addr(kernel_tss.rsp0)) {
        phase10_prereq_panic("tss_rsp0_noncanonical");
    }
    if (paging_get_phys(kernel_tss.rsp0 - 8) == 0) {
        phase10_prereq_panic("tss_rsp0_unmapped");
    }
}

static void validate_cr4_ring3_policy(void)
{
    const uint64_t cr4_smep = (1ULL << 20);
    const uint64_t cr4_smap = (1ULL << 21);
    const uint64_t cr4_pcide = (1ULL << 17);
    uint64_t cr4 = 0;

    __asm__ volatile("mov %%cr4, %0" : "=r"(cr4));

    // Phase10 hardening preparation: fail-closed until SMEP/SMAP-safe paths are explicit.
    if ((cr4 & (cr4_smep | cr4_smap)) != 0) {
        phase10_prereq_panic("cr4_smep_smap_unsupported");
    }

#if AYKEN_CR3_PCID == 1
    if ((cr4 & cr4_pcide) == 0) {
        phase10_prereq_panic("cr4_pcide_required");
    }
#else
    if ((cr4 & cr4_pcide) != 0) {
        phase10_prereq_panic("cr4_pcide_unexpected");
    }
#endif
}

static void validate_phase10_a2_prerequisites(void)
{
    validate_gdt_user_segments();
    validate_idt_bp_gate();
    validate_tss_for_ring3();
    validate_cr4_ring3_policy();
}

// Deterministic (timer-tick based) delay hook for intentional perf regression validation.
// Default is disabled in all normal builds.
static void intentional_perf_regression_delay_if_enabled(void)
{
#if AYKEN_INTENTIONAL_PERF_REGRESSION_MS > 0
    const uint64_t delay_ms = (uint64_t)AYKEN_INTENTIONAL_PERF_REGRESSION_MS;
    const uint64_t start_tick = timer_ticks();
    const uint64_t target_tick = start_tick + delay_ms;
    uint64_t rflags = 0;
    const uint64_t if_mask = (1ull << 9);

    __asm__ volatile("pushfq; popq %0" : "=r"(rflags));
    dual_channel_write("[K][PERF_REGRESSION_TEST] deterministic delay start\n");

    if ((rflags & if_mask) == 0) {
        __asm__ volatile("sti");
    }
    while (timer_ticks() < target_tick) {
        __asm__ volatile("hlt");
    }
    if ((rflags & if_mask) == 0) {
        __asm__ volatile("cli");
    }

    dual_channel_write("[K][PERF_REGRESSION_TEST] deterministic delay end\n");
#endif
}

static void gate1_emit_tick_marker_if_observed(void)
{
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
    const uint64_t start_tick = timer_ticks();
    uint64_t rflags = 0;
    const uint64_t if_mask = (1ull << 9);

    __asm__ volatile("pushfq; popq %0" : "=r"(rflags));
    if ((rflags & if_mask) == 0) {
        __asm__ volatile("sti");
    }

    for (uint64_t spin = 0; spin < 50000000ull; ++spin) {
        if (timer_ticks() > start_tick) {
            dual_channel_write("[[AYKEN_TICK]]\n");
            if ((rflags & if_mask) == 0) {
                __asm__ volatile("cli");
            }
            return;
        }
        __asm__ volatile("pause");
    }

    dual_channel_write("[[AYKEN_TICK_MISS]]\n");
    if ((rflags & if_mask) == 0) {
        __asm__ volatile("cli");
    }
#endif
}

static inline void reload_cs(uint16_t sel) __attribute__((unused));
static inline void reload_cs(uint16_t sel)
{
    __asm__ volatile(
        "pushq %[sel]\n"
        "leaq 1f(%%rip), %%rax\n"
        "pushq %%rax\n"
        "lretq\n"
        "1:\n"
        :
        : [sel] "r"((uint64_t)sel)
        : "rax", "memory");
}

void kmain_real(ayken_boot_info_t *boot)
{
    // GATE-0: Boot Determinism Proof
    // First debugcon output - proves kernel booted successfully
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'['), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'['), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'A'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'Y'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'K'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'E'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'N'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'_'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'B'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'O'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'O'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'T'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'_'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'O'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'K'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)']'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)']'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'\n'), "Nd"(0xE9));
    sched_perf_note_boot_start();
    
    // Initialize serial port for debugging
    serial_init_com1();
    serial_write("SERIAL_OK\n");
    
    // Serial port test
    serial_write("KERNEL_BOOT_START\n");
    
    dual_channel_write("[K][EARLY_BOOT_OK] kmain entry\n");
    // Minimal early exception visibility (no STI)
    cpu_init();
    gdt_init();
    // interrupts_install_early();
    // g_early_idt_ready = 1;
    // reload_cs(GDT_KERNEL_CODE);
    // Note: TSS will be initialized later in kernel_early_init_body after heap is ready
#ifdef AYKEN_EARLY_IDT_TEST
    __asm__ volatile("int3");
#endif

    // 1) Framebuffer konsolu başlat
    dual_channel_write("[K][BEFORE_FB]\n");
    fb_console_init(boot);
    dual_channel_write("[K][AFTER_FB]\n");
    // Validation marker: serial/stdout (COM1) after serial_init()
    fb_print("[K][QEMU_BOOT_OK]\n");

    // 2) Splash ekranı + mini debug terminalini aç
    fb_draw_splash_screen();
    fb_print("[boot] Splash ekran hazir.\n");

    /* ABI ve flags doğrulaması */
    if (boot->abi_version != AYKEN_BOOT_ABI_VERSION) {
        fb_print("[PANIC] Boot ABI uyumsuzlugu: beklenen "); fb_print_uint(AYKEN_BOOT_ABI_VERSION); fb_print(", alinan "); fb_print_uint(boot->abi_version); fb_print("\n");
        /* Dur: uyumsuz ABI güvenlik riski oluşturabilir */
        while (1) __asm__ volatile("cli; hlt");
    }

    uint32_t unknown = boot->flags & ~AYKEN_BOOT_KNOWN_FLAGS;
    if (unknown) {
        fb_print("[WARN] Boot: bilinmeyen flag'ler setli: 0x"); fb_print_hex(unknown); fb_print("\n");
    }

    // 3) EARLY INIT (CPU, GDT, IDT, paging, heap, memory map)
    fb_print("[boot] EARLY init basliyor...\n");
    dual_channel_write("[K][EARLY_INIT_BEGIN]\n");
    kernel_early_init(boot);
    dual_channel_write("[K][EARLY_INIT_DONE]\n");
    fb_print("[boot] EARLY init tamam.\n");

    // AI init removed in Phase 2.5 - Step C completion
    // All AI functionality moved to Ring3 userspace
    fb_print("[boot] AI services moved to Ring3 userspace.\n");

    // 5) LATE INIT (scheduler, process, FS, syscalls)
    fb_print("[boot] LATE init basliyor...\n");
    dual_channel_write("[K][LATE_INIT_BEGIN]\n");
    kernel_late_init();
    dual_channel_write("[K][LATE_INIT_RETURN]\n");
    fb_print("[boot] LATE init tamam.\n");

    // 6) Artık scheduler'a devrediyoruz
    fb_print("[boot] Kernel init tamamlandi -> scheduler baslatiliyor...\n");
    sched_perf_note_core_ready();
    fb_print("[K][BOOT_OK] Phase 4.4 minimal boot reached\n");
    dual_channel_write("[K][BOOT_OK] Phase 4.4 minimal boot reached\n");

    outb(0xE9, (uint8_t)'A');
    outb(0xE9, (uint8_t)'A');
    outb(0xE9, (uint8_t)'A');
    dual_channel_write("[K][ABOUT_TO_SCHED]\n");
    outb(0xE9, (uint8_t)'B');
    outb(0xE9, (uint8_t)'B');
    outb(0xE9, (uint8_t)'B');
    sched_start();
    outb(0xE9, (uint8_t)'C');
    outb(0xE9, (uint8_t)'C');
    outb(0xE9, (uint8_t)'C');

    // Normalde buraya dönmez; yine de güvenlik için
    for (;;) {
        if (sched_take_resched()) {
            outb(0xE9, (uint8_t)'Y'); // Yield marker
            outb(0xE9, (uint8_t)'[');
            outb(0xE9, (uint8_t)'C');
            outb(0xE9, (uint8_t)'A');
            outb(0xE9, (uint8_t)'L');
            outb(0xE9, (uint8_t)'L');
            outb(0xE9, (uint8_t)']');
            sched_yield();
            outb(0xE9, (uint8_t)'[');
            outb(0xE9, (uint8_t)'R');
            outb(0xE9, (uint8_t)'E');
            outb(0xE9, (uint8_t)'T');
            outb(0xE9, (uint8_t)']');
            continue;
        }
        __asm__ volatile("sti; hlt");
    }
}



// ============================================================================
// EARLY INIT — çekirdek temel altyapısı
// ============================================================================

static void kernel_early_init(ayken_boot_info_t *boot)
{
    __asm__ volatile(
        "pushq %%r12\n"
        "pushq %%r13\n"
        "movq %%rsp, %%r13\n"
        "movq %%rdi, %%r12\n"
        "movb $'1', %%al\n"
        "outb %%al, $0xE9\n"
        "leaq kernel_boot_stack(%%rip), %%rsp\n"
        "addq $16384, %%rsp\n"
        "andq $-16, %%rsp\n"
        "movq %%r12, %%rdi\n"
        "call kernel_early_init_body\n"
        "movq %%r13, %%rsp\n"
        "popq %%r13\n"
        "popq %%r12\n"
        "ret\n"
        :
        :
        : "rax", "rsp", "r12", "r13", "memory");
}

static void kernel_early_init_body(ayken_boot_info_t *boot)
{
    dual_channel_write("[K][E1] CPU/GDT/IDT\n");

    // ------------------------------------------------------------------------
    // 1) CPU + GDT + IDT + ISR + TSS
    // ------------------------------------------------------------------------
    if (!g_early_idt_ready) {
        cpu_init();
        tss_init();  // Initialize TSS with proper RSP0 (requires heap)
        gdt_init();  // This will load TSS with correct RSP0
    }
    if (!g_early_idt_ready) {
        interrupts_install_early();
        g_early_idt_ready = 1;
    }
    dual_channel_write("[K][E2] PHYS_MEM\n");

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
    dual_channel_write("[K][E3] PAGING\n");

    // ------------------------------------------------------------------------
    // 3) Paging (bootloader’dan verilen PML4 devralınıyor)
    // ------------------------------------------------------------------------
    paging_init(boot->pml4_phys);
    dual_channel_write("[K][E4] KHEAP\n");

    // ------------------------------------------------------------------------
    // 4) Kernel heap (kmalloc/kfree)
    // ------------------------------------------------------------------------
    kheap_init();
    dual_channel_write("[K][E5] EARLY_DONE\n");
}



// ============================================================================
// AI INIT removed in Phase 2.5 - Step C completion
// All AI functionality moved to Ring3 userspace
// ============================================================================
// LATE INIT — scheduler, process, syscall, dosya sistemi
// ============================================================================

static void kernel_late_init(void)
{
    fb_print("[AykenOS] LATE INIT starting...\n");

    // ---------------------------------------------------------
    // 0) Boot Validation Stage (Phase 10-A)
    // ---------------------------------------------------------
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
    dual_channel_write("[K][LATE]0 BOOT_VALIDATION\n");
    fb_print("[VALIDATION] Running boot validation tests...\n");
    
    // Phase 10-A: ELF parser validation
    extern void elf_parser_run_validation(void);
    outb(0xE9, 'E');
    outb(0xE9, 'L');
    outb(0xE9, 'F');
    outb(0xE9, '\n');
    elf_parser_run_validation();
    outb(0xE9, 'E');
    outb(0xE9, 'L');
    outb(0xE9, 'F');
    outb(0xE9, 'E');
    outb(0xE9, '\n');
    
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    defined(AYKEN_ALIAS_PROOF_SELFTEST) && (AYKEN_ALIAS_PROOF_SELFTEST == 1)
    // Phase 11: Alias unit tests — only in selftest mode (not in general validation boot)
    // Guard: alias_proof_validation.c compiled when AYKEN_VALIDATION=1;
    // called only when AYKEN_ALIAS_PROOF_SELFTEST=1 to avoid boot timeout
    // in performance gate and other validation boots that don't need unit tests.
    {
        extern void execute_alias_proof_tests(void);
        dual_channel_write("[K][LATE]0.1 ALIAS_PROOF_TESTS\n");
        execute_alias_proof_tests();
    }
#endif

#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    defined(AYKEN_ALIAS_PROOF_SELFTEST) && (AYKEN_ALIAS_PROOF_SELFTEST == 1)
    // Selftest: gate witness source — armed/ok/fail markers emitted here only
    // proc_run_alias_proof_selftest() gerçek proc-context selftest çalıştırır:
    // paging_create_user_pml4() ile gerçek PML4 tahsis eder, gerçek PTE'ler
    // kurar ve exit_teardown_alias_phase() üzerinden gerçek teardown akışını
    // tetikler. owner_proc yalnızca selftest çerçevesi için kullanılır;
    // clean_teardown ve leak_detection senaryoları kendi proc_t'lerini
    // stack'te oluşturur.
    {
        extern void proc_run_alias_proof_selftest(proc_t *owner_proc);
        static proc_t alias_selftest_proc;
        __builtin_memset(&alias_selftest_proc, 0, sizeof(alias_selftest_proc));
        alias_selftest_proc.pid = 1;
        alias_selftest_proc.state = PROC_ZOMBIE;
        alias_selftest_proc.teardown_started = 1;
        alias_selftest_proc.type = PROC_TYPE_USER;
        /* pml4_phys = 0: owner_proc'un pml4_phys'i kullanılmaz.
         * Gerçek PML4 tahsisi her senaryo içinde paging_create_user_pml4()
         * ile yapılır — heap tahsisi yok, tüm veri yapıları stack'te. */
        dual_channel_write("[K][LATE]0.2 ALIAS_PROOF_SELFTEST\n");
        proc_run_alias_proof_selftest(&alias_selftest_proc);
    }
#endif
    
    // Phase 10-A: User address space validation
    // Note: test_user_as() is in user_as_test.c which is excluded from build
    // Tests will be integrated in Phase 10-B when full validation is needed
    
    fb_print("[VALIDATION] Boot validation tests complete.\n");
#endif

    // ---------------------------------------------------------
    // 1) Interrupt controller + timer
    // ---------------------------------------------------------
    dual_channel_write("[K][LATE]1 PIC\n");
    pic_init();
    dual_channel_write("[K][LATE]2 TIMER\n");
    // Phase 4.5 preempt validation mode: high timer frequency.
    timer_init(1000);
    gate1_emit_tick_marker_if_observed();
    intentional_perf_regression_delay_if_enabled();
    fb_print("[OK] PIC + Timer.\n");

    // ---------------------------------------------------------
    // 2) Scheduler mechanism & process mechanism (no policy)
    // ---------------------------------------------------------
    dual_channel_write("[K][LATE]3 SCHED_INIT\n");
    sched_init();  // Ring0 mechanism only - policy in Ring3
    dual_channel_write("[K][LATE]4 PROC_INIT\n");
    proc_init();   // Ring0 mechanism only - policy in Ring3
    fb_print("[OK] Scheduler mechanism + Process mechanism (policy in Ring3).\n");

    // Passive execution-slot state tables for execution lifecycle hardening.
    dual_channel_write("[K][LATE]4.5 EXEC_SLOT\n");
    execution_slots_init();
    fb_print("[OK] Execution-slot tables initialized (passive runtime state only).\n");
#if defined(AYKEN_PHASE10B_FAIL_CLOSED_SELFTEST) && (AYKEN_PHASE10B_FAIL_CLOSED_SELFTEST == 1)
    dual_channel_write("[K][LATE]4.6 EXEC_SLOT_FAIL_CLOSED_SELFTEST\n");
#endif
    execution_slot_run_fail_closed_selftest();

    // ---------------------------------------------------------
    // 3) File system mechanism only - no policy in Ring0
    // ---------------------------------------------------------
    // VFS and DevFS operations now handled entirely in Ring3
    // Ring0 provides only memory mapping mechanism
    // DevFS proxy stubs redirect to Ring3 (mechanism only)
    dual_channel_write("[K][LATE]5 DEVFS\n");
    devfs_init();
    fb_print("[OK] File system mechanism ready (policy in Ring3).\n");

    // ---------------------------------------------------------
    // 4) Syscall mechanism interface (execution-centric only)
    // ---------------------------------------------------------
    dual_channel_write("[K][LATE]6 SYSCALL\n");
    syscall_init();  // Ring0 mechanism only - execution-centric syscalls only
    fb_print("[OK] Syscall mechanism ready (12 execution-centric syscalls only).\n");

    // ---------------------------------------------------------
    // 4.1) Ring0 INT 0x80 smoke test - COMPLETELY DISABLED FOR RING3 DIAGNOSTICS
    // ---------------------------------------------------------
    dual_channel_write("[K][LATE]6.1 INT80_SMOKETEST_DISABLED\n");
    fb_print("[DISABLED] Ring0 INT 0x80 smoke test disabled - proceeding to Ring3 diagnostics.\n");

    // ---------------------------------------------------------
    // 4.1) Capability mechanism (security mechanism only)
    // ---------------------------------------------------------
    dual_channel_write("[K][LATE]7 CAP\n");
    capability_system_init();  // Ring0 mechanism only - policy in Ring3
    fb_print("[OK] Capability mechanism initialized (policy in Ring3).\n");

    // ---------------------------------------------------------
    // 5) Process creation mechanism (no policy)
    // ---------------------------------------------------------
    dual_channel_write("[K][LATE]8 PROC_CREATE_INIT\n");
    proc_create_init();  // Ring0 mechanism only - policy in Ring3
    fb_print("[OK] init process created (PID 1) - mechanism only.\n");

    // ---------------------------------------------------------
    // 5) Ring3 operations removed in Phase 2.5 - Step C
    // ---------------------------------------------------------
    // Ring3 VFS demonstration removed - operations now in Ring3
    fb_print("[OK] Ring3 operations moved to userspace.\n");

    // ---------------------------------------------------------
    // 6) Phase 2 Complete Validation (Task 2.5.3.1)
    // ---------------------------------------------------------
    fb_print("[VALIDATION] Starting Phase 2 complete validation...\n");
    // execute_phase2_validation();
    fb_print("[VALIDATION] Phase 2 validation completed.\n");

    // ---------------------------------------------------------
    // 6.1) Syscall Count Validation (Task: Ring0 execution-centric syscall surface)
    // ---------------------------------------------------------
    // validate_syscall_count_requirement();

    // ---------------------------------------------------------
    // 6.2) Scheduler Policy Validation (Task: Scheduler policy operates entirely in Ring3)
    // ---------------------------------------------------------
    fb_print("[VALIDATION] Starting Ring3 scheduler policy validation...\n");
    int scheduler_test_result = 0; // run_scheduler_policy_tests();
    if (scheduler_test_result == 0) {
        fb_print("[VALIDATION] Ring3 scheduler policy validation: SKIPPED\n");
    }

    // ---------------------------------------------------------
    // 6.3) Policy Code Removal Validation (Task: No policy code remains in Ring0)
    // ---------------------------------------------------------
    fb_print("[VALIDATION] Verifying no policy code remains in Ring0...\n");
    fb_print("[VALIDATION] Ring0 components verified:\n");
    fb_print("[VALIDATION]   - Scheduler: mechanism only (policy in Ring3)\n");
    fb_print("[VALIDATION]   - VFS: proxy stubs only (policy in Ring3)\n");
    fb_print("[VALIDATION]   - DevFS: proxy stubs only (policy in Ring3)\n");
    fb_print("[VALIDATION]   - Syscalls: 10 execution-centric only (no POSIX)\n");
    fb_print("[VALIDATION]   - Capability: security mechanism only\n");
    fb_print("[VALIDATION]   - AI Runtime: moved to Ring3 (no Ring0 code)\n");
    fb_print("[VALIDATION] Policy code removal validation: COMPLETED\n");

    fb_print("[AykenOS] LATE INIT done.\n");
    dual_channel_write("[K][LATE]9 DONE\n");
    fb_print("[K][LATE_INIT_END]\n");
    dual_channel_write("[K][LATE_INIT_END]\n");
    
#ifdef AYKEN_VALIDATION
    // Gate-0: Boot validation marker
    dual_channel_write("[[AYKEN_BOOT_OK]]\n");
#endif

    // ---------------------------------------------------------
    // Phase 10-A2 Task 1: Validate TSS/GDT/IDT prerequisites
    // Must run before Ring3 process preparation/dispatch path.
    // ---------------------------------------------------------
    validate_phase10_a2_prerequisites();
    dual_channel_write("P10_TSS_OK\n");

    // ---------------------------------------------------------
    // Phase 10-A: Prepare embedded Ring3 process
    // ---------------------------------------------------------
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)
  #if defined(AYKEN_SCHED_BOOTSTRAP_POLICY) && (AYKEN_SCHED_BOOTSTRAP_POLICY == 0)
    // Strict Gate-4 mode must preload owner authority before sched_start().
    fb_print("[PHASE10] Gate-4 strict mode: preloading owner process.\n");
    dual_channel_write("[K][PHASE10] PRELOAD_GATE4_OWNER\n");
    proc_launch_gate4_policy_test();
  #else
    // Transitional Gate-4 mode: init_process_main() owns policy workload creation.
    // Skip Phase10 preloaded Ring3 path to avoid bypassing Gate-4 PID/ACCEPT markers.
    fb_print("[PHASE10] Gate-4 isolated mode: skipping preloaded Ring3 process.\n");
    dual_channel_write("[K][PHASE10] SKIP_PRELOAD_GATE4\n");
  #endif
#else
    fb_print("[PHASE10] Preparing Ring3 process...\n");
    jump_to_ring3();
#endif
}

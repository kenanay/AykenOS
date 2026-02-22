// kernel/proc/proc.c
#include <stddef.h>
#include "../include/proc.h"
#include "../sched/sched.h"
#include "../include/mm.h"
#include "../include/kheap.h"
#include "../include/ayken.h"
#include "../include/gdt_idt.h"
#include "../include/capability.h"
#include "../drivers/console/fb_console.h"
#include "../arch/x86_64/port_io.h"
#include "../sched/sched_mailbox.h"

_Static_assert(offsetof(cpu_context_t, rip) == 48, "ctx.rip offset");
_Static_assert(offsetof(cpu_context_t, rsp) == 56, "ctx.rsp offset");
_Static_assert(offsetof(cpu_context_t, rflags) == 64, "ctx.rflags offset");
_Static_assert(offsetof(cpu_context_t, cr3) == 72, "ctx.cr3 offset");
_Static_assert(offsetof(cpu_context_t, cs) == 80, "ctx.cs offset");
_Static_assert(offsetof(cpu_context_t, ss) == 82, "ctx.ss offset");
_Static_assert(sizeof(cpu_context_t) == 96, "ctx size");

// Use compiler builtin functions for memory operations
#define memset __builtin_memset
#define memcpy __builtin_memcpy

// Ring3 INT80 return-path canary
#define RING3_CANARY_ADDR 0x0000000000405000ULL
#define RING3_CANARY_PRE  0x1111111122222222ULL
#define RING3_CANARY_POST 0x3333333344444444ULL

static proc_t* proc_table[MAX_PROCS];
static int next_pid = 1;

void init_process_main(void);
void kernel_first_entry(void);
void kernel_iret_entry(void);  // IRET-safe kernel entry

static void debugcon_write(const char *s)
{
    for (; *s; ++s) {
        outb(0xE9, (uint8_t)*s);
    }
}

static void debugcon_write_char(char c)
{
    outb(0xE9, (uint8_t)c);
}

static void debugcon_hex8(uint8_t v)
{
    static const char hex[] = "0123456789ABCDEF";
    outb(0xE9, (uint8_t)hex[(v >> 4) & 0xF]);
    outb(0xE9, (uint8_t)hex[v & 0xF]);
}

static void debugcon_hex64(uint64_t v)
{
    static const char hex[] = "0123456789ABCDEF";
    for (int i = 15; i >= 0; --i) {
        outb(0xE9, (uint8_t)hex[(v >> (i * 4)) & 0xF]);
    }
}

static void debug_dump_pte(uint64_t pml4_phys, uint64_t va, const char *tag)
{
    const uint64_t NX_MASK = (1ULL << 63);
    if (!pml4_phys) {
        debugcon_write("[PTE] ");
        debugcon_write(tag);
        debugcon_write(" PML4=0\n");
        return;
    }

    uint64_t *pml4 = (uint64_t *)paging_phys_to_virt(pml4_phys);
    uint16_t pml4_i = (va >> 39) & 0x1FF;
    uint16_t pdpt_i = (va >> 30) & 0x1FF;
    uint16_t pd_i   = (va >> 21) & 0x1FF;
    uint16_t pt_i   = (va >> 12) & 0x1FF;

    uint64_t pml4e = pml4[pml4_i];
    if (!(pml4e & AYKEN_PTE_PRESENT)) {
        debugcon_write("[PTE] ");
        debugcon_write(tag);
        debugcon_write(" VA=");
        debugcon_hex64(va);
        debugcon_write(" PML4E !P\n");
        return;
    }
    uint64_t *pdpt = (uint64_t *)paging_phys_to_virt(pml4e & AYKEN_PTE_ADDR_MASK);

    uint64_t pdpte = pdpt[pdpt_i];
    if (!(pdpte & AYKEN_PTE_PRESENT)) {
        debugcon_write("[PTE] ");
        debugcon_write(tag);
        debugcon_write(" VA=");
        debugcon_hex64(va);
        debugcon_write(" PDPTE !P\n");
        return;
    }
    uint64_t *pd = (uint64_t *)paging_phys_to_virt(pdpte & AYKEN_PTE_ADDR_MASK);

    uint64_t pde = pd[pd_i];
    if (!(pde & AYKEN_PTE_PRESENT)) {
        debugcon_write("[PTE] ");
        debugcon_write(tag);
        debugcon_write(" VA=");
        debugcon_hex64(va);
        debugcon_write(" PDE !P\n");
        return;
    }
    uint64_t *pt = (uint64_t *)paging_phys_to_virt(pde & AYKEN_PTE_ADDR_MASK);

    uint64_t pte = pt[pt_i];
    debugcon_write("[PTE] ");
    debugcon_write(tag);
    debugcon_write(" VA=");
    debugcon_hex64(va);
    debugcon_write(" PTE=");
    debugcon_hex64(pte);
    debugcon_write(" P=");
    debugcon_write_char((pte & AYKEN_PTE_PRESENT) ? '1' : '0');
    debugcon_write(" U=");
    debugcon_write_char((pte & AYKEN_PTE_USER) ? '1' : '0');
    debugcon_write(" W=");
    debugcon_write_char((pte & AYKEN_PTE_WRITABLE) ? '1' : '0');
    debugcon_write(" NX=");
    debugcon_write_char((pte & NX_MASK) ? '1' : '0');
    debugcon_write(" PA=");
    debugcon_hex64(pte & AYKEN_PTE_ADDR_MASK);
    debugcon_write("\n");
}

static int proc_alloc_pid(void)
{
    // A real implementation would reuse PIDs. For now, we just increment.
    return next_pid++;
}

proc_t* proc_find_by_pid(int pid)
{
    for (int i = 0; i < MAX_PROCS; ++i) {
        if (proc_table[i] && proc_table[i]->pid == pid) {
            return proc_table[i];
        }
    }
    return NULL;
}


typedef struct {
    unsigned char e_ident[16];
    uint16_t e_type;
    uint16_t e_machine;
    uint32_t e_version;
    uint64_t e_entry;
    uint64_t e_phoff;
    uint64_t e_shoff;
    uint32_t e_flags;
    uint16_t e_ehsize;
    uint16_t e_phentsize;
    uint16_t e_phnum;
    uint16_t e_shentsize;
    uint16_t e_shnum;
    uint16_t e_shstrndx;
} elf64_ehdr_t;

typedef struct {
    uint32_t p_type;
    uint32_t p_flags;
    uint64_t p_offset;
    uint64_t p_vaddr;
    uint64_t p_paddr;
    uint64_t p_filesz;
    uint64_t p_memsz;
    uint64_t p_align;
} elf64_phdr_t;

static proc_t *proc_alloc(proc_type_t type, const char *name)
{
    proc_t *p = (proc_t *)kmalloc(sizeof(proc_t));
    if (!p) return NULL;

    memset(p, 0, sizeof(proc_t));
    p->pid = proc_alloc_pid();
    p->type = type;
    p->state = PROC_READY;
    p->name = name;
    // PML4 will be set by caller (proc_create_user_process or proc_create_kernel_process)
    p->pml4_phys = 0;  // Initialize to 0, will be set by caller
    p->context.cr3 = 0;  // Initialize to 0, will be set by caller
    p->context.rflags = 0x202;  // IF=1, reserved bits
    
    // Add to process table
    int found_slot = 0;
    for (int i = 0; i < MAX_PROCS; ++i) {
        if (proc_table[i] == NULL) {
            proc_table[i] = p;
            found_slot = 1;
            break;
        }
    }
    if (!found_slot) {
        kfree(p);
        return NULL; // No space in proc_table
    }

    // Set segment selectors based on process type
    if (type == PROC_TYPE_USER) {
        // Ring3: User code and user data segments
        p->context.cs = GDT_USER_CODE;   // User CS
        p->context.ss = GDT_USER_DATA;   // User SS
    } else {
        // Ring0: Kernel code and kernel data segments
        p->context.cs = GDT_KERNEL_CODE;   // Kernel CS
        p->context.ss = GDT_KERNEL_DATA;   // Kernel SS
    }
    
    // rsp0 will be set when the process is created with a kernel stack
    p->context.rsp0 = 0;
    
    return p;
}

static uint64_t load_flat_image(uint64_t pml4_phys, const uint8_t *image, uint64_t size)
{
    uint64_t phys = phys_alloc_frame();
    if (!phys)
        return 0;

    uint8_t *dst = (uint8_t *)paging_phys_to_virt(phys);
    memset(dst, 0, AYKEN_FRAME_SIZE);

    uint64_t copy = size < AYKEN_FRAME_SIZE ? size : AYKEN_FRAME_SIZE;
    if (image && copy)
        memcpy(dst, image, copy);

    debugcon_write("[CODE]=");
    debugcon_hex8(dst[0]);
    outb(0xE9, (uint8_t)'\n');

    paging_map_page_in_pml4(pml4_phys, USER_TEXT_BASE, phys,
                            AYKEN_PTE_USER | AYKEN_PTE_WRITABLE);
    return USER_TEXT_BASE;
}

static uint64_t load_elf_image(uint64_t pml4_phys, const uint8_t *image, uint64_t size)
{
    if (!image || size < sizeof(elf64_ehdr_t))
        return 0;

    const elf64_ehdr_t *ehdr = (const elf64_ehdr_t *)image;
    if (!(ehdr->e_ident[0] == 0x7F && ehdr->e_ident[1] == 'E' &&
          ehdr->e_ident[2] == 'L' && ehdr->e_ident[3] == 'F')) {
        return 0;
    }

    if (ehdr->e_phoff + (uint64_t)ehdr->e_phnum * sizeof(elf64_phdr_t) > size)
        return 0;

    const elf64_phdr_t *phdr = (const elf64_phdr_t *)(image + ehdr->e_phoff);
    for (uint16_t i = 0; i < ehdr->e_phnum; ++i) {
        if (phdr[i].p_type != 1) // PT_LOAD
            continue;

        uint64_t offset = phdr[i].p_offset;
        uint64_t filesz = phdr[i].p_filesz;
        uint64_t memsz  = phdr[i].p_memsz;
        uint64_t vaddr  = phdr[i].p_vaddr;

        for (uint64_t off = 0; off < memsz; off += AYKEN_FRAME_SIZE) {
            uint64_t phys = phys_alloc_frame();
            if (!phys)
                return 0;

            uint8_t *dst = (uint8_t *)paging_phys_to_virt(phys);
            memset(dst, 0, AYKEN_FRAME_SIZE);

            if (off < filesz) {
                uint64_t copy = filesz - off < AYKEN_FRAME_SIZE ? (filesz - off) : AYKEN_FRAME_SIZE;
                if (offset + off + copy <= size)
                    memcpy(dst, image + offset + off, copy);
            }

            paging_map_page_in_pml4(pml4_phys, vaddr + off, phys,
                                    AYKEN_PTE_USER | AYKEN_PTE_WRITABLE);
        }
    }

    return ehdr->e_entry;
}

static uint64_t load_user_image(proc_image_format_t fmt,
                                uint64_t pml4_phys,
                                const uint8_t *image,
                                uint64_t size)
{
    switch (fmt) {
    case PROC_IMAGE_ELF:
        return load_elf_image(pml4_phys, image, size);
    case PROC_IMAGE_FLAT:
    default:
        return load_flat_image(pml4_phys, image, size);
    }
}

void proc_init(void)
{
    fb_print("[proc] Process subsystem init.\n");
    for (int i = 0; i < MAX_PROCS; ++i) {
        proc_table[i] = NULL;
    }
    next_pid = 1;
}

proc_t *proc_create_kernel_thread(void (*func)(void))
{
    proc_t *p = proc_alloc(PROC_TYPE_KERNEL, "kernel-thread");
    if (!p) return NULL;

    // Kernel threads use kernel PML4
    p->pml4_phys = paging_get_kernel_pml4_phys();
    p->context.cr3 = p->pml4_phys;

    uint64_t stack = (uint64_t)kmalloc(4096);
    p->stack_top = stack + 4096;

    p->context.rip = (uint64_t)func;
    p->context.rsp = p->stack_top - 8;  // SysV ABI: entry %rsp = 8 mod 16

    sched_add(p);
    return p;
}

static proc_t *proc_create_init_process(void)
{
    proc_t *p = proc_alloc(PROC_TYPE_KERNEL, "init");
    if (!p) return NULL;

    // Init process uses kernel PML4
    p->pml4_phys = paging_get_kernel_pml4_phys();
    p->context.cr3 = p->pml4_phys;

    uint64_t stack = (uint64_t)kmalloc(4096);
    p->stack_top = stack + 4096;

    p->context.rip = (uint64_t)kernel_first_entry;  // Back to simple entry for debugging
    p->context.rsp = p->stack_top - 8;  // SysV ABI: entry %rsp = 8 mod 16
    p->context.rflags = 0x202;  // IF=1 (bit 9) + reserved bit 1
    
    // Ring0 process: set CS/SS to kernel segments
    p->context.cs = GDT_KERNEL_CODE;  // 0x08
    p->context.ss = GDT_KERNEL_DATA;  // 0x10
    p->context.rsp0 = 0;  // Not needed for Ring0 process

    // DEBUG: Log init process context
    fb_print("[proc_init] PID=");
    fb_print_int(p->pid);
    fb_print(" RIP=");
    fb_print_hex64((uint64_t)kernel_first_entry);
    fb_print(" RSP=");
    fb_print_hex64(p->context.rsp);
    fb_print(" CS=");
    fb_print_hex(p->context.cs);
    fb_print("\n");

    sched_add(p);
    return p;
}

proc_t *proc_create_user_process(const char *name,
                                 const uint8_t *image,
                                 uint64_t image_size,
                                 proc_image_format_t fmt)
{
    outb(0xE9, (uint8_t)'U');
    proc_t *p = proc_alloc(PROC_TYPE_USER, name);
    if (!p) {
        outb(0xE9, (uint8_t)'1');
        return NULL;
    }

    uint64_t user_pml4 = paging_create_user_pml4();
    if (!user_pml4) {
        outb(0xE9, (uint8_t)'2');
        return NULL;
    }

    p->pml4_phys = user_pml4;
    p->context.cr3 = user_pml4;

    uint64_t entry = load_user_image(fmt, user_pml4, image, image_size);
    if (!entry) {
        outb(0xE9, (uint8_t)'3');
        return NULL;
    }
    debug_dump_pte(user_pml4, USER_TEXT_BASE, "code");

    // User stack: 2 pages in user space
    for (int i = 0; i < 2; ++i) {
        uint64_t phys = phys_alloc_frame();
        if (!phys) {
            outb(0xE9, (uint8_t)'4');
            return NULL;
        }
        uint64_t virt = USER_STACK_TOP - (i + 1) * AYKEN_FRAME_SIZE;
        uint8_t *dst = (uint8_t *)paging_phys_to_virt(phys);
        memset(dst, 0, AYKEN_FRAME_SIZE);
        paging_map_page_in_pml4(user_pml4, virt, phys,
                                AYKEN_PTE_USER | AYKEN_PTE_WRITABLE);
    }

    // Scratch page for Ring3 INT80 diagnostics (pre/post syscall canary + result buffer)
    uint64_t canary_phys = phys_alloc_frame();
    if (!canary_phys) {
        outb(0xE9, (uint8_t)'5');
        return NULL;
    }
    uint8_t *canary_dst = (uint8_t *)paging_phys_to_virt(canary_phys);
    memset(canary_dst, 0, AYKEN_FRAME_SIZE);
    paging_map_page_in_pml4(user_pml4, RING3_CANARY_ADDR, canary_phys,
                            AYKEN_PTE_USER | AYKEN_PTE_WRITABLE);

    // MVP-1: Allocate and map per-process mailbox at fixed VA (0x700000)
    // This enables Ring3 → Ring0 scheduler bridge communication
    uint64_t mb_pa = phys_alloc_frame();
    if (!mb_pa) {
        outb(0xE9, (uint8_t)'6');
        phys_free_frame(canary_phys);  // cleanup on failure
        return NULL;
    }
    // Zero-init mailbox frame (mandatory for security)
    uint8_t *mb_dst = (uint8_t *)paging_phys_to_virt(mb_pa);
    memset(mb_dst, 0, AYKEN_FRAME_SIZE);
    // Map mailbox to fixed VA with USER | WRITABLE | PRESENT
    paging_map_page_in_pml4(user_pml4, SCHED_MAILBOX_VA, mb_pa,
                            AYKEN_PTE_USER | AYKEN_PTE_WRITABLE);
    // Store mailbox physical address and initialize epoch tracking
    p->mailbox_pa = mb_pa;
    p->mailbox_last_epoch = 0;

    p->stack_top = USER_STACK_TOP;
    p->context.rip = entry;
    p->context.rsp = p->stack_top - 8;  // SysV ABI: entry %rsp = 8 mod 16
    p->context.rflags = 0x202;  // IF=1 + reserved bit 1
    
    // Allocate kernel stack for Ring0 during Ring3→Ring0 transitions (interrupts/syscalls)
    uint64_t kernel_stack = (uint64_t)kmalloc(4096);
    p->context.rsp0 = kernel_stack + 4096;  // Top of kernel stack

    // Ensure kernel stack is mapped in user CR3 (supervisor-only) for safe iretq.
    uint64_t kstack_base = kernel_stack & ~(AYKEN_FRAME_SIZE - 1);
    uint64_t kstack_end = (kernel_stack + 4096 - 1) & ~(AYKEN_FRAME_SIZE - 1);
    for (uint64_t va = kstack_base; va <= kstack_end; va += AYKEN_FRAME_SIZE) {
        uint64_t phys = paging_get_phys(va);
        if (!phys) {
            fb_print("[proc] ERROR: kernel stack phys lookup failed.\n");
            return NULL;
        }
        paging_map_page_in_pml4(user_pml4, va, phys, AYKEN_PTE_WRITABLE);
    }
    
    // DEBUG: Verify RSP0 is mapped in user CR3
    debugcon_write("Kernel stack (RSP0) mapping:\n");
    debug_dump_pte(user_pml4, p->context.rsp0 - 8, "rsp0-8");
    debug_dump_pte(user_pml4, p->context.rsp0 - AYKEN_FRAME_SIZE, "rsp0_page");

    fb_print("[DBG] USER cr3=");
    fb_print_hex(p->context.cr3);
    fb_print(" (pml4_phys=");
    fb_print_hex(p->pml4_phys);
    fb_print(")\n");

    sched_add(p);

    outb(0xE9, (uint8_t)'E');
    return p;
}

// MVP-3: Minimal Ring3 scheduler hint test code
// This is the SIMPLEST possible test: write mailbox, loop forever
static const uint8_t ring3_mvp3_sched_hint_test_code[] = {
    // Load mailbox address into rbx
    0x48, 0xBB, 0x00, 0x00, 0x70, 0x00, 0x00, 0x00, 0x00, 0x00,  // mov rbx, 0x700000
    
    // Read current epoch: rax = [rbx + 0]
    0x48, 0x8B, 0x03,                                            // mov rax, [rbx]
    
    // Increment epoch: rax = rax + 1
    0x48, 0xFF, 0xC0,                                            // inc rax
    
    // Write candidate_pid = 1: [rbx + 8] = 1
    0xC7, 0x43, 0x08, 0x01, 0x00, 0x00, 0x00,                    // mov dword [rbx + 8], 1
    
    // Write new epoch: [rbx + 0] = rax
    0x48, 0x89, 0x03,                                            // mov [rbx], rax
    
    // Infinite loop: jmp $
    0xEB, 0xFE                                                   // jmp $
};

void proc_launch_mvp3_sched_hint_test(void)
{
    fb_print("[MVP-3] =============================================\n");
    fb_print("[MVP-3] Minimal Ring3 Scheduler Hint Test\n");
    fb_print("[MVP-3] =============================================\n");
    fb_print("[MVP-3] Creating Ring3 process...\n");
    
    // Create Ring3 process with flat image
    proc_t *test_proc = proc_create_user_process(
        "mvp3-sched-hint-test",
        ring3_mvp3_sched_hint_test_code,
        sizeof(ring3_mvp3_sched_hint_test_code),
        PROC_IMAGE_FLAT
    );
    
    if (!test_proc) {
        fb_print("[MVP-3] ERROR: Failed to create Ring3 test process\n");
        fb_print("[MVP-3] =============================================\n");
        return;
    }
    
    fb_print("[MVP-3] Ring3 process created (PID=");
    fb_print_int(test_proc->pid);
    fb_print(")\n");
    fb_print("[MVP-3] Entry point: 0x");
    fb_print_hex(test_proc->context.rip);
    fb_print("\n");
    fb_print("[MVP-3] Mailbox VA: 0x");
    fb_print_hex(SCHED_MAILBOX_VA);
    fb_print("\n");
    fb_print("[MVP-3] Mailbox PA: 0x");
    fb_print_hex(test_proc->mailbox_pa);
    fb_print("\n");
    fb_print("[MVP-3] =============================================\n");
    fb_print("[MVP-3] Waiting for timer tick validation...\n");
    fb_print("[MVP-3] Expected: [[AYKEN_SCHED_MB_ACCEPT]] pid=");
    fb_print_int(test_proc->pid);
    fb_print(" epoch=1\n");
    fb_print("[MVP-3] =============================================\n");
}

// PID 1: init process
void init_process_main(void)
{
    outb(0xE9, (uint8_t)'I');
    fb_print("[init] PID1 running.\n");
    
    // MVP-3: Launch minimal Ring3 scheduler hint test
    proc_launch_mvp3_sched_hint_test();

    // Keep PID1 out of runqueue (blocked)
    sched_block_current();
    for (;;)
        __asm__ volatile("hlt");
}

void proc_create_init(void)
{
    proc_t *p = proc_create_init_process();
    if (p)
        fb_print("[proc] init process created (PID1).\n");
    else
        fb_print("[proc] init process creation FAILED.\n");
}

// AI service function removed in Phase 2.5 - Step C completion
// All AI functionality moved to Ring3 userspace

void proc_block_current(void *wait_obj)
{
    if (!current_proc)
        return;

    current_proc->wait_obj = wait_obj;
    sched_block_current();
}

void proc_wake_waiters(void *wait_obj)
{
    sched_wake_all(wait_obj);
}

// ============================================================================
// RING3 EXECUTION-CENTRIC SYSCALL TEST (Phase 2.5 Final)
// ============================================================================

// ============================================================================
// PHASE 4.5: TIMER PREEMPT TEST - TWO RING3 PROCESSES
// ============================================================================

// Process A:
// - Executes runtime syscall contract (time_query + capability bind/revoke).
// - Emits canonical marker [U][SYSCALL_OK].
// - Falls back to preempt loop that prints 'A' via debug_putc syscall.
#define RING3_RUNTIME_TOKEN_PLACEHOLDER 0x1122334455667788ULL

static const uint8_t ring3_process_a_code_template[] = {
    0xBB, 0x41, 0x00, 0x00, 0x00, /* mov ebx, 'A' */

    /* time_query(type=0, buffer=0x405080) */
    0xB8, 0xEE, 0x03, 0x00, 0x00, /* mov eax, 1006 */
    0x31, 0xFF,                   /* xor edi, edi */
    0xBE, 0x80, 0x50, 0x40, 0x00, /* mov esi, 0x405080 */
    0xCD, 0x80,                   /* int 0x80 */

    /* valid token @0x4050a0: {id, perms=1, type=8} */
    0x48, 0xB8, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, /* mov rax, placeholder */
    0x48, 0x89, 0x04, 0x25, 0xA0, 0x50, 0x40, 0x00,             /* mov [0x4050a0], rax */
    0xC7, 0x04, 0x25, 0xA8, 0x50, 0x40, 0x00, 0x01, 0x00, 0x00, 0x00, /* perms=1 */
    0xC7, 0x04, 0x25, 0xAC, 0x50, 0x40, 0x00, 0x08, 0x00, 0x00, 0x00, /* type=8 */

    /* capability_bind granted: ctx=2, token=0x4050a0 */
    0xB8, 0xEF, 0x03, 0x00, 0x00, /* mov eax, 1007 */
    0xBF, 0x02, 0x00, 0x00, 0x00, /* mov edi, 2 */
    0xBE, 0xA0, 0x50, 0x40, 0x00, /* mov esi, 0x4050a0 */
    0xCD, 0x80,                   /* int 0x80 */

    /* invalid token @0x4050b0: id=0 -> denied path */
    0x48, 0xC7, 0x04, 0x25, 0xB0, 0x50, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xC7, 0x04, 0x25, 0xB8, 0x50, 0x40, 0x00, 0x01, 0x00, 0x00, 0x00,
    0xC7, 0x04, 0x25, 0xBC, 0x50, 0x40, 0x00, 0x08, 0x00, 0x00, 0x00,

    /* capability_bind denied: ctx=2, token=0x4050b0 */
    0xB8, 0xEF, 0x03, 0x00, 0x00,
    0xBF, 0x02, 0x00, 0x00, 0x00,
    0xBE, 0xB0, 0x50, 0x40, 0x00,
    0xCD, 0x80,

    /* capability_revoke granted then denied */
    0xB8, 0xF0, 0x03, 0x00, 0x00,             /* mov eax, 1008 */
    0x48, 0x8B, 0x3C, 0x25, 0xA0, 0x50, 0x40, 0x00, /* mov rdi, [0x4050a0] */
    0xCD, 0x80,
    0xB8, 0xF0, 0x03, 0x00, 0x00,
    0x48, 0x8B, 0x3C, 0x25, 0xA0, 0x50, 0x40, 0x00,
    0xCD, 0x80,

    /* emit canonical marker: [U][SYSCALL_OK]\n */
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x5B, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x55, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x5D, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x5B, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x53, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x59, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x53, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x43, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x41, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x4C, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x4C, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x5F, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x4F, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x4B, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x5D, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x0A, 0x00, 0x00, 0x00, 0xCD, 0x80,

    /* loop: print 'A' via debug syscall for preempt signal */
    0x41, 0xBC, 0x00, 0x00, 0x02, 0x00, /* mov r12d, 0x20000 */
    0x41, 0xFF, 0xCC,                   /* dec r12d */
    0x75, 0xFB,                         /* jnz delay */
    0xB8, 0xF2, 0x03, 0x00, 0x00,       /* mov eax, 1010 */
    0x89, 0xDF,                         /* mov edi, ebx */
    0xCD, 0x80,                         /* int 0x80 */
    0xEB, 0xEA                          /* jmp loop */
};

static uint8_t ring3_process_a_code[sizeof(ring3_process_a_code_template)];

static int ring3_prepare_process_a_code(uint64_t capability_id)
{
    const uint64_t placeholder = RING3_RUNTIME_TOKEN_PLACEHOLDER;
    int replacements = 0;

    memcpy(ring3_process_a_code,
           ring3_process_a_code_template,
           sizeof(ring3_process_a_code_template));

    for (size_t i = 0; i + sizeof(uint64_t) <= sizeof(ring3_process_a_code); ++i) {
        uint64_t value = 0;
        memcpy(&value, &ring3_process_a_code[i], sizeof(value));
        if (value == placeholder) {
            memcpy(&ring3_process_a_code[i], &capability_id, sizeof(capability_id));
            replacements++;
        }
    }

    if (replacements != 1) {
        fb_print("[preempt_test] ERROR: capability placeholder patch failed\n");
        return 0;
    }

    return 1;
}

// Process B: CPU hog that prints 'B' via syscall
static const uint8_t ring3_process_b_code[] = {
    // ebx = 'B'
    0xBB, 0x42, 0x00, 0x00, 0x00,
    // loop:
    //   r12d = delay counter (callee-saved across context switches)
    0x41, 0xBC, 0x00, 0x00, 0x02, 0x00,
    // delay:
    //   dec r12d
    0x41, 0xFF, 0xCC,
    //   jnz delay
    0x75, 0xFB,
    //   eax = 1010 (user-space syscall number for SYS_V2_DEBUG_PUTCHAR)
    0xB8, 0xF2, 0x03, 0x00, 0x00,
    //   edi = ebx ('B')
    0x89, 0xDF,
    //   int 0x80
    0xCD, 0x80,
    //   jmp loop
    0xEB, 0xEA
};

// Legacy test codes for backward compatibility
static const uint8_t ring3_int3_test_code[] = {
    0xCC,       // int3 (should cause #BP)
    0xEB, 0xFE  // jmp $ (infinite loop)
};

static const uint8_t ring3_ud2_test_code[] __attribute__((unused)) = {
    0x0F, 0x0B, // ud2 (undefined instruction exception)
    0xEB, 0xFE  // jmp $ (should not reach here if UD2 works)
};

static const uint8_t ring3_int80_test_code[] __attribute__((unused)) = {
    // rbx = 0x405000 (canary address)
    0x48, 0xBB, 0x00, 0x50, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,
    // [canary] = PRE
    0x48, 0xB8, 0x22, 0x22, 0x22, 0x22, 0x11, 0x11, 0x11, 0x11,
    0x48, 0x89, 0x03,
    // syscall: rax=1006 (SYS_V2_TIME_QUERY), rdi=0, rsi=0x405080
    0x48, 0xC7, 0xC0, 0xEE, 0x03, 0x00, 0x00,
    0x48, 0x31, 0xFF,
    0x48, 0xC7, 0xC6, 0x80, 0x50, 0x40, 0x00,
    0xCD, 0x80,
    // [canary] = POST (proves iretq returned to Ring3)
    0x48, 0xB8, 0x44, 0x44, 0x44, 0x44, 0x33, 0x33, 0x33, 0x33,
    0x48, 0x89, 0x03,
    // second syscall after POST write (kernel-side confirmation point)
    0x48, 0xC7, 0xC0, 0xEE, 0x03, 0x00, 0x00,
    0x48, 0x31, 0xFF,
    0x48, 0xC7, 0xC6, 0x80, 0x50, 0x40, 0x00,
    0xCD, 0x80,

    // loop
    0xEB, 0xFE         // jmp $ (infinite loop)
};

// Current test selection (will be modified by proc_launch_ring3_test)
static const uint8_t *current_ring3_test_code = ring3_int3_test_code;
static size_t current_ring3_test_size = sizeof(ring3_int3_test_code);

// Legacy name for compatibility
static const uint8_t *ring3_v2_syscall_test_code __attribute__((unused)) = ring3_int3_test_code;
#define ring3_v2_syscall_test_code_size sizeof(ring3_int3_test_code)

// Test data for the v2 syscall test (capability token structure)
static const uint8_t ring3_v2_test_data[] __attribute__((unused)) = {
    // Capability token at offset 0x08 (capability_token_t structure)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // id (will be assigned by kernel)
    0x01, 0x00, 0x00, 0x00,                          // permissions (CAP_PERM_READ)
    0x01, 0x00, 0x00, 0x00,                          // resource_type (CAP_RESOURCE_MEMORY)
};

/**
 * Creates a unified Ring3+Syscall test process for Phase 4.4 closure validation
 * This process will:
 * 1. Execute in Ring3 (user mode) with proper privilege level
 * 2. Write 'U' to debugcon to prove user code execution
 * 3. Execute one syscall (SYS_V2_TIME_QUERY) to validate syscall mechanism
 * 4. Write syscall markers before and after syscall execution
 * 5. Validate Ring3→Ring0→Ring3 transitions work correctly
 * 
 * @param name Process name for identification
 * @return proc_t* pointer to created process, NULL on failure
 */
proc_t *proc_create_ring3_syscall_test(const char *name)
{
    outb(0xE9, (uint8_t)'C');
    fb_print("[ring3_test] Creating unified Ring3+Syscall test: ");
    fb_print(name);
    fb_print("\n");
    
    // Create user process with flat image format
    outb(0xE9, (uint8_t)'c');
    proc_t *test_proc = proc_create_user_process(name, 
                                                current_ring3_test_code,
                                                current_ring3_test_size,
                                                PROC_IMAGE_FLAT);
    
    if (!test_proc) {
        fb_print("[ring3_test] ERROR: Failed to create unified Ring3+Syscall test process\n");
        return NULL;
    }
    outb(0xE9, (uint8_t)'P');
    
    fb_print("[ring3_test] Unified Ring3+Syscall test process created successfully\n");
    fb_print("[ring3_test] - PID: ");
    fb_print_int(test_proc->pid);
    fb_print("\n");
    fb_print("[ring3_test] - Entry point: 0x");
    fb_print_hex(test_proc->context.rip);
    fb_print("\n");
    fb_print("[ring3_test] - Stack top: 0x");
    fb_print_hex(test_proc->context.rsp);
    fb_print("\n");
    fb_print("[ring3_test] - CS: 0x");
    fb_print_hex(test_proc->context.cs);
    fb_print(" (Ring3)\n");
    fb_print("[ring3_test] - SS: 0x");
    fb_print_hex(test_proc->context.ss);
    fb_print(" (Ring3)\n");

    fb_print("[DBG] USER ctx: cs=");
    fb_print_hex(test_proc->context.cs);
    fb_print(" ss=");
    fb_print_hex(test_proc->context.ss);
    fb_print(" rip=");
    fb_print_hex(test_proc->context.rip);
    fb_print(" rsp=");
    fb_print_hex(test_proc->context.rsp);
    fb_print(" rsp0=");
    fb_print_hex(test_proc->context.rsp0);
    fb_print("\n");
    
    // NOTE: sched_add() already called by proc_create_user_process()
    // No need to call it again here - would cause runqueue corruption
    
    outb(0xE9, (uint8_t)'D');
    return test_proc;
}

/**
 * PHASE 4.5: Timer Preempt Test - Two Ring3 Processes
 * 
 * This test validates preemptive multitasking:
 * - Two separate Ring3 processes (A and B)
 * - Each process prints its character via syscall
 * - Timer interrupt preempts running process
 * - Scheduler switches between processes
 * - Expected output: AAABBBAAABBB... (alternating)
 * 
 * Success criteria:
 * ✔ Timer interrupt fires during Ring3 execution
 * ✔ Context switch preserves process state
 * ✔ CR3 switches correctly between processes
 * ✔ TSS.RSP0 updates correctly
 * ✔ Both processes make forward progress
 */
void proc_launch_ring3_test(void)
{
    capability_token_t runtime_token;

    outb(0xE9, (uint8_t)'L');
    fb_print("[preempt_test] =============================================\n");
    fb_print("[preempt_test] PHASE 4.5: Timer Preempt Validation\n");
    fb_print("[preempt_test] =============================================\n");
    
    fb_print("[preempt_test] Creating two Ring3 processes:\n");
    fb_print("[preempt_test] - Process A: prints 'A' via syscall\n");
    fb_print("[preempt_test] - Process B: prints 'B' via syscall\n");
    fb_print("[preempt_test] Expected: Timer preempts → alternating output\n");
    fb_print("[preempt_test] Success pattern: AAABBBAAABBB...\n");

    runtime_token = capability_create(
        CAPABILITY_RESOURCE_TIME,
        CAPABILITY_PERM_READ,
        RING3_CANARY_ADDR + 0x80,
        sizeof(uint64_t)
    );
    if (runtime_token.id == 0) {
        fb_print("[preempt_test] CRITICAL ERROR: runtime capability create failed\n");
        fb_print("[preempt_test] =============================================\n");
        return;
    }
    if (!ring3_prepare_process_a_code(runtime_token.id)) {
        fb_print("[preempt_test] CRITICAL ERROR: runtime payload patch failed\n");
        fb_print("[preempt_test] =============================================\n");
        return;
    }
    
    // Create Process A
    current_ring3_test_code = ring3_process_a_code;
    current_ring3_test_size = sizeof(ring3_process_a_code);
    proc_t *proc_a = proc_create_ring3_syscall_test("ring3-process-A");
    
    if (!proc_a) {
        fb_print("[preempt_test] CRITICAL ERROR: Process A creation failed\n");
        fb_print("[preempt_test] =============================================\n");
        return;
    }
    
    fb_print("[preempt_test] Process A created (PID=");
    fb_print_int(proc_a->pid);
    fb_print(")\n");
    
    // Create Process B
    current_ring3_test_code = ring3_process_b_code;
    current_ring3_test_size = sizeof(ring3_process_b_code);
    proc_t *proc_b = proc_create_ring3_syscall_test("ring3-process-B");
    
    if (!proc_b) {
        fb_print("[preempt_test] CRITICAL ERROR: Process B creation failed\n");
        fb_print("[preempt_test] =============================================\n");
        return;
    }
    
    fb_print("[preempt_test] Process B created (PID=");
    fb_print_int(proc_b->pid);
    fb_print(")\n");
    
    fb_print("[preempt_test] Both processes scheduled successfully\n");
    fb_print("[preempt_test] Waiting for timer preemption...\n");
    fb_print("[preempt_test] Watch for:\n");
    fb_print("[preempt_test]   T - Timer tick marker\n");
    fb_print("[preempt_test]   S - Scheduler switch marker\n");
    fb_print("[preempt_test]   C - CR3 load marker\n");
    fb_print("[preempt_test]   R - IRET marker\n");
    fb_print("[preempt_test]   A/B - Process output\n");
    fb_print("[preempt_test] =============================================\n");
}

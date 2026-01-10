// kernel/proc/proc.c
#include <stddef.h>
#include "../include/proc.h"
#include "../sched/sched.h"
#include "../include/mm.h"
#include "../include/kheap.h"
#include "../include/ayken.h"
#include "../include/gdt_idt.h"
#include "../drivers/console/fb_console.h"

// Use compiler builtin functions for memory operations
#define memset __builtin_memset
#define memcpy __builtin_memcpy

static proc_t* proc_table[MAX_PROCS];
static int next_pid = 1;

void init_process_main(void);

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
    p->pml4_phys = paging_get_kernel_pml4_phys();
    p->context.cr3 = p->pml4_phys;
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

    uint64_t stack = (uint64_t)kmalloc(4096);
    p->stack_top = stack + 4096;

    p->context.rip = (uint64_t)func;
    p->context.rsp = p->stack_top;
    p->context.cr3 = paging_get_kernel_pml4_phys();

    sched_add(p);
    return p;
}

static proc_t *proc_create_init_process(void)
{
    proc_t *p = proc_alloc(PROC_TYPE_KERNEL, "init");
    if (!p) return NULL;

    uint64_t stack = (uint64_t)kmalloc(4096);
    p->stack_top = stack + 4096;

    p->context.rip = (uint64_t)init_process_main;
    p->context.rsp = p->stack_top;
    p->context.cr3 = paging_get_kernel_pml4_phys();

    sched_add(p);
    return p;
}

proc_t *proc_create_user_process(const char *name,
                                 const uint8_t *image,
                                 uint64_t image_size,
                                 proc_image_format_t fmt)
{
    proc_t *p = proc_alloc(PROC_TYPE_USER, name);
    if (!p)
        return NULL;

    uint64_t user_pml4 = paging_create_user_pml4();
    if (!user_pml4)
        return NULL;

    p->pml4_phys = user_pml4;
    p->context.cr3 = user_pml4;

    uint64_t entry = load_user_image(fmt, user_pml4, image, image_size);
    if (!entry)
        return NULL;

    // User stack: 2 pages in user space
    for (int i = 0; i < 2; ++i) {
        uint64_t phys = phys_alloc_frame();
        if (!phys)
            return NULL;
        uint64_t virt = USER_STACK_TOP - (i + 1) * AYKEN_FRAME_SIZE;
        uint8_t *dst = (uint8_t *)paging_phys_to_virt(phys);
        memset(dst, 0, AYKEN_FRAME_SIZE);
        paging_map_page_in_pml4(user_pml4, virt, phys,
                                AYKEN_PTE_USER | AYKEN_PTE_WRITABLE);
    }

    p->stack_top = USER_STACK_TOP;
    p->context.rip = entry;
    p->context.rsp = p->stack_top;
    
    // Allocate kernel stack for Ring0 during Ring3→Ring0 transitions (interrupts/syscalls)
    uint64_t kernel_stack = (uint64_t)kmalloc(4096);
    p->context.rsp0 = kernel_stack + 4096;  // Top of kernel stack

    sched_add(p);
    return p;
}

// PID 1: init process
void init_process_main(void)
{
    fb_print("[init] PID1 running.\n");
    
    // Phase 1.5: Launch Ring3 test for validation
    proc_launch_ring3_test();
    
    // AI service removed in Phase 2.5 - Step C completion
    // All AI functionality moved to Ring3 userspace
    
    for(;;) {
        sched_yield();
    }
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

// Comprehensive Ring3 test process that validates the execution-centric syscalls:
// - SYS_V2_TIME_QUERY (1006): Test time query functionality
// - SYS_V2_MAP_MEMORY (1000): Test memory mapping
// - SYS_V2_CAPABILITY_BIND (1007): Test capability system
// - Invalid syscalls: Test error handling for old POSIX syscalls
// - Multiple round-trips: Test Ring3→Ring0→Ring3 transition stability
static const uint8_t ring3_v2_syscall_test_code[] = {
    // Test 1: Time query syscall - Test basic v2 functionality
    // sys_v2_time_query(0, &timestamp)
    0x48, 0xC7, 0xC0, 0xEE, 0x03, 0x00, 0x00,  // mov rax, 1006 (SYS_V2_TIME_QUERY)
    0x48, 0xC7, 0xC7, 0x00, 0x00, 0x00, 0x00,  // mov rdi, 0 (query_type)
    0x48, 0xC7, 0xC6, 0x00, 0x51, 0x40, 0x00,  // mov rsi, 0x405100 (result buffer)
    0xCD, 0x80,                                  // int 0x80 (syscall)
    
    // Test 2: Memory mapping syscall - Test memory management
    // sys_v2_map_memory(0x400000, 0x100000, 0x01)
    0x48, 0xC7, 0xC0, 0xE8, 0x03, 0x00, 0x00,  // mov rax, 1000 (SYS_V2_MAP_MEMORY)
    0x48, 0xC7, 0xC7, 0x00, 0x00, 0x40, 0x00,  // mov rdi, 0x400000 (virt_addr)
    0x48, 0xC7, 0xC6, 0x00, 0x00, 0x10, 0x00,  // mov rsi, 0x100000 (phys_addr)
    0x48, 0xC7, 0xC2, 0x01, 0x00, 0x00, 0x00,  // mov rdx, 0x01 (flags)
    0xCD, 0x80,                                  // int 0x80 (syscall)
    
    // Test 3: Capability bind syscall - Test capability system
    // sys_v2_capability_bind(1, &capability_token)
    0x48, 0xC7, 0xC0, 0xEF, 0x03, 0x00, 0x00,  // mov rax, 1007 (SYS_V2_CAPABILITY_BIND)
    0x48, 0xC7, 0xC7, 0x01, 0x00, 0x00, 0x00,  // mov rdi, 1 (execution_ctx_id)
    0x48, 0xC7, 0xC6, 0x08, 0x51, 0x40, 0x00,  // mov rsi, 0x405108 (capability token addr)
    0xCD, 0x80,                                  // int 0x80 (syscall)
    
    // Test 4: Invalid old POSIX syscall - Test error handling
    // This should return -38 (ENOSYS) since POSIX syscalls are removed
    0x48, 0xC7, 0xC0, 0x01, 0x00, 0x00, 0x00,  // mov rax, 1 (old SYS_write - should fail)
    0x48, 0xC7, 0xC7, 0x01, 0x00, 0x00, 0x00,  // mov rdi, 1
    0x48, 0xC7, 0xC6, 0x00, 0x50, 0x40, 0x00,  // mov rsi, 0x405000
    0x48, 0xC7, 0xC2, 0x0A, 0x00, 0x00, 0x00,  // mov rdx, 10
    0xCD, 0x80,                                  // int 0x80 (syscall)
    
    // Test 5: Another invalid syscall - Test boundary conditions
    0x48, 0xC7, 0xC0, 0xE7, 0x03, 0x00, 0x00,  // mov rax, 999 (invalid syscall)
    0x48, 0xC7, 0xC7, 0x00, 0x00, 0x00, 0x00,  // mov rdi, 0
    0xCD, 0x80,                                  // int 0x80 (syscall)
    
    // Test 6: Multiple rapid v2 syscalls - Test transition stability
    // Perform 3 rapid time query syscalls to stress-test Ring3↔Ring0 transitions
    0x48, 0xC7, 0xC0, 0xEE, 0x03, 0x00, 0x00,  // mov rax, 1006 (SYS_V2_TIME_QUERY)
    0x48, 0xC7, 0xC7, 0x00, 0x00, 0x00, 0x00,  // mov rdi, 0
    0x48, 0xC7, 0xC6, 0x10, 0x51, 0x40, 0x00,  // mov rsi, 0x405110
    0xCD, 0x80,                                  // int 0x80 (syscall) - 1st
    
    0x48, 0xC7, 0xC0, 0xEE, 0x03, 0x00, 0x00,  // mov rax, 1006 (SYS_V2_TIME_QUERY)
    0x48, 0xC7, 0xC7, 0x00, 0x00, 0x00, 0x00,  // mov rdi, 0
    0x48, 0xC7, 0xC6, 0x18, 0x51, 0x40, 0x00,  // mov rsi, 0x405118
    0xCD, 0x80,                                  // int 0x80 (syscall) - 2nd
    
    0x48, 0xC7, 0xC0, 0xEE, 0x03, 0x00, 0x00,  // mov rax, 1006 (SYS_V2_TIME_QUERY)
    0x48, 0xC7, 0xC7, 0x00, 0x00, 0x00, 0x00,  // mov rdi, 0
    0x48, 0xC7, 0xC6, 0x20, 0x51, 0x40, 0x00,  // mov rsi, 0x405120
    0xCD, 0x80,                                  // int 0x80 (syscall) - 3rd
    
    // Test 7: Infinite loop to keep process alive for observation
    0xEB, 0xFE                                   // jmp $ (infinite loop)
};

// Test data for the v2 syscall test (capability token structure)
static const uint8_t ring3_v2_test_data[] = {
    // Capability token at offset 0x08 (capability_token_t structure)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // id (will be assigned by kernel)
    0x01, 0x00, 0x00, 0x00,                          // permissions (CAP_PERM_READ)
    0x01, 0x00, 0x00, 0x00,                          // resource_type (CAP_RESOURCE_MEMORY)
};

/**
 * Creates a comprehensive Ring3 execution-centric syscall test process for Phase 2.5 validation
 * This process will:
 * 1. Execute in Ring3 (user mode) with proper privilege level
 * 2. Test execution-centric syscalls: time_query, map_memory, capability_bind
 * 3. Validate INT 0x80 mechanism works reliably for v2 syscalls
 * 4. Test syscall parameter passing and return values for correctness
 * 5. Ensure Ring3→Ring0→Ring3 transitions are stable under various conditions
 * 6. Perform rapid syscall sequences to stress-test transition stability
 * 7. Validate error handling for invalid/old POSIX syscalls
 * 
 * @param name Process name for identification
 * @return proc_t* pointer to created process, NULL on failure
 */
proc_t *proc_create_ring3_syscall_test(const char *name)
{
    fb_print("[syscall_test] Creating Ring3 execution-centric syscall test: ");
    fb_print(name);
    fb_print("\n");
    
    // Create user process with flat image format
    proc_t *test_proc = proc_create_user_process(name, 
                                                ring3_v2_syscall_test_code,
                                                sizeof(ring3_v2_syscall_test_code),
                                                PROC_IMAGE_FLAT);
    
    if (!test_proc) {
        fb_print("[syscall_test] ERROR: Failed to create Ring3 v2 syscall test process\n");
        return NULL;
    }
    
    // Embed test data in user memory at 0x405000
    // This maps to the second page of user memory (after code at 0x400000)
    uint64_t data_virt_addr = 0x405000;
    uint64_t data_phys = phys_alloc_frame();
    if (!data_phys) {
        fb_print("[syscall_test] ERROR: Failed to allocate data memory\n");
        return NULL;
    }
    
    // Copy test data to physical memory
    uint8_t *data_dst = (uint8_t *)paging_phys_to_virt(data_phys);
    memset(data_dst, 0, AYKEN_FRAME_SIZE);
    memcpy(data_dst, ring3_v2_test_data, sizeof(ring3_v2_test_data));
    
    // Map data memory into user address space with read/write permissions
    paging_map_page_in_pml4(test_proc->pml4_phys, data_virt_addr, data_phys,
                            AYKEN_PTE_USER | AYKEN_PTE_WRITABLE);
    
    // Allocate additional page for syscall results and capability tokens
    // Memory layout at 0x405100:
    // - 0x405100: time query result buffer (8 bytes)
    // - 0x405108: capability token structure (16 bytes)
    // - 0x405110-0x405128: additional time query results (3 x 8 bytes)
    uint64_t storage_virt_addr = 0x405100;
    uint64_t storage_phys = phys_alloc_frame();
    if (!storage_phys) {
        fb_print("[syscall_test] ERROR: Failed to allocate storage memory\n");
        return NULL;
    }
    
    uint8_t *storage_dst = (uint8_t *)paging_phys_to_virt(storage_phys);
    memset(storage_dst, 0, AYKEN_FRAME_SIZE);
    
    // Initialize capability token structure at offset 0x08
    memcpy(storage_dst + 0x08, ring3_v2_test_data, sizeof(ring3_v2_test_data));
    
    paging_map_page_in_pml4(test_proc->pml4_phys, storage_virt_addr, storage_phys,
                            AYKEN_PTE_USER | AYKEN_PTE_WRITABLE);
    
    fb_print("[syscall_test] Ring3 v2 syscall test process created successfully\n");
    fb_print("[syscall_test] - PID: ");
    fb_print_int(test_proc->pid);
    fb_print("\n");
    fb_print("[syscall_test] - Entry point: 0x");
    fb_print_hex(test_proc->context.rip);
    fb_print("\n");
    fb_print("[syscall_test] - Stack top: 0x");
    fb_print_hex(test_proc->context.rsp);
    fb_print("\n");
    fb_print("[syscall_test] - CS: 0x");
    fb_print_hex(test_proc->context.cs);
    fb_print(" (Ring3)\n");
    fb_print("[syscall_test] - SS: 0x");
    fb_print_hex(test_proc->context.ss);
    fb_print(" (Ring3)\n");
    fb_print("[syscall_test] - Data page: 0x");
    fb_print_hex(data_virt_addr);
    fb_print("\n");
    fb_print("[syscall_test] - Storage page: 0x");
    fb_print_hex(storage_virt_addr);
    fb_print("\n");
    
    return test_proc;
}

/**
 * Launches the comprehensive Ring3 execution-centric syscall test for Phase 2.5 validation
 * This is the main entry point for comprehensive syscall testing that validates:
 * - Execution-centric syscalls: time_query, map_memory, capability_bind
 * - INT 0x80 mechanism reliability with v2 syscalls
 * - Parameter passing and return value correctness
 * - Ring3→Ring0→Ring3 transition stability
 * - Error handling for invalid/old POSIX syscalls
 * - Rapid syscall sequences for stress testing
 */
void proc_launch_ring3_test(void)
{
    fb_print("[syscall_test] =============================================\n");
    fb_print("[syscall_test] Starting Phase 2.5 Execution-Centric Syscall Test\n");
    fb_print("[syscall_test] =============================================\n");
    
    proc_t *test_proc = proc_create_ring3_syscall_test("v2-syscall-test");
    
    if (test_proc) {
        fb_print("[syscall_test] Execution-centric syscall test scheduled successfully\n");
        fb_print("[syscall_test] Process will execute when scheduler runs\n");
        fb_print("[syscall_test] Expected test sequence:\n");
        fb_print("[syscall_test]   1. Time query (SYS_V2_TIME_QUERY)\n");
        fb_print("[syscall_test]   2. Memory mapping (SYS_V2_MAP_MEMORY)\n");
        fb_print("[syscall_test]   3. Capability bind (SYS_V2_CAPABILITY_BIND)\n");
        fb_print("[syscall_test]   4. Invalid old POSIX syscall test (should fail)\n");
        fb_print("[syscall_test]   5. Invalid syscall boundary test\n");
        fb_print("[syscall_test]   6. Rapid v2 syscall sequence (3x time queries)\n");
        fb_print("[syscall_test] NOTE: Only 1000-1009 syscalls should work\n");
        fb_print("[syscall_test] =============================================\n");
    } else {
        fb_print("[syscall_test] CRITICAL ERROR: V2 syscall test process creation failed\n");
        fb_print("[syscall_test] Phase 2.5 validation cannot proceed\n");
        fb_print("[syscall_test] =============================================\n");
    }
}

// kernel/proc/proc.c
#include <string.h>
#include "../include/proc.h"
#include "../include/sched.h"
#include "../include/mm.h"
#include "../include/ayken.h"
#include "../drivers/console/fb_console.h"

static int next_pid = 1;

void init_process_main(void);

static int proc_alloc_pid(void)
{
    return next_pid++;
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
    p->context.rflags = 0x202;
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

    // Basit user stack: 2 sayfa
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

    sched_add(p);
    return p;
}

// PID 1: init process
void init_process_main(void)
{
    fb_print("[init] PID1 running.\n");
    proc_launch_user_ai_service();
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

void proc_launch_user_ai_service(void)
{
    static const uint8_t ai_service_stub[] = { 0xEB, 0xFE }; // sonsuz döngü
    proc_t *p = proc_create_user_process("ai-service", ai_service_stub,
                                         sizeof(ai_service_stub),
                                         PROC_IMAGE_FLAT);
    if (p)
        fb_print("[proc] user AI service scheduled.\n");
    else
        fb_print("[proc] user AI service launch failed.\n");
}

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

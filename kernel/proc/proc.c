// kernel/proc/proc.c
#include "../include/proc.h"
#include "../include/sched.h"
#include "../include/mm.h"
#include "../drivers/console/fb_console.h"

static int next_pid = 1;

void proc_init(void)
{
    fb_print("[proc] Process subsystem init.\n");
}

proc_t *proc_create_kernel_thread(void (*func)(void))
{
    proc_t *p = (proc_t *)kmalloc(sizeof(proc_t));
    if (!p) return NULL;

    p->pid = next_pid++;
    p->state = PROC_READY;

    // Kernel stack
    uint64_t stack = (uint64_t)kmalloc(4096);
    p->stack_top = stack + 4096;

    p->pml4_phys = paging_get_phys(KERNEL_VIRT_BASE); // geçici çözüm

    // Context ayarla
    p->context.rip = (uint64_t)func;
    p->context.rsp = p->stack_top;
    p->context.rflags = 0x202;
    p->context.cr3 = p->pml4_phys;

    // Ready queue'ya ekle
    sched_add(p);

    return p;
}

// PID 1: init process
void init_process_main(void)
{
    fb_print("[init] PID1 running.\n");
    for(;;);
}

void proc_create_init(void)
{
    proc_create_kernel_thread(init_process_main);
    fb_print("[proc] init process created (PID1).\n");
}

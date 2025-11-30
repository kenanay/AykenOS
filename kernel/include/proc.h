// kernel/include/proc.h
#ifndef AYKEN_PROC_H
#define AYKEN_PROC_H

#include <stdint.h>

typedef struct cpu_context {
    uint64_t r15, r14, r13, r12;
    uint64_t rbx, rbp;
    uint64_t rip;
    uint64_t rsp;
    uint64_t rflags;
    uint64_t cr3;
} cpu_context_t;

typedef enum {
    PROC_READY = 0,
    PROC_RUNNING,
    PROC_BLOCKED,
    PROC_ZOMBIE
} proc_state_t;

typedef enum {
    PROC_TYPE_KERNEL = 0,
    PROC_TYPE_USER
} proc_type_t;

typedef enum {
    PROC_IMAGE_FLAT = 0,
    PROC_IMAGE_ELF
} proc_image_format_t;

typedef struct proc {
    int pid;
    cpu_context_t context;
    uint64_t stack_top;
    uint64_t pml4_phys;   // her process'e özel (şimdilik kernel same map)
    proc_state_t state;
    proc_type_t type;
    const char *name;
    void *wait_obj;
    struct proc *next;    // ready queue için
} proc_t;

// API
void proc_init(void);
proc_t *proc_create_kernel_thread(void (*func)(void));
void proc_create_init(void);
proc_t *proc_create_user_process(const char *name,
                                 const uint8_t *image,
                                 uint64_t image_size,
                                 proc_image_format_t fmt);
void proc_launch_user_ai_service(void);
void proc_block_current(void *wait_obj);
void proc_wake_waiters(void *wait_obj);

#endif

// kernel/include/proc.h
#ifndef AYKEN_PROC_H
#define AYKEN_PROC_H

#include <stdint.h>

#define MAX_PROCS 64

typedef struct cpu_context {
    // Callee-saved general registers
    uint64_t r15, r14, r13, r12;
    uint64_t rbx, rbp;
    
    // Instruction pointer, stack pointer, flags
    uint64_t rip;
    uint64_t rsp;
    uint64_t rflags;
    
    // Memory management
    uint64_t cr3;
    
    // Ring3 context (for privilege level transitions)
    uint16_t cs;            // Code segment selector (Ring3: 0x23, Ring0: 0x08)
    uint16_t ss;            // Stack segment selector (Ring3: 0x1B, Ring0: 0x10)
    uint64_t rsp0;          // Kernel stack RSP (for Ring0 when interrupted from Ring3)
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
// AI service function removed in Phase 2.5 - Step C completion
// All AI functionality moved to Ring3 userspace
proc_t *proc_create_ring3_syscall_test(const char *name);
void proc_launch_ring3_test(void);
void proc_block_current(void *wait_obj);
void proc_wake_waiters(void *wait_obj);
proc_t* proc_find_by_pid(int pid);

#endif

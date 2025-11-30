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

typedef struct proc {
    int pid;
    cpu_context_t context;
    uint64_t stack_top;
    uint64_t pml4_phys;   // her process'e özel (şimdilik kernel same map)
    int state;
    struct proc *next;    // ready queue için
} proc_t;

// API
void proc_init(void);
proc_t *proc_create_kernel_thread(void (*func)(void));
void proc_create_init(void);

#endif

// kernel/include/proc.h
#ifndef AYKEN_PROC_H
#define AYKEN_PROC_H

#include <stddef.h>
#include <stdint.h>
#include "ayken_abi.h"

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

/* cpu_context_t ABI hard-locks (must match ayken_abi.{h,inc}) */
_Static_assert(offsetof(cpu_context_t, r15) == CTX_R15, "cpu_context_t: CTX_R15 drift");
_Static_assert(offsetof(cpu_context_t, r14) == CTX_R14, "cpu_context_t: CTX_R14 drift");
_Static_assert(offsetof(cpu_context_t, r13) == CTX_R13, "cpu_context_t: CTX_R13 drift");
_Static_assert(offsetof(cpu_context_t, r12) == CTX_R12, "cpu_context_t: CTX_R12 drift");
_Static_assert(offsetof(cpu_context_t, rbx) == CTX_RBX, "cpu_context_t: CTX_RBX drift");
_Static_assert(offsetof(cpu_context_t, rbp) == CTX_RBP, "cpu_context_t: CTX_RBP drift");
_Static_assert(offsetof(cpu_context_t, rip) == CTX_RIP, "cpu_context_t: CTX_RIP drift");
_Static_assert(offsetof(cpu_context_t, rsp) == CTX_RSP, "cpu_context_t: CTX_RSP drift");
_Static_assert(offsetof(cpu_context_t, rflags) == CTX_RFLAGS, "cpu_context_t: CTX_RFLAGS drift");
_Static_assert(offsetof(cpu_context_t, cr3) == CTX_CR3, "cpu_context_t: CTX_CR3 drift");
_Static_assert(offsetof(cpu_context_t, cs) == CTX_CS, "cpu_context_t: CTX_CS drift");
_Static_assert(offsetof(cpu_context_t, ss) == CTX_SS, "cpu_context_t: CTX_SS drift");
_Static_assert(offsetof(cpu_context_t, rsp0) == CTX_RSP0, "cpu_context_t: CTX_RSP0 drift");
_Static_assert(sizeof(cpu_context_t) == CTX_SIZE, "cpu_context_t: CTX_SIZE drift");

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
    
    // MVP-1: Scheduler bridge mailbox (Ring3 → Ring0 interaction)
    uint64_t mailbox_pa;        // Physical address of per-process mailbox
    uint64_t mailbox_last_epoch; // Last validated epoch (monotonicity check)
#if defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)
    // Gate-4 isolated proof: per-process publish marker one-shot latch.
    uint8_t gate4_publish_emitted;
#endif
} proc_t;

// API
void proc_init(void);
proc_t *proc_create_kernel_thread(void (*func)(void));
void proc_create_init(void);
#if defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)
void proc_launch_gate4_policy_test(void);
#endif
proc_t *proc_create_user_process(const char *name,
                                 const uint8_t *image,
                                 uint64_t image_size,
                                 proc_image_format_t fmt);
// AI service function removed in Phase 2.5 - Step C completion
// All AI functionality moved to Ring3 userspace
void proc_block_current(void *wait_obj);
void proc_wake_waiters(void *wait_obj);
proc_t* proc_find_by_pid(int pid);

#endif

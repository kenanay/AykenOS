// kernel/include/proc.h
#ifndef AYKEN_PROC_H
#define AYKEN_PROC_H

#include <stddef.h>
#include <stdint.h>
#include "ayken_abi.h"
#include "execution_output_abi.h"
#include "alias_registry.h"

#define MAX_PROCS 64
#define AYKEN_MAX_PROC_GENERIC_MAPPINGS 64

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
    PROC_ZOMBIE,
    PROC_TERMINAL  // Phase-16 Task 10: Immediate kill, never reschedule
} proc_state_t;

typedef enum {
    PROC_TYPE_KERNEL = 0,
    PROC_TYPE_USER
} proc_type_t;

/* Phase-16: Execution role for boundary enforcement */
typedef enum {
    PROC_EXECUTION_ROLE_UNKNOWN = 0,
    PROC_EXECUTION_ROLE_BCIB = 1,
    PROC_EXECUTION_ROLE_RUNTIME_BRIDGE = 2,
    PROC_EXECUTION_ROLE_USER = 3,
    PROC_EXECUTION_ROLE_KERNEL = 4,
    PROC_EXECUTION_ROLE_MAX = 5  /* Sentinel for array bounds checking */
} proc_execution_role_t;

typedef enum {
    PROC_IMAGE_FLAT = 0,
    PROC_IMAGE_ELF
} proc_image_format_t;

typedef enum {
    PROC_MAPPING_CLASS_NONE = 0,
    PROC_MAPPING_CLASS_GENERIC = 1
} proc_mapping_class_t;

typedef struct proc_mapping_entry {
    uint8_t in_use;
    uint8_t reserved[7];
    uint64_t map_id;
    uint64_t owner_pid;
    uint64_t user_va;
    uint64_t phys_addr;
    uint64_t flags;
    uint64_t capability_id;
    uint64_t page_count;
    uint32_t mapping_class;
    uint32_t reserved0;
} proc_mapping_entry_t;

typedef struct proc {
    int pid;
    cpu_context_t context;
    uint64_t stack_top;
    uint64_t pml4_phys;   // her process'e özel (şimdilik kernel same map)
    proc_state_t state;
    proc_type_t type;
    proc_execution_role_t execution_role;  // Phase-16: Explicit execution role for boundary enforcement
    
    /* Boundary context cache for syscall hot-path enforcement.
     * Cached from execution_role and updated only through role transitions.
     */
    uint8_t boundary_context_type_cached;  // execution_context_type_t cached value
    uint8_t boundary_cache_valid;          // 1 if cache valid, 0 if needs update
    uint8_t reserved_patch_c1[6];          // alignment padding
    
    const char *name;
    void *wait_obj;
    uint64_t active_execution_id;
    struct proc *next;    // ready queue için
    
    // MVP-1: Scheduler bridge mailbox (Ring3 → Ring0 interaction)
    uint64_t mailbox_pa;        // Physical address of per-process mailbox
    uint64_t mailbox_last_epoch; // Last validated epoch (monotonicity check)
    
    // Deferred validation: IRQ sets flag, scheduler-safe context processes
    volatile uint8_t mailbox_validation_pending;
    uint8_t reserved_validation[7];  // alignment padding
    
    uint64_t execution_inbox_pa;
    uint64_t execution_payload_pas[AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES];
    uint64_t execution_output_mapped_id;
    uint64_t execution_delivery_seq;
    uint64_t next_mapping_id;
    proc_mapping_entry_t mapping_ledger[AYKEN_MAX_PROC_GENERIC_MAPPINGS];
    
    // Phase 11: Alias-aware address space leak proof
    alias_registry_t alias_reg;      /* alias eşleme kaydı */
    uint8_t teardown_started;        /* 0=normal, 1=teardown aktif (Freeze Invariant) */
    uint8_t reserved_phase11[7];     /* alignment padding */
    
#if defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)
    // Gate-4 isolated proof: per-process publish marker one-shot latch.
    uint8_t gate4_publish_emitted;
    // Gate-4.5 proof: emit owner ACCEPT(epoch=1) only once per process.
    uint8_t gate4_accept_epoch1_emitted;
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
proc_mapping_entry_t *proc_find_generic_mapping(proc_t *p, uint64_t user_va);
int proc_record_generic_mapping(proc_t *p,
                                uint64_t user_va,
                                uint64_t phys_addr,
                                uint64_t flags,
                                uint64_t capability_id,
                                uint64_t page_count,
                                uint64_t *out_map_id);
int proc_remove_generic_mapping(proc_t *p,
                                uint64_t user_va,
                                proc_mapping_entry_t *removed_entry);
uint32_t proc_revoke_generic_mappings(proc_t *p);
void proc_drain_deferred_reap(void);
int proc_bind_execution_output_window(proc_t *p,
                                      const uint64_t *output_pas,
                                      uint32_t frame_count,
                                      uint64_t execution_id);
void proc_unmap_execution_output_window(proc_t *p);
void proc_teardown_exit_surfaces(proc_t *p,
                                 const uint64_t *result_vas,
                                 const uint64_t *hash_vas,
                                 uint32_t result_count);
void proc_emit_low_half_kheap_runtime_proof(proc_t *p, const char *phase);

// Phase-16 Task 5: BCIB Role Provisioning - Worker Creation API
// These functions are ONLY available in validation profile builds
int bcib_worker_create(void);
uint64_t bcib_worker_get_pid(void);
proc_t *bcib_worker_get_proc(void);

int user_worker_create(void);
uint64_t user_worker_get_pid(void);
proc_t *user_worker_get_proc(void);

#endif

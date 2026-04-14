// kernel/proc/bcib_worker.c
// Phase-16 Task 5: BCIB Role Provisioning - Worker Creation Infrastructure
//
// Purpose: Create kernel-managed BCIB worker process for execution pipeline validation
// Authority: Kernel-authoritative role assignment (PROC_EXECUTION_ROLE_BCIB)
// Scope: Validation profile only (#ifdef AYKEN_VALIDATION)
//
// Constitutional Compliance:
// - SECURITY.BOUNDARY.VIOLATION: Role enforcement is NON_OVERRIDABLE
// - KERNEL.SAFETY.CRITICAL: Role assignment is kernel-authoritative
// - No backdoors, no role escalation, no enforcement bypass

#include "proc.h"
#include "embedded_elf.h"
#include "mm.h"
#include "../arch/x86_64/port_io.h"
#include "../../shared/abi/sched_mailbox_abi.h"
#include <stddef.h>
#include <stdint.h>

// External symbols for embedded BCIB worker payload (linked from userspace/minimal)
// These will be provided by the kernel embedding mechanism (similar to embedded_elf)
// For now, we'll use a placeholder approach until the embedding is set up
extern const uint8_t embedded_elf[];
extern const uint64_t embedded_elf_size;

// BCIB worker process state (kernel-managed, validation profile only)
static proc_t *g_bcib_worker_proc = NULL;
static uint64_t g_bcib_worker_pid = 0;

// Forward declarations
static void bcib_worker_emit_marker(const char *marker, const char *detail);

// ============================================================================
// Marker Emission (Audit Trail)
// ============================================================================

static void bcib_worker_emit_marker(const char *marker, const char *detail)
{
    // Use both debugcon and serial for marker emission
    const char *p = marker;
    while (*p) {
        outb(0xE9, (uint8_t)*p);
        outb(0x3F8, (uint8_t)*p);  // Serial port
        p++;
    }
    if (detail) {
        outb(0xE9, ' ');
        outb(0x3F8, ' ');
        p = detail;
        while (*p) {
            outb(0xE9, (uint8_t)*p);
            outb(0x3F8, (uint8_t)*p);
            p++;
        }
    }
    outb(0xE9, '\n');
    outb(0x3F8, '\n');
}

// ============================================================================
// BCIB Worker Creation (Kernel-Authoritative)
// ============================================================================

/**
 * bcib_worker_create - Create kernel-managed BCIB worker process
 * 
 * Creates a dedicated BCIB-role process for execution pipeline validation.
 * This function is ONLY available in validation profile builds.
 * 
 * Role Assignment:
 * - Process created with PROC_EXECUTION_ROLE_BCIB from creation
 * - No runtime role assignment or escalation
 * - Role is kernel-authoritative and immutable
 * 
 * Lifecycle:
 * - Worker created during kernel initialization (validation profile only)
 * - Worker has inbox/payload regions mapped
 * - Worker can call SYS_V2_SUBMIT_EXECUTION without enforcement violation
 * 
 * Returns:
 * - 0 on success
 * - -1 on failure (worker creation failed)
 */
int bcib_worker_create(void)
{
#if !defined(AYKEN_VALIDATION) || (AYKEN_VALIDATION != 1)
    // BCIB worker creation is ONLY available in validation profile
    bcib_worker_emit_marker("[[AYKEN_BCIB_WORKER_CREATE_FAIL]]", "code=NOT_VALIDATION_PROFILE");
    return -1;
#else
    bcib_worker_emit_marker("[[AYKEN_BCIB_WORKER_CREATE_BEGIN]]", NULL);
    
    // For Phase 1 Checkpoint A, we use the existing embedded_elf mechanism
    // Later phases will use a dedicated BCIB worker payload
    // Validate payload is present and has minimum ELF size
    if (embedded_elf_size < 64) {
        bcib_worker_emit_marker("[[AYKEN_BCIB_WORKER_CREATE_FAIL]]", "code=ELF_TOO_SMALL");
        return -1;
    }
    
    bcib_worker_emit_marker("[[AYKEN_BCIB_WORKER_PAYLOAD_OK]]", NULL);
    
    // Create user process with BCIB worker payload
    // Note: proc_create_user_process will assign default role (USER)
    // We will override this immediately after creation
    g_bcib_worker_proc = proc_create_user_process(
        "bcib-worker",
        embedded_elf,
        embedded_elf_size,
        PROC_IMAGE_ELF  // ELF binary (not flat)
    );
    
    if (!g_bcib_worker_proc) {
        bcib_worker_emit_marker("[[AYKEN_BCIB_WORKER_CREATE_FAIL]]", "code=PROC_CREATE_FAILED");
        return -1;
    }
    
    bcib_worker_emit_marker("[[AYKEN_BCIB_WORKER_PROC_CREATED]]", NULL);
    
    // Sanity check: entry point (RIP) must be non-zero
    if (g_bcib_worker_proc->context.rip == 0) {
        bcib_worker_emit_marker("[[AYKEN_BCIB_WORKER_CREATE_FAIL]]", "code=ENTRY_ZERO");
        return -1;
    }
    
    // CRITICAL: Override execution role to BCIB (kernel-authoritative)
    // This is the ONLY place where BCIB role is assigned
    // No runtime role assignment or escalation is permitted
    g_bcib_worker_proc->execution_role = PROC_EXECUTION_ROLE_BCIB;
    g_bcib_worker_pid = (uint64_t)g_bcib_worker_proc->pid;
    
    // CRITICAL: Bootstrap mailbox for first scheduler handoff
    // BCIB worker needs to be schedulable immediately without Ring3 publish
    // Advance epoch to 2 so scheduler accepts it (scheduler starts at epoch 2)
    if (g_bcib_worker_proc->mailbox_pa) {
        ayken_sched_mailbox_t *mb = (ayken_sched_mailbox_t *)paging_phys_to_virt(g_bcib_worker_proc->mailbox_pa);
        mb->epoch = 2;  // Match scheduler's initial epoch
        mb->candidate_pid = (uint32_t)g_bcib_worker_pid;
        mb->proposer_pid = (uint32_t)g_bcib_worker_pid;
        bcib_worker_emit_marker("[[AYKEN_BCIB_WORKER_MB_BOOTSTRAP]]", "epoch=2");
    }
    
    // Emit success marker with PID and role
    {
        char buf[128];
        char *p = buf;
        
        // Copy marker prefix
        const char *prefix = "[[AYKEN_BCIB_WORKER_CREATE_OK]] pid=";
        while (*prefix) *p++ = *prefix++;
        
        // Append PID (simple decimal conversion)
        uint64_t pid = g_bcib_worker_pid;
        if (pid == 0) {
            *p++ = '0';
        } else {
            char tmp[20];
            int i = 0;
            while (pid > 0) {
                tmp[i++] = '0' + (pid % 10);
                pid /= 10;
            }
            while (i > 0) {
                *p++ = tmp[--i];
            }
        }
        
        // Append role
        const char *role = " role=BCIB";
        while (*role) *p++ = *role++;
        
        *p = '\0';
        
        // Emit via debugcon
        p = buf;
        while (*p) {
            outb(0xE9, (uint8_t)*p);
            p++;
        }
        outb(0xE9, '\n');
    }
    
    return 0;
#endif
}

/**
 * bcib_worker_get_pid - Get BCIB worker PID
 * 
 * Returns the PID of the BCIB worker process for test coordination.
 * 
 * Returns:
 * - BCIB worker PID if worker exists
 * - 0 if worker not created or not in validation profile
 */
uint64_t bcib_worker_get_pid(void)
{
    return g_bcib_worker_pid;
}

/**
 * bcib_worker_get_proc - Get BCIB worker process struct
 * 
 * Returns the process struct of the BCIB worker for kernel-internal use.
 * 
 * Returns:
 * - Pointer to BCIB worker process struct if worker exists
 * - NULL if worker not created or not in validation profile
 */
proc_t *bcib_worker_get_proc(void)
{
    return g_bcib_worker_proc;
}

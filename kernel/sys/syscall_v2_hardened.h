#ifndef AYKEN_SYSCALL_V2_HARDENED_H
#define AYKEN_SYSCALL_V2_HARDENED_H

#include "syscall_v2.h"
#include "boundary_enforcement.h"

/*
 * Phase-16 Hardened Syscall Interface
 * 
 * Provides boundary-enforced syscall handling with fail-closed semantics.
 * Integrates with existing syscall_v2 infrastructure while adding strict
 * kernel boundary controls.
 */

/* Hardened syscall handler - replaces syscall_v2_handler */
uint64_t syscall_v2_hardened_handler(uint64_t syscall_num, uint64_t arg1,
                                     uint64_t arg2, uint64_t arg3, uint64_t arg4);

/* BCIB-specific validation functions */
int validate_bcib_syscall_restriction(uint64_t syscall_num, uint64_t context_id);
int validate_bridge_syscall_restriction(uint64_t syscall_num, uint64_t context_id);
int prevent_kernel_api_exposure(uint64_t syscall_num, execution_context_type_t context_type);

/* Integration macros for Phase-16 deployment */
#ifdef PHASE_16_BOUNDARY_ENFORCEMENT
#define syscall_v2_handler syscall_v2_hardened_handler
#endif

/* Constitutional compliance verification */
int verify_constitutional_compliance(void);

#endif /* AYKEN_SYSCALL_V2_HARDENED_H */
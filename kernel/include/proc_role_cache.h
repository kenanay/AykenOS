#ifndef AYKEN_PROC_ROLE_CACHE_H
#define AYKEN_PROC_ROLE_CACHE_H

#include "proc.h"
#include "../sys/boundary_enforcement.h"

static inline int proc_set_execution_role(proc_t *proc, proc_execution_role_t role)
{
    if (!proc) {
        return -1;
    }

    if ((int)role < 0 || role >= PROC_EXECUTION_ROLE_MAX) {
        proc->execution_role = PROC_EXECUTION_ROLE_UNKNOWN;
        proc->boundary_context_type_cached = (uint8_t)EXEC_CONTEXT_UNKNOWN;
        proc->boundary_cache_valid = 1;
        return -1;
    }

    proc->execution_role = role;
    proc->boundary_context_type_cached = (uint8_t)boundary_role_to_context_type(role);
    proc->boundary_cache_valid = 1;
    return 0;
}

#endif /* AYKEN_PROC_ROLE_CACHE_H */

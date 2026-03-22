#ifndef AYKEN_SYSCALL_V2_H
#define AYKEN_SYSCALL_V2_H

#include <stdint.h>
#include "capability.h"

#define SYS_V2_BASE        1000
#define SYS_V2_MAX_INDEX   11
#define SYS_V2_NR          (SYS_V2_MAX_INDEX + 1)
#define SYS_V2_LAST        (SYS_V2_BASE + SYS_V2_MAX_INDEX)

#define SYS_V2_MAP_MEMORY        0
#define SYS_V2_UNMAP_MEMORY      1
#define SYS_V2_SWITCH_CONTEXT    2
#define SYS_V2_SUBMIT_EXECUTION  3
#define SYS_V2_WAIT_RESULT       4
#define SYS_V2_INTERRUPT_RETURN  5
#define SYS_V2_TIME_QUERY        6
#define SYS_V2_CAPABILITY_BIND   7
#define SYS_V2_CAPABILITY_REVOKE 8
#define SYS_V2_EXIT              9
#define SYS_V2_DEBUG_PUTCHAR    10
#define SYS_V2_COMPLETE_EXECUTION 11

#define SYS_V2_MAX_SYSCALL      11

#define CAP_PERM_READ       0x01
#define CAP_PERM_WRITE      0x02
#define CAP_PERM_EXECUTE    0x04
#define CAP_PERM_ADMIN      0x08

#define CAP_RESOURCE_MEMORY     1
#define CAP_RESOURCE_DEVICE     2
#define CAP_RESOURCE_EXECUTION  3
#define CAP_RESOURCE_TIME       4

/*
 * sys_v2_time_query(query_type, out)
 *   TIME_QUERY_MONOTONIC -> raw monotonic PIT ticks
 *   TIME_QUERY_UPTIME    -> uptime in milliseconds derived from PIT ticks
 */
#define TIME_QUERY_MONOTONIC    0
#define TIME_QUERY_UPTIME       1

typedef struct execution_context {
    uint64_t context_id;
    uint64_t process_id;
    void *memory_base;
    uint64_t memory_size;
    capability_token_t *capabilities;
    uint32_t capability_count;
    uint64_t creation_time;
    uint32_t status;
} execution_context_t;

#define EXEC_STATUS_CREATED     0x01
#define EXEC_STATUS_RUNNING     0x02
#define EXEC_STATUS_WAITING     0x04
#define EXEC_STATUS_COMPLETED   0x08
#define EXEC_STATUS_ERROR       0x10

#define EXEC_COMPLETION_COMPLETED 0
#define EXEC_COMPLETION_FAILED    1

uint64_t sys_v2_map_memory(uint64_t virt_addr, uint64_t phys_addr, uint64_t flags);
uint64_t sys_v2_unmap_memory(uint64_t virt_addr, uint64_t size);
uint64_t sys_v2_switch_context(uint64_t old_ctx_id, uint64_t new_ctx_id);
uint64_t sys_v2_submit_execution(void *bcib_graph, uint64_t graph_size, uint64_t context_id);
uint64_t sys_v2_wait_result(uint64_t execution_id, uint64_t timeout_ms);
uint64_t sys_v2_interrupt_return(uint64_t interrupt_id, uint64_t result_code);
uint64_t sys_v2_time_query(uint64_t query_type, uint64_t *result_buffer);
uint64_t sys_v2_capability_bind(uint64_t execution_ctx_id, capability_token_t *token);
uint64_t sys_v2_capability_revoke(uint64_t token_id);
uint64_t sys_v2_exit(uint64_t exit_code);
uint64_t sys_v2_debug_putchar(uint64_t character);
uint64_t sys_v2_complete_execution(uint64_t execution_id, uint64_t completion_code);

uint64_t syscall_v2_handler(uint64_t syscall_num, uint64_t arg1,
                            uint64_t arg2, uint64_t arg3, uint64_t arg4);

#define ESYS_V2_SUCCESS         0
#define ESYS_V2_INVALID_SYSCALL -1
#define ESYS_V2_INVALID_PARAM   -2
#define ESYS_V2_NO_PERMISSION   -3
#define ESYS_V2_NO_MEMORY       -4
#define ESYS_V2_NO_CAPABILITY   -5
#define ESYS_V2_TIMEOUT         -6
#define ESYS_V2_CONTEXT_ERROR   -7
#define ESYS_V2_RESOURCE_BUSY   -8
#define ESYS_V2_NOT_IMPLEMENTED -9
#define ESYS_V2_INVALID_STATE   -10
#define ESYS_V2_INVALID_ID      -11
#define ESYS_V2_PERMISSION_DENIED ESYS_V2_NO_PERMISSION

#endif

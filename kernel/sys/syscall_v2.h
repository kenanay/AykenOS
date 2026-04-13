#ifndef AYKEN_SYSCALL_V2_WRAPPER_H
#define AYKEN_SYSCALL_V2_WRAPPER_H

#include "../../shared/abi/syscall_v2.h"

/*
 * Keep the legacy syscall-v2 contract macros visible in this wrapper so CI
 * gates that parse kernel/sys/syscall_v2.h directly do not need include
 * expansion to validate the frozen ABI contract.
 */
#ifndef SYS_V2_BASE
#define SYS_V2_BASE        1000
#define SYS_V2_MAX_INDEX   14
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
#define SYS_V2_DEVICE_OPERATION  12
#define SYS_V2_EXTERNAL_CALL     13
#define SYS_V2_ABDF_OPERATION    14

#define SYS_V2_MAX_SYSCALL      14
#endif

#endif

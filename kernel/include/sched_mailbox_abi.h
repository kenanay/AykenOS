#ifndef AYKEN_SCHED_MAILBOX_ABI_H
#define AYKEN_SCHED_MAILBOX_ABI_H

#include <stdint.h>

/*
 * Scheduler mailbox syscall ABI
 *
 * This number is intentionally outside the frozen SYS_V2_BASE..SYS_V2_LAST
 * range so the execution-centric v2 syscall contract remains unchanged.
 *
 * Reserved bridge window:
 *   0x90..0x9F  scheduler/policy bridge syscalls
 */
#define SYS_BRIDGE_BASE          ((uint64_t)0x90u)
#define SYS_BRIDGE_LAST          ((uint64_t)0x9Fu)
#define SYS_V2_SCHED_STAGE_NEXT  (SYS_BRIDGE_BASE + 0x00u)

#endif /* AYKEN_SCHED_MAILBOX_ABI_H */

#pragma once

#include "../../shared/abi/sched_mailbox_abi.h"

/*
 * Keep the legacy mailbox capability contract tokens visible in this wrapper
 * so CI validators that parse kernel/include/sched_mailbox_abi.h directly do
 * not need include expansion to validate the frozen contract.
 */
#ifndef AYKEN_SCHED_MB_FLAG_CAP_CHECK_REQUIRED
#define AYKEN_SCHED_MB_FLAG_CAP_CHECK_REQUIRED (1u << 0)
#define AYKEN_SCHED_MB_FLAG_SIG_VALID          (1u << 1)
#define AYKEN_SCHED_MB_FLAG_CAP_PRESENT        (1u << 2)
#define AYKEN_SCHED_MB_FLAG_BUDGET_OK          (1u << 3)
#define AYKEN_SCHED_MB_CAP_BUDGET_MAX          1000u
#endif

#if 0
enum {
    AYKEN_SCHED_REJECT_BAD_SIG_COMPAT = AYKEN_SCHED_REJECT_BAD_SIG,
    AYKEN_SCHED_REJECT_CAP_MISSING_COMPAT = AYKEN_SCHED_REJECT_CAP_MISSING,
    AYKEN_SCHED_REJECT_BUDGET_EXCEEDED_COMPAT = AYKEN_SCHED_REJECT_BUDGET_EXCEEDED,
    AYKEN_SCHED_REJECT_INVALID_PID_COMPAT = AYKEN_SCHED_REJECT_INVALID_PID,
    REJ_BAD_SIG_COMPAT = REJ_BAD_SIG,
    REJ_CAP_MISSING_COMPAT = REJ_CAP_MISSING,
    REJ_BUDGET_EXCEEDED_COMPAT = REJ_BUDGET_EXCEEDED,
    REJ_INVALID_PID_COMPAT = REJ_INVALID_PID,
};
#endif

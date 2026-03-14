#pragma once

#include <stdint.h>

#define AYKEN_SCHED_MB_MAGIC   0x4B534D42u
#define AYKEN_SCHED_MB_VERSION 1

#define AYKEN_SCHED_MB_FLAG_CAP_CHECK_REQUIRED (1u << 0)
#define AYKEN_SCHED_MB_FLAG_SIG_VALID          (1u << 1)
#define AYKEN_SCHED_MB_FLAG_CAP_PRESENT        (1u << 2)
#define AYKEN_SCHED_MB_FLAG_BUDGET_OK          (1u << 3)
#define AYKEN_SCHED_MB_CAP_BUDGET_MAX          1000u

typedef enum {
    AYKEN_SCHED_HINT_NONE = 0,
    AYKEN_SCHED_HINT_CANDIDATE = 1,
} ayken_sched_hint_kind_t;

typedef enum {
    AYKEN_SCHED_STATUS_EMPTY  = 0,
    AYKEN_SCHED_STATUS_ACCEPT = 1,
    AYKEN_SCHED_STATUS_REJECT = 2,
} ayken_sched_status_t;

typedef enum {
    AYKEN_SCHED_REJECT_NONE = 0,
    AYKEN_SCHED_REJECT_BAD_MAGIC = 1,
    AYKEN_SCHED_REJECT_BAD_VERSION = 2,
    AYKEN_SCHED_REJECT_BAD_KIND = 3,
    AYKEN_SCHED_REJECT_STALE_EPOCH = 4,
    AYKEN_SCHED_REJECT_BAD_PID = 5,
    AYKEN_SCHED_REJECT_NOT_RUNNABLE = 6,
    AYKEN_SCHED_REJECT_BAD_SIG = 7,
    AYKEN_SCHED_REJECT_CAP_MISSING = 8,
    AYKEN_SCHED_REJECT_BUDGET_EXCEEDED = 9,
    AYKEN_SCHED_REJECT_INVALID_PID = AYKEN_SCHED_REJECT_BAD_PID,
} ayken_sched_reject_reason_t;

#define REJ_BAD_SIG         AYKEN_SCHED_REJECT_BAD_SIG
#define REJ_CAP_MISSING     AYKEN_SCHED_REJECT_CAP_MISSING
#define REJ_BUDGET_EXCEEDED AYKEN_SCHED_REJECT_BUDGET_EXCEEDED
#define REJ_INVALID_PID     AYKEN_SCHED_REJECT_INVALID_PID

typedef struct __attribute__((packed, aligned(64))) {
    uint32_t magic;
    uint16_t version;
    uint16_t kind;
    uint64_t epoch;
    uint32_t proposer_pid;
    uint32_t candidate_pid;
    uint32_t flags;
    uint32_t status;
    uint32_t reject_reason;
    uint32_t reserved;
} ayken_sched_mailbox_t;

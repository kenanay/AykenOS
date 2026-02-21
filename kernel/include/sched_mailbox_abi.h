// kernel/include/sched_mailbox_abi.h
// Scheduler Bridge Mailbox ABI
// Ring3 → Ring0 scheduler candidate proposal mechanism
//
// Constitutional requirement: Ring0 mechanism only, Ring3 policy
// This ABI defines the communication surface between Ring3 scheduler policy
// and Ring0 scheduler mechanism.
//
// Author: Kenan AY
// Project: AykenOS - Advanced AI-Integrated Operating System
// Phase: 4.5 - Scheduler Bridge MVP

#pragma once

#include <stdint.h>

#define AYKEN_SCHED_MB_MAGIC   0x4B534D42u  /* 'KSMB' */
#define AYKEN_SCHED_MB_VERSION 1

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
} ayken_sched_reject_reason_t;

typedef struct __attribute__((packed, aligned(64))) {
    uint32_t magic;
    uint16_t version;
    uint16_t kind;            /* ayken_sched_hint_kind_t */
    uint64_t epoch;
    uint32_t proposer_pid;
    uint32_t candidate_pid;
    uint32_t flags;
    uint32_t status;          /* ayken_sched_status_t */
    uint32_t reject_reason;   /* ayken_sched_reject_reason_t */
    uint32_t reserved;
} ayken_sched_mailbox_t;

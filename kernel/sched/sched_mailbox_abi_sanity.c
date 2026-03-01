// kernel/sched/sched_mailbox_abi_sanity.c
// Compile-time freeze guards for scheduler mailbox ABI v1.

#include <stddef.h>
#include "../include/sched_mailbox_abi.h"

_Static_assert(AYKEN_SCHED_MB_MAGIC == 0x4B534D42u,
               "mailbox ABI: magic mismatch");
_Static_assert(AYKEN_SCHED_MB_VERSION == 1,
               "mailbox ABI: version mismatch");

_Static_assert(sizeof(ayken_sched_mailbox_t) == 64,
               "mailbox ABI: size must remain 64 bytes");
_Static_assert(_Alignof(ayken_sched_mailbox_t) == 64,
               "mailbox ABI: alignment must remain 64 bytes");

_Static_assert(offsetof(ayken_sched_mailbox_t, magic) == 0,
               "mailbox ABI: magic offset drift");
_Static_assert(offsetof(ayken_sched_mailbox_t, version) == 4,
               "mailbox ABI: version offset drift");
_Static_assert(offsetof(ayken_sched_mailbox_t, kind) == 6,
               "mailbox ABI: kind offset drift");
_Static_assert(offsetof(ayken_sched_mailbox_t, epoch) == 8,
               "mailbox ABI: epoch offset drift");
_Static_assert(offsetof(ayken_sched_mailbox_t, proposer_pid) == 16,
               "mailbox ABI: proposer_pid offset drift");
_Static_assert(offsetof(ayken_sched_mailbox_t, candidate_pid) == 20,
               "mailbox ABI: candidate_pid offset drift");
_Static_assert(offsetof(ayken_sched_mailbox_t, flags) == 24,
               "mailbox ABI: flags offset drift");
_Static_assert(offsetof(ayken_sched_mailbox_t, status) == 28,
               "mailbox ABI: status offset drift");
_Static_assert(offsetof(ayken_sched_mailbox_t, reject_reason) == 32,
               "mailbox ABI: reject_reason offset drift");
_Static_assert(offsetof(ayken_sched_mailbox_t, reserved) == 36,
               "mailbox ABI: reserved offset drift");
_Static_assert(sizeof(((ayken_sched_mailbox_t *)0)->reserved) == 4,
               "mailbox ABI: reserved field size drift");
_Static_assert((sizeof(ayken_sched_mailbox_t) -
                offsetof(ayken_sched_mailbox_t, reserved)) == (64 - 36),
               "mailbox ABI: reserved tail span drift");

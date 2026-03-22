#ifndef AYKEN_SHARED_EXECUTION_INBOX_ABI_H
#define AYKEN_SHARED_EXECUTION_INBOX_ABI_H

#include <stdint.h>

#ifndef AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES
#define AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES 4u
#define AYKEN_EXECUTION_PAYLOAD_WINDOW_SIZE 0x4000ULL
#endif

#define EXECUTION_INBOX_VA   0x701000ULL
#define EXECUTION_PAYLOAD_VA 0x702000ULL

#define AYKEN_EXECUTION_INBOX_MAGIC   0x42495841u /* 'AXIB' */
#define AYKEN_EXECUTION_INBOX_VERSION 1u

#define AXIB_STATE_EMPTY 0u
#define AXIB_STATE_READY 1u

typedef struct ayken_execution_inbox_v1 {
    uint32_t magic;
    uint16_t version;
    uint16_t state;
    uint64_t delivery_seq;
    uint64_t execution_id;
    uint64_t target_context_id;
    uint64_t bcib_user_va;
    uint64_t bcib_size;
    uint64_t bcib_window_size;
    uint64_t flags;
    uint64_t reserved[6];
} ayken_execution_inbox_v1_t;

#endif

#ifndef AYKEN_SHARED_EXECUTION_OUTPUT_ABI_H
#define AYKEN_SHARED_EXECUTION_OUTPUT_ABI_H

#include <stdint.h>

#include "execution_inbox_abi.h"

#define EXECUTION_OUTPUT_VA 0x900000ULL

#define AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES
#define AYKEN_EXECUTION_OUTPUT_WINDOW_SIZE  AYKEN_EXECUTION_PAYLOAD_WINDOW_SIZE

#define AYKEN_EXECUTION_OUTPUT_MAGIC   0x54554F41u /* 'AOUT' */
#define AYKEN_EXECUTION_OUTPUT_VERSION 1u

typedef struct ayken_execution_output_v1 {
    uint32_t magic;
    uint32_t abi_version;
    uint32_t flags;
    uint32_t reserved0;
    uint64_t bytes_written;
    uint64_t reserved[3];
} ayken_execution_output_v1_t;

#endif

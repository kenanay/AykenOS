#ifndef AYKEN_SHARED_EXECUTION_OUTPUT_STRUCTURED_ABI_H
#define AYKEN_SHARED_EXECUTION_OUTPUT_STRUCTURED_ABI_H

#include <stdint.h>

#define AYKEN_EXECUTION_OUTPUT_V2_MAGIC   0x32554F41u /* 'AOU2' */
#define AYKEN_EXECUTION_OUTPUT_V2_VERSION 2u

#define AYKEN_OUTPUT_KIND_RAW  0u
#define AYKEN_OUTPUT_KIND_BLOB 1u

typedef struct ayken_execution_output_v2 {
    uint32_t magic;
    uint32_t abi_version;
    uint32_t kind;
    uint32_t flags;
    uint64_t bytes_written;
    uint64_t reserved[3];
} ayken_execution_output_v2_t;

#endif

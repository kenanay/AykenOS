#ifndef AYKEN_SHARED_EXECUTION_RESULT_HASH_ABI_H
#define AYKEN_SHARED_EXECUTION_RESULT_HASH_ABI_H

#include <stdint.h>

#define AYKEN_EXECUTION_RESULT_HASH_BASE_VA 0xA00000ULL
#define AYKEN_EXECUTION_RESULT_HASH_WINDOW_SIZE 0x1000ULL

#define AYKEN_EXECUTION_RESULT_HASH_MAGIC   0x48534541u /* 'AESH' */
#define AYKEN_EXECUTION_RESULT_HASH_VERSION 1u

#define AYKEN_RESULT_HASH_ALG_SHA256 1u
#define AYKEN_RESULT_HASH_DIGEST_SIZE_SHA256 32u

typedef struct ayken_execution_result_hash_v1 {
    uint32_t magic;
    uint32_t abi_version;
    uint32_t algorithm;
    uint32_t flags;
    uint64_t hashed_size;
    uint8_t digest[AYKEN_RESULT_HASH_DIGEST_SIZE_SHA256];
    uint8_t reserved[16];
} ayken_execution_result_hash_v1_t;

#endif

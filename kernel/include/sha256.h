#ifndef AYKEN_SHA256_H
#define AYKEN_SHA256_H

#include <stdint.h>

#define AYKEN_SHA256_DIGEST_SIZE 32u

typedef struct ayken_sha256_ctx {
    uint32_t state[8];
    uint64_t total_len;
    uint8_t block[64];
    uint32_t block_used;
} ayken_sha256_ctx_t;

void ayken_sha256_init(ayken_sha256_ctx_t *ctx);
void ayken_sha256_update(ayken_sha256_ctx_t *ctx, const void *data, uint64_t len);
void ayken_sha256_final(ayken_sha256_ctx_t *ctx, uint8_t out[AYKEN_SHA256_DIGEST_SIZE]);
void ayken_sha256_compute(const void *data,
                          uint64_t len,
                          uint8_t out[AYKEN_SHA256_DIGEST_SIZE]);

#endif

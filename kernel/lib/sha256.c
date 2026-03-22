#include <stdint.h>

#include "../include/sha256.h"

static uint32_t ayken_sha256_rotr32(uint32_t value, uint32_t count)
{
    return (value >> count) | (value << (32u - count));
}

static uint32_t ayken_sha256_ch(uint32_t x, uint32_t y, uint32_t z)
{
    return (x & y) ^ (~x & z);
}

static uint32_t ayken_sha256_maj(uint32_t x, uint32_t y, uint32_t z)
{
    return (x & y) ^ (x & z) ^ (y & z);
}

static uint32_t ayken_sha256_big_sigma0(uint32_t x)
{
    return ayken_sha256_rotr32(x, 2u) ^
           ayken_sha256_rotr32(x, 13u) ^
           ayken_sha256_rotr32(x, 22u);
}

static uint32_t ayken_sha256_big_sigma1(uint32_t x)
{
    return ayken_sha256_rotr32(x, 6u) ^
           ayken_sha256_rotr32(x, 11u) ^
           ayken_sha256_rotr32(x, 25u);
}

static uint32_t ayken_sha256_small_sigma0(uint32_t x)
{
    return ayken_sha256_rotr32(x, 7u) ^
           ayken_sha256_rotr32(x, 18u) ^
           (x >> 3u);
}

static uint32_t ayken_sha256_small_sigma1(uint32_t x)
{
    return ayken_sha256_rotr32(x, 17u) ^
           ayken_sha256_rotr32(x, 19u) ^
           (x >> 10u);
}

static uint32_t ayken_sha256_load_be32(const uint8_t *src)
{
    return ((uint32_t)src[0] << 24u) |
           ((uint32_t)src[1] << 16u) |
           ((uint32_t)src[2] << 8u) |
           (uint32_t)src[3];
}

static void ayken_sha256_store_be32(uint8_t *dst, uint32_t value)
{
    dst[0] = (uint8_t)(value >> 24u);
    dst[1] = (uint8_t)(value >> 16u);
    dst[2] = (uint8_t)(value >> 8u);
    dst[3] = (uint8_t)value;
}

static const uint32_t k_ayken_sha256[64] = {
    0x428a2f98u, 0x71374491u, 0xb5c0fbcfu, 0xe9b5dba5u,
    0x3956c25bu, 0x59f111f1u, 0x923f82a4u, 0xab1c5ed5u,
    0xd807aa98u, 0x12835b01u, 0x243185beu, 0x550c7dc3u,
    0x72be5d74u, 0x80deb1feu, 0x9bdc06a7u, 0xc19bf174u,
    0xe49b69c1u, 0xefbe4786u, 0x0fc19dc6u, 0x240ca1ccu,
    0x2de92c6fu, 0x4a7484aau, 0x5cb0a9dcu, 0x76f988dau,
    0x983e5152u, 0xa831c66du, 0xb00327c8u, 0xbf597fc7u,
    0xc6e00bf3u, 0xd5a79147u, 0x06ca6351u, 0x14292967u,
    0x27b70a85u, 0x2e1b2138u, 0x4d2c6dfcu, 0x53380d13u,
    0x650a7354u, 0x766a0abbu, 0x81c2c92eu, 0x92722c85u,
    0xa2bfe8a1u, 0xa81a664bu, 0xc24b8b70u, 0xc76c51a3u,
    0xd192e819u, 0xd6990624u, 0xf40e3585u, 0x106aa070u,
    0x19a4c116u, 0x1e376c08u, 0x2748774cu, 0x34b0bcb5u,
    0x391c0cb3u, 0x4ed8aa4au, 0x5b9cca4fu, 0x682e6ff3u,
    0x748f82eeu, 0x78a5636fu, 0x84c87814u, 0x8cc70208u,
    0x90befffau, 0xa4506cebu, 0xbef9a3f7u, 0xc67178f2u,
};

static void ayken_sha256_process_block(uint32_t state[8], const uint8_t block[64])
{
    uint32_t w[64];
    uint32_t i;
    uint32_t a;
    uint32_t b;
    uint32_t c;
    uint32_t d;
    uint32_t e;
    uint32_t f;
    uint32_t g;
    uint32_t h;

    for (i = 0; i < 16u; ++i) {
        w[i] = ayken_sha256_load_be32(block + (i * 4u));
    }
    for (i = 16u; i < 64u; ++i) {
        w[i] = ayken_sha256_small_sigma1(w[i - 2u]) + w[i - 7u] +
               ayken_sha256_small_sigma0(w[i - 15u]) + w[i - 16u];
    }

    a = state[0];
    b = state[1];
    c = state[2];
    d = state[3];
    e = state[4];
    f = state[5];
    g = state[6];
    h = state[7];

    for (i = 0; i < 64u; ++i) {
        uint32_t t1 = h + ayken_sha256_big_sigma1(e) +
                      ayken_sha256_ch(e, f, g) +
                      k_ayken_sha256[i] + w[i];
        uint32_t t2 = ayken_sha256_big_sigma0(a) +
                      ayken_sha256_maj(a, b, c);
        h = g;
        g = f;
        f = e;
        e = d + t1;
        d = c;
        c = b;
        b = a;
        a = t1 + t2;
    }

    state[0] += a;
    state[1] += b;
    state[2] += c;
    state[3] += d;
    state[4] += e;
    state[5] += f;
    state[6] += g;
    state[7] += h;
}

void ayken_sha256_init(ayken_sha256_ctx_t *ctx)
{
    if (!ctx) {
        return;
    }

    ctx->state[0] = 0x6a09e667u;
    ctx->state[1] = 0xbb67ae85u;
    ctx->state[2] = 0x3c6ef372u;
    ctx->state[3] = 0xa54ff53au;
    ctx->state[4] = 0x510e527fu;
    ctx->state[5] = 0x9b05688cu;
    ctx->state[6] = 0x1f83d9abu;
    ctx->state[7] = 0x5be0cd19u;
    ctx->total_len = 0;
    ctx->block_used = 0;
}

void ayken_sha256_update(ayken_sha256_ctx_t *ctx, const void *data, uint64_t len)
{
    const uint8_t *src = (const uint8_t *)data;

    if (!ctx || (!src && len != 0)) {
        return;
    }

    ctx->total_len += len;
    while (len > 0) {
        uint32_t copy_len = 64u - ctx->block_used;

        if (copy_len > len) {
            copy_len = (uint32_t)len;
        }

        for (uint32_t i = 0; i < copy_len; ++i) {
            ctx->block[ctx->block_used + i] = src[i];
        }
        ctx->block_used += copy_len;
        src += copy_len;
        len -= copy_len;

        if (ctx->block_used == 64u) {
            ayken_sha256_process_block(ctx->state, ctx->block);
            ctx->block_used = 0;
        }
    }
}

void ayken_sha256_final(ayken_sha256_ctx_t *ctx, uint8_t out[AYKEN_SHA256_DIGEST_SIZE])
{
    uint64_t bit_len;
    uint32_t i;

    if (!ctx || !out) {
        return;
    }

    ctx->block[ctx->block_used++] = 0x80u;
    if (ctx->block_used > 56u) {
        while (ctx->block_used < 64u) {
            ctx->block[ctx->block_used++] = 0u;
        }
        ayken_sha256_process_block(ctx->state, ctx->block);
        ctx->block_used = 0;
    }

    while (ctx->block_used < 56u) {
        ctx->block[ctx->block_used++] = 0u;
    }

    bit_len = ctx->total_len * 8u;
    for (i = 0; i < 8u; ++i) {
        ctx->block[56u + i] = (uint8_t)(bit_len >> ((7u - i) * 8u));
    }
    ayken_sha256_process_block(ctx->state, ctx->block);

    for (i = 0; i < 8u; ++i) {
        ayken_sha256_store_be32(out + (i * 4u), ctx->state[i]);
    }
}

void ayken_sha256_compute(const void *data,
                          uint64_t len,
                          uint8_t out[AYKEN_SHA256_DIGEST_SIZE])
{
    ayken_sha256_ctx_t ctx;

    ayken_sha256_init(&ctx);
    ayken_sha256_update(&ctx, data, len);
    ayken_sha256_final(&ctx, out);
}

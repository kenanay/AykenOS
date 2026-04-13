// SPDX-License-Identifier: ASAL-1.0
// Copyright (C) 2026 Kenan AY
//
// Ring3 Process Preparation (Scheduler Dispatch Path)
// Authority: Phase 10-A2
// Constitutional: Ring0 mechanism only (no policy)

#include <stdint.h>
#include "arch/x86_64/port_io.h"
#include "drivers/console/fb_console.h"
#include "embedded_elf.h"
#include "ring3_jump.h"
#include "gdt_idt.h"
#include "include/proc.h"
#include "include/mm.h"
#include "sched/sched_mailbox.h"

#ifndef AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST
#define AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST 0
#endif

#ifndef AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_SELFTEST
#define AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_SELFTEST 0
#endif

#ifndef AYKEN_LOW_HALF_KHEAP_INTERLEAVING_PROOF_SELFTEST
#define AYKEN_LOW_HALF_KHEAP_INTERLEAVING_PROOF_SELFTEST 0
#endif

#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    ((AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST == 1) || \
     (AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_SELFTEST == 1) || \
     (AYKEN_LOW_HALF_KHEAP_INTERLEAVING_PROOF_SELFTEST == 1))
static const uint8_t ring3_exit_proof_owner_code[] = {
    0xF3, 0x90, /* pause */
    0xEB, 0xFC  /* jmp .-2 */
};

static int ring3_seed_mailbox_candidate(proc_t *publisher,
                                        uint64_t epoch,
                                        uint32_t candidate_pid)
{
    ayken_sched_mailbox_t *mb;

    if (!publisher || publisher->mailbox_pa == 0 || publisher->pid <= 0) {
        return 0;
    }

    mb = (ayken_sched_mailbox_t *)paging_phys_to_virt(publisher->mailbox_pa);
    if (!mb) {
        return 0;
    }

    mb->magic = AYKEN_SCHED_MB_MAGIC;
    mb->version = AYKEN_SCHED_MB_VERSION;
    mb->kind = AYKEN_SCHED_HINT_CANDIDATE;
    mb->epoch = epoch;
    mb->proposer_pid = (uint32_t)publisher->pid;
    mb->candidate_pid = candidate_pid;
    mb->flags = 0;
    mb->status = AYKEN_SCHED_STATUS_EMPTY;
    mb->reject_reason = AYKEN_SCHED_REJECT_NONE;
    mb->reserved = 0;
    return 1;
}
#endif

static uint32_t rotr32(uint32_t value, uint32_t count)
{
    return (value >> count) | (value << (32u - count));
}

static uint32_t sha256_ch(uint32_t x, uint32_t y, uint32_t z)
{
    return (x & y) ^ (~x & z);
}

static uint32_t sha256_maj(uint32_t x, uint32_t y, uint32_t z)
{
    return (x & y) ^ (x & z) ^ (y & z);
}

static uint32_t sha256_big_sigma0(uint32_t x)
{
    return rotr32(x, 2u) ^ rotr32(x, 13u) ^ rotr32(x, 22u);
}

static uint32_t sha256_big_sigma1(uint32_t x)
{
    return rotr32(x, 6u) ^ rotr32(x, 11u) ^ rotr32(x, 25u);
}

static uint32_t sha256_small_sigma0(uint32_t x)
{
    return rotr32(x, 7u) ^ rotr32(x, 18u) ^ (x >> 3u);
}

static uint32_t sha256_small_sigma1(uint32_t x)
{
    return rotr32(x, 17u) ^ rotr32(x, 19u) ^ (x >> 10u);
}

static uint32_t load_be32(const uint8_t *src)
{
    return ((uint32_t)src[0] << 24u) |
           ((uint32_t)src[1] << 16u) |
           ((uint32_t)src[2] << 8u)  |
           (uint32_t)src[3];
}

static void store_be32(uint8_t *dst, uint32_t value)
{
    dst[0] = (uint8_t)(value >> 24u);
    dst[1] = (uint8_t)(value >> 16u);
    dst[2] = (uint8_t)(value >> 8u);
    dst[3] = (uint8_t)value;
}

static const uint32_t k_sha256[64] = {
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

static void sha256_process_block(uint32_t state[8], const uint8_t block[64])
{
    uint32_t w[64];
    for (uint32_t i = 0; i < 16u; ++i) {
        w[i] = load_be32(block + (i * 4u));
    }
    for (uint32_t i = 16u; i < 64u; ++i) {
        w[i] = sha256_small_sigma1(w[i - 2u]) + w[i - 7u] +
               sha256_small_sigma0(w[i - 15u]) + w[i - 16u];
    }

    uint32_t a = state[0];
    uint32_t b = state[1];
    uint32_t c = state[2];
    uint32_t d = state[3];
    uint32_t e = state[4];
    uint32_t f = state[5];
    uint32_t g = state[6];
    uint32_t h = state[7];

    for (uint32_t i = 0; i < 64u; ++i) {
        uint32_t t1 = h + sha256_big_sigma1(e) + sha256_ch(e, f, g) + k_sha256[i] + w[i];
        uint32_t t2 = sha256_big_sigma0(a) + sha256_maj(a, b, c);
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

static void sha256_compute(const uint8_t *data, uint64_t len, uint8_t out[32])
{
    uint32_t state[8] = {
        0x6a09e667u, 0xbb67ae85u, 0x3c6ef372u, 0xa54ff53au,
        0x510e527fu, 0x9b05688cu, 0x1f83d9abu, 0x5be0cd19u,
    };
    uint64_t full_blocks = len / 64u;
    for (uint64_t i = 0; i < full_blocks; ++i) {
        sha256_process_block(state, data + (i * 64u));
    }

    uint8_t tail[128];
    uint64_t rem = len % 64u;
    uint64_t tail_len = 0;
    uint64_t tail_base = full_blocks * 64u;

    for (uint64_t i = 0; i < rem; ++i) {
        tail[i] = data[tail_base + i];
    }
    tail_len = rem;
    tail[tail_len++] = 0x80u;

    uint64_t zeros = (tail_len <= 56u) ? (56u - tail_len) : (120u - tail_len);
    for (uint64_t i = 0; i < zeros; ++i) {
        tail[tail_len + i] = 0u;
    }
    tail_len += zeros;

    uint64_t bit_len = len * 8u;
    for (uint32_t i = 0; i < 8u; ++i) {
        tail[tail_len + i] = (uint8_t)(bit_len >> ((7u - i) * 8u));
    }
    tail_len += 8u;

    for (uint64_t i = 0; i < tail_len; i += 64u) {
        sha256_process_block(state, tail + i);
    }

    for (uint32_t i = 0; i < 8u; ++i) {
        store_be32(out + (i * 4u), state[i]);
    }
}

static int hex_nibble(char c)
{
    if (c >= '0' && c <= '9') {
        return c - '0';
    }
    if (c >= 'a' && c <= 'f') {
        return c - 'a' + 10;
    }
    if (c >= 'A' && c <= 'F') {
        return c - 'A' + 10;
    }
    return -1;
}

static int parse_sha256_hex(const char *hex, uint8_t out[32])
{
    if (!hex) {
        return -1;
    }
    for (uint32_t i = 0; i < 32u; ++i) {
        int hi = hex_nibble(hex[i * 2u]);
        int lo = hex_nibble(hex[i * 2u + 1u]);
        if (hi < 0 || lo < 0) {
            return -1;
        }
        out[i] = (uint8_t)((hi << 4) | lo);
    }
    if (hex[64] != '\0') {
        return -1;
    }
    return 0;
}

static int sha256_equal(const uint8_t lhs[32], const uint8_t rhs[32])
{
    uint8_t diff = 0u;
    for (uint32_t i = 0; i < 32u; ++i) {
        diff |= (uint8_t)(lhs[i] ^ rhs[i]);
    }
    return diff == 0u;
}

static void debugcon_write(const char *s)
{
    if (!s) {
        return;
    }
    while (*s) {
        outb(0xE9, (uint8_t)*s);
        s++;
    }
}

static void halt_forever(void)
{
    for (;;) {
        __asm__ volatile("cli; hlt");
    }
}

static void ring3_prep_panic(const char *marker, const char *msg)
{
    if (marker) {
        debugcon_write(marker);
        debugcon_write("\n");
    }
    if (msg) {
        fb_print(msg);
        fb_print("\n");
        debugcon_write(msg);
        debugcon_write("\n");
    }
    halt_forever();
}

void jump_to_ring3(void)
{
    const uint64_t min_elf64_ehdr_size = 64;
    uint8_t expected_hash[32];
    uint8_t computed_hash[32];

    debugcon_write("[K][PHASE10] KERNEL_BEFORE_RING3\n");
    fb_print("[PHASE10] Preparing Ring3 process for scheduler dispatch...\n");

    if (parse_sha256_hex(embedded_elf_sha256, expected_hash) != 0) {
        ring3_prep_panic("[[AYKEN_RING3_PREP_FAIL]] bad_hash_hex",
                         "[PANIC] Phase10: Embedded ELF SHA256 literal is invalid.");
    }
    sha256_compute(embedded_elf, (uint64_t)embedded_elf_size, computed_hash);
    if (!sha256_equal(expected_hash, computed_hash)) {
        ring3_prep_panic("[[AYKEN_RING3_PREP_FAIL]] hash_mismatch",
                         "[PANIC] Phase10: Embedded ELF hash verification failed.");
    }
    debugcon_write("P10_EMBED_HASH_OK\n");

    if (embedded_elf_size < min_elf64_ehdr_size) {
        ring3_prep_panic("[[AYKEN_RING3_PREP_FAIL]] no_elf",
                         "[PANIC] Phase10: Embedded ELF is missing or truncated.");
    }
    if (embedded_elf[0] != 0x7F || embedded_elf[1] != 'E' ||
        embedded_elf[2] != 'L' || embedded_elf[3] != 'F') {
        ring3_prep_panic("[[AYKEN_RING3_PREP_FAIL]] bad_magic",
                         "[PANIC] Phase10: Embedded ELF magic is invalid.");
    }

#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    ((AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST == 1) || \
     (AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_SELFTEST == 1) || \
     (AYKEN_LOW_HALF_KHEAP_INTERLEAVING_PROOF_SELFTEST == 1))
    {
        proc_t *owner_proc = proc_create_user_process(
            "phase10-exit-proof-owner",
            ring3_exit_proof_owner_code,
            (uint64_t)sizeof(ring3_exit_proof_owner_code),
            PROC_IMAGE_FLAT
        );
        if (!owner_proc) {
            ring3_prep_panic("[[AYKEN_RING3_PREP_FAIL]] exit_owner_create",
                             "[PANIC] Phase10: Exit-proof owner process creation failed.");
        }
        if (!ring3_seed_mailbox_candidate(owner_proc, 2, 1)) {
            ring3_prep_panic("[[AYKEN_RING3_PREP_FAIL]] exit_owner_seed",
                             "[PANIC] Phase10: Exit-proof owner mailbox seed failed.");
        }
        debugcon_write("[[AYKEN_RING3_PREP_OK]]\n");
        debugcon_write("P10_SCHED_ARMED\n");
        fb_print("[PHASE10] Exit-proof owner process prepared and queued.\n");
        return;
    }
#endif

    // [K][USER_ELF_SELECTED] - Entry marker: ELF selection point
    debugcon_write("[K][USER_ELF_SELECTED] name=phase10-minimal type=ELF size=");
    {
        char buf[20];
        uint64_t sz = (uint64_t)embedded_elf_size;
        int i = 0;
        if (sz == 0) {
            buf[i++] = '0';
        } else {
            char tmp[20];
            int j = 0;
            while (sz > 0) {
                tmp[j++] = '0' + (sz % 10);
                sz /= 10;
            }
            while (j > 0) {
                buf[i++] = tmp[--j];
            }
        }
        buf[i] = '\0';
        debugcon_write(buf);
    }
    debugcon_write(" ptr=");
    {
        uint64_t ptr = (uint64_t)embedded_elf;
        char hex[17];
        const char *digits = "0123456789abcdef";
        for (int i = 0; i < 16; i++) {
            hex[15 - i] = digits[ptr & 0xF];
            ptr >>= 4;
        }
        hex[16] = '\0';
        debugcon_write(hex);
    }
    debugcon_write("\n");

    proc_t *ring3_proc = proc_create_user_process(
        "phase10-minimal",
        embedded_elf,
        (uint64_t)embedded_elf_size,
        PROC_IMAGE_ELF
    );
    
    if (!ring3_proc) {
        debugcon_write("[K][USER_ELF_CREATE_FAIL]\n");
        ring3_prep_panic("[[AYKEN_RING3_PREP_FAIL]] create",
                         "[PANIC] Phase10: Ring3 process creation failed.");
    }
    
    // Phase-16: Set execution role based on minimal mode
    // BCIB-forbidden test requires BCIB execution role for boundary enforcement
    if (embedded_elf_mode[0] == 'b' && embedded_elf_mode[1] == 'c' &&
        embedded_elf_mode[2] == 'i' && embedded_elf_mode[3] == 'b' &&
        embedded_elf_mode[4] == '-' && embedded_elf_mode[5] == 'f') {
        // Mode starts with "bcib-f" -> bcib-forbidden
        ring3_proc->execution_role = PROC_EXECUTION_ROLE_BCIB;
        debugcon_write("[K][EXEC_ROLE] BCIB (bcib-forbidden mode)\n");
    }
    
    // [K][USER_ELF_CREATE_OK] - Process created successfully
    debugcon_write("[K][USER_ELF_CREATE_OK] pid=");
    {
        uint32_t pid = (uint32_t)ring3_proc->pid;
        char buf[12];
        int i = 0;
        if (pid == 0) {
            buf[i++] = '0';
        } else {
            char tmp[12];
            int j = 0;
            while (pid > 0) {
                tmp[j++] = '0' + (pid % 10);
                pid /= 10;
            }
            while (j > 0) {
                buf[i++] = tmp[--j];
            }
        }
        buf[i] = '\0';
        debugcon_write(buf);
    }
    debugcon_write(" rip=");
    {
        uint64_t rip = ring3_proc->context.rip;
        char hex[17];
        const char *digits = "0123456789abcdef";
        for (int i = 0; i < 16; i++) {
            hex[15 - i] = digits[rip & 0xF];
            rip >>= 4;
        }
        hex[16] = '\0';
        debugcon_write(hex);
    }
    debugcon_write(" rsp=");
    {
        uint64_t rsp = ring3_proc->context.rsp;
        char hex[17];
        const char *digits = "0123456789abcdef";
        for (int i = 0; i < 16; i++) {
            hex[15 - i] = digits[rsp & 0xF];
            rsp >>= 4;
        }
        hex[16] = '\0';
        debugcon_write(hex);
    }
    debugcon_write(" cr3=");
    {
        uint64_t cr3 = ring3_proc->context.cr3;
        char hex[17];
        const char *digits = "0123456789abcdef";
        for (int i = 0; i < 16; i++) {
            hex[15 - i] = digits[cr3 & 0xF];
            cr3 >>= 4;
        }
        hex[16] = '\0';
        debugcon_write(hex);
    }
    debugcon_write("\n");
    
    // Phase-16: BCIB fail-closed proof test
    // Set execution role to BCIB for boundary enforcement testing
    #if defined(AYKEN_PHASE16_BCIB_PROOF_TEST) && (AYKEN_PHASE16_BCIB_PROOF_TEST == 1)
    ring3_proc->execution_role = PROC_EXECUTION_ROLE_BCIB;
    fb_print("[PHASE16] User process role set to BCIB for fail-closed proof test\n");
    #endif

    if (proc_find_by_pid(ring3_proc->pid) != ring3_proc) {
        ring3_prep_panic("[[AYKEN_RING3_PREP_FAIL]] not_registered",
                         "[PANIC] Phase10: Ring3 process not present in pid table.");
    }
    if (ring3_proc->state != PROC_READY) {
        ring3_prep_panic("[[AYKEN_RING3_PREP_FAIL]] not_ready",
                         "[PANIC] Phase10: Ring3 process is not runnable.");
    }
    if (ring3_proc->context.cs != GDT_USER_CODE || ring3_proc->context.ss != GDT_USER_DATA) {
        ring3_prep_panic("[[AYKEN_RING3_PREP_FAIL]] bad_segments",
                         "[PANIC] Phase10: Ring3 selectors are invalid.");
    }
    if (!ring3_proc->context.rsp0) {
        ring3_prep_panic("[[AYKEN_RING3_PREP_FAIL]] no_rsp0",
                         "[PANIC] Phase10: Ring3 process has no kernel rsp0.");
    }

    debugcon_write("[[AYKEN_RING3_PREP_OK]]\n");
    debugcon_write("P10_SCHED_ARMED\n");
    fb_print("[PHASE10] Ring3 process prepared and queued.\n");
}

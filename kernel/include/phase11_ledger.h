// kernel/include/phase11_ledger.h
//
// Phase-11 Decision Ledger v1 contract surface (P11-02).
// This header defines the canonical ledger entry payload used by
// CI evidence generation and upcoming kernel-side append hooks.
//
// Author: Kenan AY

#pragma once

#include <stdint.h>

#define AYKEN_LEDGER_FILE_MAGIC 0x3147444Cu /* "LDG1" */
#define AYKEN_LEDGER_ENTRY_MAGIC AYKEN_LEDGER_FILE_MAGIC
#define AYKEN_LEDGER_VERSION 1u
#define AYKEN_LEDGER_HASH_BYTES 32u
/* Bootstrap contract values for P11-02; canonical taxonomy remains
 * docs/architecture-board/PHASE11_EVENT_TAXONOMY.md.
 */
#define AYKEN_LEDGER_EVT_CTX_SWITCH 1u
#define AYKEN_LEDGER_EVT_MAX 53u

typedef uint64_t ay_event_seq_t;
typedef uint64_t ay_ltick_t;
typedef uint64_t ay_ctx_id_t;
typedef uint64_t ay_cap_id_t;

typedef struct __attribute__((packed)) ay_hash256_s {
    uint8_t bytes[AYKEN_LEDGER_HASH_BYTES];
} ay_hash256_t;

typedef struct __attribute__((packed)) ay_decision_ledger_entry_s {
    uint32_t magic;
    uint16_t version;
    uint16_t flags;

    ay_event_seq_t event_seq;
    ay_ltick_t ltick;

    uint32_t cpu_id;
    uint32_t event_type;

    ay_ctx_id_t prev_ctx;
    ay_ctx_id_t next_ctx;
    ay_cap_id_t decision_cap;

    uint64_t reason_code;
    uint64_t aux0;
    uint64_t aux1;

    ay_hash256_t payload_hash;
    ay_hash256_t prev_hash;
    ay_hash256_t entry_hash;
} ay_decision_ledger_entry_t;

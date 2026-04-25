// kernel/sys/execution_slot.c
// Execution-slot runtime state skeleton for Phase 10-B / 10-C hardening.

#include <stddef.h>
#include <stdint.h>

#include "../arch/x86_64/cpu.h"
#include "../arch/x86_64/port_io.h"
#include "../arch/x86_64/timer.h"
#include "../drivers/console/fb_console.h"
#include "../include/execution_slot.h"
#include "../include/mm.h"
#include "../include/proc.h"
#include "../include/sha256.h"

#define memset __builtin_memset
#define memcpy __builtin_memcpy

static exec_slot_t g_execution_slots[AYKEN_MAX_EXECUTION_SLOTS];
static execution_context_queue_t g_execution_queues[AYKEN_MAX_EXECUTION_CONTEXT_QUEUES];
static uint64_t g_next_execution_id = 1;
static execution_trace_actor_t g_execution_trace_actor = EXEC_TRACE_ACTOR_NONE;

#define AYKEN_EXECUTION_RESULT_BASE_VA (EXECUTION_PAYLOAD_VA + AYKEN_EXECUTION_PAYLOAD_WINDOW_SIZE)

#ifndef AYKEN_BCIB_STUB_RESULT_VALUE_U64
#define AYKEN_BCIB_STUB_RESULT_VALUE_U64 0xDEADBEEFCAFEBABEULL
#endif

static uint64_t execution_slot_read_rflags(void)
{
    uint64_t rflags = 0;
    __asm__ volatile("pushfq; popq %0" : "=r"(rflags));
    return rflags;
}

static void execution_slot_debugcon_write_char(char ch)
{
    outb(0xE9, (uint8_t)ch);
}

static void execution_slot_debugcon_write(const char *text)
{
    if (!text) {
        return;
    }

    while (*text != '\0') {
        execution_slot_debugcon_write_char(*text);
        text++;
    }
}

static void execution_slot_proof_emit_bytes(ayken_sha256_ctx_t *hash_ctx,
                                            const char *bytes,
                                            uint64_t len)
{
    uint64_t i;

    if (!bytes || len == 0) {
        return;
    }

    if (hash_ctx) {
        ayken_sha256_update(hash_ctx, bytes, len);
    }

    for (i = 0; i < len; ++i) {
        execution_slot_debugcon_write_char(bytes[i]);
    }
}

static void execution_slot_proof_emit_string(ayken_sha256_ctx_t *hash_ctx,
                                             const char *text)
{
    uint64_t len = 0;

    if (!text) {
        return;
    }

    while (text[len] != '\0') {
        len++;
    }

    execution_slot_proof_emit_bytes(hash_ctx, text, len);
}

static void execution_slot_proof_emit_u64(ayken_sha256_ctx_t *hash_ctx, uint64_t value)
{
    char buffer[21];
    uint32_t length = 0;
    uint32_t i;

    if (value == 0) {
        buffer[length++] = '0';
    } else {
        while (value != 0 && length < sizeof(buffer)) {
            buffer[length++] = (char)('0' + (value % 10));
            value /= 10;
        }
    }

    for (i = 0; i < length / 2u; ++i) {
        char tmp = buffer[i];
        buffer[i] = buffer[length - 1u - i];
        buffer[length - 1u - i] = tmp;
    }

    execution_slot_proof_emit_bytes(hash_ctx, buffer, length);
}

static void execution_slot_proof_emit_line_end(ayken_sha256_ctx_t *hash_ctx)
{
    execution_slot_proof_emit_bytes(hash_ctx, "\n", 1);
}

static void execution_slot_debugcon_write_sha256(const uint8_t digest[AYKEN_SHA256_DIGEST_SIZE])
{
    static const char hex_digits[] = "0123456789abcdef";
    uint32_t i;

    if (!digest) {
        return;
    }

    for (i = 0; i < AYKEN_SHA256_DIGEST_SIZE; ++i) {
        execution_slot_debugcon_write_char(hex_digits[(digest[i] >> 4) & 0x0Fu]);
        execution_slot_debugcon_write_char(hex_digits[digest[i] & 0x0Fu]);
    }
}

static int execution_slot_copy_into_frames_locked(const uint64_t *frames,
                                                  uint32_t frame_count,
                                                  uint64_t start_offset,
                                                  const void *src,
                                                  uint64_t size)
{
    const uint8_t *src_bytes = (const uint8_t *)src;
    uint64_t remaining = size;
    uint64_t offset = start_offset;

    if (!frames || frame_count == 0) {
        return -1;
    }
    if (remaining == 0) {
        return 0;
    }
    if (!src_bytes) {
        return -1;
    }

    while (remaining > 0) {
        uint32_t frame_index = (uint32_t)(offset / AYKEN_FRAME_SIZE);
        uint64_t frame_offset = offset % AYKEN_FRAME_SIZE;
        uint64_t chunk_size;
        uint8_t *dst;

        if (frame_index >= frame_count || frames[frame_index] == 0) {
            return -1;
        }

        dst = (uint8_t *)paging_phys_to_virt(frames[frame_index]);
        if (!dst) {
            return -1;
        }

        chunk_size = AYKEN_FRAME_SIZE - frame_offset;
        if (chunk_size > remaining) {
            chunk_size = remaining;
        }

        memcpy(dst + frame_offset, src_bytes, chunk_size);
        src_bytes += chunk_size;
        offset += chunk_size;
        remaining -= chunk_size;
    }

    return 0;
}

static void execution_slot_emit_fail_closed_proof_locked(const char *site,
                                                         const exec_slot_t *slot,
                                                         exec_slot_state_t expected_from,
                                                         exec_slot_state_t next_state)
{
    ayken_sha256_ctx_t proof_hash_ctx;
    execution_trace_entry_t entry = {0};
    uint8_t proof_hash[AYKEN_SHA256_DIGEST_SIZE];
    uint32_t trace_count = 0;
    uint32_t i;
    uint64_t execution_id = 0;
    uint64_t generation = 0;
    uint64_t final_state = EXEC_SLOT_CREATED;
    int invariants_ok = execution_slot_verify_global_invariants_locked() == 0;

    if (slot && slot->in_use) {
        execution_id = slot->execution_id;
        generation = slot->generation;
        final_state = (uint64_t)slot->state;
        trace_count = execution_slot_trace_count_locked(slot);
    }

    execution_slot_debugcon_write("[[P10B_FAIL_CLOSED_BEGIN]]\n");

    ayken_sha256_init(&proof_hash_ctx);
    execution_slot_proof_emit_string(&proof_hash_ctx, "[[P10B_FAIL_CLOSED_META]] site=");
    execution_slot_proof_emit_string(&proof_hash_ctx, site ? site : "<null>");
    execution_slot_proof_emit_string(&proof_hash_ctx, " exec_id=");
    execution_slot_proof_emit_u64(&proof_hash_ctx, execution_id);
    execution_slot_proof_emit_string(&proof_hash_ctx, " generation=");
    execution_slot_proof_emit_u64(&proof_hash_ctx, generation);
    execution_slot_proof_emit_string(&proof_hash_ctx, " current=");
    execution_slot_proof_emit_u64(&proof_hash_ctx,
                                  slot && slot->in_use ? (uint64_t)slot->state : 0);
    execution_slot_proof_emit_string(&proof_hash_ctx, " expected=");
    execution_slot_proof_emit_u64(&proof_hash_ctx, (uint64_t)expected_from);
    execution_slot_proof_emit_string(&proof_hash_ctx, " next=");
    execution_slot_proof_emit_u64(&proof_hash_ctx, (uint64_t)next_state);
    execution_slot_proof_emit_string(&proof_hash_ctx, " final_state=");
    execution_slot_proof_emit_u64(&proof_hash_ctx, final_state);
    execution_slot_proof_emit_string(&proof_hash_ctx, " invariants_ok=");
    execution_slot_proof_emit_u64(&proof_hash_ctx, invariants_ok ? 1u : 0u);
    execution_slot_proof_emit_string(&proof_hash_ctx, " trace_count=");
    execution_slot_proof_emit_u64(&proof_hash_ctx, trace_count);
    execution_slot_proof_emit_line_end(&proof_hash_ctx);

    for (i = 0; i < trace_count; ++i) {
        if (execution_slot_trace_get_locked(slot, i, &entry) != 0) {
            break;
        }

        execution_slot_proof_emit_string(&proof_hash_ctx, "[[P10B_FAIL_CLOSED_TRACE]] idx=");
        execution_slot_proof_emit_u64(&proof_hash_ctx, i);
        execution_slot_proof_emit_string(&proof_hash_ctx, " tick=");
        execution_slot_proof_emit_u64(&proof_hash_ctx, entry.tick);
        execution_slot_proof_emit_string(&proof_hash_ctx, " exec_id=");
        execution_slot_proof_emit_u64(&proof_hash_ctx, entry.execution_id);
        execution_slot_proof_emit_string(&proof_hash_ctx, " generation=");
        execution_slot_proof_emit_u64(&proof_hash_ctx, entry.generation);
        execution_slot_proof_emit_string(&proof_hash_ctx, " actor=");
        execution_slot_proof_emit_u64(&proof_hash_ctx, (uint64_t)entry.actor);
        execution_slot_proof_emit_string(&proof_hash_ctx, " from=");
        execution_slot_proof_emit_u64(&proof_hash_ctx, (uint64_t)entry.from_state);
        execution_slot_proof_emit_string(&proof_hash_ctx, " to=");
        execution_slot_proof_emit_u64(&proof_hash_ctx, (uint64_t)entry.to_state);
        execution_slot_proof_emit_line_end(&proof_hash_ctx);
    }

    ayken_sha256_final(&proof_hash_ctx, proof_hash);
    execution_slot_debugcon_write("[[P10B_FAIL_CLOSED_HASH]] sha256=");
    execution_slot_debugcon_write_sha256(proof_hash);
    execution_slot_debugcon_write("\n");
    execution_slot_debugcon_write("[[P10B_FAIL_CLOSED_END]]\n");
}

static __attribute__((noreturn)) void execution_slot_runtime_panic(const char *site,
                                                                   const exec_slot_t *slot,
                                                                   exec_slot_state_t expected_from,
                                                                   exec_slot_state_t next_state)
{
    execution_slot_emit_fail_closed_proof_locked(site, slot, expected_from, next_state);

    fb_print("[PANIC] execution-slot invalid transition");
    if (site) {
        fb_print(" site=");
        fb_print(site);
    }
    if (slot) {
        fb_print(" exec_id=");
        fb_print_int(slot->execution_id);
        fb_print(" current=");
        fb_print_int((uint64_t)slot->state);
    }
    fb_print(" expected=");
    fb_print_int((uint64_t)expected_from);
    fb_print(" next=");
    fb_print_int((uint64_t)next_state);
    fb_print("\n");

    for (;;) {
        __asm__ volatile("cli; hlt");
    }
}

static void execution_slot_zero_slot(exec_slot_t *slot)
{
    uint32_t i;

    if (!slot) {
        return;
    }

    slot->in_use = 0;
    slot->execution_id = 0;
    slot->owner_pid = 0;
    slot->target_context_id = 0;
    slot->created_tick = 0;
    slot->deadline_tick = 0;
    slot->state = EXEC_SLOT_CREATED;
    slot->bcib_frame_count = 0;
    slot->result_frame_count = 0;
    slot->output_frame_count = 0;
    for (i = 0; i < AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES; ++i) {
        slot->bcib_frames[i] = 0;
        slot->result_frames[i] = 0;
    }
    for (i = 0; i < AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES; ++i) {
        slot->output_frames[i] = 0;
    }
    slot->bcib_size = 0;
    slot->result_size = 0;
    slot->output_size = 0;
    slot->hash_frame = 0;
    slot->hash_size = 0;
    slot->hashed_size = 0;
    slot->mapped_result_va = 0;
    slot->mapped_hash_va = 0;
    slot->result_map_flags = 0;
    slot->error_code = 0;
    slot->queue_next_index = AYKEN_EXECUTION_INVALID_INDEX;
    slot->wait_key.execution_id = 0;
    slot->wait_key.generation = slot->generation;
    slot->trace_count = 0;
    slot->trace_head = 0;
    memset(slot->trace_entries, 0, sizeof(slot->trace_entries));
}

static void execution_slot_zero_queue(execution_context_queue_t *queue)
{
    if (!queue) {
        return;
    }

    queue->in_use = 0;
    queue->context_id = 0;
    queue->head_index = AYKEN_EXECUTION_INVALID_INDEX;
    queue->tail_index = AYKEN_EXECUTION_INVALID_INDEX;
    queue->depth = 0;
}

static uint32_t execution_slot_index(const exec_slot_t *slot)
{
    if (!slot) {
        return AYKEN_EXECUTION_INVALID_INDEX;
    }
    if (slot < &g_execution_slots[0] ||
        slot >= &g_execution_slots[AYKEN_MAX_EXECUTION_SLOTS]) {
        return AYKEN_EXECUTION_INVALID_INDEX;
    }
    return (uint32_t)(slot - &g_execution_slots[0]);
}

static uint32_t execution_slot_frame_count_for_size(uint64_t size)
{
    if (size == 0) {
        return 0;
    }

    return (uint32_t)((size + (AYKEN_FRAME_SIZE - 1)) / AYKEN_FRAME_SIZE);
}

static void execution_slot_zero_phys_frame(uint64_t phys_addr)
{
    void *dst;

    if (phys_addr == 0) {
        return;
    }

    dst = paging_phys_to_virt(phys_addr);
    if (!dst) {
        return;
    }

    memset(dst, 0, AYKEN_FRAME_SIZE);
}

static void execution_slot_zero_frame_tail(uint64_t *frames,
                                           uint32_t frame_count,
                                           uint64_t start_offset)
{
    uint64_t total_size = (uint64_t)frame_count * AYKEN_FRAME_SIZE;
    uint64_t offset = start_offset;

    if (!frames || frame_count == 0 || start_offset >= total_size) {
        return;
    }

    while (offset < total_size) {
        uint32_t frame_index = (uint32_t)(offset / AYKEN_FRAME_SIZE);
        uint64_t frame_offset = offset % AYKEN_FRAME_SIZE;
        uint64_t chunk_size = AYKEN_FRAME_SIZE - frame_offset;
        uint8_t *dst;

        if (frame_index >= frame_count || frames[frame_index] == 0) {
            return;
        }

        dst = (uint8_t *)paging_phys_to_virt(frames[frame_index]);
        if (!dst) {
            return;
        }

        memset(dst + frame_offset, 0, chunk_size);
        offset += chunk_size;
    }
}

static void execution_slot_release_bcib_backing_locked(exec_slot_t *slot)
{
    uint32_t i;

    if (!slot) {
        return;
    }

    for (i = 0; i < AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES; ++i) {
        uint64_t phys_addr = slot->bcib_frames[i];
        if (phys_addr == 0) {
            continue;
        }

        execution_slot_zero_phys_frame(phys_addr);
        phys_free_frame(phys_addr);
        slot->bcib_frames[i] = 0;
    }

    slot->bcib_frame_count = 0;
    slot->bcib_size = 0;
}

static void execution_slot_release_result_backing_locked(exec_slot_t *slot)
{
    uint32_t i;

    if (!slot) {
        return;
    }

    for (i = 0; i < AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES; ++i) {
        uint64_t phys_addr = slot->result_frames[i];
        if (phys_addr == 0) {
            continue;
        }

        execution_slot_zero_phys_frame(phys_addr);
        phys_free_frame(phys_addr);
        slot->result_frames[i] = 0;
    }

    slot->result_frame_count = 0;
    slot->result_size = 0;
    slot->mapped_result_va = 0;
    slot->result_map_flags = 0;
}

static void execution_slot_release_hash_backing_locked(exec_slot_t *slot)
{
    if (!slot) {
        return;
    }

    if (slot->hash_frame != 0) {
        execution_slot_zero_phys_frame(slot->hash_frame);
        phys_free_frame(slot->hash_frame);
        slot->hash_frame = 0;
    }

    slot->hash_size = 0;
    slot->hashed_size = 0;
    slot->mapped_hash_va = 0;
}

static void execution_slot_release_output_backing_locked(exec_slot_t *slot)
{
    uint32_t i;

    if (!slot) {
        return;
    }

    for (i = 0; i < AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES; ++i) {
        uint64_t phys_addr = slot->output_frames[i];
        if (phys_addr == 0) {
            continue;
        }

        execution_slot_zero_phys_frame(phys_addr);
        phys_free_frame(phys_addr);
        slot->output_frames[i] = 0;
    }

    slot->output_frame_count = 0;
    slot->output_size = 0;
}

static execution_context_queue_t *execution_slot_alloc_queue_locked(uint64_t context_id)
{
    uint32_t i;

    for (i = 0; i < AYKEN_MAX_EXECUTION_CONTEXT_QUEUES; ++i) {
        if (!g_execution_queues[i].in_use) {
            g_execution_queues[i].in_use = 1;
            g_execution_queues[i].context_id = context_id;
            g_execution_queues[i].head_index = AYKEN_EXECUTION_INVALID_INDEX;
            g_execution_queues[i].tail_index = AYKEN_EXECUTION_INVALID_INDEX;
            g_execution_queues[i].depth = 0;
            return &g_execution_queues[i];
        }
    }

    return NULL;
}

static uint64_t execution_slot_allocate_id_locked(void)
{
    uint64_t execution_id;

    if (g_next_execution_id == 0) {
        return 0;
    }

    execution_id = g_next_execution_id;
    g_next_execution_id++;
    return execution_id;
}

static void execution_slot_trace_append_locked(exec_slot_t *slot,
                                               exec_slot_state_t from_state,
                                               exec_slot_state_t to_state)
{
    execution_trace_entry_t *entry;
    uint32_t trace_index;

    if (!slot || !slot->in_use) {
        return;
    }

    trace_index = slot->trace_head;
    if (trace_index >= AYKEN_EXECUTION_TRACE_CAPACITY) {
        trace_index = 0;
    }

    entry = &slot->trace_entries[trace_index];
    entry->tick = timer_ticks();
    entry->execution_id = slot->execution_id;
    entry->generation = slot->generation;
    entry->actor = (uint8_t)g_execution_trace_actor;
    entry->from_state = (uint8_t)from_state;
    entry->to_state = (uint8_t)to_state;
    memset(entry->reserved0, 0, sizeof(entry->reserved0));

    slot->trace_head = (trace_index + 1u) % AYKEN_EXECUTION_TRACE_CAPACITY;
    if (slot->trace_count < AYKEN_EXECUTION_TRACE_CAPACITY) {
        slot->trace_count++;
    }
}

static uint32_t execution_slot_trace_oldest_index_locked(const exec_slot_t *slot)
{
    if (!slot || slot->trace_count == 0) {
        return 0;
    }

    if (slot->trace_count < AYKEN_EXECUTION_TRACE_CAPACITY) {
        return 0;
    }

    return slot->trace_head;
}

static int execution_slot_can_transition(exec_slot_state_t from, exec_slot_state_t to)
{
    switch (from) {
    case EXEC_SLOT_CREATED:
        return to == EXEC_SLOT_READY || to == EXEC_SLOT_FAILED;
    case EXEC_SLOT_READY:
        return to == EXEC_SLOT_RUNNING ||
               to == EXEC_SLOT_TIMEOUT ||
               to == EXEC_SLOT_ABORTED;
    case EXEC_SLOT_RUNNING:
        return to == EXEC_SLOT_COMPLETED ||
               to == EXEC_SLOT_FAILED ||
               to == EXEC_SLOT_TIMEOUT ||
               to == EXEC_SLOT_ABORTED;
    case EXEC_SLOT_COMPLETED:
        return to == EXEC_SLOT_RESULT_MAPPED;
    case EXEC_SLOT_RESULT_MAPPED:
        return to == EXEC_SLOT_RESULT_MAPPED;
    case EXEC_SLOT_FAILED:
    case EXEC_SLOT_TIMEOUT:
    case EXEC_SLOT_ABORTED:
    default:
        return 0;
    }
}

static int execution_slot_trace_state_is_immutable(exec_slot_state_t state)
{
    return state == EXEC_SLOT_FAILED ||
           state == EXEC_SLOT_TIMEOUT ||
           state == EXEC_SLOT_ABORTED ||
           state == EXEC_SLOT_RESULT_MAPPED;
}

static void execution_slot_clear_target_latch_locked(const exec_slot_t *slot)
{
    proc_t *target_proc;

    if (!slot || slot->target_context_id == 0) {
        return;
    }

    target_proc = proc_find_by_pid((int)slot->target_context_id);
    if (!target_proc) {
        return;
    }

    if (target_proc->execution_output_mapped_id == slot->execution_id) {
        proc_unmap_execution_output_window(target_proc);
    }

    if (target_proc->active_execution_id == slot->execution_id) {
        target_proc->active_execution_id = 0;
    }
}

static int execution_slot_remove_from_queue_locked(exec_slot_t *slot)
{
    execution_context_queue_t *queue;
    uint32_t slot_index;
    uint32_t iter_index;
    uint32_t prev_index = AYKEN_EXECUTION_INVALID_INDEX;

    if (!slot || !slot->in_use) {
        return -1;
    }

    slot_index = execution_slot_index(slot);
    if (slot_index == AYKEN_EXECUTION_INVALID_INDEX) {
        return -1;
    }

    queue = execution_slot_find_queue_locked(slot->target_context_id);
    if (!queue) {
        slot->queue_next_index = AYKEN_EXECUTION_INVALID_INDEX;
        return 0;
    }

    iter_index = queue->head_index;
    while (iter_index != AYKEN_EXECUTION_INVALID_INDEX) {
        exec_slot_t *iter = &g_execution_slots[iter_index];
        uint32_t next_index = iter->queue_next_index;

        if (iter_index == slot_index) {
            if (prev_index == AYKEN_EXECUTION_INVALID_INDEX) {
                queue->head_index = next_index;
            } else {
                g_execution_slots[prev_index].queue_next_index = next_index;
            }
            if (queue->tail_index == slot_index) {
                queue->tail_index = prev_index;
            }
            if (queue->depth > 0) {
                queue->depth--;
            }
            slot->queue_next_index = AYKEN_EXECUTION_INVALID_INDEX;
            if (queue->depth == 0) {
                execution_slot_zero_queue(queue);
            }
            return 0;
        }

        prev_index = iter_index;
        iter_index = next_index;
    }

    slot->queue_next_index = AYKEN_EXECUTION_INVALID_INDEX;
    return 0;
}

int execution_slot_finish_locked(exec_slot_t *slot, exec_slot_state_t next_state)
{
    exec_slot_state_t prior_state;

    if (!slot || !slot->in_use) {
        return -1;
    }

    prior_state = slot->state;
    if (execution_slot_state_is_terminal(prior_state)) {
        return -1;
    }

    if (prior_state == EXEC_SLOT_READY) {
        if (execution_slot_remove_from_queue_locked(slot) != 0) {
            return -1;
        }
    }

    if (execution_slot_transition_locked(slot, prior_state, next_state) != 0) {
        return -1;
    }

    if (next_state == EXEC_SLOT_FAILED ||
        next_state == EXEC_SLOT_TIMEOUT ||
        next_state == EXEC_SLOT_ABORTED) {
        execution_slot_release_bcib_backing_locked(slot);
        execution_slot_release_result_backing_locked(slot);
        execution_slot_release_output_backing_locked(slot);
        execution_slot_release_hash_backing_locked(slot);
    }

    slot->deadline_tick = 0;
    execution_slot_clear_target_latch_locked(slot);
    proc_wake_waiters(&slot->wait_key);
    return 0;
}

int execution_slot_require_finish_locked(exec_slot_t *slot,
                                         exec_slot_state_t next_state,
                                         const char *site)
{
    if (execution_slot_finish_locked(slot, next_state) != 0) {
        execution_slot_runtime_panic(site,
                                     slot,
                                     slot ? slot->state : EXEC_SLOT_CREATED,
                                     next_state);
    }

    return 0;
}

void execution_slots_init(void)
{
    uint32_t i;

    for (i = 0; i < AYKEN_MAX_EXECUTION_SLOTS; ++i) {
        g_execution_slots[i].generation = 0;
        execution_slot_zero_slot(&g_execution_slots[i]);
    }

    for (i = 0; i < AYKEN_MAX_EXECUTION_CONTEXT_QUEUES; ++i) {
        execution_slot_zero_queue(&g_execution_queues[i]);
    }

    g_next_execution_id = 1;
}

void execution_slot_run_fail_closed_selftest(void)
{
#if defined(AYKEN_PHASE10B_FAIL_CLOSED_SELFTEST) && (AYKEN_PHASE10B_FAIL_CLOSED_SELFTEST == 1)
    execution_slot_guard_t slot_guard = {0};
    execution_slot_trace_scope_t trace_scope = {0};
    exec_slot_t *slot = NULL;

    execution_slot_debugcon_write("[[P10B_FAIL_CLOSED_SELFTEST_BEGIN]]\n");
    execution_slot_enter_critical(&slot_guard);
    execution_slot_trace_scope_enter(&trace_scope, EXEC_TRACE_ACTOR_VALIDATION);

    slot = execution_slot_alloc_locked(1, 1);
    if (!slot) {
        execution_slot_runtime_panic("phase10b_fail_closed_selftest.alloc",
                                     NULL,
                                     EXEC_SLOT_CREATED,
                                     EXEC_SLOT_READY);
    }

    execution_slot_require_transition_locked(slot,
                                             EXEC_SLOT_CREATED,
                                             EXEC_SLOT_READY,
                                             "phase10b_fail_closed_selftest.seed");
    execution_slot_require_transition_locked(slot,
                                             EXEC_SLOT_RUNNING,
                                             EXEC_SLOT_COMPLETED,
                                             "phase10b_fail_closed_selftest.trigger");
    execution_slot_runtime_panic("phase10b_fail_closed_selftest.missed_panic",
                                 slot,
                                 EXEC_SLOT_RUNNING,
                                 EXEC_SLOT_COMPLETED);
#endif
}

uint32_t execution_slots_capacity(void)
{
    return AYKEN_MAX_EXECUTION_SLOTS;
}

uint32_t execution_slot_queue_capacity(void)
{
    return AYKEN_MAX_EXECUTION_CONTEXT_QUEUES;
}

void execution_slot_enter_critical(execution_slot_guard_t *guard)
{
    uint64_t rflags;

    if (!guard) {
        return;
    }

    // Initial landing relies on the current single-core runtime: disabling
    // interrupts serializes all execution-slot mutations until a real lock
    // primitive exists in the kernel tree.
    rflags = execution_slot_read_rflags();
    disable_interrupts();

    guard->saved_rflags = rflags;
    guard->interrupts_were_enabled = (rflags & (1ULL << 9)) ? 1u : 0u;
    guard->entered = 1u;
}

void execution_slot_exit_critical(execution_slot_guard_t *guard)
{
    if (!guard || !guard->entered) {
        return;
    }

    if (guard->interrupts_were_enabled) {
        enable_interrupts();
    }

    guard->saved_rflags = 0;
    guard->interrupts_were_enabled = 0;
    guard->entered = 0;
}

void execution_slot_trace_scope_enter(execution_slot_trace_scope_t *scope,
                                      execution_trace_actor_t actor)
{
    if (!scope) {
        return;
    }

    scope->previous_actor = g_execution_trace_actor;
    scope->active = 1u;
    g_execution_trace_actor = actor;
}

void execution_slot_trace_scope_exit(execution_slot_trace_scope_t *scope)
{
    if (!scope || !scope->active) {
        return;
    }

    g_execution_trace_actor = scope->previous_actor;
    scope->previous_actor = EXEC_TRACE_ACTOR_NONE;
    scope->active = 0u;
}

exec_slot_t *execution_slot_alloc_locked(uint64_t owner_pid, uint64_t target_context_id)
{
    uint32_t i;
    exec_slot_t *slot;

    for (i = 0; i < AYKEN_MAX_EXECUTION_SLOTS; ++i) {
        slot = &g_execution_slots[i];
        if (slot->in_use) {
            continue;
        }

        slot->generation++;
        execution_slot_zero_slot(slot);

        slot->in_use = 1;
        slot->execution_id = execution_slot_allocate_id_locked();
        if (slot->execution_id == 0) {
            slot->in_use = 0;
            return NULL;
        }
        slot->owner_pid = owner_pid;
        slot->target_context_id = target_context_id;
        slot->state = EXEC_SLOT_CREATED;
        slot->queue_next_index = AYKEN_EXECUTION_INVALID_INDEX;
        slot->wait_key.execution_id = slot->execution_id;
        slot->wait_key.generation = slot->generation;

        return slot;
    }

    return NULL;
}

void execution_slot_release_locked(exec_slot_t *slot)
{
    if (!slot || !slot->in_use) {
        return;
    }

    execution_slot_release_bcib_backing_locked(slot);
    execution_slot_release_result_backing_locked(slot);
    execution_slot_release_output_backing_locked(slot);
    execution_slot_release_hash_backing_locked(slot);
    slot->execution_id = 0;
    slot->owner_pid = 0;
    slot->target_context_id = 0;
    slot->created_tick = 0;
    slot->deadline_tick = 0;
    slot->state = EXEC_SLOT_CREATED;
    slot->result_frame_count = 0;
    slot->result_size = 0;
    slot->mapped_result_va = 0;
    slot->mapped_hash_va = 0;
    slot->result_map_flags = 0;
    slot->error_code = 0;
    slot->queue_next_index = AYKEN_EXECUTION_INVALID_INDEX;
    slot->wait_key.execution_id = 0;
    slot->wait_key.generation = slot->generation;
    slot->trace_count = 0;
    slot->trace_head = 0;
    memset(slot->trace_entries, 0, sizeof(slot->trace_entries));
    slot->in_use = 0;
}

int execution_slot_store_bcib_locked(exec_slot_t *slot,
                                     const void *bcib_graph,
                                     uint64_t graph_size)
{
    const uint8_t *src = (const uint8_t *)bcib_graph;
    uint32_t frame_count;
    uint32_t i;
    uint64_t remaining;

    if (!slot || !slot->in_use || !bcib_graph || graph_size == 0) {
        return -1;
    }

    if (graph_size > AYKEN_EXECUTION_PAYLOAD_WINDOW_SIZE) {
        return -1;
    }

    frame_count = execution_slot_frame_count_for_size(graph_size);
    if (frame_count == 0 || frame_count > AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES) {
        return -1;
    }

    execution_slot_release_bcib_backing_locked(slot);

    remaining = graph_size;
    for (i = 0; i < frame_count; ++i) {
        uint64_t phys_addr = phys_alloc_frame();
        uint64_t copy_size = remaining > AYKEN_FRAME_SIZE ? AYKEN_FRAME_SIZE : remaining;
        void *dst;

        if (phys_addr == 0) {
            execution_slot_release_bcib_backing_locked(slot);
            return -1;
        }

        dst = paging_phys_to_virt(phys_addr);
        if (!dst) {
            phys_free_frame(phys_addr);
            execution_slot_release_bcib_backing_locked(slot);
            return -1;
        }

        memset(dst, 0, AYKEN_FRAME_SIZE);
        memcpy(dst, src + ((uint64_t)i * AYKEN_FRAME_SIZE), copy_size);

        slot->bcib_frames[i] = phys_addr;
        remaining -= copy_size;
    }

    slot->bcib_frame_count = frame_count;
    slot->bcib_size = graph_size;
    return 0;
}

int execution_slot_prepare_output_locked(exec_slot_t *slot)
{
    uint32_t i;

    if (!slot || !slot->in_use || slot->state != EXEC_SLOT_RUNNING) {
        return -1;
    }

    if (slot->output_frame_count == AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES) {
        for (i = 0; i < AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES; ++i) {
            if (slot->output_frames[i] == 0) {
                execution_slot_release_output_backing_locked(slot);
                break;
            }
            execution_slot_zero_phys_frame(slot->output_frames[i]);
        }
        if (slot->output_frame_count == AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES) {
            slot->output_size = 0;
            return 0;
        }
    }

    execution_slot_release_output_backing_locked(slot);

    for (i = 0; i < AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES; ++i) {
        uint64_t phys_addr = phys_alloc_frame();

        if (phys_addr == 0) {
            execution_slot_release_output_backing_locked(slot);
            return -1;
        }

        execution_slot_zero_phys_frame(phys_addr);
        slot->output_frames[i] = phys_addr;
    }

    slot->output_frame_count = AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES;
    slot->output_size = 0;
    return 0;
}

int execution_slot_write_output_v1_locked(exec_slot_t *slot,
                                          const void *payload,
                                          uint64_t payload_size)
{
    ayken_execution_output_v1_t header;

    if (!slot || !slot->in_use || slot->state != EXEC_SLOT_RUNNING) {
        return -1;
    }
    if (slot->output_frame_count != AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES) {
        return -1;
    }
    if (payload_size > (AYKEN_EXECUTION_OUTPUT_WINDOW_SIZE - sizeof(header))) {
        return -1;
    }

    memset(&header, 0, sizeof(header));
    header.magic = AYKEN_EXECUTION_OUTPUT_MAGIC;
    header.abi_version = AYKEN_EXECUTION_OUTPUT_VERSION;
    header.bytes_written = payload_size;

    if (execution_slot_copy_into_frames_locked(slot->output_frames,
                                               slot->output_frame_count,
                                               0,
                                               &header,
                                               sizeof(header)) != 0) {
        return -1;
    }
    if (payload_size > 0 &&
        execution_slot_copy_into_frames_locked(slot->output_frames,
                                               slot->output_frame_count,
                                               sizeof(header),
                                               payload,
                                               payload_size) != 0) {
        return -1;
    }

    slot->output_size = sizeof(header) + payload_size;
    return 0;
}

int execution_slot_validate_output_locked(exec_slot_t *slot, uint64_t *published_size)
{
    const ayken_execution_output_v1_t *raw_header;
    const ayken_execution_output_v2_t *structured_header;
    uint64_t total_size;
    uint32_t i;

    if (!slot || !slot->in_use || slot->state != EXEC_SLOT_RUNNING) {
        return -1;
    }
    if (slot->output_frame_count != AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES) {
        return -1;
    }
    for (i = 0; i < AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES; ++i) {
        if (slot->output_frames[i] == 0) {
            return -1;
        }
    }

    raw_header = (const ayken_execution_output_v1_t *)paging_phys_to_virt(slot->output_frames[0]);
    if (!raw_header) {
        return -1;
    }

    if (raw_header->magic == AYKEN_EXECUTION_OUTPUT_MAGIC &&
        raw_header->abi_version == AYKEN_EXECUTION_OUTPUT_VERSION) {
        if (raw_header->bytes_written >
            (AYKEN_EXECUTION_OUTPUT_WINDOW_SIZE - sizeof(*raw_header))) {
            return -1;
        }

        total_size = sizeof(*raw_header) + raw_header->bytes_written;
        slot->output_size = total_size;
        if (published_size) {
            *published_size = total_size;
        }
        return 0;
    }

    structured_header = (const ayken_execution_output_v2_t *)raw_header;
    if (structured_header->magic != AYKEN_EXECUTION_OUTPUT_V2_MAGIC) {
        return -1;
    }
    if (structured_header->abi_version != AYKEN_EXECUTION_OUTPUT_V2_VERSION) {
        return -1;
    }
    if (structured_header->kind != AYKEN_OUTPUT_KIND_RAW &&
        structured_header->kind != AYKEN_OUTPUT_KIND_BLOB) {
        return -1;
    }
    if (structured_header->bytes_written >
        (AYKEN_EXECUTION_OUTPUT_WINDOW_SIZE - sizeof(*structured_header))) {
        return -1;
    }

    total_size = sizeof(*structured_header) + structured_header->bytes_written;
    slot->output_size = total_size;
    if (published_size) {
        *published_size = total_size;
    }
    return 0;
}

int execution_slot_can_publish_locked(const exec_slot_t *slot)
{
    uint32_t required_frames;
    uint32_t i;

    if (!slot || !slot->in_use) {
        return 0;
    }

    if (slot->state != EXEC_SLOT_RUNNING) {
        return 0;
    }

    if (slot->bcib_size == 0 || slot->bcib_size > AYKEN_EXECUTION_PAYLOAD_WINDOW_SIZE) {
        return 0;
    }

    required_frames = execution_slot_frame_count_for_size(slot->bcib_size);
    if (required_frames == 0 ||
        required_frames > AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES ||
        slot->bcib_frame_count != required_frames) {
        return 0;
    }

    for (i = 0; i < required_frames; ++i) {
        if (slot->bcib_frames[i] == 0) {
            return 0;
        }
    }

    return 1;
}

uint64_t execution_slot_result_va_locked(const exec_slot_t *slot)
{
    uint32_t slot_index = execution_slot_index(slot);

    if (slot_index == AYKEN_EXECUTION_INVALID_INDEX) {
        return 0;
    }

    return AYKEN_EXECUTION_RESULT_BASE_VA +
           ((uint64_t)slot_index * AYKEN_EXECUTION_PAYLOAD_WINDOW_SIZE);
}

uint64_t execution_slot_result_hash_va_locked(const exec_slot_t *slot)
{
    uint32_t slot_index = execution_slot_index(slot);

    if (slot_index == AYKEN_EXECUTION_INVALID_INDEX) {
        return 0;
    }

    return AYKEN_EXECUTION_RESULT_HASH_BASE_VA +
           ((uint64_t)slot_index * AYKEN_EXECUTION_RESULT_HASH_WINDOW_SIZE);
}

static int execution_slot_hash_result_frames_locked(const exec_slot_t *slot,
                                                    uint8_t digest[AYKEN_SHA256_DIGEST_SIZE])
{
    ayken_sha256_ctx_t ctx;
    uint64_t remaining;
    uint32_t i;

    if (!slot || !digest || slot->result_size == 0 || slot->result_frame_count == 0) {
        return -1;
    }

    ayken_sha256_init(&ctx);
    remaining = slot->result_size;
    for (i = 0; i < slot->result_frame_count && remaining > 0; ++i) {
        const uint8_t *src = (const uint8_t *)paging_phys_to_virt(slot->result_frames[i]);
        uint64_t chunk_size = remaining > AYKEN_FRAME_SIZE ? AYKEN_FRAME_SIZE : remaining;

        if (slot->result_frames[i] == 0 || !src) {
            return -1;
        }

        ayken_sha256_update(&ctx, src, chunk_size);
        remaining -= chunk_size;
    }

    if (remaining != 0) {
        return -1;
    }

    ayken_sha256_final(&ctx, digest);
    return 0;
}

static int execution_slot_prepare_hash_locked(exec_slot_t *slot)
{
    ayken_execution_result_hash_v1_t *header;
    uint8_t digest[AYKEN_SHA256_DIGEST_SIZE];
    void *dst;

    if (!slot || !slot->in_use || slot->result_size == 0 || slot->result_frame_count == 0) {
        return -1;
    }

    if (slot->hash_frame != 0 &&
        slot->hash_size == sizeof(ayken_execution_result_hash_v1_t) &&
        slot->hashed_size == slot->result_size) {
        return 0;
    }

    if (execution_slot_hash_result_frames_locked(slot, digest) != 0) {
        return -1;
    }

    execution_slot_release_hash_backing_locked(slot);

    slot->hash_frame = phys_alloc_frame();
    if (slot->hash_frame == 0) {
        return -1;
    }

    execution_slot_zero_phys_frame(slot->hash_frame);
    dst = paging_phys_to_virt(slot->hash_frame);
    if (!dst) {
        execution_slot_release_hash_backing_locked(slot);
        return -1;
    }

    header = (ayken_execution_result_hash_v1_t *)dst;
    memset(header, 0, sizeof(*header));
    header->magic = AYKEN_EXECUTION_RESULT_HASH_MAGIC;
    header->abi_version = AYKEN_EXECUTION_RESULT_HASH_VERSION;
    header->algorithm = AYKEN_RESULT_HASH_ALG_SHA256;
    header->hashed_size = slot->result_size;
    memcpy(header->digest, digest, AYKEN_SHA256_DIGEST_SIZE);

    slot->hash_size = sizeof(*header);
    slot->hashed_size = slot->result_size;
    slot->mapped_hash_va = 0;
    return 0;
}

int execution_slot_prepare_result_locked(exec_slot_t *slot)
{
    uint64_t published_size;
    uint32_t required_frames;
    uint32_t i;

    if (!slot || !slot->in_use) {
        return -1;
    }

    if (slot->state != EXEC_SLOT_RUNNING &&
        slot->state != EXEC_SLOT_COMPLETED &&
        slot->state != EXEC_SLOT_RESULT_MAPPED) {
        return -1;
    }

    if (slot->result_frame_count != 0 && slot->result_size != 0) {
        return execution_slot_prepare_hash_locked(slot);
    }

    if (slot->output_size < sizeof(ayken_execution_output_v1_t) ||
        slot->output_size > AYKEN_EXECUTION_OUTPUT_WINDOW_SIZE) {
        return -1;
    }

    published_size = slot->output_size;
    required_frames = execution_slot_frame_count_for_size(published_size);
    if (required_frames == 0 ||
        required_frames > AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES ||
        slot->output_frame_count != AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES) {
        return -1;
    }

    execution_slot_release_result_backing_locked(slot);
    execution_slot_release_hash_backing_locked(slot);

    for (i = 0; i < required_frames; ++i) {
        if (slot->output_frames[i] == 0) {
            execution_slot_release_result_backing_locked(slot);
            execution_slot_release_hash_backing_locked(slot);
            return -1;
        }

        slot->result_frames[i] = slot->output_frames[i];
        slot->output_frames[i] = 0;
    }

    execution_slot_zero_frame_tail(slot->result_frames, required_frames, published_size);

    for (i = required_frames; i < AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES; ++i) {
        uint64_t phys_addr = slot->output_frames[i];

        if (phys_addr == 0) {
            continue;
        }

        execution_slot_zero_phys_frame(phys_addr);
        phys_free_frame(phys_addr);
        slot->output_frames[i] = 0;
    }

    slot->result_frame_count = required_frames;
    slot->result_size = slot->output_size;
    slot->output_frame_count = 0;
    slot->output_size = 0;
    execution_slot_release_bcib_backing_locked(slot);
    slot->mapped_result_va = 0;
    slot->mapped_hash_va = 0;
    slot->result_map_flags = AYKEN_PTE_USER | AYKEN_PTE_READ_ONLY | AYKEN_PTE_NO_EXEC;
    return execution_slot_prepare_hash_locked(slot);
}

int execution_slot_record_result_mapping_locked(exec_slot_t *slot,
                                                uint64_t mapped_result_va,
                                                uint64_t mapped_hash_va,
                                                uint64_t map_flags)
{
    if (!slot || !slot->in_use || mapped_result_va == 0) {
        return -1;
    }

    if (slot->state == EXEC_SLOT_COMPLETED) {
        execution_slot_require_transition_locked(slot,
                                                 EXEC_SLOT_COMPLETED,
                                                 EXEC_SLOT_RESULT_MAPPED,
                                                 "execution_slot_record_result_mapping_locked");
    } else if (slot->state != EXEC_SLOT_RESULT_MAPPED) {
        return -1;
    }

    if (slot->mapped_result_va != 0 && slot->mapped_result_va != mapped_result_va) {
        return -1;
    }
    if (mapped_hash_va == 0) {
        return -1;
    }
    if (slot->mapped_hash_va != 0 && slot->mapped_hash_va != mapped_hash_va) {
        return -1;
    }

    slot->mapped_result_va = mapped_result_va;
    slot->mapped_hash_va = mapped_hash_va;
    slot->result_map_flags = map_flags;
    return 0;
}

uint32_t execution_slot_prepare_process_exit_locked(uint64_t process_pid,
                                                    uint64_t *result_vas,
                                                    uint64_t *hash_vas,
                                                    uint32_t max_result_vas)
{
    uint32_t i;
    uint32_t result_count = 0;

    if (process_pid == 0) {
        return 0;
    }

    for (i = 0; i < AYKEN_MAX_EXECUTION_SLOTS; ++i) {
        exec_slot_t *slot = &g_execution_slots[i];
        int owner_match;
        int target_match;

        if (!slot->in_use) {
            continue;
        }

        owner_match = slot->owner_pid == process_pid;
        target_match = slot->target_context_id == process_pid;
        if (!owner_match && !target_match) {
            continue;
        }

        if (slot->state == EXEC_SLOT_CREATED ||
            slot->state == EXEC_SLOT_READY ||
            slot->state == EXEC_SLOT_RUNNING) {
            execution_slot_require_finish_locked(slot,
                                                 EXEC_SLOT_ABORTED,
                                                 "execution_slot_prepare_process_exit_locked");
        }

        if (owner_match &&
            slot->mapped_result_va != 0 &&
            result_vas != NULL &&
            result_count < max_result_vas) {
            result_vas[result_count++] = slot->mapped_result_va;
            if (hash_vas != NULL) {
                hash_vas[result_count - 1] = slot->mapped_hash_va;
            }
        }
    }

    return result_count;
}

uint32_t execution_slot_release_owned_by_owner_locked(uint64_t owner_pid)
{
    uint32_t i;
    uint32_t released = 0;

    if (owner_pid == 0) {
        return 0;
    }

    for (i = 0; i < AYKEN_MAX_EXECUTION_SLOTS; ++i) {
        exec_slot_t *slot = &g_execution_slots[i];

        if (!slot->in_use || slot->owner_pid != owner_pid) {
            continue;
        }

        execution_slot_release_locked(slot);
        released++;
    }

    return released;
}

exec_slot_t *execution_slot_find_locked(uint64_t execution_id)
{
    uint32_t i;

    if (execution_id == 0) {
        return NULL;
    }

    for (i = 0; i < AYKEN_MAX_EXECUTION_SLOTS; ++i) {
        if (g_execution_slots[i].in_use &&
            g_execution_slots[i].execution_id == execution_id) {
            return &g_execution_slots[i];
        }
    }

    return NULL;
}

exec_slot_t *execution_slot_pickup_locked(uint64_t context_id)
{
    exec_slot_t *slot = execution_slot_dequeue_locked(context_id);

    if (!slot) {
        return NULL;
    }

    execution_slot_require_transition_locked(slot,
                                             EXEC_SLOT_READY,
                                             EXEC_SLOT_RUNNING,
                                             "execution_slot_pickup_locked");

    return slot;
}

uint32_t execution_slot_process_timeouts_locked(uint64_t now_tick)
{
    uint32_t i;
    uint32_t timed_out = 0;

    for (i = 0; i < AYKEN_MAX_EXECUTION_SLOTS; ++i) {
        exec_slot_t *slot = &g_execution_slots[i];

        if (!slot->in_use ||
            slot->deadline_tick == 0 ||
            execution_slot_state_is_terminal(slot->state)) {
            continue;
        }

        if (now_tick < slot->deadline_tick) {
            continue;
        }

        if (slot->state != EXEC_SLOT_READY && slot->state != EXEC_SLOT_RUNNING) {
            continue;
        }

        execution_slot_require_finish_locked(slot,
                                             EXEC_SLOT_TIMEOUT,
                                             "execution_slot_process_timeouts_locked");
        timed_out++;
    }

    return timed_out;
}

int execution_slot_transition_locked(exec_slot_t *slot,
                                     exec_slot_state_t expected_from,
                                     exec_slot_state_t next_state)
{
    if (!slot || !slot->in_use) {
        return -1;
    }

    if (slot->state != expected_from) {
        return -1;
    }

    if (!execution_slot_can_transition(expected_from, next_state)) {
        return -1;
    }

    execution_slot_trace_append_locked(slot, expected_from, next_state);
    slot->state = next_state;
    return 0;
}

int execution_slot_require_transition_locked(exec_slot_t *slot,
                                             exec_slot_state_t expected_from,
                                             exec_slot_state_t next_state,
                                             const char *site)
{
    if (execution_slot_transition_locked(slot, expected_from, next_state) != 0) {
        execution_slot_runtime_panic(site, slot, expected_from, next_state);
    }

    return 0;
}

int execution_slot_state_is_terminal(exec_slot_state_t state)
{
    return state == EXEC_SLOT_COMPLETED ||
           state == EXEC_SLOT_FAILED ||
           state == EXEC_SLOT_TIMEOUT ||
           state == EXEC_SLOT_RESULT_MAPPED ||
           state == EXEC_SLOT_ABORTED;
}

uint32_t execution_slot_trace_count_locked(const exec_slot_t *slot)
{
    if (!slot || !slot->in_use) {
        return 0;
    }

    return slot->trace_count;
}

int execution_slot_trace_get_locked(const exec_slot_t *slot,
                                    uint32_t ordinal,
                                    execution_trace_entry_t *entry)
{
    uint32_t index;
    uint32_t oldest_index;

    if (!slot || !slot->in_use || !entry || ordinal >= slot->trace_count) {
        return -1;
    }

    oldest_index = execution_slot_trace_oldest_index_locked(slot);
    index = (oldest_index + ordinal) % AYKEN_EXECUTION_TRACE_CAPACITY;
    *entry = slot->trace_entries[index];
    return 0;
}

int execution_slot_verify_global_invariants_locked(void)
{
    uint32_t i;

    for (i = 0; i < AYKEN_MAX_EXECUTION_SLOTS; ++i) {
        exec_slot_t *slot = &g_execution_slots[i];
        uint32_t j;
        uint64_t previous_tick = 0;
        exec_slot_state_t previous_state = EXEC_SLOT_CREATED;
        int have_previous = 0;
        int immutable_seen = 0;

        if (!slot->in_use) {
            continue;
        }

        if (slot->execution_id == 0) {
            return -1;
        }

        for (j = i + 1; j < AYKEN_MAX_EXECUTION_SLOTS; ++j) {
            exec_slot_t *other = &g_execution_slots[j];

            if (other->in_use && other->execution_id == slot->execution_id) {
                return -1;
            }
        }

        if (slot->state == EXEC_SLOT_RUNNING) {
            uint32_t running_for_target = 0;

            if (slot->target_context_id == 0) {
                return -1;
            }

            for (j = 0; j < AYKEN_MAX_EXECUTION_SLOTS; ++j) {
                exec_slot_t *other = &g_execution_slots[j];

                if (!other->in_use ||
                    other->state != EXEC_SLOT_RUNNING ||
                    other->target_context_id != slot->target_context_id) {
                    continue;
                }
                running_for_target++;
            }

            if (running_for_target > 1) {
                return -1;
            }
        }

        if ((slot->state == EXEC_SLOT_COMPLETED ||
             slot->state == EXEC_SLOT_RESULT_MAPPED) &&
            (slot->result_size == 0 ||
             slot->result_frame_count == 0 ||
             slot->hash_frame == 0 ||
             slot->hash_size != sizeof(ayken_execution_result_hash_v1_t))) {
            return -1;
        }

        if (slot->state == EXEC_SLOT_RESULT_MAPPED &&
            (slot->mapped_result_va == 0 || slot->mapped_hash_va == 0)) {
            return -1;
        }

        for (j = 0; j < slot->trace_count; ++j) {
            execution_trace_entry_t entry = {0};

            if (execution_slot_trace_get_locked(slot, j, &entry) != 0) {
                return -1;
            }

            if (entry.execution_id != slot->execution_id ||
                entry.generation != slot->generation ||
                !execution_slot_can_transition((exec_slot_state_t)entry.from_state,
                                               (exec_slot_state_t)entry.to_state)) {
                return -1;
            }

            if (have_previous) {
                if (entry.tick < previous_tick ||
                    (exec_slot_state_t)entry.from_state != previous_state ||
                    immutable_seen) {
                    return -1;
                }
            }

            previous_tick = entry.tick;
            previous_state = (exec_slot_state_t)entry.to_state;
            have_previous = 1;
            if (execution_slot_trace_state_is_immutable(previous_state)) {
                immutable_seen = 1;
            }
        }

        if (have_previous && previous_state != slot->state) {
            return -1;
        }
    }

    return 0;
}

execution_context_queue_t *execution_slot_find_queue_locked(uint64_t context_id)
{
    uint32_t i;

    if (context_id == 0) {
        return NULL;
    }

    for (i = 0; i < AYKEN_MAX_EXECUTION_CONTEXT_QUEUES; ++i) {
        if (g_execution_queues[i].in_use &&
            g_execution_queues[i].context_id == context_id) {
            return &g_execution_queues[i];
        }
    }

    return NULL;
}

int execution_slot_enqueue_locked(exec_slot_t *slot)
{
    execution_context_queue_t *queue;
    uint32_t slot_index;

    if (!slot || !slot->in_use) {
        return -1;
    }

    slot_index = execution_slot_index(slot);
    if (slot_index == AYKEN_EXECUTION_INVALID_INDEX) {
        return -1;
    }
    if (slot->queue_next_index != AYKEN_EXECUTION_INVALID_INDEX) {
        return -1;
    }

    queue = execution_slot_find_queue_locked(slot->target_context_id);
    if (!queue) {
        queue = execution_slot_alloc_queue_locked(slot->target_context_id);
    }
    if (!queue) {
        return -1;
    }

    if (queue->tail_index == AYKEN_EXECUTION_INVALID_INDEX) {
        queue->head_index = slot_index;
        queue->tail_index = slot_index;
    } else {
        g_execution_slots[queue->tail_index].queue_next_index = slot_index;
        queue->tail_index = slot_index;
    }

    slot->queue_next_index = AYKEN_EXECUTION_INVALID_INDEX;
    queue->depth++;
    return 0;
}

exec_slot_t *execution_slot_dequeue_locked(uint64_t context_id)
{
    execution_context_queue_t *queue;
    exec_slot_t *slot;
    uint32_t head_index;

    queue = execution_slot_find_queue_locked(context_id);
    if (!queue || queue->head_index == AYKEN_EXECUTION_INVALID_INDEX) {
        return NULL;
    }

    head_index = queue->head_index;
    slot = &g_execution_slots[head_index];

    queue->head_index = slot->queue_next_index;
    if (queue->head_index == AYKEN_EXECUTION_INVALID_INDEX) {
        queue->tail_index = AYKEN_EXECUTION_INVALID_INDEX;
    }
    if (queue->depth > 0) {
        queue->depth--;
    }

    slot->queue_next_index = AYKEN_EXECUTION_INVALID_INDEX;

    if (queue->depth == 0) {
        execution_slot_zero_queue(queue);
    }

    return slot;
}

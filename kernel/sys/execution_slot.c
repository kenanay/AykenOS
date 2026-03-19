// kernel/sys/execution_slot.c
// Execution-slot runtime state skeleton for Phase 10-B / 10-C hardening.

#include <stddef.h>
#include <stdint.h>

#include "../arch/x86_64/cpu.h"
#include "../include/execution_slot.h"
#include "../include/mm.h"
#include "../include/proc.h"

#define memset __builtin_memset
#define memcpy __builtin_memcpy

static exec_slot_t g_execution_slots[AYKEN_MAX_EXECUTION_SLOTS];
static execution_context_queue_t g_execution_queues[AYKEN_MAX_EXECUTION_CONTEXT_QUEUES];
static uint64_t g_next_execution_id = 1;

static uint64_t execution_slot_read_rflags(void)
{
    uint64_t rflags = 0;
    __asm__ volatile("pushfq; popq %0" : "=r"(rflags));
    return rflags;
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
    for (i = 0; i < AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES; ++i) {
        slot->bcib_frames[i] = 0;
    }
    slot->bcib_size = 0;
    slot->result_phys = 0;
    slot->result_size = 0;
    slot->mapped_result_va = 0;
    slot->result_map_flags = 0;
    slot->error_code = 0;
    slot->queue_next_index = AYKEN_EXECUTION_INVALID_INDEX;
    slot->wait_key.execution_id = 0;
    slot->wait_key.generation = slot->generation;
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

static uint32_t execution_slot_bcib_frame_count_for_size(uint64_t graph_size)
{
    if (graph_size == 0) {
        return 0;
    }

    return (uint32_t)((graph_size + (AYKEN_FRAME_SIZE - 1)) / AYKEN_FRAME_SIZE);
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

static int execution_slot_finish_locked(exec_slot_t *slot, exec_slot_state_t next_state)
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
    }

    slot->deadline_tick = 0;
    execution_slot_clear_target_latch_locked(slot);
    proc_wake_waiters(&slot->wait_key);
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
        slot->execution_id = g_next_execution_id++;
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
    slot->execution_id = 0;
    slot->owner_pid = 0;
    slot->target_context_id = 0;
    slot->created_tick = 0;
    slot->deadline_tick = 0;
    slot->state = EXEC_SLOT_CREATED;
    slot->result_phys = 0;
    slot->result_size = 0;
    slot->mapped_result_va = 0;
    slot->result_map_flags = 0;
    slot->error_code = 0;
    slot->queue_next_index = AYKEN_EXECUTION_INVALID_INDEX;
    slot->wait_key.execution_id = 0;
    slot->wait_key.generation = slot->generation;
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

    frame_count = execution_slot_bcib_frame_count_for_size(graph_size);
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

    if (execution_slot_transition_locked(slot, EXEC_SLOT_READY, EXEC_SLOT_RUNNING) != 0) {
        return NULL;
    }

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

        if (execution_slot_finish_locked(slot, EXEC_SLOT_TIMEOUT) == 0) {
            timed_out++;
        }
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

    slot->state = next_state;
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

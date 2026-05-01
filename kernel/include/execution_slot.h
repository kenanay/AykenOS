#ifndef AYKEN_EXECUTION_SLOT_H
#define AYKEN_EXECUTION_SLOT_H

#include <stddef.h>
#include <stdint.h>

#include "execution_output_abi.h"
#include "execution_output_structured_abi.h"
#include "execution_result_hash_abi.h"

#define AYKEN_MAX_EXECUTION_SLOTS 64u
#define AYKEN_MAX_EXECUTION_CONTEXT_QUEUES 64u
#define AYKEN_EXECUTION_INVALID_INDEX UINT32_MAX
#define AYKEN_EXECUTION_TRACE_CAPACITY 8u

typedef enum {
    EXEC_SLOT_CREATED = 0,
    EXEC_SLOT_READY,
    EXEC_SLOT_RUNNING,
    EXEC_SLOT_COMPLETED,
    EXEC_SLOT_FAILED,
    EXEC_SLOT_TIMEOUT,
    EXEC_SLOT_RESULT_MAPPED,
    EXEC_SLOT_ABORTED,
} exec_slot_state_t;

typedef enum {
    EXEC_TRACE_ACTOR_NONE = 0,
    EXEC_TRACE_ACTOR_SUBMIT,
    EXEC_TRACE_ACTOR_PICKUP,
    EXEC_TRACE_ACTOR_COMPLETE,
    EXEC_TRACE_ACTOR_WAIT_RESULT,
    EXEC_TRACE_ACTOR_TIMEOUT_IRQ,
    EXEC_TRACE_ACTOR_EXIT,
    EXEC_TRACE_ACTOR_VALIDATION,
} execution_trace_actor_t;

typedef struct execution_trace_entry {
    uint64_t tick;
    uint64_t execution_id;
    uint64_t generation;
    uint8_t actor;
    uint8_t from_state;
    uint8_t to_state;
    uint8_t reserved0[5];
} execution_trace_entry_t;

typedef struct execution_wait_key {
    uint64_t execution_id;
    uint64_t generation;
} execution_wait_key_t;

typedef struct exec_slot {
    uint8_t in_use;
    uint8_t reserved0[7];
    uint64_t execution_id;
    uint64_t generation;
    uint64_t owner_pid;
    uint64_t target_context_id;
    uint64_t created_tick;
    uint64_t deadline_tick;
    exec_slot_state_t state;
    uint32_t bcib_frame_count;
    uint32_t result_frame_count;
    uint32_t output_frame_count;
    uint64_t bcib_frames[AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES];
    uint64_t bcib_size;
    uint64_t result_frames[AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES];
    uint64_t result_size;
    uint64_t output_frames[AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES];
    uint64_t output_size;
    uint64_t hash_frame;
    uint64_t hash_size;
    uint64_t hashed_size;
    uint64_t mapped_result_va;
    uint64_t mapped_hash_va;
    uint64_t result_map_flags;
    uint32_t error_code;
    uint32_t queue_next_index;
    execution_wait_key_t wait_key;
    uint32_t trace_count;
    uint32_t trace_head;
    execution_trace_entry_t trace_entries[AYKEN_EXECUTION_TRACE_CAPACITY];
} exec_slot_t;

typedef struct execution_context_queue {
    uint8_t in_use;
    uint8_t reserved0[7];
    uint64_t context_id;
    uint32_t head_index;
    uint32_t tail_index;
    uint32_t depth;
    uint32_t reserved1;
} execution_context_queue_t;

typedef struct execution_slot_guard {
    uint64_t saved_rflags;
    uint8_t interrupts_were_enabled;
    uint8_t entered;
    uint8_t reserved0[6];
} execution_slot_guard_t;

typedef struct execution_slot_trace_scope {
    execution_trace_actor_t previous_actor;
    uint8_t active;
    uint8_t reserved0[7];
} execution_slot_trace_scope_t;

void execution_slots_init(void);
uint32_t execution_slots_capacity(void);
uint32_t execution_slot_queue_capacity(void);

void execution_slot_enter_critical(execution_slot_guard_t *guard);
void execution_slot_exit_critical(execution_slot_guard_t *guard);
void execution_slot_run_fail_closed_selftest(void);
void execution_slot_trace_scope_enter(execution_slot_trace_scope_t *scope,
                                      execution_trace_actor_t actor);
void execution_slot_trace_scope_exit(execution_slot_trace_scope_t *scope);

exec_slot_t *execution_slot_alloc_locked(uint64_t owner_pid, uint64_t target_context_id);
void execution_slot_release_locked(exec_slot_t *slot);
exec_slot_t *execution_slot_find_locked(uint64_t execution_id);
exec_slot_t *execution_slot_pickup_locked(uint64_t context_id);
uint32_t execution_slot_process_timeouts_locked(uint64_t now_tick);
int execution_slot_store_bcib_locked(exec_slot_t *slot,
                                     const void *bcib_graph,
                                     uint64_t graph_size);
int execution_slot_prepare_output_locked(exec_slot_t *slot);
int execution_slot_write_output_v1_locked(exec_slot_t *slot,
                                          const void *payload,
                                          uint64_t payload_size);
int execution_slot_validate_output_locked(exec_slot_t *slot, uint64_t *published_size);
int execution_slot_can_publish_locked(const exec_slot_t *slot);
uint64_t execution_slot_result_va_locked(const exec_slot_t *slot);
uint64_t execution_slot_result_hash_va_locked(const exec_slot_t *slot);
uint32_t execution_slot_prepare_process_exit_locked(uint64_t process_pid,
                                                    uint64_t *result_vas,
                                                    uint64_t *hash_vas,
                                                    uint32_t max_result_vas);
uint32_t execution_slot_release_owned_by_owner_locked(uint64_t owner_pid);
int execution_slot_prepare_result_locked(exec_slot_t *slot);
int execution_slot_record_result_mapping_locked(exec_slot_t *slot,
                                                uint64_t mapped_result_va,
                                                uint64_t mapped_hash_va,
                                                uint64_t map_flags);
int execution_slot_finish_locked(exec_slot_t *slot, exec_slot_state_t next_state);
int execution_slot_require_finish_locked(exec_slot_t *slot,
                                         exec_slot_state_t next_state,
                                         const char *site);
int execution_slot_transition_locked(exec_slot_t *slot,
                                     exec_slot_state_t expected_from,
                                     exec_slot_state_t next_state);
int execution_slot_require_transition_locked(exec_slot_t *slot,
                                             exec_slot_state_t expected_from,
                                             exec_slot_state_t next_state,
                                             const char *site);
int execution_slot_state_is_terminal(exec_slot_state_t state);
uint32_t execution_slot_trace_count_locked(const exec_slot_t *slot);
int execution_slot_trace_get_locked(const exec_slot_t *slot,
                                    uint32_t ordinal,
                                    execution_trace_entry_t *entry);
int execution_slot_verify_global_invariants_locked(void);

execution_context_queue_t *execution_slot_find_queue_locked(uint64_t context_id);
int execution_slot_enqueue_locked(exec_slot_t *slot);
exec_slot_t *execution_slot_dequeue_locked(uint64_t context_id);

#endif // AYKEN_EXECUTION_SLOT_H

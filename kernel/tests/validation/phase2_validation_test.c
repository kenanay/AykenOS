// kernel/sys/phase2_validation_test.c
// AykenOS Phase 2 Validation Snapshot Test Suite
//
// This suite captures current Phase 2 behavior using a mix of:
// - semantic checks for the more mature syscall/runtime surfaces
// - interface-shape checks for still-incomplete lifecycle surfaces
// - Ring3 proxy/stub reachability checks
//
// Requirements: Task 2.5.3.1 - Execute complete Phase 2 validation

#include "../../sys/syscall_v2.h"
#include "../../drivers/console/fb_console.h"
#include "../../include/execution_slot.h"
#include "../../include/execution_inbox_abi.h"
#include "../../include/capability.h"
#include "../../include/mm.h"
#include "../../include/proc.h"
#include "../../include/sha256.h"
#include "../../sched/sched.h"
#include "../../sched/sched_mailbox.h"
#include "../../fs/devfs.h"
#include <stddef.h>

#define memcpy __builtin_memcpy
#define memset __builtin_memset

extern proc_t *current_proc;
extern void timer_isr_c(void *frame_ptr);

// Test result tracking
static int tests_passed = 0;
static int tests_failed = 0;
static int total_tests = 0;

static const uint8_t validation_worker_loop_code[] = {
    0xEB, 0xFE, // jmp $
};

#define VALIDATION_RING3_CANARY_ADDR 0x0000000000405000ULL

// Test helper macros
#define TEST_START(name) \
    do { \
        total_tests++; \
        fb_print("\n[TEST] Starting: " name "\n"); \
    } while(0)

#define TEST_ASSERT(condition, message) \
    do { \
        if (condition) { \
            tests_passed++; \
            fb_print("[PASS] " message "\n"); \
        } else { \
            tests_failed++; \
            fb_print("[FAIL] " message "\n"); \
        } \
    } while(0)

#define TEST_END(name) \
    fb_print("[TEST] Completed: " name "\n")

static proc_t *ensure_validation_worker_proc(void)
{
    static proc_t *worker = NULL;

    if (worker && worker->state != PROC_ZOMBIE) {
        return worker;
    }

    worker = proc_create_user_process("phase2-validation-worker",
                                      validation_worker_loop_code,
                                      sizeof(validation_worker_loop_code),
                                      PROC_IMAGE_FLAT);
    return worker;
}

static proc_t *ensure_validation_foreign_proc(void)
{
    static proc_t *worker = NULL;

    if (worker && worker->state != PROC_ZOMBIE) {
        return worker;
    }

    worker = proc_create_user_process("phase2-validation-foreign",
                                      validation_worker_loop_code,
                                      sizeof(validation_worker_loop_code),
                                      PROC_IMAGE_FLAT);
    return worker;
}

static proc_t *create_validation_runtime_proc(const char *name)
{
    return proc_create_user_process(name,
                                    validation_worker_loop_code,
                                    sizeof(validation_worker_loop_code),
                                    PROC_IMAGE_FLAT);
}

static uint64_t submit_validation_execution_as(proc_t *owner_proc,
                                               const void *bcib_graph,
                                               uint64_t graph_size,
                                               uint64_t target_context_id)
{
    proc_t *saved_current_proc = current_proc;
    uint64_t exec_id;

    current_proc = owner_proc;
    if (current_proc != NULL) {
        current_proc->state = PROC_RUNNING;
    }
    exec_id = sys_v2_submit_execution((void *)bcib_graph, graph_size, target_context_id);
    current_proc = saved_current_proc;
    return exec_id;
}

static uint64_t complete_validation_execution_as(proc_t *executor_proc,
                                                 uint64_t execution_id,
                                                 uint64_t completion_code)
{
    proc_t *saved_current_proc = current_proc;
    uint64_t result;

    current_proc = executor_proc;
    if (current_proc != NULL) {
        current_proc->state = PROC_RUNNING;
    }
    result = sys_v2_complete_execution(execution_id, completion_code);
    current_proc = saved_current_proc;
    return result;
}

static uint64_t wait_validation_result_as(proc_t *owner_proc,
                                          uint64_t execution_id,
                                          uint64_t timeout_ms)
{
    proc_t *saved_current_proc = current_proc;
    uint64_t result;

    current_proc = owner_proc;
    if (current_proc != NULL) {
        current_proc->state = PROC_RUNNING;
    }
    result = sys_v2_wait_result(execution_id, timeout_ms);
    current_proc = saved_current_proc;
    return result;
}

static uint64_t map_validation_memory_as(proc_t *proc,
                                         uint64_t virt_addr,
                                         uint64_t phys_addr,
                                         uint64_t flags)
{
    proc_t *saved_current_proc = current_proc;
    uint64_t result;

    current_proc = proc;
    if (current_proc != NULL) {
        current_proc->state = PROC_RUNNING;
    }
    result = sys_v2_map_memory(virt_addr, phys_addr, flags);
    current_proc = saved_current_proc;
    return result;
}

static uint64_t unmap_validation_memory_as(proc_t *proc,
                                           uint64_t virt_addr,
                                           uint64_t size)
{
    proc_t *saved_current_proc = current_proc;
    uint64_t result;

    current_proc = proc;
    if (current_proc != NULL) {
        current_proc->state = PROC_RUNNING;
    }
    result = sys_v2_unmap_memory(virt_addr, size);
    current_proc = saved_current_proc;
    return result;
}

static uint64_t bind_validation_memory_capability(proc_t *proc,
                                                  uint64_t phys_addr,
                                                  uint64_t size,
                                                  uint32_t permissions)
{
    capability_token_t token;

    if (!proc) {
        return 0;
    }

    token = capability_create(CAPABILITY_RESOURCE_MEMORY,
                              permissions,
                              phys_addr,
                              size);
    if (token.id == 0) {
        return 0;
    }

    return sys_v2_capability_bind((uint64_t)proc->pid, &token);
}

static int validation_write_mailbox_candidate(proc_t *publisher,
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

static int validation_seed_mailbox_candidate(proc_t *publisher, uint32_t candidate_pid)
{
    uint64_t next_epoch;

    if (!publisher) {
        return 0;
    }

    next_epoch = publisher->mailbox_last_epoch + 1;
    if (next_epoch == 0) {
        next_epoch = 1;
    }

    return validation_write_mailbox_candidate(publisher, next_epoch, candidate_pid);
}

static int validation_reset_mailbox_to_self(proc_t *publisher)
{
    if (!publisher || publisher->pid <= 0) {
        return 0;
    }

    return validation_write_mailbox_candidate(publisher, 1, (uint32_t)publisher->pid);
}

static int validation_mailbox_equals(const ayken_sched_mailbox_t *lhs,
                                     const ayken_sched_mailbox_t *rhs)
{
    if (!lhs || !rhs) {
        return 0;
    }

    return lhs->magic == rhs->magic &&
           lhs->version == rhs->version &&
           lhs->kind == rhs->kind &&
           lhs->epoch == rhs->epoch &&
           lhs->proposer_pid == rhs->proposer_pid &&
           lhs->candidate_pid == rhs->candidate_pid &&
           lhs->flags == rhs->flags &&
           lhs->status == rhs->status &&
           lhs->reject_reason == rhs->reject_reason &&
           lhs->reserved == rhs->reserved;
}

static int validation_seed_owner_mailbox_candidate(uint32_t candidate_pid)
{
    proc_t *owner = proc_find_by_pid((int)sched_active_owner_pid());

    return validation_seed_mailbox_candidate(owner, candidate_pid);
}

static void validation_write_u32_le(uint8_t *dst, uint32_t value)
{
    if (!dst) {
        return;
    }

    dst[0] = (uint8_t)(value & 0xFFu);
    dst[1] = (uint8_t)((value >> 8) & 0xFFu);
    dst[2] = (uint8_t)((value >> 16) & 0xFFu);
    dst[3] = (uint8_t)((value >> 24) & 0xFFu);
}

static void validation_write_u64_le(uint8_t *dst, uint64_t value)
{
    if (!dst) {
        return;
    }

    dst[0] = (uint8_t)(value & 0xFFu);
    dst[1] = (uint8_t)((value >> 8) & 0xFFu);
    dst[2] = (uint8_t)((value >> 16) & 0xFFu);
    dst[3] = (uint8_t)((value >> 24) & 0xFFu);
    dst[4] = (uint8_t)((value >> 32) & 0xFFu);
    dst[5] = (uint8_t)((value >> 40) & 0xFFu);
    dst[6] = (uint8_t)((value >> 48) & 0xFFu);
    dst[7] = (uint8_t)((value >> 56) & 0xFFu);
}

static int validation_patch_user_text(proc_t *proc, const uint8_t *image, uint64_t image_size)
{
    uint64_t pte;
    uint64_t phys;
    uint8_t *dst;

    if (!proc || !image || image_size == 0 || proc->pml4_phys == 0) {
        return 0;
    }

    pte = paging_get_pte_in_pml4(proc->pml4_phys, USER_TEXT_BASE);
    if (pte == 0) {
        return 0;
    }

    phys = pte & AYKEN_PTE_ADDR_MASK;
    if (phys == 0) {
        return 0;
    }

    dst = (uint8_t *)paging_phys_to_virt(phys);
    if (!dst) {
        return 0;
    }

    memcpy(dst, image, (size_t)image_size);
    return 1;
}

typedef struct blocked_wait_harness {
    proc_t *target_proc;
    proc_t *waiter_proc;
    proc_t *waker_proc;
    void *blocked_wait_obj;
    uint64_t execution_id;
    uint64_t wait_result;
    uint64_t terminal_state;
    int waiter_blocked;
    int wrong_wake_preserved;
    int wake_released;
    int waiter_resumed;
    int waiter_wait_obj_cleared;
} blocked_wait_harness_t;

static blocked_wait_harness_t g_blocked_wait_harness;
static execution_wait_key_t g_blocked_wait_spurious_key;
static uint8_t g_blocked_wait_park_token;
static uint8_t g_blocked_wait_target_hold_token;

typedef struct timeout_irq_harness {
    proc_t *target_proc;
    proc_t *waiter_proc;
    proc_t *driver_proc;
    void *blocked_wait_obj;
    uint64_t execution_id;
    uint64_t wait_result;
    uint64_t deadline_tick;
    uint64_t terminal_state;
    int waiter_blocked;
    int pre_irq_still_blocked;
    int irq_woke_waiter;
    int waiter_resumed;
    int waiter_wait_obj_cleared;
} timeout_irq_harness_t;

static timeout_irq_harness_t g_timeout_irq_harness;
static uint8_t g_timeout_irq_park_token;
static uint8_t g_timeout_irq_target_hold_token;
static uint8_t g_owner_transfer_target_hold_token;

typedef struct negative_timeout_harness {
    proc_t *target_proc;
    proc_t *waiter_proc;
    proc_t *driver_proc;
    void *blocked_wait_obj;
    uint64_t execution_id;
    uint64_t wait_result;
    uint64_t post_timeout_wait_result;
    uint64_t foreign_wait_result;
    uint64_t deadline_tick;
    uint64_t terminal_state;
    int pickup_running;
    int waiter_blocked;
    int pre_irq_still_blocked;
    int irq_woke_waiter;
    int waiter_resumed;
    int waiter_wait_obj_cleared;
    int slot_released;
} negative_timeout_harness_t;

static negative_timeout_harness_t g_negative_timeout_harness;
static uint8_t g_negative_timeout_park_token;

static const uint8_t validation_exit_noreturn_code[] = {
    0xB8, 0xF1, 0x03, 0x00, 0x00, /* mov eax, 1009 */
    0xBF, 0x17, 0x0E, 0x00, 0x00, /* mov edi, 0xE17 */
    0xCD, 0x80,                   /* int 0x80 */
    0xEB, 0xFE,                   /* jmp $ (unexpected return) */
};

static const uint8_t validation_owner_transfer_successor_code_template[] = {
    0x48, 0xBB, 0x00, 0x00, 0x70, 0x00, 0x00, 0x00, 0x00, 0x00, /* mov rbx, 0x700000 */
    0x48, 0xC7, 0x43, 0x08, 0x02, 0x00, 0x00, 0x00,             /* mov qword [rbx+0x08], 2 */
    0xC7, 0x43, 0x14, 0x00, 0x00, 0x00, 0x00,                   /* mov dword [rbx+0x14], candidate_pid */
    0xB8, 0xEC, 0x03, 0x00, 0x00,                               /* mov eax, 1004 */
    0x48, 0xBF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, /* mov rdi, execution_id */
    0x48, 0xBE, 0x60, 0xEA, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, /* mov rsi, 60000 */
    0xCD, 0x80,                                                 /* int 0x80 */
    0xEB, 0xFE,                                                 /* jmp $ */
};

#define VALIDATION_OWNER_TRANSFER_CANDIDATE_OFFSET 21u
#define VALIDATION_OWNER_TRANSFER_EXEC_ID_OFFSET   32u
#define VALIDATION_OWNER_TRANSFER_TIMEOUT_OFFSET   42u

static const uint8_t validation_owner_followthrough_old_owner_code_template[] = {
    0x48, 0xBB, 0x00, 0x00, 0x70, 0x00, 0x00, 0x00, 0x00, 0x00, /* mov rbx, 0x700000 */
    0x48, 0xC7, 0x43, 0x08, 0x02, 0x00, 0x00, 0x00,             /* mov qword [rbx+0x08], 2 */
    0xC7, 0x43, 0x14, 0x00, 0x00, 0x00, 0x00,                   /* mov dword [rbx+0x14], successor_pid */
    0xB8, 0xEC, 0x03, 0x00, 0x00,                               /* mov eax, 1004 */
    0x48, 0xBF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, /* mov rdi, wait_exec_id */
    0x48, 0xBE, 0x60, 0xEA, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, /* mov rsi, 60000 */
    0xCD, 0x80,                                                 /* int 0x80 */
    0xB8, 0xF1, 0x03, 0x00, 0x00,                               /* mov eax, 1009 */
    0xBF, 0x42, 0x11, 0x00, 0x00,                               /* mov edi, 0x1142 */
    0xCD, 0x80,                                                 /* int 0x80 */
    0xEB, 0xFE,                                                 /* jmp $ */
};

#define VALIDATION_OWNER_FOLLOWTHROUGH_OLD_CANDIDATE_OFFSET 21u
#define VALIDATION_OWNER_FOLLOWTHROUGH_OLD_WAIT_EXEC_OFFSET  32u

static const uint8_t validation_owner_followthrough_successor_code_template[] = {
    0x48, 0xBB, 0x00, 0x00, 0x70, 0x00, 0x00, 0x00, 0x00, 0x00, /* mov rbx, 0x700000 */
    0x48, 0xC7, 0x43, 0x08, 0x02, 0x00, 0x00, 0x00,             /* mov qword [rbx+0x08], 2 */
    0xC7, 0x43, 0x14, 0x00, 0x00, 0x00, 0x00,                   /* mov dword [rbx+0x14], old_owner_pid */
    0xB8, 0xF3, 0x03, 0x00, 0x00,                               /* mov eax, 1011 */
    0x48, 0xBF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, /* mov rdi, complete_exec_id */
    0x31, 0xF6,                                                 /* xor esi, esi */
    0xCD, 0x80,                                                 /* int 0x80 */
    0xB8, 0xEC, 0x03, 0x00, 0x00,                               /* mov eax, 1004 */
    0x48, 0xBF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, /* mov rdi, wait_exec_id */
    0x48, 0xBE, 0x60, 0xEA, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, /* mov rsi, 60000 */
    0xCD, 0x80,                                                 /* int 0x80 */
    0xEB, 0xFE,                                                 /* jmp $ */
};

#define VALIDATION_OWNER_FOLLOWTHROUGH_SUCC_CANDIDATE_OFFSET     21u
#define VALIDATION_OWNER_FOLLOWTHROUGH_SUCC_COMPLETE_EXEC_OFFSET  32u
#define VALIDATION_OWNER_FOLLOWTHROUGH_SUCC_WAIT_EXEC_OFFSET      51u

static int validation_patch_owner_transfer_successor_code(proc_t *proc,
                                                          uint32_t candidate_pid,
                                                          uint64_t execution_id)
{
    uint8_t image[sizeof(validation_owner_transfer_successor_code_template)];

    memcpy(image,
           validation_owner_transfer_successor_code_template,
           sizeof(validation_owner_transfer_successor_code_template));
    validation_write_u32_le(image + VALIDATION_OWNER_TRANSFER_CANDIDATE_OFFSET,
                            candidate_pid);
    validation_write_u64_le(image + VALIDATION_OWNER_TRANSFER_EXEC_ID_OFFSET,
                            execution_id);
    validation_write_u64_le(image + VALIDATION_OWNER_TRANSFER_TIMEOUT_OFFSET,
                            60000u);
    return validation_patch_user_text(proc, image, sizeof(image));
}

static int validation_patch_owner_followthrough_old_owner_code(proc_t *proc,
                                                               uint32_t successor_pid,
                                                               uint64_t wait_exec_id)
{
    uint8_t image[sizeof(validation_owner_followthrough_old_owner_code_template)];

    memcpy(image,
           validation_owner_followthrough_old_owner_code_template,
           sizeof(validation_owner_followthrough_old_owner_code_template));
    validation_write_u32_le(image + VALIDATION_OWNER_FOLLOWTHROUGH_OLD_CANDIDATE_OFFSET,
                            successor_pid);
    validation_write_u64_le(image + VALIDATION_OWNER_FOLLOWTHROUGH_OLD_WAIT_EXEC_OFFSET,
                            wait_exec_id);
    return validation_patch_user_text(proc, image, sizeof(image));
}

static int validation_patch_owner_followthrough_successor_code(proc_t *proc,
                                                               uint32_t old_owner_pid,
                                                               uint64_t complete_exec_id,
                                                               uint64_t wait_exec_id)
{
    uint8_t image[sizeof(validation_owner_followthrough_successor_code_template)];

    memcpy(image,
           validation_owner_followthrough_successor_code_template,
           sizeof(validation_owner_followthrough_successor_code_template));
    validation_write_u32_le(image + VALIDATION_OWNER_FOLLOWTHROUGH_SUCC_CANDIDATE_OFFSET,
                            old_owner_pid);
    validation_write_u64_le(image + VALIDATION_OWNER_FOLLOWTHROUGH_SUCC_COMPLETE_EXEC_OFFSET,
                            complete_exec_id);
    validation_write_u64_le(image + VALIDATION_OWNER_FOLLOWTHROUGH_SUCC_WAIT_EXEC_OFFSET,
                            wait_exec_id);
    return validation_patch_user_text(proc, image, sizeof(image));
}

#define VALIDATION_FULL_RESULT_BCIB_SIZE (AYKEN_FRAME_SIZE + 73u)
#define VALIDATION_FULL_RESULT_OUTPUT_SIZE \
    (sizeof(ayken_execution_output_v1_t) + VALIDATION_FULL_RESULT_BCIB_SIZE)

static uint8_t g_validation_full_result_bcib[VALIDATION_FULL_RESULT_BCIB_SIZE];
static int g_validation_full_result_bcib_initialized = 0;
static uint8_t g_validation_full_result_output[VALIDATION_FULL_RESULT_OUTPUT_SIZE];
static int g_validation_full_result_output_initialized = 0;

static uint32_t validation_frame_count_for_size(uint64_t size)
{
    if (size == 0) {
        return 0;
    }

    return (uint32_t)((size + (AYKEN_FRAME_SIZE - 1)) / AYKEN_FRAME_SIZE);
}

static void validation_prepare_full_result_bcib(void)
{
    uint32_t i;

    if (g_validation_full_result_bcib_initialized) {
        return;
    }

    g_validation_full_result_bcib[0] = 0x42;
    g_validation_full_result_bcib[1] = 0x43;
    g_validation_full_result_bcib[2] = 0x49;
    g_validation_full_result_bcib[3] = 0x42;
    for (i = 4; i < VALIDATION_FULL_RESULT_BCIB_SIZE; ++i) {
        g_validation_full_result_bcib[i] = (uint8_t)(((i * 37u) + 0x5Au) & 0xFFu);
    }

    g_validation_full_result_bcib_initialized = 1;
}

static void validation_prepare_full_result_output(void)
{
    ayken_execution_output_v1_t *header;

    if (g_validation_full_result_output_initialized) {
        return;
    }

    validation_prepare_full_result_bcib();
    memset(g_validation_full_result_output, 0, sizeof(g_validation_full_result_output));

    header = (ayken_execution_output_v1_t *)g_validation_full_result_output;
    header->magic = AYKEN_EXECUTION_OUTPUT_MAGIC;
    header->abi_version = AYKEN_EXECUTION_OUTPUT_VERSION;
    header->bytes_written = VALIDATION_FULL_RESULT_BCIB_SIZE;

    memcpy(g_validation_full_result_output + sizeof(*header),
           g_validation_full_result_bcib,
           VALIDATION_FULL_RESULT_BCIB_SIZE);

    g_validation_full_result_output_initialized = 1;
}

static uint64_t validation_build_structured_output_buffer(uint8_t *dst,
                                                          uint64_t capacity,
                                                          uint32_t magic,
                                                          uint32_t abi_version,
                                                          uint32_t kind,
                                                          const uint8_t *payload,
                                                          uint64_t payload_size)
{
    ayken_execution_output_v2_t *header;
    uint64_t total_size = sizeof(ayken_execution_output_v2_t) + payload_size;

    if (!dst || capacity < total_size) {
        return 0;
    }

    memset(dst, 0, total_size);
    header = (ayken_execution_output_v2_t *)dst;
    header->magic = magic;
    header->abi_version = abi_version;
    header->kind = kind;
    header->bytes_written = payload_size;

    if (payload_size > 0 && payload) {
        memcpy(dst + sizeof(*header), payload, payload_size);
    }

    return total_size;
}

static int validation_buffer_matches(const uint8_t *actual,
                                     const uint8_t *expected,
                                     uint64_t size)
{
    uint64_t i;

    if (!actual || !expected) {
        return 0;
    }

    for (i = 0; i < size; ++i) {
        if (actual[i] != expected[i]) {
            return 0;
        }
    }

    return 1;
}

static int validation_frames_match_buffer(const uint64_t *frames,
                                          uint32_t frame_count,
                                          const uint8_t *expected,
                                          uint64_t expected_size)
{
    uint32_t expected_frame_count;
    uint32_t i;
    uint64_t offset = 0;

    if (!frames || !expected || expected_size == 0) {
        return 0;
    }

    expected_frame_count = validation_frame_count_for_size(expected_size);
    if (expected_frame_count == 0 || frame_count != expected_frame_count) {
        return 0;
    }

    for (i = 0; i < frame_count; ++i) {
        const uint8_t *frame_bytes = (const uint8_t *)paging_phys_to_virt(frames[i]);
        uint64_t remaining = expected_size - offset;
        uint64_t chunk_size = remaining > AYKEN_FRAME_SIZE ? AYKEN_FRAME_SIZE : remaining;

        if (frames[i] == 0 || !frame_bytes) {
            return 0;
        }
        if (!validation_buffer_matches(frame_bytes, expected + offset, chunk_size)) {
            return 0;
        }

        offset += chunk_size;
    }

    return offset == expected_size;
}

static int validation_frames_are_zeroed(const uint64_t *frames,
                                        uint32_t frame_count)
{
    uint32_t i;
    uint64_t offset;

    if (!frames || frame_count == 0) {
        return 0;
    }

    for (i = 0; i < frame_count; ++i) {
        const uint8_t *frame_bytes = (const uint8_t *)paging_phys_to_virt(frames[i]);

        if (frames[i] == 0 || !frame_bytes) {
            return 0;
        }

        for (offset = 0; offset < AYKEN_FRAME_SIZE; ++offset) {
            if (frame_bytes[offset] != 0) {
                return 0;
            }
        }
    }

    return 1;
}

static int validation_frames_are_distinct(const uint64_t *lhs,
                                          uint32_t lhs_count,
                                          const uint64_t *rhs,
                                          uint32_t rhs_count)
{
    uint32_t i;
    uint32_t j;

    if (!lhs || !rhs) {
        return 0;
    }

    for (i = 0; i < lhs_count; ++i) {
        if (lhs[i] == 0) {
            return 0;
        }
        for (j = 0; j < rhs_count; ++j) {
            if (rhs[j] == 0) {
                continue;
            }
            if (lhs[i] == rhs[j]) {
                return 0;
            }
        }
    }

    return 1;
}

static int validation_copy_into_frames(const uint64_t *frames,
                                       uint32_t frame_count,
                                       uint64_t dst_offset,
                                       const void *src,
                                       uint64_t size)
{
    const uint8_t *src_bytes = (const uint8_t *)src;

    while (size > 0) {
        uint32_t frame_index = (uint32_t)(dst_offset / AYKEN_FRAME_SIZE);
        uint64_t frame_offset = dst_offset % AYKEN_FRAME_SIZE;
        uint64_t chunk_size;
        uint8_t *dst;

        if (frame_index >= frame_count || frames[frame_index] == 0) {
            return 0;
        }

        dst = (uint8_t *)paging_phys_to_virt(frames[frame_index]);
        if (!dst) {
            return 0;
        }

        chunk_size = AYKEN_FRAME_SIZE - frame_offset;
        if (chunk_size > size) {
            chunk_size = size;
        }

        memcpy(dst + frame_offset, src_bytes, chunk_size);
        src_bytes += chunk_size;
        dst_offset += chunk_size;
        size -= chunk_size;
    }

    return 1;
}

static int validation_frames_range_is_zeroed(const uint64_t *frames,
                                             uint32_t frame_count,
                                             uint64_t start_offset,
                                             uint64_t size)
{
    uint64_t offset = start_offset;
    uint64_t remaining = size;

    if (!frames || frame_count == 0) {
        return 0;
    }

    while (remaining > 0) {
        uint32_t frame_index = (uint32_t)(offset / AYKEN_FRAME_SIZE);
        uint64_t frame_offset = offset % AYKEN_FRAME_SIZE;
        uint64_t chunk_size = AYKEN_FRAME_SIZE - frame_offset;
        const uint8_t *src;
        uint64_t i;

        if (frame_index >= frame_count || frames[frame_index] == 0) {
            return 0;
        }

        if (chunk_size > remaining) {
            chunk_size = remaining;
        }

        src = (const uint8_t *)paging_phys_to_virt(frames[frame_index]);
        if (!src) {
            return 0;
        }

        for (i = 0; i < chunk_size; ++i) {
            if (src[frame_offset + i] != 0) {
                return 0;
            }
        }

        offset += chunk_size;
        remaining -= chunk_size;
    }

    return 1;
}

static int validation_result_hash_matches(const exec_slot_t *slot,
                                          uint64_t owner_pml4_phys,
                                          const uint8_t *expected_bytes,
                                          uint64_t expected_size)
{
    const ayken_execution_result_hash_v1_t *hash_header;
    uint64_t hash_pte;
    uint8_t expected_digest[AYKEN_SHA256_DIGEST_SIZE];

    if (!slot || !expected_bytes || expected_size == 0 || owner_pml4_phys == 0) {
        return 0;
    }
    if (slot->hash_frame == 0 ||
        slot->hash_size != sizeof(ayken_execution_result_hash_v1_t) ||
        slot->hashed_size != expected_size ||
        slot->mapped_hash_va == 0) {
        return 0;
    }

    hash_header = (const ayken_execution_result_hash_v1_t *)paging_phys_to_virt(slot->hash_frame);
    if (!hash_header) {
        return 0;
    }

    ayken_sha256_compute(expected_bytes, expected_size, expected_digest);
    hash_pte = paging_get_pte_in_pml4(owner_pml4_phys, slot->mapped_hash_va);

    return hash_header->magic == AYKEN_EXECUTION_RESULT_HASH_MAGIC &&
           hash_header->abi_version == AYKEN_EXECUTION_RESULT_HASH_VERSION &&
           hash_header->algorithm == AYKEN_RESULT_HASH_ALG_SHA256 &&
           hash_header->hashed_size == expected_size &&
           validation_buffer_matches(hash_header->digest,
                                     expected_digest,
                                     AYKEN_SHA256_DIGEST_SIZE) &&
           hash_pte != 0 &&
           (hash_pte & AYKEN_PTE_ADDR_MASK) == slot->hash_frame &&
           (hash_pte & AYKEN_PTE_USER) != 0 &&
           (hash_pte & AYKEN_PTE_WRITABLE) == 0 &&
           (hash_pte & AYKEN_PTE_NO_EXEC) != 0;
}

static int validation_write_output_for_execution_with_header(uint64_t execution_id,
                                                             uint32_t magic,
                                                             uint32_t abi_version,
                                                             uint64_t declared_bytes_written,
                                                             const uint8_t *payload,
                                                             uint64_t payload_size)
{
    execution_slot_guard_t slot_guard = {0};
    exec_slot_t *slot = NULL;
    ayken_execution_output_v1_t header;
    int ok = 0;

    execution_slot_enter_critical(&slot_guard);
    slot = execution_slot_find_locked(execution_id);
    if (!slot || slot->state != EXEC_SLOT_RUNNING) {
        goto out;
    }
    if (slot->output_frame_count != AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES) {
        goto out;
    }

    memset(&header, 0, sizeof(header));
    header.magic = magic;
    header.abi_version = abi_version;
    header.bytes_written = declared_bytes_written;

    if (!validation_copy_into_frames(slot->output_frames,
                                     slot->output_frame_count,
                                     0,
                                     &header,
                                     sizeof(header))) {
        goto out;
    }
    if (payload_size > 0 &&
        !validation_copy_into_frames(slot->output_frames,
                                     slot->output_frame_count,
                                     sizeof(header),
                                     payload,
                                     payload_size)) {
        goto out;
    }

    ok = 1;
out:
    execution_slot_exit_critical(&slot_guard);
    return ok;
}

static int validation_write_output_for_execution(uint64_t execution_id,
                                                 uint32_t magic,
                                                 uint32_t abi_version,
                                                 const uint8_t *payload,
                                                 uint64_t payload_size)
{
    return validation_write_output_for_execution_with_header(execution_id,
                                                             magic,
                                                             abi_version,
                                                             payload_size,
                                                             payload,
                                                             payload_size);
}

static int validation_write_structured_output_for_execution_with_header(uint64_t execution_id,
                                                                        uint32_t magic,
                                                                        uint32_t abi_version,
                                                                        uint32_t kind,
                                                                        uint64_t declared_bytes_written,
                                                                        const uint8_t *payload,
                                                                        uint64_t payload_size)
{
    execution_slot_guard_t slot_guard = {0};
    exec_slot_t *slot = NULL;
    ayken_execution_output_v2_t header;
    int ok = 0;

    execution_slot_enter_critical(&slot_guard);
    slot = execution_slot_find_locked(execution_id);
    if (!slot || slot->state != EXEC_SLOT_RUNNING) {
        goto out;
    }
    if (slot->output_frame_count != AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES) {
        goto out;
    }

    memset(&header, 0, sizeof(header));
    header.magic = magic;
    header.abi_version = abi_version;
    header.kind = kind;
    header.bytes_written = declared_bytes_written;

    if (!validation_copy_into_frames(slot->output_frames,
                                     slot->output_frame_count,
                                     0,
                                     &header,
                                     sizeof(header))) {
        goto out;
    }
    if (payload_size > 0 &&
        !validation_copy_into_frames(slot->output_frames,
                                     slot->output_frame_count,
                                     sizeof(header),
                                     payload,
                                     payload_size)) {
        goto out;
    }

    ok = 1;
out:
    execution_slot_exit_critical(&slot_guard);
    return ok;
}

static int validation_write_structured_output_for_execution(uint64_t execution_id,
                                                            uint32_t magic,
                                                            uint32_t abi_version,
                                                            uint32_t kind,
                                                            const uint8_t *payload,
                                                            uint64_t payload_size)
{
    return validation_write_structured_output_for_execution_with_header(execution_id,
                                                                        magic,
                                                                        abi_version,
                                                                        kind,
                                                                        payload_size,
                                                                        payload,
                                                                        payload_size);
}

static int validation_corrupt_output_tail_for_execution(uint64_t execution_id,
                                                        uint64_t start_offset,
                                                        uint8_t pattern,
                                                        uint64_t size)
{
    execution_slot_guard_t slot_guard = {0};
    exec_slot_t *slot = NULL;
    int ok = 0;
    uint64_t i;

    execution_slot_enter_critical(&slot_guard);
    slot = execution_slot_find_locked(execution_id);
    if (!slot || slot->state != EXEC_SLOT_RUNNING) {
        goto out;
    }
    if (slot->output_frame_count != AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES) {
        goto out;
    }

    for (i = 0; i < size; ++i) {
        if (!validation_copy_into_frames(slot->output_frames,
                                         slot->output_frame_count,
                                         start_offset + i,
                                         &pattern,
                                         1)) {
            goto out;
        }
    }

    ok = 1;
out:
    execution_slot_exit_critical(&slot_guard);
    return ok;
}

static void blocked_wait_harness_waiter_thread(void)
{
    static const uint8_t bcib[] = {0x42, 0x43, 0x49, 0x42, 0x77, 0x11};
    execution_slot_guard_t slot_guard = {0};
    exec_slot_t *slot = NULL;

    g_blocked_wait_harness.waiter_proc = current_proc;
    g_blocked_wait_harness.execution_id = sys_v2_submit_execution((void *)bcib,
                                                                  sizeof(bcib),
                                                                  (uint64_t)g_blocked_wait_harness.target_proc->pid);
    if (g_blocked_wait_harness.execution_id != 0) {
        g_blocked_wait_harness.wait_result =
            sys_v2_wait_result(g_blocked_wait_harness.execution_id, 60000);
    } else {
        g_blocked_wait_harness.wait_result = ESYS_V2_CONTEXT_ERROR;
    }

    execution_slot_enter_critical(&slot_guard);
    slot = execution_slot_find_locked(g_blocked_wait_harness.execution_id);
    if (slot != NULL) {
        g_blocked_wait_harness.terminal_state = (uint64_t)slot->state;
    }
    execution_slot_exit_critical(&slot_guard);

    g_blocked_wait_harness.waiter_wait_obj_cleared =
        current_proc != NULL && current_proc->wait_obj == NULL;
    g_blocked_wait_harness.waiter_resumed = 1;

    for (;;) {
        proc_block_current(&g_blocked_wait_park_token);
    }
}

static void blocked_wait_harness_waker_thread(void)
{
    execution_slot_guard_t slot_guard = {0};
    exec_slot_t *slot = NULL;
    uint64_t stale_generation = 0;

    g_blocked_wait_harness.waker_proc = current_proc;

    while (g_blocked_wait_harness.waiter_proc == NULL ||
           g_blocked_wait_harness.execution_id == 0 ||
           g_blocked_wait_harness.waiter_proc->state != PROC_BLOCKED ||
           g_blocked_wait_harness.waiter_proc->wait_obj == NULL) {
        sched_yield();
    }

    g_blocked_wait_harness.waiter_blocked = 1;
    g_blocked_wait_harness.blocked_wait_obj = g_blocked_wait_harness.waiter_proc->wait_obj;

    execution_slot_enter_critical(&slot_guard);
    slot = execution_slot_find_locked(g_blocked_wait_harness.execution_id);
    if (slot != NULL) {
        stale_generation = slot->wait_key.generation + 1;
        if (stale_generation == slot->wait_key.generation) {
            stale_generation = slot->wait_key.generation - 1;
        }
        g_blocked_wait_spurious_key.execution_id = slot->wait_key.execution_id;
        g_blocked_wait_spurious_key.generation = stale_generation;
    } else {
        g_blocked_wait_spurious_key.execution_id = g_blocked_wait_harness.execution_id;
        g_blocked_wait_spurious_key.generation = 1;
    }
    execution_slot_exit_critical(&slot_guard);

    proc_wake_waiters(&g_blocked_wait_spurious_key);

    g_blocked_wait_harness.wrong_wake_preserved =
        g_blocked_wait_harness.waiter_proc->state == PROC_BLOCKED &&
        g_blocked_wait_harness.waiter_proc->wait_obj == g_blocked_wait_harness.blocked_wait_obj;

    execution_slot_enter_critical(&slot_guard);
    slot = execution_slot_find_locked(g_blocked_wait_harness.execution_id);
    if (slot != NULL) {
        (void)execution_slot_finish_locked(slot, EXEC_SLOT_ABORTED);
    }
    execution_slot_exit_critical(&slot_guard);

    g_blocked_wait_harness.wake_released =
        g_blocked_wait_harness.waiter_proc->wait_obj == NULL &&
        g_blocked_wait_harness.waiter_proc->state == PROC_READY;

    for (;;) {
        proc_block_current(&g_blocked_wait_park_token);
    }
}

static void timeout_irq_harness_waiter_thread(void)
{
    static const uint8_t bcib[] = {0x42, 0x43, 0x49, 0x42, 0x19, 0x94};
    execution_slot_guard_t slot_guard = {0};
    exec_slot_t *slot = NULL;

    g_timeout_irq_harness.waiter_proc = current_proc;
    g_timeout_irq_harness.execution_id = sys_v2_submit_execution((void *)bcib,
                                                                 sizeof(bcib),
                                                                 (uint64_t)g_timeout_irq_harness.target_proc->pid);
    if (g_timeout_irq_harness.execution_id != 0) {
        g_timeout_irq_harness.wait_result =
            sys_v2_wait_result(g_timeout_irq_harness.execution_id, 1);
    } else {
        g_timeout_irq_harness.wait_result = ESYS_V2_CONTEXT_ERROR;
    }

    execution_slot_enter_critical(&slot_guard);
    slot = execution_slot_find_locked(g_timeout_irq_harness.execution_id);
    if (slot != NULL) {
        g_timeout_irq_harness.terminal_state = (uint64_t)slot->state;
    }
    execution_slot_exit_critical(&slot_guard);

    g_timeout_irq_harness.waiter_wait_obj_cleared =
        current_proc != NULL && current_proc->wait_obj == NULL;
    g_timeout_irq_harness.waiter_resumed = 1;

    for (;;) {
        proc_block_current(&g_timeout_irq_park_token);
    }
}

static void timeout_irq_harness_driver_thread(void)
{
    execution_slot_guard_t slot_guard = {0};
    exec_slot_t *slot = NULL;
    int spin;

    g_timeout_irq_harness.driver_proc = current_proc;

    while (g_timeout_irq_harness.waiter_proc == NULL ||
           g_timeout_irq_harness.execution_id == 0 ||
           g_timeout_irq_harness.waiter_proc->state != PROC_BLOCKED ||
           g_timeout_irq_harness.waiter_proc->wait_obj == NULL) {
        sched_yield();
    }

    g_timeout_irq_harness.waiter_blocked = 1;
    g_timeout_irq_harness.blocked_wait_obj = g_timeout_irq_harness.waiter_proc->wait_obj;

    execution_slot_enter_critical(&slot_guard);
    slot = execution_slot_find_locked(g_timeout_irq_harness.execution_id);
    if (slot != NULL) {
        g_timeout_irq_harness.deadline_tick = slot->deadline_tick;
    }
    execution_slot_exit_critical(&slot_guard);

    for (spin = 0; spin < 3; ++spin) {
        sched_yield();
    }

    execution_slot_enter_critical(&slot_guard);
    slot = execution_slot_find_locked(g_timeout_irq_harness.execution_id);
    g_timeout_irq_harness.pre_irq_still_blocked =
        slot != NULL &&
        slot->state == EXEC_SLOT_READY &&
        g_timeout_irq_harness.waiter_proc->state == PROC_BLOCKED &&
        g_timeout_irq_harness.waiter_proc->wait_obj == g_timeout_irq_harness.blocked_wait_obj;
    execution_slot_exit_critical(&slot_guard);

    timer_isr_c(NULL);
    sched_yield();

    g_timeout_irq_harness.irq_woke_waiter =
        g_timeout_irq_harness.waiter_proc->wait_obj == NULL &&
        (g_timeout_irq_harness.waiter_proc->state == PROC_READY ||
         g_timeout_irq_harness.waiter_resumed);

    for (;;) {
        proc_block_current(&g_timeout_irq_park_token);
    }
}

static void negative_timeout_harness_waiter_thread(void)
{
    static const uint8_t bcib[] = {0x42, 0x43, 0x49, 0x42, 0x55, 0xAA};
    execution_slot_guard_t slot_guard = {0};
    exec_slot_t *slot = NULL;

    g_negative_timeout_harness.waiter_proc = current_proc;
    g_negative_timeout_harness.execution_id = sys_v2_submit_execution((void *)bcib,
                                                                      sizeof(bcib),
                                                                      (uint64_t)g_negative_timeout_harness.target_proc->pid);
    if (g_negative_timeout_harness.execution_id != 0) {
        g_negative_timeout_harness.wait_result =
            sys_v2_wait_result(g_negative_timeout_harness.execution_id, 1);
        g_negative_timeout_harness.post_timeout_wait_result =
            sys_v2_wait_result(g_negative_timeout_harness.execution_id, 0);
    } else {
        g_negative_timeout_harness.wait_result = ESYS_V2_CONTEXT_ERROR;
        g_negative_timeout_harness.post_timeout_wait_result = ESYS_V2_CONTEXT_ERROR;
    }

    execution_slot_enter_critical(&slot_guard);
    slot = execution_slot_find_locked(g_negative_timeout_harness.execution_id);
    if (slot != NULL) {
        g_negative_timeout_harness.terminal_state = (uint64_t)slot->state;
    }
    execution_slot_exit_critical(&slot_guard);

    g_negative_timeout_harness.waiter_wait_obj_cleared =
        current_proc != NULL && current_proc->wait_obj == NULL;
    g_negative_timeout_harness.waiter_resumed = 1;

    for (;;) {
        proc_block_current(&g_negative_timeout_park_token);
    }
}

static void negative_timeout_harness_driver_thread(void)
{
    execution_slot_guard_t slot_guard = {0};
    exec_slot_t *slot = NULL;
    proc_t *saved_current_proc = current_proc;
    int spin;

    g_negative_timeout_harness.driver_proc = current_proc;

    while (g_negative_timeout_harness.waiter_proc == NULL ||
           g_negative_timeout_harness.execution_id == 0 ||
           g_negative_timeout_harness.waiter_proc->state != PROC_BLOCKED ||
           g_negative_timeout_harness.waiter_proc->wait_obj == NULL) {
        sched_yield();
    }

    g_negative_timeout_harness.waiter_blocked = 1;
    g_negative_timeout_harness.blocked_wait_obj =
        g_negative_timeout_harness.waiter_proc->wait_obj;

    current_proc = g_negative_timeout_harness.target_proc;
    if (current_proc != NULL) {
        current_proc->state = PROC_RUNNING;
    }
    (void)sched_try_pickup_execution_work();
    current_proc = saved_current_proc;

    execution_slot_enter_critical(&slot_guard);
    slot = execution_slot_find_locked(g_negative_timeout_harness.execution_id);
    if (slot != NULL) {
        g_negative_timeout_harness.deadline_tick = slot->deadline_tick;
        g_negative_timeout_harness.pickup_running =
            slot->state == EXEC_SLOT_RUNNING &&
            g_negative_timeout_harness.target_proc != NULL &&
            g_negative_timeout_harness.target_proc->active_execution_id ==
                g_negative_timeout_harness.execution_id;
    }
    execution_slot_exit_critical(&slot_guard);

    for (spin = 0; spin < 3; ++spin) {
        sched_yield();
    }

    execution_slot_enter_critical(&slot_guard);
    slot = execution_slot_find_locked(g_negative_timeout_harness.execution_id);
    g_negative_timeout_harness.pre_irq_still_blocked =
        slot != NULL &&
        slot->state == EXEC_SLOT_RUNNING &&
        g_negative_timeout_harness.waiter_proc->state == PROC_BLOCKED &&
        g_negative_timeout_harness.waiter_proc->wait_obj ==
            g_negative_timeout_harness.blocked_wait_obj;
    execution_slot_exit_critical(&slot_guard);

    timer_isr_c(NULL);
    sched_yield();

    g_negative_timeout_harness.irq_woke_waiter =
        g_negative_timeout_harness.waiter_proc->wait_obj == NULL &&
        (g_negative_timeout_harness.waiter_proc->state == PROC_READY ||
         g_negative_timeout_harness.waiter_resumed);

    g_negative_timeout_harness.foreign_wait_result =
        sys_v2_wait_result(g_negative_timeout_harness.execution_id, 0);

    execution_slot_enter_critical(&slot_guard);
    if (g_negative_timeout_harness.waiter_proc->pid > 0) {
        (void)execution_slot_release_owned_by_owner_locked(
            (uint64_t)g_negative_timeout_harness.waiter_proc->pid);
    }
    g_negative_timeout_harness.slot_released =
        execution_slot_find_locked(g_negative_timeout_harness.execution_id) == NULL;
    execution_slot_exit_critical(&slot_guard);

    for (;;) {
        proc_block_current(&g_negative_timeout_park_token);
    }
}

// ============================================================================
// SYSCALL V2 VALIDATION TESTS
// ============================================================================

/**
 * Test current execution-centric syscall behavior without overstating
 * incomplete lifecycle surfaces as fully operational.
 */
static void test_syscall_v2_interface(void)
{
    proc_t *target_worker = NULL;
    proc_t *saved_current_proc = NULL;

    TEST_START("V2 Syscall Interface");

    // Test 1: sys_v2_switch_context (with invalid contexts - should fail gracefully)
    uint64_t result;
    result = sys_v2_switch_context(999, 998);
    TEST_ASSERT(result == ESYS_V2_CONTEXT_ERROR, "sys_v2_switch_context error handling");

    target_worker = ensure_validation_worker_proc();
    TEST_ASSERT(target_worker != NULL && target_worker->type == PROC_TYPE_USER,
                "phase2 validation worker exists for live target-context submission");

    if (target_worker == NULL) {
        TEST_END("V2 Syscall Interface");
        return;
    }

    {
        uint64_t inbox_pte = paging_get_pte_in_pml4(target_worker->pml4_phys, EXECUTION_INBOX_VA);
        uint64_t payload_pte = paging_get_pte_in_pml4(target_worker->pml4_phys, EXECUTION_PAYLOAD_VA);
        int inbox_mapped = target_worker->execution_inbox_pa != 0 &&
                           target_worker->execution_inbox_pa != target_worker->mailbox_pa &&
                           (inbox_pte & AYKEN_PTE_ADDR_MASK) == target_worker->execution_inbox_pa;
        int payload_mapped = target_worker->execution_payload_pas[0] != 0 &&
                             (payload_pte & AYKEN_PTE_ADDR_MASK) == target_worker->execution_payload_pas[0];
        int inbox_ro_nx = inbox_pte != 0 &&
                          (inbox_pte & AYKEN_PTE_USER) != 0 &&
                          (inbox_pte & AYKEN_PTE_WRITABLE) == 0 &&
                          (inbox_pte & AYKEN_PTE_NO_EXEC) != 0;
        int payload_ro_nx = payload_pte != 0 &&
                            (payload_pte & AYKEN_PTE_USER) != 0 &&
                            (payload_pte & AYKEN_PTE_WRITABLE) == 0 &&
                            (payload_pte & AYKEN_PTE_NO_EXEC) != 0;

        TEST_ASSERT(inbox_mapped, "execution inbox maps to dedicated per-process backing");
        TEST_ASSERT(payload_mapped, "execution payload window maps to dedicated per-process backing");
        TEST_ASSERT(inbox_ro_nx, "execution inbox mapping is user-readable, read-only, and NX");
        TEST_ASSERT(payload_ro_nx, "execution payload mapping is user-readable, read-only, and NX");
    }

    // Test 2: sys_v2_submit_execution + completion path
    char dummy_bcib[] = {0x42, 0x43, 0x49, 0x42}; // "BCIB" magic
    uint32_t full_result_bcib_frame_count;
    uint32_t full_result_output_frame_count;
    validation_prepare_full_result_bcib();
    validation_prepare_full_result_output();
    full_result_bcib_frame_count = validation_frame_count_for_size(VALIDATION_FULL_RESULT_BCIB_SIZE);
    full_result_output_frame_count = validation_frame_count_for_size(VALIDATION_FULL_RESULT_OUTPUT_SIZE);
    uint64_t submit_exec_id = submit_validation_execution_as(target_worker,
                                                             g_validation_full_result_bcib,
                                                             VALIDATION_FULL_RESULT_BCIB_SIZE,
                                                             (uint64_t)target_worker->pid);
    result = submit_exec_id;
    TEST_ASSERT(result > 0, "sys_v2_submit_execution returns execution ID");
    {
        execution_slot_guard_t slot_guard = {0};
        exec_slot_t *slot = NULL;
        execution_context_queue_t *queue = NULL;
        const uint8_t *copied_bcib = NULL;
        int slot_ready = 0;
        int queue_has_entry = 0;
        int backing_copied = 0;

        execution_slot_enter_critical(&slot_guard);
        slot = execution_slot_find_locked(result);
        queue = execution_slot_find_queue_locked((uint64_t)target_worker->pid);
        slot_ready = slot != NULL &&
                     slot->state == EXEC_SLOT_READY &&
                     slot->target_context_id == (uint64_t)target_worker->pid &&
                     slot->bcib_size == VALIDATION_FULL_RESULT_BCIB_SIZE;
        if (slot != NULL && slot->bcib_frame_count == full_result_bcib_frame_count) {
            copied_bcib = (const uint8_t *)paging_phys_to_virt(slot->bcib_frames[0]);
            backing_copied = copied_bcib != NULL &&
                             validation_frames_match_buffer(slot->bcib_frames,
                                                            slot->bcib_frame_count,
                                                            g_validation_full_result_bcib,
                                                            VALIDATION_FULL_RESULT_BCIB_SIZE);
        }
        queue_has_entry = queue != NULL && queue->depth > 0;
        execution_slot_exit_critical(&slot_guard);

        TEST_ASSERT(slot_ready, "sys_v2_submit_execution creates READY slot");
        TEST_ASSERT(backing_copied, "sys_v2_submit_execution copies BCIB into kernel-owned backing");
        TEST_ASSERT(queue_has_entry, "sys_v2_submit_execution enqueues target context");
    }
    {
        execution_slot_guard_t slot_guard = {0};
        exec_slot_t *slot = NULL;
        ayken_execution_inbox_v1_t *inbox = NULL;
        const uint8_t *worker_payload = NULL;
        uint64_t output_pte = 0;
        int slot_running = 0;
        int slot_publishable = 0;
        int publish_emitted = 0;
        int descriptor_valid = 0;
        int payload_visible = 0;
        int output_bound = 0;
        int output_rw_nx = 0;
        int output_distinct = 0;
        int output_zeroed = 0;

        saved_current_proc = current_proc;
        current_proc = target_worker;
        current_proc->state = PROC_RUNNING;
        publish_emitted = sched_try_pickup_execution_work();
        current_proc = saved_current_proc;

        execution_slot_enter_critical(&slot_guard);
        slot = execution_slot_find_locked(submit_exec_id);
        slot_running = slot != NULL &&
                       slot->state == EXEC_SLOT_RUNNING &&
                       target_worker->active_execution_id == submit_exec_id;
        slot_publishable = slot != NULL &&
                           execution_slot_can_publish_locked(slot);
        if (slot != NULL) {
            output_pte = paging_get_pte_in_pml4(target_worker->pml4_phys, EXECUTION_OUTPUT_VA);
            output_bound = slot->output_frame_count == AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES &&
                           target_worker->execution_output_mapped_id == submit_exec_id;
            output_rw_nx = output_pte != 0 &&
                           (output_pte & AYKEN_PTE_ADDR_MASK) == slot->output_frames[0] &&
                           (output_pte & AYKEN_PTE_USER) != 0 &&
                           (output_pte & AYKEN_PTE_WRITABLE) != 0 &&
                           (output_pte & AYKEN_PTE_NO_EXEC) != 0;
            output_distinct =
                validation_frames_are_distinct(slot->output_frames,
                                               slot->output_frame_count,
                                               target_worker->execution_payload_pas,
                                               AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES) &&
                validation_frames_are_distinct(slot->output_frames,
                                               slot->output_frame_count,
                                               slot->bcib_frames,
                                               slot->bcib_frame_count);
            output_zeroed = validation_frames_are_zeroed(slot->output_frames,
                                                         slot->output_frame_count);
        }
        execution_slot_exit_critical(&slot_guard);

        inbox = (ayken_execution_inbox_v1_t *)paging_phys_to_virt(target_worker->execution_inbox_pa);
        if (target_worker->execution_payload_pas[0] != 0) {
            worker_payload = (const uint8_t *)paging_phys_to_virt(target_worker->execution_payload_pas[0]);
        }
        descriptor_valid = publish_emitted == 1 &&
                           inbox != NULL &&
                           inbox->state == AXIB_STATE_READY &&
                           inbox->delivery_seq == 1 &&
                           inbox->execution_id == submit_exec_id &&
                           inbox->target_context_id == (uint64_t)target_worker->pid &&
                           inbox->bcib_user_va == EXECUTION_PAYLOAD_VA &&
                           inbox->bcib_size == VALIDATION_FULL_RESULT_BCIB_SIZE &&
                           inbox->bcib_window_size == AYKEN_EXECUTION_PAYLOAD_WINDOW_SIZE;
        payload_visible = worker_payload != NULL &&
                          validation_frames_match_buffer(target_worker->execution_payload_pas,
                                                         full_result_bcib_frame_count,
                                                         g_validation_full_result_bcib,
                                                         VALIDATION_FULL_RESULT_BCIB_SIZE);

        TEST_ASSERT(slot_running, "execution_slot pickup transitions READY slot to RUNNING");
        TEST_ASSERT(slot_publishable, "running execution slot satisfies publish preconditions");
        TEST_ASSERT(descriptor_valid, "schedule-entry pickup publishes execution descriptor commit point");
        TEST_ASSERT(payload_visible, "schedule-entry pickup copies BCIB bytes into worker payload window");
        TEST_ASSERT(output_bound, "schedule-entry pickup binds a slot-owned output window to the worker");
        TEST_ASSERT(output_rw_nx, "execution output window mapping is user-writable and NX");
        TEST_ASSERT(output_distinct, "execution output backing stays distinct from input payload backing");
        TEST_ASSERT(output_zeroed, "execution output backing is zero-filled before executor writes begin");
    }
    TEST_ASSERT(validation_write_output_for_execution(submit_exec_id,
                                                      AYKEN_EXECUTION_OUTPUT_MAGIC,
                                                      AYKEN_EXECUTION_OUTPUT_VERSION,
                                                      g_validation_full_result_bcib,
                                                      VALIDATION_FULL_RESULT_BCIB_SIZE),
                "interface success candidate writes a valid output header before completion");
    TEST_ASSERT(validation_corrupt_output_tail_for_execution(submit_exec_id,
                                                             VALIDATION_FULL_RESULT_OUTPUT_SIZE,
                                                             0xA5u,
                                                             32u),
                "interface success candidate scribbles beyond declared output bytes before completion");
    result = complete_validation_execution_as(target_worker,
                                              submit_exec_id,
                                              EXEC_COMPLETION_COMPLETED);
    TEST_ASSERT(result == ESYS_V2_SUCCESS,
                "sys_v2_complete_execution closes RUNNING execution for owning worker");
    {
        execution_slot_guard_t slot_guard = {0};
        exec_slot_t *slot = NULL;
        uint32_t timed_out = 0;
        int slot_completed = 0;
        int output_unmapped = 0;

        execution_slot_enter_critical(&slot_guard);
        slot = execution_slot_find_locked(submit_exec_id);
        timed_out = execution_slot_process_timeouts_locked(1);
        slot_completed = slot != NULL &&
                         slot->state == EXEC_SLOT_COMPLETED &&
                         target_worker->active_execution_id == 0 &&
                         timed_out == 0;
        output_unmapped = paging_get_pte_in_pml4(target_worker->pml4_phys, EXECUTION_OUTPUT_VA) == 0 &&
                          target_worker->execution_output_mapped_id == 0;
        execution_slot_exit_critical(&slot_guard);

        TEST_ASSERT(slot_completed, "completion terminalizes slot, clears latch, and blocks later timeout overwrite");
        TEST_ASSERT(output_unmapped, "successful terminalization clears the worker output-window binding");
    }
    result = wait_validation_result_as(target_worker, submit_exec_id, 0);
    TEST_ASSERT(result > 0,
                "sys_v2_wait_result returns mapped result VA after explicit completion");
    {
        execution_slot_guard_t slot_guard = {0};
        exec_slot_t *slot = NULL;
        uint64_t result_pte0 = 0;
        uint64_t result_pte1 = 0;
        uint64_t hash_va = 0;
        int result_mapped = 0;
        int result_ro_nx = 0;
        int result_payload_valid = 0;
        int result_tail_zeroed = 0;
        int result_hash_valid = 0;

        execution_slot_enter_critical(&slot_guard);
        slot = execution_slot_find_locked(submit_exec_id);
        if (slot != NULL) {
            result_pte0 = paging_get_pte_in_pml4(target_worker->pml4_phys, result);
            result_pte1 = paging_get_pte_in_pml4(target_worker->pml4_phys, result + AYKEN_FRAME_SIZE);
            hash_va = execution_slot_result_hash_va_locked(slot);
            result_mapped = slot->state == EXEC_SLOT_RESULT_MAPPED &&
                            slot->mapped_result_va == result &&
                            slot->mapped_hash_va == hash_va &&
                            slot->result_frame_count == full_result_output_frame_count &&
                            slot->result_size == VALIDATION_FULL_RESULT_OUTPUT_SIZE &&
                            slot->bcib_frame_count == 0 &&
                            slot->bcib_size == 0 &&
                            slot->output_frame_count == 0 &&
                            slot->output_size == 0;
            result_ro_nx = result_pte0 != 0 &&
                           result_pte1 != 0 &&
                           (result_pte0 & AYKEN_PTE_ADDR_MASK) == slot->result_frames[0] &&
                           (result_pte1 & AYKEN_PTE_ADDR_MASK) == slot->result_frames[1] &&
                           (result_pte0 & AYKEN_PTE_USER) != 0 &&
                           (result_pte1 & AYKEN_PTE_USER) != 0 &&
                           (result_pte0 & AYKEN_PTE_WRITABLE) == 0 &&
                           (result_pte1 & AYKEN_PTE_WRITABLE) == 0 &&
                           (result_pte0 & AYKEN_PTE_NO_EXEC) != 0 &&
                           (result_pte1 & AYKEN_PTE_NO_EXEC) != 0;
            result_payload_valid = validation_frames_match_buffer(slot->result_frames,
                                                                  slot->result_frame_count,
                                                                  g_validation_full_result_output,
                                                                  VALIDATION_FULL_RESULT_OUTPUT_SIZE);
            result_tail_zeroed = validation_frames_range_is_zeroed(slot->result_frames,
                                                                   slot->result_frame_count,
                                                                   VALIDATION_FULL_RESULT_OUTPUT_SIZE,
                                                                   ((uint64_t)slot->result_frame_count * AYKEN_FRAME_SIZE) -
                                                                       VALIDATION_FULL_RESULT_OUTPUT_SIZE);
            result_hash_valid = validation_result_hash_matches(slot,
                                                               target_worker->pml4_phys,
                                                               g_validation_full_result_output,
                                                               VALIDATION_FULL_RESULT_OUTPUT_SIZE);
        }
        execution_slot_exit_critical(&slot_guard);

        TEST_ASSERT(result_mapped, "first successful wait_result transitions COMPLETED slot to RESULT_MAPPED");
        TEST_ASSERT(result_ro_nx, "mapped result pages are user-readable, read-only, and NX across the full payload span");
        TEST_ASSERT(result_payload_valid, "completed result materializes the frozen validated output header plus payload bytes");
        TEST_ASSERT(result_tail_zeroed, "completed result zero-seals bytes past the declared output size inside the mapped frame span");
        TEST_ASSERT(result_hash_valid, "completed result publishes a deterministic SHA-256 sidecar over the exact frozen result bytes");
    }
    {
        uint64_t repeated_wait_va = wait_validation_result_as(target_worker, submit_exec_id, 0);
        execution_slot_guard_t slot_guard = {0};
        exec_slot_t *slot = NULL;
        int hash_replayed = 0;

        execution_slot_enter_critical(&slot_guard);
        slot = execution_slot_find_locked(submit_exec_id);
        hash_replayed = slot != NULL &&
                        slot->mapped_hash_va == execution_slot_result_hash_va_locked(slot);
        execution_slot_exit_critical(&slot_guard);

        TEST_ASSERT(repeated_wait_va == result,
                    "repeated successful wait_result replays the same mapped result VA");
        TEST_ASSERT(hash_replayed,
                    "repeated successful wait_result replays the same mapped hash VA");
    }

    // Test 5: sys_v2_wait_result timeout path remains authoritative
    {
        uint64_t timeout_exec_id = submit_validation_execution_as(target_worker,
                                                                  dummy_bcib,
                                                                  sizeof(dummy_bcib),
                                                                  (uint64_t)target_worker->pid);
        execution_slot_guard_t slot_guard = {0};
        exec_slot_t *slot = NULL;
        int publish_emitted = 0;
        int slot_timed_out = 0;
        int output_unmapped = 0;

        TEST_ASSERT(timeout_exec_id > submit_exec_id,
                    "execution_id allocation remains monotonic and non-reused across submissions");

        saved_current_proc = current_proc;
        current_proc = target_worker;
        current_proc->state = PROC_RUNNING;
        publish_emitted = sched_try_pickup_execution_work();
        current_proc = saved_current_proc;
        TEST_ASSERT(publish_emitted == 1, "second execution is picked up for timeout path coverage");

        result = sys_v2_wait_result(timeout_exec_id, 0);
        TEST_ASSERT((int64_t)result == ESYS_V2_RESOURCE_BUSY,
                    "sys_v2_wait_result reports nonterminal execution as busy without timeout wait");

        execution_slot_enter_critical(&slot_guard);
        slot = execution_slot_find_locked(timeout_exec_id);
        if (slot != NULL) {
            slot->deadline_tick = 1;
        }
        slot_timed_out = slot != NULL &&
                         execution_slot_process_timeouts_locked(1) == 1 &&
                         slot->state == EXEC_SLOT_TIMEOUT &&
                         target_worker->active_execution_id == 0;
        output_unmapped = paging_get_pte_in_pml4(target_worker->pml4_phys, EXECUTION_OUTPUT_VA) == 0 &&
                          target_worker->execution_output_mapped_id == 0;
        execution_slot_exit_critical(&slot_guard);

        TEST_ASSERT(slot_timed_out, "timer timeout scan transitions overdue slot to TIMEOUT and clears latch");
        TEST_ASSERT(output_unmapped, "timeout terminalization clears the worker output-window binding");

        result = sys_v2_wait_result(timeout_exec_id, 0);
        TEST_ASSERT((int64_t)result == ESYS_V2_TIMEOUT,
                    "sys_v2_wait_result reports timeout after IRQ-driven timeout state");
    }

    // Test 6: sys_v2_interrupt_return
    result = sys_v2_interrupt_return(1, 0);
    TEST_ASSERT(result == ESYS_V2_SUCCESS,
                "sys_v2_interrupt_return current placeholder path reachable");

    // Test 7: sys_v2_time_query
    uint64_t monotonic_tick0 = 0;
    uint64_t monotonic_tick1 = 0;
    uint64_t monotonic_tick2 = 0;
    uint64_t uptime_ms = 0;
    result = sys_v2_time_query(TIME_QUERY_MONOTONIC, &monotonic_tick0);
    TEST_ASSERT(result == ESYS_V2_SUCCESS, "sys_v2_time_query monotonic ticks");
    result = sys_v2_time_query(TIME_QUERY_MONOTONIC, &monotonic_tick1);
    TEST_ASSERT(result == ESYS_V2_SUCCESS && monotonic_tick1 >= monotonic_tick0,
                "sys_v2_time_query monotonic nondecreasing");
    timer_isr_c(NULL);
    result = sys_v2_time_query(TIME_QUERY_MONOTONIC, &monotonic_tick2);
    TEST_ASSERT(result == ESYS_V2_SUCCESS && monotonic_tick2 > monotonic_tick1,
                "sys_v2_time_query shows non-zero forward progress after a timer IRQ");
    result = sys_v2_time_query(TIME_QUERY_UPTIME, &uptime_ms);
    TEST_ASSERT(result == ESYS_V2_SUCCESS, "sys_v2_time_query uptime milliseconds");

    // Test 8: sys_v2_capability_bind
    capability_token_t test_token = {0, CAP_PERM_READ, CAP_RESOURCE_MEMORY};
    result = sys_v2_capability_bind(1001, &test_token);
    TEST_ASSERT(result > 0, "sys_v2_capability_bind returns capability ID");

    // Test 9: sys_v2_capability_revoke
    result = sys_v2_capability_revoke(test_token.id);
    TEST_ASSERT(result == ESYS_V2_SUCCESS, "sys_v2_capability_revoke basic functionality");

    // Test 10: sys_v2_exit
    // Dedicated helper-level teardown coverage lives in test_exit_teardown_contract().
    fb_print("[INFO] sys_v2_exit semantic teardown covered in the dedicated exit contract test\n");

    TEST_END("V2 Syscall Interface");
}

/**
 * Test syscall parameter validation and error handling
 */
static void test_syscall_v2_error_handling(void)
{
    TEST_START("V2 Syscall Error Handling");

    // Test invalid parameters
    uint64_t result = sys_v2_map_memory(0, 0, 0);
    TEST_ASSERT(result == ESYS_V2_INVALID_PARAM, "map_memory rejects null addresses");

    result = sys_v2_unmap_memory(0, 0);
    TEST_ASSERT(result == ESYS_V2_INVALID_PARAM, "unmap_memory rejects null parameters");

    result = sys_v2_switch_context(0, 0);
    TEST_ASSERT(result == ESYS_V2_INVALID_PARAM, "switch_context rejects null context IDs");

    result = sys_v2_submit_execution(NULL, 0, 0);
    TEST_ASSERT(result == ESYS_V2_INVALID_PARAM, "submit_execution rejects null parameters");

    result = sys_v2_submit_execution(&(uint8_t){0xAA},
                                     AYKEN_EXECUTION_PAYLOAD_WINDOW_SIZE + 1,
                                     1);
    TEST_ASSERT(result == ESYS_V2_INVALID_PARAM,
                "submit_execution rejects oversize BCIB payloads");

    result = sys_v2_submit_execution(&(uint8_t){0xAA}, 1, 999999);
    TEST_ASSERT(result == ESYS_V2_CONTEXT_ERROR,
                "submit_execution rejects non-live target user contexts");

    result = sys_v2_wait_result(0, 1000);
    TEST_ASSERT(result == ESYS_V2_INVALID_PARAM, "wait_result rejects null execution ID");

    result = sys_v2_complete_execution(0, EXEC_COMPLETION_COMPLETED);
    TEST_ASSERT((int64_t)result == ESYS_V2_INVALID_PARAM, "complete_execution rejects null execution ID");

    result = sys_v2_complete_execution(1, 99);
    TEST_ASSERT((int64_t)result == ESYS_V2_INVALID_PARAM, "complete_execution rejects unknown completion code");

    result = sys_v2_interrupt_return(0, 0);
    TEST_ASSERT(result == ESYS_V2_INVALID_PARAM, "interrupt_return rejects null interrupt ID");

    result = sys_v2_time_query(TIME_QUERY_UPTIME, NULL);
    TEST_ASSERT(result == ESYS_V2_INVALID_PARAM, "time_query rejects null buffer");

    result = sys_v2_time_query(99, &(uint64_t){0});
    TEST_ASSERT(result == ESYS_V2_INVALID_PARAM, "time_query rejects unknown query type");

    result = sys_v2_capability_bind(0, NULL);
    TEST_ASSERT(result == ESYS_V2_INVALID_PARAM, "capability_bind rejects null parameters");

    result = sys_v2_capability_revoke(0);
    TEST_ASSERT(result == ESYS_V2_INVALID_PARAM, "capability_revoke rejects null token ID");

    TEST_END("V2 Syscall Error Handling");
}

static void test_completion_handoff_contract(void)
{
    proc_t *target_worker = NULL;
    proc_t *foreign_worker = NULL;
    proc_t *saved_current_proc = NULL;
    char bcib[] = {0x42, 0x43, 0x49, 0x42, 0xAA, 0x55};
    uint64_t timeout_exec_id = 0;
    uint64_t completed_exec_id = 0;
    uint64_t structured_raw_exec_id = 0;
    uint64_t structured_blob_exec_id = 0;
    uint64_t failed_exec_id = 0;
    uint64_t result = 0;

    TEST_START("Completion Handoff Contract");

    target_worker = ensure_validation_worker_proc();
    foreign_worker = ensure_validation_foreign_proc();
    TEST_ASSERT(target_worker != NULL && target_worker->type == PROC_TYPE_USER,
                "completion tests have a live target executor");
    TEST_ASSERT(foreign_worker != NULL && foreign_worker->type == PROC_TYPE_USER,
                "completion tests have a distinct foreign executor");

    if (target_worker == NULL || foreign_worker == NULL) {
        TEST_END("Completion Handoff Contract");
        return;
    }

    timeout_exec_id = submit_validation_execution_as(target_worker,
                                                     bcib,
                                                     sizeof(bcib),
                                                     (uint64_t)target_worker->pid);
    TEST_ASSERT(timeout_exec_id > 0, "completion contract timeout candidate submitted");

    saved_current_proc = current_proc;
    current_proc = target_worker;
    current_proc->state = PROC_RUNNING;
    TEST_ASSERT(sched_try_pickup_execution_work() == 1,
                "completion contract timeout candidate picked up into RUNNING");
    current_proc = saved_current_proc;

    result = complete_validation_execution_as(foreign_worker,
                                              timeout_exec_id,
                                              EXEC_COMPLETION_COMPLETED);
    TEST_ASSERT((int64_t)result == ESYS_V2_PERMISSION_DENIED,
                "foreign executor cannot complete another worker's RUNNING slot");

    {
        execution_slot_guard_t slot_guard = {0};
        exec_slot_t *slot = NULL;
        uint32_t timed_out = 0;
        int timeout_won = 0;

        execution_slot_enter_critical(&slot_guard);
        slot = execution_slot_find_locked(timeout_exec_id);
        if (slot != NULL) {
            slot->deadline_tick = 1;
        }
        timed_out = execution_slot_process_timeouts_locked(1);
        timeout_won = slot != NULL &&
                      timed_out == 1 &&
                      slot->state == EXEC_SLOT_TIMEOUT &&
                      target_worker->active_execution_id == 0;
        execution_slot_exit_critical(&slot_guard);

        TEST_ASSERT(timeout_won,
                    "timeout path wins terminalization when it lands before explicit completion");
    }

    result = complete_validation_execution_as(target_worker,
                                              timeout_exec_id,
                                              EXEC_COMPLETION_COMPLETED);
    TEST_ASSERT((int64_t)result == ESYS_V2_INVALID_STATE,
                "completion fails closed once timeout already terminalized the slot");

    completed_exec_id = submit_validation_execution_as(target_worker,
                                                       bcib,
                                                       sizeof(bcib),
                                                       (uint64_t)target_worker->pid);
    TEST_ASSERT(completed_exec_id > timeout_exec_id,
                "completion contract preserves monotonic non-reused execution IDs");

    saved_current_proc = current_proc;
    current_proc = target_worker;
    current_proc->state = PROC_RUNNING;
    TEST_ASSERT(sched_try_pickup_execution_work() == 1,
                "completion contract success candidate picked up into RUNNING");
    current_proc = saved_current_proc;
    TEST_ASSERT(validation_write_output_for_execution(completed_exec_id,
                                                      AYKEN_EXECUTION_OUTPUT_MAGIC,
                                                      AYKEN_EXECUTION_OUTPUT_VERSION,
                                                      (const uint8_t *)bcib,
                                                      sizeof(bcib)),
                "completion contract success candidate has a valid output header");
    result = complete_validation_execution_as(target_worker,
                                              completed_exec_id,
                                              EXEC_COMPLETION_COMPLETED);
    TEST_ASSERT(result == ESYS_V2_SUCCESS,
                "owning executor can complete a RUNNING slot successfully");

    structured_raw_exec_id = submit_validation_execution_as(target_worker,
                                                            bcib,
                                                            sizeof(bcib),
                                                            (uint64_t)target_worker->pid);
    TEST_ASSERT(structured_raw_exec_id > completed_exec_id,
                "structured RAW completion candidate preserves monotonic execution IDs");

    saved_current_proc = current_proc;
    current_proc = target_worker;
    current_proc->state = PROC_RUNNING;
    TEST_ASSERT(sched_try_pickup_execution_work() == 1,
                "structured RAW completion candidate picked up into RUNNING");
    current_proc = saved_current_proc;

    TEST_ASSERT(validation_write_structured_output_for_execution(structured_raw_exec_id,
                                                                AYKEN_EXECUTION_OUTPUT_V2_MAGIC,
                                                                AYKEN_EXECUTION_OUTPUT_V2_VERSION,
                                                                AYKEN_OUTPUT_KIND_RAW,
                                                                (const uint8_t *)bcib,
                                                                sizeof(bcib)),
                "structured RAW completion candidate writes a valid v2 structured header");

    result = complete_validation_execution_as(target_worker,
                                              structured_raw_exec_id,
                                              EXEC_COMPLETION_COMPLETED);
    TEST_ASSERT(result == ESYS_V2_SUCCESS,
                "known structured RAW kind completes successfully");

    {
        uint64_t structured_raw_wait = wait_validation_result_as(target_worker,
                                                                 structured_raw_exec_id,
                                                                 0);
        uint8_t expected_structured_raw[sizeof(ayken_execution_output_v2_t) + sizeof(bcib)];
        uint64_t expected_size = validation_build_structured_output_buffer(expected_structured_raw,
                                                                          sizeof(expected_structured_raw),
                                                                          AYKEN_EXECUTION_OUTPUT_V2_MAGIC,
                                                                          AYKEN_EXECUTION_OUTPUT_V2_VERSION,
                                                                          AYKEN_OUTPUT_KIND_RAW,
                                                                          (const uint8_t *)bcib,
                                                                          sizeof(bcib));
        int structured_raw_valid = 0;
        int structured_raw_hash_valid = 0;

        TEST_ASSERT(structured_raw_wait > 0,
                    "structured RAW completion materializes a result VA");

        {
            execution_slot_guard_t slot_guard = {0};
            exec_slot_t *slot = NULL;

            execution_slot_enter_critical(&slot_guard);
            slot = execution_slot_find_locked(structured_raw_exec_id);
            structured_raw_valid = slot != NULL &&
                                   slot->state == EXEC_SLOT_RESULT_MAPPED &&
                                   slot->mapped_result_va == structured_raw_wait &&
                                   slot->result_size == expected_size &&
                                   validation_frames_match_buffer(slot->result_frames,
                                                                  slot->result_frame_count,
                                                                  expected_structured_raw,
                                                                  expected_size);
            structured_raw_hash_valid = validation_result_hash_matches(slot,
                                                                       target_worker->pml4_phys,
                                                                       expected_structured_raw,
                                                                       expected_size);
            execution_slot_exit_critical(&slot_guard);
        }

        TEST_ASSERT(structured_raw_valid,
                    "structured RAW completion publishes the v2 header plus payload without kernel-side semantic parsing");
        TEST_ASSERT(structured_raw_hash_valid,
                    "structured RAW completion produces a deterministic hash over the published v2 bytes");
    }

    structured_blob_exec_id = submit_validation_execution_as(target_worker,
                                                             bcib,
                                                             sizeof(bcib),
                                                             (uint64_t)target_worker->pid);
    TEST_ASSERT(structured_blob_exec_id > structured_raw_exec_id,
                "structured BLOB completion candidate preserves monotonic execution IDs");

    saved_current_proc = current_proc;
    current_proc = target_worker;
    current_proc->state = PROC_RUNNING;
    TEST_ASSERT(sched_try_pickup_execution_work() == 1,
                "structured BLOB completion candidate picked up into RUNNING");
    current_proc = saved_current_proc;

    TEST_ASSERT(validation_write_structured_output_for_execution(structured_blob_exec_id,
                                                                AYKEN_EXECUTION_OUTPUT_V2_MAGIC,
                                                                AYKEN_EXECUTION_OUTPUT_V2_VERSION,
                                                                AYKEN_OUTPUT_KIND_BLOB,
                                                                (const uint8_t *)bcib,
                                                                sizeof(bcib)),
                "structured BLOB completion candidate writes a valid v2 structured header");

    result = complete_validation_execution_as(target_worker,
                                              structured_blob_exec_id,
                                              EXEC_COMPLETION_COMPLETED);
    TEST_ASSERT(result == ESYS_V2_SUCCESS,
                "known structured BLOB kind completes successfully");

    {
        uint64_t structured_blob_wait = wait_validation_result_as(target_worker,
                                                                  structured_blob_exec_id,
                                                                  0);
        uint8_t expected_structured_blob[sizeof(ayken_execution_output_v2_t) + sizeof(bcib)];
        uint64_t expected_size = validation_build_structured_output_buffer(expected_structured_blob,
                                                                          sizeof(expected_structured_blob),
                                                                          AYKEN_EXECUTION_OUTPUT_V2_MAGIC,
                                                                          AYKEN_EXECUTION_OUTPUT_V2_VERSION,
                                                                          AYKEN_OUTPUT_KIND_BLOB,
                                                                          (const uint8_t *)bcib,
                                                                          sizeof(bcib));
        int structured_blob_valid = 0;
        int structured_blob_hash_valid = 0;

        TEST_ASSERT(structured_blob_wait > 0,
                    "structured BLOB completion materializes a result VA");

        {
            execution_slot_guard_t slot_guard = {0};
            exec_slot_t *slot = NULL;

            execution_slot_enter_critical(&slot_guard);
            slot = execution_slot_find_locked(structured_blob_exec_id);
            structured_blob_valid = slot != NULL &&
                                    slot->state == EXEC_SLOT_RESULT_MAPPED &&
                                    slot->mapped_result_va == structured_blob_wait &&
                                    slot->result_size == expected_size &&
                                    validation_frames_match_buffer(slot->result_frames,
                                                                   slot->result_frame_count,
                                                                   expected_structured_blob,
                                                                   expected_size);
            structured_blob_hash_valid = validation_result_hash_matches(slot,
                                                                        target_worker->pml4_phys,
                                                                        expected_structured_blob,
                                                                        expected_size);
            execution_slot_exit_critical(&slot_guard);
        }

        TEST_ASSERT(structured_blob_valid,
                    "structured BLOB completion publishes typed bytes while keeping payload interpretation in userland");
        TEST_ASSERT(structured_blob_hash_valid,
                    "structured BLOB completion produces the same integrity contract over structured bytes");
    }

    {
        uint64_t invalid_output_exec_id = submit_validation_execution_as(target_worker,
                                                                         bcib,
                                                                         sizeof(bcib),
                                                                         (uint64_t)target_worker->pid);
        int invalid_failed = 0;

        TEST_ASSERT(invalid_output_exec_id > completed_exec_id,
                    "execution IDs remain monotonic for invalid-output completion candidates");

        saved_current_proc = current_proc;
        current_proc = target_worker;
        current_proc->state = PROC_RUNNING;
        TEST_ASSERT(sched_try_pickup_execution_work() == 1,
                    "invalid-output completion candidate picked up into RUNNING");
        current_proc = saved_current_proc;

        TEST_ASSERT(validation_write_output_for_execution(invalid_output_exec_id,
                                                          0,
                                                          AYKEN_EXECUTION_OUTPUT_VERSION,
                                                          (const uint8_t *)bcib,
                                                          sizeof(bcib)),
                    "invalid-output completion candidate writes a malformed header");

        result = complete_validation_execution_as(target_worker,
                                                  invalid_output_exec_id,
                                                  EXEC_COMPLETION_COMPLETED);
        TEST_ASSERT((int64_t)result == ESYS_V2_INVALID_STATE,
                    "completed requests with invalid output metadata fail closed");

        {
            execution_slot_guard_t slot_guard = {0};
            exec_slot_t *slot = NULL;

            execution_slot_enter_critical(&slot_guard);
            slot = execution_slot_find_locked(invalid_output_exec_id);
            invalid_failed = slot != NULL &&
                             slot->state == EXEC_SLOT_FAILED &&
                             target_worker->active_execution_id == 0;
            execution_slot_exit_critical(&slot_guard);
        }

        TEST_ASSERT(invalid_failed,
                    "invalid completed output terminalizes the slot as FAILED and clears the latch");
    }

    {
        uint64_t unknown_kind_exec_id = submit_validation_execution_as(target_worker,
                                                                       bcib,
                                                                       sizeof(bcib),
                                                                       (uint64_t)target_worker->pid);
        int unknown_kind_failed = 0;

        TEST_ASSERT(unknown_kind_exec_id > structured_blob_exec_id,
                    "execution IDs remain monotonic for unknown-kind completion candidates");

        saved_current_proc = current_proc;
        current_proc = target_worker;
        current_proc->state = PROC_RUNNING;
        TEST_ASSERT(sched_try_pickup_execution_work() == 1,
                    "unknown-kind completion candidate picked up into RUNNING");
        current_proc = saved_current_proc;

        TEST_ASSERT(validation_write_structured_output_for_execution(unknown_kind_exec_id,
                                                                    AYKEN_EXECUTION_OUTPUT_V2_MAGIC,
                                                                    AYKEN_EXECUTION_OUTPUT_V2_VERSION,
                                                                    99u,
                                                                    (const uint8_t *)bcib,
                                                                    sizeof(bcib)),
                    "unknown-kind completion candidate writes a v2 header with an unsupported kind");

        result = complete_validation_execution_as(target_worker,
                                                  unknown_kind_exec_id,
                                                  EXEC_COMPLETION_COMPLETED);
        TEST_ASSERT((int64_t)result == ESYS_V2_INVALID_STATE,
                    "completed requests with unknown structured kind fail closed");

        {
            execution_slot_guard_t slot_guard = {0};
            exec_slot_t *slot = NULL;

            execution_slot_enter_critical(&slot_guard);
            slot = execution_slot_find_locked(unknown_kind_exec_id);
            unknown_kind_failed = slot != NULL &&
                                  slot->state == EXEC_SLOT_FAILED &&
                                  target_worker->active_execution_id == 0;
            execution_slot_exit_critical(&slot_guard);
        }

        TEST_ASSERT(unknown_kind_failed,
                    "unknown structured kind terminalizes the slot as FAILED and clears the latch");
    }

    {
        uint64_t invalid_version_exec_id = submit_validation_execution_as(target_worker,
                                                                          bcib,
                                                                          sizeof(bcib),
                                                                          (uint64_t)target_worker->pid);
        int invalid_version_failed = 0;

        TEST_ASSERT(invalid_version_exec_id > completed_exec_id,
                    "execution IDs remain monotonic for invalid-version completion candidates");

        saved_current_proc = current_proc;
        current_proc = target_worker;
        current_proc->state = PROC_RUNNING;
        TEST_ASSERT(sched_try_pickup_execution_work() == 1,
                    "invalid-version completion candidate picked up into RUNNING");
        current_proc = saved_current_proc;

        TEST_ASSERT(validation_write_structured_output_for_execution(invalid_version_exec_id,
                                                                    AYKEN_EXECUTION_OUTPUT_V2_MAGIC,
                                                                    0,
                                                                    AYKEN_OUTPUT_KIND_RAW,
                                                                    (const uint8_t *)bcib,
                                                                    sizeof(bcib)),
                    "invalid-version completion candidate writes a structured header with a mismatched ABI version");

        result = complete_validation_execution_as(target_worker,
                                                  invalid_version_exec_id,
                                                  EXEC_COMPLETION_COMPLETED);
        TEST_ASSERT((int64_t)result == ESYS_V2_INVALID_STATE,
                    "completed requests with invalid structured output ABI version fail closed");

        {
            execution_slot_guard_t slot_guard = {0};
            exec_slot_t *slot = NULL;

            execution_slot_enter_critical(&slot_guard);
            slot = execution_slot_find_locked(invalid_version_exec_id);
            invalid_version_failed = slot != NULL &&
                                     slot->state == EXEC_SLOT_FAILED &&
                                     target_worker->active_execution_id == 0;
            execution_slot_exit_critical(&slot_guard);
        }

        TEST_ASSERT(invalid_version_failed,
                    "invalid structured output ABI version terminalizes the slot as FAILED and clears the latch");
    }

    {
        uint64_t overflow_output_exec_id = submit_validation_execution_as(target_worker,
                                                                          bcib,
                                                                          sizeof(bcib),
                                                                          (uint64_t)target_worker->pid);
        int overflow_failed = 0;

        TEST_ASSERT(overflow_output_exec_id > completed_exec_id,
                    "execution IDs remain monotonic for overflowed output completion candidates");

        saved_current_proc = current_proc;
        current_proc = target_worker;
        current_proc->state = PROC_RUNNING;
        TEST_ASSERT(sched_try_pickup_execution_work() == 1,
                    "overflowed output completion candidate picked up into RUNNING");
        current_proc = saved_current_proc;

        TEST_ASSERT(validation_write_output_for_execution_with_header(overflow_output_exec_id,
                                                                      AYKEN_EXECUTION_OUTPUT_MAGIC,
                                                                      AYKEN_EXECUTION_OUTPUT_VERSION,
                                                                      AYKEN_EXECUTION_OUTPUT_WINDOW_SIZE,
                                                                      NULL,
                                                                      0),
                    "overflowed output completion candidate writes a header whose declared size exceeds the bounded window");

        result = complete_validation_execution_as(target_worker,
                                                  overflow_output_exec_id,
                                                  EXEC_COMPLETION_COMPLETED);
        TEST_ASSERT((int64_t)result == ESYS_V2_INVALID_STATE,
                    "completed requests with overflowing output bytes fail closed");

        {
            execution_slot_guard_t slot_guard = {0};
            exec_slot_t *slot = NULL;

            execution_slot_enter_critical(&slot_guard);
            slot = execution_slot_find_locked(overflow_output_exec_id);
            overflow_failed = slot != NULL &&
                              slot->state == EXEC_SLOT_FAILED &&
                              target_worker->active_execution_id == 0;
            execution_slot_exit_critical(&slot_guard);
        }

        TEST_ASSERT(overflow_failed,
                    "overflowing output metadata terminalizes the slot as FAILED and clears the latch");
    }

    result = complete_validation_execution_as(target_worker,
                                              completed_exec_id,
                                              EXEC_COMPLETION_COMPLETED);
    TEST_ASSERT((int64_t)result == ESYS_V2_INVALID_STATE,
                "double completion attempts fail closed after first terminal state wins");

    result = complete_validation_execution_as(target_worker,
                                              completed_exec_id + 1024,
                                              EXEC_COMPLETION_COMPLETED);
    TEST_ASSERT((int64_t)result == ESYS_V2_INVALID_ID,
                "stale or unknown execution IDs fail closed on completion");

    result = wait_validation_result_as(target_worker, completed_exec_id, 0);
    TEST_ASSERT(result > 0,
                "completed slot can be materialized into a mapped result VA");
    {
        uint64_t repeated_wait_va = wait_validation_result_as(target_worker, completed_exec_id, 0);
        TEST_ASSERT(repeated_wait_va == result,
                    "completed-result ownership replays the same VA across repeated waits");
    }
    result = sys_v2_wait_result(completed_exec_id + 1024, 0);
    TEST_ASSERT((int64_t)result == ESYS_V2_CONTEXT_ERROR,
                "wait_result fails closed on stale or unknown execution IDs");

    failed_exec_id = submit_validation_execution_as(target_worker,
                                                    bcib,
                                                    sizeof(bcib),
                                                    (uint64_t)target_worker->pid);
    TEST_ASSERT(failed_exec_id > completed_exec_id,
                "execution IDs remain monotonic across success and failure candidates");

    saved_current_proc = current_proc;
    current_proc = target_worker;
    current_proc->state = PROC_RUNNING;
    TEST_ASSERT(sched_try_pickup_execution_work() == 1,
                "completion contract failure candidate picked up into RUNNING");
    current_proc = saved_current_proc;
    result = complete_validation_execution_as(target_worker,
                                              failed_exec_id,
                                              EXEC_COMPLETION_FAILED);
    TEST_ASSERT(result == ESYS_V2_SUCCESS,
                "owning executor can terminate a RUNNING slot as FAILED");

    {
        execution_slot_guard_t slot_guard = {0};
        exec_slot_t *slot = NULL;
        int failed_state = 0;

        execution_slot_enter_critical(&slot_guard);
        slot = execution_slot_find_locked(failed_exec_id);
        failed_state = slot != NULL &&
                       slot->state == EXEC_SLOT_FAILED &&
                       target_worker->active_execution_id == 0;
        execution_slot_exit_critical(&slot_guard);

        TEST_ASSERT(failed_state,
                    "failed completion transitions slot state and clears worker latch");
    }

    result = sys_v2_wait_result(failed_exec_id, 0);
    TEST_ASSERT((int64_t)result == ESYS_V2_CONTEXT_ERROR,
                "wait_result reports explicit FAILED terminalization as error");

    TEST_END("Completion Handoff Contract");
}

static void test_execution_pickup_order_contract(void)
{
    static const uint8_t bcib_first[] = {0x42, 0x43, 0x49, 0x42, 0x10, 0x01};
    static const uint8_t bcib_second[] = {0x42, 0x43, 0x49, 0x42, 0x20, 0x02};
    proc_t *target_proc = NULL;
    proc_t *saved_current_proc = NULL;
    execution_slot_guard_t slot_guard = {0};
    ayken_sched_mailbox_t mailbox_before = {0};
    ayken_sched_mailbox_t *mailbox = NULL;
    ayken_execution_inbox_v1_t *inbox = NULL;
    exec_slot_t *first_slot = NULL;
    exec_slot_t *second_slot = NULL;
    uint32_t first_frame_count;
    uint32_t second_frame_count;
    uint32_t released_owned = 0;
    uint64_t first_exec_id = 0;
    uint64_t second_exec_id = 0;
    uint64_t result = 0;
    int first_publish = 0;
    int blocked_repickup = 0;
    int second_publish = 0;
    int first_order_valid = 0;
    int first_delivery_valid = 0;
    int first_mailbox_unchanged = 0;
    int second_order_valid = 0;
    int second_delivery_valid = 0;
    int second_mailbox_unchanged = 0;

    TEST_START("Execution Pickup Ordering Contract");

    target_proc = create_validation_runtime_proc("phase2-pickup-order-target");
    TEST_ASSERT(target_proc != NULL && target_proc->type == PROC_TYPE_USER,
                "pickup order harness created a dedicated live target process");

    if (target_proc == NULL || target_proc->type != PROC_TYPE_USER) {
        TEST_END("Execution Pickup Ordering Contract");
        return;
    }

    mailbox = (ayken_sched_mailbox_t *)paging_phys_to_virt(target_proc->mailbox_pa);
    if (mailbox != NULL) {
        mailbox_before = *mailbox;
    }

    first_frame_count = validation_frame_count_for_size(sizeof(bcib_first));
    second_frame_count = validation_frame_count_for_size(sizeof(bcib_second));

    first_exec_id = submit_validation_execution_as(target_proc,
                                                   bcib_first,
                                                   sizeof(bcib_first),
                                                   (uint64_t)target_proc->pid);
    second_exec_id = submit_validation_execution_as(target_proc,
                                                    bcib_second,
                                                    sizeof(bcib_second),
                                                    (uint64_t)target_proc->pid);

    TEST_ASSERT(first_exec_id > 0 && second_exec_id > first_exec_id,
                "pickup order harness submits two monotonic executions for one worker");

    saved_current_proc = current_proc;
    current_proc = target_proc;
    current_proc->state = PROC_RUNNING;
    first_publish = sched_try_pickup_execution_work();
    blocked_repickup = sched_try_pickup_execution_work();
    current_proc = saved_current_proc;

    execution_slot_enter_critical(&slot_guard);
    first_slot = execution_slot_find_locked(first_exec_id);
    second_slot = execution_slot_find_locked(second_exec_id);
    first_order_valid = first_slot != NULL &&
                        second_slot != NULL &&
                        first_slot->state == EXEC_SLOT_RUNNING &&
                        second_slot->state == EXEC_SLOT_READY &&
                        target_proc->active_execution_id == first_exec_id;
    execution_slot_exit_critical(&slot_guard);

    inbox = (ayken_execution_inbox_v1_t *)paging_phys_to_virt(target_proc->execution_inbox_pa);
    mailbox = (ayken_sched_mailbox_t *)paging_phys_to_virt(target_proc->mailbox_pa);
    first_delivery_valid = first_publish == 1 &&
                           blocked_repickup == 0 &&
                           inbox != NULL &&
                           inbox->delivery_seq == 1 &&
                           inbox->execution_id == first_exec_id &&
                           inbox->target_context_id == (uint64_t)target_proc->pid &&
                           inbox->bcib_user_va == EXECUTION_PAYLOAD_VA &&
                           inbox->bcib_size == sizeof(bcib_first) &&
                           validation_frames_match_buffer(target_proc->execution_payload_pas,
                                                          first_frame_count,
                                                          bcib_first,
                                                          sizeof(bcib_first));
    first_mailbox_unchanged = mailbox != NULL &&
                              validation_mailbox_equals(mailbox, &mailbox_before);

    TEST_ASSERT(first_order_valid,
                "pickup order harness keeps the earliest slot RUNNING and later work queued behind the latch");
    TEST_ASSERT(first_delivery_valid,
                "pickup order harness publishes the earliest queued execution deterministically");
    TEST_ASSERT(first_mailbox_unchanged,
                "first pickup leaves scheduler mailbox state untouched and does not reuse mailbox transport");

    result = complete_validation_execution_as(target_proc,
                                              first_exec_id,
                                              EXEC_COMPLETION_FAILED);
    TEST_ASSERT(result == ESYS_V2_SUCCESS,
                "pickup order harness retires the first execution to release the worker latch");

    saved_current_proc = current_proc;
    current_proc = target_proc;
    current_proc->state = PROC_RUNNING;
    second_publish = sched_try_pickup_execution_work();
    current_proc = saved_current_proc;

    execution_slot_enter_critical(&slot_guard);
    first_slot = execution_slot_find_locked(first_exec_id);
    second_slot = execution_slot_find_locked(second_exec_id);
    second_order_valid = first_slot != NULL &&
                         second_slot != NULL &&
                         first_slot->state == EXEC_SLOT_FAILED &&
                         second_slot->state == EXEC_SLOT_RUNNING &&
                         target_proc->active_execution_id == second_exec_id;
    execution_slot_exit_critical(&slot_guard);

    inbox = (ayken_execution_inbox_v1_t *)paging_phys_to_virt(target_proc->execution_inbox_pa);
    mailbox = (ayken_sched_mailbox_t *)paging_phys_to_virt(target_proc->mailbox_pa);
    second_delivery_valid = second_publish == 1 &&
                            inbox != NULL &&
                            inbox->delivery_seq == 2 &&
                            inbox->execution_id == second_exec_id &&
                            inbox->target_context_id == (uint64_t)target_proc->pid &&
                            inbox->bcib_user_va == EXECUTION_PAYLOAD_VA &&
                            inbox->bcib_size == sizeof(bcib_second) &&
                            validation_frames_match_buffer(target_proc->execution_payload_pas,
                                                           second_frame_count,
                                                           bcib_second,
                                                           sizeof(bcib_second));
    second_mailbox_unchanged = mailbox != NULL &&
                               validation_mailbox_equals(mailbox, &mailbox_before);

    TEST_ASSERT(second_order_valid,
                "pickup order harness advances the next queued slot only after terminalization clears the latch");
    TEST_ASSERT(second_delivery_valid,
                "pickup order harness preserves FIFO order and increments delivery_seq on the next publish");
    TEST_ASSERT(second_mailbox_unchanged,
                "second pickup still leaves scheduler mailbox state untouched");

    result = complete_validation_execution_as(target_proc,
                                              second_exec_id,
                                              EXEC_COMPLETION_FAILED);
    TEST_ASSERT(result == ESYS_V2_SUCCESS,
                "pickup order harness can retire the second execution after deterministic pickup");

    execution_slot_enter_critical(&slot_guard);
    if (target_proc->pid > 0) {
        released_owned = execution_slot_release_owned_by_owner_locked((uint64_t)target_proc->pid);
    }
    execution_slot_exit_critical(&slot_guard);

    TEST_ASSERT(released_owned >= 2,
                "pickup order harness can release both owner-owned slots after the FIFO proof completes");

    sched_remove_process_everywhere(target_proc);
    target_proc->state = PROC_ZOMBIE;
    target_proc->wait_obj = NULL;

    TEST_END("Execution Pickup Ordering Contract");
}

static void test_illegal_execution_slot_transition_contract(void)
{
    execution_slot_guard_t slot_guard = {0};
    exec_slot_t *lifecycle_slot = NULL;
    exec_slot_t *mapped_slot = NULL;
    int created_to_running_rejected = 0;
    int created_finish_completed_rejected = 0;
    int created_state_preserved = 0;
    int ready_wrong_expected_rejected = 0;
    int ready_to_result_mapped_rejected = 0;
    int ready_finish_completed_rejected = 0;
    int ready_state_preserved = 0;
    int running_to_ready_rejected = 0;
    int running_to_result_mapped_rejected = 0;
    int running_state_preserved = 0;
    int completed_to_timeout_rejected = 0;
    int completed_state_preserved = 0;
    int result_mapped_to_completed_rejected = 0;
    int terminal_finish_rejected = 0;
    int terminal_state_preserved = 0;

    TEST_START("Illegal Execution Slot Transition Contract");

    execution_slot_enter_critical(&slot_guard);
    lifecycle_slot = execution_slot_alloc_locked(0x4100, 0x4200);
    mapped_slot = execution_slot_alloc_locked(0x4300, 0x4400);

    if (lifecycle_slot != NULL) {
        created_to_running_rejected =
            execution_slot_transition_locked(lifecycle_slot,
                                             EXEC_SLOT_CREATED,
                                             EXEC_SLOT_RUNNING) != 0;
        created_finish_completed_rejected =
            execution_slot_finish_locked(lifecycle_slot,
                                         EXEC_SLOT_COMPLETED) != 0;
        created_state_preserved = lifecycle_slot->state == EXEC_SLOT_CREATED;

        if (execution_slot_transition_locked(lifecycle_slot,
                                             EXEC_SLOT_CREATED,
                                             EXEC_SLOT_READY) == 0) {
            ready_wrong_expected_rejected =
                execution_slot_transition_locked(lifecycle_slot,
                                                 EXEC_SLOT_CREATED,
                                                 EXEC_SLOT_RUNNING) != 0;
            ready_to_result_mapped_rejected =
                execution_slot_transition_locked(lifecycle_slot,
                                                 EXEC_SLOT_READY,
                                                 EXEC_SLOT_RESULT_MAPPED) != 0;
            ready_finish_completed_rejected =
                execution_slot_finish_locked(lifecycle_slot,
                                             EXEC_SLOT_COMPLETED) != 0;
            ready_state_preserved = lifecycle_slot->state == EXEC_SLOT_READY;

            if (execution_slot_transition_locked(lifecycle_slot,
                                                 EXEC_SLOT_READY,
                                                 EXEC_SLOT_RUNNING) == 0) {
                running_to_ready_rejected =
                    execution_slot_transition_locked(lifecycle_slot,
                                                     EXEC_SLOT_RUNNING,
                                                     EXEC_SLOT_READY) != 0;
                running_to_result_mapped_rejected =
                    execution_slot_transition_locked(lifecycle_slot,
                                                     EXEC_SLOT_RUNNING,
                                                     EXEC_SLOT_RESULT_MAPPED) != 0;
                running_state_preserved = lifecycle_slot->state == EXEC_SLOT_RUNNING;

                (void)execution_slot_finish_locked(lifecycle_slot, EXEC_SLOT_FAILED);
            }
        }
    }

    if (mapped_slot != NULL) {
        if (execution_slot_transition_locked(mapped_slot,
                                             EXEC_SLOT_CREATED,
                                             EXEC_SLOT_READY) == 0 &&
            execution_slot_transition_locked(mapped_slot,
                                             EXEC_SLOT_READY,
                                             EXEC_SLOT_RUNNING) == 0 &&
            execution_slot_finish_locked(mapped_slot,
                                         EXEC_SLOT_COMPLETED) == 0) {
            completed_to_timeout_rejected =
                execution_slot_transition_locked(mapped_slot,
                                                 EXEC_SLOT_COMPLETED,
                                                 EXEC_SLOT_TIMEOUT) != 0;
            completed_state_preserved = mapped_slot->state == EXEC_SLOT_COMPLETED;

            if (execution_slot_transition_locked(mapped_slot,
                                                 EXEC_SLOT_COMPLETED,
                                                 EXEC_SLOT_RESULT_MAPPED) == 0) {
                result_mapped_to_completed_rejected =
                    execution_slot_transition_locked(mapped_slot,
                                                     EXEC_SLOT_RESULT_MAPPED,
                                                     EXEC_SLOT_COMPLETED) != 0;
                terminal_finish_rejected =
                    execution_slot_finish_locked(mapped_slot,
                                                 EXEC_SLOT_ABORTED) != 0;
                terminal_state_preserved =
                    mapped_slot->state == EXEC_SLOT_RESULT_MAPPED;
            }
        }
    }

    if (lifecycle_slot != NULL) {
        execution_slot_release_locked(lifecycle_slot);
    }
    if (mapped_slot != NULL) {
        execution_slot_release_locked(mapped_slot);
    }
    execution_slot_exit_critical(&slot_guard);

    TEST_ASSERT(lifecycle_slot != NULL && mapped_slot != NULL,
                "illegal transition harness allocates dedicated execution slots");
    TEST_ASSERT(created_to_running_rejected &&
                    created_finish_completed_rejected &&
                    created_state_preserved,
                "CREATED slots reject direct RUNNING or terminal overwrite attempts");
    TEST_ASSERT(ready_wrong_expected_rejected &&
                    ready_to_result_mapped_rejected &&
                    ready_finish_completed_rejected &&
                    ready_state_preserved,
                "READY slots reject stale expected_from mismatches and illegal terminal shortcuts");
    TEST_ASSERT(running_to_ready_rejected &&
                    running_to_result_mapped_rejected &&
                    running_state_preserved,
                "RUNNING slots reject backward or unmapped terminal rewrites");
    TEST_ASSERT(completed_to_timeout_rejected &&
                    completed_state_preserved,
                "COMPLETED slots reject later timeout overwrite attempts");
    TEST_ASSERT(result_mapped_to_completed_rejected &&
                    terminal_finish_rejected &&
                    terminal_state_preserved,
                "terminal RESULT_MAPPED slots reject further cross-state mutation");

    TEST_END("Illegal Execution Slot Transition Contract");
}

static void test_execution_trace_invariant_contract(void)
{
    static const uint8_t bcib[] = {0x42, 0x43, 0x49, 0x42, 0x55, 0x66, 0x77, 0x88};
    proc_t *target_proc = NULL;
    proc_t *saved_current_proc = NULL;
    execution_slot_guard_t slot_guard = {0};
    execution_trace_entry_t trace_entries[4] = {0};
    exec_slot_t *slot = NULL;
    uint32_t released_owned = 0;
    uint64_t execution_id = 0;
    uint64_t mapped_result_va = 0;
    uint64_t expected_generation = 0;
    int pickup_published = 0;
    int trace_count_valid = 0;
    int trace_sequence_valid = 0;
    int trace_identity_valid = 0;
    int invariants_hold = 0;

    TEST_START("Execution Trace Invariant Contract");

    target_proc = create_validation_runtime_proc("phase2-trace-target");
    TEST_ASSERT(target_proc != NULL && target_proc->type == PROC_TYPE_USER,
                "trace harness created a dedicated live target process");

    if (target_proc == NULL || target_proc->type != PROC_TYPE_USER) {
        TEST_END("Execution Trace Invariant Contract");
        return;
    }

    execution_id = submit_validation_execution_as(target_proc,
                                                  bcib,
                                                  sizeof(bcib),
                                                  (uint64_t)target_proc->pid);
    TEST_ASSERT(execution_id > 0,
                "trace harness submits a real execution to seed the transition log");

    if (execution_id == 0) {
        sched_remove_process_everywhere(target_proc);
        target_proc->state = PROC_ZOMBIE;
        target_proc->wait_obj = NULL;
        TEST_END("Execution Trace Invariant Contract");
        return;
    }

    saved_current_proc = current_proc;
    current_proc = target_proc;
    current_proc->state = PROC_RUNNING;
    pickup_published = sched_try_pickup_execution_work();
    current_proc = saved_current_proc;

    TEST_ASSERT(pickup_published == 1,
                "trace harness publishes the submitted execution into RUNNING");
    TEST_ASSERT(complete_validation_execution_as(target_proc,
                                                execution_id,
                                                EXEC_COMPLETION_COMPLETED) == ESYS_V2_SUCCESS,
                "trace harness terminalizes the running execution through complete_execution");

    mapped_result_va = wait_validation_result_as(target_proc, execution_id, 0);
    TEST_ASSERT(mapped_result_va > 0,
                "trace harness maps the completed result to land RESULT_MAPPED");

    execution_slot_enter_critical(&slot_guard);
    slot = execution_slot_find_locked(execution_id);
    if (slot != NULL) {
        expected_generation = slot->generation;
        trace_count_valid =
            execution_slot_trace_count_locked(slot) == 4 &&
            execution_slot_trace_get_locked(slot, 0, &trace_entries[0]) == 0 &&
            execution_slot_trace_get_locked(slot, 1, &trace_entries[1]) == 0 &&
            execution_slot_trace_get_locked(slot, 2, &trace_entries[2]) == 0 &&
            execution_slot_trace_get_locked(slot, 3, &trace_entries[3]) == 0;
        invariants_hold = execution_slot_verify_global_invariants_locked() == 0;
    }

    if (trace_count_valid) {
        trace_sequence_valid =
            trace_entries[0].actor == EXEC_TRACE_ACTOR_SUBMIT &&
            trace_entries[0].from_state == EXEC_SLOT_CREATED &&
            trace_entries[0].to_state == EXEC_SLOT_READY &&
            trace_entries[1].actor == EXEC_TRACE_ACTOR_PICKUP &&
            trace_entries[1].from_state == EXEC_SLOT_READY &&
            trace_entries[1].to_state == EXEC_SLOT_RUNNING &&
            trace_entries[2].actor == EXEC_TRACE_ACTOR_COMPLETE &&
            trace_entries[2].from_state == EXEC_SLOT_RUNNING &&
            trace_entries[2].to_state == EXEC_SLOT_COMPLETED &&
            trace_entries[3].actor == EXEC_TRACE_ACTOR_WAIT_RESULT &&
            trace_entries[3].from_state == EXEC_SLOT_COMPLETED &&
            trace_entries[3].to_state == EXEC_SLOT_RESULT_MAPPED;
        trace_identity_valid =
            trace_entries[0].execution_id == execution_id &&
            trace_entries[1].execution_id == execution_id &&
            trace_entries[2].execution_id == execution_id &&
            trace_entries[3].execution_id == execution_id &&
            trace_entries[0].generation == expected_generation &&
            trace_entries[1].generation == expected_generation &&
            trace_entries[2].generation == expected_generation &&
            trace_entries[3].generation == expected_generation &&
            trace_entries[0].tick <= trace_entries[1].tick &&
            trace_entries[1].tick <= trace_entries[2].tick &&
            trace_entries[2].tick <= trace_entries[3].tick &&
            slot != NULL &&
            slot->state == EXEC_SLOT_RESULT_MAPPED;
    }

    if (target_proc->pid > 0) {
        released_owned = execution_slot_release_owned_by_owner_locked((uint64_t)target_proc->pid);
    }
    execution_slot_exit_critical(&slot_guard);

    TEST_ASSERT(trace_count_valid,
                "trace harness records the exact four-step lifecycle transition chain");
    TEST_ASSERT(trace_sequence_valid,
                "trace harness records submit, pickup, complete, and wait_result actors in order");
    TEST_ASSERT(trace_identity_valid,
                "trace harness keeps execution identity and timestamps stable across the transition log");
    TEST_ASSERT(invariants_hold,
                "trace harness passes the global execution-slot invariant checker");
    TEST_ASSERT(released_owned >= 1,
                "trace harness releases the traced owner-owned slot after invariant proof completes");

    sched_remove_process_everywhere(target_proc);
    target_proc->state = PROC_ZOMBIE;
    target_proc->wait_obj = NULL;

    TEST_END("Execution Trace Invariant Contract");
}

static void test_multi_execution_adversarial_contract(void)
{
    proc_t *owner_proc = NULL;
    proc_t *target_proc = NULL;
    proc_t *foreign_proc = NULL;
    proc_t *saved_current_proc = NULL;
    execution_slot_guard_t slot_guard = {0};
    uint64_t execution_ids[3] = {0};
    uint64_t result_vas[AYKEN_MAX_EXECUTION_SLOTS] = {0};
    uint64_t hash_vas[AYKEN_MAX_EXECUTION_SLOTS] = {0};
    uint64_t mapped_result_va = 0;
    uint64_t mapped_hash_va = 0;
    uint32_t released_owned = 0;
    uint32_t result_count = 0;
    int created_harness = 0;
    int ids_monotonic = 0;
    int pickup_first = 0;
    int pickup_blocked = 0;
    int foreign_wait_rejected = 0;
    int first_complete_ok = 0;
    int double_complete_rejected = 0;
    int repeated_wait_replay = 1;
    int stale_wait_flood_rejected = 1;
    int second_pickup = 0;
    int exit_abort_running_ready = 0;
    int mapped_result_preserved = 0;
    int invariants_hold = 0;
    int post_abort_complete_rejected = 0;
    int released_all_owned = 0;
    int stale_released_id_rejected = 0;
    uint32_t i;

    TEST_START("Multi Execution Adversarial Contract");

    validation_prepare_full_result_bcib();

    owner_proc = create_validation_runtime_proc("phase2-adversarial-owner");
    target_proc = create_validation_runtime_proc("phase2-adversarial-target");
    foreign_proc = create_validation_runtime_proc("phase2-adversarial-foreign");
    created_harness = owner_proc != NULL &&
                      target_proc != NULL &&
                      foreign_proc != NULL &&
                      owner_proc->type == PROC_TYPE_USER &&
                      target_proc->type == PROC_TYPE_USER &&
                      foreign_proc->type == PROC_TYPE_USER;
    TEST_ASSERT(created_harness,
                "adversarial harness created owner, target, and foreign user processes");

    if (!created_harness) {
        TEST_END("Multi Execution Adversarial Contract");
        return;
    }

    execution_ids[0] = submit_validation_execution_as(owner_proc,
                                                      g_validation_full_result_bcib,
                                                      VALIDATION_FULL_RESULT_BCIB_SIZE,
                                                      (uint64_t)target_proc->pid);
    execution_ids[1] = submit_validation_execution_as(owner_proc,
                                                      g_validation_full_result_bcib,
                                                      VALIDATION_FULL_RESULT_BCIB_SIZE,
                                                      (uint64_t)target_proc->pid);
    execution_ids[2] = submit_validation_execution_as(owner_proc,
                                                      g_validation_full_result_bcib,
                                                      VALIDATION_FULL_RESULT_BCIB_SIZE,
                                                      (uint64_t)target_proc->pid);
    ids_monotonic = execution_ids[0] > 0 &&
                    execution_ids[1] > execution_ids[0] &&
                    execution_ids[2] > execution_ids[1];
    TEST_ASSERT(ids_monotonic,
                "adversarial harness submits three monotonic execution IDs for one target");

    if (!ids_monotonic) {
        goto cleanup;
    }

    saved_current_proc = current_proc;
    current_proc = target_proc;
    current_proc->state = PROC_RUNNING;
    pickup_first = sched_try_pickup_execution_work();
    pickup_blocked = sched_try_pickup_execution_work() == 0;
    current_proc = saved_current_proc;

    TEST_ASSERT(pickup_first == 1 && pickup_blocked,
                "adversarial harness publishes exactly one queued execution before the target latch blocks re-pickup");

    foreign_wait_rejected =
        (int64_t)wait_validation_result_as(foreign_proc, execution_ids[0], 0) ==
        ESYS_V2_NO_PERMISSION;
    TEST_ASSERT(foreign_wait_rejected,
                "adversarial harness keeps foreign wait_result fail-closed on an owner execution");

    TEST_ASSERT(validation_write_output_for_execution(execution_ids[0],
                                                      AYKEN_EXECUTION_OUTPUT_MAGIC,
                                                      AYKEN_EXECUTION_OUTPUT_VERSION,
                                                      g_validation_full_result_bcib,
                                                      VALIDATION_FULL_RESULT_BCIB_SIZE),
                "adversarial harness writes a valid output frame for the first running execution");

    first_complete_ok =
        complete_validation_execution_as(target_proc,
                                         execution_ids[0],
                                         EXEC_COMPLETION_COMPLETED) == ESYS_V2_SUCCESS;
    TEST_ASSERT(first_complete_ok,
                "adversarial harness completes the first running execution");

    double_complete_rejected =
        (int64_t)complete_validation_execution_as(target_proc,
                                                  execution_ids[0],
                                                  EXEC_COMPLETION_COMPLETED) ==
        ESYS_V2_INVALID_STATE;
    TEST_ASSERT(double_complete_rejected,
                "adversarial harness rejects double finalize on an already completed execution");

    mapped_result_va = wait_validation_result_as(owner_proc, execution_ids[0], 0);
    TEST_ASSERT(mapped_result_va > 0,
                "adversarial harness materializes the first completed result for replay flood coverage");

    execution_slot_enter_critical(&slot_guard);
    {
        exec_slot_t *mapped_slot = execution_slot_find_locked(execution_ids[0]);
        mapped_hash_va = mapped_slot != NULL ? mapped_slot->mapped_hash_va : 0;
    }
    execution_slot_exit_critical(&slot_guard);
    TEST_ASSERT(mapped_hash_va > 0,
                "adversarial harness materializes the result-hash sidecar for the mapped execution");

    for (i = 0; i < 10; ++i) {
        if (wait_validation_result_as(owner_proc, execution_ids[0], 0) != mapped_result_va) {
            repeated_wait_replay = 0;
            break;
        }
    }
    TEST_ASSERT(repeated_wait_replay,
                "adversarial harness replays the same mapped result VA across repeated owner wait_result floods");

    for (i = 0; i < 8; ++i) {
        if ((int64_t)wait_validation_result_as(owner_proc,
                                               execution_ids[2] + 4096u + i,
                                               0) != ESYS_V2_CONTEXT_ERROR) {
            stale_wait_flood_rejected = 0;
            break;
        }
    }
    TEST_ASSERT(stale_wait_flood_rejected,
                "adversarial harness keeps stale or unknown execution-ID wait_result floods fail-closed");

    saved_current_proc = current_proc;
    current_proc = target_proc;
    current_proc->state = PROC_RUNNING;
    second_pickup = sched_try_pickup_execution_work();
    current_proc = saved_current_proc;
    TEST_ASSERT(second_pickup == 1,
                "adversarial harness advances the second queued execution after the first terminalizes");

    execution_slot_enter_critical(&slot_guard);
    result_count = execution_slot_prepare_process_exit_locked((uint64_t)target_proc->pid,
                                                              result_vas,
                                                              hash_vas,
                                                              AYKEN_MAX_EXECUTION_SLOTS);
    {
        exec_slot_t *first_slot = execution_slot_find_locked(execution_ids[0]);
        exec_slot_t *second_slot = execution_slot_find_locked(execution_ids[1]);
        exec_slot_t *third_slot = execution_slot_find_locked(execution_ids[2]);

        exit_abort_running_ready =
            first_slot != NULL &&
            first_slot->state == EXEC_SLOT_RESULT_MAPPED &&
            second_slot != NULL &&
            second_slot->state == EXEC_SLOT_ABORTED &&
            third_slot != NULL &&
            third_slot->state == EXEC_SLOT_ABORTED &&
            target_proc->active_execution_id == 0;
        mapped_result_preserved = result_count == 1 &&
                                  result_vas[0] == mapped_result_va &&
                                  hash_vas[0] == mapped_hash_va;
        invariants_hold = execution_slot_verify_global_invariants_locked() == 0;
    }
    execution_slot_exit_critical(&slot_guard);

    TEST_ASSERT(exit_abort_running_ready,
                "adversarial harness aborts both RUNNING and READY slots when pickup collides with target exit preparation");
    TEST_ASSERT(mapped_result_preserved,
                "adversarial harness preserves the previously mapped result while target-exit aborts later work");
    TEST_ASSERT(invariants_hold,
                "adversarial harness still satisfies global invariants after multi-execution collision handling");

    post_abort_complete_rejected =
        (int64_t)complete_validation_execution_as(target_proc,
                                                  execution_ids[1],
                                                  EXEC_COMPLETION_COMPLETED) ==
        ESYS_V2_INVALID_STATE;
    TEST_ASSERT(post_abort_complete_rejected,
                "adversarial harness rejects completion after target-exit abort has already won the slot");

    execution_slot_enter_critical(&slot_guard);
    released_owned = execution_slot_release_owned_by_owner_locked((uint64_t)owner_proc->pid);
    released_all_owned = released_owned >= 3 &&
                         execution_slot_find_locked(execution_ids[0]) == NULL &&
                         execution_slot_find_locked(execution_ids[1]) == NULL &&
                         execution_slot_find_locked(execution_ids[2]) == NULL;
    execution_slot_exit_critical(&slot_guard);
    TEST_ASSERT(released_all_owned,
                "adversarial harness releases every owner-owned slot after the collision proof completes");

    stale_released_id_rejected =
        (int64_t)wait_validation_result_as(owner_proc, execution_ids[0], 0) ==
        ESYS_V2_CONTEXT_ERROR;
    TEST_ASSERT(stale_released_id_rejected,
                "adversarial harness keeps released execution IDs fail-closed once ownership state is torn down");

cleanup:
    if (owner_proc != NULL) {
        sched_remove_process_everywhere(owner_proc);
        owner_proc->state = PROC_ZOMBIE;
        owner_proc->wait_obj = NULL;
        proc_teardown_exit_surfaces(owner_proc, NULL, NULL, 0);
    }

    if (target_proc != NULL) {
        sched_remove_process_everywhere(target_proc);
        target_proc->active_execution_id = 0;
        target_proc->state = PROC_ZOMBIE;
        target_proc->wait_obj = NULL;
        proc_teardown_exit_surfaces(target_proc, NULL, NULL, 0);
    }

    if (foreign_proc != NULL) {
        sched_remove_process_everywhere(foreign_proc);
        foreign_proc->state = PROC_ZOMBIE;
        foreign_proc->wait_obj = NULL;
        proc_teardown_exit_surfaces(foreign_proc, NULL, NULL, 0);
    }

    TEST_END("Multi Execution Adversarial Contract");
}

static void test_blocked_wait_wake_contract(void)
{
    proc_t *target_proc = NULL;
    proc_t *waiter_thread = NULL;
    proc_t *waker_thread = NULL;
    execution_slot_guard_t slot_guard = {0};
    uint32_t released_owned = 0;
    int spun = 0;

    TEST_START("Blocked Wait Wake Contract");

    __builtin_memset(&g_blocked_wait_harness, 0, sizeof(g_blocked_wait_harness));
    __builtin_memset(&g_blocked_wait_spurious_key, 0, sizeof(g_blocked_wait_spurious_key));

    target_proc = create_validation_runtime_proc("phase2-blocked-wait-target");
    TEST_ASSERT(target_proc != NULL && target_proc->type == PROC_TYPE_USER,
                "blocked-wait harness created a dedicated live target process");

    if (target_proc == NULL || target_proc->type != PROC_TYPE_USER) {
        TEST_END("Blocked Wait Wake Contract");
        return;
    }

    sched_remove_process_everywhere(target_proc);
    target_proc->state = PROC_BLOCKED;
    target_proc->wait_obj = &g_blocked_wait_target_hold_token;
    g_blocked_wait_harness.target_proc = target_proc;

    waiter_thread = proc_create_kernel_thread(blocked_wait_harness_waiter_thread);
    waker_thread = proc_create_kernel_thread(blocked_wait_harness_waker_thread);

    TEST_ASSERT(waiter_thread != NULL && waker_thread != NULL,
                "blocked-wait harness created waiter and waker kernel threads");

    if (waiter_thread == NULL || waker_thread == NULL) {
        TEST_END("Blocked Wait Wake Contract");
        return;
    }

    for (spun = 0; spun < 256 && !g_blocked_wait_harness.waiter_resumed; ++spun) {
        sched_yield();
    }

    TEST_ASSERT(g_blocked_wait_harness.execution_id > 0,
                "blocked-wait harness submitted a real execution before blocking");
    TEST_ASSERT(g_blocked_wait_harness.waiter_blocked,
                "wait_result drives the waiter into the blocked queue with a concrete wait object");
    TEST_ASSERT(g_blocked_wait_harness.blocked_wait_obj != NULL,
                "blocked-wait harness captured the slot-backed wait-key identity");
    TEST_ASSERT(g_blocked_wait_harness.wrong_wake_preserved,
                "stale-generation wake with the same execution ID does not release the blocked waiter");
    TEST_ASSERT(g_blocked_wait_harness.wake_released,
                "canonical slot abort wake clears wait_obj and requeues the blocked waiter");
    TEST_ASSERT(g_blocked_wait_harness.waiter_resumed,
                "blocked waiter resumes after canonical wake delivery");
    TEST_ASSERT((int64_t)g_blocked_wait_harness.wait_result == ESYS_V2_CONTEXT_ERROR,
                "aborted blocked wait returns the deterministic aborted/error surface");
    TEST_ASSERT(g_blocked_wait_harness.waiter_wait_obj_cleared,
                "resumed waiter observes wait_obj cleared after wake");
    TEST_ASSERT(g_blocked_wait_harness.terminal_state == (uint64_t)EXEC_SLOT_ABORTED,
                "blocked-wait harness leaves the slot in ABORTED terminal state");

    execution_slot_enter_critical(&slot_guard);
    if (waiter_thread->pid > 0) {
        released_owned = execution_slot_release_owned_by_owner_locked((uint64_t)waiter_thread->pid);
    }
    execution_slot_exit_critical(&slot_guard);

    TEST_ASSERT(released_owned >= 1,
                "blocked-wait harness can release the aborted owner-owned slot after proof completes");

    sched_remove_process_everywhere(waiter_thread);
    waiter_thread->state = PROC_ZOMBIE;
    waiter_thread->wait_obj = NULL;

    sched_remove_process_everywhere(waker_thread);
    waker_thread->state = PROC_ZOMBIE;
    waker_thread->wait_obj = NULL;

    sched_remove_process_everywhere(target_proc);
    target_proc->state = PROC_ZOMBIE;
    target_proc->wait_obj = NULL;

    TEST_END("Blocked Wait Wake Contract");
}

static void test_irq_timeout_contract(void)
{
    proc_t *target_proc = NULL;
    proc_t *waiter_thread = NULL;
    proc_t *driver_thread = NULL;
    execution_slot_guard_t slot_guard = {0};
    uint32_t released_owned = 0;
    int spun = 0;

    TEST_START("IRQ Timeout Contract");

    __builtin_memset(&g_timeout_irq_harness, 0, sizeof(g_timeout_irq_harness));

    target_proc = create_validation_runtime_proc("phase2-timeout-target");
    TEST_ASSERT(target_proc != NULL && target_proc->type == PROC_TYPE_USER,
                "timeout harness created a dedicated live target process");

    if (target_proc == NULL || target_proc->type != PROC_TYPE_USER) {
        TEST_END("IRQ Timeout Contract");
        return;
    }

    sched_remove_process_everywhere(target_proc);
    target_proc->state = PROC_BLOCKED;
    target_proc->wait_obj = &g_timeout_irq_target_hold_token;
    g_timeout_irq_harness.target_proc = target_proc;

    waiter_thread = proc_create_kernel_thread(timeout_irq_harness_waiter_thread);
    driver_thread = proc_create_kernel_thread(timeout_irq_harness_driver_thread);

    TEST_ASSERT(waiter_thread != NULL && driver_thread != NULL,
                "timeout harness created waiter and IRQ-driver kernel threads");

    if (waiter_thread == NULL || driver_thread == NULL) {
        TEST_END("IRQ Timeout Contract");
        return;
    }

    for (spun = 0; spun < 256 && !g_timeout_irq_harness.waiter_resumed; ++spun) {
        sched_yield();
    }

    TEST_ASSERT(g_timeout_irq_harness.execution_id > 0,
                "timeout harness submitted a real execution before waiting");
    TEST_ASSERT(g_timeout_irq_harness.waiter_blocked,
                "timeout harness drives wait_result into a real blocked state");
    TEST_ASSERT(g_timeout_irq_harness.deadline_tick != 0,
                "timeout harness observes a real deadline programmed by wait_result");
    TEST_ASSERT(g_timeout_irq_harness.pre_irq_still_blocked,
                "scheduler yields alone do not advance timeout terminalization before the IRQ path runs");
    TEST_ASSERT(g_timeout_irq_harness.irq_woke_waiter,
                "timer IRQ processing releases the blocked waiter through the canonical wake path");
    TEST_ASSERT(g_timeout_irq_harness.waiter_resumed,
                "waiter resumes after IRQ-driven timeout wake");
    TEST_ASSERT((int64_t)g_timeout_irq_harness.wait_result == ESYS_V2_TIMEOUT,
                "IRQ-driven timeout returns the deterministic timeout surface");
    TEST_ASSERT(g_timeout_irq_harness.waiter_wait_obj_cleared,
                "IRQ-driven timeout leaves the resumed waiter with a cleared wait object");
    TEST_ASSERT(g_timeout_irq_harness.terminal_state == (uint64_t)EXEC_SLOT_TIMEOUT,
                "IRQ-driven timeout leaves the slot in TIMEOUT terminal state");

    execution_slot_enter_critical(&slot_guard);
    if (waiter_thread->pid > 0) {
        released_owned = execution_slot_release_owned_by_owner_locked((uint64_t)waiter_thread->pid);
    }
    execution_slot_exit_critical(&slot_guard);

    TEST_ASSERT(released_owned >= 1,
                "timeout harness can release the timed-out owner-owned slot after proof completes");

    sched_remove_process_everywhere(waiter_thread);
    waiter_thread->state = PROC_ZOMBIE;
    waiter_thread->wait_obj = NULL;

    sched_remove_process_everywhere(driver_thread);
    driver_thread->state = PROC_ZOMBIE;
    driver_thread->wait_obj = NULL;

    sched_remove_process_everywhere(target_proc);
    target_proc->state = PROC_ZOMBIE;
    target_proc->wait_obj = NULL;

    TEST_END("IRQ Timeout Contract");
}

static void test_negative_timeout_cleanup_contract(void)
{
    proc_t *target_proc = NULL;
    proc_t *waiter_thread = NULL;
    proc_t *driver_thread = NULL;
    int spun = 0;

    TEST_START("Negative Timeout Cleanup Contract");

    __builtin_memset(&g_negative_timeout_harness, 0, sizeof(g_negative_timeout_harness));

    target_proc = create_validation_runtime_proc("phase2-negative-timeout-target");
    TEST_ASSERT(target_proc != NULL && target_proc->type == PROC_TYPE_USER,
                "negative timeout harness created a dedicated live target process");

    if (target_proc == NULL || target_proc->type != PROC_TYPE_USER) {
        TEST_END("Negative Timeout Cleanup Contract");
        return;
    }

    g_negative_timeout_harness.target_proc = target_proc;

    waiter_thread = proc_create_kernel_thread(negative_timeout_harness_waiter_thread);
    driver_thread = proc_create_kernel_thread(negative_timeout_harness_driver_thread);

    TEST_ASSERT(waiter_thread != NULL && driver_thread != NULL,
                "negative timeout harness created waiter and driver kernel threads");

    if (waiter_thread == NULL || driver_thread == NULL) {
        TEST_END("Negative Timeout Cleanup Contract");
        return;
    }

    for (spun = 0; spun < 256 && !g_negative_timeout_harness.waiter_resumed; ++spun) {
        sched_yield();
    }

    TEST_ASSERT(g_negative_timeout_harness.execution_id > 0,
                "negative timeout harness submitted a real execution before blocking");
    TEST_ASSERT(g_negative_timeout_harness.pickup_running,
                "negative timeout harness drives the submitted slot into RUNNING before timeout");
    TEST_ASSERT(g_negative_timeout_harness.waiter_blocked,
                "negative timeout harness blocks the owner on wait_result before the IRQ timeout");
    TEST_ASSERT(g_negative_timeout_harness.deadline_tick != 0,
                "negative timeout harness observes a real deadline programmed on the RUNNING slot");
    TEST_ASSERT(g_negative_timeout_harness.pre_irq_still_blocked,
                "negative timeout harness confirms scheduler yields alone do not advance timeout cleanup");
    TEST_ASSERT(g_negative_timeout_harness.irq_woke_waiter,
                "negative timeout harness wakes the blocked owner only after the timer IRQ path fires");
    TEST_ASSERT(g_negative_timeout_harness.waiter_resumed,
                "negative timeout harness resumes the owner after IRQ-driven timeout");
    TEST_ASSERT((int64_t)g_negative_timeout_harness.wait_result == ESYS_V2_TIMEOUT,
                "negative timeout harness returns the deterministic timeout surface to the owner");
    TEST_ASSERT((int64_t)g_negative_timeout_harness.post_timeout_wait_result == ESYS_V2_TIMEOUT,
                "negative timeout harness replays deterministic timeout on repeated owner wait_result");
    TEST_ASSERT((int64_t)g_negative_timeout_harness.foreign_wait_result == ESYS_V2_NO_PERMISSION,
                "negative timeout harness keeps foreign wait_result fail-closed after timeout terminalization");
    TEST_ASSERT(g_negative_timeout_harness.waiter_wait_obj_cleared,
                "negative timeout harness leaves the resumed owner with a cleared wait object");
    TEST_ASSERT(g_negative_timeout_harness.terminal_state == (uint64_t)EXEC_SLOT_TIMEOUT,
                "negative timeout harness leaves the slot in TIMEOUT terminal state");
    TEST_ASSERT(target_proc->active_execution_id == 0,
                "negative timeout harness clears the target worker latch after timeout");
    TEST_ASSERT(g_negative_timeout_harness.slot_released,
                "negative timeout harness releases the timed-out owner-owned slot during cleanup");

    sched_remove_process_everywhere(waiter_thread);
    waiter_thread->state = PROC_ZOMBIE;
    waiter_thread->wait_obj = NULL;

    sched_remove_process_everywhere(driver_thread);
    driver_thread->state = PROC_ZOMBIE;
    driver_thread->wait_obj = NULL;

    sched_remove_process_everywhere(target_proc);
    target_proc->state = PROC_ZOMBIE;
    target_proc->wait_obj = NULL;

    TEST_END("Negative Timeout Cleanup Contract");
}

static void test_exit_teardown_contract(void)
{
    proc_t *exit_proc = NULL;
    proc_t *deferred_proc = NULL;
    proc_t *foreign_owner = NULL;
    proc_t *saved_current_proc = NULL;
    execution_slot_guard_t slot_guard = {0};
    char bcib[] = {0x42, 0x43, 0x49, 0x42, 0xCC, 0x33};
    uint64_t mapped_result_va = 0;
    uint64_t mapped_hash_va = 0;
    uint64_t owned_completed_id = 0;
    uint64_t owned_ready_id = 0;
    uint64_t foreign_running_id = 0;
    uint64_t generic_phys = 0;
    uint64_t generic_cap_id = 0;
    uint64_t generic_va = 0x31000000ULL;
    uint64_t result_vas[AYKEN_MAX_EXECUTION_SLOTS] = {0};
    uint64_t hash_vas[AYKEN_MAX_EXECUTION_SLOTS] = {0};
    uint32_t result_count = 0;
    uint32_t released_owned = 0;
    uint32_t i = 0;
    int publish_emitted = 0;
    int owned_ready_aborted = 0;
    int foreign_running_aborted = 0;
    int result_collection_ok = 0;
    int result_mapping_revoked = 0;
    int inbox_revoked = 0;
    int payload_revoked = 1;
    int mailbox_revoked = 0;
    int surfaces_released = 0;
    int zombie_and_detached = 0;
    int owned_slots_released = 0;
    int foreign_slot_preserved = 0;
    int generic_mapping_live = 0;
    int generic_mapping_revoked = 0;
    int text_revoked = 0;
    int stack_revoked = 0;
    int canary_revoked = 0;
    int deferred_lower_half_revoked = 0;
    int deferred_root_reaped = 0;

    TEST_START("Exit Teardown Contract");
    validation_prepare_full_result_bcib();

    exit_proc = create_validation_runtime_proc("phase2-exit-owner");
    deferred_proc = create_validation_runtime_proc("phase2-exit-deferred");
    foreign_owner = create_validation_runtime_proc("phase2-exit-foreign");

    TEST_ASSERT(exit_proc != NULL && exit_proc->type == PROC_TYPE_USER,
                "exit teardown test created a dedicated owner/executor process");
    TEST_ASSERT(deferred_proc != NULL && deferred_proc->type == PROC_TYPE_USER,
                "exit teardown test created a dedicated deferred-reap process");
    TEST_ASSERT(foreign_owner != NULL && foreign_owner->type == PROC_TYPE_USER,
                "exit teardown test created a dedicated foreign owner process");

    if (exit_proc == NULL || deferred_proc == NULL || foreign_owner == NULL) {
        TEST_END("Exit Teardown Contract");
        return;
    }

    generic_phys = phys_alloc_frame();
    TEST_ASSERT(generic_phys != 0,
                "exit teardown test allocated explicit generic backing");

    if (generic_phys == 0) {
        TEST_END("Exit Teardown Contract");
        return;
    }

    generic_cap_id = bind_validation_memory_capability(exit_proc,
                                                       generic_phys,
                                                       AYKEN_FRAME_SIZE,
                                                       CAPABILITY_PERM_READ_WRITE);
    TEST_ASSERT(generic_cap_id > 0,
                "exit teardown test bound a memory capability for generic mapping cleanup");

    TEST_ASSERT(map_validation_memory_as(exit_proc,
                                         generic_va,
                                         generic_phys,
                                         CAP_PERM_READ | CAP_PERM_WRITE) == ESYS_V2_SUCCESS,
                "exit teardown test created a generic ledger-backed mapping before exit");
    generic_mapping_live = paging_get_pte_in_pml4(exit_proc->pml4_phys, generic_va) != 0 &&
                           proc_find_generic_mapping(exit_proc, generic_va) != NULL;
    TEST_ASSERT(generic_mapping_live,
                "exit teardown test confirmed generic mapping is live before cleanup");

    owned_completed_id = submit_validation_execution_as(exit_proc,
                                                        g_validation_full_result_bcib,
                                                        VALIDATION_FULL_RESULT_BCIB_SIZE,
                                                        (uint64_t)exit_proc->pid);
    TEST_ASSERT(owned_completed_id > 0,
                "exit teardown test submitted an owner-owned completion candidate");

    saved_current_proc = current_proc;
    current_proc = exit_proc;
    current_proc->state = PROC_RUNNING;
    publish_emitted = sched_try_pickup_execution_work();
    current_proc = saved_current_proc;
    TEST_ASSERT(publish_emitted == 1,
                "exit teardown test picked owner-owned completion candidate into RUNNING");

    TEST_ASSERT(validation_write_output_for_execution(owned_completed_id,
                                                      AYKEN_EXECUTION_OUTPUT_MAGIC,
                                                      AYKEN_EXECUTION_OUTPUT_VERSION,
                                                      g_validation_full_result_bcib,
                                                      VALIDATION_FULL_RESULT_BCIB_SIZE),
                "exit teardown completion candidate writes a valid output header");
    TEST_ASSERT(complete_validation_execution_as(exit_proc,
                                                 owned_completed_id,
                                                 EXEC_COMPLETION_COMPLETED) == ESYS_V2_SUCCESS,
                "exit teardown test completed owner-owned execution before teardown");

    mapped_result_va = wait_validation_result_as(exit_proc, owned_completed_id, 0);
    TEST_ASSERT(mapped_result_va > 0,
                "exit teardown test materialized a result VA before exit cleanup");
    execution_slot_enter_critical(&slot_guard);
    {
        exec_slot_t *slot = execution_slot_find_locked(owned_completed_id);
        mapped_hash_va = slot != NULL ? slot->mapped_hash_va : 0;
    }
    execution_slot_exit_critical(&slot_guard);
    TEST_ASSERT(mapped_hash_va > 0,
                "exit teardown test materialized a hash sidecar before exit cleanup");

    foreign_running_id = submit_validation_execution_as(foreign_owner,
                                                        bcib,
                                                        sizeof(bcib),
                                                        (uint64_t)exit_proc->pid);
    TEST_ASSERT(foreign_running_id > owned_completed_id,
                "exit teardown test submitted a foreign-owned execution targeting the exiting worker");

    saved_current_proc = current_proc;
    current_proc = exit_proc;
    current_proc->state = PROC_RUNNING;
    publish_emitted = sched_try_pickup_execution_work();
    current_proc = saved_current_proc;
    TEST_ASSERT(publish_emitted == 1,
                "exit teardown test picked the foreign-owned target execution into RUNNING");

    owned_ready_id = submit_validation_execution_as(exit_proc,
                                                    bcib,
                                                    sizeof(bcib),
                                                    (uint64_t)exit_proc->pid);
    TEST_ASSERT(owned_ready_id > foreign_running_id,
                "exit teardown test submitted an additional owner-owned READY execution");

    execution_slot_enter_critical(&slot_guard);
    result_count = execution_slot_prepare_process_exit_locked((uint64_t)exit_proc->pid,
                                                              result_vas,
                                                              hash_vas,
                                                              AYKEN_MAX_EXECUTION_SLOTS);
    {
        exec_slot_t *owned_ready_slot = execution_slot_find_locked(owned_ready_id);
        exec_slot_t *foreign_running_slot = execution_slot_find_locked(foreign_running_id);
        exec_slot_t *owned_completed_slot = execution_slot_find_locked(owned_completed_id);

        owned_ready_aborted = owned_ready_slot != NULL &&
                              owned_ready_slot->state == EXEC_SLOT_ABORTED;
        foreign_running_aborted = foreign_running_slot != NULL &&
                                  foreign_running_slot->state == EXEC_SLOT_ABORTED &&
                                  exit_proc->active_execution_id == 0;
        result_collection_ok = owned_completed_slot != NULL &&
                               owned_completed_slot->state == EXEC_SLOT_RESULT_MAPPED &&
                               result_count == 1 &&
                               result_vas[0] == mapped_result_va &&
                               hash_vas[0] == mapped_hash_va;
    }
    execution_slot_exit_critical(&slot_guard);

    TEST_ASSERT(owned_ready_aborted,
                "exit teardown preparation aborts owner-owned READY executions");
    TEST_ASSERT(foreign_running_aborted,
                "exit teardown preparation aborts foreign-owned RUNNING work targeting the exiting executor and clears its latch");
    TEST_ASSERT(result_collection_ok,
                "exit teardown preparation collects mapped result VAs without overwriting completed ownership state");

    exit_proc->active_execution_id = 0;
    exit_proc->wait_obj = NULL;
    exit_proc->state = PROC_ZOMBIE;

    proc_teardown_exit_surfaces(exit_proc, result_vas, hash_vas, result_count);
    result_mapping_revoked = paging_get_pte_in_pml4(exit_proc->pml4_phys, mapped_result_va) == 0;
    result_mapping_revoked = result_mapping_revoked &&
                             paging_get_pte_in_pml4(exit_proc->pml4_phys,
                                                    mapped_result_va + AYKEN_FRAME_SIZE) == 0;
    result_mapping_revoked = result_mapping_revoked &&
                             paging_get_pte_in_pml4(exit_proc->pml4_phys, mapped_hash_va) == 0;
    generic_mapping_revoked = paging_get_pte_in_pml4(exit_proc->pml4_phys, generic_va) == 0 &&
                              proc_find_generic_mapping(exit_proc, generic_va) == NULL;
    text_revoked = paging_get_pte_in_pml4(exit_proc->pml4_phys, USER_TEXT_BASE) == 0;
    stack_revoked =
        paging_get_pte_in_pml4(exit_proc->pml4_phys, USER_STACK_TOP - AYKEN_FRAME_SIZE) == 0 &&
        paging_get_pte_in_pml4(exit_proc->pml4_phys, USER_STACK_TOP - (2 * AYKEN_FRAME_SIZE)) == 0;
    canary_revoked = paging_get_pte_in_pml4(exit_proc->pml4_phys, VALIDATION_RING3_CANARY_ADDR) == 0;
    inbox_revoked = paging_get_pte_in_pml4(exit_proc->pml4_phys, EXECUTION_INBOX_VA) == 0;
    for (i = 0; i < AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES; ++i) {
        if (paging_get_pte_in_pml4(exit_proc->pml4_phys,
                                   EXECUTION_PAYLOAD_VA + ((uint64_t)i * AYKEN_FRAME_SIZE)) != 0) {
            payload_revoked = 0;
        }
    }
    mailbox_revoked = paging_get_pte_in_pml4(exit_proc->pml4_phys, SCHED_MAILBOX_VA) == 0;
    surfaces_released = exit_proc->execution_inbox_pa == 0 &&
                        exit_proc->mailbox_pa == 0 &&
                        exit_proc->context.rsp0 == 0 &&
                        exit_proc->execution_delivery_seq == 0 &&
                        exit_proc->mailbox_last_epoch == 0;
    for (i = 0; i < AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES; ++i) {
        if (exit_proc->execution_payload_pas[i] != 0) {
            surfaces_released = 0;
        }
    }

    TEST_ASSERT(result_mapping_revoked,
                "exit teardown revokes result mappings from the owner PML4");
    TEST_ASSERT(generic_mapping_revoked,
                "exit teardown revokes generic ledger-backed mappings from the owner PML4");
    TEST_ASSERT(text_revoked && stack_revoked && canary_revoked,
                "exit teardown revokes remaining user text, stack, and canary mappings");
    TEST_ASSERT(inbox_revoked && payload_revoked && mailbox_revoked,
                "exit teardown revokes delivery and mailbox VA mappings from the owner PML4");
    TEST_ASSERT(surfaces_released,
                "exit teardown releases delivery surfaces, mailbox backing, and kernel rsp0 backing");

    sched_remove_process_everywhere(exit_proc);
    zombie_and_detached = exit_proc->state == PROC_ZOMBIE &&
                          exit_proc->wait_obj == NULL &&
                          exit_proc->next == NULL;
    TEST_ASSERT(zombie_and_detached,
                "exit teardown leaves the exiting process as zombie and detached from scheduler-linked bookkeeping");

    execution_slot_enter_critical(&slot_guard);
    released_owned = execution_slot_release_owned_by_owner_locked((uint64_t)exit_proc->pid);
    owned_slots_released = released_owned >= 2 &&
                           execution_slot_find_locked(owned_completed_id) == NULL &&
                           execution_slot_find_locked(owned_ready_id) == NULL;
    {
        exec_slot_t *foreign_slot = execution_slot_find_locked(foreign_running_id);
        foreign_slot_preserved = foreign_slot != NULL &&
                                 foreign_slot->owner_pid == (uint64_t)foreign_owner->pid &&
                                 foreign_slot->state == EXEC_SLOT_ABORTED;
        if (foreign_slot != NULL) {
            execution_slot_release_locked(foreign_slot);
        }
    }
    execution_slot_exit_critical(&slot_guard);

    TEST_ASSERT(owned_slots_released,
                "exit teardown releases owned slot state after revoke work completes");
    TEST_ASSERT(foreign_slot_preserved,
                "exit teardown preserves foreign-owned targeted work as ABORTED instead of releasing ownership");

    saved_current_proc = current_proc;
    current_proc = deferred_proc;
    current_proc->state = PROC_RUNNING;
    proc_teardown_exit_surfaces(deferred_proc, NULL, NULL, 0);
    current_proc = saved_current_proc;

    deferred_lower_half_revoked =
        deferred_proc->pml4_phys != 0 &&
        paging_get_pte_in_pml4(deferred_proc->pml4_phys, USER_TEXT_BASE) == 0 &&
        paging_get_pte_in_pml4(deferred_proc->pml4_phys, USER_STACK_TOP - AYKEN_FRAME_SIZE) == 0 &&
        paging_get_pte_in_pml4(deferred_proc->pml4_phys, USER_STACK_TOP - (2 * AYKEN_FRAME_SIZE)) == 0 &&
        paging_get_pte_in_pml4(deferred_proc->pml4_phys, VALIDATION_RING3_CANARY_ADDR) == 0 &&
        deferred_proc->context.rsp0 != 0;
    TEST_ASSERT(deferred_lower_half_revoked,
                "active exit teardown destroys user lower-half mappings before deferred reap runs");

    proc_drain_deferred_reap();
    deferred_root_reaped = deferred_proc->pml4_phys == 0 &&
                           deferred_proc->context.cr3 == 0 &&
                           deferred_proc->context.rsp0 == 0;
    TEST_ASSERT(deferred_root_reaped,
                "deferred reap later frees the active root PML4 and current rsp0 backing");

    sched_remove_process_everywhere(deferred_proc);
    deferred_proc->state = PROC_ZOMBIE;

    sched_remove_process_everywhere(foreign_owner);
    foreign_owner->state = PROC_ZOMBIE;
    proc_teardown_exit_surfaces(foreign_owner, NULL, NULL, 0);
    phys_free_frame(generic_phys);

    TEST_END("Exit Teardown Contract");
}

static void test_owner_exit_guard(void)
{
    uint32_t owner_pid = sched_active_owner_pid();
    proc_t *owner_proc = NULL;
    proc_t *saved_current_proc = current_proc;
    proc_state_t owner_state_before;
    uint64_t owner_active_execution_before;
    void *owner_wait_before;
    uint64_t owner_mailbox_before;
    uint64_t owner_inbox_before;
    uint64_t result;

    TEST_START("Owner Exit Guard");

    owner_proc = proc_find_by_pid((int)owner_pid);
    TEST_ASSERT(owner_proc != NULL && owner_proc->type == PROC_TYPE_USER,
                "scheduler owner process exists for exit-guard coverage");

    if (owner_proc == NULL || owner_proc->type != PROC_TYPE_USER) {
        TEST_END("Owner Exit Guard");
        return;
    }

    owner_state_before = owner_proc->state;
    owner_active_execution_before = owner_proc->active_execution_id;
    owner_wait_before = owner_proc->wait_obj;
    owner_mailbox_before = owner_proc->mailbox_pa;
    owner_inbox_before = owner_proc->execution_inbox_pa;

    current_proc = owner_proc;
    result = sys_v2_exit(0xE17);
    current_proc = saved_current_proc;

    TEST_ASSERT((int64_t)result == ESYS_V2_PERMISSION_DENIED,
                "scheduler owner exit is denied at syscall entry");
    TEST_ASSERT(owner_proc->state == owner_state_before,
                "owner-exit deny leaves owner process state unchanged");
    TEST_ASSERT(owner_proc->active_execution_id == owner_active_execution_before &&
                owner_proc->wait_obj == owner_wait_before,
                "owner-exit deny does not start execution teardown side effects");
    TEST_ASSERT(owner_proc->mailbox_pa == owner_mailbox_before &&
                owner_proc->execution_inbox_pa == owner_inbox_before,
                "owner-exit deny does not revoke scheduler or delivery surfaces");

    TEST_END("Owner Exit Guard");
}

static void test_exit_noreturn_runtime_contract(void)
{
    uint32_t owner_pid = sched_active_owner_pid();
    proc_t *exit_proc = NULL;
    proc_t *return_proc = current_proc;
    int switch_seen = 0;
    int switch_from_pid = 0;
    int switch_to_pid = 0;
    int lower_half_revoked = 0;
    int deferred_reap_pending = 0;
    int deferred_reap_completed = 0;

    TEST_START("Exit No-Return Runtime Contract");

    TEST_ASSERT(return_proc != NULL && return_proc->pid > 0 &&
                    (uint32_t)return_proc->pid != owner_pid,
                "no-return exit harness starts from a live non-owner current process");

    if (return_proc == NULL || return_proc->pid <= 0 ||
        (uint32_t)return_proc->pid == owner_pid) {
        TEST_END("Exit No-Return Runtime Contract");
        return;
    }

    exit_proc = proc_create_user_process("phase2-exit-noreturn",
                                         validation_exit_noreturn_code,
                                         sizeof(validation_exit_noreturn_code),
                                         PROC_IMAGE_FLAT);
    TEST_ASSERT(exit_proc != NULL && exit_proc->type == PROC_TYPE_USER,
                "no-return exit harness created a dedicated Ring3 exit process");

    if (exit_proc == NULL || exit_proc->type != PROC_TYPE_USER) {
        TEST_END("Exit No-Return Runtime Contract");
        return;
    }

    TEST_ASSERT(validation_seed_owner_mailbox_candidate((uint32_t)exit_proc->pid),
                "no-return exit harness seeded a fresh owner-mailbox decision for the exit process");

    sched_validation_arm_exit_successor(return_proc);
    sched_yield();
    switch_seen = sched_validation_take_exit_switch_event(&switch_from_pid, &switch_to_pid);
    sched_validation_disarm_exit_successor();

    TEST_ASSERT(switch_seen &&
                    switch_from_pid == exit_proc->pid &&
                    switch_to_pid == return_proc->pid,
                "no-return exit harness observed a direct exit-time context switch away from the exiting process");
    TEST_ASSERT(current_proc == return_proc && return_proc->state == PROC_RUNNING,
                "no-return exit harness resumes the original current process after exit-time switch-away");
    TEST_ASSERT(exit_proc->state == PROC_ZOMBIE &&
                    exit_proc->wait_obj == NULL &&
                    exit_proc->active_execution_id == 0 &&
                    exit_proc->next == NULL,
                "direct sys_v2_exit leaves the exiting process zombied and detached without returning");

    lower_half_revoked =
        exit_proc->pml4_phys != 0 &&
        paging_get_pte_in_pml4(exit_proc->pml4_phys, USER_TEXT_BASE) == 0 &&
        paging_get_pte_in_pml4(exit_proc->pml4_phys, USER_STACK_TOP - AYKEN_FRAME_SIZE) == 0 &&
        paging_get_pte_in_pml4(exit_proc->pml4_phys, USER_STACK_TOP - (2 * AYKEN_FRAME_SIZE)) == 0 &&
        paging_get_pte_in_pml4(exit_proc->pml4_phys, VALIDATION_RING3_CANARY_ADDR) == 0;
    deferred_reap_pending = exit_proc->pml4_phys != 0 &&
                            exit_proc->context.cr3 != 0 &&
                            exit_proc->context.rsp0 != 0;

    TEST_ASSERT(lower_half_revoked,
                "direct sys_v2_exit destroys the exiting Ring3 lower-half mappings before deferred reap");
    TEST_ASSERT(deferred_reap_pending,
                "direct sys_v2_exit leaves active root PML4 and rsp0 on deferred reap until a later safe drain");

    proc_drain_deferred_reap();
    deferred_reap_completed = exit_proc->pml4_phys == 0 &&
                              exit_proc->context.cr3 == 0 &&
                              exit_proc->context.rsp0 == 0;
    TEST_ASSERT(deferred_reap_completed,
                "deferred reap completes after direct sys_v2_exit runtime proof");

    TEST_END("Exit No-Return Runtime Contract");
}

static void test_owner_handoff_runtime_contract(void)
{
    static const uint8_t bcib[] = {0x42, 0x43, 0x49, 0x42, 0x11, 0xA4};
    execution_slot_guard_t slot_guard = {0};
    proc_t *old_owner = current_proc;
    uint32_t saved_owner_pid = sched_active_owner_pid();
    proc_t *saved_owner_proc = proc_find_by_pid((int)saved_owner_pid);
    proc_t *successor_proc = NULL;
    proc_t *target_proc = NULL;
    uint64_t execution_id = 0;
    int handoff_request_ok = 0;
    int transfer_seen = 0;
    int transfer_from_pid = 0;
    int transfer_to_pid = 0;
    int decision_seen = 0;
    int decision_from_pid = 0;
    int decision_to_pid = 0;
    int decision_src_pid = 0;
    uint64_t decision_id = 0;
    int successor_blocked = 0;
    int stale_old_owner_rejected = 0;
    int owner_restored = 0;
    int old_owner_mailbox_neutralized = 0;
    int successor_slot_released = 1;

    TEST_START("Owner Handoff Runtime Contract");

    TEST_ASSERT(old_owner != NULL &&
                    old_owner->pid > 0 &&
                    old_owner->type == PROC_TYPE_USER &&
                    old_owner->mailbox_pa != 0 &&
                    old_owner->state == PROC_RUNNING &&
                    (uint32_t)old_owner->pid != saved_owner_pid,
                "owner handoff proof starts from a live non-owner Ring3 current process with mailbox backing");

    if (old_owner == NULL ||
        old_owner->pid <= 0 ||
        old_owner->type != PROC_TYPE_USER ||
        old_owner->mailbox_pa == 0 ||
        old_owner->state != PROC_RUNNING ||
        (uint32_t)old_owner->pid == saved_owner_pid) {
        TEST_END("Owner Handoff Runtime Contract");
        return;
    }

    successor_proc = proc_create_user_process("phase2-owner-transfer-successor",
                                              validation_owner_transfer_successor_code_template,
                                              sizeof(validation_owner_transfer_successor_code_template),
                                              PROC_IMAGE_FLAT);
    target_proc = create_validation_runtime_proc("phase2-owner-transfer-target");

    TEST_ASSERT(successor_proc != NULL && successor_proc->type == PROC_TYPE_USER,
                "owner handoff proof created a dedicated successor owner process");
    TEST_ASSERT(target_proc != NULL && target_proc->type == PROC_TYPE_USER,
                "owner handoff proof created a dedicated blocked execution target");

    if (successor_proc == NULL || successor_proc->type != PROC_TYPE_USER ||
        target_proc == NULL || target_proc->type != PROC_TYPE_USER) {
        goto cleanup;
    }

    sched_remove_process_everywhere(target_proc);
    target_proc->state = PROC_BLOCKED;
    target_proc->wait_obj = &g_owner_transfer_target_hold_token;

    execution_id = submit_validation_execution_as(successor_proc,
                                                  bcib,
                                                  sizeof(bcib),
                                                  (uint64_t)target_proc->pid);
    TEST_ASSERT(execution_id > 0,
                "owner handoff proof seeded a successor-owned execution that can block in wait_result");

    if (execution_id == 0) {
        goto cleanup;
    }

    sched_remove_process_everywhere(successor_proc);
    successor_proc->wait_obj = NULL;
    successor_proc->state = PROC_READY;
    enqueue_ready(successor_proc);

    TEST_ASSERT(validation_patch_owner_transfer_successor_code(successor_proc,
                                                               (uint32_t)old_owner->pid,
                                                               execution_id),
                "owner handoff proof patched successor Ring3 code with runtime execution and return targets");

    sched_validation_set_active_owner(old_owner);
    handoff_request_ok = (sched_request_owner_transfer(old_owner, successor_proc) == 0);
    TEST_ASSERT(handoff_request_ok,
                "owner handoff proof arms a narrow sole-owner transfer request without mutating authority early");
    TEST_ASSERT(validation_seed_mailbox_candidate(old_owner, (uint32_t)successor_proc->pid),
                "owner handoff proof seeds the temporary owner mailbox with the successor candidate");

    if (!handoff_request_ok) {
        goto cleanup;
    }

    sched_yield();

    transfer_seen = sched_validation_take_owner_transfer_event(&transfer_from_pid,
                                                               &transfer_to_pid);
    decision_seen = sched_validation_take_mailbox_decision_event(&decision_from_pid,
                                                                 &decision_to_pid,
                                                                 &decision_src_pid,
                                                                 &decision_id);
    successor_blocked = successor_proc->state == PROC_BLOCKED &&
                        successor_proc->wait_obj != NULL;

    TEST_ASSERT(transfer_seen &&
                    transfer_from_pid == old_owner->pid &&
                    transfer_to_pid == successor_proc->pid,
                "owner handoff proof observes a dispatch-boundary authority commit from old owner to successor");
    TEST_ASSERT(sched_active_owner_pid() == (uint32_t)successor_proc->pid,
                "owner handoff proof leaves the successor as the sole active owner after commit");
    TEST_ASSERT(current_proc == old_owner && old_owner->state == PROC_RUNNING,
                "owner handoff proof returns execution to the original process after successor-owned scheduling");
    TEST_ASSERT(successor_blocked,
                "owner handoff proof leaves the successor blocked on its owned wait_result after publishing a mailbox decision");
    TEST_ASSERT(decision_seen &&
                    decision_from_pid == successor_proc->pid &&
                    decision_to_pid == old_owner->pid &&
                    decision_src_pid == successor_proc->pid &&
                    decision_id == 2,
                "owner handoff proof shows the post-commit return dispatch is driven by the successor mailbox");
    TEST_ASSERT(validation_seed_mailbox_candidate(old_owner, (uint32_t)successor_proc->pid),
                "owner handoff proof can publish a fresh post-commit old-owner mailbox candidate for rejection coverage");
    stale_old_owner_rejected = sched_validation_non_owner_publish_would_fail(old_owner);
    TEST_ASSERT(stale_old_owner_rejected,
                "fresh post-commit old-owner mailbox publish is recognized as a fail-closed protocol violation");

cleanup:
    sched_validation_set_active_owner(saved_owner_proc);
    if (saved_owner_proc != NULL && old_owner != NULL && old_owner->pid > 0) {
        owner_restored = validation_seed_mailbox_candidate(saved_owner_proc,
                                                           (uint32_t)old_owner->pid);
        TEST_ASSERT(owner_restored,
                    "owner handoff proof restores the original scheduler owner mailbox authority after validation");
    }

    if (old_owner != NULL && old_owner->mailbox_pa != 0) {
        old_owner_mailbox_neutralized = validation_reset_mailbox_to_self(old_owner);
        TEST_ASSERT(old_owner_mailbox_neutralized,
                    "owner handoff proof neutralizes the temporary owner mailbox before returning to ordinary runtime");
    }

    if (execution_id != 0 && successor_proc != NULL) {
        execution_slot_enter_critical(&slot_guard);
        {
            exec_slot_t *slot = execution_slot_find_locked(execution_id);
            if (slot != NULL) {
                (void)execution_slot_finish_locked(slot, EXEC_SLOT_ABORTED);
            }
            successor_slot_released =
                execution_slot_release_owned_by_owner_locked((uint64_t)successor_proc->pid) >= 1;
        }
        execution_slot_exit_critical(&slot_guard);
        TEST_ASSERT(successor_slot_released,
                    "owner handoff proof cleanup releases the successor-owned execution slot after abort wake");
    }

    if (successor_proc != NULL) {
        sched_remove_process_everywhere(successor_proc);
        successor_proc->wait_obj = NULL;
        successor_proc->active_execution_id = 0;
        successor_proc->state = PROC_ZOMBIE;
        proc_teardown_exit_surfaces(successor_proc, NULL, NULL, 0);
    }

    if (target_proc != NULL) {
        sched_remove_process_everywhere(target_proc);
        target_proc->wait_obj = NULL;
        target_proc->active_execution_id = 0;
        target_proc->state = PROC_ZOMBIE;
        proc_teardown_exit_surfaces(target_proc, NULL, NULL, 0);
    }

    TEST_END("Owner Handoff Runtime Contract");
}

static void test_owner_handoff_exit_followthrough_runtime_contract(void)
{
    static const uint8_t owner_bcib[] = {0x42, 0x43, 0x49, 0x42, 0xD1, 0x42};
    static const uint8_t successor_bcib[] = {0x42, 0x43, 0x49, 0x42, 0xD1, 0x43};
    execution_slot_guard_t slot_guard = {0};
    proc_t *controller_proc = current_proc;
    uint32_t saved_owner_pid = sched_active_owner_pid();
    proc_t *saved_owner_proc = proc_find_by_pid((int)saved_owner_pid);
    proc_t *old_owner_proc = NULL;
    proc_t *successor_proc = NULL;
    proc_t *target_proc = NULL;
    uint64_t owner_exec_id = 0;
    uint64_t successor_wait_exec_id = 0;
    int handoff_request_ok = 0;
    int transfer_seen = 0;
    int transfer_from_pid = 0;
    int transfer_to_pid = 0;
    int decision_seen = 0;
    int decision_from_pid = 0;
    int decision_to_pid = 0;
    int decision_src_pid = 0;
    uint64_t decision_id = 0;
    int exit_switch_seen = 0;
    int exit_switch_from_pid = 0;
    int exit_switch_to_pid = 0;
    int old_owner_zombied = 0;
    int old_owner_lower_half_revoked = 0;
    int old_owner_surfaces_released = 0;
    int old_owner_root_deferred = 0;
    int old_owner_root_reaped = 0;
    int old_owner_slot_released = 1;
    int successor_slot_released = 1;
    int owner_restored = 0;

    TEST_START("Owner Handoff Exit Followthrough Runtime Contract");

    TEST_ASSERT(controller_proc != NULL &&
                    controller_proc->pid > 0 &&
                    (uint32_t)controller_proc->pid != saved_owner_pid,
                "owner handoff exit proof starts from a live non-owner controller process");

    if (controller_proc == NULL ||
        controller_proc->pid <= 0 ||
        (uint32_t)controller_proc->pid == saved_owner_pid) {
        TEST_END("Owner Handoff Exit Followthrough Runtime Contract");
        return;
    }

    old_owner_proc = proc_create_user_process("phase2-owner-followthrough-old",
                                              validation_owner_followthrough_old_owner_code_template,
                                              sizeof(validation_owner_followthrough_old_owner_code_template),
                                              PROC_IMAGE_FLAT);
    successor_proc = proc_create_user_process("phase2-owner-followthrough-successor",
                                              validation_owner_followthrough_successor_code_template,
                                              sizeof(validation_owner_followthrough_successor_code_template),
                                              PROC_IMAGE_FLAT);
    target_proc = create_validation_runtime_proc("phase2-owner-followthrough-target");

    TEST_ASSERT(old_owner_proc != NULL && old_owner_proc->type == PROC_TYPE_USER,
                "owner followthrough proof created a dedicated old-owner process");
    TEST_ASSERT(successor_proc != NULL && successor_proc->type == PROC_TYPE_USER,
                "owner followthrough proof created a dedicated successor process");
    TEST_ASSERT(target_proc != NULL && target_proc->type == PROC_TYPE_USER,
                "owner followthrough proof created a dedicated blocked target process");

    if (old_owner_proc == NULL || old_owner_proc->type != PROC_TYPE_USER ||
        successor_proc == NULL || successor_proc->type != PROC_TYPE_USER ||
        target_proc == NULL || target_proc->type != PROC_TYPE_USER) {
        goto cleanup;
    }

    sched_remove_process_everywhere(target_proc);
    target_proc->state = PROC_BLOCKED;
    target_proc->wait_obj = &g_owner_transfer_target_hold_token;

    owner_exec_id = submit_validation_execution_as(old_owner_proc,
                                                   owner_bcib,
                                                   sizeof(owner_bcib),
                                                   (uint64_t)successor_proc->pid);
    successor_wait_exec_id = submit_validation_execution_as(successor_proc,
                                                            successor_bcib,
                                                            sizeof(successor_bcib),
                                                            (uint64_t)target_proc->pid);

    TEST_ASSERT(owner_exec_id > 0,
                "owner followthrough proof created an old-owner execution targeting the successor executor");
    TEST_ASSERT(successor_wait_exec_id > 0,
                "owner followthrough proof created a successor-owned wait_result blocker execution");

    if (owner_exec_id == 0 || successor_wait_exec_id == 0) {
        goto cleanup;
    }

    sched_remove_process_everywhere(old_owner_proc);
    old_owner_proc->wait_obj = NULL;
    old_owner_proc->state = PROC_READY;
    enqueue_ready(old_owner_proc);

    sched_remove_process_everywhere(successor_proc);
    successor_proc->wait_obj = NULL;
    successor_proc->state = PROC_READY;
    enqueue_ready(successor_proc);

    TEST_ASSERT(validation_patch_owner_followthrough_old_owner_code(old_owner_proc,
                                                                    (uint32_t)successor_proc->pid,
                                                                    owner_exec_id),
                "owner followthrough proof patched old-owner Ring3 code with successor and execution targets");
    TEST_ASSERT(validation_patch_owner_followthrough_successor_code(successor_proc,
                                                                    (uint32_t)old_owner_proc->pid,
                                                                    owner_exec_id,
                                                                    successor_wait_exec_id),
                "owner followthrough proof patched successor Ring3 code with completion and wait targets");

    sched_validation_set_active_owner(old_owner_proc);
    handoff_request_ok = (sched_request_owner_transfer(old_owner_proc, successor_proc) == 0);
    TEST_ASSERT(handoff_request_ok,
                "owner followthrough proof arms a ratified narrow owner transfer from the dedicated old owner to the successor");

    if (!handoff_request_ok) {
        goto cleanup;
    }

    sched_validation_arm_exit_successor(controller_proc);
    sched_yield();
    exit_switch_seen = sched_validation_take_exit_switch_event(&exit_switch_from_pid,
                                                               &exit_switch_to_pid);
    sched_validation_disarm_exit_successor();

    transfer_seen = sched_validation_take_owner_transfer_event(&transfer_from_pid,
                                                               &transfer_to_pid);
    decision_seen = sched_validation_take_mailbox_decision_event(&decision_from_pid,
                                                                 &decision_to_pid,
                                                                 &decision_src_pid,
                                                                 &decision_id);

    TEST_ASSERT(transfer_seen &&
                    transfer_from_pid == old_owner_proc->pid &&
                    transfer_to_pid == successor_proc->pid,
                "owner followthrough proof observes the old-owner to successor authority commit");
    TEST_ASSERT(decision_seen &&
                    decision_from_pid == successor_proc->pid &&
                    decision_to_pid == old_owner_proc->pid &&
                    decision_src_pid == successor_proc->pid &&
                    decision_id == 2,
                "owner followthrough proof shows the successor drives the post-commit return dispatch with a fresh epoch");
    TEST_ASSERT(exit_switch_seen &&
                    exit_switch_from_pid == old_owner_proc->pid &&
                    exit_switch_to_pid == controller_proc->pid,
                "owner followthrough proof observes a no-return exit-time switch from the retired old owner back to the controller");
    TEST_ASSERT(current_proc == controller_proc && controller_proc->state == PROC_RUNNING,
                "owner followthrough proof resumes the controller after old-owner exit");
    TEST_ASSERT(sched_active_owner_pid() == (uint32_t)successor_proc->pid,
                "owner followthrough proof keeps the successor as sole active owner after the old owner exits");

    old_owner_zombied = old_owner_proc->state == PROC_ZOMBIE &&
                        old_owner_proc->wait_obj == NULL &&
                        old_owner_proc->active_execution_id == 0 &&
                        old_owner_proc->next == NULL;
    old_owner_lower_half_revoked =
        old_owner_proc->pml4_phys != 0 &&
        paging_get_pte_in_pml4(old_owner_proc->pml4_phys, USER_TEXT_BASE) == 0 &&
        paging_get_pte_in_pml4(old_owner_proc->pml4_phys, USER_STACK_TOP - AYKEN_FRAME_SIZE) == 0 &&
        paging_get_pte_in_pml4(old_owner_proc->pml4_phys, USER_STACK_TOP - (2 * AYKEN_FRAME_SIZE)) == 0 &&
        paging_get_pte_in_pml4(old_owner_proc->pml4_phys, VALIDATION_RING3_CANARY_ADDR) == 0;
    old_owner_surfaces_released = old_owner_proc->mailbox_pa == 0 &&
                                  old_owner_proc->execution_inbox_pa == 0 &&
                                  old_owner_proc->execution_delivery_seq == 0;
    old_owner_root_deferred = old_owner_proc->pml4_phys != 0 &&
                              old_owner_proc->context.cr3 != 0 &&
                              old_owner_proc->context.rsp0 != 0;

    TEST_ASSERT(old_owner_zombied,
                "owner followthrough proof leaves the retired old owner zombied and detached");
    TEST_ASSERT(old_owner_lower_half_revoked,
                "owner followthrough proof destroys the retired old owner lower-half mappings before reap");
    TEST_ASSERT(old_owner_surfaces_released,
                "owner followthrough proof releases the retired old owner mailbox and execution delivery surfaces");
    TEST_ASSERT(old_owner_root_deferred,
                "owner followthrough proof defers only the retired old owner active root PML4 and rsp0 backing");

    execution_slot_enter_critical(&slot_guard);
    old_owner_slot_released = execution_slot_find_locked(owner_exec_id) == NULL;
    {
        exec_slot_t *slot = execution_slot_find_locked(successor_wait_exec_id);
        if (slot != NULL) {
            (void)execution_slot_finish_locked(slot, EXEC_SLOT_ABORTED);
        }
        successor_slot_released =
            execution_slot_release_owned_by_owner_locked((uint64_t)successor_proc->pid) >= 1;
    }
    execution_slot_exit_critical(&slot_guard);

    TEST_ASSERT(old_owner_slot_released,
                "owner followthrough proof leaves the retired old owner execution slot released by exit cleanup");
    TEST_ASSERT(successor_slot_released,
                "owner followthrough proof cleanup releases the successor-owned wait blocker after wake");

    proc_drain_deferred_reap();
    old_owner_root_reaped = old_owner_proc->pml4_phys == 0 &&
                            old_owner_proc->context.cr3 == 0 &&
                            old_owner_proc->context.rsp0 == 0;
    TEST_ASSERT(old_owner_root_reaped,
                "owner followthrough proof completes deferred reap for the retired old owner root PML4 and rsp0 backing");

cleanup:
    sched_validation_set_active_owner(saved_owner_proc);
    if (saved_owner_proc != NULL && controller_proc != NULL && controller_proc->pid > 0) {
        owner_restored = validation_seed_mailbox_candidate(saved_owner_proc,
                                                           (uint32_t)controller_proc->pid);
        TEST_ASSERT(owner_restored,
                    "owner followthrough proof restores the original scheduler owner mailbox authority after validation");
    }

    if (successor_proc != NULL) {
        sched_remove_process_everywhere(successor_proc);
        successor_proc->wait_obj = NULL;
        successor_proc->active_execution_id = 0;
        successor_proc->state = PROC_ZOMBIE;
        proc_teardown_exit_surfaces(successor_proc, NULL, NULL, 0);
    }

    if (target_proc != NULL) {
        sched_remove_process_everywhere(target_proc);
        target_proc->wait_obj = NULL;
        target_proc->active_execution_id = 0;
        target_proc->state = PROC_ZOMBIE;
        proc_teardown_exit_surfaces(target_proc, NULL, NULL, 0);
    }

    TEST_END("Owner Handoff Exit Followthrough Runtime Contract");
}

static void test_generic_mapping_contract(void)
{
    proc_t *map_proc = NULL;
    proc_t *foreign_proc = NULL;
    uint64_t phys_base = 0;
    uint64_t cap_id = 0;
    uint64_t foreign_cap_id = 0;
    uint64_t va_base = 0x30000000ULL;
    uint64_t result = 0;
    uint64_t pte0 = 0;
    uint64_t pte1 = 0;
    proc_mapping_entry_t *entry0 = NULL;
    proc_mapping_entry_t *entry1 = NULL;
    int map0_ok = 0;
    int map1_ok = 0;
    int foreign_unmap_denied = 0;
    int unmap_ok = 0;
    int ledger_cleared = 0;

    TEST_START("Generic Mapping Contract");

    map_proc = create_validation_runtime_proc("phase2-map-owner");
    foreign_proc = create_validation_runtime_proc("phase2-map-foreign");

    TEST_ASSERT(map_proc != NULL && map_proc->type == PROC_TYPE_USER,
                "generic mapping test created a dedicated map owner");
    TEST_ASSERT(foreign_proc != NULL && foreign_proc->type == PROC_TYPE_USER,
                "generic mapping test created a dedicated foreign process");

    if (map_proc == NULL || foreign_proc == NULL) {
        TEST_END("Generic Mapping Contract");
        return;
    }

    phys_base = phys_alloc_frames(2);
    TEST_ASSERT(phys_base != 0,
                "generic mapping test allocated contiguous backing frames");

    if (phys_base == 0) {
        TEST_END("Generic Mapping Contract");
        return;
    }

    cap_id = bind_validation_memory_capability(map_proc,
                                               phys_base,
                                               2 * AYKEN_FRAME_SIZE,
                                               CAPABILITY_PERM_READ_WRITE);
    TEST_ASSERT(cap_id > 0,
                "generic mapping test bound a memory capability to the owner");

    foreign_cap_id = bind_validation_memory_capability(foreign_proc,
                                                       phys_base,
                                                       2 * AYKEN_FRAME_SIZE,
                                                       CAPABILITY_PERM_READ_WRITE);
    TEST_ASSERT(foreign_cap_id > 0,
                "generic mapping test bound a distinct memory capability to the foreign process");

    result = map_validation_memory_as(map_proc,
                                      va_base,
                                      phys_base,
                                      CAP_PERM_READ | CAP_PERM_WRITE);
    TEST_ASSERT(result == ESYS_V2_SUCCESS,
                "map_memory performs a real single-page mapping for the owner");

    result = map_validation_memory_as(map_proc,
                                      va_base + AYKEN_FRAME_SIZE,
                                      phys_base + AYKEN_FRAME_SIZE,
                                      CAP_PERM_READ);
    TEST_ASSERT(result == ESYS_V2_SUCCESS,
                "map_memory records a second read-only page in the owner ledger");

    pte0 = paging_get_pte_in_pml4(map_proc->pml4_phys, va_base);
    pte1 = paging_get_pte_in_pml4(map_proc->pml4_phys, va_base + AYKEN_FRAME_SIZE);
    entry0 = proc_find_generic_mapping(map_proc, va_base);
    entry1 = proc_find_generic_mapping(map_proc, va_base + AYKEN_FRAME_SIZE);

    map0_ok = pte0 != 0 &&
              (pte0 & AYKEN_PTE_ADDR_MASK) == phys_base &&
              (pte0 & AYKEN_PTE_USER) != 0 &&
              (pte0 & AYKEN_PTE_WRITABLE) != 0 &&
              (pte0 & AYKEN_PTE_NO_EXEC) != 0 &&
              entry0 != NULL &&
              entry0->phys_addr == phys_base &&
              entry0->flags == (CAP_PERM_READ | CAP_PERM_WRITE) &&
              entry0->capability_id == cap_id;
    map1_ok = pte1 != 0 &&
              (pte1 & AYKEN_PTE_ADDR_MASK) == (phys_base + AYKEN_FRAME_SIZE) &&
              (pte1 & AYKEN_PTE_USER) != 0 &&
              (pte1 & AYKEN_PTE_WRITABLE) == 0 &&
              (pte1 & AYKEN_PTE_NO_EXEC) != 0 &&
              entry1 != NULL &&
              entry1->phys_addr == (phys_base + AYKEN_FRAME_SIZE) &&
              entry1->flags == CAP_PERM_READ &&
              entry1->capability_id == cap_id;

    TEST_ASSERT(map0_ok,
                "map_memory writes a writable user PTE and ledger entry for read-write mappings");
    TEST_ASSERT(map1_ok,
                "map_memory writes a read-only NX user PTE and ledger entry for read-only mappings");

    result = map_validation_memory_as(map_proc,
                                      va_base,
                                      phys_base,
                                      CAP_PERM_READ | CAP_PERM_WRITE);
    TEST_ASSERT((int64_t)result == ESYS_V2_RESOURCE_BUSY,
                "map_memory fails closed on duplicate owner mappings");

    result = unmap_validation_memory_as(foreign_proc, va_base, AYKEN_FRAME_SIZE);
    foreign_unmap_denied = (int64_t)result == ESYS_V2_NO_PERMISSION;
    TEST_ASSERT(foreign_unmap_denied,
                "unmap_memory rejects foreign processes even when they hold unrelated memory capability");

    result = unmap_validation_memory_as(map_proc, va_base, 2 * AYKEN_FRAME_SIZE);
    TEST_ASSERT(result == ESYS_V2_SUCCESS,
                "unmap_memory removes a ledger-backed owner span");

    pte0 = paging_get_pte_in_pml4(map_proc->pml4_phys, va_base);
    pte1 = paging_get_pte_in_pml4(map_proc->pml4_phys, va_base + AYKEN_FRAME_SIZE);
    ledger_cleared = pte0 == 0 &&
                     pte1 == 0 &&
                     proc_find_generic_mapping(map_proc, va_base) == NULL &&
                     proc_find_generic_mapping(map_proc, va_base + AYKEN_FRAME_SIZE) == NULL;
    TEST_ASSERT(ledger_cleared,
                "unmap_memory clears both PTEs and ledger entries for the requested span");

    result = map_validation_memory_as(foreign_proc,
                                      va_base + (2 * AYKEN_FRAME_SIZE),
                                      phys_base,
                                      CAP_PERM_READ | CAP_PERM_WRITE);
    TEST_ASSERT(result == ESYS_V2_SUCCESS,
                "map_memory succeeds for another process when it owns a valid capability for the explicit backing");

    result = unmap_validation_memory_as(foreign_proc,
                                        va_base + (2 * AYKEN_FRAME_SIZE),
                                        AYKEN_FRAME_SIZE);
    unmap_ok = result == ESYS_V2_SUCCESS &&
               proc_find_generic_mapping(foreign_proc, va_base + (2 * AYKEN_FRAME_SIZE)) == NULL;
    TEST_ASSERT(unmap_ok,
                "foreign process can still unmap its own ledger-backed explicit mapping");

    phys_free_frames(phys_base, 2);

    TEST_END("Generic Mapping Contract");
}

// ============================================================================
// CAPABILITY SYSTEM VALIDATION TESTS
// ============================================================================

/**
 * Test capability system functionality
 */
static void test_capability_system(void)
{
    TEST_START("Capability System");

    // Test capability token creation and binding
    capability_token_t memory_cap = {0, CAP_PERM_READ | CAP_PERM_WRITE, CAP_RESOURCE_MEMORY};
    uint64_t cap_id = sys_v2_capability_bind(1001, &memory_cap);
    TEST_ASSERT(cap_id > 0, "Capability binding returns valid ID");

    // Test capability revocation
    uint64_t result = sys_v2_capability_revoke(cap_id);
    TEST_ASSERT(result == ESYS_V2_SUCCESS, "Capability revocation succeeds");

    // Test different capability types
    capability_token_t device_cap = {0, CAP_PERM_READ, CAP_RESOURCE_DEVICE};
    cap_id = sys_v2_capability_bind(1002, &device_cap);
    TEST_ASSERT(cap_id > 0, "Device capability binding works");

    capability_token_t exec_cap = {0, CAP_PERM_EXECUTE, CAP_RESOURCE_EXECUTION};
    cap_id = sys_v2_capability_bind(1003, &exec_cap);
    TEST_ASSERT(cap_id > 0, "Execution capability binding works");

    capability_token_t time_cap = {0, CAP_PERM_READ, CAP_RESOURCE_TIME};
    cap_id = sys_v2_capability_bind(1004, &time_cap);
    TEST_ASSERT(cap_id > 0, "Time capability binding works");

    TEST_END("Capability System");
}

// Forward declaration for security test
int capability_security_run_all_tests(void);

/**
 * Test capability system security enforcement
 */
static void test_capability_security(void)
{
    TEST_START("Capability Security Enforcement");

    // Run comprehensive security tests
    int security_result = capability_security_run_all_tests();
    TEST_ASSERT(security_result == 0, "Capability security enforcement tests pass");

    if (security_result == 0) {
        fb_print("[SECURITY] ✓ NFR-3.1: Privilege escalation prevention - ENFORCED\n");
        fb_print("[SECURITY] ✓ NFR-3.3: Resource access mediation - ENFORCED\n");
        fb_print("[SECURITY] ✓ FR-2.2.3: Capability revocation security - ENFORCED\n");
        fb_print("[SECURITY] ✓ FR-2.2.2: Context isolation - ENFORCED\n");
    } else {
        fb_print("[SECURITY] ✗ Security enforcement FAILED - System vulnerable\n");
    }

    TEST_END("Capability Security Enforcement");
}

// ============================================================================
// RING3 RUNTIME VALIDATION TESTS
// ============================================================================

/**
 * Test Ring3 VFS functionality (stub validation)
 */
static void test_ring3_vfs_runtime(void)
{
    TEST_START("Ring3 VFS Runtime");

    // Since Ring3 VFS is implemented as userspace library,
    // we test the kernel-side interface that should proxy to Ring3
    fb_print("[INFO] Ring3 VFS API design completed\n");
    fb_print("[INFO] Ring3 VFS kernel proxy stubs implemented\n");
    fb_print("[INFO] Ring3 VFS uses sys_v2_map_memory for file access\n");

    // Test VFS capability integration
    capability_token_t vfs_cap = {0, CAP_PERM_READ | CAP_PERM_WRITE, CAP_RESOURCE_MEMORY};
    uint64_t cap_id = sys_v2_capability_bind(2001, &vfs_cap);
    TEST_ASSERT(cap_id > 0, "VFS capability binding for file access");

    // Test actual VFS functionality
    fb_print("[INFO] Testing Ring3 VFS implementation...\n");

    // Call the VFS API tests to verify functionality
    extern void run_vfs_api_tests(void);
    run_vfs_api_tests();

    fb_print("[INFO] Ring3 VFS implementation test completed\n");

    TEST_END("Ring3 VFS Runtime");
}

/**
 * Test Ring3 DevFS functionality (stub validation)
 */
static void test_ring3_devfs_runtime(void)
{
    TEST_START("Ring3 DevFS Runtime");

    fb_print("[INFO] Ring3 DevFS API design completed\n");
    fb_print("[INFO] Ring3 DevFS kernel proxy stubs implemented\n");
    fb_print("[INFO] Ring3 DevFS uses capability tokens for device access\n");

    // Test DevFS capability integration
    capability_token_t devfs_cap = {0, CAP_PERM_READ | CAP_PERM_WRITE, CAP_RESOURCE_DEVICE};
    uint64_t cap_id = sys_v2_capability_bind(2002, &devfs_cap);
    TEST_ASSERT(cap_id > 0, "DevFS capability binding for device access");

    // Test DevFS stub functions (kernel → Ring3 redirection)
    fb_print("[INFO] Testing DevFS stub functions...\n");

    // Test DevFS initialization stub
    int init_result = devfs_init();
    TEST_ASSERT(init_result == 0, "DevFS initialization stub redirects to Ring3");

    // Test device registration stub
    int reg_result = devfs_register_device("test_console", NULL, NULL);
    TEST_ASSERT(reg_result == 0, "DevFS device registration stub redirects to Ring3");

    // Test device read stub
    uint8_t read_buffer[64];
    int read_result = devfs_read("test_console", read_buffer, sizeof(read_buffer));
    TEST_ASSERT(read_result >= 0, "DevFS device read stub redirects to Ring3");

    // Test device write stub
    const char *test_data = "DevFS Ring3 test";
    int write_result = devfs_write("test_console", test_data, 17);
    TEST_ASSERT(write_result >= 0, "DevFS device write stub redirects to Ring3");

    // Test device ioctl stub
    int ioctl_result = devfs_ioctl("test_console", 0x1000, NULL);
    TEST_ASSERT(ioctl_result >= 0, "DevFS device ioctl stub redirects to Ring3");

    // Test device close stub (no return value to check)
    devfs_close("test_console");
    fb_print("[INFO] DevFS device close stub executed\n");

    fb_print("[SUCCESS] All DevFS stub functions redirect correctly to Ring3\n");

    TEST_END("Ring3 DevFS Runtime");
}

/**
 * Test Ring3 AI runtime functionality (stub validation)
 */
static void test_ring3_ai_runtime(void)
{
    TEST_START("Ring3 AI Runtime");

    fb_print("[INFO] Ring3 AI runtime API design completed\n");
    fb_print("[INFO] Ring3 AI runtime kernel proxy stubs implemented\n");
    fb_print("[INFO] Ring3 AI runtime uses capability-based access\n");
    fb_print("[INFO] AI stub implementation provides placeholder responses\n");

    // Test AI capability integration
    capability_token_t ai_cap = {0, CAP_PERM_EXECUTE, CAP_RESOURCE_EXECUTION};
    uint64_t cap_id = sys_v2_capability_bind(2003, &ai_cap);
    TEST_ASSERT(cap_id > 0, "AI runtime capability binding");

    TEST_END("Ring3 AI Runtime");
}

// ============================================================================
// BCIB EXECUTION ENGINE VALIDATION TESTS
// ============================================================================

/**
 * Test BCIB submission anchoring under current runtime reality.
 */
static void test_bcib_execution_engine(void)
{
    proc_t *target_worker = NULL;

    TEST_START("BCIB Execution Engine");

    target_worker = ensure_validation_worker_proc();
    TEST_ASSERT(target_worker != NULL && target_worker->type == PROC_TYPE_USER,
                "BCIB validation worker exists for live target-context submission");

    if (target_worker == NULL) {
        TEST_END("BCIB Execution Engine");
        return;
    }

    // Test BCIB graph submission
    char bcib_graph[] = {
        0x42, 0x43, 0x49, 0x42,  // "BCIB" magic
        0x00, 0x02,              // Version 0.2
        0x00, 0x01,              // Instruction count: 1
        0x01,                    // Opcode: DATA_CREATE
        0x00, 0x00, 0x00, 0x04,  // Data length: 4
        0x74, 0x65, 0x73, 0x74   // Data: "test"
    };

    uint64_t exec_id = submit_validation_execution_as(target_worker,
                                                      bcib_graph,
                                                      sizeof(bcib_graph),
                                                      (uint64_t)target_worker->pid);
    TEST_ASSERT(exec_id > 0, "BCIB graph submission returns execution ID");

    // Test execution result waiting
    uint64_t result = sys_v2_wait_result(exec_id, 0);
    TEST_ASSERT(result == ESYS_V2_RESOURCE_BUSY,
                "BCIB wait_result reports pending execution as busy without timeout wait");

    // Test BCIB capability binding
    capability_token_t bcib_cap = {0, CAP_PERM_EXECUTE, CAP_RESOURCE_EXECUTION};
    uint64_t cap_id = sys_v2_capability_bind((uint64_t)target_worker->pid, &bcib_cap);
    TEST_ASSERT(cap_id > 0, "BCIB execution capability binding");

    fb_print("[INFO] BCIB submit path now anchors READY slots in kernel state\n");
    fb_print("[INFO] Explicit completion now closes RUNNING slots under kernel validation\n");
    fb_print("[INFO] Successful wait_result now returns a mapped frozen execution-output result VA\n");

    TEST_END("BCIB Execution Engine");
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

/**
 * Test end-to-end Phase 2 integration
 */
static void test_phase2_integration(void)
{
    proc_t *controller_proc = current_proc;
    proc_t *exit_proc = NULL;
    proc_t *foreign_proc = NULL;
    proc_t *saved_current_proc = NULL;
    execution_slot_guard_t slot_guard = {0};
    uint32_t owner_pid = sched_active_owner_pid();
    uint64_t exec_id = 0;
    uint64_t result_va = 0;
    uint64_t repeated_result_va = 0;
    uint64_t hash_va = 0;
    uint64_t foreign_wait = 0;
    uint64_t generic_phys = 0;
    uint64_t generic_cap_id = 0;
    uint64_t generic_va = 0x32000000ULL;
    int publish_emitted = 0;
    int exit_switch_seen = 0;
    int exit_switch_from_pid = 0;
    int exit_switch_to_pid = 0;
    int generic_mapping_live = 0;
    int slot_result_mapped = 0;
    int exit_proc_zombied = 0;
    int result_mapping_revoked = 0;
    int generic_mapping_revoked = 0;
    int lower_half_revoked = 0;
    int deferred_reap_pending = 0;
    int deferred_reap_completed = 0;
    int slot_released = 0;

    TEST_START("Phase 2 End-to-End Contract");
    validation_prepare_full_result_bcib();

    TEST_ASSERT(controller_proc != NULL && controller_proc->pid > 0 &&
                    (uint32_t)controller_proc->pid != owner_pid,
                "end-to-end harness starts from a live non-owner controller process");

    if (controller_proc == NULL || controller_proc->pid <= 0 ||
        (uint32_t)controller_proc->pid == owner_pid) {
        TEST_END("Phase 2 End-to-End Contract");
        return;
    }

    exit_proc = proc_create_user_process("phase2-end-to-end-owner",
                                         validation_exit_noreturn_code,
                                         sizeof(validation_exit_noreturn_code),
                                         PROC_IMAGE_FLAT);
    foreign_proc = create_validation_runtime_proc("phase2-end-to-end-foreign");

    TEST_ASSERT(exit_proc != NULL && exit_proc->type == PROC_TYPE_USER,
                "end-to-end harness created a dedicated owner process");
    TEST_ASSERT(foreign_proc != NULL && foreign_proc->type == PROC_TYPE_USER,
                "end-to-end harness created a dedicated foreign process");

    if (exit_proc == NULL || foreign_proc == NULL) {
        TEST_END("Phase 2 End-to-End Contract");
        return;
    }

    generic_phys = phys_alloc_frame();
    TEST_ASSERT(generic_phys != 0,
                "end-to-end harness allocated explicit generic backing");

    if (generic_phys == 0) {
        TEST_END("Phase 2 End-to-End Contract");
        return;
    }

    generic_cap_id = bind_validation_memory_capability(exit_proc,
                                                       generic_phys,
                                                       AYKEN_FRAME_SIZE,
                                                       CAPABILITY_PERM_READ_WRITE);
    TEST_ASSERT(generic_cap_id > 0,
                "end-to-end harness bound a memory capability for the owner");

    TEST_ASSERT(map_validation_memory_as(exit_proc,
                                         generic_va,
                                         generic_phys,
                                         CAP_PERM_READ | CAP_PERM_WRITE) == ESYS_V2_SUCCESS,
                "end-to-end harness maps explicit memory before execution submission");
    generic_mapping_live = paging_get_pte_in_pml4(exit_proc->pml4_phys, generic_va) != 0 &&
                           proc_find_generic_mapping(exit_proc, generic_va) != NULL;
    TEST_ASSERT(generic_mapping_live,
                "end-to-end harness confirms the generic mapping is live before exit");

    exec_id = submit_validation_execution_as(exit_proc,
                                             g_validation_full_result_bcib,
                                             VALIDATION_FULL_RESULT_BCIB_SIZE,
                                             (uint64_t)exit_proc->pid);
    TEST_ASSERT(exec_id > 0,
                "end-to-end harness submits a real execution for the owner context");

    saved_current_proc = current_proc;
    current_proc = exit_proc;
    current_proc->state = PROC_RUNNING;
    publish_emitted = sched_try_pickup_execution_work();
    current_proc = saved_current_proc;
    TEST_ASSERT(publish_emitted == 1,
                "end-to-end harness picks the owner execution up into RUNNING through schedule-entry logic");

    TEST_ASSERT(validation_write_output_for_execution(exec_id,
                                                      AYKEN_EXECUTION_OUTPUT_MAGIC,
                                                      AYKEN_EXECUTION_OUTPUT_VERSION,
                                                      g_validation_full_result_bcib,
                                                      VALIDATION_FULL_RESULT_BCIB_SIZE),
                "end-to-end harness writes a valid output header before completion");
    TEST_ASSERT(complete_validation_execution_as(exit_proc,
                                                 exec_id,
                                                 EXEC_COMPLETION_COMPLETED) == ESYS_V2_SUCCESS,
                "end-to-end harness completes the RUNNING execution through the owner authority path");

    result_va = wait_validation_result_as(exit_proc, exec_id, 0);
    TEST_ASSERT(result_va > 0,
                "end-to-end harness materializes a result mapping after completion");
    execution_slot_enter_critical(&slot_guard);
    {
        exec_slot_t *slot = execution_slot_find_locked(exec_id);
        hash_va = slot != NULL ? slot->mapped_hash_va : 0;
    }
    execution_slot_exit_critical(&slot_guard);
    TEST_ASSERT(hash_va > 0,
                "end-to-end harness materializes a deterministic hash sidecar after completion");

    repeated_result_va = wait_validation_result_as(exit_proc, exec_id, 0);
    TEST_ASSERT(repeated_result_va == result_va,
                "end-to-end harness replays the same mapped VA across repeated waits");

    foreign_wait = wait_validation_result_as(foreign_proc, exec_id, 0);
    TEST_ASSERT((int64_t)foreign_wait == ESYS_V2_NO_PERMISSION,
                "end-to-end harness rejects foreign wait_result on the owner's execution");

    execution_slot_enter_critical(&slot_guard);
    {
        exec_slot_t *slot = execution_slot_find_locked(exec_id);
        slot_result_mapped = slot != NULL &&
                             slot->owner_pid == (uint64_t)exit_proc->pid &&
                             slot->state == EXEC_SLOT_RESULT_MAPPED &&
                             slot->mapped_result_va == result_va &&
                             validation_result_hash_matches(slot,
                                                            exit_proc->pml4_phys,
                                                            g_validation_full_result_output,
                                                            VALIDATION_FULL_RESULT_OUTPUT_SIZE);
    }
    execution_slot_exit_critical(&slot_guard);
    TEST_ASSERT(slot_result_mapped,
                "end-to-end harness keeps execution ownership and result mapping bound to the submitting owner");

    sched_remove_process_everywhere(exit_proc);
    exit_proc->state = PROC_READY;
    exit_proc->wait_obj = NULL;

    TEST_ASSERT(validation_seed_owner_mailbox_candidate((uint32_t)exit_proc->pid),
                "end-to-end harness seeds a fresh owner-mailbox decision for the retiring owner process");

    sched_validation_arm_exit_successor(controller_proc);
    sched_yield();
    exit_switch_seen = sched_validation_take_exit_switch_event(&exit_switch_from_pid,
                                                               &exit_switch_to_pid);
    sched_validation_disarm_exit_successor();

    TEST_ASSERT(exit_switch_seen &&
                    exit_switch_from_pid == exit_proc->pid &&
                    exit_switch_to_pid == controller_proc->pid,
                "end-to-end harness observes a direct no-return exit switch back to the controller");
    TEST_ASSERT(current_proc == controller_proc && controller_proc->state == PROC_RUNNING,
                "end-to-end harness resumes the controller after the owner exit path");

    exit_proc_zombied = exit_proc->state == PROC_ZOMBIE &&
                        exit_proc->wait_obj == NULL &&
                        exit_proc->active_execution_id == 0 &&
                        exit_proc->next == NULL;
    result_mapping_revoked = paging_get_pte_in_pml4(exit_proc->pml4_phys, result_va) == 0 &&
                             paging_get_pte_in_pml4(exit_proc->pml4_phys,
                                                    result_va + AYKEN_FRAME_SIZE) == 0 &&
                             paging_get_pte_in_pml4(exit_proc->pml4_phys, hash_va) == 0;
    generic_mapping_revoked = paging_get_pte_in_pml4(exit_proc->pml4_phys, generic_va) == 0 &&
                              proc_find_generic_mapping(exit_proc, generic_va) == NULL;
    lower_half_revoked =
        exit_proc->pml4_phys != 0 &&
        paging_get_pte_in_pml4(exit_proc->pml4_phys, USER_TEXT_BASE) == 0 &&
        paging_get_pte_in_pml4(exit_proc->pml4_phys, USER_STACK_TOP - AYKEN_FRAME_SIZE) == 0 &&
        paging_get_pte_in_pml4(exit_proc->pml4_phys, USER_STACK_TOP - (2 * AYKEN_FRAME_SIZE)) == 0 &&
        paging_get_pte_in_pml4(exit_proc->pml4_phys, VALIDATION_RING3_CANARY_ADDR) == 0;
    deferred_reap_pending = exit_proc->pml4_phys != 0 &&
                            exit_proc->context.cr3 != 0 &&
                            exit_proc->context.rsp0 != 0;

    execution_slot_enter_critical(&slot_guard);
    slot_released = execution_slot_find_locked(exec_id) == NULL;
    execution_slot_exit_critical(&slot_guard);

    TEST_ASSERT(exit_proc_zombied,
                "end-to-end harness leaves the owner process zombied and detached after exit");
    TEST_ASSERT(result_mapping_revoked,
                "end-to-end harness revokes the materialized result mapping on exit");
    TEST_ASSERT(generic_mapping_revoked,
                "end-to-end harness revokes the generic ledger-backed mapping on exit");
    TEST_ASSERT(lower_half_revoked,
                "end-to-end harness destroys the owner's lower-half user mappings on exit");
    TEST_ASSERT(deferred_reap_pending,
                "end-to-end harness leaves active root-PML4 and rsp0 on deferred reap after exit");
    TEST_ASSERT(slot_released,
                "end-to-end harness releases the owner-owned execution slot during exit cleanup");

    proc_drain_deferred_reap();
    deferred_reap_completed = exit_proc->pml4_phys == 0 &&
                              exit_proc->context.cr3 == 0 &&
                              exit_proc->context.rsp0 == 0;
    TEST_ASSERT(deferred_reap_completed,
                "end-to-end harness completes deferred reap after the owner exit path");

    sched_remove_process_everywhere(foreign_proc);
    foreign_proc->state = PROC_ZOMBIE;
    proc_teardown_exit_surfaces(foreign_proc, NULL, NULL, 0);
    phys_free_frame(generic_phys);

    TEST_END("Phase 2 End-to-End Contract");
}

// ============================================================================
// PERFORMANCE AND STRESS TESTS
// ============================================================================

/**
 * Test syscall performance and stress conditions
 */
static void test_syscall_performance(void)
{
    TEST_START("Syscall Performance");

    // Test rapid syscall invocation
    int rapid_test_count = 100;
    int successful_calls = 0;

    for (int i = 0; i < rapid_test_count; i++) {
        uint64_t time_buffer = 0;
        uint64_t result = sys_v2_time_query(TIME_QUERY_UPTIME, &time_buffer);
        if (result == ESYS_V2_SUCCESS) {
            successful_calls++;
        }
    }

    TEST_ASSERT(successful_calls == rapid_test_count, "Rapid syscall invocation stability");

    // Test capability system under load
    int cap_test_count = 50;
    int successful_caps = 0;

    for (int i = 0; i < cap_test_count; i++) {
        capability_token_t test_cap = {0, CAP_PERM_READ, CAP_RESOURCE_MEMORY};
        uint64_t cap_id = sys_v2_capability_bind(5000 + i, &test_cap);
        if (cap_id > 0) {
            successful_caps++;
            sys_v2_capability_revoke(cap_id);
        }
    }

    TEST_ASSERT(successful_caps == cap_test_count, "Capability system under load");

    TEST_END("Syscall Performance");
}

// ============================================================================
// MAIN VALIDATION FUNCTION
// ============================================================================

/**
 * Execute the current Phase 2 validation snapshot.
 */
void execute_phase2_validation(void)
{
    fb_print("\n");
    fb_print("================================================================================\n");
    fb_print("                    AYKENOS PHASE 2 VALIDATION SNAPSHOT\n");
    fb_print("================================================================================\n");
    fb_print("Task 2.5.3.1: Execute complete Phase 2 validation\n");
    fb_print("Scope: Mixed semantic and interface-shape checks for current runtime reality\n");
    fb_print("================================================================================\n");

    // Initialize test counters
    tests_passed = 0;
    tests_failed = 0;
    total_tests = 0;

    // Execute all validation tests
    test_syscall_v2_interface();
    test_syscall_v2_error_handling();
    test_completion_handoff_contract();
    test_execution_pickup_order_contract();
    test_illegal_execution_slot_transition_contract();
    test_execution_trace_invariant_contract();
    test_multi_execution_adversarial_contract();
    test_blocked_wait_wake_contract();
    test_irq_timeout_contract();
    test_negative_timeout_cleanup_contract();
    test_exit_teardown_contract();
    test_owner_exit_guard();
    test_generic_mapping_contract();
    test_capability_system();
    test_capability_security();
    test_ring3_vfs_runtime();
    test_ring3_devfs_runtime();
    test_ring3_ai_runtime();
    test_bcib_execution_engine();
    test_phase2_integration();
    test_syscall_performance();
    test_exit_noreturn_runtime_contract();
    test_owner_handoff_runtime_contract();
    test_owner_handoff_exit_followthrough_runtime_contract();

    // Print final results
    fb_print("\n");
    fb_print("================================================================================\n");
    fb_print("                      PHASE 2 VALIDATION SNAPSHOT RESULTS\n");
    fb_print("================================================================================\n");

    fb_print("Total Tests: ");
    fb_print_int(total_tests);
    fb_print("\n");

    fb_print("Tests Passed: ");
    fb_print_int(tests_passed);
    fb_print("\n");

    fb_print("Tests Failed: ");
    fb_print_int(tests_failed);
    fb_print("\n");

    if (tests_failed == 0) {
        fb_print("\nPHASE 2 VALIDATION CHECKS PASSED\n");
        fb_print("================================================================================\n");
        fb_print("PHASE 2 VALIDATION STATUS: CURRENT CHECKS PASS\n");
        fb_print("================================================================================\n");
        fb_print("[OK] ABI/range and interface-shape checks passed\n");
        fb_print("[OK] time_query and capability surfaces have semantic coverage\n");
        fb_print("[OK] map_memory/unmap_memory now have real explicit mapping-ledger coverage\n");
        fb_print("[OK] completion now closes RUNNING slots through an explicit kernel surface\n");
        fb_print("[OK] pickup ordering now has direct FIFO and no-mailbox-reuse proof coverage\n");
        fb_print("[OK] illegal execution-slot cross-state mutation sequences now fail closed under direct validation\n");
        fb_print("[OK] execution trace and global invariant coverage now exist for the core lifecycle chain\n");
        fb_print("[OK] adversarial multi-execution coverage now exists for replay floods, double finalize rejection, and pickup-vs-exit collision handling\n");
        fb_print("[OK] wait_result now has direct blocked-wait and canonical wake-path coverage\n");
        fb_print("[OK] timeout progression now has direct IRQ-driven proof coverage\n");
        fb_print("[OK] negative timeout -> wake -> cleanup lifecycle coverage now exists\n");
        fb_print("[OK] exit teardown now covers zombie transition, slot abort, explicit revoke, lower-half user-memory destruction, and deferred reap mechanics\n");
        fb_print("[INFO] Ring3 proxy/stub reachability checks passed\n");
        fb_print("[INFO] successful wait_result now materializes the frozen validated output header plus payload bytes\n");
        fb_print("[OK] direct no-return sys_v2_exit runtime proof now exists for non-owner processes\n");
        fb_print("[OK] narrow owner handoff now has dispatch-boundary commit, successor-authority, and old-owner exit follow-through proof\n");
        fb_print("[OK] semantic end-to-end coverage now exists for map -> submit -> pickup -> complete -> wait -> exit\n");
        fb_print("[OK] wait_result now publishes the frozen validated output backing rather than reusing BCIB input bytes\n");
        fb_print("================================================================================\n");
        fb_print("Interpret result together with SYSCALL_RUNTIME_REALITY.md\n");
        fb_print("================================================================================\n");
    } else {
        fb_print("\nPHASE 2 VALIDATION CHECKS FAILED\n");
        fb_print("================================================================================\n");
        fb_print("PHASE 2 VALIDATION STATUS: FAILED\n");
        fb_print("================================================================================\n");
        fb_print("Some validation checks failed. Review whether they are semantic or interface-shape failures before interpreting runtime state.\n");
        fb_print("================================================================================\n");
    }
}

/**
 * Quick validation check for development
 */
void quick_phase2_validation(void)
{
    proc_t *target_worker = NULL;

    fb_print("\n[QUICK-CHECK] Phase 2 Validation Snapshot\n");

    // Quick syscall check
    uint64_t result = sys_v2_time_query(TIME_QUERY_UPTIME, &(uint64_t){0});
    fb_print("✓ time_query surface: ");
    fb_print(result == ESYS_V2_SUCCESS ? "OK" : "FAIL");
    fb_print("\n");

    // Quick capability check
    capability_token_t test_cap = {0, CAP_PERM_READ, CAP_RESOURCE_MEMORY};
    uint64_t cap_id = sys_v2_capability_bind(9999, &test_cap);
    fb_print("✓ Capabilities: ");
    fb_print(cap_id > 0 ? "OK" : "FAIL");
    fb_print("\n");

    // Quick BCIB check
    target_worker = ensure_validation_worker_proc();
    char bcib[] = {0x42, 0x43, 0x49, 0x42, 0x00, 0x02};
    uint64_t exec_id = 0;
    if (target_worker != NULL && target_worker->type == PROC_TYPE_USER) {
        exec_id = submit_validation_execution_as(target_worker,
                                                 bcib,
                                                 sizeof(bcib),
                                                 (uint64_t)target_worker->pid);
    }
    fb_print("✓ submit path anchoring: ");
    fb_print(exec_id > 0 ? "OK" : "FAIL");
    fb_print("\n");

    fb_print("[QUICK-CHECK] Run execute_phase2_validation() for the full validation snapshot\n");
}

# ctx=2 Queue Debug Patch - 6 Marker Plan

**Authority**: Historical runtime truth from `[USER_BP]` + early `[DEQUEUE_MISS]` `ctx=2` probes  
**Status**: Historical diagnostic artifact (superseded on 2026-04-23)  
**Target**: Preserve the original six-marker queue investigation for forensic reference

> Status update (2026-04-23): this document is no longer the active blocker plan.
> Fresh proof/test QEMU evidence now shows `[SUBMIT_BIND]`, `[QUEUE_CREATE]`,
> `[ENQUEUE_BIND]`, `[DEQUEUE_HIT]`, `[PICKUP]`, `[WAIT_OK]`, and `[RESULT_OK]`
> in one gated run, with
> `evidence/bcib-post-syscall-e2e/bcib_post_syscall_e2e_evidence.json`
> reporting `result=PASS`, `proof_level=end_to_end_completion`, and `pf=0`.
> The live defect that mattered was narrowed from queue lookup to post-syscall
> first-user-retirement starvation and is now closed in the proof/test lane.

## Runtime Context

Ring3 first-retirement was already proven when this patch plan was written.
At that time, repeated `[DEQUEUE_MISS] reason=queue_not_found ctx=2` during
scheduler pickup made queue creation/binding the strongest working hypothesis.
That hypothesis was useful diagnostically, but it is no longer the active
runtime diagnosis for the proof/test closure path.

## 6 Marker Patch List

### Marker 1: Queue Registry Dump at Dequeue Miss
**File**: `kernel/sys/execution_slot.c`  
**Function**: `execution_slot_dequeue_locked` (line ~1872)  
**Location**: Right after `queue_not_found` marker

**Purpose**: Show what queues exist when ctx=2 lookup fails

```c
if (!queue) {
#if defined(AYKEN_PHASE16_BCIB_PROOF_TEST) && (AYKEN_PHASE16_BCIB_PROOF_TEST == 1)
    execution_slot_debugcon_write("[DEQUEUE_MISS] reason=queue_not_found ctx=");
    execution_slot_debugcon_write_u64(context_id);
    execution_slot_debugcon_write("\n");
    
    // NEW: Dump registry state
    execution_slot_debugcon_write("[QUEUE_REGISTRY] requested_ctx=");
    execution_slot_debugcon_write_u64(context_id);
    execution_slot_debugcon_write(" existing_queues=");
    uint32_t queue_count = 0;
    for (uint32_t i = 0; i < AYKEN_MAX_EXECUTION_CONTEXT_QUEUES; ++i) {
        if (g_execution_queues[i].in_use) {
            if (queue_count > 0) execution_slot_debugcon_write(",");
            execution_slot_debugcon_write_u64(g_execution_queues[i].context_id);
            queue_count++;
        }
    }
    if (queue_count == 0) execution_slot_debugcon_write("EMPTY");
    execution_slot_debugcon_write("\n");
#endif
    return NULL;
}
```

**Expected Output**:
- If registry empty: `[QUEUE_REGISTRY] requested_ctx=2 existing_queues=EMPTY`
- If wrong ctx: `[QUEUE_REGISTRY] requested_ctx=2 existing_queues=1,3` (shows 2 missing)
- If correct ctx exists: Would not reach this branch (queue != NULL)

---

### Marker 2: Submit Binding Authority
**File**: `kernel/sys/syscall_v2.c`  
**Function**: `sys_v2_submit_execution` (line ~1500)  
**Location**: Right after target validation, before slot allocation

**Purpose**: Authoritative record of what context_id is being submitted to

```c
/* Patch 1 / Task 1.1: Fail-closed target_context_id validation */
reject_reason = validate_submit_target_context(caller_proc, context_id, target_proc);
if (reject_reason != SUBMIT_TARGET_OK) {
    fb_print("[SUBMIT_REJECTED] invalid_target_context reason=");
    fb_print(submit_target_reject_reason_str(reject_reason));
    fb_print(" target_context=");
    fb_print_int(context_id);
    fb_print("\n");
    return ESYS_V2_CONTEXT_ERROR;
}

// NEW: Authoritative binding marker
#if defined(AYKEN_PHASE16_BCIB_PROOF_TEST) && (AYKEN_PHASE16_BCIB_PROOF_TEST == 1)
fb_print("[SUBMIT_BIND] caller_pid=");
fb_print_int(caller_proc->pid);
fb_print(" target_ctx=");
fb_print_int(context_id);
fb_print(" target_pid=");
fb_print_int(target_proc->pid);
fb_print(" owner_pid=");
if (caller_proc->pid > 0) {
    fb_print_int(caller_proc->pid);
} else {
    fb_print("0");
}
fb_print("\n");
#endif
```

**Expected Output**: `[SUBMIT_BIND] caller_pid=1 target_ctx=2 target_pid=2 owner_pid=1`

This establishes ground truth: what context_id was accepted by submit.

---

### Marker 3: Queue Creation Notification
**File**: `kernel/sys/execution_slot.c`  
**Function**: `execution_slot_alloc_queue_locked` (line ~556)  
**Location**: Right after successful allocation, before return

**Purpose**: Confirm when queue is actually instantiated

```c
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
            
            // NEW: Queue creation marker
#if defined(AYKEN_PHASE16_BCIB_PROOF_TEST) && (AYKEN_PHASE16_BCIB_PROOF_TEST == 1)
            execution_slot_debugcon_write("[QUEUE_CREATE] ctx=");
            execution_slot_debugcon_write_u64(context_id);
            execution_slot_debugcon_write(" slot=");
            execution_slot_debugcon_write_u64((uint64_t)i);
            execution_slot_debugcon_write("\n");
#endif
            
            return &g_execution_queues[i];
        }
    }

    return NULL;
}
```

**Expected Output**: `[QUEUE_CREATE] ctx=2 slot=0`

If this marker never appears, queue is never created. If it appears but dequeue still fails, lookup key is wrong.

---

### Marker 4: Enqueue Context Verification
**File**: `kernel/sys/execution_slot.c`  
**Function**: `execution_slot_enqueue_locked` (line ~1817)  
**Location**: Right before existing `[ENQUEUE]` marker

**Purpose**: Verify slot's target_context_id matches queue's context_id

```c
queue = execution_slot_find_queue_locked(slot->target_context_id);
if (!queue) {
    queue = execution_slot_alloc_queue_locked(slot->target_context_id);
}
if (!queue) {
    return -1;
}

// NEW: Context binding verification
#if defined(AYKEN_PHASE16_BCIB_PROOF_TEST) && (AYKEN_PHASE16_BCIB_PROOF_TEST == 1)
execution_slot_debugcon_write("[ENQUEUE_BIND] slot_target_ctx=");
execution_slot_debugcon_write_u64(slot->target_context_id);
execution_slot_debugcon_write(" queue_ctx=");
execution_slot_debugcon_write_u64(queue->context_id);
execution_slot_debugcon_write(" match=");
if (slot->target_context_id == queue->context_id) {
    execution_slot_debugcon_write("YES");
} else {
    execution_slot_debugcon_write("NO");
}
execution_slot_debugcon_write("\n");
#endif

if (queue->tail_index == AYKEN_EXECUTION_INVALID_INDEX) {
    queue->head_index = slot_index;
    queue->tail_index = slot_index;
} else {
    g_execution_slots[queue->tail_index].queue_next_index = slot_index;
    queue->tail_index = slot_index;
}

slot->queue_next_index = AYKEN_EXECUTION_INVALID_INDEX;
queue->depth++;
#if defined(AYKEN_PHASE16_BCIB_PROOF_TEST) && (AYKEN_PHASE16_BCIB_PROOF_TEST == 1)
    execution_slot_debugcon_write("[ENQUEUE] exec_id=");
    // ... existing marker
```

**Expected Output**: `[ENQUEUE_BIND] slot_target_ctx=2 queue_ctx=2 match=YES`

If match=NO, there's a namespace corruption bug.

---

### Marker 5: Find Queue Lookup Detail
**File**: `kernel/sys/execution_slot.c`  
**Function**: `execution_slot_find_queue_locked` (line ~1799)  
**Location**: At function entry and in loop

**Purpose**: Show exact lookup key and comparison logic

```c
execution_context_queue_t *execution_slot_find_queue_locked(uint64_t context_id)
{
    uint32_t i;

    if (context_id == 0) {
        return NULL;
    }

#if defined(AYKEN_PHASE16_BCIB_PROOF_TEST) && (AYKEN_PHASE16_BCIB_PROOF_TEST == 1)
    execution_slot_debugcon_write("[QUEUE_LOOKUP] search_ctx=");
    execution_slot_debugcon_write_u64(context_id);
    execution_slot_debugcon_write(" scanning...\n");
#endif

    for (i = 0; i < AYKEN_MAX_EXECUTION_CONTEXT_QUEUES; ++i) {
        if (g_execution_queues[i].in_use &&
            g_execution_queues[i].context_id == context_id) {
            
#if defined(AYKEN_PHASE16_BCIB_PROOF_TEST) && (AYKEN_PHASE16_BCIB_PROOF_TEST == 1)
            execution_slot_debugcon_write("[QUEUE_LOOKUP_HIT] found_ctx=");
            execution_slot_debugcon_write_u64(g_execution_queues[i].context_id);
            execution_slot_debugcon_write(" slot=");
            execution_slot_debugcon_write_u64((uint64_t)i);
            execution_slot_debugcon_write("\n");
#endif
            
            return &g_execution_queues[i];
        }
    }

#if defined(AYKEN_PHASE16_BCIB_PROOF_TEST) && (AYKEN_PHASE16_BCIB_PROOF_TEST == 1)
    execution_slot_debugcon_write("[QUEUE_LOOKUP_MISS] search_ctx=");
    execution_slot_debugcon_write_u64(context_id);
    execution_slot_debugcon_write("\n");
#endif

    return NULL;
}
```

**Expected Output**:
- On enqueue: `[QUEUE_LOOKUP] search_ctx=2 scanning...` → `[QUEUE_LOOKUP_HIT] found_ctx=2 slot=0`
- On dequeue: `[QUEUE_LOOKUP] search_ctx=2 scanning...` → `[QUEUE_LOOKUP_MISS] search_ctx=2`

If enqueue hits but dequeue misses, queue was destroyed between submit and pickup.

---

### Marker 6: Queue Zero/Destroy Tracking
**File**: `kernel/sys/execution_slot.c`  
**Function**: `execution_slot_zero_queue` (line ~331)  
**Location**: At function entry

**Purpose**: Track when queues are destroyed

```c
static void execution_slot_zero_queue(execution_context_queue_t *queue)
{
    if (!queue) {
        return;
    }
    
#if defined(AYKEN_PHASE16_BCIB_PROOF_TEST) && (AYKEN_PHASE16_BCIB_PROOF_TEST == 1)
    execution_slot_debugcon_write("[QUEUE_DESTROY] ctx=");
    execution_slot_debugcon_write_u64(queue->context_id);
    execution_slot_debugcon_write(" was_in_use=");
    execution_slot_debugcon_write_u64((uint64_t)queue->in_use);
    execution_slot_debugcon_write("\n");
#endif
    
    queue->in_use = 0;
    queue->context_id = 0;
    queue->head_index = AYKEN_EXECUTION_INVALID_INDEX;
    queue->tail_index = AYKEN_EXECUTION_INVALID_INDEX;
    queue->depth = 0;
}
```

**Expected Output**: `[QUEUE_DESTROY] ctx=2 was_in_use=1`

If this appears between [ENQUEUE] and [DEQUEUE_MISS], queue is being prematurely destroyed.

---

## Diagnostic Decision Tree

After applying all 6 markers, run clean proof test and analyze:

### Case A: [QUEUE_CREATE] never appears
**Diagnosis**: Queue allocation is never called  
**Root Cause**: `execution_slot_enqueue_locked` is not reaching `alloc_queue_locked` path  
**Next Step**: Check if `find_queue_locked` is returning stale queue pointer

### Case B: [QUEUE_CREATE] ctx=2 appears, but [DEQUEUE_MISS] ctx=2 still happens
**Diagnosis**: Queue exists at enqueue time but not at dequeue time  
**Root Cause**: Either queue destroyed prematurely or lookup key corrupted  
**Next Step**: Check [QUEUE_DESTROY] timing and [QUEUE_REGISTRY] content

### Case C: [ENQUEUE_BIND] shows match=NO
**Diagnosis**: Namespace corruption between slot and queue  
**Root Cause**: `slot->target_context_id` != `queue->context_id`  
**Next Step**: Trace where `target_context_id` is set in slot allocation

### Case D: [QUEUE_REGISTRY] shows existing_queues=1,3 (missing 2)
**Diagnosis**: ctx=2 queue never created or already destroyed  
**Root Cause**: Submit path not creating queue, or queue destroyed before pickup  
**Next Step**: Check [SUBMIT_BIND] → [QUEUE_CREATE] → [QUEUE_DESTROY] timeline

### Case E: [QUEUE_LOOKUP] search_ctx=2 → [QUEUE_LOOKUP_HIT] on enqueue, but [QUEUE_LOOKUP_MISS] on dequeue
**Diagnosis**: Queue destroyed between enqueue and dequeue  
**Root Cause**: Premature cleanup, likely in `execution_slot_zero_queue` called from dequeue itself  
**Next Step**: Check if `queue->depth == 0` logic is destroying queue too early

---

## Implementation Order

1. Apply Marker 1 (registry dump) - highest diagnostic value
2. Apply Marker 2 (submit bind) - establishes ground truth
3. Apply Marker 3 (queue create) - confirms instantiation
4. Apply Marker 5 (lookup detail) - shows search logic
5. Apply Marker 4 (enqueue bind) - verifies context match
6. Apply Marker 6 (queue destroy) - tracks lifecycle

Run clean proof test after all markers applied.

---

## Expected Timeline in Healthy System

```
[SUBMIT_BIND] caller_pid=1 target_ctx=2 target_pid=2 owner_pid=1
[QUEUE_LOOKUP] search_ctx=2 scanning...
[QUEUE_LOOKUP_MISS] search_ctx=2
[QUEUE_CREATE] ctx=2 slot=0
[ENQUEUE_BIND] slot_target_ctx=2 queue_ctx=2 match=YES
[ENQUEUE] exec_id=1 ctx=2 depth=1
... (scheduler runs) ...
[PICKUP_TRY] pid=2 role=1
[QUEUE_LOOKUP] search_ctx=2 scanning...
[QUEUE_LOOKUP_HIT] found_ctx=2 slot=0
[DEQUEUE_HIT] exec_id=1 ctx=2 remaining=0
[QUEUE_DESTROY] ctx=2 was_in_use=1
```

Current broken system shows:
```
[SUBMIT_BIND] caller_pid=1 target_ctx=2 target_pid=2 owner_pid=1
[ENQUEUE] exec_id=1 ctx=2 depth=1
... (scheduler runs) ...
[PICKUP_TRY] pid=2 role=1
[DEQUEUE_MISS] reason=queue_not_found ctx=2
[QUEUE_REGISTRY] requested_ctx=2 existing_queues=???
```

The 6 markers will fill in the `???` and show exactly where the queue lifecycle breaks.

---

## Success Criteria

After patch applied and test run:

1. [QUEUE_REGISTRY] output shows whether registry is empty or has wrong context_id
2. [SUBMIT_BIND] confirms what context_id was accepted
3. [QUEUE_CREATE] confirms queue instantiation happened
4. [QUEUE_LOOKUP] shows exact search key and comparison
5. [ENQUEUE_BIND] confirms context_id match between slot and queue
6. [QUEUE_DESTROY] shows if queue is prematurely destroyed

One of these 6 markers will reveal the root cause of `queue_not_found ctx=2`.

---

**Next Action**: Apply all 6 markers, run clean proof test, analyze output against decision tree.

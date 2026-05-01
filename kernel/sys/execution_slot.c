/**
 * Phase-17 Execution Slot Implementation
 * 
 * This is the FIRST REAL EXECUTION implementation.
 * 
 * Level 1 (Day 0-2):
 * - Kernel-produced output (fake but deterministic)
 * - Inline verification skeleton
 * - State machine enforcement
 * 
 * Level 2 (Next):
 * - Real BCIB interpreter
 * - execution_context_snapshot enforcement
 * - Real AI model execution
 */

#include "execution_slot.h"
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

// Temporary stubs for kernel functions
// TODO: Replace with real kernel API
static void* kmalloc(size_t size) {
    return malloc(size);
}

static void kfree(void* ptr) {
    free(ptr);
}

static void debug_puts(const char* msg) {
    printf("%s\n", msg);
}

static void panic(const char* msg) {
    fprintf(stderr, "KERNEL PANIC: %s\n", msg);
    exit(1);
}

// Temporary SHA256 stub
// TODO: Replace with real crypto implementation
static void sha256(const uint8_t* data, size_t len, uint8_t* hash) {
    // Deterministic fake hash for now
    memset(hash, 0xAA, 32);
    if (len > 0) {
        hash[0] = data[0];
        hash[31] = data[len - 1];
    }
}

// Temporary publish stub
// TODO: Replace with real userspace publish
static void publish_result_to_userspace(execution_slot_t* slot) {
    debug_puts("[PUBLISH_TO_USERSPACE]");
    // Real implementation will copy result_buffer to userspace
}

// Initialize execution slot
void execution_slot_init(execution_slot_t *slot, uint64_t id) {
    memset(slot, 0, sizeof(execution_slot_t));
    slot->id = id;
    slot->state = SLOT_STATE_IDLE;
    
    // Allocate result buffer (4KB for now)
    slot->result_capacity = 4096;
    slot->result_buffer = kmalloc(slot->result_capacity);
    if (!slot->result_buffer) {
        panic("execution_slot: failed to allocate result buffer");
    }
    
    debug_puts("[EXEC_SLOT_INIT]");
}

// Destroy execution slot
void execution_slot_destroy(execution_slot_t *slot) {
    if (slot->result_buffer) {
        kfree(slot->result_buffer);
        slot->result_buffer = NULL;
    }
    slot->state = SLOT_STATE_IDLE;
}

// Execute (FAKE but kernel-produced)
void execution_slot_execute(execution_slot_t *slot) {
    if (slot->state != SLOT_STATE_IDLE) {
        panic("execution_slot: invalid state for execute");
    }
    
    debug_puts("[EXEC_START]");
    slot->state = SLOT_STATE_EXECUTING;
    
    // FAKE deterministic output (kernel-produced)
    // This is NOT Python stub - this is kernel code
    // Phase-17 Level 1: fake but deterministic
    // Phase-17 Level 2: real BCIB execution
    const char *msg = "HELLO_BCIB_EXECUTION";
    size_t len = strlen(msg);
    
    if (len > slot->result_capacity) {
        panic("execution_slot: result too large");
    }
    
    memcpy(slot->result_buffer, msg, len);
    slot->result_size = len;
    
    debug_puts("[EXEC_OUTPUT_WRITTEN]");
    slot->state = SLOT_STATE_WRITE_OUTPUT;
}

// Write output (seal buffer)
void execution_slot_write_output(execution_slot_t *slot) {
    if (slot->state != SLOT_STATE_WRITE_OUTPUT) {
        panic("execution_slot: invalid state for write_output");
    }
    
    // Buffer is sealed - no more writes allowed
    debug_puts("[EXEC_COMPLETE_OK]");
    slot->state = SLOT_STATE_VERIFYING;
}

// Compute raw output hash
static void compute_raw_output_hash(execution_slot_t *slot) {
    sha256(slot->result_buffer, slot->result_size, slot->raw_output_hash);
}

// Compute context hash (dummy for now)
static void compute_context_hash(execution_slot_t *slot) {
    // Phase-17 Level 1: fixed context hash
    // Phase-17 Level 2: real execution_context_snapshot
    memset(slot->context_hash, 0xAB, 32);
}

// Compute fingerprint
static void compute_fingerprint(execution_slot_t *slot) {
    uint8_t buffer[96];
    
    // fingerprint = SHA256(raw_output_hash || context_hash || bcib_hash)
    memcpy(buffer, slot->raw_output_hash, 32);
    memcpy(buffer + 32, slot->context_hash, 32);
    memset(buffer + 64, 0xCD, 32); // bcib_hash placeholder
    
    sha256(buffer, 96, slot->fingerprint);
}

// Verify execution
int execution_slot_verify(execution_slot_t *slot) {
    if (slot->state != SLOT_STATE_VERIFYING) {
        panic("execution_slot: invalid state for verify");
    }
    
    debug_puts("[VERIFY_START]");
    
    // Compute hashes
    compute_raw_output_hash(slot);
    compute_context_hash(slot);
    compute_fingerprint(slot);
    
    // Validation
    if (slot->result_size == 0) {
        debug_puts("[VERIFY_FAIL]");
        slot->state = SLOT_STATE_FAILED;
        return -1;
    }
    
    debug_puts("[VERIFY_PASS]");
    slot->state = SLOT_STATE_VERIFIED;
    return 0;
}

// Commit (publish to userspace)
void execution_slot_commit(execution_slot_t *slot) {
    if (slot->state != SLOT_STATE_VERIFIED) {
        panic("execution_slot: commit before verify");
    }
    
    // CRITICAL: commit = publish to userspace
    // This is the ONLY valid commit definition
    publish_result_to_userspace(slot);
    
    debug_puts("[RESULT_OK]");
    slot->state = SLOT_STATE_COMMITTED;
}

// Full pipeline
void execution_slot_run(execution_slot_t *slot) {
    execution_slot_execute(slot);
    execution_slot_write_output(slot);
    
    if (execution_slot_verify(slot) != 0) {
        debug_puts("[EXEC_FAILED]");
        return;
    }
    
    execution_slot_commit(slot);
    debug_puts("[WAIT_OK]");
}

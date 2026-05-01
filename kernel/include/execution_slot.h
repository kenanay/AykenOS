#ifndef EXECUTION_SLOT_H
#define EXECUTION_SLOT_H

#include <stdint.h>
#include <stddef.h>

/**
 * Phase-17 Execution Slot
 * 
 * State machine for BCIB execution with inline verification.
 * 
 * CRITICAL RULES:
 * 1. State order is IMMUTABLE
 * 2. No state skipping allowed
 * 3. VERIFIED → COMMITTED is the only valid commit path
 * 4. Commit = publish to userspace (NOT internal state change)
 */

// State machine (IMMUTABLE order)
typedef enum {
    SLOT_STATE_IDLE,
    SLOT_STATE_EXECUTING,
    SLOT_STATE_WRITE_OUTPUT,
    SLOT_STATE_VERIFYING,
    SLOT_STATE_VERIFIED,
    SLOT_STATE_COMMITTED,
    SLOT_STATE_FAILED
} execution_slot_state_t;

// Execution slot structure
typedef struct {
    uint64_t id;
    execution_slot_state_t state;
    
    // Input (BCIB)
    void *bcib_buffer;
    size_t bcib_size;
    
    // Output (kernel-produced)
    uint8_t *result_buffer;
    size_t result_size;
    size_t result_capacity;
    
    // Verification hashes
    uint8_t raw_output_hash[32];    // SHA256 of result_buffer
    uint8_t context_hash[32];       // execution context hash
    uint8_t fingerprint[32];        // combined hash
    
    // Metadata
    uint64_t timestamp_start;
    uint64_t timestamp_end;
} execution_slot_t;

// API
void execution_slot_init(execution_slot_t *slot, uint64_t id);
void execution_slot_destroy(execution_slot_t *slot);
void execution_slot_run(execution_slot_t *slot);

// Internal steps (exposed for testing)
void execution_slot_execute(execution_slot_t *slot);
void execution_slot_write_output(execution_slot_t *slot);
int execution_slot_verify(execution_slot_t *slot);
void execution_slot_commit(execution_slot_t *slot);

#endif // EXECUTION_SLOT_H

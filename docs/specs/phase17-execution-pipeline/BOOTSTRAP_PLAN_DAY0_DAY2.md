# Phase-17 Bootstrap Plan — Day 0 to Day 2

**Authority:** Kenan AY - Architectural Steward  
**Phase:** 17  
**Target:** First Real Execution (Minimal Path)  
**Duration:** 48 hours  
**Status:** READY TO START

---

## Executive Summary

**Goal:** Kernel → BCIB execute → raw_output → verify → userspace

**Critical Rule:**
> This is NOT about AI yet. This is about **execution pipeline bootstrap**.

**What We're Building:**
1. Execution slot state machine
2. Kernel-produced output (fake but deterministic)
3. Inline verification skeleton
4. Commit = publish to userspace
5. Determinism measurement

**What We're NOT Building:**
- Real BCIB interpreter (Phase-17 later)
- AI runtime (Phase-17 later)
- Semantic verification (Phase-18)
- Multi-threading (Phase-17 later)

---

## Day 0: Execution Slot Skeleton

### Step 1: State Machine Definition

**File:** `kernel/include/execution_slot.h`

```c
#ifndef EXECUTION_SLOT_H
#define EXECUTION_SLOT_H

#include <stdint.h>
#include <stddef.h>

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

// Internal steps
void execution_slot_execute(execution_slot_t *slot);
void execution_slot_write_output(execution_slot_t *slot);
int execution_slot_verify(execution_slot_t *slot);
void execution_slot_commit(execution_slot_t *slot);

#endif // EXECUTION_SLOT_H
```

**Critical Rules:**
- State order is IMMUTABLE
- No state skipping allowed
- VERIFIED → COMMITTED is the only valid commit path

---

### Step 2: Execution Slot Implementation

**File:** `kernel/sys/execution_slot.c`

```c
#include "execution_slot.h"
#include <string.h>
#include <kernel/debug.h>
#include <kernel/panic.h>
#include <kernel/crypto/sha256.h>

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
```

**Critical Rules:**
- NO Python stub calls
- NO external verification
- Output MUST be kernel-produced
- Commit MUST be publish to userspace

---

## Day 1: Marker System + Determinism Test

### Step 3: Marker Validation

**File:** `tools/validation/phase17_execution_markers.sh`

```bash
#!/bin/bash
# Phase-17 execution marker validation

BOOT_LOG="$1"

if [ ! -f "$BOOT_LOG" ]; then
    echo "ERROR: boot log not found: $BOOT_LOG"
    exit 1
fi

# Expected marker sequence (IMMUTABLE)
EXPECTED_SEQUENCE=(
    "EXEC_SLOT_INIT"
    "EXEC_START"
    "EXEC_OUTPUT_WRITTEN"
    "EXEC_COMPLETE_OK"
    "VERIFY_START"
    "VERIFY_PASS"
    "RESULT_OK"
    "WAIT_OK"
)

# Extract markers
MARKERS=$(grep -oE '\[(EXEC_|VERIFY_|RESULT_|WAIT_)[A-Z_]+\]' "$BOOT_LOG" | tr -d '[]')

# Validate sequence
INDEX=0
while IFS= read -r marker; do
    EXPECTED="${EXPECTED_SEQUENCE[$INDEX]}"
    
    if [ "$marker" != "$EXPECTED" ]; then
        echo "ERROR: marker sequence violation"
        echo "  Expected: $EXPECTED"
        echo "  Got: $marker"
        echo "  Position: $INDEX"
        exit 1
    fi
    
    INDEX=$((INDEX + 1))
done <<< "$MARKERS"

# Check all markers present
if [ "$INDEX" -ne "${#EXPECTED_SEQUENCE[@]}" ]; then
    echo "ERROR: incomplete marker sequence"
    echo "  Expected: ${#EXPECTED_SEQUENCE[@]} markers"
    echo "  Got: $INDEX markers"
    exit 1
fi

echo "PASS: marker sequence valid"
exit 0
```

**Critical Rule:**
> Marker order change = determinism FAIL

---

### Step 4: Determinism Test

**File:** `tools/validation/phase17_determinism_test.sh`

```bash
#!/bin/bash
# Phase-17 determinism test (run1 == run2)

set -e

QEMU_CMD="$1"
OUTPUT_DIR="$2"

mkdir -p "$OUTPUT_DIR"

# Run 1
echo "=== Run 1 ==="
$QEMU_CMD > "$OUTPUT_DIR/run1.log" 2>&1

# Extract raw output hash from run1
HASH1=$(grep -oP 'raw_output_hash: \K[0-9a-f]+' "$OUTPUT_DIR/run1.log" || echo "")

if [ -z "$HASH1" ]; then
    echo "ERROR: no raw_output_hash in run1"
    exit 1
fi

# Run 2
echo "=== Run 2 ==="
$QEMU_CMD > "$OUTPUT_DIR/run2.log" 2>&1

# Extract raw output hash from run2
HASH2=$(grep -oP 'raw_output_hash: \K[0-9a-f]+' "$OUTPUT_DIR/run2.log" || echo "")

if [ -z "$HASH2" ]; then
    echo "ERROR: no raw_output_hash in run2"
    exit 1
fi

# Compare
if [ "$HASH1" != "$HASH2" ]; then
    echo "ERROR: determinism violation"
    echo "  Run1 hash: $HASH1"
    echo "  Run2 hash: $HASH2"
    exit 1
fi

echo "PASS: determinism verified (run1 == run2)"
echo "  Hash: $HASH1"
exit 0
```

---

## Day 2: CI Gate + Integration

### Step 5: CI Gate Implementation

**File:** `scripts/ci/ci-gate-phase17-execution.sh`

```bash
#!/bin/bash
# CI Gate: Phase-17 Execution Pipeline

set -e

RUN_ID=$(date -u +%Y%m%dT%H%M%SZ)-$(git rev-parse --short HEAD)-$$
EVIDENCE_DIR="out/evidence/run-$RUN_ID/gates/phase17-execution"

mkdir -p "$EVIDENCE_DIR"

echo "== CI GATE PHASE-17 EXECUTION =="
echo "run_id: $RUN_ID"

# Build kernel
make clean
make kernel.elf

# Boot test
BOOT_LOG="$EVIDENCE_DIR/boot.log"
make qemu-test > "$BOOT_LOG" 2>&1 || true

# Marker validation
tools/validation/phase17_execution_markers.sh "$BOOT_LOG"

# Determinism test
tools/validation/phase17_determinism_test.sh "make qemu-test" "$EVIDENCE_DIR"

# Generate report
cat > "$EVIDENCE_DIR/report.json" <<EOF
{
  "gate": "phase17-execution",
  "run_id": "$RUN_ID",
  "result": "PASS",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "commit": "$(git rev-parse HEAD)"
}
EOF

echo "✅ PASS: Phase-17 Execution Gate"
exit 0
```

---

## Critical Rules (NON-OVERRIDABLE)

### 1. State Machine Order
```
IDLE → EXECUTING → WRITE_OUTPUT → VERIFYING → VERIFIED → COMMITTED
```
**No skipping. No reordering.**

### 2. Commit Definition
```c
commit = publish_result_to_userspace()
```
**NOT internal state change. NOT buffer write.**

### 3. Verification Before Commit
```c
if (state != VERIFIED) panic("commit before verify");
```
**No bypass. No skip.**

### 4. Kernel-Produced Output
```c
// ✅ CORRECT
memcpy(slot->result_buffer, kernel_data, len);

// ❌ WRONG
system("python stub.py > output.txt");
```

### 5. Marker Order
```
EXEC_START → EXEC_OUTPUT_WRITTEN → EXEC_COMPLETE_OK → 
VERIFY_START → VERIFY_PASS → RESULT_OK → WAIT_OK
```
**Order change = CI FAIL.**

---

## Common Mistakes (DO NOT DO)

### ❌ Mistake 1: Python Stub Output
```python
# WRONG - this is Phase-16 stub
output = subprocess.check_output(["python", "bcib_stub.py"])
```

**Why Wrong:** Determinism is FAKE. Kernel doesn't produce output.

### ❌ Mistake 2: Commit = State Change
```c
// WRONG
void execution_slot_commit(execution_slot_t *slot) {
    slot->state = SLOT_STATE_COMMITTED; // NOT A COMMIT
}
```

**Why Wrong:** Commit MUST publish to userspace.

### ❌ Mistake 3: Skip Verification
```c
// WRONG
execution_slot_execute(slot);
execution_slot_commit(slot); // SKIP verify
```

**Why Wrong:** Phase-17 requires inline verification.

### ❌ Mistake 4: Multi-Threading Too Early
```c
// WRONG - Phase-17 Level 1 is single-threaded
pthread_create(&thread, NULL, execution_slot_run, slot);
```

**Why Wrong:** Determinism breaks. Add in Phase-17 Level 3.

---

## Success Criteria

### Level 1 (Day 0-2)
- ✅ Execution slot state machine implemented
- ✅ Kernel-produced output (fake but deterministic)
- ✅ Inline verification skeleton
- ✅ Commit = publish to userspace
- ✅ Marker sequence validated
- ✅ Determinism test PASS (run1 == run2)

### Level 2 (Next)
- ⏳ Real BCIB interpreter
- ⏳ execution_context_snapshot enforcement
- ⏳ Real AI model execution
- ⏳ Deterministic AI bootstrap

### Level 3 (Later)
- 🔮 Multi-threading
- 🔮 Performance optimization
- 🔮 Semantic verification (Phase-18)

---

## Next Steps After Day 2

1. **BCIB Interpreter Implementation**
   - Real BCIB parsing
   - Opcode execution
   - Deterministic execution

2. **Execution Context Snapshot**
   - Compile-time enforcement (`_Static_assert`)
   - Runtime enforcement (panic)
   - CI gate validation

3. **AI Runtime Bootstrap**
   - Deterministic model loading
   - Seeded inference
   - AI-specific determinism gates

---

## References

- `docs/specs/phase17-execution-pipeline/PHASE17_PLAN.md`
- `docs/specs/phase17-execution-pipeline/IMPLEMENTATION_RULES.md`
- `docs/specs/phase17-execution-pipeline/MINIMAL_EXECUTION_PATH.md`
- `_ayken/steering/PHASES.md`
- `_ayken/steering/NON_OVERRIDABLE.md`

---

**Prepared by:** Kenan AY - Architectural Steward  
**Date:** 01 May 2026  
**Version:** 1.0  
**Status:** READY TO EXECUTE

**© 2026 Kenan AY - AykenOS Project**

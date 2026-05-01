# Phase-17 Implementation Plan (Naming Convention Compliant)

**Date**: 2026-05-01  
**Authority**: Kenan AY  
**Status**: ACTIVE

---

## Critical Naming Rule

**AykenOS Naming Convention V1** (docs/governance/NAMING_CONVENTION_V1.md):

> Phase/faz labels are planning and governance metadata. They are not stable
> technical identities. Therefore, new code MUST NOT encode phase/faz names
> into stable identifiers.

### ❌ FORBIDDEN Naming
```
execution_slot_phase17.c
execution_slot_phase17.h
phase17_execution_pipeline_probe()
AYKEN_PHASE17_EXECUTION_PIPELINE_ENABLE
```

### ✅ CORRECT Naming (Purpose-Based)
```
execution_marker_validation.c
execution_marker_validation.h
execution_marker_validation_probe()
AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
```

---

## File Structure (Corrected)

### Sandbox Prototype Files (NOT in production build)
```
kernel/sys/execution_marker_validation.c
kernel/include/execution_marker_validation.h
kernel/sys/execution_marker_validation_test.c
kernel/sys/Makefile.execution_marker_validation
```

### Purpose
These files implement **execution marker validation** - the mechanism that ensures
execution state transitions follow the immutable marker order:

```
PREPARE → EXECUTE → WRITE_OUTPUT → VERIFY → COMMIT
```

### Scope
- **NOT** in naming-convention-scope.regex (sandbox files)
- **NOT** automatically included in kernel build
- **NOT** modifying production execution_slot.c yet

---

## Implementation Steps

### Step 1: Create Sandbox Skeleton (NEXT)

**File**: `kernel/include/execution_marker_validation.h`
```c
#ifndef AYKEN_EXECUTION_MARKER_VALIDATION_H
#define AYKEN_EXECUTION_MARKER_VALIDATION_H

/*
 * Execution Marker Validation
 * 
 * Purpose: Validate execution state machine marker order
 * Scope: Sandbox prototype for Phase-17 execution pipeline
 * 
 * This is a SANDBOX implementation. It does NOT modify production
 * execution_slot.c until validation tests pass.
 */

#include <stdint.h>

// Marker definitions (immutable order)
typedef enum {
    EXECUTION_MARKER_PREPARE = 0,
    EXECUTION_MARKER_EXECUTE = 1,
    EXECUTION_MARKER_WRITE_OUTPUT = 2,
    EXECUTION_MARKER_VERIFY = 3,
    EXECUTION_MARKER_COMMIT = 4,
    EXECUTION_MARKER_COUNT = 5
} execution_marker_t;

// Validation result
typedef struct {
    int valid;
    execution_marker_t expected;
    execution_marker_t actual;
    const char *error_message;
} execution_marker_validation_result_t;

// Sandbox probe (for testing only)
void execution_marker_validation_probe(void);

// Validate marker transition
execution_marker_validation_result_t 
execution_marker_validate_transition(
    execution_marker_t current,
    execution_marker_t next
);

#endif // AYKEN_EXECUTION_MARKER_VALIDATION_H
```

**File**: `kernel/sys/execution_marker_validation.c`
```c
#include "execution_marker_validation.h"
#include <kernel/debugcon.h>

/*
 * Execution Marker Validation Implementation
 * 
 * SANDBOX PROTOTYPE - NOT IN PRODUCTION BUILD
 */

// Marker order enforcement table
static const execution_marker_t MARKER_ORDER[EXECUTION_MARKER_COUNT] = {
    EXECUTION_MARKER_PREPARE,
    EXECUTION_MARKER_EXECUTE,
    EXECUTION_MARKER_WRITE_OUTPUT,
    EXECUTION_MARKER_VERIFY,
    EXECUTION_MARKER_COMMIT
};

void execution_marker_validation_probe(void) {
    debugcon_write("[[EXECUTION_MARKER_VALIDATION_PROBE]]\n");
}

execution_marker_validation_result_t 
execution_marker_validate_transition(
    execution_marker_t current,
    execution_marker_t next
) {
    execution_marker_validation_result_t result = {0};
    
    // Check bounds
    if (current >= EXECUTION_MARKER_COUNT || next >= EXECUTION_MARKER_COUNT) {
        result.valid = 0;
        result.error_message = "marker out of bounds";
        return result;
    }
    
    // Check order
    if (next != current + 1) {
        result.valid = 0;
        result.expected = current + 1;
        result.actual = next;
        result.error_message = "marker order violation";
        return result;
    }
    
    result.valid = 1;
    result.error_message = "valid transition";
    return result;
}
```

**File**: `kernel/sys/execution_marker_validation_test.c`
```c
#include "execution_marker_validation.h"
#include <stdio.h>
#include <assert.h>

/*
 * Execution Marker Validation Tests
 * 
 * SANDBOX TEST - NOT IN KERNEL BUILD
 */

void test_valid_transitions(void) {
    execution_marker_validation_result_t result;
    
    // Test valid sequence
    result = execution_marker_validate_transition(
        EXECUTION_MARKER_PREPARE,
        EXECUTION_MARKER_EXECUTE
    );
    assert(result.valid == 1);
    
    result = execution_marker_validate_transition(
        EXECUTION_MARKER_EXECUTE,
        EXECUTION_MARKER_WRITE_OUTPUT
    );
    assert(result.valid == 1);
    
    result = execution_marker_validate_transition(
        EXECUTION_MARKER_WRITE_OUTPUT,
        EXECUTION_MARKER_VERIFY
    );
    assert(result.valid == 1);
    
    result = execution_marker_validate_transition(
        EXECUTION_MARKER_VERIFY,
        EXECUTION_MARKER_COMMIT
    );
    assert(result.valid == 1);
    
    printf("✅ Valid transitions: PASS\n");
}

void test_invalid_transitions(void) {
    execution_marker_validation_result_t result;
    
    // Test skip
    result = execution_marker_validate_transition(
        EXECUTION_MARKER_PREPARE,
        EXECUTION_MARKER_WRITE_OUTPUT
    );
    assert(result.valid == 0);
    
    // Test backward
    result = execution_marker_validate_transition(
        EXECUTION_MARKER_VERIFY,
        EXECUTION_MARKER_EXECUTE
    );
    assert(result.valid == 0);
    
    printf("✅ Invalid transitions: PASS\n");
}

int main(void) {
    printf("=== Execution Marker Validation Tests ===\n");
    test_valid_transitions();
    test_invalid_transitions();
    printf("=== ALL TESTS PASS ===\n");
    return 0;
}
```

**File**: `kernel/sys/Makefile.execution_marker_validation`
```makefile
# Execution Marker Validation - Sandbox Build
# NOT included in production kernel build

CC = gcc
CFLAGS = -Wall -Wextra -Werror -I../../include -g

SRCS = execution_marker_validation.c execution_marker_validation_test.c
OBJS = $(SRCS:.c=.o)
TARGET = execution_marker_validation_test

.PHONY: all clean test

all: $(TARGET)

$(TARGET): $(OBJS)
	$(CC) $(CFLAGS) -o $@ $^

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

test: $(TARGET)
	./$(TARGET)

clean:
	rm -f $(OBJS) $(TARGET)
```

### Step 2: CI Gate for Isolation

**File**: `scripts/ci/ci-gate-execution-marker-isolation.sh`
```bash
#!/bin/bash
# CI Gate: Execution Marker Validation Isolation
# 
# Purpose: Ensure sandbox marker validation code does NOT leak into production

set -e

RUN_ID=$(date -u +%Y%m%dT%H%M%SZ)-$(git rev-parse --short HEAD)-$$
EVIDENCE_DIR="out/evidence/run-$RUN_ID/gates/execution-marker-isolation"

mkdir -p "$EVIDENCE_DIR"

echo "== CI GATE EXECUTION MARKER ISOLATION =="
echo "run_id: $RUN_ID"

VIOLATIONS=()

# Check: execution_marker_validation.c NOT in kernel build
if grep -q "execution_marker_validation\.c" kernel/sys/Makefile 2>/dev/null; then
    VIOLATIONS+=("execution_marker_validation.c found in kernel/sys/Makefile")
fi

# Check: execution_slot.c does NOT call marker validation yet
if grep -q "execution_marker_validate_transition" kernel/sys/execution_slot.c 2>/dev/null; then
    VIOLATIONS+=("execution_marker_validate_transition called in production execution_slot.c")
fi

# Check: no phase17 naming in new files
if git diff --name-only HEAD~1 2>/dev/null | grep -i "phase17" | grep -v "docs/specs/phase17"; then
    VIOLATIONS+=("phase17 naming found in non-documentation files")
fi

# Generate report
VIOLATIONS_COUNT=${#VIOLATIONS[@]}

if [ "$VIOLATIONS_COUNT" -eq 0 ]; then
    VERDICT="PASS"
else
    VERDICT="FAIL"
fi

cat > "$EVIDENCE_DIR/report.json" <<EOF
{
  "gate": "execution-marker-isolation",
  "verdict": "$VERDICT",
  "run_id": "$RUN_ID",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "commit": "$(git rev-parse HEAD)",
  "violations": [$(printf '"%s",' "${VIOLATIONS[@]}" | sed 's/,$//')]
  "violations_count": $VIOLATIONS_COUNT,
  "meta": {
    "purpose": "prevent_sandbox_code_leaking_to_production"
  }
}
EOF

echo ""
if [ "$VERDICT" = "PASS" ]; then
    echo "✅ PASS: Execution Marker Isolation Gate"
else
    echo "❌ FAIL: Execution Marker Isolation Gate"
    echo "Violations:"
    printf '  - %s\n' "${VIOLATIONS[@]}"
    exit 1
fi

echo "Evidence: $EVIDENCE_DIR/report.json"
exit 0
```

### Step 3: Integration Plan (LATER)

**Only after sandbox tests PASS**:

1. Add feature flag to production code:
   ```c
   #ifdef AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
   #include "execution_marker_validation.h"
   #endif
   ```

2. Add optional validation calls:
   ```c
   #ifdef AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE
       execution_marker_validation_result_t result = 
           execution_marker_validate_transition(current_marker, next_marker);
       if (!result.valid) {
           panic("marker order violation");
       }
   #endif
   ```

3. Add to Makefile with flag guard

---

## Commit Sequence

```
✅ Commit 1: fix: Correct execution_slot integrity gate verdict field (DONE)
⏳ Commit 2: feat: Add execution marker validation sandbox
⏳ Commit 3: feat: Add execution marker isolation gate
⏳ Commit 4: test: Verify execution marker validation in sandbox
⏳ Commit 5: feat: Add optional execution marker validation integration (feature flag)
```

---

## Key Principles

1. **Purpose-based naming**: `execution_marker_validation` NOT `phase17`
2. **Isolation first**: Sandbox MUST NOT leak to production
3. **Feature flag integration**: Optional, guarded, testable
4. **CI enforcement**: Gates prevent accidental integration

---

**Status**: Ready for sandbox implementation  
**Next**: Create execution_marker_validation sandbox files  
**Authority**: Kenan AY

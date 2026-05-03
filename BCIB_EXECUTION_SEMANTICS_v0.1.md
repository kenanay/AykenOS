# BCIB Execution Semantics v0.1

**Authority**: Kenan AY - Architectural Steward  
**Status**: LOCKED DRAFT  
**Version**: 0.1  
**Date**: 2026-05-03  
**Scope**: BCIB runtime execution contract

---

## 1. Fundamental Decision

**BCIB instruction execution means:**

```
fetch → decode → validate → execute → commit
```

**No instruction can directly mutate global state.** All mutations become effective only at the commit phase.

---

## 2. Instruction Lifecycle

### 2.1 Fetch

Runtime retrieves the next instruction from the instruction table.

**Rules:**
- `program_counter` must be within valid range
- `instruction offset` must be valid
- Cannot exceed `instruction_count`

**Violation:**
```
BCIB_ERR_INVALID_PC → FAIL_CLOSED
```

---

### 2.2 Decode

Runtime decodes `opcode`, `flags`, `arg_start`, `arg_count` fields.

**Rules:**
- Opcode must be known
- `arg_start + arg_count` must not exceed bounds
- Argument type must be compatible with opcode

**Violation:**
```
BCIB_ERR_DECODE → FAIL_CLOSED
```

---

### 2.3 Validate

Every instruction is validated before execution.

**Validation checks:**
- Is context valid?
- Are capabilities sufficient?
- Is ABDF validated?
- Is GPU/UI target ready?
- Is instruction permitted in current phase?

**Violation:**
```
BCIB_ERR_VALIDATION → FAIL_CLOSED
```

---

### 2.4 Execute

Instruction produces temporary execution state.

**Rule:** Execute phase CANNOT mutate persistent state.

**Examples:**
- `OP_DATA_QUERY` → temporary result buffer
- `OP_GPU_BUFFER_CREATE` → pending GPU resource
- `OP_UI_RENDER` → pending frame command

---

### 2.5 Commit

Persistent changes are made ONLY at commit phase.

**If commit fails:**
- Partial state is FORBIDDEN
- Rollback is MANDATORY
- FAIL_CLOSED

---

## 3. Context Model

### 3.1 Context Definition

**Context is an isolated execution space.**

```c
struct BcibContext {
    uint64_t context_id;
    uint64_t owner_id;
    uint32_t context_type;
    uint32_t state;
    uint64_t parent_context;
    uint64_t capability_mask;
};
```

---

### 3.2 Context Types

- `DATA_CONTEXT`
- `SYSTEM_CONTEXT`
- `UI_CONTEXT`
- `GPU_CONTEXT`
- `AI_CONTEXT`

---

### 3.3 Isolation Rule

**One context CANNOT directly mutate another context's state.**

**Permitted communication:**
- Explicit binding
- Capability token
- Runtime-mediated bridge

**Forbidden:**
- Direct pointer sharing
- Implicit global state
- Cross-context mutation

---

### 3.4 Context Stack

- `OP_CTX_PUSH` pushes active context onto stack
- `OP_CTX_POP` returns to previous context

**Rules:**
- Stack overflow → `FAIL_CLOSED`
- Stack underflow → `FAIL_CLOSED`

---

### 3.5 Lifetime

**Context states:**
1. `CREATED`
2. `ACTIVE`
3. `SUSPENDED`
4. `COMMITTED`
5. `FAILED`
6. `CLOSED`

**A `FAILED` context cannot be re-executed.**

---

## 4. Memory Ownership Model

### 4.1 Ownership Decision

| Component | Role |
|-----------|------|
| **ABDF** | Immutable source |
| **Runtime** | Execution memory owner |
| **GPU** | Borrowed execution target |
| **UI** | Read-only consumer |
| **AI** | Planner-only consumer |

---

### 4.2 ABDF Rule

**ABDF cannot be modified in-place by any instruction.**

```
update = new snapshot
```

---

### 4.3 Runtime Buffers

Runtime can create temporary buffers.

**Rules:**
- Owner is runtime
- Lifetime is bounded by context
- After commit: either released or converted to snapshot

---

### 4.4 GPU Borrowing

**GPU buffer ownership remains with runtime.**

GPU only borrows.

**Rules:**
- Buffer must be sealed before `GPU_DISPATCH`
- Completion fence must be awaited after `GPU_DISPATCH`

**GPU memory failure:**
```
BCIB_ERR_GPU_FAULT → FAIL_CLOSED
```

---

### 4.5 UI Consumer

**UI can read ABDF or runtime results but CANNOT modify them.**

```
UI_RENDER = read-only render command
```

---

## 5. Error Model

### 5.1 Core Principle

**Silent errors are FORBIDDEN.**

```
No silent failure.
No partial commit.
No undefined state.
```

---

### 5.2 Error Classes

- `BCIB_ERR_INVALID_HEADER`
- `BCIB_ERR_INVALID_PC`
- `BCIB_ERR_DECODE`
- `BCIB_ERR_VALIDATION`
- `BCIB_ERR_CAPABILITY`
- `BCIB_ERR_CONTEXT`
- `BCIB_ERR_MEMORY`
- `BCIB_ERR_GPU_FAULT`
- `BCIB_ERR_UI_FAULT`
- `BCIB_ERR_AI_POLICY`
- `BCIB_ERR_COMMIT`

---

### 5.3 Fail-Closed Rule

**Runtime immediately shuts down on:**
- Invalid opcode
- Invalid argument
- Capability violation
- Context violation
- Memory violation
- GPU fault
- Checksum mismatch
- Unknown mandatory feature

---

### 5.4 Error Propagation

Every error produces:

```c
struct BcibError {
    uint64_t context_id;
    uint64_t instruction_index;
    uint32_t error_code;
    uint32_t severity;
};
```

**Severity levels:**
- `WARN`
- `RECOVERABLE`
- `FATAL`

**`FATAL` always triggers fail-closed.**

---

## 6. Sync / Async Rules

### 6.1 Default Rule

**All instructions are blocking by default.**

Execution does not proceed to next instruction until current instruction completes.

---

### 6.2 Async Only With Flag

Async instructions work ONLY with explicit flag:

```
BCIB_FLAG_ASYNC
```

**Opcodes supporting async:**
- `OP_GPU_DISPATCH`
- `OP_GPU_COPY`
- `OP_AI_PLAN`
- `OP_AI_OPTIMIZE`

---

### 6.3 Dependency Rule

Async instructions produce dependency tokens.

```c
struct BcibDependency {
    uint64_t dep_id;
    uint32_t dep_type;
    uint32_t state;
};
```

**States:**
- `PENDING`
- `READY`
- `FAILED`
- `CANCELLED`

**Dependent instructions cannot execute until dependency is `READY`.**

---

### 6.4 GPU Dispatch Rule

**GPU dispatch model:**

1. Validate buffer
2. Seal buffer
3. Dispatch
4. Create fence
5. Wait or continue async
6. Commit completion state

**If GPU fence is `FAILED`:**
```
BCIB_ERR_GPU_FAULT → FAIL_CLOSED
```

---

### 6.5 UI Render Rule

**UI render MUST be blocking.**

**Reason:**
- Frame determinism
- Visual state consistency

**Async UI render is FORBIDDEN in v0.1.**

---

## 7. Determinism Rules

### 7.1 Instruction Order

**Same BCIB + Same ABDF + Same runtime config:**

```
→ MUST produce same execution trace
```

---

### 7.2 Forbidden Sources

**The following CANNOT directly influence instruction results:**
- Wall clock
- Randomness (unseeded)
- Unsorted map iteration
- Thread race
- GPU nondeterministic reduction

---

### 7.3 Deterministic Trace

Every execution produces this trace:

```
instruction_index
opcode
context_id
input_hash
output_hash
error_code
```

---

## 8. Commit Contract

**Commit MUST be atomic.**

```
all-or-nothing
```

**If commit fails:**
- Rollback
- Context → `FAILED`
- Runtime → `FAIL_CLOSED`

---

## 9. Minimal Runtime Loop

```c
while (pc < instruction_count) {
    inst = fetch(pc);
    decoded = decode(inst);
    validate(decoded, context);
    pending = execute(decoded, context);
    commit(pending, context);
    pc++;
}
```

**Exiting this loop is only possible via:**
- Normal completion
- Fatal error
- Explicit halt

---

## 10. Closing Decision

**With this spec, BCIB is no longer just an enum list.**

**Now:**

| Component | Role |
|-----------|------|
| **BCIB** | Deterministic execution contract |
| **Runtime** | Authority |
| **Context** | Isolation boundary |
| **Commit** | Only mutation point |
| **Error** | Fail-closed signal |
| **GPU** | Borrowed async target |
| **UI** | Deterministic read-only render consumer |

---

## Clear Decision

**BCIB Runtime implementation MUST NOT begin without conforming to this document.**

---

## Relationship to Other Specs

- **ABDF v0.1** - Data format and memory contract
- **BCIB Opcode Set v0.1** - Instruction definitions
- **This document** - Execution semantics and runtime contract

---

## Constitutional Compliance

This specification is subject to:

- `NON_OVERRIDABLE.md` - No silent failures, no undefined state
- `PHASES.md` - Phase-based execution rules
- Memory safety rules (no leak, no double-free, no undefined behavior)

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 0.1 | 2026-05-03 | Initial locked draft |

---

**End of BCIB Execution Semantics v0.1**

Source: `BCIB_EXECUTION_SEMANTICS_v0.1.md`

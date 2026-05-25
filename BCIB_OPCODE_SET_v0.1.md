# BCIB OPCODE SET v0.1

**Authority**: Kenan AY - Architectural Steward  
**Status**: LOCKED - Constitutional Core Document  
**Version**: 0.1  
**Date**: 2026-05-03

---

## Purpose

BCIB (Binary Command Instruction Block) is the binary instruction format for AykenOS Semantic CLI commands.

### Execution Flow

```
CLI / DSL → BCIB → Runtime → CPU/GPU/UI/AI executor
```

### Core Principles

1. **CLI is NOT human language.** CLI is a BCIB producer.
2. **Runtime does NOT execute string commands.** Runtime executes BCIB instructions.
3. **ABDF first, BCIB second.** Data format precedes instruction format.

This ordering was confirmed in the previous roadmap as the correct architectural sequence.

---

## Header Structure

```c
struct BcibHeader {
    uint32_t magic;              // "BCIB" (0x42494342)
    uint16_t major_version;      // 0
    uint16_t minor_version;      // 1
    uint32_t flags;
    uint32_t instruction_count;
    uint32_t argument_count;
    uint64_t instruction_table_offset;
    uint64_t argument_table_offset;
    uint64_t string_pool_offset;
    uint64_t object_pool_offset;
    uint64_t checksum;
};
```

**Size**: 64 bytes (aligned)

---

## Instruction Structure

```c
struct BcibInstruction {
    uint16_t opcode;
    uint16_t flags;
    uint32_t arg_start;
    uint32_t arg_count;
    uint32_t reserved;
};
```

**Size**: 16 bytes

---

## Argument Structure

```c
struct BcibArg {
    uint32_t type;
    uint32_t flags;
    uint64_t value;
    uint64_t size;
};
```

**Size**: 24 bytes

---

## Opcode Set v0.1

### A) Context Opcodes

| Opcode | Name | Purpose |
|--------|------|---------|
| `0x0001` | `OP_CTX_SELECT` | Select context (e.g., `data.users`, `sys.hw`) |
| `0x0002` | `OP_CTX_PUSH` | Push context onto stack |
| `0x0003` | `OP_CTX_POP` | Pop context from stack |

**Purpose**: Navigate contexts like:
- `data.users`
- `sys.hw`
- `ui.scene.main`
- `gpu.pipeline.main`

---

### B) Data Opcodes

| Opcode | Name | Purpose |
|--------|------|---------|
| `0x0100` | `OP_DATA_CREATE` | Create data structure |
| `0x0101` | `OP_DATA_INSERT` | Insert data |
| `0x0102` | `OP_DATA_QUERY` | Query data |
| `0x0103` | `OP_DATA_DELETE` | Delete data |
| `0x0104` | `OP_DATA_BIND` | Bind data to context |

**Sufficient for Phase 1.**

---

### C) ABDF Opcodes

| Opcode | Name | Purpose |
|--------|------|---------|
| `0x0200` | `OP_ABDF_OPEN` | Open ABDF file |
| `0x0201` | `OP_ABDF_VALIDATE` | Validate ABDF structure |
| `0x0202` | `OP_ABDF_SEGMENT_SELECT` | Select segment |
| `0x0203` | `OP_ABDF_READ` | Read segment data |
| `0x0204` | `OP_ABDF_SNAPSHOT` | Create snapshot |

**ABDF validation is the runtime's security gate.**

---

### D) GPU Opcodes

| Opcode | Name | Purpose |
|--------|------|---------|
| `0x0300` | `OP_GPU_BUFFER_CREATE` | Create GPU buffer |
| `0x0301` | `OP_GPU_BUFFER_BIND` | Bind GPU buffer |
| `0x0302` | `OP_GPU_COPY` | Copy to GPU |
| `0x0303` | `OP_GPU_DISPATCH` | Dispatch GPU work |
| `0x0304` | `OP_GPU_RELEASE` | Release GPU resource |

**Phase 1 Note**: Real GPU driver is not required. But the opcode contract MUST exist now.

---

### E) UI Opcodes

| Opcode | Name | Purpose |
|--------|------|---------|
| `0x0400` | `OP_UI_SCENE_CREATE` | Create UI scene |
| `0x0401` | `OP_UI_NODE_CREATE` | Create UI node |
| `0x0402` | `OP_UI_BIND_DATA` | Bind data to UI |
| `0x0403` | `OP_UI_RENDER` | Render UI scene |
| `0x0404` | `OP_UI_UPDATE` | Update UI state |

**UI is NOT direct drawing. UI is scene graph execution.**

---

### F) AI Opcodes

| Opcode | Name | Purpose |
|--------|------|---------|
| `0x0500` | `OP_AI_SUMMARIZE` | Summarize data |
| `0x0501` | `OP_AI_EXPLAIN` | Explain pattern |
| `0x0502` | `OP_AI_PLAN` | Generate plan |
| `0x0503` | `OP_AI_OPTIMIZE` | Optimize structure |

**Clear Rule**: AI does NOT make decisions. AI proposes plans. Runtime executes.

---

### G) System Opcodes

| Opcode | Name | Purpose |
|--------|------|---------|
| `0x0600` | `OP_SYS_HW_STATUS` | Hardware status |
| `0x0601` | `OP_SYS_PROC_STATUS` | Process status |
| `0x0602` | `OP_SYS_TELEMETRY_READ` | Read telemetry |

Aligns with Semantic CLI's `sys.hw`, `sys.proc` namespace structure.

---

### H) Control Opcodes

| Opcode | Name | Purpose |
|--------|------|---------|
| `0x0700` | `OP_BRANCH_BEGIN` | Begin branch |
| `0x0701` | `OP_BRANCH_END` | End branch |
| `0x0702` | `OP_PIPE_BEGIN` | Begin pipe |
| `0x0703` | `OP_PIPE_END` | End pipe |
| `0x0704` | `OP_NOOP` | No operation |

---

## Minimal v0.1 Required Opcode List

**Phase 1 MUST implement these opcodes:**

1. `OP_CTX_SELECT`
2. `OP_DATA_CREATE`
3. `OP_DATA_INSERT`
4. `OP_DATA_QUERY`
5. `OP_ABDF_VALIDATE`
6. `OP_ABDF_READ`
7. `OP_GPU_BUFFER_CREATE`
8. `OP_GPU_BUFFER_BIND`
9. `OP_UI_SCENE_CREATE`
10. `OP_UI_RENDER`
11. `OP_SYS_HW_STATUS`

**Total**: 11 opcodes for Phase 1

---

## Argument Types

| Type | Code | Description |
|------|------|-------------|
| `ARG_NONE` | `0x00` | No argument |
| `ARG_U64` | `0x01` | Unsigned 64-bit integer |
| `ARG_I64` | `0x02` | Signed 64-bit integer |
| `ARG_F64` | `0x03` | IEEE754 double |
| `ARG_STRING` | `0x04` | String pool offset |
| `ARG_OBJECT` | `0x05` | Object pool offset |
| `ARG_ABDF` | `0x06` | ABDF file reference |
| `ARG_CONTEXT` | `0x07` | Context path |

---

## Execution Model

### Sequential Execution

```
instruction[0] → instruction[1] → instruction[2] → ...
```

### Branching

```
OP_BRANCH_BEGIN
  instruction[a]
  instruction[b]
OP_BRANCH_END
```

### Piping

```
OP_PIPE_BEGIN
  OP_DATA_QUERY → OP_UI_RENDER
OP_PIPE_END
```

---

## Validation Rules

A BCIB reader MUST reject a block if:

1. `magic != "BCIB"`
2. `instruction_count == 0`
3. Opcode is unknown and not marked as optional
4. Argument type mismatch
5. Argument count exceeds limit
6. Checksum mismatch
7. Instruction references invalid offset

---

## Relationship to ABDF

- **ABDF** = Data and memory contract
- **BCIB** = Execution and instruction contract

**BCIB instructions consume ABDF files.** The runtime validates ABDF before executing BCIB.

---

## Constitutional Compliance

This specification is subject to:

- `NON_OVERRIDABLE.md` - No kernel policy decisions in instructions
- `PHASES.md` - Phase-based opcode availability
- Security boundary rules (Ring3 cannot execute Ring0 opcodes)

---

## Closing Decision

**Clear Decision:**

| Component | Role |
|-----------|------|
| **ABDF** | Data and memory contract |
| **BCIB** | Execution contract |
| **Runtime** | Authority |
| **CLI** | Compiler |
| **AI** | Planner |
| **GPU** | Execution target |
| **UI** | Render graph |

These two specifications lock the Phase 1 foundation of AykenOS in the correct direction.

---

## File Extension

`.bcib`

---

## MIME Type

`application/x-bcib`

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 0.1 | 2026-05-03 | Initial locked specification |

---

**End of BCIB Opcode Set v0.1**

Source: `BCIB_OPCODE_SET_v0.1.md`

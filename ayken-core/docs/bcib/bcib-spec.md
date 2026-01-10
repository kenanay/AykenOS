# BCIB Format Specification v0.2

*(Binary CLI Instruction Buffer)*  
**Status:** Draft (Faz 2)  
**Author:** AykenOS Project  
**License:** Open Spec – Source Required Attribution  

## 1. Overview

BCIB (Binary CLI Instruction Buffer) is a binary format for representing CLI commands and data operations in AykenOS. It provides a structured, versioned approach to command execution with support for:

- Data-oriented operations
- Context management
- UI rendering commands
- AI integration
- System operations

## 2. Binary Layout

```
+---------------------+
| BCIB HEADER         |  16 bytes
+---------------------+
| INSTRUCTION ARRAY   |  variable size
+---------------------+
| STRING POOL         |  variable size (optional)
+---------------------+
```

## 3. Header Format

```c
struct BcibHeader {
    uint8  magic[4];              // "BCIB"
    uint16 version;               // Format version (2 = v0.2)
    uint16 instruction_count;     // Number of instructions
    uint32 string_pool_offset;    // Offset to string pool (0 = none)
    uint32 reserved;              // Future use
};
```

## 4. Instruction Format

Each instruction is 8 bytes:

```c
struct BcibInstruction {
    uint8  opcode;     // Operation code
    uint8  flags;      // Operation flags
    uint16 arg0;       // Argument 1
    uint16 arg1;       // Argument 2
    uint16 arg2;       // Argument 3
};
```

## 5. Opcodes

### Context Management
- `0x01` - **CtxSelect**: Select active data container

### Data Operations
- `0x10` - **DataCreate**: Create new data container
- `0x11` - **DataAdd**: Add data to selected container
- `0x12` - **DataQuery**: Query data from container
- `0x13` - **DataUpdate**: Update data in container
- `0x14` - **DataDelete**: Delete data from container

### UI Operations
- `0x20` - **UiRender**: Render UI scene
- `0x21` - **UiEvent**: Send UI event

### AI Operations
- `0x30` - **AiAsk**: Query AI module

### System Operations
- `0x40` - **SysInfo**: Query system information
- `0xFF` - **End**: Terminate execution

## 6. String Pool

Optional section containing null-terminated UTF-8 strings referenced by instruction arguments.

Format:
```
"string1\0string2\0string3\0"
```

## 7. Usage Examples

### Simple Data Query
```
Header: magic="BCIB", version=2, count=3
Instructions:
  CtxSelect(container_id=0)
  DataQuery(query_string_idx=0)
  End()
String Pool: "SELECT * FROM users\0"
```

### UI Rendering
```
Header: magic="BCIB", version=2, count=2
Instructions:
  UiRender(scene_idx=0)
  End()
```

## 8. Integration with ABDF

BCIB instructions often reference ABDF segments:
- `arg0` may contain ABDF segment index
- `arg1` may contain ABDF meta index
- String pool may reference ABDF string pool

## 9. Version History

- v0.1: Initial format
- v0.2: Added AI operations, improved data ops

## 10. Future Extensions

- Conditional execution
- Loop constructs
- Error handling
- Async operations
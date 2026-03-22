# BCIB Executor Architecture Documentation

## Overview

The BCIB (Binary CLI Instruction Buffer) Executor is a Ring3 runtime component responsible for validating, managing, and submitting BCIB graphs for execution. This architecture implements the execution-centric paradigm required for AykenOS Phase 2.3.

## Core Components

### 1. BcibExecutor

The main executor struct that manages execution contexts and capabilities:

```rust
pub struct BcibExecutor {
    pub execution_contexts: HashMap<u64, ExecutionContext>,
    pub capability_manager: CapabilityManager,
}
```

**Responsibilities:**
- Validate BCIB graphs before submission
- Manage execution contexts throughout their lifecycle
- Interface with Ring0 via v2 syscalls (1000-1009 range)
- Coordinate capability token management

### 2. ExecutionContext

Tracks the state of individual BCIB graph executions:

```rust
pub struct ExecutionContext {
    pub id: u64,
    pub active_container: Option<String>,
    pub string_pool: Vec<String>,
    pub logger_enabled: bool,
}
```

**Features:**
- Unique execution tracking
- Container binding support
- String pool for efficient memory management
- Configurable logging per execution

### 3. CapabilityManager

Implements capability-based security for execution submissions:

```rust
pub struct CapabilityManager {
    active_tokens: HashSet<u64>,
}
```

**Security Model:**
- Token-based access control
- Bind/revoke capability lifecycle
- Prevents unauthorized execution submissions
- Integrates with Ring0 capability system

### 4. BcibGraph

Wrapper for BCIB binary data with validation:

```rust
pub struct BcibGraph<'a> {
    data: &'a [u8],
}
```

**Validation Features:**
- BCIB header verification (magic, version)
- Opcode validation against known instruction set
- Buffer integrity checks
- Zero-copy design for performance

## Execution Flow

### 1. Graph Submission

```rust
pub fn submit_execution(&mut self, graph: &BcibGraph, context_id: u64) -> Result<u64, ExecutionError>
```

**Process:**
1. Validate graph is non-empty
2. Perform BCIB structure validation
3. Validate caller-supplied target `context_id`
4. Submit to Ring0 via SYS_V2_SUBMIT_EXECUTION using that `context_id`
5. Bind a capability token to the authoritative kernel-returned `execution_id`
6. Ensure the userspace execution context entry exists for the target context
7. Return the kernel-owned execution ID or error

### 2. Result Waiting

```rust
pub fn wait_result(&self, execution_id: u64, timeout_ms: u64) -> Result<u64, ExecutionError>
```

**Process:**
1. Call Ring0 via SYS_V2_WAIT_RESULT
2. Handle timeout and error conditions
3. Return execution result

## Integration with AykenOS

### Ring0 Interface

Uses the execution-centric syscall interface (v2):
- **SYS_V2_SUBMIT_EXECUTION (1003)**: Submit BCIB graph for execution
- **SYS_V2_WAIT_RESULT (1004)**: Wait for execution completion

### BCIB Instruction Set

Supports the complete BCIB v0.2 instruction set:
- **Data Operations**: DataCreate, DataAdd, DataQuery
- **UI Operations**: UiRender
- **AI Operations**: AiAsk
- **Control Flow**: Nop, End

### Error Handling

Comprehensive error types:
- **InvalidGraph**: BCIB validation failures
- **Decode**: BCIB parsing errors
- **Syscall**: Ring0 syscall failures
- **Capability**: Security token errors

## Security Architecture

### Capability-Based Access Control

1. **Token Generation**: Each kernel-returned execution gets a unique capability token
2. **Permission Validation**: Tokens specify resource access permissions
3. **Lifecycle Management**: Tokens are revoked on execution completion or failure
4. **Ring0 Integration**: Capability system enforced at kernel level

### Validation Pipeline

1. **Structure Validation**: BCIB header and layout verification
2. **Opcode Validation**: All instructions must be valid opcodes
3. **Security Checks**: Capability tokens must be valid
4. **Resource Limits**: Execution contexts are bounded

## Performance Characteristics

### Memory Management
- Zero-copy BCIB graph handling
- Efficient execution context pooling
- Minimal allocation during hot paths

### Syscall Optimization
- Direct assembly syscall interface
- Minimal parameter marshaling
- Efficient error propagation

## Requirements Compliance

✅ **Ring3 BCIB execution engine**: Implemented in userspace  
✅ **Execution context management**: HashMap-based context tracking  
✅ **Capability management**: Token-based security system  
✅ **BCIB validation**: Complete validation pipeline  
✅ **v2 syscall integration**: Uses 1000-1009 syscall range  
✅ **Error handling**: Comprehensive error types and propagation  

## Future Extensions

The architecture supports future enhancements:
- **Parallel Execution**: Multiple concurrent BCIB graphs
- **Resource Quotas**: Per-execution resource limits
- **Execution Profiling**: Performance monitoring and metrics
- **Graph Optimization**: BCIB instruction optimization passes

---

**Status**: ✅ Architecture Complete - Ready for Phase 2.3 Integration  
**Author**: AykenOS Development Team  
**Date**: January 2026

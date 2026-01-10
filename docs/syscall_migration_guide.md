# AykenOS Syscall Migration Guide
## Phase 2.1 - Transition from POSIX-like to Execution-Centric Syscalls

**Author:** Kenan AY  
**Project:** AykenOS - Advanced AI-Integrated Operating System  
**Created:** January 9, 2026

## Overview

This guide documents the migration path from legacy POSIX-like (v1) syscalls to the new execution-centric (v2) syscalls during AykenOS Phase 2 architectural transformation.

## Syscall Numbering Plan

### Legacy POSIX-like Syscalls (v1) - Range 0-99
- **Status:** Backward compatibility during transition period
- **Removal:** Phase 2.5 (complete removal)
- **Usage:** Existing applications only

| Number | Name | Description | Status |
|--------|------|-------------|---------|
| 0 | sys_read | Read from file descriptor | ✅ Working |
| 1 | sys_write | Write to file descriptor | ✅ Working |
| 2 | sys_open | Open file | ✅ Working |
| 3 | sys_close | Close file | ✅ Working |
| 60 | sys_exit | Process termination | ✅ Working |

### Execution-Centric Syscalls (v2) - Range 1000-1009
- **Status:** Active development and implementation
- **Philosophy:** Mechanism-only, capability-based, execution-focused
- **Usage:** All new applications and Ring3 runtime components

| Number | Name | Description | Status |
|--------|------|-------------|---------|
| 1000 | sys_v2_map_memory | Memory mapping mechanism | ✅ Implemented |
| 1001 | sys_v2_unmap_memory | Memory unmapping mechanism | ✅ Implemented |
| 1002 | sys_v2_switch_context | Context switching mechanism | ✅ Implemented |
| 1003 | sys_v2_submit_execution | BCIB execution submission | ✅ Implemented |
| 1004 | sys_v2_wait_result | Execution result waiting | ✅ Implemented |
| 1005 | sys_v2_interrupt_return | Interrupt handling return | ✅ Implemented |
| 1006 | sys_v2_time_query | Time query mechanism | ✅ Implemented |
| 1007 | sys_v2_capability_bind | Capability token binding | ✅ Implemented |
| 1008 | sys_v2_capability_revoke | Capability token revocation | ✅ Implemented |
| 1009 | sys_v2_exit | Process termination | ✅ Implemented |

## Migration Examples

### Example 1: File Operations Migration

**Legacy v1 Approach:**
```c
// Old POSIX-like file operations
int fd = syscall(2, "/path/to/file", 0);  // sys_open
char buffer[1024];
int bytes_read = syscall(0, fd, buffer, 1024);  // sys_read
syscall(3, fd);  // sys_close
```

**New v2 Approach:**
```c
// New execution-centric approach via Ring3 VFS
capability_token_t file_cap = request_file_capability("/path/to/file", CAP_PERM_READ);
syscall(1007, execution_ctx, &file_cap);  // sys_v2_capability_bind

// Map file into memory
uint64_t virt_addr = 0x10000000;
syscall(1000, virt_addr, file_phys_addr, MAP_READ_ONLY);  // sys_v2_map_memory

// Access file data directly through memory mapping
char *file_data = (char *)virt_addr;
// Process data...

syscall(1001, virt_addr, file_size);  // sys_v2_unmap_memory
syscall(1008, file_cap.id);  // sys_v2_capability_revoke
```

### Example 2: Process Management Migration

**Legacy v1 Approach:**
```c
// Old process termination
syscall(60, exit_code);  // sys_exit
```

**New v2 Approach:**
```c
// New execution-centric termination
syscall(1009, exit_code);  // sys_v2_exit
```

### Example 3: BCIB Execution (New Capability)

**v2 Only - No v1 Equivalent:**
```c
// Submit BCIB graph for execution
bcib_graph_t *graph = create_bcib_graph();
uint64_t execution_id = syscall(1003, graph, graph_size, context_id);  // sys_v2_submit_execution

// Wait for execution completion
uint64_t result = syscall(1004, execution_id, timeout_ms);  // sys_v2_wait_result
```

## Developer Guidelines

### Phase 2.1-2.4: Dual Interface Period
- **Existing applications:** Continue using v1 syscalls (0-99 range)
- **New applications:** Use v2 syscalls (1000-1009 range)
- **Ring3 runtime components:** Must use v2 syscalls exclusively

### Phase 2.5: Legacy Cleanup
- **All v1 syscalls removed:** Applications must migrate to v2
- **Ring0 contains exactly 10 syscalls:** No more, no less
- **Capability-based access:** All resource access via capability tokens

## Key Architectural Differences

### Philosophy Shift
| Aspect | v1 (POSIX-like) | v2 (Execution-Centric) |
|--------|-----------------|-------------------------|
| **Focus** | File/Process operations | Execution contexts and capabilities |
| **Security** | Traditional permissions | Capability-based tokens |
| **Policy** | Mixed Ring0/Ring3 | Ring3 only |
| **Mechanism** | Mixed Ring0/Ring3 | Ring0 only |
| **Resource Access** | Direct syscalls | Capability tokens + memory mapping |

### Capability System Integration
All v2 syscalls integrate with the capability system:
- **Resource access** requires capability tokens
- **Fine-grained permissions** via capability flags
- **Secure token binding** to execution contexts
- **Automatic revocation** for security

## Testing and Validation

### Hybrid Dispatcher Testing
```c
// Test v1 syscall routing
uint64_t result_v1 = syscall(1, 1, "Hello v1\n", 10, 0);  // Should route to v1 handler

// Test v2 syscall routing  
uint64_t time_result = 0;
uint64_t result_v2 = syscall(1006, 0, &time_result, 0, 0);  // Should route to v2 handler

// Test invalid syscall
uint64_t result_invalid = syscall(500, 0, 0, 0, 0);  // Should return -ENOSYS
```

### Capability System Testing
```c
// Create and bind capability
capability_token_t token = capability_create(CAPABILITY_RESOURCE_MEMORY, 
                                           CAPABILITY_PERM_READ_WRITE,
                                           0x1000000, 4096);
int bind_result = syscall(1007, execution_ctx, &token);  // sys_v2_capability_bind

// Revoke capability
int revoke_result = syscall(1008, token.id);  // sys_v2_capability_revoke
```

## Implementation Status

### ✅ Completed (Phase 2.1)
- [x] Execution-centric syscall interface (10 syscalls exactly)
- [x] Capability token system with full lifecycle management
- [x] Hybrid syscall dispatcher with clear numbering plan
- [x] Backward compatibility for existing v1 applications
- [x] Comprehensive testing framework

### 🚧 In Progress (Phase 2.2-2.4)
- [ ] Ring3 VFS library implementation
- [ ] Ring3 scheduler policy migration
- [ ] Ring3 DevFS proxy implementation
- [ ] BCIB execution engine in Ring3
- [ ] AI runtime migration to Ring3

### ⏳ Planned (Phase 2.5)
- [ ] Complete removal of v1 syscalls (0-99 range)
- [ ] Ring0 policy code cleanup
- [ ] Final validation and documentation

## Critical Success Factors

1. **Strict numbering plan adherence:** Never deviate from 0-99 (v1) and 1000-1009 (v2) ranges
2. **Capability system integration:** All v2 syscalls must use capability tokens
3. **Ring0 minimalism:** No policy decisions in Ring0, mechanism only
4. **Backward compatibility:** v1 syscalls must work until Phase 2.5
5. **Testing discipline:** Validate both interfaces throughout transition period

## Support and Resources

- **Implementation:** `kernel/sys/syscall_v2.c`
- **Interface:** `kernel/sys/syscall_v2.h`
- **Capabilities:** `kernel/include/capability.h`
- **Testing:** `kernel/sys/syscall_hybrid_test.c`
- **Documentation:** This migration guide

For questions or issues during migration, refer to the Phase 2 documentation and architectural transformation specifications.
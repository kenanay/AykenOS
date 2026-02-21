# AykenOS Syscall Transition Guide
This document is subordinate to PHASE 0 – FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

**Author:** Kenan AY  
**Project:** AykenOS - Advanced AI-Integrated Operating System  
**Created:** January 10, 2026  
**Phase:** 2.1 - Ring0 Syscall Redesign

## Overview

This guide documents the migration path from legacy POSIX-like syscalls (v1) to the new execution-centric syscalls (v2) in AykenOS. The transition maintains backward compatibility during Phase 2.1-2.4 and provides a clear upgrade path for applications.

## Syscall Numbering Plan

### Current Dual Interface (Phase 2.1-2.4)

```c
// Legacy POSIX-like syscalls (v1) - Range: 0-99
#define SYS_read       0
#define SYS_write      1
#define SYS_open       2
#define SYS_close      3
#define SYS_exit       60

// New execution-centric syscalls (v2) - Range: 1000-1010
#define SYS_V2_MAP_MEMORY        1000  // (internal: 0)
#define SYS_V2_UNMAP_MEMORY      1001  // (internal: 1)
#define SYS_V2_SWITCH_CONTEXT    1002  // (internal: 2)
#define SYS_V2_SUBMIT_EXECUTION  1003  // (internal: 3)
#define SYS_V2_WAIT_RESULT       1004  // (internal: 4)
#define SYS_V2_INTERRUPT_RETURN  1005  // (internal: 5)
#define SYS_V2_TIME_QUERY        1006  // (internal: 6)
#define SYS_V2_CAPABILITY_BIND   1007  // (internal: 7)
#define SYS_V2_CAPABILITY_REVOKE 1008  // (internal: 8)
#define SYS_V2_EXIT              1009  // (internal: 9)
#define SYS_V2_DEBUG_PUTCHAR     1010  // (internal: 10)
```

### Future State (Phase 2.5+)

- **Legacy syscalls (0-99):** Completely removed
- **Execution-centric syscalls (1000-1010):** Only interface available
- **Total syscalls:** 11 (10 core + debug heartbeat syscall)

## Migration Examples

### 1. Memory Management Migration

#### Legacy v1 Approach (Deprecated)
```c
// Old way: Direct memory operations (limited)
int fd = open("/dev/mem", O_RDWR);
void *ptr = mmap(NULL, 4096, PROT_READ|PROT_WRITE, MAP_SHARED, fd, 0x100000);
close(fd);
```

#### New v2 Approach (Recommended)
```c
// New way: Capability-based memory mapping
#include "syscall_v2.h"

// 1. Get capability token for memory region
capability_token_t mem_cap = {
    .id = 0,  // Will be assigned by system
    .permissions = CAPABILITY_PERM_READ_WRITE,
    .resource_type = CAPABILITY_RESOURCE_MEMORY
};

// 2. Bind capability to execution context
uint64_t execution_ctx = get_current_execution_context();
uint64_t cap_result = syscall(SYS_V2_CAPABILITY_BIND, execution_ctx, &mem_cap);

// 3. Map memory using capability
uint64_t map_result = syscall(SYS_V2_MAP_MEMORY, 0x200000, 0x100000, MAP_READ_WRITE);
```

### 2. Process Termination Migration

#### Legacy v1 Approach (Deprecated)
```c
// Old way: Simple exit
exit(0);
```

#### New v2 Approach (Recommended)
```c
// New way: Execution-centric exit with cleanup
#include "syscall_v2.h"

// 1. Revoke all capabilities
revoke_all_capabilities(current_execution_context);

// 2. Clean exit through v2 interface
syscall(SYS_V2_EXIT, 0);
```

### 3. File Operations Migration

#### Legacy v1 Approach (Deprecated)
```c
// Old way: Traditional file operations
int fd = open("/path/to/file", O_RDONLY);
char buffer[1024];
ssize_t bytes = read(fd, buffer, sizeof(buffer));
close(fd);
```

#### New v2 Approach (Recommended)
```c
// New way: Capability-based file access (Ring3 VFS)
#include "libayken/vfs.h"

// 1. Get file capability
capability_token_t file_cap = request_file_capability("/path/to/file", CAP_PERM_READ);

// 2. Bind capability
uint64_t ctx = get_current_execution_context();
syscall(SYS_V2_CAPABILITY_BIND, ctx, &file_cap);

// 3. Access file through Ring3 VFS
userspace_vfs_t *vfs = get_userspace_vfs();
int fd = vfs->open("/path/to/file", VFS_READ);
char buffer[1024];
ssize_t bytes = vfs->read(fd, buffer, sizeof(buffer));
vfs->close(fd);
```

## Hybrid Syscall Dispatcher

### Implementation Details

The hybrid dispatcher routes syscalls based on number ranges:

```c
uint64_t syscall_handler(uint64_t syscall_num, uint64_t arg1,
                         uint64_t arg2, uint64_t arg3, uint64_t arg4)
{
    // Route based on Syscall Numbering Plan
    if (syscall_num >= 1000 && syscall_num <= 1010) {
        // New execution-centric syscalls (v2)
        return syscall_v2_handler(syscall_num - 1000, arg1, arg2, arg3, arg4);
    } else if (syscall_num >= 0 && syscall_num <= 99) {
        // Legacy POSIX-like syscalls (v1 - backward compatibility)
        return syscall_v1_handler(syscall_num, arg1, arg2, arg3, arg4);
    } else {
        // Invalid syscall number
        return -ENOSYS;
    }
}
```

### Error Handling

- **Invalid ranges:** Return `-ENOSYS` with descriptive logging
- **Valid ranges:** Route to appropriate handler
- **Consistent error codes:** Both v1 and v2 use standard error conventions

## Development Workflow

### Phase 2.1-2.2: Dual Development
```bash
# Applications can use both interfaces
gcc -DUSE_V1_SYSCALLS app_legacy.c -o app_legacy
gcc -DUSE_V2_SYSCALLS app_modern.c -lrayken -o app_modern
```

### Phase 2.3-2.4: Migration Period
```bash
# Gradual migration with warnings
gcc -DWARN_V1_DEPRECATED app_hybrid.c -lrayken -o app_hybrid
```

### Phase 2.5+: V2 Only
```bash
# Only v2 syscalls available
gcc app_v2_only.c -lrayken -o app_v2_only
```

## Testing Both Interfaces

### V1 Syscall Test
```c
#include <unistd.h>
#include <sys/syscall.h>

void test_v1_syscalls(void) {
    // Test legacy write syscall
    const char *msg = "Hello from v1 syscall\n";
    syscall(SYS_write, 1, msg, strlen(msg));
    
    // Test legacy exit syscall
    syscall(SYS_exit, 0);
}
```

### V2 Syscall Test
```c
#include "syscall_v2.h"

void test_v2_syscalls(void) {
    // Test v2 time query
    uint64_t timestamp;
    uint64_t result = syscall(SYS_V2_TIME_QUERY, 0, &timestamp);
    
    if (result == ESYS_V2_SUCCESS) {
        printf("Current timestamp: %lu\n", timestamp);
    }
    
    // Test v2 exit
    syscall(SYS_V2_EXIT, 0);
}
```

## Capability System Integration

### Basic Capability Usage
```c
#include "capability.h"
#include "syscall_v2.h"

void demonstrate_capabilities(void) {
    // 1. Create capability for memory access
    capability_token_t token = capability_create(
        CAPABILITY_RESOURCE_MEMORY,
        CAPABILITY_PERM_READ_WRITE,
        0x100000,  // resource address
        4096       // resource size
    );
    
    // 2. Bind to current execution context
    uint64_t ctx = get_current_execution_context();
    uint64_t bind_result = syscall(SYS_V2_CAPABILITY_BIND, ctx, &token);
    
    if (bind_result == ESYS_V2_SUCCESS) {
        // 3. Use the capability for memory mapping
        uint64_t map_result = syscall(SYS_V2_MAP_MEMORY, 
                                     0x200000,  // virtual address
                                     0x100000,  // physical address  
                                     MAP_READ_WRITE);
        
        // 4. Revoke capability when done
        syscall(SYS_V2_CAPABILITY_REVOKE, token.id);
    }
}
```

## Performance Considerations

### Syscall Overhead Comparison

| Interface | Overhead | Notes |
|-----------|----------|-------|
| V1 Legacy | ~100ns | Direct kernel calls |
| V2 Execution-centric | ~150ns | Capability validation overhead |
| Hybrid Dispatcher | ~10ns | Routing overhead only |

### Optimization Tips

1. **Batch Operations:** Use `SYS_V2_SUBMIT_EXECUTION` for multiple operations
2. **Capability Caching:** Reuse capability tokens when possible
3. **Context Switching:** Minimize `SYS_V2_SWITCH_CONTEXT` calls
4. **Memory Mapping:** Use large pages with `SYS_V2_MAP_MEMORY`

## Migration Timeline

### Phase 2.1 (Current)
- ✅ Hybrid dispatcher implemented
- ✅ Both v1 and v2 interfaces available
- ✅ Capability system functional

### Phase 2.2-2.4 (Migration Period)
- 🔄 Ring3 VFS/DevFS/Scheduler migration
- 🔄 Application migration to v2 interface
- 🔄 Deprecation warnings for v1 usage

### Phase 2.5 (Legacy Cleanup)
- ❌ V1 syscalls completely removed
- ✅ Only v2 interface available
- ✅ Ring0 contains 11 execution-centric syscalls

## Troubleshooting

### Common Migration Issues

#### Issue: "Invalid syscall number" errors
```bash
# Check syscall number ranges
echo "V1 range: 0-99, V2 range: 1000-1010"
```

#### Issue: Capability binding failures
```c
// Debug capability issues
if (bind_result != ESYS_V2_SUCCESS) {
    switch (bind_result) {
    case ESYS_V2_INVALID_PARAM:
        printf("Invalid capability parameters\n");
        break;
    case ESYS_V2_NO_CAPABILITY:
        printf("Capability not found or invalid\n");
        break;
    case ESYS_V2_NO_PERMISSION:
        printf("Insufficient permissions\n");
        break;
    }
}
```

#### Issue: Performance degradation
```c
// Profile syscall usage
#ifdef DEBUG_SYSCALLS
#define SYSCALL_TRACE(num) printf("Syscall %lu called\n", num)
#else
#define SYSCALL_TRACE(num)
#endif
```

## Best Practices

### 1. Gradual Migration
- Start with non-critical components
- Test thoroughly in dual-interface mode
- Migrate high-performance paths last

### 2. Error Handling
- Always check syscall return values
- Use appropriate error codes from `syscall_v2.h`
- Implement proper cleanup on failures

### 3. Capability Management
- Request minimal required permissions
- Revoke capabilities promptly when done
- Use capability delegation for sub-components

### 4. Performance Optimization
- Batch related operations when possible
- Cache capability tokens appropriately
- Profile syscall usage patterns

## References

- [AykenOS Phase 2 Documentation](../docs/phase2_specification.md)
- [Capability System Design](../kernel/include/capability.h)
- [Syscall V2 Interface](../kernel/sys/syscall_v2.h)
- [Ring3 Runtime Libraries](../userspace/libayken/)

---

**Migration Support:** For assistance with syscall migration, consult the AykenOS development team or file an issue in the project repository.

**Deprecation Notice:** V1 syscalls will be completely removed in Phase 2.5. Plan your migration accordingly.

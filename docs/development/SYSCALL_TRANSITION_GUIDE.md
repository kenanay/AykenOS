# AykenOS Syscall Transition Guide - COMPLETED
This document is subordinate to PHASE 0 – FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

**Author:** Kenan AY  
**Project:** AykenOS - Advanced AI-Integrated Operating System  
**Created:** January 3, 2026  
**Updated:** February 21, 2026  
**Version:** 2.1 - ABI Lock + Runtime Reality

## Overview

This guide documents the **completed migration** from AykenOS v1 (POSIX-like) syscalls to v2 (execution-centric) syscalls. The Phase 2.5 architectural transformation has been successfully implemented with the new execution-context and capability-based interface fully operational.

## Syscall Interface Evolution - COMPLETED ✅

### Previous State (Phase 1) - REMOVED
- **5 POSIX-like syscalls**: read, write, open, close, exit (REMOVED)
- **Ring0-heavy implementation**: File system operations in kernel (REMOVED)
- **Traditional file descriptor model**: POSIX-compatible interface (REMOVED)

### Current State (Phase 4.5 Stabilization) - OPERATIONAL ✅
- **11 execution-centric syscalls**: 1000-1010 aralığı aktif
- **Minimal Ring0 attack surface**: Ring0 mekanizma odaklı
- **Capability-based security**: Resource access through tokens
- **Ring3 policy hedefi sürüyor**: VFS/DevFS Ring0 tarafında minimal placeholder olarak korunuyor

### Transition Period (Phase 2.1-2.4) - COMPLETED
- **Dual interface support**: Successfully migrated from v1 to v2
- **Clear numbering plan**: 1000-1010 range implemented
- **Complete migration**: Kernel syscall interface v2-only

## Syscall Numbering Plan - FINAL IMPLEMENTATION

### V1 Syscalls (Legacy) - COMPLETELY REMOVED ✅
```c
// Legacy POSIX-like syscalls - REMOVED in Phase 2.5
// #define SYS_read       0    // REMOVED
// #define SYS_write      1    // REMOVED  
// #define SYS_open       2    // REMOVED
// #define SYS_close      3    // REMOVED
// #define SYS_exit       60   // REMOVED
// All legacy syscalls completely removed from kernel
```

**Removal Status:**
- **Phase 2.5**: ✅ Completely removed from kernel
- **Current**: No legacy syscalls remain

### V2 Syscalls (Current - Range 1000-1010) - OPERATIONAL ✅
```c
// Execution-centric syscalls (operational interface)
#define SYS_V2_MAP_MEMORY        1000  // Map physical memory to virtual
#define SYS_V2_UNMAP_MEMORY      1001  // Unmap virtual memory
#define SYS_V2_SWITCH_CONTEXT    1002  // Switch execution context
#define SYS_V2_SUBMIT_EXECUTION  1003  // Submit BCIB for execution
#define SYS_V2_WAIT_RESULT       1004  // Wait for execution result
#define SYS_V2_INTERRUPT_RETURN  1005  // Return from interrupt context
#define SYS_V2_TIME_QUERY        1006  // Query system time
#define SYS_V2_CAPABILITY_BIND   1007  // Bind capability token
#define SYS_V2_CAPABILITY_REVOKE 1008  // Revoke capability token
#define SYS_V2_EXIT              1009  // Process termination
#define SYS_V2_DEBUG_PUTCHAR     1010  // Ring3 debug heartbeat
```

**Implementation Status:**
- **ABI/range lock**: ✅ `SYS_V2_BASE=1000`, `SYS_V2_LAST=1010`, `SYS_V2_NR=11`
- **INT 0x80 interface**: ✅ Functional with roundtrip validation
- **Mature handlers**: `switch_context`, `capability_bind`, `capability_revoke`, `debug_putchar`
- **Placeholder/TODO handlers**: `map_memory`, `unmap_memory`, `submit_execution`, `wait_result`, `interrupt_return`, `time_query`, `exit`
- **Performance**: ✅ Current CI eşiği altında

### Invalid Ranges
- **100-999**: Reserved for future use, returns -ENOSYS
- **1011+**: Invalid, returns -ENOSYS

## Migration Examples

### Example 1: File Operations Migration

#### V1 Approach (Legacy)
```c
// Traditional file operations using POSIX-like syscalls
int fd = syscall(SYS_open, "/dev/console", 0);  // syscall 2
if (fd >= 0) {
    char buffer[256];
    int bytes = syscall(SYS_read, fd, buffer, sizeof(buffer));  // syscall 0
    syscall(SYS_write, 1, buffer, bytes);  // syscall 1 (stdout)
    syscall(SYS_close, fd);  // syscall 3
}
```

#### V2 Approach (New)
```c
// Memory-mapped file access using execution-centric syscalls
#include "capability.h"

// Step 1: Get capability token for device access
capability_token_t device_cap = get_device_capability("/dev/console");

// Step 2: Bind capability to current execution context
uint64_t ctx_id = get_current_execution_context();
syscall(SYS_V2_CAPABILITY_BIND, ctx_id, &device_cap);  // syscall 1007

// Step 3: Map device memory directly
void *device_mem = (void*)syscall(SYS_V2_MAP_MEMORY,   // syscall 1000
                                  0x10000000,           // virtual address
                                  device_cap.phys_addr, // physical address  
                                  MAP_V2_READ_WRITE | MAP_V2_USER_ACCESS);

// Step 4: Direct memory access (no syscalls needed)
char *console_buffer = (char*)device_mem;
memcpy(console_buffer, "Hello World", 11);

// Step 5: Cleanup
syscall(SYS_V2_UNMAP_MEMORY, 0x10000000, 4096);  // syscall 1001
syscall(SYS_V2_CAPABILITY_REVOKE, device_cap.id);  // syscall 1008
```

### Example 2: Process Management Migration

#### V1 Approach (Legacy)
```c
// Simple process exit
syscall(SYS_exit, 0);  // syscall 60
```

#### V2 Approach (New)
```c
// Execution context management
uint64_t execution_id = syscall(SYS_V2_SUBMIT_EXECUTION,  // syscall 1003
                                bcib_graph, graph_size);

// Wait for completion with timeout
uint64_t result = syscall(SYS_V2_WAIT_RESULT,  // syscall 1004
                          execution_id, 5000);  // 5 second timeout

// Clean exit
syscall(SYS_V2_EXIT, result);  // syscall 1009
```

### Example 3: Time Query Migration

#### V1 Approach (Legacy)
```c
// No direct time query in v1 - would require additional syscalls
// Applications had to use workarounds or estimate time
```

#### V2 Approach (New)
```c
// Direct system time queries
uint64_t current_time;
syscall(SYS_V2_TIME_QUERY, TIME_QUERY_MONOTONIC, &current_time);  // syscall 1006

uint64_t uptime;
syscall(SYS_V2_TIME_QUERY, TIME_QUERY_UPTIME, &uptime);  // syscall 1006
```

## Assembly Interface Examples

### V1 Syscall Assembly (Legacy)
```asm
; V1 write syscall example
mov rax, 1          ; SYS_write (syscall number 1)
mov rdi, 1          ; fd = stdout
mov rsi, msg        ; buffer pointer
mov rdx, msg_len    ; buffer length
int 0x80            ; invoke syscall
; Result in rax
```

### V2 Syscall Assembly (New)
```asm
; V2 memory mapping syscall example
mov rax, 1000       ; SYS_V2_MAP_MEMORY (syscall number 1000)
mov rdi, 0x10000000 ; virtual address
mov rsi, 0x20000000 ; physical address
mov rdx, 0x03       ; flags (READ_WRITE)
int 0x80            ; invoke syscall
; Result in rax
```

## Capability System Integration

### Capability Token Structure
```c
typedef struct capability_token {
    uint64_t id;              // Unique capability identifier
    uint32_t permissions;     // Permission flags
    uint32_t resource_type;   // Type of resource (device, memory, etc.)
    uint64_t phys_addr;       // Physical address (for memory resources)
    uint64_t size;            // Resource size
    uint64_t expiry;          // Expiration timestamp
} capability_token_t;
```

### Permission Flags
```c
#define CAP_PERM_READ     0x01    // Read access
#define CAP_PERM_WRITE    0x02    // Write access  
#define CAP_PERM_EXECUTE  0x04    // Execute access
#define CAP_PERM_ADMIN    0x08    // Administrative access
```

### Resource Types
```c
#define CAP_RESOURCE_MEMORY   1   // Physical memory region
#define CAP_RESOURCE_DEVICE   2   // Hardware device
#define CAP_RESOURCE_NETWORK  3   // Network interface
#define CAP_RESOURCE_GPU      4   // GPU resources
```

## Error Handling Comparison

### V1 Error Handling (Legacy)
```c
int fd = syscall(SYS_open, "/nonexistent", 0);
if (fd < 0) {
    // Generic error - limited information
    printf("Open failed\n");
    return -1;
}
```

### V2 Error Handling (New)
```c
uint64_t result = syscall(SYS_V2_MAP_MEMORY, virt, phys, flags);
switch (result) {
    case V2_SUCCESS:
        printf("Mapping successful\n");
        break;
    case V2_ERROR_INVALID:
        printf("Invalid parameters provided\n");
        break;
    case V2_ERROR_NOMEM:
        printf("Insufficient memory available\n");
        break;
    case V2_ERROR_PERM:
        printf("Permission denied - check capability tokens\n");
        break;
    default:
        printf("Unknown error: %ld\n", result);
}
```

## Performance Considerations

### V1 Performance Characteristics
- **High syscall overhead**: Each file operation requires kernel transition
- **Multiple syscalls per operation**: open → read/write → close sequence
- **Kernel policy execution**: VFS operations in Ring0

### V2 Performance Characteristics  
- **Reduced syscall frequency**: Memory mapping eliminates repeated calls
- **Direct memory access**: No syscalls needed for mapped regions
- **Ring3 policy execution**: Only mechanism in Ring0

### Performance Comparison Example
```c
// V1: Multiple syscalls for file processing
int fd = syscall(SYS_open, "data.txt", 0);     // Syscall 1
for (int i = 0; i < 1000; i++) {
    syscall(SYS_read, fd, buffer, 1024);       // Syscall 2-1001
}
syscall(SYS_close, fd);                        // Syscall 1002
// Total: 1002 syscalls

// V2: Single mapping for file processing  
capability_token_t file_cap = get_file_capability("data.txt");
syscall(SYS_V2_CAPABILITY_BIND, ctx, &file_cap);    // Syscall 1
void *file_mem = syscall(SYS_V2_MAP_MEMORY, ...);   // Syscall 2
for (int i = 0; i < 1000; i++) {
    memcpy(buffer, file_mem + i*1024, 1024);        // No syscalls
}
syscall(SYS_V2_UNMAP_MEMORY, ...);                  // Syscall 3
syscall(SYS_V2_CAPABILITY_REVOKE, file_cap.id);     // Syscall 4
// Total: 4 syscalls (250x reduction)
```

## Security Model Comparison

### V1 Security Model (Legacy)
- **Discretionary Access Control**: File permissions and ownership
- **Process isolation**: Memory protection between processes
- **Limited capability**: Basic user/kernel privilege separation

### V2 Security Model (New)
- **Capability-based access**: Fine-grained resource permissions
- **Principle of least privilege**: Minimal required capabilities
- **Temporal security**: Capability expiration and revocation
- **Audit trail**: All capability operations logged

### Security Example
```c
// V1: Broad file system access
int fd = syscall(SYS_open, "/dev/gpu0", 0);  // Either works or doesn't

// V2: Fine-grained GPU access
capability_token_t gpu_cap = {
    .id = 12345,
    .permissions = CAP_PERM_READ | CAP_PERM_WRITE,
    .resource_type = CAP_RESOURCE_GPU,
    .phys_addr = 0xE0000000,
    .size = 0x1000000,
    .expiry = current_time + 3600  // 1 hour expiry
};
syscall(SYS_V2_CAPABILITY_BIND, ctx_id, &gpu_cap);
```

## Migration Strategy

### Phase 1: Preparation (Current)
1. **Understand current v1 interface**: Review existing syscall usage
2. **Identify migration candidates**: Applications using file operations
3. **Plan capability requirements**: Determine needed resource access

### Phase 2: Dual Interface (Phase 2.1-2.4)
1. **Test v2 syscalls**: Validate new interface functionality
2. **Implement capability management**: Set up token acquisition
3. **Gradual migration**: Convert applications one by one
4. **Performance testing**: Compare v1 vs v2 performance

### Phase 3: V1 Deprecation (Phase 2.5)
1. **Complete migration**: All applications using v2 interface
2. **Remove v1 support**: Clean up legacy syscall handlers
3. **Validate functionality**: Ensure all features work with v2 only

## Testing and Validation

### Compatibility Testing
```c
// Test both interfaces during transition
void test_dual_interface() {
    // Test v1 syscall
    uint64_t v1_result = syscall(1, 1, "Hello v1\n", 9);  // write
    
    // Test v2 syscall  
    uint64_t time_result;
    uint64_t v2_result = syscall(1006, TIME_QUERY_UPTIME, &time_result);
    
    // Test invalid syscall
    uint64_t invalid_result = syscall(500, 0, 0, 0);  // Should return -ENOSYS
    
    printf("V1 result: %ld, V2 result: %ld, Invalid: %ld\n", 
           v1_result, v2_result, invalid_result);
}
```

### Performance Benchmarking
```c
// Benchmark syscall overhead
uint64_t start_time, end_time;

// V1 benchmark
syscall(SYS_V2_TIME_QUERY, TIME_QUERY_MONOTONIC, &start_time);
for (int i = 0; i < 1000; i++) {
    syscall(SYS_write, 1, ".", 1);  // V1 syscall
}
syscall(SYS_V2_TIME_QUERY, TIME_QUERY_MONOTONIC, &end_time);
printf("V1 time: %ld microseconds\n", end_time - start_time);

// V2 benchmark  
syscall(SYS_V2_TIME_QUERY, TIME_QUERY_MONOTONIC, &start_time);
for (int i = 0; i < 1000; i++) {
    syscall(SYS_V2_TIME_QUERY, TIME_QUERY_UPTIME, &end_time);  // V2 syscall
}
syscall(SYS_V2_TIME_QUERY, TIME_QUERY_MONOTONIC, &end_time);
printf("V2 time: %ld microseconds\n", end_time - start_time);
```

## Common Migration Patterns

### Pattern 1: File I/O → Memory Mapping
```c
// Before (V1)
int process_file_v1(const char *filename) {
    int fd = syscall(SYS_open, filename, 0);
    if (fd < 0) return -1;
    
    char buffer[4096];
    int bytes = syscall(SYS_read, fd, buffer, sizeof(buffer));
    
    // Process buffer...
    
    syscall(SYS_close, fd);
    return bytes;
}

// After (V2)
int process_file_v2(const char *filename) {
    capability_token_t file_cap = get_file_capability(filename);
    if (file_cap.id == 0) return -1;
    
    uint64_t ctx = get_current_execution_context();
    syscall(SYS_V2_CAPABILITY_BIND, ctx, &file_cap);
    
    void *file_mem = (void*)syscall(SYS_V2_MAP_MEMORY, 
                                    0x20000000, file_cap.phys_addr,
                                    MAP_V2_READ_ONLY | MAP_V2_USER_ACCESS);
    
    // Process memory directly (no syscalls)...
    
    syscall(SYS_V2_UNMAP_MEMORY, 0x20000000, file_cap.size);
    syscall(SYS_V2_CAPABILITY_REVOKE, file_cap.id);
    return file_cap.size;
}
```

### Pattern 2: Process Control → Execution Context
```c
// Before (V1)
void simple_exit_v1(int code) {
    syscall(SYS_exit, code);  // Never returns
}

// After (V2)
void controlled_exit_v2(int code) {
    // Submit final cleanup execution
    cleanup_bcib_graph_t cleanup = create_cleanup_graph();
    uint64_t cleanup_id = syscall(SYS_V2_SUBMIT_EXECUTION, 
                                  &cleanup, sizeof(cleanup));
    
    // Wait for cleanup completion
    syscall(SYS_V2_WAIT_RESULT, cleanup_id, 1000);  // 1 second timeout
    
    // Clean exit
    syscall(SYS_V2_EXIT, code);
}
```

### Pattern 3: Polling → Event-Driven
```c
// Before (V1) - No direct support, manual polling needed
void wait_for_condition_v1() {
    while (1) {
        // Check condition manually
        if (check_condition()) break;
        
        // Yield CPU (inefficient)
        syscall(SYS_exit, 0);  // Crude yield simulation
    }
}

// After (V2) - Event-driven execution
void wait_for_condition_v2() {
    // Create event-waiting BCIB graph
    event_wait_graph_t wait_graph = create_event_wait_graph(EVENT_CONDITION);
    
    uint64_t wait_id = syscall(SYS_V2_SUBMIT_EXECUTION, 
                               &wait_graph, sizeof(wait_graph));
    
    // Efficient event-driven wait
    uint64_t result = syscall(SYS_V2_WAIT_RESULT, wait_id, 0);  // No timeout
    
    // Condition met, continue execution
}
```

## Troubleshooting Guide

### Common Issues and Solutions

#### Issue 1: Invalid Syscall Number
```
Error: syscall returns -38 (ENOSYS)
```
**Cause**: Using syscall number outside valid ranges (0-99, 1000-1010)  
**Solution**: Check syscall numbering plan and use correct constants

#### Issue 2: Capability Permission Denied
```
Error: SYS_V2_MAP_MEMORY returns V2_ERROR_PERM (-3)
```
**Cause**: Missing or insufficient capability token  
**Solution**: Acquire proper capability token before memory operations

#### Issue 3: Memory Mapping Failure
```
Error: SYS_V2_MAP_MEMORY returns V2_ERROR_NOMEM (-2)
```
**Cause**: Virtual address space exhaustion or invalid physical address  
**Solution**: Use different virtual address or check physical address validity

#### Issue 4: Execution Timeout
```
Error: SYS_V2_WAIT_RESULT returns V2_ERROR_TIMEOUT (-5)
```
**Cause**: BCIB execution taking longer than expected  
**Solution**: Increase timeout value or optimize BCIB graph

### Debug Techniques

#### Syscall Tracing
```c
// Enable syscall debugging
#define DEBUG_SYSCALLS 1

uint64_t debug_syscall(uint64_t num, uint64_t arg1, uint64_t arg2, 
                       uint64_t arg3, uint64_t arg4) {
    printf("SYSCALL: num=%ld, args=[%ld, %ld, %ld, %ld]\n", 
           num, arg1, arg2, arg3, arg4);
    
    uint64_t result = syscall(num, arg1, arg2, arg3, arg4);
    
    printf("SYSCALL RESULT: %ld\n", result);
    return result;
}
```

#### Capability Validation
```c
// Validate capability token before use
int validate_capability(capability_token_t *cap) {
    if (!cap) return 0;
    if (cap->id == 0) return 0;
    if (cap->expiry < get_current_time()) return 0;
    if (cap->permissions == 0) return 0;
    return 1;
}
```

## Best Practices

### 1. Capability Management
- **Acquire minimal capabilities**: Only request necessary permissions
- **Check expiration**: Validate capability tokens before use
- **Revoke promptly**: Release capabilities when no longer needed
- **Handle failures gracefully**: Always check capability acquisition results

### 2. Memory Mapping
- **Use consistent virtual addresses**: Avoid conflicts between mappings
- **Align to page boundaries**: Ensure proper memory alignment
- **Unmap when done**: Prevent virtual address space leaks
- **Check mapping success**: Validate return values before use

### 3. Execution Context Management
- **Submit well-formed graphs**: Validate BCIB graphs before submission
- **Use appropriate timeouts**: Balance responsiveness with reliability
- **Handle execution failures**: Implement proper error recovery
- **Clean up resources**: Ensure proper cleanup on exit

### 4. Performance Optimization
- **Batch operations**: Group related operations to reduce syscall overhead
- **Cache capabilities**: Reuse capability tokens when possible
- **Optimize memory usage**: Use memory mapping efficiently
- **Profile syscall usage**: Identify and optimize hot paths

## Future Considerations

### Phase 3 and Beyond
- **Extended capability system**: More fine-grained permissions
- **Hardware acceleration**: Direct GPU/AI accelerator access
- **Distributed execution**: Cross-node BCIB execution
- **Real-time guarantees**: Deterministic execution timing

### API Evolution
- **Higher-level libraries**: Wrapper libraries for common patterns
- **Language bindings**: Native support in Rust, C++, Python
- **Development tools**: Debuggers and profilers for v2 interface
- **Documentation**: Comprehensive API reference and tutorials

## Conclusion

The migration from v1 to v2 syscalls represents a fundamental shift in AykenOS architecture, moving from traditional POSIX-like operations to execution-centric, capability-based computing. This transition enables:

- **Enhanced security** through capability-based access control
- **Improved performance** via reduced syscall overhead and memory mapping
- **Greater flexibility** with execution context management
- **Future scalability** for AI and distributed computing workloads

The dual interface approach ensures smooth migration while maintaining backward compatibility during the transition period. Applications should begin migrating to the v2 interface to take advantage of improved performance and security features.

For additional support and examples, refer to:
- **Phase 2 Documentation**: Detailed architectural specifications
- **Code Examples**: Sample applications in `/examples/syscall_v2/`
- **Test Suite**: Validation tests in `/tests/syscall_transition/`
- **Performance Benchmarks**: Comparative analysis in `/benchmarks/syscall/`

---

**Document Version**: 1.0  
**Last Updated**: January 3, 2026  
**Next Review**: Phase 2.2 completion

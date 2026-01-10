# Ring3 VFS Library - API Design

**Author:** Kenan AY  
**Project:** AykenOS - Advanced AI-Integrated Operating System  
**Created:** January 3, 2026  
**Phase:** Phase 2.2 - Ring3 Runtime Development  
**Task:** 2.2.1.1 - Design Ring3 VFS interface (Step A: API Design)

## Overview

This directory contains the Ring3 VFS (Virtual File System) library API design for AykenOS Phase 2.2. The Ring3 VFS library provides userspace file system operations that communicate with Ring0 through the new execution-centric syscall interface, specifically using memory mapping mechanisms rather than traditional POSIX-like syscalls.

## Architecture

The Ring3 VFS library follows a layered architecture:

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
├─────────────────────────────────────────────────────────────┤
│              Ring3 VFS Public API (vfs.h)                  │
├─────────────────────────────────────────────────────────────┤
│         VFS Implementation Layer (vfs_impl.h)              │
├─────────────────────────────────────────────────────────────┤
│           VFS Internal Types (vfs_types.h)                 │
├─────────────────────────────────────────────────────────────┤
│              Capability System Integration                  │
├─────────────────────────────────────────────────────────────┤
│                Ring0 Syscall Interface                     │
│         (sys_v2_map_memory, sys_v2_capability_bind)        │
└─────────────────────────────────────────────────────────────┘
```

## Key Design Principles

### 1. Memory Mapping Based I/O
- All file operations use Ring0 memory mapping mechanisms
- No traditional read/write syscalls - everything goes through `sys_v2_map_memory`
- Files are accessed through memory-mapped regions established by Ring0

### 2. Capability-Based Security
- All file access is mediated through capability tokens
- Capabilities grant specific permissions (read, write, create, delete)
- Ring0 enforces capability-based access control

### 3. Pluggable Implementation Architecture
- Multiple VFS implementations can coexist
- Factory pattern for creating different VFS backends
- Support for layered and overlay file systems

### 4. Ring0 Mechanism Only
- Ring0 provides only low-level mechanisms (memory mapping, capability enforcement)
- All file system policy decisions happen in Ring3
- Ring0 has no knowledge of file system semantics

## API Components

### Core API (`vfs.h`)

The main public interface provides:

```c
typedef struct userspace_vfs {
    int (*open)(const char *path, int flags);
    int (*read)(int fd, void *buf, size_t count);
    int (*write)(int fd, const void *buf, size_t count);
    int (*close)(int fd);
    int64_t (*seek)(int fd, int64_t offset, int whence);
    int (*stat)(const char *path, vfs_stat_t *stat);
    int (*fstat)(int fd, vfs_stat_t *stat);
    int (*fsync)(int fd);
    int (*unlink)(const char *path);
    int (*mkdir)(const char *path, uint32_t mode);
    int (*rmdir)(const char *path);
} userspace_vfs_t;
```

### Internal Types (`vfs_types.h`)

Defines internal structures for:
- File descriptor management
- Memory mapping regions
- Capability token integration
- VFS implementation context
- Error handling and diagnostics

### Implementation Framework (`vfs_impl.h`)

Provides framework for creating VFS implementations:
- Base implementation class
- Factory pattern for implementation creation
- Implementation registry system
- Built-in implementations (memory, Ring0 proxy, capability-based)

## Integration with Ring0

### Syscall Interface

The Ring3 VFS uses these Ring0 syscalls:

```c
// Memory mapping for file access
uint64_t sys_v2_map_memory(uint64_t virt, uint64_t phys, uint64_t flags);
uint64_t sys_v2_unmap_memory(uint64_t virt, uint64_t size);

// Capability system integration
uint64_t sys_v2_capability_bind(uint64_t execution_ctx, capability_token_t *token);
uint64_t sys_v2_capability_revoke(uint64_t token_id);
```

### File Access Flow

1. **Open File**: Application calls `vfs_open(path, flags)`
2. **Acquire Capability**: VFS library requests capability token for file
3. **Bind Capability**: Use `sys_v2_capability_bind` to associate capability with execution context
4. **Map Memory**: Use `sys_v2_map_memory` to establish memory-mapped access to file
5. **File Operations**: Read/write operations work directly on mapped memory
6. **Close File**: Unmap memory and revoke capability

## Implementation Strategy

### Phase 2.2.1 (Current Task)
- **Step A: API Design** ✅ - Design Ring3 VFS interface (this task)
- **Step B: Kernel Stub Conversion** - Convert kernel VFS to Ring3 proxy
- **Step C: Full Implementation** - Complete Ring3 VFS using new syscalls

### Built-in Implementations

1. **Memory VFS**: In-memory file system for temporary files
2. **Ring0 Proxy VFS**: Primary implementation that proxies to Ring0 mechanisms
3. **Capability VFS**: Enhanced security through strict capability enforcement
4. **Layered VFS**: Support for union mounts and overlay file systems

## Error Handling

The VFS library provides comprehensive error handling:

```c
typedef enum {
    VFS_SUCCESS         = 0,
    VFS_ERROR_NOENT     = -2,    // No such file or directory
    VFS_ERROR_PERM      = -3,    // Permission denied
    VFS_ERROR_CAPABILITY = -14   // Capability system error
} vfs_error_t;
```

## Usage Example

```c
#include "userspace/libayken/vfs.h"

int main() {
    // Initialize VFS library
    if (vfs_init() != 0) {
        return -1;
    }
    
    // Open a file
    int fd = vfs_open("/system/config.txt", VFS_O_RDONLY);
    if (fd < 0) {
        return -1;
    }
    
    // Read from file
    char buffer[1024];
    int bytes_read = vfs_read(fd, buffer, sizeof(buffer));
    
    // Close file
    vfs_close(fd);
    
    // Shutdown VFS library
    vfs_shutdown();
    
    return 0;
}
```

## Compatibility

### Backward Compatibility
- During Phase 2.1-2.4 transition period, both v1 (POSIX-like) and v2 (execution-centric) syscalls coexist
- Applications can gradually migrate from kernel VFS to Ring3 VFS
- Ring3 VFS provides POSIX-compatible interface for existing applications

### Forward Compatibility
- Designed to support future file system features
- Extensible through implementation plugins
- Capability system allows for fine-grained security policies

## Requirements Validation

This API design satisfies the following requirements from the specification:

- **FR-3.1.1**: VFS operations execute entirely in Ring3 userspace ✅
- **FR-3.1.2**: File access uses Ring0 memory mapping mechanism only ✅
- **FR-3.1.3**: VFS library provides POSIX-compatible interface ✅
- **FR-3.1.4**: File system policy decisions do not involve Ring0 ✅

## Next Steps

1. **Task 2.2.1.2**: Convert kernel VFS to Ring3 proxy (Step B)
2. **Task 2.2.1.3**: Implement Ring3 VFS using new syscalls (Step C)
3. Integration with capability system
4. Performance optimization and testing

## Files in this Directory

- `vfs.h` - Main public API interface
- `vfs_types.h` - Internal types and structures
- `vfs_impl.h` - Implementation framework
- `README.md` - This documentation file

## Dependencies

- Ring0 syscall interface (sys_v2_* functions)
- Capability system integration
- Memory management subsystem
- Error handling framework

---

**Status**: API Design Complete ✅  
**Next Task**: 2.2.1.2 - Convert kernel VFS to Ring3 proxy (Step B)
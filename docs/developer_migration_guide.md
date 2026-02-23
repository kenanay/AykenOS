# AykenOS Developer Migration Guide
This document is subordinate to PHASE 0 – FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

**Author:** Kenan AY  
**Project:** AykenOS - Advanced AI-Integrated Operating System  
**Created:** January 10, 2026  
**Phase:** 2.1 - Developer Migration Support

## Quick Start Migration

### 1. Update Your Build System

#### Makefile Changes
```makefile
# Add v2 syscall support
CFLAGS += -DAYKEN_V2_SYSCALLS
LDFLAGS += -lrayken

# Include new headers
INCLUDES += -Iuserspace/libayken -Ikernel/sys
```

#### CMake Changes
```cmake
# CMakeLists.txt
find_package(AykenOS REQUIRED)
target_link_libraries(your_app AykenOS::libayken)
target_compile_definitions(your_app PRIVATE AYKEN_V2_SYSCALLS)
```

### 2. Update Your Headers

#### Replace Legacy Includes
```c
// OLD - Remove these
#include <unistd.h>
#include <sys/syscall.h>
#include <fcntl.h>

// NEW - Add these
#include "syscall_v2.h"
#include "capability.h"
#include "libayken/vfs.h"
```

### 3. Basic Function Replacements

#### File Operations
```c
// OLD v1 approach
int fd = open("/path/file", O_RDONLY);
read(fd, buffer, size);
close(fd);

// NEW v2 approach
userspace_vfs_t *vfs = get_userspace_vfs();
int fd = vfs->open("/path/file", VFS_READ);
vfs->read(fd, buffer, size);
vfs->close(fd);
```

#### Memory Operations
```c
// OLD v1 approach
void *ptr = mmap(NULL, size, PROT_READ|PROT_WRITE, MAP_PRIVATE, -1, 0);

// NEW v2 approach
capability_token_t cap = request_memory_capability(size, CAP_PERM_READ_WRITE);
syscall(SYS_V2_CAPABILITY_BIND, get_current_execution_context(), &cap);
syscall(SYS_V2_MAP_MEMORY, virt_addr, phys_addr, MAP_READ_WRITE);
```

## Step-by-Step Migration Process

### Step 1: Assessment Phase

#### Analyze Your Syscall Usage
```bash
# Find all syscall usage in your codebase
grep -r "syscall\|open\|read\|write\|close\|exit" src/
grep -r "mmap\|munmap\|fork\|exec" src/
```

#### Create Migration Checklist
```markdown
- [ ] File I/O operations (open, read, write, close)
- [ ] Memory management (mmap, munmap, brk)
- [ ] Process management (fork, exec, exit)
- [ ] Inter-process communication (pipe, socket)
- [ ] Device access (/dev/* files)
```

### Step 2: Preparation Phase

#### Set Up Dual Build Support
```c
// config.h
#ifdef AYKEN_V2_SYSCALLS
    #define USE_V2_INTERFACE 1
    #include "syscall_v2.h"
#else
    #define USE_V2_INTERFACE 0
    #include <unistd.h>
#endif
```

#### Create Compatibility Layer
```c
// compat.h - Temporary compatibility layer
#if USE_V2_INTERFACE
    #define compat_open(path, flags) userspace_vfs_open(path, flags)
    #define compat_read(fd, buf, size) userspace_vfs_read(fd, buf, size)
    #define compat_write(fd, buf, size) userspace_vfs_write(fd, buf, size)
    #define compat_close(fd) userspace_vfs_close(fd)
#else
    #define compat_open(path, flags) open(path, flags)
    #define compat_read(fd, buf, size) read(fd, buf, size)
    #define compat_write(fd, buf, size) write(fd, buf, size)
    #define compat_close(fd) close(fd)
#endif
```

### Step 3: Incremental Migration

#### Phase 1: File Operations
```c
// Before
void read_config_file(void) {
    int fd = open("/etc/config", O_RDONLY);
    if (fd < 0) return;
    
    char buffer[1024];
    ssize_t bytes = read(fd, buffer, sizeof(buffer));
    close(fd);
    
    process_config(buffer, bytes);
}

// After
void read_config_file(void) {
    // Get file capability
    capability_token_t file_cap = request_file_capability("/etc/config", CAP_PERM_READ);
    
    // Bind capability to current context
    uint64_t ctx = get_current_execution_context();
    if (syscall(SYS_V2_CAPABILITY_BIND, ctx, &file_cap) != ESYS_V2_SUCCESS) {
        return;
    }
    
    // Use Ring3 VFS
    userspace_vfs_t *vfs = get_userspace_vfs();
    int fd = vfs->open("/etc/config", VFS_READ);
    if (fd < 0) {
        syscall(SYS_V2_CAPABILITY_REVOKE, file_cap.id);
        return;
    }
    
    char buffer[1024];
    ssize_t bytes = vfs->read(fd, buffer, sizeof(buffer));
    vfs->close(fd);
    
    // Clean up capability
    syscall(SYS_V2_CAPABILITY_REVOKE, file_cap.id);
    
    process_config(buffer, bytes);
}
```

#### Phase 2: Memory Management
```c
// Before
void allocate_work_buffer(void) {
    void *buffer = mmap(NULL, BUFFER_SIZE, 
                       PROT_READ|PROT_WRITE, 
                       MAP_PRIVATE|MAP_ANONYMOUS, -1, 0);
    if (buffer == MAP_FAILED) return;
    
    // Use buffer...
    
    munmap(buffer, BUFFER_SIZE);
}

// After
void allocate_work_buffer(void) {
    // Create memory capability
    capability_token_t mem_cap = capability_create(
        CAPABILITY_RESOURCE_MEMORY,
        CAPABILITY_PERM_READ_WRITE,
        0,  // Let system choose address
        BUFFER_SIZE
    );
    
    // Bind capability
    uint64_t ctx = get_current_execution_context();
    if (syscall(SYS_V2_CAPABILITY_BIND, ctx, &mem_cap) != ESYS_V2_SUCCESS) {
        return;
    }
    
    // Map memory
    uint64_t virt_addr = 0x10000000;  // Suggested virtual address
    uint64_t result = syscall(SYS_V2_MAP_MEMORY, virt_addr, 0, MAP_READ_WRITE);
    if (result != ESYS_V2_SUCCESS) {
        syscall(SYS_V2_CAPABILITY_REVOKE, mem_cap.id);
        return;
    }
    
    void *buffer = (void *)virt_addr;
    
    // Use buffer...
    
    // Clean up
    syscall(SYS_V2_UNMAP_MEMORY, virt_addr, BUFFER_SIZE);
    syscall(SYS_V2_CAPABILITY_REVOKE, mem_cap.id);
}
```

#### Phase 3: Process Management
```c
// Before
void spawn_worker_process(void) {
    pid_t pid = fork();
    if (pid == 0) {
        // Child process
        execl("/bin/worker", "worker", NULL);
        exit(1);
    } else if (pid > 0) {
        // Parent process
        int status;
        waitpid(pid, &status, 0);
    }
}

// After
void spawn_worker_execution(void) {
    // Create execution context
    execution_context_t worker_ctx = {
        .context_id = 0,  // Will be assigned
        .process_id = get_current_process_id(),
        .memory_base = NULL,
        .memory_size = WORKER_MEMORY_SIZE,
        .capabilities = NULL,
        .capability_count = 0,
        .creation_time = 0,
        .status = EXEC_STATUS_CREATED
    };
    
    // Submit execution
    bcib_graph_t *worker_graph = create_worker_graph();
    uint64_t exec_id = syscall(SYS_V2_SUBMIT_EXECUTION, 
                              worker_graph, 
                              sizeof(*worker_graph),
                              worker_ctx.context_id);
    
    if (exec_id > 0) {
        // Wait for completion
        uint64_t result = syscall(SYS_V2_WAIT_RESULT, exec_id, 30000);  // 30s timeout
        
        if (result == ESYS_V2_SUCCESS) {
            printf("Worker execution completed successfully\n");
        }
    }
    
    free_worker_graph(worker_graph);
}
```

### Step 4: Testing and Validation

#### Unit Testing Framework
```c
// test_migration.c
#include "test_framework.h"

void test_file_operations_v2(void) {
    // Test v2 file operations
    userspace_vfs_t *vfs = get_userspace_vfs();
    
    // Create test file
    int fd = vfs->open("/tmp/test", VFS_CREATE | VFS_WRITE);
    TEST_ASSERT(fd >= 0, "Failed to create test file");
    
    // Write data
    const char *data = "Hello AykenOS v2";
    ssize_t written = vfs->write(fd, data, strlen(data));
    TEST_ASSERT(written == strlen(data), "Write failed");
    
    vfs->close(fd);
    
    // Read back data
    fd = vfs->open("/tmp/test", VFS_READ);
    TEST_ASSERT(fd >= 0, "Failed to open test file for reading");
    
    char buffer[64];
    ssize_t read_bytes = vfs->read(fd, buffer, sizeof(buffer));
    TEST_ASSERT(read_bytes == strlen(data), "Read failed");
    TEST_ASSERT(memcmp(buffer, data, strlen(data)) == 0, "Data mismatch");
    
    vfs->close(fd);
}

void test_capability_system(void) {
    // Test capability creation and binding
    capability_token_t cap = capability_create(
        CAPABILITY_RESOURCE_MEMORY,
        CAPABILITY_PERM_READ,
        0x100000, 4096
    );
    
    TEST_ASSERT(cap.id > 0, "Failed to create capability");
    
    uint64_t ctx = get_current_execution_context();
    uint64_t result = syscall(SYS_V2_CAPABILITY_BIND, ctx, &cap);
    TEST_ASSERT(result == ESYS_V2_SUCCESS, "Failed to bind capability");
    
    // Test capability revocation
    result = syscall(SYS_V2_CAPABILITY_REVOKE, cap.id);
    TEST_ASSERT(result == ESYS_V2_SUCCESS, "Failed to revoke capability");
}
```

#### Integration Testing
```bash
#!/bin/bash
# test_migration.sh

echo "Testing v1 to v2 migration..."

# Build both versions
make clean
make CFLAGS="-DAYKEN_V1_SYSCALLS" app_v1
make CFLAGS="-DAYKEN_V2_SYSCALLS" app_v2

# Test v1 version (should work during transition)
echo "Testing v1 interface..."
./app_v1 || echo "V1 test failed"

# Test v2 version
echo "Testing v2 interface..."
./app_v2 || echo "V2 test failed"

# Test hybrid mode
echo "Testing hybrid mode..."
make CFLAGS="-DAYKEN_HYBRID_MODE" app_hybrid
./app_hybrid || echo "Hybrid test failed"

echo "Migration testing complete"
```

## Common Migration Patterns

### Pattern 1: Error Handling Migration
```c
// OLD v1 error handling
if (open("/path/file", O_RDONLY) < 0) {
    perror("open failed");
    return -1;
}

// NEW v2 error handling
capability_token_t cap = request_file_capability("/path/file", CAP_PERM_READ);
uint64_t result = syscall(SYS_V2_CAPABILITY_BIND, ctx, &cap);
if (result != ESYS_V2_SUCCESS) {
    switch (result) {
    case ESYS_V2_NO_PERMISSION:
        fprintf(stderr, "Permission denied for file access\n");
        break;
    case ESYS_V2_NO_CAPABILITY:
        fprintf(stderr, "Invalid capability for file\n");
        break;
    default:
        fprintf(stderr, "Capability bind failed: %ld\n", result);
    }
    return -1;
}
```

### Pattern 2: Resource Cleanup Migration
```c
// OLD v1 cleanup
void cleanup_resources(void) {
    if (fd >= 0) close(fd);
    if (ptr != MAP_FAILED) munmap(ptr, size);
}

// NEW v2 cleanup
void cleanup_resources_v2(void) {
    // Revoke all capabilities
    for (int i = 0; i < capability_count; i++) {
        syscall(SYS_V2_CAPABILITY_REVOKE, capabilities[i].id);
    }
    
    // Unmap memory regions
    for (int i = 0; i < memory_region_count; i++) {
        syscall(SYS_V2_UNMAP_MEMORY, regions[i].virt_addr, regions[i].size);
    }
    
    // Close VFS handles
    userspace_vfs_t *vfs = get_userspace_vfs();
    for (int i = 0; i < fd_count; i++) {
        if (fds[i] >= 0) vfs->close(fds[i]);
    }
}
```

### Pattern 3: Batch Operations Migration
```c
// OLD v1 individual operations
for (int i = 0; i < file_count; i++) {
    int fd = open(files[i], O_RDONLY);
    read(fd, buffers[i], sizes[i]);
    close(fd);
}

// NEW v2 batch operations
bcib_graph_t *batch_graph = create_batch_file_operations(files, file_count);
uint64_t exec_id = syscall(SYS_V2_SUBMIT_EXECUTION, 
                          batch_graph, 
                          calculate_graph_size(batch_graph),
                          get_current_execution_context());

uint64_t result = syscall(SYS_V2_WAIT_RESULT, exec_id, BATCH_TIMEOUT);
if (result == ESYS_V2_SUCCESS) {
    extract_batch_results(batch_graph, buffers, sizes);
}
```

## Performance Optimization Tips

### 1. Capability Caching
```c
// Cache frequently used capabilities
static capability_token_t cached_file_cap = {0};
static capability_token_t cached_mem_cap = {0};

capability_token_t get_file_capability(const char *path) {
    if (cached_file_cap.id == 0) {
        cached_file_cap = request_file_capability(path, CAP_PERM_READ);
    }
    return cached_file_cap;
}
```

### 2. Batch Syscalls
```c
// Instead of multiple individual syscalls
syscall(SYS_V2_MAP_MEMORY, addr1, phys1, flags);
syscall(SYS_V2_MAP_MEMORY, addr2, phys2, flags);
syscall(SYS_V2_MAP_MEMORY, addr3, phys3, flags);

// Use batch execution
memory_map_batch_t batch = {
    .operations = {{addr1, phys1, flags}, {addr2, phys2, flags}, {addr3, phys3, flags}},
    .count = 3
};
bcib_graph_t *graph = create_memory_map_batch_graph(&batch);
syscall(SYS_V2_SUBMIT_EXECUTION, graph, graph_size, ctx);
```

### 3. Context Reuse
```c
// Reuse execution contexts for related operations
static uint64_t worker_context = 0;

uint64_t get_worker_context(void) {
    if (worker_context == 0) {
        worker_context = create_execution_context("worker", WORKER_MEMORY_SIZE);
    }
    return worker_context;
}
```

## Troubleshooting Guide

### Common Issues and Solutions

#### Issue: "Capability not found" errors
```c
// Debug capability issues
void debug_capability_error(uint64_t result) {
    switch (result) {
    case ESYS_V2_NO_CAPABILITY:
        printf("Capability not found - check capability creation\n");
        break;
    case ESYS_V2_INVALID_PARAM:
        printf("Invalid capability parameters\n");
        break;
    case ESYS_V2_NO_PERMISSION:
        printf("Insufficient permissions for capability\n");
        break;
    }
}
```

#### Issue: Memory mapping failures
```c
// Check memory mapping prerequisites
uint64_t safe_map_memory(uint64_t virt, uint64_t phys, uint64_t flags) {
    // Validate addresses
    if (virt == 0 || phys == 0) {
        printf("Invalid memory addresses: virt=0x%lx phys=0x%lx\n", virt, phys);
        return ESYS_V2_INVALID_PARAM;
    }
    
    // Check alignment
    if ((virt & 0xFFF) || (phys & 0xFFF)) {
        printf("Addresses not page-aligned: virt=0x%lx phys=0x%lx\n", virt, phys);
        return ESYS_V2_INVALID_PARAM;
    }
    
    return syscall(SYS_V2_MAP_MEMORY, virt, phys, flags);
}
```

#### Issue: Performance degradation
```c
// Profile syscall usage
#ifdef PROFILE_SYSCALLS
static uint64_t syscall_counts[11] = {0};
static uint64_t syscall_times[11] = {0};

uint64_t profiled_syscall(uint64_t num, uint64_t arg1, uint64_t arg2, 
                         uint64_t arg3, uint64_t arg4) {
    uint64_t start = get_timestamp();
    uint64_t result = syscall(num, arg1, arg2, arg3, arg4);
    uint64_t end = get_timestamp();
    
    if (num >= 1000 && num <= 1010) {
        int idx = num - 1000;
        syscall_counts[idx]++;
        syscall_times[idx] += (end - start);
    }
    
    return result;
}

void print_syscall_profile(void) {
    for (int i = 0; i < 11; i++) {
        if (syscall_counts[i] > 0) {
            printf("Syscall %d: %lu calls, avg time: %lu ns\n", 
                   i, syscall_counts[i], syscall_times[i] / syscall_counts[i]);
        }
    }
}
#endif
```

## Migration Checklist

### Pre-Migration
- [ ] Analyze current syscall usage
- [ ] Identify performance-critical paths
- [ ] Set up dual build system
- [ ] Create compatibility layer
- [ ] Write migration tests

### During Migration
- [ ] Migrate file operations first
- [ ] Update memory management
- [ ] Convert process management
- [ ] Test each component thoroughly
- [ ] Profile performance impact

### Post-Migration
- [ ] Remove compatibility layer
- [ ] Clean up old code paths
- [ ] Update documentation
- [ ] Optimize v2 usage patterns
- [ ] Prepare for v1 removal (Phase 2.5)

## Support and Resources

### Getting Help
- **Documentation:** `/docs/syscall_transition_guide.md`
- **Examples:** `/examples/migration/`
- **Test Suite:** `/tests/migration/`
- **Community:** AykenOS Developer Forum

### Migration Tools
- **Syscall Analyzer:** `tools/analyze_syscalls.sh`
- **Compatibility Checker:** `tools/check_v2_compat.py`
- **Performance Profiler:** `tools/profile_migration.c`

---

**Remember:** The migration to v2 syscalls is not just about changing function calls - it's about adopting AykenOS's execution-centric, capability-based paradigm for better security and performance.

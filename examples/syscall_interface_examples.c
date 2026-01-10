// examples/syscall_interface_examples.c
// AykenOS Syscall Interface Examples - Phase 2.5 Final
//
// This file demonstrates the execution-centric syscall interface only.
// All legacy POSIX syscalls have been removed as part of the architectural
// transformation to a data-centric, AI-native operating system.
//
// Author: Kenan AY
// Project: AykenOS - Advanced AI-Integrated Operating System
// Updated: January 10, 2026 - Phase 2.5 Legacy Cleanup

#include <stdint.h>
#include <stddef.h>
#include <string.h>

// Only v2 execution-centric syscalls are supported
#include "syscall_v2.h"
#include "capability.h"
#include "libayken/vfs.h"

// Forward declarations
void fb_print(const char *s);
void fb_print_int(int64_t value);
void fb_print_hex(uint64_t v);

// ============================================================================
// EXAMPLE 1: BASIC V2 SYSCALL USAGE
// ============================================================================

/**
 * example_v2_basic_usage - Demonstrate basic v2 syscall usage
 */
void example_v2_basic_usage(void)
{
    fb_print("=== V2 Execution-Centric Syscall Examples ===\n");
    
    // Example 1: Query system time using v2 syscall
    uint64_t timestamp;
    uint64_t result = syscall(1006, 0, &timestamp);  // SYS_V2_TIME_QUERY = 1006
    
    if (result == ESYS_V2_SUCCESS) {
        fb_print("Current timestamp: ");
        fb_print_hex(timestamp);
        fb_print("\n");
    } else {
        fb_print("Time query failed\n");
    }
    
    // Example 2: Test capability system
    capability_token_t test_cap = {
        .id = 0,
        .permissions = CAPABILITY_PERM_READ,
        .resource_type = CAPABILITY_RESOURCE_MEMORY
    };
    
    uint64_t ctx = 1001;  // Test execution context
    result = syscall(1007, ctx, &test_cap);  // SYS_V2_CAPABILITY_BIND = 1007
    
    if (result == ESYS_V2_SUCCESS) {
        fb_print("Capability bind successful, ID: ");
        fb_print_int(test_cap.id);
        fb_print("\n");
        
        // Revoke the capability
        syscall(1008, test_cap.id);  // SYS_V2_CAPABILITY_REVOKE = 1008
        fb_print("Capability revoked\n");
    } else {
        fb_print("Capability bind failed\n");
    }
    
    fb_print("V2 basic usage complete\n");
}

// ============================================================================
// EXAMPLE 2: FILE OPERATIONS VIA RING3 VFS
// ============================================================================

/**
 * example_v2_file_operations - Demonstrate v2 file operations via Ring3 VFS
 */
void example_v2_file_operations(void)
{
    fb_print("=== V2 File Operations (Ring3 VFS) ===\n");
    
    // Note: In v2, file operations go through Ring3 VFS with capabilities
    // This is a simplified example showing the concept
    
    // Step 1: Create file capability
    capability_token_t file_cap = {
        .id = 0,
        .permissions = CAPABILITY_PERM_READ,
        .resource_type = CAPABILITY_RESOURCE_FILE
    };
    
    // Step 2: Bind capability to execution context
    uint64_t ctx = 1001;
    uint64_t result = syscall(1007, ctx, &file_cap);  // SYS_V2_CAPABILITY_BIND
    
    if (result == ESYS_V2_SUCCESS) {
        fb_print("File capability bound, ID: ");
        fb_print_int(file_cap.id);
        fb_print("\n");
        
        // Step 3: File operations would go through Ring3 VFS
        // (This is conceptual - actual Ring3 VFS implementation in Phase 2.2)
        fb_print("File operations would use Ring3 VFS with capability\n");
        
        // Step 4: Revoke capability when done
        syscall(1008, file_cap.id);  // SYS_V2_CAPABILITY_REVOKE
        fb_print("File capability revoked\n");
    } else {
        fb_print("Failed to bind file capability\n");
    }
}

// ============================================================================
// EXAMPLE 3: MEMORY MANAGEMENT VIA V2 SYSCALLS
// ============================================================================

/**
 * example_v2_memory_management - Demonstrate v2 memory management
 */
void example_v2_memory_management(void)
{
    fb_print("=== V2 Memory Management ===\n");
    
    // Step 1: Create memory capability
    capability_token_t mem_cap = {
        .id = 0,
        .permissions = CAPABILITY_PERM_READ_WRITE,
        .resource_type = CAPABILITY_RESOURCE_MEMORY
    };
    
    // Step 2: Bind memory capability
    uint64_t ctx = 1001;
    uint64_t result = syscall(1007, ctx, &mem_cap);  // SYS_V2_CAPABILITY_BIND
    
    if (result == ESYS_V2_SUCCESS) {
        fb_print("Memory capability bound, ID: ");
        fb_print_int(mem_cap.id);
        fb_print("\n");
        
        // Step 3: Map memory using v2 syscall
        uint64_t virt_addr = 0x200000;
        uint64_t phys_addr = 0x100000;
        uint64_t flags = 0x03;  // Read/Write flags
        
        result = syscall(1000, virt_addr, phys_addr, flags);  // SYS_V2_MAP_MEMORY
        
        if (result == ESYS_V2_SUCCESS) {
            fb_print("Memory mapped: virt=0x");
            fb_print_hex(virt_addr);
            fb_print(" phys=0x");
            fb_print_hex(phys_addr);
            fb_print("\n");
            
            // Step 4: Unmap memory when done
            result = syscall(1001, virt_addr, 4096);  // SYS_V2_UNMAP_MEMORY
            if (result == ESYS_V2_SUCCESS) {
                fb_print("Memory unmapped successfully\n");
            }
        } else {
            fb_print("Memory mapping failed\n");
        }
        
        // Step 5: Revoke memory capability
        syscall(1008, mem_cap.id);  // SYS_V2_CAPABILITY_REVOKE
        fb_print("Memory capability revoked\n");
    } else {
        fb_print("Failed to bind memory capability\n");
    }
}

// ============================================================================
// EXAMPLE 4: EXECUTION CONTEXT MANAGEMENT (V2 ONLY)
// ============================================================================

/**
 * example_v2_execution_management - Demonstrate v2 execution management
 */
void example_v2_execution_management(void)
{
    fb_print("=== V2 Execution Management ===\n");
    
    // Example: Submit a simple execution graph
    // Note: This is conceptual - actual BCIB graphs implemented in Phase 2.3
    
    // Create a simple execution graph (placeholder)
    struct simple_execution_graph {
        uint32_t operation_type;
        uint64_t parameter1;
        uint64_t parameter2;
    } graph = {
        .operation_type = 1,  // Simple computation
        .parameter1 = 42,
        .parameter2 = 24
    };
    
    uint64_t ctx = 1001;
    uint64_t exec_id = syscall(1003, &graph, sizeof(graph), ctx);  // SYS_V2_SUBMIT_EXECUTION
    
    if (exec_id > 0) {
        fb_print("Execution submitted, ID: ");
        fb_print_int(exec_id);
        fb_print("\n");
        
        // Wait for execution result
        uint64_t timeout = 5000;  // 5 seconds
        uint64_t result = syscall(1004, exec_id, timeout);  // SYS_V2_WAIT_RESULT
        
        if (result == ESYS_V2_SUCCESS) {
            fb_print("Execution completed successfully\n");
        } else if (result == ESYS_V2_TIMEOUT) {
            fb_print("Execution timed out\n");
        } else {
            fb_print("Execution failed\n");
        }
    } else {
        fb_print("Failed to submit execution\n");
    }
}

/**
 * example_v2_context_switching - Demonstrate v2 context switching
 */
void example_v2_context_switching(void)
{
    fb_print("=== V2 Context Switching ===\n");
    
    uint64_t old_ctx = 1001;
    uint64_t new_ctx = 1002;
    
    uint64_t result = syscall(1002, old_ctx, new_ctx);  // SYS_V2_SWITCH_CONTEXT
    
    if (result == ESYS_V2_SUCCESS) {
        fb_print("Context switch successful: ");
        fb_print_int(old_ctx);
        fb_print(" -> ");
        fb_print_int(new_ctx);
        fb_print("\n");
    } else {
        fb_print("Context switch failed\n");
    }
}

// ============================================================================
// EXAMPLE 5: ERROR HANDLING PATTERNS
// ============================================================================

/**
 * example_v2_error_handling - Demonstrate v2 error handling patterns
 */
void example_v2_error_handling(void)
{
    fb_print("=== V2 Error Handling ===\n");
    
    // Test invalid syscall number
    uint64_t result = syscall(9999, 0, 0, 0, 0);  // Invalid syscall number
    
    fb_print("Invalid syscall result: ");
    fb_print_int((int64_t)result);
    fb_print(" (expected -38 = -ENOSYS)\n");
    
    // Test invalid parameters
    result = syscall(1007, 0, NULL);  // SYS_V2_CAPABILITY_BIND with invalid params
    
    if (result == (uint64_t)ESYS_V2_INVALID_PARAM) {
        fb_print("Invalid parameter error handled correctly\n");
    } else {
        fb_print("Unexpected error code: ");
        fb_print_int((int64_t)result);
        fb_print("\n");
    }
    
    // Test capability not found
    result = syscall(1008, 99999);  // SYS_V2_CAPABILITY_REVOKE with non-existent ID
    
    if (result == (uint64_t)ESYS_V2_NO_CAPABILITY) {
        fb_print("Capability not found error handled correctly\n");
    } else {
        fb_print("Unexpected error code for missing capability: ");
        fb_print_int((int64_t)result);
        fb_print("\n");
    }
}

// ============================================================================
// MAIN EXAMPLE RUNNER
// ============================================================================

/**
 * run_all_syscall_examples - Run all available syscall examples
 */
void run_all_syscall_examples(void)
{
    fb_print("\n");
    fb_print("========================================\n");
    fb_print("AykenOS Execution-Centric Syscall Examples\n");
    fb_print("Phase 2.5 - Legacy POSIX Syscalls Removed\n");
    fb_print("========================================\n");
    
    example_v2_basic_usage();
    example_v2_file_operations();
    example_v2_memory_management();
    example_v2_execution_management();
    example_v2_context_switching();
    example_v2_error_handling();
    fb_print("\n");
    
    fb_print("========================================\n");
    fb_print("All execution-centric syscall examples completed\n");
    fb_print("========================================\n");
    fb_print("\n");
}

// ============================================================================
// UTILITY FUNCTIONS FOR EXAMPLES
// ============================================================================

/**
 * demonstrate_syscall_numbering - Show the final syscall numbering plan
 */
void demonstrate_syscall_numbering(void)
{
    fb_print("=== Final Syscall Numbering Plan (Phase 2.5) ===\n");
    fb_print("ONLY Execution-Centric Range: 1000-1009\n");
    fb_print("  SYS_V2_MAP_MEMORY = 1000\n");
    fb_print("  SYS_V2_UNMAP_MEMORY = 1001\n");
    fb_print("  SYS_V2_SWITCH_CONTEXT = 1002\n");
    fb_print("  SYS_V2_SUBMIT_EXECUTION = 1003\n");
    fb_print("  SYS_V2_WAIT_RESULT = 1004\n");
    fb_print("  SYS_V2_INTERRUPT_RETURN = 1005\n");
    fb_print("  SYS_V2_TIME_QUERY = 1006\n");
    fb_print("  SYS_V2_CAPABILITY_BIND = 1007\n");
    fb_print("  SYS_V2_CAPABILITY_REVOKE = 1008\n");
    fb_print("  SYS_V2_EXIT = 1009\n");
    fb_print("\n");
    fb_print("Legacy POSIX Range: REMOVED (all return -ENOSYS)\n");
    fb_print("Invalid Range: All other numbers return -ENOSYS\n");
    fb_print("===============================\n");
}

/**
 * test_syscall_dispatcher - Test the final syscall dispatcher
 */
void test_syscall_dispatcher(void)
{
    fb_print("=== Testing Final Syscall Dispatcher ===\n");
    
    // Test v2 range (only valid range)
    fb_print("Testing v2 range (1000-1009) - ONLY valid range:\n");
    uint64_t timestamp;
    uint64_t result = syscall(1006, 0, &timestamp);  // Should work
    fb_print("V2 time query result: ");
    fb_print_int((int64_t)result);
    fb_print("\n");
    
    // Test legacy ranges (should all return -ENOSYS)
    fb_print("Testing legacy ranges (should ALL return -38):\n");
    result = syscall(0, 0, 0, 0, 0);  // Old SYS_read
    fb_print("Legacy syscall 0 result: ");
    fb_print_int((int64_t)result);
    fb_print(" (expected -38)\n");
    
    result = syscall(1, 1, "test", 4);  // Old SYS_write
    fb_print("Legacy syscall 1 result: ");
    fb_print_int((int64_t)result);
    fb_print(" (expected -38)\n");
    
    result = syscall(500, 0, 0, 0, 0);  // Invalid range
    fb_print("Invalid syscall 500 result: ");
    fb_print_int((int64_t)result);
    fb_print(" (expected -38)\n");
    
    fb_print("Final dispatcher test complete\n");
    fb_print("==================================\n");
}

// ============================================================================
// PERFORMANCE TESTING
// ============================================================================

/**
 * compare_syscall_performance - Test v2 syscall performance
 */
void compare_syscall_performance(void)
{
    fb_print("=== V2 Syscall Performance Testing ===\n");
    
    const int iterations = 1000;
    
    // Measure v2 syscall performance (time query)
    fb_print("Measuring v2 syscall performance...\n");
    uint64_t timestamp;
    for (int i = 0; i < iterations; i++) {
        syscall(1006, 0, &timestamp);  // v2 time query
    }
    fb_print("V2 syscalls completed\n");
    
    fb_print("Performance testing complete\n");
    fb_print("(Actual timing requires timer implementation)\n");
    fb_print("======================================\n");
}

// ============================================================================
// EXAMPLE ENTRY POINT
// ============================================================================

/**
 * syscall_examples_main - Main entry point for syscall examples
 * 
 * This function can be called from the kernel to demonstrate
 * the execution-centric syscall interface during system boot or testing.
 */
void syscall_examples_main(void)
{
    demonstrate_syscall_numbering();
    test_syscall_dispatcher();
    run_all_syscall_examples();
    compare_syscall_performance();
    
    fb_print("All execution-centric syscall interface examples completed successfully!\n");
    fb_print("Legacy POSIX syscalls have been completely removed.\n");
}
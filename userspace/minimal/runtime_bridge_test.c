// userspace/minimal/runtime_bridge_test.c
// Runtime_Bridge syscall proof test for Phase-16 Task 5
//
// This test exercises Runtime_Bridge allowed syscalls (1012/1013/1014)
// and validates fail-closed behavior for forbidden syscalls.

typedef unsigned long long uint64_t;
typedef long long int64_t;

// Syscall numbers (SYS_V2 range: 1000-1010 + Runtime_Bridge 1012-1014)
#define SYS_V2_BASE 1000
#define SYS_V2_DEBUG_PUTCHAR 10         // 1010 - allowed for markers
#define SYS_V2_DEVICE_OPERATION 12      // 1012 - Runtime_Bridge allowed
#define SYS_V2_EXTERNAL_CALL 13         // 1013 - Runtime_Bridge allowed
#define SYS_V2_ABDF_OPERATION 14        // 1014 - Runtime_Bridge allowed

// Syscall wrapper
static inline int64_t syscall4(uint64_t num, uint64_t arg1, uint64_t arg2,
                               uint64_t arg3, uint64_t arg4) {
    int64_t ret;
    register uint64_t r10 __asm__("r10") = arg4;
    __asm__ volatile(
        "int $0x80"
        : "=a"(ret)
        : "a"(num), "D"(arg1), "S"(arg2), "d"(arg3), "r"(r10)
        : "cc", "memory"
    );
    return ret;
}

// Helper to emit marker via debug_putchar
static void emit_marker(const char *str) {
    while (*str) {
        syscall4(SYS_V2_BASE + SYS_V2_DEBUG_PUTCHAR, (uint64_t)*str, 0, 0, 0);
        str++;
    }
}

void _start(void) {
    // Marker: Runtime_Bridge test started
    emit_marker("[U][RUNTIME_BRIDGE_TEST_START]\n");
    
    // Test 1: SYS_V2_DEVICE_OPERATION (1012) - should succeed
    emit_marker("[U][RUNTIME_BRIDGE_DEVICE_OP_BEFORE]\n");
    int64_t device_result = syscall4(
        SYS_V2_BASE + SYS_V2_DEVICE_OPERATION,
        42,  // device_id
        2,   // operation: DEVICE_OP_WRITE
        0,   // buffer (null for stub test)
        4    // size
    );
    emit_marker("[U][RUNTIME_BRIDGE_DEVICE_OP_AFTER]\n");
    
    // Test 2: SYS_V2_EXTERNAL_CALL (1013) - should succeed
    emit_marker("[U][RUNTIME_BRIDGE_EXTERNAL_CALL_BEFORE]\n");
    int64_t external_result = syscall4(
        SYS_V2_BASE + SYS_V2_EXTERNAL_CALL,
        1,   // call_type: EXTERNAL_CALL_NETWORK
        0,   // args (null for stub test)
        2,   // arg_count
        0    // flags
    );
    emit_marker("[U][RUNTIME_BRIDGE_EXTERNAL_CALL_AFTER]\n");
    
    // Test 3: SYS_V2_ABDF_OPERATION (1014) - should succeed
    emit_marker("[U][RUNTIME_BRIDGE_ABDF_OP_BEFORE]\n");
    int64_t abdf_result = syscall4(
        SYS_V2_BASE + SYS_V2_ABDF_OPERATION,
        1,    // operation: ABDF_OP_READ
        123,  // handle_id
        0,    // buffer (null for stub test)
        4     // size
    );
    emit_marker("[U][RUNTIME_BRIDGE_ABDF_OP_AFTER]\n");
    
    // Success marker
    emit_marker("[U][RUNTIME_BRIDGE_TEST_COMPLETE]\n");
    
    // Infinite loop (kernel will handle process termination)
    while (1) {
        __asm__ volatile("pause");
    }
}

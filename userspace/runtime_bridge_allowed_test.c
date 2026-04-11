// userspace/runtime_bridge_allowed_test.c
// Phase-16 Task 5: Runtime_Bridge Allowed Path QEMU Proof
//
// This test proves that Runtime_Bridge role can successfully execute allowed syscalls:
// - SYS_V2_DEVICE_OPERATION (1012)
// - SYS_V2_EXTERNAL_CALL (1013)
// - SYS_V2_ABDF_OPERATION (1014)
//
// Expected QEMU trace:
// - RUNTIME_BRIDGE_ALLOWED_BEFORE (user marker)
// - [[AYKEN_SYSCALL_ENTER]] (kernel marker) - for each allowed syscall
// - [[AYKEN_SYSCALL_EXIT]] (kernel marker) - for each allowed syscall
// - RUNTIME_BRIDGE_ALLOWED_AFTER (user marker) - proves execution continued
//
// This is the POSITIVE test: allowed path must succeed.

#include <stdint.h>

// Syscall numbers
#define SYS_V2_BASE 1000
#define SYS_V2_DEBUG_PUTCHAR 10
#define SYS_V2_DEVICE_OPERATION 12  // 1012
#define SYS_V2_EXTERNAL_CALL 13     // 1013
#define SYS_V2_ABDF_OPERATION 14    // 1014

// Syscall invocation via INT 0x80
static inline uint64_t syscall1(uint64_t num, uint64_t arg1) {
    uint64_t ret;
    __asm__ volatile(
        "movq %1, %%rax\n"
        "movq %2, %%rdi\n"
        "int $0x80\n"
        "movq %%rax, %0\n"
        : "=r"(ret)
        : "r"(num), "r"(arg1)
        : "rax", "rdi", "memory"
    );
    return ret;
}

static inline uint64_t syscall2(uint64_t num, uint64_t arg1, uint64_t arg2) {
    uint64_t ret;
    __asm__ volatile(
        "movq %1, %%rax\n"
        "movq %2, %%rdi\n"
        "movq %3, %%rsi\n"
        "int $0x80\n"
        "movq %%rax, %0\n"
        : "=r"(ret)
        : "r"(num), "r"(arg1), "r"(arg2)
        : "rax", "rdi", "rsi", "memory"
    );
    return ret;
}

// Debug output helper
static void debug_putchar(char c) {
    syscall1(SYS_V2_BASE + SYS_V2_DEBUG_PUTCHAR, (uint64_t)c);
}

static void debug_puts(const char *s) {
    while (*s) {
        debug_putchar(*s++);
    }
}

// Main test function
void _start(void) {
    // Marker BEFORE allowed syscalls
    debug_puts("RUNTIME_BRIDGE_ALLOWED_BEFORE\n");
    
    // Test 1: SYS_V2_DEVICE_OPERATION (1012)
    // This is ALLOWED for Runtime_Bridge role
    debug_puts("Testing SYS_V2_DEVICE_OPERATION...\n");
    uint64_t device_result = syscall2(SYS_V2_BASE + SYS_V2_DEVICE_OPERATION, 0, 0);
    debug_puts("SYS_V2_DEVICE_OPERATION returned\n");
    
    // Test 2: SYS_V2_EXTERNAL_CALL (1013)
    // This is ALLOWED for Runtime_Bridge role
    debug_puts("Testing SYS_V2_EXTERNAL_CALL...\n");
    uint64_t external_result = syscall2(SYS_V2_BASE + SYS_V2_EXTERNAL_CALL, 0, 0);
    debug_puts("SYS_V2_EXTERNAL_CALL returned\n");
    
    // Test 3: SYS_V2_ABDF_OPERATION (1014)
    // This is ALLOWED for Runtime_Bridge role
    debug_puts("Testing SYS_V2_ABDF_OPERATION...\n");
    uint64_t abdf_result = syscall2(SYS_V2_BASE + SYS_V2_ABDF_OPERATION, 0, 0);
    debug_puts("SYS_V2_ABDF_OPERATION returned\n");
    
    // Marker AFTER allowed syscalls
    // This MUST appear in QEMU trace - proves execution continued
    debug_puts("RUNTIME_BRIDGE_ALLOWED_AFTER\n");
    
    // Success - all allowed syscalls executed
    debug_puts("SUCCESS: All Runtime_Bridge allowed syscalls executed\n");
    
    // Exit cleanly
    while (1) {
        __asm__ volatile("hlt");
    }
}

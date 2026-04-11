// userspace/runtime_bridge_forbidden_test.c
// Phase-16 Task 5: Runtime_Bridge Forbidden Path QEMU Proof
//
// This test proves that Runtime_Bridge role CANNOT execute forbidden syscalls:
// - SYS_V2_SUBMIT_EXECUTION (1003) is FORBIDDEN for Runtime_Bridge
//
// Expected QEMU trace:
// - RUNTIME_BRIDGE_FORBIDDEN_BEFORE (user marker)
// - [[AYKEN_SYSCALL_ENTER]] (kernel marker)
// - [[AYKEN_BOUNDARY_KILL]] (kernel marker)
// - RUNTIME_BRIDGE_FORBIDDEN_AFTER (NEVER appears - proves hard fail-closed)
//
// This is the NEGATIVE test: forbidden path must terminate with hard fail-closed.

#include <stdint.h>

// Syscall numbers
#define SYS_V2_BASE 1000
#define SYS_V2_SUBMIT_EXECUTION 3   // 1003 - FORBIDDEN for Runtime_Bridge
#define SYS_V2_DEBUG_PUTCHAR 10

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
    // Marker BEFORE forbidden syscall
    // This MUST appear in QEMU trace
    debug_puts("RUNTIME_BRIDGE_FORBIDDEN_BEFORE\n");
    
    // Attempt forbidden syscall for Runtime_Bridge role
    // Runtime_Bridge is allowed: 1012, 1013, 1014
    // SYS_V2_SUBMIT_EXECUTION (1003) is FORBIDDEN for Runtime_Bridge
    //
    // Expected kernel behavior:
    // 1. syscall_handler receives INT 0x80
    // 2. Routes to syscall_v2_hardened_handler
    // 3. boundary_validate_syscall() detects role violation
    // 4. boundary_fail_closed_termination() executes:
    //    - Logs violation
    //    - Marks process ZOMBIE
    //    - Removes from scheduler
    //    - cli + hlt (NEVER RETURNS)
    debug_puts("Attempting SYS_V2_SUBMIT_EXECUTION (FORBIDDEN)...\n");
    syscall2(SYS_V2_BASE + SYS_V2_SUBMIT_EXECUTION, 0, 0);
    
    // Marker AFTER forbidden syscall
    // This MUST NEVER appear in QEMU trace
    // If this appears, hard fail-closed is BROKEN
    debug_puts("RUNTIME_BRIDGE_FORBIDDEN_AFTER\n");
    
    // If we reach here, fail-closed is broken
    debug_puts("CRITICAL: Execution continued after violation!\n");
    
    // Infinite loop (should never reach)
    while (1) {
        __asm__ volatile("hlt");
    }
}

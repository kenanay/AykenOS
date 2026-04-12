// userspace/runtime_bridge_forbidden_test.c
// Phase-16 Task 5: Runtime_Bridge Forbidden Path QEMU Proof
//
// This test proves that Runtime_Bridge role CANNOT execute forbidden syscalls:
// - SYS_V2_SUBMIT_EXECUTION (1003) is FORBIDDEN for Runtime_Bridge
//
// Expected QEMU trace:
// - [[AYKEN_RUNTIME_BRIDGE_EXTERNAL_CALL]] for the preflight allowed syscall
// - [[AYKEN_SYSCALL_ENTER]] for the forbidden syscall
// - [[AYKEN_BOUNDARY_KILL]] (kernel marker)
// - no post-violation Runtime_Bridge handler marker
//
// This is the NEGATIVE test: forbidden path must terminate with hard fail-closed.

// Freestanding types
typedef unsigned long long uint64_t;

// Syscall numbers
#define SYS_V2_BASE 1000
#define SYS_V2_SUBMIT_EXECUTION 3   // 1003 - FORBIDDEN for Runtime_Bridge
#define SYS_V2_EXTERNAL_CALL 13     // 1013 - ALLOWED preflight/post-violation probe

static volatile uint64_t external_args[2] = {0xC33C, 0xD44D};
static volatile uint64_t result_guard;

// Syscall invocation via INT 0x80. The fourth argument follows the kernel's
// x86_64 calling convention and is placed in r10.
static inline uint64_t syscall4(uint64_t num, uint64_t arg1, uint64_t arg2,
                                uint64_t arg3, uint64_t arg4) {
    uint64_t ret;
    register uint64_t r10 __asm__("r10") = arg4;
    __asm__ volatile(
        "int $0x80"
        : "=a"(ret)
        : "a"(num), "D"(arg1), "S"(arg2), "d"(arg3), "r"(r10)
        : "cc", "memory"
    );
    return ret;
}

// Main test function
void _start(void) {
    result_guard = 0xDEADBEEF;

    // Allowed preflight proves the Runtime_Bridge test artifact started without
    // relying on SYS_V2_DEBUG_PUTCHAR, which Runtime_Bridge must not need.
    result_guard ^= syscall4(
        SYS_V2_BASE + SYS_V2_EXTERNAL_CALL,
        1,
        (uint64_t)external_args,
        2,
        0
    );
    
    // Attempt forbidden syscall for Runtime_Bridge role
    // Runtime_Bridge is allowed: 1012, 1013, 1014
    // SYS_V2_SUBMIT_EXECUTION (1003) is FORBIDDEN for Runtime_Bridge
    //
    // Expected kernel behavior:
    // 1. syscall_handler receives INT 0x80
    // 2. Routes to syscall_v2_hardened_handler (with index = 1003 - 1000 = 3)
    // 3. boundary_validate_syscall() detects role violation
    // 4. boundary_fail_closed_termination() executes:
    //    - Logs violation
    //    - Marks process ZOMBIE
    //    - Removes from scheduler
    //    - cli + hlt (NEVER RETURNS)
    syscall4(SYS_V2_BASE + SYS_V2_SUBMIT_EXECUTION, 0, 0, 0, 0);

    // If execution continues, this second Runtime_Bridge handler marker exposes
    // the fail-closed break without requiring a forbidden debug syscall.
    result_guard = 0xBADBAD;
    syscall4(
        SYS_V2_BASE + SYS_V2_EXTERNAL_CALL,
        1,
        (uint64_t)external_args,
        2,
        0
    );
    
    while (1) {
        __asm__ volatile("pause");
    }
}

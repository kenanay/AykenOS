// userspace/runtime_bridge_allowed_test.c
// Phase-16 Task 5: Runtime_Bridge Allowed Path QEMU Proof
//
// This test proves that Runtime_Bridge role can successfully execute allowed syscalls:
// - SYS_V2_DEVICE_OPERATION (1012)
// - SYS_V2_EXTERNAL_CALL (1013)
// - SYS_V2_ABDF_OPERATION (1014)
//
// Expected QEMU trace:
// - [[AYKEN_SYSCALL_ENTER]] / [[AYKEN_SYSCALL_RETURN]] for each allowed syscall
// - [[AYKEN_RUNTIME_BRIDGE_DEVICE_OP]]
// - [[AYKEN_RUNTIME_BRIDGE_EXTERNAL_CALL]]
// - [[AYKEN_RUNTIME_BRIDGE_ABDF_OP]]
//
// This is the POSITIVE test: allowed path must succeed.

// Freestanding types
typedef unsigned long long uint64_t;

// Syscall numbers
#define SYS_V2_BASE 1000
#define SYS_V2_DEBUG_PUTCHAR 10
#define SYS_V2_DEVICE_OPERATION 12  // 1012
#define SYS_V2_EXTERNAL_CALL 13     // 1013
#define SYS_V2_ABDF_OPERATION 14    // 1014

static volatile uint64_t device_buffer[4];
static volatile uint64_t external_args[2] = {0xA11A, 0xB22B};
static volatile uint64_t abdf_data[4];
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
    // Test 1: SYS_V2_DEVICE_OPERATION (1012)
    // This is ALLOWED for Runtime_Bridge role
    uint64_t device_result = syscall4(
        SYS_V2_BASE + SYS_V2_DEVICE_OPERATION,
        42,  // device id: unique proof marker input
        2,   // DEVICE_OP_WRITE: emits kernel-side handler marker
        (uint64_t)device_buffer,
        4
    );
    
    // Test 2: SYS_V2_EXTERNAL_CALL (1013)
    // This is ALLOWED for Runtime_Bridge role
    uint64_t external_result = syscall4(
        SYS_V2_BASE + SYS_V2_EXTERNAL_CALL,
        1,  // EXTERNAL_CALL_NETWORK
        (uint64_t)external_args,
        2,
        0
    );
    
    // Test 3: SYS_V2_ABDF_OPERATION (1014)
    // This is ALLOWED for Runtime_Bridge role
    uint64_t abdf_result = syscall4(
        SYS_V2_BASE + SYS_V2_ABDF_OPERATION,
        1,    // ABDF_OP_READ
        123,  // handle id: unique proof marker input
        (uint64_t)abdf_data,
        4
    );

    result_guard = device_result ^ external_result ^ abdf_result ^
                   device_buffer[0] ^ abdf_data[0];
    
    while (1) {
        __asm__ volatile("pause");
    }
}

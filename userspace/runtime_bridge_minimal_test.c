// userspace/runtime_bridge_minimal_test.c
// Minimal test to prove kernel executes user code

typedef unsigned long long uint64_t;

#define SYS_V2_BASE 1000
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
        : "cc", "memory"
    );
    return ret;
}

static void debug_putchar(char c) {
    syscall1(SYS_V2_BASE + SYS_V2_DEBUG_PUTCHAR, (uint64_t)c);
}

static void debug_puts(const char *s) {
    while (*s) {
        debug_putchar(*s++);
    }
}

void _start(void) {
    // PROOF: User code is executing
    debug_puts("[[USER_CODE_EXECUTING]]\n");
    
    // Infinite loop
    while (1) {
        __asm__ volatile("hlt");
    }
}

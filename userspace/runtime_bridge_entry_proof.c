// Ultra-minimal payload: ONLY proves entry
// No syscalls, no logic - just a unique fingerprint marker

typedef unsigned long long uint64_t;

#define SYS_V2_BASE 1000
#define SYS_V2_DEBUG_PUTCHAR 10

static inline uint64_t syscall1(uint64_t num, uint64_t arg1) {
    uint64_t ret;
    __asm__ volatile(
        "int $0x80"
        : "=a"(ret)
        : "a"(num), "D"(arg1)
        : "cc", "memory"
    );
    return ret;
}

static void putchar(char c) {
    syscall1(SYS_V2_BASE + SYS_V2_DEBUG_PUTCHAR, (uint64_t)c);
}

static void puts(const char* s) {
    while (*s) {
        putchar(*s++);
    }
}

void _start(void) {
    // UNIQUE FINGERPRINT - cannot be confused with anything else
    puts("[RB_PAYLOAD_V1_ENTRY]\n");
    
    // Infinite loop to keep process alive
    while (1) {
        __asm__ volatile("pause");
    }
}

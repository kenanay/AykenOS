// userspace/tests/gate3_ring3_sched_hint/main.c
// Gate-3: Ring3 Runtime Validation - Minimal Test
//
// Purpose:
//   Prove Ring3 execution is real and can communicate with Ring0.
//
// Success Criteria:
//   - Ring3 code executes (_start reached)
//   - Ring3 can call syscalls (debug_putchar)
//   - Ring0 receives syscall and emits [[AYKEN_RING3_OK]]
//
// Copyright © 2026 Kenan AY

// Minimal Ring3 entry point (no libc)
void _start(void) {
    // Gate-3: Ring3 → Ring0 communication proof
    // Use SYS_V2_DEBUG_PUTCHAR (1010) to emit marker
    // This proves Ring3 can execute and call syscalls
    
    // Emit marker string: "R3OK\n"
    const char marker[] = "R3OK\n";
    for (int i = 0; marker[i] != '\0'; i++) {
        __asm__ volatile(
            "movq $1010, %%rax\n"      // SYS_V2_DEBUG_PUTCHAR
            "movq %0, %%rdi\n"         // character
            "int $0x80\n"              // syscall
            :
            : "r"((unsigned long)marker[i])
            : "rax", "rdi", "memory"
        );
    }
    
    // Infinite loop (kernel will preempt)
    for (;;) {
        __asm__ volatile("pause");
    }
}

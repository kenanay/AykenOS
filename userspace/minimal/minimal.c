// Minimal Ring3 userspace entry for Phase 10-A proof.
// No writable globals and no BSS; execution proof is INT3 (#BP).

__attribute__((noreturn)) void _start(void)
{
    __asm__ volatile("int3");

    // If #BP handler ever returns, stay in user mode without privileged ops.
    for (;;) {
        __asm__ volatile("pause");
    }
}

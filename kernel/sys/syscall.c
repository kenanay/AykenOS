// kernel/sys/syscall.c
// System call stub implementation

#include <stdint.h>

void syscall_init(void)
{
    // TODO: Syscall initialization
    // - Setup syscall MSR (x86_64)
    // - Register syscall handler
    // - Setup syscall table
}

// Syscall handler (will be called from assembly)
uint64_t syscall_handler(uint64_t syscall_num, uint64_t arg1, 
                         uint64_t arg2, uint64_t arg3, uint64_t arg4)
{
    // TODO: Implement syscall dispatch
    (void)syscall_num;
    (void)arg1;
    (void)arg2;
    (void)arg3;
    (void)arg4;
    
    return (uint64_t)-1; // ENOSYS
}

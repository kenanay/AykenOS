// kernel/include/syscall.h
#ifndef AYKEN_SYSCALL_H
#define AYKEN_SYSCALL_H

#include <stdint.h>

// Syscall initialization
void syscall_init(void);

// Syscall handler (called from assembly)
uint64_t syscall_handler(uint64_t syscall_num, uint64_t arg1, 
                         uint64_t arg2, uint64_t arg3, uint64_t arg4);

#endif // AYKEN_SYSCALL_H

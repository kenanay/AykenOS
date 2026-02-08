#ifndef AYKEN_KERNEL_LIMITS_H
#define AYKEN_KERNEL_LIMITS_H

#include <stdint.h>

// Higher half base for kernel virtual addresses
#define KERNEL_VIRT_BASE 0xFFFFFFFF80000000ULL
// Bootloader identity-maps the first 1GB; use this for early phys access.
#define AYKEN_IDENTITY_MAP_SIZE (1ULL << 30)

// Default user address space layout helpers
#define USER_TEXT_BASE   0x0000000000400000ULL
#define USER_STACK_TOP   0x0000000000800000ULL

#endif // AYKEN_KERNEL_LIMITS_H

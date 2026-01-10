#ifndef AYKEN_BOOT_FLAGS_H
#define AYKEN_BOOT_FLAGS_H

#include <stdint.h>

/* Boot ABI version expected by current kernel */
#define AYKEN_BOOT_ABI_VERSION 1u

/*
 * Boot flags bitmask (shared between bootloader and kernel)
 *
 * Usage:
 *  - Bootloader sets bits in `boot_info.flags` to indicate provided
 *    capabilities or discovered platform features.
 *  - Kernel should only act on known bits and ignore unknown bits
 *    (but may log or warn about them).
 *
 * Example:
 *   boot.flags = 0;
 *   if (gop_available) boot.flags |= AYKEN_BOOT_FLAG_FB_VALID;
 *   if (paging_ready) boot.flags |= AYKEN_BOOT_FLAG_PAGING_READY;
 *
 */
#define AYKEN_BOOT_FLAG_PAGING_READY   (1u << 0) /* Bootloader set if paging/PML4 prepared */
#define AYKEN_BOOT_FLAG_FB_VALID       (1u << 1) /* Framebuffer (GOP) info valid */
#define AYKEN_BOOT_FLAG_ACPI_PRESENT   (1u << 2) /* ACPI RSDP present */
#define AYKEN_BOOT_FLAG_SMP_AVAILABLE  (1u << 3) /* APIC/Multiple CPU table found */

/* Mask of flags the current kernel recognizes */
#define AYKEN_BOOT_KNOWN_FLAGS ( \
    AYKEN_BOOT_FLAG_PAGING_READY | \
    AYKEN_BOOT_FLAG_FB_VALID     | \
    AYKEN_BOOT_FLAG_ACPI_PRESENT | \
    AYKEN_BOOT_FLAG_SMP_AVAILABLE )

/*
 * Notes for kernel implementers:
 *  - Verify `boot_info.abi_version` against `AYKEN_BOOT_ABI_VERSION`.
 *  - Treat unknown flags as informational: `unknown = flags & ~AYKEN_BOOT_KNOWN_FLAGS`.
 *  - Keep the ABI number in sync between bootloader and kernel; bump
 *    `AYKEN_BOOT_ABI_VERSION` on breaking changes to `ayken_boot_info_t`.
 */

#endif // AYKEN_BOOT_FLAGS_H

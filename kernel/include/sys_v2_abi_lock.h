#ifndef AYKEN_SYS_V2_ABI_LOCK_H
#define AYKEN_SYS_V2_ABI_LOCK_H

/*
 * AykenOS Syscall v2 ABI Immutability Guard
 *
 * Constitutional rule:
 * - Syscall base must remain fixed.
 * - Syscall surface size must remain fixed.
 * - Count/index arithmetic must remain consistent.
 */

#define AYKEN_SYS_V2_EXPECTED_BASE       1000
#define AYKEN_SYS_V2_EXPECTED_MAX_INDEX  10
#define AYKEN_SYS_V2_EXPECTED_NR         11

_Static_assert(SYS_V2_BASE == AYKEN_SYS_V2_EXPECTED_BASE,
               "SYS_V2_BASE modified: ABI violation");

_Static_assert(SYS_V2_MAX_INDEX == AYKEN_SYS_V2_EXPECTED_MAX_INDEX,
               "SYS_V2_MAX_INDEX modified: ABI surface changed");

_Static_assert(SYS_V2_NR == AYKEN_SYS_V2_EXPECTED_NR,
               "SYS_V2_NR mismatch: syscall count changed");

_Static_assert(SYS_V2_NR == (SYS_V2_MAX_INDEX + 1),
               "SYS_V2_NR inconsistent with SYS_V2_MAX_INDEX");

_Static_assert(SYS_V2_DEBUG_PUTCHAR == SYS_V2_MAX_INDEX,
               "SYS_V2_DEBUG_PUTCHAR must remain the terminal index");

#endif

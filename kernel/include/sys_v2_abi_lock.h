#ifndef AYKEN_SYS_V2_ABI_LOCK_H
#define AYKEN_SYS_V2_ABI_LOCK_H

/*
 * AykenOS Syscall v2 ABI Immutability Guard
 *
 * Constitutional Lock
 * Surface: Execution-Centric Syscall v2
 * Freeze Mode: ACTIVE
 *
 * Locked By: Kenan AY
 * Date: 2026-02-15
 * Authority Class: Constitutional ABI
 * Review Class: ABI Surface Change Required For Modification
 *
 * Constitutional rule:
 * - Syscall base must remain fixed.
 * - Syscall surface size must remain fixed.
 * - Count/index arithmetic must remain consistent.
 *
 * Any modification to syscall numbering or count requires
 * constitutional review and baseline refresh.
 */

#define AYKEN_SYS_V2_EXPECTED_BASE       1000
#define AYKEN_SYS_V2_EXPECTED_MAX_INDEX  10
#define AYKEN_SYS_V2_EXPECTED_NR         11
#define AYKEN_SYS_V2_ABI_SIGNATURE       "Kenan-AY-20260215"
#define AYKEN_SYS_V2_ABI_FINGERPRINT     0xA2C6F4B1u

_Static_assert(SYS_V2_BASE == AYKEN_SYS_V2_EXPECTED_BASE,
               "SYS_V2_BASE modified: ABI violation");

_Static_assert(SYS_V2_MAX_INDEX == AYKEN_SYS_V2_EXPECTED_MAX_INDEX,
               "SYS_V2_MAX_INDEX modified: ABI surface changed");

_Static_assert(SYS_V2_NR == AYKEN_SYS_V2_EXPECTED_NR,
               "SYS_V2_NR mismatch: syscall count changed");

_Static_assert(SYS_V2_NR == (SYS_V2_MAX_INDEX + 1),
               "SYS_V2_NR inconsistent with SYS_V2_MAX_INDEX");

_Static_assert(SYS_V2_LAST == (SYS_V2_BASE + SYS_V2_MAX_INDEX),
               "SYS_V2_LAST inconsistent with SYS_V2_BASE/SYS_V2_MAX_INDEX");

_Static_assert(SYS_V2_MAX_SYSCALL == SYS_V2_MAX_INDEX,
               "SYS_V2_MAX_SYSCALL must match SYS_V2_MAX_INDEX");

_Static_assert(SYS_V2_DEBUG_PUTCHAR == SYS_V2_MAX_INDEX,
               "SYS_V2_DEBUG_PUTCHAR must remain the terminal index");

_Static_assert(sizeof(AYKEN_SYS_V2_ABI_SIGNATURE) > 1,
               "ABI signature missing");

_Static_assert(AYKEN_SYS_V2_ABI_FINGERPRINT == 0xA2C6F4B1u,
               "ABI fingerprint mismatch");

#endif

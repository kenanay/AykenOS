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
 * Date: 2026-03-19
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
#define AYKEN_SYS_V2_EXPECTED_MAX_INDEX  14
#define AYKEN_SYS_V2_EXPECTED_NR         15
#define AYKEN_SYS_V2_ABI_SIGNATURE       "Kenan-AY-20260319-phase16-runtime-bridge"
#define AYKEN_SYS_V2_ABI_FINGERPRINT     0x4B17A514u

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

_Static_assert(SYS_V2_COMPLETE_EXECUTION == 11,
               "SYS_V2_COMPLETE_EXECUTION must remain at index 11");

_Static_assert(SYS_V2_DEVICE_OPERATION == 12,
               "SYS_V2_DEVICE_OPERATION must remain at index 12");

_Static_assert(SYS_V2_EXTERNAL_CALL == 13,
               "SYS_V2_EXTERNAL_CALL must remain at index 13");

_Static_assert(SYS_V2_ABDF_OPERATION == SYS_V2_MAX_INDEX,
               "SYS_V2_ABDF_OPERATION must remain the terminal index");

_Static_assert(sizeof(AYKEN_SYS_V2_ABI_SIGNATURE) > 1,
               "ABI signature missing");

_Static_assert(AYKEN_SYS_V2_ABI_FINGERPRINT == 0x4B17A514u,
               "ABI fingerprint mismatch");

#endif

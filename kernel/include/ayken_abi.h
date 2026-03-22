#ifndef AYKEN_ABI_WRAPPER_H
#define AYKEN_ABI_WRAPPER_H

#include "../../shared/abi/ayken_abi.h"

/*
 * Keep the legacy macro surface visible in this wrapper so CI gates that
 * parse kernel/include/ayken_abi.h directly do not need to resolve includes.
 */
/* ABI version bump: 0x00010000 → 0x00010001
 * Reason: SYS_V2_COMPLETE_EXECUTION (index=11, public=1011) added.
 * Syscall range extended: 1000-1010 → 1000-1011 (12 syscalls).
 * RFC: Phase 10B BCIB execution engine completion — complete_execution
 *      syscall required for execution slot lifecycle finalization.
 * Authority: Kenan AY — approved as part of Phase 10B merge.
 */
#ifndef AYKEN_ABI_VERSION
#define AYKEN_ABI_VERSION 0x00010001u
#endif

#ifndef CTX_R15
#define CTX_R15      0u
#define CTX_R14      8u
#define CTX_R13      16u
#define CTX_R12      24u
#define CTX_RBX      32u
#define CTX_RBP      40u
#define CTX_RIP      48u
#define CTX_RSP      56u
#define CTX_RFLAGS   64u
#define CTX_CR3      72u
#define CTX_CS       80u
#define CTX_SS       82u
#define CTX_RSP0     88u
#define CTX_SIZE     96u
#endif

#ifndef IRQF_R15
#define IRQF_R15      0u
#define IRQF_R14      8u
#define IRQF_R13      16u
#define IRQF_R12      24u
#define IRQF_R11      32u
#define IRQF_R10      40u
#define IRQF_R9       48u
#define IRQF_R8       56u
#define IRQF_RBP      64u
#define IRQF_RDI      72u
#define IRQF_RSI      80u
#define IRQF_RDX      88u
#define IRQF_RCX      96u
#define IRQF_RBX      104u
#define IRQF_RAX      112u
#define IRQF_RIP      120u
#define IRQF_CS       128u
#define IRQF_RFLAGS   136u
#define IRQF_RSP      144u
#define IRQF_SS       152u
#define IRQF_SIZE     160u
#endif

#endif

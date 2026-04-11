#ifndef AYKEN_ABI_H
#define AYKEN_ABI_H

#include <stdint.h>

#define AYKEN_ABI_VERSION 0x00010002u

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

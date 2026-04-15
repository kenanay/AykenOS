#ifndef AYKEN_RING3_CONTRACT_H
#define AYKEN_RING3_CONTRACT_H

/*
 * Frozen Ring3 transition contract shared by C and assembly.
 * Selector values match the current GDT layout and are asserted in gdt_idt.c.
 */
#define AYKEN_RING3_USER_DATA_SELECTOR 0x1B
#define AYKEN_RING3_USER_CODE_SELECTOR 0x23

/* Baseline user RFLAGS: bit1 reserved set, IF forced on. */
#if AYKEN_PHASE16_BCIB_PROOF_TEST == 1
/* PHASE 3B FIX: Set IOPL=3 for Ring3 debugging (test builds only) */
#define AYKEN_RING3_RFLAGS_BASE 0x3202  /* IF=1, IOPL=3 */
#define AYKEN_RING3_RFLAGS_CLEARMASK \
    ((1 << 14) | (1 << 16) | (1 << 17) | (1 << 8))  /* Clear NT, RF, VM, TF, but NOT IOPL */
#else
#define AYKEN_RING3_RFLAGS_BASE 0x202
#define AYKEN_RING3_RFLAGS_CLEARMASK \
    ((3 << 12) | (1 << 14) | (1 << 16) | (1 << 17) | (1 << 8))
#endif

/*
 * Dedicated low-half trampoline VA used by fetch-stub validation builds.
 * The scheduler maps a separate trampoline frame here in both roots; this must
 * not be treated as a general mirror of the higher-half transition page.
 * Keep it above the retained 1GiB identity map so the kernel root can map it
 * without colliding with boot-time low aliases.
 */
#define AYKEN_RING3_TRAMPOLINE_VA 0x0000000040000000ULL

/*
 * Diagnostic second canonical higher-half alias used by fetch probes and as
 * the stage-A CR3-pivot stub in canonical trampoline validation lanes.
 */
#define AYKEN_RING3_SECOND_CANONICAL_PROBE_VA 0xFFFFFFFF80100000ULL

/*
 * Diagnostic third canonical higher-half alias used as the post-CR3 iretq
 * landing page in two-stage canonical trampoline validation lanes, or the
 * fetch-only bridge page in split-stage validation lanes.
 */
#define AYKEN_RING3_THIRD_CANONICAL_PROBE_VA 0xFFFFFFFF80101000ULL

/*
 * Diagnostic fourth canonical higher-half alias used only by split-stage
 * validation lanes where the third page proves fetch and tails into a final
 * iretq-only page.
 */
#define AYKEN_RING3_FOURTH_CANONICAL_PROBE_VA 0xFFFFFFFF80102000ULL

/*
 * Diagnostic fifth canonical higher-half alias used by shifted canonical
 * window lanes where the stage-A/stage-B/stage-C trio moves up by one page to
 * answer whether the failure is specific to the original third-page VA.
 */
#define AYKEN_RING3_FIFTH_CANONICAL_PROBE_VA 0xFFFFFFFF80103000ULL

#ifndef AYKEN_RING3_SHIFT_CANONICAL_WINDOW
#define AYKEN_RING3_SHIFT_CANONICAL_WINDOW 0
#endif

#if defined(AYKEN_RING3_SHIFT_CANONICAL_WINDOW) && (AYKEN_RING3_SHIFT_CANONICAL_WINDOW == 1)
#define AYKEN_RING3_CANONICAL_STAGE_A_VA AYKEN_RING3_THIRD_CANONICAL_PROBE_VA
#define AYKEN_RING3_CANONICAL_STAGE_B_VA AYKEN_RING3_FOURTH_CANONICAL_PROBE_VA
#define AYKEN_RING3_CANONICAL_STAGE_C_VA AYKEN_RING3_FIFTH_CANONICAL_PROBE_VA
#else
#define AYKEN_RING3_CANONICAL_STAGE_A_VA AYKEN_RING3_SECOND_CANONICAL_PROBE_VA
#define AYKEN_RING3_CANONICAL_STAGE_B_VA AYKEN_RING3_THIRD_CANONICAL_PROBE_VA
#define AYKEN_RING3_CANONICAL_STAGE_C_VA AYKEN_RING3_FOURTH_CANONICAL_PROBE_VA
#endif

#endif

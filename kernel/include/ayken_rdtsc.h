// SPDX-License-Identifier: ASAL-1.0
// Copyright (C) 2026 Kenan AY
//
// RDTSC Helper for Performance Profiling
// Authority: Scheduler Performance Regression RCA - Patch H

#ifndef AYKEN_RDTSC_H
#define AYKEN_RDTSC_H

#include <stdint.h>

/**
 * ayken_rdtsc - Read Time-Stamp Counter
 *
 * Returns the current value of the processor's time-stamp counter (TSC).
 * This is a deterministic cycle counter suitable for performance profiling.
 *
 * Note: RDTSC is serializing on modern x86_64 processors when used with
 * LFENCE/MFENCE, but for profiling purposes we use the non-serializing
 * variant to minimize measurement overhead.
 *
 * Returns: 64-bit TSC value
 */
static inline uint64_t ayken_rdtsc(void)
{
    uint32_t lo, hi;
    __asm__ volatile("rdtsc" : "=a"(lo), "=d"(hi));
    return ((uint64_t)hi << 32) | lo;
}

#endif /* AYKEN_RDTSC_H */

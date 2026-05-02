/*
 * execution_marker_injection.h - Phase-17 Marker Validation Injection Harness
 *
 * ⚠️ CRITICAL: TEST-ONLY CODE
 * This file MUST NOT compile into production builds.
 * Top-level guard prevents ANY injection code from production path.
 *
 * Authority: Kenan AY - Architectural Steward
 * Mandate: Test-only guard is NON-NEGOTIABLE
 */

#ifndef EXECUTION_MARKER_INJECTION_H
#define EXECUTION_MARKER_INJECTION_H

/*
 * ⚠️ TOP-LEVEL GUARD: ONLY enabled in test builds
 * Without this flag, NOTHING in this file compiles.
 * This prevents production sızma (contamination).
 */
#if defined(AYKEN_PHASE17_MARKER_INJECTION_TEST) && (AYKEN_PHASE17_MARKER_INJECTION_TEST == 1)

#include "execution_slot.h"

/*
 * Individual injection functions (each with own flag)
 * Only ONE should be enabled at a time during testing
 */

#if defined(AYKEN_MARKER_INJECT_INVALID_ORDER) && (AYKEN_MARKER_INJECT_INVALID_ORDER == 1)
void inject_invalid_order(exec_slot_t *slot);
#endif

#if defined(AYKEN_MARKER_INJECT_DUPLICATE) && (AYKEN_MARKER_INJECT_DUPLICATE == 1)
void inject_duplicate(exec_slot_t *slot);
#endif

#if defined(AYKEN_MARKER_INJECT_MISSING) && (AYKEN_MARKER_INJECT_MISSING == 1)
void inject_missing(exec_slot_t *slot);
#endif

#if defined(AYKEN_MARKER_INJECT_OVERFLOW) && (AYKEN_MARKER_INJECT_OVERFLOW == 1)
void inject_overflow(exec_slot_t *slot);
#endif

#if defined(AYKEN_MARKER_INJECT_STALE_DATA) && (AYKEN_MARKER_INJECT_STALE_DATA == 1)
void inject_stale_data(exec_slot_t *slot);
#endif

#if defined(AYKEN_MARKER_INJECT_CORRUPT_BITMAP) && (AYKEN_MARKER_INJECT_CORRUPT_BITMAP == 1)
void inject_corrupt_bitmap(exec_slot_t *slot);
#endif

#if defined(AYKEN_MARKER_INJECT_PARTIAL_WRITE) && (AYKEN_MARKER_INJECT_PARTIAL_WRITE == 1)
void inject_partial_write(exec_slot_t *slot);
#endif

#endif /* AYKEN_PHASE17_MARKER_INJECTION_TEST */

#endif /* EXECUTION_MARKER_INJECTION_H */

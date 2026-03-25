// kernel/include/barrier.h
// Memory barrier primitives for SMP synchronization
//
// These barriers enforce memory ordering constraints to prevent CPU reordering
// of memory operations across barrier boundaries.

#ifndef AYKEN_BARRIER_H
#define AYKEN_BARRIER_H

/* smp_mb: Full memory barrier
 * 
 * Ensures all memory operations (loads and stores) before the barrier
 * are globally visible before any memory operations after the barrier.
 * 
 * Use case: Critical synchronization points where both read and write
 * ordering must be enforced (e.g., after setting teardown_started flag).
 */
static inline void smp_mb(void)
{
    __asm__ volatile("mfence" ::: "memory");
}

/* smp_wmb: Write memory barrier
 * 
 * Ensures all store operations before the barrier are globally visible
 * before any store operations after the barrier.
 * 
 * Use case: Ensuring writes to data structures happen-before flag updates
 * (e.g., alias_registry_record() writes happen-before teardown_started=1).
 */
static inline void smp_wmb(void)
{
    __asm__ volatile("sfence" ::: "memory");
}

/* smp_rmb: Read memory barrier
 * 
 * Ensures all load operations before the barrier complete before any
 * load operations after the barrier.
 * 
 * Use case: Ensuring flag reads are fresh and not reordered with subsequent
 * reads (e.g., reading teardown_started before checking registry state).
 */
static inline void smp_rmb(void)
{
    __asm__ volatile("lfence" ::: "memory");
}

#endif /* AYKEN_BARRIER_H */

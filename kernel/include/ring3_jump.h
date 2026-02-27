// kernel/include/ring3_jump.h
// ============================================================================
// AykenOS Phase 10-A: Ring3 preparation wrapper
// Prepares embedded Ring3 process for scheduler-managed CPL3 entry.
// ============================================================================

#ifndef RING3_JUMP_H
#define RING3_JUMP_H

// Prepare Ring3 process for scheduler startup.
void jump_to_ring3(void);

#endif // RING3_JUMP_H

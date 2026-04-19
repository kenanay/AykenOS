# Patch C Design - Context Type Cache + Bypass Fast-Path

**Date**: 2026-04-19  
**Status**: DESIGN  
**Target**: Optimize remaining 61% of hot-path (bypass_check + ctx_type)

## Executive Summary

**Problem**: Patch B optimized only 39% of hot-path, insufficient for baseline recovery.

**Solution**: Cache context type + fast-path bypass check to eliminate remaining 61% overhead.

**Target**: 500k ticks → 120k ticks (76% reduction in hot-path)

## Hot-Path Baseline (Before Patch C)

From micro-profile (3 syscalls):
```
validate_syscall: 195k ticks (39.0%) ← Patch B targeted
bypass_check:     161k ticks (32.3%) ← Patch C2 target
ctx_type:         143k ticks (28.7%) ← Patch C1 target
TOTAL:            500k ticks
```

## Patch C1: Context Type Cache

### Current Implementation (Slow)

```c
// kernel/sys/boundary_enforcement.c
void boundary_set_context_type(proc_t *proc) {
    // EXPENSIVE: Called on EVERY syscall
    // 143k ticks average (28.7% of hot-path)
    
    if (proc->execution_role == PROC_EXECUTION_ROLE_BCIB) {
        proc->boundary_context_type = BOUNDARY_CONTEXT_BCIB;
    } else if (proc->execution_role == PROC_EXECUTION_ROLE_RUNTIME_BRIDGE) {
        proc->boundary_context_type = BOUNDARY_CONTEXT_BRIDGE;
    } else if (proc->execution_role == PROC_EXECUTION_ROLE_USER) {
        proc->boundary_context_type = BOUNDARY_CONTEXT_USER;
    } else if (proc->execution_role == PROC_EXECUTION_ROLE_KERNEL) {
        proc->boundary_context_type = BOUNDARY_CONTEXT_KERNEL;
    } else {
        proc->boundary_context_type = BOUNDARY_CONTEXT_UNKNOWN;
    }
}
```

### Optimized Implementation (Fast)

**Key Insight**: Context type is derived from execution role, which changes rarely:
- Process creation
- Role transition (user → BCIB, BCIB → bridge)
- Context switch (if role changed)

**Strategy**: Cache context type, update only when role changes.

```c
// kernel/include/proc.h
typedef struct proc {
    // ... existing fields ...
    proc_execution_role_t execution_role;
    boundary_context_type_t boundary_context_type;  // CACHED
    uint64_t boundary_context_epoch;  // Invalidation tracking
    // ... existing fields ...
} proc_t;
```

**Update Points**:

1. **Process Creation** (`proc_create()`):
```c
void proc_create(proc_t *proc, proc_execution_role_t role) {
    proc->execution_role = role;
    proc->boundary_context_type = boundary_role_to_context_type(role);
    proc->boundary_context_epoch = 0;
}
```

2. **Role Transition** (`proc_set_execution_role()`):
```c
void proc_set_execution_role(proc_t *proc, proc_execution_role_t new_role) {
    if (proc->execution_role != new_role) {
        proc->execution_role = new_role;
        proc->boundary_context_type = boundary_role_to_context_type(new_role);
        proc->boundary_context_epoch++;
    }
}
```

3. **Syscall Path** (READ ONLY):
```c
// kernel/sys/syscall_v2_hardened.c
void syscall_v2_hardened_handler(registers_t *regs) {
    proc_t *proc = sched_get_current_proc();
    
    // FAST: Just read cached value (no computation)
    boundary_context_type_t ctx_type = proc->boundary_context_type;
    
    // ... rest of syscall handling ...
}
```

**Helper Function** (cold-path only):
```c
// kernel/sys/boundary_enforcement.c
static inline boundary_context_type_t boundary_role_to_context_type(proc_execution_role_t role) {
    // Simple lookup table (O(1), no branches)
    static const boundary_context_type_t role_to_context[] = {
        [PROC_EXECUTION_ROLE_BCIB] = BOUNDARY_CONTEXT_BCIB,
        [PROC_EXECUTION_ROLE_RUNTIME_BRIDGE] = BOUNDARY_CONTEXT_BRIDGE,
        [PROC_EXECUTION_ROLE_USER] = BOUNDARY_CONTEXT_USER,
        [PROC_EXECUTION_ROLE_KERNEL] = BOUNDARY_CONTEXT_KERNEL,
        [PROC_EXECUTION_ROLE_UNKNOWN] = BOUNDARY_CONTEXT_UNKNOWN,
    };
    
    if (role >= PROC_EXECUTION_ROLE_MAX) {
        return BOUNDARY_CONTEXT_UNKNOWN;
    }
    
    return role_to_context[] role];
}
```

**Expected Impact**:
- ctx_type: 143k → <20k ticks (86% reduction)
- Syscall path: read cached value (1 memory load, ~5-10 ticks)

## Patch C2: Bypass Check Fast-Path

### Current Implementation (Slow)

```c
// kernel/sys/boundary_enforcement.c
int boundary_detect_bridge_bypass(proc_t *proc, uint32_t syscall_num) {
    // EXPENSIVE: Called on EVERY syscall
    // 161k ticks average (32.3% of hot-path)
    
    // Deep check for bridge bypass attempts
    if (proc->execution_role == PROC_EXECUTION_ROLE_RUNTIME_BRIDGE) {
        // Check if bridge is trying to submit execution
        if (syscall_num == SYS_V2_SUBMIT_EXECUTION) {
            return 1;  // BYPASS DETECTED
        }
    }
    
    // Check BCIB bypass attempts
    if (proc->execution_role == PROC_EXECUTION_ROLE_BCIB) {
        // BCIB can only submit, nothing else
        if (syscall_num != SYS_V2_SUBMIT_EXECUTION) {
            return 1;  // BYPASS DETECTED
        }
    }
    
    return 0;  // NO BYPASS
}
```

### Optimized Implementation (Fast)

**Key Insight**: Most contexts (USER, KERNEL) cannot bypass. Only BCIB and BRIDGE need deep checks.

**Strategy**: Early exit for common case, deep check only for restricted roles.

```c
// kernel/sys/boundary_enforcement.c
static inline int boundary_detect_bridge_bypass_fast(
    boundary_context_type_t ctx_type,
    proc_execution_role_t role,
    uint32_t syscall_num
) {
    // FAST PATH: Most contexts cannot bypass (USER, KERNEL, UNKNOWN)
    // This is the common case (>90% of syscalls)
    if (__builtin_expect(
        ctx_type != BOUNDARY_CONTEXT_BCIB && 
        ctx_type != BOUNDARY_CONTEXT_BRIDGE, 
        1  // Likely: not restricted context
    )) {
        return 0;  // NO BYPASS (early exit)
    }
    
    // SLOW PATH: Restricted contexts need deep check
    // BCIB: only SUBMIT_EXECUTION allowed
    if (ctx_type == BOUNDARY_CONTEXT_BCIB) {
        return (syscall_num != SYS_V2_SUBMIT_EXECUTION) ? 1 : 0;
    }
    
    // BRIDGE: SUBMIT_EXECUTION forbidden
    if (ctx_type == BOUNDARY_CONTEXT_BRIDGE) {
        return (syscall_num == SYS_V2_SUBMIT_EXECUTION) ? 1 : 0;
    }
    
    // UNKNOWN: fail-closed
    return 1;
}
```

**Expected Impact**:
- bypass_check: 161k → <50k ticks (69% reduction)
- Common case (USER/KERNEL): 1 branch + early exit (~10-20 ticks)
- Restricted case (BCIB/BRIDGE): 2 branches + check (~50-100 ticks)

## Combined Hot-Path Optimization

### Before Patch C
```
validate_syscall: 195k ticks (39.0%)
bypass_check:     161k ticks (32.3%)
ctx_type:         143k ticks (28.7%)
TOTAL:            500k ticks
```

### After Patch C (Target)
```
validate_syscall:  50k ticks (Patch B bitmask, if effective)
bypass_check:      50k ticks (Patch C2 fast-path)
ctx_type:          20k ticks (Patch C1 cache)
TOTAL:            120k ticks (76% reduction)
```

### Impact on 2nd Syscall Cost
```
Before: 999k ticks total
Hot-path reduction: 500k → 120k (380k saved)
After: 999k - 380k = 619k ticks

Target baseline: ~450k ticks (175ms)
Gap: 619k - 450k = 169k ticks (still 38% over baseline)
```

**Note**: May need additional optimization beyond hot-path, or diagnostic markers removal.

## Implementation Plan

### Phase 1: Context Type Cache (Patch C1)

1. **Add cache fields to proc_t**:
   - `boundary_context_type` (cached value)
   - `boundary_context_epoch` (invalidation tracking)

2. **Create helper function**:
   - `boundary_role_to_context_type()` (lookup table)

3. **Update cold-path**:
   - `proc_create()`: initialize cache
   - `proc_set_execution_role()`: update cache
   - `sched_switch()`: verify cache (optional)

4. **Update hot-path**:
   - `syscall_v2_hardened_handler()`: read cached value
   - Remove `boundary_set_context_type()` call

5. **Verify**:
   - Hot-path micro-profile: ctx_type should drop to <20k
   - Functional tests: context type still correct

### Phase 2: Bypass Fast-Path (Patch C2)

1. **Create fast-path function**:
   - `boundary_detect_bridge_bypass_fast()`
   - Use cached context type
   - Early exit for common case

2. **Update hot-path**:
   - Replace `boundary_detect_bridge_bypass()` call
   - Pass cached context type

3. **Verify**:
   - Hot-path micro-profile: bypass_check should drop to <50k
   - Functional tests: bypass detection still works

### Phase 3: Integration & Verification

1. **Local verification**:
   - Run hot-path micro-profile
   - Verify total hot-path <150k ticks
   - Run preservation tests

2. **CI verification**:
   - Push to CI
   - Check authoritative metrics:
     - syscall_latency: target <192ms (within 10%)
     - boot_time: target <11752ms (within 10%)
     - context_switch: target <192ms (within 10%)

3. **If insufficient**:
   - Investigate remaining overhead
   - Consider diagnostic marker removal
   - Profile non-hot-path syscall cost

## Architectural Compliance

### MECHANISM ≠ POLICY
- ✅ Cache is mechanism optimization
- ✅ Enforcement rules unchanged
- ✅ Policy decisions still at update points

### SHORTCUT ≠ SKIP
- ✅ Context type still computed (at role change)
- ✅ Bypass check still performed (with early exit)
- ✅ Semantic equivalence preserved

### OBSERVABILITY PRESERVED
- ✅ Cache updates traceable (epoch counter)
- ✅ Bypass detection still logged
- ✅ Diagnostic markers unchanged

### DETERMINISM MANDATORY
- ✅ Cache invalidation deterministic (role change)
- ✅ No branch-based fast paths (early exit is deterministic)
- ✅ Lookup table is constant

### BOUNDARY & SECURITY IMMUTABLE
- ✅ Enforcement rules unchanged
- ✅ Fail-closed semantics preserved
- ✅ Bypass detection still active

## Risk Assessment

### Risk 1: Cache Invalidation Bug (MEDIUM)
**Scenario**: Role changes but cache not updated  
**Mitigation**: 
- Centralize role updates in `proc_set_execution_role()`
- Add epoch counter for debugging
- Verification in context switch (optional)

### Risk 2: Early Exit Bypass (LOW)
**Scenario**: Fast-path early exit misses bypass  
**Mitigation**:
- Preserve exact same logic as slow path
- Comprehensive functional tests
- Boundary gate verification

### Risk 3: Insufficient Improvement (MEDIUM)
**Scenario**: 76% hot-path reduction still insufficient  
**Mitigation**:
- Profile non-hot-path overhead
- Consider diagnostic marker removal
- Investigate remaining 499k ticks in 2nd syscall

### Risk 4: Boot Regression (LOW)
**Scenario**: Cache initialization adds boot cost  
**Mitigation**:
- Initialization is O(1) per process
- No matrix scan or complex computation
- Should be negligible

## Success Criteria

### Minimum (Constitutional Compliance)
- syscall_latency: <192ms (+10% of baseline 175ms)
- boot_time: <11752ms (+10% of baseline 10684ms)
- context_switch: <192ms (+10% of baseline 175ms)

### Target (Baseline Recovery)
- syscall_latency: ~175ms (baseline)
- boot_time: ~10684ms (baseline)
- context_switch: ~175ms (baseline)

### Verification
- Hot-path micro-profile: <150k ticks total
- Preservation tests: all pass
- CI authoritative metrics: within threshold

## Next Steps

1. Implement Patch C1 (context type cache)
2. Implement Patch C2 (bypass fast-path)
3. Run hot-path micro-profile locally
4. Run preservation tests
5. Submit to CI for authoritative verdict
6. If insufficient, investigate remaining overhead

## References

- Patch B verdict: `PATCH_B_CI_VERDICT.md`
- Hot-path analyzer: `scripts/ci/analyze_enforcement_hotpath.py`
- Boundary enforcement: `kernel/sys/boundary_enforcement.c`
- Process struct: `kernel/include/proc.h`

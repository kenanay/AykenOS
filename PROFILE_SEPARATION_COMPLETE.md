# Profile Separation Implementation - Complete

**Date:** 2026-02-22  
**Commit:** d568691e  
**Status:** COMPLETE  
**Authority:** MVP-1 Prerequisite

## Objective

Separate self-test code from production (release) build path while maintaining validation profile enforcement for CI gates.

## Implementation

### 1. Makefile Changes

Added `-DAYKEN_VALIDATION=1` flag for validation profile:

```makefile
ifeq ($(KERNEL_PROFILE),validation)
KERNEL_CFLAGS += -O0 -g3 -DAYKEN_VALIDATION=1
else
KERNEL_CFLAGS += -O2 -g1
endif
```

### 2. Kernel Changes (kernel/sched/sched.c)

Wrapped self-test call with compile-time guard:

```c
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
    outb(0xE9, 'M');
    outb(0xE9, 'B');
    outb(0xE9, 'T');
    outb(0xE9, '\n');
    sched_mailbox_selftest();
    outb(0xE9, 'M');
    outb(0xE9, 'B');
    outb(0xE9, 'E');
    outb(0xE9, '\n');
#endif
```

### 3. Gate Script Changes (scripts/ci/gate_sched_bridge_runtime.sh)

Added fail-closed profile enforcement:

```bash
if [[ "${KERNEL_PROFILE:-}" != "validation" ]]; then
    echo "ERROR: sched-bridge-runtime gate requires KERNEL_PROFILE=validation"
    exit 2
fi
```

### 4. Makefile Target Update

Gate target now enforces validation profile:

```makefile
ci-gate-sched-bridge-runtime: ci-evidence-dir
	@RUN_ID=$(RUN_ID) KERNEL_PROFILE=validation bash scripts/ci/gate_sched_bridge_runtime.sh
```

## Verification Results

### Release Build (KERNEL_PROFILE=release)
- ✓ Self-test call NOT present in binary (verified via objdump)
- ✓ No MBT/MBE markers in boot log
- ✓ Clean production path

### Validation Build (KERNEL_PROFILE=validation)
- ✓ Self-test call present and executed
- ✓ Markers detected by gate
- ✓ Gate PASS

### Fail-Closed Enforcement
- ✓ Gate rejects release profile with clear error
- ✓ Gate rejects unset KERNEL_PROFILE
- ✓ No silent bypass possible

### CI Gate Suite
- ✓ ci-gate-abi: PASS
- ✓ ci-gate-boundary: PASS
- ✓ ci-gate-hygiene: PASS
- ✓ ci-gate-constitutional: PASS
- ✓ ci-gate-sched-bridge-runtime: PASS

## Benefits

1. **Production Cleanliness**: Release builds contain no test instrumentation
2. **Gate Semantics**: Clear separation between measurement and production paths
3. **Fail-Closed**: Wrong profile usage fails immediately with diagnostic message
4. **Governance Alignment**: Profile discipline enforced at build and gate levels
5. **MVP-1 Ready**: Clean foundation for Ring3 stub integration

## Next Steps

MVP-1 can now proceed with:
- Ring3 stub skeleton (mailbox write interface)
- Real interaction test (Ring3 → Ring0 validation)
- Epoch monotonicity enforcement
- Reject count determinism

## Evidence

- Commit: d568691e
- CI Gates: All PASS
- Profile Enforcement: Verified
- Binary Analysis: Confirmed compile-out

**Status:** Profile separation complete. MVP-1 prerequisite satisfied.

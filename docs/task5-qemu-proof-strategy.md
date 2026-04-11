# Task 5 QEMU Proof Strategy

## Decision: BCIB Pipeline Integration (NOT Separate Binary)

**Date**: 2026-04-11  
**Authority**: Kenan AY - Architectural Steward

## The Question

Should Runtime_Bridge QEMU proof use:
1. Separate test binary (standalone Runtime_Bridge executable)
2. BCIB pipeline integration (Runtime_Bridge as part of BCIB execution)

## The Answer: BCIB Pipeline Integration

Runtime_Bridge tests are integrated into BCIB execution pipeline, NOT separate binaries.

## Rationale

### Why NOT Separate Binary

1. **Architectural Contradiction**
   - Runtime_Bridge is NOT a standalone component
   - Runtime_Bridge is BCIB's external communication layer
   - Separate binary = fake architecture

2. **Integration Risk**
   - Separate binary can pass tests but fail in production
   - "Works in test, breaks in real BCIB" scenario
   - Cannot prove real integration

3. **Fake Success Pipeline**
   - Separate binary tempts mock/stub implementations
   - Easy to bypass real kernel path
   - Defeats purpose of QEMU proof

### Why BCIB Pipeline Integration

1. **Architectural Truth**
   - Runtime_Bridge IS part of BCIB
   - Tests prove real integration
   - No architectural fiction

2. **Real Kernel Path**
   - Uses actual BCIB role assignment
   - Uses actual syscall path (INT 0x80)
   - Uses actual hardened dispatcher
   - Uses actual enforcement matrix

3. **Production Equivalence**
   - Test environment = production environment
   - Same binary, same path, same enforcement
   - QEMU proof = production proof

## Implementation

### Test Binaries

Two freestanding C binaries that BCIB loads:

1. **runtime_bridge_allowed_test.c**
   - Tests allowed syscalls: 1012, 1013, 1014
   - Expected: All syscalls succeed, execution continues
   - Markers: `RUNTIME_BRIDGE_ALLOWED_BEFORE` → syscalls → `RUNTIME_BRIDGE_ALLOWED_AFTER`

2. **runtime_bridge_forbidden_test.c**
   - Tests forbidden syscall: 1003 (SYS_V2_SUBMIT_EXECUTION)
   - Expected: Fail-closed termination, no continuation
   - Markers: `RUNTIME_BRIDGE_FORBIDDEN_BEFORE` → syscall → `[[AYKEN_BOUNDARY_KILL]]` → NO `AFTER`

### Build Process

```bash
./scripts/build-runtime-bridge-tests.sh
```

Produces:
- `build/runtime-bridge-tests/runtime_bridge_allowed_test.elf`
- `build/runtime-bridge-tests/runtime_bridge_forbidden_test.elf`

### QEMU Harness

```bash
./scripts/qemu-runtime-bridge-proof-harness.sh
```

Launches QEMU with:
- Kernel: `build/kernel.elf`
- Initrd: Test binary
- Append: `execution_role=PROC_EXECUTION_ROLE_RUNTIME_BRIDGE`
- Captures: debugcon + serial → unified trace

Generates:
- `evidence/runtime-bridge-proof/qemu_kernel_trace_allowed.log`
- `evidence/runtime-bridge-proof/qemu_kernel_trace_forbidden.log`

### Validation

Forbidden trace must pass fail-closed proof gate:

```bash
# Copy forbidden trace to gate input
cp evidence/runtime-bridge-proof/qemu_kernel_trace_forbidden.log \
   evidence/fail-closed-proof/qemu_kernel_trace.log

# Run validation
./scripts/ci-gate-fail-closed-proof.sh
```

Expected: PASS with canonical marker flow.

## Task 5 Closure Criteria

Task 5 CANNOT be marked complete without:

1. ✅ QEMU proof infrastructure (DONE - 2026-04-11)
2. ⏳ Allowed trace shows 1012/1013/1014 succeed
3. ⏳ Forbidden trace shows 1003 triggers fail-closed
4. ⏳ `ci-gate-fail-closed-proof` PASS on forbidden trace
5. ⏳ Process identity consistent
6. ⏳ Single kill guarantee
7. ⏳ Bounded execution window
8. ⏳ No continuation after kill

## Next Steps

1. **Immediate**: Run QEMU harness
   ```bash
   ./scripts/qemu-runtime-bridge-proof-harness.sh
   ```

2. **Validate**: Check traces for expected markers

3. **Gate**: Run fail-closed proof gate on forbidden trace

4. **Debug**: If gate fails, fix kernel enforcement path

5. **Iterate**: Until gate PASS

## Critical Notes

- Host tests DO NOT satisfy Task 5 closure
- Syscall adapter tests prove marshalling only
- QEMU kernel trace is MANDATORY
- No shortcuts, no mocks, no fake success

## Authority

This strategy is MANDATORY per Phase-16 specification.  
Deviation requires constitutional amendment.

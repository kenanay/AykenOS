# Bugfix Requirements Document

**Author**: Kenan AY - Architectural Steward  
**Created**: 2026-04-11  
**Status**: Requirements Review

## Introduction

The kernel build pipeline produces drift between the requested userspace payload mode and the actually embedded/booted payload. Build logs show the kernel being compiled with `DAYKEN_USER_MINIMAL_MODE_STRING="phase10a2"` (default) even when entry-proof or runtime-bridge payloads are intended. This causes execution proofs to run against the wrong payload, making userspace execution debugging results unreliable. The build/boot chain does not deterministically lock which userspace payload is embedded into the kernel, with multiple authority sources (USER_MINIMAL_MODE, MINIMAL_MODE, embedded header generation) falling back to defaults silently.

## Bug Analysis

### Current Behavior (Defect)

1.1 WHEN the build system compiles the kernel with entry-proof payload requested THEN the build log shows `DAYKEN_USER_MINIMAL_MODE_STRING="phase10a2"` instead of entry-proof

1.2 WHEN the build system compiles the kernel with runtime-bridge-test payload requested THEN the build log shows `DAYKEN_USER_MINIMAL_MODE_STRING="phase10a2"` instead of runtime-bridge-test

1.3 WHEN the kernel boots with entry-proof payload intended THEN the entry-proof trace shows `user_minimal_mode=entry-proof` followed by `P10_RING3_ENTER` then `P10_MAILBOX_MISS_PRE_USER_BYPASS` (scheduler bypass without userspace execution)

1.4 WHEN the runtime-bridge harness runs THEN all Runtime_Bridge marker counts are 0 despite large log output

1.5 WHEN the build system generates embedded_elf.h THEN the embedded payload hash does not match the requested payload ELF hash

1.6 WHEN the kernel boots THEN no mode/hash marker is emitted to debugcon to verify which payload was actually embedded

1.7 WHEN multiple authority sources exist (USER_MINIMAL_MODE, MINIMAL_MODE, embedded header) THEN silent fallback to default phase10a2 occurs without build or boot failure

### Expected Behavior (Correct)

2.1 WHEN the build system compiles the kernel with entry-proof payload requested THEN the build log SHALL show `DAYKEN_USER_MINIMAL_MODE_STRING="entry-proof"` as single source of authority

2.2 WHEN the build system compiles the kernel with runtime-bridge-test payload requested THEN the build log SHALL show `DAYKEN_USER_MINIMAL_MODE_STRING="runtime-bridge-test"` as single source of authority

2.3 WHEN the build system generates embedded_elf.h THEN the embedded payload hash SHALL match the selected payload ELF hash exactly

2.4 WHEN the kernel boots THEN it SHALL emit the embedded payload mode and hash marker to debugcon for verification

2.5 WHEN a mismatch occurs between requested mode and embedded hash THEN the build system SHALL HARD FAIL with explicit error message

2.6 WHEN a mismatch occurs between embedded hash and boot-emitted hash THEN the kernel boot SHALL HARD FAIL with explicit error message

2.7 WHEN the build system selects a payload mode THEN that mode SHALL be the single source of authority throughout the build/embed/boot chain

2.8 WHEN the correctness invariants are violated THEN the system SHALL HARD FAIL at the earliest detection point

**Mode Authority Invariant**: `build_selected_mode == embedded_elf_mode == boot_emitted_mode`

**Payload Integrity Invariant**: `expected_payload_sha == embedded_elf_sha == boot_emitted_sha`

**Note**: Boot-time enforcement is phased:
- **Phase A** (initial landing): Emit observable mismatch markers (`[K][PAYLOAD_HASH_MISMATCH]`, `[K][PAYLOAD_MODE_MISMATCH]`) without halting
- **Phase B** (after evidence chain stabilizes): Add hard fail/halt on mismatch

### Unchanged Behavior (Regression Prevention)

3.1 WHEN the kernel boots with phase10a2 payload (default) THEN the system SHALL CONTINUE TO execute Ring3 userspace code correctly

3.2 WHEN other traces in the repo run (showing successful P10_RING3_USER_CODE) THEN they SHALL CONTINUE TO work without regression

3.3 WHEN existing CI gates run THEN they SHALL CONTINUE TO pass without breaking due to payload authority changes

3.4 WHEN the build system compiles the kernel THEN it SHALL CONTINUE TO produce valid ELF binaries

3.5 WHEN the kernel initializes THEN it SHALL CONTINUE TO set up paging, heap, scheduler, and memory management correctly

3.6 WHEN the fix is implemented THEN it SHALL remain within non-architectural bugfix boundaries (no new syscalls, execution layers, or contracts) per architectural freeze requirements

3.7 WHEN Phase-10 Ring3 execution runs THEN it SHALL CONTINUE TO work as proven by existing successful traces

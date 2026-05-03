# Implementation Plan: AYKEN VCP Execution Binding

## Overview

This implementation plan transforms the AYKEN Validation Control Plane (VCP) from a CI-only authority into a runtime-enforced authority. The implementation extends the execution slot structure with validation state, introduces kernel-level runtime hooks for enforcement, binds BCIB and ABDF to VCP decisions, implements fail-closed mechanisms, and ensures comprehensive evidence emission.

**Implementation Language**: C (kernel enforcement core)  
**Note**: Rust MAY be used for userland tooling, evidence verification, and CI-side analyzers in future phases. Rust MUST NOT be introduced into kernel execution enforcement paths in this phase.

**Key Files**:
- `kernel/sys/execution_slot.c` - Execution slot with validation state
- `kernel/sys/vcp_runtime.c` - Runtime validation enforcement hook (new)
- `kernel/sys/vcp_evidence.c` - Runtime evidence emission (new)
- `kernel/sys/bcib_executor.c` - BCIB contract enforcement binding
- `kernel/sys/boundary_enforcement.c` - ABDF boundary validation binding
- `kernel/sys/fail_closed.c` - Fail-closed enforcement mechanism (new)
- `kernel/include/vcp_runtime.h` - VCP runtime API definitions (new)
- `kernel/include/execution_slot.h` - Extended execution slot structure

## Tasks

- [x] 1. Extend execution slot structure with validation state
  - [x] 1.1 Define validation state structure in `kernel/include/execution_slot.h`
    - Add `struct vcp_validation_state` with FINAL ABI fields: `validation_result`, `contract_id`, `boundary_policy`, `context_hash`, `nonce`, `signature`, `capability_id`, `evidence_id`, `timestamp`
    - **CRITICAL**: This is the FINAL ABI. Do NOT evolve this structure later. Task 18 will implement verification functions for this layout.
    - Add `struct vcp_validation_state *validation_state` field to `struct execution_slot`
    - Ensure structure is aligned for deterministic memory layout
    - _Requirements: 1.1, 7.1, 7.4, 11.7_
  
  - [ ]* 1.2 Write property test for validation state initialization
    - **Property 1: Execution Slot Validation State Initialization** [REQUIRED]
    - **Validates: Requirements 1.1, 7.1**
    - Test that all newly created execution slots contain initialized validation state
  
  - [x] 1.3 Implement validation state initialization in `kernel/sys/execution_slot.c`
    - Modify `execution_slot_create()` to initialize `validation_state` field
    - Ensure validation state is set to NULL if no VCP state is available (for fail-closed detection)
    - Add validation state cleanup in `execution_slot_destroy()`
    - _Requirements: 1.1, 7.1, 7.3_
  
  - [ ]* 1.4 Write unit tests for execution slot lifecycle
    - Test execution slot creation with validation state
    - Test execution slot destruction and cleanup
    - Test validation state preservation during slot lifetime
    - _Requirements: 7.1, 7.2, 7.3_

- [x] 2. Implement VCP runtime validation hook
  - [x] 2.1 Create VCP runtime API header `kernel/include/vcp_runtime.h`
    - Define `vcp_runtime_validate(struct execution_slot *slot)` function signature
    - Define validation result codes: `VCP_VALID`, `VCP_INVALID`, `VCP_MISSING`, `VCP_FAIL_CLOSED`
    - Define evidence emission function signatures
    - Add constitutional compliance annotations (NO global state, NO capability bypass)
    - _Requirements: 1.2, 1.3, 1.4, 8.2, 8.3_
  
  - [x] 2.2 Implement runtime validation hook in `kernel/sys/vcp_runtime.c`
    - Implement `vcp_runtime_validate()` to check validation state in execution slot
    - Return `VCP_FAIL_CLOSED` if validation state is NULL (missing)
    - Return `VCP_INVALID` if validation state indicates invalid execution
    - Return `VCP_VALID` if validation state is present and valid
    - Ensure deterministic execution (no global state mutations)
    - _Requirements: 1.2, 1.3, 1.5, 8.2_
  
  - [x] 2.3 Write property test for fail-closed on missing validation state [CRITICAL]
    - **Property 2: Fail-Closed on Missing Validation State** [CRITICAL]
    - **Validates: Requirements 1.2, 4.1, 5.3**
    - Test that execution is blocked when validation state is NULL
  
  - [x] 2.4 Write property test for invalid validation state blocking [CRITICAL]
    - **Property 3: Invalid Validation State Blocks Execution** [CRITICAL]
    - **Validates: Requirements 1.3, 2.3, 3.3**
    - Test that execution is blocked when validation state indicates invalid execution
  
  - [x] 2.5 Write property test for valid validation state permitting execution [CRITICAL]
    - **Property 4: Valid Validation State Permits Execution** [CRITICAL]
    - **Validates: Requirements 1.5**
    - Test that execution proceeds when validation state is valid

- [x] 3. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Implement fail-closed enforcement mechanism
  - [x] 4.1 Create fail-closed handler in `kernel/sys/fail_closed.c`
    - Implement `vcp_fail_closed(struct execution_slot *slot, const char *reason)` function
    - Block execution permanently (return error code, do not continue)
    - Preserve system state integrity (no partial state mutations)
    - Emit evidence describing failure context
    - Ensure no panic or undefined behavior (handle errors gracefully)
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 9.3_
  
  - [x] 4.2 Write property test for fail-closed permanence [CRITICAL]
    - **Property 9: Fail-Closed Permanence** [CRITICAL]
    - **Validates: Requirements 4.2, 4.4**
    - Test that execution cannot continue after fail-closed is triggered
  
  - [x] 4.3 Write property test for fail-closed state integrity [CRITICAL]
    - **Property 10: Fail-Closed State Integrity** [CRITICAL]
    - **Validates: Requirements 4.5**
    - Test that system state remains consistent when fail-closed is triggered
  
  - [x] 4.4 Integrate fail-closed handler into runtime validation hook
    - Modify `vcp_runtime_validate()` to call `vcp_fail_closed()` when validation fails
    - Ensure fail-closed is invoked for both missing and invalid validation states
    - _Requirements: 1.2, 1.3, 4.1_

- [-] 5. Implement diagnostic evidence emission stubs (DIAGNOSTIC ONLY - Authoritative evidence in Task 20-23)
  - [x] 5.1 Create diagnostic evidence emission API stubs in `kernel/sys/vcp_evidence.c`
    - Implement `vcp_emit_validation_check(struct execution_slot *slot, int result)` function (DIAGNOSTIC STUB)
    - Implement `vcp_emit_execution_block(struct execution_slot *slot, const char *reason)` function (DIAGNOSTIC STUB)
    - Implement `vcp_emit_contract_execution(struct execution_slot *slot, const char *contract_id)` function (DIAGNOSTIC STUB)
    - Implement `vcp_emit_boundary_crossing(struct execution_slot *slot, const char *boundary_id)` function (DIAGNOSTIC STUB)
    - **CRITICAL**: These are DIAGNOSTIC STUBS ONLY. Authoritative evidence (signed, verified, durable-before-proceed) will be implemented in Task 20-23.
    - **ALLOWED**: Diagnostic ring buffer MAY be asynchronous (non-authoritative)
    - Ensure stub format is compatible with future authoritative evidence (Task 20-23)
    - _Requirements: 6.1, 6.2, 6.3, 6.4_
    - _Guarantees: diagnostic telemetry only, no authority_
  
  - [x]* 5.2 Write property test for comprehensive evidence emission
    - **Property 8: Comprehensive Evidence Emission** [QUALITY]
    - **Validates: Requirements 2.4, 3.4, 6.1, 6.2, 6.3, 6.4**
    - Test that all validation checks, blocks, and enforcement events emit evidence
  
  - [x] 5.3 Write property test for fail-closed evidence completeness [CRITICAL]
    - **Property 11: Fail-Closed Evidence Completeness** [CRITICAL]
    - **Validates: Requirements 4.3, 6.6**
    - Test that fail-closed conditions emit complete failure context
  
  - [x] 5.4 Integrate evidence emission into runtime validation hook
    - Modify `vcp_runtime_validate()` to call `vcp_emit_validation_check()` for all validation checks
    - Modify `vcp_fail_closed()` to call `vcp_emit_execution_block()` when blocking execution
    - _Requirements: 6.1, 6.2_
  
  - [ ]* 5.5 Write property test for audit trail integrity
    - **Property 21: Audit Trail Integrity**
    - **Validates: Requirements 8.6**
    - Test that evidence entries are immutable and cannot be tampered with
  
  - [ ] 5.6 Write property test for diagnostic evidence isolation [CRITICAL]
    - **Property 49: Diagnostic Evidence Isolation** [CRITICAL]
    - **Validates: Design isolation contract**
    - Test that diagnostic evidence emission does NOT affect validation outcome
    - Test that diagnostic evidence emission does NOT affect trust verification
    - Test that diagnostic evidence emission does NOT affect execution path
    - Test strategy:
      1. Run validation with evidence enabled vs disabled → same outcome
      2. Inject evidence buffer overflow → execution unaffected
      3. Inject evidence write failure → execution unaffected
      4. Verify evidence functions return void (no error propagation)
    - **CRITICAL**: This test locks the isolation guarantee and prevents future refactoring from breaking it
    - _Guarantees: evidence emission is side-effect free_

- [ ] 6. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 7. Bind BCIB execution contracts to VCP validation
  - [ ] 7.1 Integrate VCP validation into BCIB executor in `kernel/sys/bcib_executor.c`
    - Add `#include "vcp_runtime.h"` to BCIB executor
    - Modify BCIB contract invocation to call `vcp_runtime_validate()` before execution
    - Block contract execution if validation returns `VCP_FAIL_CLOSED` or `VCP_INVALID`
    - Emit evidence for contract execution using `vcp_emit_contract_execution()`
    - _Requirements: 2.1, 2.2, 2.3, 2.4_
  
  - [ ]* 7.2 Write property test for BCIB contract validation enforcement
    - **Property 5: BCIB Contract Validation Enforcement**
    - **Validates: Requirements 2.1, 2.2**
    - Test that BCIB contracts verify validation state before execution
  
  - [ ] 7.3 Ensure BCIB enforcement matches CI enforcement decisions
    - Review BCIB validation logic to ensure consistency with CI validation
    - Add comments documenting CI-runtime consistency requirements
    - _Requirements: 2.5, 10.1, 10.3_

- [ ] 8. Bind ABDF boundary validation to VCP validation
  - [ ] 8.1 Integrate VCP validation into ABDF boundary enforcement in `kernel/sys/boundary_enforcement.c`
    - Add `#include "vcp_runtime.h"` to boundary enforcement
    - Modify boundary crossing logic to call `vcp_runtime_validate()` before permitting crossing
    - Block boundary crossing if validation returns `VCP_FAIL_CLOSED` or `VCP_INVALID`
    - Emit evidence for boundary crossing using `vcp_emit_boundary_crossing()`
    - _Requirements: 3.1, 3.2, 3.3, 3.4_
  
  - [ ]* 8.2 Write property test for ABDF boundary validation enforcement
    - **Property 6: ABDF Boundary Validation Enforcement**
    - **Validates: Requirements 3.1, 3.2**
    - Test that boundary crossings check validation state before permitting
  
  - [ ] 8.3 Implement Ring3/Ring0 boundary policy enforcement
    - Add constitutional boundary policy check in boundary enforcement
    - Block Ring3 to Ring0 direct access according to `SECURITY.BOUNDARY.VIOLATION` rule
    - Emit evidence when boundary policy violations are detected
    - _Requirements: 3.5, 8.4_
  
  - [ ]* 8.4 Write property test for constitutional boundary policy enforcement
    - **Property 7: Constitutional Boundary Policy Enforcement**
    - **Validates: Requirements 3.5, 8.4**
    - Test that Ring3 to Ring0 crossings are blocked according to constitutional rules
  
  - [ ] 8.5 Ensure ABDF enforcement matches CI enforcement decisions
    - Review ABDF validation logic to ensure consistency with CI validation
    - Add comments documenting CI-runtime consistency requirements
    - _Requirements: 10.2, 10.3_

- [ ] 9. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 10. Implement CLI authority reduction
  - [ ] 10.1 Integrate VCP validation into CLI handler in `kernel/sys/syscall.c` or CLI entry point
    - Modify CLI command execution path to attach validation state to execution slot
    - Call `vcp_runtime_validate()` before CLI command execution
    - Block CLI execution if validation fails
    - Emit evidence for CLI execution attempts using `vcp_emit_validation_check()`
    - _Requirements: 5.1, 5.3, 5.5_
  
  - [ ]* 10.2 Write property test for CLI validation state attachment
    - **Property 12: CLI Validation State Attachment**
    - **Validates: Requirements 5.1**
    - Test that CLI commands attach validation state to execution slots
  
  - [ ]* 10.3 Write property test for CLI boundary validation
    - **Property 13: CLI Boundary Validation**
    - **Validates: Requirements 5.4**
    - Test that CLI operations crossing boundaries are validated by ABDF
  
  - [ ]* 10.4 Write property test for CLI evidence emission
    - **Property 14: CLI Evidence Emission**
    - **Validates: Requirements 5.5**
    - Test that CLI execution attempts emit evidence
  
  - [ ] 10.5 Ensure CLI does not provide VCP bypass mechanisms
    - Review CLI code paths to ensure no bypass mechanisms exist
    - Add assertions or checks to prevent capability bypass
    - _Requirements: 5.2, 8.3_

- [ ] 11. Implement execution slot lifecycle validation state management
  - [ ] 11.1 Implement validation state preservation in `kernel/sys/execution_slot.c`
    - Ensure validation state remains unchanged while execution slot is active
    - Add immutability checks to prevent external modification of validation state
    - _Requirements: 7.2, 7.4_
  
  - [ ]* 11.2 Write property test for validation state preservation
    - **Property 15: Execution Slot Validation State Preservation**
    - **Validates: Requirements 7.2**
    - Test that validation state remains unchanged during slot lifetime
  
  - [ ]* 11.3 Write property test for validation state immutability
    - **Property 17: Validation State Immutability**
    - **Validates: Requirements 7.4**
    - Test that external attempts to modify validation state are blocked
  
  - [ ] 11.4 Implement execution slot destruction evidence emission
    - Modify `execution_slot_destroy()` to emit final evidence before cleanup
    - Use `vcp_emit_validation_check()` to record slot lifecycle completion
    - _Requirements: 7.3_
  
  - [ ]* 11.5 Write property test for execution slot destruction evidence
    - **Property 16: Execution Slot Destruction Evidence**
    - **Validates: Requirements 7.3**
    - Test that slot destruction emits final evidence
  
  - [ ] 11.6 Implement nested execution slot independence
    - Ensure nested execution slots maintain independent validation state
    - Add checks to prevent validation state leakage between nested slots
    - _Requirements: 7.5_
  
  - [ ]* 11.7 Write property test for nested slot independence
    - **Property 18: Nested Slot Independence**
    - **Validates: Requirements 7.5**
    - Test that nested slots maintain independent validation state

- [ ] 12. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 13. Implement constitutional compliance checks
  - [ ] 13.1 Add determinism checks to runtime validation hook
    - Review `vcp_runtime_validate()` to ensure no global state mutations
    - Add static analysis annotations or assertions to enforce `DETERMINISM.GLOBAL` rule
    - _Requirements: 8.2_
  
  - [ ]* 13.2 Write property test for deterministic execution
    - **Property 19: Deterministic Execution (No Global State Mutation)**
    - **Validates: Requirements 8.2**
    - Test that runtime hook does not introduce global state mutations
  
  - [ ] 13.3 Add capability security enforcement checks
    - Review all VCP enforcement paths to ensure capability checks are enforced
    - Ensure no bypass mechanisms exist (verify against `KERNEL.CAPABILITY.BYPASS` rule)
    - _Requirements: 8.3_
  
  - [ ]* 13.4 Write property test for capability security enforcement
    - **Property 20: Capability Security Enforcement**
    - **Validates: Requirements 8.3**
    - Test that capability checks are enforced and cannot be bypassed
  
  - [ ] 13.5 Add memory safety checks to validation enforcement paths
    - Review all VCP code paths for memory leaks or safety violations
    - Add cleanup code to prevent memory leaks in error paths
    - _Requirements: 9.2_
  
  - [ ]* 13.6 Write property test for memory safety in validation paths
    - **Property 22: Memory Safety in Validation Paths**
    - **Validates: Requirements 9.2**
    - Test that validation enforcement does not introduce memory leaks
  
  - [ ] 13.7 Add error handling without panic
    - Review all VCP code paths to ensure errors are handled gracefully
    - Replace any panic-like behavior with controlled error returns
    - _Requirements: 9.3_
  
  - [ ]* 13.8 Write property test for error handling without panic
    - **Property 23: Error Handling Without Panic**
    - **Validates: Requirements 9.3**
    - Test that validation errors are handled without panicking

- [ ] 14. Implement performance and reliability requirements
  - [ ] 14.1 Add deterministic time bounds to runtime validation hook
    - Profile `vcp_runtime_validate()` execution time
    - Ensure validation completes within deterministic time bounds
    - Add timeout or performance assertions if needed
    - _Requirements: 9.1_
  
  - [ ] 14.2 Test validation enforcement under high load
    - Create stress test that exercises validation enforcement under high load
    - Verify that validation enforcement remains reliable under load
    - _Requirements: 9.5_
  
  - [ ]* 14.3 Write unit tests for performance and reliability
    - Test validation hook performance under various conditions
    - Test evidence emission performance
    - Test fail-closed handler reliability
    - _Requirements: 9.1, 9.4, 9.5_

- [ ] 15. Implement CI-runtime consistency verification
  - [ ] 15.1 Document CI-runtime validation consistency requirements
    - Add documentation describing how runtime validation matches CI validation
    - Document validation logic that must remain synchronized between CI and runtime
    - _Requirements: 10.1, 10.2, 10.3_
  
  - [ ] 15.2 Add CI-runtime consistency checks to evidence emission
    - Ensure evidence format produced at runtime matches CI evidence format
    - Add version or schema identifiers to evidence for consistency verification
    - _Requirements: 10.5_
  
  - [ ] 15.3 Create CI verification tooling integration points
    - Add hooks or interfaces for CI tools to verify runtime evidence
    - Document how CI can verify runtime enforcement consistency
    - _Requirements: 10.5_

- [ ] 16. Final integration and wiring
  - [ ] 16.1 Wire all VCP enforcement points together
    - Ensure runtime hook is called from all execution entry points
    - Ensure BCIB and ABDF enforcement are properly integrated
    - Ensure CLI authority reduction is complete
    - Verify fail-closed mechanism is reachable from all enforcement points
    - _Requirements: 1.1, 2.1, 3.1, 4.1, 5.1_
  
  - [ ] 16.2 Add system-wide VCP enforcement verification
    - Create integration test that exercises all enforcement paths
    - Verify evidence is emitted correctly from all paths
    - Verify fail-closed works correctly from all paths
    - _Requirements: 1.2, 2.2, 3.2, 4.1_
  
  - [ ]* 16.3 Write integration tests for complete VCP enforcement
    - Test end-to-end execution with valid validation state
    - Test end-to-end execution with missing validation state (fail-closed)
    - Test end-to-end execution with invalid validation state (fail-closed)
    - Test BCIB contract enforcement with VCP validation
    - Test ABDF boundary enforcement with VCP validation
    - Test CLI execution with VCP validation
    - _Requirements: 1.1, 1.2, 1.3, 2.1, 3.1, 5.1_

- [ ] 17. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [-] 18. Implement VCP Trust Token Verification (CRITICAL - MUST BE DONE BEFORE BCIB/ABDF BINDING)
  - [x] 18.1 Verify Task 1 FINAL ABI layout in `kernel/include/vcp_runtime.h`
    - **CRITICAL**: Task 1 already defined the FINAL ABI. This task VERIFIES that layout, does NOT redefine it.
    - Verify `struct vcp_validation_state` from Task 1.1 contains all required fields: `validation_result`, `contract_id`, `boundary_policy`, `context_hash`, `nonce`, `signature`, `capability_id`, `evidence_id`, `timestamp`
    - Add trust verification result codes: `VCP_TRUST_VERIFIED`, `VCP_TRUST_FAILED_CAPABILITY`, `VCP_TRUST_FAILED_CONTEXT`, `VCP_TRUST_FAILED_SIGNATURE`, `VCP_TRUST_FAILED_NONCE`
    - Document trust token verification requirements (NOT structure definition - that's in Task 1)
    - _Requirements: 11.1, 11.2, 11.3, 11.7_
    - _Guarantees: ABI consistency, no drift_
  
  - [x] 18.2 Implement context hash computation in `kernel/sys/vcp_runtime.c`
    - Implement `vcp_compute_context_hash(struct execution_slot *slot)` function
    - Hash inputs: BCIB contract_id, ABDF boundary_policy, execution_slot_id, metadata
    - Ensure deterministic hash computation (no global state)
    - _Requirements: 11.2_
  
  - [x] 18.3 Implement kernel capability binding in `kernel/sys/vcp_runtime.c`
    - Implement `vcp_verify_capability(struct execution_slot *slot, struct vcp_validation_state *state)` function
    - Verify that validation state is bound to a kernel-issued capability
    - Return failure if capability binding is invalid or missing
    - _Requirements: 11.1_
  
  - [x] 18.4 Implement signature verification in `kernel/sys/vcp_runtime.c`
    - Implement `vcp_verify_signature(struct vcp_validation_state *state)` function
    - Add VCP trust root interface (stub for now, to be implemented with cryptographic backend)
    - Verify signature against VCP trust root
    - Return failure if signature is invalid
    - _Requirements: 11.3_
  
  - [x] 18.5 Implement nonce and replay protection in `kernel/sys/vcp_runtime.c`
    - Implement `vcp_verify_nonce(struct vcp_validation_state *state)` function
    - **CRITICAL**: Nonce registry MUST be append-only ledger (NOT hidden mutable global map)
    - Implement nonce ledger as append-only structure (deterministic, no global state mutation)
    - Return failure if nonce has been used before
    - Ensure nonce registry complies with DETERMINISM.GLOBAL constitutional rule
    - _Requirements: 11.2, 11.6, 16.2_
    - _Guarantees: replay protection, determinism compliance_
  
  - [x] 18.6 Integrate trust verification into runtime validation hook
    - Modify `vcp_runtime_validate()` to call `vcp_verify_validation_state()` BEFORE checking validation result
    - Ensure trust verification happens first (capability → context → signature → nonce → result)
    - Fail-closed if any trust verification step fails
    - _Requirements: 11.4, 11.5_
  
  - [x] 18.7 Implement fail-closed on trust verification failure
    - Modify `vcp_fail_closed()` to handle trust verification failures
    - Emit evidence describing which trust check failed (capability, context, signature, nonce)
    - Ensure fail-closed is permanent (no recovery after trust failure)
    - _Requirements: 11.5_
  
  - [x] 18.8 Write property test for fake state rejection [CRITICAL]
    - **Property 25: Fake Validation State Rejection** [CRITICAL]
    - **Validates: Requirements 11.1, 11.5**
    - Test that validation state without valid capability binding is rejected
  
  - [x] 18.9 Write property test for replayed state rejection [CRITICAL]
    - **Property 26: Replayed Validation State Rejection** [CRITICAL]
    - **Validates: Requirements 11.2, 11.5**
    - Test that validation state with mismatched context hash or replayed nonce is rejected
  
  - [x] 18.10 Write property test for signature verification [CRITICAL]
    - **Property 27: Signature Verification Enforcement** [CRITICAL]
    - **Validates: Requirements 11.3, 11.5**
    - Test that validation state with invalid signature is rejected
  
  - [x] 18.11 Write property test for trust verification before enforcement [CRITICAL]
    - **Property 28: Trust Verification Before Enforcement** [CRITICAL]
    - **Validates: Requirements 11.4**
    - Test that trust verification happens before validation result is checked
  
  - [x] 18.12 Write property test for validation state trust verification [CRITICAL]
    - **Property 24: Validation State Trust Verification** [CRITICAL]
    - **Validates: Requirements 11.1, 11.2, 11.3, 11.4, 11.5**
    - Test that all trust checks (capability, context, signature, nonce) are performed
  
  - [x]* 18.13 Write integration tests for trust token verification
    - Test valid trust token is accepted
    - Test fake trust token is rejected (capability failure)
    - Test replayed trust token is rejected (context hash failure)
    - Test replayed nonce is rejected (nonce failure)
    - Test modified trust token is rejected (signature failure)
    - Test trust verification failure triggers fail-closed
    - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5_

- [x] 19. Checkpoint - Ensure all trust verification tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 20. Implement Evidence Foundation (CRITICAL - DETERMINISM & EMISSION CONTRACT)
  - [ ] 20.1 Implement deterministic evidence format
    - Define `struct evidence_entry` in `kernel/include/vcp_evidence.h` with fixed-size fields: `index`, `prev_hash`, `event_hash`, `timestamp`, `slot_id`, `event_type`, `validation_result`, `context_hash`, `signature`, `signer_id`
    - Implement `evidence_compute_hash(entry)` function using deterministic hash algorithm
    - Ensure no nondeterministic fields (no system time, no random values)
    - Ensure same input produces same evidence entry
    - Document evidence format schema for CI verification
    - _Requirements: 12.2, 12.9, 13.1_
    - _Guarantees: CI replay, determinism_
  
  - [ ] 20.2 Define evidence emission contract
    - Document WHEN evidence MUST be emitted (validation check, fail-closed, boundary crossing, contract execution)
    - Document HOW evidence MUST be emitted (format, signature, verification)
    - Document WHERE evidence MUST be stored (slot-local, global, both)
    - Define emission priority: critical events → global chain, frequent events → slot-local chain
    - _Requirements: 12.3, 12.6_
    - _Guarantees: clear emission semantics_
  
  - [ ] 20.3 Implement evidence emission failure semantics
    - Define failure behavior: evidence write fails → fail-closed
    - Implement `evidence_emission_failed()` handler that calls `vcp_fail_closed()`
    - Ensure no silent skip or bypass allowed
    - Document that evidence emission failure is a critical system failure
    - _Requirements: 12.4, 12.8_
    - _Guarantees: audit cannot be bypassed_
  
  - [ ] 20.4 Write property test for deterministic evidence format [CRITICAL]
    - **Property 29: Deterministic Evidence Format** [CRITICAL]
    - **Validates: Requirements 12.2, 12.9**
    - Test that same input produces identical evidence entry
    - Test that no nondeterministic fields exist

- [ ] 21. Implement Evidence Trust Layer (CRITICAL - AUTHENTICITY & VERIFICATION)
  - [ ] 21.1 Implement evidence signature model
    - Add `signature` and `signer_id` fields to `struct evidence_entry` (already in 20.1)
    - **CRITICAL**: Kernel does NOT hold VCP trust root private key
    - **Evidence Producer Key Model**:
      - Kernel holds evidence producer key (separate from trust root)
      - Evidence signed with producer key
      - CI verifies: (1) evidence signature with producer key, (2) producer key authorized by trust root
      - Trust root registry authorizes kernel evidence producer key
    - Implement `evidence_sign(entry, producer_key_id)` function in `kernel/sys/vcp_evidence.c`
    - Ensure signature covers all entry fields (index, prev_hash, event_hash, timestamp, slot_id, event_type, validation_result, context_hash)
    - _Requirements: 13.1, 13.2, 16.3_
    - _Guarantees: evidence authenticity, secure key model_
  
  - [ ] 21.2 Implement evidence signature verification
    - Implement `evidence_verify_signature(entry)` function in `kernel/sys/vcp_evidence.c`
    - Verify signature against VCP trust root
    - Return failure if signature is invalid or signer_id is not trusted
    - _Requirements: 13.2, 13.3_
    - _Guarantees: forged evidence rejection_
  
  - [ ] 21.3 Enforce verification-before-accept rule
    - Modify all evidence append functions to call `evidence_verify_signature()` BEFORE accepting entry
    - Reject unsigned evidence (signature == 0 or signer_id == 0)
    - Reject evidence with invalid signature
    - _Requirements: 13.3, 13.4_
    - _Guarantees: only verified evidence enters chain_
  
  - [ ] 21.4 Implement fail-closed on invalid evidence
    - If evidence signature verification fails → call `vcp_fail_closed()`
    - If unsigned evidence is detected → call `vcp_fail_closed()`
    - Emit evidence describing which trust check failed (signature invalid, signer untrusted, unsigned)
    - _Requirements: 13.4, 13.5_
    - _Guarantees: invalid evidence triggers fail-closed_
  
  - [ ] 21.5 Bind evidence to ABDF snapshot hash
    - Modify evidence emission to include ABDF snapshot hash in `context_hash` field
    - Ensure `context_hash = HASH(execution_context || abdf_snapshot_hash)`
    - This binds evidence to deterministic execution state
    - _Requirements: 13.6_
    - _Guarantees: evidence bound to execution state, replay determinism_
  
  - [ ] 21.9 Implement trust root versioning and key rotation support [REQUIRED]
    - Define `struct vcp_trust_root` with fields: `trust_root_id`, `version`, `public_key`, `valid_from`, `valid_until`, `status`
    - Add `trust_root_version` field to `struct evidence_entry` (already in 20.1)
    - Implement `get_trust_root_by_version(version)` function for historical trust root lookup
    - Implement backward verification: old evidence verified with historical trust root
    - Document key rotation flow: new key → old key marked ROTATED → new evidence uses new version
    - Document revocation policy: revoked trust roots invalidate all evidence signed with that key
    - _Requirements: 13.8_
    - _Guarantees: key rotation without breaking old evidence, revocation support_
  
  - [ ] 21.10 Enforce logical monotonic counter for timestamps [CRITICAL]
    - Implement `get_logical_timestamp()` function that returns monotonic counter (NOT wall clock)
    - Use execution tick counter or event sequence number
    - Ban wall clock time, system time, rdtsc (non-deterministic)
    - Document: same execution → same timestamps (deterministic replay requirement)
    - _Requirements: 13.9_
    - _Guarantees: timestamp determinism, CI replay_
  
  - [ ] 21.6 Write property test for evidence signature integrity [CRITICAL]
    - **Property 34: Evidence Signature Integrity** [CRITICAL]
    - **Validates: Requirements 13.1, 13.2**
    - Test that all evidence entries are signed
    - Test that signature verification succeeds for valid evidence
  
  - [ ] 21.7 Write property test for forged evidence rejection [CRITICAL]
    - **Property 35: Forged Evidence Rejection** [CRITICAL]
    - **Validates: Requirements 13.3, 13.4**
    - Test that unsigned evidence is rejected
    - Test that evidence with invalid signature is rejected
    - Test that forged evidence triggers fail-closed
  
  - [ ] 21.8 Write property test for verification-before-accept [CRITICAL]
    - **Property 36: Verification Before Accept** [CRITICAL]
    - **Validates: Requirements 13.3**
    - Test that signature verification happens before evidence is added to chain
    - Test that invalid evidence never enters chain

- [ ] 22. Implement Evidence Chain Architecture (CRITICAL - HYBRID CHAIN MODEL)
  - [ ] 22.1 Implement slot-local append-only evidence chain
    - Create per-slot evidence chain structure in `kernel/sys/vcp_evidence.c`
    - Define `struct slot_evidence_chain` with fields: `entries[]`, `head_index`, `head_hash`, `slot_id`
    - Implement `slot_evidence_append(slot_id, entry)` function with append-only semantics
    - Call `evidence_verify_signature(entry)` BEFORE appending (trust layer integration)
    - Store slot chains in: `runtime/slots/slot-{id}/local_chain.bin`
    - Maintain per-slot head hash for integrity verification
    - Ensure no overwrite or delete operations allowed
    - _Requirements: 12.1, 12.2, 12.7, 13.3_
    - _Guarantees: isolation, determinism, slot-local authority, verified evidence only_
  
  - [ ] 22.2 Implement global append-only evidence chain (authority)
    - Create global evidence chain structure in `kernel/sys/vcp_evidence.c`
    - Define `struct global_evidence_chain` with fields: `entries[]`, `head_index`, `head_hash`
    - Implement `global_evidence_append(entry)` function with append-only semantics
    - Call `evidence_verify_signature(entry)` BEFORE appending (trust layer integration)
    - Store global chain in: `out/evidence/run-{id}/chain/global_chain.bin`
    - Maintain global head hash for system-wide integrity
    - Enforce fail-closed if global chain write fails
    - _Requirements: 12.1, 12.3, 12.4, 13.3_
    - _Guarantees: system-wide authority, CI replayability, verified evidence only_
  
  - [ ] 22.3 Implement slot chain anchoring to global chain (CRITICAL)
    - Implement `slot_chain_anchor(slot_id)` function in `kernel/sys/vcp_evidence.c`
    - On slot completion: compute slot head hash, create anchor event, sign anchor event, append to global chain
    - Define anchor event type: `EVENT_SLOT_CHAIN_COMMIT` with fields: `slot_id`, `slot_head_hash`, `timestamp`, `signature`, `signer_id`
    - Ensure anchor event is signed and verified before appending
    - Ensure anchor event is written to global chain before slot destruction
    - Fail-closed if anchor write fails or signature verification fails
    - _Requirements: 12.5, 12.6, 13.1_
    - _Guarantees: replayability, audit integrity, slot-to-global binding, verified anchors_
  
  - [ ] 22.4 Implement optional ring buffer for diagnostics (NON-AUTHORITY)
    - Create ring buffer structure in `kernel/sys/vcp_evidence.c`
    - Define `struct evidence_ring_buffer` with fields: `entries[]`, `head`, `size`
    - Implement `ring_buffer_append(entry)` with overwrite semantics
    - Store ring buffer in: `runtime/ring/recent_events.bin`
    - Document that ring buffer is NOT used for CI validation
    - Ensure ring buffer failure does NOT trigger fail-closed
    - Ring buffer does NOT require signature verification (diagnostics only)
    - _Requirements: 12.10_
    - _Guarantees: debug only, non-authoritative_
  
  - [ ] 22.5 Implement evidence directory structure
    - Create directory structure on system initialization:
      - `out/evidence/run-{id}/meta/` (run.json, environment.json)
      - `out/evidence/run-{id}/chain/` (global_chain.bin, head.hash)
      - `out/evidence/run-{id}/runtime/slots/` (slot-specific chains)
      - `out/evidence/run-{id}/validation/` (trust_tokens.json, verification_results.json)
      - `out/evidence/run-{id}/bcib/` (contract_checks.json)
      - `out/evidence/run-{id}/abdf/` (boundary_events.json)
      - `out/evidence/run-{id}/summary/` (summary.json, summary.md)
    - Generate run ID: `run-{timestamp}-{commit_sha}`
    - Write metadata files with run context (commit, phase, validation standard)
    - _Requirements: 12.11_
    - _Guarantees: structured audit, CI integration_
  
  - [ ]* 22.6 Write property test for evidence append-only integrity
    - **Property 37: Evidence Append-Only Integrity** [REQUIRED]
    - **Validates: Requirements 12.1, 12.2**
    - Test that no overwrite or deletion is allowed in evidence chains
    - Test that append operations are strictly sequential
  
  - [ ]* 22.7 Write property test for slot chain isolation
    - **Property 38: Slot Chain Isolation** [REQUIRED]
    - **Validates: Requirements 12.7**
    - Test that Slot A cannot modify Slot B's evidence chain
    - Test that slot chains remain independent
  
  - [ ]* 22.8 Write property test for global anchor integrity
    - **Property 39: Global Anchor Integrity** [REQUIRED]
    - **Validates: Requirements 12.5, 12.6**
    - Test that slot head hash matches anchored hash in global chain
    - Test that anchor events are immutable and signed
  
  - [ ] 22.9 Write property test for evidence failure triggers fail-closed [CRITICAL]
    - **Property 40: Evidence Failure → Fail-Closed** [CRITICAL]
    - **Validates: Requirements 12.4, 12.8**
    - Test that if evidence write fails, execution MUST stop
    - Test that no execution proceeds without evidence
  
  - [ ]* 22.10 Write property test for deterministic evidence replay
    - **Property 41: Deterministic Evidence Replay** [REQUIRED]
    - **Validates: Requirements 12.2, 12.9**
    - Test that same execution produces identical evidence chain
    - Test that evidence chain is deterministically replayable

- [ ] 23. Implement Evidence System Integration (CRITICAL - BCIB/ABDF/VCP BINDING)
  - [ ] 23.1 Integrate evidence chain with VCP runtime hook
    - Modify `vcp_runtime_validate()` to emit signed evidence to slot-local chain
    - Modify `vcp_fail_closed()` to emit signed evidence to both slot and global chains
    - Modify `execution_slot_destroy()` to anchor slot chain to global chain with signed anchor
    - Ensure all validation events are recorded in evidence chains
    - _Requirements: 12.3, 12.5, 13.1_
  
  - [ ] 23.2 Integrate evidence chain with BCIB
    - Modify BCIB contract execution to emit signed evidence to slot-local chain
    - Ensure critical events (contract violations) emit signed evidence to global chain
    - Bind evidence to BCIB contract ID
    - _Requirements: 12.3, 12.6, 13.1_
  
  - [ ] 23.3 Integrate evidence chain with ABDF
    - Modify ABDF boundary crossing to emit signed evidence to slot-local chain
    - Ensure critical events (boundary violations) emit signed evidence to global chain
    - Bind evidence to ABDF snapshot hash (context_hash field)
    - _Requirements: 12.3, 12.6, 13.6_
  
  - [ ]* 23.4 Write integration tests for evidence system
    - Test slot-local chain creation and signed append
    - Test global chain creation and signed append
    - Test slot chain anchoring to global chain with signature verification
    - Test evidence failure triggers fail-closed
    - Test deterministic evidence replay with signature verification
    - Test ring buffer does not affect authority
    - Test forged evidence is rejected at all integration points
    - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.5, 12.6, 12.7, 12.8, 12.9, 13.1, 13.2, 13.3, 13.4_

- [ ] 24. Final checkpoint - Ensure all evidence system tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 25. Implement CI/Merge Governance (CRITICAL - AUTHORITY LAYER)
  - [ ] 25.1 Define CI merge policy [CRITICAL]
    - Document merge policy: merge ONLY if ci-freeze PASS
    - Document feature branch workflow: main → always green, feature/* → development
    - Document PR requirements: CRITICAL tests PASS, build PASS, local checks PASS
    - Add policy to `.kiro/governance/merge-policy.md`
    - _Requirements: 14.1_
    - _Guarantees: authority integrity, verified merges_
  
  - [ ] 25.2 Enforce ci-freeze gate before merge [CRITICAL]
    - Configure CI to block merge if ci-freeze fails
    - Add branch protection rules: require ci-freeze PASS status
    - Document ci-freeze as authority gate (not just quality check)
    - _Requirements: 14.2_
    - _Guarantees: no unverified code enters main_
  
  - [ ] 25.3 Block merge on CI failure [CRITICAL]
    - Configure CI to fail PR if any CRITICAL test fails
    - Add CI status check: CRITICAL tests → blocking, QUALITY tests → non-blocking
    - Document failure handling: FAIL → fix → re-run CI → merge
    - _Requirements: 14.3_
    - _Guarantees: CI authority enforcement_
  
  - [ ] 25.4 Define feature branch workflow [REQUIRED]
    - Document branch model: `main` (protected), `feature/*` (development)
    - Document commit discipline: local commits allowed, push requires local tests PASS
    - Document PR workflow: feature → PR → CI → review → merge
    - Add workflow documentation to `.kiro/governance/workflow.md`
    - _Requirements: 14.4_
    - _Guarantees: structured development, isolated changes_
  
  - [ ] 25.5 Define phase closure CI requirement [CRITICAL]
    - Document phase closure rule: phase closes ONLY if ci-freeze PASS + evidence exists
    - Add phase closure checklist: all CRITICAL tests PASS, all REQUIRED tests PASS, evidence chain verified
    - Document phase closure authority: ci-freeze = phase authority gate
    - _Requirements: 14.5_
    - _Guarantees: phase closure = verified authority_
  
  - [ ] 25.6 Document push discipline [REQUIRED]
    - Document local push: allowed for diagnostic, NOT authority
    - Document PR push: requires local CRITICAL tests PASS
    - Document merge push: requires CI ci-freeze PASS
    - Add push discipline to `.kiro/governance/push-discipline.md`
    - _Requirements: 14.6_
    - _Guarantees: clear authority boundaries_
  
  - [ ] 25.7 Prohibit CI override [CRITICAL]
    - Document CI override prohibition: NO admin bypass, NO emergency override, NO exceptions
    - Configure branch protection: disable "Allow administrators to bypass required pull requests"
    - Document emergency response: fix faster through CI, not bypass CI
    - Add override prohibition to `.kiro/governance/ci-override-prohibition.md`
    - _Requirements: 14.8_
    - _Guarantees: authority integrity, no bypass mechanism_
  
  - [ ]* 25.8 Add CI governance verification tests [REQUIRED]
    - Test that merge is blocked when ci-freeze fails
    - Test that CRITICAL test failure blocks merge
    - Test that phase closure requires ci-freeze PASS
    - _Requirements: 14.2, 14.3, 14.5_

- [ ] 26. Final checkpoint - Ensure all governance tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 27. Implement Naming & Directory Governance (CRITICAL - DETERMINISM FOUNDATION)
  - [ ] 27.1 Define naming specification [CRITICAL]
    - Document file naming convention: snake_case ONLY, lowercase ONLY
    - Document module prefix system for VCP components:
      - `vcp_*` → Validation Control Plane (runtime, evidence)
      - `bcib_*` → BCIB execution contracts
      - `boundary_*` → ABDF boundary enforcement
      - `slot_*` → Execution slot management
    - Document function naming: `<module>_<action>_<object>()`
    - Document struct naming: `struct <domain>_<entity>`
    - Document macro/enum naming: `UPPER_CASE` + domain prefix
    - Add specification to `.kiro/governance/naming-spec.md`
    - _Requirements: 15.1_
    - _Guarantees: deterministic naming, machine-parsable code_
  
  - [ ] 27.2 Enforce directory structure mapping [CRITICAL]
    - Document directory-to-domain mapping:
      - `kernel/sys/` → system-level enforcement (VCP, BCIB, ABDF, slots)
      - `kernel/include/` → public headers
      - `out/evidence/` → evidence chain artifacts
    - Document evidence directory structure (already defined in Task 22.5)
    - Ensure new VCP files follow existing kernel structure
    - _Requirements: 15.2_
    - _Guarantees: architectural clarity, module isolation_
  
  - [ ] 27.3 Ban generic filenames [CRITICAL]
    - Document forbidden names: `utils.c`, `helper.c`, `common.c`, `misc.c` (without domain prefix)
    - Exception: domain-specific managers allowed (e.g., `capability_manager.c` is valid)
    - Rationale: non-deterministic responsibility, architectural ambiguity
    - _Requirements: 15.3_
    - _Guarantees: clear responsibility boundaries_
  
  - [ ] 27.4 Define evidence file naming rules [CRITICAL]
    - Document strict evidence naming:
      - `global_chain.bin` (fixed name, no variation)
      - `local_chain.bin` (fixed name, no variation)
      - `head.hash` (fixed name, no variation)
      - `run-{timestamp}-{commit_sha}/` (deterministic run ID format)
    - Ban: random suffixes, timestamp drift, dynamic file names
    - Rationale: CI replay determinism requires fixed paths
    - _Requirements: 15.4_
    - _Guarantees: CI replay determinism, evidence integrity_
  
  - [ ] 27.5 Add CI naming lint check [REQUIRED]
    - Create `ci-naming-check.sh` script in `.ci/` directory
    - Check 1: Detect forbidden filenames (utils/helper/common without prefix)
    - Check 2: Detect uppercase in filenames
    - Check 3: Detect camelCase in code
    - Check 4: Verify VCP module prefix (vcp_*, bcib_*, boundary_*, slot_*)
    - Integrate into ci-freeze pipeline
    - _Requirements: 15.5_
    - _Guarantees: automated naming enforcement_
  
  - [ ] 27.6 Block merge on naming violation [CRITICAL]
    - Configure CI to fail if naming check fails
    - Add naming check to ci-freeze gate (blocking)
    - Document: naming violation = architectural violation = merge blocked
    - _Requirements: 15.6_
    - _Guarantees: naming discipline enforcement_
  
  - [ ]* 27.7 Add naming governance verification tests [REQUIRED]
    - Test that forbidden filenames are detected
    - Test that uppercase filenames are detected
    - Test that missing module prefix is detected
    - Test that naming violation blocks CI
    - _Requirements: 15.5, 15.6_

- [ ] 28. Final checkpoint - Ensure all naming governance tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 29. Implement ABDF Canonical Data Layer (CRITICAL - DETERMINISM FOUNDATION)
  - [ ] 29.1 Define ABDF as canonical internal format [CRITICAL]
    - Document: ABDF = canonical internal format (kernel execution format)
    - Document: Kernel execution ONLY accepts ABDF format
    - Document: External formats (JSON, CLI, AI output) MUST convert to ABDF before execution
    - **CRITICAL**: Converters are USERLAND tools, NOT kernel code
    - Add specification to `.kiro/governance/abdf-canonical-layer.md`
    - _Requirements: 16.4_
    - _Guarantees: deterministic execution, canonical truth_
  
  - [ ] 29.2 Document JSON → ABDF conversion contract [REQUIRED]
    - **USERLAND TOOL**: `ayken-core/crates/abdf-builder` (Rust) or `tools/ayken-cli` (userland)
    - **NOT KERNEL**: Kernel does NOT parse JSON
    - Document deterministic conversion contract: same JSON → same ABDF
    - Document JSON ambiguity handling (1 vs 1.0, field order, whitespace)
    - _Requirements: 16.5_
    - _Guarantees: JSON input determinism contract_
  
  - [ ] 29.3 Document CLI → ABDF conversion contract [REQUIRED]
    - **USERLAND TOOL**: `userspace/semantic-cli` or `tools/ayken-cli`
    - **NOT KERNEL**: Kernel does NOT parse CLI commands
    - Document deterministic conversion contract: same CLI input → same ABDF
    - _Requirements: 16.6_
    - _Guarantees: CLI input determinism contract_
  
  - [ ] 29.4 Document AI output → ABDF conversion contract [REQUIRED]
    - **USERLAND TOOL**: AI planner output → BCIB compiler → ABDF
    - **NOT KERNEL**: Kernel does NOT parse AI output
    - Document: AI output MUST go through ABDF canonicalization before execution
    - _Requirements: 16.7_
    - _Guarantees: AI input determinism contract, no AI bypass_
  
  - [ ] 29.5 Enforce "no execution without ABDF" in kernel [CRITICAL]
    - Modify `execution_slot_create()` to require ABDF payload
    - Reject non-ABDF payloads at execution slot creation
    - Kernel validates ABDF format, does NOT convert external formats
    - Ensure evidence context_hash includes ABDF snapshot hash
    - _Requirements: 16.8_
    - _Guarantees: ABDF enforcement, no bypass_
  
  - [ ] 29.6 Add property test for canonical equivalence [CRITICAL]
    - **Property 42: Canonical Determinism** [CRITICAL]
    - **Validates: Requirements 16.4, 16.5, 16.6, 16.7**
    - Test that Input A (JSON), Input B (CLI), Input C (AI) → ABDF(A) == ABDF(B) == ABDF(C)
    - Test that same input produces identical ABDF
    - Test that ABDF snapshot hash is deterministic
    - **NOTE**: This tests userland converters, not kernel
  
  - [ ]* 29.7 Add integration tests for ABDF canonical layer [REQUIRED]
    - Test userland JSON → ABDF → kernel execution
    - Test userland CLI → ABDF → kernel execution
    - Test userland AI output → ABDF → kernel execution
    - Test kernel rejects non-ABDF payload
    - _Requirements: 16.4, 16.5, 16.6, 16.7, 16.8_

- [ ] 30. Checkpoint - Ensure all ABDF canonical layer tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 31. Implement Future Extension Boundary Contract (CRITICAL - AI/USERLAND GUARDRAIL)
  - [ ] 31.1 Define AI/userland extension boundary [CRITICAL]
    - Document: AI planner = advisory-only (cannot execute directly)
    - Document: AI output → BCIB candidate → VCP validation → execution
    - Document: Userland CLI → VCP-bound execution slot
    - Document: Semantic CLI → BCIB compiler → VCP validation
    - Document: NO AI bypass, NO userland bypass
    - Add specification to `.kiro/governance/extension-boundary.md`
    - _Requirements: 16.9_
    - _Guarantees: future AI/userland integration without bypass_
  
  - [ ] 31.2 Implement AI output validation gate [CRITICAL]
    - Implement `ai_output_validate(ai_output)` function
    - Ensure AI output goes through: AI → ABDF → VCP → execution
    - Block direct AI → execution path
    - _Requirements: 16.10_
    - _Guarantees: AI cannot bypass validation_
  
  - [ ] 31.3 Implement userland CLI validation gate [CRITICAL]
    - Ensure CLI commands create VCP-bound execution slots
    - Block CLI → direct execution path
    - _Requirements: 16.11_
    - _Guarantees: CLI cannot bypass validation_
  
  - [ ] 31.4 Document extension integration pattern [REQUIRED]
    - Document pattern: External Input → ABDF Canonicalization → VCP Validation → Execution
    - Document: This pattern applies to ALL future extensions (AI, semantic planner, auto-execution, etc.)
    - Add pattern to `.kiro/governance/extension-pattern.md`
    - _Requirements: 16.12_
    - _Guarantees: consistent extension model_
  
  - [ ] 31.5 Add property test for extension boundary enforcement [CRITICAL]
    - **Property 43: Extension Boundary Enforcement** [CRITICAL]
    - **Validates: Requirements 16.9, 16.10, 16.11**
    - Test that AI output cannot bypass VCP validation
    - Test that CLI cannot bypass VCP validation
    - Test that all external inputs go through ABDF canonicalization

- [ ] 32. Checkpoint - Ensure all extension boundary tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 33. Implement Interaction & Control Surface Layer (CRITICAL - UI/GRAPH/AI GUARDRAIL)
  - [ ] 33.1 Define UI → ABDF builder contract [CRITICAL]
    - Document: UI actions MUST produce ABDF graph (NO direct syscall/execution)
    - Document: UI = description layer, NOT execution layer
    - Document: button → build ABDF → validate → run (if valid)
    - Add specification to `.kiro/governance/ui-abdf-contract.md`
    - _Requirements: 17.1_
    - _Guarantees: UI cannot bypass validation_
  
  - [ ] 33.2 Implement graph-based execution model [REQUIRED]
    - Define node-based execution graph: nodes (operations), edges (dependencies)
    - Implement `graph_to_abdf(graph)` function in userland ABDF builder
    - **CRITICAL**: This is a USERLAND tool, NOT kernel code
    - Ensure deterministic graph → ABDF conversion
    - Document: graph = visual programming language, ABDF = compiled form
    - _Requirements: 17.2_
    - _Guarantees: graph determinism, visual programming support_
  
  - [ ] 33.3 Define ABDF graph representation [CRITICAL]
    - Define ABDF node types: operation nodes, data nodes, control flow nodes
    - Define ABDF edge types: data dependency, control dependency, ordering
    - Ensure graph representation is immutable and deterministic
    - Add graph depth limit, cycle detection, bounded execution
    - _Requirements: 17.3_
    - _Guarantees: graph safety, no infinite loops_
  
  - [ ] 33.4 Implement data flow manipulation layer [REQUIRED]
    - Implement data transformations as ABDF nodes
    - Implement edges as dependency graph
    - Document: NO mutable runtime graph, ONLY immutable ABDF snapshot
    - _Requirements: 17.4_
    - _Guarantees: data flow determinism_
  
  - [ ] 33.5 Separate UI state from execution state [CRITICAL]
    - Document: UI state ≠ execution state
    - Document: UI drag-drop = design only, execution = snapshot-triggered
    - Implement state separation: UI state (mutable), execution state (immutable ABDF)
    - _Requirements: 17.5_
    - _Guarantees: UI cannot corrupt execution state_
  
  - [ ] 33.6 Implement preview/simulation layer [REQUIRED]
    - Implement `simulate_abdf_graph(graph)` function for validation preview
    - Simulate ABDF graph WITHOUT execution (dry-run mode)
    - Show validation results before execution
    - _Requirements: 17.6_
    - _Guarantees: safe preview, no side effects_
  
  - [ ] 33.7 Add property test for UI bypass prevention [CRITICAL]
    - **Property 44: UI Cannot Bypass VCP** [CRITICAL]
    - **Validates: Requirements 17.1, 17.5**
    - Test that UI actions cannot trigger execution without VCP validation
    - Test that all UI actions produce valid ABDF
  
  - [ ] 33.8 Add property test for graph determinism [CRITICAL]
    - **Property 45: Graph Determinism** [CRITICAL]
    - **Validates: Requirements 17.2, 17.3**
    - Test that same graph produces identical ABDF
    - Test that graph depth limit is enforced
    - Test that cycle detection works
  
  - [ ]* 33.9 Add integration tests for interaction layer [REQUIRED]
    - Test UI → ABDF → VCP → execution flow
    - Test graph → ABDF → execution flow
    - Test preview/simulation without execution
    - Test UI state separation from execution state
    - _Requirements: 17.1, 17.2, 17.3, 17.4, 17.5, 17.6_

- [ ] 34. Checkpoint - Ensure all interaction layer tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 35. Implement Graph Canonicalization Engine (CRITICAL - DETERMINISM FOUNDATION)
  - [ ] 35.1 Define canonical graph structure [CRITICAL]
    - Define `struct abdf_node` with deterministic fields: `node_id`, `node_type`, `operation`, `fields[]` (sorted), `inputs[]` (sorted), `outputs[]` (sorted)
    - Define `struct abdf_edge` with deterministic fields: `source_id`, `target_id`, `edge_type`, `weight`
    - Define `struct abdf_graph` with: `nodes[]` (sorted by node_id), `edges[]` (sorted by source_id, target_id), `graph_hash`
    - Document: All arrays MUST be sorted for canonical form
    - _Requirements: 18.1, 18.7_
    - _Guarantees: deterministic structure_
  
  - [ ] 35.2 Implement deterministic node ID assignment [CRITICAL]
    - Implement `compute_node_id(node)` using content-based hash (node_type || operation || fields)
    - Ensure same node content → same node_id
    - Alternative: topological sort + monotonic assignment (if content-hash not suitable)
    - Document: node_id MUST be deterministic, NOT insertion-order dependent
    - _Requirements: 18.2_
    - _Guarantees: stable node identifiers_
  
  - [ ] 35.3 Implement canonical node ordering [CRITICAL]
    - Implement `canonicalize_nodes(graph)` function
    - Sort nodes by node_id (ascending)
    - Sort fields within each node by field_id
    - Sort inputs/outputs within each node
    - _Requirements: 18.1, 18.7_
    - _Guarantees: deterministic node ordering_
  
  - [ ] 35.4 Implement canonical edge ordering [CRITICAL]
    - Implement `canonicalize_edges(graph)` function
    - Sort edges by (source_id, target_id) tuple
    - Ensure deterministic edge ordering
    - _Requirements: 18.1_
    - _Guarantees: deterministic edge ordering_
  
  - [ ] 35.5 Implement graph hash computation [CRITICAL]
    - Implement `compute_graph_hash(graph)` function
    - Hash canonical form: HASH(sorted_nodes || sorted_edges)
    - Ensure deterministic hash (same graph → same hash)
    - Store graph_hash in ABDF structure
    - _Requirements: 18.6_
    - _Guarantees: stable graph hash_
  
  - [ ] 35.6 Implement non-canonical graph rejection [CRITICAL]
    - Implement `validate_graph_canonicalization(graph)` function
    - Check 1: Nodes sorted by node_id
    - Check 2: Edges sorted by (source_id, target_id)
    - Check 3: Fields within nodes sorted by field_id
    - Reject graph if any check fails
    - Emit evidence describing canonicalization failure
    - _Requirements: 18.4, 18.8_
    - _Guarantees: only canonical graphs accepted_
  
  - [ ] 35.7 Integrate canonicalization into graph_to_abdf conversion [CRITICAL]
    - Modify `graph_to_abdf(input_graph, output)` to:
      1. Assign deterministic node IDs
      2. Convert to ABDF structure
      3. Canonicalize (sort nodes, edges, fields)
      4. Validate canonicalization
      5. Compute graph hash
    - Fail-closed if canonicalization fails
    - _Requirements: 18.3, 18.4_
    - _Guarantees: all ABDF graphs are canonical_
  
  - [ ] 35.8 Integrate graph_hash into evidence context [CRITICAL]
    - Modify `compute_evidence_context_hash()` to include graph_hash
    - Ensure evidence is bound to canonical graph structure
    - Document: context_hash = HASH(slot_id || contract_id || boundary_policy || abdf_snapshot_hash || graph_hash)
    - _Requirements: 18.6_
    - _Guarantees: evidence bound to canonical graph_
  
  - [ ] 35.9 Add property test for graph canonicalization determinism [CRITICAL]
    - **Property 46: Graph Canonicalization Determinism** [CRITICAL]
    - **Validates: Requirements 18.1, 18.2, 18.3, 18.5**
    - Test that Graph A (UI order) and Graph B (AI order) with same logical structure → ABDF(A) == ABDF(B)
    - Test that node ordering is deterministic
    - Test that edge ordering is deterministic
  
  - [ ] 35.10 Add property test for graph hash stability [CRITICAL]
    - **Property 47: Graph Hash Stability** [CRITICAL]
    - **Validates: Requirements 18.6**
    - Test that graph_hash is deterministic
    - Test that graph_hash does NOT depend on insertion order
    - Test that same graph → same hash
  
  - [ ] 35.11 Add property test for non-canonical graph rejection [CRITICAL]
    - **Property 48: Non-Canonical Graph Rejection** [CRITICAL]
    - **Validates: Requirements 18.4, 18.8**
    - Test that non-canonical graphs are rejected
    - Test that canonicalization failure emits evidence
    - Test that ambiguous graphs are rejected
  
  - [ ]* 35.12 Add integration tests for graph canonicalization [REQUIRED]
    - Test UI → canonical graph → ABDF
    - Test AI → canonical graph → ABDF
    - Test CLI → canonical graph → ABDF
    - Test same logical graph from different sources → identical ABDF
    - Test graph_hash included in evidence
    - _Requirements: 18.1, 18.2, 18.3, 18.4, 18.5, 18.6, 18.7, 18.8_

- [ ] 36. Checkpoint - Ensure all graph canonicalization tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 37. Implement Architecture Dependency Firewall (CRITICAL - FUTURE-PROOF BOUNDARY)
  - [ ] 37.1 Define architecture.manifest [CRITICAL]
    - Create `.kiro/governance/architecture.manifest` file
    - Define allowed module dependencies:
      - `kernel/sys` MAY depend on: `vcp_runtime`, `fail_closed`, `evidence`, `execution_slot`
      - `vcp_runtime` MUST NOT depend on: UI, AI, driver semantics, userland
      - `BCIB` MUST NOT depend on: driver implementation, device pointers
      - `ABDF` MUST NOT depend on: execution policy, validation logic
      - `evidence` MUST NOT depend on: UI, AI, driver
    - Define forbidden patterns:
      - UI → kernel direct execution (FORBIDDEN)
      - AI → execution direct (FORBIDDEN)
      - BCIB → driver pointer (FORBIDDEN)
      - driver → ABDF policy logic (FORBIDDEN)
      - VCP → UI/AI semantic dependency (FORBIDDEN)
    - _Requirements: 19.1, 19.2, 19.4, 19.7_
    - _Guarantees: explicit dependency contract_
  
  - [ ] 37.2 Implement ci-gate-dependency-graph [CRITICAL]
    - Create `ci-dependency-check.sh` script in `.ci/` directory
    - Parse architecture.manifest
    - Analyze source code for #include dependencies
    - Build dependency graph
    - Detect circular dependencies
    - Detect forbidden dependencies
    - Fail CI if violations detected
    - _Requirements: 19.3, 19.5, 19.6_
    - _Guarantees: automated dependency enforcement_
  
  - [ ] 37.3 Block circular dependencies [CRITICAL]
    - Implement cycle detection in dependency graph
    - If module A depends on B and B depends on A → CI FAIL
    - Emit evidence describing circular dependency
    - _Requirements: 19.3_
    - _Guarantees: no circular dependencies_
  
  - [ ] 37.4 Block forbidden dependencies [CRITICAL]
    - Check each source file against architecture.manifest
    - If forbidden dependency detected → CI FAIL
    - Examples:
      - UI code includes kernel execution headers → FAIL
      - AI code includes direct syscall headers → FAIL
      - BCIB code includes driver pointer headers → FAIL
    - _Requirements: 19.4_
    - _Guarantees: no bypass paths_
  
  - [ ] 37.5 Integrate dependency check into ci-freeze [CRITICAL]
    - Add dependency check to ci-freeze pipeline (blocking)
    - Dependency violation → CI FAIL → merge BLOCKED
    - Document: dependency violation = architectural violation
    - _Requirements: 19.6_
    - _Guarantees: CI authority enforcement_
  
  - [ ]* 37.6 Add property test for dependency firewall [REQUIRED]
    - **Property 49: Architecture Dependency Firewall** [REQUIRED]
    - **Validates: Requirements 19.2, 19.3, 19.4**
    - Test that forbidden dependencies are detected
    - Test that circular dependencies are detected
    - Test that dependency violations block CI

- [ ] 38. Implement Device-Originated Data Boundary (CRITICAL - FUTURE DEVICE INTEGRATION)
  - [ ] 38.1 Define DeviceEvent ABDF segment contract [CRITICAL]
    - Define `struct abdf_device_event` in `kernel/include/abdf_device.h`:
      - `event_type` (INPUT / STATUS / ERROR)
      - `source_device_id` (device identifier)
      - `logical_timestamp` (monotonic counter, NOT wall clock)
      - `event_data` (device-specific payload)
      - `capability_id` (required capability for this event)
    - Document: Driver output MUST be converted to ABDF DeviceEvent before execution
    - _Requirements: 20.1, 20.2, 20.3_
    - _Guarantees: canonical device event format_
  
  - [ ] 38.2 Define InputEvent ABDF segment contract [CRITICAL]
    - Define `struct abdf_input_event` for keyboard, mouse, touch, sensor:
      - `input_type` (KEYBOARD / MOUSE / TOUCH / SENSOR)
      - `source_device_id`
      - `logical_timestamp`
      - `input_data` (key code, mouse position, touch coordinates, sensor reading)
      - `capability_id`
    - Document: All user input MUST follow canonical InputEvent format
    - _Requirements: 20.6_
    - _Guarantees: canonical input format_
  
  - [ ] 38.3 Define DeviceStatus ABDF segment contract [CRITICAL]
    - Define `struct abdf_device_status` for device state changes:
      - `status_type` (CONNECTED / DISCONNECTED / ERROR / READY)
      - `source_device_id`
      - `logical_timestamp`
      - `status_data` (device-specific status)
    - Document: Device state changes MUST be ABDF-typed
    - _Requirements: 20.7_
    - _Guarantees: canonical device status format_
  
  - [ ] 38.4 Require capability for device-originated execution [CRITICAL]
    - Document: Device-originated execution MUST be capability-bound
    - Device event → execution slot creation MUST check capability
    - No capability → fail-closed
    - _Requirements: 20.4_
    - _Guarantees: device access control_
  
  - [ ] 38.5 Require evidence for device-originated BCIB [CRITICAL]
    - Document: Device-triggered BCIB execution MUST emit evidence
    - Evidence MUST include: device_id, event_type, capability_id, timestamp
    - Device-originated execution without evidence → fail-closed
    - _Requirements: 20.5_
    - _Guarantees: device execution audit trail_
  
  - [ ] 38.6 Block direct device → execution path [CRITICAL]
    - Document: Device input MUST go through ABDF → VCP → execution
    - Direct device → execution bypass is FORBIDDEN
    - Add to architecture.manifest: device → execution direct (FORBIDDEN)
    - _Requirements: 20.8_
    - _Guarantees: no device bypass_
  
  - [ ]* 38.7 Add property test for device boundary enforcement [REQUIRED]
    - **Property 50: Device-Originated Data Boundary** [REQUIRED]
    - **Validates: Requirements 20.1, 20.4, 20.5, 20.8**
    - Test that device events follow ABDF contract
    - Test that device execution requires capability
    - Test that device execution emits evidence
    - Test that direct device → execution is blocked

- [ ] 39. Implement Performance Budget Contract (CRITICAL - DETERMINISTIC BOUNDS)
  - [ ] 39.1 Define VCP validation operation bound [CRITICAL]
    - Document maximum bounded operations for `vcp_runtime_validate()`
    - **CRITICAL**: Use bounded operations (hash computations, comparisons), NOT cycle count
    - Profile validation path: count hash operations, signature verifications, comparisons
    - Set deterministic upper bound (e.g., max 10 hash ops, 1 signature verification)
    - Add operation counter in validation code
    - _Requirements: 21.1_
    - _Guarantees: bounded validation operations_
  
  - [ ] 39.2 Define evidence append operation bound [CRITICAL]
    - Document maximum bounded operations for evidence chain append
    - **CRITICAL**: Use bounded operations (writes, hash computations), NOT cycle count
    - Profile evidence append: count write operations, hash computations
    - Set deterministic upper bound
    - Add operation counter in evidence code
    - _Requirements: 21.2_
    - _Guarantees: bounded evidence operations_
  
  - [ ] 39.3 Define signature verification operation budget [CRITICAL]
    - Document maximum bounded operations for signature verification
    - **CRITICAL**: Use bounded operations (cryptographic operations), NOT cycle count
    - Profile signature verification: count cryptographic operations
    - Set deterministic upper bound
    - Add operation counter in signature code
    - _Requirements: 21.3_
    - _Guarantees: bounded signature operations_
  
  - [ ] 39.4 Define fail-closed path operation budget [CRITICAL]
    - Document maximum bounded operations for fail-closed enforcement
    - **CRITICAL**: Use bounded operations, NOT cycle count
    - Profile fail-closed path: count operations
    - Set deterministic upper bound
    - Ensure fail-closed completes within budget
    - _Requirements: 21.4_
    - _Guarantees: bounded fail-closed operations_
  
  - [ ] 39.5 Implement fallback behavior on budget exceeded [CRITICAL]
    - If validation exceeds operation budget → fail-closed with evidence "validation operation budget exceeded"
    - If evidence exceeds operation budget → fail-closed with evidence "evidence operation budget exceeded"
    - If signature exceeds operation budget → fail-closed with evidence "signature operation budget exceeded"
    - Document: operation budget exceeded = critical failure, not recoverable
    - _Requirements: 21.5_
    - _Guarantees: budget overflow handling_
  
  - [ ] 39.6 Document performance budget [REQUIRED]
    - Create `.kiro/governance/performance-budget.md`
    - Document maximum bounded operations for each enforcement path
    - Document maximum memory for each enforcement path
    - Document maximum I/O operations for each enforcement path
    - **CRITICAL**: Document why bounded operations (not cycle count) ensures determinism
    - _Requirements: 21.6_
    - _Guarantees: explicit performance contract_
  
  - [ ] 39.7 Test under load [REQUIRED]
    - Create stress test for validation enforcement
    - Test validation under high load (1000+ concurrent slots)
    - Verify operation budget maintained under load
    - _Requirements: 21.7_
    - _Guarantees: load resilience_
  
  - [ ]* 39.8 Add property test for performance budget [REQUIRED]
    - **Property 51: Performance Budget Enforcement** [REQUIRED]
    - **Validates: Requirements 21.1, 21.2, 21.3, 21.4, 21.5**
    - Test that validation completes within operation budget
    - Test that evidence append completes within operation budget
    - Test that signature verification completes within operation budget
    - Test that operation budget exceeded triggers fail-closed

- [ ] 40. Final checkpoint - Ensure all boundary and budget tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

### Test Classification System

Tests are classified by criticality, NOT by "optional" status:

- **[CRITICAL]** - System is UNSAFE without this test. Blocking for all phases. Must pass before any merge.
  - Examples: fail-closed enforcement, trust verification, signature integrity
  - **Rule**: If CRITICAL test fails → STOP, no further tasks until fixed
  
- **[REQUIRED]** - System is UNVERIFIABLE without this test. Needed for correctness guarantees.
  - Examples: append-only integrity, slot isolation, deterministic replay
  - **Rule**: Must pass before phase closure
  
- **[QUALITY]** - System works but audit/debug quality degrades without this test.
  - Examples: comprehensive evidence, CLI evidence emission
  - **Rule**: Should pass but not blocking

**IMPORTANT**: The `*` marker in tasks indicates test implementation timing flexibility for REQUIRED and QUALITY tests only. **CRITICAL tests have NO `*` marker** - they are mandatory and must be implemented immediately.

### Implementation Guidelines

- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- Unit tests validate specific examples and edge cases
- **Implementation language**: C for kernel enforcement core
- **Constitutional compliance**: All code must comply with NON_OVERRIDABLE rules (no global state, no capability bypass, no Ring3→Ring0 direct access, no audit tampering)
- **Determinism**: All runtime enforcement must be deterministic (no global state mutations)
- **Memory safety**: All code paths must be memory-safe (no leaks, no undefined behavior)
- **Fail-closed**: All validation failures must result in execution blocking (no bypass)

## Workflow Completion

This workflow is ONLY for creating design and planning artifacts. Implementation of these tasks should be performed separately by:
1. Opening the `tasks.md` file
2. Clicking "Start task" next to task items to begin execution

---

## MVP Implementation Priority

**CRITICAL**: Do NOT attempt all tasks simultaneously. Follow this MVP sequence:

### Phase 1: Core Enforcement (MVP - Foundation)
1. **Task 1** (Execution Slot Validation State) - Foundation with FINAL ABI
2. **Task 18** (Trust Token Verification) - **CRITICAL: SYSTEM HEART - MUST BE DONE BEFORE HOOK**
3. **Task 2** (VCP Runtime Hook) - Core enforcement point (uses Task 18 verification)
4. **Task 4** (Fail-Closed Mechanism) - Security guarantee

**CRITICAL ORDER RATIONALE:**
- Task 1 → ABI lock (no verification without structure)
- Task 18 → Trust verification (capability + context + signature + nonce)
- Task 2 → Runtime hook (calls `vcp_verify_validation_state()` from Task 18)
- Task 4 → Fail-closed (triggered by Task 2 when verification fails)

**WITHOUT Task 18 FIRST:**
- Task 2 hook is meaningless (no verification function to call)
- Task 4 fail-closed blocks wrong things (no trust verification)
- System becomes "trusted-input" not "verified-input"
- **Result: Fake state accepted, security theater**

### Phase 2: Evidence System (CRITICAL - MUST BE DONE BEFORE PRODUCTION)
5. **Task 20** (Evidence Foundation - Determinism & Emission)
6. **Task 21** (Evidence Trust Layer - Signature & Verification) ← **CRITICAL**
7. **Task 22** (Evidence Chain Architecture - Hybrid Model)
8. **Task 23** (Evidence System Integration)

### Phase 3: ABDF Canonical Layer (CRITICAL - MUST BE DONE BEFORE BINDING)
9. **Task 29** (ABDF Canonical Layer) - **CRITICAL: MUST BE DONE BEFORE BCIB/ABDF BINDING**

### Phase 4: Binding Integration
10. **Task 7** (BCIB Binding)
11. **Task 8** (ABDF Binding)
12. **Task 10** (CLI Authority Reduction)
13. **Task 16.1** (Wire enforcement points) - Integration
14. **Task 16.2** (System-wide verification) - Validation

**CRITICAL ORDER RULE**: Task 1 (ABI) → Task 18 (Trust) → Task 2 (Hook) → Task 4 (Fail-Closed) → Evidence (Task 20-23) → ABDF Canonical (Task 29) → Binding (Task 7-10)

### Phase 5: Hardening
15. **Task 11** (Lifecycle Management)
16. **Task 13** (Constitutional Compliance)
17. **Task 14** (Performance & Reliability)
18. **Task 15** (CI-Runtime Consistency)

### Phase 6: Governance (CRITICAL - AUTHORITY CONTROL)
19. **Task 25** (CI/Merge Governance) - **MUST BE DONE BEFORE PRODUCTION**
20. **Task 27** (Naming & Directory Governance) - **MUST BE DONE BEFORE PRODUCTION**

### Phase 7: Extension Boundary (CRITICAL - FUTURE-PROOF)
21. **Task 31** (Future Extension Boundary) - **MUST BE DONE BEFORE AI/USERLAND**
22. **Task 33** (Interaction & Control Surface Layer) - **MUST BE DONE BEFORE UI/GRAPH**
23. **Task 35** (Graph Canonicalization Engine) - **MUST BE DONE BEFORE UI/GRAPH/AI**

### Phase 8: Authority Foundation (CRITICAL - ARCHITECTURAL FIREWALL)
24. **Task 37** (Architecture Dependency Firewall) - **MUST BE DONE BEFORE PRODUCTION**
25. **Task 38** (Device-Originated Data Boundary) - **MUST BE DONE BEFORE DRIVER INTEGRATION**
26. **Task 39** (Performance Budget Contract) - **MUST BE DONE BEFORE PRODUCTION**

**CRITICAL Rule**: If any CRITICAL test fails → STOP, no further tasks until fixed. CI authority gate (ci-freeze) is non-negotiable.

**Naming Rule**: Naming violations are architectural violations. CI blocks merge on naming check failure.

**Extension Rule**: ALL external inputs (AI, CLI, userland) MUST go through ABDF → VCP → Execution. No bypass allowed.

**Interaction Rule**: UI/graph NEVER executes directly. UI = description layer, execution = ABDF snapshot-triggered.

**Canonicalization Rule**: Graph → ABDF conversion MUST be deterministic. Same logical graph → identical ABDF binary, regardless of input source (UI/AI/CLI).

**Dependency Rule**: Architecture dependencies MUST be validated by CI. Forbidden dependencies (UI → kernel, AI → execution, BCIB → driver pointer) are blocked.

**Device Rule**: Device inputs MUST follow ABDF canonical contract. Direct device → execution bypass is FORBIDDEN.

**Performance Rule**: All enforcement paths MUST have deterministic operation bounds (NOT cycle count). Operation budget exceeded → fail-closed.

**Property Tests Marked "Optional" Are NOT Optional for Production:**
- **CRITICAL Tests** (blocking, must pass before any merge):
  - Property 2, 3, 4: Fail-closed enforcement
  - Property 9, 10, 11: Fail-closed integrity
  - Property 24-28: Trust verification
  - Property 29: Deterministic evidence format
  - Property 34-36: Evidence signature & verification
  - Property 40: Evidence failure → fail-closed
  - Property 46-48: Graph canonicalization determinism
  
- **REQUIRED Tests** (must pass before phase closure):
  - Property 1: Validation state initialization
  - Property 37-39: Evidence chain integrity
  - Property 41: Deterministic replay
  - Property 49: Architecture dependency firewall
  - Property 50: Device-originated data boundary
  - Property 51: Performance budget enforcement
  
- **QUALITY Tests** (should pass but not blocking):
  - Property 8: Comprehensive evidence emission

**The `*` marker indicates timing flexibility, NOT importance. All CRITICAL tests MUST be implemented before production.**

**Why Task 18 (Trust Verification) is Critical:**
- Without trust verification, the system is a "trusted-input system" (vulnerable to fake state injection)
- With trust verification, the system becomes a "verified-input system" (cryptographically secure)
- **CRITICAL**: Trust verification MUST happen BEFORE runtime hook (Task 2) because hook calls `vcp_verify_validation_state()`
- Task 2 without Task 18 = meaningless hook (no verification function exists)
- Task 4 without Task 18 = fail-closed blocks wrong things (no trust checks)
- **Correct order**: Task 1 (ABI) → Task 18 (Trust) → Task 2 (Hook) → Task 4 (Fail-Closed)

**Why Task 21 (Evidence Trust Layer) is Critical:**
- Without evidence trust, the system produces "logs" not "proof"
- Evidence signature ensures authenticity (who produced this evidence?)
- Evidence verification ensures integrity (was this evidence tampered with?)
- Trust layer transforms evidence from "audit-friendly" to "cryptographically verifiable"
- **CRITICAL RULE**: Unsigned evidence = invalid evidence = fail-closed

**Why Task 22 (Evidence Chain Architecture) is Critical:**
- Without evidence chain, the system can enforce but cannot prove enforcement
- Evidence chain transforms the system from "secure-looking" to "provable"
- Hybrid model (slot-local + global + anchoring) provides isolation, authority, and replayability
- Evidence failure → fail-closed ensures audit cannot be bypassed
- **CRITICAL RULE**: Execution without evidence emission is invalid

**Why Task 35 (Graph Canonicalization Engine) is Critical:**
- Without graph canonicalization, same logical graph from different sources (UI/AI/CLI) produces different ABDF
- Different ABDF → different hashes → different evidence → replay fails → determinism broken
- Graph canonicalization ensures: UI order ≠ AI order ≠ CLI order → same canonical form → identical ABDF
- This completes the determinism chain: Input → Graph → Canonicalization → ABDF → Evidence → Verification
- **CRITICAL RULE**: Non-canonical graphs MUST be rejected before ABDF conversion

---

## Signature

```
────────────────────────────────────────
Kenan AY
Architectural Steward — AykenOS

Document: AYKEN VCP Execution Binding - Implementation Plan
Status: APPROVED (Authority Foundation Complete)
Scope: Runtime validation enforcement with trust, graph determinism, architecture firewall, device boundary, and performance budget

Date: 2026-05-03
────────────────────────────────────────
```

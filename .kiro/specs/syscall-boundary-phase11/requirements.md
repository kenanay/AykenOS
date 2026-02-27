# Requirements Document: Syscall Boundary Phase 11

## Introduction

Phase 11 implements the syscall boundary mechanism as the controlled transition from Ring3 policy back to Ring0 mechanism. This phase establishes syscall 1000 (exit) as the minimal bring-up syscall, proving deterministic Ring3→Ring0 transitions via the intended mechanism (SYSCALL instruction) rather than exceptions.

Building on Phase 10-A's Ring3 execution proof, Phase 11 completes the bidirectional Ring0↔Ring3 boundary with marker-based evidence and constitutional compliance.

## Glossary

- **Syscall_Handler**: Ring0 mechanism that dispatches syscall requests (1000-1010 range)
- **Syscall_Entry**: Assembly entry point for SYSCALL instruction (MSR-configured)
- **Syscall_Exit**: Return path from Ring0 back to Ring3 (SYSRET instruction)
- **Ring3_Context**: User-mode execution state (CPL=3, user stack, user RIP)
- **Ring0_Context**: Kernel-mode execution state (CPL=0, kernel stack, kernel RIP)
- **BCIB_Hook_Point**: Future integration point for BCIB execution engine (Phase 12+)
- **Syscall_1000**: Exit syscall - terminates current process/thread
- **CPL**: Current Privilege Level (0=Ring0, 3=Ring3)
- **MSR**: Model-Specific Register (IA32_LSTAR, IA32_STAR, IA32_FMASK)

## Requirements

### Requirement 1: Syscall Entry Mechanism

**User Story:** As a Ring3 process, I want to invoke syscall 1000 via SYSCALL instruction, so that I can request kernel services through the intended mechanism.

#### Acceptance Criteria

1. WHEN a Ring3 process executes SYSCALL instruction, THE Syscall_Entry SHALL transition to Ring0 with CPL=0
2. WHEN Syscall_Entry executes, THE Syscall_Entry SHALL switch from user stack to kernel stack
3. WHEN Syscall_Entry executes, THE Syscall_Entry SHALL preserve all general-purpose registers (RAX, RBX, RCX, RDX, RSI, RDI, R8-R15)
4. WHEN Syscall_Entry executes, THE Syscall_Entry SHALL save Ring3_Context (RIP, RSP, RFLAGS) for return
5. WHEN MSRs are configured, THE System SHALL set IA32_LSTAR to Syscall_Entry address
6. WHEN MSRs are configured, THE System SHALL set IA32_STAR with Ring0/Ring3 segment selectors
7. WHEN MSRs are configured, THE System SHALL set IA32_FMASK to disable interrupts during syscall entry

### Requirement 2: Syscall Handler Dispatch

**User Story:** As a kernel developer, I want syscall handler to dispatch based on syscall number, so that I can route requests to appropriate mechanisms.

#### Acceptance Criteria

1. WHEN Syscall_Handler receives syscall number in RAX, THE Syscall_Handler SHALL validate it is in range 1000-1010
2. WHEN syscall number is 1000, THE Syscall_Handler SHALL invoke exit mechanism
3. WHEN syscall number is outside 1000-1010, THE Syscall_Handler SHALL return error code -ENOSYS
4. THE Syscall_Handler SHALL NOT contain policy decisions (scheduler logic, VFS access control, AI inference)
5. WHEN Syscall_Handler completes, THE Syscall_Handler SHALL place return value in RAX
6. WHEN error occurs, THE Syscall_Handler SHALL return negative errno value in RAX

### Requirement 3: Syscall Exit Mechanism

**User Story:** As a Ring3 process, I want syscall to return control to Ring3, so that I can continue execution after kernel service.

#### Acceptance Criteria

1. WHEN Syscall_Exit executes, THE Syscall_Exit SHALL restore Ring3_Context (RIP, RSP, RFLAGS)
2. WHEN Syscall_Exit executes, THE Syscall_Exit SHALL transition to Ring3 with CPL=3
3. WHEN Syscall_Exit executes, THE Syscall_Exit SHALL switch from kernel stack to user stack
4. WHEN Syscall_Exit executes, THE Syscall_Exit SHALL preserve return value in RAX
5. WHEN Syscall_Exit executes, THE Syscall_Exit SHALL use SYSRET instruction for Ring0→Ring3 transition
6. WHEN Syscall_Exit completes, THE System SHALL resume Ring3 execution at saved RIP

### Requirement 4: Syscall 1000 Implementation

**User Story:** As a Ring3 process, I want to invoke syscall 1000 (exit), so that I can terminate cleanly.

#### Acceptance Criteria

1. WHEN syscall 1000 is invoked, THE System SHALL terminate current process/thread
2. WHEN syscall 1000 executes, THE System SHALL clean up process resources (memory, handles)
3. WHEN syscall 1000 completes, THE System SHALL NOT return to Ring3 (process terminated)
4. WHEN syscall 1000 is invoked with exit code in RDI, THE System SHALL record exit code
5. THE Syscall_1000 implementation SHALL NOT contain policy decisions

### Requirement 5: Marker-Based Proof

**User Story:** As a kernel developer, I want marker-based proof of syscall boundary, so that I can validate deterministic Ring3→Ring0→Ring3 transitions.

#### Acceptance Criteria

1. WHEN Ring3 code prepares syscall, THE System SHALL emit marker RING3_BEFORE_SYSCALL
2. WHEN Syscall_Entry begins, THE System SHALL emit marker SYSCALL_ENTRY
3. WHEN Syscall_Handler dispatches, THE System SHALL emit marker SYSCALL_HANDLER_1000
4. WHEN Syscall_Exit begins, THE System SHALL emit marker SYSCALL_EXIT
5. WHEN syscall completes, THE System SHALL emit marker RING3_AFTER_SYSCALL (if returning)
6. WHEN markers are emitted, THE System SHALL include CPL, RIP, RSP, RAX in marker data

### Requirement 6: Register Preservation

**User Story:** As a Ring3 process, I want syscall to preserve my registers, so that I can continue execution with correct state.

#### Acceptance Criteria

1. WHEN syscall executes, THE System SHALL preserve RBX, RBP, R12, R13, R14, R15 (callee-saved)
2. WHEN syscall executes, THE System SHALL allow RAX, RCX, R11 to be clobbered (per AMD64 ABI)
3. WHEN syscall returns, THE System SHALL preserve RDI, RSI, RDX, R8, R9, R10 (argument registers)
4. WHEN syscall returns, THE System SHALL place return value in RAX
5. WHEN syscall returns, THE System SHALL preserve RFLAGS (except IF, which is restored)

### Requirement 7: Stack Switching

**User Story:** As a kernel developer, I want deterministic stack switching during syscall, so that Ring0 and Ring3 stacks remain isolated.

#### Acceptance Criteria

1. WHEN Syscall_Entry executes, THE System SHALL switch RSP from user stack to kernel stack
2. WHEN Syscall_Exit executes, THE System SHALL switch RSP from kernel stack to user stack
3. WHEN stack switch occurs, THE System SHALL validate kernel stack is within valid range
4. WHEN stack switch occurs, THE System SHALL save user RSP for return
5. THE System SHALL NOT allow Ring3 to directly access kernel stack

### Requirement 8: BCIB Hook Point Identification

**User Story:** As a kernel architect, I want BCIB hook point identified in syscall path, so that Phase 12+ can integrate BCIB execution engine.

#### Acceptance Criteria

1. WHEN Syscall_Handler dispatches, THE System SHALL identify BCIB_Hook_Point before mechanism invocation
2. THE BCIB_Hook_Point SHALL be documented in design with integration strategy
3. THE BCIB_Hook_Point SHALL NOT be implemented in Phase 11 (out of scope)
4. WHEN BCIB_Hook_Point is identified, THE System SHALL document required context (registers, stack, CPL)

### Requirement 9: Constitutional Compliance

**User Story:** As a kernel architect, I want syscall boundary to comply with constitutional rules, so that Phase 11 passes all CI gates.

#### Acceptance Criteria

1. THE Syscall_Handler SHALL NOT contain policy decisions (Rule 1: Ring0 Policy Prohibition)
2. WHEN syscall ABI is defined, THE System SHALL use range 1000-1010 only (Rule 2: ABI Stability)
3. WHEN new Ring0 exports are added, THE System SHALL require ADR approval (Rule 3: Ring0 Export Surface)
4. WHEN evidence is generated, THE System SHALL NOT modify evidence directory manually (Rule 4: Evidence Integrity)
5. THE Syscall boundary SHALL be deterministic and reproducible (Rule 5: Determinism Requirement)

### Requirement 10: Validation Test

**User Story:** As a kernel developer, I want validation test for syscall 1000, so that I can prove Ring3→Ring0→exit transition works.

#### Acceptance Criteria

1. WHEN validation test runs, THE System SHALL load Ring3 ELF that invokes syscall 1000
2. WHEN Ring3 code executes SYSCALL instruction, THE System SHALL transition to Ring0 via Syscall_Entry
3. WHEN Syscall_Handler processes syscall 1000, THE System SHALL terminate process cleanly
4. WHEN validation test completes, THE System SHALL emit marker sequence: RING3_BEFORE_SYSCALL → SYSCALL_ENTRY → SYSCALL_HANDLER_1000
5. WHEN validation test completes, THE System SHALL NOT triple fault or reset loop

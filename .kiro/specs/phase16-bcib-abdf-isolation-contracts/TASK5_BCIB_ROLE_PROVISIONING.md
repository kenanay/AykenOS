# Task 5: BCIB Role Provisioning Design

## Problem Statement

**Current Blocker**: Execution delivery proof payload cannot test submit → pickup → delivery → complete pipeline because:
- `SYS_V2_SUBMIT_EXECUTION` (syscall 1003) requires `PROC_EXECUTION_ROLE_BCIB`
- Default user processes have `PROC_EXECUTION_ROLE_USER`
- Syscall enforcement matrix correctly blocks USER role from calling submit_execution
- This is NOT a bug - it's correct architectural enforcement

**Evidence**:
- Trace: `[[AYKEN_BOUNDARY_KILL]]` with "Unauthorized use of BCIB execution interface"
- Enforcement: `syscall_enforcement_validate()` returns `BOUNDARY_ERR_UNAUTHORIZED_SYSCALL`
- Process defaults: USER processes start as `PROC_EXECUTION_ROLE_USER`

## Design Constraints

### Constitutional Requirements
- `SECURITY.BOUNDARY.VIOLATION` - Role enforcement is NON_OVERRIDABLE
- `KERNEL.SAFETY.CRITICAL` - Role assignment must be kernel-authoritative
- No backdoors, no role escalation, no enforcement bypass

### Architectural Requirements
- USER/BCIB role separation must remain strict
- Role assignment must be deterministic and auditable
- Test infrastructure must not weaken production security
- Role provisioning must be explicit, not implicit

### Forbidden Approaches
- ❌ Forcing USER process to BCIB role via hack
- ❌ Disabling enforcement matrix for tests
- ❌ String-based or pattern-based role assignment
- ❌ Allowing USER role to call submit_execution
- ❌ Hidden or implicit role transitions

## Proposed Solution: Kernel-Created BCIB Worker Context

### Overview
Create a dedicated kernel-managed BCIB worker process for validation testing. This approach:
- Maintains strict USER/BCIB separation
- Provides deterministic role assignment
- Enables end-to-end pipeline testing
- Preserves production security model

### Implementation Components

#### 1. BCIB Worker Process Creation (Kernel-Side)

**File**: `kernel/proc/bcib_worker.c` (new)

**Responsibilities**:
- Create dedicated BCIB-role process during kernel initialization (validation profile only)
- Assign `PROC_EXECUTION_ROLE_BCIB` at process creation
- Map execution inbox and payload regions
- Initialize process with BCIB worker entry point

**Key Functions**:
```c
// Create BCIB worker process (validation profile only)
int bcib_worker_create(void);

// Get BCIB worker PID (for test coordination)
uint64_t bcib_worker_get_pid(void);
```

**Acceptance Criteria**:
- Worker process created with PID deterministically assigned
- Process has `PROC_EXECUTION_ROLE_BCIB` from creation
- Worker process has inbox/payload regions mapped
- Worker process can call `SYS_V2_SUBMIT_EXECUTION` without enforcement violation

#### 2. BCIB Worker Userspace Payload

**File**: `userspace/minimal/minimal_bcib_worker.S` (new)

**Responsibilities**:
- Submit minimal BCIB graph via `SYS_V2_SUBMIT_EXECUTION`
- Target execution to USER worker process (PID 2)
- Emit markers for submit success/failure
- Exit cleanly after submission

**Markers**:
- `BW_START` - Worker started
- `BW_SUBMIT_OK` - Submission succeeded
- `BW_SUBMIT_FAIL` - Submission failed
- `BW_EXIT` - Worker exiting

**Acceptance Criteria**:
- Worker can call `SYS_V2_SUBMIT_EXECUTION` without kill
- Submission returns valid execution_id
- No `[[AYKEN_BOUNDARY_KILL]]` marker
- Markers visible in QEMU trace

#### 3. USER Worker Delivery Validation

**File**: `userspace/minimal/minimal_execution_delivery_proof.S` (existing, modified)

**Responsibilities**:
- Poll inbox until `AXIB_STATE_READY`
- Read execution_id from inbox
- Complete execution via `SYS_V2_COMPLETE_EXECUTION`
- Emit delivery/completion markers

**Markers**:
- `UW_START` - USER worker started
- `UW_DELIVERED` - Inbox delivery detected
- `UW_COMPLETE_OK` - Completion succeeded
- `UW_COMPLETE_FAIL` - Completion failed

**Acceptance Criteria**:
- USER worker can poll inbox without enforcement violation
- Inbox transitions to `AXIB_STATE_READY` after BCIB submission
- USER worker can call `SYS_V2_COMPLETE_EXECUTION` with delivered execution_id
- No role escalation or boundary violation

#### 4. Build System Integration

**Files**: `Makefile`, `userspace/minimal/Makefile`

**New Mode**: `execution-pipeline-proof`

**Build Configuration**:
- Kernel: validation profile, BCIB worker enabled
- Userspace: Dual payload (BCIB worker + USER worker)
- QEMU: Both processes launched, coordinated execution

**Acceptance Criteria**:
- `USER_MINIMAL_MODE=execution-pipeline-proof` builds successfully
- EFI.img contains both BCIB and USER worker payloads
- Kernel initializes BCIB worker at boot

#### 5. QEMU Proof Harness

**File**: `scripts/qemu-execution-pipeline-proof-harness.sh` (new)

**Responsibilities**:
- Build EFI image with execution-pipeline-proof mode
- Run QEMU with proper firmware and timeout
- Capture kernel trace (debugcon + serial)
- Validate marker sequence

**Expected Marker Flow**:
```
BW_START          (BCIB worker starts)
BW_SUBMIT_OK      (BCIB submits execution)
UW_START          (USER worker starts)
UW_DELIVERED      (USER worker sees inbox delivery)
UW_COMPLETE_OK    (USER worker completes execution)
BW_EXIT           (BCIB worker exits)
```

**Acceptance Criteria**:
- All markers present in correct order
- No `[[AYKEN_BOUNDARY_KILL]]` markers
- No enforcement violations
- Deterministic execution across multiple runs

## Implementation Plan

### Phase 1: BCIB Worker Infrastructure (Kernel)
1. Create `kernel/proc/bcib_worker.c`
2. Implement `bcib_worker_create()` with role assignment
3. Add worker initialization to kernel boot (validation profile only)
4. Add worker PID query function

### Phase 2: BCIB Worker Payload (Userspace)
1. Create `userspace/minimal/minimal_bcib_worker.S`
2. Implement submit_execution call with minimal BCIB graph
3. Add marker emission (BW_START, BW_SUBMIT_OK, BW_SUBMIT_FAIL, BW_EXIT)
4. Test with enforcement matrix (should NOT trigger kill)

### Phase 3: USER Worker Delivery Validation
1. Modify `userspace/minimal/minimal_execution_delivery_proof.S`
2. Update markers (UW_* prefix)
3. Ensure inbox polling and completion logic correct
4. Test delivery detection and completion

### Phase 4: Build System Integration
1. Add `execution-pipeline-proof` mode to Makefile
2. Add mode mapping in userspace/minimal/Makefile
3. Configure dual-payload build
4. Test build produces correct EFI.img

### Phase 5: QEMU Harness and Validation
1. Create `scripts/qemu-execution-pipeline-proof-harness.sh`
2. Implement marker sequence validation
3. Run harness and capture trace
4. Validate marker flow and absence of violations

## Validation Criteria

### Positive Guarantees
- ✅ BCIB worker can call `SYS_V2_SUBMIT_EXECUTION` without kill
- ✅ USER worker receives inbox delivery
- ✅ USER worker can call `SYS_V2_COMPLETE_EXECUTION` without kill
- ✅ Marker sequence matches expected flow
- ✅ Execution_id propagates correctly (submit → inbox → complete)

### Negative Guarantees
- ✅ No `[[AYKEN_BOUNDARY_KILL]]` markers
- ✅ No enforcement violations
- ✅ No role escalation (USER cannot submit, BCIB cannot complete)
- ✅ No cross-role contamination

### Determinism Validation
- ✅ 5 consecutive runs produce identical marker sequences
- ✅ Execution_id values deterministic
- ✅ Inbox delivery timing bounded and consistent

## Security Considerations

### Production Safety
- BCIB worker creation ONLY in validation profile
- Worker creation gated by `#ifdef AYKEN_VALIDATION`
- No runtime role assignment or escalation
- No USER → BCIB transition path

### Test Isolation
- BCIB worker is purpose-built for validation
- Worker has no production functionality
- Worker cannot be triggered by USER processes
- Worker lifecycle controlled by kernel only

### Audit Trail
- Worker creation logged with kernel marker
- Role assignment auditable in process table
- Submit/complete operations logged with execution_id
- Enforcement violations logged with process_id and role

## Alternative Approaches (Rejected)

### Alternative 1: Allow USER to Submit (Rejected)
**Why Rejected**: Violates architectural separation, weakens enforcement matrix, creates production security risk

### Alternative 2: Runtime Role Assignment (Rejected)
**Why Rejected**: Introduces role escalation path, non-deterministic, difficult to audit

### Alternative 3: Test-Only Enforcement Bypass (Rejected)
**Why Rejected**: Weakens fail-closed guarantees, creates maintenance burden, risks production leakage

## Next Steps

1. Implement Phase 1 (BCIB worker infrastructure)
2. Test worker creation and role assignment
3. Implement Phase 2 (BCIB worker payload)
4. Test submit_execution without enforcement violation
5. Continue through Phase 5 with incremental validation

## References

- Task 5 requirements: `.kiro/specs/phase16-bcib-abdf-isolation-contracts/tasks.md`
- Enforcement matrix: `kernel/sys/syscall_enforcement_matrix.c`
- Process structure: `kernel/include/proc.h`
- Execution inbox ABI: `shared/abi/execution_inbox_abi.h`

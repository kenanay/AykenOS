# AykenOS Runtime State Machine

**Version:** 1.0  
**Authority:** Architecture Board  
**Status:** NORMATIVE  
**Date:** 2026-03-06  
**Prerequisite:** ABDF_BCIB_PHASE11_CONTRACT_MATRIX.md

## Purpose

This document defines the **formal state machine** for AykenOS runtime execution flow:

```
BCIB Instruction → Syscall → Kernel Event → Phase-11 Entry
```

This is the **canonical execution path** from userspace intent to kernel proof.

---

## 1. State Machine Overview

```
┌─────────────────────────────────────────────────────────────┐
│ USERSPACE (Ring3)                                           │
│                                                             │
│  BCIB Instruction                                           │
│      ↓                                                      │
│  BCIB Runtime Decode                                        │
│      ↓                                                      │
│  ABDF Object Resolution (if needed)                         │
│      ↓                                                      │
│  Syscall Preparation                                        │
└─────────────────────────────────────────────────────────────┘
                         │
                         │ syscall (1000-1011)
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ KERNEL (Ring0)                                              │
│                                                             │
│  Syscall Entry                                              │
│      ↓                                                      │
│  Capability Check                                           │
│      ↓                                                      │
│  Mechanism Execution                                        │
│      ↓                                                      │
│  Syscall Exit                                               │
└─────────────────────────────────────────────────────────────┘
                         │
                         │ kernel event
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ PHASE-11 (Verification Substrate)                           │
│                                                             │
│  Event Ordering                                             │
│      ↓                                                      │
│  Ledger Entry Creation                                      │
│      ↓                                                      │
│  Transcript Entry Creation                                  │
│      ↓                                                      │
│  Hash Chain Update                                          │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. BCIB Instruction → Syscall Mapping

### 2.1 DataCreate

**BCIB Instruction:**
```
opcode: DataCreate
flags: 0
args: [obj_type, size]
```

**Syscall Sequence:**
```
1. sys_v2_map_memory(size, PROT_RW)
   → returns: memory_addr

2. sys_v2_bind_capability(memory_addr, CAP_DATA_WRITE)
   → returns: cap_id
```

**Kernel Events:**
```
EVT_SYSCALL_ENTER (sys_v2_map_memory)
EVT_SYSCALL_EXIT  (sys_v2_map_memory)
EVT_SYSCALL_ENTER (sys_v2_bind_capability)
EVT_SYSCALL_EXIT  (sys_v2_bind_capability)
```

**Phase-11 Entries:**
```
Ledger:
  - event_seq: N
  - event_type: EVT_SYSCALL_ENTER
  - decision_cap: CAP_MEMORY_MAP
  - reason_code: REASON_DATA_CREATE

Transcript:
  - event_seq: N
  - syscall_no: 1000 (map_memory)
  - arg0: size
  - result0: memory_addr
```

---

### 2.2 DataQuery

**BCIB Instruction:**
```
opcode: DataQuery
flags: 0
args: [obj_id, filter_idx]
```

**Syscall Sequence:**
```
1. sys_v2_submit_execution(obj_id, filter_idx)
   → returns: exec_id

2. sys_v2_wait_result(exec_id)
   → returns: result_addr
```

**Kernel Events:**
```
EVT_SYSCALL_ENTER (sys_v2_submit_execution)
EVT_CTX_SWITCH    (scheduler decision)
EVT_SYSCALL_EXIT  (sys_v2_submit_execution)
EVT_SYSCALL_ENTER (sys_v2_wait_result)
EVT_CTX_BLOCK     (wait for result)
EVT_CTX_WAKE      (result ready)
EVT_SYSCALL_EXIT  (sys_v2_wait_result)
```

**Phase-11 Entries:**
```
Ledger:
  - event_seq: N
  - event_type: EVT_CTX_SWITCH
  - prev_ctx: ctx_A
  - next_ctx: ctx_B
  - decision_cap: CAP_SCHED_SWITCH
  - reason_code: REASON_SUBMIT_EXECUTION

Transcript:
  - event_seq: N
  - ctx_id: ctx_A
  - rip: syscall_entry_point
  - syscall_no: 1003 (submit_execution)
  - arg0: obj_id
  - arg1: filter_idx
  - result0: exec_id
```

---

### 2.3 UiRender

**BCIB Instruction:**
```
opcode: UiRender
flags: 0
args: [scene_id, target_buffer]
```

**Syscall Sequence:**
```
1. sys_v2_submit_execution(scene_id, RENDER_OP)
   → returns: exec_id

2. sys_v2_map_memory(target_buffer, PROT_RW)
   → returns: mapped_addr
```

**Kernel Events:**
```
EVT_SYSCALL_ENTER (sys_v2_submit_execution)
EVT_MAILBOX_ACCEPT (scheduler accepts render request)
EVT_CTX_SWITCH    (switch to render context)
EVT_SYSCALL_EXIT  (sys_v2_submit_execution)
```

**Phase-11 Entries:**
```
Ledger:
  - event_seq: N
  - event_type: EVT_MAILBOX_ACCEPT
  - decision_cap: CAP_MAILBOX_RENDER
  - reason_code: REASON_UI_RENDER_REQUEST

Transcript:
  - event_seq: N
  - ctx_id: render_ctx
  - syscall_no: 1003 (submit_execution)
  - arg0: scene_id
  - arg1: RENDER_OP
  - result0: exec_id
```

---

### 2.4 AiAsk

**BCIB Instruction:**
```
opcode: AiAsk
flags: 0
args: [model_id, input_tensor_id]
```

**Syscall Sequence:**
```
1. sys_v2_submit_execution(model_id, input_tensor_id)
   → returns: exec_id

2. sys_v2_wait_result(exec_id)
   → returns: output_tensor_addr
```

**Kernel Events:**
```
EVT_SYSCALL_ENTER (sys_v2_submit_execution)
EVT_POLICY_SWAP   (AI scheduler policy swap)
EVT_CTX_SWITCH    (switch to AI runtime context)
EVT_SYSCALL_EXIT  (sys_v2_submit_execution)
EVT_SYSCALL_ENTER (sys_v2_wait_result)
EVT_CTX_BLOCK     (wait for inference)
EVT_CTX_WAKE      (inference complete)
EVT_SYSCALL_EXIT  (sys_v2_wait_result)
```

**Phase-11 Entries:**
```
Ledger:
  - event_seq: N
  - event_type: EVT_POLICY_SWAP
  - decision_cap: CAP_POLICY_AI_SCHED
  - reason_code: REASON_AI_INFERENCE_REQUEST

Transcript:
  - event_seq: N
  - ctx_id: ai_runtime_ctx
  - syscall_no: 1003 (submit_execution)
  - arg0: model_id
  - arg1: input_tensor_id
  - result0: exec_id
```

---

## 3. Syscall → Kernel Event Mapping

| Syscall | Kernel Events | Phase-11 Event Types |
|---------|--------------|---------------------|
| `sys_v2_map_memory` | entry, mechanism, exit | `EVT_SYSCALL_ENTER`, `EVT_SYSCALL_EXIT` |
| `sys_v2_unmap_memory` | entry, mechanism, exit | `EVT_SYSCALL_ENTER`, `EVT_SYSCALL_EXIT` |
| `sys_v2_switch_context` | entry, switch, exit | `EVT_SYSCALL_ENTER`, `EVT_CTX_SWITCH`, `EVT_SYSCALL_EXIT` |
| `sys_v2_submit_execution` | entry, mailbox, switch, exit | `EVT_SYSCALL_ENTER`, `EVT_MAILBOX_ACCEPT/REJECT`, `EVT_CTX_SWITCH`, `EVT_SYSCALL_EXIT` |
| `sys_v2_wait_result` | entry, block, wake, exit | `EVT_SYSCALL_ENTER`, `EVT_CTX_BLOCK`, `EVT_CTX_WAKE`, `EVT_SYSCALL_EXIT` |
| `sys_v2_interrupt_return` | entry, mechanism, exit | `EVT_SYSCALL_ENTER`, `EVT_IRQ_EXIT`, `EVT_SYSCALL_EXIT` |
| `sys_v2_time_query` | entry, mechanism, exit | `EVT_SYSCALL_ENTER`, `EVT_SYSCALL_EXIT` |
| `sys_v2_bind_capability` | entry, mechanism, exit | `EVT_SYSCALL_ENTER`, `EVT_SYSCALL_EXIT` |
| `sys_v2_revoke_capability` | entry, mechanism, exit | `EVT_SYSCALL_ENTER`, `EVT_SYSCALL_EXIT` |
| `sys_v2_exit` | entry, exit | `EVT_SYSCALL_ENTER`, `EVT_CTX_EXIT` |
| `sys_v2_debug` | entry, mechanism, exit | `EVT_SYSCALL_ENTER`, `EVT_SYSCALL_EXIT` |

---

## 4. Kernel Event → Phase-11 Entry Mapping

### 4.1 Context Switch Event

**Kernel Event:**
```c
ay_event_type_t event = EVT_CTX_SWITCH;
ay_ctx_id_t prev_ctx = current_ctx;
ay_ctx_id_t next_ctx = target_ctx;
ay_cap_id_t decision_cap = scheduler_cap;
uint64_t reason_code = REASON_MAILBOX_ACCEPT;
```

**Phase-11 Ledger Entry:**
```c
ay_decision_ledger_entry_t entry = {
    .magic = AYKEN_LEDGER_MAGIC,
    .version = 1,
    .flags = 0,
    .event_seq = global_event_seq++,
    .ltick = global_ltick,
    .cpu_id = current_cpu,
    .event_type = EVT_CTX_SWITCH,
    .prev_ctx = prev_ctx,
    .next_ctx = next_ctx,
    .decision_cap = decision_cap,
    .reason_code = reason_code,
    .payload_hash = H(normalized_payload),
    .prev_hash = ledger_tip_hash,
    .entry_hash = H(header || normalized_payload)
};
```

**Phase-11 Transcript Entry:**
```c
ay_transcript_entry_t entry = {
    .magic = AYKEN_TRANSCRIPT_MAGIC,
    .version = 1,
    .flags = 0,
    .event_seq = global_event_seq,
    .ltick = global_ltick,
    .cpu_id = current_cpu,
    .event_type = EVT_CTX_SWITCH,
    .ctx_id = next_ctx,
    .rip = next_ctx->rip,
    .rsp = next_ctx->rsp,
    .cr3 = next_ctx->cr3,
    .state_hash_before = H(prev_ctx_state),
    .state_hash_after = H(next_ctx_state)
};
```

---

### 4.2 Syscall Entry Event

**Kernel Event:**
```c
ay_event_type_t event = EVT_SYSCALL_ENTER;
uint64_t syscall_no = rax;
uint64_t arg0 = rdi;
uint64_t arg1 = rsi;
uint64_t arg2 = rdx;
```

**Phase-11 Transcript Entry:**
```c
ay_transcript_entry_t entry = {
    .magic = AYKEN_TRANSCRIPT_MAGIC,
    .version = 1,
    .flags = 0,
    .event_seq = global_event_seq++,
    .ltick = global_ltick,
    .cpu_id = current_cpu,
    .event_type = EVT_SYSCALL_ENTER,
    .ctx_id = current_ctx,
    .rip = saved_rip,
    .rsp = saved_rsp,
    .cr3 = current_cr3,
    .syscall_no = syscall_no,
    .arg0 = arg0,
    .arg1 = arg1,
    .arg2 = arg2,
    .state_hash_before = H(kernel_state)
};
```

---

### 4.3 Mailbox Accept Event

**Kernel Event:**
```c
ay_event_type_t event = EVT_MAILBOX_ACCEPT;
ay_cap_id_t mailbox_cap = proposal_cap;
uint64_t reason_code = REASON_SCHEDULER_PROPOSAL;
```

**Phase-11 Ledger Entry:**
```c
ay_decision_ledger_entry_t entry = {
    .magic = AYKEN_LEDGER_MAGIC,
    .version = 1,
    .flags = 0,
    .event_seq = global_event_seq++,
    .ltick = global_ltick,
    .cpu_id = current_cpu,
    .event_type = EVT_MAILBOX_ACCEPT,
    .prev_ctx = current_ctx,
    .next_ctx = proposed_ctx,
    .decision_cap = mailbox_cap,
    .reason_code = reason_code,
    .payload_hash = H(mailbox_proposal),
    .prev_hash = ledger_tip_hash,
    .entry_hash = H(header || mailbox_proposal)
};
```

---

## 5. State Transition Rules

### 5.1 Normal Execution Flow

```
State: USERSPACE_RUNNING
    ↓ (BCIB instruction decoded)
State: SYSCALL_PREPARE
    ↓ (syscall invoked)
State: KERNEL_ENTRY
    ↓ (capability check)
State: KERNEL_EXECUTING
    ↓ (mechanism complete)
State: KERNEL_EXIT
    ↓ (return to userspace)
State: USERSPACE_RUNNING
```

**Phase-11 Recording:**
- `SYSCALL_PREPARE` → no Phase-11 entry
- `KERNEL_ENTRY` → `EVT_SYSCALL_ENTER` transcript
- `KERNEL_EXECUTING` → mechanism-specific events
- `KERNEL_EXIT` → `EVT_SYSCALL_EXIT` transcript

---

### 5.2 Context Switch Flow

```
State: USERSPACE_RUNNING (ctx_A)
    ↓ (submit_execution syscall)
State: KERNEL_ENTRY
    ↓ (mailbox proposal)
State: MAILBOX_DECISION
    ↓ (accept)
State: CONTEXT_SWITCH
    ↓ (switch to ctx_B)
State: USERSPACE_RUNNING (ctx_B)
```

**Phase-11 Recording:**
- `KERNEL_ENTRY` → `EVT_SYSCALL_ENTER` transcript
- `MAILBOX_DECISION` → `EVT_MAILBOX_ACCEPT` ledger
- `CONTEXT_SWITCH` → `EVT_CTX_SWITCH` ledger + transcript
- `USERSPACE_RUNNING` → no Phase-11 entry

---

### 5.3 Interrupt Flow

```
State: USERSPACE_RUNNING
    ↓ (timer interrupt)
State: INTERRUPT_ENTRY
    ↓ (save context)
State: INTERRUPT_HANDLER
    ↓ (handle interrupt)
State: INTERRUPT_EXIT
    ↓ (restore context)
State: USERSPACE_RUNNING
```

**Phase-11 Recording:**
- `INTERRUPT_ENTRY` → `EVT_IRQ_ENTER` transcript
- `INTERRUPT_HANDLER` → mechanism-specific events
- `INTERRUPT_EXIT` → `EVT_IRQ_EXIT` transcript

---

## 6. Error Handling State Machine

### 6.1 Capability Violation

```
State: KERNEL_ENTRY
    ↓ (capability check fails)
State: CAPABILITY_VIOLATION
    ↓ (reject syscall)
State: KERNEL_EXIT (error)
    ↓ (return -EPERM)
State: USERSPACE_RUNNING
```

**Phase-11 Recording:**
```
Ledger:
  - event_type: EVT_MAILBOX_REJECT
  - reason_code: REASON_CAP_VIOLATION
  - result: -EPERM

Transcript:
  - event_type: EVT_SYSCALL_EXIT
  - result0: -EPERM
```

---

### 6.2 Ordering Violation

```
State: PHASE11_ORDERING
    ↓ (event_seq not monotonic)
State: ORDERING_VIOLATION
    ↓ (kernel panic)
State: SYSTEM_HALT
```

**Phase-11 Recording:**
```
Ledger:
  - event_type: EVT_LEDGER_SEAL
  - reason_code: REASON_ORDERING_VIOLATION
  - flags: FLAG_PANIC

Transcript:
  - event_type: EVT_TRAP_ENTER
  - trap_no: TRAP_ORDERING_VIOLATION
```

**Enforcement:** FAIL-CLOSED (kernel panic)

---

## 7. Multicore State Coordination

### 7.1 DLT (Deterministic Logical Time)

**Purpose:** Assign global logical time to local CPU events

```
CPU0: local_event_A
    ↓
DLT: assign ltick = 100
    ↓
Phase-11: record event_seq=N, ltick=100

CPU1: local_event_B
    ↓
DLT: assign ltick = 101
    ↓
Phase-11: record event_seq=N+1, ltick=101
```

**Invariant:** `ltick` MUST be globally monotonic

---

### 7.2 GCP (Global Commit Protocol)

**Purpose:** Deterministic finalization across all CPUs

```
State: PREPARE
    ↓ (all CPUs ready)
State: COMMIT_VOTE
    ↓ (unanimous yes)
State: COMMIT
    ↓ (finalize state)
State: SEALED
```

**Phase-11 Recording:**
```
Ledger:
  - event_type: EVT_GCP_PREPARE
  - event_type: EVT_GCP_COMMIT

GCP Record:
  - commit_id: unique_id
  - commit_ltick: final_ltick
  - participant_count: num_cpus
  - state: COMMITTED
  - transcript_root_hash: H(all_transcripts)
  - ledger_root_hash: H(all_ledgers)
```

---

## 8. Replay State Machine

### 8.1 Replay Initialization

```
State: REPLAY_INIT
    ↓ (load ABDF snapshot)
State: SNAPSHOT_LOADED
    ↓ (load BCIB plan)
State: PLAN_LOADED
    ↓ (load Phase-11 transcript)
State: TRANSCRIPT_LOADED
    ↓ (verify hashes)
State: REPLAY_READY
```

---

### 8.2 Replay Execution

```
State: REPLAY_READY
    ↓ (execute BCIB instruction)
State: REPLAY_EXECUTING
    ↓ (compare with transcript)
State: REPLAY_VERIFY
    ↓ (match: continue, mismatch: fail)
State: REPLAY_NEXT or REPLAY_FAIL
```

**Verification:**
```c
if (actual_event_seq != expected_event_seq) {
    replay_state.mismatch_count++;
    if (replay_state.strict_mode) {
        kernel_panic("Replay mismatch");
    }
}

if (actual_state_hash != expected_state_hash) {
    replay_state.mismatch_count++;
    if (replay_state.strict_mode) {
        kernel_panic("State hash mismatch");
    }
}
```

---

## 9. Implementation Checklist

### Kernel Side

- [ ] Implement `ay_phase11_record_event(event_type, ...)`
- [ ] Hook syscall entry/exit to Phase-11
- [ ] Hook context switch to Phase-11
- [ ] Hook interrupt entry/exit to Phase-11
- [ ] Hook mailbox accept/reject to Phase-11
- [ ] Implement ordering layer (event_seq, ltick)
- [ ] Implement ledger append
- [ ] Implement transcript append
- [ ] Implement hash chain update

### Userspace Side

- [ ] Implement BCIB → syscall mapping
- [ ] Implement execution trace recording
- [ ] Implement replay engine
- [ ] Implement snapshot capture
- [ ] Implement verification logic

### CI Side

- [ ] Implement `ci-gate-ledger-completeness`
- [ ] Implement `ci-gate-transcript-integrity`
- [ ] Implement `ci-gate-replay-determinism`
- [ ] Implement `ci-gate-hash-chain-validity`

---

## 10. Critical Invariants

1. **Event Sequence Monotonicity**
   - `event_seq` MUST be globally monotonic
   - Violation → kernel panic

2. **Hash Chain Integrity**
   - `entry_hash = H(prev_hash || payload)`
   - Violation → replay fail

3. **Transcript Completeness**
   - EVERY significant kernel event MUST produce transcript entry
   - Violation → CI fail

4. **Syscall Boundary**
   - BCIB → Kernel ONLY via syscall (1000-1011)
   - Violation → PR auto-reject

5. **Deterministic Replay**
   - Same transcript → same final state hash
   - Violation → replay fail

---

## 11. References

- `ABDF_BCIB_PHASE11_CONTRACT_MATRIX.md` - Layer contracts
- `shared/abi/ayken_abi.h` and `shared/abi/syscall_v2.h` - canonical syscall ABI
- `kernel/sys/syscall_v2.c` - Syscall implementation
- `ayken-core/crates/bcib/` - BCIB implementation

---

**Maintained by:** AykenOS Architecture Board  
**Last Updated:** 2026-03-06  
**Next Review:** Before Phase-11 implementation

**This document is binding for Phase-11 implementation.**

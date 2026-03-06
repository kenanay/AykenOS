# Phase-11 Event Taxonomy

**Version:** 1.0  
**Authority:** Architecture Board  
**Status:** NORMATIVE  
**Date:** 2026-03-06  
**Prerequisite:** RUNTIME_STATE_MACHINE.md

## Purpose

This document defines the **complete taxonomy** of kernel events that MUST be recorded by Phase-11.

Every significant kernel state transition MUST produce Phase-11 record(s) according to event class:
1. **Ledger entry** (decision record) for decision-class events
2. **Transcript entry** (execution reality) for execution-class events
3. **Both** for dual-class events (where explicitly required)

This is the **canonical event specification** for Phase-11 implementation.

---

## 1. Event Type Enumeration

```c
typedef enum {
    AY_EVT_NONE = 0,
    
    /* Scheduler / Execution (1-9) */
    AY_EVT_CTX_SWITCH        = 1,
    AY_EVT_CTX_BLOCK         = 2,
    AY_EVT_CTX_WAKE          = 3,
    AY_EVT_CTX_EXIT          = 4,
    AY_EVT_CTX_CREATE        = 5,
    
    /* Syscall / Interrupt / Trap (10-19) */
    AY_EVT_SYSCALL_ENTER     = 10,
    AY_EVT_SYSCALL_EXIT      = 11,
    AY_EVT_IRQ_ENTER         = 12,
    AY_EVT_IRQ_EXIT          = 13,
    AY_EVT_TRAP_ENTER        = 14,
    AY_EVT_TRAP_EXIT         = 15,
    
    /* Mailbox / Policy Bridge (20-29) */
    AY_EVT_MAILBOX_ACCEPT    = 20,
    AY_EVT_MAILBOX_REJECT    = 21,
    AY_EVT_POLICY_SWAP       = 22,
    AY_EVT_CAPABILITY_BIND   = 23,
    AY_EVT_CAPABILITY_REVOKE = 24,
    
    /* Proof / Commit (30-39) */
    AY_EVT_LEDGER_SEAL       = 30,
    AY_EVT_GCP_PREPARE       = 31,
    AY_EVT_GCP_COMMIT        = 32,
    AY_EVT_GCP_ABORT         = 33,
    
    /* Memory / Resource (40-49) */
    AY_EVT_MEMORY_MAP        = 40,
    AY_EVT_MEMORY_UNMAP      = 41,
    AY_EVT_MEMORY_PROTECT    = 42,
    
    /* Error / Violation (50-59) */
    AY_EVT_CAPABILITY_VIOLATION = 50,
    AY_EVT_ORDERING_VIOLATION   = 51,
    AY_EVT_REPLAY_MISMATCH      = 52,

    /* Bounds */
    AY_EVT_MAX                  = 53
} ay_event_type_t;
```

---

## 2. Event Recording Rules

### 2.1 MUST Record (Mandatory)

These events MUST ALWAYS produce the required Phase-11 record(s) shown below:

| Event | Ledger | Transcript | Reason |
|-------|--------|-----------|--------|
| `CTX_SWITCH` | YES | YES | Scheduler decision + state transition |
| `SYSCALL_ENTER` | NO | YES | Execution reality |
| `SYSCALL_EXIT` | NO | YES | Execution reality |
| `MAILBOX_ACCEPT` | YES | NO | Policy decision |
| `MAILBOX_REJECT` | YES | NO | Policy decision |
| `IRQ_ENTER` | NO | YES | Execution reality |
| `IRQ_EXIT` | NO | YES | Execution reality |
| `GCP_COMMIT` | YES | NO | Multicore finalization |

### 2.2 SHOULD Record (Recommended)

These events SHOULD produce entries for audit/debug:

| Event | Ledger | Transcript | Reason |
|-------|--------|-----------|--------|
| `CTX_BLOCK` | YES | YES | Scheduler decision |
| `CTX_WAKE` | YES | YES | Scheduler decision |
| `POLICY_SWAP` | YES | NO | Policy change |
| `CAPABILITY_BIND` | YES | NO | Security decision |

### 2.3 MAY Record (Optional)

These events MAY be recorded based on configuration:

| Event | Ledger | Transcript | Reason |
|-------|--------|-----------|--------|
| `MEMORY_MAP` | NO | YES | Resource allocation |
| `MEMORY_UNMAP` | NO | YES | Resource deallocation |
| `TRAP_ENTER` | NO | YES | Exception handling |

### 2.4 MUST NOT Record

These events MUST NOT produce Phase-11 entries:

- Userspace-only events (no kernel involvement)
- Kernel internal bookkeeping (no observable state change)
- High-frequency timer ticks (unless causing context switch)

---

## 3. Event Specification by Category

### 3.1 Context Switch Event

**Event Type:** `AY_EVT_CTX_SWITCH`

**When:** Kernel switches from one execution context to another

**Ledger Entry:**
```c
ay_decision_ledger_entry_t {
    .event_type = AY_EVT_CTX_SWITCH,
    .prev_ctx = current_ctx_id,
    .next_ctx = target_ctx_id,
    .decision_cap = scheduler_cap_id,
    .reason_code = REASON_MAILBOX_ACCEPT | REASON_PREEMPT | REASON_YIELD,
    .payload_hash = H(normalized_payload),
    .prev_hash = ledger_tip_hash,
    .entry_hash = H(prev_hash || payload_hash)
}
```

**Transcript Entry:**
```c
ay_transcript_entry_t {
    .event_type = AY_EVT_CTX_SWITCH,
    .ctx_id = next_ctx_id,
    .rip = next_ctx->rip,
    .rsp = next_ctx->rsp,
    .cr3 = next_ctx->cr3,
    .state_hash_before = H(prev_ctx_state),
    .state_hash_after = H(next_ctx_state)
}
```

**Reason Codes:**
- `REASON_MAILBOX_ACCEPT` (0x01): Scheduler accepted mailbox proposal
- `REASON_PREEMPT` (0x02): Timer preemption
- `REASON_YIELD` (0x03): Voluntary yield
- `REASON_BLOCK` (0x04): Context blocked on wait

---

### 3.2 Syscall Entry Event

**Event Type:** `AY_EVT_SYSCALL_ENTER`

**When:** Userspace invokes syscall (1000-1010)

**Ledger Entry:** NONE (syscall entry is not a decision)

**Transcript Entry:**
```c
ay_transcript_entry_t {
    .event_type = AY_EVT_SYSCALL_ENTER,
    .ctx_id = current_ctx_id,
    .rip = saved_rip,
    .rsp = saved_rsp,
    .cr3 = current_cr3,
    .syscall_no = rax,
    .arg0 = rdi,
    .arg1 = rsi,
    .arg2 = rdx,
    .state_hash_before = H(kernel_state)
}
```

---

### 3.3 Syscall Exit Event

**Event Type:** `AY_EVT_SYSCALL_EXIT`

**When:** Kernel returns from syscall to userspace

**Ledger Entry:** NONE (syscall exit is not a decision)

**Transcript Entry:**
```c
ay_transcript_entry_t {
    .event_type = AY_EVT_SYSCALL_EXIT,
    .ctx_id = current_ctx_id,
    .rip = return_rip,
    .rsp = return_rsp,
    .cr3 = current_cr3,
    .syscall_no = original_syscall_no,
    .result0 = rax,
    .state_hash_after = H(kernel_state)
}
```

---

### 3.4 Mailbox Accept Event

**Event Type:** `AY_EVT_MAILBOX_ACCEPT`

**When:** Kernel accepts scheduler mailbox proposal

**Ledger Entry:**
```c
ay_decision_ledger_entry_t {
    .event_type = AY_EVT_MAILBOX_ACCEPT,
    .prev_ctx = current_ctx_id,
    .next_ctx = proposed_ctx_id,
    .decision_cap = mailbox_cap_id,
    .reason_code = REASON_SCHEDULER_PROPOSAL,
    .payload_hash = H(normalized_payload),
    .prev_hash = ledger_tip_hash,
    .entry_hash = H(prev_hash || payload_hash)
}
```

**Transcript Entry:** NONE (decision only, no execution state change yet)

**Reason Codes:**
- `REASON_SCHEDULER_PROPOSAL` (0x10): Userspace scheduler proposal
- `REASON_AI_SCHEDULER` (0x11): AI scheduler decision
- `REASON_FALLBACK_SCHEDULER` (0x12): Kernel fallback scheduler

---

### 3.5 Mailbox Reject Event

**Event Type:** `AY_EVT_MAILBOX_REJECT`

**When:** Kernel rejects scheduler mailbox proposal

**Ledger Entry:**
```c
ay_decision_ledger_entry_t {
    .event_type = AY_EVT_MAILBOX_REJECT,
    .prev_ctx = current_ctx_id,
    .next_ctx = 0,  // no switch
    .decision_cap = mailbox_cap_id,
    .reason_code = REASON_CAP_VIOLATION | REASON_INVALID_PROPOSAL,
    .payload_hash = H(normalized_payload),
    .prev_hash = ledger_tip_hash,
    .entry_hash = H(prev_hash || payload_hash)
}
```

**Transcript Entry:** NONE (decision only)

**Reason Codes:**
- `REASON_CAP_VIOLATION` (0x20): Capability check failed
- `REASON_INVALID_PROPOSAL` (0x21): Malformed proposal
- `REASON_INVALID_CTX` (0x22): Target context invalid

**Reject Aliases (P11-01 Mailbox Capability Contract):**
- `REJ_BAD_SIG`: signature/envelope validation failed
- `REJ_CAP_MISSING`: capability proof missing
- `REJ_BUDGET_EXCEEDED`: budget envelope missing/invalid/exceeded
- `REJ_INVALID_PID`: target pid invalid

---

### 3.6 Interrupt Entry Event

**Event Type:** `AY_EVT_IRQ_ENTER`

**When:** Hardware interrupt fires

**Ledger Entry:** NONE (interrupt is not a decision)

**Transcript Entry:**
```c
ay_transcript_entry_t {
    .event_type = AY_EVT_IRQ_ENTER,
    .ctx_id = interrupted_ctx_id,
    .rip = interrupted_rip,
    .rsp = interrupted_rsp,
    .cr3 = current_cr3,
    .irq_vec = interrupt_vector,
    .state_hash_before = H(kernel_state)
}
```

---

### 3.7 Interrupt Exit Event

**Event Type:** `AY_EVT_IRQ_EXIT`

**When:** Kernel returns from interrupt handler

**Ledger Entry:** NONE (interrupt exit is not a decision)

**Transcript Entry:**
```c
ay_transcript_entry_t {
    .event_type = AY_EVT_IRQ_EXIT,
    .ctx_id = resumed_ctx_id,
    .rip = resume_rip,
    .rsp = resume_rsp,
    .cr3 = current_cr3,
    .irq_vec = interrupt_vector,
    .state_hash_after = H(kernel_state)
}
```

---

### 3.8 Policy Swap Event

**Event Type:** `AY_EVT_POLICY_SWAP`

**When:** Kernel switches active policy module (e.g., AI scheduler)

**Ledger Entry:**
```c
ay_decision_ledger_entry_t {
    .event_type = AY_EVT_POLICY_SWAP,
    .prev_ctx = 0,  // not context-specific
    .next_ctx = 0,
    .decision_cap = policy_swap_cap_id,
    .reason_code = REASON_POLICY_CHANGE,
    .aux0 = old_policy_id,
    .aux1 = new_policy_id,
    .payload_hash = H(normalized_payload),
    .prev_hash = ledger_tip_hash,
    .entry_hash = H(prev_hash || payload_hash)
}
```

**Transcript Entry:** NONE (policy change is decision, not execution)

---

### 3.9 GCP Commit Event

**Event Type:** `AY_EVT_GCP_COMMIT`

**When:** Global Commit Protocol finalizes multicore state

**Ledger Entry:**
```c
ay_decision_ledger_entry_t {
    .event_type = AY_EVT_GCP_COMMIT,
    .prev_ctx = 0,
    .next_ctx = 0,
    .decision_cap = gcp_coordinator_cap,
    .reason_code = REASON_MULTICORE_FINALIZE,
    .aux0 = commit_id,
    .aux1 = participant_count,
    .payload_hash = H(normalized_payload),
    .prev_hash = ledger_tip_hash,
    .entry_hash = H(prev_hash || payload_hash)
}
```

**Transcript Entry:** NONE (GCP is coordination, not execution)

---

## 4. Event Recording Hooks

### 4.1 Kernel Hook Points

| Hook Point | Event Type | Function |
|-----------|-----------|----------|
| `context_switch()` | `CTX_SWITCH` | `ay_phase11_record_ctx_switch()` |
| `syscall_entry()` | `SYSCALL_ENTER` | `ay_phase11_record_syscall_enter()` |
| `syscall_exit()` | `SYSCALL_EXIT` | `ay_phase11_record_syscall_exit()` |
| `irq_entry()` | `IRQ_ENTER` | `ay_phase11_record_irq_enter()` |
| `irq_exit()` | `IRQ_EXIT` | `ay_phase11_record_irq_exit()` |
| `mailbox_accept()` | `MAILBOX_ACCEPT` | `ay_phase11_record_mailbox_accept()` |
| `mailbox_reject()` | `MAILBOX_REJECT` | `ay_phase11_record_mailbox_reject()` |

### 4.2 Recording Function Signature

```c
void ay_phase11_record_event(
    ay_event_type_t event_type,
    ay_ctx_id_t prev_ctx,
    ay_ctx_id_t next_ctx,
    ay_cap_id_t decision_cap,
    uint64_t reason_code,
    const void *payload,
    size_t payload_len
);
```

---

## 5. Event Payload Specification

### 5.1 Context Switch Payload

```c
struct ay_ctx_switch_payload {
    uint64_t prev_rip;
    uint64_t prev_rsp;
    uint64_t prev_cr3;
    uint64_t next_rip;
    uint64_t next_rsp;
    uint64_t next_cr3;
    uint64_t switch_reason;
};
```

### 5.2 Mailbox Proposal Payload

```c
struct ay_mailbox_proposal_payload {
    ay_ctx_id_t proposed_ctx;
    uint64_t priority;
    uint64_t deadline;
    uint64_t proposal_hash;
};
```

### 5.3 GCP Record Payload

```c
struct ay_gcp_record_payload {
    uint64_t commit_id;
    ay_ltick_t commit_ltick;
    uint32_t participant_count;
    uint32_t coordinator_cpu;
    ay_hash256_t transcript_root_hash;
    ay_hash256_t ledger_root_hash;
};
```

---

## 6. Event Ordering Rules

### 6.1 Global Ordering

**Rule:** `event_seq` MUST be globally monotonic across all CPUs

**Enforcement:**
```c
static atomic_uint64_t global_event_seq = 0;

ay_event_seq_t ay_phase11_next_event_seq(void) {
    return atomic_fetch_add(&global_event_seq, 1);
}
```

### 6.2 Logical Time Ordering

**Rule:** `ltick` MUST be deterministic logical time

**Enforcement:**
```c
ay_ltick_t ay_phase11_assign_ltick(ay_event_type_t event_type) {
    // DLT assigns ltick based on event type and CPU
    return dlt_assign_logical_time(event_type, current_cpu);
}
```

### 6.3 Per-CPU Ordering

**Rule:** Events on same CPU MUST maintain local order

**Enforcement:**
```c
static __thread ay_event_seq_t last_cpu_event_seq = 0;

void ay_phase11_record_event(...) {
    ay_event_seq_t seq = ay_phase11_next_event_seq();
    
    if (seq <= last_cpu_event_seq) {
        kernel_panic("CPU event ordering violation");
    }
    
    last_cpu_event_seq = seq;
    // ... record event
}
```

---

## 7. Event Filtering Rules

### 7.1 High-Frequency Event Filtering

**Problem:** Timer ticks fire at 100 Hz, producing excessive events

**Solution:** Record only ticks that cause observable state change

```c
void timer_tick_handler(void) {
    bool caused_switch = false;
    
    // Handle timer tick
    if (should_preempt()) {
        context_switch(next_ctx);
        caused_switch = true;
    }
    
    // Only record if state changed
    if (caused_switch) {
        ay_phase11_record_ctx_switch(...);
    }
}
```

### 7.2 Syscall Filtering

**Rule:** Record ALL syscalls (no filtering)

**Rationale:** Syscalls are observable userspace→kernel transitions

---

## 8. Event Validation Rules

### 8.1 Ledger Entry Validation

```c
bool ay_phase11_validate_ledger_entry(
    const ay_decision_ledger_entry_t *entry
) {
    // Magic check
    if (entry->magic != AYKEN_LEDGER_MAGIC) return false;
    
    // Version check
    if (entry->version != 1) return false;
    
    // Event type check
    if (entry->event_type >= AY_EVT_MAX) return false;
    
    // Hash chain check
    ay_hash256_t computed_hash = H(entry->prev_hash || entry->payload_hash);
    if (memcmp(&computed_hash, &entry->entry_hash, 32) != 0) return false;
    
    return true;
}
```

### 8.2 Transcript Entry Validation

```c
bool ay_phase11_validate_transcript_entry(
    const ay_transcript_entry_t *entry
) {
    // Magic check
    if (entry->magic != AYKEN_TRANSCRIPT_MAGIC) return false;
    
    // Version check
    if (entry->version != 1) return false;
    
    // Event type check
    if (entry->event_type >= AY_EVT_MAX) return false;
    
    // State hash check (if replay mode)
    if (replay_mode) {
        if (memcmp(&entry->state_hash_after, &expected_hash, 32) != 0) {
            return false;
        }
    }
    
    return true;
}
```

---

## 9. Event Serialization Format

### 9.1 Binary Format (ledger.bin)

```
[Header: 64 bytes]
  magic: 4 bytes ("LDG1")
  version: 2 bytes
  entry_count: 8 bytes
  total_size: 8 bytes
  reserved: 42 bytes

[Entry 0: variable size]
  ay_decision_ledger_entry_t

[Entry 1: variable size]
  ay_decision_ledger_entry_t

...
```

### 9.2 JSON Lines Format (transcript.jsonl)

```json
{"event_seq":1,"ltick":100,"event_type":"CTX_SWITCH","ctx_id":1,"rip":"0x400000"}
{"event_seq":2,"ltick":101,"event_type":"SYSCALL_ENTER","syscall_no":1000}
{"event_seq":3,"ltick":102,"event_type":"SYSCALL_EXIT","result0":0}
```

---

## 10. CI Gate Validation

### 10.1 Ledger Completeness Gate

**Gate:** `make ci-gate-ledger-completeness`

**Checks:**
- Every context switch has ledger entry
- Every mailbox decision has ledger entry
- No missing event_seq gaps
- Hash chain is valid

### 10.2 Transcript Integrity Gate

**Gate:** `make ci-gate-transcript-integrity`

**Checks:**
- Every syscall has enter + exit transcript
- Every interrupt has enter + exit transcript
- State hashes are consistent
- No missing event_seq gaps

---

## 11. Implementation Checklist

### Kernel Side

- [ ] Define `ay_event_type_t` enum
- [ ] Implement `ay_phase11_record_event()`
- [ ] Hook `context_switch()` → `record_ctx_switch()`
- [ ] Hook `syscall_entry()` → `record_syscall_enter()`
- [ ] Hook `syscall_exit()` → `record_syscall_exit()`
- [ ] Hook `irq_entry()` → `record_irq_enter()`
- [ ] Hook `irq_exit()` → `record_irq_exit()`
- [ ] Hook `mailbox_accept()` → `record_mailbox_accept()`
- [ ] Hook `mailbox_reject()` → `record_mailbox_reject()`
- [ ] Implement event ordering (event_seq, ltick)
- [ ] Implement ledger append
- [ ] Implement transcript append
- [ ] Implement hash chain update
- [ ] Implement event validation

### CI Side

- [ ] Implement `ci-gate-ledger-completeness`
- [ ] Implement `ci-gate-transcript-integrity`
- [ ] Implement event sequence gap detection
- [ ] Implement hash chain validation

---

## 12. Critical Invariants

1. **Event Sequence Monotonicity**
   - `event_seq` MUST be globally monotonic
   - Violation → kernel panic

2. **Ledger Append-Only**
   - Ledger entries MUST NOT be modified after creation
   - Violation → CI fail

3. **Transcript Completeness**
   - Every event classified as `Transcript=YES` in Section 2 MUST produce transcript entry
   - Violation → CI fail

4. **Hash Chain Integrity**
   - `payload_hash = H(normalized_payload)`
   - `entry_hash = H(prev_hash || payload_hash)`
   - Violation → replay fail

---

## 13. References

- `RUNTIME_STATE_MACHINE.md` - State machine specification
- `ABDF_BCIB_PHASE11_CONTRACT_MATRIX.md` - Layer contracts
- `kernel/include/ayken_abi.h` - Syscall ABI

---

**Maintained by:** AykenOS Architecture Board  
**Last Updated:** 2026-03-06  
**Next Review:** Before Phase-11 implementation

**This document is binding for Phase-11 implementation.**

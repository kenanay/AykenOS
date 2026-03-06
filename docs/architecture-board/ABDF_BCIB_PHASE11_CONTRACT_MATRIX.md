# ABDF + BCIB + Phase-11 Contract Matrix

**Version:** 1.0  
**Authority:** Architecture Board  
**Status:** NORMATIVE  
**Date:** 2026-03-06

## Purpose

This document defines the **formal contracts** between AykenOS's three substrate layers:

- **ABDF**: Data substrate (what exists)
- **BCIB**: Execution substrate (what is intended)
- **Phase-11**: Verification substrate (what actually happened)

Without this contract matrix, layer boundaries blur, replay fails, and proof integrity breaks.

---

## 1. Layer Responsibilities Matrix

| Layer | Primary Responsibility | What It Knows | What It MUST NOT Know |
|-------|----------------------|---------------|----------------------|
| **ABDF** | Typed data container | segment layout, type system, meta, schema, embeddings | kernel events, execution order, syscall semantics |
| **BCIB** | Execution intent | instruction sequence, opcode semantics, data references | kernel mechanism, actual execution state, hardware |
| **Phase-11** | Kernel reality record | events, ordering, state transitions, decisions | high-level intent, data schema, policy logic |

### Enforcement

- **ABDF** MUST NOT contain kernel event types
- **BCIB** MUST NOT contain CPU state or interrupt vectors
- **Phase-11** MUST NOT contain ABDF schema or BCIB opcodes

---

## 2. Data Flow Matrix

| Source | Target | Format | Validation | Authority |
|--------|--------|--------|-----------|-----------|
| **ABDF → BCIB** | object reference | `obj_id` | type check | BCIB runtime |
| **BCIB → Kernel** | syscall | syscall ABI (1000-1010) | capability check | kernel |
| **Kernel → Phase-11** | event | `ay_event_type_t` | sequence check | ordering layer |
| **Phase-11 → Evidence** | serialized proof | JSON/binary | hash check | CI gates |
| **ABDF → Phase-11** | snapshot | ABDF buffer | schema validation | replay engine |

### Critical Rules

1. **BCIB → Kernel**: ONLY via syscall interface (no direct kernel calls)
2. **Kernel → Phase-11**: EVERY significant event MUST produce ledger/transcript entry
3. **Phase-11 → Evidence**: Evidence MUST be immutable after creation

---

## 3. Hash Production Matrix

| Layer | Hash Type | Input | Algorithm | Purpose |
|-------|-----------|-------|-----------|---------|
| **ABDF** | `content_hash` | segment data | SHA-256 | data integrity |
| **ABDF** | `schema_hash` | type + meta | SHA-256 | schema versioning |
| **BCIB** | `plan_hash` | instruction stream | SHA-256 | execution plan identity |
| **Phase-11** | `entry_hash` | ledger entry | SHA-256 | hash chain link |
| **Phase-11** | `transcript_hash` | transcript entry | SHA-256 | execution reality |
| **Phase-11** | `proof_hash` | manifest | SHA-256 | final proof seal |

### Hash Chain Rules

- **Ledger**: `entry_hash = H(prev_hash || normalized_payload)`
- **Transcript**: `transcript_hash = H(state_before || event || state_after)`
- **Proof**: `proof_hash = H(ledger_root || transcript_root || replay_result)`

---

## 4. Replay Dependency Matrix

| Replay Target | Input Required | Verification Method | Output |
|--------------|----------------|---------------------|--------|
| **ABDF** | input snapshot | schema validation | data state |
| **BCIB** | execution plan | opcode validation | execution trace |
| **Phase-11** | transcript + ledger | hash chain + ordering | proof manifest |

### Replay Invariants

1. **ABDF Replay**: Same input snapshot → same data state
2. **BCIB Replay**: Same plan + same data → same syscall sequence
3. **Phase-11 Replay**: Same transcript → same final state hash

---

## 5. Boundary Crossing Matrix

| Boundary | Allowed Operations | Forbidden Operations | Enforcement |
|----------|-------------------|---------------------|-------------|
| **BCIB → ABDF** | read segment, query meta, resolve type | modify kernel state, direct memory access | runtime validation |
| **BCIB → Kernel** | syscall (1000-1010), capability ops | direct hardware access, interrupt injection | syscall gate |
| **Kernel → Phase-11** | append ledger, append transcript | modify past entries, skip ordering | ordering layer |
| **Phase-11 → Evidence** | serialize, export | modify evidence, delete entries | CI hygiene gate |

### Critical Violations

- **BCIB calling kernel function directly** → PR AUTO-REJECT
- **Phase-11 modifying past ledger entry** → PANIC
- **Evidence directory modification** → CI FAIL

---

## 6. Type System Compatibility Matrix

| ABDF Type | BCIB Opcode | Phase-11 Event | Mapping |
|-----------|-------------|----------------|---------|
| `Tabular` | `DataQuery` | `EVT_SYSCALL_ENTER` | BCIB query → syscall → ledger entry |
| `Log` | `DataAdd` | `EVT_SYSCALL_EXIT` | BCIB append → syscall → transcript entry |
| `UiScene` | `UiRender` | `EVT_CTX_SWITCH` | BCIB render → context switch → ledger |
| `GpuBuffer` | `DataCreate` | `EVT_MAILBOX_ACCEPT` | BCIB create → mailbox → decision ledger |
| `Tensor` | `AiAsk` | `EVT_POLICY_SWAP` | BCIB AI call → policy swap → ledger |

### Type Preservation Rules

- **ABDF type** MUST be preserved across BCIB operations
- **BCIB opcode** MUST map to valid syscall sequence
- **Phase-11 event** MUST NOT leak ABDF schema details

---

## 7. Evidence Export Matrix

| Layer | Evidence Format | Location | Immutability |
|-------|----------------|----------|--------------|
| **ABDF** | `snapshot.abdf` | `evidence/run-*/input/` | YES |
| **BCIB** | `plan.bcib` | `evidence/run-*/execution/` | YES |
| **Phase-11** | `ledger.bin`, `transcript.jsonl`, `proof.json` | `evidence/run-*/` | YES |

### Evidence Integrity Rules

1. Evidence MUST be committed to git
2. Evidence MUST NOT be modified after creation
3. Evidence MUST include all three layers for complete replay

---

## 8. Multicore Coordination Matrix

| Layer | Multicore Role | Synchronization | Ordering |
|-------|---------------|-----------------|----------|
| **ABDF** | shared data substrate | lock-free reads | N/A |
| **BCIB** | per-CPU execution plan | mailbox coordination | logical time |
| **Phase-11** | global ordering + GCP | DLT + commit protocol | event_seq + ltick |

### Multicore Invariants

- **ABDF**: Concurrent reads allowed, writes serialized
- **BCIB**: Each CPU has independent execution plan
- **Phase-11**: Global event_seq MUST be monotonic across all CPUs

---

## 9. Proof Composition Matrix

| Proof Component | Source Layer | Hash Input | Signature |
|----------------|--------------|------------|-----------|
| `kernel_image_hash` | Build system | kernel.elf | N/A |
| `config_hash` | Build system | .config | N/A |
| `ledger_root_hash` | Phase-11 | decision_ledger.bin | YES |
| `transcript_root_hash` | Phase-11 | transcript.jsonl | YES |
| `replay_result_hash` | Phase-11 | replay engine output | YES |
| `final_state_hash` | Phase-11 | kernel state snapshot | YES |

### Proof Validity Rules

- **All hashes** MUST be SHA-256
- **Signature** MUST cover entire proof manifest
- **Trust anchor** MUST be defined (CI runner or hardware root)

---

## 10. CI Gate Validation Matrix

| Gate | ABDF Check | BCIB Check | Phase-11 Check |
|------|-----------|-----------|----------------|
| **ABI** | schema stability | opcode stability | event type stability |
| **Boundary** | N/A | syscall-only enforcement | Ring0 mechanism-only |
| **Hygiene** | snapshot committed | plan committed | evidence committed |
| **Constitutional** | type system compliance | instruction compliance | ordering compliance |
| **Performance** | N/A | N/A | deterministic baseline |
| **Replay** | snapshot match | plan match | transcript match |

### Gate Failure Policy

- **Any gate failure** → PR BLOCKED
- **Evidence missing** → CI FAIL
- **Hash mismatch** → REPLAY FAIL

---

## 11. Evolution Policy Matrix

| Layer | Allowed Changes | Forbidden Changes | Version Bump |
|-------|----------------|-------------------|--------------|
| **ABDF** | new segment type, new scalar type | remove existing type, change header layout | MINOR |
| **BCIB** | new opcode, new flag | remove opcode, change instruction size | MINOR |
| **Phase-11** | new event type, new hash algorithm | remove event type, change ledger format | MAJOR |

### Backward Compatibility Rules

- **ABDF v2** MUST read ABDF v1 snapshots
- **BCIB v2** MUST validate BCIB v1 plans
- **Phase-11 v2** MUST replay Phase-11 v1 transcripts

---

## 12. Critical Invariants (Non-Negotiable)

### ABDF Invariants

1. `segment_count` MUST match actual segment table entries
2. `meta_idx` MUST be valid index into meta table
3. `offset + length` MUST NOT exceed buffer size
4. `type` MUST be valid `AbdfType` variant

### BCIB Invariants

1. `instr_count` MUST match actual instruction array length
2. `opcode` MUST be valid `BcibOpcode` variant
3. `DataQuery` MUST reference valid ABDF object
4. `End` MUST be final instruction

### Phase-11 Invariants

1. `event_seq` MUST be globally monotonic
2. `ltick` MUST be deterministic logical time
3. `entry_hash` MUST match `H(prev_hash || payload)`
4. `transcript` MUST record ALL significant kernel events

---

## 13. Failure Mode Matrix

| Failure | ABDF Response | BCIB Response | Phase-11 Response |
|---------|--------------|--------------|------------------|
| **Invalid type** | return error | halt execution | N/A |
| **Invalid opcode** | N/A | return error | N/A |
| **Hash mismatch** | integrity fail | N/A | replay fail |
| **Ordering violation** | N/A | N/A | panic |
| **Capability violation** | N/A | syscall reject | ledger reject entry |

### Fail-Closed Policy

- **ABDF**: Invalid data → reject operation
- **BCIB**: Invalid instruction → halt execution
- **Phase-11**: Ordering violation → kernel panic

---

## 14. Implementation Checklist

### ABDF Implementation

- [ ] Add `segment_table_offset` to header
- [ ] Add `meta_table_offset` to header
- [ ] Add `content_hash` to header
- [ ] Add `schema_hash` to header
- [ ] Implement hash verification on load

### BCIB Implementation

- [ ] Add `plan_hash` to header
- [ ] Implement opcode validation
- [ ] Add ABDF object reference validation
- [ ] Implement execution trace export

### Phase-11 Implementation

- [ ] Implement `ay_decision_ledger_entry_t`
- [ ] Implement `ay_transcript_entry_t`
- [ ] Implement `ay_ordering_state_t`
- [ ] Implement `ay_replay_state_t`
- [ ] Implement `ay_gcp_record_t`
- [ ] Implement `ay_proof_manifest_t`
- [ ] Add hash chain validation
- [ ] Add replay engine
- [ ] Add evidence export

---

## 15. References

- `ayken-core/crates/abdf/` - ABDF implementation
- `ayken-core/crates/bcib/` - BCIB implementation
- `kernel/include/ayken_abi.h` - Syscall ABI
- `docs/architecture-board/decisions/` - ADRs
- `evidence/` - Evidence directory structure

---

**Maintained by:** AykenOS Architecture Board  
**Last Updated:** 2026-03-06  
**Next Review:** Before Phase-11 implementation

**This document is binding. Violations result in PR rejection.**

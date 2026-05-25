# Design Document: AYKEN VCP Execution Binding

## Overview

This feature transforms AYKEN Validation Control Plane (VCP) from a documented/CI-only authority into a runtime-enforced authority by binding validation state and fail-closed behavior to the execution lifecycle. The design introduces a kernel-level validation enforcement point that carries validation state through execution slots, binds BCIB execution contracts to VCP decisions, and enforces ABDF boundary validation at runtime.

**Current Gap**: VCP exists as "documented authority" (CI-enforced) but NOT "runtime authority" (execution-enforced). Validation observes but doesn't enforce at runtime. Execution can proceed without validation state. No fail-closed mechanism during execution lifecycle.

**Target State**: Validation becomes mandatory at runtime. Execution slots carry validation state. Invalid execution fails closed. BCIB/ABDF boundaries are bound to VCP. Runtime enforcement matches CI enforcement.

## Architecture

```mermaid
graph TD
    A[CI/Spec Layer] -->|Validation State| B[VCP Runtime Hook]
    B -->|Attach State| C[Execution Slot]
    C -->|Validation Check| D[BCIB Execution Contract]
    C -->|Validation Check| E[ABDF Boundary Validation]
    D -->|Evidence| F[Runtime Evidence Emission]
    E -->|Evidence| F
    F -->|Artifacts| G[CI Verification]
    
    B -->|Fail-Closed| H[Invalid Execution Block]
    D -->|Fail-Closed| H
    E -->|Fail-Closed| H
    
    style B fill:#f96,stroke:#333,stroke-width:4px
    style H fill:#f66,stroke:#333,stroke-width:2px


## Validation State Trust Model

**Critical Principle**: Validation_State MUST NOT be trusted by value alone.

### Trust Requirements

A Validation_State is considered valid and trustworthy ONLY if ALL of the following conditions are met:

1. **Capability Binding**: It is bound to a kernel-issued capability (prevents forgery)
2. **Context Integrity**: Its context_hash matches the current execution context (prevents replay)
3. **Signature Verification**: Its signature verifies against the VCP trust root (ensures authenticity)
4. **Validation Result**: Its validation_result field is VCP_VALID (indicates CI approval)
5. **Nonce Uniqueness**: Its nonce has not been replayed (prevents reuse)

**Failure of ANY check MUST trigger fail-closed enforcement.**

### Why Hybrid Model is Required

**Hash alone is insufficient:**
- Hash proves: content has not been modified
- Hash does NOT prove: state was produced for this slot, state came from authorized source, state has not been replayed
- **Risk**: Replay attack (same hash used in different context)

**Signature alone is insufficient:**
- Signature proves: authorized source signed the state
- Signature does NOT prove: state is bound to this specific execution context
- **Risk**: Context replay (same signed state used in different slot)

**Capability alone is insufficient:**
- Capability proves: runtime binding is strong
- Capability does NOT prove: state content matches CI/VCP decision
- **Risk**: Content authenticity not guaranteed

**Hybrid model (Capability + Context Hash + Signature) provides:**
- **Capability binding** → prevents forgery at runtime
- **Context hash** → prevents replay across different execution contexts
- **Signature** → ensures CI→runtime authenticity bridge
- **Combined** → verified-input system (not trusted-input system)

### VCP Trust Token Structure

```c
struct vcp_validation_state {
    uint64_t validation_result;  // VCP_VALID, VCP_INVALID, VCP_MISSING
    uint64_t contract_id;        // BCIB contract identifier
    uint64_t boundary_policy;    // ABDF boundary policy identifier
    uint64_t context_hash;       // Hash of execution context
    uint64_t nonce;              // Unique nonce (replay protection)
    uint64_t signature;          // VCP trust root signature
    uint64_t capability_id;      // Kernel capability binding
    uint64_t evidence_id;        // Evidence trail reference
};
```

### Runtime Verification Flow

```c
int vcp_verify_validation_state(struct execution_slot *slot) {
    if (!slot || !slot->validation_state)
        return VCP_FAIL_CLOSED;  // Missing state
    
    if (!vcp_verify_capability(slot, slot->validation_state))
        return VCP_FAIL_CLOSED;  // Capability binding failed
    
    if (!vcp_verify_context_hash(slot, slot->validation_state))
        return VCP_FAIL_CLOSED;  // Context mismatch (replay)
    
    if (!vcp_verify_signature(slot->validation_state))
        return VCP_FAIL_CLOSED;  // Signature verification failed
    
    if (!vcp_verify_nonce(slot->validation_state))
        return VCP_FAIL_CLOSED;  // Nonce replayed
    
    if (slot->validation_state->validation_result != VCP_VALID)
        return VCP_FAIL_CLOSED;  // CI rejected execution
    
    return VCP_VALID;  // All checks passed
}
```

### Authoritative Principle

**CRITICAL RULE**: The `validation_result` flag is NEVER authoritative by itself.

**Verification is authoritative.**

```c
// ❌ WRONG (trusted-input system)
if (state->validation_result == VCP_VALID) {
    proceed();
}

// ✅ CORRECT (verified-input system)
if (vcp_verify_validation_state(slot) == VCP_VALID) {
    proceed();
}
```

### Attack Scenarios Prevented

1. **Fake State Injection**: Attacker creates fake validation_state → Capability binding fails → Fail-closed
2. **Replay Attack**: Attacker reuses valid state from different slot → Context hash mismatch → Fail-closed
3. **Context Replay**: Attacker reuses signed state in different execution → Context hash mismatch → Fail-closed
4. **Nonce Replay**: Attacker reuses same state twice → Nonce check fails → Fail-closed
5. **Signature Forgery**: Attacker modifies state content → Signature verification fails → Fail-closed

### Trust Model Guarantees

**What the trust model guarantees:**
- Validation state cannot be forged (capability binding)
- Validation state cannot be replayed (context hash + nonce)
- Validation state authenticity is verifiable (signature)
- Validation state is bound to specific execution context (context hash)
- All verification failures result in fail-closed enforcement

**What the trust model does NOT guarantee:**
- Semantic correctness of validator logic (validator may have bugs)
- Zero-day attacks on cryptographic primitives (signature algorithm vulnerabilities)
- Physical attacks on kernel memory (out of scope for software enforcement)

### Integration with VCP Runtime Hook

The VCP runtime hook MUST call `vcp_verify_validation_state()` BEFORE checking the validation result:

```c
int vcp_runtime_validate(struct execution_slot *slot) {
    // Step 1: Verify trust (capability + context + signature + nonce)
    int trust_result = vcp_verify_validation_state(slot);
    if (trust_result != VCP_VALID) {
        vcp_fail_closed(slot, "Trust verification failed");
        return trust_result;
    }
    
    // Step 2: Check validation result (only after trust verified)
    if (slot->validation_state->validation_result != VCP_VALID) {
        vcp_fail_closed(slot, "Validation result invalid");
        return VCP_INVALID;
    }
    
    // Step 3: Emit evidence
    vcp_emit_validation_check(slot, VCP_VALID);
    
    return VCP_VALID;
}
```

## Hybrid Evidence Chain Architecture

**Critical Principle**: Validation enforcement without evidence is unverifiable. Evidence chain transforms the system from "secure-looking" to "provable."

### Evidence Chain Model

AykenOS uses a **hybrid evidence chain** model combining:
1. **Slot-local append-only chains** → isolation, determinism, slot-local authority
2. **Global append-only chain** → system-wide authority, CI replayability
3. **Slot-to-global anchoring** → binding slot chains to global chain for integrity
4. **Optional ring buffer** → diagnostics only (non-authoritative)

**This is NOT a blockchain:**
- ❌ No network, no consensus, no nondeterminism
- ✅ Append-only, hash-linked, kernel-controlled log
- ✅ Deterministic, replayable, tamper-evident

### Evidence Entry Structure

```c
struct evidence_entry {
    uint64_t index;              // Monotonically increasing
    uint64_t prev_hash;          // Hash of previous entry
    uint64_t event_hash;         // Hash of this event
    uint64_t timestamp;          // Logical monotonic counter (NOT wall clock)
    uint64_t slot_id;            // Execution slot ID
    uint64_t event_type;         // VALIDATION / FAIL / BCIB / ABDF / ANCHOR
    uint64_t validation_result;  // VCP_VALID / VCP_INVALID / VCP_FAIL_CLOSED
    uint64_t context_hash;       // Execution context hash (includes ABDF snapshot)
    uint64_t signature;          // Cryptographic signature (NEW)
    uint64_t signer_id;          // Signer identity (NEW)
    uint64_t trust_root_version; // Trust root key version (NEW)
};

// Chain integrity
entry_hash = HASH(index || prev_hash || event_hash || timestamp || slot_id);
```

**CRITICAL: Timestamp Determinism**

`timestamp` MUST be a **logical monotonic counter**, NOT wall clock time:
- ✅ Valid: execution tick counter, event sequence number
- ❌ Invalid: system time, wall clock, rdtsc (non-deterministic)

**Why:**
- Wall clock → non-deterministic (different on each run)
- Logical counter → deterministic (same execution → same timestamps)
- CI replay requires deterministic timestamps

### Evidence Trust Model

**Critical Principle**: Evidence without signature = untrusted log entry = rejected.

Evidence authenticity is guaranteed through:
1. **Signature**: Every evidence entry MUST be signed by VCP trust root
2. **Verification**: Signature MUST be verified BEFORE evidence enters chain
3. **Rejection**: Unsigned or invalid evidence triggers fail-closed
4. **Unified Trust**: Evidence uses same trust anchor as validation state

**Why signature is required:**
- Hash alone proves: content not modified
- Hash does NOT prove: who produced this evidence, evidence came from authorized source
- **Signature proves**: authorized source produced this evidence, evidence is authentic

**Evidence Trust Flow:**

```c
int evidence_emit_and_verify(struct evidence_entry *entry) {
    // Step 1: Sign evidence with kernel evidence producer key
    // CRITICAL: Kernel holds producer key (NOT trust root private key)
    // CI verifies: (1) evidence signature with producer key, (2) producer key authorized by trust root
    if (!evidence_sign(entry, KERNEL_EVIDENCE_PRODUCER_KEY_ID, current_trust_root_version)) {
        vcp_fail_closed(slot, "Evidence signing failed");
        return VCP_FAIL_CLOSED;
    }
    
    // Step 2: Verify signature BEFORE accepting
    if (!evidence_verify_signature(entry)) {
        vcp_fail_closed(slot, "Evidence signature verification failed");
        return VCP_FAIL_CLOSED;
    }
    
    // Step 3: Append to chain (only verified evidence)
    if (!chain_append(entry)) {
        vcp_fail_closed(slot, "Evidence append failed");
        return VCP_FAIL_CLOSED;
    }
    
    return VCP_VALID;
}
```

### Trust Root Lifecycle and Key Rotation

**Critical Principle**: Trust roots are not static. Key rotation is inevitable. The system MUST handle trust root lifecycle without breaking evidence verification.

**Trust Root Versioning:**

```c
struct vcp_trust_root {
    uint64_t trust_root_id;      // Unique trust root identifier
    uint64_t version;            // Key version (incremented on rotation)
    uint64_t public_key;         // Public key for signature verification
    uint64_t valid_from;         // Logical timestamp when this key became valid
    uint64_t valid_until;        // Logical timestamp when this key expires (0 = current)
    uint64_t status;             // ACTIVE / ROTATED / REVOKED
};
```

**Key Rotation Flow:**

```
1. New key generated (version N+1)
2. Old key (version N) marked ROTATED
3. New evidence signed with version N+1
4. Old evidence remains verifiable with version N
5. CI maintains trust root history for backward verification
```

**Backward Verification Policy:**

```c
int evidence_verify_with_trust_root_history(struct evidence_entry *entry) {
    // Get trust root for the version recorded in evidence
    struct vcp_trust_root *root = get_trust_root_by_version(entry->trust_root_version);
    
    if (!root) {
        // Trust root version not found (revoked or unknown)
        return VCP_FAIL_CLOSED;
    }
    
    if (root->status == REVOKED) {
        // Trust root revoked → evidence invalid
        return VCP_FAIL_CLOSED;
    }
    
    // Verify signature with historical trust root
    return verify_signature(entry, root->public_key);
}
```

**Key Rotation Rules:**
- New evidence MUST use current trust root version
- Old evidence MUST remain verifiable with historical trust root
- Revoked trust roots invalidate all evidence signed with that key
- CI MUST maintain trust root history for replay verification

**Why This Matters:**
- Without versioning: key rotation breaks all old evidence
- With versioning: old evidence remains verifiable, new evidence uses new key
- Revocation: compromised keys can be revoked, invalidating their evidence

### Evidence Chain Properties

**Tamper-evident**: If any entry is modified, the entire chain becomes invalid (hash chain breaks)

**Deterministic**: Same execution produces identical evidence chain (enables CI replay)

**Kernel authority**: Userland cannot write to evidence chains (only kernel emitter writes)

**Fail-closed binding**: If evidence emission fails, execution MUST fail-closed

**Cryptographically signed**: All evidence is signed and verified (authenticity guaranteed)

**ABDF-bound**: Evidence context_hash includes ABDF snapshot hash (execution state binding)

### Hybrid Architecture

```
┌─────────────────────────────────────────────────────────┐
│ Global Append-Only Chain (AUTHORITATIVE)                │
│ out/evidence/run-{id}/chain/global_chain.bin            │
│ - System-wide authority                                 │
│ - CI replay source                                      │
│ - Fail-closed events                                    │
│ - Slot anchor events                                    │
└─────────────────────────────────────────────────────────┘
                          ▲
                          │ anchor (slot head hash)
                          │
┌─────────────────────────────────────────────────────────┐
│ Slot-Local Append-Only Chains (EXECUTION-LOCAL)         │
│ runtime/slots/slot-{id}/local_chain.bin                 │
│ - Slot-local authority                                  │
│ - Execution isolation                                   │
│ - Validation events                                     │
│ - BCIB/ABDF events                                      │
└─────────────────────────────────────────────────────────┘
                          │
                          │ (optional, non-authoritative)
                          ▼
┌─────────────────────────────────────────────────────────┐
│ Ring Buffer (DIAGNOSTICS ONLY)                          │
│ runtime/ring/recent_events.bin                          │
│ - Fast access                                           │
│ - Overwrite allowed                                     │
│ - NOT used for CI                                       │
└─────────────────────────────────────────────────────────┘
```

### Diagnostic Evidence Isolation Guarantee (Task 5)

**CRITICAL PRINCIPLE**: Diagnostic evidence emission is strictly observational and MUST be side-effect free.

**Isolation Contract**:
- Diagnostic evidence emission MUST NOT affect validation outcome
- Diagnostic evidence emission MUST NOT affect trust verification
- Diagnostic evidence emission MUST NOT affect execution path
- Diagnostic evidence emission failure MUST NOT trigger fail-closed
- Diagnostic evidence emission MUST NOT block execution

**Implementation Requirements**:
- All diagnostic evidence functions MUST return `void` (no error propagation)
- All diagnostic evidence functions MUST handle NULL inputs gracefully
- All diagnostic evidence functions MUST handle buffer overflow gracefully
- All diagnostic evidence functions MUST NOT allocate memory dynamically
- All diagnostic evidence functions MUST NOT call fail-closed handlers

**Rationale**: Diagnostic evidence (Task 5) is for debugging and telemetry only. Authoritative evidence (Task 20-23) will enforce fail-closed on emission failure. Mixing these concerns would create a fragile system where debug telemetry could break execution.

**Verification**: Property test MUST verify that evidence emission can be disabled without affecting execution outcome.

### Evidence Emission Strategy (Authoritative - Task 20-23)

**Hot path** (frequent events):
- Validation checks → slot-local chain
- Boundary checks → slot-local chain
- Contract checks → slot-local chain

**Commit point** (critical events):
- Fail-closed → slot-local + global chain
- Trust failures → slot-local + global chain
- Slot completion → global chain (anchor slot head hash)

This strategy maintains performance while ensuring critical events are globally recorded.

### Critical Rule: No Evidence = No Execution

```c
// Evidence emission MUST succeed for execution to proceed
int vcp_emit_validation_check(struct execution_slot *slot, int result) {
    if (!slot_evidence_append(slot->id, create_event(result))) {
        vcp_fail_closed(slot, "Slot evidence emission failed");
        return VCP_FAIL_CLOSED;
    }
    
    // Critical events also go to global chain
    if (result == VCP_FAIL_CLOSED || result == VCP_INVALID) {
        if (!global_evidence_append(create_event(result))) {
            vcp_fail_closed(slot, "Global evidence emission failed");
            return VCP_FAIL_CLOSED;
        }
    }
    
    return VCP_VALID;
}
```

### Slot Chain Anchoring

When an execution slot completes, its head hash is anchored to the global chain:

```c
void execution_slot_destroy(struct execution_slot *slot) {
    // Compute slot chain head hash
    uint64_t slot_head_hash = compute_slot_head_hash(slot->id);
    
    // Create anchor event
    struct evidence_entry anchor = {
        .event_type = EVENT_SLOT_CHAIN_COMMIT,
        .slot_id = slot->id,
        .context_hash = slot_head_hash,
        .timestamp = get_deterministic_time(),
    };
    
    // Anchor to global chain (MUST succeed)
    if (!global_evidence_append(anchor)) {
        vcp_fail_closed(slot, "Slot anchor failed");
        return;
    }
    
    // Emit final evidence
    vcp_emit_validation_check(slot, VCP_VALID);
    
    // Cleanup
    free_validation_state(slot->validation_state);
}
```

### Evidence Directory Structure

```
out/evidence/
  run-{ci_run_id}/
    meta/
      run.json              # Run metadata (commit, phase, validation standard)
      environment.json      # Environment context
    chain/
      global_chain.bin      # AUTHORITATIVE global chain
      global_chain.json     # Human-readable format
      head.hash             # Global chain head hash
    runtime/
      slots/
        slot-{id}/
          local_chain.bin   # Slot-local chain
          local_chain.json  # Human-readable format
          head.hash         # Slot chain head hash
      ring/
        recent_events.bin   # Ring buffer (non-authoritative)
    validation/
      trust_tokens.json     # Trust verification results
      verification_results.json
    bcib/
      contract_checks.json  # BCIB enforcement events
    abdf/
      boundary_events.json  # ABDF boundary events
    summary/
      summary.json          # Execution summary
      summary.md            # Human-readable summary
```

### CI Integration

CI verification workflow:
1. Load `chain/global_chain.bin`
2. Replay evidence chain
3. Verify hash chain integrity
4. For each slot anchor event, load and verify slot chain
5. Detect violations (missing evidence, hash mismatch, tampered entries)
6. FAIL if any violation detected

### Attack Scenarios Prevented

1. **Evidence Deletion**: Attacker deletes evidence entry → hash chain breaks → CI detects
2. **Evidence Modification**: Attacker modifies entry → hash mismatch → CI detects
3. **Evidence Reordering**: Attacker reorders entries → index/hash mismatch → CI detects
4. **Slot Chain Forgery**: Attacker forges slot chain → anchor hash mismatch → CI detects
5. **Silent Failure**: System fails without evidence → execution blocked (fail-closed)

### Evidence Chain Guarantees

**What the evidence chain guarantees:**
- All validation events are recorded (comprehensive audit trail)
- Evidence cannot be tampered with (tamper-evident via hash chain)
- Evidence is authentic (cryptographically signed and verified)
- Evidence is deterministically replayable (CI can verify)
- Evidence emission failure blocks execution (no evidence = no execution)
- Slot chains are isolated (slot A cannot affect slot B)
- Global chain is authoritative (single source of truth)
- Evidence is bound to execution state (ABDF snapshot hash in context_hash)

**What the evidence chain does NOT guarantee:**
- Semantic correctness of validation logic (validator may have bugs)
- Protection against physical attacks on storage (out of scope)
- Real-time evidence availability (evidence is written asynchronously)
- Zero-day attacks on cryptographic primitives (signature algorithm vulnerabilities)

### Evidence Binding to ABDF Snapshot

**Critical Integration**: Evidence context_hash MUST include ABDF snapshot hash to bind evidence to deterministic execution state.

```c
uint64_t compute_evidence_context_hash(struct execution_slot *slot) {
    // Get ABDF snapshot hash for current execution state
    uint64_t abdf_snapshot_hash = abdf_compute_snapshot_hash(slot);
    
    // Combine execution context with ABDF snapshot
    uint64_t context_hash = HASH(
        slot->id ||
        slot->validation_state->contract_id ||
        slot->validation_state->boundary_policy ||
        abdf_snapshot_hash  // ← CRITICAL: binds evidence to execution state
    );
    
    return context_hash;
}
```

**Why ABDF binding is critical:**
- ABDF provides deterministic, pointer-free, immutable execution state snapshots
- Binding evidence to ABDF snapshot ensures evidence is tied to specific execution state
- This enables replay determinism: same execution state → same evidence
- CI can replay execution, compute ABDF snapshot, and verify evidence context_hash matches

**Replay Verification Flow:**

```
CI Replay:
1. Load evidence chain
2. Replay execution
3. Compute ABDF snapshot at each evidence point
4. Verify evidence.context_hash == HASH(execution_context || abdf_snapshot_hash)
5. If mismatch → evidence does not match execution state → FAIL
```

This integration completes the determinism guarantee: **execution → ABDF snapshot → evidence → verification**.

## CI/Merge Governance and Authority Control

**Critical Principle**: In AykenOS, merge = authority change. Unverified code entering main branch compromises system authority.

### Authority Model

```
Code Authority Hierarchy:
1. ci-freeze PASS → authoritative (verified)
2. ci-freeze FAIL → non-authoritative (unverified)
3. local build → diagnostic only (not authority)
```

**Rule**: Only ci-freeze PASS code can be merged to main.

### Merge Policy

**MERGE ONLY IF:**
- ci-freeze == PASS
- All CRITICAL tests == PASS
- Evidence chain verified

**IF ci-freeze FAIL:**
- Merge BLOCKED
- Fix required
- Re-run CI
- Merge only after PASS

### Development Workflow

```
Developer Flow:
1. Implement task
2. Run local tests (diagnostic)
3. Commit to feature branch
4. Push feature branch
5. Open PR
6. CI runs (ci-freeze gate)
7. IF PASS → merge allowed
8. IF FAIL → fix → re-run CI

Branch Model:
- main: protected, always green, ci-freeze PASS required
- feature/*: development, isolated changes
```

### Test Classification and Blocking

**CRITICAL Tests** (blocking):
- Fail-closed enforcement (Property 2, 3, 4, 9, 10, 11)
- Trust verification (Property 24-28)
- Evidence signature (Property 34-36)
- Evidence failure → fail-closed (Property 40)
- Deterministic format (Property 29)

**IF CRITICAL test fails → CI FAIL → merge BLOCKED**

**REQUIRED Tests** (phase closure):
- Must pass before phase closure
- Not blocking for individual merges
- Examples: append-only integrity, slot isolation, replay

**QUALITY Tests** (non-blocking):
- Should pass but not blocking
- Examples: comprehensive evidence emission

### Phase Closure Rule

**Phase closes ONLY IF:**
- ci-freeze PASS
- All CRITICAL tests PASS
- All REQUIRED tests PASS
- Evidence chain exists and verified

**Phase closure = verified authority checkpoint**

### Push Discipline

**Local Push:**
- Allowed for diagnostic
- NOT authority
- No CI requirement

**PR Push:**
- Requires local CRITICAL tests PASS
- Triggers CI (ci-freeze)

**Merge Push:**
- Requires CI ci-freeze PASS
- Authority change
- Protected by branch rules

### CI as Authority Gate

**ci-freeze is NOT a quality check.**

**ci-freeze is an authority gate:**
- PASS → code is verified, can become authority
- FAIL → code is unverified, cannot become authority

**This is the foundation of AykenOS authority model:**
- Validation → Trust → Enforcement → Evidence → CI Verification → Authority

### CI Override Prohibition

**CRITICAL RULE: NO MANUAL OVERRIDE OF CI AUTHORITY**

```
IF ci-freeze FAIL:
  → merge IMPOSSIBLE
  → NO admin bypass
  → NO emergency override
  → NO "just this once" exception
```

**Why no override:**
- Override = authority bypass
- Authority bypass = system compromise
- One override → precedent → discipline collapse

**What if production is blocked:**
1. Fix the code
2. Re-run CI
3. Wait for PASS
4. Then merge

**NO shortcuts. NO exceptions.**

**Rationale:**
- AykenOS authority model: ci-freeze = truth
- Override = "we don't trust our own authority system"
- If ci-freeze can be bypassed → entire validation/trust/evidence chain is meaningless

**Emergency Response:**
- Emergency ≠ bypass authority
- Emergency = fix faster, but still through CI
- Hot fix → feature branch → CI → merge (same process, faster execution)

**This is non-negotiable.**

## Naming and Directory Governance

**Critical Principle**: Naming is part of determinism. Inconsistent naming breaks machine-parsability, CI replay, and architectural clarity.

### Naming Convention Rules

**File Naming:**
- Format: `snake_case` ONLY, `lowercase` ONLY
- ✅ Valid: `vcp_runtime.c`, `boundary_enforcement.c`, `execution_slot.h`
- ❌ Invalid: `VcpRuntime.c`, `boundaryEnforcement.c`, `ExecutionSlot.h`

**Module Prefix System:**

| Domain | Prefix | Examples |
|--------|--------|----------|
| Validation Control Plane | `vcp_*` | `vcp_runtime.c`, `vcp_evidence.c` |
| BCIB Execution Contracts | `bcib_*` | `bcib_executor.c`, `bcib_worker.c` |
| ABDF Boundary Enforcement | `boundary_*` | `boundary_enforcement.c` |
| Execution Slots | `slot_*` | `slot_evidence_chain.c` |

**Function Naming:**
- Format: `<module>_<action>_<object>()`
- Examples: `vcp_runtime_validate()`, `evidence_verify_signature()`, `slot_chain_append()`

**Struct Naming:**
- Format: `struct <domain>_<entity>`
- Examples: `struct vcp_validation_state`, `struct evidence_entry`, `struct slot_evidence_chain`

**Macro/Enum Naming:**
- Format: `UPPER_CASE` + domain prefix
- Examples: `VCP_VALID`, `VCP_FAIL_CLOSED`, `EVENT_SLOT_CHAIN_COMMIT`

### Directory Structure Mapping

**Existing AykenOS Structure (Preserved):**
```
kernel/
  sys/          → System-level enforcement (VCP, BCIB, ABDF, slots)
  include/      → Public headers
  mm/           → Memory management
  proc/         → Process management
  sched/        → Scheduler
  fs/           → Filesystem
  lib/          → Libraries
```

**New VCP Components (Follow Existing Pattern):**
```
kernel/
  sys/
    vcp_runtime.c              ← NEW
    vcp_evidence.c             ← NEW
    fail_closed.c              ← NEW
    boundary_enforcement.c     ← EXISTING (already follows convention)
    execution_slot.c           ← EXISTING (already follows convention)
  include/
    vcp_runtime.h              ← NEW
    vcp_evidence.h             ← NEW
    execution_slot.h           ← EXISTING
```

**Evidence Directory Structure:**
```
out/
  evidence/
    run-{ci_run_id}/
      chain/
        global_chain.bin       ← FIXED NAME (no variation)
        head.hash              ← FIXED NAME
      runtime/
        slots/
          slot-{id}/
            local_chain.bin    ← FIXED NAME
            head.hash          ← FIXED NAME
      validation/
      bcib/
      abdf/
      summary/
```

### Forbidden Patterns

**Generic Filenames (WITHOUT domain prefix):**
- ❌ `utils.c`, `helper.c`, `common.c`, `misc.c`
- ✅ Exception: Domain-specific managers allowed (e.g., `capability_manager.c` is valid)
- **Rationale**: Non-deterministic responsibility, architectural ambiguity

**Evidence Naming Violations:**
- ❌ Random suffixes: `chain_abc123.bin`
- ❌ Timestamp drift: `chain_20260503.bin`
- ❌ Dynamic names: `chain_${random}.bin`
- **Rationale**: CI replay requires fixed, deterministic paths

### CI Enforcement

**Naming Lint Check (`ci-naming-check.sh`):**
1. Detect forbidden filenames (utils/helper/common without prefix)
2. Detect uppercase in filenames
3. Detect camelCase in code
4. Verify VCP module prefix (vcp_*, bcib_*, boundary_*, slot_*)

**Enforcement:**
- Naming check integrated into ci-freeze pipeline
- Naming violation → CI FAIL → merge BLOCKED
- **Rule**: Naming violation = architectural violation

### Why This Matters

**Determinism:**
- Fixed naming → deterministic paths → CI replay works
- Evidence files with dynamic names break replay verification

**Machine-Parsability:**
- Consistent prefix system → automated tooling can parse code structure
- grep, analysis tools, CI scripts rely on predictable naming

**Architectural Clarity:**
- File name reveals responsibility: `vcp_runtime.c` → VCP runtime enforcement
- Generic names hide responsibility: `utils.c` → what does this do?

**AykenOS Principle:**
- System = deterministic + verifiable
- Naming is part of determinism
- Inconsistent naming → non-deterministic system

## ABDF Canonical Data Layer

**Critical Principle**: ABDF is the canonical internal format. All external inputs MUST be canonicalized to ABDF before execution.

### Why ABDF is Canonical

**ABDF Properties:**
- Immutable
- Deterministic
- Pointer-free
- Snapshot-based

**Without ABDF canonicalization:**
```json
{"value": 1}    vs    {"value": 1.0}
```
JSON treats these as equivalent, but binary representation differs → non-deterministic execution.

**With ABDF canonicalization:**
```
JSON → ABDF → deterministic binary → same execution → same evidence
```

### Canonical Data Flow

```
External Formats (JSON, CLI input, AI output)
         ↓
Canonicalization Layer (format → ABDF)
         ↓
ABDF (internal, immutable, deterministic)
         ↓
Execution + Evidence + Replay
```

### Authority Layers

**Layer 1: Kernel (MANDATORY)**
- `execution_slot.payload = ABDF ONLY`
- Non-ABDF payloads rejected at slot creation

**Layer 2: Boundary (CONVERSION)**
- JSON → ABDF converter
- CLI → ABDF converter
- AI output → ABDF converter

**Layer 3: Evidence (BINDING)**
- `evidence.context_hash = HASH(ABDF snapshot)`
- Evidence bound to ABDF snapshot for deterministic replay

### Canonical Equivalence Property

**Property 42: Canonical Determinism**

```
Input A (JSON) → ABDF(A)
Input B (CLI)  → ABDF(B)
Input C (AI)   → ABDF(C)

MUST: ABDF(A) == ABDF(B) == ABDF(C)
```

If this property fails → system is non-deterministic.

### Why This Matters for AI/Userland

Future extensions (AI agents, semantic CLI, auto-execution) will produce diverse input formats. Without ABDF canonicalization:
- AI output format drift → non-deterministic execution
- CLI syntax variations → different execution for same intent
- Userland format evolution → replay breaks

With ABDF canonicalization:
- All inputs normalized to canonical format
- Determinism guaranteed regardless of input source
- CI replay works across all input types

## Future Extension Boundary Contract

**Critical Principle**: Future extensions (AI, userland, semantic planner) MUST NOT bypass validation. All external inputs go through ABDF → VCP → Execution.

### Extension Boundary Rules

**Rule 1: AI is Advisory-Only**
- AI planner = advisory (cannot execute directly)
- AI output → BCIB candidate
- BCIB candidate → VCP validation
- Validation PASS → execution
- Validation FAIL → blocked

**Rule 2: No Direct Execution Path**
```
❌ WRONG:
AI → execution (bypass)
CLI → execution (bypass)
Userland → execution (bypass)

✅ CORRECT:
AI → ABDF → VCP → execution
CLI → ABDF → VCP → execution
Userland → ABDF → VCP → execution
```

**Rule 3: Extension Integration Pattern**

ALL future extensions MUST follow this pattern:
```
1. External Input (AI, CLI, userland, semantic planner)
2. ABDF Canonicalization (deterministic conversion)
3. VCP Validation (trust verification)
4. Execution (if validation PASS)
```

### AI Integration Guardrails

**Phase 16+ will add:**
- AI agents
- Semantic CLI
- Auto-execution
- BCIB toolchain surface

**Without extension boundary:**
- AI could bypass VCP validation
- AI output → direct execution (authority bypass)
- Non-deterministic AI output → non-deterministic system

**With extension boundary:**
- AI output → ABDF (canonicalized)
- ABDF → VCP validation (trust verified)
- Validation PASS → execution
- Validation FAIL → blocked

**Property 43: Extension Boundary Enforcement**

```
Test: AI output cannot bypass VCP validation
Test: CLI cannot bypass VCP validation
Test: All external inputs go through ABDF canonicalization
```

### Userland CLI Integration

**CLI commands MUST:**
- Create VCP-bound execution slots
- Go through ABDF canonicalization
- Respect VCP validation decisions

**CLI commands MUST NOT:**
- Execute directly without validation
- Bypass ABDF canonicalization
- Override VCP decisions

### Why This Matters

**Without extension boundary:**
- Future AI/userland features become "bypass paths"
- Validation becomes optional
- Trust model collapses
- Evidence chain becomes meaningless

**With extension boundary:**
- All extensions respect validation authority
- Determinism guaranteed across all input sources
- Trust model remains intact
- Evidence chain covers all execution paths

**This is non-negotiable for Phase 16+ integration.**

## Interaction & Control Surface Layer

**Critical Principle**: UI NEVER EXECUTES. UI ONLY DESCRIBES. Execution happens when ABDF snapshot is validated and triggered.

### The Interaction Model

**Traditional OS (WRONG):**
```
UI button → syscall → execution (bypass)
```

**AykenOS (CORRECT):**
```
UI action → ABDF graph builder → VCP validation → execution (if valid)
```

### System Architecture

```
┌─────────────────────────────────────────┐
│ Interaction Layer (UI, Graph Editor)   │  ← Description only
│ - Visual programming                    │
│ - Node-based graph                      │
│ - AI-assisted design                    │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│ Intent / Graph Builder                  │  ← Converts to ABDF
│ - UI actions → ABDF nodes               │
│ - Graph → ABDF representation           │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│ ABDF Canonical Layer                    │  ← Immutable snapshot
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│ VCP Validation + Trust                  │  ← Authority gate
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│ Execution (BCIB)                        │  ← Execution happens here
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│ Evidence                                │  ← Audit trail
└─────────────────────────────────────────┘
```

### Graph-Based Execution Model

**What is a graph?**
- **Nodes**: Operations (compute, transform, control flow)
- **Edges**: Dependencies (data flow, control flow, ordering)
- **Graph**: Visual programming language

**Graph → ABDF Conversion:**
```c
struct abdf_node {
    uint64_t node_id;
    uint64_t node_type;      // OPERATION / DATA / CONTROL
    uint64_t operation;      // What this node does
    uint64_t inputs[];       // Input node IDs
    uint64_t outputs[];      // Output node IDs
};

struct abdf_graph {
    struct abdf_node nodes[];
    uint64_t node_count;
    uint64_t graph_hash;     // Deterministic graph hash
};
```

**Deterministic Conversion:**
```
Same graph → same ABDF → same execution → same evidence
```

### UI State vs Execution State

**CRITICAL SEPARATION:**

| Layer | State Type | Mutability | Authority |
|-------|-----------|------------|-----------|
| UI | Design state | Mutable | None |
| ABDF | Execution snapshot | Immutable | Authoritative |

**UI State (Mutable):**
- Drag-drop operations
- Node positioning
- Visual styling
- Work-in-progress design

**Execution State (Immutable):**
- ABDF snapshot
- Validated graph
- Execution-ready representation

**Rule:** UI state changes do NOT trigger execution. Execution triggered by explicit "snapshot + validate + run" action.

### Data Flow Manipulation

**Vector/Graph Editor:**
- Visual node editor
- Data flow connections
- Transformation pipelines

**Representation:**
```
UI Graph (mutable)
    ↓
ABDF Graph (immutable)
    ↓
Execution
```

**Safety Guarantees:**
- Graph depth limit (prevent infinite recursion)
- Cycle detection (prevent infinite loops)
- Bounded execution (prevent resource exhaustion)

### Preview/Simulation Layer

**Dry-Run Mode:**
```c
int simulate_abdf_graph(struct abdf_graph *graph) {
    // Validate graph structure
    if (!validate_graph_structure(graph))
        return VALIDATION_FAILED;
    
    // Check for cycles
    if (detect_cycles(graph))
        return CYCLE_DETECTED;
    
    // Check depth limit
    if (graph_depth(graph) > MAX_DEPTH)
        return DEPTH_EXCEEDED;
    
    // Simulate execution (no side effects)
    return simulate_execution(graph);
}
```

**Use Cases:**
- Validate graph before execution
- Preview execution flow
- Debug graph structure
- AI-assisted graph building

### AI Integration with Interaction Layer

**AI Role:**
- AI suggests graph structures
- AI optimizes data flow
- AI generates ABDF candidates

**AI Constraints:**
- AI output → ABDF graph (not direct execution)
- AI suggestions → user approval → validation → execution
- AI cannot bypass VCP validation

### Property 44: UI Cannot Bypass VCP

*For any* UI action, the action SHALL produce an ABDF graph that goes through VCP validation before execution, and SHALL NOT have any direct execution path.

**Validates: Requirements 17.1, 17.5**

### Property 45: Graph Determinism

*For any* graph representation, the same graph SHALL produce identical ABDF, graph depth limit SHALL be enforced, and cycle detection SHALL prevent infinite loops.

**Validates: Requirements 17.2, 17.3**

### Property 46: Graph Canonicalization Determinism

*For any* two graph representations A and B with the same logical structure, the canonical serialization SHALL produce identical node ordering, identical edge ordering, and identical ABDF output: ABDF(A) == ABDF(B).

**Validates: Requirements 18.1, 18.2, 18.3, 18.5**

### Property 47: Graph Hash Stability

*For any* graph structure, the graph_hash SHALL be deterministic and SHALL NOT depend on insertion order, UI rendering order, or AI generation order.

**Validates: Requirements 18.6**

### Property 48: Non-Canonical Graph Rejection

*For any* graph with non-deterministic ordering or ambiguous structure, the system SHALL reject the graph before ABDF conversion and SHALL emit evidence describing the canonicalization failure.

**Validates: Requirements 18.4, 18.8**

### Property 49: Diagnostic Evidence Isolation

*For any* diagnostic evidence emission operation (Task 5), the emission MUST be side-effect free and MUST NOT affect validation outcome, trust verification, or execution path under any condition including buffer overflow, write failure, or NULL inputs.

**Test Strategy**:
1. Execute validation with evidence enabled vs disabled → outcomes MUST be identical
2. Inject evidence buffer overflow → execution outcome MUST be unaffected
3. Inject evidence write failure → execution outcome MUST be unaffected
4. Verify all diagnostic evidence functions return void (no error propagation to execution path)

**Validates: Requirement 6.7 (Diagnostic Evidence Isolation), Design isolation contract**

### Why This Matters for Phase 16+

**Phase 16+ will add:**
- Visual programming interface
- Node-based execution editor
- AI-assisted system design
- Workflow automation
- Semantic CLI with graph output

**Without interaction layer:**
- UI → direct execution (bypass)
- Graph → non-deterministic execution
- AI → unvalidated execution
- Workflow → authority bypass

**With interaction layer:**
- UI → ABDF → VCP → execution
- Graph → deterministic ABDF → validated execution
- AI → ABDF candidate → VCP → execution
- Workflow → validated graph → controlled execution

**This transforms AykenOS into:**
- Visual programming OS
- AI-assisted execution engine
- Workflow automation platform
- Node-based system builder

**All while maintaining:**
- Validation authority
- Determinism guarantees
- Trust model integrity
- Evidence chain completeness

## Graph Canonicalization Engine

**Critical Principle**: Graph → ABDF conversion MUST be deterministic regardless of input source (UI, AI, CLI). Same logical graph MUST produce identical ABDF binary.

### The Canonicalization Problem

**Without canonicalization:**
```
UI generates: [Node A, Node B, Node C]
AI generates: [Node C, Node A, Node B]
CLI generates: [Node B, Node C, Node A]

→ Different ABDF binaries
→ Different hashes
→ Different evidence
→ Replay fails
→ Determinism broken
```

**With canonicalization:**
```
All sources → Canonical form: [Node A, Node B, Node C] (sorted)
→ Identical ABDF binary
→ Identical hash
→ Identical evidence
→ Replay works
→ Determinism guaranteed
```

### Canonical Graph Structure

```c
struct abdf_node {
    uint64_t node_id;           // Deterministic ID (content-hash or canonical index)
    uint64_t node_type;         // OPERATION / DATA / CONTROL
    uint64_t operation;         // Operation code
    uint64_t field_count;       // Number of fields
    struct abdf_field fields[]; // Fields sorted by field_id
    uint64_t input_count;       // Number of inputs
    uint64_t inputs[];          // Input node IDs (sorted)
    uint64_t output_count;      // Number of outputs
    uint64_t outputs[];         // Output node IDs (sorted)
};

struct abdf_edge {
    uint64_t source_id;         // Source node ID
    uint64_t target_id;         // Target node ID
    uint64_t edge_type;         // DATA / CONTROL / ORDER
    uint64_t weight;            // Edge weight (for ordering)
};

struct abdf_graph {
    uint64_t node_count;
    struct abdf_node nodes[];   // Nodes sorted by node_id
    uint64_t edge_count;
    struct abdf_edge edges[];   // Edges sorted by (source_id, target_id)
    uint64_t graph_hash;        // Deterministic hash of canonical form
};
```

### Canonicalization Algorithm

**Step 1: Deterministic Node ID Assignment**

```c
// Option A: Content-based hash (preferred)
uint64_t compute_node_id(struct abdf_node *node) {
    // Hash node content (type, operation, fields)
    // Same content → same ID
    return HASH(node->node_type || node->operation || serialize_fields(node->fields));
}

// Option B: Topological sort + monotonic assignment
uint64_t assign_canonical_node_id(struct graph *g, struct node *n) {
    // Topological sort ensures deterministic ordering
    // Assign IDs based on topological position
    return topological_position(g, n);
}
```

**Step 2: Canonical Node Ordering**

```c
void canonicalize_nodes(struct abdf_graph *graph) {
    // Sort nodes by node_id (ascending)
    qsort(graph->nodes, graph->node_count, sizeof(struct abdf_node), compare_node_id);
    
    // Sort fields within each node
    for (int i = 0; i < graph->node_count; i++) {
        qsort(graph->nodes[i].fields, graph->nodes[i].field_count, 
              sizeof(struct abdf_field), compare_field_id);
    }
    
    // Sort inputs/outputs within each node
    for (int i = 0; i < graph->node_count; i++) {
        qsort(graph->nodes[i].inputs, graph->nodes[i].input_count, 
              sizeof(uint64_t), compare_uint64);
        qsort(graph->nodes[i].outputs, graph->nodes[i].output_count, 
              sizeof(uint64_t), compare_uint64);
    }
}
```

**Step 3: Canonical Edge Ordering**

```c
void canonicalize_edges(struct abdf_graph *graph) {
    // Sort edges by (source_id, target_id)
    qsort(graph->edges, graph->edge_count, sizeof(struct abdf_edge), compare_edge);
}

int compare_edge(const void *a, const void *b) {
    struct abdf_edge *ea = (struct abdf_edge *)a;
    struct abdf_edge *eb = (struct abdf_edge *)b;
    
    if (ea->source_id != eb->source_id)
        return ea->source_id - eb->source_id;
    
    return ea->target_id - eb->target_id;
}
```

**Step 4: Graph Hash Computation**

```c
uint64_t compute_graph_hash(struct abdf_graph *graph) {
    // Hash canonical form (nodes + edges)
    uint64_t hash = HASH_INIT;
    
    // Hash all nodes (already sorted)
    for (int i = 0; i < graph->node_count; i++) {
        hash = HASH_UPDATE(hash, &graph->nodes[i], sizeof(struct abdf_node));
    }
    
    // Hash all edges (already sorted)
    for (int i = 0; i < graph->edge_count; i++) {
        hash = HASH_UPDATE(hash, &graph->edges[i], sizeof(struct abdf_edge));
    }
    
    return HASH_FINALIZE(hash);
}
```

**Step 5: Non-Canonical Graph Rejection**

```c
int validate_graph_canonicalization(struct abdf_graph *graph) {
    // Check 1: Nodes must be sorted by node_id
    for (int i = 1; i < graph->node_count; i++) {
        if (graph->nodes[i-1].node_id >= graph->nodes[i].node_id) {
            return CANONICALIZATION_FAILED_NODE_ORDER;
        }
    }
    
    // Check 2: Edges must be sorted by (source_id, target_id)
    for (int i = 1; i < graph->edge_count; i++) {
        if (compare_edge(&graph->edges[i-1], &graph->edges[i]) >= 0) {
            return CANONICALIZATION_FAILED_EDGE_ORDER;
        }
    }
    
    // Check 3: Fields within nodes must be sorted
    for (int i = 0; i < graph->node_count; i++) {
        for (int j = 1; j < graph->nodes[i].field_count; j++) {
            if (graph->nodes[i].fields[j-1].field_id >= graph->nodes[i].fields[j].field_id) {
                return CANONICALIZATION_FAILED_FIELD_ORDER;
            }
        }
    }
    
    return CANONICALIZATION_VALID;
}
```

### Canonicalization Enforcement

**At ABDF Conversion:**

```c
int graph_to_abdf(struct graph *input_graph, struct abdf_graph *output) {
    // Step 1: Assign deterministic node IDs
    assign_canonical_node_ids(input_graph);
    
    // Step 2: Convert to ABDF structure
    convert_to_abdf_structure(input_graph, output);
    
    // Step 3: Canonicalize (sort nodes, edges, fields)
    canonicalize_nodes(output);
    canonicalize_edges(output);
    
    // Step 4: Validate canonicalization
    int result = validate_graph_canonicalization(output);
    if (result != CANONICALIZATION_VALID) {
        vcp_fail_closed(slot, "Graph canonicalization failed");
        return result;
    }
    
    // Step 5: Compute graph hash
    output->graph_hash = compute_graph_hash(output);
    
    return CANONICALIZATION_VALID;
}
```

### Integration with Evidence

**Evidence context_hash includes graph_hash:**

```c
uint64_t compute_evidence_context_hash(struct execution_slot *slot) {
    uint64_t abdf_snapshot_hash = abdf_compute_snapshot_hash(slot);
    uint64_t graph_hash = slot->abdf_graph->graph_hash;
    
    uint64_t context_hash = HASH(
        slot->id ||
        slot->validation_state->contract_id ||
        slot->validation_state->boundary_policy ||
        abdf_snapshot_hash ||
        graph_hash  // ← CRITICAL: binds evidence to canonical graph
    );
    
    return context_hash;
}
```

### Property 46: Graph Canonicalization Determinism

*For any* two graph representations A and B with the same logical structure, the canonical serialization SHALL produce identical node ordering, identical edge ordering, and identical ABDF output: ABDF(A) == ABDF(B).

**Validates: Requirements 18.1, 18.2, 18.3, 18.5**

### Property 47: Graph Hash Stability

*For any* graph structure, the graph_hash SHALL be deterministic and SHALL NOT depend on insertion order, UI rendering order, or AI generation order.

**Validates: Requirements 18.6**

### Property 48: Non-Canonical Graph Rejection

*For any* graph with non-deterministic ordering or ambiguous structure, the system SHALL reject the graph before ABDF conversion and SHALL emit evidence describing the canonicalization failure.

**Validates: Requirements 18.4, 18.8**

### Why This Matters

**Without graph canonicalization:**
- UI drag-drop order affects ABDF → non-deterministic
- AI generation order affects ABDF → non-deterministic
- CLI argument order affects ABDF → non-deterministic
- Same logical graph → different evidence → replay fails

**With graph canonicalization:**
- All sources produce identical ABDF for same logical graph
- Evidence chain is deterministic across all input sources
- CI replay works regardless of input source
- Graph hash is stable and verifiable

**This completes the determinism guarantee:**
```
Input (UI/AI/CLI) → Graph → Canonicalization → ABDF → Evidence → Verification
                              ↑
                         Critical step
```

**Without this step, the entire system is non-deterministic.**

## Architecture Dependency Firewall

**Critical Principle**: Future extensions (driver, AI, UI, device) MUST NOT create circular dependencies or bypass validation boundaries. Architecture dependencies MUST be explicit and CI-validated.

### The Dependency Problem

**Without dependency firewall:**
```
UI includes kernel execution headers → direct execution bypass
AI includes syscall headers → validation bypass
BCIB includes driver pointers → isolation broken
Driver includes ABDF policy → separation of concerns violated
```

**With dependency firewall:**
```
architecture.manifest defines allowed dependencies
CI validates dependency graph
Forbidden dependencies → CI FAIL → merge BLOCKED
```

### Architecture Manifest

```yaml
# .kiro/governance/architecture.manifest

allowed_dependencies:
  kernel/sys:
    - vcp_runtime
    - fail_closed
    - evidence
    - execution_slot
    - abdf_canonical
  
  vcp_runtime:
    - evidence
    - fail_closed
    - execution_slot
    # MUST NOT depend on: UI, AI, driver semantics
  
  bcib:
    - vcp_runtime
    - abdf_canonical
    - runtime_bridge
    # MUST NOT depend on: driver implementation, device pointers
  
  abdf_canonical:
    - # No execution dependencies
    # MUST NOT depend on: execution policy, validation logic
  
  evidence:
    - vcp_runtime
    # MUST NOT depend on: UI, AI, driver

forbidden_patterns:
  - source: UI
    target: kernel/sys/execution
    reason: "UI cannot execute directly"
  
  - source: AI
    target: kernel/sys/execution
    reason: "AI cannot execute directly"
  
  - source: bcib
    target: driver/*
    reason: "BCIB cannot access drivers directly"
  
  - source: driver
    target: abdf/policy
    reason: "Driver cannot define ABDF policy"
  
  - source: vcp_runtime
    target: UI
    reason: "VCP cannot depend on UI semantics"
```

### CI Dependency Check

```bash
#!/bin/bash
# .ci/ci-dependency-check.sh

# Parse architecture.manifest
parse_manifest

# Build dependency graph from source code
build_dependency_graph() {
    for file in $(find kernel -name "*.c" -o -name "*.h"); do
        extract_includes $file
    done
}

# Detect circular dependencies
detect_cycles() {
    if has_cycle(dependency_graph); then
        echo "ERROR: Circular dependency detected"
        echo "  Module A depends on B"
        echo "  Module B depends on A"
        exit 1
    fi
}

# Detect forbidden dependencies
detect_forbidden() {
    for dep in $(get_all_dependencies); do
        if is_forbidden($dep, architecture.manifest); then
            echo "ERROR: Forbidden dependency detected"
            echo "  $dep.source → $dep.target"
            echo "  Reason: $dep.reason"
            exit 1
        fi
    done
}

# Run checks
build_dependency_graph
detect_cycles
detect_forbidden

echo "Dependency check PASSED"
```

### Property 49: Architecture Dependency Firewall

*For any* module dependency, the dependency SHALL be declared in architecture.manifest, circular dependencies SHALL be detected and rejected, and forbidden dependencies SHALL block CI.

**Validates: Requirements 19.2, 19.3, 19.4**

### Why This Matters

**Without dependency firewall:**
- Future features create circular dependencies
- UI/AI/driver bypass validation boundaries
- Architectural integrity degrades over time
- System becomes unmaintainable

**With dependency firewall:**
- Dependencies are explicit and validated
- Bypass paths are blocked at CI time
- Architecture remains clean as system grows
- Future extensions respect boundaries

## Device-Originated Data Boundary Contract

**Critical Principle**: Future device/driver integration MUST follow ABDF canonical contract. Device inputs MUST NOT bypass validation or break determinism.

### Device Event ABDF Contract

```c
// kernel/include/abdf_device.h

struct abdf_device_event {
    uint64_t event_type;        // INPUT / STATUS / ERROR
    uint64_t source_device_id;  // Device identifier (for audit)
    uint64_t logical_timestamp; // Monotonic counter (NOT wall clock)
    uint64_t capability_id;     // Required capability
    uint64_t event_data_size;
    uint8_t event_data[];       // Device-specific payload
};

struct abdf_input_event {
    uint64_t input_type;        // KEYBOARD / MOUSE / TOUCH / SENSOR
    uint64_t source_device_id;
    uint64_t logical_timestamp;
    uint64_t capability_id;
    union {
        struct keyboard_data { uint64_t key_code; uint64_t modifiers; };
        struct mouse_data { uint64_t x; uint64_t y; uint64_t buttons; };
        struct touch_data { uint64_t x; uint64_t y; uint64_t pressure; };
        struct sensor_data { uint64_t sensor_id; uint64_t reading; };
    } input_data;
};

struct abdf_device_status {
    uint64_t status_type;       // CONNECTED / DISCONNECTED / ERROR / READY
    uint64_t source_device_id;
    uint64_t logical_timestamp;
    uint64_t status_data_size;
    uint8_t status_data[];
};
```

### Device Integration Flow

```
Device Event (hardware)
    ↓
Driver (converts to ABDF DeviceEvent)
    ↓
ABDF Canonical Layer
    ↓
VCP Validation (capability check)
    ↓
Execution (if validation PASS)
    ↓
Evidence (device_id, event_type, capability_id)
```

### Property 50: Device-Originated Data Boundary

*For any* device-originated event, the event SHALL follow ABDF DeviceEvent contract, SHALL require capability, SHALL emit evidence, and SHALL NOT have direct device → execution bypass path.

**Validates: Requirements 20.1, 20.4, 20.5, 20.8**

### Why This Matters

**Without device boundary:**
- Future driver integration bypasses validation
- Device events break determinism (wall clock timestamps)
- Device-originated execution has no audit trail
- Device inputs create non-canonical execution paths

**With device boundary:**
- All device inputs follow canonical ABDF format
- Device events use logical timestamps (deterministic)
- Device execution requires capability and emits evidence
- Future driver integration respects validation authority

**This prepares for Phase 19+ driver integration without breaking Phase 18 authority foundation.**

## Performance Budget Contract

**Critical Principle**: Validation and evidence operations MUST have deterministic time bounds. Unbounded operations create unpredictable performance degradation and potential DoS vectors.

### Performance Budget Definition

```c
// kernel/include/vcp_performance.h

// CRITICAL: Use bounded operations (NOT cycle count) for determinism
#define VCP_VALIDATION_MAX_OPS       10      // max hash ops + signature verifications
#define EVIDENCE_APPEND_MAX_OPS      5       // max write ops + hash computations
#define SIGNATURE_VERIFY_MAX_OPS     1       // max signature operations
#define FAIL_CLOSED_MAX_OPS          3       // max operations in fail-closed path

struct performance_budget {
    uint64_t max_operations;  // Bounded operations (NOT cycles)
    uint64_t max_memory;
    uint64_t max_io_ops;
};
```

### Budget Enforcement

```c
int vcp_runtime_validate_with_budget(struct execution_slot *slot) {
    uint64_t operation_count = 0;
    
    // Perform validation (count operations, not cycles)
    int result = vcp_runtime_validate(slot, &operation_count);
    
    if (operation_count > VCP_VALIDATION_MAX_OPS) {
        vcp_fail_closed(slot, "Validation operation budget exceeded");
        vcp_emit_evidence(slot, "validation_budget_exceeded", operation_count);
        return VCP_FAIL_CLOSED;
    }
    
    return result;
}
```

### Fallback Behavior

```
IF validation exceeds operation budget:
  → fail-closed with evidence "validation operation budget exceeded"
  → system halts execution
  → no recovery (budget exceeded = critical failure)

IF evidence exceeds operation budget:
  → fail-closed with evidence "evidence operation budget exceeded"
  → system halts execution

IF signature exceeds operation budget:
  → fail-closed with evidence "signature operation budget exceeded"
  → system halts execution
```

### Property 51: Performance Budget Enforcement

*For any* enforcement path (validation, evidence, signature, fail-closed), the operation SHALL complete within deterministic operation bound (NOT cycle count), and operation budget exceeded SHALL trigger fail-closed with evidence.

**Validates: Requirements 21.1, 21.2, 21.3, 21.4, 21.5**

### Why This Matters

**Without performance budget:**
- Validation operations unpredictable (DoS vector)
- Evidence append can block indefinitely
- Signature verification unbounded
- System performance degrades unpredictably

**With performance budget:**
- All enforcement paths have deterministic operation bounds
- Operation budget exceeded = critical failure (fail-closed)
- Performance is predictable and testable (same ops → same behavior)
- DoS vectors are mitigated
- **CRITICAL**: Operation-based (NOT cycle-based) ensures determinism across different CPUs

**This ensures security mechanisms do not degrade system performance.**

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Execution Slot Validation State Initialization

*For any* execution context, when an execution slot is created, the slot SHALL contain initialized validation state from VCP.

**Validates: Requirements 1.1, 7.1**

### Property 2: Fail-Closed on Missing Validation State

*For any* execution attempt (execution slot, BCIB contract, CLI command, or boundary crossing), if validation state is missing, the system SHALL immediately block execution using the fail-closed mechanism.

**Validates: Requirements 1.2, 4.1, 5.3**

### Property 3: Invalid Validation State Blocks Execution

*For any* execution with invalid validation state, the runtime enforcement point (Runtime_Hook, BCIB_Enforcer, or ABDF_Validator) SHALL block execution and emit evidence describing the validation failure.

**Validates: Requirements 1.3, 2.3, 3.3**

### Property 4: Valid Validation State Permits Execution

*For any* execution slot with valid validation state, the execution SHALL proceed successfully without blocking.

**Validates: Requirements 1.5**

### Property 5: BCIB Contract Validation Enforcement

*For any* BCIB execution contract invocation, the BCIB_Enforcer SHALL verify validation state in the execution slot before allowing contract execution.

**Validates: Requirements 2.1, 2.2**

### Property 6: ABDF Boundary Validation Enforcement

*For any* authority boundary crossing, the ABDF_Validator SHALL check validation state in the execution slot before permitting the crossing.

**Validates: Requirements 3.1, 3.2**

### Property 7: Constitutional Boundary Policy Enforcement

*For any* Ring3 to Ring0 boundary crossing attempt, the ABDF_Validator SHALL block the crossing according to constitutional rules (SECURITY.BOUNDARY.VIOLATION).

**Validates: Requirements 3.5, 8.4**

### Property 8: Comprehensive Evidence Emission

*For any* validation check, execution block, BCIB contract execution, or ABDF boundary crossing, the Evidence_Emitter SHALL record the event with complete context including validation decision, enforcement action, and relevant metadata.

**Validates: Requirements 2.4, 3.4, 6.1, 6.2, 6.3, 6.4**

### Property 9: Fail-Closed Permanence

*For any* fail-closed condition, the system SHALL halt execution permanently and SHALL NOT allow continuation after the fail-closed state is reached.

**Validates: Requirements 4.2, 4.4**

### Property 10: Fail-Closed State Integrity

*For any* system state, when fail-closed is triggered, the system SHALL preserve state integrity without corruption or inconsistency.

**Validates: Requirements 4.5**

### Property 11: Fail-Closed Evidence Completeness

*For any* fail-closed condition, the Evidence_Emitter SHALL record complete failure context including the reason, location, and system state at the time of failure.

**Validates: Requirements 4.3, 6.6**

### Property 12: CLI Validation State Attachment

*For any* CLI command that initiates execution, the CLI_Handler SHALL attach validation state to the execution slot before execution begins.

**Validates: Requirements 5.1**

### Property 13: CLI Boundary Validation

*For any* CLI operation that crosses an authority boundary, the ABDF_Validator SHALL enforce validation according to boundary policies.

**Validates: Requirements 5.4**

### Property 14: CLI Evidence Emission

*For any* CLI execution attempt (successful or blocked), the CLI_Handler SHALL emit evidence recording the attempt and outcome.

**Validates: Requirements 5.5**

### Property 15: Execution Slot Validation State Preservation

*For any* active execution slot, while the slot remains active, the validation state SHALL remain unchanged and preserved (invariant property).

**Validates: Requirements 7.2**

### Property 16: Execution Slot Destruction Evidence

*For any* execution slot, when the slot is destroyed, the system SHALL emit final evidence recording the slot lifecycle completion.

**Validates: Requirements 7.3**

### Property 17: Validation State Immutability

*For any* execution slot with validation state, external attempts to modify the validation state SHALL be prevented and blocked.

**Validates: Requirements 7.4**

### Property 18: Nested Slot Independence

*For any* nested execution slot hierarchy, each slot SHALL maintain independent validation state that does not affect or depend on other slots in the hierarchy.

**Validates: Requirements 7.5**

### Property 19: Deterministic Execution (No Global State Mutation)

*For any* runtime hook execution, the hook SHALL NOT introduce global state mutations, maintaining deterministic behavior (DETERMINISM.GLOBAL constitutional rule).

**Validates: Requirements 8.2**

### Property 20: Capability Security Enforcement

*For any* operation requiring capabilities, the system SHALL enforce capability checks and SHALL NOT provide bypass mechanisms (KERNEL.CAPABILITY.BYPASS constitutional rule).

**Validates: Requirements 8.3**

### Property 21: Audit Trail Integrity

*For any* evidence emission, the Evidence_Emitter SHALL produce immutable audit trail entries that cannot be tampered with or modified (CONSTITUTIONAL.AUDIT.TAMPERING constitutional rule).

**Validates: Requirements 8.6**

### Property 22: Memory Safety in Validation Paths

*For any* validation enforcement path execution, the system SHALL NOT introduce memory leaks or memory safety violations.

**Validates: Requirements 9.2**

### Property 23: Error Handling Without Panic

*For any* validation state error or enforcement failure, the system SHALL handle the error gracefully without panicking.

**Validates: Requirements 9.3**

### Property 24: Validation State Trust Verification

*For any* validation state, before the state is used for enforcement decisions, the system SHALL verify capability binding, context hash, signature, and nonce uniqueness.

**Validates: Requirements 11.1, 11.2, 11.3, 11.4, 11.5**

### Property 25: Fake Validation State Rejection

*For any* validation state that fails capability binding verification, the system SHALL reject the state and trigger fail-closed enforcement.

**Validates: Requirements 11.1, 11.5**

### Property 26: Replayed Validation State Rejection

*For any* validation state with a context hash that does not match the current execution context OR a nonce that has been previously used, the system SHALL reject the state and trigger fail-closed enforcement.

**Validates: Requirements 11.2, 11.5**

### Property 27: Signature Verification Enforcement

*For any* validation state with an invalid signature, the system SHALL reject the state and trigger fail-closed enforcement.

**Validates: Requirements 11.3, 11.5**

### Property 28: Trust Verification Before Enforcement

*For any* enforcement decision, the system SHALL verify validation state trust (capability + context + signature + nonce) BEFORE checking the validation result flag.

**Validates: Requirements 11.4**

### Property 29: Deterministic Evidence Format

*For any* evidence entry, the same input SHALL produce an identical evidence entry with no nondeterministic fields.

**Validates: Requirements 12.2, 12.9, 13.1**

### Property 30: Evidence Append-Only Integrity

*For any* evidence chain (slot-local or global), no overwrite or deletion operations SHALL be allowed, and all append operations SHALL be strictly sequential.

**Validates: Requirements 12.1, 12.2**

### Property 31: Slot Chain Isolation

*For any* two execution slots A and B, Slot A SHALL NOT be able to modify Slot B's evidence chain, and slot chains SHALL remain independent.

**Validates: Requirements 12.7**

### Property 32: Global Anchor Integrity

*For any* slot chain anchor event in the global chain, the anchored slot head hash SHALL match the actual slot chain head hash, and anchor events SHALL be immutable and signed.

**Validates: Requirements 12.5, 12.6, 13.1**

### Property 33: Evidence Failure → Fail-Closed

*For any* evidence emission operation, if the write to an authoritative chain (slot-local or global) fails, the system SHALL immediately trigger fail-closed enforcement and SHALL NOT allow execution to proceed.

**Validates: Requirements 12.4, 12.8**

### Property 34: Evidence Signature Integrity

*For any* evidence entry, the entry SHALL include a valid signature and signer_id, and the signature SHALL verify against the VCP trust root.

**Validates: Requirements 13.1, 13.2**

### Property 35: Forged Evidence Rejection

*For any* evidence entry with an invalid signature OR missing signature (unsigned), the system SHALL reject the evidence and trigger fail-closed enforcement.

**Validates: Requirements 13.3, 13.4**

### Property 36: Verification Before Accept

*For any* evidence entry, signature verification SHALL occur BEFORE the evidence is added to any authoritative chain (slot-local or global).

**Validates: Requirements 13.3**

### Property 37: Deterministic Evidence Replay

*For any* execution, the same execution inputs SHALL produce an identical evidence chain, and the evidence chain SHALL be deterministically replayable by CI verification tools.

**Validates: Requirements 12.2, 12.9**

### Property 42: Canonical Determinism (ABDF)

*For any* three inputs A (JSON), B (CLI), C (AI) representing the same logical operation, the ABDF canonicalization SHALL produce identical ABDF representations: ABDF(A) == ABDF(B) == ABDF(C).

**Validates: Requirements 16.4, 16.5, 16.6, 16.7**

### Property 43: Extension Boundary Enforcement

*For any* external input source (AI, CLI, userland, semantic planner), the input SHALL go through ABDF canonicalization and VCP validation before execution, and SHALL NOT have any direct execution bypass path.

**Validates: Requirements 16.9, 16.10, 16.11**

---

## Signature

```
────────────────────────────────────────
Kenan AY
Architectural Steward — AykenOS

Document: AYKEN VCP Execution Binding - Design
Status: APPROVED (Authority Foundation Complete)
Scope: Runtime validation enforcement with trust, graph determinism, architecture firewall, device boundary, and performance budget

Date: 2026-05-03
────────────────────────────────────────
```

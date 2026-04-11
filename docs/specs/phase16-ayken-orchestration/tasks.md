# Ayken Orchestration Implementation Tasks

**Status:** Production-candidate; real QEMU/kernel runtime determinism pending
**Scope:** Minimal deterministic query pipeline
**Naming rule:** Long-lived implementation files, modules, binaries, and gates must use purpose-based names, not phase labels. The existing spec directory name is historical; new kernel-evidence surfaces must use names such as `kernel-runtime-equivalence`, `bcib-kernel-worker`, or `canonical-bcib-runtime-gate`.

## Enforcement Rules (Mandatory For All Tasks)

All tasks in this document must comply with the following enforcement rules.

### 1. Enforcement Authority

- Security-critical enforcement must be implemented at the authoritative boundary.
- Kernel/syscall boundary enforcement is authoritative for execution, resource, memory, and capability boundaries.
- Userspace enforcement is advisory unless a task explicitly defines it as the authoritative layer.
- Userspace-only enforcement is not sufficient for boundary or security rules.

### 2. Forbidden Implementation Patterns

The following implementations are strictly forbidden:

- String-based validation for syscall or execution control
- Pattern-based filtering such as `test_`, `debug_`, or `internal_`
- Disableable enforcement in production builds
- Userspace-only enforcement for boundary/security rules
- Fallback-to-success behavior after failed validation

### 3. Required Enforcement Mechanism

- Validation must occur at the authoritative boundary before execution continuation.
- Boundary validation must occur before resource allocation, including context, memory, handles, execution slots, and result mappings.
- Kernel-facing paths must validate concrete syscall IDs and ABI contracts, not names or inferred intent.
- Enforcement must be tied to the relevant `Execution_Context` or kernel execution slot lifecycle.

### 4. Fail-Closed Requirement

All violations must:

- Prevent execution continuation
- Prevent unauthorized resource allocation
- Return a deterministic error code or terminate the execution deterministically
- Produce auditable evidence when the task is part of a runtime/kernel gate

### 5. Evidence Requirement

- Boundary/security enforcement must be verifiable through runtime evidence.
- Kernel-level claims require QEMU/kernel trace evidence.
- Host-only tests may prove host harness behavior, but they cannot close real kernel determinism tasks.

## Tasks

- [x] 1. Freeze the minimal orchestration scope
  - [x] 1.1 Mark the supported production command surface as `list`, `show`, and `query ... where ...`
  - [x] 1.2 Mark mutation, AI, UI, debug, system, loop, and control-flow widening as explicitly unsupported in the first slice
  - [x] 1.3 Add source-level guards or tests proving unsupported commands fail closed instead of degrading to placeholder behavior
  - [x] 1.4 Rename implementation surfaces away from phase labels and toward purpose-based names
  - Reference: Requirements 1, 1A, 7

- [x] 2. Freeze the canonical IR contract
  - Enforcement Level: Kernel-level authoritative where the IR contract crosses execution, capability, or submission boundaries; userspace canonicalization is advisory until submitted through the authoritative boundary.
  - [x] 2.1 Ratify the existing execution-plan subset as the only minimal orchestration canonical IR
  - [x] 2.2 Freeze the `r0 = active context/result register` invariant
  - [x] 2.3 Ensure every successful canonical plan terminates in `Return`
  - [x] 2.4 Add validation rejecting hidden state or direct `DSL -> BCIB` production shortcuts
  - Forbidden: hidden state, direct `DSL -> BCIB` production shortcuts, string/pattern-based execution control, userspace-only authority claims
  - Reference: Requirements 2, 3

- [x] 3. Implement `DSL -> Canonical IR` for the supported surface
  - Enforcement Level: Kernel-level authoritative for execution entry; userspace parsing/lowering is not sufficient for boundary enforcement.
  - Required: only the approved submit path may create or target an execution; validation must occur before runtime/context/resource allocation.
  - Forbidden: string-based syscall validation, pattern-based entry filtering, userspace-only enforcement, disableable production enforcement.
  - [x] 3.1 Lower `list <context>` to `LoadContext -> Return`
  - [x] 3.2 Lower `show <context> <id>` to `LoadContext -> LoadLiteral/Compare -> ApplyFilter -> Return`
  - [x] 3.3 Lower `query <context> where <predicate>` to `LoadContext -> predicate lowering -> ApplyFilter -> Return`
  - [x] 3.4 Reject unsupported predicate forms with explicit lowering errors
  - Evidence Required: invalid entry attempts must reject before context allocation; kernel/runtime trace must show no execution context or slot created for invalid entry when this crosses the kernel boundary.
  - Reference: Requirements 1, 2, 7

- [x] 4. Add canonical IR validation
  - [x] 4.1 Reject empty contexts
  - [x] 4.2 Reject missing or invalid terminal `Return`
  - [x] 4.3 Reject IR nodes outside the frozen minimal orchestration subset
  - [x] 4.4 Emit deterministic plan identity/fingerprint for proof/replay binding
  - [x] 4.5 Emit a deterministic Canonical Query Binding for `list`, `show`, and `query`
  - Reference: Requirements 2, 5, 9, 10

- [x] 5. Implement NOP-free `Canonical IR -> BCIB` lowering
  - Enforcement Level: Kernel boundary remains authoritative; lowering and runtime bridge behavior are mediation only, not authority.
  - Critical Role: lowering and bridge layers may translate/package validated intent and bind capabilities, but must not expose kernel APIs, act as privileged userspace, or initiate execution outside the approved submit path.
  - Forbidden: direct kernel API exposure, arbitrary syscall proxying, embedding kernel logic in bridge/lowering layers, treating bridge/lowering as execution authority, bridge-initiated `SYS_V2_SUBMIT_EXECUTION`.
  - Required: all kernel interaction must occur via the approved syscall layer; capability validation must occur before any action; no new execution paths may be introduced.
  - [x] 5.1 Emit only `DataQuery`, `End`, and optional `TraceEmit`
  - [x] 5.2 Reject any lowering path that would require `Nop`
  - [x] 5.3 Reject any lowering path that would require `DataCreate`, `DataAdd`, `UiRender`, or `AiAsk`
  - [x] 5.4 Bind operands deterministically so the same canonical plan yields the same lowered BCIB
  - [x] 5.5 Preserve the semantic distinction between `list`, `show`, and `query` in lowering metadata
  - Evidence Required: tests must prove forbidden opcodes fail, bridge/lowering cannot initiate execution, and no unauthorized syscall path appears in runtime/kernel evidence.
  - Reference: Requirements 3, 4, 5, 9, 10

- [x] 6. Replace placeholder orchestration/submission surfaces
  - Enforcement Level: Sandbox/submission enforcement must be authoritative at kernel/syscall/resource boundaries; userspace checks alone are not sufficient.
  - Critical Role: orchestration/submission may reject or package work; it may not reinterpret semantic intent, elevate privileges, expand syscall surface, or bypass kernel boundary enforcement.
  - Sandbox Requirement: BCIB execution must remain restricted to approved bounded memory regions, assigned `Execution_Context`, declared output buffers, and approved ABDF/runtime-bridge paths.
  - Forbidden: raw pointer trust, caller-provided address trust, string-based isolation checks, pattern-based sandbox escape detection, userspace-only memory boundary enforcement, lazy boundary checks after execution start, production sandbox disablement.
  - Required: memory bounds, input buffers, output buffers, capability sets, and cross-context access must be validated before resource touch or execution continuation.
  - [x] 6.1 Replace the `userspace/orchestration` placeholder with a real submit-only router
  - [x] 6.2 Replace placeholder submission-bridge behavior with a real submit adapter
    - Forbidden Shortcuts: reusing caller memory without authoritative validation, treating userspace references as trusted, allowing lazy boundary checks after execution start
  - [x] 6.3 Implement non-empty, fail-closed capability/submission validation
  - [x] 6.4 Keep direct kernel-facing submit calls out of semantic parsing/lowering layers
  - [x] 6.5 Enforce explicit, non-empty context-read capability validation for the supported query surface
  - Evidence Required: forbidden syscall attempts, missing capabilities, out-of-bounds access, raw pointer attempts, kernel-space pointer visibility attempts, and cross-context access attempts must fail closed; QEMU/runtime evidence must show no continued execution after violation for kernel-level claims.
  - Reference: Requirements 6, 7, 8, 9

- [x] 7. Close the orchestration proof/replay slice
  - [x] 7.1 Produce one canonical end-to-end artifact chain: command string -> IR fingerprint -> Canonical Query Binding fingerprint -> BCIB SHA-256 -> submission result
  - [x] 7.2 Bind proof material to the lowered BCIB identity
  - [x] 7.3 Add replay verification for the same canonical BCIB input
  - [x] 7.4 Fail closed on replay deviation
  - Reference: Requirement 10

- [x] 8. Add regression guards
  - [x] 8.1 Add tests proving the production path never emits `Nop`
  - [x] 8.2 Add tests proving unsupported commands/predicates never degrade to placeholder success
  - [x] 8.3 Add tests proving the production path does not bypass orchestration with direct executor calls
  - [x] 8.4 Add one end-to-end deterministic query scenario for `list`, `show`, and `query`
  - [x] 8.5 Add tests proving orchestration rejects or packages work without reinterpreting semantic intent
  - Reference: Requirements 3, 4, 6, 7, 8, 9, 10

- [x] 9. Keep docs synchronized while implementation lands
  - [x] 9.1 Update `README.md` or current truth surfaces only when the production path actually changes
  - [x] 9.2 Update this checklist in the same change set as code
  - [x] 9.3 Record completed slices and validation in a follow-up progress note when implementation starts
  - [x] 9.4 Keep code-facing names stable if the phase/spec label changes later
  - Reference: All requirements

- [ ] 10. CRITICAL: Prove kernel runtime determinism (BLOCKING PRODUCTION)
  - [x] 10.1 Implement host runtime equivalence test: runtime_result == replay_result for production canonical BCIB v3 bytes
  - [x] 10.2 Verify same BCIB → same host runtime/executor harness result (no drift)
  - [x] 10.3 Verify submission result fingerprint consistency in proof binding
  - [ ] 10.4 Connect replay verifier to real QEMU/kernel runtime
  - [ ] 10.4.1 Add a purpose-named Ring3 execution worker payload that reads the kernel execution inbox, consumes canonical BCIB v3 bytes from the payload window, writes a deterministic execution-output ABI record, and calls `SYS_V2_COMPLETE_EXECUTION`
  - [ ] 10.4.2 Add a canonical BCIB v3 fixture source for the kernel-runtime gate, generated from the production `DSL -> Canonical IR -> BCIB` path and checked against its BCIB SHA-256
  - [ ] 10.4.3 Add a QEMU evidence gate that boots the worker payload, submits the canonical BCIB through the real `SYS_V2_SUBMIT_EXECUTION` path, waits through the real `SYS_V2_WAIT_RESULT` path, and captures debugcon/serial evidence
  - [ ] 10.4.4 Capture the kernel result mapping and hash sidecar from the real wait-result path; host-only executor harness output is not acceptable evidence for this item
  - [ ] 10.5 Prove execution determinism with real kernel result, not host harness only
  - [ ] 10.5.1 Compare real kernel result fingerprint against the replay/proof fingerprint for the same canonical BCIB bytes
  - [ ] 10.5.2 Run the same canonical BCIB at least twice under QEMU and prove the kernel result fingerprint does not drift
  - [ ] 10.5.3 Fail closed if QEMU evidence is missing, if only host-harness evidence is present, or if any kernel result fingerprint differs
  - [ ] 10.5.4 Update status and roadmap only after real QEMU/kernel evidence exists; do not mark production-ready from host/runtime harness tests alone
  - Reference: DETERMINISM.GLOBAL constitutional requirement

## Current Status

**Pipeline Determinism:** ✅ PROVEN (90% complete)
- DSL → Canonical IR → BCIB → Proof chain is deterministic
- 8/8 E2E closure tests passing
- Cryptographic proof of pipeline determinism

**Host Runtime / Executor Harness:** ✅ PROVEN
- Production canonical BCIB v3 bytes are accepted by `BcibGraph` / `BcibExecutor`
- Host BCIB runtime completes the canonical query graph
- 5/5 runtime equivalence tests passing, 0 ignored

**Critical Distinction:**
- We can prove: "Same input produces same BCIB"
- We can prove: "Same BCIB produces the same host runtime/executor harness result"
- We cannot yet prove: "Same BCIB produces the same real QEMU/kernel runtime result"

This is the difference between a production-candidate and production-ready system.

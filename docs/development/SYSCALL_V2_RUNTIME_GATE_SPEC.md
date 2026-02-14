# SYSCALL_V2 Runtime Gate Spec
This document is subordinate to PHASE 0 - FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

**Status:** CONTRACT-FIRST (Planned, not wired)  
**Owner:** AykenOS Core Architecture Team  
**Scope:** Freeze-mode runtime contract verification for Ring3 -> syscall_v2 -> Ring0 path  
**Gate ID (planned):** `ci-gate-syscall-v2-runtime`

---

## 1) Purpose

Static ABI lock alone is insufficient for freeze-grade enforcement.  
This gate verifies runtime syscall contract integrity without introducing policy logic into Ring0.

The gate proves:
1. Ring3 can issue v2 syscall calls on the expected execution path.
2. Ring0 dispatcher accepts valid calls and returns deterministic results.
3. Capability bind/revoke enforcement path is alive and contract-compliant.
4. Runtime regressions are evidence-backed and merge-blocking.

---

## 2) Non-Goals

This gate is not a functional product test and not a scheduler policy test.
1. It does not benchmark performance.
2. It does not validate business logic in userspace applications.
3. It does not test execution-graph semantics.
4. It does not allow fallback/waiver in strict freeze mode.

---

## 3) Contract Surface (Minimum Runtime Smoke Set)

The gate MUST validate the following syscalls at runtime:
1. `SYS_V2_DEBUG_PUTCHAR` (heartbeat liveness)
2. `SYS_V2_TIME_QUERY` (deterministic return path)
3. `SYS_V2_CAPABILITY_BIND` (valid bind path + rejection path)
4. `SYS_V2_CAPABILITY_REVOKE` (valid revoke path + rejection path)

Expected contract assertions:
1. Valid invocation returns expected success code.
2. Invalid invocation returns contract-compliant error code.
3. Dispatch path emits trace evidence for call/return.
4. No panic/triple-fault/hard hang under configured timeout.

---

## 4) Determinism Profile

Freeze-mode defaults:
1. `warmup_runs = 1`
2. `measurement_runs = 5`
3. `per_run_timeout_seconds = 8`
4. `required_success_rate = 100%`

Rules:
1. Any failed measurement run => gate `FAIL`.
2. Timeout in any measurement run => gate `FAIL`.
3. Partial success is not accepted in strict freeze mode.

---

## 5) Invocation Contract (Planned)

### 5.1 Make Target
Planned target:
1. `make ci-gate-syscall-v2-runtime`

### 5.2 Gate Script
Planned script:
1. `scripts/ci/gate_syscall_v2_runtime.sh`

### 5.3 Evidence Path
1. `evidence/run-<RUN_ID>/gates/syscall-v2-runtime/`
2. `evidence/run-<RUN_ID>/reports/syscall-v2-runtime.json`

### 5.4 CI Placement (Planned)
Recommended order in `ci-freeze`:
1. ABI
2. Boundary
3. Hygiene
4. Tooling Isolation
5. Constitutional
6. Workspace
7. Syscall v2 Runtime (this gate)
8. Performance
9. Summarize

Rationale: runtime contract is above static contract checks and below performance comparison.

---

## 6) Evidence Schema

Required files:
1. `report.json`
2. `meta.txt`
3. `violations.txt`
4. `trace.log`

`meta.txt` MUST include:
1. `run_id`
2. `git_sha`
3. `kernel_profile`
4. `warmup_runs`
5. `measurement_runs`
6. `timeout_seconds`
7. `success_rate_required`
8. `success_rate_actual`
9. `time_utc`

`report.json` MUST include:
1. `gate`
2. `verdict`
3. `violations_count`
4. `meta`
5. `results` per syscall
6. `violations`

---

## 7) Violation Taxonomy

Canonical violation keys:
1. `syscall_runtime_missing:<name>`
2. `syscall_runtime_timeout:<name>`
3. `syscall_runtime_unexpected_rc:<name>:expected=<x>:actual=<y>`
4. `syscall_runtime_trace_missing:<name>`
5. `syscall_runtime_dispatch_missing:<name>`
6. `syscall_runtime_success_rate_below_threshold:<actual>/<required>`
7. `syscall_runtime_harness_failed:<detail>`

Unknown violations are treated as `FAIL`.

---

## 8) Exit Codes

1. `0`: pass
2. `2`: contract/runtime violations
3. `3`: usage/tooling/infrastructure error

---

## 9) Freeze Linkage

This gate closes the runtime verification gap between:
1. static ABI contract enforcement (`ci-gate-abi`)
2. runtime execution path integrity (this gate)

After wiring, freeze claims about syscall_v2 MUST require this gate evidence.


# AykenOS Syscall Transition Guide
This document is subordinate to PHASE 0 - FOUNDATIONAL OATH. In case of
conflict, Phase 0 prevails.

**Status:** Current ABI and migration-intent guide, not runtime proof
**Updated:** 2026-03-19

## Overview

AykenOS has completed the syscall numbering and ABI transition from legacy
POSIX-like syscalls to the v2 execution-centric range.

That does **not** mean every v2 syscall already provides full runtime
semantics. The ABI transition is complete; runtime lifecycle implementation is
still in progress.

Use this document for:

- syscall numbering truth
- ABI-stable argument meaning
- migration intent
- current maturity map

Do **not** use this document as the only source for runtime execution behavior.
For current kernel behavior, see:

- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/requirements.md`

## Current ABI Truth

### Numbering

The active v2 syscall range is `1000-1011`.

```c
#define SYS_V2_MAP_MEMORY        1000
#define SYS_V2_UNMAP_MEMORY      1001
#define SYS_V2_SWITCH_CONTEXT    1002
#define SYS_V2_SUBMIT_EXECUTION  1003
#define SYS_V2_WAIT_RESULT       1004
#define SYS_V2_INTERRUPT_RETURN  1005
#define SYS_V2_TIME_QUERY        1006
#define SYS_V2_CAPABILITY_BIND   1007
#define SYS_V2_CAPABILITY_REVOKE 1008
#define SYS_V2_EXIT              1009
#define SYS_V2_DEBUG_PUTCHAR     1010
#define SYS_V2_COMPLETE_EXECUTION 1011
```

Current truth:

- `SYS_V2_BASE = 1000`
- `SYS_V2_LAST = 1011`
- `SYS_V2_NR = 12`
- Ring0 currently exposes 12 execution-centric syscalls total
- `SYS_V2_COMPLETE_EXECUTION` is part of that count

### Legacy Status

The legacy v1 POSIX-like syscall surface is no longer the current kernel
direction. This guide treats v2 as the active interface.

### Frozen Time Query Contract

`sys_v2_time_query(query_type, out)` is now ABI-stable with:

```c
#define TIME_QUERY_MONOTONIC 0  // raw PIT ticks
#define TIME_QUERY_UPTIME    1  // uptime milliseconds
```

Current semantics:

- `TIME_QUERY_MONOTONIC` returns raw monotonic PIT ticks
- `TIME_QUERY_UPTIME` returns milliseconds derived from PIT ticks
- unknown `query_type` values fail closed

## Current Maturity Map

The syscall ABI is stable as a numbering and calling contract, but runtime
maturity is mixed.

### Stable or More Mature Surfaces

| Syscall | Status | Notes |
|---|---|---|
| `map_memory` | operational | real explicit single-page user mapping with capability validation and process-local ledger recording |
| `unmap_memory` | operational | real caller-owned span unmap with capability-bound ledger validation |
| `switch_context` | more mature | real process/context switching path exists |
| `capability_bind` | more mature | backed by capability manager |
| `capability_revoke` | more mature | backed by capability manager |
| `debug_putchar` | operational | real debug heartbeat surface |
| `time_query` | operational | now PIT-backed, no longer dummy |
| `complete_execution` | operational | explicit latch-bound completion surface with deterministic return codes |

### ABI-Stable but Semantically Incomplete Surfaces

| Syscall | Current Reality |
|---|---|
| `submit_execution` | allocates a kernel-owned execution ID, validates a live user target context, copies bounded BCIB bytes into kernel-owned backing, creates a `READY` slot, enqueues the target context, and can be picked up into `RUNNING` on schedule entry; user processes now have dedicated execution inbox/payload/output mappings, explicit completion can close the slot, and successful waits can materialize the full bounded frozen validated output header plus payload carried by the completed slot |
| `wait_result` | validates ownership and current slot state, finite waits can observe completion/failure/timeout, first successful waits map a read-only/NX full bounded result window plus a deterministic read-only/NX SHA-256 hash sidecar, repeated successful waits replay the same result VA and hash VA, and that result window publishes frozen validated output bytes produced through either the raw v1 or structured v2 header path |
| `complete_execution` | validates latch ownership, enforces first-terminal-state-wins, fail-closes malformed or out-of-bounds output, accepts either the raw v1 header or the additive structured v2 header while keeping payload bytes opaque to the kernel, and freezes one SHA-256 digest over the exact published bytes |
| `interrupt_return` | placeholder success path |
| `exit` | now performs zombie transition, owned/targeted non-terminal slot abort, result/delivery/mailbox revoke, generic explicit-mapping revoke, lower-half user-memory destruction, and no-return switch-away for non-owner processes; active root-PML4/current-`rsp0` reap is deferred to a later safe scheduler path, and scheduler-owner exit is fail-closed until handoff exists |

This is the most important truth boundary in the current repo:

- ABI transition: complete enough to code against the v2 interface
- runtime execution lifecycle: not complete enough to claim full operation

## Migration Guidance

### Safe Assumptions

Applications and runtime work may safely assume:

- the v2 numbering plan is frozen at `1000-1011`
- the interface contains 12 syscalls total
- `map_memory` / `unmap_memory` now implement real explicit generic mapping lifecycle semantics
- `submit_execution(..., context_id)` uses `context_id` as the third argument
- `time_query` has a real two-mode contract
- `complete_execution(execution_id, completion_code)` is the explicit lifecycle closure surface

## Critical Warning

Do not build higher-level runtime logic on top of the following syscall
surfaces until execution lifecycle stabilization is complete:

- `submit_execution`
- `wait_result`
- `exit`

### Unsafe Assumptions

Do not assume the following are already true in current runtime:

- `submit_execution` creates a fully connected execution lifecycle
- `wait_result` blocks until successful completion delivery or owns mapped results
- `interrupt_return` closes a real interrupt ownership path
- `exit` can currently retire the scheduler owner through the normal no-return path
- `exit` can retire the active scheduler owner without a prior ratified owner-transfer commit

### Example: Safe Current v2 Usage

```c
uint64_t ticks = 0;
uint64_t uptime_ms = 0;

syscall(SYS_V2_TIME_QUERY, TIME_QUERY_MONOTONIC, &ticks);
syscall(SYS_V2_TIME_QUERY, TIME_QUERY_UPTIME, &uptime_ms);
```

### Example: Target Model, Not Current Operational Proof

The following categories remain target-model examples rather than proof of
today's runtime semantics:

- execution queue lifecycle via `submit_execution` and `wait_result`
- cleanup-oriented process termination via `exit`

Those flows should be treated as migration intent until the execution-path
stabilization work lands.

## Testing Interpretation

Current syscall tests are not all equal in meaning.

- ABI/range tests prove dispatcher and numbering truth
- `time_query` now has real semantic checks
- timeout-driven `wait_result` wakeup now has bounded kernel-state checks
- several legacy validation checks still exercise interface shape more than full
  runtime semantics

If a v2 syscall passes an interface test, that does **not** automatically mean
its full execution behavior is implemented.

## References

- `docs/development/SYSCALL_RUNTIME_REALITY.md`
- `docs/specs/phase10b-execution-path-hardening/requirements.md`
- `docs/specs/phase10b-execution-path-hardening/design.md`
- `docs/specs/phase10b-execution-path-hardening/tasks.md`

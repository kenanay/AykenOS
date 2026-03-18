# AykenOS Syscall Runtime Reality
This document is subordinate to PHASE 0 - FOUNDATIONAL OATH. In case of
conflict, Phase 0 prevails.

**Status:** Current kernel behavior map
**Updated:** 2026-03-19

## Purpose

This file describes what the current kernel actually does at runtime for each
v2 syscall surface.

Use this document for:

- runtime truth
- maturity classification
- distinguishing semantic tests from interface-only checks

This file does not replace the ABI guide. For numbering and migration intent,
see `docs/development/SYSCALL_TRANSITION_GUIDE.md`.

## Runtime State Summary

The repository has completed the v2 ABI transition, but not the full execution
lifecycle implementation.

Current high-level state:

- numbering and dispatch range are frozen
- `time_query` is real
- capability operations are comparatively mature
- execution lifecycle remains incomplete

## Maturity Matrix

| Syscall | ABI Status | Runtime Status | Notes |
|---|---|---|---|
| `map_memory` | stable | incomplete | input and capability checks exist, real mapping does not |
| `unmap_memory` | stable | incomplete | no real unmap lifecycle yet |
| `switch_context` | stable | more mature | real process/context switch path exists |
| `submit_execution` | stable | incomplete | creates a `READY` execution slot and queue entry; schedule-entry pickup can move queued work to `RUNNING`, but delivery/completion is not yet wired and current workers latch a single active execution at a time |
| `wait_result` | stable | incomplete | validates ownership/state and reports nonterminal work as busy, but still has no real block/wake/result ownership |
| `interrupt_return` | stable | incomplete | placeholder handler |
| `time_query` | stable | operational | PIT-backed monotonic ticks and uptime milliseconds |
| `capability_bind` | stable | more mature | capability manager-backed |
| `capability_revoke` | stable | more mature | capability manager-backed |
| `exit` | stable | incomplete | not full teardown, revoke, and scheduler removal yet |
| `debug_putchar` | stable | operational | real debug heartbeat path |

## Current Truth by Area

### ABI Truth

These statements are currently safe:

- the v2 range is `1000-1010`
- the surface contains 11 syscalls total
- `SYS_V2_DEBUG_PUTCHAR` is included in that count
- `TIME_QUERY_MONOTONIC = 0`
- `TIME_QUERY_UPTIME = 1`

### Execution Reality Truth

These statements are currently safe:

- there is now an `execution_slot` data model in kernel space
- `submit_execution()` now anchors kernel-owned `READY` slots into that model
- schedule-entry worker pickup can advance queued work to `RUNNING`
- the current pickup path allows only one active execution per user process until completion or exit plumbing lands
- `wait_result` now reflects slot state instead of returning unconditional success
- blocking wait, timeout progression, completion, and `exit` are not yet a
  fully connected lifecycle

### Time Truth

These statements are currently safe:

- monotonic time comes from PIT-backed `tick_count`
- `sys_v2_time_query()` is no longer a dummy syscall
- timeout authority is specified to belong to the timer IRQ path
- timer IRQ deadline progression is still not wired

## Explicit Non-Guarantees

The current kernel does **not** guarantee:

- execution completion
- blocking semantics
- timeout enforcement
- memory mapping side effects
- process teardown correctness

## Test Meaning

Current test signals need careful interpretation.

### Mostly ABI or Interface Shape

- syscall count / range validation
- placeholder-success validation for incomplete syscalls

These prove:

- numbering is correct
- dispatch is reachable
- handlers return expected status shapes

These do **not** prove:

- lifecycle correctness
- blocking semantics
- timeout semantics
- page-table side effects
- teardown correctness

### More Semantic Today

- `time_query` monotonic nondecreasing checks
- capability bind/revoke behavior
- switch-context error handling

## Recommended Read Order

To understand current status without mixing target architecture and current
behavior, read in this order:

1. `docs/development/SYSCALL_TRANSITION_GUIDE.md`
2. `docs/development/SYSCALL_RUNTIME_REALITY.md`
3. `docs/specs/phase10b-execution-path-hardening/requirements.md`
4. `docs/specs/phase10b-execution-path-hardening/tasks.md`

## Next Runtime Priorities

The next implementation order should remain:

1. move `wait_result()` to real block/wake semantics
2. wire timeout progression in timer IRQ context
3. implement real `exit()` teardown and wake/cleanup
4. leave `map_memory` / `unmap_memory` for the later explicit mapping slice

# 20260214 - Scheduler Arbitration Contract

## Context
AykenOS freeze mode enforces "Yol A": Ring3 proposes scheduling candidates and
Ring0 remains final arbiter for execution safety.

## Decision
- Ring3 submits scheduling hints over the bridge/mailbox surface.
- Ring0 validates candidate acceptability before context switch.
- In strict mode, fallback policy in Ring0 is disabled by default
  (`AYKEN_SCHED_FALLBACK=0`).
- If scheduler is armed and no acceptable candidate exists, system behavior is
  fail-closed.

## Consequences
- Ring0 keeps mechanism and enforcement ownership.
- Ring3 keeps policy ownership.
- Constitutional CI checks must verify this document exists and remains tracked.

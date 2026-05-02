# Phase10-C: Scheduler Mailbox Separation

## Goal

Phase10-C establishes a hard policy/mechanism split for scheduler dispatch.

- Ring3 policy produces the dispatch decision.
- Ring0 mechanism only validates and applies that decision.
- Ring0 fallback selection is forbidden.
- Missing or malformed decision fails closed.

## Invariants

1. Ring0 must not decide `next_pid`.
2. Ring0 must not use `ready_head` as an implicit fallback selector.
3. Dispatch apply requires a mailbox decision with `valid=1`.
4. After successful apply, mailbox `valid` must be consumed to `0`.
5. CI must fail when decision/order/fallback invariants drift.

## Data Contract

```c
typedef struct {
    uint32_t valid;       // 0 or 1
    uint32_t decision_id; // monotonically increasing
    uint32_t next_pid;    // selected runnable process
    uint32_t flags;       // reserved for future policy metadata
} sched_decision_t;
```

## Ring3 Responsibilities (Policy)

- Compute `next_pid` deterministically.
- Publish decision atomically:
  - write `next_pid`
  - increment `decision_id`
  - set `valid=1`
- Avoid reliance on Ring0 fallback behavior.

## Ring0 Responsibilities (Mechanism)

- Read mailbox decision.
- Fail closed if mailbox missing or `valid != 1`.
- Resolve `next_pid` to runnable process.
- Apply context switch.
- Consume decision (`valid=0`) on successful apply.
- Never call fallback selector in dispatch path.

## Forbidden Mechanism Behaviors

- `ready_head`-driven dispatch in Ring0 mechanism path.
- Implicit RR/FIFO selection when mailbox is absent.
- Silent default to idle path without explicit Ring3 decision.

## Runtime Marker Contract

Required markers:

- `P10_SCHED_DISPATCH`
- `P10_MAILBOX_DECISION`
- `P10_DECISION_APPLIED`
- `P10_RING3_USER_CODE`

Required order:

- `P10_SCHED_DISPATCH -> P10_MAILBOX_DECISION -> P10_DECISION_APPLIED -> P10_RING3_USER_CODE`

Required metadata format (Phase10-C gate default):

- `P10_MAILBOX_DECISION id=<n> pid=<pid> valid=1`
- `P10_DECISION_APPLIED id=<n> pid=<pid> valid=0`

Forbidden markers:

- `P10_SCHED_FALLBACK`
- `P10_READY_HEAD_FALLBACK`

## CI Enforcement

`ci-gate-scheduler-mailbox-phase10c` consumes Phase10-A2 evidence (`events.jsonl`, `marker.log`) and must fail closed on:

- missing required markers,
- duplicate markers in V1,
- ordering violations,
- fallback markers,
- events/log mismatch,
- malformed or missing required metadata,
- `decision_id` mismatch or non-monotonicity,
- `valid` contract violations (decision must be `1`, applied must be `0`).

## Acceptance Criteria

1. Ring0 dispatch path applies mailbox decision only.
2. Missing decision path deterministically fails.
3. Fallback behavior is explicit violation, not degraded success path.
4. Gate report provides evidence-backed violation reasons.
5. Freeze integration can be enabled only after runtime path emits required Phase10-C markers.

## Transition Policy

Phase10-C is the boundary between transitional scheduler bridge behavior and enforceable policy separation. Prior phases are not interpreted as full separation proof.

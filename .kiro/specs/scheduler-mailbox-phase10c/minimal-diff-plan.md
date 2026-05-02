# Phase10-C Minimal Diff Plan

## Objective

Deliver enforceable Ring3 policy / Ring0 mechanism separation for scheduler dispatch with fail-closed runtime evidence.

## Scope of This Plan

- Add Phase10-C spec and CI gate draft.
- Add validator that consumes Phase10-A2 artifacts (`events.jsonl`, `marker.log`).
- Keep freeze chain unchanged until runtime marker path is implemented.

## Step 1: Data Contract and Marker Plumbing

Files to touch:

- `kernel/include/...` (new mailbox contract header)
- `kernel/sched/...` (mechanism apply path)
- `kernel/proc/...` (pid lookup helper if needed)
- `kernel/arch/x86_64/...` or scheduler emit path (Phase10-C markers)

Required behavior:

- Introduce `sched_decision_t`.
- Require `valid=1` before apply.
- Clear `valid=0` after successful apply.
- Emit:
  - `P10_MAILBOX_DECISION id=<n> pid=<pid> valid=1`
  - `P10_DECISION_APPLIED id=<n> pid=<pid> valid=0`

## Step 2: Remove Ring0 Fallback Selection

Files to touch:

- `kernel/sched/sched.c`

Required behavior:

- Replace `sched_select_next()` use in dispatch path with mailbox apply path.
- Panic/fail on missing decision (`SCHED_NO_DECISION`).
- Panic/fail on bad pid (`SCHED_INVALID_PID`).
- No `ready_head` fallback in mechanism path.

## Step 3: Add CI Gate

Files:

- `scripts/ci/gate_scheduler_mailbox_phase10c.sh`
- `tools/ci/validate_scheduler_mailbox_phase10c.py`
- `tools/ci/test_validate_scheduler_mailbox_phase10c.py`

Gate contract:

- Consume Phase10-A2 evidence directory.
- Fail on missing required markers, ordering drift, events/log mismatch.
- Fail on fallback markers.
- Fail on malformed metadata by default.

## Step 4: Makefile Wiring (Draft)

Files:

- `Makefile`

Changes:

- Add `ci-gate-scheduler-mailbox-phase10c` target.
- Keep target out of `ci-freeze` until runtime marker implementation lands.
- Add help and evidence directory wiring.

## Step 5: Promotion Criteria (for freeze inclusion)

Enable freeze integration only when all are true:

1. Runtime path emits required Phase10-C markers in stable order.
2. No fallback marker is observed in repeated runs.
3. Gate passes deterministically on at least two consecutive runs.
4. Validator tests pass in CI.


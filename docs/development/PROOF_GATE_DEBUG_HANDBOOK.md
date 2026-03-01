# Proof & Gate Debug Handbook (Gate-4 / Gate-4.5)

Status: ACTIVE  
Audience: CI maintainers and kernel contributors

This handbook explains how to debug proof gate failures quickly and consistently.

## 1. Which Gate Proves What

Gate-4 (`scripts/ci/gate_4_policy_accept.sh`):

1. publish -> accept integrity
2. accept-count invariants
3. owner-bound proof path

Gate-4.5 (`scripts/ci/gate_4_5_decision_switch_proof.sh`):

1. publish -> accept -> arbiter -> switch ordering
2. endpoint consistency (`from/to`)
3. non-noop decision and switch

## 2. Fast Triage Order

1. open gate report JSON
2. inspect `violations.txt`
3. inspect gate debugcon tail
4. verify marker counts/line ordering

Core evidence files:

1. `evidence/gate-4-policy-accept/<run_id>/report.json`
2. `evidence/gate-4-policy-accept/<run_id>/debugcon.log`
3. `evidence/gate-4.5-decision-switch-proof/<run_id>/report.json`
4. `evidence/gate-4.5-decision-switch-proof/<run_id>/gate45.log`

## 3. Gate-4 Invariants

Selftest disabled (`GATE4_MB_SELFTEST=0`):

1. `target_accept_count == 1`
2. `total_accept_count == 1`
3. `ring3_publish_line < target_accept_line`

Strict bootstrap (`GATE4_BOOTSTRAP_POLICY=0`):

1. preload marker expected

Transitional bootstrap (`=1`, Gate-4 mode):

1. preload marker must remain absent

Deterministic mode (`AYKEN_DETERMINISTIC_EXIT=1`):

1. `proof_done_count >= 1`
2. `qemu_exit_code` in `{0, 1}`

## 4. Gate-4.5 Invariants

1. `target_accept_count == 1`
2. `arbiter_count == 1`
3. `switch_count == 1`
4. ordering: publish < accept < arbiter < switch
5. arbiter endpoints == switch endpoints
6. no noop: `from != to`
7. arbiter epoch == 1

## 5. Typical Violations and Root Causes

`gate4_pid_missing`:

1. gate4 process marker not emitted
2. wrong workload/profile compiled

`target_accept_mismatch`:

1. owner publish rejected
2. stale epoch (already consumed)
3. candidate not runnable

`arbiter_decision_mismatch` / `ctx_switch_mismatch`:

1. runtime produced keep-running path only
2. second runnable process not present
3. decision marker gated incorrectly

`marker_order_invalid`:

1. marker emitted at wrong boundary
2. decision marker emitted before accept

`qemu_deterministic_exit_mismatch`:

1. proof completion not reached
2. deterministic exit path not active

## 6. Minimal Reproduction Commands

Gate-4:

```sh
RUN_ID=local-g4-$(date -u +%Y%m%dT%H%M%SZ) \
AYKEN_GATE45_PROOF=0 \
AYKEN_DETERMINISTIC_EXIT=1 \
bash scripts/ci/gate_4_policy_accept.sh
```

Gate-4.5:

```sh
RUN_ID=local-g45-$(date -u +%Y%m%dT%H%M%SZ) \
AYKEN_DETERMINISTIC_EXIT=1 \
bash scripts/ci/gate_4_5_decision_switch_proof.sh
```

Combined regression:

```sh
RUN_ID=local-policy-proof-$(date -u +%Y%m%dT%H%M%SZ) \
AYKEN_DETERMINISTIC_EXIT=1 \
make ci-gate-policy-proof-regression
```

## 7. Marker Ordering Debug Method

1. extract first-line positions from debugcon log
2. compare with gate JSON (`publish_line`, `accept_line`, `arbiter_line`, `switch_line`)
3. if order fails, move marker emission boundary closer to actual transition site

Principle:

1. emit proof markers at mechanism boundary, not wrapper/helper entry.

## 8. Safety Notes

1. deterministic exit is validation-only proof behavior
2. release/profile-default paths MUST stay unaffected
3. if marker contract changes, gate scripts/tests must change in same patch


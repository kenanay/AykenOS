# Phase10-C C2 Strict Invariants

Status: ACTIVE (governance enforcement profile)  
Scope: `ci-gate-scheduler-mailbox-phase10c` with `PHASE10C_C2_STRICT=1`  
Primary validator: `tools/ci/validate_scheduler_mailbox_phase10c.py`

This document defines the formal invariant set used by Phase10-C strict mode.
It is the enforcement bridge between C2 design intent and CI behavior.

## 1. Invariant Set

1. `INV-C2-001` (`decision_id_monotonic`):
   Applied `decision_id` values MUST be strictly increasing within one boot
   session.
2. `INV-C2-002` (`owner_epoch_monotonic_applied`):
   For each owner, applied `epoch` values MUST be strictly increasing.
3. `INV-C2-003` (`reject_not_applied`):
   A rejected `(owner, epoch)` MUST NOT produce `ARBITER_DECISION` or
   `CTX_SWITCH` as an applied outcome.
4. `INV-C2-004` (`cursor_applied_only`):
   Cursor advance markers (when required) MUST map 1:1 to applied decisions and
   decision IDs.
5. `INV-C2-005` (`fairness_smoke`):
   With `N = |owner_set|` continuously eligible owners, the first `N` applied
   decisions MUST cover all owners in `owner_set`.
6. `INV-C2-006` (`owner_pid_immutability`):
   Owner identifiers appearing in strict markers MUST remain within static
   `owner_set`.
7. `INV-C2-007` (`decision_switch_endpoint_consistency`):
   For each applied `decision_id`, arbiter `from/to` endpoints MUST match
   context-switch `from/to`.
8. `INV-C2-008` (`non_noop_switch`):
   Arbiter and switch transitions MUST satisfy `from != to`.
9. `INV-C2-009` (`marker_shape_freeze`):
   C2 strict markers MUST match frozen schema and enum sets.
10. `INV-C2-010` (`activation_event_precedes_decision`):
   `P10_SCHED_EVENT_NOTIFY` MUST appear before first `P10_IRQ_SCHED_DECISION` marker.
11. `INV-C2-011` (`policy_authority_source_guard`):
   Kernel scheduler source MUST NOT introduce Ring0 policy-call paths; fallback helper call
   MUST remain compile-time guarded and timer IRQ path MUST keep notify->resched binding.

## 2. Marker Domains

Strict mode invariants are evaluated over:

1. `[[AYKEN_SCHED_MB_ACCEPT]]`
2. `[[AYKEN_SCHED_MB_REJECT]]`
3. `[[AYKEN_SCHED_ARBITER_DECISION]]`
4. `[[AYKEN_CTX_SWITCH]]`
5. `[[AYKEN_SCHED_CURSOR_ADVANCE]]` (required when
   `PHASE10C_C2_REQUIRE_CURSOR_MARKER=1`)

## 3. Enforcement Mapping

Invariant to validator mapping:

1. `INV-C2-001` -> `decision_id_not_strictly_increasing`
2. `INV-C2-002` -> `owner_epoch_not_strictly_increasing`
3. `INV-C2-003` -> `reject_followed_by_apply`, `missing_ctx_switch_for_decision_id`
4. `INV-C2-004` -> `cursor_advance_count_mismatch`, `cursor_applied_id_mismatch`
5. `INV-C2-005` -> `fairness_insufficient_applied`, `fairness_smoke_failed`
6. `INV-C2-006` -> `owner_not_in_static_set`
7. `INV-C2-007` -> `decision_switch_endpoint_mismatch`
8. `INV-C2-008` -> `arbiter_noop_forbidden`, `ctx_switch_noop_forbidden`
9. `INV-C2-009` -> `malformed_marker_shape`
10. `INV-C2-010` -> `missing_required_activation:P10_SCHED_EVENT_NOTIFY`, `activation_notify_after_irq_decision`
11. `INV-C2-011` -> `policy_authority_*`, `source_missing_*`, `source_activation_notify_not_bound_to_irq_resched`

## 4. Profile Policy

Policy defaults:

1. `ci-freeze`: `PHASE10C_ENFORCE=1`, `PHASE10C_C2_STRICT=1`
2. `ci-freeze-local`: `PHASE10C_ENFORCE=1`, `PHASE10C_C2_STRICT=0`
3. `validation` profile: strict mode MAY be enabled explicitly.
4. `release` / production profile: strict mode is not required by default.

## 5. Change Control

Any change to invariant semantics MUST update in one change set:

1. this document
2. `docs/governance/MAILBOX_PROTOCOL_V2_C2_REVIEW_FREEZE_CANDIDATE.md`
3. `tools/ci/validate_scheduler_mailbox_phase10c.py`
4. `tools/ci/test_validate_scheduler_mailbox_phase10c.py`

## 6. Scope Clarification (C1 vs C2)
1. C1 runtime contract:
   - `P10_SCHED_DISPATCH -> P10_MAILBOX_DECISION -> P10_DECISION_APPLIED -> P10_RING3_USER_CODE`
   - single-owner execution path (`AYKEN_SCHED_OWNER_PID=2`) aktif gercekliktir.
2. C2 strict contract:
   - C2 marker domain + invariants governance-grade enforcement saglar.
   - owner-set/fairness kontrolleri validator seviyesinde fail-closed calisir.
3. Snapshot notu:
   - owner-set >1 kontrolleri, multi-owner runtime'in tamamen aktif oldugu anlamina gelmez.
   - Multi-owner runtime davranisi Phase10-C sonrasi roadmap kalemi olarak ele alinir.

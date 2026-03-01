# Mailbox Protocol v2 (C2) - Review Freeze Candidate v0

Status: DRAFT (NON-NORMATIVE)  
Effective date: 2026-03-01  
Scope: C2 multi-owner arbitration model candidate  
Owner: Kernel architecture / scheduler boundary

This document is a review-freeze candidate for C2 design closure.

Normative v1 reference remains unchanged:

1. `docs/governance/MAILBOX_PROTOCOL_V1_FREEZE.md`

If this draft conflicts with v1 behavior, v1 wins.

## 1. Purpose

Define a deterministic, evolvable C2 model for multi-owner scheduler mailbox
arbitration before any v2 freeze is declared.

## 2. Freeze Boundary (C1 vs C2)

This document MUST NOT be interpreted as a silent mutation of C1.

Rules:

1. C1 (`MAILBOX_PROTOCOL_V1_FREEZE.md`) remains the only active normative
   freeze.
2. C2 changes require explicit v2 freeze activation in a separate change set.
3. Gate-4 and Gate-4.5 C1 contracts remain valid until v2 activation.

## 3. Core Entities

Core C2 entities:

1. `owner_set`: boot-defined scheduler authority set
2. `pending_request[owner]`: one pending request slot per owner in baseline C2
3. `owner_epoch[owner]`: per-owner monotonic publish epoch
4. `owner_last_epoch[owner]`: per-owner last applied epoch
5. `decision_id`: global monotonic ID assigned on applied arbitration decision
6. `site`: arbitration request source enum with exact values
   `START|YIELD|BLOCK|IRQ`

Owner identity invariant (baseline C2):

1. `owner_pid` is the canonical owner identity key.
2. `owner_pid` for each entry in `owner_set` MUST be immutable for the full
   boot session lifetime.
3. `owner_pid` reuse within the active `owner_set` in the same boot session is
   forbidden (strict mode -> fail-closed, non-strict mode -> panic).

Baseline concurrency assumptions:

1. C2 baseline MUST run in deterministic visibility conditions:
   single-core execution OR lock/atomic protected mailbox request publication.
2. Arbitration decision and context-switch markers MUST be emitted only at the
   canonical apply-point.
3. Publish path MUST NOT emit arbiter/switch markers.

## 4. Owner Set Model (C2 Baseline)

C2 baseline uses static owner configuration:

1. `owner_set` is fixed at boot.
2. Runtime owner add/remove is out of scope for baseline C2.
3. Empty `owner_set` in strict mode is fail-closed.
4. Non-owner publish attempt is rejected.

Dynamic owner membership is deferred to C2.x with explicit migration, revoke,
and epoch-reset semantics.

## 5. Request Model and Eligible Set (Baseline)

Baseline request model:

1. Each owner can have at most one pending request (`pending_request[owner]`).
2. Each request carries exactly one candidate PID in baseline C2.
3. Candidate-list payloads are out of scope for baseline C2.
4. TTL/expiry is not defined in baseline C2 (`no TTL`).

Normative eligible definition:

`Eligible(owner)` is true iff all conditions hold:

1. `owner in owner_set`
2. `pending_request[owner]` exists and is queued
3. `epoch != 0`
4. `epoch > owner_last_epoch[owner]`
5. `candidate_pid` resolves and is runnable

If any condition fails, owner is ineligible for current arbitration cycle.

## 6. Epoch Model (Hybrid)

C2 baseline uses hybrid epoch identity:

1. `owner_epoch` is owner-local and strictly monotonic.
2. Validation rule: publish is valid only if
   `owner_epoch > owner_last_epoch[owner]` and `owner_epoch != 0`.
3. `decision_id` is global, strictly monotonic within one boot session, and
   generated only at canonical apply-point.
4. `decision_id` generation MUST use atomic increment semantics.
5. `decision_id` wrap-around is forbidden:
   strict mode -> fail-closed, non-strict mode -> panic.
6. Global ordering for proof/gates is tracked by `decision_id`, not by
   cross-owner epoch comparison.

Rationale:

1. owner-local epoch preserves independent owner progress.
2. global `decision_id` gives one canonical proof chain axis.

## 7. Arbitration Sites, Canonical Apply-Point, and Cursor Advancement

Arbitration is multi-site triggered and single-point applied:

1. Requests MAY be triggered at `START`, `YIELD`, `BLOCK`, and `IRQ`.
2. Context switch apply MUST happen at one canonical scheduler apply-point
   (scheduler/IRQ tail path).
3. Request-site logic MUST NOT perform direct context switch outside canonical
   apply-point.
4. All applied decisions MUST emit marker chain from canonical apply-point.

Deterministic owner selection:

1. Build `eligible_set` from `Eligible(owner)` owners.
2. Owner traversal order is frozen as ascending `owner_pid`.
3. Traverse owners in round-robin order starting from `owner_cursor`.
4. Select first eligible owner in scan order.
5. Candidate tie-break is not applicable in baseline C2
   (one candidate per owner request).
6. If `eligible_set` is empty:
   strict mode -> fail-closed,
   non-strict mode -> keep-running current runnable task, else panic.

Cursor advancement rule (normative):

1. `owner_cursor` advances only after an `APPLIED` decision.
2. On reject/no-decision cycles, `owner_cursor` is not advanced.
3. `owner_cursor` advances to the next owner slot after the selected owner.

Epoch role in ordering:

1. Epoch is used for stale/valid/supersede checks.
2. Epoch is not the global cross-owner ordering key.

## 8. Supersede and `owner_last_epoch` Semantics

Supersede is owner-local only:

1. `(owner, epoch)` tuple MUST be unique.
2. Same-owner duplicate publish with same epoch is rejected (`EPOCH_DUP`).
3. Same-owner older epoch than pending/applied state is rejected (`EPOCH_STALE`).
4. Newer same-owner epoch supersedes older queued request for that owner.
5. Cross-owner supersede is forbidden.
6. Objective invalidation (for example non-runnable target) is reject, not
   supersede.

`owner_last_epoch[owner]` update rule:

1. `owner_last_epoch[owner]` is updated only when decision reaches `APPLIED`.
2. Validation/queue/consume without apply MUST NOT update
   `owner_last_epoch[owner]`.

## 9. Fairness and Starvation Bound

Baseline fairness contract:

1. Let `N = |owner_set|`.
2. If at least one owner is eligible, each applied arbitration advances cursor
   exactly once.
3. If one owner stays continuously eligible, selection bound is at most
   `N - 1` applied decisions.
4. If owner loses eligibility or system enters fail-closed path, bound is
   suspended until eligibility and progress are restored.

This is the required bounded-starvation statement for C2 baseline.

## 10. Marker Contract Impact (C2 Target Shape)

Proposed C2 marker chain (minimal proof set):

1. `[[AYKEN_SCHED_MB_ACCEPT]] owner=<o> epoch=<e> cand=<c> site=<s>`
2. `[[AYKEN_SCHED_MB_REJECT]] reason=<r> owner=<o> epoch=<e> cand=<c> site=<s>`
3. `[[AYKEN_SCHED_ARBITER_DECISION]] decision_id=<d> site=<s> owner=<o> from=<p> to=<q> epoch=<e>`
4. `[[AYKEN_CTX_SWITCH]] decision_id=<d> from=<p> to=<q>`

Fixed enum sets:

1. `site` values MUST be one of `START|YIELD|BLOCK|IRQ` (exact spelling).
2. `reason` values MUST be one of:
   `NON_OWNER`, `EPOCH_DUP`, `EPOCH_STALE`, `CAND_NOT_RUNNABLE`,
   `OWNERSET_VIOLATION`, `NO_ELIGIBLE_STRICT`, `MALFORMED_REQUEST`.

Strict proof checks (C2 target):

1. `accept -> arbiter_decision -> ctx_switch` ordering for applied path
2. `decision_id` consistency across decision and switch
3. endpoint consistency (`from/to`)
4. `from != to` (no noop switch)
5. reject markers MUST NOT be paired with `AYKEN_CTX_SWITCH` for the same
   owner/epoch arbitration outcome

## 11. C2 Strict Gate Matrix and Migration Plan

C2 strict-mode gates MUST validate:

1. `decision_id` is strictly increasing within one boot session.
2. applied owner epochs are monotonic per owner.
3. reject outcomes do not produce decision->switch apply chain.
4. fairness smoke: with `N` continuously eligible owners, over `N`
   applied decisions each eligible owner is selected at least once.
5. marker shape and enum values match fixed contract.
6. enforcement source of truth for strict invariants:
   `docs/governance/PHASE10C_C2_STRICT_INVARIANTS.md`

Migration phases:

1. Keep C1 active freeze unchanged.
2. Land C2 internals behind explicit non-default knobs.
3. Add C2-aware gates and marker parser updates.
4. Promote to normative `MAILBOX_PROTOCOL_V2_FREEZE.md` only after proof pass and
   governance approval.

## 12. Strict/Non-Strict Outcome Matrix and Negative Cases

Strict-mode required outcomes:

1. `owner_set` empty -> fail-closed
2. non-owner publish attempt -> reject (`NON_OWNER`)
3. same-owner duplicate epoch -> reject (`EPOCH_DUP`)
4. stale epoch publish -> reject (`EPOCH_STALE`)
5. candidate not runnable -> reject (`CAND_NOT_RUNNABLE`)
6. owner missing/invalid at consume or apply time
   -> fail-closed (`OWNERSET_VIOLATION`)
7. no eligible owner -> fail-closed (`NO_ELIGIBLE_STRICT`)
8. malformed marker chain or `decision_id` mismatch -> gate fail

Non-strict baseline behavior:

1. `owner_set` empty -> panic.
2. no eligible owner -> keep-running current runnable task, else panic.
3. strict proof and fairness claims are not guaranteed in non-strict mode.

## 13. Review Checklist

Freeze-candidate review MUST confirm:

1. owner_set mutability defined (`static@C2`, dynamic deferred to C2.x)
2. eligible_set formula is explicit and complete
3. baseline request model fixed (one pending request + one candidate per owner)
4. epoch model finalized (`hybrid: owner-local epoch + global decision_id`)
5. `decision_id` scope, atomicity, and wrap behavior are explicit
6. arbitration site model fixed (`multi-site trigger, single canonical apply-point`)
7. cursor advancement rule fixed (`APPLIED`-only)
8. supersede semantics complete (owner-local only)
9. `owner_last_epoch` update point fixed (`APPLIED`-only)
10. starvation bound expressed (`<= N - 1` under stated assumptions)
11. strict/non-strict outcome matrix is explicit
12. reject reason enum and marker shape are frozen
13. strict proof mode behavior defined (ordering + consistency + non-noop)
14. owner identity immutability is explicit (`owner_pid` immutable per boot)

# Scheduler Owner Handoff/Reap - Review Candidate v0

Status: DRAFT (NON-NORMATIVE)  
Effective date: 2026-03-20  
Scope: Scheduler-owner lifecycle closure after Phase 10-B  
Owner: Kernel architecture / scheduler boundary

This document is a review candidate. It does not silently mutate the active
mailbox v1 freeze or authorize runtime code by itself.

## 1. Purpose

Define the minimal safe shape required to eventually retire the scheduler owner,
transfer mailbox authority, and reap owner-owned scheduler surfaces without
violating fail-closed mailbox semantics.

## 2. Existing Normative Constraints

The current repo already freezes the following constraints:

1. `docs/governance/MAILBOX_PROTOCOL_V1_FREEZE.md`
   - single owner authority is `AYKEN_SCHED_OWNER_PID`
   - owner missing/not-runnable is fail-closed
   - C2 multi-owner work MUST NOT silently mutate v1
2. `docs/governance/MAILBOX_PROTOCOL_V2_C2_REVIEW_FREEZE_CANDIDATE.md`
   - baseline C2 keeps `owner_set` fixed at boot
   - runtime owner add/remove remains out of scope
3. `docs/specs/phase10b-execution-path-hardening/requirements.md`
   - scheduler owner currently MUST fail closed on `sys_v2_exit()` at syscall
     entry until an explicit handoff protocol exists

Immediate consequence:

- the current owner-exit deny path is correct
- any code change that relaxes that deny path without new governance would be an
  invalid silent mutation of the active scheduler authority model

## 3. Current Problem

Phase 10-B now has:

1. real non-owner no-return `sys_v2_exit()` proof
2. real execution lifecycle closure
3. real explicit mapping/result teardown

But the scheduler owner remains special:

1. mailbox-first successor selection still resolves authority through the owner
   mailbox path
2. owner mailbox backing is still treated as an authority anchor
3. owner exit is fail-closed because there is no ratified runtime handoff model

This is no longer a generic runtime bug. It is a governance-bound lifecycle
closure debt.

## 4. Why Existing Paths Cannot Be Reused

The following shortcuts are invalid:

### 4.1 Plain Owner `sys_v2_exit()`

Not valid because:

1. owner removal before authority transfer violates v1 owner-missing
   fail-closed semantics
2. successor selection can still dereference the owner mailbox path
3. owner mailbox backing cannot be synchronously reaped while it remains the
   active authority anchor

### 4.2 Validation-Only Forced Successor Seam

Not valid because:

1. it exists only to prove the non-owner no-return exit path safely
2. it bypasses the mailbox-first authority contract by construction
3. shipping it as runtime behavior would be a hidden scheduler fallback

### 4.3 “Just Use C2”

Not valid yet because:

1. the active normative freeze is still mailbox v1
2. the current C2 review candidate keeps owner membership fixed at boot
3. dynamic owner add/remove or single-owner transfer is still out of scope in
   that candidate

## 5. Required Semantic Shape

Any future owner-handoff/reap mechanism MUST preserve all of the following:

1. exactly one authoritative scheduler owner before and after handoff
2. no interval where owner authority is absent but normal scheduling continues
3. no reuse of the validation-only forced-successor seam as a runtime feature
4. owner teardown MUST remain fail-closed until handoff commit succeeds
5. old owner mailbox backing MUST NOT be reaped before authority transfer
6. post-handoff old-owner publishes MUST be rejected or ignored fail-closed
7. successor owner MUST be a live runnable user process with valid mailbox
   backing and valid scheduler-visible surfaces
8. mailbox epoch / last-epoch bookkeeping MUST remain deterministic across the
   handoff boundary

## 6. Minimal Future Runtime Model

This candidate freezes the minimum acceptable shape, not the final ABI.

### 6.1 Phase A: Handoff Validation

The kernel validates:

1. current caller is the active scheduler owner
2. successor process exists, is user type, and is runnable
3. successor mailbox backing exists and passes mailbox ABI shape checks
4. successor is not already `PROC_ZOMBIE`

If any validation fails:

- owner exit remains denied
- no ownership mutation begins

### 6.2 Phase B: Atomic Authority Commit

At one canonical scheduler apply-point, the kernel MUST atomically:

1. retire the old owner from authority
2. promote the successor into the sole active owner role
3. switch mailbox authority lookup to the successor
4. preserve deterministic epoch/decision behavior across the boundary

That commit point MUST be the scheduler dispatch boundary while scheduling is
paused for the current CPU. It is not an arbitrary syscall-body mutation point
and it is not a timer-validation side effect.

There MUST NOT be:

1. dual owner visibility
2. ownerless runnable scheduling
3. silent fallback to non-mailbox scheduling

### 6.3 Phase C: Old Owner Exit/Reap

Only after authority commit succeeds may the old owner:

1. enter the normal no-return `sys_v2_exit()` path
2. revoke its mailbox/delivery/result/generic mappings
3. join deferred reap for active root-PML4 / current `rsp0` if needed
4. release its former owner mailbox backing once it is no longer authoritative

## 7. Proof Obligations Before Runtime Activation

The deny path in `sys_v2_exit()` MUST NOT be relaxed until all of the following
have proof coverage:

1. owner handoff commits without producing `P10_MAILBOX_OWNER_MISSING_FATAL`
2. exactly one owner is visible before and after commit
3. old-owner mailbox publish no longer drives scheduling after handoff
4. successor owner mailbox publish does drive scheduling after handoff
5. old owner can then take the direct no-return exit path
6. old owner mailbox backing and deferred scheduler surfaces are eventually
   reaped

## 8. Ratified Governance Path

The governing narrow runtime path is now:

1. `docs/governance/MAILBOX_V1_OWNER_TRANSFER_EXCEPTION.md`

This review candidate remains useful as broader design context, but it is no
longer the ratification decision point.

Current consequence:

- owner exit still remains fail-closed for the active owner until the ratified
  narrow mechanism finishes runtime proof and landing
- scheduler-owner handoff/reap is now partially implementation debt and
  partially proof debt, not an unchosen governance question

## 9. Non-Goals

This candidate does not authorize:

1. silent mutation of mailbox v1 freeze semantics
2. multi-owner fairness/runtime activation
3. using the validation seam as production scheduling behavior
4. relaxing fail-closed owner exit before proof obligations are defined

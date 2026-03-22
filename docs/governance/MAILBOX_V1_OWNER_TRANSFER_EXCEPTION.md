# Mailbox v1 Owner Transfer Exception

Status: RATIFIED  
Effective date: 2026-03-20  
Scope: Narrow single-owner transfer exception on top of mailbox protocol v1/C1

This document is the active narrow governance exception for scheduler-owner
transfer on top of mailbox v1/C1. It does not by itself activate production
runtime handoff; runtime activation still depends on the proof and landing
conditions below.

## 1. Problem

The active mailbox v1/C1 freeze keeps scheduler authority bound to one
`AYKEN_SCHED_OWNER_PID` and fails closed when that owner is missing or not
runnable.

Phase 10-B has already closed ordinary lifecycle work:

- non-owner `sys_v2_exit()` now has direct no-return runtime proof
- execution completion and result ownership are real
- lower-half teardown and generic mapping revoke are real

The remaining lifecycle gap is now specific:

- the scheduler owner still cannot retire without breaking authority continuity

This is not a generic teardown problem. It is a narrow single-owner transfer
problem.

## 2. What This Exception Would Allow

This exception allows exactly one new semantic capability on top of mailbox
v1/C1:

1. transfer sole scheduler-owner authority from the current owner to one
   validated successor owner
2. only after that transfer commits, allow the old owner to enter the normal
   no-return exit/reap path

Nothing more is authorized.

## 3. What This Exception Would NOT Allow

This exception would **not** authorize:

1. multi-owner runtime scheduling
2. owner-set fairness or round-robin owner arbitration
3. dynamic owner add/remove beyond one-at-a-time sole-owner transfer
4. fallback scheduling when owner authority is absent
5. use of the validation-only forced-successor seam as production behavior
6. silent mutation of marker/gate meaning outside the narrow transfer path
7. relaxing non-owner or ordinary mailbox rules unrelated to owner transfer

## 4. Core Invariant After Ratification

The immutable-v1 rule:

- “the owner PID never changes during the boot”

would be replaced only by this narrower invariant:

- “exactly one scheduler owner is authoritative at every point in time”

This means:

1. dual-owner visibility is forbidden
2. ownerless runnable scheduling is forbidden
3. old-owner mailbox authority must stop exactly when successor authority begins

## 5. Minimal Transfer Contract

Any runtime implementation activated under this exception MUST preserve the
following contract.

### 5.1 Validation Phase

Before commit, the kernel validates:

1. caller is the current active scheduler owner
2. successor process exists
3. successor is a live runnable user process
4. successor mailbox backing exists and passes mailbox ABI shape checks
5. successor is not the current owner
6. successor is not `PROC_ZOMBIE`

If any validation fails:

- transfer MUST fail closed
- old owner remains authoritative
- no teardown or revoke side effects begin

### 5.2 Atomic Commit Phase

At one canonical scheduler apply-point, the kernel MUST atomically:

1. retire the old owner from authority
2. promote the successor to sole owner authority
3. switch authoritative mailbox lookup to the successor
4. preserve deterministic epoch / decision behavior across the boundary

That authority commit MUST occur at the scheduler dispatch boundary while
scheduling is paused for the current CPU.

It MUST NOT be performed:

1. opportunistically inside the initiating syscall body
2. inside timer-path mailbox validation
3. inside arbitrary mailbox read/extract helpers

Forbidden outcomes:

1. two simultaneous owners
2. no owner while scheduling still proceeds
3. hidden fallback to non-mailbox scheduling

### 5.3 Post-Commit Exit/Reap Phase

Only after commit succeeds may the old owner:

1. enter the standard no-return `sys_v2_exit()` path
2. revoke mailbox/delivery/result/generic mappings
3. join deferred reap for active root-PML4 / current `rsp0` if needed
4. release old owner mailbox backing once it is no longer authoritative

## 6. Failure If Rejected

If this exception is rejected:

- scheduler-owner exit remains permanently fail-closed
- scheduler-owner lifecycle remains structurally incomplete
- owner mailbox backing remains a persistent authority anchor for the full boot
- Phase 10-B cannot claim full authority continuity

This is not a degraded owner path.

This is an intentionally incomplete authority lifecycle.

## 7. Required Proof Before Activation

This exception MUST NOT be activated in runtime code until all of the following
are proven:

1. exactly one owner exists before and after commit
2. no ownerless scheduling window occurs
3. old-owner mailbox content no longer drives scheduling after commit
4. successor-owner mailbox content does drive scheduling after commit
5. old owner can then take the direct no-return exit path safely
6. old-owner mailbox backing and deferred scheduler surfaces are eventually
   reaped

## 8. Current Recommendation

This document was ratified because:

1. the active runtime is still single-owner
2. the open debt is single-owner continuity, not multi-owner policy
3. this is the smallest governance change that can close the remaining owner
   lifecycle gap without promoting mailbox v2/C2 prematurely

## 9. Relationship to Other Governance Docs

This candidate must be read together with:

1. `docs/governance/MAILBOX_PROTOCOL_V1_FREEZE.md`
2. `docs/governance/SCHEDULER_OWNER_HANDOFF_REAP_CANDIDATE.md`
3. `docs/governance/SCHEDULER_OWNER_HANDOFF_RATIFICATION_OPTIONS.md`

This document is the active narrow exception governing owner transfer on top of
mailbox v1/C1.

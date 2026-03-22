# Scheduler Owner Handoff Ratification Options

Status: DRAFT (NON-NORMATIVE)  
Effective date: 2026-03-20  
Scope: Comparison of the two valid ratification paths for owner handoff/reap  
Owner: Kernel architecture / scheduler boundary

This document does not ratify a runtime mechanism by itself. It narrows the
decision surface so owner handoff/reap does not drift into an unreviewed
implementation shortcut.

## 1. Problem Statement

Phase 10-B now has:

1. real non-owner `sys_v2_exit()` no-return proof
2. real execution lifecycle closure
3. real result/mapping teardown

But scheduler-owner lifecycle closure is still blocked because the active
mailbox v1 freeze binds scheduler authority to one fixed owner PID and fails
closed when that owner is missing.

The remaining problem is therefore not generic exit cleanup. It is authority
continuity:

1. exactly one owner must exist before and after handoff
2. scheduling must not continue through an ownerless window
3. the old owner must not keep influencing mailbox-first scheduling after commit
4. old-owner cleanup must start only after authority transfer succeeds

## 2. Active Constraints

The current repo already freezes these constraints:

1. mailbox v1/C1 is the only active normative owner-authority model
2. `AYKEN_SCHED_OWNER_PID` is the active single-owner authority anchor
3. owner-missing / owner-not-runnable paths are fail-closed
4. mailbox v2/C2 remains review-candidate only
5. owner `sys_v2_exit()` is currently denied at syscall entry

Immediate consequence:

- owner handoff/reap cannot be landed as “just another runtime fix”

## 3. Valid Ratification Paths

Only two narrow paths remain technically coherent.

### Option A: Narrow Mailbox v1 Transfer Exception

Ratify a narrow exception layered on top of mailbox v1/C1.

Shape:

1. keep the single-owner model
2. add one explicitly governed transfer mechanism from old owner to one
   validated successor owner
3. keep mailbox-first scheduling and fail-closed behavior everywhere else
4. permit old-owner exit only after transfer commit succeeds

What it changes:

1. mailbox v1 no longer means “owner PID is immutable for the whole boot”
2. instead it means “exactly one owner is authoritative at any point in time”

What it does **not** change:

1. no multi-owner runtime
2. no fairness/round-robin owner-set behavior
3. no dynamic owner add/remove beyond the single transfer event
4. no fallback scheduler path

### Option B: Promote Mailbox v2/C2 Owner-Transfer Path

Promote a mailbox v2/C2 authority model that explicitly includes owner transfer
or dynamic owner membership.

Shape:

1. stop treating mailbox v1/C1 as the authority model for owner lifecycle
2. move owner continuity into a ratified v2/C2 owner-set model
3. define transfer under the broader C2 arbitration rules

What it changes:

1. owner identity is no longer purely a v1 single-fixed-PID invariant
2. marker/gate interpretation shifts toward C2 governance
3. transfer semantics are absorbed into a broader scheduler-authority revision

## 4. Comparison Matrix

| Dimension | Option A: Narrow v1 Exception | Option B: Promote v2/C2 Path |
|---|---|---|
| Scope | Minimal | Broad |
| Active freeze impact | explicit narrow exception | active authority model promotion |
| Runtime churn | lower | higher |
| Gate / marker churn | low-to-medium | medium-to-high |
| Fairness / owner-set semantics | unchanged | likely touched |
| Time to close owner lifecycle debt | shorter | longer |
| Architectural risk | isolated | broader blast radius |
| Long-term extensibility | lower | higher |

## 5. Technical Tradeoff

### 5.1 Why Option A Is Attractive

1. it closes the exact debt that is still blocking lifecycle completeness
2. it keeps the active runtime model single-owner and mailbox-first
3. it minimizes proof scope to authority transfer + old-owner reap
4. it avoids dragging C2 fairness, owner-set traversal, and marker redesign into
   a Phase 10-B closure task

### 5.2 Why Option A Is Costly

1. it introduces a real exception into the active v1 freeze
2. if C2 is later promoted, some of this work may be transitional rather than
   final-form
3. it must be written narrowly enough that it does not become a disguised
   multi-owner model

### 5.3 Why Option B Is Attractive

1. it aligns owner transfer with the future scheduler-authority direction
2. it avoids carving special-case lifecycle semantics into v1
3. it may prevent a second governance migration later

### 5.4 Why Option B Is Costly

1. it expands the change set far beyond the immediate owner-exit problem
2. it couples Phase 10-B closure to C2 owner-set and arbitration governance
3. it raises the proof burden substantially
4. it delays runtime closure of the current owner fail-closed boundary

## 6. Recommended Path

If the immediate goal is to finish owner lifecycle continuity without widening
scope, the recommended path is:

1. Option A: narrow mailbox v1 transfer exception

Rationale:

1. the active runtime is still single-owner
2. the remaining debt is single-owner continuity, not multi-owner policy
3. the system already has a clear fail-closed boundary and needs the smallest
   ratified mechanism that can replace that boundary safely

Option B becomes the better choice only if the project is ready to promote C2 as
the active scheduler-authority model now, not later.

## 7. Minimum Proof Required Regardless of Path

Both options must prove all of the following before owner exit deny can be
relaxed:

1. exactly one authoritative owner exists before and after commit
2. no ownerless scheduling window appears
3. old-owner mailbox content no longer drives scheduling after commit
4. successor-owner mailbox content does drive scheduling after commit
5. old owner can then take the no-return exit path safely
6. old owner mailbox backing and deferred scheduler surfaces are eventually
   reaped

## 8. What This Document Explicitly Rejects

This comparison does not authorize:

1. using the validation-only forced-successor seam as production behavior
2. shipping owner handoff as an undocumented scheduler shortcut
3. weakening the owner-exit deny path before one ratification path is chosen
4. treating owner handoff as a generic teardown fix rather than an authority
   transfer problem

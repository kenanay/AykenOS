# Ring3 User Leaf Allocation Rule

## Rule

User-authored executable leaves mapped into a non-kernel CR3 must not use the
low-phys frame class.

Current mandatory scope:

- Ring3 flat-image text page
- Ring3 ELF `PT_LOAD` executable image pages

Current enforcement point:

- [kernel/proc/proc.c](/Users/asel/Desktop/AykenOS/kernel/proc/proc.c)
  `proc_alloc_user_image_frame()`

## Why This Exists

Phase10 runtime evidence established a repeatable MMU-visible mismatch:

- software walk reported the user text leaf as present and executable
- the same CR3, after `mov %cr3`, read zeros at `0x400000`
- moving the user text leaf to high-phys immediately restored real opcode fetch
  and produced `P10_RING3_USER_CODE`

This is not a generic paging rule. It is a concrete runtime rule for
user-authored executable leaves crossing the Ring0 -> Ring3 boundary.

## Evidence Contract

Authoritative validation signals for this rule are:

- `P10_TEXT_FRAME_WITNESS`
- `P10_POST_CR3_TEXT_PROBE`
- `P10_RING3_USER_CODE`

Valid success shape:

- pre-dispatch witness shows stable text bytes and hash
- post-CR3 probe reads the same opcode qword
- user marker is reached under the same CR3

## Diagnostic Authority Note

Software walk output is non-authoritative if it is gathered under an active
user CR3 with helpers that assume kernel direct-map visibility.

Authoritative walk diagnostics for this rule must be kernel-CR3-safe.

## Non-Rule Notes

- This document does not yet promote stack, canary, mailbox, inbox, or payload
  leaves to the same mandatory class rule.
- They remain adjacent risk surfaces and may be promoted later if equivalent
  evidence appears.

## Change Discipline

- Do not reintroduce `phys_alloc_frame()` for executable Ring3 image leaves.
- If this rule is narrowed or expanded, update both the runtime witness path
  and this governance file in the same change.

## Active CI Enforcement

This rule is now enforced by a dedicated CI gate:

- `ci-gate-ring3-user-leaf-rule`

Current authority level:

- local deterministic gate evidence is active
- Makefile wiring is active in `ci-freeze` and `ci-freeze-local`
- broader Phase10-A2 strict/global CI authority still requires successful
  full-suite evidence under the repository's primary CI path

Canonical summary:

- Ring3 executable user-leaf rule is live across code, governance, and a local
  deterministic CI gate.
- This does not by itself promote the broader Phase10-A2 strict/global closure
  claim.

Authority split:

- allocator-class policy is enforced by source guard
- first-fetch/runtime success is enforced by
  `P10_TEXT_FRAME_WITNESS -> P10_POST_CR3_TEXT_PROBE -> P10_RING3_USER_CODE`

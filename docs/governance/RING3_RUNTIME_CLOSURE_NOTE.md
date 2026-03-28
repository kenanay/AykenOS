# Ring3 Runtime Closure Note

Status: ACTIVE (runtime closure note + local CI enforcement live)  
Scope: Phase10-A2 first-user-fetch boundary

Working summary:

- Ring3 executable user-leaf rule is now live across code, governance, and a
  local deterministic CI gate.
- Broader Phase10-A2 strict/global closure still requires primary CI
  full-suite evidence.

## 1. Closure Statement

The Phase10-A2 first-user-fetch runtime blocker is locally closed.

Closed claim:

- the first user executable fetch under a non-kernel CR3 is now runtime-valid
- `P10_POST_CR3_TEXT_PROBE` reads real opcode bytes from the user text leaf
- `P10_RING3_USER_CODE` is reached under the same user CR3

This note does not claim that every user-authored leaf class is now frozen.
It closes the specific first-user-fetch/runtime boundary that was previously
blocked by MMU-visible mismatch.

This note does not by itself claim that the broader historical
`ci-gate-ring3-execution-phase10a2` blocker set is globally closed.
That higher authority claim still depends on successful full-suite evidence in
the repository's primary CI path.

## 2. Runtime Rule Produced By Closure

The closure produced one governance-grade runtime rule:

- user-authored executable leaves mapped into a non-kernel CR3 must not use the
  low-phys frame class

Normative rule text lives in:

- [RING3_USER_LEAF_ALLOCATION_RULE.md](/Users/asel/Desktop/AykenOS/docs/governance/RING3_USER_LEAF_ALLOCATION_RULE.md)

## 3. Authoritative Evidence Shape

The minimum authoritative success shape is:

1. pre-dispatch text witness shows stable executable bytes
2. post-CR3 text probe reads the same opcode qword
3. user marker is reached under the same CR3

Current authoritative markers:

- `P10_TEXT_FRAME_WITNESS`
- `P10_POST_CR3_TEXT_PROBE`
- `P10_RING3_USER_CODE`

## 4. Diagnostic Authority Note

Software walk output under an active user CR3 is non-authoritative unless the
walker is kernel-CR3-safe.

Interpretation rule:

- kernel-CR3-safe walk helpers may be used as evidence
- active-user-CR3 walk helpers that rely on kernel direct-map assumptions may
  produce false zero / false non-present diagnostics

This note exists because the closure sequence included one such false-negative
surface before the walker was made kernel-CR3-safe.

## 5. Active CI Enforcement

The closure now has a live CI gate:

- `ci-gate-ring3-user-leaf-rule`

Current authority level:

- active as a deterministic local gate with repository Makefile wiring
- suitable for fail-closed enforcement of the executable-leaf rule itself
- not sufficient on its own to restate broader Phase10-A2 strict/global closure

Authority layers:

- Rule authority: active, local deterministic, fail-closed
- CI topology authority: integrated
- Broader Phase10-A2 strict authority: pending
- Global/freeze authority: pending primary CI full-suite evidence

Implemented enforcement split:

1. Source guard forbids `phys_alloc_frame()` in the executable Ring3 image-leaf
   allocation path and requires `proc_alloc_user_image_frame() ->
   phys_alloc_frame_high()`.
2. Runtime validation is bound to the authoritative chain:
   `P10_TEXT_FRAME_WITNESS -> P10_POST_CR3_TEXT_PROBE -> P10_RING3_USER_CODE`.
3. Software-walk diagnostics remain non-authoritative unless the helper is
   kernel-CR3-safe; the dedicated runtime gate does not consume walk output as
   authority.

## 6. Scope Outside This Closure

This closure note does not automatically promote these surfaces to the same
rule:

- stack
- canary
- mailbox
- inbox
- payload

They remain evidence-driven promotion candidates. They should only be elevated
to mandatory high-phys policy if equivalent runtime evidence appears.

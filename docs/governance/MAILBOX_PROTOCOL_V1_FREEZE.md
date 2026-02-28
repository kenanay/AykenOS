# Mailbox Protocol v1 Freeze

Status: ACTIVE (FROZEN)  
Effective date: 2026-02-28  
Scope: Phase10 C1 + Gate-4/Gate-4.5 proof line  
Owner: Kernel architecture / scheduler boundary

This document freezes the scheduler mailbox protocol at v1 for the
single-authority (owner-sovereign) regime.

If code and this document conflict, this document defines the required
contract for CI/governance gates.

## 1. Purpose

Define a stable, fail-closed contract for:

1. Ring3 policy publish
2. Ring0 validation/consumption
3. decision-to-switch proof chain
4. deterministic proof-complete termination in validation mode

This freeze is the baseline before any C2 multi-owner arbitration work.

## 2. Normative Terms

The keywords MUST, MUST NOT, SHOULD, and MAY are normative.

## 3. ABI Surface (v1)

Authoritative ABI files:

1. `kernel/include/sched_mailbox_abi.h`
2. `constitution/abi_mailbox.json`

v1 constants:

1. `AYKEN_SCHED_MB_MAGIC = 0x4B534D42` (`KSMB`)
2. `AYKEN_SCHED_MB_VERSION = 1`
3. `kind = AYKEN_SCHED_HINT_CANDIDATE (1)` for valid publish

Struct contract (`ayken_sched_mailbox_t`, packed, aligned 64):

1. `magic` (offset 0)
2. `version` (4)
3. `kind` (6)
4. `epoch` (8)
5. `proposer_pid` (16)
6. `candidate_pid` (20)
7. `flags` (24)
8. `status` (28)
9. `reject_reason` (32)
10. `reserved` (36)

Any v1 field offset/size mutation is forbidden without ABI version bump and
gate updates.

## 4. Authority Model (C1 Freeze)

Single source of scheduling authority:

1. `AYKEN_SCHED_OWNER_PID` (default `2`)

Rules:

1. Ring0 MUST resolve decisions from owner mailbox path only.
2. Non-owner publish attempt MUST be treated as protocol violation.
3. Owner missing/not-runnable in strict path MUST fail-closed.
4. Legacy fallback MUST remain compile-time gated and OFF in strict mode.

Key failure markers:

1. `P10_MAILBOX_OWNER_MISSING_FATAL`
2. `P10_MAILBOX_OWNER_NOT_READY_FATAL`
3. `P10_MAILBOX_OWNER_MISMATCH`
4. `P10_SCHED_FALLBACK_FORBIDDEN`

## 5. Decision Lifecycle

Decision phases:

1. `PUBLISHED`:
   `magic/version/kind` valid and `epoch > mailbox_last_epoch`.
2. `VALIDATED`:
   timer path validation accepts mailbox payload.
3. `CONSUMED`:
   scheduler consumes epoch (`mailbox_last_epoch = epoch`) under site rules.
4. `APPLIED`:
   context transition path applies decision.

Phase10 markers:

1. `P10_MAILBOX_DECISION id=<e> pid=<target> valid=1 src=<owner>`
2. `P10_DECISION_APPLIED id=<e> pid=<target> valid=0 src=<owner>`

Semantics:

1. `valid=1` means observed/validated mailbox decision.
2. `valid=0` means slot consumed/applied in scheduler path.

## 6. Validation Contract (Ring0)

Ring0 mailbox validation MUST enforce:

1. ABI checks: magic/version/kind
2. torn-read guard: `e1 == e2` (epoch double read)
3. monotonic epoch: `epoch > mailbox_last_epoch` and `epoch != 0`
4. candidate sanity: `candidate_pid != 0`, resolvable, runnable
5. owner sovereignty checks for Gate-4/Gate-4.5 modes

Reject reasons (base):

1. `AYKEN_SCHED_REJECT_BAD_MAGIC`
2. `AYKEN_SCHED_REJECT_BAD_VERSION`
3. `AYKEN_SCHED_REJECT_BAD_KIND`
4. `AYKEN_SCHED_REJECT_STALE_EPOCH`
5. `AYKEN_SCHED_REJECT_BAD_PID`
6. `AYKEN_SCHED_REJECT_NOT_RUNNABLE`

Gate-specific internal reject IDs MAY be used for proof diagnostics
(example: torn-read/owner mismatch codes).

## 7. Site Semantics (Start/Yield/Block)

Decision sites:

1. `START`
2. `YIELD`
3. `BLOCK`

Strict-mode requirements (`AYKEN_SCHED_BOOTSTRAP_POLICY=0`):

1. `START` without valid owner decision MUST fail-closed.
2. `YIELD` without valid owner decision MUST fail-closed.
3. `BLOCK` without valid owner decision MUST fail-closed.
4. `BLOCK` MUST NOT keep-running the blocked process.

Transitional behavior (`AYKEN_SCHED_BOOTSTRAP_POLICY=1`) MAY permit keep-running
on yield, but this is not constitutional strict mode.

## 8. Gate-4 / Gate-4.5 Proof Contracts

Reference scripts:

1. `scripts/ci/gate_4_policy_accept.sh`
2. `scripts/ci/gate_4_5_decision_switch_proof.sh`
3. schema source: `constitution/markers_schema_v1.json`
4. schema governance freeze: `docs/governance/MARKER_SCHEMA_V1_FREEZE.md`

Gate-4 (`AYKEN_GATE45_PROOF=0`, `selftest=0`) requires:

1. exactly one target accept (`target_accept_count == 1`)
2. exactly one total accept (`total_accept_count == 1`)
3. publish before accept

Gate-4.5 (`AYKEN_GATE45_PROOF=1`, `selftest=0`) requires:

1. exactly one target accept
2. exactly one arbiter marker
3. exactly one ctx-switch marker
4. strict ordering:
   `publish < accept < arbiter < switch`
5. endpoint consistency (`from/to`) and non-noop transition (`from != to`)

## 9. Deterministic Proof-Complete Termination

Deterministic termination is enabled only in validation proof modes.

Knob:

1. `AYKEN_DETERMINISTIC_EXIT=1`

Completion marker:

1. `[[AYKEN_PROOF_DONE]]`

Exit paths:

1. primary: isa-debug-exit (`outw(0x501, 0)`) -> process exit code `1`
2. fallback: ACPI poweroff (`outw(0x604, 0x2000)`) -> process exit code `0`

CI deterministic contract:

1. when enabled, exit code MUST be `0` or `1`
2. `[[AYKEN_PROOF_DONE]]` MUST be present

## 10. Compile-Time Governance Knobs

This freeze covers behavior under these knobs:

1. `AYKEN_SCHED_OWNER_PID`
2. `AYKEN_SCHED_BOOTSTRAP_POLICY`
3. `AYKEN_SCHED_FALLBACK`
4. `AYKEN_GATE4_POLICY_TEST`
5. `AYKEN_GATE45_PROOF`
6. `AYKEN_DETERMINISTIC_EXIT`

Changing default semantics of these knobs requires governance review.

## 11. Change Control (Freeze Rules)

Mailbox protocol v1 changes are controlled:

1. ABI layout/value changes:
   MUST bump version and update `constitution/abi_mailbox.json`.
2. Marker contract changes:
   MUST update Gate-4/Gate-4.5 scripts and tests in same change set.
3. Fail-closed semantics changes:
   MUST include explicit rationale and migration notes.
4. C2 multi-owner work:
   MUST be introduced as a new spec version (v2), not silent mutation of v1.

## 12. Out of Scope (v1)

Not part of v1 freeze:

1. multi-owner arbitration policy
2. weighted/priority owner conflict resolution
3. cross-owner consensus protocol

These belong to C2 and require a separate v2 spec.

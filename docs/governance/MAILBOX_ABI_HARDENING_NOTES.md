# Mailbox ABI Hardening Notes

Status: ACTIVE  
Audience: Maintainers touching mailbox ABI or proof markers

This document tracks hardening requirements around mailbox ABI stability.
Normative freeze remains:

1. `docs/governance/MAILBOX_PROTOCOL_V1_FREEZE.md`

## 1. Why This Exists

`MAILBOX_PROTOCOL_V1_FREEZE.md` defines rules. This note explains how to
enforce them technically (compile-time and CI-time).

## 2. Current Risk Surface

Known risks to manage:

1. ABI layout drift without version bump
2. marker contract drift without gate updates
3. proof semantics drift under refactor
4. SMP race exposure for non-atomic proof latches (out of v1 scope but tracked)

## 3. Required Hardening Controls

### 3.1 Compile-Time ABI Assertions

In ABI-owning translation unit, maintain:

1. `static_assert(sizeof(ayken_sched_mailbox_t) == 64, ...)`
2. `offsetof` checks for each frozen field
3. alignment checks (`_Alignof(...) == 64`)

These checks SHOULD fail build on accidental layout changes.

Current implementation:

1. `kernel/sched/sched_mailbox_abi_sanity.c`

### 3.2 JSON Mirror Consistency

When ABI changes intentionally:

1. bump version in header
2. update `constitution/abi_mailbox.json`
3. update freeze/governance docs
4. update gate scripts and tests

All must land in one change set.

### 3.3 Marker Contract Consistency

Proof marker renames/shape changes MUST include:

1. script updates (`scripts/ci/gate_4_policy_accept.sh`, `...gate_4_5...`)
2. parser/test updates in `tools/ci` where applicable
3. migration note in PR description

## 4. Review Checklist (PR-Level)

For any mailbox/proof PR:

1. ABI layout unchanged or version-bumped?
2. owner/fail-closed semantics preserved?
3. Gate-4 + Gate-4.5 deterministic regression green?
4. marker ordering invariants preserved?
5. deterministic exit constraints preserved (`proof_done`, exit code set)?

If any answer is "no", PR MUST include explicit rationale.

## 5. Suggested CI Additions (Follow-up)

Optional but recommended additions:

1. dedicated ABI layout check target (header parse + offsets)
2. marker schema smoke check (regex contract)
3. strict check that v1 docs/hash match expected baseline for protected branches

## 6. State Machine Reference

Freeze lifecycle:

```text
PUBLISHED -> VALIDATED -> CONSUMED -> APPLIED
```

Hardening objective:

1. prove each transition boundary remains observable and fail-closed.

## 7. C2 Transition Rule

C2 multi-owner arbitration MUST NOT mutate v1 silently.

Required:

1. new protocol version/spec
2. updated invariants and gates
3. explicit migration path from C1 baseline

# Mailbox Protocol v2 Capabilities Contract

**Status:** NORMATIVE  
**Authority:** Kenan AY (single maintainer; governance decision `docs/architecture-board/decisions/20260525-single-maintainer-authority-model.md`)
**Date:** 2026-03-06  
**Created by:** Kenan AY  
**Maintained by:** Kenan AY  
**Last Edited by:** Kenan AY  
**Gelistiren:** Kenan AY  
**Olusturan:** Kenan AY  
**Duzenleyen:** Kenan AY

---

## 1. Purpose

This document defines P11-01 mailbox capability validation for Ring3 scheduler
proposals using fail-closed semantics.

Scope:
- capability envelope checks for mailbox proposals
- standardized reject reason mapping
- negative-matrix CI evidence requirements

Out of scope:
- mailbox ABI layout mutation
- policy/arbitration logic in Ring0
- C2 multi-owner consensus behavior

---

## 2. Compatibility with v1 Freeze

v1 ABI freeze remains authoritative:
- `docs/governance/MAILBOX_PROTOCOL_V1_FREEZE.md`
- `kernel/include/sched_mailbox_abi.h`

This v2 capability contract MUST NOT change:
- struct size
- field offsets
- alignment

Contract extension is layered over existing `flags` and `reserved` fields.

---

## 3. Capability Envelope Fields

Normative flag bits:
- `AYKEN_SCHED_MB_FLAG_CAP_CHECK_REQUIRED`
- `AYKEN_SCHED_MB_FLAG_SIG_VALID`
- `AYKEN_SCHED_MB_FLAG_CAP_PRESENT`
- `AYKEN_SCHED_MB_FLAG_BUDGET_OK`

Normative budget bound:
- `AYKEN_SCHED_MB_CAP_BUDGET_MAX`

`reserved` field semantics:
- interpreted as optional budget hint for validation
- `reserved > AYKEN_SCHED_MB_CAP_BUDGET_MAX` MUST reject

---

## 4. Standard Reject Reasons

Canonical reason aliases:
- `REJ_BAD_SIG`
- `REJ_CAP_MISSING`
- `REJ_BUDGET_EXCEEDED`
- `REJ_INVALID_PID`

Mapping rule:
- aliases MUST map to `ayken_sched_reject_reason_t` values in
  `kernel/include/sched_mailbox_abi.h`

---

## 5. Ring0 Validation Requirements

Ring0 validator (`sched_mailbox_validate_ring3`) MUST enforce:
1. fail-closed on invalid signature -> `REJ_BAD_SIG`
2. fail-closed on missing capability proof -> `REJ_CAP_MISSING`
3. fail-closed on missing/invalid budget envelope -> `REJ_BUDGET_EXCEEDED`
4. fail-closed on invalid PID -> `REJ_INVALID_PID`

Compatibility mode:
- If capability enforcement is not requested, legacy v1 behavior MAY continue.
- If strict capability enforcement is enabled by build knob, capability
  envelope MUST be required for all proposals.

---

## 6. CI Gate Contract

Gate:
- `ci-gate-mailbox-capability-negative`

Evidence directory:
- `evidence/run-<RUN_ID>/gates/mailbox-cap/`

Required artifacts:
- `negative_matrix.json`
- `report.json`
- `violations.txt`

PASS criteria:
1. all required symbols are present in ABI + Ring0 validator sources
2. negative matrix cases produce expected reject reasons
3. report verdict is `PASS`

FAIL criteria:
1. missing reject symbols
2. missing Ring0 capability validation path snippets
3. any matrix case mismatch

---

## 7. Security and Performance Notes

Security:
- validation path MUST remain fail-closed
- malformed envelopes MUST NOT execute
- no policy logic is introduced in Ring0

Performance:
- checks are O(1) bit/field tests
- no dynamic allocation
- no hash/signature cryptography in Ring0 hot path

---

## 8. Change Control

Any change to:
- reject reason aliases
- capability flag semantics
- gate artifact contract

MUST update in same change set:
- this document
- `requirements.md`
- `design.md`
- `tasks.md`

and MUST include `Documentation Delta` in PR body.

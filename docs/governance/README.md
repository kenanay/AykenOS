# Governance Documents

This directory contains governance-facing documents that define enforcement
boundaries used by CI gates.

Authoritative references:

- `CONSTITUTION_BOUNDARY.md`: constitutional vs tier-3 boundary contract.
- `MAILBOX_PROTOCOL_V1_FREEZE.md`: frozen scheduler mailbox protocol v1
  (single-authority C1 baseline + proof contracts).
- `MAILBOX_PROTOCOL_V2_C2_REVIEW_FREEZE_CANDIDATE.md`: C2 multi-owner
  arbitration review-freeze candidate (`non-normative`, pre-v2-freeze).
- `SCHEDULER_OWNER_HANDOFF_REAP_CANDIDATE.md`: review candidate for eventual
  scheduler-owner lifecycle closure without violating mailbox v1 fail-closed
  authority rules.
- `SCHEDULER_OWNER_HANDOFF_RATIFICATION_OPTIONS.md`: narrow comparison between
  the only two valid ratification paths for owner handoff/reap.
- `MAILBOX_V1_OWNER_TRANSFER_EXCEPTION.md`: ratified narrow exception for
  single-owner transfer on top of mailbox v1/C1.
- `PHASE10C_C2_STRICT_INVARIANTS.md`: formal strict invariant set and CI
  enforcement mapping for Phase10-C C2 checks.
- `MAILBOX_ABI_HARDENING_NOTES.md`: technical hardening checklist for ABI and
  marker-contract drift prevention.
- `ABI_EXCEPTION_COMPLETION_HANDOFF.md`: ratified narrow ABI exception for the
  explicit execution completion surface.
- `NAMING_CONVENTION_V1.md`: forward-only naming freeze for new execution-path
  additions with scoped CI enforcement.
- `ci-gate-constitutional`: hard constitutional lock (contract surface only).
- `ci-gate-governance-policy`: Tier-3 policy checks (source/AHS/waiver).
- `ci-gate-naming-convention`: diff-scoped naming freeze enforcement for new
  execution-path additions.
- `../../constitution/ARCHITECTURE_GOVERNANCE.md`: tier model and non-negotiable rules.
- `../../constitution/drift_blocking_activation.md`: Phase-9 drift blocking activation protocol.
- `../../constitution/abdf_context.md`: canonical drift context hash inputs.
- `../../constitution/drift_history_policy.md`: history retention and mutation policy.

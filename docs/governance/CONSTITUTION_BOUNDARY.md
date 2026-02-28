# Constitutional Boundary Lock

This document defines the enforced boundary between constitutional contracts
and evolvable governance policy.

## Constitutional Surface (Hard Lock)

The following surfaces are constitutional and fail-closed:

- ABI contract:
  - struct layout, offsets, alignment, append-only rules, semver bump policy.
- Marker contract:
  - marker identity, marker format/schema, source anchors, version bump policy.

These are enforced by Gate-5A and Gate-5B.

## Non-Constitutional Governance Surface (Tier-3)

The following controls are governance policy, not constitutional law:

- Proof scorer (behavioral invariants, fail-closed within Gate-6 scope).
- Phase10-C C2 strict invariants:
  - `docs/governance/PHASE10C_C2_STRICT_INVARIANTS.md`
  - enforced as governance fail-closed checks in freeze chain when strict mode
    is active.
- Envelope policy (threshold-based regression guard, phase/profile controlled).
- Drift telemetry (distribution/rolling analysis, phase/profile controlled).
- Source deny/allow scans, waiver audits, and AHS/AHTS threshold policy.

Tier-3 policy is allowed to evolve without redefining Tier-1/Tier-2 contracts.

## Non-Negotiable Separation Rules

- Envelope and drift are separate systems:
  - envelope = explicit threshold boundary checks
  - drift = statistical anomaly telemetry
- Source/waiver/AHS checks are policy checks and must not expand constitutional
  fail surface unless explicitly enabled for review.
- Drift is non-blocking in Phase 7 and Phase 8.
- Drift blocking is never auto-enabled from telemetry alone.
- Baseline/history auto-rewrite is forbidden.

## Activation and Authority

- Phase-9 drift blocking activation must follow:
  - `constitution/drift_blocking_activation.md`
- Drift allowlist policy must follow:
  - `constitution/drift_blocking_allowlist.json`
- Canonical context hash input set must follow:
  - `constitution/abdf_context.md`
- History retention and mutation policy must follow:
  - `constitution/drift_history_policy.md`

Authority namespace rule:
- Drift authority hash is derived from toolchain/runtime fingerprint and optional
  salt.
- CI salt must be repository-scoped (`PERF_AUTHORITY_SALT=${{ github.repository }}`) so forks start with independent state.

## CI Freeze Split

- Freeze CI keeps constitutional checks in `ci-gate-constitutional`.
- Policy checks run in dedicated `ci-gate-governance-policy`.
- Policy gate must not expand constitutional fail surface in default freeze
  path.

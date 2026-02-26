# AykenOS Governance Architecture

This document defines the permanent governance split for constitutional and
behavioral controls.

Operational boundary language for CI/doc enforcement is mirrored at:
`docs/governance/CONSTITUTION_BOUNDARY.md`.

## Layer Contract

| Layer | Scope | Gate(s) | Blocking Policy |
| --- | --- | --- | --- |
| Tier-1 Permanent Constitution | ABI layout, semver contract, privilege/export boundary | Gate-5A (`ci-gate-structural-abi`) | Always fail-closed |
| Tier-2 Phase-Scoped Structural Constitution | Marker identity/format and source anchors | Gate-5B (`ci-gate-runtime-marker-contract`) | Default fail-closed, explicit enforcement toggle |
| Tier-3 Behavioral Governance | Proof scoring, envelope guardrails, drift telemetry | Gate-6 (`ci-gate-behavioral-suite`) | Phase/profile controlled |

## Tier-3 Sub-Layers

- Proof scorer:
  - Role: invariant checks.
  - Policy: fail-closed.
- Envelope:
  - Role: threshold-based regression guard.
  - Policy: warn-first in early rollout, selective fail in stable phases.
- Drift detector:
  - Role: distribution/rolling telemetry.
  - Policy: non-blocking in Phase 7/8, controlled blocking earliest in Phase 9.

## Non-Negotiable Rules

- Drift is not constitutional and cannot be promoted into Tier-1/Tier-2 rules.
- Drift and envelope remain separate mechanisms.
- AHS/AHTS thresholds and waiver/source scan policy are Tier-3 governance, not
  constitutional invariants.
- Automatic baseline/history rewrite is forbidden.
- Any blocking drift policy requires explicit config and evidence-backed review.
- Drift blocking activation must follow
  `constitution/drift_blocking_activation.md`.

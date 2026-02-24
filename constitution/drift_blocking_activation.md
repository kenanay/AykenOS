# Drift Blocking Activation Protocol (Phase 9+)

This document defines who can activate drift blocking, under which evidence,
and with which rollback controls.

## Scope

- Applies only to Gate-6 Tier-3 drift policy.
- Does not change Tier-1/Tier-2 constitutional rules.
- Drift blocking is disabled by default (`enabled=false`).

## Separation Rule

- Envelope:
  - explicit threshold guard
  - deterministic boundary checks
- Drift:
  - statistical anomaly detector over rolling history
  - context-scoped via `context_key`

These mechanisms must remain separate.

## Activation Preconditions

All conditions are required before enabling drift blocking:

1. Phase is `>= 9`.
2. Profile is explicitly allowlisted (default: `validation`).
3. Drift profile status is `enforce`.
4. Minimum history exists for each enforced metric:
   - at least `N >= 30` samples in current context window.
5. Evidence quality checks pass:
   - context metadata complete
   - no mixed run classes within context stream
   - marker schema version stable for window.

## Statistical Confidence Guard

- Blocking thresholds must be defined per policy:
  - `warn_threshold` (consecutive WARN)
  - `fail_threshold` (consecutive FAIL)
- One-run drift spikes must not block CI.
- Recommended initial policy:
  - WARN blocking: disabled or high threshold (`>= 5`)
  - FAIL blocking: conservative threshold (`>= 3`)

## Governance and Review

Enabling drift blocking requires:

1. PR with explicit policy diff.
2. Attached evidence diff from recent runs.
3. Reviewer sign-off from runtime governance owners.
4. Rollback command/path documented in PR.

## Emergency Disable

If false-positive blocking is detected:

1. Set `suite.defaults.drift_blocking_policy.enabled=false`.
2. Keep drift telemetry active (do not disable detector entirely).
3. Open postmortem with:
   - context key
   - detector outputs
   - threshold/persistence values
   - corrective action.

## Non-Negotiable Rules

- No auto-activation based on telemetry alone.
- No auto-threshold rewrite from recent history.
- No direct edits to history artifacts to silence drift.


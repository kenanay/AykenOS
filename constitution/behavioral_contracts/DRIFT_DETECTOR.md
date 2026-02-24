# Gate-6 Drift Detector

Gate-6 drift detector is a Tier-3 observational module for AI-policy/runtime
distribution changes. It is not a constitutional lock.

## Layer Contract

- Proof scorer:
  - Purpose: architectural/behavioral invariants
  - Mode: fail-closed
- Envelope:
  - Purpose: threshold-based regression guardrails
  - Mode: phase/profile controlled (`warn` then selective `fail`)
- Drift detector:
  - Purpose: distribution/rolling drift observation
  - Mode: telemetry-first (non-blocking in Phase 7/8)

## Non-Negotiable Rules

- Baseline auto-update is forbidden.
  - Baseline changes require PR + evidence diff + review.
- Drift detector is not constitutional.
  - Drift stays in Tier-3 and must not be promoted to Tier-1/Tier-2 invariants.

## Phase Policy

- Phase 7:
  - Drift is telemetry-only.
  - `INFO/WARN/FAIL` may be reported, gate result is not blocked by drift.
- Phase 8:
  - Drift remains telemetry-first by default.
  - Phase guard keeps drift non-blocking for gate verdict.
  - Envelope may hard-fail selected stable metrics.
- Phase 9:
  - Optional persistent-drift fail policy.
  - Fail only when the same context key crosses threshold for `N` consecutive runs.
  - Activation requires explicit `drift_blocking_policy.enabled=true`.
  - Activation protocol is defined in `constitution/drift_blocking_activation.md`.

## Context Key Requirement

To avoid false drift alarms, context key must include at least:

- `kernel_profile`
- `workload_id`
- `ai_policy_hash` (or policy semver/hash equivalent)
- `marker_schema_version`
- `run_class` (`ci`, `local`, `lab`, `perf`)

Canonical normalization and hash rules are defined in:

- `constitution/abdf_context.md`

## ABDF/BCIB Alignment

Drift interpretation requires runtime context transport. ABDF/BCIB metadata
must provide deterministic identifiers so detector output can be compared
within matching contexts only.

History retention and mutation rules are defined in:

- `constitution/drift_history_policy.md`

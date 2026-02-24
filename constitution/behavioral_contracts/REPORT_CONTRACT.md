# Gate-6 Report Contract

This file freezes the Gate-6 report structure used by CI summarization and
forensic evidence analysis.

## Top-Level Contract

Gate-6 report (`report.json`) contains:

- `gate`, `tier`, `run_id`, `time_utc`
- `kernel_profile`, `phase`, `suite_version`
- `strict_mode`, `proof_modes`, `proofs`
- `signals`, `metrics`
- `envelope`
- `drift`
- `drift_policy`
- `verdict`, `violations`, `warnings`
- `violations_count`, `warnings_count`

## Drift Block Contract

`drift` is schema-governed by `drift_schema.json` and must include:

- `enabled`, `status`, `profile`, `phase`
- `context`, `context_key`
- `window`, `metrics`
- `detectors`
- `persistence`
- `verdict`

## Drift Policy Block Contract

`drift_policy` documents enforcement decisions and must include:

- `enabled` (config switch)
- `phase_min` (earliest blocking phase)
- `profiles` (eligible profiles)
- `require_status` (status gate)
- `warn_threshold`, `fail_threshold`
- `phase_guard_non_blocking`
- `eligible_phase`, `eligible_profile`
- `drift_status`, `drift_verdict`
- `consecutive_warn`, `consecutive_fail`
- `blocking_triggered`
- `reason`

## Phase Guard Rule

In Phase 7 and Phase 8, drift is telemetry-only by policy and must not change
Gate-6 exit behavior.


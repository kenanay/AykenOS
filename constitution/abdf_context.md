# ABDF Drift Context Contract

This document defines the canonical context fields used by Gate-6 drift
telemetry and `context_key` hashing.

## Canonical Field Set

Required fields:

- `kernel_profile`
- `ai_policy_hash`
- `workload_id`
- `marker_schema_version`
- `run_class`

These fields are the hash input set for drift context identity.

## Normalization Rules

- All context values are trimmed (`strip()`).
- `kernel_profile` and `run_class` are lower-cased.
- Missing values are treated as empty strings.
- `context_key` is generated from canonical JSON:
  - sort keys enabled
  - compact separators `(",", ":")`
  - SHA-256 digest of UTF-8 encoded canonical payload

## Deterministic Hash Contract

`context_key` must be stable for identical canonical context values.

Hash input order is determined by canonical sorted JSON keys, not insertion
order from runtime code.

## Drift Interpretation Boundary

Drift comparisons are valid only within the same `context_key`.
If the context changes, a new history stream is expected.

## Reset Semantics (Implicit by Context)

History is considered logically reset when any required context field changes,
including:

- new `ai_policy_hash`
- new `workload_id`
- new `marker_schema_version`
- different `kernel_profile`
- different `run_class`

No manual history rewrite is required for these transitions.


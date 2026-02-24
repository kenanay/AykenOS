# Drift History Policy

This policy governs Gate-6 drift history storage and retention.

## Storage Location

- Path: `evidence/history/<profile>/<context_key>.jsonl`
- Format: append-only JSON Lines
- Scope: runtime evidence artifact, not source-controlled baseline

## Required Guarantees

- History files are never auto-committed to the repository.
- History writes are append-only.
- Baseline auto-update is forbidden.
- Drift history is context-scoped through `context_key`.

## Retention Model

- Phase 7/8 default: append-only storage.
- Growth management: offline prune only (no in-gate mutation).
- Prune actions must preserve auditability (evidence diff + review).

## Allowed Maintenance Actions

- Remove obsolete contexts after policy/workload/schema migration windows.
- Archive old history outside default CI artifact retention window.
- Keep prune tooling non-blocking and outside Gate-6 scoring path.

## Forbidden Actions

- Rewriting history in-place to force lower drift scores.
- Automatic baseline recalibration without review.
- Mixing CI and local histories under a shared context key.


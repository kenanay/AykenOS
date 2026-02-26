# Drift Activation Protocol

This guide defines how drift blocking is activated and operated in Phase 9+.

## Scope

- Activation enforcement gate: `make ci-gate-drift-activation`
- Drift detection and persistence gate: `make ci-gate-performance`
- Policy sources:
  - `constitution/drift_blocking_activation.md`
  - `constitution/drift_blocking_allowlist.json`
  - `docs/roadmap/CURRENT_PHASE`

## Activation Rules

- Phase `< phase_minimum`: `SKIP`
- Phase `>= phase_minimum` and `enabled=false`: `FAIL`
- Phase `>= phase_minimum` and `enabled=true`: `PASS`
- Activation is explicit only; auto-enable is forbidden.

## N-Run Persistence

- Runtime state file: `.ci-state/drift_state.json` (gitignored)
- CI cache key: `drift-state-${authority_hash}`
- Authority hash input set:
  - `clang --version` first line
  - `qemu-system-x86_64 --version` first line
  - `PERF_AUTHORITY_SALT` (CI: `${{ github.repository }}`)
- Git SHA is excluded from authority hash so counters persist across commits.

## Allowlist Behavior

- Allowlist file must match schema:
  - `{"version":"1.0","metrics":[...]}`
- Allowlisted metric regressions:
  - are logged into `allowlist_bypass.txt`
  - do not add blocking violations
- Non-allowlisted regressions keep fail-closed behavior.

## Evidence

- Drift activation gate:
  - `evidence/run-<RUN_ID>/gates/drift-activation/report.json`
  - `evidence/run-<RUN_ID>/gates/drift-activation/meta.txt`
  - `evidence/run-<RUN_ID>/gates/drift-activation/violations.txt`
- Performance gate:
  - `evidence/run-<RUN_ID>/gates/performance/report.json`
  - `evidence/run-<RUN_ID>/gates/performance/drift_counters.txt`
  - `evidence/run-<RUN_ID>/gates/performance/allowlist_bypass.txt`

## Fork Behavior

- Fork and upstream are isolated by:
  - repository-scoped cache namespace (GitHub Actions)
  - repo-scoped salt (`PERF_AUTHORITY_SALT=${{ github.repository }}`)
- Result: fork starts with fresh drift state by design.

## CI Artifact Persistence Model

- Policy stays in repo (constitution/docs).
- Runtime counters stay in cache artifact (`.ci-state`).
- Evidence stays under `evidence/run-<RUN_ID>/...`.
- This separation prevents policy/state mixing and merge-noise.

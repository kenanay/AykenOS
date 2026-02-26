# Drift Evidence Dashboard Specification

## Goal

Track drift activation and persistence health across CI runs.

## Minimum Panels

1. Gate verdict timeline
   - source: `gates/drift-activation/report.json`
2. Regression trend by metric
   - source: `gates/performance/baseline.diff.txt`
3. Drift counter progression
   - source: `gates/performance/drift_counters.txt`
4. Allowlist bypass events
   - source: `gates/performance/allowlist_bypass.txt`
5. Authority hash changes
   - source: `gates/performance/meta.txt` (`drift_authority_hash`)

## Mandatory Labels

- `run_id`
- `git_sha`
- `current_phase`
- `baseline_authority`
- `drift_authority_hash`
- `drift_allowlist_bypass_count`

## Alert Inputs

- repeated `FAIL` verdicts in performance gate
- sudden authority hash churn
- increase in allowlist bypass count

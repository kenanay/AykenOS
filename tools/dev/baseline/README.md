# Baseline Dev Scripts

This directory contains helper scripts for validating and monitoring the
performance baseline workflow from a developer environment.

These scripts do not replace CI. They are convenience tools for checking
workflow/run state and baseline init progress.

## Scripts

- `check_baseline.sh`: Check a specific baseline init run result.
- `check_baseline_init.sh`: List recent baseline-init workflow runs.
- `check_ci.sh`: Check a specific CI run result.
- `monitor_baseline.sh`: Poll a baseline init run until completion.

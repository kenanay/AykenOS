# Drift Gate Alert Rules

## Severity Levels

- `P1`: Freeze pipeline blocked for production branch.
- `P2`: Repeated non-allowlisted regression failures.
- `P3`: Governance hygiene anomalies (allowlist misuse, authority churn).

## Rules

1. `P1` Freeze Block
   - Condition: `ci-freeze` fails with performance or drift-activation gate
   - Window: immediate
   - Action: page owning team, attach evidence links

2. `P2` Regression Persistence
   - Condition: same non-allowlisted metric fails in 3 consecutive runs
   - Window: rolling 24h
   - Action: open incident, require regression triage

3. `P3` Allowlist Growth
   - Condition: allowlist bypass count increases above baseline threshold
   - Window: rolling 7d
   - Action: governance review of allowlist entries

4. `P3` Authority Churn
   - Condition: authority hash changes unexpectedly between adjacent runs
   - Window: rolling 24h
   - Action: verify runner image/toolchain/salt changes

## Required Alert Payload

- repository
- branch / PR
- run_id
- gate verdict
- violated metric(s)
- evidence artifact path

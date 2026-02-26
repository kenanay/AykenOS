# Drift Activation Rollback Procedure

Use this only for approved emergency rollback.

## Trigger Conditions

- Reproducible false-positive regressions block critical fixes.
- CI environment drift causes systemic misclassification.
- Governance decision explicitly authorizes temporary rollback.

## Rollback Steps

1. Edit `constitution/drift_blocking_activation.md`:
   - set `enabled: false`
2. Keep `phase_minimum` unchanged unless governance board approves otherwise.
3. Commit with explicit rollback reason and incident reference.
4. Run:
   - `./scripts/ci/test_drift_activation_gate.sh`
   - `make ci-gate-drift-activation`
5. Push and verify CI gate behavior (`SKIP`/`FAIL` transitions as expected).

## Post-Rollback Requirements

- Open follow-up issue with root cause and re-activation criteria.
- Preserve evidence artifacts from failing runs.
- Define re-enable date/condition before closing incident.

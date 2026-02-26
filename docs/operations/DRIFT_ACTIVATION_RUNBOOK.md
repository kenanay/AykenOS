# Drift Activation Runbook

## Preconditions

- Phase is at or above configured minimum.
- Activation policy file reviewed and approved.
- Allowlist reviewed for intentional bypass entries only.

## Standard Procedure

1. Validate current phase and activation files.
2. Run local test suite:
   - `./scripts/ci/test_lib_phase.sh`
   - `./scripts/ci/test_drift_activation_gate.sh`
   - `./scripts/ci/test_drift_persistence.sh`
   - `./scripts/ci/test_drift_allowlist.sh`
   - `./scripts/ci/test_drift_properties.sh`
3. Run `make ci-gate-drift-activation`.
4. Run `make ci-gate-performance` and inspect:
   - `drift_counters.txt`
   - `allowlist_bypass.txt`
   - `report.json`
5. Push and run CI freeze workflow.
6. Confirm evidence artifacts and final verdict.

## Incident Notes

- If activation gate fails at Phase 9+: verify `enabled` field first.
- If performance gate fails: inspect non-allowlisted regression lines.
- If counters reset unexpectedly: inspect `drift_authority_hash` and CI salt.

# Phase 9 Transition Checklist (Drift Blocking)

- [ ] Confirm `docs/roadmap/CURRENT_PHASE` is set to `CURRENT_PHASE=9`.
- [ ] Confirm `constitution/drift_blocking_activation.md` has:
  - [ ] `enabled: true`
  - [ ] `phase_minimum: 9`
- [ ] Confirm allowlist file exists and validates:
  - [ ] `constitution/drift_blocking_allowlist.json`
- [ ] Run local gate tests:
  - [ ] `./scripts/ci/test_lib_phase.sh`
  - [ ] `./scripts/ci/test_drift_activation_gate.sh`
  - [ ] `./scripts/ci/test_drift_persistence.sh`
  - [ ] `./scripts/ci/test_drift_allowlist.sh`
  - [ ] `./scripts/ci/test_drift_properties.sh`
- [ ] Run freeze workflow and verify:
  - [ ] drift-activation gate evidence generated
  - [ ] performance gate reads/writes drift cache
  - [ ] fail-closed behavior preserved for non-allowlisted regressions
- [ ] Record decision and rationale in PR description.

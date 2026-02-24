# Drift Activation Phase-9 Tasks

**Feature:** Drift Blocking Activation Protocol  
**Status:** DRAFT

## Task List

- [ ] 1. Foundation Setup
  - [ ] 1.1 Create phase detection library (`scripts/ci/lib-phase.sh`)
  - [ ] 1.2 Create phase number document (`docs/roadmap/CURRENT_PHASE`, format: `CURRENT_PHASE=8`)
  - [ ] 1.3 Create activation state document (`constitution/drift_blocking_activation.md`)
  - [ ] 1.4 Create allowlist document (`constitution/drift_blocking_allowlist.json`)
  - [ ] 1.5 Add `.ci-state/` to `.gitignore` (runtime state not committed)

- [ ] 2. CI Gate Implementation (Activation Requirement Only)
  - [ ] 2.1 Implement `scripts/ci/gate_drift_activation.sh` (minimal: requirement enforcement)
  - [ ] 2.2 Add gate to Makefile (`ci-gate-drift-activation` target)
  - [ ] 2.3 Integrate gate into `ci-freeze` chain
  - [ ] 2.4 Integrate gate into `ci-freeze-local` chain
  - [ ] 2.5 Add gate to `.PHONY` targets in Makefile
  - [ ] 2.6 Document gate responsibility (activation only, not drift detection)

- [ ] 3. Phase Detection Logic
  - [ ] 3.1 Implement `get_current_phase()` function
  - [ ] 3.2 Add phase file validation (existence, format)
  - [ ] 3.3 Add phase number parsing (regex extraction)
  - [ ] 3.4 Add error handling for missing/invalid phase file
  - [ ] 3.5 Write unit tests for phase detection

- [ ] 4. Activation State Logic
  - [ ] 4.1 Implement activation state parsing (YAML front-matter)
  - [ ] 4.2 Add validation for `enabled` field (boolean)
  - [ ] 4.3 Add validation for `phase_minimum` field (integer)
  - [ ] 4.4 Add default values for missing fields
  - [ ] 4.5 Write unit tests for activation state parsing

- [ ] 5. Enforcement Logic
  - [ ] 5.1 Implement phase comparison logic
  - [ ] 5.2 Implement verdict determination (PASS/FAIL/SKIP)
  - [ ] 5.3 Implement violation detection
  - [ ] 5.4 Add reason codes for each verdict
  - [ ] 5.5 Write unit tests for enforcement logic

- [ ] 6. Evidence Generation
  - [ ] 6.1 Implement `report.json` generation
  - [ ] 6.2 Implement `meta.txt` generation
  - [ ] 6.3 Implement `violations.txt` generation
  - [ ] 6.4 Add timestamp and git SHA to evidence
  - [ ] 6.5 Write unit tests for evidence generation

- [ ] 7. N-Run Persistence (Performance Gate Integration)
  - [ ] 7.1 Create drift persistence library (`scripts/ci/lib-drift-persistence.sh`)
  - [ ] 7.2 Implement `compute_authority_hash()` function
  - [ ] 7.3 Implement `load_drift_state()` function (from `.ci-state/`)
  - [ ] 7.4 Implement `save_drift_state()` function (to `.ci-state/`)
  - [ ] 7.5 Implement `increment_drift_counter()` function
  - [ ] 7.6 Implement `check_drift_threshold()` function
  - [ ] 7.7 Add authority hash reset logic
  - [ ] 7.8 Integrate persistence logic into `ci-gate-performance` (NOT drift-activation)
  - [ ] 7.9 Add CI artifact cache/restore workflow steps
  - [ ] 7.10 Write unit tests for persistence logic

- [ ] 8. Allowlist Mechanism (Performance Gate Integration)
  - [ ] 8.1 Implement `is_metric_allowlisted()` function
  - [ ] 8.2 Add allowlist validation (JSON schema)
  - [ ] 8.3 Integrate allowlist check into `ci-gate-performance` (NOT drift-activation)
  - [ ] 8.4 Add allowlist bypass logging
  - [ ] 8.5 Write unit tests for allowlist logic

- [ ] 9. Integration Testing
  - [ ] 9.1 Test gate with Phase < 9 (expect SKIP)
  - [ ] 9.2 Test gate with Phase 9, disabled (expect FAIL)
  - [ ] 9.3 Test gate with Phase 9, enabled (expect PASS)
  - [ ] 9.4 Test gate with missing phase file (expect error)
  - [ ] 9.5 Test gate with missing activation file (expect error)
  - [ ] 9.6 Test gate with invalid phase number (expect error)
  - [ ] 9.7 Test gate with invalid activation state (expect default)
  - [ ] 9.8 Test full `ci-freeze` chain with new gate
  - [ ] 9.9 Test full `ci-freeze-local` chain with new gate

- [ ] 10. Documentation
  - [ ] 10.1 Update `ARCHITECTURE_FREEZE.md` (add drift activation gate)
  - [ ] 10.2 Update `docs/governance/CONSTITUTION_BOUNDARY.md` (governance layer)
  - [ ] 10.3 Update `Makefile` help text (add drift activation gate)
  - [ ] 10.4 Create activation protocol guide (`docs/operations/DRIFT_ACTIVATION.md`)
  - [ ] 10.5 Update `README.md` (add drift activation to gate list)

- [ ] 11. CI Workflow Integration
  - [ ] 11.1 Update `.github/workflows/ci-freeze.yml` (add artifact cache/restore)
  - [ ] 11.2 Add `AUTHORITY_HASH` environment variable computation
  - [ ] 11.3 Add drift state artifact restore step (before performance gate)
  - [ ] 11.4 Add drift state artifact save step (after performance gate)
  - [ ] 11.5 Verify gate runs in CI environment
  - [ ] 11.6 Verify evidence committed to repository
  - [ ] 11.7 Verify drift state NOT committed to repository
  - [ ] 11.8 Verify gate failure blocks PR merge

- [ ] 12. Property-Based Testing
  - [ ] 12.1 Write property test: phase-driven enforcement
  - [ ] 12.2 Write property test: explicit activation
  - [ ] 12.3 Write property test: evidence immutability
  - [ ] 12.4 Write property test: N-run persistence
  - [ ] 12.5 Write property test: authority hash reset

- [ ] 13. Rollout Preparation
  - [ ] 13.1 Create Phase 9 transition checklist
  - [ ] 13.2 Create drift activation runbook
  - [ ] 13.3 Create rollback procedure
  - [ ] 13.4 Create monitoring dashboard (evidence tracking)
  - [ ] 13.5 Create alert rules (gate failure notifications)
  - [ ] 13.6 Document fork behavior (fresh start, independent governance)
  - [ ] 13.7 Document CI artifact persistence model

- [ ] 14. Final Validation
  - [ ] 14.1 Run full `ci-freeze` suite (expect all gates PASS)
  - [ ] 14.2 Run `ci-freeze-local` suite (expect all gates PASS)
  - [ ] 14.3 Verify evidence integrity (hygiene gate)
  - [ ] 14.4 Verify no constitutional violations
  - [ ] 14.5 Verify no governance policy violations
  - [ ] 14.6 Create completion report

## Task Dependencies

```
1 (Foundation) → 2 (Gate Implementation)
1 (Foundation) → 3 (Phase Detection)
1 (Foundation) → 4 (Activation State)
3 (Phase Detection) → 5 (Enforcement)
4 (Activation State) → 5 (Enforcement)
5 (Enforcement) → 6 (Evidence)
1 (Foundation) → 7 (N-Run Persistence)
1 (Foundation) → 8 (Allowlist)
2,3,4,5,6 → 9 (Integration Testing)
9 (Integration Testing) → 10 (Documentation)
10 (Documentation) → 11 (CI Workflow)
9 (Integration Testing) → 12 (Property Testing)
11 (CI Workflow) → 13 (Rollout)
12 (Property Testing) → 14 (Final Validation)
```

## Estimated Effort

- Foundation Setup: 2 hours
- CI Gate Implementation: 3 hours
- Phase Detection Logic: 1 hour
- Activation State Logic: 1 hour
- Enforcement Logic: 2 hours
- Evidence Generation: 1 hour
- N-Run Persistence: 3 hours
- Allowlist Mechanism: 2 hours
- Integration Testing: 4 hours
- Documentation: 2 hours
- CI Workflow Integration: 1 hour
- Property-Based Testing: 3 hours
- Rollout Preparation: 2 hours
- Final Validation: 2 hours

**Total:** ~29 hours

## Success Criteria

- All tasks completed
- All tests passing
- All gates integrated into CI
- Evidence integrity verified
- Documentation complete
- Rollout plan ready
- Phase 9 transition prepared

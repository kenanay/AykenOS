# Drift Activation Phase-9 Tasks

**Feature:** Drift Blocking Activation Protocol  
**Status:** DRAFT

## Task List

- [x] 1. Foundation Setup
  - [x] 1.1 Create phase detection library (`scripts/ci/lib-phase.sh`)
  - [x] 1.2 Create phase number document (`docs/roadmap/CURRENT_PHASE`, format: `CURRENT_PHASE=8`)
  - [x] 1.3 Create activation state document (`constitution/drift_blocking_activation.md`)
  - [x] 1.4 Create allowlist document (`constitution/drift_blocking_allowlist.json`)
  - [x] 1.5 Add `.ci-state/` to `.gitignore` (runtime state not committed)

- [x] 2. CI Gate Implementation (Activation Requirement Only)
  - [x] 2.1 Implement `scripts/ci/gate_drift_activation.sh` (minimal: requirement enforcement)
  - [x] 2.2 Add gate to Makefile (`ci-gate-drift-activation` target)
  - [x] 2.3 Integrate gate into `ci-freeze` chain
  - [x] 2.4 Integrate gate into `ci-freeze-local` chain
  - [x] 2.5 Add gate to `.PHONY` targets in Makefile
  - [x] 2.6 Document gate responsibility (activation only, not drift detection)

- [x] 3. Phase Detection Logic
  - [x] 3.1 Implement `get_current_phase()` function
  - [x] 3.2 Add phase file validation (existence, format)
  - [x] 3.3 Add phase number parsing (regex extraction)
  - [x] 3.4 Add error handling for missing/invalid phase file
  - [x] 3.5 Write unit tests for phase detection

- [x] 4. Activation State Logic
  - [x] 4.1 Implement activation state parsing (YAML front-matter)
  - [x] 4.2 Add validation for `enabled` field (boolean)
  - [x] 4.3 Add validation for `phase_minimum` field (integer)
  - [x] 4.4 Add default values for missing fields
  - [x] 4.5 Write unit tests for activation state parsing

- [x] 5. Enforcement Logic
  - [x] 5.1 Implement phase comparison logic
  - [x] 5.2 Implement verdict determination (PASS/FAIL/SKIP)
  - [x] 5.3 Implement violation detection
  - [x] 5.4 Add reason codes for each verdict
  - [x] 5.5 Write unit tests for enforcement logic

- [x] 6. Evidence Generation
  - [x] 6.1 Implement `report.json` generation
  - [x] 6.2 Implement `meta.txt` generation
  - [x] 6.3 Implement `violations.txt` generation
  - [x] 6.4 Add timestamp and git SHA to evidence
  - [x] 6.5 Write unit tests for evidence generation

- [x] 7. N-Run Persistence (Performance Gate Integration)
  - [x] 7.1 Create drift persistence library (`scripts/ci/lib-drift-persistence.sh`)
  - [x] 7.2 Implement `compute_authority_hash()` function
  - [x] 7.3 Implement `load_drift_state()` function (from `.ci-state/`)
  - [x] 7.4 Implement `save_drift_state()` function (to `.ci-state/`)
  - [x] 7.5 Implement `increment_drift_counter()` function
  - [x] 7.6 Implement `check_drift_threshold()` function
  - [x] 7.7 Add authority hash reset logic
  - [x] 7.8 Integrate persistence logic into `ci-gate-performance` (NOT drift-activation)
  - [x] 7.9 Add CI artifact cache/restore workflow steps
  - [x] 7.10 Write unit tests for persistence logic

- [x] 8. Allowlist Mechanism (Performance Gate Integration)
  - [x] 8.1 Implement `is_metric_allowlisted()` function
  - [x] 8.2 Add allowlist validation (JSON schema)
  - [x] 8.3 Integrate allowlist check into `ci-gate-performance` (NOT drift-activation)
  - [x] 8.4 Add allowlist bypass logging
  - [x] 8.5 Write unit tests for allowlist logic

- [x] 9. Integration Testing
  - [x] 9.1 Test gate with Phase < 9 (expect SKIP)
  - [x] 9.2 Test gate with Phase 9, disabled (expect FAIL)
  - [x] 9.3 Test gate with Phase 9, enabled (expect PASS)
  - [x] 9.4 Test gate with missing phase file (expect error)
  - [x] 9.5 Test gate with missing activation file (expect error)
  - [x] 9.6 Test gate with invalid phase number (expect error)
  - [x] 9.7 Test gate with invalid activation state (expect default)
  - [x] 9.8 Test full `ci-freeze` chain with new gate
  - [x] 9.9 Test full `ci-freeze-local` chain with new gate

- [x] 10. Documentation
  - [x] 10.1 Update `ARCHITECTURE_FREEZE.md` (add drift activation gate)
  - [x] 10.2 Update `docs/governance/CONSTITUTION_BOUNDARY.md` (governance layer)
  - [x] 10.3 Update `Makefile` help text (add drift activation gate)
  - [x] 10.4 Create activation protocol guide (`docs/operations/DRIFT_ACTIVATION.md`)
  - [x] 10.5 Update `README.md` (add drift activation to gate list)

- [-] 11. CI Workflow Integration
  - [x] 11.1 Update `.github/workflows/ci-freeze.yml` (add artifact cache/restore)
  - [x] 11.2 Add `AUTHORITY_HASH` environment variable computation
  - [x] 11.3 Add drift state artifact restore step (before performance gate)
  - [x] 11.4 Add drift state artifact save step (after performance gate)
  - [x] 11.5 Verify gate runs in CI environment
  - [x] 11.6 Verify evidence uploaded as CI artifact (runtime evidence not committed)
  - [x] 11.7 Verify drift state NOT committed to repository
  - [ ] 11.8 Verify gate failure blocks PR merge

- [x] 12. Property-Based Testing
  - [x] 12.1 Write property test: phase-driven enforcement
  - [x] 12.2 Write property test: explicit activation
  - [x] 12.3 Write property test: evidence immutability
  - [x] 12.4 Write property test: N-run persistence
  - [x] 12.5 Write property test: authority hash reset

- [x] 13. Rollout Preparation
  - [x] 13.1 Create Phase 9 transition checklist
  - [x] 13.2 Create drift activation runbook
  - [x] 13.3 Create rollback procedure
  - [x] 13.4 Create monitoring dashboard (evidence tracking)
  - [x] 13.5 Create alert rules (gate failure notifications)
  - [x] 13.6 Document fork behavior (fresh start, independent governance)
  - [x] 13.7 Document CI artifact persistence model

- [-] 14. Final Validation
  - [ ] 14.1 Run full `ci-freeze` suite (expect all gates PASS)
  - [x] 14.2 Run `ci-freeze-local` suite (expect all gates PASS)
  - [x] 14.3 Verify evidence integrity (hygiene gate)
  - [x] 14.4 Verify no constitutional violations
  - [x] 14.5 Verify no governance policy violations
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

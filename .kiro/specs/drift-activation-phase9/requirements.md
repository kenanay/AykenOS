# Drift Activation Phase-9 Requirements

**Feature:** Drift Blocking Activation Protocol  
**Phase:** 9 (Governance Stabilization)  
**Authority:** ARCHITECTURE_FREEZE.md  
**Status:** DRAFT

## Overview

Implement phase-driven drift blocking activation protocol with explicit governance enforcement. Drift telemetry transitions from passive observation to active blocking when system reaches Phase 9 maturity.

## Core Principle

**Explicit Activation, Phase-Guarded Enforcement**

- Drift blocking NEVER auto-activates
- Phase >= 9 REQUIRES drift blocking to be enabled
- CI gate enforces requirement (fail-closed)
- Developer explicitly enables via constitution document

## User Stories

### 1. Phase-9 Drift Blocking Requirement

**As a** system architect  
**I want** drift blocking to be mandatory in Phase 9+  
**So that** governance maturity enforces deterministic execution discipline

**Acceptance Criteria:**

1.1. Phase detection logic reads current phase from canonical source  
1.2. Phase < 9: drift blocking is optional (CI skip)  
1.3. Phase >= 9 AND drift blocking disabled: CI FAIL  
1.4. Phase >= 9 AND drift blocking enabled: CI PASS  
1.5. Phase number source is `docs/roadmap/CURRENT_PHASE` (simple format: `CURRENT_PHASE=8`)

### 2. Explicit Activation Protocol

**As a** developer  
**I want** to explicitly enable drift blocking  
**So that** activation is conscious and documented

**Acceptance Criteria:**

2.1. Activation state stored in `constitution/drift_blocking_activation.md`  
2.2. Schema includes: `enabled`, `phase_minimum`, `auto_activation_policy`  
2.3. Default state: `enabled: false`  
2.4. Activation requires explicit edit (no auto-enable)  
2.5. Activation change requires git commit (evidence trail)

### 3. CI Gate Enforcement

**As a** CI system  
**I want** to enforce drift blocking requirement in Phase 9+  
**So that** governance policy is fail-closed

**Acceptance Criteria:**

3.1. New gate: `make ci-gate-drift-activation`  
3.2. Gate reads phase number from canonical source  
3.3. Gate reads activation state from constitution document  
3.4. Gate logic:
   - Phase < 9: SKIP (with reason)
   - Phase >= 9 AND enabled=false: FAIL
   - Phase >= 9 AND enabled=true: PASS
3.5. Gate produces evidence in `evidence/run-<RUN_ID>/gates/drift-activation/`  
3.6. Gate integrated into `ci-freeze` and `ci-freeze-local` chains

### 4. Evidence Immutability Guard

**As a** governance system  
**I want** drift activation evidence to be immutable  
**So that** activation history is tamper-proof

**Acceptance Criteria:**

4.1. Evidence includes: phase number, activation state, timestamp, git SHA  
4.2. Evidence format: JSON report + human-readable summary  
4.3. Evidence never modified after creation  
4.4. Evidence committed to repository (append-only)

### 5. N-Run Persistence Policy

**As a** performance regression detector  
**I want** drift blocking to persist across N runs  
**So that** transient noise doesn't trigger false positives

**Acceptance Criteria:**

5.1. Configuration parameter: `drift_blocking_n_run_threshold` (default: 3)  
5.2. Regression must appear in N consecutive runs to block  
5.3. Single-run regression → warning (not block)  
5.4. N-run regression → CI FAIL  
5.5. Counter resets on authority hash change

### 6. Authority Hash Reset Protocol

**As a** baseline authority system  
**I want** drift counters to reset when authority changes  
**So that** legitimate baseline updates don't accumulate false drift

**Acceptance Criteria:**

6.1. Authority hash computed from: git SHA + toolchain version + QEMU version  
6.2. Authority hash stored in drift state file  
6.3. Authority hash mismatch → reset all counters  
6.4. Authority hash match → preserve counters  
6.5. Reset event logged in evidence

### 7. Allowlist Metric Mechanism

**As a** developer  
**I want** to allowlist specific metrics from drift blocking  
**So that** known-variable metrics don't block CI

**Acceptance Criteria:**

7.1. Allowlist stored in `constitution/drift_blocking_allowlist.json`  
7.2. Allowlist schema: `{"version": "1.0", "metrics": ["metric_name_1", "metric_name_2"]}`  
7.3. Allowlisted metrics still logged (not blocked)  
7.4. Allowlist changes require git commit  
7.5. Allowlist bypass logged in evidence

### 8. Fork Independence

**As a** fork maintainer  
**I want** fork to have independent drift state  
**So that** upstream governance doesn't constrain fork development

**Acceptance Criteria:**

8.1. Fork has different git SHA → different authority hash  
8.2. Fork CI artifact key is different from upstream  
8.3. Fork starts with empty drift counters (fresh start)  
8.4. Fork drift state does not transfer from upstream  
8.5. Fork is independent governance instance  
8.6. Fork behavior documented in activation protocol

## Non-Functional Requirements

### Performance
- Gate execution time: < 5 seconds (no QEMU runs)
- Evidence generation: < 1 second
- CI artifact restore/save: < 10 seconds

### Reliability
- Gate must be deterministic (no network calls)
- Gate must be idempotent (re-runnable)
- Authority hash must be stable for same environment

### Security
- No auto-fix or auto-enable
- All policy changes require explicit git commit
- Runtime state never committed to repository
- Evidence integrity enforced by hygiene gate

### Maintainability
- Constitution documents contain policy only (no state)
- Runtime state isolated in CI artifact
- Fork independence guaranteed by authority hash
- Clear separation: policy vs state vs evidence

## Out of Scope

- Drift metric collection (already implemented in performance gate)
- Drift visualization (future work)
- Drift prediction (future work)
- Multi-repository drift correlation (future work)

## Dependencies

- `make ci-gate-performance` (drift detection + N-run persistence)
- `scripts/ci/perf-baseline.lock.json` (baseline authority)
- `constitution/drift_blocking_activation.md` (activation policy)
- `docs/roadmap/CURRENT_PHASE` (phase number, format: `CURRENT_PHASE=8`)
- GitHub Actions cache/artifact (runtime state persistence)
- `.ci-state/drift_state.json` (local runtime state, gitignored)

## Success Metrics

- Phase 9 transition: drift blocking activation enforced
- Zero false positives in first 30 days
- Zero bypass attempts (audit trail clean)
- Evidence integrity: 100% immutable

## References

- `ARCHITECTURE_FREEZE.md`: Determinism requirement
- `constitution/drift_blocking_activation.md`: Activation protocol
- `docs/governance/CONSTITUTION_BOUNDARY.md`: Governance vs constitutional split
- Performance gate: Drift metric collection

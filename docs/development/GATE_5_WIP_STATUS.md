# Gate-5 / Gate-6 Status

**Date:** 2026-02-24  
**Status:** Gate-5 complete, Gate-6 bootstrap complete  
**Model:** Two-layer structural constitution + behavioral proof layer

## Final Architectural Split

### Tier-1 (Permanent Structural Constitution)
- Mailbox ABI layout lock (size, alignment, offsets, field contract)
- Constitutional semver bump enforcement for ABI deltas
- Evidence gate: `scripts/ci/gate_structural_abi.sh`

### Tier-2 (Phase-Scoped Structural Constraints)
- Runtime marker identity and format lock
- Source-anchor enforcement for canonical markers
- Optional enforcement toggle: `RUNTIME_MARKER_CONTRACT_ENFORCE=0`
- Evidence gate: `scripts/ci/gate_runtime_marker_contract.sh`

### Tier-3 (Behavioral Proof Layer / Gate-6)
- Phase-driven behavioral proof scoring from runtime events
- No permanent constitutional lock on scheduler behavior
- Evidence gate: `scripts/ci/gate_behavioral_suite.sh`

## Implemented Files

### Gate-5
- `scripts/ci/gate_structural_abi.sh`
- `scripts/ci/gate_runtime_marker_contract.sh`
- `tools/dump_abi_layout.c`
- `constitution/abi_mailbox.json`
- `constitution/runtime_markers.json`
- `constitution/version.json`

### Gate-6
- `scripts/ci/gate_behavioral_suite.sh`
- `tools/ci/extract_markers.py`
- `tools/ci/score_behavioral_proofs.py`
- `constitution/behavioral_contracts/suite.json`
- `constitution/behavioral_contracts/semver_policy.json`
- `constitution/behavioral_contracts/README.md`
- `constitution/behavioral_contracts/envelopes/*.json`
- `constitution/behavioral_contracts/DRIFT_DETECTOR.md`
- `constitution/behavioral_contracts/drift_schema.json`
- `constitution/behavioral_contracts/drift_profiles/*.json`
- `constitution/behavioral_contracts/REPORT_CONTRACT.md`
- `constitution/ARCHITECTURE_GOVERNANCE.md`
- `constitution/abdf_context.md`
- `constitution/drift_history_policy.md`
- `constitution/drift_blocking_activation.md`

## Makefile Integration

- Added `ci-gate-structural-abi`
- Added `ci-gate-runtime-marker-contract`
- Added composite alias `ci-gate-structural-constitution`
- Added `ci-gate-behavioral-suite` (phase via `BEHAVIORAL_SUITE_PHASE`)
- Added evidence dir creation for `gates/behavioral-suite`
- `ci-freeze` and `ci-freeze-local` now include behavioral suite gate

## Current Behavioral Suite Policy

- Default phase: `5`
- Phase 5 proofs:
  - `sched_bridge_smoke` (required)
  - `ring3_presence_smoke` (advisory/WARN mode)
- Phase 6/7 proof roadmap defined in `constitution/behavioral_contracts/suite.json`
- Phase 8/9 proof scaffolds are defined and kept aligned with Phase 7 invariant set
- Phase 7/8 direction: profile-scoped behavioral regression envelope
  (`validation`, `experimental`, `perf`)

## Validation Snapshot

- `make ci-gate-structural-abi` -> PASS
- `make ci-gate-runtime-marker-contract` -> PASS
- `make RUNTIME_MARKER_CONTRACT_ENFORCE=0 ci-gate-runtime-marker-contract` -> SKIP
- `make ci-gate-behavioral-suite` -> PASS/WARN (phase-5; ring3 proof mode is advisory)
- `make -n ci-freeze` -> passes dry-run flow, no log path error

## Next Track

- Keep Tier-2 minimal in Phase 5
- Expand behavioral checks through Gate-6 in Phase 6+
- Phase-6 transition milestone:
  - `ring3_presence_smoke` must run in fail mode
  - phase `strict_mode` returns to `true`
  - promotion requires stable default-harness Ring3 marker evidence
- Phase-7/8 envelope milestone:
  - keep proof scorer core
  - enable profile-scoped envelope in warn-first mode (Phase 7)
  - promote selected envelope thresholds to hard-fail for stable profiles (Phase 8)
  - keep `warn_escalates_in_strict=false` for envelope telemetry rollout
  - add latency/tick-based envelope after marker timestamp expansion (Phase 9)
- Drift milestone:
  - telemetry-only drift report block enabled in Gate-6 output
  - deterministic context_key hashing enabled for drift context
  - canonical context normalization documented and active
  - rolling history telemetry stored under `evidence/history/<profile>/<context_key>.jsonl`
  - phase-7/8 non-blocking guard is explicit in drift policy scaffold
  - phase-9 blocking scaffold exists but remains disabled (`drift_blocking_policy.enabled=false`)
  - phase-9 activation governance protocol documented (`constitution/drift_blocking_activation.md`)
  - non-blocking until Phase-9 persistent drift policy
  - baseline auto-update forbidden (PR + evidence diff only)
- Revisit strategic decision: structural constitution permanence scope after Gate-6 stabilization

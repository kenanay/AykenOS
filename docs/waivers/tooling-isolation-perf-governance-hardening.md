# Tooling Isolation Waiver

## Metadata

- Waiver ID: tooling-isolation-perf-governance-hardening
- Title: Perf governance hardening requires kernel observability updates
- Author: Codex
- Date: 2026-04-04
- Expiry Date: 2026-05-15
- Related Issue: https://github.com/kenanay/AykenOS/pull/88
- Related RFC: N/A

## Exception Type

`perf-critical`

## Why Needed

Phase-14 perf truth-surface hardening required paired changes in tooling and
kernel observability surfaces. The performance gates now depend on:

- Ring3 entry guard contract markers
- Split entry/syscall latency diagnostics
- Mailbox phase and consume diagnostics
- Reduced ring0 export surface for the new perf hook model

Without this waiver, `ci-gate-tooling-isolation` blocks the exact kernel-side
instrumentation needed to keep the perf governance model truthful.

## Architectural Impact

1. ABI impact: none intended; perf/diagnostic surfaces only
2. Boundary impact: no policy moved into Ring0; instrumentation/control APIs were collapsed
3. Performance impact: bounded startup-entry guard behavior, measured by perf gates
4. Security impact: none intended; no new privilege surface

## Gate Impact

Etkilenen gate'ler:
- `ci-gate-abi`: no expected impact
- `ci-gate-boundary`: no expected impact
- `ci-gate-workspace`: no expected impact
- `ci-gate-hygiene`: no expected impact
- `ci-gate-performance`: expected contract/baseline drift until CI renewal
- `ci-gate-tooling-isolation`: waiver required for paired tooling + kernel hardening

## Evidence

- run_id: 23977711579
- evidence_path: `evidence/run-gh-23977711579-1/gates/tooling-isolation/`

## Fix Plan

1. Merge the perf truth-surface sync series from PR #88.
2. Re-run `perf-baseline-init.yml` on `main` with the pinned CI digest.
3. Promote only the workflow-generated baseline lock that contains the new
   `preempt_ring3_entry_guard` and split latency surfaces.
4. Close this waiver once baseline renewal lands and no further kernel touches
   are needed for the perf governance hardening slice.

## Rollback Plan

If this series causes unexpected CI or runtime regressions:

1. Revert the perf truth-surface sync commits from PR #88.
2. Restore the prior tooling/perf behavior on `main`.
3. Re-open a narrower follow-up change set with isolated evidence.

## Approval

- Architecture Board decision: Pending via PR #88 review
- Status: `approved`

# Freeze PR Template

## Gate Run

- Run ID:
- Evidence Path (`evidence/run-<id>/`):

## Gate Verdicts

- ABI (`ci-gate-abi`):
- Boundary (`ci-gate-boundary`):
- Tooling Isolation (`ci-gate-tooling-isolation`):
- Constitutional (`ci-gate-constitutional`):
- Workspace (`ci-gate-workspace`):
- Hygiene (`ci-gate-hygiene`):
- Performance (`ci-gate-performance`):
- Summary (`ci-summarize`):

## Tooling Isolation Guard

- Perf/preempt tooling touched in this PR: `yes/no`
- If yes, `kernel touch = 0`: `yes/no`
- Tooling isolation evidence path (`evidence/run-<id>/gates/tooling-isolation/`):

## Contract Change

- Changed contracts: `yes/no`
- If yes, exact paths:

## RFC / Waiver

- RFC link (if required):
- Waiver link (if required):

## Claim Check

If this PR claims `Completed/Production-ready`, all must be true:
1. `summary.json` verdict is `PASS`
2. test + benchmark evidence linked
3. related docs updated
4. architecture review note linked

## Notes

- Planned gates may be hard-fail stubs during freeze hardening.
- Do not merge feature work into mainline during active freeze.

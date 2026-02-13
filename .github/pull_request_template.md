# Freeze PR Template

## Gate Run

- Run ID:
- Evidence Path (`evidence/run-<id>/`):

## Gate Verdicts

- ABI (`ci-gate-abi`):
- Boundary (`ci-gate-boundary`):
- Workspace (`ci-gate-workspace`):
- Hygiene (`ci-gate-hygiene`):
- Performance (`ci-gate-performance`):
- Summary (`ci-summarize`):

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

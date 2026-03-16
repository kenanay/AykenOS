# Freeze PR Template

## Gate Run

- Run ID:
- Evidence Path (`evidence/run-<id>/`):

## Gate Verdicts

- ABI (`ci-gate-abi`):
- Boundary (`ci-gate-boundary`):
- Ring0 Exports (`ci-gate-ring0-exports`):
- Hygiene (`ci-gate-hygiene`):
- Tooling Isolation (`ci-gate-tooling-isolation`):
- Constitutional (`ci-gate-constitutional`):
- Governance Policy (`ci-gate-governance-policy`):
- Drift Activation (`ci-gate-drift-activation`):
- Structural ABI (`ci-gate-structural-abi`):
- Runtime Marker Contract (`ci-gate-runtime-marker-contract`):
- User Bin Lock (`ci-gate-user-bin-lock`):
- Embedded ELF Hash (`ci-gate-embedded-elf-hash`):
- Performance (`ci-gate-performance`):
- Ring3 Execution Phase10a2 (`ci-gate-ring3-execution-phase10a2`):
- Syscall Semantics Phase10b (`ci-gate-syscall-semantics-phase10b`):
- Scheduler Mailbox Phase10c (`ci-gate-scheduler-mailbox-phase10c`, conditional `PHASE10C_ENFORCE=1`):
- Mailbox Capability Negative (`ci-gate-mailbox-capability-negative`):
- Workspace (`ci-gate-workspace`):
- Syscall v2 Runtime (`ci-gate-syscall-v2-runtime`):
- Sched Bridge Runtime (`ci-gate-sched-bridge-runtime`):
- Behavioral Suite (`ci-gate-behavioral-suite`):
- Policy Accept (`ci-gate-policy-accept`):
- Kill-Switch Phase13 (`ci-kill-switch-phase13`):
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

- Conditional gate `ci-gate-scheduler-mailbox-phase10c` only runs when `PHASE10C_ENFORCE=1`.
- Do not merge feature work into mainline during active freeze.

# CI Run 24634827102 - Corrected Analysis

**Date**: 2026-04-19  
**Run**: 24634827102  
**Commit**: 49cd8c51  
**Corrected Verdict**: performance failed, marker absence in shell log is non-authoritative

## Observed Failure

The freeze run failed performance:

| Metric | Baseline | Actual | Max Allowed | Result |
| --- | ---: | ---: | ---: | --- |
| boot_time_ms | 10684 | 12707 | 11752.4 | FAIL |
| syscall_latency_ms_proxy | 175.081967 | 207.606557 | 183.83606535 | FAIL |

This confirms the regression is still present.

## What The Shell Log Does Not Prove

`gh run view 24634827102 --log | grep HARDENED_ENTRY` returning no rows does not prove the syscall path is dead. The freeze workflow uploads detailed evidence on failure; it does not stream every byte of performance QEMU debugcon into the Actions shell log.

Authoritative files are expected under the freeze artifact:

```text
evidence/run-*/gates/performance/boot-audit/qemu_debugcon.log
evidence/run-*/gates/performance/preempt.analysis.log
evidence/run-*/gates/performance/report.json
```

## Corrected Root Cause Candidate

The codebase had production syscall hot-path diagnostics:

- `HARDENED_ENTRY`
- `PATCH_C_CACHE_*`
- `DIAG_HOT_*`
- `PATCH_C2_FAST_PATH` / `PATCH_C2_SLOW_PATH`

Those writes target debugcon port `0xe9`; in a syscall-heavy performance path they can dominate measurement. The safer fix is to make syscall diagnostics opt-in while preserving boot markers required by boot audit.

## Applied Remediation

- Added `AYKEN_SYSCALL_DIAGNOSTIC_MARKERS_ENABLE ?= 0`.
- Guarded syscall diagnostic writes behind the new flag.
- Updated diagnostic/proof harnesses to opt in explicitly.
- Added a role-transition helper so cached context type cannot become stale when `execution_role` changes.
- Hardened fast validation masks and normalized syscall range checks.

## Follow-Up

The next CI freeze run should be evaluated by downloading the freeze artifact and reading `report.json`, `preempt.analysis.log`, and `qemu_debugcon.log`.

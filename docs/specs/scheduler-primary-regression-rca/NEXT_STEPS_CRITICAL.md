# Next Steps - CI Freeze RCA Correction

**Date**: 2026-04-19  
**Status**: Patch D local verification complete; authoritative CI freeze pending  
**Supersedes**: Prior "missing marker = dead path" interpretation

## Corrected Diagnosis

The previous conclusion treated missing marker strings from `gh run view --log` as proof that the syscall handler was not executing. That was too strong.

CI freeze does not stream the full performance `qemu_debugcon.log` into the shell log. On failure it dumps reports and selected snippets. The authoritative debugcon evidence lives in the freeze artifact under:

```text
evidence/run-*/gates/performance/boot-audit/qemu_debugcon.log
```

Therefore:

- Missing markers in `gh run view --log` are not execution-path proof.
- The performance regression is real: syscall/context remain about 207-208ms against a 183.836ms max.
- The safer architectural fix is to remove diagnostic I/O from production syscall hot paths and make the context-role cache coherent everywhere.

## Implemented Fix

1. Added `AYKEN_SYSCALL_DIAGNOSTIC_MARKERS_ENABLE ?= 0`.
2. Moved syscall diagnostic markers behind that explicit opt-in flag.
3. Kept boot diagnostic markers enabled because boot audit depends on boot markers.
4. Updated the second-syscall proof harness to opt into syscall diagnostics.
5. Added `proc_set_execution_role()` as an inline helper so `execution_role`, cached boundary context, and cache validity are updated together.
6. Updated all current role-transition sites to use the helper.
7. Hardened fast syscall validation to the frozen v2 syscall surface and added fail-closed normalized range checks.

## Current Verification

Local checks completed:

```text
make kernel.elf                                      PASS
make kernel.elf AYKEN_SYSCALL_DIAGNOSTIC_MARKERS_ENABLE=1 PASS
make ci-gate-constitutional                         PASS
bash scripts/ci/verify_diagnostic_flags.sh          PASS
nm kernel.elf | rg proc_set_execution_role          no exported symbol
```

Local `make ci-gate-performance` was also run on Darwin/arm64 and failed with `env_hash_mismatch`
against the GitHub-hosted Ubuntu baseline, so it is not an authoritative pass/fail signal. It did
confirm the production build used `AYKEN_SYSCALL_DIAGNOSTIC_MARKERS_ENABLE=0` and did not emit
`HARDENED_ENTRY`, `PATCH_C_CACHE`, `DIAG_HOT`, or `PATCH_C2_` syscall diagnostic markers in the
performance debugcon output.

The next authoritative check is GitHub `ci-freeze` on the updated branch.

## CI Evidence To Inspect

For the next failing or passing freeze run, inspect the artifact rather than only the shell log:

```bash
gh run download <run-id> --name freeze-evidence-<run-id>-<attempt> --dir /tmp/freeze
find /tmp/freeze -path '*/gates/performance/boot-audit/qemu_debugcon.log' -print
find /tmp/freeze -path '*/gates/performance/report.json' -print
```

Expected production behavior:

- No syscall diagnostic marker flood unless `AYKEN_SYSCALL_DIAGNOSTIC_MARKERS_ENABLE=1`.
- Boot markers remain present.
- `syscall_latency_ms_proxy <= 183.83606535`.
- `context_switch_latency_ms_proxy <= 183.83606535`.
- `boot_time_ms <= 11752.4`.

## If CI Still Fails

1. Compare `report.json` and `preempt.analysis.log` from the artifact.
2. Confirm diagnostic flag value in the build log is `AYKEN_SYSCALL_DIAGNOSTIC_MARKERS_ENABLE=0`.
3. If syscall/context are still ~207ms with diagnostics disabled, profile `preempt.analysis.log` and QEMU event timestamps from the artifact before changing syscall code again.
4. Do not infer execution path from `gh run view --log | grep`; use the artifact debugcon file.

# Execution Path Marker Correction

**Date**: 2026-04-19  
**Status**: superseded diagnostic interpretation

## Correction

This file previously concluded that `syscall_v2_hardened_handler()` was not called because marker strings were absent from `gh run view --log`.

That conclusion is no longer accepted. The freeze shell log is not the authoritative location for full QEMU debugcon output. The full performance evidence must be read from the freeze artifact.

## Current Position

The real confirmed facts are:

- CI freeze performance still fails around 207-208ms syscall/context latency.
- The raw artifact debugcon file is required for execution-path proof.
- Production syscall diagnostic writes should not remain in the hot path while performance is being measured.

## Replacement Fix

The code now separates syscall diagnostics from production boot diagnostics:

- `AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE` remains enabled for boot audit markers.
- `AYKEN_SYSCALL_DIAGNOSTIC_MARKERS_ENABLE` defaults disabled and must be explicitly enabled by proof harnesses.
- Role cache coherency is centralized through `proc_set_execution_role()`.

## Artifact Procedure

Use the freeze artifact for marker analysis:

```bash
gh run download <run-id> --name freeze-evidence-<run-id>-<attempt> --dir /tmp/freeze
rg "HARDENED_ENTRY|PATCH_C_CACHE|DIAG_HOT|PATCH_C2_" /tmp/freeze
```

Marker absence from `gh run view --log` alone is not sufficient evidence.

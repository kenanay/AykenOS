# Vendored Toolchain Snapshots
This document is subordinate to PHASE 0 - FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

**Last Updated:** 2026-03-10
**Status:** Informational development note

## Purpose

This note records the role of large vendored toolchain source trees kept in the repository.

Current tracked snapshots include:

- `binutils-2.42/`
- `gcc-14.2.0/`

## What They Are

These directories are full upstream source snapshots of GNU toolchain components.

- `binutils-2.42/` contains assembler, linker, object-inspection, and archive tooling sources such as `as`, `ld`, `objdump`, `readelf`, `ar`, and `nm`
- `gcc-14.2.0/` contains compiler sources

They are tracked as vendored source trees, not as generated build artifacts.

## Current Repository Role

As of 2026-03-10, these trees are treated as vendored toolchain snapshots for reference, offline availability, or local toolchain experiments.

They are **not** part of the normal AykenOS build path.

Observed current behavior:

- setup scripts and setup guides still prefer system cross-toolchains or fresh downloads/builds such as `binutils-2.40`
- hygiene gates explicitly exclude `binutils-2.42/` and `gcc-14.2.0/` from normal repo hygiene scans for performance reasons
- no current Make-based kernel or Phase-12 verifier flow consumes `binutils-2.42/` directly as a build input

## Operational Guidance

Treat these trees as vendored reference material unless a dedicated toolchain workflow explicitly says otherwise.

Practical rules:

- do not assume changes under these trees affect the default build
- do not casually edit or reformat files under these trees
- do not use their presence as evidence that the repo currently builds against those exact versions
- keep Finder metadata such as `.DS_Store` out of commits whenever possible

## Cleanup Guidance

If repo size or maintenance cost becomes a concern, cleanup should happen only in a dedicated change after confirming no private or offline workflow depends on these snapshots.

Safe cleanup options:

1. move vendored toolchain snapshots to a separate archival repository
2. replace them with documented download/build instructions only
3. keep them, but treat them as frozen vendor trees with explicit no-touch guidance

Do not mix vendored toolchain cleanup with unrelated Phase or closure work.

## References

- `scripts/ci/gate_hygiene.sh`
- `scripts/ci/gate_hygiene_simple.sh`
- `tools/setup/setup_macos_dev.sh`
- `tools/setup/setup_and_validate.sh`
- `tools/setup/install_dependencies.sh`
- `docs/setup/LINUX_SETUP_GUIDE.md`
- `docs/setup/MACOS_SETUP_GUIDE.md`
- `docs/setup/WINDOWS_WSL_SETUP_GUIDE.md`

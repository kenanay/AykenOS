# Phase-18 Platform Constitution Specs

This directory is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, and `PHASE18_TRANSITION_DECISION.md`. In case of
conflict, those documents prevail.

**Status:** PRE-ACTIVATION SPEC SET / PHASE-18 NOT ACTIVATED
**Authority basis:** `phase17-official-closure` at `416a5392`
**Attribution:** Documentation metadata only; not runtime, merge, or execution
authority.

## Purpose

This directory defines the Platform Constitution contracts that must exist
before the ayken platform can safely accept modules, packages, workspaces,
plugins, and later semantic systems above the frozen kernel execution
substrate.

These specs do not activate Phase-18 and do not change `CURRENT_PHASE=17`.

## Current Specs

1. `MODULE_MANIFEST_SCHEMA.md` - first RFC for the declarative module manifest.

## Planned Specs

1. Package Metadata Schema.
2. Capability Contract Specification.
3. Workspace Lifecycle Specification.
4. Trust Classification Model.
5. Plugin Boundary Contract.
6. Platform ABI Validation Gate.

## Non-Authority Rule

No file in this directory may grant:

1. New syscalls.
2. Kernel ABI expansion.
3. Ring0 policy authority.
4. Capability tokens.
5. Trust classification.
6. AI Runtime authority.
7. Semantic CLI execution verdict authority.

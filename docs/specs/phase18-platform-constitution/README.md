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
2. `CAPABILITY_CONTRACT_SPECIFICATION.md` - RFC for request, decision,
   receipt, and revocation boundaries.
3. `WORKSPACE_LIFECYCLE_SPECIFICATION.md` - RFC for workspace admission,
   logical mount, disable, quarantine, revocation, and removal boundaries.
4. `PACKAGE_METADATA_SCHEMA.md` - RFC for package identity, version,
   publisher, hash, signature, dependency, and Platform ABI compatibility
   metadata.
5. `TRUST_CLASSIFICATION_MODEL.md` - RFC for trust vocabulary, evidence
   inputs, classification records, lifecycle, and fail-closed policy effects.
6. `PLUGIN_BOUNDARY_CONTRACT.md` - RFC for host interfaces, extension points,
   plugin compatibility records, lifecycle, and fail-closed boundary effects.
7. `PLATFORM_ABI_VALIDATION_GATE.md` - RFC for deterministic validation order,
   input bundles, stage results, validation receipts, and fail-closed
   cross-contract separation.

## Non-Authority Rule

No file in this directory may grant:

1. New syscalls.
2. Kernel ABI expansion.
3. Ring0 policy authority.
4. Capability tokens.
5. Trust classification.
6. AI Runtime authority.
7. Semantic CLI execution verdict authority.
8. Package install, execution, workspace admission, or loader authority.
9. Plugin loading, autoload, or execution authority.

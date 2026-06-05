# Phase-18 Platform Constitution Specs

This directory is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, and `PHASE18_TRANSITION_DECISION.md`. In case of
conflict, those documents prevail.

**Status:** ACTIVE PLATFORM CONSTITUTION SPEC SET / RUNTIME NOT AUTHORIZED
**Authority basis:** `phase17-official-closure` at `416a5392`
**Attribution:** Documentation metadata only; not runtime, merge, or execution
authority.

## Purpose

This directory defines the active Phase-18 Platform Constitution contracts for
how the ayken platform can safely describe modules, packages, workspaces,
plugins, and later semantic systems above the frozen kernel execution
substrate.

These specs do not authorize runtime implementation, package installation,
workspace creation, plugin loading, capability issuance, trust assignment,
Semantic CLI authority, AI Runtime authority, new syscalls, or kernel ABI
expansion.

## Current Specs And Guards

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
8. `CROSS_CONSISTENCY_REVIEW.md` - accepted review record for cross-document
   terminology, dependency order, and authority separation.
9. `AUTHORITY_DRIFT_GUARD.md` - active review guard for preventing
   constitutional text from drifting into runtime authority.
10. `TERMINOLOGY_AUDIT.md` - accepted audit record for high-risk Phase-18
   vocabulary and required non-authority qualifiers.

## Current Review Result

`CROSS_CONSISTENCY_REVIEW.md` records the accepted cross-document review for
the current seven-RFC Platform Constitution set. This result is a documentation
review only. It does not authorize implementation.

`AUTHORITY_DRIFT_GUARD.md` defines the ongoing Phase-18 review guard for
future edits. It is not a runtime validator, CI gate, loader, issuer, or
implementation authority.

`TERMINOLOGY_AUDIT.md` records the accepted vocabulary audit for high-risk
terms such as `validated`, `trusted`, `approved`, `admitted`, `enabled`,
`compatible`, `binding`, `receipt`, `loader`, and `runtime`. The audit PASS is
not implementation authority.

## Activation Boundary

`../../../PHASE18_ACTIVATION_DECISION.md` is the accepted activation decision
package. `../../../docs/roadmap/CURRENT_PHASE` is now `18`. This activates
only Platform Constitution authority and does not authorize runtime
implementation.

The mandatory activation rule is:

```text
Constitution != Runtime
```

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

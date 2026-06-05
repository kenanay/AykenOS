# Phase-19 Platform Runtime RFC Set

This directory is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, `PHASE18_TRANSITION_DECISION.md`,
`PHASE18_ACTIVATION_DECISION.md`, the Phase-18 Platform Constitution
reference set, `AUTHORITY_DRIFT_GUARD.md`, `TERMINOLOGY_AUDIT.md`, and
`PHASE19_RUNTIME_DECISION.md`. In case of conflict, those documents prevail.

**Status:** PRE-IMPLEMENTATION RFC SET / PHASE-19 NOT ACTIVE / RUNTIME NOT AUTHORIZED
**Authority basis:** `PHASE19_RUNTIME_DECISION.md`
**Attribution:** Documentation metadata only; not runtime, merge, or execution
authority.

## Purpose

This directory defines the first Phase-19 Platform Runtime MVP RFC set.

The set narrows the future runtime implementation question before any runtime
source code exists. It defines lifecycle, static input bundle, validation
integration, workspace admission record, runtime receipt, evidence plan, and
non-goal boundaries.

It does not activate Phase-19. It does not update `CURRENT_PHASE`. It does not
authorize package installation, module loading, workspace creation, real
filesystem mounts, plugin loading, capability issuance, trust assignment,
Semantic CLI authority, AI Runtime authority, new syscalls, or kernel ABI
expansion.

## Current RFCs

1. `RUNTIME_LIFECYCLE_SPECIFICATION.md` - deterministic runtime MVP lifecycle
   states and transitions.
2. `RUNTIME_INPUT_BUNDLE_SPECIFICATION.md` - static test-owned input bundle
   boundary.
3. `PLATFORM_VALIDATION_INTEGRATION_SPECIFICATION.md` - integration with the
   Phase-18 Platform ABI Validation Gate.
4. `WORKSPACE_ADMISSION_RUNTIME_SPECIFICATION.md` - workspace admission record
   boundary without workspace creation or mount authority.
5. `RUNTIME_RECEIPT_SPECIFICATION.md` - deterministic runtime receipt schema
   and digest binding.
6. `RUNTIME_EVIDENCE_PLAN.md` - required local/remote evidence surfaces for a
   later implementation.
7. `RUNTIME_NON_GOALS_AND_DENIALS.md` - explicit denial list for installer,
   loader, issuer, trust, Semantic CLI, AI Runtime, registry, and agent drift.

## Core Rule

```text
Runtime RFC set != runtime implementation
```

The existence of these RFCs means only that the Phase-19 Runtime MVP boundary
is documented for later review. A separate exact-SHA pointer transition,
implementation RFC acceptance, runtime evidence implementation, and remote CI
authority remain required before any runtime code can be accepted.

## MVP Boundary

The only future MVP shape allowed by this set is:

```text
static input bundle
  -> Phase-18 Platform ABI validation integration
  -> workspace admission record
  -> deterministic runtime receipt
```

This flow must not install, load, mount, execute, issue, trust, publish, or
schedule anything.

## Non-Authority Rule

No file in this directory may grant:

1. `CURRENT_PHASE=19`.
2. Runtime source code authority.
3. Package installation or execution.
4. Module loading.
5. Plugin loading or instantiation.
6. Workspace creation or real mounts.
7. Capability token minting or issuance.
8. Trust assignment.
9. Registry publication.
10. Semantic CLI execution authority.
11. AI Runtime authority.
12. Agent authority.
13. New syscalls.
14. Kernel ABI expansion.
15. Ring0 policy.

Unknown authority readings fail closed.

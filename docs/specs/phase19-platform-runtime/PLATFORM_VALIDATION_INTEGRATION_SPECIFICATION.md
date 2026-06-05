# Phase-19 Platform Validation Integration Specification

This document is subordinate to `PHASE19_RUNTIME_DECISION.md`,
`RUNTIME_INPUT_BUNDLE_SPECIFICATION.md`, and
`../phase18-platform-constitution/PLATFORM_ABI_VALIDATION_GATE.md`. In case
of conflict, those documents prevail.

**Status:** PRE-IMPLEMENTATION RFC / PHASE-19 NOT ACTIVE / RUNTIME NOT AUTHORIZED
**Contract id:** `ayken.phase19.platform_validation.integration.v1`
**Authority boundary:** Documentation/specification only; not a validator
implementation, runtime implementation, installer, loader, issuer, trust
assignment, workspace creation, plugin loading, Semantic CLI authority, AI
Runtime authority, syscall, or kernel ABI expansion.

## Purpose

This RFC defines how a future Phase-19 Runtime MVP may consume the Phase-18
Platform ABI Validation Gate result.

It does not implement validation. It does not turn validation into runtime
authority.

## Core Rule

```text
Validation integration != authority grant
```

The runtime may later record that a validation decision exists and is bound to
the same input bundle. That record must not install, load, execute, mount,
issue, trust, or enable anything.

## Integration Inputs

The integration boundary may consume:

1. Runtime input bundle digest.
2. Platform ABI validation receipt digest.
3. Platform ABI validation stage summary.
4. Validation policy reference.
5. Denial reason if validation failed.

No Semantic CLI output, AI output, diagnostics dashboard, or ambient runtime
state may be consumed as authority.

## Required Checks

A later implementation must check:

1. Validation receipt contract id is known.
2. Validation receipt schema version is known.
3. Validation receipt subject matches the input bundle subject.
4. Validation receipt digest matches the referenced content.
5. Validation stage order matches Phase-18 Platform ABI Validation Gate order.
6. Validation PASS does not include authority-granting effects.
7. Validation FAIL stops the runtime lifecycle.
8. Validation receipt does not declare token, mount, loader, execution, trust,
   or capability authority.

Unknown fields, unknown stages, stale digests, missing policy references, or
authority-granting effects fail closed.

## Positive Integration Result

A positive integration result may record:

1. Input bundle digest.
2. Validation receipt digest.
3. Stage count.
4. Stage result digests.
5. Decision status `recordable`.

It must not record active handles, tokens, loaded modules, mounted paths,
plugin instances, trust levels, or runtime execution results.

## Negative Integration Result

A negative integration result must record:

1. Input bundle digest when available.
2. Failed or missing validation reference.
3. Denial reason.
4. Terminal lifecycle state.

After a negative result, workspace admission record and runtime receipt
emission must be blocked unless the receipt is an explicit denial receipt.

## Evidence Requirements

Future evidence must prove:

1. Valid validation receipt can be bound to the input bundle.
2. Mismatched subject fails closed.
3. Stale digest fails closed.
4. Unknown stage fails closed.
5. Validation FAIL prevents admission.
6. Validation PASS does not grant runtime authority.

## Acceptance Boundary

This RFC only defines integration rules. It does not create a validation gate,
runtime validator, loader, package manager, workspace engine, or authority
surface.

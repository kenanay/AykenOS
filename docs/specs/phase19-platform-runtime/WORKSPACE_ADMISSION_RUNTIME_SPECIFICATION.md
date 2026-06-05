# Phase-19 Workspace Admission Runtime Specification

This document is subordinate to `PHASE19_RUNTIME_DECISION.md`,
`PLATFORM_VALIDATION_INTEGRATION_SPECIFICATION.md`, and
`../phase18-platform-constitution/WORKSPACE_LIFECYCLE_SPECIFICATION.md`. In
case of conflict, those documents prevail.

**Status:** PRE-IMPLEMENTATION RFC / PHASE-19 NOT ACTIVE / RUNTIME NOT AUTHORIZED
**Contract id:** `ayken.phase19.workspace_admission.runtime.v1`
**Authority boundary:** Documentation/specification only; not workspace
creation, real mount authority, filesystem implementation, package
installation, module loading, plugin loading, capability issuance, trust
assignment, Semantic CLI authority, AI Runtime authority, syscall, or kernel
ABI expansion.

## Purpose

This RFC defines the workspace admission record boundary for the first
possible Phase-19 Runtime MVP.

The record may later show that a static input bundle passed enough validation
to be admitted as a record. It does not create a workspace and does not mount
anything.

## Core Rule

```text
Workspace admission record != workspace creation
```

Admission is an evidence record. It is not a real workspace, mount namespace,
filesystem handle, permission grant, or execution context.

## Admission Inputs

Admission may depend only on:

1. Runtime input bundle digest.
2. Platform validation integration result digest.
3. Phase-18 workspace declaration reference.
4. Requested logical workspace profile.
5. Denial policy reference.

No ambient filesystem state, user shell state, Semantic CLI output, AI output,
or diagnostics surface may become admission authority.

## Admission Record Fields

Version 1 admission records require:

| Field | Required | Meaning | Authority boundary |
|---|---|---|---|
| `schema_id` | yes | Must be `ayken.phase19.workspace_admission.runtime.v1` | Not runtime authority |
| `admission_id` | yes | Stable record id | Not workspace id grant |
| `subject` | yes | Bound subject from input bundle | Not execution target |
| `input_bundle_digest` | yes | Bound input bundle digest | Not request replay authority |
| `validation_result_digest` | yes | Bound validation result digest | Not capability |
| `workspace_profile` | yes | Declarative logical profile | Not mount profile |
| `admission_status` | yes | `admitted_record`, `denied`, or `blocked` | Not active workspace |
| `denial_reason` | conditional | Required for denied or blocked records | Not remediation authority |

Unknown fields fail closed.

## Allowed Status Values

| Status | Meaning | Boundary |
|---|---|---|
| `admitted_record` | Evidence record may be emitted | No workspace exists |
| `denied` | Input is rejected | Terminal denial |
| `blocked` | Prior validation or binding failed | Terminal denial |

Unknown statuses fail closed.

## Denial Conditions

Admission must be denied if:

1. Platform validation integration failed.
2. Workspace declaration is missing or stale.
3. Workspace declaration asks for real mount authority.
4. Workspace declaration asks for capability issuance.
5. Workspace declaration asks for trust assignment.
6. Workspace declaration asks for package install or execution.
7. Workspace declaration asks for plugin loading.
8. Subject binding does not match the input bundle.
9. Admission record would become a bearer token.
10. Kernel ABI expansion is required.

## Evidence Requirements

Future implementation evidence must prove:

1. Positive admission record emission for a valid static bundle.
2. Denial when validation is missing.
3. Denial when workspace declaration requests real mount.
4. Denial when subject digest changes.
5. Denial when authority-granting fields appear.

## Acceptance Boundary

This RFC does not create workspace runtime behavior. A later implementation
must still prove that admission records are inert evidence outputs.
